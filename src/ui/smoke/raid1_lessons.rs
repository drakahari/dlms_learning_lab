//! Fixed native route; evidence exists only after actual guarded execution.
//! Drives normal controls/events only. It never applies a model solution or
//! installs precomputed results into an attempt.
use super::*;
use dlms_learning_lab::{raid1 as core, raid1_workspace as work};

#[derive(Clone, Debug)]
enum ShotTarget {
    Introduction,
    Output,
    Result,
}
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    View(usize),
    SelectStep(usize),
    Draft(String),
    Command(String, u8),
    Advance { copied: u16, total: u16 },
    Check(bool),
    PracticeIsolated,
    Save,
    Reset,
    Confirm,
    Fresh,
    Shot(String, ShotTarget),
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
    let mut ids = vec![];
    for source in labs.iter().filter(|s| s.unit == "linux-raid1-recovery") {
        ids.push(source.id.as_str());
        let engine::Goal::Raid1 { definition } = &source.goal else {
            panic!("RAID unit must hold its authored definition")
        };
        let demo = &definition.lesson;
        let demo_inspect = format!("mdadm --detail {}", demo.plan.array);
        let inspect = format!("mdadm --detail {}", definition.plan.array);
        let too_small = definition
            .initial
            .devices
            .iter()
            .find(|(_, d)| {
                d.media == core::Media::Blank
                    && d.available
                    && !d.read_only
                    && !d.protected
                    && d.usable_sectors
                        < definition.initial.arrays[&definition.plan.array].component_sectors
            })
            .expect("Authored negative media must be present")
            .0;
        let survivor = definition.initial.arrays[&definition.plan.array]
            .members
            .iter()
            .find(|m| m.role == core::Role::Synced)
            .expect("Authored readable source")
            .device
            .clone();
        ops.extend([
            Op::Open(source.id.clone()),
            Op::Shot(
                format!("raid1-{}-introduction", source.id),
                ShotTarget::Introduction,
            ),
            Op::View(1),
        ]);
        let mut advances = 0;
        for (index, step) in demo.worked.iter().enumerate() {
            ops.push(Op::SelectStep(index));
            if let Some(command_text) = &step.command {
                ops.push(command(command_text));
            } else {
                let Some(core::Action::Advance { units, .. }) = &step.action else {
                    panic!("This native route supports authored Advance steps only")
                };
                assert_eq!(*units, 4, "Native button must match the worked action");
                advances += 1;
                ops.push(Op::Advance {
                    copied: advances * 4,
                    total: 8,
                });
                ops.push(Op::Check(advances == 2));
                if advances == 1 {
                    ops.extend([
                        Op::Shot(
                            format!("raid1-{}-practice-partial", source.id),
                            ShotTarget::Result,
                        ),
                        Op::Draft(demo_inspect.clone()),
                        Op::Save,
                        Op::PracticeIsolated,
                    ]);
                }
            }
        }
        assert_eq!(
            advances, 2,
            "Eight segments require two actual native advances"
        );
        ops.extend([
            Op::Check(true),
            Op::PracticeIsolated,
            Op::Shot(
                format!("raid1-{}-practice-passed", source.id),
                ShotTarget::Result,
            ),
            Op::View(2),
            Op::Draft(inspect.clone()),
            Op::Save,
            Op::Command(
                format!("mdadm {} --add {too_small}", definition.plan.array),
                1,
            ),
            Op::Shot(
                format!("raid1-{}-wrong-media-output", source.id),
                ShotTarget::Output,
            ),
            Op::Check(false),
            Op::Shot(
                format!("raid1-{}-wrong-media-feedback", source.id),
                ShotTarget::Result,
            ),
        ]);
        for solution in &source.solution {
            if solution.starts_with(work::PREFIX) {
                continue;
            }
            ops.push(command(solution));
        }
        ops.extend([
            Op::Advance {
                copied: 4,
                total: 8,
            },
            Op::Check(false),
            Op::Shot(
                format!("raid1-{}-assignment-partial", source.id),
                ShotTarget::Result,
            ),
            Op::Draft(inspect.clone()),
            Op::Save,
            Op::Advance {
                copied: 8,
                total: 8,
            },
            Op::Check(true),
            Op::Save,
            Op::Shot(
                format!("raid1-{}-assignment-passed", source.id),
                ShotTarget::Result,
            ),
            command(&format!(
                "mdadm {} --fail {survivor}",
                definition.plan.array
            )),
            Op::Check(false),
            Op::Shot(
                format!("raid1-{}-success-invalidated", source.id),
                ShotTarget::Result,
            ),
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
        ]);
    }
    assert_eq!(ids, ["lpic2-raid1-replace", "lpic2-raid1-handover"]);
    ops
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::Raid1 { simulation } = &a.progress.attempts[&source.id].state else {
        unreachable!()
    };
    simulation
}
fn practice(a: &LabApp) -> bool {
    session(a).view == teaching::View::Example
}
fn draft(a: &LabApp) -> &str {
    let s = session(a);
    if practice(a) {
        &s.example_command
    } else {
        &s.command
    }
}
impl LabApp {
    pub(in crate::ui) fn raid1_popup_choice(&mut self, ctx: &egui::Context, index: usize) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let name = format!("example:{index}");
        let Some((_, rect, clip)) = self.metrics.raid1.controls.iter().find(|c| c.0 == name) else {
            ctx.request_repaint();
            return false;
        };
        let (rect, clip) = (*rect, clip.intersect(ctx.content_rect()));
        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            return self.click_smoke_control(ctx, Some(rect));
        }
        assert!(
            clip.is_positive(),
            "Native guided popup must have a real visible viewport"
        );
        self.pending_events.extend([
            egui::Event::PointerMoved(clip.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (clip.center().y - rect.center().y)
                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                ),
                modifiers: Default::default(),
            },
        ]);
        false
    }

    pub(in crate::ui) fn raid1_lessons_tick(&mut self, ctx: &egui::Context) {
        // A layout retry is not a new raw-input frame. Do not consume queued
        // events until the normal input hook has delivered them on the next one.
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("raid1-lessons-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        let key = egui::Id::new("raid1-lessons-native-route");
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
                "RAID1_NATIVE_FLOW_COMPLETE: two authored mirrors through real terminal input, actual partial/full copying, wrong-media status/feedback, isolated practice, pending draft save/read, late-failure invalidation and selected reset; no host devices or commands"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 94000 + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let began = *r.action_started.get_or_insert(now);
        assert!(
            now - began < 15.,
            "RAID1 native action exceeded15seconds: {op:?}; phase={}",
            r.phase
        );
        if now < r.settle_until {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(10));
            return;
        }
        if r.frames == 0 {
            println!("RAID1_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "RAID1 native action timed out: {op:?}; root={root} time={now} settle={} phase={} offset={:?} content={:?} result={:?} pending={:?}",
            r.settle_until,
            r.phase,
            self.metrics.main_scroll_offset,
            self.metrics.main_content_size,
            self.metrics.raid1.result,
            self.pending_events
        );
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
            Op::SelectStep(index) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .raid1
                        .controls
                        .iter()
                        .find(|c| c.0 == "example-picker")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    if self.raid1_popup_choice(ctx, index) {
                        r.phase = 2;
                    }
                    false
                } else {
                    assert_eq!(session(self).selected_example, index);
                    true
                }
            }
            Op::Draft(text) | Op::Command(text, _) if r.phase < 3 => {
                if r.phase == 0 {
                    if self
                        .click_visible_smoke_control(ctx, self.metrics.raid1.command.map(|x| x.0))
                    {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let id = self.metrics.raid1.command.unwrap().1;
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
                    self.pending_events.push(egui::Event::Paste(text));
                    r.phase = 2;
                    false
                } else {
                    assert_eq!(draft(self), text, "actual native command draft before Run");
                    r.phase = 3;
                    false
                }
            }
            Op::Draft(text) => {
                assert_eq!(draft(self), text);
                true
            }
            Op::Command(text, status) => {
                if r.phase == 3 {
                    let rect = self
                        .metrics
                        .raid1
                        .controls
                        .iter()
                        .find(|c| c.0 == "run")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 4;
                    }
                    false
                } else {
                    let s = session(self);
                    let rows = if practice(self) {
                        &s.example_transcript
                    } else {
                        &s.transcript
                    };
                    let row = rows
                        .last()
                        .expect("Actual Run must append a terminal result");
                    assert_eq!(row.input, text, "actual native command");
                    assert_eq!(row.status, status, "{text}: {}", row.output);
                    if status == 1 {
                        assert!(
                            row.output.contains("too small"),
                            "The negative exercises usable-capacity feedback: {}",
                            row.output
                        );
                    }
                    true
                }
            }
            Op::Advance { copied, total } => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .raid1
                        .controls
                        .iter()
                        .find(|c| c.0 == "advance")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    let engine::Goal::Raid1 { definition } = &self.scenarios[i].goal else {
                        unreachable!()
                    };
                    let s = session(self);
                    let (world, target) = if practice(self) {
                        (&s.example, &definition.lesson.plan.array)
                    } else {
                        (&s.current, &definition.plan.array)
                    };
                    if copied < total {
                        assert_eq!(
                            world.health(target).unwrap(),
                            core::Health::Rebuilding { copied, total }
                        );
                    } else {
                        assert_eq!(world.health(target).unwrap(), core::Health::Healthy);
                    }
                    let rows = if practice(self) {
                        &s.example_transcript
                    } else {
                        &s.transcript
                    };
                    assert_eq!(
                        rows.last().unwrap().input,
                        "Advance simulated recovery (native action)"
                    );
                    true
                }
            }
            Op::Check(expected) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .raid1
                        .controls
                        .iter()
                        .find(|c| c.0 == "check")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let source = &self.scenarios[self.selected.unwrap()];
                    let attempt = &self.progress.attempts[&source.id];
                    let engine::Goal::Raid1 { definition } = &source.goal else {
                        unreachable!()
                    };
                    let s = session(self);
                    let rows = if practice(self) {
                        assert!(s.example_checked);
                        s.example_checks(definition)
                    } else {
                        assert!(s.checked);
                        definition.checks(s)
                    };
                    let passed = rows.iter().filter(|r| r.0).count();
                    let metric = self
                        .metrics
                        .raid1
                        .result
                        .expect("Actual rendered requirement results");
                    assert_eq!((metric.0, metric.1), (passed, rows.len()));
                    assert_eq!(
                        passed == rows.len(),
                        expected,
                        "actual native requirements: {rows:?}"
                    );
                    if practice(self) {
                        assert!(!progress::current_solved(attempt, source));
                    } else {
                        assert_eq!(progress::current_solved(attempt, source), expected);
                    }
                    true
                }
            }
            Op::PracticeIsolated => {
                let source = &self.scenarios[self.selected.unwrap()];
                let attempt = &self.progress.attempts[&source.id];
                let engine::Goal::Raid1 { definition } = &source.goal else {
                    unreachable!()
                };
                assert_eq!(
                    session(self).current,
                    definition.initial,
                    "Guided work must not change the assignment"
                );
                assert!(
                    session(self).command.is_empty(),
                    "Example drafts must not replace assignment drafts"
                );
                assert!(!progress::current_solved(attempt, source));
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap(),
                    "Actual private progress save/read retains model, pending drafts and separate example"
                );
                true
            }
            Op::Reset => self.click_visible_smoke_control(ctx, self.metrics.reset),
            Op::Confirm => self.click_visible_smoke_control(ctx, self.metrics.confirm_reset),
            Op::Fresh => {
                let source = &self.scenarios[self.selected.unwrap()];
                let attempt = &self.progress.attempts[&source.id];
                assert_eq!(
                    serde_json::to_value(&attempt.state).unwrap(),
                    serde_json::to_value(&source.initial).unwrap()
                );
                assert!(!progress::current_solved(attempt, source));
                true
            }
            Op::Shot(name, target) => {
                if r.phase == 0 {
                    let rect = match target {
                        ShotTarget::Introduction => {
                            self.metrics.lesson_tabs.first().copied().unwrap()
                        }
                        ShotTarget::Output => self.metrics.output.unwrap(),
                        ShotTarget::Result => self.metrics.raid1.result.unwrap().2,
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
            eprintln!(
                "RAID1_WHEEL root={root} time={now:.6} step={} phase={} offset={:?} content={:?} result={:?}",
                r.at,
                r.phase,
                self.metrics.main_scroll_offset,
                self.metrics.main_content_size,
                self.metrics.raid1.result
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
    fn raid1_fixed_route_keeps_partial_copies_errors_isolation_and_real_reset_in_its_recipe() {
        let (labs, _) = crate::ui::tests::ccna_pilot::catalog();
        let ops = route(&labs);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::Advance { .. }))
                .count(),
            8
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 8);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Shot(..))).count(), 16);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::Command(_, 1)))
                .count(),
            2
        );
    }
    #[test]
    fn raid1_actual_capture_phase_requests_a_native_screenshot_without_context_reentry_or_work_changes()
     {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-raid1-replace")
            .unwrap();
        a.launch(i);
        let before = serde_json::to_value(&a.progress).unwrap();
        let key = egui::Id::new("raid1-lessons-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot(
                        "raid1-capture-regression".into(),
                        ShotTarget::Introduction,
                    )],
                    ..Default::default()
                },
            )
        });
        let mut seen = false;
        for n in 0..20 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    time: Some(n as f64 * 0.05),
                    events: std::mem::take(&mut a.pending_events),
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    a.raid1_lessons_tick(ctx);
                },
            );
            seen = output.viewport_output.values().any(|v| {
                v.commands
                    .iter()
                    .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
            });
            if seen {
                break;
            }
        }
        assert!(seen && a.capture_pending);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    }
}
