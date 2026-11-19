//! Fixed conflict-repair flows through native input; private smoke profile only.
use super::*;
use dlms_learning_lab::debian_packages_terminal::Session;

#[derive(Clone, Debug)]
enum Op {
    Open(String),
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
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| s.unit == "debian-package-conflicts") {
        count += 1;
        let versioned = s.id.ends_with("upgrade");
        let (old, new, worked_old, worked_new) = if versioned {
            (
                "branch-relay",
                "branch-dashboard",
                "studio-relay",
                "studio-dashboard",
            )
        } else {
            (
                "ops-legacy",
                "ops-delivery",
                "gallery-legacy",
                "gallery-delivery",
            )
        };
        let lesson = s.debian_packages_teaching.as_ref().unwrap();
        ops.extend([
            Op::Open(s.id.clone()),
            Op::View(1),
            command(&format!("apt-cache show {worked_new}")),
            command(&format!("dpkg-query -W {worked_old}")),
            Op::Command(format!("apt-get -y install {worked_new}"), 100),
            Op::Check(false),
            Op::Shot(format!("conflicts-{}-practice-refusal", s.id), true),
        ]);
        for a in &lesson.example_actions {
            ops.push(command(a.command.as_ref().unwrap()));
        }
        ops.extend([
            Op::Check(true),
            Op::Shot(format!("conflicts-{}-practice-passed", s.id), false),
            command(&format!(
                "dpkg --remove {}",
                if versioned { worked_old } else { worked_new }
            )),
            Op::Check(false),
            Op::Shot(format!("conflicts-{}-practice-partial", s.id), false),
            command(&format!(
                "apt-get -y install {}",
                if versioned { worked_old } else { worked_new }
            )),
            Op::Check(true),
            Op::View(2),
            command(&format!("apt-cache show {new}")),
            command(&format!("dpkg-query -W {old}")),
            Op::Command(format!("apt-get -y install {new}"), 100),
            Op::Check(false),
            Op::Shot(format!("conflicts-{}-assignment-refusal", s.id), true),
            command(&s.solution[0]),
            Op::Check(false),
            Op::Shot(format!("conflicts-{}-assignment-partial", s.id), false),
            command(&s.solution[1]),
            Op::Check(true),
            Op::Save,
            Op::Shot(format!("conflicts-{}-assignment-passed", s.id), false),
            command(&format!("dpkg --remove {new}")),
            Op::Check(false),
            command(&format!("apt-get -y install {new}")),
            Op::Check(true),
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 2);
    ops
}
fn terminal(a: &LabApp) -> &Session {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let w = a.progress.attempts[id].workspace.as_ref().unwrap();
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
    pub(in crate::ui) fn debian_conflicts_tick(&mut self, ctx: &egui::Context) {
        // A layout retry is the same input frame. Never advance a pending-event
        // phase before raw_input_hook has delivered those events next frame.
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("debian-conflicts-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        let key = egui::Id::new("debian-conflicts-native-route");
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
                "DEBIAN_CONFLICTS_NATIVE_FLOW_COMPLETE: two distinct compatibility repairs, actual metadata/installed queries, explicit conflict refusals, partial states, separate practice, independent assignments, damage/retry/save-read/reset verified; no host commands"
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
            "Conflict native action exceeded15seconds: {op:?}; phase={}",
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
            println!("DEBIAN_CONFLICTS_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Conflict native action timed out: {op:?}; root={root} time={now} settle={} phase={} offset={:?} content={:?} check={:?} pending={:?}",
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
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
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
                    let a = &self.progress.attempts[&s.id];
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
                    serde_json::to_value(&self.progress.attempts[&s.id].state).unwrap(),
                    serde_json::to_value(&s.initial).unwrap()
                );
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
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
                "CONFLICTS_WHEEL root={root} time={now:.6} step={} phase={} offset={:?} content={:?} check={:?}",
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
    const ID: &str = "lpic1-package-conflict-retirement";
    #[test]
    fn conflicts_native_controller_delivers_real_input_once_per_root_frame() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let full = route(&app.scenarios);
        assert_eq!(full.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        let key = egui::Id::new("debian-conflicts-native-route");
        let ops = vec![
            Op::Open(ID.into()),
            Op::View(2),
            command("apt-cache show ops-delivery"),
            Op::Command("apt-get -y install ops-delivery".into(), 100),
            Op::Check(false),
            command("dpkg --remove ops-legacy"),
            Op::Check(false),
            command("apt-get -y install ops-delivery"),
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
        for n in 0..900 {
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
                app.debian_conflicts_tick(ctx);
                let before = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
                let pending = app.pending_events.len();
                app.debian_conflicts_tick(ctx);
                let after = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
                assert_eq!(
                    (before.at, before.phase, before.frames, pending),
                    (
                        after.at,
                        after.phase,
                        after.frames,
                        app.pending_events.len()
                    )
                );
                if ctx.current_pass_index() == 0 {
                    ctx.request_discard("Native event phases remain once per root input frame");
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
        panic!("finite native command controller did not finish");
    }
    #[test]
    fn conflicts_native_capture_uses_real_context_safe_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app.scenarios.iter().position(|s| s.id == ID).unwrap();
        app.launch(i);
        app.set_debian_packages_view(i, teaching::View::Assignment);
        let key = egui::Id::new("debian-conflicts-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("conflicts-regression-capture".into(), false)],
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
                app.debian_conflicts_tick(ctx);
            },
        );
        assert!(app.capture_pending);
        assert!(
            out.viewport_output
                .values()
                .flat_map(|v| &v.commands)
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        );
    }
}
