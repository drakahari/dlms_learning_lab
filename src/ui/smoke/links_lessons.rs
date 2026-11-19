//! Fixed authored gateway flow: real widget events, private smoke progress only.
use super::*;
use dlms_learning_lab::links_workspace as work;
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
    TerminalShot(String),
    TopShot(String),
    Inodes(String, bool),
    Error(String, bool),
    Theme(bool),
    ThemeDefault,
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
    until: f64,
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for s in labs.iter().filter(|s| s.id.starts_with("lpic1-links-")) {
        ops.push(Op::Open(s.id.clone()));
        let engine::Goal::Links { definition } = &s.goal else {
            unreachable!()
        };
        if s.id == "lpic1-links-identity" {
            ops.push(Op::Click("example".into()));
            for n in 0..definition.example_commands.len() {
                ops.extend([
                    Op::Click("picker".into()),
                    Op::Click(format!("choice-{n}")),
                    Op::Click("insert".into()),
                    Op::Click("run".into()),
                ]);
            }
            ops.extend([
                Op::Click("check".into()),
                Op::Example(true),
                Op::Shot("links-practice-passed".into()),
            ]);
        }
        ops.extend([
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("links-{}-initial-feedback", s.id)),
        ]);
        for command in &s.solution {
            ops.extend([
                Op::Field("command".into(), command.clone()),
                Op::Click("run".into()),
            ]);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("links-{}-passed", s.id)),
        ]);
        let victim = if s.id.ends_with("identity") {
            "/delivery/live"
        } else if s.id.ends_with("relative") {
            "/delivery/latest"
        } else if s.id.ends_with("move") {
            "/delivery/receipt"
        } else {
            "/delivery/current"
        };
        ops.extend([
            Op::Field("command".into(), format!("rm {victim}")),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Click("check".into()),
            Op::Shot(format!("links-{}-damage-feedback", s.id)),
        ]);
        let repair = s
            .solution
            .iter()
            .rev()
            .find(|c| c.starts_with("ln ") && c.ends_with(victim))
            .unwrap();
        ops.extend([
            Op::Field("command".into(), repair.clone()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    assert_eq!(
        labs.iter()
            .filter(|s| s.id.starts_with("lpic1-links-"))
            .count(),
        4
    );
    ops
}
fn inode_query(ops: &mut Vec<Op>, directory: &str, practice: bool) {
    ops.extend([
        Op::Field("command".into(), format!("ls -i {directory}")),
        Op::Click("run".into()),
        Op::Inodes(directory.into(), practice),
    ]);
}
fn inode_route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for source in labs.iter().filter(|s| s.id.starts_with("lpic1-links-")) {
        ops.push(Op::Open(source.id.clone()));
        if source.id.ends_with("identity") {
            ops.push(Op::TopShot("links-inode-introduction".into()));
        }
        ops.push(Op::Click("example".into()));
        inode_query(&mut ops, "/practice/in", true);
        inode_query(&mut ops, "/practice/out", true);
        let engine::Goal::Links { definition } = &source.goal else {
            unreachable!()
        };
        for command in &definition.example_commands {
            ops.extend([
                Op::Field("command".into(), command.clone()),
                Op::Click("run".into()),
            ]);
        }
        inode_query(&mut ops, "/practice/archive", true);
        inode_query(&mut ops, "/practice/out", true);
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("{}-inode-practice", source.id)),
            Op::Save,
        ]);
        if source.id.ends_with("move") {
            ops.extend([
                Op::Field(
                    "command".into(),
                    "mv /practice/archive/receipt /practice/in/receipt".into(),
                ),
                Op::Click("run".into()),
                Op::Field("command".into(), "cat /practice/out/receipt".into()),
                Op::Click("run".into()),
                Op::Error("no virtual entry".into(), true),
                Op::Example(false),
                Op::Field(
                    "command".into(),
                    "mv /practice/in/receipt /practice/archive/receipt".into(),
                ),
                Op::Click("run".into()),
                Op::Click("check".into()),
                Op::Example(true),
            ]);
        }
        if source.id.ends_with("identity") {
            ops.extend([
                Op::Click("assignment".into()),
                Op::TopShot("links-inode-assignment".into()),
                Op::Click("check".into()),
                Op::Solved(false),
            ]);
            for command in &source.solution {
                ops.extend([
                    Op::Field("command".into(), command.clone()),
                    Op::Click("run".into()),
                ]);
            }
            inode_query(&mut ops, "/delivery", false);
            ops.extend([Op::Click("check".into()), Op::Solved(true), Op::Save]);
            for light in [true, false] {
                ops.push(Op::Theme(light));
                ops.extend([
                    Op::Click("introduction".into()),
                    Op::TopShot(
                        if light {
                            "links-inode-light150-introduction"
                        } else {
                            "links-inode-dark200-introduction"
                        }
                        .into(),
                    ),
                    Op::Click("assignment".into()),
                ]);
                inode_query(&mut ops, "/work", false);
                ops.extend([
                    Op::Click("check".into()),
                    Op::Solved(true),
                    Op::TerminalShot(
                        if light {
                            "links-inode-light150"
                        } else {
                            "links-inode-dark200"
                        }
                        .into(),
                    ),
                ]);
            }
            ops.push(Op::ThemeDefault);
            ops.extend([
                Op::Field("command".into(), "rm /delivery/live".into()),
                Op::Click("run".into()),
                Op::Solved(false),
                Op::Click("check".into()),
                Op::Shot("links-inode-current-invalidated".into()),
            ]);
        }
    }
    assert_eq!(
        labs.iter()
            .filter(|s| s.id.starts_with("lpic1-links-"))
            .count(),
        4
    );
    ops
}

impl LabApp {
    pub(in crate::ui) fn links_lessons_tick(&mut self, ctx: &egui::Context) {
        self.links_native_tick(ctx, false);
    }
    pub(in crate::ui) fn links_inode_tick(&mut self, ctx: &egui::Context) {
        self.links_native_tick(ctx, true);
    }
    fn links_native_tick(&mut self, ctx: &egui::Context, inode: bool) {
        if inode {
            let frame = ctx.cumulative_frame_nr();
            let gate = egui::Id::new("links-inode-native-root-frame");
            if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(frame) {
                return;
            }
            ctx.data_mut(|d| d.insert_temp(gate, frame));
        }
        let key = egui::Id::new(if inode {
            "links-inode-native-route"
        } else {
            "links-native-route"
        });
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if inode {
                    inode_route(&self.scenarios)
                } else {
                    route(&self.scenarios)
                },
                ..Default::default()
            });
        if r.done {
            if inode {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                self.teaching_preview_tick(ctx);
            }
            return;
        }
        if r.at == r.ops.len() {
            if inode {
                println!(
                    "LINKS_INODE_NATIVE_COMPLETE: four real guided practices, own-inode listings, isolated assignment, move/dangling/retry, actual save/read, invalidation, focused editing, light150/dark200"
                );
            } else {
                println!(
                    "LINKS_UNIT_NATIVE_FLOW_COMPLETE: all4 partial/solution/damage/retry/save-reload/reset flows; separate guided example passes; experiments covered by backend tests"
                );
            }
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if inode && now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        let capture_terminal = matches!(&op, Op::TerminalShot(_));
        if inode {
            if r.frames == 0 {
                println!("LINKS_INODE_NATIVE_STEP {} {:?}", r.at, op);
            }
            r.frames += 1;
            assert!(r.frames < 2000, "Links inode native phase timeout: {op:?}");
        }
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
                        .links
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
                    .links
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
                let (engine::Goal::Links { definition }, engine::State::Links { simulation }) =
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
            Op::Inodes(directory, practice) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&source.id];
                work::context(a, source).unwrap();
                let engine::State::Links { simulation } = &a.state else {
                    unreachable!()
                };
                let scope = if practice {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                let actual = scope.transcript.last().unwrap().split_once('\n').unwrap().1;
                let rows: Vec<(String, u32)> = actual
                    .lines()
                    .map(|line| {
                        let (id, name) = line.trim_start().split_once(' ').unwrap();
                        (
                            name.into(),
                            id.parse().expect("Native ls -i must print real model IDs"),
                        )
                    })
                    .collect();
                let expected: Vec<_> = scope
                    .world
                    .directory_inode_entries(&directory)
                    .unwrap()
                    .into_iter()
                    .filter(|(n, _)| !n.starts_with('.'))
                    .collect();
                assert_eq!(rows, expected, "Actual native terminal listing");
                true
            }
            Op::Error(text, practice) => {
                let engine::State::Links { simulation } =
                    &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id].state
                else {
                    unreachable!()
                };
                let scope = if practice {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                assert!(
                    scope
                        .transcript
                        .last()
                        .unwrap()
                        .to_lowercase()
                        .contains(&text)
                );
                true
            }
            Op::Theme(light) => {
                self.scale = if light { 1.5 } else { 2. };
                ctx.set_zoom_factor(self.scale);
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
                self.lesson_scroll_reset = true;
                true
            }
            Op::ThemeDefault => {
                self.scale = 1.;
                ctx.set_zoom_factor(self.scale);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
                true
            }
            Op::TopShot(name) => {
                if r.phase == 0 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 1;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Shot(name) | Op::TerminalShot(name) => {
                if r.phase == 0 {
                    let target = if capture_terminal {
                        self.metrics
                            .links
                            .fields
                            .iter()
                            .find(|f| f.0 == "command")
                            .unwrap()
                            .1
                    } else {
                        self.metrics.links.result.unwrap().2
                    };
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
        // Let native smooth scrolling consume the wheel event before another
        // bounded reveal attempt. Release frames otherwise exhaust the same
        // 20-try guard before the viewport can move. Keep all visibility checks.
        if inode
            && self
                .pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15;
        }
        if complete {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
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
    fn inode_native_reveal_waits_for_scroll_without_bypassing_visibility_or_retry_limit() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 500.));
        app.metrics.main_viewport = Some(viewport);
        app.metrics.links.controls.push((
            "run".into(),
            egui::Rect::from_min_size(egui::pos2(10., 800.), egui::vec2(60., 30.)),
            viewport,
        ));
        let key = egui::Id::new("links-inode-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("run".into())],
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                time: Some(1.),
                ..Default::default()
            },
            |ctx| app.links_inode_tick(ctx),
        );
        assert!(
            app.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        );
        let first = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
        assert_eq!(first.at, 0);
        assert_eq!(first.frames, 1);
        assert!(first.until > 1.);
        app.pending_events.clear();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                time: Some(1.05),
                ..Default::default()
            },
            |ctx| app.links_inode_tick(ctx),
        );
        assert!(app.pending_events.is_empty());
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().frames, 1);
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                time: Some(1.2),
                ..Default::default()
            },
            |ctx| app.links_inode_tick(ctx),
        );
        assert!(
            app.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        );
        assert!(
            !app.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerButton { .. }))
        );
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at, 0);
    }
    #[test]
    fn inode_introduction_capture_requests_real_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-links-identity")
            .unwrap();
        app.launch(i);
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("links-inode-native-route"),
                Route {
                    ops: vec![Op::TopShot("links-inode-introduction-regression".into())],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(Default::default(), |ctx| app.links_inode_tick(ctx));
        assert!(app.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn links_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-links-identity")
            .unwrap();
        app.launch(i);
        if let engine::State::Links { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-links-identity")
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
        assert!(app.metrics.links.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("links-native-route"),
                Route {
                    ops: vec![Op::Shot("links-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.links_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
