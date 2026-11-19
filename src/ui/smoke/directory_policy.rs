//! Fixed native teaching flow; learner commands remain in the in-process model.
use super::*;
use dlms_learning_lab::directory_policy_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(&'static str),
    Click(&'static str),
    Command(&'static str),
    Check(bool, bool),
    Shot(&'static str),
    ResultShot(&'static str),
    ThemeDefault,
    Save,
    Fresh,
    Theme(bool),
    Challenge(bool),
    Unchecked(bool),
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
        Op::Open("lpic1-sticky-collaboration"),
        Op::Shot("sticky-collaboration-introduction"),
        Op::Click("example"),
        Op::Shot("sticky-collaboration-practice"),
        Op::Command("ls -ld /practice/shared"),
        Op::Check(true, false),
        Op::Command("chmod +t /practice/shared"),
        Op::Check(true, true),
        Op::Click("participants"),
        Op::Click("participant-run"),
        Op::Shot("sticky-collaboration-participant"),
        Op::Command("chmod 0700 /practice/shared"),
        Op::Check(true, false),
        Op::ResultShot("sticky-collaboration-overrestrictive-feedback"),
        Op::Command("chmod 1775 /practice/shared"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("sticky-collaboration-assignment"),
        Op::Check(false, false),
        Op::Command("chmod 1700 /work/shared"),
        Op::Check(false, false),
        Op::Command("chmod 1770 /work/shared"),
        Op::Check(false, true),
        Op::Save,
        Op::Command("chmod a=rwx /work/shared"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::Command("chmod 1770 /work/shared"),
        Op::Check(false, true),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
        Op::Command("chmod +t /work/shared"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("ls -ld /work/shared"),
        Op::Shot("sticky-collaboration-light150"),
        Op::Theme(false),
        Op::Command("ls -l /work/shared"),
        Op::Shot("sticky-collaboration-dark200"),
        Op::ThemeDefault,
        Op::Open("lpic1-sticky-collision-handover"),
        Op::Shot("sticky-handover-introduction"),
        Op::Click("example"),
        Op::Command("mv /practice/staging/draft.txt /practice/handover/release.txt"),
        Op::Check(true, false),
        Op::ResultShot("sticky-handover-collision-feedback"),
        Op::Command("mv /practice/staging/draft.txt /practice/handover/taylor-release.txt"),
        Op::Check(true, false),
        Op::Command("rm /practice/staging/obsolete.txt"),
        Op::Check(true, true),
        Op::Command("cp /practice/handover/taylor-release.txt /practice/handover/release.txt"),
        Op::Unchecked(true),
        Op::Check(true, false),
        Op::Command("cp /practice/retained-release.txt /practice/handover/release.txt"),
        Op::Check(true, true),
        Op::Click("assignment"),
        Op::Shot("sticky-handover-assignment"),
        Op::Command("mv /work/staging/draft.txt /work/handover/release.txt"),
        Op::Check(false, false),
        Op::Command("cp /work/staging/draft.txt /work/handover/bea-release.txt"),
        Op::Check(false, false),
        Op::Command("rm /work/staging/draft.txt"),
        Op::Command("rm /work/staging/obsolete.txt"),
        Op::Check(false, true),
        Op::Theme(false),
        Op::Command("cp /work/handover/bea-release.txt /work/handover/release.txt"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("sticky-handover-data-feedback-dark200"),
        Op::Command("cp /work/retained-release.txt /work/handover/release.txt"),
        Op::Check(false, true),
        Op::Save,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Save,
        Op::Click("learn"),
        Op::Challenge(false),
    ]
}
fn current(a: &LabApp) -> (&Scenario, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::DirectoryPolicy { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (s, simulation)
}
fn scope(s: &work::Session, p: bool) -> &work::Scope {
    if p { &s.practice } else { &s.assignment }
}
impl LabApp {
    pub(in crate::ui) fn directory_policy_tick(&mut self, ctx: &egui::Context) {
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("directory-policy-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let key = egui::Id::new("directory-policy-native-route");
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
                "DIRECTORY_POLICY_NATIVE_COMPLETE: actual terminal/Enter, both examples and independent assignments, partial and permission/content feedback, equivalent cp+rm, invalidation/reset/save-read, Challenge, light150/dark200"
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
        self.smoke_stage = 180000 + r.at;
        let op = r.ops[r.at].clone();
        let result_shot = matches!(&op, Op::ResultShot(_));
        if r.frames == 0 {
            println!("DIRECTORY_POLICY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 10000,
            "Directory-policy native step timed out: {op:?}"
        );
        let done = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                if r.at == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
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
                        .directory_policy
                        .controls
                        .iter()
                        .find(|c| c.0 == k)
                        .map(|c| c.1),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .directory_policy
                        .fields
                        .iter()
                        .find(|f| f.0 == "command")
                        .map(|f| f.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let (_, s) = current(self);
                    let p = s.view == teaching::View::Example;
                    assert_eq!(scope(s, p).command, text);
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
                    let p = s.view == teaching::View::Example;
                    assert!(
                        scope(s, p)
                            .transcript
                            .last()
                            .is_some_and(|row| row.contains(text)),
                        "real terminal did not record command {text}"
                    );
                    if p {
                        let engine::State::DirectoryPolicy {
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
            Op::Check(p, want) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .directory_policy
                        .controls
                        .iter()
                        .find(|c| c.0 == "check")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let (source, s) = current(self);
                    let engine::Goal::DirectoryPolicy { definition } = &source.goal else {
                        panic!()
                    };
                    assert!(scope(s, p).checked);
                    assert_eq!(
                        definition.checks(s, p).iter().all(|r| r.0),
                        want,
                        "unexpected model check"
                    );
                    assert_eq!(
                        self.metrics.directory_policy.result.map(|r| r.1 == 0),
                        Some(want)
                    );
                    if !p {
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            want
                        );
                    }
                    true
                }
            }
            Op::Unchecked(p) => {
                let (_, s) = current(self);
                assert!(!scope(s, p).checked);
                assert!(self.metrics.directory_policy.result.is_none());
                true
            }
            Op::Shot(name) | Op::ResultShot(name) => {
                if result_shot && r.phase == 0 {
                    let target = self
                        .metrics
                        .directory_policy
                        .result
                        .expect("Checked feedback exists")
                        .2;
                    if self.reveal_smoke_control(ctx, target) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == u8::from(result_shot) {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                    r.phase = 2;
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
                    serde_json::to_value(&read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let (source, s) = current(self);
                let engine::State::DirectoryPolicy {
                    simulation: initial,
                } = &source.initial
                else {
                    panic!()
                };
                assert_eq!(s, &**initial);
                true
            }
            Op::ThemeDefault => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.lesson_scroll_reset = true;
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
                        .directory_policy
                        .controls
                        .iter()
                        .any(|c| c.0 == "introduction"),
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
            r.until = now + 0.15;
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.until = now + 0.06;
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_capture_callback_does_not_reenter_context() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let key = egui::Id::new("directory-policy-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("directory-policy-test-capture")],
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            a.directory_policy_tick(ctx)
        });
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn actual_scroll_callback_preserves_cooldown_and_visibility_retry_guard() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        a.metrics.directory_policy.controls.push((
            "below".into(),
            egui::Rect::from_min_size(egui::pos2(20., 1500.), egui::vec2(100., 30.)),
            egui::Rect::EVERYTHING,
        ));
        a.metrics.main_viewport = Some(egui::Rect::from_min_size(
            egui::pos2(0., 0.),
            egui::vec2(700., 400.),
        ));
        let key = egui::Id::new("directory-policy-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("below")],
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800., 600.),
                )),
                ..Default::default()
            },
            |ctx| a.directory_policy_tick(ctx),
        );
        assert!(
            a.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        );
        assert!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().until >= 0.15);
    }
}
