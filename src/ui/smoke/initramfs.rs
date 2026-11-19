//! Fixed native editor, generation, inspection and boot-handoff smoke flows.
use super::*;
use dlms_learning_lab::{initramfs_source as binding, initramfs_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    Open(String),
    Click(String),
    Input(String, String),
    Check(bool, bool),
    OutputContains(String),
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
    requested: Option<egui::Vec2>,
    stable: u8,
    focus_hops: usize,
}
pub(super) fn expand(actions: &[work::Action], practice: bool) -> Vec<Op> {
    let mut ops = vec![];
    for action in actions {
        match action {
            work::Action::ConfigDraft { text } => {
                ops.push(Op::Input("config".into(), text.clone()))
            }
            work::Action::SaveConfig => ops.push(Op::KeyboardActivate("save-config".into())),
            work::Action::EntryDraft { filename, text } => ops.extend([
                Op::Input("entry-name".into(), filename.clone()),
                Op::Input("entry-text".into(), text.clone()),
            ]),
            work::Action::SaveEntry => ops.push(Op::KeyboardActivate("save-entry".into())),
            work::Action::Run { input } => ops.extend([
                Op::Input("command".into(), input.clone()),
                Op::Click("run".into()),
            ]),
            work::Action::CommandDraft { text } => {
                ops.push(Op::Input("command".into(), text.clone()))
            }
            work::Action::Check => {
                ops.extend([Op::Click("check".into()), Op::Check(true, practice)])
            }
            work::Action::Reset => panic!("Unreviewed reset in the fixed native plan"),
        }
    }
    ops
}

fn plan() -> Vec<Op> {
    let raw = serde_json::from_str::<pack::Pack>(include_str!(
        "../../../scenarios/initramfs-lessons-v1.json"
    ))
    .unwrap();
    let mut ops = vec![Op::Style(1., false)];
    for (index, source) in raw.scenarios.iter().enumerate() {
        let definition = binding::source(source).unwrap();
        let shot =
            |name: &str, key: &str| Op::Shot(format!("initramfs-{}-{name}", index + 1), key.into());
        ops.extend([
            Op::Open(source.id.clone()),
            shot("introduction", "introduction-content"),
            shot("worked", "worked"),
            Op::Click("example".into()),
            shot("practice", "supplied"),
            Op::Input(
                "command".into(),
                "lsinitrd /boot/initramfs-6.1.121.img".into(),
            ),
            Op::Click("run".into()),
            Op::OutputContains(if index == 0 { "6.1.121" } else { "6.1.120" }.into()),
            shot("inspection", "command"),
            Op::Click("check".into()),
            Op::Check(false, true),
        ]);
        ops.extend(expand(&definition.example.reference, true));
        ops.extend([
            Op::Uncredited,
            shot("example-results", "result"),
            Op::Click("assignment".into()),
            shot("assignment", "assignment-content"),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("initial-feedback", "failed-case"),
        ]);
        ops.extend(expand(&definition.assignment.reference, false));
        ops.push(shot("complete", "result"));

        let changed = if index == 0 {
            Op::Input("config".into(), "omit_drivers=\"nvme\"\n".into())
        } else {
            Op::Input("entry-name".into(), "unsaved.conf".into())
        };
        ops.extend([
            changed,
            Op::Unchecked(false),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("invalidated", "failed-case"),
        ]);
    }
    for (scale, light, label) in [(1.5, true, "light150"), (2., false, "dark200")] {
        ops.extend([
            Op::Style(scale, light),
            Op::Narrow,
            Op::Shot(format!("initramfs-{label}-inputs"), "entry-name".into()),
            Op::Shot(format!("initramfs-{label}-console"), "command".into()),
            Op::Shot(format!("initramfs-{label}-results"), "result".into()),
        ]);
    }
    ops.push(Op::Save);
    ops
}
pub(super) fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    a.metrics
        .early_userspace
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}

fn editor_hit_target(key: &str, rect: egui::Rect) -> egui::Rect {
    if matches!(key, "config" | "entry-text") {
        // Only the focus click needs to be exposed. At large text sizes a
        // multiline editor can intersect the scroll clip while its center
        // still accepts input. The action verifies the resulting full draft.
        let size = rect.size().min(egui::vec2(4., 4.));
        egui::Rect::from_center_size(rect.center(), size)
    } else {
        rect
    }
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::Initramfs { simulation } = &a.progress.attempts[&source.id].state else {
        panic!("Expected native early-userspace workspace")
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
    pub(in crate::ui) fn initramfs_tick(&mut self, ctx: &egui::Context) {
        self.initramfs_flow(ctx, &plan());
    }
    pub(super) fn initramfs_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("initramfs-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("INITRAMFS_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "initramfs native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "early-userspace control poll limit");
        if f.polls == 1 {
            println!("INITRAMFS_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
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
                    let rect = target(self, &key).map(|rect| editor_hit_target(&key, rect));
                    if self.click_visible_smoke_control(ctx, rect) {
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
                        "command" => &w.command_draft,
                        "config" => &w.config_draft,
                        "entry-name" => &w.entry_filename,
                        "entry-text" => &w.entry_draft,
                        _ => panic!("Unknown fixed early-userspace editor"),
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
            Op::OutputContains(expected) => {
                let line = scope(self, true).work.log.last().expect("Inspection ran");
                assert!(
                    line.ok && line.output.contains(&expected),
                    "{}",
                    line.output
                );
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
                    let r = target(self, &key).expect("Early-userspace keyboard target");
                    if self.reveal_smoke_control(ctx, r) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self
                        .metrics
                        .early_userspace
                        .focus
                        .iter()
                        .find(|(k, _)| k == &key)
                        .expect("Recorded early-userspace focus target")
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
                            "INITRAMFS_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
                    if self.reveal_smoke_control(
                        ctx,
                        target(self, &k).expect("initramfs capture anchor"),
                    ) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let r = target(self, &k).expect("Settled initramfs capture anchor");
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
                self.finish_initramfs_reopen(ctx);
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
fn initramfs_multiline_focus_target_uses_the_visible_editor_center() {
    let view = egui::Rect::from_min_max(egui::pos2(0., 115.), egui::pos2(530., 514.));
    let editor = egui::Rect::from_min_max(egui::pos2(18., 382.), egui::pos2(517., 520.));
    assert!(!smoke_control_visible(editor, view, 1.725));
    let focus = editor_hit_target("entry-text", editor);
    assert_eq!(focus.center(), editor.center());
    assert!(editor.contains_rect(focus) && smoke_control_visible(focus, view, 1.725));
    assert!(!smoke_control_visible(
        editor_hit_target("entry-text", editor.translate(egui::vec2(0., 250.))),
        view,
        1.725,
    ));
    assert_eq!(editor_hit_target("entry-name", editor), editor);
}

#[test]
fn initramfs_capture_phase_uses_real_geometry_without_context_reentry() {
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
        app.metrics.early_userspace.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("initramfs-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.initramfs_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
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
fn initramfs_native_plan_uses_actual_commands_checks_and_separate_examples() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert_eq!(
        ops.iter().filter(|op| matches!(op, Op::Uncredited)).count(),
        2
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::KeyboardActivate(k)if k=="save-config"))
    );
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, Op::Style(_, _)))
            .count(),
        3
    );
    assert!(
        ops.iter()
            .any(|op| matches!(op,Op::Input(k,v)if k=="command"&&v.starts_with("dracut")))
    );
}
