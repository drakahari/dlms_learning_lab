//! Fixed authored gateway flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::repository_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Enabled(bool),
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
fn action(ops: &mut Vec<Op>, event: &work::Event) {
    match event {
        work::Event::Edit {
            edit:
                work::Edit::Settings {
                    alias,
                    enabled,
                    uri,
                },
        } => ops.extend([
            Op::Click(format!("edit-{alias}")),
            Op::Enabled(*enabled),
            Op::Field("uri".into(), uri.clone()),
            Op::Click("apply".into()),
        ]),
        work::Event::Edit {
            edit: work::Edit::Refresh { alias },
        } => ops.extend([
            Op::Click(format!("edit-{alias}")),
            Op::Click("refresh".into()),
        ]),
        work::Event::Query { package } => ops.extend([
            Op::Field("package".into(), package.clone()),
            Op::Click("query".into()),
        ]),
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for source in labs.iter().filter(|s| s.id.starts_with("lpic1-repo-")) {
        count += 1;
        let engine::Goal::Repositories { definition } = &source.goal else {
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
            Op::Shot(format!("repository-{}-practice-passed", source.id)),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("repository-{}-initial-feedback", source.id)),
        ]);
        for cmd in &source.solution {
            let e = serde_json::from_str(cmd.strip_prefix("repository-action ").unwrap()).unwrap();
            action(&mut ops, &e);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("repository-{}-passed", source.id)),
            Op::Field(
                "uri".into(),
                "file:///virtual-repositories/unprovided".into(),
            ),
            Op::Solved(false),
            Op::Click("apply".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("repository-{}-damage-feedback", source.id)),
        ]);
        let first = definition
            .initial
            .inspect_all(&definition.initial_source)
            .unwrap()[0]
            .configuration;
        action(
            &mut ops,
            &work::Event::Edit {
                edit: work::Edit::Settings {
                    alias: first.alias.clone(),
                    enabled: true,
                    uri: definition.initial_source.endpoints[0].uri.clone(),
                },
            },
        );
        for cmd in &source.solution {
            let e = serde_json::from_str(cmd.strip_prefix("repository-action ").unwrap()).unwrap();
            action(&mut ops, &e);
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
    pub(in crate::ui) fn repository_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("repository-native-route");
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
                "REPOSITORY_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/draft-damage/applied-damage/retry/save-reload/reset flows; all4 separate guided examples pass; no host package/network operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("REPOSITORY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Repositories native action did not complete: {} {:?}",
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
                        .repository
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
                    .repository
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
            Op::Enabled(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::Repositories { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let actual = if simulation.view == teaching::View::Example {
                    simulation.practice.enabled
                } else {
                    simulation.assignment.enabled
                };
                if actual == expected {
                    true
                } else {
                    let rect = self
                        .metrics
                        .repository
                        .controls
                        .iter()
                        .find(|c| c.0 == "enabled")
                        .map(|c| c.1);
                    self.click_visible_smoke_control(ctx, rect)
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
                let (
                    engine::Goal::Repositories { definition },
                    engine::State::Repositories { simulation },
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
                    let target = self.metrics.repository.result.unwrap().2;
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
    fn repository_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-repo-visibility")
            .unwrap();
        app.launch(i);
        if let engine::State::Repositories { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-repo-visibility")
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
        assert!(app.metrics.repository.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("repository-native-route"),
                Route {
                    ops: vec![Op::Shot("repository-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.repository_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
