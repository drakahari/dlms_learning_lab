//! Widget-driven checks of the three authored DNS lessons, not a learner engine.
use super::*;
use dlms_learning_lab::{dns_access::acl::Acl, dns_access_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-dns-recursion-boundary",
    "lpic2-dns-secondary-transfer-policy",
    "lpic2-dns-access-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Field(&'static str, String),
    Assert(&'static str),
    Capture(String, &'static str),
    Save,
    RememberOthers,
    KeyboardRun,
}
fn fixture(n: usize) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v20.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::DnsAccess { definition } = &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!()
    };
    definition
}
fn action(p: &mut Vec<Step>, a: &work::Action) {
    match a {
        work::Action::Save { config } => {
            p.extend([Step::Field("editor", config.0.clone()), Step::Click("save")])
        }
        work::Action::Reload {} => p.push(Step::Click("reload")),
        work::Action::Query {
            client,
            name,
            record_type,
            recursive,
        } => {
            assert_eq!(*record_type, dlms_learning_lab::dns::QueryType::A);
            assert!(*recursive);
            p.extend([
                Step::Field("query-client", client.to_string()),
                Step::Field("query-name", name.0.clone()),
                Step::Click("query"),
            ]);
        }
        work::Action::Transfer { client, zone } => p.extend([
            Step::Field("transfer-client", client.to_string()),
            Step::Field("transfer-zone", zone.0.clone()),
            Step::Click("transfer"),
        ]),
        work::Action::ClearCache {} => panic!("No unexposed fixture action"),
    }
}
fn reference(p: &mut Vec<Step>, t: &work::Task) {
    for a in &t.reference {
        action(p, a);
    }
}
fn saved(t: &work::Task) -> String {
    let work::Action::Save { config } = &t.reference[0] else {
        panic!()
    };
    config.0.clone()
}
fn bad_config(t: &work::Task) -> String {
    let text = saved(t);
    if let Acl::Networks { networks } = &t.required.cached_answers {
        let net = &networks[0];
        let base = u32::from(net.address);
        text.replace(
            &format!("{}/{};", net.address, net.prefix),
            &format!(
                "{}; {};",
                std::net::Ipv4Addr::from(base + 10),
                std::net::Ipv4Addr::from(base + 20)
            ),
        )
    } else {
        let peer = t.facts.zones[0].secondary_client;
        let changed = text.replace(&format!("{peer};"), "any;");
        assert_ne!(changed, text);
        changed
    }
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let d = fixture(n);
            let t = &d.initial;
            let e = &d.example;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("dns-{n}-introduction"), "top"),
                Step::Capture(format!("dns-{n}-worked"), "worked-value-0"),
                Step::Click("example"),
                Step::Capture(format!("dns-{n}-practice-start"), "editor-viewport"),
                Step::Click("fill"),
                Step::Assert("fill-inert"),
            ]);
            p.extend([
                Step::Field("editor", "options { unknown yes; };".into()),
                Step::Click("save"),
                Step::Assert("syntax"),
                Step::Capture(format!("dns-{n}-syntax-feedback"), "output"),
            ]);
            if n > 0 {
                action(
                    &mut p,
                    &work::Action::Transfer {
                        client: "203.0.113.220".parse().unwrap(),
                        zone: work::Name(e.facts.zones[0].server.primary.origin.clone()),
                    },
                );
                p.extend([
                    Step::Assert("other-client"),
                    Step::Capture(format!("dns-{n}-other-client-transfer"), "output"),
                ]);
            }
            reference(&mut p, e);
            p.extend([
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(format!("dns-{n}-practice-pass"), "result"),
                Step::Save,
            ]);
            // Native Return in the name field must perform the current query, not save or grade.
            let q = e
                .reference
                .iter()
                .find(|a| matches!(a, work::Action::Query { .. }))
                .unwrap();
            if let work::Action::Query { client, name, .. } = q {
                p.extend([
                    Step::Field("query-client", client.to_string()),
                    Step::Field("query-name", name.0.clone()),
                    Step::KeyboardRun,
                    Step::Assert("keyboard-query"),
                ]);
            }
            p.extend([
                Step::Field("editor", bad_config(e)),
                Step::Assert("unchecked"),
                Step::Click("save"),
                Step::Click("reload"),
                Step::Click("check"),
                Step::Assert("case-pass-overall-fail"),
                Step::Capture(format!("dns-{n}-scope-feedback"), "result"),
            ]);
            p.extend([
                Step::Click("reset-example"),
                Step::Assert("practice-reset"),
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("dns-{n}-assignment"), "editor-viewport"),
                Step::Click("check"),
                Step::Assert("assignment-fail"),
                Step::Capture(format!("dns-{n}-assignment-feedback"), "result"),
            ]);
            reference(&mut p, t);
            p.extend([
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(format!("dns-{n}-assignment-pass"), "result"),
                Step::Save,
            ]);
            p.extend([
                Step::Field("editor", format!("{}\n# pending draft", saved(t))),
                Step::Assert("unchecked"),
                Step::Save,
            ]);
            reference(&mut p, t);
            p.extend([
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Click("example"),
                Step::Field("editor", "# separate unfinished practice draft".into()),
                Step::Click("assignment"),
                Step::Assert("assignment-pass"),
                Step::Save,
            ]);
            p.extend([
                Step::Style(1.5, false),
                Step::Capture(format!("dns-{n}-150-editor"), "editor-viewport"),
                Step::Capture(format!("dns-{n}-150-actions"), "save"),
                Step::Style(2., true),
                Step::Capture(format!("dns-{n}-200-light-editor"), "editor-viewport"),
                Step::Capture(format!("dns-{n}-200-light-actions"), "save"),
                Step::Capture(format!("dns-{n}-200-light-query"), "query-client"),
                Step::Capture(format!("dns-{n}-200-light-result"), "result"),
                Step::Save,
                Step::Style(1., false),
                Step::RememberOthers,
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(format!("dns-{n}-challenge"), "editor-viewport"),
                Step::Click("learn"),
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
    let engine::State::DnsAccess { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::DnsAccess { definition } = &app.scenarios[app.selected.unwrap()].goal else {
        panic!()
    };
    definition
}
fn practice(app: &LabApp) -> bool {
    !app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].challenge
        && session(app).view == teaching::View::Example
}
fn scope(app: &LabApp) -> &work::session::Draft {
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
fn solved(d: &work::session::Draft, t: &work::Task) -> bool {
    d.checked
        && d.world(t)
            .is_ok_and(|f| d.editor.0 == f.saved_text() && t.grade(&f).iter().all(|c| c.correct))
}
fn observe(app: &LabApp, what: &str, flow: &Flow) {
    let i = app.selected.unwrap();
    let a = &app.progress.attempts[&app.scenarios[i].id];
    let s = session(app);
    let d = definition(app);
    let current = scope(app);
    let t = task(app);
    match what {
        "fill-inert" => {
            assert!(practice(app));
            assert_eq!(current.editor.0, saved(t));
            assert_eq!(current.history, t.starter);
            assert!(current.last.is_none());
            assert_eq!(s.assignment, work::session::Draft::new(&d.initial).unwrap());
        }
        "syntax" => {
            assert!(current.last_output(t).unwrap().unwrap().is_err());
            assert_eq!(current.history, t.starter);
        }
        "other-client" => {
            let out = current.last_output(t).unwrap().unwrap().unwrap();
            assert!(out.contains("not the supplied secondary"));
            assert_eq!(
                current.world(t).unwrap().zones()[0].server.secondary,
                t.facts.zones[0].server.secondary
            );
        }
        "practice-pass" => {
            assert!(practice(app) && solved(current, t));
            assert_eq!(s.assignment, work::session::Draft::new(&d.initial).unwrap());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "keyboard-query" => {
            assert!(matches!(
                current.last.as_ref().unwrap().action,
                work::Action::Query { .. }
            ));
            assert!(current.last_output(t).unwrap().unwrap().is_ok());
            assert!(solved(current, t));
        }
        "unchecked" => {
            assert!(!current.checked && !current.attempted);
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "case-pass-overall-fail" => {
            let rows = t.grade(&current.world(t).unwrap());
            assert!(
                rows.iter()
                    .filter(|r| r.label.starts_with("Query case"))
                    .all(|r| r.correct)
            );
            assert!(rows.iter().any(|r| !r.correct));
            assert!(current.attempted && !current.checked);
            assert!(
                app.metrics
                    .dns_access
                    .summary
                    .as_ref()
                    .unwrap()
                    .starts_with("Needs attention")
            );
        }
        "practice-reset" => {
            assert_eq!(s.practice, work::session::Draft::new(&d.example).unwrap());
            assert_eq!(s.assignment, work::session::Draft::new(&d.initial).unwrap());
        }
        "fresh" => {
            assert_eq!(s.assignment, work::session::Draft::new(&d.initial).unwrap());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-fail" => {
            assert!(!practice(app) && current.attempted && !solved(current, t));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-pass" => {
            assert!(!practice(app) && solved(current, t));
            assert!(progress::current_solved(a, &app.scenarios[i]));
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d).unwrap());
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(
                !app.metrics
                    .dns_access
                    .controls
                    .iter()
                    .any(|r| r.0 == "fill")
            );
        }
        _ => panic!("Unknown DNS observation"),
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
        "editor" => target(app, "editor-viewport"),
        "result" => {
            let m = &app.metrics.dns_access.controls;
            let head = m.iter().find(|r| r.0 == "result")?.1;
            let failed = task(app)
                .grade(&scope(app).world(task(app)).ok()?)
                .iter()
                .position(|r| !r.correct);
            Some(
                failed
                    .and_then(|n| {
                        m.iter()
                            .find(|r| r.0 == format!("feedback-{n}"))
                            .map(|r| head.union(r.1))
                    })
                    .unwrap_or(head),
            )
        }
        _ => app
            .metrics
            .dns_access
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                app.metrics
                    .dns_access
                    .fields
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn field_value<'a>(app: &'a LabApp, key: &str) -> &'a str {
    let d = scope(app);
    match key {
        "editor" => &d.editor.0,
        "query-client" => &d.query_client.0,
        "query-name" => &d.query_name.0,
        "transfer-client" => &d.transfer_client.0,
        "transfer-zone" => &d.transfer_zone.0,
        _ => panic!("Unknown DNS field"),
    }
}
impl LabApp {
    pub(in crate::ui) fn open_dns_access_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (439, 172));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered source lesson"),
        );
    }
    fn dns_access_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn dns_access_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual DNS access control");
        self.dns_access_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn dns_access_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("dns-access-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("DNS_ACCESS_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 151000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "DNS step {} {:?}: {due:?}",
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
                self.open_dns_access_capture(id);
                assert_eq!(
                    definition(self),
                    fixture(IDS.iter().position(|v| *v == id).unwrap())
                );
                true
            }
            Step::Click(key) => self.dns_access_click(ctx, target(self, key)),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.dns_access_click(ctx, target(self, key)) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(field_value(self, key), value);
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
                    if self.dns_access_click(ctx, target(self, "query-name")) {
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
                    assert!(matches!(
                        scope(self).last.as_ref().unwrap().action,
                        work::Action::Query { .. }
                    ));
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
                    if target(self, key).is_some_and(|r| self.dns_access_reveal(ctx, r)) {
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
            println!("DNS_ACCESS_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
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
fn dns_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("dns-access-native");
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
                app.dns_access_tick(ctx);
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
    panic!("Actual source capture phases did not complete");
}

#[cfg(test)]
#[test]
fn dns_capture_covers_three_real_lessons_failures_actual_queries_and_large_text() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 3);
    for token in [
        "fill-inert",
        "syntax",
        "other-client",
        "practice-pass",
        "case-pass-overall-fail",
        "assignment-fail",
        "assignment-pass",
        "unchecked",
        "reset",
        "challenge",
        "keyboard-query",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==token)));
    }
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Style(2., true)))
            .count(),
        3
    );
    for n in 0..3 {
        for t in [&fixture(n).initial, &fixture(n).example] {
            let mut w = t.initial().unwrap();
            for a in &t.reference {
                work::apply(&mut w, a).unwrap();
            }
            assert!(t.grade(&w).iter().all(|r| r.correct));
            w.save(&bad_config(t)).unwrap();
            w.reload();
            let rows = t.grade(&w);
            let cases = rows
                .iter()
                .filter(|r| r.label.starts_with("Query case"))
                .collect::<Vec<_>>();
            assert_eq!(cases.len(), t.cases.len());
            assert!(cases.iter().all(|r| r.correct));
            assert!(rows.iter().any(|r| !r.correct));
        }
    }
}
