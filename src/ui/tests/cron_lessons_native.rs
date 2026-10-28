//! Paired real-event smoke/capture-phase gates, prior to any native-window run.
//! Viewport acknowledgements model a window manager, never a learner/model state.
use super::*;
use crate::ui::smoke::cron_lessons::{FIRST, IDS, phase_bounds};
#[derive(Default)]
struct Witness {
    frames: usize,
    shots: Vec<String>,
    focused: usize,
    close: usize,
    fullscreen: bool,
    narrow: bool,
    stop_seen: bool,
}
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (164, 42));
    assert!(a.progress.attempts.is_empty());
    assert_eq!(
        a.scenarios[160..164]
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
fn frame(ctx: &egui::Context, a: &mut LabApp, w: &mut Witness, drive: bool) -> egui::FullOutput {
    w.frames += 1;
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
        time: Some(w.frames as f64 / 20.),
        events,
        modifiers,
        ..Default::default()
    };
    let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
    viewport.native_pixels_per_point = Some(1.15);
    viewport.fullscreen = Some(w.fullscreen);
    let out = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        a.started = Instant::now() - std::time::Duration::from_secs(1);
        if drive {
            a.smoke_tick(ctx)
        }
    });
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
                    if name.contains("cron-alignment") {
                        let clip = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        for (key, rect, _, own_clip) in &a.metrics.cron.controls {
                            let horizontal = clip.intersect(*own_clip);
                            assert!(
                                rect.left() >= horizontal.left() - 0.5
                                    && rect.right() <= horizontal.right() + 0.5,
                                "Actual {name} control {key} must stay within horizontal clip: {rect:?} vs {horizontal:?}"
                            );
                        }
                        for (key, rect, _, _, own_clip) in &a.metrics.cron.fields {
                            let horizontal = clip.intersect(*own_clip);
                            assert!(
                                rect.left() >= horizontal.left() - 0.5
                                    && rect.right() <= horizontal.right() + 0.5,
                                "Actual {name} field {key} must stay within horizontal clip: {rect:?} vs {horizontal:?}"
                            );
                        }
                    }
                    if name.contains("wrong-feedback") {
                        assert!(
                            out.shapes.iter().any(|s| text(&s.shape, "Needs work ·")),
                            "The real rendered failing outcome must identify failure in words"
                        );
                    }
                    if name.contains("objectives-first") {
                        assert!(
                            out.shapes.iter().any(|s| text(&s.shape, "Pass ·")),
                            "The real rendered passing outcome must identify success in words"
                        );
                    }
                    if name.contains("editor") {
                        let f = a
                            .metrics
                            .cron
                            .fields
                            .iter()
                            .find(|f| f.0 == "document")
                            .unwrap();
                        assert!(f.3 && ctx.memory(|m| m.has_focus(f.2)));
                        let root = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        assert!(smoke::smoke_control_visible(
                            f.1,
                            f.4.intersect(root),
                            ctx.pixels_per_point()
                        ));
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
    if a.metrics.cron.controls.iter().any(|r| r.0 == "stop" && r.2) && a.cron_presentation.running()
    {
        w.stop_seen = true;
    }
    out
}
#[test]
fn cron_controller_constructor_is_closed_and_stage_selection_is_exact() {
    assert_eq!(
        LabApp::cron_lessons_stage(true, Some("cron-lessons")),
        Some(FIRST)
    );
    for (smoke, name) in [
        (false, Some("cron-lessons")),
        (true, None),
        (true, Some("cron")),
        (true, Some("cron-lessons-extra")),
    ] {
        assert_eq!(LabApp::cron_lessons_stage(smoke, name), None)
    }
    let (first, last) = phase_bounds();
    assert_eq!(first, FIRST);
    assert!((first..first + 1000).contains(&last));
    let (_, a) = opened();
    assert_eq!(
        a.scenarios[160..164]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(pack::bundled().unwrap().len(), 164);
}
#[test]
fn cron_real_controller_and_capture_phases_complete_all_four_without_model_injection() {
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
    assert_eq!(a.filter.unit, "linux-cron-foundations");
    assert_eq!(a.units[41].labs, IDS);
    assert!(
        w.shots
            .iter()
            .any(|name| name.contains("cron-unit-complete")),
        "All four current assignments must produce actual Unit complete capture"
    );
    assert_eq!(w.focused, 20);
    assert_eq!(
        w.shots.len(),
        77,
        "All original76 capture phases plus actual unit completion"
    );
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
        assert!(a.progress.favorites.contains(id));
        let w = a.progress.attempts[id].workspace.as_ref().unwrap();
        assert!(w.cron_session.as_ref().unwrap().last_run.is_some());
        let mut reset_practice =
            dlms_learning_lab::cron_teaching::Learning::new(s.cron_teaching.as_ref().unwrap())
                .unwrap();
        reset_practice.view = dlms_learning_lab::script_teaching::View::Assignment;
        assert_eq!(
            w.cron_learning.as_ref().unwrap(),
            &reset_practice,
            "The final actual selected-attempt Reset cleared only this lab's example work; its earlier actual Run/history and warm readback were proved before that reset"
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
fn cron_capture_phase_waits_for_measured_geometry_instead_of_assuming_requested_size() {
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
