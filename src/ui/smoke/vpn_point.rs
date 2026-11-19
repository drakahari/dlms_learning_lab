//! Fixed native service exposure flows; private simulated work only.
use super::*;
use dlms_learning_lab::{vpn_point_source as binding, vpn_point_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    ColdFinish,
    Open(String),
    Library,
    Click(String),
    Choose(String, String),
    Text(String, String),
    Check(bool, bool),
    Uncredited,
    Save,
    NotRun(bool),
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

pub(super) fn translate(p: &mut Vec<Op>, a: &work::Action) {
    match a {
        work::Action::Edit { peer, text } => {
            p.push(Op::Text(format!("editor-{peer}"), text.clone()))
        }
        work::Action::Save { peer } => p.push(Op::Click(format!("save-{peer}"))),
        work::Action::Apply => p.push(Op::Click("apply".into())),
        work::Action::Probe => p.push(Op::Click("probe".into())),
        work::Action::Query { query } => {
            p.push(Op::Choose("from-choice".into(), query.from.to_string()));
            p.push(Op::Text("source".into(), query.source.clone()));
            p.push(Op::Text("destination".into(), query.destination.clone()));
        }
        work::Action::Check => p.push(Op::Click("check".into())),
    }
}
fn active_plan(labs: &[pack::Scenario]) -> &'static Vec<Op> {
    static P: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![Op::Style(1., false)];
        for (number, id) in ["lpic2-vpn-peer-profile", "lpic2-vpn-lan-return"]
            .iter()
            .enumerate()
        {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(source).unwrap();
            let shot = |name: &str, key: &str| {
                Op::Shot(format!("vpn-point-{}-{name}", number + 1), key.into())
            };
            p.extend([
                Op::Open(source.id.clone()),
                shot("introduction", "introduction-content"),
                shot("worked", "worked"),
                Op::Click("example".into()),
                shot("practice-first-profile", "editor-viewport-0"),
                shot("practice-second-profile", "editor-viewport-1"),
                Op::NotRun(true),
                Op::Click("check".into()),
                Op::Check(false, true),
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
            for raw in &source.solution {
                let envelope: serde_json::Value =
                    serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                translate(
                    &mut p,
                    &serde_json::from_value(envelope["action"].clone()).unwrap(),
                );
            }
            p.extend([
                Op::Check(true, false),
                shot("assignment-passed", "result"),
                shot("actual-path", "probe-result"),
            ]);
            let good = source
                .solution
                .iter()
                .filter_map(|raw| {
                    let e: serde_json::Value =
                        serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                    let a: work::Action = serde_json::from_value(e["action"].clone()).unwrap();
                    match a {
                        work::Action::Edit { peer: 1, text } => Some(text),
                        _ => None,
                    }
                })
                .next_back()
                .unwrap();
            p.extend([
                Op::Text("editor-1".into(), format!("{good}script-security 2\n")),
                Op::NotRun(false),
                shot("edited-success-invalidated", "save-1"),
                Op::Click("save-1".into()),
                Op::Click("apply".into()),
                shot("unsupported-setting", "notice"),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Text("editor-1".into(), good),
                Op::Click("save-1".into()),
                Op::Click("apply".into()),
                Op::Click("check".into()),
                Op::Check(true, false),
            ]);
        }
        for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
            p.extend([
                Op::Style(scale, light),
                Op::Narrow,
                Op::Text("source".into(), "172.21.47.254".into()),
                Op::Text("destination".into(), "172.20.31.254".into()),
                Op::Click("probe".into()),
                Op::Check(true, false),
            ]);
            for key in [
                "source",
                "destination",
                "from-choice",
                "editor-viewport-0",
                "save-0",
                "editor-viewport-1",
                "save-1",
                "check",
            ] {
                p.push(Op::Shot(format!("vpn-point-{label}-{key}"), key.into()));
            }
        }
        p.extend([
            Op::Save,
            Op::Style(1., false),
            Op::Shot("vpn-point-common-navigation".into(), "back".into()),
            Op::Click("back".into()),
            Op::Library,
            Op::Open("lpic2-vpn-lan-return".into()),
            Op::Check(true, false),
            Op::Shot("vpn-point-return-retains-work".into(), "result".into()),
        ]);
        for (id, name) in [
            ("lpic2-ftp-local-jail", "ftp"),
            ("lpic2-exposure-client-policy", "exposure"),
        ] {
            p.extend([
                Op::Open(id.into()),
                Op::Shot(format!("vpn-related-{name}-navigation"), "back".into()),
                Op::Click("back".into()),
                Op::Library,
            ]);
        }
        p.push(Op::Save);
        p
    })
}
fn target(a: &LabApp, k: &str) -> Option<egui::Rect> {
    if k == "back" {
        return a.metrics.back;
    }
    a.metrics
        .vpn_point
        .controls
        .iter()
        .find(|v| v.0 == k)
        .map(|v| v.1)
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::VpnPoint { simulation } = &a.progress.attempts[&s.id].state else {
        panic!("VPN source state");
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
        "editor-0" => scope.editors[0].clone(),
        "editor-1" => scope.editors[1].clone(),
        "source" => scope.query.source.clone(),
        "destination" => scope.query.destination.clone(),
        _ => panic!("Unknown VPN field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Option<Vec<(bool, String)>> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let scope = scope(a, practice);
    assert!(scope.checked, "Click Check before observing a result");
    Some(
        if practice {
            scope.checks(&d.example)
        } else {
            scope.checks(&d.assignment)
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
    fn vpn_point_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn vpn_point_tick(&mut self, ctx: &egui::Context) {
        let ops = active_plan(&self.scenarios);
        self.vpn_point_flow(ctx, ops);
    }

    pub(super) fn vpn_point_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("vpn-point-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("VPN_POINT_NATIVE_COMPLETE {} fixed actions", f.at);
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
            println!("VPN_POINT_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_vpn_point_reopen(ctx);
                true
            }
            Op::Library => {
                assert!(
                    self.selected.is_none(),
                    "Back must return to the actual library"
                );
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
                            .vpn_point
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
                        .vpn_point
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
                                "VPN_POINT_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
            self.vpn_point_parent_wheel(ctx);
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
fn vpn_point_actual_capture_phase_requests_an_image_without_context_reentry() {
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
        app.metrics.vpn_point.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("vpn-point-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.vpn_point_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
    });
    assert!(app.capture_pending);
    assert!(
        out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
    );
}
