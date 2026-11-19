//! Fixed startup-file editor flows: real widget events and private smoke progress only.
use super::*;
use dlms_learning_lab::startup_workspace as work;
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
    settle_until: f64,
    began: Option<f64>,
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    fn edit(ops: &mut Vec<Op>, files: &dlms_learning_lab::shell::startup::Files) {
        for (name, file) in &files.0 {
            ops.extend([
                Op::Click("file".into()),
                Op::Click(format!("file:{name}")),
                Op::Field(name.clone(), file.text.clone()),
            ]);
        }
    }
    let mut ops = vec![];
    let mut count = 0;
    for s in labs
        .iter()
        .filter(|s| s.unit == "linux-shell-startup-files")
    {
        count += 1;
        let engine::Goal::StartupFiles { definition } = &s.goal else {
            unreachable!()
        };
        let solution: dlms_learning_lab::shell::startup::Files =
            serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        ops.extend([
            Op::Open(s.id.clone()),
            Op::Click("example".into()),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Click("case".into()),
            Op::Click("case:1".into()),
            Op::Click("insert".into()),
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Shot(format!("startup-{}-practice-passed", s.id)),
        ]);
        let (file, broken) = if s.id.ends_with("priority") {
            ("~/.bash_profile", String::new())
        } else {
            (
                "/work/startup.env",
                format!(
                    "printf 'Ready\\n'\n{}",
                    definition.example_worked.0["/work/startup.env"].text
                ),
            )
        };
        ops.extend([
            Op::Click("file".into()),
            Op::Click(format!("file:{file}")),
            Op::Field(file.into(), broken),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Shot(format!("startup-{}-practice-mistake", s.id)),
            Op::Click("insert".into()),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Click("assignment".into()),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("startup-{}-starter-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("startup-{}-passed", s.id)),
        ]);
        let file = if s.id.ends_with("priority") {
            "~/.bash_profile"
        } else {
            "/work/startup.env"
        };
        ops.extend([
            Op::Click("file".into()),
            Op::Click(format!("file:{file}")),
            Op::Field(file.into(), "BROKEN='".into()),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("startup-{}-syntax-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
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
    pub(in crate::ui) fn startup_files_lessons_tick(&mut self, ctx: &egui::Context) {
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let rootkey = egui::Id::new("startup-files-native-root");
        if ctx.data(|d| d.get_temp::<u64>(rootkey)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(rootkey, root));
        let key = egui::Id::new("startup-files-native-route");
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
                "STARTUP_FILES_UNIT_NATIVE_FLOW_COMPLETE: two actual startup/child repairs, separate practice, wrong/partial/valid cases, damage/retry/save-read/reset; no host profiles or execution"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let began = *r.began.get_or_insert(now);
        assert!(
            now - began < 20.,
            "Startup native timeout {op:?} phase {}",
            r.phase
        );
        if now < r.settle_until {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(10));
            return;
        }
        r.settle_until = now + 0.01;
        if r.frames == 0 {
            println!("STARTUP_FILES_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "StartupFiles native action did not complete: {} {:?}",
            r.at,
            op
        );
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;

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
                            .startup_files
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
                            .startup_files
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    };
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self.metrics.startup_files.editor;
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
                    if value.is_empty() {
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Backspace,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    } else {
                        self.pending_events.push(egui::Event::Paste(value));
                    }
                    r.phase = 2;
                    false
                } else {
                    let source = &self.scenarios[self.selected.unwrap()];
                    let engine::State::StartupFiles { simulation } =
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
                        scope.selected, key,
                        "Native file picker must select the requested profile before editing"
                    );
                    assert_eq!(
                        scope.files.0[&key].text, value,
                        "Native paste must reach the selected profile"
                    );
                    true
                }
            }
            Op::Solved(expected) => {
                if self.startup_files_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    expected,
                    "native current result {} notice={:?} checks={:?} state={:?}",
                    source.id,
                    self.notice,
                    engine::checks(&self.progress.attempts[&source.id].state, &source.goal),
                    self.progress.attempts[&source.id].state
                );
                true
            }
            Op::Example(expected) => {
                if self.startup_files_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::StartupFiles { definition },
                    engine::State::StartupFiles { simulation },
                ) = (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(simulation.practice.passed(&definition.example), expected);
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
                    let label = self.metrics.startup_files.result.unwrap().2;
                    let line = ctx.style().text_styles[&egui::TextStyle::Body].size;
                    let target = egui::Rect::from_min_max(
                        label.min,
                        egui::pos2(label.max.x, label.max.y + line * 8.),
                    );
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
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.settle_until = now + 0.15;
        }
        if complete {
            r.began = None;
            r.at += 1;
            r.frames = 0;
            r.phase = 0;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint_after(std::time::Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_native_recipe_expectations_are_actual_and_capture_does_not_reenter_context() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&app.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        let mut current: Option<(work::Definition, work::Session)> = None;
        for op in ops {
            match op {
                Op::Open(id) => {
                    let s = app.scenarios.iter().find(|s| s.id == id).unwrap();
                    let engine::Goal::StartupFiles { definition } = &s.goal else {
                        panic!()
                    };
                    current = Some(((**definition).clone(), work::Session::new(definition)));
                }
                Op::Click(key) => {
                    let (d, s) = current.as_mut().unwrap();
                    match key.as_str() {
                        "example" => s.view = teaching::View::Example,
                        "assignment" => s.view = teaching::View::Assignment,
                        "insert" => {
                            s.practice.replace(&d.example, &d.example_worked).unwrap();
                        }
                        "run" => {
                            if s.view == teaching::View::Example {
                                s.practice.run(&d.example).unwrap();
                            } else {
                                s.assignment.run(&d.initial).unwrap();
                            }
                        }
                        "confirm-reset" => *s = work::Session::new(d),
                        _ => {}
                    }
                }
                Op::Field(file, text) => {
                    let (d, s) = current.as_mut().unwrap();
                    if s.view == teaching::View::Example {
                        s.practice.edit(&d.example, &file, &text).unwrap();
                    } else {
                        s.assignment.edit(&d.initial, &file, &text).unwrap();
                    }
                }
                Op::Example(expected) => {
                    let (d, s) = current.as_ref().unwrap();
                    assert_eq!(s.practice.passed(&d.example), expected);
                    assert!(!d.solved(s));
                }
                Op::Solved(expected) => {
                    let (d, s) = current.as_ref().unwrap();
                    assert_eq!(d.solved(s), expected);
                }
                _ => {}
            }
        }
        // Exercise actual screenshot-request phase and context use, not only recipe lists.
        app.smoke = true;
        app.capture_pending = false;
        let key = egui::Id::new("startup-files-native-route");
        let mut requested = false;
        let _ = ctx.run(Default::default(), |ctx| {
            ctx.data_mut(|d| {
                d.insert_temp(
                    key,
                    Route {
                        ops: vec![Op::Shot("startup-capture-regression".into())],
                        phase: 1,
                        ..Default::default()
                    },
                )
            });
            app.startup_files_lessons_tick(ctx);
            requested = app.capture_pending;
        });
        assert!(requested);
    }
}
