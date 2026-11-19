//! Fixed conflict-repair flows through native input; private smoke profile only.
use super::*;
use dlms_learning_lab::debian_packages_terminal::Session;

#[derive(Clone, Debug)]
enum Op {
    Open(String),
    View(usize),
    Command(String, u8),
    Control(String),
    Choose(String, dlms_learning_lab::debian_packages::ConffileDecision),
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
fn configuration(
    ops: &mut Vec<Op>,
    d: &dlms_learning_lab::debian_packages_workspace::Definition,
    wrong: bool,
) {
    use dlms_learning_lab::{
        debian_packages::ConffileDecision, debian_packages_workspace::Requirement,
    };
    ops.push(Op::Control("review".into()));
    ops.push(Op::Save);
    let goal = d
        .requirements
        .iter()
        .find_map(|r| {
            if let Requirement::ConfigurationUpgrade { outcome } = r {
                Some(outcome)
            } else {
                None
            }
        })
        .unwrap();
    for row in &goal.choices {
        let choice = if wrong {
            match row.decision {
                ConffileDecision::KeepLocal => ConffileDecision::InstallNew,
                ConffileDecision::InstallNew => ConffileDecision::KeepLocal,
            }
        } else {
            row.decision
        };
        ops.push(Op::Choose(row.path.clone(), choice));
    }
    ops.push(Op::Control("apply".into()));
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| s.unit == "debian-conffile-upgrades") {
        count += 1;
        let lesson = s.debian_packages_teaching.as_ref().unwrap();
        let engine::Goal::DebianPackages { definition } = &s.goal else {
            unreachable!()
        };
        let find = |d: &dlms_learning_lab::debian_packages_workspace::Definition| {
            d.requirements.iter().find_map(|r|if let dlms_learning_lab::debian_packages_workspace::Requirement::ConfigurationUpgrade{outcome}=r{Some(outcome.package.clone())}else{None}).unwrap()
        };
        let demo = find(&lesson.definition);
        let demo_viewer = demo.trim_end_matches("-report").to_owned() + "-viewer";
        let pkg = find(definition);
        let viewer = pkg.trim_end_matches("-report").to_owned() + "-viewer";
        ops.extend([
            Op::Open(s.id.clone()),
            Op::View(1),
            command("apt-get --error-on=any update"),
        ]);
        configuration(&mut ops, &lesson.definition, false);
        ops.extend([
            Op::Check(true),
            Op::Shot(format!("conffile-{}-practice-passed", s.id), false),
            command(&format!("dpkg -r {demo_viewer}")),
            Op::Check(false),
            command(&format!("apt-get -y install {demo_viewer}")),
            Op::Check(true),
            Op::View(2),
            command("apt-get --error-on=any update"),
        ]);
        configuration(&mut ops, definition, true);
        ops.extend([
            Op::Check(false),
            Op::Shot(format!("conffile-{}-wrong-policy", s.id), false),
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
            Op::View(2),
            command("apt-get --error-on=any update"),
        ]);
        configuration(&mut ops, definition, false);
        ops.extend([
            Op::Check(true),
            Op::Save,
            Op::Shot(format!("conffile-{}-passed", s.id), false),
            command(&format!("dpkg -r {viewer}")),
            Op::Check(false),
            command(&format!("apt-get -y install {viewer}")),
            Op::Check(true),
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 3);
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
    pub(in crate::ui) fn conffile_lessons_tick(&mut self, ctx: &egui::Context) {
        // A layout retry is the same input frame. Never advance a pending-event
        // phase before raw_input_hook has delivered those events next frame.
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("conffile-lessons-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        let key = egui::Id::new("conffile-lessons-native-route");
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
                "CONFFILES_NATIVE_FLOW_COMPLETE: three per-file configuration upgrades through actual native choices, wrong policy, separate practice, protected-viewer damage/retry, pending draft save/read and selected reset verified; no host commands"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 93000 + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let began = *r.action_started.get_or_insert(now);
        assert!(
            now - began < 15.,
            "Configuration native action exceeded15seconds: {op:?}; phase={}",
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
            println!("CONFFILES_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Configuration native action timed out: {op:?}; root={root} time={now} settle={} phase={} offset={:?} content={:?} check={:?} pending={:?}",
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
            Op::Control(name) => {
                let rect = self
                    .metrics
                    .package_configuration
                    .iter()
                    .find(|r| r.0 == name)
                    .map(|r| r.1);
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Choose(path, choice) => {
                if r.phase == 0 {
                    let name = format!("choice:{path}");
                    let rect = self
                        .metrics
                        .package_configuration
                        .iter()
                        .find(|r| r.0 == name)
                        .map(|r| r.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let name = format!(
                        "{path}:{}",
                        if choice == dlms_learning_lab::debian_packages::ConffileDecision::KeepLocal
                        {
                            "keep"
                        } else {
                            "new"
                        }
                    );
                    let rect = self
                        .metrics
                        .package_configuration
                        .iter()
                        .find(|r| r.0 == name)
                        .map(|r| r.1);
                    if self.click_smoke_control(ctx, rect) {
                        r.phase = 2;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    let w = self.progress.attempts[&self.scenarios[i].id]
                        .workspace
                        .as_ref()
                        .unwrap();
                    let d = if self.debian_packages_view(i) == teaching::View::Example {
                        &w.debian_packages_learning.as_ref().unwrap().draft
                    } else {
                        w.debian_packages_draft.as_ref().unwrap()
                    };
                    assert_eq!(
                        d.conffile_upgrade
                            .as_ref()
                            .unwrap()
                            .files
                            .iter()
                            .find(|r| r.path == path)
                            .unwrap()
                            .decision,
                        Some(choice)
                    );
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
                "CONFFILES_WHEEL root={root} time={now:.6} step={} phase={} offset={:?} content={:?} check={:?}",
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
    fn conffile_native_recipe_covers_three_distinct_real_choices_wrong_policy_pending_saves_and_resets()
     {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios);
        let opens: Vec<_> = ops
            .iter()
            .filter_map(|o| {
                if let Op::Open(id) = o {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            opens,
            [
                "lpic1-conffile-retain",
                "lpic1-conffile-adopt",
                "lpic1-conffile-handover"
            ]
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Control(n)if n=="review"))
                .count(),
            9
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Control(n)if n=="apply"))
                .count(),
            9
        );
        assert_eq!(
            ops.iter().filter(|o| matches!(o, Op::Choose(..))).count(),
            9
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 6);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 12);
    }
}
