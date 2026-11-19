//! Native Fedora input for the authored rescue startup assignment.
use super::unit_manager::{Control, Step, draft};
use super::*;

const ID: &str = "lpic2-rescue-startup-repair";
const UNIT: &str = "linux-rescue-target-repair";

fn route() -> Vec<Step> {
    use Step::*;
    let mut steps = vec![
        Setup,
        Wait,
        Shot("rescue-startup-introduction"),
        Click(Control::Try),
        Wait,
        Click(Control::Startup(false)),
        Wait,
        Assert("practice-ordinary-blocked"),
        Shot("rescue-startup-practice-blocked"),
        Click(Control::Startup(true)),
        Wait,
        Assert("practice-rescue"),
        Shot("rescue-startup-practice-rescue"),
    ];
    draft(
        &mut steps,
        true,
        "/etc/maple/startup.conf",
        "MODE=workshop-ready\n",
        true,
    );
    steps.extend([
        Click(Control::Check),
        Wait,
        Assert("practice-passed"),
        Shot("rescue-startup-practice-passed"),
        Click(Control::Assignment),
        Wait,
        Assert("assignment-untouched"),
        Shot("rescue-startup-assignment"),
        Click(Control::Startup(false)),
        Wait,
        Assert("assignment-ordinary-blocked"),
        Click(Control::Startup(true)),
        Wait,
        Assert("assignment-rescue"),
    ]);
    draft(
        &mut steps,
        true,
        "/etc/ridge/startup.conf",
        "# Applied repair\n MODE = ready-east\n",
        true,
    );
    steps.extend([
        Click(Control::Check),
        Wait,
        Assert("assignment-passed"),
        Shot("rescue-startup-assignment-passed"),
    ]);
    draft(
        &mut steps,
        true,
        "/etc/ridge/startup.conf",
        "MODE=wrong-east\n",
        true,
    );
    steps.extend([
        Assert("assignment-invalidated"),
        Click(Control::Check),
        Wait,
        Assert("assignment-invalidated"),
        Shot("rescue-startup-assignment-wrong"),
    ]);
    draft(
        &mut steps,
        true,
        "/etc/ridge/startup.conf",
        "MODE=ready-east\n",
        true,
    );
    steps.extend([
        Click(Control::Check),
        Wait,
        Assert("assignment-passed"),
        Theme("dlms-light", 1.5),
        Wait,
        Click(Control::Check),
        Wait,
        Shot("rescue-startup-light150"),
        Theme("dlms-dark", 2.),
        Wait,
        Click(Control::Check),
        Wait,
        Shot("rescue-startup-dark200"),
        Reload,
        Assert("assignment-passed"),
        Finish,
    ]);
    steps
}

#[derive(Clone, Default)]
struct Flow {
    action: usize,
    polls: usize,
    next: f64,
    started: Option<Instant>,
    capture_sent: bool,
}

fn assert_current(app: &LabApp, expected: &str) {
    use dlms_learning_lab::unit_manager::Runtime;
    let scenario = &app.scenarios[app.selected.unwrap()];
    assert_eq!(scenario.id, ID);
    let attempt = &app.progress.attempts[ID];
    let engine::State::UnitManager { simulation } = &attempt.state else {
        panic!()
    };
    let workspace = attempt.workspace.as_ref().unwrap();
    let practice = workspace.unit_manager_learning.as_ref().unwrap();
    let is_practice = expected.starts_with("practice-");
    let state = if is_practice {
        &practice.state
    } else {
        simulation
    };
    assert_eq!(state.default_target(), "multi-user.target");
    let active = |unit: &str| state.runtime().get(unit).is_some_and(Runtime::active);
    match expected {
        "practice-ordinary-blocked" | "assignment-ordinary-blocked" => {
            let prefix = if is_practice { "maple" } else { "ridge" };
            assert!(matches!(
                state.runtime().get(&format!("{prefix}-verify.service")),
                Some(Runtime::Failed { .. })
            ));
            assert!(!active("multi-user.target"));
        }
        "practice-rescue" | "assignment-rescue" => {
            assert!(active("rescue.target"));
            assert!(!active("multi-user.target"));
        }
        "practice-passed" => {
            assert!(practice.checked);
            assert!(
                practice
                    .checks(scenario.unit_manager_teaching.as_ref().unwrap())
                    .iter()
                    .all(|row| row.0)
            );
            assert!(!progress::current_solved(attempt, scenario));
        }
        "assignment-untouched" => {
            assert_eq!(attempt.state, scenario.initial);
            assert!(!workspace.checked);
        }
        "assignment-passed" => {
            assert!(workspace.checked && progress::current_solved(attempt, scenario));
            assert!(active("rescue.target"));
        }
        "assignment-invalidated" => assert!(!progress::current_solved(attempt, scenario)),
        _ => panic!("Unknown rescue startup observation"),
    }
}

impl LabApp {
    pub(in crate::ui) fn rescue_startup_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("rescue-startup-native");
        let mut flow = ctx
            .data_mut(|data| data.get_temp::<Flow>(key))
            .unwrap_or_default();
        let plan = route();
        let now = ctx.input(|input| input.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "rescue action {} could not settle: {due:?}",
            flow.action
        );
        if !due.unwrap() {
            ctx.data_mut(|data| data.insert_temp(key, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 130000 + flow.action;
        let done = match &plan[flow.action] {
            Step::Setup => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                assert!(
                    self.units
                        .iter()
                        .any(|unit| unit.id == UNIT && unit.labs == [ID])
                );
                self.scale = 1.;
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let index = self
                    .scenarios
                    .iter()
                    .position(|scenario| scenario.id == ID)
                    .unwrap();
                self.launch(index);
                true
            }
            Step::Assert(expected) => {
                assert_current(self, expected);
                true
            }
            Step::Shot(name) => {
                if !flow.capture_sent {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        (*name).to_owned(),
                    )));
                    flow.capture_sent = true;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Step::Reload => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                self.progress = loaded;
                true
            }
            Step::Finish => {
                println!("RESCUE_STARTUP_NATIVE_COMPLETE: {} actions", plan.len());
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            input => {
                if matches!(input, Step::Theme(..)) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                }
                self.manager_native_input(ctx, input)
                    .expect("mechanical manager action")
                    .0
            }
        };
        if done {
            println!("RESCUE_STARTUP_NATIVE_STEP {}", flow.action);
            flow.action += 1;
            flow.capture_sent = false;
            flow.polls = 0;
            flow.started = None;
        }
        flow.next = now + 0.025;
        ctx.data_mut(|data| data.insert_temp(key, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
