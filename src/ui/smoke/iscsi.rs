//! Fixed native smoke exercises for the virtual initiator console.
use super::*;
use dlms_learning_lab::iscsi::{session::Session, source};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Appearance(f32, bool),
    Button(&'static str),
    Command(String, bool),
    Check { count: usize, practice: bool },
    Unchecked,
    Capture(String, &'static str),
    SaveReadback,
}

#[derive(Clone, Debug, Default)]
struct Flow {
    at: usize,
    phase: u8,
    since: Option<Instant>,
    next: f64,
}

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack =
            serde_json::from_str(include_str!("../../../scenarios/iscsi-lessons-v1.json")).unwrap();
        let fixtures: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../tests/fixtures/iscsi_tasks_v1.json"))
                .unwrap();
        let mut steps = vec![Op::Appearance(1., false)];
        for (n, (lab, fixture)) in pack.scenarios.iter().zip(fixtures).enumerate() {
            let d = source::source(lab).unwrap();
            let startup = d.assignment.goal.automatic_at_startup;
            let total = if startup { 4 } else { 3 };
            let example = fixture["example_commands"].as_array().unwrap();
            let shot = |name: &str, anchor| Op::Capture(format!("iscsi-{}-{name}", n + 1), anchor);
            steps.extend([
                Op::Open(lab.id.clone()),
                shot("introduction", "introduction-content"),
                Op::Button("example"),
                shot("practice", "command"),
                Op::Button("check"),
                Op::Check {
                    count: if startup { 2 } else { 1 },
                    practice: true,
                },
            ]);
            for (position, command) in example.iter().enumerate() {
                if !startup && position == 2 {
                    steps.push(Op::Command("iscsiadm -m session".into(), false));
                } else if startup && position == 1 {
                    steps.extend([
                        Op::Button("startup-preview"),
                        Op::Button("check"),
                        Op::Check {
                            count: 3,
                            practice: true,
                        },
                    ]);
                }
                steps.push(Op::Command(command.as_str().unwrap().into(), true));
            }
            steps.extend([
                Op::Button("check"),
                Op::Check {
                    count: total,
                    practice: true,
                },
                shot("example-complete", "result"),
                Op::Button("assignment"),
                shot("assignment", "assignment-content"),
                Op::Command("iscsiadm -m session -P 3 | cat /etc/shadow".into(), false),
                Op::Command(lab.solution[0].clone(), true),
                Op::Button("check"),
                Op::Check {
                    count: total - 1,
                    practice: false,
                },
                shot("partial", "result"),
            ]);
            for command in &lab.solution[1..lab.solution.len() - 1] {
                steps.push(Op::Command(command.clone(), true));
            }
            if startup {
                steps.push(Op::Button("startup-preview"));
            }
            steps.extend([
                Op::Button("check"),
                Op::Check {
                    count: total,
                    practice: false,
                },
                shot("complete", "result"),
            ]);
            let breakage = if startup {
                lab.solution[0].replace("-v automatic", "-v manual")
            } else {
                lab.solution[2].replace("--login", "--logout")
            };
            steps.extend([
                Op::Command(breakage, true),
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: total - 1,
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        for (scale, light, label) in [(1.5, true, "light150"), (2., false, "navy200")] {
            steps.extend([
                Op::Appearance(scale, light),
                Op::Button("assignment"),
                Op::Capture(format!("iscsi-{label}-command"), "command"),
                Op::Capture(format!("iscsi-{label}-feedback"), "result"),
            ]);
        }
        steps.push(Op::SaveReadback);
        steps
    })
}

fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::Iscsi { simulation } = &a.progress.attempts[&lab.id].state else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .iscsi
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
    pub(in crate::ui) fn iscsi_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("iscsi-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let Some(op) = plan().get(f.at) else {
            println!("ISCSI_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("ISCSI_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native iSCSI control deadline: {op:?}"
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
                            "ISCSI_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
            Op::Command(command, ok) => {
                let s = session(self);
                let w = if s.view == teaching::View::Example {
                    &s.example.work
                } else {
                    &s.assignment.work
                };
                match f.phase {
                    0 => {
                        if self.click_visible_smoke_control(ctx, control(self, "command")) {
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
                            egui::Event::Paste(command.clone()),
                            key_event(egui::Key::A, false, select),
                        ]);
                        f.phase = 2;
                        false
                    }
                    2 => {
                        assert_eq!(&w.input, command);
                        self.pending_events.extend([
                            key_event(egui::Key::Enter, true, Default::default()),
                            key_event(egui::Key::Enter, false, Default::default()),
                        ]);
                        f.phase = 3;
                        false
                    }
                    _ => {
                        let last = w.history.last().expect("A native command must run");
                        assert_eq!(&last.input, command);
                        assert_eq!(last.ok, *ok, "{}", last.output);
                        assert!(w.input.is_empty());
                        true
                    }
                }
            }
            Op::Check { count, practice } => {
                let s = session(self);
                let lab = &self.scenarios[self.selected.unwrap()];
                let d = source::source(lab).unwrap();
                let (task, work) = if *practice {
                    (&d.example, &s.example.work)
                } else {
                    (&d.assignment, &s.assignment.work)
                };
                let checks = task.checks(&work.initiator);
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
                    assert_eq!(
                        s.assignment.work,
                        dlms_learning_lab::iscsi::workspace::Attempt::new(&d.assignment)
                    );
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
        // Route parent scroll around the nested terminal output viewport.
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
            f.since = None;
        }
        ctx.data_mut(|d| d.insert_temp(id, f));
        ctx.request_repaint_after(Duration::from_millis(25));
    }
}

#[test]
fn iscsi_smoke_plan_checks_both_workspaces_and_two_scales() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert!(ops.iter().any(|op| matches!(op, Op::Appearance(2., false))));
    assert!(ops.iter().any(|op| matches!(op, Op::Command(_, false))));
    assert!(
        ops.iter()
            .any(|op| matches!(op, Op::Check { practice: true, .. }))
    );
}
