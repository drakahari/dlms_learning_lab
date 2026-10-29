//! Paired real-event smoke/capture-phase gates, prior to any native-window run.
//! Viewport acknowledgements model a window manager, never a learner/model state.
use super::*;
use crate::ui::smoke::file_backup_lessons::{FIRST, IDS, phase_bounds};
#[derive(Default)]
struct Witness {
    frames: usize,
    shots: Vec<String>,
    focused: usize,
    close: usize,
    fullscreen: bool,
    narrow: bool,
    stop_seen: bool,
    elapsed: f64,
    time_step: f64,
    discard_once: bool,
    observed_passes: usize,
}
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (168, 43));
    assert!(a.progress.attempts.is_empty());
    assert_eq!(
        a.scenarios[164..168]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    a.smoke = true;
    a.smoke_stage = FIRST;
    (ctx, a)
}
fn text(shape: &egui::epaint::Shape, wanted: &str) -> bool {
    match shape {
        egui::epaint::Shape::Text(t) => t.galley.job.text.contains(wanted),
        egui::epaint::Shape::Vec(v) => v.iter().any(|s| text(s, wanted)),
        _ => false,
    }
}
fn goal_text<'a>(
    shape: &'a egui::epaint::Shape,
    wanted: &str,
) -> Option<&'a egui::epaint::TextShape> {
    match shape {
        egui::epaint::Shape::Text(t) if t.galley.job.text == wanted => Some(t),
        egui::epaint::Shape::Vec(v) => v.iter().find_map(|s| goal_text(s, wanted)),
        _ => None,
    }
}
fn frame(ctx: &egui::Context, a: &mut LabApp, w: &mut Witness, drive: bool) -> egui::FullOutput {
    w.frames += 1;
    w.elapsed = if w.time_step > 0. {
        w.elapsed + w.time_step
    } else {
        w.frames as f64 / 20.
    };
    let events = std::mem::take(&mut a.pending_events);
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
    let physical = if w.narrow {
        egui::vec2(946., 929.)
    } else {
        egui::vec2(1920., 1080.)
    };
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            physical / (a.scale * 1.15),
        )),
        time: Some(w.elapsed),
        events,
        modifiers,
        ..Default::default()
    };
    let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
    viewport.native_pixels_per_point = Some(1.15);
    viewport.fullscreen = Some(w.fullscreen);
    let mut passes = 0;
    let discard_once = w.discard_once;
    let out = ctx.run(input, |ctx| {
        passes += 1;
        a.render(ctx);
        if discard_once && passes == 1 {
            ctx.request_discard("Bounded backup checker per-frame diagnostic");
        }
        a.save();
        a.started = Instant::now() - std::time::Duration::from_secs(1);
        if drive {
            a.smoke_tick(ctx)
        }
    });
    w.observed_passes = passes;
    for viewport in out.viewport_output.values() {
        for command in &viewport.commands {
            match command {
                egui::ViewportCommand::Fullscreen(b) => {
                    w.fullscreen = *b;
                    if *b {
                        w.narrow = false;
                    }
                }
                egui::ViewportCommand::InnerSize(size) => {
                    assert_eq!(*size, egui::vec2(946., 929.));
                    w.narrow = true;
                }
                egui::ViewportCommand::Close => w.close += 1,
                egui::ViewportCommand::Screenshot(data) => {
                    let name = data
                        .data
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap();
                    assert!(!a.settings_open && !a.reset_pending);
                    assert!(a.selected.is_some());
                    if name.contains("introduction") || name.contains("challenge-full-task") {
                        assert!(
                            out.shapes
                                .iter()
                                .any(|s| text(&s.shape, &a.scenarios[a.selected.unwrap()].title)),
                            "Actual capture paints current task heading"
                        );
                    }
                    if name.contains("backup-capstone-narrow") {
                        let source = &a.scenarios[a.selected.unwrap()];
                        let root = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        let (paint, clip) = out
                            .shapes
                            .iter()
                            .find_map(|s| {
                                goal_text(&s.shape, &source.objective)
                                    .map(|t| (t, s.clip_rect.intersect(root)))
                            })
                            .expect(
                                "Capture must paint the real complete current objective galley",
                            );
                        assert_eq!(paint.angle, 0.);
                        let body = egui::TextStyle::Body.resolve(&ctx.style());
                        assert!(!paint.galley.job.sections.is_empty());
                        for section in &paint.galley.job.sections {
                            assert_eq!(
                                section.format.font_id, body,
                                "Essential goal text stays in normal body font, never shrunk"
                            );
                        }
                        let tolerance = 1. / ctx.pixels_per_point();
                        assert!(!paint.galley.rows.is_empty());
                        for row in &paint.galley.rows {
                            let rect = row.rect().translate(paint.pos.to_vec2());
                            assert!(
                                rect.left() >= clip.left() - tolerance
                                    && rect.right() <= clip.right() + tolerance,
                                "Every actual goal galley row stays inside its horizontal clip: {name}/{rect:?}/{clip:?}"
                            );
                        }
                        let row = if name.contains("goal-end") {
                            paint.galley.rows.last().unwrap()
                        } else {
                            paint.galley.rows.first().unwrap()
                        };
                        let edge = row.rect().translate(paint.pos.to_vec2());
                        assert!(
                            smoke::smoke_control_visible(edge, clip, ctx.pixels_per_point()),
                            "The actual requested first/last goal row must be fully visible, not just a title: {name}/{edge:?}/{clip:?}"
                        );
                    }
                    if name.contains("backup-alignment") || name.contains("backup-capstone-narrow")
                    {
                        let tolerance = 1. / ctx.pixels_per_point();
                        let clip = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        for (key, rect) in &a.metrics.backup.controls {
                            assert!(
                                rect.left() >= clip.left() - tolerance
                                    && rect.right() <= clip.right() + tolerance,
                                "Actual {name} control {key} stays within horizontal clip: {rect:?} vs {clip:?}"
                            );
                        }
                        let (input, _) = a.metrics.backup.terminal.input.unwrap();
                        let own = a
                            .metrics
                            .backup
                            .terminal
                            .input_clip
                            .unwrap()
                            .intersect(clip);
                        assert!(
                            input.left() >= own.left() - tolerance
                                && input.right() <= own.right() + tolerance,
                            "Actual {name} command field stays within its actual clip"
                        );
                        let run = a.metrics.backup.terminal.run.unwrap();
                        assert!(
                            run.left() >= clip.left() - tolerance
                                && run.right() <= clip.right() + tolerance
                        );
                    }
                    if name.contains("wrong-feedback") {
                        assert!(
                            out.shapes.iter().any(|s| text(&s.shape, "Needs work ·")),
                            "The real rendered failing outcome must identify failure in words"
                        );
                    }
                    if name.contains("objectives-complete") {
                        assert!(
                            out.shapes.iter().any(|s| text(&s.shape, "Pass ·")),
                            "The real rendered passing outcome must identify success in words"
                        );
                    }
                    if name.contains("-command") {
                        let (field, id) = a.metrics.backup.terminal.input.unwrap();
                        assert!(ctx.memory(|m| m.has_focus(id)));
                        let root = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        assert!(smoke::smoke_control_visible(
                            field,
                            a.metrics
                                .backup
                                .terminal
                                .input_clip
                                .unwrap()
                                .intersect(root),
                            ctx.pixels_per_point()
                        ));
                        let attempt = &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id];
                        assert!(!attempt.workspace.as_ref().unwrap().command.is_empty());
                        w.focused += 1;
                    }
                    assert!(!w.shots.contains(name));
                    w.shots.push(name.clone());
                    assert!(w.shots.len() <= 128);
                    a.capture_pending = false;
                }
                _ => {}
            }
        }
    }
    if a.metrics.backup.controls.iter().any(|r| r.0 == "stop") && a.backup_presentation.checking() {
        w.stop_seen = true;
    }
    out
}
#[test]
fn backup_controller_constructor_is_closed_and_stage_selection_is_exact() {
    assert_eq!(
        LabApp::file_backup_lessons_stage(true, Some("file-backup-lessons")),
        Some(FIRST)
    );
    for (smoke, name) in [
        (false, Some("file-backup-lessons")),
        (true, None),
        (true, Some("backup")),
        (true, Some("file-backup-lessons-extra")),
    ] {
        assert_eq!(LabApp::file_backup_lessons_stage(smoke, name), None)
    }
    let (first, last) = phase_bounds();
    assert_eq!(first, FIRST);
    assert!((first..first + 1000).contains(&last));
    let (_, a) = opened();
    assert_eq!(
        a.scenarios[164..168]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(pack::bundled().unwrap().len(), 168);
}
#[test]
fn backup_real_controller_and_capture_phases_complete_all_four_without_model_injection() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    for _ in 0..15000 {
        frame(&ctx, &mut a, &mut w, true);
        if w.close > 0 {
            break;
        }
    }
    assert!(
        w.close > 0,
        "Bounded real-event controller did not finish at {}",
        a.smoke_stage
    );
    assert!(w.stop_seen);
    assert_eq!(
        ctx.data(|d| d.get_temp::<bool>(egui::Id::new(("backup-native", "undo-finished")))),
        Some(true),
        "Actual Undo/Redo restoration assertions ran before Finish"
    );
    assert_eq!(a.filter.unit, "backup-and-recovery");
    assert_eq!(a.units[42].labs, IDS);
    assert!(
        w.shots
            .iter()
            .any(|name| name.contains("backup-unit-complete")),
        "All four current assignments must produce actual Unit complete capture"
    );
    assert_eq!(w.focused, 20);
    assert_eq!(
        w.shots.len(),
        81,
        "All original77 captures plus actual capstone first/last goal rows at200% narrow in both themes"
    );
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
        assert!(a.progress.favorites.contains(id));
        let w = a.progress.attempts[id].workspace.as_ref().unwrap();
        let mut practice = dlms_learning_lab::file_backup_teaching::Learning::new(
            s.file_backup_teaching.as_ref().unwrap(),
        )
        .unwrap();
        practice.view = dlms_learning_lab::script_teaching::View::Assignment;
        assert_eq!(
            w.file_backup_learning.as_ref().unwrap(),
            &practice,
            "The actual final selected Reset cleared only that lab's earlier practice, whose runs/adverse/repair/warm isolation were already proved"
        );
    }
    assert_eq!(
        serde_json::to_value(&a.scenarios).unwrap(),
        serde_json::to_value(pack::bundled().unwrap()).unwrap()
    );
    // Finish must remain held while an actual asynchronous Close is pending;
    // no stage increment, duplicate output or second save/readback is fabricated.
    let stage = a.smoke_stage;
    let snapshot = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    let shots = w.shots.len();
    frame(&ctx, &mut a, &mut w, true);
    assert_eq!(a.smoke_stage, stage);
    assert_eq!(w.shots.len(), shots);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), snapshot);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
}
#[test]
fn backup_capture_phase_waits_for_measured_geometry_instead_of_assuming_requested_size() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let capture_stage = FIRST + 4;
    for _ in 0..1000 {
        frame(&ctx, &mut a, &mut w, true);
        if a.smoke_stage == capture_stage {
            break;
        }
    }
    assert_eq!(a.smoke_stage, capture_stage);
    assert!(w.shots.is_empty());
    // Delay the window-manager fullscreen acknowledgement while the real
    // capture phase runs; requested fullscreen must not be treated as observed.
    w.fullscreen = false;
    for _ in 0..2 {
        frame(&ctx, &mut a, &mut w, true);
        assert_eq!(a.smoke_stage, capture_stage);
        assert!(w.shots.is_empty());
    }
    w.fullscreen = true;
    for _ in 0..60 {
        frame(&ctx, &mut a, &mut w, true);
        if !w.shots.is_empty() {
            break;
        }
    }
    assert_eq!(w.shots.len(), 1);
    assert!(w.shots[0].contains("introduction"));
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(!a.metrics.completion);
    let source = &a.scenarios[a.selected.unwrap()];
    assert_eq!(a.progress.attempts[&source.id].state, source.initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[&source.id],
        source
    ));
    assert!(!a.capture_pending);
}

#[test]
fn backup_undo_fast_frames_with_slow_monotonic_time_preserve_actual_history() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let undo = crate::ui::smoke::file_backup_lessons::undo_stage();
    for _ in 0..15000 {
        if a.smoke_stage == undo {
            break;
        }
        frame(&ctx, &mut a, &mut w, true);
    }
    assert_eq!(
        a.smoke_stage, undo,
        "Reach Undo through the same real normal course events"
    );
    assert_eq!(w.shots.len(), 7);
    let frames = w.frames;
    let time = w.elapsed;
    w.time_step = 0.001;
    for _ in 0..5000 {
        frame(&ctx, &mut a, &mut w, true);
        if a.smoke_stage > undo {
            break;
        }
    }
    assert_eq!(
        a.smoke_stage,
        undo + 1,
        "Real stable Undo/Redo must finish without counting intentional1.2s waits as productive retries"
    );
    assert!(
        w.frames - frames > 300,
        "Exercise more than the old300 immediate-render polls"
    );
    assert!(
        w.elapsed - time >= 2.4,
        "Both real native stable-history intervals must actually elapse"
    );
    assert_eq!(
        ctx.data(|d| d.get_temp::<bool>(egui::Id::new(("backup-native", "undo-finished")))),
        Some(true)
    );
    assert_eq!(
        w.shots.len(),
        7,
        "No additional capture or credit is fabricated by the idle wait"
    );
    assert_eq!(w.close, 0);
    assert!(a.pending_events.is_empty());
}

#[test]
fn backup_stop_real_input_cancels_across_incidental_renders_and_navigation() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let stop = crate::ui::smoke::file_backup_lessons::stop_stage();
    for _ in 0..15000 {
        if a.smoke_stage == stop {
            break;
        }
        frame(&ctx, &mut a, &mut w, true);
    }
    assert_eq!(a.smoke_stage, stop);
    assert!(a.backup_presentation.checking());
    let source = &a.scenarios[a.selected.unwrap()];
    let id = source.id.clone();
    assert!(!a.backup_presentation.explicitly_stopped(&id));
    frame(&ctx, &mut a, &mut w, true);
    let progress = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    // Real incidental render frames consume queued input and advance real work,
    // independently of another capture-controller poll. No job/state is installed.
    for _ in 0..24 {
        frame(&ctx, &mut a, &mut w, false);
    }
    println!(
        "After24 incidental real renders: checking={},explicitly_stopped={},stage={}",
        a.backup_presentation.checking(),
        a.backup_presentation.explicitly_stopped(&id),
        a.smoke_stage
    );
    frame(&ctx, &mut a, &mut w, true);
    assert!(
        a.backup_presentation.explicitly_stopped(&id),
        "Actual cancellation, never natural completion"
    );
    assert!(!a.backup_presentation.checking());
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), progress);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    let counters = a.backup_presentation.observed();
    for _ in 0..24 {
        frame(&ctx, &mut a, &mut w, false);
    }
    let after = a.backup_presentation.observed();
    assert_eq!(
        (after.0, after.1, after.2),
        (counters.0, counters.1, counters.2),
        "No idle checker restart/copy/tick after actual Stop"
    );
    for _ in 0..10000 {
        if w.close > 0 {
            break;
        }
        frame(&ctx, &mut a, &mut w, true);
    }
    assert!(w.close > 0);
    assert_eq!(w.shots.len(), 81);
    assert_eq!(
        serde_json::to_value(&a.scenarios).unwrap(),
        serde_json::to_value(pack::bundled().unwrap()).unwrap()
    );
}

#[test]
fn backup_discarded_pass_checks_the_advertised_per_frame_work_bound() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let stop = crate::ui::smoke::file_backup_lessons::stop_stage();
    for _ in 0..15000 {
        if a.smoke_stage == stop {
            break;
        }
        frame(&ctx, &mut a, &mut w, true);
    }
    assert_eq!(a.smoke_stage, stop);
    assert!(a.backup_presentation.checking());
    assert!(
        a.pending_events.is_empty(),
        "No learner command/cancel is replayed in this diagnostic"
    );
    let progress = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    let before = a.backup_presentation.observed().2;
    w.discard_once = true;
    frame(&ctx, &mut a, &mut w, false);
    assert!(
        w.observed_passes >= 2,
        "Use real egui discarded passes, not manually ticked work"
    );
    let delta = a.backup_presentation.observed().2 - before;
    println!(
        "Actual oneContextRun passes={},backupCheckerTicks={delta}",
        w.observed_passes
    );
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), progress);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    assert!(
        delta <= 1,
        "The documented one-archive/row-per-frame bound must survive discarded UI passes; this is a diagnostic, no product behavior has been changed"
    );
}
