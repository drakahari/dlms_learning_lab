//! Fixed native service exposure flows; private simulated work only.
use super::*;
use dlms_learning_lab::{service_exposure_source as binding, service_exposure_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    ColdFinish,
    Open(String),
    Click(String),
    Choose(String, String),
    Text(String, String),
    Check(bool, bool),
    Uncredited,
    Save,
    NotRun(bool),
    Probe(
        i32,
        Option<dlms_learning_lab::service_exposure::PortState>,
        bool,
    ),
    Client(
        Option<dlms_learning_lab::telnet_client::Mode>,
        Option<i32>,
        bool,
    ),
    Style(f32, bool),
    Narrow,
    Shot(String, String),
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    next: f64,
    began: Option<Instant>,
    polls: usize,
    popup_reveals: usize,
    requested: Option<egui::Vec2>,
    stable: u8,
}

fn run(p: &mut Vec<Op>, key: &str, text: &str) {
    p.extend([
        Op::Text(format!("{key}-input"), text.into()),
        Op::Click(format!("{key}-run")),
    ]);
}
pub(super) fn translate(p: &mut Vec<Op>, a: &work::Action) {
    match a {
        work::Action::Event { event } => match event {
            work::Event::Admin { input } => run(p, "admin", input),
            work::Event::Scan { input }
            | work::Event::ProbeV2 { input }
            | work::Event::ObserverV3 { input } => run(p, "scan", input),
            work::Event::TelnetEscapeV3 => p.push(Op::Click("scan-escape".into())),
            work::Event::Select { viewpoint } => {
                p.push(Op::Choose("observer-choice".into(), viewpoint.to_string()))
            }
            work::Event::Check => p.push(Op::Click("check".into())),
        },
        work::Action::AdminInput { text } => p.push(Op::Text("admin-input".into(), text.clone())),
        work::Action::ScanInput { text } => p.push(Op::Text("scan-input".into(), text.clone())),
    }
}
fn active_plan(labs: &[pack::Scenario]) -> &'static Vec<Op> {
    static P: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![Op::Style(1., false)];
        for (number, id) in [
            "lpic2-exposure-client-policy",
            "lpic2-exposure-listener-retirement",
        ]
        .iter()
        .enumerate()
        {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(source).unwrap();
            let shot = |name: &str, key: &str| {
                Op::Shot(
                    format!("service-exposure-{}-{name}", number + 1),
                    key.into(),
                )
            };
            p.extend([
                Op::Open(source.id.clone()),
                shot("introduction", "introduction-content"),
                shot("worked", "worked"),
                Op::Click("example".into()),
                shot("practice-server", "admin-input"),
                shot("practice-observer", "scan-input"),
                Op::NotRun(true),
                Op::Click("check".into()),
                Op::Check(false, true),
            ]);
            use dlms_learning_lab::service_exposure::PortState;
            for (command, status, state, label) in [
                (
                    "nc -n -v -z 192.0.2.10 443",
                    0,
                    Some(PortState::Open),
                    "nc-open",
                ),
                (
                    "nc -nvz 192.0.2.10 80",
                    1,
                    Some(PortState::Closed),
                    "nc-refused",
                ),
                (
                    "nc -zvn 192.0.2.10 22",
                    if number == 0 { 1 } else { 0 },
                    Some(if number == 0 {
                        PortState::Filtered
                    } else {
                        PortState::Open
                    }),
                    "nc-observer-policy",
                ),
                ("nc -lnvz 192.0.2.10 443", 1, None, "nc-unsupported"),
            ] {
                run(&mut p, "scan", command);
                p.push(Op::Probe(status, state, true));
                p.push(shot(label, "scan-status"));
                p.push(Op::Uncredited);
            }
            use dlms_learning_lab::telnet_client::Mode;
            run(&mut p, "scan", "telnet -c 192.0.2.10 443");
            p.extend([
                Op::Client(Some(Mode::Connected { port: 443 }), None, true),
                shot("telnet-live", "scan-escape"),
                Op::Uncredited,
                Op::Click("scan-escape".into()),
                Op::Client(Some(Mode::Command { port: 443 }), None, true),
                shot("telnet-local", "scan-input"),
            ]);
            run(&mut p, "scan", "quit");
            p.extend([
                Op::Client(None, Some(0), true),
                shot("telnet-exited", "scan-status"),
                Op::Uncredited,
            ]);
            run(&mut p, "scan", "telnet -c 192.0.2.10 80");
            p.extend([
                Op::Probe(1, Some(PortState::Closed), true),
                Op::Client(None, Some(1), true),
                shot("telnet-refused", "scan-status"),
            ]);
            run(&mut p, "scan", "telnet -c 192.0.2.10 23");
            p.extend([
                Op::Probe(1, None, true),
                Op::Client(None, Some(1), true),
                shot("telnet-unsupported-default", "scan-status"),
            ]);
            for a in &d.worked {
                translate(&mut p, a);
            }
            p.extend([
                Op::Check(true, true),
                Op::Uncredited,
                shot("example-passed", "result"),
                Op::Click("assignment".into()),
                shot("assignment", "assignment-content"),
                Op::Click("check".into()),
                Op::Check(false, false),
                shot("initial-feedback", "failed-case"),
            ]);
            run(&mut p, "admin", "ss -ltnp");
            run(
                &mut p,
                "scan",
                "nmap -n -Pn -sT -p 22,80,443,9000 198.51.100.20",
            );
            p.push(shot("actual-observation", "scan-output"));
            for raw in &source.solution {
                let envelope: serde_json::Value =
                    serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                let a = serde_json::from_value(envelope["action"].clone()).unwrap();
                translate(&mut p, &a);
            }
            p.extend([Op::Check(true, false), shot("assignment-passed", "result")]);
            p.push(Op::Choose("observer-choice".into(), "1".into()));
            run(&mut p, "scan", "nc -nvz 198.51.100.20 9000");
            p.push(Op::Probe(
                1,
                Some(if number == 0 {
                    PortState::Filtered
                } else {
                    PortState::Closed
                }),
                false,
            ));
            p.push(Op::Check(true, false));
            p.push(shot("nc-correct-negative", "scan-status"));
            run(&mut p, "admin", "kill -STOP 500");
            p.extend([
                Op::NotRun(false),
                Op::Click("check".into()),
                Op::Check(false, false),
                shot("broken-success-invalidated", "result"),
            ]);
            run(&mut p, "admin", "kill -CONT 500");
            p.extend([Op::Click("check".into()), Op::Check(true, false)]);
        }
        for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
            p.extend([Op::Style(scale, light), Op::Narrow]);
            // Exercise real field editing at enlarged text, not screenshots alone.
            run(&mut p, "admin", "ss -lntp");
            p.push(Op::Choose("observer-choice".into(), "1".into()));
            run(&mut p, "scan", "nc -n -v -z 198.51.100.20 443");
            p.push(Op::Probe(
                0,
                Some(dlms_learning_lab::service_exposure::PortState::Open),
                false,
            ));
            p.push(Op::Check(true, false));
            run(&mut p, "scan", "nc -nvz 198.51.100.20 80");
            p.push(Op::Probe(
                1,
                Some(dlms_learning_lab::service_exposure::PortState::Closed),
                false,
            ));
            p.push(Op::Check(true, false));
            p.push(Op::Shot(
                format!("service-exposure-{label}-nc-refused"),
                "scan-status".into(),
            ));
            use dlms_learning_lab::telnet_client::Mode;
            run(&mut p, "scan", "telnet -c 198.51.100.20 443");
            p.extend([
                Op::Client(Some(Mode::Connected { port: 443 }), None, false),
                Op::Shot(
                    format!("service-exposure-{label}-telnet-live"),
                    "scan-escape".into(),
                ),
                Op::Click("scan-escape".into()),
                Op::Client(Some(Mode::Command { port: 443 }), None, false),
                Op::Shot(
                    format!("service-exposure-{label}-telnet-local"),
                    "scan-input".into(),
                ),
            ]);
            run(&mut p, "scan", "quit");
            p.extend([Op::Client(None, Some(0), false), Op::Check(true, false)]);
            for key in ["admin-input", "scan-input", "observer-choice", "check"] {
                p.push(Op::Shot(
                    format!("service-exposure-{label}-{key}"),
                    key.into(),
                ));
            }
        }
        p.push(Op::Save);
        p
    })
}
fn target(a: &LabApp, k: &str) -> Option<egui::Rect> {
    a.metrics
        .service_exposure
        .controls
        .iter()
        .find(|v| v.0 == k)
        .map(|v| v.1)
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::ServiceExposure { simulation } = &a.progress.attempts[&s.id].state else {
        panic!("Service-exposure source state");
    };
    simulation
}
fn scope(a: &LabApp, practice: bool) -> &work::Scope {
    let s = session(a);
    if practice { &s.example } else { &s.assignment }
}
fn field_value(a: &LabApp, key: &str) -> String {
    let s = session(a);
    let scope = scope(a, s.view == teaching::View::Example);
    match key {
        "admin-input" => scope.admin_input.clone(),
        "scan-input" => scope.scan_input.clone(),
        _ => panic!("Unknown connection field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Option<Vec<(bool, String)>> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let scope = scope(a, practice);
    assert!(scope.checked, "Click Check before observing a result");
    Some(
        if practice {
            d.example.checks(&scope.world)
        } else {
            d.assignment.checks(&scope.world)
        }
        .into_iter()
        .map(|c| (c.passed, c.detail))
        .collect(),
    )
}
fn capture_geometry(ctx: &egui::Context) -> Option<(egui::Vec2, f32)> {
    let (extent, native) = ctx.input(|i| {
        (
            i.viewport_rect().size(),
            i.viewport().native_pixels_per_point,
        )
    });
    // Wayland may omit inner_rect. The current rendered viewport still gives
    // the actual UI extent; convert it with the settled pixels-per-point.
    Some((extent * ctx.pixels_per_point(), native?))
}
impl LabApp {
    fn service_exposure_parent_wheel(&mut self, ctx: &egui::Context) {
        if !self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            return;
        }
        let view = self.metrics.main_viewport.expect("actual parent viewport");
        let region = self
            .metrics
            .main_scroll_region
            .expect("actual scroll region");
        let gutter = egui::Rect::from_min_max(
            egui::pos2(view.right(), view.top()),
            egui::pos2(region.right(), view.bottom()),
        )
        .intersect(ctx.content_rect());
        assert!(gutter.width() * ctx.pixels_per_point() > 1.);
        for e in &mut self.pending_events {
            if let egui::Event::PointerMoved(p) = e {
                *p = gutter.center();
            }
        }
    }
    pub(in crate::ui) fn service_exposure_tick(&mut self, ctx: &egui::Context) {
        let ops = active_plan(&self.scenarios);
        self.service_exposure_flow(ctx, ops);
    }

    pub(super) fn service_exposure_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("service-exposure-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("SERVICE_EXPOSURE_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "service exposure native deadline at {} {:?}",
            f.at,
            ops[f.at]
        );
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        f.next = now + 0.1;
        f.polls += 1;
        assert!(f.polls < 1200, "service exposure eligible-poll limit");
        if f.polls == 1 {
            println!("SERVICE_EXPOSURE_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_service_exposure_reopen(ctx);
                true
            }
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Choose(k, value) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let ack = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(ack)).unwrap_or(0) >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let option = format!("{k}:{value}");
                        let (_, rect, clip) = self
                            .metrics
                            .service_exposure
                            .controls
                            .iter()
                            .find(|c| c.0 == option)
                            .expect("Actual supplied popup option")
                            .clone();
                        let clip = clip.intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            f.popup_reveals += 1;
                            assert!(f.popup_reveals <= 20, "service exposure popup reveal limit");
                            self.pending_events.extend([
                                egui::Event::PointerMoved(clip.center()),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(
                                        0.,
                                        (clip.center().y - rect.center().y).clamp(-500., 500.),
                                    ),
                                    modifiers: Default::default(),
                                },
                            ]);
                            popup_wheel = true;
                            false
                        }
                    }
                }
            }
            Op::Text(k, text) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self
                        .metrics
                        .service_exposure
                        .fields
                        .iter()
                        .find(|(name, _, _)| name == &k)
                        .expect("Actual exposure text field")
                        .2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
                        egui::Event::Text(text.clone()),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    assert_eq!(field_value(self, &k), text);
                    true
                }
            }
            Op::Check(pass, practice) => {
                if let Some(checks) = checks(self, practice) {
                    assert_eq!(checks.iter().all(|c| c.0), pass, "{checks:?}");
                    if !practice {
                        assert_eq!(self.metrics.completion, pass);
                        let source = &self.scenarios[self.selected.unwrap()];
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            pass
                        );
                    }
                    true
                } else {
                    false
                }
            }
            Op::NotRun(practice) => {
                let s = scope(self, practice);
                assert!(!s.checked);
                if !practice {
                    assert!(!self.metrics.completion);
                }
                true
            }
            Op::Probe(status, state, practice) => {
                let s = scope(self, practice);
                assert_eq!(s.scan_status, Some(status));
                assert_eq!(
                    s.last_scan.as_ref().map(|r| {
                        assert_eq!(r.rows.len(), 1);
                        r.rows[0].state
                    }),
                    state
                );
                assert!(matches!(
                    s.history.last(),
                    Some(work::Event::ObserverV3 { .. })
                ));
                assert!(
                    self.metrics
                        .service_exposure
                        .controls
                        .iter()
                        .any(|r| r.0 == "scan-status")
                );
                true
            }
            Op::Client(mode, status, practice) => {
                let s = scope(self, practice);
                assert_eq!(s.telnet, mode);
                assert_eq!(s.scan_status, status);
                if mode.is_some() {
                    assert_eq!(status, None, "A live client has no exit status");
                }
                true
            }
            Op::Uncredited => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert!(!progress::current_solved(a, s));
                let d = binding::source(s).unwrap();
                assert_eq!(
                    session(self).assignment,
                    work::Session::new(d).unwrap().assignment
                );
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Narrow => {
                if f.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    f.phase = 1;
                    false
                } else if f.phase == 1 {
                    f.requested = Some(capture_size::request(ctx));
                    f.phase = 2;
                    false
                } else {
                    if let Some((actual, native)) = capture_geometry(ctx) {
                        if capture_size::fits(actual, f.requested.unwrap(), native) {
                            f.stable += 1;
                        } else {
                            f.stable = 0;
                        }
                        if f.stable >= 2 {
                            println!(
                                "SERVICE_EXPOSURE_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
                                f.requested,
                                ctx.zoom_factor()
                            );
                            true
                        } else {
                            false
                        }
                    } else {
                        f.stable = 0;
                        false
                    }
                }
            }
            Op::Style(scale, light) => {
                self.scale = scale;
                ctx.set_zoom_factor(scale);
                self.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(scale == 1.));
                if scale != 1. {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1100., 850.,
                    )));
                }
                self.lesson_scroll_reset = true;
                true
            }
            Op::Shot(name, k) => {
                if f.phase == 0 {
                    // Only capture navigation surrenders its earlier keyboard
                    // target; normal learner editing retains native focus.
                    ctx.memory_mut(|memory| {
                        if let Some(focused) = memory.focused() {
                            memory.surrender_focus(focused);
                        }
                    });
                    if self.reveal_smoke_control(ctx, target(self, &k).expect("capture anchor")) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let rect = target(self, &k).expect("settled capture anchor");
                    let visible = self
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if !smoke_control_visible(rect, visible, ctx.pixels_per_point()) {
                        f.phase = 0;
                        self.reveal_smoke_control(ctx, rect);
                        false
                    } else {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name),
                        ));
                        f.phase = 2;
                        false
                    }
                } else {
                    !self.capture_pending
                }
            }
        };
        if !popup_wheel {
            self.service_exposure_parent_wheel(ctx);
        }
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll);
            });
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.popup_reveals = 0;
            f.requested = None;
            f.stable = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[test]
fn service_exposure_actual_capture_phase_requests_an_image_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.capture_pending = false;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800., 600.),
        )),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(1.);
    let out = ctx.run(input, |ctx| {
        let clip = ctx.content_rect();
        app.metrics.main_viewport = Some(clip);
        app.metrics.service_exposure.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("service-exposure-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.service_exposure_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
    });
    assert!(app.capture_pending);
    assert!(
        out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
    );
}

#[test]
fn service_exposure_probe_plan_matches_real_models_before_native_capture() {
    use dlms_learning_lab::script_teaching::View;
    let ctx = egui::Context::default();
    let app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let mut sessions = std::collections::BTreeMap::new();
    let mut current = None;
    for op in active_plan(&app.scenarios) {
        if let Op::Open(id) = op {
            let source = app.scenarios.iter().find(|s| &s.id == id).unwrap();
            sessions
                .entry(id.clone())
                .or_insert_with(|| work::Session::new(binding::source(source).unwrap()).unwrap());
            current = Some(id.clone());
            continue;
        }
        let Some(id) = &current else {
            continue;
        };
        let source = app.scenarios.iter().find(|s| &s.id == id).unwrap();
        let d = binding::source(source).unwrap();
        let session = sessions.get_mut(id).unwrap();
        let practice = session.view == View::Example;
        let t = if practice { &d.example } else { &d.assignment };
        let s = if practice {
            &mut session.example
        } else {
            &mut session.assignment
        };
        let event = match op {
            Op::Click(k) if k == "example" || k == "assignment" => {
                session.view = if k == "example" {
                    View::Example
                } else {
                    View::Assignment
                };
                None
            }
            Op::Text(k, text) => {
                s.apply(
                    t,
                    &if k == "admin-input" {
                        work::Action::AdminInput { text: text.clone() }
                    } else {
                        assert_eq!(k, "scan-input");
                        work::Action::ScanInput { text: text.clone() }
                    },
                )
                .unwrap();
                None
            }
            Op::Click(k) if k == "admin-run" => Some(work::Event::Admin {
                input: s.admin_input.clone(),
            }),
            Op::Click(k) if k == "scan-run" => Some(work::Event::ObserverV3 {
                input: s.scan_input.clone(),
            }),
            Op::Click(k) if k == "scan-escape" => Some(work::Event::TelnetEscapeV3),
            Op::Click(k) if k == "check" => Some(work::Event::Check),
            Op::Choose(k, value) => {
                assert_eq!(k, "observer-choice");
                Some(work::Event::Select {
                    viewpoint: value.parse().unwrap(),
                })
            }
            Op::Check(pass, expected_practice) => {
                assert_eq!(*expected_practice, practice);
                assert_eq!(s.solved(t), *pass);
                None
            }
            Op::NotRun(expected_practice) => {
                assert_eq!(*expected_practice, practice);
                assert!(!s.checked);
                None
            }
            Op::Probe(status, state, expected_practice) => {
                assert_eq!(*expected_practice, practice);
                assert_eq!(s.scan_status, Some(*status));
                assert_eq!(
                    s.last_scan.as_ref().map(|r| r.rows[0].state),
                    *state,
                    "{id}"
                );
                None
            }
            Op::Client(mode, status, expected_practice) => {
                assert_eq!(*expected_practice, practice);
                assert_eq!(s.telnet, *mode);
                assert_eq!(s.scan_status, *status);
                None
            }
            _ => None,
        };
        if let Some(event) = event {
            s.apply(t, &work::Action::Event { event }).unwrap();
        }
    }
    assert_eq!(sessions.len(), 2);
    for (id, session) in sessions {
        let source = app.scenarios.iter().find(|s| s.id == id).unwrap();
        let d = binding::source(source).unwrap();
        d.bound(&session).unwrap();
        assert!(session.assignment.solved(&d.assignment));
        assert!(session.example.solved(&d.example));
    }
}
