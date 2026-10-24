//! App-only actual-input plan for an unbundled virtual manager fixture.
//! Reference values populate native input events only; no reference State is installed.
use super::*;
use dlms_learning_lab::unit_manager_workspace::{Draft, EditAction};
#[derive(Clone)]
pub(super) enum Control {
    Try,
    Fill,
    Save,
    Check,
    Assignment,
    Example,
    Kind(bool),
    Picker,
    Choice(String),
    Editor,
    Command,
    Run,
    Inspector,
    Reset,
    Cancel,
    Confirm,
    Challenge,
    ResetExample,
    ExampleActionPicker,
    ExampleActionChoice(usize),
    InsertExample,
}
#[derive(Clone)]
pub(super) enum Step {
    Setup,
    Wait,
    Click(Control),
    Type(String),
    Enter,
    CtrlEnter,
    Undo,
    Assert(&'static str),
    Record(&'static str),
    Reload,
    Theme(&'static str, f32),
    Shot(&'static str),
    Finish,
}
fn ctrl(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
pub(super) fn command(plan: &mut Vec<Step>, text: &str, keyboard: bool) {
    plan.extend([
        Step::Click(Control::Command),
        Step::Type(text.into()),
        if keyboard {
            Step::Enter
        } else {
            Step::Click(Control::Run)
        },
        Step::Wait,
    ]);
}
pub(super) fn draft(plan: &mut Vec<Step>, configuration: bool, key: &str, text: &str, save: bool) {
    assert!(key.len() <= 128 && text.len() <= 8192);
    plan.extend([
        Step::Click(Control::Kind(configuration)),
        Step::Click(Control::Picker),
        Step::Click(Control::Choice(key.into())),
        Step::Click(Control::Editor),
        Step::Type(text.into()),
        Step::Wait,
    ]);
    if save {
        plan.extend([Step::Click(Control::Save), Step::Wait]);
    }
}
fn worked(s: &Scenario) -> Draft {
    let action: EditAction = serde_json::from_str(
        s.solution
            .first()
            .unwrap()
            .strip_prefix("unit-manager ")
            .unwrap(),
    )
    .unwrap();
    let EditAction::Save { draft } = action else {
        panic!("native fixture first action is bounded editor source")
    };
    draft
}
fn plan(s: &Scenario) -> Vec<Step> {
    let target = worked(s);
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    let example = lesson
        .solution
        .iter()
        .find_map(|a| {
            if let EditAction::Save { draft } = a {
                Some(draft)
            } else {
                None
            }
        })
        .unwrap();
    let mut p = vec![
        Step::Setup,
        Step::Wait,
        Step::Shot("unit-manager-introduction"),
        Step::Click(Control::Try),
        Step::Wait,
        Step::Shot("unit-manager-example-initial"),
        Step::Click(Control::Fill),
        Step::Wait,
        Step::Assert("example-filled-only"),
        Step::Click(Control::Save),
        Step::Wait,
    ];
    for text in [
        "systemctl daemon-reload",
        "systemctl enable sample-application.service",
        "systemctl set-default sample.target",
        "systemctl start sample.target",
    ] {
        command(&mut p, text, true);
    }
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("example-pass"),
        Step::Shot("unit-manager-example-ungraded"),
        Step::Click(Control::Assignment),
        Step::Wait,
        Step::Assert("assignment-initial"),
        Step::Shot("unit-manager-assignment"),
    ]);
    command(&mut p, "systemctl start work-application.service", false);
    command(
        &mut p,
        "journalctl -u work-application.service -o cat",
        true,
    );
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("not-solved"),
        Step::Shot("unit-manager-failed-initialization"),
    ]);
    // Repair the unit/order/policy first; the still-invalid application config
    // must produce meaningful partial completion, not a canned success.
    draft(
        &mut p,
        false,
        "work-application.service",
        &target.documents["work-application.service"],
        true,
    );
    for text in [
        "systemctl daemon-reload",
        "systemctl enable work-application.service",
        "systemctl set-default work.target",
        "systemctl stop work-application.service",
        "systemctl start work.target",
    ] {
        command(&mut p, text, false);
    }
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("partial"),
        Step::Shot("unit-manager-partial-configuration"),
    ]);
    draft(
        &mut p,
        true,
        "/etc/work/application.conf",
        &target.configuration["/etc/work/application.conf"],
        true,
    );
    for text in [
        "systemctl stop work-application.service",
        "systemctl start work-application.service",
    ] {
        command(&mut p, text, true);
    }
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("solved"),
        Step::Shot("unit-manager-complete"),
        Step::Record("solved-state"),
    ]);
    command(&mut p, "systemctl status work-application.service", true);
    command(
        &mut p,
        "systemctl is-enabled work-application.service",
        false,
    );
    p.extend([
        Step::Assert("query-keeps-success"),
        Step::Shot("unit-manager-verified-runtime"),
    ]);
    // A valid different fictional program is diagnosable BEFORE checking:
    // loaded source may use observer while current runtime still consumed app.
    let different = target.documents["work-application.service"]
        .replace("/sim/bin/work-application", "/sim/bin/work-observer");
    draft(&mut p, false, "work-application.service", &different, true);
    command(&mut p, "systemctl daemon-reload", true);
    p.extend([
        Step::Click(Control::Inspector),
        Step::Wait,
        Step::Assert("loaded-running-mismatch"),
        Step::Shot("unit-manager-loaded-runtime-mismatch"),
        Step::Click(Control::Inspector),
    ]);
    draft(
        &mut p,
        false,
        "work-application.service",
        &target.documents["work-application.service"],
        true,
    );
    for text in [
        "systemctl daemon-reload",
        "systemctl stop work-application.service",
        "systemctl start work-application.service",
    ] {
        command(&mut p, text, true);
    }
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("solved"),
    ]);
    // The reviewed Before alternative is genuinely entered in both unit editors.
    let alternative: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/unit-manager-workspace-preview.json"
    ))
    .unwrap();
    let actions: Vec<EditAction> =
        serde_json::from_value(alternative["alternate"].clone()).unwrap();
    let alt = actions
        .iter()
        .find_map(|a| {
            if let EditAction::Save { draft } = a {
                Some(draft)
            } else {
                None
            }
        })
        .unwrap();
    let changed = alt
        .documents
        .iter()
        .filter(|(k, v)| target.documents.get(*k) != Some(*v))
        .collect::<Vec<_>>();
    assert!(changed.len() <= 4);
    for (key, text) in changed {
        draft(&mut p, false, key, text, true);
    }
    for text in [
        "systemctl daemon-reload",
        "systemctl stop work-application.service",
        "systemctl stop work-prepare.service",
        "systemctl start work.target",
    ] {
        command(&mut p, text, true);
    }
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("solved"),
        Step::Shot("unit-manager-alternate-before-ordering"),
    ]);
    command(&mut p, "systemctl stop work-observer.service", false);
    p.extend([
        Step::Assert("not-solved"),
        Step::Shot("unit-manager-state-invalidation"),
    ]);
    command(&mut p, "systemctl start work-observer.service", true);
    p.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("solved"),
    ]);
    draft(
        &mut p,
        true,
        "/etc/work/application.conf",
        "PORT=unfinished-assignment\n",
        false,
    );
    p.extend([
        Step::Click(Control::Command),
        Step::Type("systemctl 'unfinished assignment".into()),
        Step::Wait,
        Step::Click(Control::Example),
        Step::Wait,
    ]);
    draft(
        &mut p,
        true,
        "/etc/sample/application.conf",
        "PORT=unfinished-example\n",
        false,
    );
    p.extend([
        Step::Undo,
        Step::Wait,
        Step::Assert("assignment-keeps-own-draft"),
        Step::Type("PORT=unfinished-example\n".into()),
        Step::Wait,
        Step::Click(Control::Command),
        Step::Type("systemctl 'unfinished separate example".into()),
        Step::Wait,
        Step::Click(Control::Assignment),
        Step::Wait,
        Step::Record("before-reload"),
        Step::Reload,
        Step::Wait,
        Step::Assert("separate-resume"),
        Step::Shot("unit-manager-resumed-drafts"),
        Step::Record("cancel-state"),
        Step::Click(Control::Reset),
        Step::Wait,
        Step::CtrlEnter,
        Step::Wait,
        Step::Click(Control::Cancel),
        Step::Wait,
        Step::Assert("cancel-keeps-work"),
        Step::Click(Control::Example),
        Step::Wait,
        Step::Click(Control::ResetExample),
        Step::Wait,
        Step::Assert("example-reset-only"),
        Step::Shot("unit-manager-example-reset-isolated"),
        Step::Click(Control::Assignment),
        Step::Wait,
    ]);
    p.extend([
        Step::Click(Control::Reset),
        Step::Wait,
        Step::Click(Control::Confirm),
        Step::Wait,
        Step::Assert("selected-reset"),
        Step::Click(Control::Challenge),
        Step::Wait,
        Step::Theme("dlms-light", 1.5),
        Step::Wait,
        Step::Shot("unit-manager-challenge-light-150"),
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert("challenge-independent"),
        Step::Shot("unit-manager-challenge-light-150-feedback"),
        Step::Theme("dlms-dark", 2.),
        Step::Wait,
        Step::Shot("unit-manager-challenge-dark-200"),
        Step::Click(Control::Check),
        Step::Wait,
        Step::Shot("unit-manager-challenge-dark-200-feedback"),
        Step::Finish,
    ]);
    assert!(example.valid() && p.len() < 400);
    p
}
impl LabApp {
    fn manager_native_click(&mut self, ctx: &egui::Context, target: &Control) -> bool {
        if matches!(target, Control::Editor) {
            let view = self
                .metrics
                .editor_viewport
                .expect("manager editor viewport");
            let visible = self
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            let hit = view.intersect(visible);
            if hit.height() >= 16. && hit.width() >= 24. {
                return self.click_smoke_control(ctx, Some(hit.shrink(2.)));
            }
            let point = egui::Rect::from_center_size(view.center(), egui::vec2(20., 16.));
            self.reveal_smoke_control(ctx, point);
            return false;
        }
        let rect = match target {
            Control::Try => self.metrics.example_action,
            Control::Fill => self.metrics.insert,
            Control::Save => self.metrics.editor_save,
            Control::Check => self.metrics.check,
            Control::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Control::Example => self.metrics.lesson_tabs.get(1).copied(),
            Control::Kind(config) => self
                .metrics
                .manager_kind
                .iter()
                .find(|(kind, _)| kind == config)
                .map(|(_, r)| *r),
            Control::Picker => self.metrics.manager_slot,
            Control::Choice(name) => self
                .metrics
                .manager_items
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, r)| *r),
            Control::Command => self.metrics.terminal,
            Control::Run => self.metrics.manager_run,
            Control::Inspector => self.metrics.manager_inspector,
            Control::Reset => self.metrics.reset,
            Control::Cancel => self.metrics.reset_cancel,
            Control::Confirm => self.metrics.confirm_reset,
            Control::Challenge => self.metrics.manager_challenge,
            Control::ResetExample => self.metrics.example_reset,
            Control::ExampleActionPicker => self.metrics.manager_example_action,
            Control::ExampleActionChoice(index) => self
                .metrics
                .manager_example_items
                .iter()
                .find(|(i, _)| i == index)
                .map(|(_, rect)| *rect),
            Control::InsertExample => self.metrics.command_insert,
            Control::Editor => unreachable!(),
        };
        // Header/reset controls have their own top-panel location; popup options
        // use the actual visible menu rect, never a guessed row offset.
        if matches!(
            target,
            Control::Choice(_)
                | Control::ExampleActionChoice(_)
                | Control::Reset
                | Control::Challenge
        ) {
            self.click_smoke_control(ctx, rect)
        } else {
            self.click_visible_smoke_control(ctx, rect)
        }
    }
    pub(super) fn manager_native_input(
        &mut self,
        ctx: &egui::Context,
        step: &Step,
    ) -> Option<(bool, Option<String>)> {
        match step {
            Step::Wait => {}
            Step::Click(target) => {
                if !self.manager_native_click(ctx, target) {
                    return Some((false, None));
                }
            }
            Step::Type(text) => {
                assert!(text.len() <= 8192);
                let press = ctrl(egui::Key::A);
                let mut release = press.clone();
                if let egui::Event::Key { pressed, .. } = &mut release {
                    *pressed = false;
                }
                self.pending_events = vec![press, release, egui::Event::Text(text.clone())];
            }
            Step::Enter => {
                self.pending_events = [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    })
                    .collect();
            }
            Step::CtrlEnter | Step::Undo => {
                let key = if matches!(step, Step::Undo) {
                    egui::Key::Z
                } else {
                    egui::Key::Enter
                };
                let press = ctrl(key);
                let mut release = press.clone();
                if let egui::Event::Key { pressed, .. } = &mut release {
                    *pressed = false;
                }
                self.pending_events = vec![press, release];
            }
            Step::Shot(name) => return Some((true, Some((*name).into()))),
            Step::Theme(theme_id, scale) => {
                self.scale = *scale;
                self.theme_id = (*theme_id).into();
                theme::apply(ctx, theme_id);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 900.)));
            }
            _ => return None,
        }
        Some((true, None))
    }
    pub(super) fn unit_manager_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/unit-manager-pack-preview.json"
        ))
        .unwrap()
        .remove(0);
        let steps = plan(&s);
        let index = stage - 12000;
        let step = steps.get(index).expect("bounded manager smoke stage");
        let id = s.id.as_str();
        if matches!(step, Step::Setup) {
            assert!(self.smoke, "development fixture only inside app smoke");
            assert!(!pack::bundled().unwrap().iter().any(|x| x.id == id));
            let other = self.scenarios[0].clone();
            self.progress
                .attempts
                .entry(other.id.clone())
                .or_insert_with(|| Attempt::new(&other, 1));
            ctx.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("manager-other-attempt"),
                    (
                        other.id.clone(),
                        serde_json::to_value(&self.progress.attempts[&other.id]).unwrap(),
                    ),
                )
            });
            if !self.scenarios.iter().any(|x| x.id == id) {
                self.scenarios.push(s.clone());
            }
            let i = self.scenarios.iter().position(|x| x.id == id).unwrap();
            self.launch(i);
            self.set_unit_manager_view(i, teaching::View::Introduction);
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            return (true, None);
        }
        if let Some(result) = self.manager_native_input(ctx, step) {
            return result;
        }
        let i = self.scenarios.iter().position(|x| x.id == id).unwrap();
        let a = &self.progress.attempts[id];
        match step {
            Step::Setup => unreachable!(),
            Step::Wait
            | Step::Click(_)
            | Step::Type(_)
            | Step::Enter
            | Step::CtrlEnter
            | Step::Undo
            | Step::Shot(_)
            | Step::Theme(_, _) => {
                unreachable!("shared mechanical input handled before state observations")
            }
            Step::Record(name) => {
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("manager-snapshot", name)),
                        serde_json::to_value(a).unwrap(),
                    )
                });
            }
            Step::Reload => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let expected = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "manager-snapshot",
                            "before-reload",
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
            Step::Assert(check) => {
                let w = a.workspace.as_ref().unwrap();
                let lesson = s.unit_manager_teaching.as_ref().unwrap();
                match *check {
                    "example-filled-only" => {
                        let l = w.unit_manager_learning.as_ref().unwrap();
                        assert_eq!(l.state, lesson.definition.initial);
                        assert!(!l.draft.matches(&l.state) && !l.checked);
                        assert_eq!(a.state, s.initial);
                    }
                    "example-pass" => {
                        let l = w.unit_manager_learning.as_ref().unwrap();
                        assert!(l.checked && l.checks(lesson).iter().all(|c| c.0));
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, &s));
                    }
                    "assignment-initial" => {
                        assert_eq!(a.state, s.initial);
                        assert!(
                            w.unit_manager_terminal
                                .as_ref()
                                .is_none_or(|t| t.entries.is_empty())
                        );
                    }
                    "solved" => {
                        assert!(progress::current_solved(a, &s));
                        assert!(self.metrics.completion);
                    }
                    "not-solved" => assert!(!progress::current_solved(a, &s)),
                    "partial" => {
                        let engine::State::UnitManager { simulation } = &a.state else {
                            panic!()
                        };
                        let engine::Goal::UnitManager { definition } = &s.goal else {
                            panic!()
                        };
                        let checks = dlms_learning_lab::unit_manager_workspace::checks(
                            definition,
                            simulation,
                            w.unit_manager_draft.as_ref().unwrap(),
                        );
                        assert!(checks.iter().any(|x| x.0) && checks.iter().any(|x| !x.0));
                        assert!(!progress::current_solved(a, &s));
                    }
                    "query-keeps-success" => {
                        let before = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-snapshot",
                                    "solved-state",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(&a.state).unwrap(), before["state"]);
                        assert!(
                            w.checked && self.metrics.completion && progress::current_solved(a, &s)
                        );
                    }
                    "loaded-running-mismatch" => {
                        let engine::State::UnitManager { simulation } = &a.state else {
                            panic!()
                        };
                        let rows = crate::ui::unit_manager_workspace::inspection_rows(
                            simulation,
                            "work-application.service",
                        );
                        assert!(
                            rows.iter().any(|(k, v)| k == "Loaded virtual program"
                                && v == "/sim/bin/work-observer")
                        );
                        assert!(rows.iter().any(|(k, v)| k == "Runtime virtual program"
                            && v == "/sim/bin/work-application"));
                        assert!(self.metrics.manager_inspector_open);
                        assert!(!progress::current_solved(a, &s));
                    }
                    "assignment-keeps-own-draft" => {
                        let d = w.unit_manager_draft.as_ref().unwrap();
                        assert_eq!(
                            d.configuration["/etc/work/application.conf"],
                            "PORT=unfinished-assignment\n"
                        );
                        let l = w.unit_manager_learning.as_ref().unwrap();
                        assert_ne!(
                            l.draft.configuration["/etc/sample/application.conf"],
                            "PORT=unfinished-example\n"
                        );
                    }
                    "example-reset-only" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-snapshot",
                                    "cancel-state",
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
                        let l = w.unit_manager_learning.as_ref().unwrap();
                        assert_eq!(l.state, lesson.definition.initial);
                        assert!(
                            l.terminal
                                .as_ref()
                                .is_none_or(|t| t.entries.is_empty() && t.command.is_empty())
                        );
                    }
                    "separate-resume" => {
                        let d = w.unit_manager_draft.as_ref().unwrap();
                        let l = w.unit_manager_learning.as_ref().unwrap();
                        assert!(d.editing_configuration && l.draft.editing_configuration);
                        assert_eq!(
                            d.configuration["/etc/work/application.conf"],
                            "PORT=unfinished-assignment\n"
                        );
                        assert_eq!(
                            l.draft.configuration["/etc/sample/application.conf"],
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
                    "cancel-keeps-work" => {
                        let expected = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new((
                                    "manager-snapshot",
                                    "cancel-state",
                                )))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(a).unwrap(), expected);
                        assert!(!self.reset_pending);
                    }
                    "selected-reset" => {
                        assert_eq!(a.state, s.initial);
                        assert!(
                            w.unit_manager_terminal
                                .as_ref()
                                .is_none_or(|t| t.entries.is_empty() && t.command.is_empty())
                        );
                        assert_eq!(
                            w.unit_manager_learning.as_ref().unwrap().state,
                            lesson.definition.initial
                        );
                        let (other, expected) = ctx
                            .data(|d| {
                                d.get_temp::<(String, serde_json::Value)>(egui::Id::new(
                                    "manager-other-attempt",
                                ))
                            })
                            .unwrap();
                        assert_eq!(
                            serde_json::to_value(&self.progress.attempts[&other]).unwrap(),
                            expected
                        );
                    }
                    "challenge-independent" => {
                        assert!(
                            a.challenge
                                && self.metrics.lesson_tabs.is_empty()
                                && !self.metrics.help_visible
                        );
                        assert!(!progress::current_solved(a, &s));
                    }
                    _ => panic!("unknown fixed smoke assertion"),
                }
            }
            Step::Finish => {
                assert!(self.smoke);
                println!(
                    "PASS native unit-manager private fixture: real fixed-slot menus/editor typing/Save and virtual commands, isolated ungraded example, partial/failure/repair, current+fresh completion and Before alternative, pure query retention, loaded/runtime mismatch inspector, state/edit invalidation, exact separate draft/Session/editor-kind resume, selected Reset/Cancel, Challenge/light150/dark200. No new catalog/real-systemd/novice/platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_manager_native_plan_is_bounded_and_private_with_input_targets_only() {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/unit-manager-pack-preview.json"
        ))
        .unwrap()
        .remove(0);
        let steps = plan(&s);
        assert!(steps.len() < 400);
        assert_eq!(steps.iter().filter(|s| matches!(s, Step::Setup)).count(), 1);
        assert_eq!(
            steps.iter().filter(|s| matches!(s, Step::Finish)).count(),
            1
        );
        assert!(
            steps
                .iter()
                .all(|s| !matches!(s,Step::Type(text)if text.len()>8192))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, Step::Assert("loaded-running-mismatch")))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, Step::Assert("separate-resume")))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s, Step::Assert("example-reset-only")))
        );
        assert!(!pack::bundled().unwrap().iter().any(|lab| lab.id == s.id));
    }
}
