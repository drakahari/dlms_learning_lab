//! Fixed native private-index flow on the smoke profile; every edit uses real widgets.
use super::*;
use dlms_learning_lab::file_index_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open,
    Click(&'static str),
    Input(&'static str, String),
    Command(String),
    Check(bool, bool),
    Shot(&'static str),
    Save,
    Theme(bool),
    Fresh,
    Challenge(bool),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
}
fn plan() -> Vec<Op> {
    vec![
        Op::Open,
        Op::Shot("file-index-introduction"),
        Op::Click("example"),
        Op::Shot("file-index-guided-start"),
        Op::Command("locate -d /indexes/demo.db guide".into()),
        Op::Command("find /gallery -name 'guide*.txt'".into()),
        Op::Input("config", "PRUNEPATHS = \"/gallery/archive\"\n".into()),
        Op::Click("save"),
        Op::Command(
            "updatedb -l 0 -o /indexes/demo.db -U /gallery --config-file /practice/index.conf"
                .into(),
        ),
        Op::Command("cp /gallery/current/guide.txt /practice/handover.txt".into()),
        Op::Check(true, true),
        Op::Shot("file-index-guided-results"),
        Op::Command("rm /gallery/current/guide.txt".into()),
        Op::Command("locate -d /indexes/demo.db /gallery/current/guide.txt".into()),
        Op::Command("locate -e -d /indexes/demo.db /gallery/current/guide.txt".into()),
        Op::Check(true, false),
        Op::Shot("file-index-stale-practice-feedback"),
        Op::Command("cp /gallery/source/guide.txt /gallery/current/guide.txt".into()),
        Op::Command(
            "updatedb -l 0 -o /indexes/demo.db -U /gallery --config-file /practice/index.conf"
                .into(),
        ),
        Op::Check(true, true),
        Op::Click("assignment"),
        Op::Shot("file-index-assignment"),
        Op::Command("locate -d /indexes/intake.db runbook".into()),
        Op::Check(false, false),
        Op::Input("config", "PRUNEPATHS = \"/srv/intake/archive\"\n".into()),
        Op::Click("save"),
        Op::Command("cp /srv/intake/current/runbook.txt /work/handover.txt".into()),
        Op::Check(false, false),
        Op::Shot("file-index-stale-assignment-feedback"),
        Op::Command(
            "updatedb -l 0 -o /indexes/intake.db -U /srv/intake --config-file /work/index.conf"
                .into(),
        ),
        Op::Check(false, true),
        Op::Command("cp /srv/intake/archive/runbook.txt /work/handover.txt".into()),
        Op::Check(false, false),
        Op::Shot("file-index-wrong-copy-feedback"),
        Op::Command("cp /srv/intake/current/runbook.txt /work/handover.txt".into()),
        Op::Check(false, true),
        Op::Save,
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
        Op::Input(
            "config",
            "PRUNEPATHS=\"/srv/intake/archive /srv/intake/archive\"\n".into(),
        ),
        Op::Click("save"),
        Op::Command(
            "updatedb -l 0 -o /indexes/intake.db -U /srv/intake --config-file /work/index.conf"
                .into(),
        ),
        Op::Command("cp ../srv/intake/current/runbook.txt handover.txt".into()),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Input(
            "config",
            "# Alternative whitespace\nPRUNEPATHS = \"/srv/intake/archive\"\n".into(),
        ),
        Op::Click("save"),
        Op::Command("locate -d /indexes/intake.db runbook".into()),
        Op::Check(false, true),
        Op::Shot("file-index-light150"),
        Op::Theme(false),
        Op::Command("cat /work/handover.txt".into()),
        Op::Check(false, true),
        Op::Shot("file-index-dark200"),
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Save,
        Op::Click("learn"),
        Op::Challenge(false),
    ]
}
fn current(a: &LabApp) -> (&Scenario, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::FileIndex { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (s, simulation)
}
impl LabApp {
    pub(in crate::ui) fn file_index_tick(&mut self, ctx: &egui::Context) {
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("file-index-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let key = egui::Id::new("file-index-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: plan(),
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "FILE_INDEX_NATIVE_COMPLETE: real config/Save, command/Enter, example isolation, stale/copy feedback, alternate repair, invalidation/reset/save-read, Challenge, light150/dark200"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 170000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("FILE_INDEX_NATIVE_STEP {} {:?}", r.at, op)
        }
        r.frames += 1;
        assert!(
            r.frames < 10000,
            "Private-index native step timed out: {op:?}"
        );
        let done = match op {
            Op::Open => {
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic1-file-index-handover")
                    .unwrap();
                self.launch(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    "challenge" => self.metrics.mode_challenge,
                    "learn" => self.metrics.mode_learn,
                    _ => self
                        .metrics
                        .file_index
                        .controls
                        .iter()
                        .find(|c| c.0 == k)
                        .map(|c| c.1),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Input(k, text) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .file_index
                        .fields
                        .iter()
                        .find(|f| f.0 == k)
                        .map(|f| f.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else {
                    let (_, s) = current(self);
                    let scope = if s.view == teaching::View::Example {
                        work::Scope::Example
                    } else {
                        work::Scope::Assignment
                    };
                    let c = s.current(scope);
                    assert_eq!(
                        if k == "config" {
                            &c.config_draft
                        } else {
                            &c.command
                        },
                        &text
                    );
                    true
                }
            }
            Op::Command(text) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .file_index
                        .fields
                        .iter()
                        .find(|f| f.0 == "command")
                        .map(|f| f.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let (_, s) = current(self);
                    let scope = if s.view == teaching::View::Example {
                        work::Scope::Example
                    } else {
                        work::Scope::Assignment
                    };
                    assert_eq!(s.current(scope).command, text);
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
                    r.phase = 3;
                    false
                } else {
                    let (source, s) = current(self);
                    let scope = if s.view == teaching::View::Example {
                        work::Scope::Example
                    } else {
                        work::Scope::Assignment
                    };
                    assert!(
                        s.current(scope)
                            .transcript
                            .last()
                            .is_some_and(|row| row.input == text)
                    );
                    if scope == work::Scope::Example {
                        let engine::State::FileIndex {
                            simulation: initial,
                        } = &source.initial
                        else {
                            panic!()
                        };
                        assert_eq!(s.assignment, initial.assignment);
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&source.id],
                            source
                        ));
                    }
                    true
                }
            }
            Op::Check(example, want) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .file_index
                        .controls
                        .iter()
                        .find(|c| c.0 == "check")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1
                    }
                    false
                } else {
                    let (source, s) = current(self);
                    let engine::Goal::FileIndex { definition } = &source.goal else {
                        panic!()
                    };
                    let scope = if example {
                        work::Scope::Example
                    } else {
                        work::Scope::Assignment
                    };
                    assert!(s.current(scope).checked);
                    assert_eq!(s.checks(definition, scope).unwrap().passed(), want);
                    if !example {
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            want
                        );
                    }
                    true
                }
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                    r.phase = 1;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let read = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let (source, s) = current(self);
                let engine::State::FileIndex {
                    simulation: initial,
                } = &source.initial
                else {
                    panic!()
                };
                assert_eq!(s, &**initial);
                true
            }
            Op::Theme(light) => {
                self.scale = if light { 1.5 } else { 2. };
                ctx.set_zoom_factor(self.scale);
                self.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
                self.lesson_scroll_reset = true;
                true
            }
            Op::Challenge(want) => {
                let (source, _) = current(self);
                assert_eq!(self.progress.attempts[&source.id].challenge, want);
                assert_eq!(
                    self.metrics
                        .file_index
                        .controls
                        .iter()
                        .any(|c| c.0 == "intro"),
                    !want
                );
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.until = now + 0.06
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_route_covers_views_editor_terminal_failure_repair_and_accessible_scales() {
        let p = plan();
        assert!(p.iter().any(|op| matches!(op, Op::Input("config", _))));
        assert!(p.iter().any(|op| matches!(op, Op::Check(false, false))));
        assert!(p.iter().any(|op| matches!(op, Op::Theme(true))));
        assert!(p.iter().any(|op| matches!(op, Op::Theme(false))));
        assert_eq!(p.iter().filter(|op| matches!(op, Op::Shot(_))).count(), 9);
    }
    #[test]
    fn actual_capture_phase_queues_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let key = egui::Id::new("file-index-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("test-private-index-capture")],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(egui::RawInput::default(), |ctx| a.file_index_tick(ctx));
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
}
