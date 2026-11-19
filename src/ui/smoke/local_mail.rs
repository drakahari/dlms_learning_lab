//! Fixed native checks for the authored local-mail unit.
use super::*;
use dlms_learning_lab::local_mail_workspace::{Draft, Task};

const ALIAS_ID: &str = "lpic1-mail-team-alias";
const FORWARD_ID: &str = "lpic1-mail-personal-forward";
const RECOVERY_ID: &str = "lpic1-mail-index-recovery";
const ALIAS_EXAMPLE: &str = "crew: casey, taylor\npostmaster: morgan\n";
const ALIAS_ALTERNATE: &str = "postmaster: sam\nteam: helper\nhelper: JULES, ALEX\n";
const FORWARD_EXAMPLE: &str = "casey, taylor\n";
const FORWARD_CORRECT: &str = "alex, jules\n";
const FORWARD_MISSING: &str = "jules\n";
const RECOVERY_EXAMPLE: &str = "crew: casey\npostmaster: morgan\n";
const RECOVERY_ALTERNATE: &str = "team: first\nfirst: alex\npostmaster: sam\n";

#[derive(Clone, Copy)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Edit(&'static str, &'static str),
    Assert(&'static str),
    Capture(&'static str, &'static str),
    Save,
}

const PLAN: &[Step] = {
    use Step::*;
    &[
        Style(1., false),
        Open(ALIAS_ID),
        Capture("mail-alias-introduction", "top"),
        Click("example"),
        Capture("mail-alias-practice", "editor"),
        Edit("editor", ALIAS_EXAMPLE),
        Click("save-file"),
        Click("test"),
        Assert("practice-fails"),
        Capture("mail-alias-stale-index", "result"),
        Click("rebuild"),
        Click("test"),
        Assert("practice-pass"),
        Capture("mail-alias-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("mail-alias-assignment", "editor"),
        Click("challenge"),
        Assert("challenge"),
        Capture("mail-alias-challenge", "editor"),
        Click("learn"),
        Edit("editor", ALIAS_ALTERNATE),
        Assert("pending"),
        Click("save-file"),
        Click("rebuild"),
        Click("test"),
        Assert("assignment-pass"),
        Capture("mail-alias-assignment-pass", "result"),
        Save,
        Open(FORWARD_ID),
        Capture("mail-forward-introduction", "top"),
        Click("example"),
        Capture("mail-forward-practice", "editor"),
        Edit("editor", FORWARD_EXAMPLE),
        Click("save-file"),
        Click("test"),
        Assert("practice-pass"),
        Capture("mail-forward-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("mail-forward-assignment", "editor"),
        Click("challenge"),
        Assert("challenge"),
        Capture("mail-forward-challenge", "editor"),
        Click("learn"),
        Edit("editor", FORWARD_MISSING),
        Click("save-file"),
        Click("test"),
        Assert("assignment-fails"),
        Capture("mail-forward-missing-copy", "result"),
        Edit("editor", FORWARD_CORRECT),
        Assert("pending"),
        Click("save-file"),
        Click("test"),
        Assert("assignment-pass"),
        Capture("mail-forward-assignment-pass", "result"),
        Save,
        Open(RECOVERY_ID),
        Capture("mail-recovery-introduction", "top"),
        Click("example"),
        Capture("mail-recovery-practice", "editor"),
        Click("help"),
        Capture("mail-recovery-current-index", "help"),
        Click("help"),
        Click("test"),
        Assert("practice-fails"),
        Click("rebuild"),
        Click("test"),
        Assert("practice-loop"),
        Capture("mail-recovery-loop", "result"),
        Edit("editor", RECOVERY_EXAMPLE),
        Click("save-file"),
        Click("rebuild"),
        Click("test"),
        Assert("practice-pass"),
        Capture("mail-recovery-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("mail-recovery-assignment", "editor"),
        Click("challenge"),
        Assert("challenge"),
        Capture("mail-recovery-challenge", "editor"),
        Click("learn"),
        Edit("editor", RECOVERY_ALTERNATE),
        Click("save-file"),
        Click("test"),
        Assert("assignment-fails"),
        Capture("mail-recovery-needs-rebuild", "result"),
        Click("rebuild"),
        Click("test"),
        Assert("assignment-pass"),
        Click("document"),
        Click("document-forward-alex"),
        Edit("editor", FORWARD_MISSING),
        Click("save-file"),
        Click("test"),
        Assert("assignment-fails"),
        Capture("mail-recovery-forward-broken", "result"),
        Edit("editor", FORWARD_CORRECT),
        Click("save-file"),
        Click("test"),
        Assert("assignment-pass"),
        Capture("mail-recovery-assignment-pass", "result"),
        Style(1.5, false),
        Capture("mail-recovery-dark150-editor", "editor"),
        Capture("mail-recovery-dark150-result", "result"),
        Style(2., true),
        Capture("mail-recovery-light200-editor", "editor"),
        Capture("mail-recovery-light200-result", "result"),
        Save,
    ]
};

#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    polls: usize,
    next: f64,
    started: Option<Instant>,
    viewport: Option<egui::Rect>,
    capture_pixels: Option<egui::Vec2>,
    capture_reported: bool,
}

fn lesson_state(app: &LabApp) -> (&Draft, &Task) {
    let scenario = &app.scenarios[app.selected.unwrap()];
    let engine::State::LocalMail { simulation } = &app.progress.attempts[&scenario.id].state else {
        panic!()
    };
    let engine::Goal::LocalMail { definition } = &scenario.goal else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        (&simulation.practice, &definition.example)
    } else {
        (&simulation.assignment, &definition.initial)
    }
}

fn check_state(app: &LabApp, expected: &str) {
    let (draft, task) = lesson_state(app);
    let scenario = &app.scenarios[app.selected.unwrap()];
    let solved = progress::current_solved(&app.progress.attempts[&scenario.id], scenario);
    match expected {
        "challenge" => {
            assert!(app.progress.attempts[&scenario.id].challenge);
            assert!(
                app.metrics
                    .local_mail
                    .controls
                    .iter()
                    .all(|control| !matches!(
                        control.0.as_str(),
                        "introduction" | "example" | "assignment"
                    ))
            );
        }
        "practice-fails" => {
            assert!(!solved);
            assert!(draft.tested.is_some());
            assert!(draft.checks(task).iter().any(|row| !row.0));
        }
        "practice-loop" => {
            assert!(!solved);
            assert!(
                draft
                    .checks(task)
                    .iter()
                    .any(|(_, text)| text.contains("loop"))
            );
        }
        "practice-pass" => {
            assert!(!solved);
            assert!(draft.checks(task).iter().all(|row| row.0));
        }
        "assignment-fresh" => assert_eq!(*draft, Draft::new(task)),
        "pending" => assert!(draft.tested.is_none() && !solved),
        "assignment-fails" => {
            assert!(!solved);
            assert!(draft.tested.is_some());
            assert!(draft.checks(task).iter().any(|row| row.0));
            assert!(draft.checks(task).iter().any(|row| !row.0));
        }
        "assignment-pass" => assert!(solved),
        _ => panic!("Unknown local-mail observation"),
    }
}

fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    if key == "challenge" {
        return app.metrics.mode_challenge;
    }
    if key == "learn" {
        return app.metrics.mode_learn;
    }
    app.metrics
        .local_mail
        .controls
        .iter()
        .find(|row| row.0 == key)
        .map(|row| row.1)
}

impl LabApp {
    pub(in crate::ui) fn local_mail_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("local-mail-native-flow");
        let mut flow = ctx
            .data_mut(|data| data.get_temp::<Flow>(id))
            .unwrap_or_default();
        if flow.at == PLAN.len() {
            println!("LOCAL_MAIL_NATIVE_COMPLETE: {} actions", PLAN.len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 133000 + flow.at;
        let now = ctx.input(|input| input.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "local-mail action {} could not settle: {due:?}",
            flow.at
        );
        if !due.unwrap() {
            ctx.data_mut(|data| data.insert_temp(id, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match PLAN[flow.at] {
            Step::Style(scale, narrow) => {
                if flow.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
                    self.theme_id = if narrow {
                        theme::inventory()
                            .iter()
                            .find(|theme| theme.light)
                            .unwrap()
                            .id
                            .clone()
                    } else {
                        theme::Preferences::default().theme
                    };
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));

                    self.lesson_scroll_reset = true;
                    flow.phase = 1;
                    false
                } else {
                    let viewport = ctx.content_rect();
                    if flow.capture_pixels.is_some()
                        && !flow.capture_reported
                        && started.elapsed().as_secs_f32() > 1.
                    {
                        println!(
                            "NATIVE_CAPTURE_OBSERVED actual_pixels={:?} requested_pixels={:?} fullscreen={:?} scale={} ppp={}",
                            viewport.size() * ctx.pixels_per_point(),
                            flow.capture_pixels,
                            ctx.input(|input| input.viewport().fullscreen),
                            ctx.zoom_factor(),
                            ctx.pixels_per_point()
                        );
                        flow.capture_reported = true;
                    }
                    let scale_ready = (ctx.zoom_factor() - scale).abs() < 0.01;
                    let mode_ready =
                        ctx.input(|input| input.viewport().fullscreen) == Some(!narrow);
                    if narrow && flow.capture_pixels.is_none() {
                        if scale_ready && mode_ready && flow.viewport == Some(viewport) {
                            flow.capture_pixels = Some(capture_size::request(ctx));
                        }
                        flow.viewport = Some(viewport);
                        false
                    } else {
                        let proper_size = !narrow
                            || capture_size::fits(
                                viewport.size() * ctx.pixels_per_point(),
                                flow.capture_pixels.expect("requested narrow capture size"),
                                ctx.input(|input| input.viewport().native_pixels_per_point)
                                    .expect("native display scale"),
                            );
                        let settled = flow.viewport == Some(viewport)
                            && proper_size
                            && scale_ready
                            && mode_ready;
                        if settled && narrow {
                            println!(
                                "NATIVE_CAPTURE_SETTLED actual_pixels={:?} requested_pixels={:?} scale={} ppp={}",
                                viewport.size() * ctx.pixels_per_point(),
                                flow.capture_pixels,
                                ctx.zoom_factor(),
                                ctx.pixels_per_point()
                            );
                        }
                        flow.viewport = Some(viewport);
                        settled
                    }
                }
            }
            Step::Open(id) => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                let index = self
                    .scenarios
                    .iter()
                    .position(|scenario| scenario.id == id)
                    .unwrap();
                self.launch(index);
                true
            }
            Step::Click(key) => self.click_visible_smoke_control(ctx, target(self, key)),
            Step::Edit(key, text) => {
                if flow.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, key)) {
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers {
                                command: true,
                                ..egui::Modifiers::CTRL
                            },
                        },
                        egui::Event::Paste(text.into()),
                    ]);
                    flow.phase = 2;
                    false
                } else {
                    let (draft, _) = lesson_state(self);
                    let actual = draft.text();
                    assert_eq!(actual, text);
                    true
                }
            }
            Step::Assert(expected) => {
                check_state(self, expected);
                true
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    if key == "top"
                        || target(self, key).is_some_and(|r| self.reveal_smoke_control(ctx, r))
                    {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                    flow.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
        };
        if done {
            println!("LOCAL_MAIL_NATIVE_STEP {}", flow.at);
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
            flow.capture_pixels = None;
            flow.capture_reported = false;
            flow.next = now + 0.025;
        } else {
            flow.next = flow.next.max(now + 0.025);
        }
        ctx.data_mut(|data| data.insert_temp(id, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
