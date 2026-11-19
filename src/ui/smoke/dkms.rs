//! Fixed own-app kernel editing/build/staging flows; no learner host execution.
use super::*;
use dlms_learning_lab::{dkms_source as binding, dkms_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    Open(String),
    Click(String),
    Input(String, String),
    Check(bool, bool),
    Unchecked(bool),
    Uncredited,
    KeyboardActivate(String),
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
pub(super) fn expand(task: &work::Task, practice: bool) -> Vec<Op> {
    let mut ops = vec![];
    for a in &task.reference {
        match a {
            work::Action::Run { input } => ops.extend([
                Op::Input("command".into(), input.clone()),
                Op::KeyboardActivate("run".into()),
            ]),
            work::Action::Check => {
                ops.extend([Op::Click("check".into()), Op::Check(true, practice)])
            }
            _ => panic!("Unreviewed DKMS reference action"),
        }
    }
    ops
}
fn plan() -> Vec<Op> {
    let raw =
        serde_json::from_str::<pack::Pack>(include_str!("../../../scenarios/dkms-lessons-v1.json"))
            .unwrap();
    let mut ops = vec![Op::Style(1., false)];
    for (n, s) in raw.scenarios.iter().enumerate() {
        let d = binding::source(s).unwrap();
        let shot = |name: &str, key: &str| Op::Shot(format!("dkms-{}-{name}", n + 1), key.into());
        ops.extend([
            Op::Open(s.id.clone()),
            shot("introduction", "introduction-content"),
            shot("worked", "worked"),
            Op::Click("example".into()),
            shot("practice", "supplied"),
            Op::Input("command".into(), "dkms status".into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Check(false, true),
            Op::Click("worked-answer".into()),
            Op::Click("fill-example".into()),
            Op::Unchecked(true),
            Op::Click("check".into()),
            Op::Check(true, true),
            Op::Uncredited,
            shot("example-results", "result"),
            Op::Click("assignment".into()),
            shot("assignment", "assignment-content"),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("initial-feedback", "failed-case"),
        ]);
        ops.extend(expand(&d.assignment, false));
        ops.push(shot("complete", "result"));
        let source = &d.assignment.facts.sources[if n == 0 { 0 } else { 1 }];
        let release = &d.assignment.facts.kernels[1].release;
        ops.extend([
            Op::Input(
                "command".into(),
                format!(
                    "dkms uninstall {}/{} -k {release}",
                    source.name, source.version
                ),
            ),
            Op::Click("run".into()),
            Op::Unchecked(false),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("invalidated", "failed-case"),
            Op::Input(
                "command".into(),
                format!(
                    "dkms install {}/{} -k {release}",
                    source.name, source.version
                ),
            ),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Check(true, false),
        ]);
    }
    for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
        ops.extend([
            Op::Style(scale, light),
            Op::Narrow,
            Op::Shot(format!("dkms-{label}-inputs"), "supplied".into()),
            Op::Shot(format!("dkms-{label}-console"), "command".into()),
            Op::Shot(format!("dkms-{label}-results"), "result".into()),
        ]);
    }
    ops.push(Op::Save);
    ops
}
pub(super) fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    a.metrics
        .dkms
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::Dkms { simulation } = &a.progress.attempts[&source.id].state else {
        panic!("Expected native kernel workspace")
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
    pub(in crate::ui) fn dkms_tick(&mut self, ctx: &egui::Context) {
        self.dkms_flow(ctx, &plan());
    }
    pub(super) fn dkms_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("dkms-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("DKMS_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "DKMS native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "kernel eligible-poll limit");
        if f.polls == 1 {
            println!("DKMS_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }

        let done = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Input(key, value) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &key)) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
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
                        egui::Event::Paste(value.clone()),
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers: egui::Modifiers {
                                command: true,
                                ..egui::Modifiers::CTRL
                            },
                        },
                    ]);
                    f.phase = 2;
                    false
                } else {
                    let practice = session(self).view == teaching::View::Example;
                    let w = &scope(self, practice).work;
                    assert_eq!(key, "command");
                    let actual = &w.command_draft;
                    assert_eq!(actual, &value, "Native input must change the actual draft");
                    true
                }
            }
            Op::Check(pass, practice) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                let s = scope(self, practice);
                let t = if practice { &d.example } else { &d.assignment };
                assert!(s.work.checked, "Click Check before inspecting a result");
                assert_eq!(t.solved(&s.work), pass, "{:?}", t.checks(&s.work));
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
                assert!(!scope(self, practice).work.checked);
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
            Op::KeyboardActivate(key) => {
                if f.phase == 0 {
                    let r = target(self, &key).expect("Kernel keyboard target");
                    if self.reveal_smoke_control(ctx, r) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self
                        .metrics
                        .dkms
                        .focus
                        .iter()
                        .find(|(k, _)| k == &key)
                        .expect("Recorded kernel focus target")
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
                            "Bounded actual Tab navigation must reach Save"
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
                            "DKMS_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
                        .reveal_smoke_control(ctx, target(self, &k).expect("DKMS capture anchor"))
                    {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let r = target(self, &k).expect("Settled DKMS capture anchor");
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
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_dkms_reopen(ctx);
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
        };
        if self
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
fn dkms_capture_phase_uses_real_geometry_without_context_reentry() {
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
        app.metrics.dkms.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("dkms-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.dkms_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
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
fn dkms_native_plan_uses_actual_commands_checks_and_separate_examples() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert_eq!(
        ops.iter().filter(|op| matches!(op, Op::Uncredited)).count(),
        2
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::KeyboardActivate(k)if k=="run"))
    );
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, Op::Style(_, _)))
            .count(),
        3
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::Input(k,v)if k=="command"&&v.starts_with("dkms uninstall")))
    );
}
