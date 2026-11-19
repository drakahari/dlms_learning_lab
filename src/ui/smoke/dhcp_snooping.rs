//! Fixed native DHCP observer actions with private smoke progress; no real network.
use super::*;
use dlms_learning_lab::dhcp_snooping_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String, String),
    Grade {
        practice: bool,
        passed: Option<bool>,
    },
    Notice(String),
    Fresh,
    Save,
    Shot(String),
    ShotAt(String, String),
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
pub(in crate::ui) const IDS: [&str; 2] =
    ["ccna-dhcp-server-boundary", "ccna-dhcp-client-mac-check"];
fn value<'a>(scope: &'a work::Scope, key: &str) -> Option<&'a str> {
    (key == "vlans").then_some(scope.draft.vlans.as_str())
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for id in IDS {
        assert!(labs.iter().any(|s| s.id == id));
        ops.extend([
            Op::Style(1., false, false),
            Op::Open(id.into()),
            Op::Shot(format!("{id}-introduction")),
            Op::Click("example".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-example-wrong")),
            Op::Click("worked-example".into()),
            Op::Click("fill-example".into()),
            Op::Click("replay".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(true),
            },
            Op::Shot(format!("{id}-example-worked")),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-assignment-initial")),
        ]);
        if id == IDS[0] {
            ops.extend([
                Op::Field("vlans".into(), "ten".into()),
                Op::Click("apply".into()),
                Op::Notice("VLANs".into()),
                Op::Field("vlans".into(), "10".into()),
                Op::Click("enabled".into()),
                Op::Click("trust-server".into()),
                Op::Click("apply".into()),
            ]);
        } else {
            ops.extend([Op::Click("verify-source".into()), Op::Click("apply".into())]);
        }
        ops.extend([
            Op::Click("replay".into()),
            Op::Shot(format!("{id}-actual-replay")),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Shot(format!("{id}-assignment-passed")),
            Op::Save,
            Op::Field("vlans".into(), "".into()),
            Op::Grade {
                practice: false,
                passed: None,
            },
            Op::Click("apply".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Shot(format!("{id}-assignment-broken")),
            Op::Field("vlans".into(), "10".into()),
            Op::Click("apply".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Save,
        ]);
        if id == IDS[1] {
            ops.extend([
                Op::Style(2., true, true),
                Op::ShotAt("dhcp-snooping-light200-narrow".into(), "vlans".into()),
                Op::Style(1.5, false, false),
                Op::ShotAt("dhcp-snooping-dark150-wide".into(), "vlans".into()),
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
    /// A collapsing body's widget can exist before its hit area is revealed.
    /// Wait for the actual child clip, rather than clicking through its animation.
    fn dhcp_snooping_click(
        &mut self,
        ctx: &egui::Context,
        target: Option<(egui::Rect, egui::Rect)>,
    ) -> Option<bool> {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            // A popup or disclosure target can disappear after a delivered click.
            return Some(self.click_smoke_control(ctx, None));
        }
        let (rect, clip) = target?;
        if !self.reveal_smoke_control(ctx, rect) {
            return Some(false);
        }
        if !clip.expand(1. / ctx.pixels_per_point()).contains_rect(rect) {
            return None;
        }
        Some(self.click_smoke_control(ctx, Some(rect)))
    }

    pub(in crate::ui) fn dhcp_snooping_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("dhcp_snooping-native-route");
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
                "DHCP_SNOOPING_NATIVE_FLOW_COMPLETE: two DHCP inspection assignments; separate practice, actual replay, wrong/partial/broken/retry/save-read/reset; no real packets or IOS"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 97000 + r.at;
        let op = r.ops[r.at].clone();
        let began = *r.action_started.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed().as_secs() < 15,
            "DhcpSnooping native action exceeded15seconds: {} {:?}",
            r.at,
            op
        );
        // Optimized rendering can yield thousands of callbacks during a short
        // asynchronous settle delay. Waiting consumes time, not a work attempt.
        // The independent wall deadline still bounds even a stalled egui clock.
        if matches!(&op, Op::Shot(_) | Op::ShotAt(_, _) if (r.phase==0 && ctx.input(|i|i.time)<r.reveal_after)
            || (r.phase==2 && self.capture_pending))
            || matches!(&op, Op::Click(_) if ctx.input(|i| i.time) < r.reveal_after)
        {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        if r.frames == 0 {
            println!("DHCP_SNOOPING_NATIVE_STEP {} {:?}", r.at, op);
        }
        // A resize acknowledgement still needs geometry observations, but
        // those observations are waiting rather than new work attempts.
        if !matches!(&op, Op::Style(..) if r.phase==1) {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "DhcpSnooping native action did not complete: {} {:?}",
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
                        "DhcpSnooping native window geometry did not settle"
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
                let viewport = self.metrics.main_viewport.unwrap_or(ctx.content_rect());
                let target = if name == "reset-header" {
                    self.metrics.reset.map(|r| (r, ctx.content_rect()))
                } else if name == "confirm-reset" {
                    self.metrics.confirm_reset.map(|r| (r, viewport))
                } else {
                    self.metrics
                        .dhcp_snooping
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| (c.1, c.2))
                };
                match self.dhcp_snooping_click(ctx, target) {
                    Some(done) => done,
                    None => {
                        r.reveal_after = ctx.input(|i| i.time) + 0.025;
                        false
                    }
                }
            }
            Op::Field(name, value) => {
                let field = self
                    .metrics
                    .dhcp_snooping
                    .fields
                    .iter()
                    .find(|f| f.0 == name)
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
                        "native dhcp_snooping settings field focus"
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
                    let engine::State::DhcpSnooping { simulation } =
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
                        self::value(scope, &name),
                        Some(value.as_str()),
                        "Native paste must reach the actual dhcp_snooping settings field"
                    );
                    true
                }
            }
            Op::Grade { practice, passed } => {
                let source = &self.scenarios[self.selected.unwrap()];
                let definition = work::source(source).unwrap();
                let engine::State::DhcpSnooping { simulation } =
                    &self.progress.attempts[&source.id].state
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
                        self.metrics.dhcp_snooping.result.map(|r| (r.0, r.1)),
                        Some((count, rows.len()))
                    );
                    assert_eq!(
                        count == rows.len(),
                        expected,
                        "Actual configuration/ordered-frame grade: {rows:?}"
                    );
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
            Op::Notice(text) => {
                assert!(
                    self.notice.as_deref().is_some_and(|s| s.contains(&text)),
                    "Specific native retry guidance must be visible: {:?}",
                    self.notice
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
            op @ (Op::Shot(_) | Op::ShotAt(_, _)) => {
                let (name, target_key) = match op {
                    Op::Shot(n) => (n, None),
                    Op::ShotAt(n, k) => (n, Some(k)),
                    _ => unreachable!(),
                };
                if r.phase == 0 {
                    let now = ctx.input(|i| i.time);
                    r.reveal_after = now + 0.15;
                    let target = if let Some(key) = target_key {
                        self.metrics
                            .dhcp_snooping
                            .fields
                            .iter()
                            .find(|f| f.0 == key)
                            .map(|f| f.1)
                    } else {
                        self.metrics.dhcp_snooping.result.map(|r| r.2)
                    };
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
    fn animated_child_clip_must_reveal_the_hit_area_before_clicking() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let rect = egui::Rect::from_min_size(egui::pos2(40., 100.), egui::vec2(160., 40.));
        let partial = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(800., 105.));
        let full = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.));
        app.metrics.main_viewport = Some(full);
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(full),
                    ..Default::default()
                },
                |ctx| assert_eq!(app.dhcp_snooping_click(ctx, Some((rect, partial))), None),
            );
            assert!(
                app.pending_events.is_empty(),
                "No click through an animated child clip"
            );
        }
        for _ in 0..2 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(full),
                    ..Default::default()
                },
                |ctx| {
                    assert_eq!(
                        app.dhcp_snooping_click(ctx, Some((rect, full))),
                        Some(false)
                    )
                },
            );
        }
        assert_eq!(
            app.pending_events
                .iter()
                .filter(|e| matches!(e, egui::Event::PointerButton { .. }))
                .count(),
            2
        );
        let _ = ctx.run(Default::default(), |ctx| {
            assert_eq!(app.dhcp_snooping_click(ctx, None), Some(true));
        });
    }

    #[test]
    fn child_animation_waits_do_not_spend_the_work_budget() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("dhcp_snooping-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("fill-example".into())],
                    frames: 1199,
                    reveal_after: 1000.,
                    ..Default::default()
                },
            )
        });
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(0.),
                    ..Default::default()
                },
                |ctx| app.dhcp_snooping_tick(ctx),
            );
            let r = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!((r.at, r.frames), (0, 1199));
            assert!(app.pending_events.is_empty());
        }
    }

    #[test]
    fn async_screenshot_and_resize_waits_do_not_spend_the_work_budget() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("dhcp_snooping-native-route");
        for (phase, op, capture) in [
            (2, Op::Shot("pending-screenshot".into()), true),
            (2, Op::ShotAt("pending-field".into(), "vlans".into()), true),
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
                |ctx| app.dhcp_snooping_tick(ctx),
            );
            let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!(route.at, 0);
            assert_eq!(route.frames, 1199);
        }
    }
    #[test]
    #[should_panic(expected = "DhcpSnooping native action exceeded15seconds")]
    fn a_stalled_clock_wait_still_has_a_real_fifteen_second_deadline() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("dhcp_snooping-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("dhcp_snooping-stalled-clock".into())],
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
            |ctx| app.dhcp_snooping_tick(ctx),
        );
    }
    #[test]
    fn a_timer_wait_does_not_consume_the_last_available_work_attempt() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("dhcp_snooping-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("dhcp_snooping-clock-regression".into())],
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
            |ctx| app.dhcp_snooping_tick(ctx),
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
        let key = egui::Id::new("dhcp_snooping-native-route");
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
                |ctx| app.dhcp_snooping_tick(ctx),
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
    fn fixed_route_exercises_both_real_tasks_and_checked_failure_paths() {
        let ctx = egui::Context::default();
        let app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&app.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(k,v)if k=="vlans"&&v=="ten"))
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(
                    o,
                    Op::Grade {
                        practice: true,
                        passed: Some(true)
                    }
                ))
                .count(),
            2
        );
    }
    #[test]
    fn actual_capture_phase_requests_a_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-dhcp-server-boundary")
            .unwrap();
        app.launch(i);
        let engine::State::DhcpSnooping { simulation } = &mut app
            .progress
            .attempts
            .get_mut("ccna-dhcp-server-boundary")
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
        assert!(!app.metrics.dhcp_snooping.fields.is_empty());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("dhcp_snooping-native-route"),
                Route {
                    ops: vec![Op::ShotAt(
                        "dhcp-snooping-regression-capture".into(),
                        "vlans".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.dhcp_snooping_tick(ctx));
        assert!(app.capture_pending);
    }
}
