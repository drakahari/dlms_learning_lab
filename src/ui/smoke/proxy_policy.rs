//! Fixed policy-editor workflow; actual native events and capture phases, synthetic profiles only.
use super::*;
use dlms_learning_lab::proxy_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-proxy-client-boundary", "lpic2-proxy-connect-order"];
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
            "../../../scenarios/boot-entry-lessons-v28.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::ProxyPolicy { definition } =
        &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!("MIB-monitor source")
    };
    definition
}
fn method(p: &mut Vec<Step>, t: &work::Task) {
    p.push(Step::Field("editor".into(), t.required_policy.clone()));
    for key in ["save", "syntax", "apply", "check"] {
        p.push(Step::Click(key));
    }
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = Vec::new();
        for (n, id) in IDS.iter().enumerate() {
            let tag = |name: &str| format!("proxy-{n}-{name}");
            let t = &fixture(n).initial;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(tag("introduction"), "top"),
                Step::Capture(tag("worked"), "try"),
                Step::Click("example"),
                Step::Capture(tag("practice"), "editor"),
                Step::Click("fill"),
                Step::Assert("filled-inert"),
                Step::Click("check"),
                Step::Assert("unsaved"),
                Step::Capture(tag("unsaved"), "check"),
            ]);
            for key in ["save", "syntax", "apply", "check"] {
                p.push(Step::Click(key));
            }
            p.extend([
                Step::Assert("practice-pass"),
                Step::Capture(tag("practice-result"), "check"),
                Step::Select(
                    "request".into(),
                    format!(
                        "request-{}",
                        if n == 0 {
                            "before-staff"
                        } else {
                            "unsafe-tunnel"
                        }
                    ),
                ),
                Step::Click("inspect"),
                Step::Capture(tag("decision"), "inspect"),
                Step::Click("assignment"),
                Step::Capture(tag("assignment"), "top"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(tag("initial-feedback"), "check"),
                Step::Field("editor".into(), "include /etc/passwd\n".into()),
                Step::Click("save"),
                Step::Click("syntax"),
                Step::Assert("syntax-error"),
                Step::Click("check"),
                Step::Capture(tag("syntax-error"), "check"),
                Step::Field("editor".into(), t.required_policy.clone()),
                Step::Click("save"),
                Step::Click("check"),
                Step::Assert("partial"),
                Step::Capture(tag("partial"), "check"),
                Step::Click("apply"),
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(tag("complete"), "check"),
            ]);
            let narrow = if n == 0 {
                t.required_policy
                    .replace("10.44.16.0/20", "10.44.16.1 10.44.31.254")
            } else {
                t.required_policy.replace("10.70.8.0/24", "10.70.8.12")
            };
            p.push(Step::Field("editor".into(), narrow));
            for key in ["save", "apply", "check"] {
                p.push(Step::Click(key));
            }
            p.extend([
                Step::Assert("hardcoded"),
                Step::Capture(tag("range-feedback"), "policy-saved"),
            ]);
            method(&mut p, t);
            p.extend([
                Step::Field("editor".into(), "http_access allow all\n".into()),
                Step::Assert("unchecked"),
                Step::Capture(tag("broken"), "check"),
                Step::RememberOthers,
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
                Step::Click("assignment"),
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(tag("challenge"), "editor"),
                Step::Click("learn"),
            ]);
            method(&mut p, t);
            for (scale, narrow, name) in [(1.5, false, "dark150"), (2., true, "light200")] {
                p.extend([
                    Step::Style(scale, narrow),
                    Step::Click("introduction"),
                    Step::Capture(tag(&format!("{name}-introduction")), "top"),
                    Step::Click("assignment"),
                    Step::Field("editor".into(), String::new()),
                    Step::Assert("empty-editor"),
                    Step::Capture(tag(&format!("{name}-empty-editor")), "editor"),
                    Step::Field("editor".into(), "acl staff src 10.0.0.0/33\n".into()),
                    Step::Click("save"),
                    Step::Click("syntax"),
                    Step::Assert("syntax-error"),
                    Step::Click("check"),
                    Step::Capture(tag(&format!("{name}-syntax-error")), "check"),
                ]);
                method(&mut p, t);
                p.extend([
                    Step::Assert("assignment-pass"),
                    Step::Capture(tag(&format!("{name}-check")), "check"),
                    Step::Capture(tag(&format!("{name}-requests")), "results"),
                    Step::Click("example"),
                    Step::Capture(tag(&format!("{name}-practice")), "editor"),
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
    let engine::State::ProxyPolicy { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::ProxyPolicy { definition } = &app.scenarios[app.selected.unwrap()].goal
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
    let s = session(app);
    let d = definition(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let a = &app.progress.attempts[&source.id];
    match what {
        "filled-inert" => {
            assert_eq!(s.practice.editor, d.example.required_policy);
            assert_eq!(s.practice.world.saved(), d.example.initial_config);
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
                    .proxy
                    .summary
                    .as_deref()
                    .unwrap()
                    .starts_with("All request")
            );
        }
        "unchecked" => {
            assert!(!scope(app).checked());
            assert!(!app.metrics.completion);
            assert!(
                app.metrics
                    .proxy
                    .summary
                    .as_deref()
                    .unwrap()
                    .contains("not checked")
            );
        }
        "fail" => {
            assert!(scope(app).checked());
            assert!(!task(app).report(scope(app)).unwrap().passed());
            assert!(!app.metrics.completion);
            assert!(app.metrics.proxy.cases.iter().any(|c| c.1));
            assert!(app.metrics.proxy.cases.iter().any(|c| !c.1));
        }
        "unsaved" => {
            assert!(scope(app).checked());
            assert!(
                app.metrics
                    .proxy
                    .feedback
                    .iter()
                    .any(|s| s.contains("unsaved"))
            );
            assert!(!app.metrics.completion);
        }
        "partial" => {
            let r = task(app).report(scope(app)).unwrap();
            assert!(!r.policies[0].passed && r.policies[1].passed);
            assert!(!app.metrics.completion);
        }
        "hardcoded" => {
            let r = task(app).report(scope(app)).unwrap();
            assert!(r.cases.iter().all(|c| c.passed));
            assert!(r.policies.iter().all(|p| !p.passed));
            assert!(!app.metrics.completion);
            assert!(
                app.metrics
                    .proxy
                    .feedback
                    .iter()
                    .any(|s| s.contains("whole required range"))
            );
        }
        "syntax-error" => {
            assert!(app.notice.as_ref().unwrap().contains("Line 1"));
            assert!(scope(app).validate_saved(task(app)).is_err());
        }
        "empty-editor" => {
            assert!(scope(app).editor.is_empty());
            assert!(!scope(app).checked());
            assert!(!app.metrics.completion);
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d).unwrap());
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(!app.metrics.proxy.controls.iter().any(|r| r.0 == "fill"));
            assert!(!app.metrics.proxy.worked_answer_visible);
        }
        _ => panic!("Unknown fixed proxy observation"),
    }
}
// Only the automation hit area is clipped. TextEdit keeps its full native
// geometry, cursor and scrolling; the shared visibility guard remains unchanged.
fn editor_hit_target(
    rect: egui::Rect,
    clip: egui::Rect,
    viewport: egui::Rect,
    line: f32,
) -> egui::Rect {
    let visible = rect.intersect(clip).intersect(viewport);
    if visible.is_positive() && visible.height() >= line {
        visible
    } else {
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), line))
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "editor" => app
            .metrics
            .proxy
            .controls
            .iter()
            .find(|r| r.0 == "editor")
            .map(|r| {
                editor_hit_target(
                    r.1,
                    r.2,
                    app.metrics.main_viewport.expect("actual main viewport"),
                    app.metrics.proxy.editor_line_height,
                )
            }),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        _ => app
            .metrics
            .proxy
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                app.metrics
                    .proxy
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn field_value(app: &LabApp, key: &str) -> String {
    assert_eq!(key, "editor");
    scope(app).editor.clone()
}
fn choice_value(app: &LabApp, key: &str) -> String {
    assert_eq!(key, "request");
    format!("request-{}", scope(app).selected_case)
}
impl LabApp {
    pub(in crate::ui) fn queue_proxy_policy_field_text(&mut self, value: &str) {
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
    pub(in crate::ui) fn open_proxy_policy_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (439, 172));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered proxy_policy lesson"),
        );
    }
    fn proxy_policy_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn proxy_policy_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual proxy_policy control");
        if let Some(clip) = popup {
            return smoke_control_visible(
                r,
                clip.intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            ) && self.click_smoke_control(ctx, Some(r));
        }
        self.proxy_policy_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn proxy_policy_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("proxy_policy-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("PROXY_POLICY_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 163000 + flow.at;
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
                self.open_proxy_policy_capture(id);
                assert_eq!(
                    serde_json::to_value(definition(self)).unwrap(),
                    serde_json::to_value(fixture(IDS.iter().position(|v| *v == id).unwrap()))
                        .unwrap()
                );
                true
            }
            Step::Click(key) => self.proxy_policy_click(
                ctx,
                target(self, key),
                self.metrics
                    .proxy
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.2),
            ),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.proxy_policy_click(ctx, target(self, &key), None) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_proxy_policy_field_text(&value);
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
                        if self.proxy_policy_click(ctx, target(self, &control), None) {
                            flow.phase = 1;
                        }
                        false
                    }
                } else {
                    let popup = self
                        .metrics
                        .proxy
                        .choices
                        .iter()
                        .find(|r| r.0 == choice)
                        .map(|r| r.2);
                    if self.proxy_policy_click(ctx, target(self, &choice), popup) {
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
                    if target(self, key).is_some_and(|r| self.proxy_policy_reveal(ctx, r)) {
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
            println!("PROXY_POLICY_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
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
fn proxy_policy_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("proxy_policy-native");
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
                app.proxy_policy_tick(ctx);
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
    panic!("Actual proxy_policy capture phases did not complete");
}

#[cfg(test)]
#[test]
fn proxy_policy_plan_covers_real_edit_save_apply_partial_hardcoded_reset_and_scaled_views() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Style(2., true)))
            .count(),
        2
    );
    for key in [
        "filled-inert",
        "practice-pass",
        "assignment-pass",
        "partial",
        "unchecked",
        "syntax-error",
        "fail",
        "reset",
        "challenge",
        "hardcoded",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==key)));
    }
    assert!(
        p.iter()
            .any(|s| matches!(s,Step::Select(_,v)if v=="request-unsafe-tunnel"))
    );
    println!(
        "PROXY_PLAN actions={} captures={}",
        p.len(),
        p.iter().filter(|s| matches!(s, Step::Capture(..))).count()
    );
}

#[cfg(test)]
#[test]
fn proxy_policy_short_window_editor_hit_area_is_visible_without_changing_the_field() {
    let rect = egui::Rect::from_min_max(egui::pos2(18., 78.5), egui::pos2(602., 304.1));
    let viewport = egui::Rect::from_min_max(egui::pos2(0., 116.), egui::pos2(634., 316.));
    let target = editor_hit_target(rect, viewport, viewport, 18.);
    assert_eq!(target, rect.intersect(viewport));
    assert!(smoke_control_visible(target, viewport, 2.));
    assert_eq!(rect.top(), 78.5); // The actual editor was never resized or moved.
    let offscreen = rect.translate(egui::vec2(0., 500.));
    let anchor = editor_hit_target(offscreen, viewport, viewport, 18.);
    assert_eq!(anchor.min, offscreen.min);
    assert_eq!(anchor.height(), 18.);
    assert!(!smoke_control_visible(anchor, viewport, 2.));
}
