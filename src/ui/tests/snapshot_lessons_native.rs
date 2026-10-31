//! Paired actual-egui flow/capture checks; no solved model or loaded profile installed.
use super::*;
use crate::ui::smoke::snapshot_lessons as route;
#[derive(Clone)]
struct Window {
    size: egui::Vec2,
    fullscreen: bool,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            size: egui::vec2(760., 900.),
            fullscreen: false,
        }
    }
}
fn tick(
    ctx: &egui::Context,
    a: &mut LabApp,
    window: &mut Window,
    time: f64,
) -> (Option<String>, egui::FullOutput) {
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
    let mut input = egui::RawInput {
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            if window.fullscreen {
                egui::vec2(1920., 1080.)
            } else {
                window.size
            } / ctx.zoom_factor(),
        )),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .fullscreen = Some(window.fullscreen);
    let mut capture = None;
    let output = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        let (advance, shot) = a.snapshot_lessons_native(ctx, a.smoke_stage);
        if advance {
            a.smoke_stage += 1;
        }
        if shot.is_some() {
            assert!(
                capture.is_none(),
                "At most one capture phase per root frame"
            );
            capture = shot;
        }
    });
    for v in output.viewport_output.values() {
        for c in &v.commands {
            match c {
                // Deliberately ignore InnerSize, as the observed native tiling WM did.
                egui::ViewportCommand::InnerSize(_) => {}
                egui::ViewportCommand::Fullscreen(full) => window.fullscreen = *full,
                _ => {}
            }
        }
    }
    (capture, output)
}
fn full_progress(p: &Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
fn painted_rows(s: &egui::Shape, text: &str) -> Option<Vec<egui::Rect>> {
    match s {
        egui::Shape::Text(s)
            if s.galley.text() == text && s.override_text_color.is_none_or(|c| c.a() > 0) =>
        {
            assert!(!s.galley.elided && s.opacity_factor > 0.);
            assert!(s.galley.rows.iter().any(|r| {
                !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
            }));
            assert!(s.galley.rows.iter().all(|r| {
                r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                    .iter()
                    .all(|v| {
                        s.override_text_color
                            .unwrap_or_else(|| {
                                if v.color == egui::Color32::PLACEHOLDER {
                                    s.fallback_color
                                } else {
                                    v.color
                                }
                            })
                            .a()
                            > 0
                    })
            }));
            let rows = s
                .galley
                .rows
                .iter()
                .filter(|r| !r.text().trim().is_empty())
                .map(|r| r.rect().translate(s.pos.to_vec2()))
                .collect::<Vec<_>>();
            assert!(!rows.is_empty());
            Some(rows)
        }
        egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| painted_rows(s, text)),
        _ => None,
    }
}
#[test]
fn focused_snapshot_native_route_uses_real_edit_run_stop_reset_continue_and_every_capture_phase() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = route::BASE;
    assert_eq!((a.scenarios.len(), a.units.len()), (201, 54));
    assert!(a.progress.attempts.is_empty());
    assert!(a._smoke_profile.is_some());
    let (actions, expected) = route::inventory();
    assert_eq!(actions, 479);
    assert_eq!(expected.len(), 74);
    assert_eq!(
        expected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        expected.len()
    );
    let mut window = Window::default();
    let mut actual = Vec::new();
    let mut old_stage = a.smoke_stage;
    let mut stagnant = 0;
    let mut ended = false;
    let mut settings = std::collections::BTreeSet::new();
    for frame in 0..30_000 {
        let goal_before = route::goal_stage(a.smoke_stage).then(|| full_progress(&a.progress));
        let (shot, output) = tick(&ctx, &mut a, &mut window, frame as f64 * 0.05);
        if let Some(before) = goal_before {
            assert_eq!(
                full_progress(&a.progress),
                before,
                "Goal reveal/capture preserves full Progress including genuine receipts"
            );
        }
        if a.smoke_stage == old_stage {
            stagnant += 1;
        } else {
            old_stage = a.smoke_stage;
            stagnant = 0;
        }
        assert!(
            stagnant < 3000,
            "Actual phase failed to progress within existing bounded frame premise"
        );
        if let Some(name) = shot {
            let i = a.selected.expect("Real capture has a selected normal task");
            let id = a.scenarios[i].id.clone();
            assert!(route::IDS.contains(&id.as_str()));
            if name.contains("whole-assignment") || name.contains("goal200") {
                let task = &a.scenarios[i].objective;
                let viewport = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let found = output
                    .shapes
                    .iter()
                    .find_map(|s| {
                        painted_rows(&s.shape, task)
                            .map(|rows| (rows, s.clip_rect.intersect(viewport)))
                    })
                    .expect("Actual capture must paint the full nonelided task galley");
                let tolerance = 1. / ctx.pixels_per_point();
                assert!(
                    found
                        .0
                        .iter()
                        .all(|r| r.left() >= found.1.left() - tolerance
                            && r.right() <= found.1.right() + tolerance),
                    "All task galley rows fit horizontally; vertical edge rows are separate captures"
                );
                let row = if name.ends_with("bottom") {
                    *found.0.last().unwrap()
                } else {
                    found.0[0]
                };
                assert!(smoke::smoke_control_visible(
                    row,
                    found.1,
                    ctx.pixels_per_point()
                ));
            }
            if name.ends_with("-wrong-feedback") || name.ends_with("-partial-feedback") {
                let engine::State::Script {
                    run: Some(record), ..
                } = &a.progress.attempts[&id].state
                else {
                    panic!("Real Run required")
                };
                let engine::Goal::Script { cases } = &a.scenarios[i].goal else {
                    panic!()
                };
                let failed = cases
                    .iter()
                    .zip(&record.results)
                    .filter(|(c, r)| !c.accepts(r))
                    .count();
                assert!(failed > 0 && failed < cases.len());
                assert!(!progress::current_solved(
                    &a.progress.attempts[&id],
                    &a.scenarios[i]
                ));
                let text = format!(
                    "Overall result · Needs another edit · {failed} of {} input sets need attention. Select a failing case below to inspect it.",
                    cases.len()
                );
                let visible = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(
                    output
                        .shapes
                        .iter()
                        .any(|s| painted_rows(&s.shape, &text).is_some_and(|rows| rows
                            .iter()
                            .all(|r| smoke::smoke_control_visible(
                                *r,
                                s.clip_rect.intersect(visible),
                                ctx.pixels_per_point()
                            )))),
                    "Wrong/partial capture must paint actual failed overall result despite passing individual cases"
                );
            }
            if name.ends_with("-complete")
                && ["100-", "150-", "200-"].iter().any(|p| name.contains(p))
            {
                let summary = "Overall result · Correct solution · All input sets pass.";
                let viewport = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(
                    output
                        .shapes
                        .iter()
                        .any(|s| painted_rows(&s.shape, summary).is_some_and(|rows| rows
                            .iter()
                            .all(|r| smoke::smoke_control_visible(
                                *r,
                                s.clip_rect.intersect(viewport),
                                ctx.pixels_per_point()
                            )))),
                    "Capture must paint readable current overall completion"
                );
                assert!(progress::current_solved(
                    &a.progress.attempts[&id],
                    &a.scenarios[i]
                ));
                settings.insert((
                    id,
                    (a.scale * 100.).round() as u32,
                    a.theme_id.clone(),
                    window.fullscreen,
                ));
                if name.contains("-windowed-") {
                    assert!(
                        !window.fullscreen && (window.size.x - 760.).abs() < 1.,
                        "Representative windowed capture must use the actual narrow provider"
                    );
                } else {
                    assert!(
                        window.fullscreen,
                        "Wide capture must acknowledge actual fullscreen"
                    );
                }
            }
            actual.push(name);
        }
        if output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }) {
            ended = true;
            break;
        }
    }
    assert!(
        ended,
        "Real route must reach its final Close only after all assertions"
    );
    assert_eq!(actual, expected);
    assert_eq!(settings.len(), 24);
    assert!(a.selected.is_none());
    assert!(a.script_job.is_none());
    for id in route::IDS {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn focused_snapshot_controller_never_dispatches_twice_in_one_root_frame() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = route::BASE;
    let mut first = None;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1100., 900.),
        )),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .fullscreen = Some(false);
    let _ = ctx.run(input, |ctx| {
        a.render(ctx);
        if first.is_none() {
            first = Some(a.snapshot_lessons_native(ctx, a.smoke_stage));
        }
        let before = (
            serde_json::to_value(&a.progress).unwrap(),
            a.pending_events.clone(),
            a.smoke_stage,
        );
        assert_eq!(a.snapshot_lessons_native(ctx, a.smoke_stage), (false, None));
        assert_eq!(
            (
                serde_json::to_value(&a.progress).unwrap(),
                a.pending_events.clone(),
                a.smoke_stage
            ),
            before
        );
    });
    assert!(first.is_some_and(|r| r.0));
}
