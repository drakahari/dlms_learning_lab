//! Fixed mixed-family repair and prerequisite-teaching checks; all input virtual.
use super::*;
use dlms_learning_lab::host_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-host-address",
    "lpic1-host-routing",
    "lpic1-host-mixed-ip",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Field(String, String),
    Assert(&'static str),
    Capture(String, &'static str),
    Save,
    RememberOthers,
}
fn fixture(id: &str) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        let mut old = pack::accepted436().unwrap();
        old.extend(
            serde_json::from_str::<pack::Pack>(include_str!(
                "../../../tests/fixtures/mixed-ip-preview-v1.json"
            ))
            .unwrap()
            .scenarios,
        );
        old
    });
    let engine::Goal::HostNetwork { definition } = &all.iter().find(|s| s.id == id).unwrap().goal
    else {
        panic!()
    };
    definition
}
fn method(p: &mut Vec<Step>, commands: &[String]) {
    for cmd in commands {
        p.push(Step::Field("command".into(), cmd.clone()));
        p.push(Step::Click("run"));
    }
}
fn assignment_commands() -> Vec<String> {
    let data: pack::Pack = serde_json::from_str(include_str!(
        "../../../tests/fixtures/mixed-ip-preview-v1.json"
    ))
    .unwrap();
    data.scenarios[0]
        .solution
        .iter()
        .map(|s| {
            let e: work::Event =
                serde_json::from_str(s.strip_prefix("host-action ").unwrap()).unwrap();
            let work::Event::Command { input } = e else {
                panic!()
            };
            input
        })
        .collect()
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = Vec::new();
        for (id, label) in [(IDS[0], "address"), (IDS[1], "routing")] {
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Assert("introduced"),
                Step::Capture(format!("foundations-{label}-introduction"), "top"),
                Step::Capture(format!("foundations-{label}-worked"), "try"),
            ]);
        }
        let id = IDS[2];
        let d = fixture(id);
        let cmds = assignment_commands();
        p.extend([
            Step::Style(1., false),
            Step::Open(id),
            Step::Capture("mixed-ip-introduction".into(), "top"),
            Step::Capture("mixed-ip-worked".into(), "try"),
            Step::Click("example"),
            Step::Capture("mixed-ip-practice".into(), "command"),
            Step::Click("fill-example"),
            Step::Assert("insert-inert"),
        ]);
        let example = d
            .example_actions
            .iter()
            .map(|e| {
                let work::Event::Command { input } = e else {
                    panic!()
                };
                input.clone()
            })
            .collect::<Vec<_>>();
        method(&mut p, &example);
        p.extend([
            Step::Click("check"),
            Step::Assert("practice-pass"),
            Step::Capture("mixed-ip-practice-results".into(), "results"),
            Step::Click("assignment"),
            Step::Capture("mixed-ip-assignment".into(), "top"),
            Step::Click("check"),
            Step::Assert("fail"),
            Step::Capture("mixed-ip-initial-feedback".into(), "results"),
        ]);
        method(&mut p, &cmds[..3]);
        p.extend([
            Step::Click("check"),
            Step::Assert("partial"),
            Step::Capture("mixed-ip-partial-family".into(), "results"),
        ]);
        method(&mut p, &cmds[3..]);
        p.extend([
            Step::Click("check"),
            Step::Assert("assignment-pass"),
            Step::Capture("mixed-ip-complete".into(), "results"),
        ]);
        for (address, observe) in [
            ("203.0.113.80", "probe4"),
            ("2001:db8:80::80", "probe6"),
            ("not-an-address", "bad-probe"),
        ] {
            p.extend([
                Step::Field("probe".into(), address.into()),
                Step::Click("probe-run"),
                Step::Assert(observe),
            ]);
        }
        method(
            &mut p,
            &["ip -6 route replace default via fe80::1 dev ens4 metric 100".into()],
        );
        p.extend([
            Step::Assert("unchecked"),
            Step::Click("check"),
            Step::Assert("partial"),
            Step::Capture("mixed-ip-broken-ipv6".into(), "results"),
        ]);
        method(&mut p, &[cmds[5].clone()]);
        p.extend([
            Step::Click("check"),
            Step::Assert("assignment-pass"),
            Step::RememberOthers,
            Step::Click("reset"),
            Step::Click("confirm-reset"),
            Step::Assert("reset"),
            Step::Click("assignment"),
            Step::Click("challenge"),
            Step::Assert("challenge"),
            Step::Capture("mixed-ip-challenge".into(), "command"),
            Step::Click("learn"),
        ]);
        method(&mut p, &cmds);
        p.extend([Step::Click("check"), Step::Assert("assignment-pass")]);
        for (scale, light, name) in [(1.5, false, "dark150"), (2., true, "light200-wide")] {
            p.extend([
                Step::Style(scale, light),
                Step::Click("introduction"),
                Step::Capture(format!("mixed-ip-{name}-introduction"), "top"),
                Step::Click("assignment"),
                Step::Capture(format!("mixed-ip-{name}-command"), "command"),
                Step::Capture(format!("mixed-ip-{name}-results"), "results"),
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
    others: Option<serde_json::Value>,
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::HostNetwork { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::HostNetwork { definition } = &app.scenarios[app.selected.unwrap()].goal
    else {
        panic!()
    };
    definition
}
fn practice(app: &LabApp) -> bool {
    !app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].challenge
        && session(app).view == teaching::View::Example
}
fn scope(app: &LabApp) -> &work::Scope {
    if practice(app) {
        &session(app).practice
    } else {
        &session(app).assignment
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
        "introduced" => {
            assert!(!progress::current_solved(a, source));
            assert!(!app.metrics.host.fields.iter().any(|r| r.0 == "command"));
            let e = lesson_presentation::entry(source).unwrap();
            assert_eq!(e.concepts.len(), 4);
        }
        "insert-inert" => {
            assert_eq!(s.practice.world, d.example);
            assert_eq!(s.assignment.world, d.initial);
            assert!(!s.practice.command.is_empty());
            assert!(!progress::current_solved(a, source));
        }
        "practice-pass" => {
            assert!(practice(app) && d.checks(s, true).iter().all(|r| r.0));
            assert!(!progress::current_solved(a, source));
        }
        "assignment-pass" => {
            assert!(
                !practice(app) && progress::current_solved(a, source) && app.metrics.completion
            );
            assert_eq!(app.metrics.host.result.unwrap().1, 0);
        }
        "partial" => {
            assert!(!app.metrics.completion);
            let (n, failed, _) = app.metrics.host.result.unwrap();
            assert!(failed > 0 && failed < n);
        }
        "fail" => {
            assert!(!progress::current_solved(a, source) && !app.metrics.completion);
            assert!(app.metrics.host.result.unwrap().1 > 0);
        }
        "unchecked" => {
            assert!(!scope(app).checked && !app.metrics.completion);
        }
        "probe4" => assert!(scope(app).probe_result.starts_with("IPv4 path available")),
        "probe6" => assert!(scope(app).probe_result.starts_with("IPv6 path available")),
        "bad-probe" => assert!(
            scope(app)
                .probe_result
                .contains("Enter a literal IPv4 or IPv6")
        ),
        "reset" => {
            assert_eq!(*s, work::Session::new(d));
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(
                !app.metrics
                    .host
                    .controls
                    .iter()
                    .any(|r| r.0 == "fill-example")
            );
        }
        _ => panic!("Unknown mixed-IP observation {what}"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "results" => app.metrics.host.result.map(|r| r.2),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        _ => app
            .metrics
            .host
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn field_value(app: &LabApp, key: &str) -> String {
    match key {
        "command" => scope(app).command.clone(),
        "probe" => scope(app).probe_ipv6_input.clone(),
        _ => panic!("Unknown mixed-IP field"),
    }
}
impl LabApp {
    pub(in crate::ui) fn queue_mixed_ip_field_text(&mut self, value: &str) {
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
    pub(in crate::ui) fn open_mixed_ip_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (441, 173));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered mixed_ip lesson"),
        );
    }
    fn mixed_ip_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn mixed_ip_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual mixed_ip control");
        if let Some(clip) = popup {
            return smoke_control_visible(
                r,
                clip.intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            ) && self.click_smoke_control(ctx, Some(r));
        }
        self.mixed_ip_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn mixed_ip_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("mixed_ip-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "MIXED_IP_GEOMETRY_SCOPE: wide native only; current compositor narrow resizing remains unverified"
            );
            println!("MIXED_IP_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 165000 + flow.at;
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
            Step::Style(scale, light) => {
                if flow.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.lesson_scroll_reset = true;
                    flow.phase = 1;
                    false
                } else {
                    let viewport = ctx.content_rect();
                    let settled = flow.viewport == Some(viewport)
                        && (ctx.zoom_factor() - scale).abs() < 0.01
                        && ctx.input(|i| i.viewport().fullscreen) == Some(true);
                    flow.viewport = Some(viewport);
                    settled
                }
            }
            Step::Open(id) => {
                self.open_mixed_ip_capture(id);
                assert_eq!(
                    serde_json::to_value(definition(self)).unwrap(),
                    serde_json::to_value(fixture(id)).unwrap()
                );
                true
            }
            Step::Click(key) => self.mixed_ip_click(ctx, target(self, key), None),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.mixed_ip_click(ctx, target(self, &key), None) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_mixed_ip_field_text(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(field_value(self, &key), value);
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
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, key).is_some_and(|r| self.mixed_ip_reveal(ctx, r)) {
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
            println!("MIXED_IP_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
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
fn mixed_ip_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[2]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(name, "top") if name=="mixed-ip-introduction"))
        .unwrap();
    let id = egui::Id::new("mixed_ip-native");
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
                app.mixed_ip_tick(ctx);
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
    panic!("Actual mixed_ip capture phases did not complete");
}

#[cfg(test)]
#[test]
fn mixed_ip_plan_keeps_both_family_repairs_full_tasks_and_explicit_wide_scope() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 3);
    for required in [
        "introduced",
        "insert-inert",
        "practice-pass",
        "assignment-pass",
        "partial",
        "probe4",
        "probe6",
        "bad-probe",
        "unchecked",
        "reset",
        "challenge",
    ] {
        assert!(
            p.iter().any(|s| matches!(s,Step::Assert(v)if *v==required)),
            "{required}"
        );
    }
    assert!(p.iter().any(|s| matches!(s, Step::Style(1.5, false))));
    assert!(p.iter().any(|s| matches!(s, Step::Style(2., true))));
    for family in [
        "ip -4 address",
        "ip -6 address",
        "ip -4 route",
        "ip -6 route",
    ] {
        assert!(
            p.iter()
                .any(|s| matches!(s,Step::Field(k,v)if k=="command"&&v.starts_with(family)))
        );
    }
    println!(
        "MIXED_IP_PLAN actions={} captures={}",
        p.len(),
        p.iter().filter(|s| matches!(s, Step::Capture(..))).count()
    );
}
