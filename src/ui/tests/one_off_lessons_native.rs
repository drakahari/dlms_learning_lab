//! Executes the real normal-catalog route with native egui events. These are
//! headless capture-phase/paint assertions, not Fedora PNGs or novice acceptance.
use super::*;
use crate::ui::smoke::one_off_lessons as route;
use std::collections::BTreeSet;
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

fn assert_goal_edge(ctx: &egui::Context, app: &LabApp, out: &egui::FullOutput, tag: &str) {
    let source = &app.scenarios[app.selected.unwrap()];
    let root = app
        .metrics
        .main_viewport
        .unwrap()
        .intersect(ctx.content_rect());
    let mut flat = Vec::new();
    for shape in &out.shapes {
        flatten_paint(&shape.shape, shape.clip_rect, &mut flat);
    }
    let (paint, clip) = flat
        .into_iter()
        .find_map(|(clip, shape)| match shape {
            egui::Shape::Text(t) if t.galley.text() == source.objective => {
                Some((t, clip.intersect(root)))
            }
            _ => None,
        })
        .expect("Capture paints the actual complete assignment galley");
    assert_eq!(paint.angle, 0.);
    assert!(paint.opacity_factor > 0.);
    let body = egui::TextStyle::Body.resolve(&ctx.style());
    assert!(!paint.galley.job.sections.is_empty());
    for section in &paint.galley.job.sections {
        assert_eq!(
            section.format.font_id, body,
            "Normal readable goal font is retained"
        );
    }
    let tolerance = 1. / ctx.pixels_per_point();
    for row in &paint.galley.rows {
        let r = row.rect().translate(paint.pos.to_vec2());
        assert!(
            r.left() >= clip.left() - tolerance && r.right() <= clip.right() + tolerance,
            "All actual goal rows fit their horizontal clip {tag}/{r:?}/{clip:?}"
        );
    }
    let row = if tag.ends_with("-last") {
        paint.galley.rows.last()
    } else {
        paint.galley.rows.first()
    }
    .unwrap();
    let edge = row.rect().translate(paint.pos.to_vec2());
    assert!(
        smoke::smoke_control_visible(edge, clip, ctx.pixels_per_point()),
        "Actual first/last assignment row is fully visible {tag}/{edge:?}/{clip:?}"
    );
}
fn captures() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for n in 0..4 {
        for tag in [
            "introduction",
            "example-verified-ungraded",
            "example-unsaved-feedback",
            "full-assignment",
            "wrong-specific-feedback",
            "reference-verified",
            "challenge-task",
        ] {
            assert!(names.insert(format!("one-off-{n}-{tag}")));
        }
    }
    names.insert("one-off-1-captured-variables".into());
    for scale in [1f32, 1.15, 1.25, 1.5, 2.] {
        for dark in [true, false] {
            for narrow in [true, false] {
                names.insert(format!(
                    "one-off-0-matrix-{scale:.2}-{}-{}",
                    if dark { "dark" } else { "light" },
                    if narrow { "narrow" } else { "wide" }
                ));
            }
        }
    }
    for theme in ["dark", "light"] {
        for edge in ["first", "last"] {
            names.insert(format!("one-off-3-goal-{theme}-{edge}"));
        }
    }
    names
}
#[test]
fn one_off_normal_route_is_closed_and_only_selected_by_its_exact_smoke_option() {
    assert_eq!(
        LabApp::one_off_lessons_stage(false, Some("one-off-lessons")),
        None
    );
    assert_eq!(LabApp::one_off_lessons_stage(true, None), None);
    assert_eq!(
        LabApp::one_off_lessons_stage(true, Some("one-off-lessons-extra")),
        None
    );
    assert_eq!(
        LabApp::one_off_lessons_stage(true, Some("one-off-lessons")),
        Some(73000)
    );
    let (first, last) = route::phase_bounds();
    assert_eq!(first, 73000);
    assert!(last < 74000);
    assert_eq!(
        last - first + 1,
        906,
        "Independent authored-action plan inventory"
    );
    let actual = route::planned_captures();
    assert_eq!(actual.len(), 53);
    assert_eq!(actual.iter().cloned().collect::<BTreeSet<_>>(), captures());
    assert_eq!(
        (first..=last).filter(|s| route::immediate_stop(*s)).count(),
        3
    );
    assert!(!route::immediate_stop(72999) && !route::immediate_stop(74000));
}
#[test]
fn one_off_normal_four_tasks_real_events_finish_all_capture_phases_and_preserve_work() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 73000;
    assert!(!a.developer && !a.synthetic);
    assert_eq!(
        (
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&a.scenarios)
                .len(),
            one_off_boundary::loop_registration::lvm_registration::before_typed_units(&a.units)
                .len()
        ),
        (182, 48)
    );
    assert_eq!((one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&a.scenarios).len(),one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_typed_units(&a.units).len()),(186,49));
    assert_eq!((a.scenarios.len(), a.units.len()), (190, 50));
    let catalog = serde_json::to_vec(&a.scenarios).unwrap();
    let units = serde_json::to_vec(&a.units).unwrap();
    let mut physical = egui::vec2(1920., 1080.);
    let mut fullscreen = true;
    let mut time = 0.;
    let mut shots = BTreeSet::new();
    let mut closed = 0;
    let mut closing_work = None;
    let mut previous_stage = a.smoke_stage;
    let mut stage_frames = 0;
    let mut slow_wait_frames = 0;
    let mut stop_interstitial_proved = false;
    let mut stop_run_observed = false;
    let mut painted = 0;
    for _ in 0..60_000 {
        let before = route::observations(&ctx);
        let waiting = before.is_some_and(|v| v.2.is_some_and(|end| time < end));
        if waiting && route::undo_waiting(&ctx) {
            time += 0.001;
            slow_wait_frames += 1;
        } else {
            time += 0.05;
        }
        let events = std::mem::take(&mut a.pending_events);
        assert!(events.len() <= 300);
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| match e {
                egui::Event::Key { modifiers, .. } => Some(*modifiers),
                _ => None,
            })
            .unwrap_or_default();
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                physical / (a.scale * 1.15),
            )),
            time: Some(time),
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
        if waiting {
            assert!(
                route::observations(&ctx).unwrap().1 < 300,
                "Natural wheel settlement and Undo checkpoint waits do not consume productive poll budget"
            );
        }
        for output in out.viewport_output.values() {
            for command in &output.commands {
                match command {
                    egui::ViewportCommand::Fullscreen(value) => {
                        fullscreen = *value;
                        if *value {
                            physical = egui::vec2(1920., 1080.);
                        }
                    }
                    egui::ViewportCommand::InnerSize(size) => {
                        if !fullscreen {
                            physical = *size * 1.15;
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
                        if tag.starts_with("one-off-3-goal-") {
                            assert_goal_edge(&ctx, &a, &out, &tag);
                        }
                        painted += 1;
                        assert!(
                            captures().contains(&tag),
                            "Only independently inventoried captures"
                        );
                        assert!(shots.insert(tag));
                        assert!(shots.len() <= 128);
                        // Acknowledge only screenshot-delivery plumbing. No model,
                        // focus, cursor, scroll, grade or native pixel is installed.
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed += 1,
                    _ => {}
                }
            }
        }
        if route::immediate_stop(a.smoke_stage) && a.one_off_presentation.running() {
            stop_run_observed = true;
            let source = &a.scenarios[a.selected.unwrap()];
            let attempt = &a.progress.attempts[&source.id];
            let engine::State::OneOff { simulation } = &attempt.state else {
                panic!("Actual OneOff Run");
            };
            assert!(
                simulation.last_run.is_none(),
                "Queue edits cleared old proof; Stop cannot borrow solved credit"
            );
            let probes = simulation
                .simulation
                .jobs
                .values()
                .filter(|j| j.owner == "tess" && j.script_path == "/work/handover-job.sh")
                .collect::<Vec<_>>();
            assert_eq!(
                probes.len(),
                4,
                "Four actual queued jobs sustain a real bounded Run"
            );
            for job in probes {
                assert_eq!(job.source.lines().count(), 32);
                assert!(
                    job.source
                        .starts_with("printf 'staged Stop probe\\n' > /work/handover-stage\n")
                );
            }
        }
        if !stop_interstitial_proved && a.selected.is_some() {
            let source = &a.scenarios[a.selected.unwrap()];
            let attempt = &a.progress.attempts[&source.id];
            if a.one_off_presentation.memo.paused(attempt, source)
                && route::observations(&ctx).is_some_and(|q| q.3 == 3)
            {
                let before = serde_json::to_vec(&a.progress).unwrap();
                let receipts = serde_json::to_vec(&a.progress.migration_receipts).unwrap();
                let steps = a.one_off_presentation.memo.counters.steps;
                for _ in 0..24 {
                    time += 0.01;
                    let mut input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            physical / (a.scale * 1.15),
                        )),
                        time: Some(time),
                        ..Default::default()
                    };
                    let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
                    viewport.native_pixels_per_point = Some(1.15);
                    viewport.fullscreen = Some(fullscreen);
                    let _ = ctx.run(input, |ctx| {
                        a.render(ctx);
                        a.save();
                    });
                    assert!(
                        !a.one_off_presentation.running(),
                        "Incidental frames cannot restart the stopped Run"
                    );
                    assert_eq!(serde_json::to_vec(&a.progress).unwrap(), before);
                    assert_eq!(
                        serde_json::to_vec(&a.progress.migration_receipts).unwrap(),
                        receipts
                    );
                    assert_eq!(
                        a.one_off_presentation.memo.counters.steps, steps,
                        "Actual Stop survives incidental renders without natural completion or replay"
                    );
                    let s = &a.scenarios[a.selected.unwrap()];
                    assert!(
                        a.one_off_presentation
                            .memo
                            .paused(&a.progress.attempts[&s.id], s)
                    );
                }
                stop_interstitial_proved = true;
            }
        }
        if a.smoke_stage == previous_stage {
            stage_frames += 1;
        } else {
            previous_stage = a.smoke_stage;
            stage_frames = 0;
        }
        assert!(
            stage_frames < 3000,
            "Finite route deadline/stage {}",
            a.smoke_stage
        );
        if closed > 0 {
            assert!(route::completed(&ctx));
            let current = (
                serde_json::to_vec(&a.progress).unwrap(),
                serde_json::to_vec(&a.progress.migration_receipts).unwrap(),
                a.one_off_presentation.memo.counters.steps,
                a.one_off_presentation.memo.counters.snapshots,
                a.one_off_presentation.command_executions(),
            );
            if let Some(before) = &closing_work {
                assert_eq!(
                    &current, before,
                    "Delayed native close does no new grading/copy/publication/work"
                );
            } else {
                closing_work = Some(current);
            }
            if closed >= 3 {
                break;
            }
        }
    }
    assert!(closed >= 3 && stop_interstitial_proved);
    assert!(
        stop_run_observed,
        "Real Run was active before the actual Stop event"
    );
    assert!(
        slow_wait_frames >= 2200,
        "Real slow-time Undo waits were exercised across many root frames"
    );
    let (environment_clicks, environment_fills, wheels, settled, stop_hovers) =
        route::delivery_observations(&ctx).unwrap();
    assert!(
        environment_clicks > 0 && environment_fills > 0,
        "Fresh absent LABEL is included, acknowledged, really focused and filled"
    );
    assert!(wheels > 0);
    assert_eq!(
        wheels, settled,
        "Every real wheel naturally settles before another delta or capture"
    );
    assert!(
        stop_hovers > 0,
        "Actual Stop has a separate natural hover frame before the button batch"
    );
    assert_eq!(shots, captures());
    assert_eq!(painted, 53);
    assert_eq!(serde_json::to_vec(&a.scenarios).unwrap(), catalog);
    assert_eq!(serde_json::to_vec(&a.units).unwrap(), units);
    assert_eq!(a.progress.attempts.len(), 4);
    assert!(a.progress.favorites.contains(route::IDS[0]));
    for id in route::IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert_eq!(
            a.one_off_presentation
                .memo
                .passed(&a.progress.attempts[id], s),
            Some(true)
        );
    }
}
