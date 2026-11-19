//! Fixed authored gateway flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::host_workspace as work;
mod ipv6;
mod direct_dns;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Solved(bool),
    Example(bool),
    Fresh,
    Save,
    Shot(String),
    ControlShot(String, String),
    Theme(u8),
    Unrun,
    Rows(Vec<bool>, bool),
    ProbeContains(&'static str),
    Challenge(bool),
    TranscriptContains(&'static str),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
    until: f64,
    began: Option<Instant>,
    next: f64,
}
fn action(ops: &mut Vec<Op>, e: &work::Event) {
    match e {
        work::Event::Command { input } => ops.extend([
            Op::Field("command".into(), input.clone()),
            Op::Click("run".into()),
        ]),
        work::Event::Resolver { text } => ops.extend([
            Op::Field("resolver".into(), text.clone()),
            Op::Click("save-resolver".into()),
        ]),
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for source in labs.iter().filter(|s|matches!(s.id.as_str(),"lpic1-host-address"|"lpic1-host-routing"|"lpic1-host-resolver"|"lpic1-host-handover")) {
        count += 1;
        let engine::Goal::HostNetwork { definition } = &source.goal else {
            unreachable!()
        };
        ops.push(Op::Open(source.id.clone()));
        ops.extend([
            Op::Click("example".into()),
            Op::Click("fill-example".into()),
            Op::Example(false),
            Op::Click("reset-example".into()),
        ]);
        for e in &definition.example_actions {
            action(&mut ops, e);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("host-{}-practice-passed", source.id)),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("host-{}-initial-feedback", source.id)),
        ]);
        for cmd in &source.solution {
            let e = serde_json::from_str(cmd.strip_prefix("host-action ").unwrap()).unwrap();
            action(&mut ops, &e);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("host-{}-passed", source.id)),
        ]);
        if source.id.ends_with("resolver") || source.id.ends_with("handover") {
            ops.extend([
                Op::Field("resolver".into(), "nameserver 203.0.113.54\n".into()),
                Op::Solved(false),
                Op::Click("save-resolver".into()),
                Op::Click("check".into()),
                Op::Solved(false),
                Op::Shot(format!("host-{}-damage-feedback", source.id)),
            ]);
            action(
                &mut ops,
                &work::Event::Resolver {
                    text: "nameserver 203.0.113.53\n".into(),
                },
            );
        } else {
            action(
                &mut ops,
                &work::Event::Command {
                    input: "ip link set dev ens4 down".into(),
                },
            );
            ops.extend([
                Op::Click("check".into()),
                Op::Solved(false),
                Op::Shot(format!("host-{}-damage-feedback", source.id)),
            ]);
            action(
                &mut ops,
                &work::Event::Command {
                    input: "ip link set dev ens4 up".into(),
                },
            );
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 4);
    ops
}
impl LabApp {
    pub(in crate::ui) fn host_lessons_tick(&mut self, ctx: &egui::Context) {
        let ipv6 = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("host-ipv6-batch");
        let direct_dns = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("direct-dns-batch");
        if ipv6 {
            let frame = ctx.cumulative_frame_nr();
            let guard = egui::Id::new("host-ipv6-native-frame");
            if ctx.data(|d| d.get_temp::<u64>(guard)) == Some(frame) {
                return;
            }
            ctx.data_mut(|d| d.insert_temp(guard, frame));
        }
        let key = egui::Id::new("host-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if direct_dns {
                    direct_dns::route(&self.scenarios)
                } else if ipv6 {
                    ipv6::route(&self.scenarios)
                } else {
                    route(&self.scenarios)
                },
                ..Default::default()
            });
        if r.done {
            if direct_dns {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            if direct_dns {
                println!("DIRECT_DNS_NATIVE_COMPLETE: actual direct queries, refusal, separate practice, unchanged independent grading, wrong-first-server failure, alternate repair, read-only verification, Save/readback and light150/dark200 controls; no host DNS");
            } else if ipv6 {
                println!(
                    "HOST_IPV6_NATIVE_COMPLETE: actual three-lab guided/adverse/retry/partial/independent/probe/current-invalidation/save-read/reset/Challenge, light150/dark200 and retained IPv4 rendering; no host network operations"
                );
            } else {
                println!(
                    "HOST_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/damage/retry/save-reload/reset flows; all4 separate guided examples pass; no host network/file operations"
                );
            }
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if !direct_dns && now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("HOST_NATIVE_STEP {} {:?}", r.at, op);
        }
        if direct_dns {
            let due = native_action_due(&mut r.frames, now, r.next, r.began.get_or_insert_with(Instant::now).elapsed());
            assert!(due.is_ok(), "Direct DNS step {}: {due:?}", r.at);
            if !due.unwrap() {
                ctx.data_mut(|d| d.insert_temp(key,r));
                ctx.request_repaint_after(std::time::Duration::from_millis(25));
                return;
            }
            r.next = now + 0.1;
        } else { r.frames += 1; }
        assert!(
            r.frames < 1200,
            "HostNetwork native action did not complete: {} {:?}",
            r.at,
            op
        );
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                if ipv6 {
                    self.scale = 1.;
                    ctx.set_zoom_factor(1.);
                    self.theme_id = theme::Preferences::default().theme;
                    theme::apply(ctx, &self.theme_id);
                }
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(key) => {
                let rect = if key == "reset-header" {
                    self.metrics.reset
                } else if key == "confirm-reset" {
                    self.metrics.confirm_reset
                } else if key == "challenge-header" {
                    self.metrics.mode_challenge
                } else if key == "learn-header" {
                    self.metrics.mode_learn
                } else {
                    self.metrics
                        .host
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Field(key, value) => {
                let field = self
                    .metrics
                    .host
                    .fields
                    .iter()
                    .find(|f| f.0 == key)
                    .map(|f| (f.1, f.2));
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native field focus {key}"
                    );
                    let modifiers = egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    };
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers,
                        });
                    }
                    self.pending_events.push(egui::Event::Paste(value));
                    r.phase = 2;
                    false
                } else {
                    if ipv6 {
                        let source = &self.scenarios[self.selected.unwrap()];
                        let attempt = &self.progress.attempts[&source.id];
                        let engine::State::HostNetwork { simulation } = &attempt.state else {
                            panic!()
                        };
                        let scope =
                            if !attempt.challenge && simulation.view == teaching::View::Example {
                                &simulation.practice
                            } else {
                                &simulation.assignment
                            };
                        let actual = match key.as_str() {
                            "command" => &scope.command,
                            "probe" => &scope.probe_ipv6_input,
                            _ => panic!("Unsupported fixed IPv6 review field"),
                        };
                        assert_eq!(actual, &value, "Actual native input must retain full text");
                    }
                    true
                }
            }
            Op::Unrun => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::HostNetwork { definition } = &source.goal else {
                    panic!()
                };
                let engine::State::HostNetwork { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    panic!()
                };
                assert_eq!(
                    simulation.assignment,
                    work::Session::new(definition).assignment
                );
                assert_eq!(simulation.practice.world, definition.example);
                assert!(simulation.practice.history.is_empty());
                true
            }
            Op::TranscriptContains(expected) => {
                let engine::State::HostNetwork { simulation } = &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id].state else { panic!() };
                let scope = if simulation.view == teaching::View::Example { &simulation.practice } else { &simulation.assignment };
                assert!(scope.transcript.last().is_some_and(|s|s.contains(expected)), "Expected actual terminal fragment {expected:?}; got {:?}",scope.transcript.last());
                true
            }
            Op::Rows(expected, practice) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::HostNetwork { definition } = &source.goal else {
                    panic!()
                };
                let engine::State::HostNetwork { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    panic!()
                };
                assert_eq!(
                    definition
                        .checks(simulation, practice)
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    expected
                );
                let passed = expected.iter().filter(|b| **b).count();
                assert_eq!(
                    self.metrics.host.result.map(|r| (r.0, r.1)),
                    Some((expected.len(), expected.len() - passed))
                );
                true
            }
            Op::ProbeContains(text) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::HostNetwork { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    panic!()
                };
                assert!(
                    simulation.assignment.probe_result.contains(text),
                    "Actual probe output {:?}",
                    simulation.assignment.probe_result
                );
                true
            }
            Op::Challenge(want) => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&source.id].challenge, want);
                assert_eq!(
                    self.metrics
                        .host
                        .controls
                        .iter()
                        .any(|c| c.0 == "introduction"),
                    !want
                );
                true
            }
            Op::Theme(mode) => {
                self.scale = match mode {
                    0 => 1.,
                    1 => 1.5,
                    _ => 2.,
                };
                ctx.set_zoom_factor(self.scale);
                self.theme_id = if mode == 1 {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(mode == 0));
                if mode != 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1100., 850.,
                    )));
                }
                self.lesson_scroll_reset = true;
                true
            }
            Op::ControlShot(name, key) => {
                if r.phase == 0 {
                    let target = if key == "reset-header" {
                        self.metrics.reset
                    } else {
                        self.metrics
                            .host
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    }
                    .expect("Actual host capture control");
                    if self.reveal_smoke_control(ctx, target) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Solved(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
                if ipv6 {
                    let engine::Goal::HostNetwork { definition } = &source.goal else {
                        panic!()
                    };
                    let engine::State::HostNetwork { simulation } =
                        &self.progress.attempts[&source.id].state
                    else {
                        panic!()
                    };
                    if let Some((total, failed, _)) = self.metrics.host.result {
                        let rows = definition.checks(simulation, false);
                        let n = rows.iter().filter(|r| r.0).count();
                        assert_eq!((total, failed), (rows.len(), rows.len() - n));
                        assert_eq!(failed == 0, expected);
                    }
                }

                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    expected,
                    "native current result {} notice={:?}",
                    source.id,
                    self.notice
                );
                true
            }
            Op::Example(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::HostNetwork { definition },
                    engine::State::HostNetwork { simulation },
                ) = (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(
                    definition.checks(simulation, true).iter().all(|r| r.0),
                    expected
                );
                assert_eq!(
                    simulation.assignment,
                    work::Session::new(definition).assignment
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
                true
            }
            Op::Fresh => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&source.id].state).unwrap(),
                    serde_json::to_value(&source.initial).unwrap()
                );
                true
            }
            Op::Save => {
                self.save();
                assert!(
                    self.storage_error.is_none(),
                    "native save {:?}",
                    self.storage_error
                );
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    let target = self.metrics.host.result.unwrap().2;
                    if self.reveal_smoke_control(ctx, target) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if complete {
            r.at += 1;
            r.frames = 0;
            r.phase = 0;
            r.began = None;
            self.started = Instant::now();
            if ipv6 {
                r.until = now + 0.06;
            }
        }
        if ipv6
            && self
                .pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15;
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-host-address")
            .unwrap();
        app.launch(i);
        if let engine::State::HostNetwork { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-host-address")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            simulation.assignment.checked = true;
        }
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.host.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("host-native-route"),
                Route {
                    ops: vec![Op::Shot("host-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.host_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}

#[cfg(test)]
mod ipv6_capture_tests {
    use super::*;
    #[test]
    fn ipv6_host_control_capture_executes_the_actual_context_safe_phase() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-host-ipv6-addition")
            .unwrap();
        a.launch(i);
        let engine::State::HostNetwork { simulation } = &mut a
            .progress
            .attempts
            .get_mut("lpic2-host-ipv6-addition")
            .unwrap()
            .state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        assert!(a.metrics.host.fields.iter().any(|f| f.0 == "probe"));
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("host-native-route"),
                Route {
                    ops: vec![Op::ControlShot(
                        "synthetic-host6-capture".into(),
                        "probe".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(Default::default(), |ctx| a.host_lessons_tick(ctx));
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn ipv6_host_route_covers_three_units_shared_scope_fields_and_scaled_capture_actions() {
        let labs = pack::ccna_pilot().unwrap();
        let ops = ipv6::route(&labs);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Open(id)if id.starts_with("lpic2-host-")))
                .count(),
            3
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Unrun)).count(), 3);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::Rows(_, true)))
                .count(),
            3
        );
        for mode in [0, 1, 2] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Theme(n)if *n==mode))
                    .count(),
                3
            );
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Open(id)if id=="lpic1-host-address"))
        );
    }
}
