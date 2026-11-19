//! Actual native notice editing/commands; never a learner-host execution route.
use super::*;
use dlms_learning_lab::maintenance_notice_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-maintenance-login-notices",
    "lpic2-maintenance-restart-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Choose(work::Editor),
    Field(&'static str, String),
    Assert(&'static str),
    Capture(String, &'static str),
    KeyboardRun,
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v18.json"
        ))
        .unwrap()
        .scenarios
    });
    let source = all.iter().find(|s| s.id == IDS[n]).unwrap();
    let engine::Goal::MaintenanceNotice { definition } = &source.goal else {
        panic!()
    };
    definition
}
fn command(p: &mut Vec<Step>, raw: &str) {
    p.push(Step::Field("command", raw.into()));
    p.push(Step::Click("run"));
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let def = fixture(n);
            let t = &def.initial;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("notice-{n}-introduction"), "top"),
                Step::Capture(format!("notice-{n}-worked"), "start"),
                Step::Click("example"),
                Step::Capture(format!("notice-{n}-practice-start"), "editor-viewport"),
                Step::Click("fill-worked"),
                Step::Assert("example-only"),
                Step::Click("save"),
                Step::Click("check"),
                Step::Assert("practice-partial"),
                Step::Capture(format!("notice-{n}-practice-partial"), "result"),
            ]);
            if t.requires_restart {
                command(&mut p, "date +%H:%M");
                command(&mut p, "shutdown -c");
                command(&mut p, "shutdown -r +30");
            }
            p.extend([
                Step::Choose(work::Editor::Stdin),
                Step::Click("fill-worked"),
            ]);
            command(&mut p, "who");
            p.push(Step::Capture(
                format!("notice-{n}-actual-who"),
                "last-result",
            ));
            p.extend([
                Step::Field("command", "wall".into()),
                Step::KeyboardRun,
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(format!("notice-{n}-practice-pass"), "result"),
                Step::Field(
                    "editor",
                    def.example
                        .required
                        .render()
                        .replace("Service:", "Unknown:"),
                ),
                Step::Click("check"),
                Step::Assert("unsent"),
                Step::Capture(format!("notice-{n}-unsent-feedback"), "result"),
                Step::Click("run"),
                Step::Click("check"),
                Step::Assert("practice-partial"),
                Step::Capture(format!("notice-{n}-live-feedback"), "result"),
                Step::Click("fill-worked"),
                Step::Click("run"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("notice-{n}-assignment"), "top"),
                Step::Capture(format!("notice-{n}-assignment-editor"), "editor-viewport"),
            ]);
            if t.requires_restart {
                command(&mut p, "date +%H:%M");
                command(&mut p, "shutdown -c");
                p.extend([
                    Step::Click("check"),
                    Step::Assert("assignment-fail"),
                    Step::Capture(format!("notice-{n}-cancel-partial"), "result"),
                ]);
                command(&mut p, "shutdown -r +30");
            }
            p.extend([
                Step::Field("editor", t.required.render()),
                Step::Click("save"),
                Step::Choose(work::Editor::Stdin),
                Step::Field("editor", t.required.render()),
                Step::Field("command", "wall".into()),
                Step::KeyboardRun,
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(format!("notice-{n}-assignment-pass"), "result"),
                Step::Save,
                Step::Field("editor", t.required.render().replace("Starts:", "Missing:")),
                Step::Assert("assignment-unchecked"),
                Step::Click("check"),
                Step::Assert("assignment-fail"),
                Step::Capture(format!("notice-{n}-assignment-unsent"), "result"),
                Step::Field("editor", t.required.render()),
                Step::Click("run"),
                Step::Click("check"),
                Step::Assert("assignment-pass"),
            ]);
            if t.requires_restart {
                command(&mut p, "shutdown -c");
                p.push(Step::Click("clock-help"));
                p.push(Step::Click("advance-1"));
                command(&mut p, "shutdown -r +0029");
                command(&mut p, "wall");
                p.extend([Step::Click("check"), Step::Assert("assignment-pass")]);
            }
            p.extend([
                Step::Field("command", "cat ../../personal".into()),
                Step::Click("run"),
                Step::Assert("unsupported"),
                Step::Capture(format!("notice-{n}-unsupported"), "last-result"),
                Step::Save,
            ]);
            if n == 1 {
                p.extend([
                    Step::RememberOthers,
                    Step::Click("reset"),
                    Step::Click("confirm-reset"),
                    Step::Assert("reset"),
                    Step::Click("challenge"),
                    Step::Assert("challenge"),
                    Step::Capture(format!("notice-{n}-challenge"), "top"),
                ]);
            }
            p.extend([
                Step::Style(1.5, false),
                Step::Capture(format!("notice-{n}-dark150-editor"), "editor-viewport"),
                Step::Style(2., true),
                Step::Capture(format!("notice-{n}-light200-editor"), "editor-viewport"),
                Step::Capture(format!("notice-{n}-light200-command"), "command"),
                Step::Capture(format!("notice-{n}-light200-state"), "observations"),
                Step::Save,
            ]);
        }
        p
    })
}
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
    reveals: usize,
    others: Option<serde_json::Value>,
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::MaintenanceNotice { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::MaintenanceNotice { definition } = &app.scenarios[app.selected.unwrap()].goal
    else {
        panic!()
    };
    definition
}
fn practice(app: &LabApp) -> bool {
    !app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].challenge
        && session(app).view == teaching::View::Example
}
fn scope(app: &LabApp) -> &work::Draft {
    if practice(app) {
        &session(app).practice
    } else {
        &session(app).assignment
    }
}
fn task(app: &LabApp) -> &work::Task {
    if practice(app) {
        &definition(app).example
    } else {
        &definition(app).initial
    }
}
fn others(app: &LabApp) -> serde_json::Value {
    let id = &app.scenarios[app.selected.unwrap()].id;
    serde_json::to_value(
        app.progress
            .attempts
            .iter()
            .filter(|(key, _)| *key != id)
            .collect::<std::collections::BTreeMap<_, _>>(),
    )
    .unwrap()
}
fn observe(app: &LabApp, what: &str, flow: &Flow) {
    let i = app.selected.unwrap();
    let a = &app.progress.attempts[&app.scenarios[i].id];
    let d = definition(app);
    let s = session(app);
    let current = scope(app);
    let t = task(app);
    match what {
        "example-only" => {
            assert!(practice(app));
            assert_eq!(s.assignment, work::Draft::new(&d.initial).unwrap());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "practice-partial" => {
            assert!(practice(app) && current.checked && !current.solved(t));
            let checks = current.checks(t).unwrap();
            assert!(checks[0].correct);
            assert!(checks.iter().any(|c| !c.correct));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "practice-pass" => {
            assert!(practice(app) && current.solved(t));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "unsent" => {
            assert!(current.stdin_pending && current.checked && !current.solved(t));
        }
        "assignment-pass" => {
            assert!(!practice(app) && current.solved(t));
            assert!(progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-fail" => {
            assert!(!practice(app) && current.checked && !current.solved(t));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-unchecked" => {
            assert!(!current.checked);
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "fresh" => {
            assert_eq!(s.assignment, work::Draft::new(&d.initial).unwrap());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "unsupported" => {
            let error = current.last_output(t).unwrap().unwrap_err();
            assert_eq!(
                error.kind,
                dlms_learning_lab::maintenance_notice::command::Kind::Unsupported
            );
            assert!(current.solved(t));
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d).unwrap());
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert_eq!(s.assignment, work::Draft::new(&d.initial).unwrap());
        }
        _ => panic!("Unknown notice observation"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "last-result" => app
            .metrics
            .maintenance_notice
            .controls
            .iter()
            .find(|r| r.0 == "actual-run")
            .map(|r| r.1),
        "result" => {
            let m = &app.metrics.maintenance_notice.controls;
            let head = m.iter().find(|r| r.0 == "result")?.1;
            let fail = scope(app)
                .checks(task(app))
                .ok()?
                .iter()
                .position(|c| !c.correct);
            Some(
                fail.and_then(|n| {
                    m.iter()
                        .find(|r| r.0 == format!("feedback-{n}"))
                        .map(|r| head.union(r.1))
                })
                .unwrap_or(head),
            )
        }
        _ => app
            .metrics
            .maintenance_notice
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
impl LabApp {
    fn notice_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, rect);
        if !done {
            let viewport = self.metrics.main_viewport.expect("actual main viewport");
            let region = self
                .metrics
                .main_scroll_region
                .expect("actual main scroll area");
            let gutter = egui::Rect::from_min_max(
                egui::pos2(viewport.right(), viewport.top()),
                egui::pos2(region.right(), viewport.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(
                gutter.width() * ctx.pixels_per_point() > 1.,
                "Need an actual outer scrollbar gutter"
            );
            // A wheel inside the notice editor scrolls its text, not the lesson.
            // Use the actual parent gutter, retaining all existing guard limits.
            for event in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    fn notice_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual notice control");
        self.notice_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn notice_choice(&mut self, ctx: &egui::Context, key: &str, flow: &mut Flow) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (_, r, clip) = self
            .metrics
            .maintenance_notice
            .controls
            .iter()
            .find(|r| r.0 == key)
            .cloned()
            .expect("actual notice popup row");
        let clip = clip.intersect(ctx.content_rect());
        if !smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            flow.reveals += 1;
            assert!(flow.reveals < 20);
            self.pending_events.extend([
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ]);
            false
        } else {
            self.click_smoke_control(ctx, Some(r))
        }
    }
    pub(in crate::ui) fn maintenance_notice_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("maintenance-notice-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "MAINTENANCE_NOTICE_NATIVE_COMPLETE: {} actions",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 146000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Notice step {} {:?}: {due:?}",
            flow.at,
            plan()[flow.at]
        );
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(id, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match plan()[flow.at].clone() {
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
                assert_eq!((self.scenarios.len(), self.units.len()), (406, 156));
                self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
                assert_eq!(
                    definition(self),
                    fixture(IDS.iter().position(|v| *v == id).unwrap())
                );
                true
            }
            Step::Click(key) => self.notice_click(ctx, target(self, key)),
            Step::Choose(choice) => {
                if flow.phase == 0 && scope(self).selected == choice {
                    true
                } else if flow.phase == 0 {
                    if self.notice_click(ctx, target(self, "document")) {
                        flow.phase = 1;
                    }
                    false
                } else {
                    self.notice_choice(
                        ctx,
                        match choice {
                            work::Editor::Motd => "document-motd",
                            work::Editor::Issue => "document-issue",
                            work::Editor::Stdin => "document-stdin",
                        },
                        &mut flow,
                    )
                }
            }
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.notice_click(ctx, target(self, key)) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(
                        if key == "command" {
                            scope(self).command.as_str()
                        } else {
                            scope(self).text()
                        },
                        value
                    );
                    true
                }
            }
            Step::Assert(what) => {
                observe(self, what, &flow);
                true
            }
            Step::RememberOthers => {
                flow.others = Some(others(self));
                true
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                assert_eq!(
                    serde_json::to_value(progress::load(&self.path, &self.scenarios).unwrap())
                        .unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Step::KeyboardRun => {
                if flow.phase == 0 {
                    if self.notice_click(ctx, target(self, "command")) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.pending_events
                        .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        }));
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(scope(self).last_run.as_ref().unwrap().command, "wall");
                    assert!(!scope(self).stdin_pending);
                    true
                }
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, key).is_some_and(|r| self.notice_reveal(ctx, r)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, key).is_some_and(|r| {
                        smoke_control_visible(
                            r,
                            self.metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point(),
                        )
                    })
                {
                    flow.phase = 0;
                    false
                } else if flow.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    flow.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if done {
            println!(
                "MAINTENANCE_NOTICE_NATIVE_STEP {} {:?}",
                flow.at,
                plan()[flow.at]
            );
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
            flow.capture_pixels = None;
            flow.capture_reported = false;
            flow.reveals = 0;
            flow.next = now + 0.025;
        } else {
            flow.next = flow.next.max(now + 0.025);
        }
        ctx.data_mut(|d| d.insert_temp(id, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[cfg(test)]
#[test]
fn notice_capture_plan_exercises_actual_editors_commands_failures_and_selected_reset() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::RememberOthers))
            .count(),
        1
    );
    for token in [
        "practice-pass",
        "practice-partial",
        "assignment-unchecked",
        "assignment-fail",
        "assignment-pass",
        "unsupported",
        "reset",
        "challenge",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if*v==token)));
    }
    assert!(p.iter().any(|s| matches!(s, Step::KeyboardRun)));
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s,Step::Style(scale,true)if*scale==2.))
            .count(),
        2
    );
}
