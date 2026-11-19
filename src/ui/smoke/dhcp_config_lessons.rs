//! Fixed DHCP editor flows: real widget events and private smoke progress only.
use super::*;
use dlms_learning_lab::dhcp_server_workspace as work;
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
    let mut n = 0;
    for s in labs.iter().filter(|s| s.id.starts_with("lpic2-dhcp-")) {
        n += 1;
        let engine::Goal::DhcpConfig { definition } = &s.goal else {
            unreachable!()
        };
        let raw = s.solution[0].strip_prefix(work::PREFIX).unwrap();
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
            Op::Shot(format!("dhcp-config-{}-practice-passed", s.id)),
            Op::Field(
                "editor".into(),
                definition
                    .example_worked
                    .replace("203.0.113.53", "203.0.113.99"),
            ),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Field("editor".into(), definition.example_worked.clone()),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Click("assignment".into()),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("dhcp-config-{}-starter-feedback", s.id)),
            Op::Field("editor".into(), raw.into()),
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("dhcp-config-{}-passed", s.id)),
            Op::Field("editor".into(), raw.replace("203.0.113.53", "203.0.113.99")),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("dhcp-config-{}-damage-feedback", s.id)),
            Op::Field("editor".into(), raw.into()),
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    assert_eq!(n, 2);
    ops
}
impl LabApp {
    pub(in crate::ui) fn dhcp_config_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("dhcp-config-native-route");
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
                "DHCP_CONFIG_UNIT_NATIVE_FLOW_COMPLETE: both partial/solution/damage/retry/save-read/reset flows; both separate guided examples pass; no daemon, host commands or network operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("DHCP_CONFIG_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "DhcpConfig native action did not complete: {} {:?}",
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
                        .dhcp_config
                        .controls
                        .iter()
                        .find(|c| c.0 == key)
                        .map(|c| c.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Field(key, value) => {
                let field = self.metrics.dhcp_config.editor;
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
                if self.dhcp_config_run.is_some() {
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
                if self.dhcp_config_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::DhcpConfig { definition },
                    engine::State::DhcpConfig { simulation },
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
                    let target = self.metrics.dhcp_config.result.unwrap().2;
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
    fn dhcp_config_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-dhcp-dynamic-ranges")
            .unwrap();
        app.launch(i);
        if let engine::State::DhcpConfig { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-dhcp-dynamic-ranges")
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
        assert!(app.metrics.dhcp_config.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("dhcp-config-native-route"),
                Route {
                    ops: vec![Op::Shot("dhcp-config-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.dhcp_config_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
