//! Fixed native regression for the separate Apache configuration and client trials.
use super::*;
use dlms_learning_lab::apache_https::{session::Session, source, workspace::Work};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Appearance(f32, bool),
    Button(&'static str),
    SelectedTrial(&'static str),
    KeyboardSave,
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

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/apache-https-lessons-v1.json"
        ))
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/apache_https_tasks_v1.json"
        ))
        .unwrap();
        let mut steps = vec![Op::Appearance(1., false)];
        for (n, lab) in pack.scenarios.iter().enumerate() {
            let d = source::source(lab).unwrap();
            let answers = &fixture[&lab.id]["answers"];
            let considered = d.assignment.checks.len() + 1;
            let wrong = if n == 0 { 2 } else { 1 };
            let shot = |name: &str, anchor| Op::Capture(format!("https-{}-{name}", n + 1), anchor);
            steps.extend([
                Op::Open(lab.id.clone()),
                shot("introduction", "introduction-content"),
                Op::Button("example"),
                shot("practice", "config"),
                Op::Button("check"),
                Op::Check {
                    count: wrong,
                    practice: true,
                },
                Op::Input("config", answers["example"].as_str().unwrap().into()),
                Op::KeyboardSave,
                Op::Button("apply"),
                Op::Button("client"),
                Op::Button("trial-1"),
                Op::SelectedTrial("selected-trial-1"),
                Op::Button("check"),
                Op::Check {
                    count: considered,
                    practice: true,
                },
                shot("practice-result", "result"),
                Op::Button("assignment"),
                shot("assignment", "assignment-content"),
                Op::Button("check"),
                Op::Check {
                    count: wrong,
                    practice: false,
                },
                shot("initial-feedback", "result"),
                Op::Input("config", answers["assignment"].as_str().unwrap().into()),
                Op::Unchecked,
                Op::Button("save"),
                Op::Button("check"),
                Op::Check {
                    count: wrong - 1,
                    practice: false,
                },
                Op::Button("apply"),
                Op::Button("check"),
                Op::Check {
                    count: considered,
                    practice: false,
                },
                shot("complete", "result"),
                Op::Input("config", d.assignment.starter.clone()),
                Op::Unchecked,
                Op::Button("save"),
                Op::Button("apply"),
                Op::Button("check"),
                Op::Check {
                    count: wrong,
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        steps.extend([
            Op::Button("client"),
            Op::Button("trial-3"),
            Op::SelectedTrial("selected-trial-3"),
            Op::Appearance(1.5, true),
            Op::Capture("https-light150-editor".into(), "save"),
            Op::Appearance(2., false),
            Op::Capture("https-navy200-default".into(), "client"),
            Op::SaveReadback,
        ]);
        steps
    })
}
fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::ApacheHttps { simulation } = &a.progress.attempts[&lab.id].state else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .apache_https
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
    pub(in crate::ui) fn apache_https_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("apache-https-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let Some(op) = plan().get(f.at) else {
            println!("HTTPS_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("HTTPS_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native HTTPS control deadline: {op:?}"
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
                            "HTTPS_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
            Op::SelectedTrial(name) => control(self, name).is_some(),
            Op::KeyboardSave => match f.phase {
                0 => {
                    let rect = control(self, "save").expect("Rendered Save target");
                    if self.reveal_smoke_control(ctx, rect) {
                        f.phase = 1;
                    }
                    false
                }
                1 => {
                    let id = self
                        .metrics
                        .apache_https
                        .focus
                        .iter()
                        .find(|(key, _)| key == "save")
                        .expect("Recorded Save focus")
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
                            "Tab navigation must reach the enabled Save button"
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
                        if *name == "config" {
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
                    assert_eq!(*name, "config");
                    let current = &w.draft;
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
fn apache_https_native_plan_exercises_both_selections_in_the_actual_editor() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert!(ops.iter().any(|op| matches!(op, Op::Button("client"))));
    assert!(ops.iter().any(|op| matches!(op, Op::Button("apply"))));
    assert!(ops.iter().any(|op| matches!(op, Op::Button("trial-3"))));
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&context, true, false, false, true);
    let name = "introduction-content";
    let other = egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(10., 10.));
    let actual = other.translate(egui::vec2(80., 0.));
    app.metrics
        .ata_smart
        .controls
        .push((name.into(), other, other));
    assert_eq!(control(&app, name), None);
    app.metrics
        .apache_https
        .controls
        .push((name.into(), actual, actual));
    assert_eq!(control(&app, name), Some(actual));
}
