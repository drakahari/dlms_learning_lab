//! Fixed virtual MIB/monitor native workflow. No learner command or host network operation.
use super::*;
use dlms_learning_lab::snmp_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = ["ccna-snmp-link-monitor", "ccna-snmp-counter-monitor"];
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
            "../../../scenarios/boot-entry-lessons-v27.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::SnmpMonitor { definition } =
        &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!("MIB-monitor source")
    };
    definition
}
fn method(p: &mut Vec<Step>, t: &work::Task) {
    let f = &t.reference;
    p.push(Step::Field("description".into(), f.description.clone()));
    match t.activity {
        work::Activity::Link => p.push(Step::Select(
            "rule".into(),
            format!("rule-{:?}", f.link_rule),
        )),
        work::Activity::Traffic => {
            p.push(Step::Select(
                "direction".into(),
                format!("direction-{:?}", f.direction),
            ));
            p.push(Step::Select("width".into(), format!("width-{:?}", f.width)));
            p.push(Step::Field("threshold".into(), f.threshold.clone()));
        }
    }
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = Vec::new();
        for (n, id) in IDS.iter().enumerate() {
            let tag = |name: &str| format!("mib-{n}-{name}");
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(tag("introduction"), "top"),
                Step::Capture(tag("worked"), "try"),
                Step::Click("example"),
                Step::Capture(tag("practice"), "query-oid"),
                Step::Click("names"),
                Step::Assert("query-not-run"),
                Step::Click("poll"),
                Step::Assert("query-name"),
                Step::Capture(tag("query"), "query-output"),
                Step::Field("query-oid".into(), "not-an-oid".into()),
                Step::Click("poll"),
                Step::Assert("query-error"),
                Step::Capture(tag("query-error"), "query-output"),
                Step::Click("names"),
                Step::Click("poll"),
                Step::Click("fill"),
                Step::Assert("filled-inert"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(tag("practice-result"), "result"),
                Step::Save,
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(tag("assignment"), "top"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(
                    tag("initial-feedback"),
                    if n == 0 {
                        "case-unexpected-down"
                    } else {
                        "case-busy"
                    },
                ),
            ]);
            if n == 0 {
                p.extend([
                    Step::Field(
                        "description".into(),
                        fixture(n).initial.reference.description.clone(),
                    ),
                    Step::Click("check"),
                    Step::Assert("partial"),
                    Step::Capture(tag("maintenance-feedback"), "case-maintenance"),
                ]);
            } else {
                p.extend([
                    Step::Field("threshold".into(), "abc".into()),
                    Step::Click("check"),
                    Step::Assert("format-error"),
                    Step::Capture(tag("configuration-feedback"), "configuration-feedback"),
                ]);
            }
            method(&mut p, &fixture(n).initial);
            p.extend([
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(tag("complete"), "result"),
                Step::Save,
            ]);
            if n == 1 {
                p.extend([
                    Step::Field("threshold".into(), "70".into()),
                    Step::Click("check"),
                    Step::Assert("assignment-pass"),
                    Step::Capture(tag("alternate-threshold"), "case-busy"),
                ]);
            }
            p.extend([
                Step::Click("names"),
                Step::Click("poll"),
                Step::Assert("assignment-pass"),
                Step::Field(
                    if n == 0 { "description" } else { "threshold" }.into(),
                    if n == 0 { "visitor-uplink" } else { "99" }.into(),
                ),
                Step::Assert("unchecked"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(
                    tag("broken"),
                    if n == 0 {
                        "case-unexpected-down"
                    } else {
                        "case-busy"
                    },
                ),
                Step::Save,
                Step::RememberOthers,
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(tag("challenge"), "query-oid"),
                Step::Click("learn"),
            ]);
            for (scale, light, name) in [(1.5, false, "dark150"), (2., true, "light200")] {
                p.extend([
                    Step::Style(scale, light),
                    Step::Click("introduction"),
                    Step::Capture(tag(&format!("{name}-introduction")), "top"),
                    Step::Click("assignment"),
                    Step::Field("query-oid".into(), "".into()),
                    Step::Capture(tag(&format!("{name}-placeholder")), "query-oid"),
                    Step::Click("poll"),
                    Step::Assert("query-error"),
                    Step::Capture(tag(&format!("{name}-query-error")), "query-output"),
                    Step::Click("names"),
                    Step::Click("poll"),
                    Step::Capture(tag(&format!("{name}-query")), "query-output"),
                ]);
                method(&mut p, &fixture(n).initial);
                p.extend([
                    Step::Click("check"),
                    Step::Assert("assignment-pass"),
                    Step::Capture(tag(&format!("{name}-check")), "check"),
                    Step::Capture(
                        tag(&format!("{name}-case")),
                        if n == 0 {
                            "case-unexpected-down"
                        } else {
                            "case-busy"
                        },
                    ),
                    Step::Click("example"),
                    Step::Capture(tag(&format!("{name}-practice")), "query-oid"),
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
    let engine::State::SnmpMonitor { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::SnmpMonitor { definition } = &app.scenarios[app.selected.unwrap()].goal
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
            assert_eq!(s.practice.fields, d.example.reference);
            assert!(!s.practice.checked());
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, source));
        }
        "practice-pass" => {
            assert!(practice(app) && scope(app).solved(task(app)));
            assert!(!progress::current_solved(a, source));
            assert_eq!(app.metrics.snmp.summary.as_deref(), Some("pass"));
        }
        "assignment-pass" => {
            assert!(!practice(app));
            assert!(progress::current_solved(a, source) && app.metrics.completion);
            assert_eq!(app.metrics.snmp.summary.as_deref(), Some("pass"));
        }
        "unchecked" => {
            assert!(!scope(app).checked());
            assert_eq!(app.metrics.snmp.summary.as_deref(), Some("unchecked"));
            assert!(!app.metrics.completion);
        }
        "fail" => {
            assert!(scope(app).checked());
            assert!(!task(app).report(&scope(app).fields).unwrap().passed());
            assert!(!app.metrics.completion);
            assert_eq!(app.metrics.snmp.summary.as_deref(), Some("fail"));
        }
        "format-error" => {
            assert!(scope(app).checked());
            assert!(
                task(app)
                    .report(&scope(app).fields)
                    .unwrap()
                    .configuration_error
                    .is_some()
            );
            assert!(!app.metrics.completion);
        }
        "partial" => {
            let r = task(app).report(&scope(app).fields).unwrap();
            assert!(r.cases.iter().any(|c| c.passed) && r.cases.iter().any(|c| !c.passed));
            assert!(!app.metrics.completion);
        }
        "query-not-run" => assert!(scope(app).observed(task(app)).is_none()),
        "query-name" => {
            let r = scope(app).observed(task(app)).unwrap().unwrap();
            let dlms_learning_lab::snmp::Reply::Value {
                value: dlms_learning_lab::snmp::Value::Text(text),
                ..
            } = r
            else {
                panic!("actual name response")
            };
            assert_eq!(text, task(app).reference.description);
        }
        "query-error" => {
            assert!(scope(app).observed(task(app)).unwrap().is_err());
            assert!(
                app.metrics
                    .snmp
                    .query_text
                    .as_ref()
                    .unwrap()
                    .starts_with("Query error:")
            );
        }
        "fresh" => {
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, source));
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d));
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(!app.metrics.snmp.controls.iter().any(|r| r.0 == "fill"));
            assert!(!app.metrics.snmp.worked_answer_visible);
        }
        _ => panic!("Unknown fixed MIB-monitor observation"),
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
            .snmp
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                app.metrics
                    .snmp
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn field_value(app: &LabApp, key: &str) -> String {
    let d = scope(app);
    match key {
        "description" => d.fields.description.clone(),
        "threshold" => d.fields.threshold.clone(),
        "query-oid" => d.query.oid.clone(),
        _ => panic!("Unknown fixed MIB field {key}"),
    }
}
fn choice_value(app: &LabApp, key: &str) -> String {
    let d = scope(app);
    match key {
        "rule" => format!("rule-{:?}", d.fields.link_rule),
        "direction" => format!("direction-{:?}", d.fields.direction),
        "width" => format!("width-{:?}", d.fields.width),
        "operation" => format!("operation-{}", d.query.next),
        "case" => format!("case-{}", d.query.case),
        "sample" => format!("sample-{}", d.query.before),
        _ => panic!("Unknown fixed MIB choice {key}"),
    }
}
impl LabApp {
    pub(in crate::ui) fn queue_snmp_monitor_field_text(&mut self, value: &str) {
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
    pub(in crate::ui) fn open_snmp_monitor_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (439, 172));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered snmp_monitor lesson"),
        );
    }
    fn snmp_monitor_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn snmp_monitor_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual snmp_monitor control");
        if let Some(clip) = popup {
            return smoke_control_visible(
                r,
                clip.intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            ) && self.click_smoke_control(ctx, Some(r));
        }
        self.snmp_monitor_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn snmp_monitor_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("snmp_monitor-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("SNMP_MONITOR_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 162000 + flow.at;
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
                self.open_snmp_monitor_capture(id);
                assert_eq!(
                    serde_json::to_value(definition(self)).unwrap(),
                    serde_json::to_value(fixture(IDS.iter().position(|v| *v == id).unwrap()))
                        .unwrap()
                );
                true
            }
            Step::Click(key) => self.snmp_monitor_click(
                ctx,
                target(self, key),
                self.metrics
                    .snmp
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.2),
            ),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.snmp_monitor_click(ctx, target(self, &key), None) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_snmp_monitor_field_text(&value);
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
                        if self.snmp_monitor_click(ctx, target(self, &control), None) {
                            flow.phase = 1;
                        }
                        false
                    }
                } else {
                    let popup = self
                        .metrics
                        .snmp
                        .choices
                        .iter()
                        .find(|r| r.0 == choice)
                        .map(|r| r.2);
                    if self.snmp_monitor_click(ctx, target(self, &choice), popup) {
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
                    if target(self, key).is_some_and(|r| self.snmp_monitor_reveal(ctx, r)) {
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
            println!("SNMP_MONITOR_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
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
fn snmp_monitor_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("snmp_monitor-native");
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
                app.snmp_monitor_tick(ctx);
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
    panic!("Actual snmp_monitor capture phases did not complete");
}

#[cfg(test)]
#[test]
fn snmp_monitor_plan_covers_real_mib_queries_alternate_partial_reset_and_scaled_views() {
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
        "format-error",
        "fail",
        "reset",
        "challenge",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==key)));
    }
    assert!(
        p.iter()
            .any(|s| matches!(s,Step::Select(_,v)if v=="width-Counter64"))
    );
    println!(
        "MIB_PLAN actions={} captures={}",
        p.len(),
        p.iter().filter(|s| matches!(s, Step::Capture(..))).count()
    );
}
