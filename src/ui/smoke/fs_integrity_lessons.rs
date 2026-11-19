//! Fixed virtual-filesystem flows; native input and private smoke profile only.
use super::*;
use dlms_learning_lab::fs_integrity_workspace as work;

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
    for s in labs
        .iter()
        .filter(|s| s.unit == "linux-filesystem-consistency")
    {
        count += 1;
        let engine::Goal::FsIntegrity { definition } = &s.goal else {
            unreachable!()
        };
        ops.extend([
            Op::Open(s.id.clone()),
            Op::View(1),
            command("findmnt"),
            Op::Check(false),
        ]);
        let mut first_readonly = true;
        for text in &definition.example_commands {
            let status = if text.contains("-fn") {
                let status = if first_readonly { 4 } else { 0 };
                first_readonly = false;
                status
            } else if text.contains("-fp") {
                if s.id.ends_with("recovery") { 4 } else { 1 }
            } else if text.contains("-fy") {
                1
            } else {
                0
            };
            ops.push(Op::Command(text.clone(), status));
        }
        ops.extend([
            Op::Check(true),
            Op::Shot(format!("integrity-{}-practice-passed", s.id), false),
            command("umount /mnt/workshop"),
            Op::Check(false),
            Op::Shot(format!("integrity-{}-practice-partial", s.id), false),
            command("mount /dev/vdd1 /mnt/workshop"),
            Op::Check(true),
            Op::View(2),
            Op::Command("e2fsck -fy /dev/vdb1".into(), 8),
            Op::Check(false),
            Op::Shot(format!("integrity-{}-mounted-policy", s.id), true),
        ]);
        for text in &s.solution {
            match serde_json::from_str(text.strip_prefix(work::PREFIX).unwrap()).unwrap() {
                work::Action::Command { input } => {
                    let status = if input.contains("-fp") || input.contains("-fy") {
                        1
                    } else {
                        0
                    };
                    ops.push(Op::Command(input, status));
                }
                work::Action::Check {} => {}
            }
        }
        ops.extend([
            Op::Check(true),
            Op::Shot(format!("integrity-{}-assignment-passed", s.id), false),
            command("umount /mnt/data"),
            Op::Check(false),
            Op::Shot(format!("integrity-{}-assignment-partial", s.id), false),
            command("mount /dev/vdb1 /mnt/data"),
            Op::Check(true),
            Op::Save,
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
        ]);
    }
    assert_eq!(count, 2);
    ops
}
fn scope(app: &LabApp) -> &work::Scope {
    let s = &app.scenarios[app.selected.unwrap()];
    let engine::State::FsIntegrity { simulation } = &app.progress.attempts[&s.id].state else {
        unreachable!()
    };
    if simulation.view == teaching::View::Example {
        &simulation.example
    } else {
        &simulation.assignment
    }
}
impl LabApp {
    pub(in crate::ui) fn fs_integrity_lessons_tick(&mut self, ctx: &egui::Context) {
        // A layout retry is the same input frame. Never advance a pending-event
        // phase before raw_input_hook has delivered those events next frame.
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("fs-integrity-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        let key = egui::Id::new("fs-integrity-native-route");
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
                "FS_INTEGRITY_NATIVE_FLOW_COMPLETE: two distinct consistency repairs, actual virtual terminal output, mounted-check refusal, partial states, separate practice, independent assignments, damage/retry/save-read/reset verified; no host commands"
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
            "Integrity native action exceeded15seconds: {op:?}; phase={}",
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
            println!("FS_INTEGRITY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Integrity native action timed out: {op:?}; root={root} time={now} settle={} phase={} offset={:?} content={:?} check={:?} pending={:?}",
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
            Op::View(n) => self.click_visible_smoke_control(
                ctx,
                self.metrics
                    .fs_integrity
                    .controls
                    .iter()
                    .find(|c| c.0 == ["introduction", "example", "assignment"][n])
                    .map(|c| c.1),
            ),
            Op::Command(command, status) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(
                        ctx,
                        self.metrics.fs_integrity.command.map(|x| x.0),
                    ) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let id = self.metrics.fs_integrity.command.unwrap().1;
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
                    assert_eq!(scope(self).command, command);
                    r.phase = 3;
                    false
                } else if r.phase == 3 {
                    if self.click_visible_smoke_control(
                        ctx,
                        self.metrics
                            .fs_integrity
                            .controls
                            .iter()
                            .find(|c| c.0 == "run")
                            .map(|c| c.1),
                    ) {
                        r.phase = 4;
                    }
                    false
                } else {
                    let entry = scope(self).transcript.last().unwrap();
                    assert_eq!(entry.input, command, "actual native command");
                    assert_eq!(entry.status, status, "{}: {}", command, entry.text);
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
                    if self.fs_integrity_run.is_some() {
                        ctx.data_mut(|d| d.insert_temp(key, r));
                        ctx.request_repaint();
                        return;
                    }
                    let engine::State::FsIntegrity { simulation } = &a.state else {
                        unreachable!()
                    };
                    let practice = simulation.view == teaching::View::Example;
                    let passed = if practice {
                        simulation.example.passed(&simulation.definition.example)
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
                        self.metrics.fs_integrity.output.unwrap()
                    } else {
                        let label = self.metrics.fs_integrity.result.unwrap().2;
                        let line = ctx.style().text_styles[&egui::TextStyle::Body].size;
                        egui::Rect::from_min_max(
                            label.min,
                            egui::pos2(label.max.x, label.max.y + line * 10.),
                        )
                    };
                    if self.reveal_smoke_control(ctx, rect) {
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
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            // The optimized native binary can repaint faster than smooth-wheel
            // motion. Keep the visibility requirement and twenty-try bound;
            // allow each real wheel event to settle before retrying it.
            eprintln!(
                "FS_INTEGRITY_WHEEL root={root} time={now:.6} step={} phase={} offset={:?} content={:?} check={:?}",
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
    fn integrity_native_route_keeps_two_real_tasks_and_context_safe_capture() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&app.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        let mut first = true;
        let mut seen = 0;
        for op in &ops {
            match op {
                Op::Open(_) => first = true,
                Op::Command(text, status) if text == "e2fsck -fn /dev/vdd1" => {
                    assert_eq!(*status, if first { 4 } else { 0 });
                    first = false;
                    seen += 1;
                }
                _ => {}
            }
        }
        assert_eq!(
            seen, 4,
            "Both workshops diagnose before and verify after repair"
        );

        // Exercise every static native expectation against both authored tasks
        // before spending a native launch on a mistaken cross-task assumption.
        let mut current: Option<work::Session> = None;
        for op in &ops {
            match op {
                Op::Open(id) => {
                    let source = app.scenarios.iter().find(|s| s.id == *id).unwrap();
                    let engine::Goal::FsIntegrity { definition } = &source.goal else {
                        unreachable!()
                    };
                    current = Some(work::Session::new((**definition).clone()).unwrap());
                }
                Op::View(n) => {
                    current.as_mut().unwrap().view = if *n == 1 {
                        teaching::View::Example
                    } else {
                        teaching::View::Assignment
                    }
                }
                Op::Command(text, expected) => {
                    let session = current.as_mut().unwrap();
                    let (task, scope) = if session.view == teaching::View::Example {
                        (&session.definition.example, &mut session.example)
                    } else {
                        (&session.definition.assignment, &mut session.assignment)
                    };
                    let status = scope
                        .execute_recorded(task, text)
                        .map_or_else(|e| e.status, |o| o.status);
                    assert_eq!(status, *expected, "Recipe {:?}: {text}", session.view);
                }
                Op::Check(expected) => {
                    let session = current.as_mut().unwrap();
                    let (task, scope) = if session.view == teaching::View::Example {
                        (&session.definition.example, &mut session.example)
                    } else {
                        (&session.definition.assignment, &mut session.assignment)
                    };
                    assert_eq!(
                        scope
                            .run_checked(task)
                            .unwrap()
                            .iter()
                            .all(work::CaseResult::passed),
                        *expected
                    );
                }
                Op::Confirm => {
                    let definition = current.as_ref().unwrap().definition.clone();
                    current = Some(work::Session::new(definition).unwrap());
                }
                _ => {}
            }
        }
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-fs-offline-accounting")
            .unwrap();
        app.launch(i);
        if let engine::State::FsIntegrity { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-fs-offline-accounting")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            simulation
                .assignment
                .run_checked(&simulation.definition.assignment)
                .unwrap();
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
        let key = egui::Id::new("fs-integrity-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("integrity-context-regression".into(), false)],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.fs_integrity_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}
