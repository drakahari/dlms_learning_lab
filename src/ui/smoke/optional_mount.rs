//! Fixed DHCP editor flows: real widget events and private smoke progress only.
use super::*;
use dlms_learning_lab::removable_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Solved(bool),
    Example(bool),
    Required(bool),
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
    let s = labs
        .iter()
        .find(|s| s.id == "lpic1-removable-startup-policy")
        .unwrap();
    let engine::Goal::RemovableMounts { definition } = &s.goal else {
        unreachable!()
    };
    let raw = s.solution[0].strip_prefix(work::PREFIX).unwrap();
    vec![
        Op::Open(s.id.clone()),
        Op::Click("example".into()),
        Op::Click("run".into()),
        Op::Example(false),
        Op::Click("case".into()),
        Op::Click("case:2".into()),
        Op::Click("insert".into()),
        Op::Example(false),
        Op::Click("run".into()),
        Op::Example(true),
        Op::Required(false),
        Op::Shot("optional-practice-expected-required-failure".into()),
        Op::Field(
            "editor".into(),
            definition
                .example_worked
                .replace("rw,auto", "rw,auto,nofail"),
        ),
        Op::Click("run".into()),
        Op::Example(false),
        Op::Required(true),
        Op::Shot("optional-practice-all-nofail-mistake".into()),
        Op::Field("editor".into(), definition.example_worked.clone()),
        Op::Click("run".into()),
        Op::Example(true),
        Op::Click("assignment".into()),
        Op::Click("run".into()),
        Op::Solved(false),
        Op::Shot("optional-assignment-starter-feedback".into()),
        Op::Field("editor".into(), raw.into()),
        Op::Click("run".into()),
        Op::Solved(true),
        Op::Click("case".into()),
        Op::Click("case:2".into()),
        Op::Required(false),
        Op::Save,
        Op::Shot("optional-assignment-expected-required-failure".into()),
        Op::Field("editor".into(), raw.replace("000000000091", "000000000099")),
        Op::Solved(false),
        Op::Click("run".into()),
        Op::Solved(false),
        Op::Shot("optional-assignment-wrong-uuid-mixed-cases".into()),
        Op::Field("editor".into(), raw.into()),
        Op::Click("run".into()),
        Op::Solved(true),
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]
}
impl LabApp {
    pub(in crate::ui) fn optional_mount_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("optional-mount-native-route");
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
                "OPTIONAL_MOUNT_UNIT_NATIVE_FLOW_COMPLETE: separate workshop/experiment/repair and independent starter/solution/expected-negative/damage/retry/save-read/reset pass; no host commands or real boot"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("OPTIONAL_MOUNT_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "RemovableMounts native action did not complete: {} {:?}",
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
                if key.starts_with("file:") || key.starts_with("case:") {
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
                            .optional_mount
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
                    let rect = if key == "reset-header" {
                        self.metrics.reset
                    } else if key == "confirm-reset" {
                        self.metrics.confirm_reset
                    } else {
                        self.metrics
                            .optional_mount
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    };
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self.metrics.optional_mount.editor;
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
                    let engine::State::RemovableMounts { simulation } =
                        &self.progress.attempts[&source.id].state
                    else {
                        unreachable!()
                    };
                    let scope = if simulation.view == teaching::View::Example {
                        &simulation.practice
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
                if self.optional_mount_run.is_some() {
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
                if self.optional_mount_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::RemovableMounts { definition },
                    engine::State::RemovableMounts { simulation },
                ) = (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(simulation.practice.passed(&definition.example), expected);
                assert_eq!(
                    simulation.assignment,
                    work::Session::new(definition).unwrap().assignment
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
                true
            }
            Op::Required(expected) => {
                assert_eq!(
                    self.metrics.optional_mount.actual_required,
                    Some(expected),
                    "Actual selected sample required-storage outcome"
                );
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
                    let target = self.metrics.optional_mount.result.unwrap().2;
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
    fn optional_mount_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-removable-startup-policy")
            .unwrap();
        app.launch(i);
        if let engine::State::RemovableMounts { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-removable-startup-policy")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            simulation.assignment.tested = Some(simulation.assignment.draft.clone());
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
        assert!(app.metrics.optional_mount.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("optional-mount-native-route"),
                Route {
                    ops: vec![Op::Shot("optional-mount-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| {
            app.optional_mount_lessons_tick(ctx)
        });
        assert!(app.capture_pending);
    }
}
