//! Actual native input plan for four normal authored manager activities.
//! Never appends scenarios or publishes a reference State. Not executed yet.
use super::unit_manager::{Control, Step, command, draft};
use super::*;
use dlms_learning_lab::{
    unit_manager::Action,
    unit_manager_workspace::{Draft, EditAction},
};
use serde::Deserialize;
const IDS: [&str; 4] = [
    "lpic2-unit-loaded-recovery",
    "lpic2-unit-preparation-order",
    "lpic2-unit-optional-wants",
    "lpic2-unit-target-handover",
];
const UNIT: &str = "linux-unit-manager-foundations";
const WIDTH: usize = 500;
#[derive(Deserialize)]
struct Review {
    plans: Vec<ReviewPlan>,
}
#[derive(Deserialize)]
struct ReviewPlan {
    id: String,
    reference: Vec<EditAction>,
    alternates: Vec<Vec<EditAction>>,
}
#[derive(Clone)]
enum LessonStep {
    Setup,
    Open,
    Ui(Step),
    Check(&'static str),
    Capture(&'static str),
    Remember(&'static str),
    Reload,
    Finish,
}
fn add(p: &mut Vec<LessonStep>, steps: impl IntoIterator<Item = Step>) {
    p.extend(steps.into_iter().map(LessonStep::Ui));
}
fn run_command(p: &mut Vec<LessonStep>, text: &str, keyboard: bool) {
    let mut steps = vec![];
    command(&mut steps, text, keyboard);
    add(p, steps);
}
fn edit(p: &mut Vec<LessonStep>, config: bool, key: &str, text: &str, save: bool) {
    let mut steps = vec![];
    draft(&mut steps, config, key, text, save);
    add(p, steps);
}
fn cli(action: &EditAction) -> Option<String> {
    let EditAction::Manager { action } = action else {
        return None;
    };
    Some(match action {
        Action::DaemonReload {} => "systemctl daemon-reload".into(),
        Action::Start { unit } => format!("systemctl start {unit}"),
        Action::Stop { unit } => format!("systemctl stop {unit}"),
        Action::Enable { unit } => format!("systemctl enable {unit}"),
        Action::Disable { unit } => format!("systemctl disable {unit}"),
        Action::SetDefault { target } => format!("systemctl set-default {target}"),
        Action::AdvancePrograms {} => return None,
        _ => panic!("native authored reference must use supported editor/actions"),
    })
}
fn saved(actions: &[EditAction]) -> &Draft {
    actions
        .iter()
        .find_map(|a| {
            if let EditAction::Save { draft } = a {
                Some(draft)
            } else {
                None
            }
        })
        .expect("bounded authored saved target")
}
fn from_initial(s: &Scenario) -> Draft {
    let engine::State::UnitManager { simulation } = &s.initial else {
        panic!()
    };
    Draft::new(simulation).unwrap()
}
fn edit_changed(p: &mut Vec<LessonStep>, from: &Draft, to: &Draft, configuration: bool) {
    assert!(from.valid() && to.valid());
    let a = if configuration {
        &from.configuration
    } else {
        &from.documents
    };
    let b = if configuration {
        &to.configuration
    } else {
        &to.documents
    };
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let changed = b
        .iter()
        .filter(|(k, v)| a.get(*k) != Some(*v))
        .collect::<Vec<_>>();
    assert!(changed.len() <= 8);
    for (k, v) in changed {
        edit(p, configuration, k, v, true);
    }
}
fn action_commands(p: &mut Vec<LessonStep>, actions: &[EditAction], keyboard: bool) {
    assert!(actions.len() <= 32);
    for a in actions {
        if let Some(command) = cli(a) {
            run_command(p, &command, keyboard);
        }
    }
}
fn stop_changed_work(p: &mut Vec<LessonStep>, initial: &Draft, app: &str) {
    // A previous failed attempt can leave successful RemainAfterExit preparation
    // active with old input. A second start intentionally does not rerun it.
    run_command(p, &format!("systemctl stop {app}"), true);
    if let Some(unit) = initial
        .documents
        .keys()
        .find(|u| u.ends_with("-prepare.service"))
    {
        run_command(p, &format!("systemctl stop {unit}"), true);
    }
}
fn check(p: &mut Vec<LessonStep>, kind: &'static str, shot: &'static str) {
    add(p, [Step::Click(Control::Check), Step::Wait]);
    p.push(LessonStep::Check(kind));
    p.push(LessonStep::Capture(shot));
}
fn lesson_plan(s: &Scenario, review: &ReviewPlan) -> Vec<LessonStep> {
    assert_eq!(s.id, review.id);
    assert!(review.alternates.len() == 2);
    let initial = from_initial(s);
    let target = saved(&review.reference);
    let alternative = saved(&review.alternates[0]);
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    let app = initial
        .documents
        .keys()
        .find(|u| u.ends_with("-app.service"))
        .unwrap();
    let example_app = lesson
        .definition
        .initial
        .inventory()
        .units
        .iter()
        .find(|u| u.ends_with("-app.service"))
        .unwrap();
    let app_config = initial
        .configuration
        .keys()
        .find(|path| path.ends_with("/app.conf"))
        .unwrap();
    let example_config = lesson
        .definition
        .initial
        .inventory()
        .config_paths
        .iter()
        .find(|path| path.ends_with("/app.conf"))
        .unwrap();
    let mut p = vec![
        LessonStep::Setup,
        LessonStep::Ui(Step::Wait),
        LessonStep::Capture("library"),
        LessonStep::Open,
        LessonStep::Ui(Step::Wait),
        LessonStep::Capture("introduction"),
    ];
    add(&mut p, [Step::Click(Control::Try), Step::Wait]);
    p.push(LessonStep::Capture("example-initial"));
    add(&mut p, [Step::Click(Control::Fill), Step::Wait]);
    p.push(LessonStep::Check("fill-only"));
    add(&mut p, [Step::Click(Control::Save), Step::Wait]);
    // Exercise visible teaching selection/Insert before executing anything.
    let selected = lesson
        .example_actions
        .iter()
        .position(|a| {
            matches!(
                &a.action,
                EditAction::Manager {
                    action: Action::DaemonReload {}
                }
            )
        })
        .unwrap();
    p.push(LessonStep::Remember("before-insert"));
    add(
        &mut p,
        [
            Step::Click(Control::ExampleActionPicker),
            Step::Click(Control::ExampleActionChoice(selected)),
            Step::Click(Control::InsertExample),
            Step::Wait,
        ],
    );
    p.push(LessonStep::Check("insert-only"));
    add(&mut p, [Step::Click(Control::Run), Step::Wait]);
    // Repeating a read of the same saved definitions is harmless; remaining
    // reference commands still execute through the real terminal controls.
    action_commands(&mut p, &lesson.solution, true);
    check(&mut p, "example-pass", "example-complete");
    action_commands(&mut p, &lesson.try_actions, false);
    check(&mut p, "example-fails", "example-experiment");
    action_commands(&mut p, &lesson.repair, true);
    check(&mut p, "example-pass", "example-repaired");
    add(&mut p, [Step::Click(Control::Assignment), Step::Wait]);
    p.push(LessonStep::Check("assignment-initial"));
    p.push(LessonStep::Capture("assignment"));
    run_command(&mut p, &format!("systemctl start {app}"), false);
    run_command(&mut p, &format!("journalctl -u {app} -o cat"), true);
    check(&mut p, "assignment-fails", "wrong-feedback");
    // Enter only changed unit files first. Preserved configs remain untouched;
    // a real partial state is checked before completing the full authored target.
    edit_changed(&mut p, &initial, target, false);
    run_command(&mut p, "systemctl daemon-reload", true);
    check(&mut p, "partial", "partial");
    edit_changed(&mut p, &initial, target, true);
    stop_changed_work(&mut p, &initial, app);
    action_commands(&mut p, &review.reference, false);
    check(&mut p, "assignment-pass", "complete");
    p.push(LessonStep::Remember("query-state"));
    run_command(&mut p, &format!("systemctl status {app}"), true);
    run_command(&mut p, &format!("systemctl is-enabled {app}"), false);
    p.push(LessonStep::Check("query-preserves"));
    p.push(LessonStep::Capture("queried-success"));
    edit_changed(&mut p, target, alternative, false);
    edit_changed(&mut p, target, alternative, true);
    stop_changed_work(&mut p, &initial, app);
    action_commands(&mut p, &review.alternates[0], true);
    check(&mut p, "assignment-pass", "alternate");
    // Each real lesson edits and retries at BOTH larger-text sizes, not merely
    // Check/layout captures. Values always pass through native input events.
    for (theme, scale, wrong_shot, repaired_shot) in [
        ("dlms-light", 1.5, "light-150-wrong", "light-150-repaired"),
        ("dlms-dark", 2., "dark-200-wrong", "dark-200-repaired"),
    ] {
        add(&mut p, [Step::Theme(theme, scale), Step::Wait]);
        edit(&mut p, true, app_config, "PORT=1\n", true);
        run_command(&mut p, &format!("systemctl stop {app}"), true);
        run_command(&mut p, &format!("systemctl start {app}"), false);
        check(&mut p, "assignment-fails", wrong_shot);
        edit(
            &mut p,
            true,
            app_config,
            &alternative.configuration[app_config],
            true,
        );
        run_command(&mut p, &format!("systemctl stop {app}"), false);
        run_command(&mut p, &format!("systemctl start {app}"), true);
        check(&mut p, "assignment-pass", repaired_shot);
    }
    edit(
        &mut p,
        true,
        app_config,
        "PORT=unfinished-assignment\n",
        false,
    );
    add(
        &mut p,
        [
            Step::Click(Control::Command),
            Step::Type("systemctl 'unfinished assignment".into()),
            Step::Wait,
            Step::Click(Control::Example),
            Step::Wait,
        ],
    );
    edit(
        &mut p,
        true,
        example_config,
        "PORT=unfinished-example\n",
        false,
    );
    add(&mut p, [Step::Undo, Step::Wait]);
    p.push(LessonStep::Check("undo-isolated"));
    add(
        &mut p,
        [
            Step::Type("PORT=unfinished-example\n".into()),
            Step::Wait,
            Step::Click(Control::Command),
            Step::Type("systemctl 'unfinished separate example".into()),
            Step::Wait,
            Step::Click(Control::Assignment),
            Step::Wait,
        ],
    );
    p.push(LessonStep::Remember("resume"));
    p.push(LessonStep::Reload);
    add(&mut p, [Step::Wait]);
    p.push(LessonStep::Check("resume-separated"));
    p.push(LessonStep::Capture("resumed"));
    p.push(LessonStep::Remember("cancel"));
    add(
        &mut p,
        [
            Step::Click(Control::Reset),
            Step::Wait,
            Step::CtrlEnter,
            Step::Wait,
            Step::Click(Control::Cancel),
            Step::Wait,
        ],
    );
    p.push(LessonStep::Check("cancel-retains"));
    add(
        &mut p,
        [
            Step::Click(Control::Example),
            Step::Wait,
            Step::Click(Control::ResetExample),
            Step::Wait,
        ],
    );
    p.push(LessonStep::Check("example-reset"));
    add(
        &mut p,
        [
            Step::Click(Control::Assignment),
            Step::Wait,
            Step::Click(Control::Reset),
            Step::Wait,
            Step::Click(Control::Confirm),
            Step::Wait,
        ],
    );
    p.push(LessonStep::Check("selected-reset"));
    add(&mut p, [Step::Click(Control::Challenge), Step::Wait]);
    p.push(LessonStep::Check("challenge"));
    p.push(LessonStep::Capture("challenge"));
    p.push(LessonStep::Finish);
    assert!(example_app.len() <= 64 && p.len() < WIDTH);
    p
}
impl LabApp {
    pub(super) fn unit_manager_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let ordinal = (stage - 13000) / WIDTH;
        let phase = (stage - 13000) % WIDTH;
        let id = IDS[ordinal];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("normal registered unit-manager scenario; never append placeholders");
        let s = self.scenarios[i].clone();
        assert!(
            self.units
                .iter()
                .any(|u| u.id == UNIT && u.labs.iter().map(String::as_str).eq(IDS))
        );
        let Review { plans } = serde_json::from_str(include_str!(
            "../../../tests/fixtures/unit-manager-content-review.json"
        ))
        .unwrap();
        let review = plans.iter().find(|p| p.id == id).unwrap();
        let steps = lesson_plan(&s, review);
        let step = steps
            .get(phase)
            .expect("bounded normal manager native stage");
        if let LessonStep::Ui(input) = step {
            return self
                .manager_native_input(ctx, input)
                .expect("normal plan emits mechanical input only");
        }
        match step {
            LessonStep::Ui(_) => unreachable!(),
            LessonStep::Setup => {
                assert!(self.smoke);
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.selected = None;
                self.filter = Filter {
                    track: "LPIC-2".into(),
                    unit: UNIT.into(),
                    ..Default::default()
                };
                self.page = 0;
                self.help_open = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                let other = self
                    .scenarios
                    .iter()
                    .find(|s| !IDS.contains(&s.id.as_str()))
                    .unwrap()
                    .clone();
                self.progress
                    .attempts
                    .entry(other.id.clone())
                    .or_insert_with(|| Attempt::new(&other, 1));
                let others = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .map(|(key, a)| (key.clone(), serde_json::to_value(a).unwrap()))
                    .collect::<std::collections::BTreeMap<_, _>>();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new(("manager-lesson-others", id)), others)
                });
            }
            LessonStep::Open => {
                self.launch(i);
                self.set_unit_manager_view(i, teaching::View::Introduction);
            }
            LessonStep::Capture(suffix) => {
                return (true, Some(format!("unit-lesson-{ordinal}-{suffix}")));
            }
            LessonStep::Remember(key) => {
                let a = &self.progress.attempts[id];
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("manager-lesson-snapshot", id, key)),
                        serde_json::to_value(a).unwrap(),
                    )
                });
            }
            LessonStep::Reload => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let expected = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "manager-lesson-snapshot",
                            id,
                            "resume",
                        )))
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded.attempts[id]).unwrap(),
                    expected
                );
                self.progress = loaded;
                self.launch(i);
            }
            LessonStep::Check(check) => {
                let a = &self.progress.attempts[id];
                let w = a.workspace.as_ref().unwrap();
                let l = w.unit_manager_learning.as_ref().unwrap();
                let lesson = s.unit_manager_teaching.as_ref().unwrap();
                let engine::State::UnitManager { simulation } = &a.state else {
                    panic!()
                };
                let engine::Goal::UnitManager { definition } = &s.goal else {
                    panic!()
                };
                match *check {
                    "insert-only" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-lesson-snapshot",
                                    id,
                                    "before-insert",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(&a.state).unwrap(), expected["state"]);
                        assert_eq!(
                            serde_json::to_value(&l.state).unwrap(),
                            expected["workspace"]["unit_manager_learning"]["state"]
                        );
                        assert_eq!(
                            serde_json::to_value(&l.draft).unwrap(),
                            expected["workspace"]["unit_manager_learning"]["draft"]
                        );
                        assert_eq!(
                            l.terminal.as_ref().unwrap().command,
                            "systemctl daemon-reload"
                        );
                        assert!(l.terminal.as_ref().unwrap().entries.is_empty());
                    }
                    "fill-only" => {
                        assert_eq!(a.state, s.initial);
                        assert_eq!(l.state, lesson.definition.initial);
                        assert!(!l.checked && !l.draft.matches(&l.state));
                    }
                    "example-pass" => {
                        assert!(l.checked && l.checks(lesson).iter().all(|c| c.0));
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, &s));
                    }
                    "example-fails" => {
                        assert!(l.checked && l.checks(lesson).iter().any(|c| !c.0));
                        assert_eq!(a.state, s.initial);
                    }
                    "assignment-initial" => {
                        assert_eq!(a.state, s.initial);
                        assert!(
                            w.unit_manager_terminal
                                .as_ref()
                                .is_none_or(|t| t.entries.is_empty())
                        );
                    }
                    "assignment-fails" => {
                        assert!(!progress::current_solved(a, &s) && !self.metrics.completion)
                    }
                    "assignment-pass" => assert!(
                        progress::current_solved(a, &s) && self.metrics.completion && a.step == 0,
                        "{id}: failed current checks: {:?}; pending draft: {}",
                        dlms_learning_lab::unit_manager_workspace::checks(
                            definition,
                            simulation,
                            w.unit_manager_draft.as_ref().unwrap()
                        )
                        .into_iter()
                        .filter(|r| !r.0)
                        .collect::<Vec<_>>(),
                        !w.unit_manager_draft.as_ref().unwrap().matches(simulation)
                    ),
                    "partial" => {
                        let rows = dlms_learning_lab::unit_manager_workspace::checks(
                            definition,
                            simulation,
                            w.unit_manager_draft.as_ref().unwrap(),
                        );
                        assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
                        assert!(!progress::current_solved(a, &s));
                    }
                    "query-preserves" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-lesson-snapshot",
                                    id,
                                    "query-state",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(&a.state).unwrap(), expected["state"]);
                        assert!(
                            w.checked && self.metrics.completion && progress::current_solved(a, &s)
                        );
                    }
                    "undo-isolated" => {
                        let config = w
                            .unit_manager_draft
                            .as_ref()
                            .unwrap()
                            .selected_config
                            .as_str();
                        assert_eq!(
                            w.unit_manager_draft.as_ref().unwrap().configuration[config],
                            "PORT=unfinished-assignment\n"
                        );
                        let config = l.draft.selected_config.as_str();
                        assert_ne!(l.draft.configuration[config], "PORT=unfinished-example\n");
                    }
                    "resume-separated" => {
                        let d = w.unit_manager_draft.as_ref().unwrap();
                        assert!(d.editing_configuration && l.draft.editing_configuration);
                        assert_eq!(
                            d.configuration[&d.selected_config],
                            "PORT=unfinished-assignment\n"
                        );
                        assert_eq!(
                            l.draft.configuration[&l.draft.selected_config],
                            "PORT=unfinished-example\n"
                        );
                        assert_eq!(
                            w.unit_manager_terminal.as_ref().unwrap().command,
                            "systemctl 'unfinished assignment"
                        );
                        assert_eq!(
                            l.terminal.as_ref().unwrap().command,
                            "systemctl 'unfinished separate example"
                        );
                        assert!(
                            !w.unit_manager_terminal.as_ref().unwrap().entries.is_empty()
                                && !l.terminal.as_ref().unwrap().entries.is_empty()
                        );
                        assert!(!progress::current_solved(a, &s));
                    }
                    "cancel-retains" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-lesson-snapshot",
                                    id,
                                    "cancel",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(a).unwrap(), expected);
                        assert!(!self.reset_pending);
                    }
                    "example-reset" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-lesson-snapshot",
                                    id,
                                    "cancel",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(&a.state).unwrap(), expected["state"]);
                        assert_eq!(
                            serde_json::to_value(&w.unit_manager_draft).unwrap(),
                            expected["workspace"]["unit_manager_draft"]
                        );
                        assert_eq!(
                            serde_json::to_value(&w.unit_manager_terminal).unwrap(),
                            expected["workspace"]["unit_manager_terminal"]
                        );
                        assert_eq!(l.state, lesson.definition.initial);
                    }
                    "selected-reset" => {
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, &s));
                        assert_eq!(l.state, lesson.definition.initial);
                        let others = ctx
                            .data(|d| {
                                d.get_temp::<std::collections::BTreeMap<String, serde_json::Value>>(
                                    egui::Id::new(("manager-lesson-others", id)),
                                )
                            })
                            .unwrap();
                        for (other, expected) in others {
                            assert_eq!(
                                serde_json::to_value(&self.progress.attempts[&other]).unwrap(),
                                expected
                            );
                        }
                    }
                    "challenge" => assert!(
                        a.challenge
                            && self.metrics.lesson_tabs.is_empty()
                            && !self.metrics.help_visible
                            && self.metrics.manager_example_action.is_none()
                            && !progress::current_solved(a, &s)
                    ),
                    _ => panic!("unknown fixed manager lesson observation"),
                }
            }
            LessonStep::Finish => {
                println!(
                    "PASS native normal unit-manager lesson {id}: introduction, isolated example/experiment/repair, full task, partial/wrong/reference/alternate, current+fresh outcomes, pure queries, actual150/200configtyping+Save+terminalretry, exact isolated drafts/Session/editor-kind resume, Undo, real resets/Cancel and independent Challenge. No novice/systemd/platform claim."
                );
                if ordinal + 1 < IDS.len() {
                    self.smoke_stage = 13000 + (ordinal + 1) * WIDTH - 1;
                } else {
                    println!(
                        "PASS native normal unit-manager all4 flow; actual observed outcomes only, no new lab titles."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        (true, None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_unit_manager_native_plans_cover_all_four_and_type_retry_at_both_larger_scales() {
        let Review { plans } = serde_json::from_str(include_str!(
            "../../../tests/fixtures/unit-manager-content-review.json"
        ))
        .unwrap();
        let labs = pack::load_catalog(&[
            include_str!("../../../scenarios/starter-v1.json"),
            include_str!("../../../scenarios/unit-manager-v1.json"),
        ])
        .unwrap();
        for id in IDS {
            let s = labs
                .iter()
                .find(|s| s.id == id)
                .expect("normal authored scenario registered before compiling acceptance plan");
            let p = lesson_plan(s, plans.iter().find(|p| p.id == id).unwrap());
            assert!(p.len() < WIDTH);
            assert_eq!(
                p.iter()
                    .filter(|s| matches!(s,LessonStep::Ui(Step::Theme(_,scale))if *scale>=1.5))
                    .count(),
                2
            );
            assert!(
                p.iter()
                    .filter(|s| matches!(s,LessonStep::Ui(Step::Type(text))if text=="PORT=1\n"))
                    .count()
                    >= 2
            );
            assert!(
                p.iter()
                    .any(|s| matches!(s, LessonStep::Check("resume-separated")))
            );
            assert!(
                p.iter()
                    .any(|s| matches!(s, LessonStep::Check("selected-reset")))
            );
        }
    }
}
