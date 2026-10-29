//! Actual route/paint-phase regression, not a Fedora pixel or novice acceptance claim.
use super::*;
use crate::ui::smoke::lvm_lessons as route;
use std::collections::BTreeSet;
fn expected_captures() -> BTreeSet<String> {
    let mut names = BTreeSet::from([
        "lvm-normal-library-search-placeholder".to_owned(),
        "lvm-assignment-retained-files".to_owned(),
        "lvm-example-retained-files".to_owned(),
    ]);
    for id in [
        "lpic2-lvm-free-space",
        "lpic2-lvm-add-pv",
        "lpic2-lvm-filesystem-gap",
        "lpic2-lvm-handover",
    ] {
        for suffix in [
            "introduction",
            "whole-assignment",
            "example-inserted-not-run",
            "example-experiment",
            "command-refusal",
            "wrong-retry",
            "partial-capacity",
            "reference-complete",
            "alternate-1",
            "alternate-2",
            "challenge",
        ] {
            assert!(names.insert(format!("{id}-{suffix}")));
        }
    }
    names.insert("lpic2-lvm-free-space-success-invalidated".into());
    for scale in [100, 115, 125, 150, 200] {
        for theme in ["light", "dark"] {
            for width in ["wide", "narrow"] {
                assert!(names.insert(format!("lvm-handover-layout-{scale}-{theme}-{width}")));
            }
        }
    }
    names
}
fn flatten<'a>(s: &'a egui::Shape, clip: egui::Rect, out: &mut Vec<(egui::Rect, &'a egui::Shape)>) {
    if let egui::Shape::Vec(items) = s {
        for s in items {
            flatten(s, clip, out);
        }
    } else {
        out.push((clip, s));
    }
}
fn paint(ctx: &egui::Context, a: &LabApp, out: &egui::FullOutput, name: &str) {
    let mut items = Vec::new();
    for s in &out.shapes {
        flatten(&s.shape, s.clip_rect, &mut items);
    }
    let tolerance = 1. / out.pixels_per_point;
    assert!(out.pixels_per_point > 0. && out.pixels_per_point.is_finite());
    let theme_name = theme::find(&a.theme_id).name.clone();
    let scale = format!("{:.0}%", a.scale * 100.);
    for label in [
        "DLMS Learning Lab",
        "Offline simulation",
        "Text size",
        scale.as_str(),
        theme_name.as_str(),
        "Settings",
    ] {
        let (at, bounds) = items
            .iter()
            .enumerate()
            .find_map(|(at, (clip, shape))| {
                let egui::Shape::Text(text) = shape else {
                    return None;
                };
                let bounds = text.visual_bounding_rect();
                (text.galley.text() == label
                    && text.opacity_factor > 0.
                    && bounds.is_positive()
                    && clip.expand(tolerance).contains_rect(bounds)
                    && ctx.content_rect().expand(tolerance).contains_rect(bounds)
                    && text.galley.rows.iter().any(|r| {
                        !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
                    })
                    && text.galley.rows.iter().all(|r| {
                        r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                            .iter()
                            .all(|v| {
                                text.override_text_color
                                    .unwrap_or_else(|| {
                                        if v.color == egui::Color32::PLACEHOLDER {
                                            text.fallback_color
                                        } else {
                                            v.color
                                        }
                                    })
                                    .a()
                                    > 0
                            })
                    }))
                .then_some((at, bounds))
            })
            .unwrap_or_else(|| {
                panic!("Actual capture{name} lacks visible unclipped header{label:?}")
            });
        assert!(!items[at+1..].iter().any(|(clip,shape)|matches!(shape,egui::Shape::Rect(r)if r.fill.a()==255&&clip.intersect(r.rect).contains_rect(bounds))),"Later opaque paint covers header{label:?}");
    }
    if name.starts_with("lvm-handover-layout-") {
        let (_, field, id, clip) = &a.metrics.lvm.fields[0];
        assert!(clip.expand(tolerance).contains_rect(*field));
        assert_eq!(ctx.memory(|m| m.focused()), Some(*id));
        let text = items
            .iter()
            .find_map(|(clip, s)| match s {
                egui::Shape::Text(t)
                    if t.galley.text() == a.lvm_session_for_test_command()
                        && field.intersects(t.visual_bounding_rect()) =>
                {
                    Some((clip, t))
                }
                _ => None,
            })
            .expect("Populated command is actually painted");
        let bounds = text.1.visual_bounding_rect();
        assert!(text.0.expand(tolerance).contains_rect(bounds));
        assert!(field.expand(tolerance).contains_rect(bounds));
        let font = egui::TextStyle::Body.resolve(&ctx.style());
        let row = ctx.fonts_mut(|f| f.row_height(&font));
        assert!(
            (bounds.center().y - field.center().y).abs() <= row * 0.25 + tolerance,
            "Field text should use shared font-centered layout, no top-border alignment"
        );
    }
    if matches!(
        name,
        "lvm-assignment-retained-files" | "lvm-example-retained-files"
    ) {
        let source = &a.scenarios[a.selected.unwrap()];
        let State::Lvm { simulation } = &a.progress.attempts[&source.id].state else {
            panic!()
        };
        let world = if name == "lvm-example-retained-files" {
            &simulation.example
        } else {
            &simulation.current
        };
        let expected = &world
            .groups
            .values()
            .flat_map(|g| g.volumes.values())
            .filter_map(|v| v.filesystem.as_ref())
            .flat_map(|fs| fs.files.values())
            .next()
            .unwrap()
            .contents;
        let viewport = a
            .metrics
            .lvm
            .files_viewport
            .expect("Actual native opened inspection");
        assert!(
            a.metrics
                .main_viewport
                .unwrap()
                .expand(tolerance)
                .contains_rect(viewport)
        );
        assert!(
            items.iter().any(|(clip, shape)| {
                let egui::Shape::Text(text) = shape else {
                    return false;
                };
                let bounds = text.visual_bounding_rect();
                text.galley.text() == expected
                    && text.opacity_factor > 0.
                    && bounds.is_positive()
                    && clip.expand(tolerance).contains_rect(bounds)
                    && viewport.expand(tolerance).contains_rect(bounds)
                    && text.galley.rows.iter().any(|row| {
                        !row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()]
                            .is_empty()
                    })
                    && text.galley.rows.iter().all(|row| {
                        row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()]
                            .iter()
                            .all(|v| {
                                text.override_text_color
                                    .unwrap_or_else(|| {
                                        if v.color == egui::Color32::PLACEHOLDER {
                                            text.fallback_color
                                        } else {
                                            v.color
                                        }
                                    })
                                    .a()
                                    > 0
                            })
                    })
            }),
            "Opened native inspection capture must paint actual retained file text fully inside its clip, not only a scrollbar"
        );
    }
}
impl LabApp {
    fn lvm_session_for_test_command(&self) -> &str {
        let s = &self.scenarios[self.selected.unwrap()];
        let State::Lvm { simulation } = &self.progress.attempts[&s.id].state else {
            panic!()
        };
        &simulation.command
    }
}
#[test]
fn lvm_native_route_has_closed_inventory_and_exact_selection_without_weakening_other_routes() {
    assert_eq!(LabApp::lvm_stage(false, Some("lvm-lessons")), None);
    assert_eq!(LabApp::lvm_stage(true, Some("lvm-lessons")), Some(78000));
    for name in [
        None,
        Some("lvm"),
        Some("loop-recovery-lessons"),
        Some("one-off-lessons"),
    ] {
        assert_eq!(LabApp::lvm_stage(true, name), None);
    }
    let (first, last) = route::phase_bounds();
    assert_eq!(first, 78000);
    assert_eq!(
        last - first + 1,
        1046,
        "Independently counted fixed source action inventory"
    );
    assert!(last < 80000);
    let shots = route::planned_captures();
    assert_eq!(shots.len(), 68);
    assert_eq!(
        shots.iter().cloned().collect::<BTreeSet<_>>(),
        expected_captures()
    );
}
#[test]
fn lvm_normal186_real_events_complete_all_tasks_and_every_capture_phase_without_installed_success()
{
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 78000;
    assert!(!a.synthetic && !a.developer);
    assert_eq!((a.scenarios.len(), a.units.len()), (186, 49));
    assert!(a.progress.attempts.is_empty());
    let catalog = serde_json::to_vec(&a.scenarios).unwrap();
    let units = serde_json::to_vec(&a.units).unwrap();
    let mut pixels = egui::vec2(1920., 1080.);
    let mut fullscreen = true;
    let mut time = 0.;
    let mut shots = BTreeSet::new();
    let mut closed = 0;
    let mut previous = a.smoke_stage;
    let mut frames = 0;
    for _ in 0..60_000 {
        time += 0.05;
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
                pixels / (a.scale * 1.15),
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
        for v in out.viewport_output.values() {
            for command in &v.commands {
                match command {
                    egui::ViewportCommand::Fullscreen(value) => {
                        fullscreen = *value;
                        if *value {
                            pixels = egui::vec2(1920., 1080.);
                        }
                    }
                    egui::ViewportCommand::InnerSize(size) => {
                        if !fullscreen {
                            pixels = *size * 1.15;
                        }
                    }
                    egui::ViewportCommand::Screenshot(data) => {
                        let name = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(expected_captures().contains(&name));
                        assert!(shots.insert(name.clone()));
                        paint(&ctx, &a, &out, &name);
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => {
                        assert!(route::completed(&ctx));
                        closed += 1;
                    }
                    _ => {}
                }
            }
        }
        if a.smoke_stage == previous {
            frames += 1;
        } else {
            previous = a.smoke_stage;
            frames = 0;
        }
        assert!(frames < 1000, "Finite native stage{}", a.smoke_stage);
        if closed >= 3 {
            break;
        }
    }
    assert!(closed >= 3, "Actual idempotent close reached");
    assert_eq!(
        route::recency_observations(&ctx),
        Some((1, 4)),
        "The real Next/return and all four real Back/Continue recency laws were exercised, not skipped"
    );
    assert_eq!(shots, expected_captures());
    assert_eq!(a.progress.attempts.len(), 4);
    assert_eq!(serde_json::to_vec(&a.scenarios).unwrap(), catalog);
    assert_eq!(serde_json::to_vec(&a.units).unwrap(), units);
    for id in [
        "lpic2-lvm-free-space",
        "lpic2-lvm-add-pv",
        "lpic2-lvm-filesystem-gap",
        "lpic2-lvm-handover",
    ] {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
    }
}
