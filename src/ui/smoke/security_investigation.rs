//! Fixed Wayland check for the actual evidence and role-policy native controls.
use super::*;
use dlms_learning_lab::security_investigation::{
    session::Session,
    source,
    workspace::{Scope, Work},
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
    Choice(&'static str),
    KeyboardApply,
    Input(&'static str, String),
    Minimum(u8),
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

fn select_steps(steps: &mut Vec<Op>, area: &'static str, choice: &'static str) {
    steps.extend([Op::Button(area), Op::Choice(choice)]);
}

fn repair(steps: &mut Vec<Op>, n: usize, answer: &serde_json::Value, practice: bool) {
    if n == 0 {
        select_steps(steps, "approval", "approval-2");
        select_steps(steps, "applied-record", "applied-record-4");
        select_steps(steps, "observation", "observation-6");
        select_steps(
            steps,
            "signin",
            if practice { "signin-1" } else { "signin-2" },
        );
        steps.extend([
            Op::Input("policy", answer["contents"].as_str().unwrap().into()),
            Op::KeyboardApply,
        ]);
    } else {
        select_steps(
            steps,
            "operator",
            if practice { "operator-1" } else { "operator-2" },
        );
        select_steps(steps, "technician", "technician-1");
        select_steps(steps, "visitor", "visitor-3");
        steps.push(Op::Button("record-denials"));
        select_steps(steps, "report-desk", "report-desk-2");
    }
}

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/security-investigation-lessons-v1.json"
        ))
        .unwrap();
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/security_investigation_tasks_v1.json"
        ))
        .unwrap();
        let mut steps = vec![Op::Appearance(1., false)];
        for (i, lab) in pack.scenarios.iter().enumerate() {
            let d = source::source(lab).unwrap();
            let row = &fixtures[&lab.id]["answers"];
            let key = if i == 0 { "approval" } else { "operator" };
            let shot =
                |part: &str, anchor| Op::Capture(format!("investigate-{}-{part}", i + 1), anchor);
            steps.extend([
                Op::Open(lab.id.clone()),
                shot("introduction", "introduction-content"),
                Op::Button("example"),
                shot("practice", key),
                Op::Button("check"),
                Op::Check {
                    count: [4, 2][i],
                    practice: true,
                },
            ]);
            repair(&mut steps, i, &row["example"], true);
            steps.extend([
                Op::Button("check"),
                Op::Check {
                    count: [9, 7][i],
                    practice: true,
                },
                shot("example-complete", "result"),
                Op::Button("assignment"),
                shot("assignment", "assignment-content"),
                Op::Button("check"),
                Op::Check {
                    count: [3, 2][i],
                    practice: false,
                },
                shot("initial-feedback", "result"),
            ]);
            repair(&mut steps, i, &row["assignment"], false);
            steps.extend([
                Op::Button("check"),
                Op::Check {
                    count: [9, 8][i],
                    practice: false,
                },
                shot("complete", "result"),
            ]);
            if let dlms_learning_lab::security_investigation::workspace::Task::Change { supplied } =
                &d.assignment
            {
                steps.extend([
                    Op::Input("policy", supplied.starter.clone()),
                    Op::Button("apply-policy"),
                ]);
            } else {
                select_steps(&mut steps, "visitor", "visitor-0");
            }
            steps.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: 7,
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        steps.extend([
            Op::Appearance(1.5, true),
            Op::Capture("investigate-light150-controls".into(), "visitor"),
            Op::Appearance(2., false),
            Op::Capture("investigate-navy200-check".into(), "check"),
            Op::SaveReadback,
        ]);
        steps
    })
}
fn additional_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/credentials-vpn-lessons-v1.json"
        ))
        .unwrap();
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/credentials_vpn_tasks_v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (i, lab) in pack.scenarios.iter().enumerate() {
            let a = &fixtures[&lab.id]["answers"];
            let scope = |practice: bool| &a[if practice { "example" } else { "assignment" }];
            let good = [10, 9][i];
            let snapshot = |part: &str, anchor| {
                Op::Capture(format!("credential-vpn-{}-{part}", i + 1), anchor)
            };
            let first = ["minimum-length", "vpn-0-left"][i];
            ops.extend([
                Op::Open(lab.id.clone()),
                snapshot("introduction", "introduction-content"),
            ]);
            for practice in [true, false] {
                ops.extend([
                    Op::Button(if practice { "example" } else { "assignment" }),
                    if practice {
                        snapshot("practice", first)
                    } else {
                        snapshot("assignment", "assignment-content")
                    },
                    Op::Button("check"),
                    Op::Check { count: 4, practice },
                ]);
                if i == 0 {
                    let desired = scope(practice)["minimum_length"].as_u64().unwrap() as u8;
                    ops.push(Op::Minimum(desired));
                    ops.push(Op::Button("blocklist"));
                    select_steps(&mut ops, "factor", "factor-2");
                    ops.push(Op::Button("replace"));
                } else {
                    select_steps(&mut ops, "vpn-0-left", "vpn-0-left-0");
                    for (control, row, key) in [
                        ("vpn-0-left-prefix", 0, "left_traffic"),
                        ("vpn-1-right-prefix", 1, "right_traffic"),
                    ] {
                        ops.push(Op::Input(
                            control,
                            scope(practice)[row][key].as_str().unwrap().into(),
                        ));
                    }
                    ops.push(Op::Button("apply-vpn"));
                }
                ops.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: good,
                        practice,
                    },
                    snapshot(
                        if practice {
                            "example-complete"
                        } else {
                            "complete"
                        },
                        "result",
                    ),
                ]);
            }
            if i == 0 {
                ops.push(Op::Button("blocklist"));
            } else {
                ops.push(Op::Input("vpn-1-right-prefix", "pending".into()));
                ops.push(Op::Button("apply-vpn"));
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: good - 1,
                    practice: false,
                },
                snapshot("needs-work", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture(
                "credential-vpn-light150-editor".into(),
                "vpn-1-right-prefix",
            ),
            Op::Appearance(2., false),
            Op::Capture("credential-vpn-navy200-trials".into(), "vpn-trials"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::SecurityInvestigation { simulation } = &a.progress.attempts[&lab.id].state
    else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .security_investigation
        .controls
        .iter()
        .find(|row| row.0 == name)
        .map(|row| row.1)
}

fn editor_scroll_point(a: &LabApp, ctx: &egui::Context) -> Option<egui::Pos2> {
    let editor = control(a, "policy")?;
    let view = a.metrics.main_viewport?.intersect(ctx.content_rect());
    let inside_parent = egui::pos2(view.left() + 1., view.center().y);
    if !editor.contains(inside_parent) {
        return None;
    }
    let region = a.metrics.main_scroll_region?.intersect(ctx.content_rect());
    let gutter = region.right() - view.right();
    if gutter * ctx.pixels_per_point() > 1. {
        return Some(egui::pos2(view.right() + gutter / 2., view.center().y));
    }
    let offset = 1. / ctx.pixels_per_point();
    [editor.top() - offset, editor.bottom() + offset]
        .into_iter()
        .map(|y| egui::pos2(inside_parent.x, y))
        .find(|point| view.contains(*point) && !editor.contains(*point))
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
    pub(in crate::ui) fn security_investigation_tick(
        &mut self,
        ctx: &egui::Context,
        credentials_vpn: bool,
    ) {
        let id = egui::Id::new(("security-investigation-native-flow", credentials_vpn));
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let recipe = if credentials_vpn {
            additional_plan()
        } else {
            plan()
        };
        let Some(op) = recipe.get(f.at) else {
            println!(
                "SECURITY_INVESTIGATION_NATIVE_COMPLETE {} fixed actions",
                f.at
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("SECURITY_INVESTIGATION_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native security control deadline: {op:?}"
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
                    if let Some(focused) = ctx.memory(|memory| memory.focused()) {
                        ctx.memory_mut(|memory| memory.surrender_focus(focused));
                    }
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
                            "SECURITY_INVESTIGATION_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
            Op::Choice(name) => {
                let item = self
                    .metrics
                    .security_investigation
                    .controls
                    .iter()
                    .find(|row| row.0 == *name);
                self.mac_popup_choice(ctx, item.map(|row| row.1), item.map(|row| row.2))
            }
            Op::KeyboardApply => match f.phase {
                0 => {
                    let rect = control(self, "apply-policy").expect("Rendered Apply target");
                    if self.reveal_smoke_control(ctx, rect) {
                        f.phase = 1;
                    }
                    false
                }
                1 => {
                    let id = self
                        .metrics
                        .security_investigation
                        .focus
                        .iter()
                        .find(|(key, _)| key == "apply-policy")
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
            Op::Minimum(value) => match f.phase {
                0 => {
                    if self.click_visible_smoke_control(ctx, control(self, "minimum-length")) {
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
                        egui::Event::Text(value.to_string()),
                        key_event(egui::Key::A, false, select),
                    ]);
                    f.phase = 2;
                    false
                }
                2 => {
                    self.pending_events.extend([
                        key_event(egui::Key::Enter, true, Default::default()),
                        key_event(egui::Key::Enter, false, Default::default()),
                    ]);
                    f.phase = 3;
                    false
                }
                _ => {
                    let s = session(self);
                    let w = if s.view == teaching::View::Example {
                        &s.example.work
                    } else {
                        &s.assignment.work
                    };
                    assert!(
                        matches!(w.current, Scope::Credentials { policy }
                        if policy.minimum_length == *value),
                        "The visible numeric control updates the staff policy"
                    );
                    true
                }
            },
            Op::Input(name, value) => match f.phase {
                0 => {
                    let rect = control(self, name).map(|r| {
                        if *name == "policy" {
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
                    let current = match name {
                        &"policy" => match &w.current {
                            Scope::Change { draft, .. } => draft,
                            _ => panic!("Expected actual virtual management policy editor"),
                        },
                        _ => {
                            let Scope::VpnDesign { drafts, .. } = &w.current else {
                                panic!("Expected actual VPN profile editor");
                            };
                            match *name {
                                "vpn-0-left-prefix" => &drafts[0].left_traffic,
                                "vpn-1-right-prefix" => &drafts[1].right_traffic,
                                _ => panic!("Unknown fixed edit fixture"),
                            }
                        }
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
                let checks = work.outcomes(task);
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
        // Only redirect page scrolling when it would land in the nested editor.
        // A facility page or a popup needs its own normal scroll input.
        if !matches!(op, Op::Choice(_))
            && self
                .pending_events
                .iter()
                .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
            && let Some(destination) = editor_scroll_point(self, ctx)
        {
            for event in &mut self.pending_events {
                if let egui::Event::PointerMoved(point) = event {
                    *point = destination;
                }
            }
        }
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            let popup_scroll = egui::Id::new(("mac-popup-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll);
                d.remove::<u8>(popup_scroll);
            });
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
fn the_security_flow_operates_evidence_and_policy_controls() {
    let ops = plan();
    assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 2);
    for name in [
        "approval-2",
        "observation-6",
        "signin-2",
        "visitor-3",
        "report-desk-2",
    ] {
        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Choice(key) if *key==name))
        );
    }
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&context, true, false, false, true);
    let name = "introduction-content";
    let other = egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(10., 10.));
    let actual = other.translate(egui::vec2(80., 0.));
    app.metrics
        .desktop_profile
        .controls
        .push((name.into(), other, other));
    assert_eq!(control(&app, name), None);
    app.metrics
        .security_investigation
        .controls
        .push((name.into(), actual, actual));
    assert_eq!(control(&app, name), Some(actual));
}

#[test]
fn scroll_stays_inside_the_page_when_there_is_no_editor_or_reserved_gutter() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let view = egui::Rect::from_min_max(egui::pos2(20., 100.), egui::pos2(500., 540.));
    app.metrics.main_viewport = Some(view);
    app.metrics.main_scroll_region = Some(view);
    assert_eq!(editor_scroll_point(&app, &ctx), None);
    let editor = egui::Rect::from_min_max(egui::pos2(20., 210.), egui::pos2(499., 430.));
    app.metrics
        .security_investigation
        .controls
        .push(("policy".into(), editor, view));
    let point = editor_scroll_point(&app, &ctx).expect("Measured parent point outside the editor");
    assert!(view.contains(point));
    assert!(!editor.contains(point));
}
