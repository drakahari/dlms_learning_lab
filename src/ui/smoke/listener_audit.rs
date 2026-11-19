//! Fixed native listener review. Commands enter real widgets on private smoke data.
use super::*;
#[derive(Clone, Debug)]
enum Op {
    Open,
    Click(&'static str),
    Command(String, bool),
    Practice(bool),
    Check(bool),
    Shot(String),
    Save,
    Fresh,
    Theme(bool),
    Challenge,
    Learn,
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
fn plan(s: &Scenario) -> Vec<Op> {
    let mut p = vec![
        Op::Open,
        Op::Shot("listener-introduction".into()),
        Op::Click("example"),
    ];
    for c in &s.process_teaching.as_ref().unwrap().example {
        p.push(Op::Command(c.clone(), true));
    }
    p.extend([
        Op::Practice(true),
        Op::Shot("listener-practice-retired".into()),
        Op::Command("kill -STOP 6020".into(), true),
        Op::Command("ss -lntup".into(), true),
        Op::Practice(false),
        Op::Shot("listener-protected-practice-paused".into()),
        Op::Command("kill -CONT 6020".into(), true),
        Op::Practice(true),
        Op::Click("assignment"),
        Op::Shot("listener-assignment".into()),
        Op::Command("ss -lntup".into(), false),
        Op::Command("kill -STOP 410".into(), false),
        Op::Command("ss -lntp".into(), false),
        Op::Check(false),
        Op::Shot("listener-stop-is-not-closed".into()),
        Op::Command("kill -TERM 410".into(), false),
        Op::Check(false),
        Op::Shot("listener-pending-term".into()),
        Op::Command("kill -CONT 410".into(), false),
        Op::Check(true),
        Op::Shot("listener-completed".into()),
        Op::Save,
        Op::Command("kill -STOP 411".into(), false),
        Op::Check(false),
        Op::Shot("listener-protected-owner-paused".into()),
        Op::Command("kill -CONT 411".into(), false),
        Op::Check(true),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
        Op::Command("kill -9 410".into(), false),
        Op::Check(true),
        Op::Save,
    ]);
    for light in [true, false] {
        p.extend([
            Op::Theme(light),
            Op::Command("ss -lntup".into(), false),
            Op::Check(true),
            Op::Shot(
                if light {
                    "listener-light150-actual-terminal"
                } else {
                    "listener-dark200-actual-terminal"
                }
                .into(),
            ),
        ]);
    }
    p.extend([
        Op::Click("challenge"),
        Op::Challenge,
        Op::Save,
        Op::Click("learn"),
        Op::Learn,
    ]);
    p
}
impl LabApp {
    pub(in crate::ui) fn listener_audit_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("listener-audit-native-route");
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("listener-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: plan(
                    self.scenarios
                        .iter()
                        .find(|s| s.id == "lpic1-listener-audit")
                        .unwrap(),
                ),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "LISTENER_NATIVE_COMPLETE: real terminal input/Enter, worked practice, STOP/pending TERM, alternate retirement, protected owner feedback, invalidation, save/read, reset, Challenge and light150/dark200 typing"
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
        self.smoke_stage = 140000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("LISTENER_NATIVE_STEP {} {:?}", r.at, op)
        }
        r.frames += 1;
        assert!(r.frames < 10000, "Listener native step timed out: {op:?}");
        let done = match op {
            Op::Open => {
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic1-listener-audit")
                    .unwrap();
                self.launch(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "example" => self.metrics.example_action,
                    "assignment" => self
                        .metrics
                        .assignment_action
                        .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    "challenge" => self.metrics.mode_challenge,
                    "learn" => self.metrics.mode_learn,
                    _ => panic!("Unknown fixed listener control"),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text, practice) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.terminal) {
                        r.phase = 1
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let i = self.selected.unwrap();
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let draft = if practice {
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .process_learning
                            .as_ref()
                            .unwrap()
                            .command
                    } else {
                        &self.input
                    };
                    assert_eq!(
                        draft, &text,
                        "Actual input widget must contain the fixed command before Enter"
                    );
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
                    let s = &self.scenarios[self.selected.unwrap()];
                    let a = &self.progress.attempts[&s.id];
                    let transcript = if practice {
                        assert_eq!(a.state, s.initial);
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .process_learning
                            .as_ref()
                            .unwrap()
                            .transcript
                    } else {
                        &a.transcript
                    };
                    assert!(
                        transcript.last().is_some_and(|line| line.contains(&text)),
                        "Real command must appear in terminal transcript: {text}: {transcript:?}"
                    );
                    true
                }
            }
            Op::Practice(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                let l = s.process_teaching.as_ref().unwrap();
                let learn = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .process_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.outcomes.iter().all(|c| c.check(&learn.table).0), want);
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Check(want) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.check) {
                        r.phase = 1
                    }
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&s.id], s),
                        want
                    );
                    assert_eq!(self.metrics.completion, want);
                    true
                }
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
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
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert_eq!(a.state, s.initial);
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .process_learning
                        .as_ref()
                        .unwrap()
                        .table,
                    s.process_teaching.as_ref().unwrap().fixture
                );
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
            Op::Challenge => {
                let i = self.selected.unwrap();
                assert!(self.progress.attempts[&self.scenarios[i].id].challenge);
                assert_eq!(self.process_view(i), teaching::View::Assignment);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                true
            }
            Op::Learn => {
                let i = self.selected.unwrap();
                assert!(!self.progress.attempts[&self.scenarios[i].id].challenge);
                assert_eq!(self.metrics.lesson_tabs.len(), 3);
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
            self.started = Instant::now()
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn listener_native_plan_retains_real_input_partial_recovery_reset_and_protected_invalidation() {
        let c = pack::ccna_pilot().unwrap();
        let s = c.iter().find(|s| s.id == "lpic1-listener-audit").unwrap();
        let p = plan(s);
        assert_eq!(
            p.iter()
                .filter(|o| matches!(o, Op::Command(_, true)))
                .count(),
            11
        );
        for command in [
            "kill -STOP 410",
            "kill -TERM 410",
            "kill -CONT 410",
            "kill -STOP 411",
            "kill -9 410",
        ] {
            assert!(
                p.iter()
                    .any(|o| matches!(o,Op::Command(c,false)if c==command))
            )
        }
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 1);
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Save)).count(), 3);
        assert!(
            matches!(p.last(), Some(Op::Learn)),
            "Return to Learn before teaching screenshots"
        );
    }
    #[test]
    fn listener_capture_phase_emits_native_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-listener-audit")
            .unwrap();
        a.launch(i);
        a.set_process_view(i, teaching::View::Example);
        let raw = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw(), |ctx| a.render(ctx));
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("listener-audit-native-route"),
                Route {
                    ops: vec![Op::Shot("listener-capture-regression".into())],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(raw(), |ctx| {
            a.render(ctx);
            a.listener_audit_tick(ctx);
        });
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
}
