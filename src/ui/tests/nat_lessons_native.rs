//! Actual smoke_tick regressions with fake screenshot delivery, never native-window acceptance.
use super::super::smoke::nat_lessons::{FIRST, IDS, LAST, STRIDE};
use super::*;
#[derive(Default)]
struct Witness {
    shots: Vec<String>,
    inner_wheel: bool,
    fullscreen: Vec<bool>,
    focused: Vec<(String, String, bool)>,
    max_phase_frames: usize,
    measured_window: Option<(f32, f32, f32)>,
    tall_child_focused_intersection: bool,
}
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (154, 39));
    assert!(a.progress.attempts.is_empty());
    a.smoke = true;
    a.smoke_stage = FIRST;
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, age: bool, w: &mut Witness) -> egui::FullOutput {
    let events = std::mem::take(&mut a.pending_events);
    if events
        .iter()
        .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        && let Some(point) = events.iter().find_map(|e| {
            if let egui::Event::PointerMoved(p) = e {
                Some(*p)
            } else {
                None
            }
        })
    {
        w.inner_wheel |= a
            .metrics
            .nat_static_viewport
            .is_some_and(|r| r.contains(point));
    }
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
    let width = if a.scale == 1.5 {
        1400.
    } else if a.scale == 2. {
        946.
    } else {
        1200.
    };
    let (width, height, density) = w.measured_window.unwrap_or((width, 950., 1.));
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / (a.scale * density), height / (a.scale * density)),
        )),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(density);
    let out = ctx.run(input, |ctx| {
        a.render(ctx);
        if a.scale == 2.
            && density == 1.15
            && let (Some(child), Some(root)) =
                (a.metrics.nat_static_viewport, a.metrics.main_viewport)
        {
            let root = root.intersect(ctx.content_rect());
            let focus = ctx.memory(|m| m.focused());
            w.tall_child_focused_intersection |= child.height() > root.height()
                && a.metrics.nat_fields.iter().any(|r| {
                    r.0.starts_with("static:")
                        && Some(r.2) == focus
                        && super::super::smoke::smoke_control_visible(
                            r.1,
                            r.4.intersect(child).intersect(root),
                            ctx.pixels_per_point(),
                        )
                });
        }
        a.save();
        if age {
            a.started = Instant::now() - std::time::Duration::from_secs(1);
        }
        a.smoke_tick(ctx);
    });
    let mut screenshots = 0;
    for viewport in out.viewport_output.values() {
        for command in &viewport.commands {
            if let egui::ViewportCommand::Fullscreen(value) = command {
                w.fullscreen.push(*value);
            }
            if let egui::ViewportCommand::Screenshot(data) = command {
                screenshots += 1;
                let label = data
                    .data
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<String>()
                    .unwrap()
                    .clone();
                if label.contains("focused-input") {
                    let id = IDS.iter().find(|id| label.contains(**id)).unwrap();
                    let d = a.progress.attempts[*id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .nat_draft
                        .as_ref()
                        .unwrap();
                    let dark = label.contains("dark200");
                    let (raw, key) = if *id == IDS[0] {
                        let row = d.statics.iter().position(|r| r.inside == "server").unwrap();
                        let key = a
                            .metrics
                            .nat_fields
                            .iter()
                            .filter(|r| r.0.starts_with("static:") && r.0.ends_with(":global"))
                            .nth(row)
                            .unwrap()
                            .0
                            .as_str();
                        (d.statics[row].global.clone(), key)
                    } else {
                        (d.prefix.clone(), "prefix")
                    };
                    let expected = if *id == IDS[0] {
                        if dark {
                            " 198.51.100.40"
                        } else {
                            "198.51.100.40 "
                        }
                    } else if dark {
                        " 30"
                    } else {
                        "030"
                    };
                    assert_eq!(
                        raw, expected,
                        "Exact new text, not merely generic goal success"
                    );
                    let field = a.metrics.nat_fields.iter().find(|r| r.0 == key).unwrap();
                    let focused = ctx.memory(|m| m.has_focus(field.2));
                    assert!(focused);
                    assert!(super::super::smoke::smoke_control_visible(
                        field.1,
                        field
                            .4
                            .intersect(a.metrics.main_viewport.unwrap())
                            .intersect(ctx.content_rect()),
                        ctx.pixels_per_point()
                    ));
                    w.focused.push((label.clone(), raw, focused));
                }
                if label.contains("all-rows") {
                    let first = a.metrics.nat_result_rows.first().unwrap();
                    let last = a.metrics.nat_result_rows.last().unwrap();
                    let root = a
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    assert!(super::super::smoke::smoke_control_visible(
                        first.2,
                        first.3.intersect(root),
                        ctx.pixels_per_point()
                    ));
                    assert!(super::super::smoke::smoke_control_visible(
                        last.2,
                        last.3.intersect(root),
                        ctx.pixels_per_point()
                    ));
                }
                w.shots.push(label);
            }
        }
    }
    if a.capture_pending {
        assert!(
            screenshots > 0,
            "Only acknowledge an actual Screenshot command"
        );
        a.capture_pending = false;
    }
    out
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, w: &mut Witness) {
    let mut old = a.smoke_stage;
    let mut frames = 0;
    for _ in 0..36000 {
        if a.smoke_stage == target {
            return;
        }
        assert!((FIRST..=LAST).contains(&a.smoke_stage));
        if old == a.smoke_stage {
            frames += 1;
        } else {
            old = a.smoke_stage;
            frames = 0;
        }
        w.max_phase_frames = w.max_phase_frames.max(frames);
        assert!(
            frames <= 1200,
            "Actual bounded smoke_tick phase{} stalled",
            a.smoke_stage
        );
        tick(ctx, a, true, w);
    }
    panic!("Three lesson total frame bound")
}
#[test]
fn closed_normal_selector_and_exact_actual_opening_obey_pacing_and_private_start() {
    const {
        assert!(FIRST == 39000 && LAST == 39215 && STRIDE == 72 && FIRST > 38071 && LAST < 40000);
    }
    assert_eq!(LabApp::nat_lessons_stage(false, Some("nat-lessons")), None);
    assert_eq!(LabApp::nat_lessons_stage(true, None), None);
    assert_eq!(LabApp::nat_lessons_stage(true, Some("nat-workspace")), None);
    assert_eq!(
        LabApp::nat_lessons_stage(true, Some("nat-lessons")),
        Some(FIRST)
    );
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    a.started = Instant::now();
    tick(&ctx, &mut a, false, &mut w);
    assert_eq!(a.smoke_stage, FIRST);
    assert!(a.progress.attempts.is_empty());
    until(&ctx, &mut a, FIRST + 4, &mut w);
    assert_eq!(a.scenarios.len(), 154);
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(a.progress.favorites.contains(IDS[0]));
    assert_eq!(
        a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .nat_learning
            .as_ref()
            .unwrap()
            .view,
        teaching::View::Example
    );
    assert!(w.shots.iter().any(|s| s.contains("intro")));
    assert!(w.fullscreen.contains(&true));
    assert_eq!(
        a.path.parent(),
        Some(a._smoke_profile.as_ref().unwrap().path())
    );
}
#[test]
fn first_real_normal_flow_captures_examples_failures_rows_typed_scales_and_exact_warm_reset() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + STRIDE, &mut w);
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap()]
    ));
    assert!(a.selected.is_none());
    for piece in [
        "worked-example",
        "example-adverse",
        "example-repaired",
        "full-assignment",
        "wrong-feedback",
        "partial-feedback",
        "reference-complete",
        "actual-reverse",
        "success-invalidated",
        "alternate-policy-one",
        "alternate-policy-two",
        "warm-resumed-pending",
        "challenge-full",
        "light150-focused-input",
        "dark200-focused-input",
        "selected-reset-intro",
        "unit-library",
    ] {
        assert!(
            w.shots.iter().any(|s| s.contains(piece)),
            "Missing actual capture phase {piece}"
        );
    }
    assert_eq!(w.focused.len(), 2);
    assert!(w.inner_wheel);
    assert!(w.fullscreen.contains(&false));
    assert!(w.shots.len() <= 38);
}
#[test]
fn all_three_actual_normal_flows_preserve_prior_work_finish_holds_and_bounded_readback() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    until(&ctx, &mut a, LAST, &mut w);
    assert_eq!(a.scenarios.len(), 154);
    assert_eq!(a.units.len(), 39);
    assert_eq!(a.progress.attempts.len(), 3);
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
        assert!(a.progress.favorites.contains(id));
        assert!(w.shots.iter().any(|s| s.contains(id)));
    }
    assert_eq!(w.focused.len(), 6);
    assert!(w.shots.len() <= 114);
    assert!(w.max_phase_frames <= 1200);
    let prior = serde_json::to_value(&a.progress).unwrap();
    for _ in 0..2 {
        let out = tick(&ctx, &mut a, true, &mut w);
        assert_eq!(a.smoke_stage, LAST);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }));
    }
    assert!(serde_json::to_value(&a.progress).unwrap() == prior);
    let bytes = progress::read_bounded(&a.path, 512 * 1024).unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let receipts = raw
        .as_object_mut()
        .unwrap()
        .remove("migration_receipts")
        .unwrap();
    assert!(raw == serde_json::to_value(&a.progress).unwrap());
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded).unwrap() == raw);
    assert_eq!(
        serde_json::to_value(&loaded.migration_receipts).unwrap(),
        receipts
    );
}
#[test]
fn actual_opened_normal_practice_keeps_modal_background_input_and_check_inert() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    let target = FIRST + 4;
    until(&ctx, &mut a, target, &mut w);
    let id = IDS[0];
    let before = serde_json::to_value(&a.progress).unwrap();
    a.settings_open = true;
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Text("hidden edit".into()),
            acl_ctrl(egui::Key::Enter),
        ],
    );
    assert!(a.metrics.nat_fields.iter().all(|r| !r.3));
    assert!(a.metrics.nat_controls.iter().all(|r| !r.2));
    assert!(
        serde_json::to_value(&a.progress).unwrap() == before,
        "Real modal input cannot change assignment/practice/current proof"
    );
    assert_eq!(a.smoke_stage, target);
    assert_eq!(
        a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .nat_learning
            .as_ref()
            .unwrap()
            .view,
        teaching::View::Example
    );
}

#[test]
fn actual_tiled_fractional_density_at_150_reaches_focused_text_without_child_field_stability_loop()
{
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + 55, &mut w);
    // Actual failed Fedora geometry:946x929 pixels, native1.15, text150%,
    // giving PPP1.725. Metrics come only from real rendered widgets.
    w.measured_window = Some((946., 929., 1.15));
    until(&ctx, &mut a, FIRST + 58, &mut w);
    assert_eq!(a.scale, 1.5);
    assert!((ctx.pixels_per_point() - 1.725).abs() < 0.0001);
    assert!(w.focused.iter().any(
        |(label, raw, focused)| label.contains("light150-focused-input")
            && raw == "198.51.100.40 "
            && *focused
    ));
    assert_eq!(
        a.smoke_stage,
        FIRST + 58,
        "Actual bounded queue must complete typing/capture, not alternate child and field reveal targets until300ticks"
    );
}

#[test]
fn actual_tiled_fractional_density_at_200_types_visible_field_in_child_taller_than_root() {
    let (ctx, mut a) = opened();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + 55, &mut w);
    w.measured_window = Some((946., 929., 1.15));
    until(&ctx, &mut a, FIRST + 63, &mut w);
    assert_eq!(a.scale, 2.);
    assert!((ctx.pixels_per_point() - 2.3).abs() < 0.0001);
    assert!(
        w.tall_child_focused_intersection,
        "Exercise actual taller-child geometry with the complete focused field inside child/root intersection; never require impossible full-child containment"
    );
    assert!(w.focused.iter().any(
        |(label, raw, focused)| label.contains("dark200-focused-input")
            && raw == " 198.51.100.40"
            && *focused
    ));
    assert_eq!(
        a.smoke_stage,
        FIRST + 63,
        "Actual dark200 queue and capture complete under original20wheel/300tick bounds"
    );
}

#[test]
fn measured_fractional_child_edge_does_not_alternate_actual_reveal_stability_targets() {
    use super::super::smoke::nat::{Queue, static_root_reveal_piece};
    use super::super::smoke::smoke_control_visible;
    let (ctx, mut a) = opened();
    a.scale = 1.5;
    for _ in 0..3 {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946. / 1.725, 929. / 1.725),
            )),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let _ = ctx.run(input, |ctx| a.render(ctx));
    }
    let ppp = ctx.pixels_per_point();
    assert!((ppp - 1.725).abs() < 0.0001);
    // Exact rounded rectangles from retained native R7 last3 trace. These are
    // direct arguments to visibility/stability, never installed widget metrics.
    let root = egui::Rect::from_min_max(egui::pos2(18., 115.8), egui::pos2(516.4, 520.6));
    let child = egui::Rect::from_min_max(egui::pos2(18., 246.8), egui::pos2(502.7, 520.8));
    let target = egui::Rect::from_min_max(egui::pos2(195., 471.9), egui::pos2(502.6, 506.9));
    let visible = child.intersect(root);
    assert!(
        !root.contains_rect(child),
        "The old strict guard demanded outer reveal"
    );
    assert!(smoke_control_visible(child, root, ppp));
    assert!(smoke_control_visible(target, visible, ppp));
    let mut old = Queue::default();
    for _ in 0..3 {
        assert!(!a.nat_reveal(&ctx, child, root, false, &mut old));
        assert!(!a.nat_reveal(&ctx, target, visible, true, &mut old));
    }
    assert!(
        static_root_reveal_piece(target, visible, child, root, ppp).is_none(),
        "Actual routing skips the redundant child target"
    );
    let mut corrected = Queue::default();
    assert!(!a.nat_reveal(&ctx, target, visible, true, &mut corrected));
    assert!(
        a.nat_reveal(&ctx, target, visible, true, &mut corrected),
        "The same actual stability helper now settles instead of alternating"
    );
    let shift = egui::vec2(0., root.height() * 2.);
    let off_child = child.translate(shift);
    let off_target = target.translate(shift);
    let off_visible = off_child.intersect(root);
    assert!(!smoke_control_visible(off_target, off_visible, ppp));
    let piece = static_root_reveal_piece(off_target, off_visible, off_child, root, ppp)
        .expect("Wholly offscreen child still requires root-first reveal");
    assert!(piece.height() <= root.height() && piece.width() <= root.width());
    assert!(
        a.pending_events.is_empty(),
        "Visible-edge settling must not queue fake clicks or wheel events"
    );
}
