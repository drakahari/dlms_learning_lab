//! Fixed native editor actions with private smoke progress; no host authentication.
use super::*;
use dlms_learning_lab::pam_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String),
    Cases(usize, usize),
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
    style_rect: Option<egui::Rect>,
    stable_frames: usize,
    reveal_after: f64,
    action_started: Option<Instant>,
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let s = labs
        .iter()
        .find(|s| s.id == "lpic2-pam-auth-bypass")
        .unwrap();
    let good: String =
        serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    vec![
        Op::Style(1., false, false),
        Op::Open(s.id.clone()),
        Op::Shot("pam-auth-introduction".into()),
        Op::Click("example".into()),
        Op::Shot("pam-auth-example-unchecked".into()),
        Op::Click("run".into()),
        Op::Cases(1, 5),
        Op::Example(false),
        Op::Shot("pam-auth-example-bypass".into()),
        Op::Click("trace:1".into()),
        Op::Shot("pam-auth-visited-failure".into()),
        Op::Click("insert".into()),
        Op::Example(false),
        Op::Click("run".into()),
        Op::Cases(5, 5),
        Op::Example(true),
        Op::Shot("pam-auth-example-passed".into()),
        Op::Field("auth required pam_unix.so\nauth required pam_deny.so".into()),
        Op::Click("run".into()),
        Op::Cases(4, 5),
        Op::Example(false),
        Op::Shot("pam-auth-good-user-locked-out".into()),
        Op::Click("insert".into()),
        Op::Click("run".into()),
        Op::Cases(5, 5),
        Op::Example(true),
        Op::Click("assignment".into()),
        Op::Shot("pam-auth-assignment-unchecked".into()),
        Op::Click("run".into()),
        Op::Cases(1, 5),
        Op::Solved(false),
        Op::Shot("pam-auth-assignment-bypass".into()),
        Op::Field(good),
        Op::Click("run".into()),
        Op::Cases(5, 5),
        Op::Solved(true),
        Op::Save,
        Op::Shot("pam-auth-assignment-passed".into()),
        Op::Field("auth required pam_unix.so nullok".into()),
        Op::Solved(false),
        Op::Click("run".into()),
        Op::Cases(0, 5),
        Op::Shot("pam-auth-unsupported".into()),
        Op::Field("auth".into()),
        Op::Click("run".into()),
        Op::Cases(0, 5),
        Op::Shot("pam-auth-syntax".into()),
        Op::Field("auth sufficient pam_unix.so\nauth required pam_deny.so".into()),
        Op::Click("run".into()),
        Op::Cases(5, 5),
        Op::Solved(true),
        Op::Style(2., true, true),
        Op::Shot("pam-auth-light200-narrow".into()),
        Op::Style(1.5, false, false),
        Op::Shot("pam-auth-dark150-wide".into()),
        Op::Style(1., false, false),
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]
}
impl LabApp {
    pub(in crate::ui) fn pam_auth_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("pam-auth-native-route");
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
                "PAM_AUTH_NATIVE_FLOW_COMPLETE: one virtual auth-stack diagnosis; actual1/5bypass,5/5repair,4/5lockout,0/5unsupported/syntax, equivalent repair/retry/save-read/reset; no host authentication"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 92000 + r.at;
        let op = r.ops[r.at].clone();
        let began = *r.action_started.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed().as_secs() < 15,
            "PAM native action exceeded15seconds: {} {:?}",
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
            println!("PAM_AUTH_NATIVE_STEP {} {:?}", r.at, op);
        }
        // A resize acknowledgement still needs geometry observations, but
        // those observations are waiting rather than new work attempts.
        if !matches!(&op, Op::Style(..) if r.phase==1) {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "PAM native action did not complete: {} {:?}",
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
                        "PAM native window geometry did not settle"
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
                        .pam
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Field(value) => {
                let field = self.metrics.pam.editor;
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native PAM editor focus"
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
                    let engine::State::PamStack { simulation } =
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
                        "Native paste must reach the actual PAM editor"
                    );
                    true
                }
            }
            Op::Cases(passed, total) => {
                if self.pam_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                assert_eq!(
                    self.metrics.pam.result.map(|x| (x.0, x.1)),
                    Some((passed, total)),
                    "Actual native case count"
                );
                true
            }
            Op::Solved(expected) => {
                if self.pam_run.is_some() {
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
                if self.pam_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (engine::Goal::PamStack { definition }, engine::State::PamStack { simulation }) =
                    (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(
                    simulation.practice.tested.as_ref() == Some(&simulation.practice.draft)
                        && definition
                            .example
                            .run(&simulation.practice.draft)
                            .unwrap()
                            .iter()
                            .all(work::CaseResult::passed),
                    expected
                );
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
                    let now = ctx.input(|i| i.time);
                    r.reveal_after = now + 0.15;
                    let target = self
                        .metrics
                        .pam
                        .result
                        .map(|r| r.2)
                        .or_else(|| self.metrics.pam.editor.map(|r| r.0));
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
        let key = egui::Id::new("pam-auth-native-route");
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
                |ctx| app.pam_auth_tick(ctx),
            );
            let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!(route.at, 0);
            assert_eq!(route.frames, 1199);
        }
    }
    #[test]
    #[should_panic(expected = "PAM native action exceeded15seconds")]
    fn a_stalled_clock_wait_still_has_a_real_fifteen_second_deadline() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("pam-auth-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("pam-stalled-clock".into())],
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
            |ctx| app.pam_auth_tick(ctx),
        );
    }
    #[test]
    fn a_timer_wait_does_not_consume_the_last_available_work_attempt() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("pam-auth-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("pam-clock-regression".into())],
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
            |ctx| app.pam_auth_tick(ctx),
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
        let key = egui::Id::new("pam-auth-native-route");
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
                |ctx| app.pam_auth_tick(ctx),
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
    fn fixed_route_checks_actual_decisions_and_independent_failure_paths() {
        let ctx = egui::Context::default();
        let app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&app.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Shot(_))).count(), 13);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Style(..))).count(), 4);
        assert!(ops.iter().any(|o| matches!(o, Op::Cases(1, 5))));
        assert!(ops.iter().any(|o| matches!(o, Op::Cases(4, 5))));
        assert!(ops.iter().any(|o| matches!(o, Op::Cases(0, 5))));
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s.ends_with("nullok")))
        );
        assert!(ops.iter().any(|o| matches!(o,Op::Field(s)if s=="auth")));
        assert!(ops.iter().any(|o| matches!(o, Op::Save)));
        assert!(matches!(ops.last(), Some(Op::Fresh)));
    }
    #[test]
    fn actual_capture_phase_requests_a_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-pam-auth-bypass")
            .unwrap();
        app.launch(i);
        let engine::State::PamStack { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-pam-auth-bypass")
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
        assert!(app.metrics.pam.editor.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("pam-auth-native-route"),
                Route {
                    ops: vec![Op::Shot("pam-auth-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.pam_auth_tick(ctx));
        assert!(app.capture_pending);
    }
}
