//! Fixed native router lessons: virtual state, actual keyboard/pointer controls and captures.
use super::*;
use dlms_learning_lab::{
    linux_service_router as work, linux_service_router_source as binding,
    linux_v6_workspace as router,
};
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    FileSaved(bool, bool),
    Reply(bool, bool),
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
        work::Action::Edit { text } => p.push(Op::Text("editor".into(), text.clone())),
        work::Action::Save => p.push(Op::Click("save".into())),
        work::Action::Router { action } => match action {
            router::Action::Command { line } => run(p, line),
            router::Action::Input { text } => p.push(Op::Text("command".into(), text.clone())),
            router::Action::SelectCase { index } => {
                p.push(Op::Choose("case".into(), index.to_string()))
            }
            router::Action::RunTraffic => p.push(Op::Click("run-traffic".into())),
            router::Action::Check => p.push(Op::Click("check".into())),
        },
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
        let source = labs
            .iter()
            .find(|s| s.id == "lpic2-router-service-name-recovery")
            .unwrap();
        let d = binding::source(source).unwrap();
        let shot = |name: &str, key: &str| Op::Shot(format!("services-router-{name}"), key.into());
        let mut p = vec![
            Op::Style(1., false),
            Op::Open(source.id.clone()),
            shot("introduction", "introduction"),
            shot("worked", "worked"),
            Op::Click("example".into()),
            shot("practice", "editor"),
            Op::Click("insert-command".into()),
            Op::NotRun(true),
            Op::Click("run".into()),
            Op::OutputContains("8443/tcp".into(), true),
        ];
        p.extend([
            Op::Text(
                "editor".into(),
                "preview not-a-port/tcp previews\nclock 123/udp time\n".into(),
            ),
            Op::Click("save".into()),
            Op::FileSaved(false, true),
            shot("example-syntax-error", "save-error"),
            Op::Click("worked-file-toggle".into()),
            shot("example-worked-file", "fill-file"),
            Op::Click("fill-file".into()),
            Op::FileSaved(false, true),
            Op::Uncredited,
            Op::Click("save".into()),
            Op::FileSaved(true, true),
        ]);
        run(&mut p, "getent services previews/tcp");
        p.push(Op::OutputContains("9443/tcp".into(), true));
        run(&mut p, "ip6tables -S");
        p.extend([
            Op::OutputContains("--dport 8443".into(), true),
            Op::Click("check".into()),
            Op::Check(false, true),
            shot("example-file-saved-rules-old", "failed-case"),
        ]);
        for a in d.worked.iter().skip(2) {
            translate(&mut p, a);
        }
        p.extend([
            Op::Check(true, true),
            Op::Uncredited,
            Op::Click("run-traffic".into()),
            Op::Choose("event".into(), "1".into()),
            Op::Reply(true, true),
            shot("example-tcp-reply", "event-feedback"),
            Op::Choose("case".into(), "1".into()),
            Op::Click("run-traffic".into()),
            Op::Choose("event".into(), "1".into()),
            Op::Reply(true, true),
            shot("example-udp-reply", "event-feedback"),
            Op::Check(true, true),
            Op::Click("assignment".into()),
            shot("assignment", "assignment"),
            shot("assignment-editor", "editor"),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("assignment-initial-feedback", "failed-case"),
        ]);
        let actions = source
            .solution
            .iter()
            .map(|raw| {
                serde_json::from_str::<work::Action>(raw.strip_prefix(binding::PREFIX).unwrap())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        for a in &actions[..2] {
            translate(&mut p, a);
        }
        p.extend([
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("assignment-file-only", "failed-case"),
        ]);
        for a in &actions[2..] {
            translate(&mut p, a);
        }
        p.extend([
            Op::Check(true, false),
            Op::Click("run-traffic".into()),
            Op::Choose("event".into(), "1".into()),
            Op::Reply(false, true),
            shot("assignment-tcp-reply", "event-feedback"),
            shot("assignment-complete", "result"),
            Op::Save,
            Op::Text("editor".into(), d.assignment.initial_services.clone()),
            Op::NotRun(false),
            Op::Click("check".into()),
            Op::Check(false, false),
            shot("stale-success-cleared", "failed-case"),
        ]);
        for (scale, light, name) in [(1.5, true, "light150"), (2., false, "dark200")] {
            p.extend([
                Op::Style(scale, light),
                Op::Narrow,
                shot(&format!("{name}-editor"), "editor"),
                shot(&format!("{name}-save"), "save"),
                shot(&format!("{name}-command"), "command"),
                shot(&format!("{name}-traffic"), "run-traffic"),
            ]);
        }
        p.extend([
            Op::Style(1., false),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
        p
    })
}

fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        _ => a
            .metrics
            .services_router
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::LinuxServiceRouter { simulation } = &a.progress.attempts[&s.id].state else {
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
        "command" => scope.router.input.clone(),
        "editor" => scope.editor.clone(),
        _ => panic!("Unknown connection field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Option<Vec<(bool, String)>> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let scope = scope(a, practice);
    assert!(
        scope.router.checked,
        "Click Check before observing a result"
    );
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
    fn services_router_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn services_router_tick(&mut self, ctx: &egui::Context) {
        self.services_router_flow(ctx, active_plan());
    }

    pub(super) fn services_router_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("services-router-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("SERVICES_ROUTER_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Services/router native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Services/router eligible-poll limit");
        if f.polls == 1 {
            println!("SERVICES_ROUTER_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::Reply(practice, tracked) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let d = binding::source(source).unwrap();
                let task = if practice { &d.example } else { &d.assignment };
                let scope = scope(self, practice);
                let case = &task.router.cases[scope.router.selected_case];
                let key = egui::Id::new(("v6-selected-event", &source.id, practice, &case.id));
                let event = ctx
                    .data(|data| data.get_temp::<usize>(key))
                    .expect("Actual selected event");
                let step = &scope.router.last_trial.as_ref().unwrap().steps[event];
                assert!(step.check.passed);
                let trace = step.trace.as_ref().unwrap();
                if tracked {
                    assert!(matches!(
                        case.events[event].packet,
                        router::PacketInput::Reply { .. }
                    ));
                    let observed = trace.observation.as_ref().expect("Actual tracked reply");
                    assert_eq!(
                        observed.class,
                        dlms_learning_lab::linux_v6_connections::Class::Established
                    );
                    assert_eq!(
                        observed.direction,
                        dlms_learning_lab::linux_v6_connections::Direction::Reply
                    );
                } else {
                    assert!(trace.observation.is_none());
                    assert_eq!(step.header.as_ref().unwrap().icmp_type, Some(2));
                    assert_eq!(step.header.as_ref().unwrap().icmp_code, Some(0));
                }
                true
            }
            Op::FileSaved(expected, practice) => {
                let scope = scope(self, practice);
                assert_eq!(scope.editor == scope.saved_services, expected);
                true
            }
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_services_router_reopen(ctx);
                true
            }
            Op::OutputContains(text, practice) => {
                assert!(
                    scope(self, practice).router.output.contains(&text),
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
                            .services_router
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
                            assert!(f.popup_reveals <= 20, "Services/router popup reveal limit");
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
                    let id = self
                        .metrics
                        .services_router
                        .fields
                        .iter()
                        .find(|(name, _, _)| name == &k)
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
                assert!(!s.router.checked);
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
                                "SERVICES_ROUTER_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
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
            self.services_router_parent_wheel(ctx);
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
fn services_router_plan_exercises_saved_file_old_rules_and_real_replies() {
    let p = active_plan();
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Open(_))).count(), 1);
    for pair in [(false, true), (true, true), (false, false), (true, false)] {
        assert!(p.iter().any(|o| matches!(o,Op::Check(a,b)if(*a,*b)==pair)));
    }
    assert_eq!(
        p.iter().filter(|o| matches!(o, Op::Reply(_, true))).count(),
        3
    );
    assert!(p.iter().any(|o| matches!(o, Op::FileSaved(false, true))));
    assert!(p.iter().any(|o| matches!(o, Op::FileSaved(true, true))));
    let shots = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(n, _) = o {
                Some(n.as_str())
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
    for name in [
        "services-router-introduction",
        "services-router-practice",
        "services-router-example-syntax-error",
        "services-router-example-file-saved-rules-old",
        "services-router-example-udp-reply",
        "services-router-assignment",
        "services-router-assignment-file-only",
        "services-router-assignment-complete",
        "services-router-light150-save",
        "services-router-dark200-command",
    ] {
        assert!(shots.contains(&name), "Missing {name}");
    }
    assert!(matches!(p.last(), Some(Op::Fresh)));
}
#[test]
fn services_router_actual_capture_phase_requests_an_image_without_context_reentry() {
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
    let out = ctx.run(input, |ctx| {
        let clip = ctx.content_rect();
        app.metrics.main_viewport = Some(clip);
        app.metrics.services_router.controls.push((
            "phase-probe".into(),
            egui::Rect::from_min_size(egui::pos2(30., 30.), egui::vec2(150., 40.)),
            clip,
        ));
        let key = egui::Id::new("services-router-native-flow");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Flow {
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        app.services_router_flow(ctx, &[Op::Shot("phase-probe".into(), "phase-probe".into())]);
    });
    assert!(app.capture_pending);
    assert!(
        out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
    );
}
#[test]
fn services_router_geometry_uses_actual_viewport_without_wayland_inner_rect() {
    let ctx = egui::Context::default();
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800., 600.),
        )),
        ..Default::default()
    };
    let v = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
    v.native_pixels_per_point = Some(1.);
    v.inner_rect = None;
    let _ = ctx.run(input, |ctx| {
        assert_eq!(capture_geometry(ctx).unwrap().0, egui::vec2(800., 600.))
    });
}
