//! Fixed native terminal actions with private smoke progress; no host authentication.
use super::*;
use dlms_learning_lab::aging_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String),
    Grade {
        practice: bool,
        passed: Option<bool>,
    },
    Error,
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
    style_rect: Option<egui::Rect>,
    stable_frames: usize,
    reveal_after: f64,
    action_started: Option<Instant>,
}
pub(in crate::ui) const IDS: [&str; 3] = [
    "lpic1-aging-rotation",
    "lpic1-aging-grace",
    "lpic1-aging-contract-reset",
];
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for (id, partial, good, broken, retry) in [
        (
            IDS[0],
            "chage -M 45 morgan",
            "chage --maxdays=45 --warndays=10 morgan",
            "chage -W 0 morgan",
            "chage -W 10 morgan",
        ),
        (
            IDS[1],
            "chage -I 6 river",
            "chage --inactive=7 river",
            "chage -I 0 river",
            "chage -I 7 river",
        ),
        (
            IDS[2],
            "chage -E 2026-10-21 sage",
            "chage --expiredate=2026-10-20 sage",
            "chage -d -1 sage",
            "chage -d 0 sage",
        ),
    ] {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        let definition = work::source(source).unwrap();
        ops.extend([
            Op::Style(1., false, false),
            Op::Open(id.into()),
            Op::Shot(format!("{id}-introduction")),
            Op::Click("example".into()),
            Op::Click("query".into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-example-wrong")),
            Op::Field(definition.example_worked[0].0.clone()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(true),
            },
            Op::Shot(format!("{id}-example-passed")),
            Op::Click("assignment".into()),
            Op::Grade {
                practice: false,
                passed: None,
            },
            Op::Shot(format!("{id}-assignment-unchecked")),
            Op::Field(partial.into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-assignment-partial")),
            Op::Field(good.into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Shot(format!("{id}-assignment-passed")),
            Op::Field(broken.into()),
            Op::Click("run".into()),
            Op::Grade {
                practice: false,
                passed: None,
            },
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-assignment-broken")),
            Op::Field(retry.into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Save,
        ]);
        if id == IDS[0] {
            ops.extend([
                Op::Field("chage -M 45 --prefix=/ morgan".into()),
                Op::Click("run".into()),
                Op::Error,
                Op::Grade {
                    practice: false,
                    passed: None,
                },
                Op::Shot("aging-unsupported".into()),
                Op::Click("check".into()),
                Op::Grade {
                    practice: false,
                    passed: Some(true),
                },
                Op::Style(2., true, true),
                Op::Shot("aging-light200-narrow".into()),
                Op::Style(1.5, false, false),
                Op::Shot("aging-dark150-wide".into()),
                Op::Style(1., false, false),
            ]);
        }
        ops.extend([
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    ops
}
impl LabApp {
    pub(in crate::ui) fn aging_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("aging-native-route");
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
                "AGING_NATIVE_FLOW_COMPLETE: three substantive virtual aging assignments; separate practice, partial/wrong/alternate/unsupported/retry/save-read/reset; no host accounts or authentication"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 94000 + r.at;
        let op = r.ops[r.at].clone();
        let began = *r.action_started.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed().as_secs() < 15,
            "Aging native action exceeded15seconds: {} {:?}",
            r.at,
            op
        );
        // Optimized rendering can yield thousands of callbacks during a short
        // asynchronous settle delay. Waiting consumes time, not a work attempt.
        // The independent wall deadline still bounds even a stalled egui clock.
        if matches!(&op, Op::Shot(_) if (r.phase==0 && ctx.input(|i|i.time)<r.reveal_after)
            || (r.phase==2 && self.capture_pending))
        {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        if r.frames == 0 {
            println!("AGING_NATIVE_STEP {} {:?}", r.at, op);
        }
        // A resize acknowledgement still needs geometry observations, but
        // those observations are waiting rather than new work attempts.
        if !matches!(&op, Op::Style(..) if r.phase==1) {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "Aging native action did not complete: {} {:?}",
                r.at,
                op
            );
        }
        let complete = match op {
            Op::Style(scale, light, narrow) => {
                if r.phase == 0 {
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1268., 1308.,
                        )));
                    }
                    self.lesson_scroll_reset = true;
                    r.phase = 1;
                } else {
                    let rect = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let dimensions = if narrow {
                        (rect.width() * ppp - 1268.).abs() <= 4.
                            && (rect.height() * ppp - 1308.).abs() <= 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if dimensions
                        && (ctx.zoom_factor() - scale).abs() < 0.01
                        && r.style_rect == Some(rect)
                    {
                        r.stable_frames += 1;
                    } else {
                        r.stable_frames = 0;
                    }
                    r.style_rect = Some(rect);
                    if r.stable_frames >= 2 {
                        r.phase = 2;
                    }
                }
                if r.phase != 2 {
                    assert!(
                        self.started.elapsed().as_secs() < 15,
                        "Aging native window geometry did not settle"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint_after(std::time::Duration::from_millis(25));
                    return;
                }
                true
            }

            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(name) => {
                let rect = if name == "reset-header" {
                    self.metrics.reset
                } else if name == "confirm-reset" {
                    self.metrics.confirm_reset
                } else {
                    self.metrics
                        .aging
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Field(value) => {
                let field = self.metrics.aging.input;
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native aging command field focus"
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
                    let engine::State::Aging { simulation } =
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
                        scope.input, value,
                        "Native paste must reach the actual aging command field"
                    );
                    true
                }
            }
            Op::Grade { practice, passed } => {
                let source = &self.scenarios[self.selected.unwrap()];
                let definition = work::source(source).unwrap();
                let engine::State::Aging { simulation } = &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let (task, scope) = if practice {
                    (&definition.example, &simulation.practice)
                } else {
                    (&definition.initial, &simulation.assignment)
                };
                assert_eq!(scope.checked, passed.is_some());
                if let Some(expected) = passed {
                    let rows = task.checks(&scope.world);
                    let count = rows.iter().filter(|r| r.0).count();
                    assert_eq!(
                        self.metrics.aging.result.map(|r| (r.0, r.1)),
                        Some((count, rows.len()))
                    );
                    assert_eq!(
                        count == rows.len(),
                        expected,
                        "Actual dated/property grade: {rows:?}"
                    );
                    if !expected {
                        assert!(count > 0, "Expected a meaningful partial result");
                    }
                }
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    !practice && passed == Some(true)
                );
                if practice {
                    assert_eq!(
                        simulation.assignment,
                        work::Session::new(definition).unwrap().assignment
                    );
                }
                true
            }
            Op::Error => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::Aging { simulation } = &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                assert!(
                    simulation
                        .assignment
                        .transcript
                        .last()
                        .is_some_and(|r| r.error && r.text.contains("Unsupported chage option"))
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
                    let now = ctx.input(|i| i.time);
                    r.reveal_after = now + 0.15;
                    let target = self
                        .metrics
                        .aging
                        .result
                        .map(|r| r.2)
                        .or_else(|| self.metrics.aging.input.map(|r| r.0));
                    if target.is_none_or(|target| self.reveal_smoke_control(ctx, target)) {
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
            r.style_rect = None;
            r.stable_frames = 0;
            r.reveal_after = 0.;
            r.action_started = None;
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
    fn async_screenshot_and_resize_waits_do_not_spend_the_work_budget() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("aging-native-route");
        for (phase, op, capture) in [
            (2, Op::Shot("pending-screenshot".into()), true),
            (1, Op::Style(2., true, true), false),
        ] {
            app.capture_pending = capture;
            ctx.data_mut(|d| {
                d.insert_temp(
                    key,
                    Route {
                        ops: vec![op],
                        phase,
                        frames: 1199,
                        ..Default::default()
                    },
                )
            });
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280., 720.),
                    )),
                    time: Some(0.),
                    ..Default::default()
                },
                |ctx| app.aging_tick(ctx),
            );
            let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!(route.at, 0);
            assert_eq!(route.frames, 1199);
        }
    }
    #[test]
    #[should_panic(expected = "Aging native action exceeded15seconds")]
    fn a_stalled_clock_wait_still_has_a_real_fifteen_second_deadline() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("aging-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("aging-stalled-clock".into())],
                    reveal_after: 1000.,
                    action_started: Some(Instant::now() - std::time::Duration::from_secs(16)),
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                time: Some(0.),
                ..Default::default()
            },
            |ctx| app.aging_tick(ctx),
        );
    }
    #[test]
    fn a_timer_wait_does_not_consume_the_last_available_work_attempt() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("aging-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("aging-clock-regression".into())],
                    frames: 1199,
                    reveal_after: 1000.,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                time: Some(0.),
                ..Default::default()
            },
            |ctx| app.aging_tick(ctx),
        );
        let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
        assert_eq!(route.at, 0);
        assert_eq!(route.frames, 1199);
        assert!(!app.capture_pending);
    }
    #[test]
    fn style_waits_for_requested_window_geometry_before_advancing_to_capture() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("aging-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Style(2., true, true)],
                    ..Default::default()
                },
            )
        });
        let run = |size: egui::Vec2, app: &mut LabApp| {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ctx| app.aging_tick(ctx),
            );
        };
        run(egui::vec2(1280., 720.), &mut app);
        assert_eq!(
            ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at,
            0,
            "requesting a native resize is not geometry acceptance"
        );
        run(egui::vec2(1280., 720.), &mut app);
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at, 0);
        for _ in 0..3 {
            run(egui::vec2(634., 654.), &mut app);
        }
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at, 1);
    }
    #[test]
    fn fixed_route_exercises_all_three_tasks_and_their_real_failure_paths() {
        let ctx = egui::Context::default();
        let app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&app.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 3);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 3);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 3);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s.contains("--prefix=/")))
        );
        assert!(
            ops.iter()
                .filter(|o| matches!(
                    o,
                    Op::Grade {
                        practice: true,
                        passed: Some(true)
                    }
                ))
                .count()
                == 3
        );
        assert!(
            ops.iter()
                .filter(|o| matches!(
                    o,
                    Op::Grade {
                        practice: false,
                        passed: Some(false)
                    }
                ))
                .count()
                >= 6
        );
    }
    #[test]
    fn actual_capture_phase_requests_a_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-aging-rotation")
            .unwrap();
        app.launch(i);
        let engine::State::Aging { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-aging-rotation")
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
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.aging.input.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("aging-native-route"),
                Route {
                    ops: vec![Op::Shot("aging-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.aging_tick(ctx));
        assert!(app.capture_pending);
    }
}
