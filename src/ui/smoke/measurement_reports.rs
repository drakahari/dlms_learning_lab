//! Fixed native scripts: actual editor/Run/Stop events, never installed grades.
use super::*;
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "shell-counter-rates",
    "shell-cpu-share",
    "shell-measurement-recovery",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Edit(&'static str),
    Wait(&'static str),
    Assert(&'static str),
    Capture(String, &'static str),
    Save,
    KeyboardRun,
    Home,
    RememberOthers,
}
fn fixture(index: usize) -> &'static serde_json::Value {
    static F: OnceLock<Vec<serde_json::Value>> = OnceLock::new();
    &F.get_or_init(|| {
        [
            include_str!("../../../tests/fixtures/shell-measurement-rates-reference.json"),
            include_str!("../../../tests/fixtures/shell-measurement-cpu-reference.json"),
            include_str!("../../../tests/fixtures/shell-measurement-recovery-reference.json"),
        ]
        .into_iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
    })[index]
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.into_iter().enumerate() {
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("measure-{n}-introduction"), "top"),
                Step::Capture(format!("measure-{n}-worked"), "intro-end"),
                Step::Click("example"),
                Step::Capture(format!("measure-{n}-practice-initial"), "editor"),
                Step::Click("run"),
                Step::Wait("example-pass"),
                Step::Capture(format!("measure-{n}-practice-result"), "output-top"),
                Step::Edit("experiment"),
                Step::Assert("example-unchecked"),
                Step::Click("run"),
                Step::Wait("example-changed"),
                Step::Capture(format!("measure-{n}-practice-experiment"), "output-top"),
                Step::Edit("example"),
                Step::Click("run"),
                Step::Wait("example-pass"),
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("measure-{n}-assignment"), "goal"),
                Step::Edit("wrong"),
                Step::Click("run"),
                Step::Wait("partial"),
                Step::Capture(format!("measure-{n}-partial"), "feedback"),
                Step::Click(if n == 0 {
                    "case-7"
                } else if n == 1 {
                    "case-1"
                } else {
                    "case-0"
                }),
                Step::Assert("selected-pass-overall-fail"),
                Step::Capture(
                    format!("measure-{n}-individual-pass-overall-fail"),
                    "case-status",
                ),
                Step::Edit("alternate"),
                Step::KeyboardRun,
                Step::Wait("assignment-pass"),
                Step::Capture(format!("measure-{n}-alternate-pass"), "feedback"),
                Step::Save,
                Step::Edit("loop"),
                Step::Assert("assignment-unchecked"),
                Step::Click("run"),
                Step::Assert("running"),
                Step::Click("stop"),
                Step::Assert("stopped"),
                Step::Edit("reference"),
                Step::Click("run"),
                Step::Wait("assignment-pass"),
                Step::Save,
            ]);
            if n == 2 {
                p.extend([
                    Step::RememberOthers,
                    Step::Click("reset"),
                    Step::Click("confirm-reset"),
                    Step::Assert("reset"),
                    Step::Click("challenge"),
                    Step::Assert("challenge"),
                    Step::Capture("measure-2-challenge".into(), "editor"),
                    Step::Click("learn"),
                    Step::Click("assignment"),
                    Step::Edit("alternate"),
                    Step::Click("run"),
                    Step::Wait("assignment-pass"),
                    Step::Save,
                ]);
            }
            p.extend([
                Step::Style(1.5, false),
                Step::Home,
                Step::Capture(format!("measure-{n}-dark150-editor"), "editor"),
                Step::Style(2., true),
                Step::Home,
                Step::Capture(format!("measure-{n}-light200-editor"), "editor"),
                Step::Capture(format!("measure-{n}-light200-case"), "case-status"),
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
    others: Option<serde_json::Value>,
}
fn selected(app: &LabApp) -> usize {
    let id = &app.scenarios[app.selected.unwrap()].id;
    IDS.iter().position(|v| *v == id).unwrap()
}
fn source_text(app: &LabApp, kind: &str) -> String {
    let n = selected(app);
    let f = fixture(n);
    let text = f[if kind == "alternate" {
        "alternate"
    } else if matches!(kind, "example" | "experiment") {
        "example_source"
    } else {
        "source"
    }]
    .as_str()
    .unwrap()
    .to_owned();
    match kind {
        "reference" | "alternate" | "example" => text,
        "loop" => "while true; do :; done\n".into(),
        "experiment" => match n {
            0 => text.replace(" * 1000 / elapsed", " / elapsed"),
            1 => text.replace("1|2|3|6|7)", "1|2|3|5|6|7)"),
            _ => text.replace(
                "if test \"$warnings\" -gt 0; then exit 1; fi",
                "if test \"$warnings\" -gt 0; then exit 0; fi",
            ),
        },
        "wrong" => match n {
            0 => text.replace(" * 1000 / elapsed", " / elapsed"),
            1 => text.replace("1|2|3|6|7)", "1|2|3)"),
            _ => text.replace(
                "if test \"$warnings\" -gt 0; then exit 1; fi",
                "if test \"$warnings\" -gt 0; then :; fi",
            ),
        },
        _ => panic!(),
    }
}
fn program(app: &LabApp) -> &str {
    let s = &app.scenarios[app.selected.unwrap()];
    let a = &app.progress.attempts[&s.id];
    if app.script_view(app.selected.unwrap()) == teaching::View::Example {
        &a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source
    } else {
        let engine::State::Script { source, .. } = &a.state else {
            panic!()
        };
        source
    }
}
fn other_attempts(app: &LabApp) -> serde_json::Value {
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
    let index = app.selected.unwrap();
    let s = &app.scenarios[index];
    let a = &app.progress.attempts[&s.id];
    let lesson = s.teaching.as_ref().unwrap();
    let learning = a
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    match what {
        "example-pass" | "example-changed" => {
            assert!(app.script_job.is_none());
            let result = learning.example_result.as_ref().unwrap();
            assert!(result.error.is_none());
            assert_eq!(lesson.fixture.accepts(result), what == "example-pass");
            assert!(!progress::current_solved(a, s));
        }
        "example-unchecked" => {
            assert!(learning.example_result.is_none());
            assert_eq!(a.state, s.initial);
            assert!(!progress::current_solved(a, s));
        }
        "fresh" => {
            assert_eq!(a.state, s.initial);
            assert!(!progress::current_solved(a, s));
        }
        "partial" => {
            let checks = engine::checks(&a.state, &s.goal);
            assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
            assert!(!progress::current_solved(a, s));
        }
        "selected-pass-overall-fail" => {
            assert!(!progress::current_solved(a, s));
            assert!(app.metrics.script_case_status.as_ref().unwrap().1);
        }
        "assignment-pass" => assert!(progress::current_solved(a, s)),
        "assignment-unchecked" => {
            let engine::State::Script { run, .. } = &a.state else {
                panic!()
            };
            assert!(run.is_none());
            assert!(!progress::current_solved(a, s));
        }
        "running" => assert!(app.script_job.is_some()),
        "stopped" => {
            assert!(app.script_job.is_none());
            assert!(!progress::current_solved(a, s));
            assert!(
                app.notice
                    .as_ref()
                    .is_some_and(|n| n.to_ascii_lowercase().contains("stop"))
            );
        }
        "reset" => {
            assert_eq!(a.state, s.initial);
            assert_eq!(learning.example_source, lesson.example);
            assert_eq!(other_attempts(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge);
            assert!(app.metrics.lesson_tabs.is_empty());
            assert_eq!(a.state, s.initial);
        }
        _ => panic!("Unknown observation"),
    }
}
fn target(app: &LabApp, ctx: &egui::Context, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "intro-end" => app.metrics.assignment_action,
        "example" => app.metrics.lesson_tabs.get(1).copied(),
        "assignment" => app.metrics.lesson_tabs.get(2).copied(),
        "editor" => app.metrics.editor_viewport,
        "run" => app.metrics.check,
        "stop" => app.metrics.stop,
        "goal" => app.metrics.script_goal,
        "feedback" => app.metrics.script_feedback,
        "case-status" => app.metrics.script_case_status.map(|r| r.2),
        "output-top" => app.metrics.output.map(|r| {
            egui::Rect::from_min_size(
                r.min,
                egui::vec2(r.width(), 2. * ctx.style().spacing.interact_size.y),
            )
        }),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        _ => key
            .strip_prefix("case-")
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| {
                app.metrics
                    .script_case_rows
                    .iter()
                    .find(|r| r.0 == n)
                    .map(|r| r.1)
            }),
    }
}
impl LabApp {
    fn measurement_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, r);
        if !done {
            let v = self.metrics.main_viewport.unwrap();
            let region = self.metrics.main_scroll_region.unwrap();
            let gutter = egui::Rect::from_min_max(
                egui::pos2(v.right(), v.top()),
                egui::pos2(region.right(), v.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(gutter.width() * ctx.pixels_per_point() > 1.);
            for e in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = e {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    pub(in crate::ui) fn measurement_reports_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("measurement-reports-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "MEASUREMENT_REPORTS_NATIVE_COMPLETE: {} actions",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 138000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Measurement step {} {:?}: {due:?}",
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
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
                true
            }
            Step::Click(key) => self.click_visible_smoke_control(ctx, target(self, ctx, key)),
            Step::Edit(kind) => {
                let text = source_text(self, kind);
                if flow.phase == 0 {
                    let r = target(self, ctx, "editor").unwrap();
                    if self.measurement_reveal(ctx, r) && self.case_lesson_focus(ctx) {
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&text);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(program(self), text);
                    true
                }
            }
            Step::Wait(what) => {
                if self.case_lesson_wait(ctx) {
                    observe(self, what, &flow);
                    true
                } else {
                    false
                }
            }
            Step::Assert(what) => {
                observe(self, what, &flow);
                true
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, ctx, key).is_some_and(|r| self.measurement_reveal(ctx, r)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, ctx, key).is_some_and(|r| {
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
            Step::KeyboardRun | Step::Home => {
                if flow.phase == 0 {
                    let r = target(self, ctx, "editor").unwrap();
                    if self.measurement_reveal(ctx, r) && self.case_lesson_focus(ctx) {
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    let key = if matches!(plan()[flow.at], Step::Home) {
                        egui::Key::Home
                    } else {
                        egui::Key::Enter
                    };
                    self.pending_events
                        .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: egui::Modifiers::CTRL,
                        }));
                    flow.phase = 2;
                    false
                } else {
                    true
                }
            }
            Step::RememberOthers => {
                flow.others = Some(other_attempts(self));
                true
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let saved = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(saved).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
        };
        if done {
            println!(
                "MEASUREMENT_REPORTS_NATIVE_STEP {} {:?}",
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
fn all_three_editor_flows_cover_guided_errors_independent_alternates_and_real_stop() {
    let p = plan();
    for id in IDS {
        assert_eq!(
            p.iter()
                .filter(|s| matches!(s,Step::Open(v)if *v==id))
                .count(),
            1
        );
    }
    assert_eq!(
        p.iter().filter(|s| matches!(s, Step::KeyboardRun)).count(),
        3
    );
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Assert("stopped")))
            .count(),
        3
    );
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Wait("example-changed")))
            .count(),
        3
    );
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Assert("selected-pass-overall-fail")))
            .count(),
        3
    );
    assert!(p.iter().any(|s| matches!(s, Step::Assert("reset"))));
}

#[cfg(test)]
#[test]
fn capture_phase_rechecks_late_clipping_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "editor")))
        .unwrap();
    let id = egui::Id::new("measurement-reports-native");
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    app.metrics.main_viewport = Some(clip);
    app.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    app.metrics.editor_viewport = Some(clip.translate(egui::vec2(0., 400.)));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500., 400.),
            )),
            ..Default::default()
        },
        |ctx| app.measurement_reports_tick(ctx),
    );
    let flow = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap();
    assert_eq!(flow.phase, 0);
    assert_eq!(flow.at, at);
    assert!(!app.capture_pending);
    app.metrics.editor_viewport = Some(egui::Rect::from_min_size(clip.min, egui::vec2(200., 100.)));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500., 400.),
            )),
            ..Default::default()
        },
        |ctx| app.measurement_reports_tick(ctx),
    );
    assert!(app.capture_pending);
    assert_eq!(ctx.data(|d| d.get_temp::<Flow>(id)).unwrap().phase, 2);
}
