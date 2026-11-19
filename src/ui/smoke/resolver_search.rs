//! Fixed client-name editor flows: native events and isolated smoke progress.
use super::*;
use dlms_learning_lab::client_names_workspace as work;
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
    fn edit(ops: &mut Vec<Op>, files: &work::Files) {
        for (name, text) in files {
            if files.len() > 1 {
                ops.extend([Op::Click("file".into()), Op::Click(format!("file:{name}"))]);
            }
            ops.push(Op::Field(name.clone(), text.clone()));
        }
    }
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| {
        matches!(
            s.id.as_str(),
            "lpic1-dns-search-suffixes" | "lpic1-dns-absolute-precedence"
        )
    }) {
        count += 1;
        let engine::Goal::ClientNames { definition } = &s.goal else {
            unreachable!()
        };
        let solution: work::Files =
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
            Op::Shot(format!("resolver-{}-practice-passed", s.id)),
        ]);
        let mut wrong = definition.example_worked.clone();
        let text = if s.id == "lpic1-dns-search-suffixes" {
            wrong["resolv.conf"].replace(" archive.workshop.lab.test", "")
        } else {
            wrong["resolv.conf"].replace("ndots:1", "ndots:0")
        };
        wrong.insert("resolv.conf".into(), text);
        edit(&mut ops, &wrong);
        ops.extend([
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Shot(format!("resolver-{}-practice-experiment", s.id)),
        ]);
        edit(&mut ops, &definition.example_worked);
        ops.extend([Op::Click("run".into()), Op::Example(true)]);
        ops.extend([
            Op::Click("assignment".into()),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("resolver-{}-starter-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("resolver-{}-passed", s.id)),
        ]);
        ops.extend([
            Op::Field("resolv.conf".into(), "nameserver 192.0.2.53\n".into()),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("resolver-{}-damage-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
        assert_ne!(definition.example_worked, definition.initial.starter);
    }
    assert_eq!(count, 2);
    ops
}
impl LabApp {
    pub(in crate::ui) fn resolver_search_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("resolver-search-native-route");
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
                "RESOLVER_SEARCH_UNIT_NATIVE_FLOW_COMPLETE: both starter/solution/damage/retry/save-read/reset flows and separate guided examples/experiments pass; no host resolver, commands or network operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("RESOLVER_SEARCH_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "ClientNames native action did not complete: {} {:?}",
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
                            .client_names
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
                            .client_names
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    };
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self.metrics.client_names.editor;
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
                    let engine::State::ClientNames { simulation } =
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
                        "Native file picker must select the requested virtual file before editing"
                    );
                    assert_eq!(
                        scope.draft[&key], value,
                        "Native paste must reach the selected virtual file"
                    );
                    true
                }
            }
            Op::Solved(expected) => {
                if self.client_names_run.is_some() {
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
                if self.client_names_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::ClientNames { definition },
                    engine::State::ClientNames { simulation },
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
                    let target = self.metrics.client_names.result.unwrap().2;
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
    fn resolver_search_native_phases_edit_the_single_excerpt_before_paste_and_run() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let source = app
            .scenarios
            .iter()
            .find(|s| s.id == "lpic1-dns-absolute-precedence")
            .unwrap()
            .clone();
        let files: work::Files =
            serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        let ops = vec![
            Op::Open(source.id.clone()),
            Op::Click("assignment".into()),
            Op::Field("resolv.conf".into(), files["resolv.conf"].clone()),
            Op::Click("run".into()),
            Op::Solved(true),
        ];
        let len = ops.len();
        let key = egui::Id::new("resolver-search-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    ..Default::default()
                },
            )
        });
        for n in 0..300 {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(2560., 1440.),
                )),
                time: Some(n as f64 / 60.),
                ..Default::default()
            };
            eframe::App::raw_input_hook(&mut app, &ctx, &mut input);
            let _ = ctx.run(input, |ctx| {
                app.render(ctx);
                app.resolver_search_lessons_tick(ctx);
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert!(progress::current_solved(
                    &app.progress.attempts[&source.id],
                    &source
                ));
                return;
            }
        }
        panic!("Native single-excerpt phases did not complete");
    }
    #[test]
    fn resolver_search_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-dns-search-suffixes")
            .unwrap();
        app.launch(i);
        if let engine::State::ClientNames { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-dns-search-suffixes")
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
        assert!(app.metrics.client_names.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("resolver-search-native-route"),
                Route {
                    ops: vec![Op::Shot("client-names-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| {
            app.resolver_search_lessons_tick(ctx)
        });
        assert!(app.capture_pending);
    }
}
