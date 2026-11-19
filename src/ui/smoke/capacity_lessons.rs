//! Fixed authored gateway flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::capacity_workspace as work;
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
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
}
fn action(ops: &mut Vec<Op>, a: &work::Action) {
    match a {
        work::Action::Put { path, bytes } => ops.extend([
            Op::Field("path".into(), path.clone()),
            Op::Field("bytes".into(), bytes.to_string()),
            Op::Click("put".into()),
        ]),
        work::Action::Remove { names } => {
            for name in names {
                ops.push(Op::Click(format!("select-{name}")));
            }
            ops.extend([Op::Click("preview".into()), Op::Click("remove".into())]);
        }
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| s.id.starts_with("lpic2-capacity-")) {
        count += 1;
        let engine::Goal::Capacity { definition } = &s.goal else {
            unreachable!()
        };
        ops.push(Op::Open(s.id.clone()));
        ops.extend([
            Op::Click("example".into()),
            Op::Click("fill-example".into()),
            Op::Example(false),
            Op::Click("reset-example".into()),
        ]);
        for a in &definition.example_actions {
            action(&mut ops, a);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("capacity-{}-practice-passed", s.id)),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("capacity-{}-initial-feedback", s.id)),
        ]);
        for cmd in &s.solution {
            let a = serde_json::from_str(cmd.strip_prefix("capacity-action ").unwrap()).unwrap();
            action(&mut ops, &a);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("capacity-{}-passed", s.id)),
            Op::Field("bytes".into(), "not a number".into()),
            Op::Solved(false),
            Op::Click("put".into()),
            Op::Click("check".into()),
            Op::Shot(format!("capacity-{}-mistake-feedback", s.id)),
        ]);
        let repair = definition
            .requirements
            .iter()
            .rev()
            .find_map(|r| match r {
                work::Requirement::File { path, bytes } => Some(work::Action::Put {
                    path: path.clone(),
                    bytes: *bytes,
                }),
                _ => None,
            })
            .unwrap();
        action(&mut ops, &repair);
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
    pub(in crate::ui) fn capacity_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("capacity-native-route");
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
                "CAPACITY_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/draft-damage/retry/save-reload/reset flows; all4 separate guided examples pass; no host disk operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("CAPACITY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Capacity native action did not complete: {} {:?}",
            r.at,
            op
        );
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
                        .capacity
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
                    .capacity
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
                let (engine::Goal::Capacity { definition }, engine::State::Capacity { simulation }) =
                    (&source.goal, &self.progress.attempts[&source.id].state)
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
                    let target = self.metrics.capacity.result.unwrap().2;
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
    fn capacity_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-capacity-inodes")
            .unwrap();
        app.launch(i);
        if let engine::State::Capacity { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-capacity-inodes")
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
        assert!(app.metrics.capacity.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("capacity-native-route"),
                Route {
                    ops: vec![Op::Shot("capacity-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.capacity_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
