//! Fixed authored gateway flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::{hsrp as core, hsrp_workspace as work};
#[derive(Clone)]
enum Op {
    Open(String),
    Click(String),
    Toggle(String, bool),
    Field(String, String),
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
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for s in labs.iter().filter(|s| s.id.starts_with("ccna-hsrp-")) {
        ops.push(Op::Open(s.id.clone()));
        if s.id == "ccna-hsrp-virtual-gateway" {
            let engine::Goal::Hsrp { definition } = &s.goal else {
                unreachable!()
            };
            ops.push(Op::Click("example".into()));
            for n in 0..definition.lesson.worked.len() {
                ops.extend([
                    Op::Click("worked-picker".into()),
                    Op::Click(format!("worked-choice-{n}")),
                    Op::Click("insert".into()),
                    Op::Click("apply".into()),
                ]);
            }
            ops.extend([
                Op::Click("check".into()),
                Op::Example(true),
                Op::Shot("gateway-guided-practice-passed".into()),
            ]);
            configure(&mut ops, &definition.lesson.experiment);
            ops.extend([
                Op::Click("check".into()),
                Op::Example(false),
                Op::Shot("gateway-guided-practice-experiment".into()),
            ]);
            configure(&mut ops, &definition.lesson.repair);
            ops.extend([Op::Click("check".into()), Op::Example(true)]);
        }
        ops.extend([
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("gateway-{}-initial-feedback", s.id)),
        ]);
        let actions = s
            .solution
            .iter()
            .map(|line| {
                serde_json::from_str::<work::Action>(line.strip_prefix(work::PREFIX).unwrap())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        for action in &actions {
            configure(&mut ops, action);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("gateway-{}-passed", s.id)),
        ]);
        let engine::Goal::Hsrp { definition } = &s.goal else {
            unreachable!()
        };
        let damage = if let Some(client) = definition.policy.editable_clients.first() {
            work::Action::Gateway {
                client: *client,
                gateway: definition.policy.initial.router(core::RouterId::A).address,
            }
        } else {
            let mut config = actions
                .iter()
                .find_map(|action| match action {
                    work::Action::Router {
                        router: core::RouterId::A,
                        config,
                    } => Some(config.clone()),
                    _ => None,
                })
                .unwrap();
            config.preempt = false;
            work::Action::Router {
                router: core::RouterId::A,
                config,
            }
        };
        configure(&mut ops, &damage);
        ops.extend([
            Op::Solved(false),
            Op::Click("check".into()),
            Op::Shot(format!("gateway-{}-damage-feedback", s.id)),
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
    assert_eq!(
        labs.iter()
            .filter(|s| s.id.starts_with("ccna-hsrp-"))
            .count(),
        4
    );
    ops
}
fn configure(ops: &mut Vec<Op>, action: &work::Action) {
    match action {
        work::Action::Gateway { client, gateway } => {
            ops.push(Op::Field(format!("client-{client}"), gateway.to_string()))
        }
        work::Action::Router { router, config } => {
            ops.push(Op::Toggle(
                format!("router-{router:?}-participating"),
                config.participating,
            ));
            ops.push(Op::Field(
                format!("router-{router:?}-priority"),
                config.priority.to_string(),
            ));
            ops.push(Op::Toggle(
                format!("router-{router:?}-preempt"),
                config.preempt,
            ));
            ops.push(Op::Toggle(
                format!("router-{router:?}-track"),
                config.track_wan_decrement.is_some(),
            ));
            if let Some(n) = config.track_wan_decrement {
                ops.push(Op::Field(
                    format!("router-{router:?}-decrement"),
                    n.to_string(),
                ));
            }
        }
    }
    ops.push(Op::Click("apply".into()));
}
impl LabApp {
    pub(in crate::ui) fn gateway_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("gateway-native-route");
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
                "GATEWAY_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/damage/retry/save-reload/reset flows; separate guided example passes/fails/retries"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
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
                        .hsrp
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Toggle(key, want) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::Hsrp { simulation } = &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let ix = usize::from(key.contains("-B-"));
                let draft = if simulation.view == teaching::View::Example {
                    &simulation.example_draft.routers[ix]
                } else {
                    &simulation.draft.routers[ix]
                };
                let actual = if key.ends_with("participating") {
                    draft.participating
                } else if key.ends_with("preempt") {
                    draft.preempt
                } else {
                    draft.track
                };
                // Complete an already queued click even after its state has changed.
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
                        .hsrp
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1);
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self
                    .metrics
                    .hsrp
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
                let (engine::Goal::Hsrp { definition }, engine::State::Hsrp { simulation }) =
                    (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(
                    definition.practice_checks(simulation).iter().all(|r| r.0),
                    expected
                );
                assert_eq!(simulation.current, definition.policy.initial);
                assert!(simulation.history.is_empty());
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
                    let target = self.metrics.hsrp.summary.unwrap().2;
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
    fn gateway_batch_capture_phase_is_context_safe_and_requests_a_real_screenshot() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-hsrp-virtual-gateway")
            .unwrap();
        app.launch(i);
        let source = app.scenarios[i].clone();
        for request in [
            work::binding::Change::View(teaching::View::Assignment),
            work::binding::Change::Check { practice: false },
        ] {
            work::binding::change(
                app.progress.attempts.get_mut(&source.id).unwrap(),
                &source,
                request,
            )
            .unwrap();
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
        assert!(app.metrics.hsrp.summary.is_some());
        let route = Route {
            ops: vec![Op::Shot("gateway-regression-capture".into())],
            phase: 1,
            ..Default::default()
        };
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("gateway-native-route"), route));
        let _ = ctx.run(Default::default(), |ctx| app.gateway_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
