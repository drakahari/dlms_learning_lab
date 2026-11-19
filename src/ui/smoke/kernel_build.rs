//! Fixed own-app kernel editing/build/staging flows; no learner host execution.
use super::*;
use dlms_learning_lab::{kernel_source as binding, kernel_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    Open(String),
    Click(String),
    Choose(String, String),
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
            work::Action::Source { index } => ops.push(Op::Choose(
                "source".into(),
                format!("source-option:{index}"),
            )),
            work::Action::Config { text } => ops.extend([
                Op::Input("config".into(), text.clone()),
                Op::KeyboardActivate("save-config".into()),
            ]),
            work::Action::SaveConfig => {}
            work::Action::Run { input } => ops.extend([
                Op::Input("command".into(), input.clone()),
                Op::Click("run".into()),
            ]),
            work::Action::StageImage => ops.push(Op::Click("stage-image".into())),
            work::Action::WorkingInitramfs => ops.push(Op::Click("build-initramfs".into())),
            work::Action::Initramfs { .. } => ops.extend([
                Op::Choose(
                    "staged-choice".into(),
                    format!("staged-option:{}", task.initial_store.images.len()),
                ),
                Op::Click("staged-initramfs".into()),
            ]),
            work::Action::EntryDraft { filename, text } => ops.extend([
                Op::Input("entry-name".into(), filename.clone()),
                Op::Input("entry-text".into(), text.clone()),
            ]),
            work::Action::SaveEntry => ops.push(Op::Click("save-entry".into())),
            work::Action::Check => {
                ops.extend([Op::Click("check".into()), Op::Check(true, practice)])
            }
            _ => panic!("Unreviewed native kernel action"),
        }
    }
    ops
}
fn plan() -> Vec<Op> {
    let raw = serde_json::from_str::<pack::Pack>(include_str!(
        "../../../scenarios/kernel-build-lessons-v1.json"
    ))
    .unwrap();
    let mut ops = vec![Op::Style(1., false)];
    for (n, s) in raw.scenarios.iter().enumerate() {
        let d = binding::source(s).unwrap();
        let shot = |name: &str, key: &str| Op::Shot(format!("kernel-{}-{name}", n + 1), key.into());
        ops.extend([
            Op::Open(s.id.clone()),
            shot("introduction", "introduction-content"),
            shot("worked", "worked"),
            Op::Click("example".into()),
            shot("practice", "config"),
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
        if n == 0 {
            ops.extend([
                Op::Input("config".into(), d.assignment.initial_config.clone()),
                Op::Unchecked(false),
                Op::KeyboardActivate("save-config".into()),
                Op::Input("command".into(), "make all".into()),
                Op::Click("run".into()),
                Op::Click("build-initramfs".into()),
            ]);
        } else {
            let text = d
                .assignment
                .reference
                .iter()
                .find_map(|a| {
                    if let work::Action::EntryDraft { text, .. } = a {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .unwrap()
                .replace(
                    &d.assignment.root.as_ref().unwrap().uuid,
                    "99999999-aaaa-bbbb-cccc-000000000000",
                );
            ops.extend([
                Op::Input("entry-text".into(), text),
                Op::Unchecked(false),
                Op::Click("save-entry".into()),
            ]);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("invalidated", "failed-case"),
        ]);
        ops.extend(expand(&d.assignment, false));
    }
    for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
        ops.extend([
            Op::Style(scale, light),
            Op::Narrow,
            Op::Shot(format!("kernel-{label}-config"), "config".into()),
            Op::Shot(format!("kernel-{label}-console"), "command".into()),
            Op::Shot(format!("kernel-{label}-entry"), "entry-text".into()),
            Op::Shot(format!("kernel-{label}-result"), "result".into()),
        ]);
    }
    ops.push(Op::Save);
    ops
}
pub(super) fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    a.metrics
        .kernel_build
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::KernelBuild { simulation } = &a.progress.attempts[&source.id].state else {
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
    pub(in crate::ui) fn kernel_build_tick(&mut self, ctx: &egui::Context) {
        self.kernel_build_flow(ctx, &plan());
    }
    pub(super) fn kernel_build_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("kernel_build-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("KERNEL_BUILD_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "kernel native deadline at {} {:?}",
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
            println!("KERNEL_BUILD_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }

        let done = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Choose(key, option) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &key)) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let ack = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(ack)).unwrap_or(0) >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        self.click_visible_smoke_control(ctx, target(self, &option))
                    }
                }
            }
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
                    let actual = match key.as_str() {
                        "config" => &w.config_draft,
                        "command" => &w.command_draft,
                        "entry-name" => &w.entry_filename,
                        "entry-text" => &w.entry_draft,
                        _ => panic!("Unknown fixed kernel field"),
                    };
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
                        .kernel_build
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
                            "KERNEL_BUILD_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
                        .reveal_smoke_control(ctx, target(self, &k).expect("Kernel capture anchor"))
                    {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let r = target(self, &k).expect("Settled kernel capture anchor");
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
                self.finish_kernel_build_reopen(ctx);
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
fn kernel_build_capture_phase_uses_real_geometry_without_context_reentry() {
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
        app.metrics.kernel_build.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("kernel_build-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.kernel_build_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
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
fn kernel_build_native_plan_uses_widgets_for_build_and_boot_actions() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::Input(k,_)if k=="config"))
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::Input(k,_)if k=="entry-text"))
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::KeyboardActivate(k)if k=="save-config"))
    );
    assert!(ops.iter().filter(|op| matches!(op, Op::Uncredited)).count() == 2);
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, Op::Style(_, _)))
            .count(),
        3
    );
}
