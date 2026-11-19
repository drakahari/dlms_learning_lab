//! Fixed native smoke exercises for ATA diagnosis and monitoring.
use super::*;
use dlms_learning_lab::ata_smart::{
    session::Session,
    source::{self, Action},
    workspace::{Assignment, Disposition, Work},
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
    Input(&'static str, String),
    Ran(String),
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

fn native_actions(actions: &[Action]) -> Vec<Op> {
    let mut ops = Vec::new();
    for action in actions {
        match action {
            Action::Run { input } => ops.extend([
                Op::Input("command", input.clone()),
                Op::Button("run"),
                Op::Ran(input.clone()),
            ]),
            Action::Advance => ops.push(Op::Button("advance")),
            Action::Draft { contents } => ops.push(Op::Input("config", contents.clone())),
            Action::Save => ops.push(Op::Button("save")),
            Action::Record { ticket } => {
                ops.push(Op::Button("disposition"));
                let choice = match ticket.disposition {
                    Disposition::Completed => "ticket-completed",
                    Disposition::ReadFailure => "ticket-read-failure",
                    Disposition::Unset => "ticket-unset",
                };
                ops.push(Op::Button(choice));
                if ticket.disposition == Disposition::ReadFailure {
                    ops.push(Op::Input("lba", ticket.first_lba.clone()));
                }
            }
            Action::Check => ops.push(Op::Button("check")),
            Action::Reset => panic!("The fixed reference must contain the actual work"),
        }
    }
    ops
}

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack =
            serde_json::from_str(include_str!("../../../scenarios/ata-smart-lessons-v1.json"))
                .unwrap();
        let fixtures: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/ata_smart_tasks_v1.json"
        ))
        .unwrap();
        let mut steps = vec![Op::Appearance(1., false)];
        for (n, (lab, fixture)) in pack.scenarios.iter().zip(fixtures).enumerate() {
            let d = source::source(lab).unwrap();
            let diagnostic = matches!(d.assignment.goal, Assignment::Diagnostic(_));
            let task = if diagnostic { "command" } else { "config" };
            let total = if diagnostic { 3 } else { 4 };
            let partial = if diagnostic { 1 } else { 2 };
            let sample: Vec<Action> =
                serde_json::from_value(fixture["example_actions"].clone()).unwrap();
            let graded: Vec<Action> =
                serde_json::from_value(fixture["assignment_actions"].clone()).unwrap();
            let shot = |name: &str, anchor| Op::Capture(format!("smart-{}-{name}", n + 1), anchor);
            steps.extend([
                Op::Open(lab.id.clone()),
                shot("introduction", "introduction-content"),
                Op::Button("example"),
                shot("practice", task),
                Op::Button("check"),
                Op::Check {
                    count: if diagnostic { 1 } else { 3 },
                    practice: true,
                },
            ]);
            if !diagnostic {
                steps.push(Op::Button("maintenance-layout"));
            }
            steps.extend(native_actions(&sample));
            steps.extend([
                Op::Check {
                    count: total,
                    practice: true,
                },
                shot("example-complete", "result"),
                Op::Button("assignment"),
                shot("assignment", "assignment-content"),
                Op::Button("check"),
                Op::Check {
                    count: partial,
                    practice: false,
                },
                shot("initial-feedback", "result"),
            ]);
            if diagnostic {
                let start = graded
                    .iter()
                    .position(|a| matches!(a, Action::Advance))
                    .unwrap();
                steps.extend(native_actions(&graded[..start]));
                steps.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: 1,
                        practice: false,
                    },
                ]);
                steps.extend(native_actions(&graded[start..]));
            } else {
                steps.push(Op::Button("maintenance-layout"));
                steps.extend(native_actions(&graded));
            }
            steps.extend([
                Op::Check {
                    count: total,
                    practice: false,
                },
                shot("complete", "result"),
            ]);
            if diagnostic {
                steps.push(Op::Input("lba", "invalid-block-number".into()));
            } else {
                let Assignment::Monitoring(g) = &d.assignment.goal else {
                    panic!()
                };
                steps.push(Op::Input("config", g.initial_config.clone()));
                steps.push(Op::Button("save"));
            }
            steps.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: 2,
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        steps.extend([
            Op::Appearance(1.5, true),
            Op::Capture("smart-monitor-light150".into(), "maintenance-layout"),
            Op::Appearance(2., false),
            Op::Capture("smart-monitor-dark200".into(), "check"),
            Op::SaveReadback,
        ]);
        steps
    })
}

fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::AtaSmart { simulation } = &a.progress.attempts[&lab.id].state else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .ata_smart
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
    pub(in crate::ui) fn ata_smart_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("ata-smart-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let Some(op) = plan().get(f.at) else {
            println!("SMART_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("SMART_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native SMART control deadline: {op:?}"
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
                            "SMART_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
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
                    let current = match *name {
                        "command" => &w.command,
                        "config" => &w.monitor_draft,
                        "lba" => &w.ticket.first_lba,
                        _ => panic!("Unknown SMART field"),
                    };
                    assert_eq!(current, value);
                    true
                }
            },
            Op::Ran(expected) => {
                let s = session(self);
                let w = if s.view == teaching::View::Example {
                    &s.example.work
                } else {
                    &s.assignment.work
                };
                let actual = w
                    .transcript
                    .last()
                    .expect("The native Run button must execute");
                assert_eq!(&actual.command, expected);
                assert!(actual.status.is_some());
                assert!(w.command.is_empty());
                true
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
fn ata_smart_smoke_plan_exercises_distinct_device_and_configuration_work() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    assert!(ops.iter().any(|op| matches!(op, Op::Button("advance"))));
    assert!(
        ops.iter()
            .any(|op| matches!(op, Op::Button("maintenance-layout")))
    );
    assert!(ops.iter().any(|op| matches!(op, Op::Input("lba", _))));
    assert!(ops.iter().any(|op| matches!(op, Op::Input("config", _))));
}

#[test]
fn ata_smart_capture_uses_only_its_own_rendered_control() {
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&context, true, false, false, true);
    let name = "introduction-content";
    let other = egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(10., 10.));
    let actual = other.translate(egui::vec2(80., 0.));
    app.metrics.iscsi.controls.push((name.into(), other, other));
    assert_eq!(control(&app, name), None);
    app.metrics
        .ata_smart
        .controls
        .push((name.into(), actual, actual));
    assert_eq!(control(&app, name), Some(actual));
}
