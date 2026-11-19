//! Fixed client-name editor flows: native events and isolated smoke progress.
use super::*;
use dlms_learning_lab::client_names_workspace as work;
use std::time::Duration;
#[derive(Clone, Debug)]
enum Op {
    Style(f32, bool),
    Open(String),
    Click(String),
    Field(String, String),
    Solved(bool),
    Example(bool),
    Fresh,
    Save,
    Shot(String),
    Capture(String, String),
    Diagnostic(u8),
    DiagnosticUnsupported,
}
fn diagnostic_route(labs: &[Scenario]) -> Vec<Op> {
    let s = labs
        .iter()
        .find(|s| s.id == "lpic1-client-name-investigation")
        .expect("integrated diagnostic capstone");
    work::source(s).unwrap();
    let solution: work::Files =
        serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    let mut ops = vec![
        Op::Style(1., false),
        Op::Open(s.id.clone()),
        Op::Capture("client-diagnostic-introduction".into(), "top".into()),
        Op::Capture("client-diagnostic-worked".into(), "try".into()),
        Op::Click("example".into()),
        Op::Capture(
            "client-diagnostic-practice".into(),
            "diagnostic-input".into(),
        ),
        Op::Field("diagnostic".into(), "getent ahostsv4 preview".into()),
        Op::Click("diagnostic-run".into()),
        Op::Diagnostic(0),
        Op::Click("diagnostic-trace".into()),
        Op::Example(false),
        Op::Capture(
            "client-diagnostic-stale-alias".into(),
            "diagnostic-output".into(),
        ),
        Op::Click("insert".into()),
        Op::Click("run".into()),
        Op::Example(true),
        Op::Click("case".into()),
        Op::Click("case:1".into()),
        Op::Click("diagnostic-run".into()),
        Op::Diagnostic(0),
        Op::Capture(
            "client-diagnostic-second-sample".into(),
            "diagnostic-output".into(),
        ),
        Op::Field("diagnostic".into(), "getent ahostsv4 preview.".into()),
        Op::Click("diagnostic-run".into()),
        Op::Diagnostic(2),
        Op::Capture(
            "client-diagnostic-absolute-miss".into(),
            "diagnostic-output".into(),
        ),
        Op::Field("diagnostic".into(), "getent hosts preview".into()),
        Op::Click("diagnostic-run".into()),
        Op::DiagnosticUnsupported,
        Op::Capture(
            "client-diagnostic-unsupported".into(),
            "diagnostic-output".into(),
        ),
        Op::Click("assignment".into()),
        Op::Capture("client-diagnostic-assignment".into(), "top".into()),
        Op::Click("run".into()),
        Op::Solved(false),
        Op::Shot("client-diagnostic-initial-feedback".into()),
    ];
    for (name, text) in &solution {
        ops.extend([
            Op::Click("file".into()),
            Op::Click(format!("file:{name}")),
            Op::Field(name.clone(), text.clone()),
        ]);
    }
    ops.extend([
        Op::Click("run".into()),
        Op::Solved(true),
        Op::Save,
        Op::Shot("client-diagnostic-complete".into()),
        Op::Style(1.5, false),
        Op::Capture(
            "client-diagnostic-dark150".into(),
            "diagnostic-input".into(),
        ),
        Op::Style(2., true),
        Op::Capture(
            "client-diagnostic-light200".into(),
            "diagnostic-input".into(),
        ),
        Op::Capture(
            "client-diagnostic-light200-actions".into(),
            "diagnostic-run".into(),
        ),
        Op::Style(1., false),
        Op::Click("file".into()),
        Op::Click("file:nsswitch.conf".into()),
        Op::Field(
            "nsswitch.conf".into(),
            "hosts: files [NOTFOUND=return] dns".into(),
        ),
        Op::Solved(false),
        Op::Click("run".into()),
        Op::Solved(false),
        Op::Shot("client-diagnostic-broken-again".into()),
        Op::Click("file".into()),
        Op::Click("file:nsswitch.conf".into()),
        Op::Field("nsswitch.conf".into(), solution["nsswitch.conf"].clone()),
        Op::Click("run".into()),
        Op::Solved(true),
        Op::Save,
    ]);
    ops
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
    throttled: bool,
    next: f64,
    began: Option<Instant>,
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    fn edit(ops: &mut Vec<Op>, files: &work::Files) {
        for (name, text) in files {
            ops.extend([
                Op::Click("file".into()),
                Op::Click(format!("file:{name}")),
                Op::Field(name.clone(), text.clone()),
            ]);
        }
    }
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| {
        matches!(
            s.id.as_str(),
            "lpic1-local-host-alias" | "lpic1-name-source-fallback"
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
            Op::Shot(format!("names-{}-practice-passed", s.id)),
        ]);
        let mut wrong = definition.example_worked.clone();
        if s.id == "lpic1-local-host-alias" {
            let text = wrong["hosts"].replace(
                "studio.workshop.lab.test studio",
                "studio.workshop.lab.test studio-old",
            );
            wrong.insert("hosts".into(), text);
        } else {
            wrong.insert(
                "nsswitch.conf".into(),
                "hosts: files [NOTFOUND=return] dns\n".into(),
            );
        }
        edit(&mut ops, &wrong);
        ops.extend([
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Shot(format!("names-{}-practice-experiment", s.id)),
        ]);
        edit(&mut ops, &definition.example_worked);
        ops.extend([Op::Click("run".into()), Op::Example(true)]);
        ops.extend([
            Op::Click("assignment".into()),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("names-{}-starter-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("names-{}-passed", s.id)),
        ]);
        ops.extend([
            Op::Click("file".into()),
            Op::Click("file:nsswitch.conf".into()),
            Op::Field("nsswitch.conf".into(), "hosts: resolve files\n".into()),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("names-{}-damage-feedback", s.id)),
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
    pub(in crate::ui) fn client_names_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("client-names-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if std::env::var("DLMS_SMOKE_STAGE").as_deref()
                    == Ok("client-diagnostic-batch")
                {
                    diagnostic_route(&self.scenarios)
                } else {
                    route(&self.scenarios)
                },
                throttled: std::env::var("DLMS_SMOKE_STAGE").as_deref()
                    == Ok("client-diagnostic-batch"),
                ..Default::default()
            });
        if r.done {
            if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("client-diagnostic-batch") {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "CLIENT_NAMES_UNIT_NATIVE_FLOW_COMPLETE: selected bounded editor/diagnostic route finished; actual steps above define acceptance scope; no host resolver, commands or network operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("CLIENT_NAMES_NATIVE_STEP {} {:?}", r.at, op);
        }
        if r.throttled {
            let now = ctx.input(|i| i.time);
            let elapsed = r.began.get_or_insert_with(Instant::now).elapsed();
            let due = native_action_due(&mut r.frames, now, r.next, elapsed);
            assert!(
                due.is_ok(),
                "Client diagnostic step {} {op:?}: {due:?}",
                r.at
            );
            if !due.unwrap() {
                ctx.data_mut(|d| d.insert_temp(key, r));
                ctx.request_repaint_after(Duration::from_millis(25));
                return;
            }
            // Count eligible polls, not optimized passive paints. Give each
            // queued scroll/input event time to settle; keep20scroll/1200poll/
            // 15second limits through the existing shared helper.
            r.next = now + 0.1;
        } else {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "ClientNames native action did not complete: {} {:?}",
                r.at,
                op
            );
        }
        let complete = match op {
            Op::Style(scale, light) => {
                // A real toolbar gesture leaves the old lab control's focus.
                // Direct smoke-style changes must do the same before capture.
                if let Some(id) = ctx.memory(|m| m.focused()) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
                self.scale = scale;
                ctx.set_zoom_factor(scale);
                self.theme_id = if light {
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
                self.lesson_scroll_reset = true;
                true
            }
            Op::Diagnostic(status) => {
                assert_eq!(
                    self.metrics.client_names.diagnostic_status,
                    Some(Ok(status))
                );
                true
            }
            Op::DiagnosticUnsupported => {
                assert!(
                    matches!(&self.metrics.client_names.diagnostic_status,Some(Err(e)) if e.starts_with("Unsupported:") && e.contains("getent ahostsv4 NAME"))
                );
                true
            }
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
                let field = if key == "diagnostic" {
                    self.metrics.client_names.diagnostic_input
                } else {
                    self.metrics.client_names.editor
                };
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
                    if key == "diagnostic" {
                        assert_eq!(scope.diagnostic.as_ref().unwrap().input, value);
                    } else {
                        assert_eq!(
                            scope.selected, key,
                            "Native file picker must select the requested virtual file before editing"
                        );
                        assert_eq!(
                            scope.draft[&key], value,
                            "Native paste must reach the selected virtual file"
                        );
                    }
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
            Op::Capture(name, target) => {
                let rect = if target == "diagnostic-output" {
                    self.metrics.client_names.diagnostic_output
                } else {
                    self.metrics
                        .client_names
                        .controls
                        .iter()
                        .find(|c| c.0 == target)
                        .map(|c| c.1)
                };
                if r.phase == 0 {
                    if let Some(rect) = rect
                        && self.reveal_smoke_control(ctx, rect)
                    {
                        r.phase = 1;
                        self.started = Instant::now();
                    }
                    false
                } else if r.phase < 4 {
                    let visible = rect.is_some_and(|rect| {
                        smoke_control_visible(
                            rect,
                            self.metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point(),
                        )
                    });
                    if visible {
                        r.phase += 1;
                    } else {
                        r.phase = 0;
                    }
                    false
                } else if r.phase == 4 {
                    // Several fast paints can precede egui's smooth-scroll decay.
                    // Wait in the event loop, then recheck actual geometry.
                    if self.started.elapsed() < Duration::from_millis(300) {
                        ctx.data_mut(|d| d.insert_temp(key, r));
                        ctx.request_repaint_after(Duration::from_millis(25));
                        return;
                    }
                    if rect.is_some_and(|rect| {
                        smoke_control_visible(
                            rect,
                            self.metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point(),
                        )
                    }) {
                        println!(
                            "CLIENT_DIAGNOSTIC_CAPTURE_VISIBLE {name}: target={rect:?} viewport={:?} zoom={} scale={}",
                            self.metrics.main_viewport,
                            ctx.zoom_factor(),
                            ctx.pixels_per_point()
                        );
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name),
                        ));
                        r.phase = 5;
                    } else {
                        r.phase = 0;
                    }
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
            r.next = 0.;
            r.began = None;
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
    fn client_diagnostic_passive_release_paints_keep_poll_and_scroll_budgets() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("client-names-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Capture("passive".into(), "top".into())],
                    frames: 7,
                    throttled: true,
                    next: 10.,
                    began: Some(Instant::now()),
                    ..Default::default()
                },
            )
        });
        for _ in 0..100 {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(9.),
                    ..Default::default()
                },
                |ctx| app.client_names_lessons_tick(ctx),
            );
        }
        let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
        assert_eq!((route.frames, route.at, route.phase), (7, 0, 0));
        assert!(app.pending_events.is_empty());
        assert!(
            ctx.data(|d| d.get_temp::<usize>(egui::Id::new(("bounded-native-scroll", 90000usize))))
                .is_none()
        );
    }
    #[test]
    fn client_diagnostic_style_capture_releases_the_previous_native_control_focus() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let id = egui::Id::new("existing-control-before-style");
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut text = String::new();
                ui.add(egui::TextEdit::singleline(&mut text).id(id))
                    .request_focus();
            });
        });
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        let key = egui::Id::new("client-names-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Style(2., true)],
                    throttled: true,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| {
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            app.client_names_lessons_tick(ctx);
            assert!(ctx.memory(|m| m.focused()).is_none());
        });
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at, 1);
    }
    #[test]
    fn client_diagnostic_native_phases_edit_and_run_the_actual_command_field() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let ops = vec![
            Op::Open("lpic1-client-name-investigation".into()),
            Op::Click("assignment".into()),
            Op::Field("diagnostic".into(), "getent ahostsv4 report".into()),
            Op::Click("diagnostic-run".into()),
            Op::Diagnostic(0),
            Op::Solved(false),
        ];
        let count = ops.len();
        let key = egui::Id::new("client-names-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    throttled: true,
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
                app.client_names_lessons_tick(ctx);
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == count {
                return;
            }
        }
        panic!("Actual diagnostic input/run route did not finish");
    }
    #[test]
    fn client_diagnostic_capture_phase_requests_an_actual_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = false;
        let index = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-client-name-investigation")
            .unwrap();
        app.launch(index);
        for _ in 0..3 {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(2560., 1440.),
                )),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| app.render(ctx));
        }
        let key = egui::Id::new("client-names-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Capture(
                        "client-diagnostic-capture-regression".into(),
                        "top".into(),
                    )],
                    phase: 4,
                    throttled: true,
                    ..Default::default()
                },
            )
        });
        app.started = Instant::now() - Duration::from_secs(1);
        let output = ctx.run(Default::default(), |ctx| app.client_names_lessons_tick(ctx));
        assert!(app.capture_pending);
        assert!(
            output
                .viewport_output
                .values()
                .flat_map(|v| &v.commands)
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        );
    }
    #[test]
    fn client_names_native_phases_choose_the_second_file_before_paste_and_run() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let source = app
            .scenarios
            .iter()
            .find(|s| s.id == "lpic1-name-source-fallback")
            .unwrap()
            .clone();
        let files: work::Files =
            serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        let ops = vec![
            Op::Open(source.id.clone()),
            Op::Click("assignment".into()),
            Op::Click("file".into()),
            Op::Click("file:nsswitch.conf".into()),
            Op::Field("nsswitch.conf".into(), files["nsswitch.conf"].clone()),
            Op::Click("run".into()),
            Op::Solved(true),
        ];
        let len = ops.len();
        let key = egui::Id::new("client-names-native-route");
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
                app.client_names_lessons_tick(ctx);
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert!(progress::current_solved(
                    &app.progress.attempts[&source.id],
                    &source
                ));
                return;
            }
        }
        panic!("Native second-file phases did not complete");
    }
    #[test]
    fn client_names_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-local-host-alias")
            .unwrap();
        app.launch(i);
        if let engine::State::ClientNames { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-local-host-alias")
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
                egui::Id::new("client-names-native-route"),
                Route {
                    ops: vec![Op::Shot("client-names-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.client_names_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
