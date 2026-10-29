//! Executes the actual closed route/capture code with native egui events. Emitted
//! screenshot commands are not rendered Fedora screenshots or novice acceptance.
use super::*;
use crate::ui::smoke::physical_link_lessons as route;

fn flatten_paint<'a>(
    shape: &'a egui::Shape,
    clip: egui::Rect,
    output: &mut Vec<(egui::Rect, &'a egui::Shape)>,
) {
    if let egui::Shape::Vec(children) = shape {
        for child in children {
            flatten_paint(child, clip, output);
        }
    } else {
        output.push((clip, shape));
    }
}
fn assert_header_painted_for_capture(
    ctx: &egui::Context,
    app: &LabApp,
    out: &egui::FullOutput,
    tag: &str,
) {
    let tolerance = 1. / out.pixels_per_point;
    assert!(out.pixels_per_point.is_finite() && out.pixels_per_point > 0.);
    let mut shapes = Vec::new();
    for item in &out.shapes {
        flatten_paint(&item.shape, item.clip_rect, &mut shapes);
    }
    let header = app
        .metrics
        .header
        .iter()
        .copied()
        .chain(app.metrics.settings)
        .reduce(|a, b| a.union(b))
        .expect("Rendered actual header controls");
    let scale = format!("{:.0}%", app.scale * 100.);
    let theme_name = theme::find(&app.theme_id).name.clone();
    for expected in [
        "DLMS Learning Lab",
        "Offline simulation",
        "Text size",
        scale.as_str(),
        theme_name.as_str(),
        "Settings",
    ] {
        let Some((at, bounds)) = shapes.iter().enumerate().find_map(|(at, (clip, shape))| {
            let egui::Shape::Text(text) = shape else {
                return None;
            };
            if text.galley.text() != expected
                || !text.opacity_factor.is_finite()
                || text.opacity_factor <= 0.
            {
                return None;
            }
            let bounds = text.visual_bounding_rect();
            if !bounds.is_positive()
                || !clip.expand(tolerance).contains_rect(bounds)
                || !ctx.content_rect().expand(tolerance).contains_rect(bounds)
                || !header.expand(tolerance).contains_rect(bounds)
            {
                return None;
            }
            let mut glyphs = 0;
            for row in &text.galley.rows {
                let vertices = &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()];
                for vertex in vertices {
                    let color = text.override_text_color.unwrap_or_else(|| {
                        if vertex.color == egui::Color32::PLACEHOLDER {
                            text.fallback_color
                        } else {
                            vertex.color
                        }
                    });
                    if color.a() == 0 {
                        return None;
                    }
                    glyphs += 1;
                }
            }
            (glyphs > 0).then_some((at, bounds))
        }) else {
            panic!(
                "Screenshot {tag} lacks fully clipped-in, nontransparent header text {expected:?}; header{header:?}, pixelsPerPoint{}, shapes{}",
                out.pixels_per_point,
                shapes.len()
            );
        };
        assert!(!shapes[at+1..].iter().any(|(clip,shape)| {
            matches!(shape,egui::Shape::Rect(rect) if rect.fill.a()==255 && clip.intersect(rect.rect).contains_rect(bounds))
        }), "Screenshot {tag}: a later opaque rectangle covers header text {expected:?}");
    }
}
#[test]
fn physical_controller_normal_catalog_108_capture_phases_finish_with_real_controls_and_measured_geometry()
 {
    assert_eq!(
        LabApp::physical_stage(false, true, Some("physical-link-lessons")),
        None
    );
    assert_eq!(
        LabApp::physical_stage(true, false, Some("physical-link-lessons")),
        None
    );
    assert_eq!(
        LabApp::physical_stage(true, true, Some("physical-link-lessons-extra")),
        None
    );
    assert_eq!(
        LabApp::physical_stage(true, true, Some("physical-link-lessons")),
        Some(66000)
    );
    let (ctx, mut a) = app();
    a.smoke = true;
    a.developer = true;
    a.smoke_stage = 66000;
    let original = serde_json::to_vec(&a.scenarios).unwrap();
    let units = serde_json::to_vec(&a.units).unwrap();
    let mut physical_size = egui::vec2(1920., 1080.);
    let mut fullscreen = true;
    let mut shots = std::collections::BTreeSet::new();
    let mut closed = false;
    let mut closing_frames = 0;
    let mut closing_work = None;
    let mut last = 66000;
    let mut waiting = 0;
    let mut injected = false;
    let mut reentry = false;
    let mut geometry_shots = 0;
    let mut painted_header_captures = 0;
    for n in 0..60_000 {
        let facts = route::facts(&ctx);
        let transient = !injected
            && a.scale == 2.
            && facts.is_some_and(|f| f.requested_geometry == Some((2., true)))
            && !fullscreen;
        let size = if transient {
            injected = true;
            egui::vec2(709., physical_size.y)
        } else {
            physical_size
        };
        let events = std::mem::take(&mut a.pending_events);
        assert!(events.len() <= 300);
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
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                size / (a.scale * 1.15),
            )),
            time: Some(n as f64 / 20.),
            events,
            modifiers,
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.native_pixels_per_point = Some(1.15);
        viewport.fullscreen = Some(fullscreen);
        let out = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        });
        if !reentry && route::facts(&ctx).is_some_and(|f| f.completed_configs > 0) {
            let before = serde_json::to_vec(&a.progress).unwrap();
            let receipts = serde_json::to_vec(&a.progress.migration_receipts).unwrap();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        physical_size / (a.scale * 1.15),
                    )),
                    events: vec![
                        egui::Event::PointerMoved(egui::pos2(100., 400.)),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., -100.),
                            modifiers: Default::default(),
                        },
                    ],
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    assert!(a.physical_completed_reentry(ctx));
                },
            );
            assert_eq!(serde_json::to_vec(&a.progress).unwrap(), before);
            assert_eq!(
                serde_json::to_vec(&a.progress.migration_receipts).unwrap(),
                receipts
            );
            reentry = true;
        }
        for output in out.viewport_output.values() {
            for command in &output.commands {
                match command {
                    egui::ViewportCommand::Fullscreen(value) => {
                        fullscreen = *value;
                        if *value {
                            physical_size = egui::vec2(1920., 1080.);
                        }
                    }
                    egui::ViewportCommand::InnerSize(logical) => {
                        if !fullscreen {
                            physical_size = *logical * 1.15;
                        }
                    }
                    egui::ViewportCommand::Screenshot(data) => {
                        let tag = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert_header_painted_for_capture(&ctx, &a, &out, &tag);
                        painted_header_captures += 1;
                        if tag.starts_with("physical-alignment-") {
                            geometry_shots += 1;
                            let f = route::facts(&ctx).unwrap();
                            assert!(f.geometry_same >= 2);
                            assert!(!transient, "Never capture the injected transient geometry");
                            assert!((ctx.zoom_factor() - a.scale).abs() < 0.01);
                        }
                        assert!(shots.insert(tag));
                        assert!(shots.len() <= 128);
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        if a.smoke_stage == last {
            waiting += 1;
        } else {
            last = a.smoke_stage;
            waiting = 0;
        }
        assert!(
            waiting <= 900,
            "Actual controller phase{} did not finish",
            a.smoke_stage
        );
        if closed {
            closing_frames += 1;
            let index = a.completion_index.observed().2;
            let current = (
                serde_json::to_vec(&a.progress).unwrap(),
                serde_json::to_vec(&a.progress.migration_receipts).unwrap(),
                a.physical_presentation.native_counts(),
                (index.contexts, index.key_copies, index.certificates),
                dlms_learning_lab::native_observations::json_admission_calls(),
            );
            if let Some(before) = &closing_work {
                assert_eq!(
                    &current, before,
                    "Deferred close frames retain full progress/receipts and do no new grading, key copying, or JSON admission"
                );
            } else {
                closing_work = Some(current);
            }
            assert_eq!(
                route::facts(&ctx).unwrap().finish_emissions,
                1,
                "A completed route emits PASS exactly once despite delayed OS close"
            );
            if closing_frames == 3 {
                break;
            }
        }
    }
    assert_eq!(
        closing_frames, 3,
        "Render two additional real root frames after first Close command"
    );
    assert!(closed && injected && reentry);
    assert_eq!(shots.len(), 108);
    assert_eq!(
        painted_header_captures, 108,
        "Every emitted Screenshot frame must visibly paint its complete global toolbar"
    );
    assert!(shots.contains("physical-unrelated-sibling-draft-retained"));
    assert_eq!(geometry_shots, 40);
    for id in [
        "ccna-physical-media-span",
        "ccna-physical-duplex-diagnosis",
        "ccna-physical-counter-baseline",
        "ccna-physical-uplink-handover",
    ] {
        for name in [
            "introduction",
            "example-filled-ungraded",
            "example-verified",
            "example-adverse-retry",
            "wrong-specific-feedback",
            "partial-feedback",
            "reference-verified",
            "all-objective-details",
            "pending-edit-invalidates-success",
            "broken-state-invalidates-success",
            "reset-introduction",
            "alternate-one-verified",
            "alternate-two-verified",
            "challenge-full-assignment",
        ] {
            assert!(shots.contains(&format!("{id}-{name}")));
        }
    }
    assert_eq!(serde_json::to_vec(&a.scenarios).unwrap(), original);
    assert_eq!(serde_json::to_vec(&a.units).unwrap(), units);
    assert_eq!(a.scenarios.len(), 176);
    assert_eq!(a.units.len(), 46);
}
