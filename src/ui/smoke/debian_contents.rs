//! Fixed Contents lesson flow through native input; private smoke profile only.
use super::*;
use dlms_learning_lab::debian_packages_terminal::Session;
const ID: &str = "lpic1-package-file-provider";
#[derive(Clone, Debug)]
enum Op {
    Open,
    View(usize),
    Command(String, u8),
    Check(bool),
    Save,
    Reset,
    Confirm,
    Fresh,
    Shot(String, bool),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    done: bool,
    settle_until: f64,
    action_started: Option<f64>,
}
fn command(s: &str) -> Op {
    Op::Command(s.into(), 0)
}
fn route() -> Vec<Op> {
    vec![
        Op::Open,
        Op::View(1),
        Op::Command("dpkg-query -S /usr/bin/exhibit-audit".into(), 1),
        command("apt-get --error-on=any update"),
        command("apt-file search exhibit-audit"),
        command("apt-file -F search usr/bin/exhibit-audit"),
        command("apt-file list gallery-tools"),
        command("apt-get -y install gallery-tools"),
        Op::Check(true),
        Op::Shot("contents-practice-passed".into(), false),
        command("apt-get -y install gallery-private"),
        Op::Check(false),
        Op::Shot("contents-practice-lookalike-mistake".into(), false),
        command("dpkg -r gallery-private"),
        Op::Check(true),
        Op::View(2),
        Op::Command("dpkg-query -S /usr/bin/ledger-check".into(), 1),
        command("apt-file search ledger-check"),
        Op::Check(false),
        command("apt-get --error-on=any update"),
        command("apt-file -F search usr/bin/ledger-check"),
        Op::Shot("contents-actual-provider-query".into(), true),
        command("apt-file list records-tools"),
        command("apt-get -y install records-docs"),
        Op::Check(false),
        Op::Shot("contents-wrong-provider-feedback".into(), false),
        command("dpkg -r records-docs"),
        command("apt-get -y install records-tools"),
        Op::Check(true),
        Op::Save,
        Op::Shot("contents-assignment-passed".into(), false),
        command("dpkg -r records-tools"),
        Op::Check(false),
        command("apt-get -y install records-tools=1.0-1"),
        Op::Check(true),
        Op::Reset,
        Op::Confirm,
        Op::Fresh,
    ]
}
fn terminal(a: &LabApp) -> &Session {
    let w = a.progress.attempts[ID].workspace.as_ref().unwrap();
    if a.debian_packages_view(a.selected.unwrap()) == teaching::View::Example {
        w.debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
    } else {
        w.debian_packages_terminal.as_ref().unwrap()
    }
}
impl LabApp {
    pub(in crate::ui) fn debian_contents_tick(&mut self, ctx: &egui::Context) {
        // A layout retry is the same input frame. Never advance a pending-event
        // phase before raw_input_hook has delivered those events next frame.
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("debian-contents-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        let key = egui::Id::new("debian-contents-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "DEBIAN_CONTENTS_NATIVE_FLOW_COMPLETE: installed/repository evidence, lookalikes, independent assignment, retry, save-read and selected reset verified; no host commands"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 91000 + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let began = *r.action_started.get_or_insert(now);
        assert!(
            now - began < 15.,
            "Contents native action exceeded15seconds: {op:?}; phase={}",
            r.phase
        );
        // Repaint frequency is not elapsed time. The optimized app can render
        // thousands of root frames while one real wheel animation settles.
        // Waiting frames do not consume the bounded1200 interaction attempts;
        // the separate15second deadline still bounds a stalled action.
        if now < r.settle_until {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(10));
            return;
        }
        if r.frames == 0 {
            println!("DEBIAN_CONTENTS_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Contents native action timed out: {op:?}; root={root} time={now} settle={} phase={} offset={:?} content={:?} check={:?} pending={:?}",
            r.settle_until,
            r.phase,
            self.metrics.main_scroll_offset,
            self.metrics.main_content_size,
            self.metrics.check,
            self.pending_events
        );
        // Also pace ordinary phases, rather than spinning at renderer speed.
        r.settle_until = now + 0.01;
        let complete = match op {
            Op::Open => {
                let i = self.scenarios.iter().position(|s| s.id == ID).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::View(n) => {
                self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(n).copied())
            }
            Op::Command(command, status) => {
                if r.phase == 0 {
                    if self
                        .click_visible_smoke_control(ctx, self.metrics.package_command.map(|x| x.0))
                    {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let id = self.metrics.package_command.unwrap().1;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
                    self.pending_events.push(egui::Event::Paste(command));
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    // Assert delivery before Run. A successful Run deliberately
                    // clears the command field while its click settles.
                    assert_eq!(terminal(self).command, command);
                    r.phase = 3;
                    false
                } else if r.phase == 3 {
                    if self.click_visible_smoke_control(ctx, self.metrics.package_run.map(|x| x.0))
                    {
                        r.phase = 4;
                    }
                    false
                } else {
                    let entry = terminal(self).entries.last().unwrap();
                    assert_eq!(entry.command, command, "actual native command");
                    assert_eq!(entry.status, status, "{}: {}", command, entry.stderr);
                    true
                }
            }
            Op::Check(expected) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.check) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    let s = &self.scenarios[i];
                    let a = &self.progress.attempts[ID];
                    let practice = self.debian_packages_view(i) == teaching::View::Example;
                    let passed = if practice {
                        a.workspace
                            .as_ref()
                            .unwrap()
                            .debian_packages_learning
                            .as_ref()
                            .unwrap()
                            .checks(s.debian_packages_teaching.as_ref().unwrap())
                            .iter()
                            .all(|r| r.0)
                    } else {
                        progress::current_solved(a, s)
                    };
                    assert_eq!(passed, expected, "native current requirement results");
                    if practice {
                        assert!(!progress::current_solved(a, s));
                    }
                    true
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Reset => self.click_visible_smoke_control(ctx, self.metrics.reset),
            Op::Confirm => self.click_visible_smoke_control(ctx, self.metrics.confirm_reset),
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[ID].state).unwrap(),
                    serde_json::to_value(&s.initial).unwrap()
                );
                assert!(!progress::current_solved(&self.progress.attempts[ID], s));
                true
            }
            Op::Shot(name, output) => {
                if r.phase == 0 {
                    let rect = if output {
                        self.metrics.output.unwrap()
                    } else {
                        self.metrics.package_result.unwrap()
                    };
                    if self.reveal_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    if output {
                        let (offset, max, _) = self.metrics.package_output_scroll.unwrap();
                        if (offset - max).abs() >= 2. {
                            // Keep the same required final position; let the native
                            // scroll animation settle within the route's hard bound.
                            ctx.request_repaint_after(std::time::Duration::from_millis(25));
                            ctx.data_mut(|d| d.insert_temp(key, r));
                            return;
                        }
                    }
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
            // The optimized native binary can repaint faster than smooth-wheel
            // motion. Keep the visibility requirement and twenty-try bound;
            // allow each real wheel event to settle before retrying it.
            eprintln!(
                "CONTENTS_WHEEL root={root} time={now:.6} step={} phase={} offset={:?} content={:?} check={:?}",
                r.at,
                r.phase,
                self.metrics.main_scroll_offset,
                self.metrics.main_content_size,
                self.metrics.check
            );
            r.settle_until = now + 0.15;
        }
        if complete {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.action_started = None;
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
    fn contents_native_command_controller_uses_actual_focus_paste_run_and_status() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let key = egui::Id::new("debian-contents-native-route");
        let ops = vec![
            Op::Open,
            Op::View(2),
            command("apt-file search ledger-check"),
            Op::Check(false),
            command("apt-get --error-on=any update"),
            command("apt-get -y install records-tools"),
            Op::Check(true),
        ];
        let len = ops.len();
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    ..Default::default()
                },
            )
        });
        for n in 0..350 {
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
                app.debian_contents_tick(ctx);
                let snapshot=ctx.data(|d|d.get_temp::<Route>(key)).unwrap();
                let queued=app.pending_events.len();
                app.debian_contents_tick(ctx);
                let repeated=ctx.data(|d|d.get_temp::<Route>(key)).unwrap();
                assert_eq!((snapshot.at,snapshot.phase,snapshot.frames,queued),(repeated.at,repeated.phase,repeated.frames,app.pending_events.len()),"A repeated callback in one root input frame cannot advance or queue duplicate events");
                if ctx.current_pass_index() == 0 {
                    ctx.request_discard(
                        "Regression: native event phases stay once per input frame",
                    );
                }
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert!(progress::current_solved(
                    &app.progress.attempts[ID],
                    &app.scenarios[app.selected.unwrap()]
                ));
                return;
            }
        }
        panic!("actual Contents controller did not finish");
    }
    #[test]
    fn optimized_paced_narrow_view_switch_reaches_assignment_input_with_real_wheel_events() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let key = egui::Id::new("debian-contents-native-route");
        let ops = vec![
            Op::Open,
            Op::View(1),
            command("apt-get --error-on=any update"),
            command("apt-get -y install gallery-tools"),
            Op::Check(true),
            Op::View(2),
            Op::Command("dpkg-query -S /usr/bin/ledger-check".into(), 1),
            command("apt-file search ledger-check"),
            Op::Check(false),
        ];
        let len = ops.len();
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    ..Default::default()
                },
            )
        });
        for n in 0..6000 {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1268., 632.),
                )),
                time: Some(n as f64 / 200.),
                ..Default::default()
            };
            eframe::App::raw_input_hook(&mut app, &ctx, &mut input);
            let _ = ctx.run(input, |ctx| {
                app.render(ctx);
                app.debian_contents_tick(ctx);
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert_eq!(
                    terminal(&app).entries.last().unwrap().command,
                    "apt-file search ledger-check"
                );
                assert!(!progress::current_solved(
                    &app.progress.attempts[ID],
                    &app.scenarios[app.selected.unwrap()]
                ));
                return;
            }
        }
        panic!("Narrow view switch must reach the actual command field, not bypass visibility");
    }
    #[test]
    fn renderer_frames_cannot_exhaust_the_budget_while_a_real_time_wait_is_pending() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("debian-contents-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Open],
                    frames: 1199,
                    settle_until: 0.15,
                    action_started: Some(0.),
                    ..Default::default()
                },
            )
        });
        for n in 0..1500 {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(n as f64 / 100000.),
                    ..Default::default()
                },
                |ctx| app.debian_contents_tick(ctx),
            );
            let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!(
                route.frames, 1199,
                "Waiting is not another interaction attempt"
            );
            assert_eq!(route.at, 0, "Waiting must not silently complete the action");
        }
    }
    #[test]
    #[should_panic(expected = "exceeded15seconds")]
    fn a_waiting_native_action_still_has_a_real_elapsed_deadline() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("debian-contents-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Open],
                    settle_until: 30.,
                    action_started: Some(0.),
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                time: Some(16.),
                ..Default::default()
            },
            |ctx| app.debian_contents_tick(ctx),
        );
    }
    #[test]
    fn contents_capture_request_does_not_reenter_context_data_locks() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app.scenarios.iter().position(|s| s.id == ID).unwrap();
        app.launch(i);
        app.set_debian_packages_view(i, teaching::View::Assignment);
        let key = egui::Id::new("debian-contents-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("contents-regression-capture".into(), true)],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 3000.),
                )),
                ..Default::default()
            },
            |ctx| {
                app.render(ctx);
                app.debian_contents_tick(ctx);
            },
        );
        assert!(app.capture_pending);
        assert!(
            out.viewport_output
                .values()
                .flat_map(|x| &x.commands)
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        );
    }
}
