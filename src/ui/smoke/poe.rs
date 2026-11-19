//! Fixed own-app native power-plan interaction; never a learner command runner.
use super::*;
use dlms_learning_lab::{poe_source as binding, poe_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    Open(String),
    Click(String),
    Choose(usize, Option<usize>),
    Check(bool, bool),
    Unchecked(bool),
    Uncredited,
    KeyboardPower(usize, bool, bool),
    Style(f32, bool),
    Narrow,
    Shot(String, String),
    Save,
    ColdAppearance,
    ColdFinish,
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
    focus_hops: usize,
}
fn plan() -> Vec<Op> {
    let mut p = vec![Op::Style(1., false)];
    for (number, id) in ["ccna-poe-compatible-ports", "ccna-poe-budget-recovery"]
        .iter()
        .enumerate()
    {
        let shot =
            |label: &str, key: &str| Op::Shot(format!("poe-{}-{label}", number + 1), key.into());
        p.extend([
            Op::Open((*id).into()),
            shot("introduction", "introduction-content"),
            shot("worked", "worked"),
            Op::Click("example".into()),
            shot("practice", "topology"),
            Op::Click("check".into()),
            Op::Check(false, true),
            Op::Click("worked-answer".into()),
            Op::Click("fill-example".into()),
            Op::Unchecked(true),
            shot("worked-plan", "port-choice:0"),
            Op::Click("check".into()),
            Op::Check(true, true),
            Op::Uncredited,
        ]);
        let port = if number == 0 { 1 } else { 3 };
        p.extend([
            Op::Click(format!("enabled:{port}")),
            Op::Unchecked(true),
            Op::Click("check".into()),
            Op::Check(false, true),
            shot("practice-mistake", "failed-case"),
            Op::KeyboardPower(port, true, true),
            Op::Click("check".into()),
            Op::Check(true, true),
            Op::Click("assignment".into()),
            shot("assignment", "assignment-content"),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("initial-feedback", "failed-case"),
        ]);
        if number == 0 {
            p.extend([
                Op::Choose(0, None),
                Op::Choose(2, Some(0)),
                Op::Choose(0, Some(1)),
            ]);
        } else {
            p.extend([
                Op::Choose(0, Some(3)),
                Op::Click("enabled:3".into()),
                Op::Unchecked(false),
                Op::Click("check".into()),
                Op::Check(false, false),
                shot("budget-denial", "failed-case"),
                Op::Choose(2, None),
            ]);
        }
        p.extend([
            Op::Unchecked(false),
            Op::Click("check".into()),
            Op::Check(true, false),
            shot("assignment-passed", "result"),
            Op::Click(format!("enabled:{port}")),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("success-invalidated", "failed-case"),
            Op::KeyboardPower(port, true, false),
            Op::Click("check".into()),
            Op::Check(true, false),
        ]);
    }
    for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
        p.extend([
            Op::Style(scale, light),
            Op::Narrow,
            Op::Shot(format!("poe-{label}-connections"), "port-choice:0".into()),
            Op::Shot(format!("poe-{label}-power-controls"), "enabled:3".into()),
            Op::Click("enabled:3".into()),
            Op::Click("check".into()),
            Op::Check(false, false),
            Op::Shot(format!("poe-{label}-feedback"), "failed-case".into()),
            Op::KeyboardPower(3, true, false),
            Op::Click("check".into()),
            Op::Check(true, false),
            Op::Shot(format!("poe-{label}-completion"), "result".into()),
        ]);
    }
    p.push(Op::Save);
    p
}
pub(super) fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    a.metrics
        .poe
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::Poe { simulation } = &a.progress.attempts[&source.id].state else {
        panic!("Expected native power workspace")
    };
    simulation
}
fn scope(a: &LabApp, practice: bool) -> &work::Scope {
    let s = session(a);
    if practice { &s.example } else { &s.assignment }
}
fn geometry(ctx: &egui::Context) -> Option<(egui::Vec2, f32)> {
    let (extent, native) = ctx.input(|i| {
        (
            i.viewport_rect().size(),
            i.viewport().native_pixels_per_point,
        )
    });
    Some((extent * ctx.pixels_per_point(), native?))
}
impl LabApp {
    pub(in crate::ui) fn poe_tick(&mut self, ctx: &egui::Context) {
        self.poe_flow(ctx, &plan());
    }
    pub(super) fn poe_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("poe-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("POE_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "power native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "power eligible-poll limit");
        if f.polls == 1 {
            println!("POE_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Choose(device, port) => {
                let k = format!("port-choice:{device}");
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
                        let option = format!(
                            "port-option:{device}:{}",
                            port.map(|p| p.to_string()).unwrap_or_else(|| "none".into())
                        );
                        let (_, rect, clip) = self
                            .metrics
                            .poe
                            .controls
                            .iter()
                            .find(|r| r.0 == option)
                            .expect("Actual power port option")
                            .clone();
                        let clip = clip.intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            f.popup_reveals += 1;
                            assert!(f.popup_reveals <= 20, "power popup reveal limit");
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
            Op::Check(pass, practice) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                let s = scope(self, practice);
                let t = if practice { &d.example } else { &d.assignment };
                assert!(s.checked, "Click Check before inspecting a result");
                assert_eq!(s.solved(t), pass, "{:?}", s.checks(t));
                if !practice {
                    assert_eq!(self.metrics.completion, pass);
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&source.id], source),
                        pass
                    );
                }
                true
            }
            Op::Unchecked(practice) => {
                assert!(!scope(self, practice).checked);
                true
            }
            Op::Uncredited => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                assert_eq!(
                    session(self).assignment,
                    work::Session::new(d).unwrap().assignment
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
                true
            }
            Op::KeyboardPower(port, want, practice) => {
                let target_key = format!("enabled:{port}");
                if f.phase == 0 {
                    if self.reveal_smoke_control(
                        ctx,
                        target(self, &target_key).expect("Power keyboard target"),
                    ) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self
                        .metrics
                        .poe
                        .focus
                        .iter()
                        .find(|(key, _)| key == &target_key)
                        .expect("Actual power focus target")
                        .1;
                    if ctx.memory(|m| m.focused()) == Some(id) {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::Space,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: Default::default(),
                        });
                        f.phase = 2;
                    } else {
                        f.focus_hops += 1;
                        assert!(
                            f.focus_hops <= 48,
                            "Bounded real Tab navigation must reach the power control"
                        );
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Tab,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    }
                    false
                } else {
                    assert_eq!(scope(self, practice).plan.enabled[port], want);
                    self.pending_events.push(egui::Event::Key {
                        key: egui::Key::Space,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: Default::default(),
                    });
                    true
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
            Op::Narrow => {
                if f.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    f.phase = 1;
                    false
                } else if f.phase == 1 {
                    f.requested = Some(capture_size::request(ctx));
                    f.phase = 2;
                    false
                } else if let Some((actual, native)) = geometry(ctx) {
                    if capture_size::fits(actual, f.requested.unwrap(), native) {
                        f.stable += 1;
                    } else {
                        f.stable = 0;
                    }
                    if f.stable >= 2 {
                        println!(
                            "POE_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
            Op::Shot(name, k) => {
                if f.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(id) = m.focused() {
                            m.surrender_focus(id);
                        }
                    });
                    if self
                        .reveal_smoke_control(ctx, target(self, &k).expect("Power capture anchor"))
                    {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let r = target(self, &k).expect("Settled power capture anchor");
                    let visible = self
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if !smoke_control_visible(r, visible, ctx.pixels_per_point()) {
                        f.phase = 0;
                        self.reveal_smoke_control(ctx, r);
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
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_poe_reopen(ctx);
                true
            }
        };
        if !popup_wheel
            && self
                .pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            let view = self.metrics.main_viewport.expect("Actual parent viewport");
            let region = self
                .metrics
                .main_scroll_region
                .expect("Actual parent scroll region");
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
            f.focus_hops = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[test]
fn poe_capture_phase_uses_real_geometry_without_context_reentry() {
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
    let output = ctx.run(input, |ctx| {
        let clip = ctx.content_rect();
        app.metrics.main_viewport = Some(clip);
        app.metrics.poe.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("poe-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.poe_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
    });
    assert!(app.capture_pending);
    assert!(
        output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
    );
}
#[test]
fn poe_native_plan_reaches_both_assignments_with_real_power_outcomes() {
    let labs = pack::current().unwrap();
    let mut sessions = std::collections::BTreeMap::new();
    let mut selected = None;
    for op in plan() {
        if let Op::Open(id) = op {
            let s = labs.iter().find(|s| s.id == id).unwrap();
            sessions
                .entry(id.clone())
                .or_insert_with(|| work::Session::new(binding::source(s).unwrap()).unwrap());
            selected = Some(id);
            continue;
        }
        let Some(id) = &selected else { continue };
        let source = labs.iter().find(|s| &s.id == id).unwrap();
        let d = binding::source(source).unwrap();
        let s = sessions.get_mut(id).unwrap();
        let practice = s.view == teaching::View::Example;
        let action = match op {
            Op::Click(key) if key == "example" || key == "assignment" => {
                s.view = if key == "example" {
                    teaching::View::Example
                } else {
                    teaching::View::Assignment
                };
                None
            }
            Op::Click(key) if key == "fill-example" => Some(work::Action::Edit {
                plan: d.example.reference.clone(),
            }),
            Op::Click(key) if key == "check" => Some(work::Action::Check),
            Op::Click(key) if key.starts_with("enabled:") => {
                let i = key
                    .strip_prefix("enabled:")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                let scope = if practice { &s.example } else { &s.assignment };
                let mut p = scope.plan.clone();
                p.enabled[i] = !p.enabled[i];
                Some(work::Action::Edit { plan: p })
            }
            Op::Choose(device, port) => {
                let scope = if practice { &s.example } else { &s.assignment };
                let mut p = scope.plan.clone();
                p.attachments[device] = port;
                Some(work::Action::Edit { plan: p })
            }
            Op::KeyboardPower(port, want, expected_practice) => {
                assert_eq!(practice, expected_practice);
                let scope = if practice { &s.example } else { &s.assignment };
                let mut p = scope.plan.clone();
                assert_ne!(p.enabled[port], want);
                p.enabled[port] = want;
                Some(work::Action::Edit { plan: p })
            }
            Op::Check(want, expected_practice) => {
                assert_eq!(practice, expected_practice);
                let (scope, t) = if practice {
                    (&s.example, &d.example)
                } else {
                    (&s.assignment, &d.assignment)
                };
                assert!(scope.checked);
                assert_eq!(scope.solved(t), want, "{id}: {:?}", scope.checks(t));
                None
            }
            Op::Unchecked(expected_practice) => {
                assert_eq!(practice, expected_practice);
                assert!(!if practice { &s.example } else { &s.assignment }.checked);
                None
            }
            Op::Uncredited => {
                assert_eq!(s.assignment, work::Session::new(d).unwrap().assignment);
                None
            }
            _ => None,
        };
        if let Some(action) = action {
            s.action(d, &action, practice).unwrap();
        }
    }
    assert_eq!(sessions.len(), 2);
    for (id, s) in sessions {
        let d = binding::source(labs.iter().find(|l| l.id == id).unwrap()).unwrap();
        assert!(s.assignment.solved(&d.assignment) && s.example.solved(&d.example));
    }
}
