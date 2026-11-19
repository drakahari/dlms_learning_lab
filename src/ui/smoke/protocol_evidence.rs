//! Fixed source-bound protocol investigation through actual native controls.
use super::*;
use dlms_learning_lab::transport_workspace as work;
use std::sync::OnceLock;
const ID: &str = "lpic1-protocol-round-trips";
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
fn fixture(_: &str) -> &'static work::Definition {
    static DATA: OnceLock<work::Definition> = OnceLock::new();
    DATA.get_or_init(|| {
        let p: pack::Pack = serde_json::from_str(include_str!(
            "../../../tests/fixtures/protocol-evidence-preview-v1.json"
        ))
        .unwrap();
        let engine::Goal::Transport { definition } = p.scenarios.into_iter().next().unwrap().goal
        else {
            panic!()
        };
        *definition
    })
}
fn policy(practice: bool, kind: &str) -> String {
    let d = fixture(ID);
    let t = if practice { &d.example } else { &d.assignment };
    match kind {
  "worked"=>if practice {&d.worked_example}else{&d.worked_assignment}.documents.routers["R1"].clone(),
  "echo-only"=>{let mut text=work::Draft::new(t).unwrap().documents.routers["R1"].clone();text=text.replace("20 deny icmp any any",&format!("20 permit icmp host {} host {}",t.transport.hosts[1].address,t.transport.hosts[0].address));text},
  "broken-echo"=>{let mut text=if practice {&d.worked_example}else{&d.worked_assignment}.documents.routers["R1"].clone();text=text.replace(&format!("20 permit icmp host {} host {}",t.transport.hosts[1].address,t.transport.hosts[0].address),"20 deny icmp any any");text},
  "permit-all"=>"ip access-list extended TRAFFIC\n 10 permit ip any any\nexit\ninterface office\n ip access-group TRAFFIC in\n ip access-group TRAFFIC out\nexit\n".into(),
  "alternate"=>"ip access-list extended TRAFFIC\n 10 permit tcp any any\n 20 permit icmp any any\n 30 deny udp any any\n 40 deny ip any any\nexit\ninterface office\n ip access-group TRAFFIC in\n ip access-group TRAFFIC out\nexit\n".into(),
  _=>panic!("Unknown fixed policy kind")
 }
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = vec![
            Step::Style(1., false),
            Step::Open(ID),
            Step::Capture("protocol-evidence-introduction".into(), "top"),
            Step::Capture("protocol-evidence-worked".into(), "try"),
            Step::Click("example"),
            Step::Capture("protocol-evidence-practice".into(), "editor"),
            Step::Click("check"),
            Step::Assert("fail"),
            Step::Capture("protocol-evidence-starter-feedback".into(), "results"),
            Step::Field("config-R1".into(), policy(true, "echo-only")),
            Step::Assert("unchecked"),
            Step::Click("check"),
            Step::Assert("partial"),
            Step::Capture("protocol-evidence-echo-only".into(), "cases"),
            Step::Click("fill-example"),
            Step::Assert("fill-inert"),
            Step::Click("check"),
            Step::Assert("practice-pass"),
            Step::Capture("protocol-evidence-practice-complete".into(), "results"),
            Step::Save,
            Step::Click("assignment"),
            Step::Capture("protocol-evidence-assignment".into(), "top"),
            Step::Capture("protocol-evidence-assignment-editor".into(), "editor"),
            Step::Click("check"),
            Step::Assert("fail"),
            Step::Field("config-R1".into(), policy(false, "worked")),
            Step::Assert("unchecked"),
            Step::Click("check"),
            Step::Assert("assignment-pass"),
            Step::Capture("protocol-evidence-complete".into(), "results"),
            Step::Capture("protocol-evidence-completion-actions".into(), "unit-back"),
            Step::Click("echo-client-echo"),
            Step::Capture(
                "protocol-evidence-actual-echo-fields".into(),
                "echo-client-echo",
            ),
            Step::Save,
            Step::Field("config-R1".into(), policy(false, "broken-echo")),
            Step::Assert("unchecked"),
            Step::Click("check"),
            Step::Assert("broken-echo"),
            Step::Capture("protocol-evidence-return-filtered".into(), "cases"),
            Step::Field("config-R1".into(), policy(false, "permit-all")),
            Step::Click("check"),
            Step::Assert("broad"),
            Step::Capture("protocol-evidence-broad-policy".into(), "cases"),
            Step::Field("config-R1".into(), policy(false, "alternate")),
            Step::Click("check"),
            Step::Assert("assignment-pass"),
            Step::Save,
            Step::Click("challenge"),
            Step::Assert("challenge"),
            Step::Capture("protocol-evidence-challenge".into(), "top"),
            Step::Click("learn"),
            Step::RememberOthers,
            Step::Click("reset"),
            Step::Click("confirm-reset"),
            Step::Assert("reset"),
            Step::Capture("protocol-evidence-reset".into(), "top"),
            Step::Click("assignment"),
            Step::Field("config-R1".into(), policy(false, "worked")),
            Step::Click("check"),
            Step::Assert("assignment-pass"),
            Step::Save,
        ];
        for (scale, light, name) in [(1.5, false, "dark150"), (2., true, "light200-wide")] {
            p.extend([
                Step::Style(scale, light),
                Step::Click("introduction"),
                Step::Capture(format!("protocol-evidence-{name}-introduction"), "top"),
                Step::Click("assignment"),
                Step::Capture(format!("protocol-evidence-{name}-editor"), "editor"),
                Step::Field(
                    "config-R1".into(),
                    policy(false, if light { "worked" } else { "alternate" }),
                ),
                Step::Assert("unchecked"),
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(format!("protocol-evidence-{name}-results"), "results"),
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
    let engine::State::Transport { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::Transport { definition } = &app.scenarios[app.selected.unwrap()].goal else {
        panic!()
    };
    definition
}
fn practice(app: &LabApp) -> bool {
    !app.progress.attempts[ID].challenge && session(app).view == teaching::View::Example
}
fn scope(app: &LabApp) -> &work::Scope {
    if practice(app) {
        &session(app).practice
    } else {
        &session(app).assignment
    }
}
fn others(app: &LabApp) -> serde_json::Value {
    serde_json::to_value(
        app.progress
            .attempts
            .iter()
            .filter(|(id, _)| id.as_str() != ID)
            .collect::<std::collections::BTreeMap<_, _>>(),
    )
    .unwrap()
}
fn observe(app: &LabApp, what: &str, flow: &Flow) {
    let d = definition(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let attempt = &app.progress.attempts[ID];
    match what {
        "unchecked" => {
            assert!(scope(app).tested.is_none());
            assert!(app.metrics.transport.result.is_none() && app.metrics.unit_complete.is_none());
        }
        "fill-inert" => {
            assert!(scope(app).tested.is_none());
            assert_eq!(
                session(app).assignment,
                work::Scope::new(&d.assignment).unwrap()
            );
            assert!(!progress::current_solved(attempt, source));
        }
        "practice-pass" => {
            assert!(practice(app) && scope(app).checks(&d.example).iter().all(|r| r.0));
            assert!(!progress::current_solved(attempt, source));
        }
        "assignment-pass" => {
            assert!(
                !practice(app)
                    && progress::current_solved(attempt, source)
                    && app.metrics.unit_complete.is_some()
                    && app.metrics.unit_back.is_some()
            );
            assert_eq!(
                app.metrics.transport.result.map(|r| (r.0, r.1)),
                Some((5, 5))
            );
        }
        "partial" => {
            assert_eq!(
                app.metrics.transport.result.map(|r| (r.0, r.1)),
                Some((2, 5))
            );
            assert!(
                !progress::current_solved(attempt, source) && app.metrics.unit_complete.is_none()
            );
        }
        "fail" => {
            let r = app.metrics.transport.result.unwrap();
            assert!(
                r.0 < r.1
                    && !progress::current_solved(attempt, source)
                    && app.metrics.unit_complete.is_none()
            );
        }
        "broken-echo" => {
            assert_eq!(
                app.metrics.transport.result.map(|r| (r.0, r.1)),
                Some((3, 5))
            );
            assert!(!progress::current_solved(attempt, source));
        }
        "broad" => {
            assert_eq!(
                app.metrics.transport.result.map(|r| (r.0, r.1)),
                Some((4, 5))
            );
            assert!(!progress::current_solved(attempt, source));
        }
        "challenge" => {
            assert!(attempt.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(
                !app.metrics
                    .transport
                    .controls
                    .iter()
                    .any(|r| r.0 == "fill-example")
            );
        }
        "reset" => {
            assert_eq!(*session(app), work::Session::new(d).unwrap());
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        _ => panic!("Unknown fixed protocol observation {what}"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "results" => app.metrics.transport.result.map(|r| r.2),
        "cases" => app.metrics.transport.case_feedback,
        "unit-back" => app.metrics.unit_back,
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        "editor" | "config-R1" => app.metrics.transport.editor_region.and_then(|rect| {
            (rect.width() > 8. && rect.height() > 8.).then(|| {
                egui::Rect::from_min_size(
                    rect.min + egui::vec2(3., 3.),
                    egui::vec2((rect.width() - 6.).min(200.), (rect.height() - 6.).min(24.)),
                )
            })
        }),
        _ => app
            .metrics
            .transport
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn field_value(app: &LabApp, key: &str) -> String {
    assert_eq!(key, "config-R1");
    scope(app).draft.documents.routers["R1"].clone()
}
impl LabApp {
    pub(in crate::ui) fn queue_protocol_evidence_field_text(&mut self, value: &str) {
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
    pub(in crate::ui) fn open_protocol_evidence_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (441, 173));
        assert_eq!(id, ID);
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered protocol_evidence lesson"),
        );
    }
    fn protocol_evidence_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn protocol_evidence_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual protocol_evidence control");
        if let Some(clip) = popup {
            return smoke_control_visible(
                r,
                clip.intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            ) && self.click_smoke_control(ctx, Some(r));
        }
        self.protocol_evidence_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn protocol_evidence_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("protocol_evidence-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "PROTOCOL_EVIDENCE_GEOMETRY_SCOPE: wide native only; current compositor narrow resizing remains unverified"
            );
            println!(
                "PROTOCOL_EVIDENCE_NATIVE_COMPLETE: {} actions",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 166000 + flow.at;
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
                self.open_protocol_evidence_capture(id);
                assert_eq!(
                    serde_json::to_value(definition(self)).unwrap(),
                    serde_json::to_value(fixture(id)).unwrap()
                );
                true
            }
            Step::Click(key) => self.protocol_evidence_click(ctx, target(self, key), None),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.protocol_evidence_click(ctx, target(self, &key), None) {
                        let expected = self
                            .metrics
                            .transport
                            .fields
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap()
                            .2;
                        assert_eq!(ctx.memory(|m| m.focused()), Some(expected));
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_protocol_evidence_field_text(&value);
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
                    if target(self, key).is_some_and(|r| self.protocol_evidence_reveal(ctx, r)) {
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
                "PROTOCOL_EVIDENCE_NATIVE_STEP {} {:?}",
                flow.at,
                plan()[flow.at]
            );
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
fn protocol_evidence_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == ID).unwrap());
    let at = plan()
        .iter()
        .position(
            |s| matches!(s, Step::Capture(name, "top") if name=="protocol-evidence-introduction"),
        )
        .unwrap();
    let id = egui::Id::new("protocol_evidence-native");
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
                app.protocol_evidence_tick(ctx);
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
    panic!("Actual protocol_evidence capture phases did not complete");
}

#[cfg(test)]
#[test]
fn protocol_evidence_plan_covers_partial_wrong_alternate_reset_and_large_editor_input() {
    for name in [
        "partial",
        "practice-pass",
        "assignment-pass",
        "broken-echo",
        "broad",
        "reset",
        "challenge",
        "fill-inert",
    ] {
        assert!(
            plan()
                .iter()
                .any(|s| matches!(s,Step::Assert(v)if *v==name))
        );
    }
    assert_eq!(
        plan().iter().filter(|s| matches!(s, Step::Open(_))).count(),
        1
    );
    assert!(plan().iter().any(|s| matches!(s, Step::Style(1.5, false))));
    assert!(plan().iter().any(|s| matches!(s, Step::Style(2., true))));
    println!(
        "PROTOCOL_EVIDENCE_PLAN actions={} captures={}",
        plan().len(),
        plan()
            .iter()
            .filter(|s| matches!(s, Step::Capture(..)))
            .count()
    );
}

#[cfg(test)]
#[test]
fn protocol_evidence_editor_capture_reveals_an_offscreen_editor_at_a_realistic_large_text_viewport()
{
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    app.scale = 1.5;
    ctx.set_zoom_factor(1.5);
    app.launch(app.scenarios.iter().position(|s| s.id == ID).unwrap());
    let engine::State::Transport { simulation } =
        &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    // RawInput uses logical points: reproduce1920×1080physical at150% text.
    // Settle fonts, zoom, scroll reset and the newly selected lesson view.
    for warmup in 0..3 {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920. / 1.5, 1080. / 1.5),
                )),
                time: Some(warmup as f64 * 0.1),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
    }
    let at = plan()
        .iter()
        .position(
            |s| matches!(s,Step::Capture(name,"editor")if name=="protocol-evidence-dark150-editor"),
        )
        .unwrap();
    let key = egui::Id::new("protocol_evidence-native");
    ctx.data_mut(|d| {
        d.insert_temp(
            key,
            Flow {
                at,
                ..Default::default()
            },
        )
    });
    let mut requested = false;
    let mut was_offscreen = false;
    for frame in 0..120 {
        let events = std::mem::take(&mut app.pending_events);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920. / 1.5, 1080. / 1.5),
                )),
                time: Some(frame as f64 * 0.1 + 1.),
                events,
                ..Default::default()
            },
            |ctx| {
                app.render(ctx);
                if frame == 0 {
                    let rect =
                        target(&app, "editor").expect("Offscreen geometry must remain navigable");
                    println!(
                        "EDITOR_REVEAL_INITIAL region={:?} target={:?} viewport={:?} zoom={}",
                        app.metrics.transport.editor_region,
                        rect,
                        app.metrics.main_viewport,
                        ctx.zoom_factor()
                    );
                    was_offscreen = !smoke_control_visible(
                        rect,
                        app.metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect()),
                        ctx.pixels_per_point(),
                    );
                }
                app.protocol_evidence_tick(ctx);
            },
        );
        requested |= output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        });
        let flow = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap();
        if flow.phase == 2 {
            assert!(requested && app.capture_pending);
            app.capture_pending = false;
        }
        if flow.at > at {
            assert!(requested && was_offscreen);
            assert_eq!(flow.at, at + 1);
            return;
        }
    }
    panic!("Bounded editor reveal/capture did not complete");
}
