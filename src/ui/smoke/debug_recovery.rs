//! Real native events for fixed authored repair projects; private smoke progress only.
use super::*;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(&'static str),
    Type(String),
    Result(bool),
    Example(bool),
    Save,
    Fresh,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    // QA scroll pacing uses elapsed time, independent of native frame rate.
    wheel_until: f64,
    done: bool,
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    route_for(labs, "shell-debug-recovery")
}
fn route_for(labs: &[Scenario], unit: &str) -> Vec<Op> {
    let mut ops = vec![];
    let triage = unit == "shell-log-correlation";
    let managed = unit == "shell-managed-configuration";
    let backups = unit == "shell-content-backups";
    let maps = unit == "shell-associative-maps";
    let file_lifecycle = unit == "shell-file-lifecycle";
    let tracing = unit == "shell-tracing-diagnosis";
    let lookup = unit == "linux-command-search";
    let environment = lookup || unit == "linux-shell-environment";
    let prefix = if triage {
        "log-triage"
    } else if backups {
        "content-backup"
    } else if lookup {
        "path-lookup"
    } else if managed {
        "managed-config"
    } else if maps {
        "associative"
    } else if file_lifecycle {
        "file-lifecycle"
    } else if environment {
        "environment"
    } else if tracing {
        "tracing"
    } else {
        "debug"
    };
    let mut count = 0;
    for s in labs.iter().filter(|s| s.unit == unit) {
        count += 1;
        let lesson = s.teaching.as_ref().unwrap();
        let changed = if triage {
            lesson
                .example
                .replace("owner=\"${owners[$device]}\"", "owner='Unassigned'")
        } else if backups {
            if s.id == "shell-backup-content-set" {
                lesson.example.replace(
                    "cat < \"$name\" > \"saved-$name\"",
                    "content=$(cat < \"$name\")\n    printf '%s\\n' \"$content\" > \"saved-$name\"",
                )
            } else {
                lesson.example.replace(" || test -n \"$name\"", "")
            }
        } else if lookup {
            lesson.example.replace("$TOOL_DIR", "$LEGACY_DIR")
        } else if managed {
            lesson
                .example
                .replace("while IFS= read -r", "while read -r")
        } else if maps {
            format!("{}printf '%s\\n' 'extra observation'\n", lesson.example)
        } else if file_lifecycle {
            match s.id.as_str() {
                "shell-stage-report" => lesson
                    .example
                    .replace("final='notice.txt'", "final='board.txt'"),
                "shell-cleanup-preview" => lesson.example.replace("mode=preview", "mode=apply"),
                _ => lesson
                    .example
                    .replace("cat < 'workshop-next'", "cat < 'workshop-backup'"),
            }
        } else if environment {
            if s.id.ends_with("parent-note") {
                lesson
                    .example
                    .replace("export -n REMINDER", "REMINDER=\"\"")
            } else {
                lesson
                    .example
                    .replace("export LOCATION STYLE", "export STYLE")
            }
        } else if tracing && s.id.ends_with("delivery") {
            lesson.example.replace("\"$place\"", "$place")
        } else if tracing {
            lesson
                .example
                .replace("first='gallery.log'", "first='live.log'")
        } else if s.id.ends_with("debug-status") {
            lesson.example.replace("result=$?", "result=0")
        } else {
            lesson.example.replace("fern moss", "elm moss")
        };
        let start = ops.len();
        ops.extend([
            Op::Open(s.id.clone()),
            Op::Click("example"),
            Op::Click("run"),
            Op::Example(true),
            Op::Shot(format!("{prefix}-{}-practice-passed", s.id)),
            Op::Type(changed),
            Op::Click("run"),
            Op::Example(false),
            Op::Type(lesson.example.clone()),
            Op::Click("run"),
            Op::Example(true),
            Op::Click("assignment"),
            Op::Click("run"),
            Op::Result(false),
            Op::Shot(format!("{prefix}-{}-starter-feedback", s.id)),
            Op::Type(s.solution[0].clone()),
            Op::Click("run"),
            Op::Result(true),
            Op::Save,
            Op::Shot(format!("{prefix}-{}-passed", s.id)),
            Op::Type(if environment {
                ":\n".repeat(2000)
            } else {
                "while true; do :; done\n".into()
            }),
            Op::Result(false),
            Op::Click("run"),
            Op::Click("stop"),
            Op::Result(false),
            Op::Type(s.solution[0].clone()),
            Op::Click("run"),
            Op::Result(true),
            Op::Click("reset"),
            Op::Click("confirm"),
            Op::Fresh,
        ]);
        if lookup {
            let assignment = s.solution[0].clone();
            ops.splice(
                start + 19..start + 19,
                [
                    Op::Type("PATH=\"$TOOL_DIR:$BASE_PATH\"\n".into()),
                    Op::Click("run"),
                    Op::Result(false),
                    Op::Shot("path-lookup-missing-export-partial".into()),
                    Op::Type("export PATH=\"$TOOL_DIR:$BASE_PATH:/home/alex/bin\"\n".into()),
                    Op::Click("run"),
                    Op::Result(false),
                    Op::Shot("path-lookup-untrusted-suffix".into()),
                    Op::Type(assignment.replace("$TOOL_DIR:$BASE_PATH", "$BASE_PATH:$TOOL_DIR")),
                    Op::Click("run"),
                    Op::Result(true),
                    Op::Shot("path-lookup-equivalent-order-passed".into()),
                ],
            );
            ops.splice(
                start + 8..start + 8,
                [Op::Shot("path-lookup-practice-old-provider".into())],
            );
        }
        if tracing {
            ops.splice(
                start + 15..start + 15,
                [
                    Op::Type(format!("set -x\n{}", s.solution[0])),
                    Op::Click("run"),
                    Op::Result(false),
                    Op::Shot(format!("tracing-{}-traces-left-on", s.id)),
                ],
            );
        }
    }
    assert_eq!(
        count,
        if triage || managed || lookup {
            1
        } else if file_lifecycle {
            3
        } else {
            2
        }
    );
    ops
}
// A clipped ScrollArea can expose an empty or very thin viewport after
// scrolling to feedback. Never derive a full-height band from that rectangle.
fn focus_band(viewport: egui::Rect, height: f32) -> Option<egui::Rect> {
    let inner = viewport.shrink(1.);
    if !inner.is_positive() || inner.height() < height {
        return None;
    }
    Some(
        egui::Rect::from_min_size(
            egui::pos2(inner.left(), inner.center().y - height / 2.),
            egui::vec2(inner.width(), height),
        )
        .intersect(inner),
    )
}
impl LabApp {
    fn debug_recovery_focus(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let vp = self.metrics.editor_viewport.expect("real editor viewport");
        let height = ctx.style().spacing.interact_size.y;
        let Some(band) = focus_band(vp, height) else {
            let visible = self
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            let y = if vp.bottom() <= visible.top() + height {
                vp.bottom() - height
            } else {
                vp.top() + height
            };
            let target = egui::Rect::from_center_size(
                egui::pos2(visible.center().x, y),
                egui::vec2(8., height),
            );
            self.reveal_smoke_control(ctx, target);
            return false;
        };
        assert!(vp.contains_rect(band));
        if !self.reveal_smoke_control(ctx, band) {
            return false;
        }
        self.click_smoke_control(ctx, Some(band))
    }
    pub(in crate::ui) fn debug_recovery_tick(&mut self, ctx: &egui::Context) {
        let triage = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("log-triage-batch");
        let lookup = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("path-lookup-batch");
        let managed = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("managed-config-batch");
        let backups = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("content-backup-batch");
        let maps = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("associative-batch");
        let file_lifecycle =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("file-lifecycle-batch");
        let tracing = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("tracing-batch");
        let environment = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("environment-batch");
        if triage
            || backups
            || lookup
            || tracing
            || environment
            || file_lifecycle
            || maps
            || managed
        {
            let root = ctx.cumulative_frame_nr();
            let key = egui::Id::new("tracing-native-root");
            if ctx.data(|d| d.get_temp::<u64>(key)) == Some(root) {
                return;
            }
            ctx.data_mut(|d| d.insert_temp(key, root));
        }
        let key = egui::Id::new("debug-recovery-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if triage {
                    route_for(&self.scenarios, "shell-log-correlation")
                } else if backups {
                    route_for(&self.scenarios, "shell-content-backups")
                } else if lookup {
                    route_for(&self.scenarios, "linux-command-search")
                } else if managed {
                    route_for(&self.scenarios, "shell-managed-configuration")
                } else if maps {
                    route_for(&self.scenarios, "shell-associative-maps")
                } else if file_lifecycle {
                    route_for(&self.scenarios, "shell-file-lifecycle")
                } else if environment {
                    route_for(&self.scenarios, "linux-shell-environment")
                } else if tracing {
                    route_for(&self.scenarios, "shell-tracing-diagnosis")
                } else {
                    route(&self.scenarios)
                },
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "{}: actual examples/change/retry, starter failure, full solution, edit invalidation, Stop, save-read, retry and reset pass",
                if triage {
                    "LOG_TRIAGE_NATIVE_COMPLETE"
                } else if backups {
                    "CONTENT_BACKUP_NATIVE_COMPLETE"
                } else if lookup {
                    "PATH_LOOKUP_NATIVE_COMPLETE"
                } else if managed {
                    "MANAGED_CONFIG_NATIVE_COMPLETE"
                } else if maps {
                    "ASSOCIATIVE_NATIVE_COMPLETE"
                } else if file_lifecycle {
                    "FILE_LIFECYCLE_NATIVE_COMPLETE"
                } else if environment {
                    "ENVIRONMENT_NATIVE_COMPLETE"
                } else if tracing {
                    "TRACING_NATIVE_COMPLETE"
                } else {
                    "DEBUG_RECOVERY_NATIVE_COMPLETE"
                }
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.wheel_until {
            let delay = r.wheel_until - now;
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(delay));
            return;
        }
        self.smoke_stage = 120000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("DEBUG_RECOVERY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 10000,
            "Debug/recovery native step timed out: {op:?}"
        );
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "example" => self
                        .metrics
                        .example_action
                        .or_else(|| self.metrics.lesson_tabs.get(1).copied()),
                    "assignment" => self
                        .metrics
                        .assignment_action
                        .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
                    "run" => self.metrics.check,
                    "stop" => self.metrics.stop,
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    _ => panic!("Unknown fixed project control"),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Type(source) => {
                if r.phase == 0 {
                    if self.debug_recovery_focus(ctx) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&source);
                    r.phase = 2;
                    false
                } else {
                    let i = self.selected.unwrap();
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let actual = if let engine::State::Environment { simulation } = &a.state {
                        if simulation.view == teaching::View::Example {
                            simulation.example.source.as_str()
                        } else {
                            simulation.assignment.source.as_str()
                        }
                    } else if self.script_view(i) == teaching::View::Example {
                        a.workspace
                            .as_ref()
                            .unwrap()
                            .script_learning
                            .as_ref()
                            .unwrap()
                            .example_source
                            .as_str()
                    } else {
                        let engine::State::Script { source, .. } = &a.state else {
                            unreachable!()
                        };
                        source
                    };
                    assert_eq!(actual, source);
                    true
                }
            }
            Op::Result(want) => {
                if self.script_job.is_some() || self.environment_run.is_some() {
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&s.id], s),
                        want,
                        "{} native result",
                        s.id
                    );
                    true
                }
            }
            Op::Example(want) => {
                if self.script_job.is_some() || self.environment_run.is_some() {
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    let a = &self.progress.attempts[&s.id];
                    if let engine::State::Environment { simulation } = &a.state {
                        let checks = dlms_learning_lab::environment_workspace::checks(
                            &simulation.definition.example,
                            &simulation.example,
                        );
                        assert_eq!(checks.iter().all(|r| r.0), want);
                        let engine::State::Environment {
                            simulation: initial,
                        } = &s.initial
                        else {
                            unreachable!()
                        };
                        assert_eq!(simulation.assignment, initial.assignment);
                        assert!(!progress::current_solved(a, s));
                        true
                    } else {
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .script_learning
                            .as_ref()
                            .unwrap();
                        assert_eq!(
                            s.teaching
                                .as_ref()
                                .unwrap()
                                .fixture
                                .accepts(l.example_result.as_ref().unwrap()),
                            want
                        );
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, s));
                        true
                    }
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let p = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&p).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    let target = if name.ends_with("traces-left-on") {
                        self.metrics.output.unwrap()
                    } else {
                        self.metrics
                            .script_feedback
                            .or(self.metrics.output)
                            .unwrap()
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
        if self
            .pending_events
            .iter()
            .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
        {
            // Let egui apply its actual scroll animation before spending another
            // one of the existing20 wheel attempts. Never alter learner timing.
            r.wheel_until = now + 0.15;
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
    fn log_triage_native_route_preserves_example_isolation_real_run_stop_save_and_reset() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "shell-log-correlation");
        let ids: Vec<_> = ops
            .iter()
            .filter_map(|op| {
                if let Op::Open(id) = op {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ids, ["shell-log-owner-triage"]);
        for key in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|op| matches!(op,Op::Click(k)if *k==key))
                    .count(),
                1
            );
        }
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Save)).count(), 1);
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Shot(_))).count(), 3);
        let s = a.scenarios.iter().find(|s| s.id == ids[0]).unwrap();
        let example = &s.teaching.as_ref().unwrap().example;
        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Type(text)if text==example))
        );
        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Type(text)if text==&s.solution[0]))
        );
        let changed = example.replace("owner=\"${owners[$device]}\"", "owner='Unassigned'");
        assert_ne!(&changed, example);
        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Type(text)if text==&changed))
        );
    }
    #[test]
    fn content_backup_native_route_uses_real_examples_edits_run_stop_save_and_reset() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "shell-content-backups");
        let ids: Vec<_> = ops
            .iter()
            .filter_map(|op| {
                if let Op::Open(id) = op {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            ids,
            ["shell-backup-content-set", "shell-restore-validated-set"]
        );
        for key in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|op| matches!(op,Op::Click(k)if *k==key))
                    .count(),
                2
            );
        }
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Save)).count(), 2);
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Shot(_))).count(), 6);
        for s in a
            .scenarios
            .iter()
            .filter(|s| s.unit == "shell-content-backups")
        {
            let example = &s.teaching.as_ref().unwrap().example;
            assert!(
                ops.iter()
                    .any(|op| matches!(op,Op::Type(text)if text==example))
            );
            assert!(
                ops.iter()
                    .any(|op| matches!(op,Op::Type(text)if text==&s.solution[0]))
            );
        }
    }
    #[test]
    fn path_lookup_native_route_requires_provider_policy_partial_alternate_and_stop_cases() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "linux-command-search");
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
        assert_eq!(opens, ["lpic1-env-tool-path"]);
        for label in [
            "path-lookup-practice-old-provider",
            "path-lookup-missing-export-partial",
            "path-lookup-untrusted-suffix",
            "path-lookup-equivalent-order-passed",
        ] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Shot(name)if name==label))
                    .count(),
                1
            );
        }
        for label in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Click(name)if *name==label))
                    .count(),
                1
            );
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Type(s)if s.contains("$LEGACY_DIR")))
        );
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Type(s)if s.contains("$BASE_PATH:$TOOL_DIR")))
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 1);
    }
    #[test]
    fn tracing_native_plan_keeps_real_run_isolation_trace_failure_stop_save_and_reset() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "shell-tracing-diagnosis");
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
        assert_eq!(opens, vec!["shell-trace-delivery", "shell-trace-fallback"]);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Type(s)if s.starts_with("set -x\n")))
                .count(),
            2
        );
        for required in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Click(k)if *k==required))
                    .count(),
                2
            );
        }
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 2);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o,Op::Shot(s)if s.ends_with("traces-left-on")))
                .count(),
            2
        );
    }
    #[test]
    fn tracing_native_capture_uses_actual_output_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "shell-trace-delivery")
            .unwrap();
        a.launch(i);
        a.set_script_view(i, teaching::View::Example);
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        let route = Route {
            ops: vec![Op::Shot("tracing-test-traces-left-on".into())],
            phase: 1,
            ..Default::default()
        };
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("debug-recovery-route"), route));
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 5000.),
                )),
                ..Default::default()
            },
            |ctx| {
                a.render(ctx);
                a.debug_recovery_tick(ctx);
            },
        );
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn environment_native_capture_uses_actual_output_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-env-child-settings")
            .unwrap();
        a.launch(i);
        let engine::State::Environment { simulation } = &mut a
            .progress
            .attempts
            .get_mut("lpic1-env-child-settings")
            .unwrap()
            .state
        else {
            panic!()
        };
        simulation.view = teaching::View::Example;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        let route = Route {
            ops: vec![Op::Shot("environment-test-starter-feedback".into())],
            phase: 1,
            ..Default::default()
        };
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("debug-recovery-route"), route));
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 5000.),
                )),
                ..Default::default()
            },
            |ctx| {
                a.render(ctx);
                a.debug_recovery_tick(ctx);
            },
        );
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn debug_recovery_focus_band_refuses_empty_or_clipped_and_stays_inside_fractional_bounds() {
        for vp in [
            egui::Rect::from_min_max(egui::pos2(30.123, 500.234), egui::pos2(700.456, 500.235)),
            egui::Rect::from_min_max(egui::pos2(30., 600.), egui::pos2(700., 500.)),
        ] {
            assert!(focus_band(vp, 24.).is_none());
        }
        for y in [0.123, 50.125, 799.999, 2048.43] {
            let vp = egui::Rect::from_min_size(egui::pos2(30.234, y), egui::vec2(500.123, 100.789));
            let r = focus_band(vp, 24.).unwrap();
            assert!(vp.contains_rect(r));
            assert!(r.height() >= 23.99);
        }
    }
    #[test]
    fn debug_recovery_capture_requests_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "shell-debug-status")
            .unwrap();
        a.launch(i);
        a.set_script_view(i, teaching::View::Example);
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        assert!(a.metrics.output.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("debug-recovery-route"),
                Route {
                    ops: vec![Op::Shot("debug-recovery-regression".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| a.debug_recovery_tick(ctx));
        assert!(a.capture_pending);
    }
    #[test]
    fn file_lifecycle_route_uses_actual_runs_three_drafts_stop_save_reset_and_changed_examples() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "shell-file-lifecycle");
        let ids: Vec<_> = ops
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
            ids,
            [
                "shell-stage-report",
                "shell-cleanup-preview",
                "shell-recover-publication"
            ]
        );
        for required in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Click(k)if *k==required))
                    .count(),
                3
            );
        }
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 3);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 3);
        for id in ids {
            let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
            let l = s.teaching.as_ref().unwrap();
            let start = ops
                .iter()
                .position(|o| matches!(o,Op::Open(x)if x==id))
                .unwrap();
            let changed = ops[start..]
                .iter()
                .find_map(|o| if let Op::Type(x) = o { Some(x) } else { None })
                .unwrap();
            assert_ne!(changed, &l.example);
            assert!(!l.fixture.accepts(&shell::run(changed, &l.fixture)));
        }
    }
    #[test]
    fn associative_route_uses_actual_runs_two_drafts_stop_save_reset_and_changed_examples() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, "shell-associative-maps");
        let ids: Vec<_> = ops
            .iter()
            .filter_map(|o| {
                if let Op::Open(id) = o {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ids, ["shell-map-lookup", "shell-map-reconcile"]);
        for required in ["example", "assignment", "stop", "reset", "confirm"] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Click(k)if *k==required))
                    .count(),
                2
            );
        }
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 2);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
        for id in ids {
            let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
            let l = s.teaching.as_ref().unwrap();
            let start = ops
                .iter()
                .position(|o| matches!(o,Op::Open(x)if x==id))
                .unwrap();
            let changed = ops[start..]
                .iter()
                .find_map(|o| if let Op::Type(x) = o { Some(x) } else { None })
                .unwrap();
            assert_ne!(changed, &l.example);
            assert!(!l.fixture.accepts(&shell::run(changed, &l.fixture)));
        }
    }
    #[test]
    fn fast_native_frames_reveal_the_actual_long_map_introduction_before_clicking() {
        for dt in [0.001, 0.05] {
            let ctx = egui::Context::default();
            let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
            a.smoke = true;
            let id = "shell-map-reconcile";
            let key = egui::Id::new("debug-recovery-route");
            ctx.data_mut(|d| {
                d.insert_temp(
                    key,
                    Route {
                        ops: vec![Op::Open(id.into()), Op::Click("example")],
                        ..Default::default()
                    },
                )
            });
            let mut time = 0.;
            let mut wheel_times: Vec<f64> = Vec::new();
            let mut saw_offscreen_action = false;
            let mut reached = false;
            for _ in 0..5000 {
                time += dt;
                let events = std::mem::take(&mut a.pending_events);
                if events
                    .iter()
                    .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
                {
                    wheel_times.push(time);
                }
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(2560., 1440.),
                        )),
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                if let (Some(action), Some(viewport)) =
                    (a.metrics.example_action, a.metrics.main_viewport)
                {
                    saw_offscreen_action |= action.bottom() > viewport.bottom();
                }
                a.debug_recovery_tick(&ctx);
                let route = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
                if route.at == 2 {
                    reached = true;
                    break;
                }
            }
            assert!(
                saw_offscreen_action,
                "Reproduce the actual below-fold Introduction action"
            );
            assert!(
                reached,
                "The bounded real scroll/click must reach practice at {dt}s frames"
            );
            assert!(!wheel_times.is_empty());
            assert!(wheel_times.len() <= 20, "Keep the original wheel limit");
            assert!(
                wheel_times
                    .windows(2)
                    .all(|pair| pair[1] - pair[0] >= 0.149),
                "Wheel attempts need elapsed animation time, not frame-count pacing: {wheel_times:?}"
            );
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            assert_eq!(a.script_view(i), teaching::View::Example);
            assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
            assert!(!progress::current_solved(
                &a.progress.attempts[id],
                &a.scenarios[i]
            ));
        }
    }
}
