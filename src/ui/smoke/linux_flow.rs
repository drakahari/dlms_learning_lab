//! Fixed native router lessons: virtual state, actual keyboard/pointer controls and captures.
use super::*;
use dlms_learning_lab::{linux_flow_source as binding, linux_flow_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    Reply(bool),
    LiveIndex(usize),
    ColdFinish,
    OutputContains(String, bool),
    Open(String),
    Click(String),
    Choose(String, String),
    Text(String, String),
    Check(bool, bool),
    Uncredited,
    Save,
    NotRun(bool),
    Fresh,
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
        work::Action::Command { line } => run(p, line),
        work::Action::Input { text } => p.push(Op::Text("command".into(), text.clone())),
        work::Action::SelectCase { index } => p.push(Op::Choose("case".into(), index.to_string())),
        work::Action::RunTraffic => p.push(Op::Click("run-traffic".into())),
        work::Action::Check => p.push(Op::Click("check".into())),
    }
}
fn run(p: &mut Vec<Op>, line: &str) {
    p.extend([
        Op::Text("command".into(), line.into()),
        Op::Click("run".into()),
    ]);
}
fn active_plan() -> &'static Vec<Op> {
    static P: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::current().unwrap();
        let mut p = vec![];
        for (n, id) in [
            "lpic2-nat-shared-address-replies",
            "lpic2-nat-published-service-policy",
        ]
        .iter()
        .enumerate()
        {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(source).unwrap();
            let tag = |name: &str| format!("connections-{n}-{name}");
            let reply = |task: &work::Task| {
                task.cases[0]
                    .events
                    .iter()
                    .position(|e| matches!(e.packet, work::PacketInput::Reply { .. }))
                    .expect("Authored actual reply")
                    .to_string()
            };
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(tag("worked"), "worked".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "command".into()),
                Op::Click("insert-command".into()),
                Op::NotRun(true),
                Op::Click("run".into()),
                Op::Click("run-traffic".into()),
                Op::Shot(tag("guided-first-trial"), "event-feedback".into()),
                Op::Click("check".into()),
                Op::Check(false, true),
            ]);
            for a in &d.worked {
                translate(&mut p, a);
            }
            p.extend([
                Op::Check(true, true),
                Op::Uncredited,
                Op::Click("run-traffic".into()),
                Op::Check(true, true),
                Op::Choose("event".into(), reply(&d.example)),
                Op::Reply(true),
                Op::Shot(tag("guided-reply"), "event-feedback".into()),
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "assignment".into()),
                Op::Shot(tag("workspace"), "command".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("initial-feedback"), "failed-case".into()),
            ]);
            run(&mut p, "iptables -L");
            p.extend([
                Op::OutputContains("[Unsupported]".into(), false),
                Op::Shot(tag("unsupported"), "output-viewport".into()),
            ]);
            for raw in &source.solution {
                let a: work::Action =
                    serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                translate(&mut p, &a);
            }
            p.extend([
                Op::Check(true, false),
                Op::Click("run-traffic".into()),
                Op::Check(true, false),
                Op::Choose("event".into(), reply(&d.assignment)),
                Op::Reply(false),
                Op::Shot(tag("assignment-reply"), "event-feedback".into()),
                Op::Shot(tag("assignment-complete"), "result".into()),
                Op::Save,
                Op::Text("command".into(), "iptables -P FORWARD ACCEPT".into()),
                Op::NotRun(false),
                Op::Click("run".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("stale-success-cleared"), "failed-case".into()),
            ]);
            for (scale, light, name) in [(1.5, true, "light150"), (2., false, "dark200")] {
                p.extend([
                    Op::Style(scale, light),
                    Op::Narrow,
                    Op::Shot(tag(&format!("{name}-input")), "command".into()),
                    Op::Shot(tag(&format!("{name}-run")), "run".into()),
                    Op::Shot(tag(&format!("{name}-traffic")), "run-traffic".into()),
                ]);
            }
            p.extend([
                Op::Style(1., false),
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        p
    })
}
fn feedback_plan() -> &'static Vec<Op> {
    static P: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::current().unwrap();
        let mut p = vec![];
        for (n, id) in [
            "lpic2-nat-shared-address-replies",
            "lpic2-nat-published-service-policy",
        ]
        .iter()
        .enumerate()
        {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Click("assignment".into()),
            ]);
            for raw in &source.solution {
                let a: work::Action =
                    serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                translate(&mut p, &a);
            }
            p.push(Op::Check(true, false));
            run(
                &mut p,
                "iptables -I FORWARD 1 -m conntrack --ctstate ESTABLISHED -j DROP",
            );
            run(&mut p, "iptables -A FORWARD -j ACCEPT");
            run(&mut p, "iptables -S");
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::LiveIndex(4 + n),
                Op::Shot(
                    format!("connections-{n}-live-rule-index"),
                    "failed-case".into(),
                ),
            ]);
        }
        p
    })
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        _ => a
            .metrics
            .flow
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::LinuxFlow { simulation } = &a.progress.attempts[&s.id].state else {
        panic!("Connection source state");
    };
    simulation
}
fn scope(a: &LabApp, practice: bool) -> &work::Scope {
    let s = session(a);
    if practice { &s.practice } else { &s.assignment }
}
fn field_value(a: &LabApp, key: &str) -> String {
    let s = session(a);
    let scope = scope(a, s.view == teaching::View::Example);
    match key {
        "command" => scope.input.clone(),
        _ => panic!("Unknown connection field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Option<Vec<(bool, String)>> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let scope = scope(a, practice);
    assert!(scope.checked, "Click Check before observing a result");
    Some(
        if practice {
            d.example.checks(scope)
        } else {
            d.assignment.checks(scope)
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
    fn connection_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn connection_tick(&mut self, ctx: &egui::Context) {
        self.connection_flow(
            ctx,
            if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("linux-flow-feedback") {
                feedback_plan()
            } else {
                active_plan()
            },
        );
    }
    pub(super) fn connection_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("connection-filters-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("LINUX_FLOW_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Connection native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Connection eligible-poll limit");
        if f.polls == 1 {
            println!("LINUX_FLOW_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::LiveIndex(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                let scope = scope(self, false);
                assert!(scope.checked);
                assert_eq!(scope.state.filter.base.forward.rules.len(), expected);
                let checks = d.assignment.checks(scope);
                let failed = checks
                    .iter()
                    .find(|c| c.label == "FORWARD · NEW · destination unchanged")
                    .unwrap();
                assert!(!failed.passed);
                assert!(
                    failed
                        .detail
                        .contains(&format!("Inspect FORWARD rule {expected},")),
                    "{}",
                    failed.detail
                );
                true
            }
            Op::Reply(practice) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                let t = if practice { &d.example } else { &d.assignment };
                let scope = scope(self, practice);
                let case = &t.cases[scope.selected_case];
                let key = egui::Id::new(("flow-selected-event", &source.id, practice, &case.id));
                let event = ctx
                    .data(|data| data.get_temp::<usize>(key))
                    .expect("Actual selected event");
                assert!(matches!(
                    case.events[event].packet,
                    work::PacketInput::Reply { .. }
                ));
                let step = &scope.last_trial.as_ref().unwrap().steps[event];
                assert!(step.check.passed);
                let trace = step.trace.as_ref().unwrap();
                assert_eq!(
                    trace.class,
                    Some(dlms_learning_lab::linux_connections::Class::Established)
                );
                assert_eq!(
                    trace.direction,
                    Some(dlms_learning_lab::linux_connections::Direction::Reply)
                );
                true
            }
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_flow_reopen(ctx);
                true
            }
            Op::OutputContains(text, practice) => {
                assert!(
                    scope(self, practice).output.contains(&text),
                    "Missing {text:?} in actual console output"
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
                            .flow
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
                            assert!(f.popup_reveals <= 20, "Connection popup reveal limit");
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
                    let rect = target(self, &k).unwrap();
                    let id = self
                        .metrics
                        .flow
                        .fields
                        .iter()
                        .find(|(_, r, _)| *r == rect)
                        .expect("Actual mail text field")
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
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
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
                                "LINUX_FLOW_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
                    if k == "timeline" {
                        eprintln!(
                            "SIEVE_TIMELINE_CAPTURE target={:?} parent={:?} region={:?} offset={:?} ppp={} focus={:?}",
                            target(self, &k),
                            self.metrics.main_viewport,
                            self.metrics.main_scroll_region,
                            self.metrics.main_scroll_offset,
                            ctx.pixels_per_point(),
                            ctx.memory(|m| m.focused())
                        );
                    }
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
            self.connection_parent_wheel(ctx);
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
fn connection_native_plan_keeps_independent_work_and_observes_failures() {
    let p = active_plan();
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
    for pair in [(false, true), (true, true), (false, false), (true, false)] {
        assert!(
            p.iter()
                .filter(|o| matches!(o,Op::Check(a,b)if(*a,*b)==pair))
                .count()
                >= 2
        );
    }
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Uncredited)).count(), 2);
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Reply(_))).count(), 4);
    let shots = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(n, _) = o {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        shots.len(),
        shots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
}
#[test]
fn connection_capture_uses_actual_viewport_when_wayland_inner_rect_is_absent() {
    let ctx = egui::Context::default();
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800., 600.),
        )),
        ..Default::default()
    };
    let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
    viewport.native_pixels_per_point = Some(1.);
    viewport.inner_rect = None;
    let _ = ctx.run(input, |ctx| {
        assert_eq!(capture_geometry(ctx).unwrap().0, egui::vec2(800., 600.));
    });
}

#[test]
fn connection_feedback_plan_checks_actual_original_indices_after_inactive_rows() {
    let p = feedback_plan();
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
    assert!(p.iter().any(|o| matches!(o, Op::LiveIndex(4))));
    assert!(p.iter().any(|o| matches!(o, Op::LiveIndex(5))));
    assert_eq!(
        p.iter()
            .filter(|o| matches!(o, Op::Check(true, false)))
            .count(),
        2
    );
    assert_eq!(
        p.iter()
            .filter(|o| matches!(o, Op::Check(false, false)))
            .count(),
        2
    );
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Shot(..))).count(), 2);
}
