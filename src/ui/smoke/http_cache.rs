//! Fixed policy-editor workflow; actual native events and capture phases, synthetic profiles only.
use super::*;
use dlms_learning_lab::http_cache_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-cache-shared-privacy",
    "lpic2-cache-freshness-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Field(String, String),
    Select(String, String),
    Assert(&'static str),
    Capture(String, &'static str),
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v29.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::HttpCache { definition } = &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!("cache source")
    };
    definition
}
fn method(p: &mut Vec<Step>, t: &work::Task) {
    for (i, h) in t.reference_headers().iter().enumerate() {
        p.push(Step::Field(format!("header-{i}"), h.value.clone()));
    }
    p.push(Step::Click("check"));
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = Vec::new();
        for (n, id) in IDS.iter().enumerate() {
            let tag = |v: &str| format!("cache-{n}-{v}");
            let t = &fixture(n).initial;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::RememberOthers,
                Step::Capture(tag("introduction"), "top"),
                Step::Capture(tag("worked"), "try"),
                Step::Click("example"),
                Step::Capture(tag("practice"), "header-0"),
                Step::Click("fill"),
                Step::Assert("filled-inert"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(tag("practice-result"), "summary"),
            ]);
            p.push(Step::Field(
                if n == 0 { "header-1" } else { "header-0" }.into(),
                if n == 0 {
                    "no-cache, max-age=45"
                } else {
                    "max-age=900, s-maxage=60"
                }
                .into(),
            ));
            p.extend([
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(tag("practice-mistake"), "summary"),
                Step::Click("fill"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Click("assignment"),
                Step::Capture(tag("assignment"), "top"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(tag("initial-feedback"), "summary"),
                Step::Field("header-0".into(), "max-age=wrong".into()),
                Step::Click("check"),
                Step::Assert("syntax"),
                Step::Capture(tag("syntax"), "summary"),
                Step::Field("header-0".into(), "immutable".into()),
                Step::Click("check"),
                Step::Assert("unsupported"),
                Step::Capture(tag("unsupported"), "summary"),
            ]);
            if n == 0 {
                p.extend([
                    Step::Field("header-0".into(), "max-age=30".into()),
                    Step::Click("check"),
                    Step::Assert("partial"),
                    Step::Capture(tag("partial"), "summary"),
                    Step::Select("trace".into(), "trace-public-repeat".into()),
                    Step::Assert("selected-pass-global-fail"),
                    Step::Capture(tag("passing-trace-overall-fail"), "summary"),
                ]);
            } else {
                p.extend([
                    Step::Field("header-0".into(), "max-age=100, s-maxage=40".into()),
                    Step::Click("check"),
                    Step::Assert("header-requirement"),
                    Step::Capture(tag("header-requirement"), "summary"),
                ]);
            }
            method(&mut p, t);
            p.extend([
                Step::Assert("assignment-pass"),
                Step::Capture(tag("complete"), "summary"),
                Step::Field("header-0".into(), "no-store".into()),
                Step::Assert("unchecked"),
                Step::Capture(tag("edited"), "summary"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(tag("broken"), "summary"),
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(tag("challenge"), "header-0"),
                Step::Click("learn"),
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
            ]);
            for (scale, narrow, name) in [(1.5, false, "dark150"), (2., true, "light200")] {
                p.extend([
                    Step::Style(scale, narrow),
                    Step::Click("introduction"),
                    Step::Capture(tag(&format!("{name}-introduction")), "top"),
                    Step::Click("assignment"),
                    Step::Field("header-0".into(), String::new()),
                    Step::Assert("empty"),
                    Step::Capture(tag(&format!("{name}-empty")), "header-0"),
                    Step::Field("header-0".into(), "max-age=wrong".into()),
                    Step::Click("check"),
                    Step::Assert("syntax"),
                    Step::Capture(tag(&format!("{name}-syntax")), "summary"),
                ]);
                method(&mut p, t);
                p.extend([
                    Step::Assert("assignment-pass"),
                    Step::Capture(tag(&format!("{name}-check")), "summary"),
                    Step::Capture(tag(&format!("{name}-requests")), "request-0"),
                    Step::Click("example"),
                    Step::Capture(tag(&format!("{name}-practice")), "header-0"),
                    Step::Save,
                    Step::Click("introduction"),
                ]);
            }
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
fn session(app: &LabApp) -> &work::Session {
    let engine::State::HttpCache { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::HttpCache { definition } = &app.scenarios[app.selected.unwrap()].goal else {
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
    let s = session(app);
    let d = definition(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let a = &app.progress.attempts[&source.id];
    match what {
        "filled-inert" => {
            assert_eq!(s.practice.headers, d.example.reference_headers());
            assert!(!s.practice.checked());
            assert_eq!(s.assignment, work::Draft::new(&d.initial).unwrap());
            assert!(!progress::current_solved(a, source));
        }
        "practice-pass" => {
            assert!(practice(app) && task(app).report(scope(app)).unwrap().passed());
            assert!(!progress::current_solved(a, source));
        }
        "assignment-pass" => {
            assert!(!practice(app));
            assert!(progress::current_solved(a, source) && app.metrics.completion);
            assert!(
                app.metrics
                    .http_cache
                    .summary
                    .as_deref()
                    .unwrap()
                    .starts_with("All declared")
            );
        }
        "unchecked" => {
            assert!(!scope(app).checked());
            assert!(!app.metrics.completion);
            assert!(app.metrics.http_cache.cases.is_empty());
        }
        "fail" => {
            assert!(scope(app).checked());
            assert!(!task(app).report(scope(app)).unwrap().passed());
            assert!(!app.metrics.completion);
        }
        "partial" => {
            let r = task(app).report(scope(app)).unwrap();
            assert!(!r.passed());
            assert!(r.traces[0].steps.iter().all(|s| s.passed));
            assert!(r.traces[1].steps.iter().any(|s| !s.passed));
            assert!(!app.metrics.completion);
        }
        "selected-pass-global-fail" => {
            assert!(app.metrics.http_cache.cases.iter().all(|c| c.1));
            assert!(
                app.metrics
                    .http_cache
                    .controls
                    .iter()
                    .any(|c| c.0 == "failed-two-people")
            );
            assert!(
                app.metrics
                    .http_cache
                    .summary
                    .as_deref()
                    .unwrap()
                    .contains("traces fail")
            );
            assert!(!app.metrics.completion);
        }
        "syntax" => assert!(
            app.metrics
                .http_cache
                .feedback
                .iter()
                .any(|s| s.contains("Syntax"))
        ),
        "unsupported" => assert!(
            app.metrics
                .http_cache
                .feedback
                .iter()
                .any(|s| s.contains("Unsupported"))
        ),
        "header-requirement" => assert!(
            app.metrics
                .http_cache
                .feedback
                .iter()
                .any(|s| s.contains("must retain") && s.contains("max-age=600"))
        ),
        "empty" => {
            assert!(scope(app).headers[0].value.is_empty());
            assert!(!scope(app).checked());
            assert!(!app.metrics.completion);
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d).unwrap());
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(
                !app.metrics
                    .http_cache
                    .controls
                    .iter()
                    .any(|c| c.0 == "fill")
            );
            assert!(!app.metrics.http_cache.worked_answer_visible);
        }
        _ => panic!("Unknown fixed cache observation {what}"),
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
        "learn" => app.metrics.mode_learn,
        _ => app
            .metrics
            .http_cache
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                app.metrics
                    .http_cache
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn field_value(app: &LabApp, key: &str) -> String {
    let i = key
        .strip_prefix("header-")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    scope(app).headers[i].value.clone()
}
fn choice_value(app: &LabApp, key: &str) -> String {
    assert_eq!(key, "trace");
    format!("trace-{}", scope(app).selected_trace)
}
impl LabApp {
    pub(in crate::ui) fn queue_http_cache_field_text(&mut self, value: &str) {
        self.case_lesson_type(value);
        if value.is_empty() {
            assert!(
                matches!(self.pending_events.pop(), Some(egui::Event::Text(text)) if text.is_empty())
            );
            self.pending_events
                .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                    key: egui::Key::Delete,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                }));
        }
    }
    pub(in crate::ui) fn open_http_cache_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (439, 172));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered http_cache lesson"),
        );
    }
    fn http_cache_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
            // A wheel inside nested output scrolls its text, not the lesson.
            // Use the actual parent gutter, retaining all existing guard limits.
            for event in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    fn http_cache_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual http_cache control");
        if let Some(clip) = popup {
            return smoke_control_visible(
                r,
                clip.intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            ) && self.click_smoke_control(ctx, Some(r));
        }
        self.http_cache_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn http_cache_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("http_cache-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("HTTP_CACHE_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 164000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Source step {} {:?}: {due:?}",
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
                self.open_http_cache_capture(id);
                assert_eq!(
                    serde_json::to_value(definition(self)).unwrap(),
                    serde_json::to_value(fixture(IDS.iter().position(|v| *v == id).unwrap()))
                        .unwrap()
                );
                true
            }
            Step::Click(key) => self.http_cache_click(
                ctx,
                target(self, key),
                self.metrics
                    .http_cache
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.2),
            ),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.http_cache_click(ctx, target(self, &key), None) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_http_cache_field_text(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(field_value(self, &key), value);
                    true
                }
            }
            Step::Select(control, choice) => {
                if flow.phase == 0 {
                    if choice_value(self, &control) == choice {
                        true
                    } else {
                        if self.http_cache_click(ctx, target(self, &control), None) {
                            flow.phase = 1;
                        }
                        false
                    }
                } else {
                    let popup = self
                        .metrics
                        .http_cache
                        .choices
                        .iter()
                        .find(|r| r.0 == choice)
                        .map(|r| r.2);
                    if self.http_cache_click(ctx, target(self, &choice), popup) {
                        assert_eq!(choice_value(self, &control), choice);
                        true
                    } else {
                        false
                    }
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
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, key).is_some_and(|r| self.http_cache_reveal(ctx, r)) {
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
            println!("HTTP_CACHE_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
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
fn http_cache_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("http_cache-native");
    let mut flow = Flow {
        at,
        ..Default::default()
    };
    flow.next = 0.;
    ctx.data_mut(|d| d.insert_temp(id, flow));
    let mut requested = false;
    for frame in 0..12 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920., 1080.),
                )),
                time: Some(frame as f64 + 1.),
                ..Default::default()
            },
            |ctx| {
                app.render(ctx);
                app.http_cache_tick(ctx);
            },
        );
        requested |= output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        });
        let flow = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap();
        if flow.phase == 2 {
            assert!(requested && app.capture_pending);
            app.capture_pending = false;
        }
        if flow.at > at {
            assert_eq!(flow.at, at + 1);
            assert!(requested);
            return;
        }
    }
    panic!("Actual http_cache capture phases did not complete");
}

#[cfg(test)]
#[test]
fn http_cache_plan_covers_real_headers_practice_failures_reset_and_scaled_views() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Style(2., true)))
            .count(),
        2
    );
    for expected in [
        "filled-inert",
        "practice-pass",
        "assignment-pass",
        "fail",
        "partial",
        "selected-pass-global-fail",
        "syntax",
        "unsupported",
        "header-requirement",
        "unchecked",
        "reset",
        "challenge",
        "empty",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==expected)));
    }
    println!(
        "CACHE_PLAN actions={} captures={}",
        p.len(),
        p.iter().filter(|s| matches!(s, Step::Capture(..))).count()
    );
}
