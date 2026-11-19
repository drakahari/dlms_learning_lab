//! Fixed startup-policy checks through real widgets and simulated commands only.
use super::unit_manager::{Control, Step, command, draft};
use super::*;
use dlms_learning_lab::{
    unit_manager::Action,
    unit_manager_workspace::{self as ws, Draft, EditAction},
};
#[derive(Clone)]
enum Op {
    Open(String),
    Ui(Step),
    Example(bool),
    Solved(bool),
    Save,
    Fresh,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    done: bool,
}
fn ui(out: &mut Vec<Op>, steps: Vec<Step>) {
    out.extend(steps.into_iter().map(Op::Ui));
}
fn cli(out: &mut Vec<Op>, text: &str) {
    let mut p = vec![];
    command(&mut p, text, true);
    ui(out, p);
}
fn action(out: &mut Vec<Op>, previous: &mut Draft, a: &EditAction) {
    match a {
        EditAction::Startup { .. } => {
            panic!("The retained LPIC-1 fixtures do not use the LPIC-2 startup controls")
        }
        EditAction::Manager { action } => {
            let text = match action {
                Action::Start { unit } => format!("systemctl start {unit}"),
                Action::Stop { unit } => format!("systemctl stop {unit}"),
                Action::Enable { unit } => format!("systemctl enable {unit}"),
                Action::Disable { unit } => format!("systemctl disable {unit}"),
                Action::SetDefault { target } => format!("systemctl set-default {target}"),
                Action::DaemonReload {} => "systemctl daemon-reload".into(),
                Action::AdvancePrograms {} => return,
                _ => panic!("authored route must edit saved inputs through editor"),
            };
            cli(out, &text);
        }
        EditAction::Save { draft: target } => {
            for (path, text) in &target.documents {
                if previous.documents[path] != *text {
                    let mut p = vec![];
                    draft(&mut p, false, path, text, true);
                    ui(out, p);
                }
            }
            for (path, text) in &target.configuration {
                if previous.configuration[path] != *text {
                    let mut p = vec![];
                    draft(&mut p, true, path, text, true);
                    ui(out, p);
                }
            }
            *previous = target.clone();
        }
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut p = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| s.id.starts_with("lpic1-startup-")) {
        count += 1;
        let engine::Goal::UnitManager { definition } = &s.goal else {
            unreachable!()
        };
        let l = s.unit_manager_teaching.as_ref().unwrap();
        p.extend([
            Op::Open(s.id.clone()),
            Op::Ui(Step::Click(Control::Try)),
            Op::Ui(Step::Wait),
        ]);
        cli(&mut p, "systemctl get-default");
        let mut d = Draft::new(&l.definition.initial).unwrap();
        for a in &l.solution {
            action(&mut p, &mut d, a);
        }
        p.extend([
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Example(true),
            Op::Shot(format!("startup-{}-practice-passed", s.id)),
        ]);
        for a in &l.try_actions {
            action(&mut p, &mut d, a);
        }
        p.extend([
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Example(false),
        ]);
        for a in &l.repair {
            action(&mut p, &mut d, a);
        }
        p.extend([
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Example(true),
            Op::Ui(Step::Click(Control::Assignment)),
            Op::Ui(Step::Wait),
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Solved(false),
            Op::Shot(format!("startup-{}-starter-feedback", s.id)),
        ]);
        let mut d = Draft::new(&definition.initial).unwrap();
        for line in &s.solution {
            let a: EditAction =
                serde_json::from_str(line.strip_prefix("unit-manager ").unwrap()).unwrap();
            action(&mut p, &mut d, &a);
        }
        p.extend([
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("startup-{}-passed", s.id)),
        ]);
        let unit = definition
            .initial
            .inventory()
            .units
            .iter()
            .find(|n| n.ends_with("-app.service"))
            .unwrap();
        cli(&mut p, &format!("systemctl disable {unit}"));
        p.extend([
            Op::Solved(false),
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Solved(false),
            Op::Shot(format!("startup-{}-damage-feedback", s.id)),
        ]);
        cli(&mut p, &format!("systemctl enable {unit}"));
        p.extend([
            Op::Ui(Step::Click(Control::Check)),
            Op::Ui(Step::Wait),
            Op::Solved(true),
            Op::Ui(Step::Click(Control::Reset)),
            Op::Ui(Step::Click(Control::Confirm)),
            Op::Ui(Step::Wait),
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 3);
    p
}
impl LabApp {
    pub(in crate::ui) fn startup_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("startup-policy-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "STARTUP_POLICY_NATIVE_COMPLETE: all3 examples/experiments/retry, assignments/wrong/correct/disable-invalidation/save-read/reset; no host service or boot"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 120000 + r.at;
        let op = r.ops[r.at].clone();
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Startup native step {} phase{}",
            r.at,
            r.phase
        );
        if r.frames == 1 {
            println!("STARTUP_POLICY_NATIVE_STEP {}", r.at);
        }
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Ui(step) => {
                self.manager_native_input(ctx, &step)
                    .expect("fixed native input only")
                    .0
            }
            Op::Solved(expected) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    expected,
                    "{} notice={:?}",
                    s.id,
                    self.notice
                );
                true
            }
            Op::Example(expected) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .unit_manager_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    ws::solved(
                        &s.unit_manager_teaching.as_ref().unwrap().definition,
                        &l.state,
                        &l.draft
                    ),
                    expected
                );
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    if self.reveal_smoke_control(ctx, self.metrics.check.unwrap()) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if complete {
            r.at += 1;
            r.frames = 0;
            r.phase = 0;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_native_capture_phase_has_no_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-startup-current-future")
            .unwrap();
        a.launch(i);
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("startup-policy-native-route"),
                Route {
                    ops: vec![Op::Shot("fixed-native-test".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| a.startup_lessons_tick(ctx));
        assert!(a.capture_pending);
    }
}
