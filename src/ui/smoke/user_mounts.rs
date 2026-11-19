//! Fixed actor-aware mount flows: real widget events and private smoke progress only.
use super::*;
use dlms_learning_lab::user_mounts_workspace as work;
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
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    fn actions(ops: &mut Vec<Op>, actions: &[work::Action]) {
        for a in actions {
            match a {
                work::Action::Save { configuration } => ops.extend([
                    Op::Field("editor".into(), configuration.clone()),
                    Op::Click("save".into()),
                ]),
                work::Action::Mount { actor, target } | work::Action::Unmount { actor, target } => {
                    ops.extend([
                        Op::Click("actor".into()),
                        Op::Click(format!("actor:{actor:?}")),
                        Op::Click("target".into()),
                        Op::Click(format!("target:{target}")),
                        Op::Click(
                            if matches!(a, work::Action::Mount { .. }) {
                                "mount"
                            } else {
                                "unmount"
                            }
                            .into(),
                        ),
                    ])
                }
            }
        }
    }
    for s in labs.iter().filter(|s| s.unit == "linux-removable-callers") {
        count += 1;
        let engine::Goal::UserMounts { definition } = &s.goal else {
            unreachable!()
        };
        ops.extend([Op::Open(s.id.clone()), Op::Click("example".into())]);
        actions(&mut ops, &definition.example_actions);
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("user-mounts-{}-practice-pass", s.id)),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("user-mounts-{}-starter-feedback", s.id)),
        ]);
        let solution = s
            .solution
            .iter()
            .map(|c| {
                serde_json::from_str::<work::Action>(c.strip_prefix(work::PREFIX).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();
        actions(&mut ops, &solution[..1]);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("user-mounts-{}-saved-not-applied", s.id)),
        ]);
        actions(&mut ops, &solution[1..]);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("user-mounts-{}-assignment-pass", s.id)),
            Op::Field("editor".into(), "invalid configuration\n".into()),
            Op::Solved(false),
            Op::Click("save".into()),
            Op::Click("check".into()),
            Op::Solved(false),
        ]);
        actions(&mut ops, &solution[..1]);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 2);
    ops
}
impl LabApp {
    pub(in crate::ui) fn user_mounts_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("user-mounts-native-route");
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
                "USER_MOUNTS_UNIT_NATIVE_FLOW_COMPLETE: two distinct workshop/session repair and assignment starter/save-only failure/actual mount/caller handover/damage/retry/save-read/reset flows pass; no host mount"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("USER_MOUNTS_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "UserMounts native action did not complete: {} {:?}",
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
                if key.starts_with("actor:") || key.starts_with("target:") {
                    let queued = ctx
                        .data(|d| {
                            d.get_temp::<u8>(egui::Id::new((
                                "native-click-phases",
                                self.smoke_stage,
                            )))
                        })
                        .unwrap_or(0);
                    if queued >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let (_, rect, clip) = self
                            .metrics
                            .user_mounts
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .expect("Open popup choice");
                        let (rect, clip) = (*rect, clip.intersect(ctx.content_rect()));
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
                                        (clip.center().y - rect.center().y).clamp(
                                            -clip.height().max(0.) * 0.75,
                                            clip.height().max(0.) * 0.75,
                                        ),
                                    ),
                                    modifiers: Default::default(),
                                },
                            ]);
                            r.phase = 6;
                            false
                        }
                    }
                } else {
                    let rect = if key == "example" {
                        self.metrics.example_action
                    } else if key == "assignment" {
                        self.metrics.assignment_action
                    } else if key == "reset-header" {
                        self.metrics.reset
                    } else if key == "confirm-reset" {
                        self.metrics.confirm_reset
                    } else {
                        self.metrics
                            .user_mounts
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    };
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self.metrics.user_mounts.editor;
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
                    let source = &self.scenarios[self.selected.unwrap()];
                    let engine::State::UserMounts { simulation } =
                        &self.progress.attempts[&source.id].state
                    else {
                        unreachable!()
                    };
                    let scope = if simulation.view == teaching::View::Example {
                        &simulation.example
                    } else {
                        &simulation.assignment
                    };
                    assert_eq!(
                        scope.draft, value,
                        "Native paste must reach the current draft"
                    );
                    true
                }
            }
            Op::Solved(expected) => {
                if self.user_mounts_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
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
                if self.user_mounts_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::UserMounts { definition },
                    engine::State::UserMounts { simulation },
                ) = (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(simulation.example.solved(&definition.example), expected);
                assert_eq!(
                    simulation.assignment,
                    work::Session::new((**definition).clone()).assignment
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
                    let target = self.metrics.user_mounts.result.unwrap();
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
    fn user_mounts_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-removable-personal-session")
            .unwrap();
        app.launch(i);
        if let engine::State::UserMounts { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-removable-personal-session")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            let record =
                work::run(&simulation.definition.assignment, &simulation.assignment).unwrap();
            simulation
                .assignment
                .store(record, &simulation.definition.assignment);
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
        assert!(app.metrics.user_mounts.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("user-mounts-native-route"),
                Route {
                    ops: vec![Op::Shot("user-mounts-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.user_mounts_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
