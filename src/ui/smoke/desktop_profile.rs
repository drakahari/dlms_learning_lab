//! Fixed native actions for the desktop settings, key probes and client trials.
use super::*;
use dlms_learning_lab::desktop::{
    session::Session,
    source,
    workspace::{Work, Workspace},
};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Appearance(f32, bool),
    Button(&'static str),
    KeyboardApply,
    Input(&'static str, String),
    Check { count: usize, practice: bool },
    Unchecked,
    Capture(String, &'static str),
    SaveReadback,
}

#[derive(Clone, Debug, Default)]
struct Flow {
    at: usize,
    phase: u8,
    focus_hops: usize,
    since: Option<Instant>,
    next: f64,
}

fn select(ops: &mut Vec<Op>, label: &'static str, option: &'static str) {
    ops.extend([Op::Button(label), Op::Button(option)]);
}

fn repair(ops: &mut Vec<Op>, activity: usize, answers: &serde_json::Value, practice: bool) {
    match activity {
        0 => {
            ops.push(Op::Input(
                "keyboard-editor",
                answers[if practice { "example" } else { "assignment" }]
                    .as_str()
                    .unwrap()
                    .into(),
            ));
            ops.push(Op::KeyboardApply);
            ops.push(Op::Button(if practice { "key-Y" } else { "key-Q" }));
        }
        1 => {
            select(ops, "sharing", "sharing-1");
            if practice {
                ops.push(Op::Button("remote-login"));
                select(ops, "observer-port", "observer-port-0");
            } else {
                select(ops, "observer-port", "observer-port-1");
                select(ops, "employee-client", "employee-client-0");
            }
        }
        2 => {
            select(ops, "text-size", "text-size-1");
            if !practice {
                ops.push(Op::Button("contrast"));
            }
            ops.push(Op::Button("sticky"));
            select(
                ops,
                "typing-filter",
                if practice {
                    "typing-filter-1"
                } else {
                    "typing-filter-2"
                },
            );
        }
        _ => unreachable!(),
    }
}

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/desktop-profile-lessons-v1.json"
        ))
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/desktop_profile_tasks_v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (i, lab) in pack.scenarios.iter().enumerate() {
            let d = source::source(lab).unwrap();
            let answer = &fixture[&lab.id]["answers"];
            let key = ["keyboard-editor", "sharing", "text-size"][i];
            let good = [4, 2, 3][i];
            let initial = [2, 0, 0][i];
            let after_change = [2, 1, 2][i];
            let shot =
                |name: &str, anchor| Op::Capture(format!("desktop-{}-{name}", i + 1), anchor);
            ops.extend([
                Op::Open(lab.id.clone()),
                shot("introduction", "introduction-content"),
                Op::Button("example"),
                shot("practice", key),
                Op::Button("check"),
                Op::Check {
                    count: initial,
                    practice: true,
                },
            ]);
            repair(&mut ops, i, answer, true);
            ops.extend([
                Op::Button("check"),
                Op::Check {
                    count: good,
                    practice: true,
                },
                shot("example-result", "result"),
                Op::Button("assignment"),
                shot("assignment", "assignment-content"),
                Op::Button("check"),
                Op::Check {
                    count: initial,
                    practice: false,
                },
                shot("initial-feedback", "result"),
            ]);
            repair(&mut ops, i, answer, false);
            ops.extend([
                Op::Button("check"),
                Op::Check {
                    count: good,
                    practice: false,
                },
                shot("complete", "result"),
            ]);
            match i {
                0 => {
                    assert_ne!(d.example, d.assignment);
                    let broad = answer["assignment"]
                        .as_str()
                        .unwrap()
                        .replace("Language-desk keyboard FR", "Workshop");
                    ops.extend([
                        Op::Input("keyboard-editor", broad),
                        Op::Button("apply-keyboard"),
                    ]);
                }
                1 => select(&mut ops, "sharing", "sharing-2"),
                2 => select(&mut ops, "typing-filter", "typing-filter-1"),
                _ => unreachable!(),
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: after_change,
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("desktop-light150-reading".into(), "text-size"),
            Op::Appearance(2., false),
            Op::Capture("desktop-navy200-input".into(), "typing-filter"),
            Op::SaveReadback,
        ]);
        ops
    })
}
fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::Desktop { simulation } = &a.progress.attempts[&lab.id].state else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .desktop_profile
        .controls
        .iter()
        .find(|row| row.0 == name)
        .map(|row| row.1)
}

fn key_event(key: egui::Key, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    }
}

impl LabApp {
    pub(in crate::ui) fn desktop_profile_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("desktop-profile-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let Some(op) = plan().get(f.at) else {
            println!("DESKTOP_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("DESKTOP_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native desktop control deadline: {op:?}"
        );
        f.next = now + 0.12;
        let done = match op {
            Op::Open(lab) => {
                self.launch(self.scenarios.iter().position(|s| &s.id == lab).unwrap());
                self.lesson_scroll_reset = true;
                true
            }
            Op::Appearance(scale, light) => {
                if f.phase == 0 {
                    self.scale = *scale;
                    ctx.set_zoom_factor(*scale);
                    self.theme_id = theme::inventory()
                        .iter()
                        .find(|t| t.light == *light)
                        .unwrap()
                        .id
                        .clone();
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*scale == 1.));
                    self.lesson_scroll_reset = true;
                    f.phase = 1;
                    false
                } else if *scale == 1. {
                    true
                } else if f.phase == 1 {
                    capture_size::request(ctx);
                    f.phase = 2;
                    false
                } else {
                    let pixels = ctx.content_rect().size() * ctx.pixels_per_point();
                    if pixels.x < 1500. {
                        println!(
                            "DESKTOP_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
            Op::KeyboardApply => match f.phase {
                0 => {
                    let rect = control(self, "apply-keyboard").expect("Rendered Apply target");
                    if self.reveal_smoke_control(ctx, rect) {
                        f.phase = 1;
                    }
                    false
                }
                1 => {
                    let id = self
                        .metrics
                        .desktop_profile
                        .focus
                        .iter()
                        .find(|(key, _)| key == "apply-keyboard")
                        .expect("Recorded Apply focus")
                        .1;
                    if ctx.memory(|m| m.focused()) == Some(id) {
                        self.pending_events.push(key_event(
                            egui::Key::Space,
                            true,
                            Default::default(),
                        ));
                        f.phase = 2;
                    } else {
                        f.focus_hops += 1;
                        assert!(
                            f.focus_hops <= 48,
                            "Tab navigation must reach the Apply button"
                        );
                        for pressed in [true, false] {
                            self.pending_events.push(key_event(
                                egui::Key::Tab,
                                pressed,
                                Default::default(),
                            ));
                        }
                    }
                    false
                }
                _ => {
                    self.pending_events.push(key_event(
                        egui::Key::Space,
                        false,
                        Default::default(),
                    ));
                    true
                }
            },
            Op::Input(name, value) => match f.phase {
                0 => {
                    let rect = control(self, name).map(|r| {
                        if *name == "keyboard-editor" {
                            egui::Rect::from_center_size(
                                r.center(),
                                r.size().min(egui::vec2(4., 4.)),
                            )
                        } else {
                            r
                        }
                    });
                    if self.click_visible_smoke_control(ctx, rect) {
                        f.phase = 1;
                    }
                    false
                }
                1 => {
                    let select = egui::Modifiers {
                        command: true,
                        ..egui::Modifiers::CTRL
                    };
                    self.pending_events.extend([
                        key_event(egui::Key::A, true, select),
                        egui::Event::Paste(value.clone()),
                        key_event(egui::Key::A, false, select),
                    ]);
                    f.phase = 2;
                    false
                }
                _ => {
                    let s = session(self);
                    let w = if s.view == teaching::View::Example {
                        &s.example.work
                    } else {
                        &s.assignment.work
                    };
                    assert_eq!(*name, "keyboard-editor");
                    let Workspace::Keyboard { draft: current, .. } = &w.workspace else {
                        panic!("Expected actual virtual keyboard editor");
                    };
                    assert_eq!(current, value);
                    true
                }
            },
            Op::Check { count, practice } => {
                let s = session(self);
                let lab = &self.scenarios[self.selected.unwrap()];
                let d = source::source(lab).unwrap();
                let (task, work) = if *practice {
                    (&d.example, &s.example.work)
                } else {
                    (&d.assignment, &s.assignment.work)
                };
                let checks = work.checks(task);
                assert!(work.checked);
                assert_eq!(
                    checks.iter().filter(|c| c.passed).count(),
                    *count,
                    "{checks:?}"
                );
                assert_eq!(
                    self.metrics.completion,
                    !*practice && *count == checks.len()
                );
                if *practice {
                    assert_eq!(s.assignment.work, Work::new(&d.assignment));
                }
                true
            }
            Op::Unchecked => {
                assert!(!session(self).assignment.work.checked);
                assert!(!self.metrics.completion);
                true
            }
            Op::Capture(name, anchor) => {
                if f.phase == 0 {
                    let r = control(self, anchor).expect("Actual rendered capture anchor");
                    if self.reveal_smoke_control(ctx, r) {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name.clone()),
                        ));
                        f.phase = 1;
                    }
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::SaveReadback => {
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
        // Route parent scroll around the nested configuration editor viewport.
        if self
            .pending_events
            .iter()
            .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
        {
            let view = self.metrics.main_viewport.unwrap();
            let region = self.metrics.main_scroll_region.unwrap();
            let gutter = egui::Rect::from_min_max(
                egui::pos2(view.right(), view.top()),
                egui::pos2(region.right(), view.bottom()),
            );
            for event in &mut self.pending_events {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| d.remove::<usize>(scroll));
            f.at += 1;
            f.phase = 0;
            f.focus_hops = 0;
            f.since = None;
        }
        ctx.data_mut(|d| d.insert_temp(id, f));
        ctx.request_repaint_after(Duration::from_millis(25));
    }
}

#[test]
fn desktop_native_flow_uses_all_three_actual_workspaces() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 3);
    for name in ["key-Y", "observer-port-1", "sticky", "typing-filter-2"] {
        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Button(key) if *key==name))
        );
    }
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&context, true, false, false, true);
    let name = "introduction-content";
    let other = egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(10., 10.));
    let actual = other.translate(egui::vec2(80., 0.));
    app.metrics
        .apache_https
        .controls
        .push((name.into(), other, other));
    assert_eq!(control(&app, name), None);
    app.metrics
        .desktop_profile
        .controls
        .push((name.into(), actual, actual));
    assert_eq!(control(&app, name), Some(actual));
}
