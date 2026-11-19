//! Fixed native command-location review. Commands enter real widgets on private smoke data.
use super::*;
#[derive(Clone, Debug)]
enum Op {
    Open,
    Click(&'static str),
    Command(String, bool),
    Practice(bool),
    Check(bool),
    Shot(String),
    Save,
    Fresh,
    Theme(bool),
    Challenge,
    Learn,
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
}
fn plan(s: &Scenario) -> Vec<Op> {
    let mut p = vec![
        Op::Open,
        Op::Shot("command-location-introduction".into()),
        Op::Click("example"),
    ];
    p.extend(
        s.terminal_teaching
            .as_ref()
            .unwrap()
            .example
            .iter()
            .cloned()
            .map(|c| Op::Command(c, true)),
    );
    p.extend([
        Op::Practice(true),
        Op::Shot("command-location-guided-results".into()),
        Op::Command("chmod u-x /gallery/preferred/demo-report".into(), true),
        Op::Command("which demo-report".into(), true),
        Op::Practice(false),
        Op::Shot("command-location-practice-skipped".into()),
        Op::Command("chmod u+x /gallery/preferred/demo-report".into(), true),
        Op::Practice(true),
        Op::Click("assignment"),
        Op::Shot("command-location-assignment".into()),
        Op::Command("which reportctl".into(), false),
        Op::Check(false),
        Op::Shot("command-location-missing-execute-feedback".into()),
        Op::Command("chmod 0755 /tools/preferred/reportctl".into(), false),
        Op::Check(false),
        Op::Shot("command-location-excess-permissions-feedback".into()),
        Op::Command("chmod 0740 /tools/preferred/reportctl".into(), false),
        Op::Check(true),
        Op::Save,
        Op::Command("chmod u-x /tools/preferred/reportctl".into(), false),
        Op::Check(false),
        Op::Command("chmod u+x /tools/preferred/reportctl".into(), false),
        Op::Check(true),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
        Op::Command("chmod u+x /tools/preferred/reportctl".into(), false),
        Op::Command("type -a -p reportctl".into(), false),
        Op::Command("which auditctl".into(), false),
        Op::Check(true),
        Op::Save,
    ]);
    for light in [true, false] {
        p.extend([
            Op::Theme(light),
            Op::Command("type -a -p reportctl".into(), false),
            Op::Check(true),
            Op::Shot(
                if light {
                    "command-location-light150-terminal"
                } else {
                    "command-location-dark200-terminal"
                }
                .into(),
            ),
        ]);
    }
    p.extend([
        Op::Click("challenge"),
        Op::Challenge,
        Op::Save,
        Op::Click("learn"),
        Op::Learn,
    ]);
    p
}

fn whereis_plan(s: &Scenario) -> Vec<Op> {
    let mut p = vec![
        Op::Open,
        Op::Shot("whereis-introduction".into()),
        Op::Click("example"),
    ];
    p.extend(
        s.terminal_teaching
            .as_ref()
            .unwrap()
            .example
            .iter()
            .cloned()
            .map(|c| Op::Command(c, true)),
    );
    p.extend([
        Op::Practice(true),
        Op::Shot("whereis-guided-results".into()),
        Op::Command(
            "cp /gallery/archive/democtl.c /gallery/source/democtl.c".into(),
            true,
        ),
        Op::Command("whereis -s -S /gallery/source -f democtl".into(), true),
        Op::Practice(false),
        Op::Shot("whereis-guided-old-release".into()),
        Op::Command(
            "cp /gallery/stage/democtl.c /gallery/source/democtl.c".into(),
            true,
        ),
        Op::Practice(true),
        Op::Click("assignment"),
        Op::Shot("whereis-assignment".into()),
        Op::Command("whereis -m -M /apps/man -f reportctl".into(), false),
        Op::Check(false),
        Op::Shot("whereis-missing-manual-feedback".into()),
        Op::Command(
            "cp /apps/archive/reportctl.1 /apps/man/reportctl.1".into(),
            false,
        ),
        Op::Check(false),
        Op::Shot("whereis-found-old-release-feedback".into()),
        Op::Command(s.solution[0].clone(), false),
        Op::Command(s.solution[1].clone(), false),
        Op::Check(false),
        Op::Shot("whereis-partial-retired-manual-feedback".into()),
        Op::Command(s.solution[2].clone(), false),
        Op::Check(true),
        Op::Save,
        Op::Command(
            "cp /apps/archive/reportctl.c /apps/source/reportctl.c".into(),
            false,
        ),
        Op::Command("whereis -s -S /apps/source -f reportctl".into(), false),
        Op::Check(false),
        Op::Shot("whereis-invalidated-release-feedback".into()),
        Op::Command(s.solution[1].clone(), false),
        Op::Check(true),
        Op::Save,
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
    ]);
    p.extend(s.solution.iter().cloned().map(|c| Op::Command(c, false)));
    p.extend([Op::Check(true), Op::Save]);
    for light in [true, false] {
        p.extend([
            Op::Theme(light),
            Op::Command("whereis -s -S /apps/source -f reportctl".into(), false),
            Op::Check(true),
            Op::Shot(
                if light {
                    "whereis-light150-terminal"
                } else {
                    "whereis-dark200-terminal"
                }
                .into(),
            ),
        ]);
    }
    p.extend([
        Op::Click("challenge"),
        Op::Challenge,
        Op::Save,
        Op::Click("learn"),
        Op::Learn,
    ]);
    p
}

impl LabApp {
    pub(in crate::ui) fn command_location_tick(&mut self, ctx: &egui::Context) {
        self.discovery_tick(ctx, false);
    }
    pub(in crate::ui) fn whereis_tick(&mut self, ctx: &egui::Context) {
        self.discovery_tick(ctx, true);
    }
    fn discovery_tick(&mut self, ctx: &egui::Context, whereis: bool) {
        let id = if whereis {
            "lpic1-whereis-support-handover"
        } else {
            "lpic1-command-permission-search"
        };
        let make_plan: fn(&Scenario) -> Vec<Op> = if whereis { whereis_plan } else { plan };
        let key = egui::Id::new(if whereis {
            "whereis-native-route"
        } else {
            "command-location-native-route"
        });
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new(if whereis {
            "whereis-native-root"
        } else {
            "command-location-native-root"
        });
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: make_plan(self.scenarios.iter().find(|s| s.id == id).unwrap()),
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            if whereis {
                println!(
                    "WHEREIS_NATIVE_COMPLETE: real terminal/Enter, separate guided practice, missing/archived/partial feedback, complete handover, invalidation, save/read/reset, Challenge and light150/dark200 typing"
                );
            } else {
                println!(
                    "COMMAND_LOCATION_NATIVE_COMPLETE: real terminal input/Enter, separate practice, permission skip/excess feedback, alternate repair, invalidation, save/read, reset, Challenge and light150/dark200 typing"
                );
            }
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = if whereis { 170000 } else { 160000 } + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("COMMAND_LOCATION_NATIVE_STEP {} {:?}", r.at, op)
        }
        r.frames += 1;
        assert!(
            r.frames < 10000,
            "Command-location native step timed out: {op:?}"
        );
        let done = match op {
            Op::Open => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "example" => self.metrics.example_action,
                    "assignment" => self
                        .metrics
                        .assignment_action
                        .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    "challenge" => self.metrics.mode_challenge,
                    "learn" => self.metrics.mode_learn,
                    _ => panic!("Unknown fixed command-location control"),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text, practice) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.terminal) {
                        r.phase = 1
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let i = self.selected.unwrap();
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let draft = if practice {
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .terminal_learning
                            .as_ref()
                            .unwrap()
                            .command
                    } else {
                        &self.input
                    };
                    assert_eq!(
                        draft, &text,
                        "Actual input widget must contain the fixed command before Enter"
                    );
                    self.pending_events = [true, false]
                        .into_iter()
                        .map(|pressed| egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        })
                        .collect();
                    r.phase = 3;
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    let a = &self.progress.attempts[&s.id];
                    let transcript = if practice {
                        assert_eq!(a.state, s.initial);
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .terminal_learning
                            .as_ref()
                            .unwrap()
                            .transcript
                    } else {
                        &a.transcript
                    };
                    assert!(
                        transcript.last().is_some_and(|line| line.contains(&text)),
                        "Real command must appear in terminal transcript: {text}: {transcript:?}"
                    );
                    true
                }
            }
            Op::Practice(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                let l = s.terminal_teaching.as_ref().unwrap();
                let learn = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .terminal_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.outcomes.iter().all(|c| c.check(&learn.tree).0), want);
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Check(want) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.check) {
                        r.phase = 1
                    }
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&s.id], s),
                        want
                    );
                    assert_eq!(self.metrics.completion, want);
                    true
                }
            }
            Op::Shot(name) => {
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
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert_eq!(a.state, s.initial);
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .terminal_learning
                        .as_ref()
                        .unwrap()
                        .tree,
                    s.terminal_teaching.as_ref().unwrap().fixture
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
            Op::Challenge => {
                let i = self.selected.unwrap();
                assert!(self.progress.attempts[&self.scenarios[i].id].challenge);
                assert_eq!(self.terminal_view(i), teaching::View::Assignment);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                true
            }
            Op::Learn => {
                let i = self.selected.unwrap();
                assert!(!self.progress.attempts[&self.scenarios[i].id].challenge);
                assert_eq!(self.metrics.lesson_tabs.len(), 3);
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            self.started = Instant::now()
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whereis_plan_preserves_missing_partial_wrong_release_and_native_retry() {
        let c = pack::ccna_pilot().unwrap();
        let s = c
            .iter()
            .find(|s| s.id == "lpic1-whereis-support-handover")
            .unwrap();
        let p = whereis_plan(s);
        assert!(p.iter().any(|o|matches!(o,Op::Command(c,false)if c=="cp /apps/archive/reportctl.c /apps/source/reportctl.c")));
        assert!(p.iter().any(|o| matches!(o, Op::Practice(false))));
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 1);
        assert!(p.iter().filter(|o| matches!(o, Op::Check(false))).count() >= 4);
        assert!(matches!(p.last(), Some(Op::Learn)));
    }
    #[test]
    fn command_location_plan_retains_actual_terminal_practice_wrong_repair_reset_and_challenge() {
        let c = pack::ccna_pilot().unwrap();
        let s = c
            .iter()
            .find(|s| s.id == "lpic1-command-permission-search")
            .unwrap();
        let p = plan(s);
        for cmd in [
            "chmod 0755 /tools/preferred/reportctl",
            "chmod 0740 /tools/preferred/reportctl",
            "chmod u-x /tools/preferred/reportctl",
            "which auditctl",
        ] {
            assert!(p.iter().any(|o| matches!(o,Op::Command(c,false)if c==cmd)));
        }
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 1);
        assert!(p.iter().any(|o| matches!(o, Op::Practice(false))));
        assert!(matches!(p.last(), Some(Op::Learn)));
    }
    #[test]
    fn command_location_capture_phase_uses_native_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-command-permission-search")
            .unwrap();
        a.launch(i);
        a.set_terminal_view(i, teaching::View::Example);
        let raw = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw(), |ctx| a.render(ctx));
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("command-location-native-route"),
                Route {
                    ops: vec![Op::Shot("command-location-capture-regression".into())],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(raw(), |ctx| {
            a.render(ctx);
            a.command_location_tick(ctx);
        });
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn whereis_capture_phase_uses_native_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-whereis-support-handover")
            .unwrap();
        a.launch(i);
        a.set_terminal_view(i, teaching::View::Example);
        let raw = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw(), |ctx| a.render(ctx));
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("whereis-native-route"),
                Route {
                    ops: vec![Op::Shot("whereis-capture-regression".into())],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(raw(), |ctx| {
            a.render(ctx);
            a.whereis_tick(ctx);
        });
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
}
