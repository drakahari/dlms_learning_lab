//! Fixed native teaching flow; learner commands remain in the in-process model.
use super::*;
use dlms_learning_lab::recursive_tree_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(&'static str),
    Click(&'static str),
    Command(&'static str),
    Output(&'static str),
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
        Op::Open("lpic1-tree-private-stage"),
        Op::Shot("tree-private-introduction"),
        Op::Click("example"),
        Op::Shot("tree-private-practice"),
        Op::Check(true, false),
        Op::Command("umask 077"),
        Op::Command("cp -R /practice/template /practice/working"),
        Op::Command("ls -la /practice/working"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("tree-private-assignment"),
        Op::Check(false, false),
        Op::Command("cp -R /work/incoming /work/private-copy"),
        Op::Command("umask 077"),
        Op::Check(false, false),
        Op::ResultShot("tree-private-partial-feedback"),
        Op::Command("rm -r /work/private-copy"),
        Op::Command("cp --recursive /work/incoming /work/private-copy"),
        Op::Check(false, true),
        Op::Save,
        Op::Command("chmod 000 /work/private-copy/README.txt"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("tree-private-broken-feedback"),
        Op::Command("chmod 600 /work/private-copy/README.txt"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("ls -la /work/private-copy"),
        Op::Shot("tree-private-light150"),
        Op::ResultShot("tree-private-light150-controls"),
        Op::Theme(false),
        Op::Command("ls -la /work/private-copy"),
        Op::Shot("tree-private-dark200"),
        Op::ResultShot("tree-private-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Open("lpic1-tree-verified-retirement"),
        Op::Shot("tree-retire-introduction"),
        Op::Click("example"),
        Op::Shot("tree-retire-practice"),
        Op::Check(true, false),
        Op::Command("cp /practice/retired-notes/.index /practice/ready-notes/.index"),
        Op::Command(
            "cp /practice/retired-notes/docs/runbook.txt /practice/ready-notes/docs/runbook.txt",
        ),
        Op::Command("rm -r /practice/retired-notes"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("tree-retire-assignment"),
        Op::Check(false, false),
        Op::Command("cp /work/old-project/.index /work/handover/.index"),
        Op::Check(false, false),
        Op::ResultShot("tree-retire-partial-feedback"),
        Op::Command("cp /work/old-project/docs/runbook.txt /work/handover/docs/runbook.txt"),
        Op::Command("rm -R /work/old-project"),
        Op::Check(false, true),
        Op::Save,
        Op::Command("chmod 000 /work/handover/README.txt"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("tree-retire-broken-feedback"),
        Op::Command("chmod 644 /work/handover/README.txt"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("ls -la /work/handover"),
        Op::Shot("tree-retire-light150"),
        Op::ResultShot("tree-retire-light150-controls"),
        Op::Theme(false),
        Op::Command("ls -la /work/handover"),
        Op::Shot("tree-retire-dark200"),
        Op::ResultShot("tree-retire-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
    ]
}
fn size_plan() -> Vec<Op> {
    vec![
        Op::Open("lpic1-find-sized-incident"),
        Op::Shot("size-bytes-introduction"),
        Op::Click("example"),
        Op::Shot("size-bytes-practice"),
        Op::Command("find /practice/messages -type f -size +20c"),
        Op::Output("multibyte.log"),
        Op::Check(true, false),
        Op::Command("umask 077"),
        Op::Command("mkdir /practice/evidence/remote"),
        Op::Command("cp /practice/messages/remote/error.log /practice/evidence/remote/error.log"),
        Op::Check(true, false),
        Op::ResultShot("size-bytes-practice-partial"),
        Op::Command("cp /practice/messages/multibyte.log /practice/evidence/multibyte.log"),
        Op::Check(true, true),
        Op::Command(
            "cp /practice/messages/exact-boundary.log /practice/evidence/exact-boundary.log",
        ),
        Op::Unchecked(true),
        Op::Check(true, false),
        Op::ResultShot("size-bytes-practice-wrong-rule"),
        Op::Command("rm /practice/evidence/exact-boundary.log"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("size-bytes-assignment"),
        Op::Command("find /work/incident-records -type f -size +120c"),
        Op::Output("multibyte.log"),
        Op::Check(false, false),
        Op::Command("umask 077"),
        Op::Command("mkdir /work/incident-evidence/remote"),
        Op::Command(
            "cp /work/incident-records/remote/error.log /work/incident-evidence/remote/error.log",
        ),
        Op::Check(false, false),
        Op::ResultShot("size-bytes-partial"),
        Op::Command(
            "cp /work/incident-records/multibyte.log /work/incident-evidence/multibyte.log",
        ),
        Op::Check(false, true),
        Op::Save,
        Op::Command(
            "cp /work/incident-records/exact-boundary.log /work/incident-evidence/exact-boundary.log",
        ),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("size-bytes-extra-feedback"),
        Op::Command("rm /work/incident-evidence/exact-boundary.log"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("ls -la /work/incident-evidence"),
        Op::Shot("size-bytes-light150"),
        Op::ResultShot("size-bytes-light150-controls"),
        Op::Theme(false),
        Op::Command("ls -la /work/incident-evidence"),
        Op::Shot("size-bytes-dark200"),
        Op::ResultShot("size-bytes-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Open("lpic1-find-rounded-evidence"),
        Op::Shot("size-rounded-introduction"),
        Op::Click("example"),
        Op::Shot("size-rounded-practice"),
        Op::Command("find /practice/review-candidates -type f -size -1024c"),
        Op::Output("tiny.log"),
        Op::Check(true, false),
        Op::Command("find /practice/review-candidates -type f -size -1k"),
        Op::Output(".empty-note"),
        Op::Command("umask 077"),
        Op::Command("rm /practice/empty-notes/tiny.log"),
        Op::Command("rm /practice/empty-notes/exact-block.log"),
        Op::Command("mkdir /practice/empty-notes/nested"),
        Op::Command("cp /practice/review-candidates/.empty-note /practice/empty-notes/.empty-note"),
        Op::Command(
            "cp /practice/review-candidates/nested/blank.txt /practice/empty-notes/nested/blank.txt",
        ),
        Op::Check(true, true),
        Op::Command("cp /practice/review-candidates/tiny.log /practice/empty-notes/tiny.log"),
        Op::Unchecked(true),
        Op::Check(true, false),
        Op::ResultShot("size-rounded-practice-wrong-rule"),
        Op::Command("rm /practice/empty-notes/tiny.log"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("size-rounded-assignment"),
        Op::Command("find /work/staged-reports -type f -size -1k"),
        Op::Output(".unfinished"),
        Op::Check(false, false),
        Op::Command("umask 077"),
        Op::Command("rm /work/empty-evidence/tiny.log"),
        Op::Check(false, false),
        Op::ResultShot("size-rounded-partial"),
        Op::Command("rm /work/empty-evidence/exact-block.log"),
        Op::Command("mkdir /work/empty-evidence/nested"),
        Op::Command("cp /work/staged-reports/.unfinished /work/empty-evidence/.unfinished"),
        Op::Command(
            "cp /work/staged-reports/nested/empty.log /work/empty-evidence/nested/empty.log",
        ),
        Op::Check(false, true),
        Op::Save,
        Op::Command("cp /work/staged-reports/unicode.log /work/empty-evidence/unicode.log"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("size-rounded-extra-feedback"),
        Op::Command("rm /work/empty-evidence/unicode.log"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("ls -la /work/empty-evidence"),
        Op::Shot("size-rounded-light150"),
        Op::ResultShot("size-rounded-light150-controls"),
        Op::Theme(false),
        Op::Command("ls -la /work/empty-evidence"),
        Op::Shot("size-rounded-dark200"),
        Op::ResultShot("size-rounded-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
    ]
}
fn current(a: &LabApp) -> (&Scenario, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::RecursiveTree { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (s, simulation)
}
fn scope(s: &work::Session, p: bool) -> &work::Scope {
    if p { &s.practice } else { &s.assignment }
}
impl LabApp {
    pub(in crate::ui) fn recursive_tree_tick(&mut self, ctx: &egui::Context) {
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("recursive-tree-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let key = egui::Id::new("recursive-tree-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("size-search-batch") {
                    size_plan()
                } else {
                    plan()
                },
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("size-search-batch") {
                println!(
                    "FILE_SIZE_NATIVE_COMPLETE: actual terminal/Enter, observed query output, independent practice and assignments, wrong-unit/partial/extra feedback, invalidation/reset/save-read, Learn/Challenge, light150/dark200"
                );
            } else {
                println!(
                    "RECURSIVE_TREE_NATIVE_COMPLETE: actual terminal/Enter, independent practice and assignments, incomplete/private-mode feedback, recursive aliases, invalidation/reset/save-read, Challenge, light150/dark200"
                );
            }
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
            println!("RECURSIVE_TREE_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 10000,
            "Recursive-tree native step timed out: {op:?}"
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
                        .recursive_tree
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
                        .recursive_tree
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
                        let engine::State::RecursiveTree {
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
            Op::Output(text) => {
                let (_, session) = current(self);
                let p = session.view == teaching::View::Example;
                assert!(
                    scope(session, p).transcript.last().is_some_and(|row| row
                        .split_once('\n')
                        .is_some_and(|(_, out)| out.contains(text))),
                    "Actual query output misses {text}"
                );
                true
            }
            Op::Check(p, want) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .recursive_tree
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
                    let engine::Goal::RecursiveTree { definition } = &source.goal else {
                        panic!()
                    };
                    assert!(scope(s, p).checked);
                    assert_eq!(
                        scope(s, p)
                            .checks(if p {
                                &definition.example
                            } else {
                                &definition.assignment
                            })
                            .iter()
                            .all(|r| r.0),
                        want,
                        "unexpected model check"
                    );
                    assert_eq!(
                        self.metrics.recursive_tree.result.map(|r| r.1 == 0),
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
                assert!(self.metrics.recursive_tree.result.is_none());
                true
            }
            Op::Shot(name) | Op::ResultShot(name) => {
                if result_shot && r.phase == 0 {
                    let target = self
                        .metrics
                        .recursive_tree
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
                let engine::State::RecursiveTree {
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
                        .recursive_tree
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
        let key = egui::Id::new("recursive-tree-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("recursive-tree-test-capture")],
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| a.recursive_tree_tick(ctx));
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
        a.metrics.recursive_tree.controls.push((
            "below".into(),
            egui::Rect::from_min_size(egui::pos2(20., 1500.), egui::vec2(100., 30.)),
            egui::Rect::EVERYTHING,
        ));
        a.metrics.main_viewport = Some(egui::Rect::from_min_size(
            egui::pos2(0., 0.),
            egui::vec2(700., 400.),
        ));
        let key = egui::Id::new("recursive-tree-native-route");
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
            |ctx| a.recursive_tree_tick(ctx),
        );
        assert!(
            a.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        );
        assert!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().until >= 0.15);
    }
}
