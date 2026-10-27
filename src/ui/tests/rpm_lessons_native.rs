//! Actual smoke_tick/events with screenshot acknowledgement; no native acceptance claim.
use super::super::smoke::rpm_lessons::{FIRST, IDS, checkpoints, phase_bounds};
use super::*;
#[derive(Default)]
struct Witness {
    frames: usize,
    shots: Vec<String>,
    focused: Vec<(String, String, bool)>,
    fullscreen: Vec<bool>,
    inspections: usize,
}
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (147, 37));
    assert!(a.progress.attempts.is_empty());
    a.smoke = true;
    a.smoke_stage = FIRST;
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, age: bool, w: &mut Witness) -> egui::FullOutput {
    event_frame(ctx, a, age, w, true)
}
fn event_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    age: bool,
    w: &mut Witness,
    drive: bool,
) -> egui::FullOutput {
    let events = std::mem::take(&mut a.pending_events);
    w.frames += 1;
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| {
            if let egui::Event::Key { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let (width, height) = if a.scale == 1. {
        (1920., 1080.)
    } else {
        (946., 929.)
    };
    let scale = a.scale;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / (scale * 1.15), height / (scale * 1.15)),
        )),
        time: Some(w.frames as f64 / 20.),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(1.15);
    let out = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        if age {
            a.started = Instant::now() - std::time::Duration::from_secs(1);
        }
        if drive {
            a.smoke_tick(ctx);
        }
    });
    let mut shots = 0;
    for v in out.viewport_output.values() {
        for cmd in &v.commands {
            match cmd {
                egui::ViewportCommand::Fullscreen(b) => w.fullscreen.push(*b),
                egui::ViewportCommand::Screenshot(data) => {
                    shots += 1;
                    let label = data
                        .data
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    if label.contains("focused-input") {
                        let i = a.selected.unwrap();
                        let id = &a.scenarios[i].id;
                        let raw = a.progress.attempts[id]
                            .workspace
                            .as_ref()
                            .unwrap()
                            .rpm_session
                            .as_ref()
                            .unwrap()
                            .command
                            .clone();
                        let review: serde_json::Value = serde_json::from_str(include_str!(
                            "../../../tests/fixtures/rpm-packages-review.json"
                        ))
                        .unwrap();
                        let expected = review["lessons"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|r| r["id"] == id.as_str())
                            .unwrap()["read_only"]["commands"][0]
                            .as_str()
                            .unwrap();
                        assert_eq!(raw, expected);
                        let (r, input, enabled) = a.metrics.rpm.command.unwrap();
                        let focus = ctx.memory(|m| m.has_focus(input));
                        assert!(enabled && focus);
                        assert!(super::super::smoke::smoke_control_visible(
                            r,
                            a.metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point()
                        ));
                        w.focused.push((label.clone(), raw, focus));
                    }
                    if label.contains("declared-data-inspection") {
                        assert!(a.metrics.rpm.inspect_selected.as_ref().unwrap().1.is_some());
                        w.inspections += 1;
                    }
                    w.shots.push(label);
                }
                _ => {}
            }
        }
    }
    if a.capture_pending {
        assert!(shots > 0, "Only acknowledge a real Screenshot command");
        a.capture_pending = false;
    }
    out
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, w: &mut Witness) {
    let mut old = a.smoke_stage;
    let mut frames = 0;
    for _ in 0..70000 {
        if a.smoke_stage == target {
            return;
        }
        assert!(a.smoke_stage < target);
        if old == a.smoke_stage {
            frames += 1
        } else {
            old = a.smoke_stage;
            frames = 0
        }
        assert!(frames <= 400, "Actual RPM step{} stalled", a.smoke_stage);
        tick(ctx, a, true, w);
    }
    panic!("Bounded four-task RPM flow")
}
fn mark(a: &LabApp, label: &str) -> usize {
    checkpoints(&a.scenarios)
        .iter()
        .find(|(n, _)| n == label)
        .unwrap()
        .1
}
#[test]
fn closed_normal_rpm_plan_uses_actual_registered_scope_and_bounded_controls() {
    let (ctx, mut a) = opened();
    assert_eq!(LabApp::rpm_lessons_stage(false, Some("rpm-lessons")), None);
    assert_eq!(
        LabApp::rpm_lessons_stage(true, Some("rpm-lessons-extra")),
        None
    );
    assert_eq!(
        LabApp::rpm_lessons_stage(true, Some("rpm-lessons")),
        Some(FIRST)
    );
    let (first, last) = phase_bounds(&a.scenarios);
    assert_eq!(first, 40000);
    assert!(last < 41000 && last > 40000);
    assert!(!a.rpm_lessons_contains_stage(first - 1));
    assert!(a.rpm_lessons_contains_stage(last));
    assert!(!a.rpm_lessons_contains_stage(last + 1));
    let mut w = Witness::default();
    a.started = Instant::now();
    tick(&ctx, &mut a, false, &mut w);
    assert_eq!(a.smoke_stage, FIRST);
    assert!(a.progress.attempts.is_empty());
    let end = mark(&a, "rpm-0-intro") + 1;
    until(&ctx, &mut a, end, &mut w);
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(a.progress.favorites.contains(IDS[0]));
    assert_eq!(a.scenarios[a.selected.unwrap()].id, IDS[0]);
    assert!(w.shots.iter().any(|s| s.contains("intro")));
    assert!(w.fullscreen.contains(&true));
    assert_eq!(
        a.path.parent(),
        Some(a._smoke_profile.as_ref().unwrap().path())
    );
}
#[test]
fn first_actual_rpm_task_covers_insert_run_inspector_failures_alternatives_modal_and_warm_reset() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let end = mark(&a, "select-1");
    until(&ctx, &mut a, end, &mut w);
    let s = a.scenarios.iter().find(|s| s.id == IDS[0]).unwrap();
    assert!(progress::current_solved(&a.progress.attempts[IDS[0]], s));
    assert!(a.selected.is_none());
    assert_eq!(a.progress.attempts.len(), 1);
    for name in [
        "example-worked",
        "example-adverse",
        "example-repaired",
        "assignment",
        "declared-data-inspection",
        "wrong-specific-failure",
        "partial",
        "reference",
        "truthful-unsupported",
        "invalidated",
        "alternate-0",
        "alternate-1",
        "light150-focused-input",
        "dark200-focused-input",
        "resumed-separate-draft",
        "challenge-full-task",
        "after-reset-repaired",
    ] {
        assert!(
            w.shots.iter().any(|s| s.contains(name)),
            "Actual capture missing {name}"
        );
    }
    assert_eq!(w.focused.len(), 2);
    assert_eq!(w.inspections, 1);
    assert!(w.fullscreen.contains(&false));
}
#[test]
fn all_four_actual_rpm_normal_flows_preserve_full_readbacks_and_hold_delayed_finish() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let last = phase_bounds(&a.scenarios).1;
    until(&ctx, &mut a, last, &mut w);
    assert_eq!(a.progress.attempts.len(), 4);
    assert_eq!(w.focused.len(), 8);
    assert_eq!(w.inspections, 4);
    assert!(w.shots.len() <= 128);
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
        assert!(a.progress.favorites.contains(id));
    }
    let before = serde_json::to_value(&a.progress).unwrap();
    for _ in 0..2 {
        let out = tick(&ctx, &mut a, true, &mut w);
        assert_eq!(a.smoke_stage, last);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }));
    }
    assert!(serde_json::to_value(&a.progress).unwrap() == before);
    let raw = progress::read_bounded(&a.path, 2 * 1024 * 1024).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let receipts = value
        .as_object_mut()
        .unwrap()
        .remove("migration_receipts")
        .unwrap();
    assert!(value == before);
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded).unwrap() == before);
    assert_eq!(
        serde_json::to_value(&loaded.migration_receipts).unwrap(),
        receipts
    );
}
#[test]
fn controller_source_has_no_model_or_profile_install_shortcut_and_exact_fixed_inputs() {
    let source = include_str!("../smoke/rpm_lessons.rs");
    assert!(
        !source.contains("engine::execute")
            && !source.contains("self.launch(")
            && !source.contains("scenarios.push")
            && !source.contains("self.progress =")
    );
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let end = mark(&a, "rpm-0-example-worked-first") + 1;
    until(&ctx, &mut a, end, &mut w);
    let attempt = &a.progress.attempts[IDS[0]];
    assert_eq!(attempt.state, a.scenarios[a.selected.unwrap()].initial);
    assert!(!progress::current_solved(
        attempt,
        &a.scenarios[a.selected.unwrap()]
    ));
    assert!(
        attempt
            .workspace
            .as_ref()
            .unwrap()
            .rpm_learning
            .as_ref()
            .unwrap()
            .terminal
            .entries
            .len()
            >= 2
    );
    assert!(
        attempt
            .workspace
            .as_ref()
            .unwrap()
            .rpm_session
            .as_ref()
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn real_silent_and_unsupported_rpm_output_remains_readable_at_every_scale() {
    use dlms_learning_lab::rpm_packages_terminal::Outcome;
    fn contains_text(shape: &egui::Shape, text: &str) -> bool {
        match shape {
            egui::Shape::Text(t) => t.galley.text().contains(text),
            egui::Shape::Vec(v) => v.iter().any(|s| contains_text(s, text)),
            _ => false,
        }
    }
    for scale in [1., 1.5, 2.] {
        for unsupported in [false, true] {
            let (ctx, mut a) = opened();
            let mut w = Witness::default();
            let end = if unsupported {
                mark(&a, "rpm-0-truthful-unsupported")
            } else {
                mark(&a, "rpm-0-example-worked-first") + 1
            };
            until(&ctx, &mut a, end, &mut w);
            let attempt = &a.progress.attempts[IDS[0]];
            let session = if unsupported {
                attempt
                    .workspace
                    .as_ref()
                    .unwrap()
                    .rpm_session
                    .as_ref()
                    .unwrap()
            } else {
                &attempt
                    .workspace
                    .as_ref()
                    .unwrap()
                    .rpm_learning
                    .as_ref()
                    .unwrap()
                    .terminal
            };
            let (entry_kind, text) = match &session.entries.last().unwrap().output {
                Outcome::Utility { stdout, status, .. } => {
                    assert!(
                        !unsupported && stdout.is_empty() && *status == 0,
                        "Actual install remains genuinely silent"
                    );
                    ("status", "RPM command success · status 0".to_owned())
                }
                Outcome::SimulatorUnsupported { reason } => {
                    assert!(unsupported && !reason.is_empty());
                    ("unsupported", reason.clone())
                }
            };
            let row_key = format!("{}:{entry_kind}", session.entries.len() - 1);
            let before = serde_json::to_value(&a.progress).unwrap();
            a.scale = scale;
            let mut shown = false;
            for _ in 0..300 {
                // Pause only the controller in this focused test. Render/save
                // and real wheel events still use native update ordering.
                let out = event_frame(&ctx, &mut a, false, &mut w, false);
                let (done, selected, target, clip) =
                    super::super::smoke::rpm_lessons::output_probe(&mut a, &ctx);
                assert_eq!(
                    selected, row_key,
                    "Silent success selects existing status; refusal selects its real reason"
                );
                let child = a.metrics.rpm.output.unwrap();
                assert!(
                    child.width() > 0. && child.height() > 0.,
                    "Actual output child must have positive area"
                );
                if done {
                    assert!(super::super::smoke::smoke_control_visible(
                        target,
                        clip,
                        ctx.pixels_per_point()
                    ));
                    assert!(
                        out.shapes.iter().any(|s| contains_text(&s.shape, &text)),
                        "The real status/reason is painted at scale{scale}, not a synthesized stdout/header fallback"
                    );
                    shown = true;
                    break;
                }
            }
            assert!(
                shown,
                "Real output not accessible within original bounded reveal at scale{scale}"
            );
            assert!(
                serde_json::to_value(&a.progress).unwrap() == before,
                "Reading/revealing output cannot change State/session/progress"
            );
        }
    }
}
