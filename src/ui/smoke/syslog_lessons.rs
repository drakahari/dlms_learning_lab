//! Fixed authored Syslog flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::{syslog as core, syslog_workspace as work};
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Choice(String),
    Process(bool),
    Hosts(usize),
    Selectors(usize),
    Solved(bool),
    Example(bool),
    Fresh,
    Save,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
}
fn choose(ops: &mut Vec<Op>, control: &str, choice: String) {
    ops.extend([Op::Click(control.into()), Op::Choice(choice)]);
}
fn configure(ops: &mut Vec<Op>, action: &work::Action) {
    match action {
        work::Action::ApplySource { device, policy } => {
            choose(ops, "component", format!("source-{device}"));
            ops.push(Op::Process(policy.process_enabled));
            choose(
                ops,
                "buffer",
                format!(
                    "buffer-{}",
                    policy
                        .buffer_threshold
                        .map_or_else(|| "off".into(), |s| s.number().to_string())
                ),
            );
            choose(
                ops,
                "remote",
                format!(
                    "remote-{}",
                    policy
                        .remote_threshold
                        .map_or_else(|| "off".into(), |s| s.number().to_string())
                ),
            );
            choose(
                ops,
                "facility",
                format!("facility-{}", policy.export_facility.number()),
            );
            ops.push(Op::Hosts(policy.hosts.len()));
            for (n, host) in policy.hosts.iter().enumerate() {
                ops.push(Op::Field(format!("host-{n}"), host.to_string()));
            }
            ops.push(Op::Click("apply".into()));
        }
        work::Action::ApplyCollector { collector, policy } => {
            choose(ops, "component", format!("collector-{collector}"));
            ops.push(Op::Selectors(policy.selectors.len()));
            for (n, (facility, severity)) in policy.selectors.iter().enumerate() {
                choose(
                    ops,
                    &format!("selector-facility-{n}"),
                    format!("selector-facility-{n}-{}", facility.number()),
                );
                choose(
                    ops,
                    &format!("selector-threshold-{n}"),
                    format!("selector-threshold-{n}-{}", severity.number()),
                );
            }
            ops.push(Op::Click("apply".into()));
        }
        work::Action::Generate { device, event } => {
            choose(ops, "component", format!("source-{device}"));
            choose(ops, "event", format!("event-{}", event.as_str()));
            ops.push(Op::Click("generate".into()));
        }
        _ => panic!("unexpected fixed authored action"),
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| s.id.starts_with("ccna-syslog-")) {
        count += 1;
        ops.push(Op::Open(s.id.clone()));
        let engine::Goal::Syslog { definition } = &s.goal else {
            unreachable!()
        };
        if s.id == "ccna-syslog-local-remote" {
            ops.push(Op::Click("example".into()));
            for n in 0..definition.lesson.worked.len() {
                choose(&mut ops, "worked-select", format!("worked-{n}"));
                ops.extend([Op::Click("fill-example".into()), Op::Click("apply".into())]);
            }
            ops.extend([Op::Click("check".into()), Op::Example(true)]);
            configure(&mut ops, &definition.lesson.experiment);
            ops.extend([
                Op::Click("check".into()),
                Op::Example(false),
                Op::Shot("syslog-practice-mistake".into()),
            ]);
            configure(&mut ops, &definition.lesson.repair);
            ops.extend([Op::Click("check".into()), Op::Example(true)]);
        }
        ops.extend([
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("syslog-{}-initial", s.id)),
        ]);
        let actions = s
            .solution
            .iter()
            .map(|a| serde_json::from_str::<work::Action>(a).unwrap())
            .collect::<Vec<_>>();
        for action in &actions {
            configure(&mut ops, action);
        }
        let event = definition
            .targets
            .iter()
            .find(|t| t.local == Some(true) || t.remote.values().any(|r| r.accept))
            .expect("authored native route needs an accepted local or remote case");
        configure(
            &mut ops,
            &work::Action::Generate {
                device: event.device,
                event: event.event.clone(),
            },
        );
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("syslog-{}-passed", s.id)),
        ]);
        let damage = match &actions[0] {
            work::Action::ApplySource { device, policy } => {
                let mut policy = policy.clone();
                policy.remote_threshold = None;
                work::Action::ApplySource {
                    device: *device,
                    policy,
                }
            }
            work::Action::ApplyCollector { collector, .. } => work::Action::ApplyCollector {
                collector: *collector,
                policy: core::CollectorPolicy {
                    selectors: Default::default(),
                },
            },
            _ => unreachable!(),
        };
        configure(&mut ops, &damage);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("syslog-{}-damage", s.id)),
        ]);
        for action in &actions {
            configure(&mut ops, action);
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
    pub(in crate::ui) fn syslog_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("syslog-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "SYSLOG_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/generate/damage/retry/save-reload/reset flows; separate example passes/fails/retries"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        r.frames += 1;
        assert!(
            r.frames <= 1200,
            "Fixed native step timed out: {} {:?}",
            r.at,
            r.ops[r.at]
        );
        let step = 90000 + r.at;
        if self.smoke_stage != step {
            println!("SYSLOG_NATIVE_STEP {} {:?}", r.at, r.ops[r.at]);
        }
        self.smoke_stage = step;
        let op = r.ops[r.at].clone();
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
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
                } else {
                    self.metrics
                        .syslog
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Choice(key) => {
                let queued = ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0);
                if queued >= 2 {
                    self.click_smoke_control(ctx, None)
                } else {
                    let choice = self
                        .metrics
                        .syslog
                        .choices
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| (c.1, c.2))
                        .expect("fixed visible popup choice");
                    let (rect, clip) = choice;
                    let clip = clip.intersect(ctx.content_rect());
                    if r.phase > 0 {
                        r.phase -= 1;
                        false
                    } else if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                        self.click_smoke_control(ctx, Some(rect))
                    } else {
                        self.pending_events.extend([
                            egui::Event::PointerMoved(clip.center()),
                            egui::Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(
                                    0.,
                                    (clip.center().y - rect.center().y)
                                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                                ),
                                modifiers: Default::default(),
                            },
                        ]);
                        r.phase = 6;
                        false
                    }
                }
            }
            Op::Process(want) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::Syslog { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let practice = simulation.view == teaching::View::Example;
                let engine::Goal::Syslog { definition } = &source.goal else {
                    unreachable!()
                };
                let (_, editor) = simulation.inspect(definition, practice).unwrap();
                let work::Component::Source(ip) = editor.selected else {
                    unreachable!()
                };
                let actual = editor.sources[&ip].enabled;
                let queued = ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0)
                    > 0;
                if actual == want && !queued {
                    true
                } else {
                    let rect = self
                        .metrics
                        .syslog
                        .controls
                        .iter()
                        .find(|c| c.0 == "process")
                        .map(|c| c.1);
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Hosts(want) | Op::Selectors(want) => {
                let hosts = matches!(op, Op::Hosts(_));
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::Syslog { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let engine::Goal::Syslog { definition } = &source.goal else {
                    unreachable!()
                };
                let (_, editor) = simulation
                    .inspect(definition, simulation.view == teaching::View::Example)
                    .unwrap();
                let len = match editor.selected {
                    work::Component::Source(ip) => editor.sources[&ip].hosts.len(),
                    work::Component::Collector(ip) => editor.collectors[&ip].rows.len(),
                };
                let queued = ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0);
                if len == want && queued == 0 {
                    true
                } else if queued >= 2 {
                    self.click_smoke_control(ctx, None);
                    len == want
                } else {
                    let key = if hosts {
                        if len > want {
                            "host-remove-0"
                        } else {
                            "host-add"
                        }
                    } else if len > want {
                        "selector-remove-0"
                    } else {
                        "selector-add"
                    };
                    let rect = self
                        .metrics
                        .syslog
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1);
                    self.click_visible_smoke_control(ctx, rect);
                    false
                }
            }
            Op::Field(key, value) => {
                let field = self
                    .metrics
                    .syslog
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
                    true
                }
            }
            Op::Solved(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
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
                let (engine::Goal::Syslog { definition }, engine::State::Syslog { simulation }) =
                    (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(
                    definition.practice_checks(simulation).iter().all(|r| r.0),
                    expected
                );
                let fresh = work::Session::new(definition).unwrap();
                assert_eq!(
                    serde_json::to_value(&simulation.current).unwrap(),
                    serde_json::to_value(&fresh.current).unwrap()
                );
                assert_eq!(
                    serde_json::to_value(&simulation.editor).unwrap(),
                    serde_json::to_value(&fresh.editor).unwrap()
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
                    let target = self
                        .metrics
                        .syslog
                        .controls
                        .iter()
                        .find(|c| c.0 == "result-summary")
                        .unwrap()
                        .1;
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
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn syslog_native_route_constructs_all_four_including_remote_only_targets() {
        let scenarios = pack::ccna_pilot().unwrap();
        let ops = route(&scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 4);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Click(key) if key=="generate"))
                .count(),
            4
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::Example(false)))
                .count(),
            1
        );
        for source in scenarios
            .iter()
            .filter(|s| s.id.starts_with("ccna-syslog-"))
        {
            let engine::Goal::Syslog { definition } = &source.goal else {
                unreachable!()
            };
            let event = definition
                .targets
                .iter()
                .find(|t| t.local == Some(true) || t.remote.values().any(|r| r.accept))
                .unwrap();
            assert!(ops.iter().any(
                |o| matches!(o,Op::Choice(key) if key==&format!("event-{}",event.event.as_str()))
            ));
        }
    }

    #[test]
    fn syslog_native_capture_requests_a_real_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-syslog-local-remote")
            .unwrap();
        app.launch(i);
        if let engine::State::Syslog { simulation } = &mut app
            .progress
            .attempts
            .get_mut("ccna-syslog-local-remote")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            simulation.checked = true;
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
        assert!(
            app.metrics
                .syslog
                .controls
                .iter()
                .any(|c| c.0 == "result-summary")
        );
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("syslog-native-route"),
                Route {
                    ops: vec![Op::Shot("syslog-capture-regression".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.syslog_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
