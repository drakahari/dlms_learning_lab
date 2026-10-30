//! Real-egui phase and actual-paint assertions, not installed/native PNG acceptance.
use super::*;
use crate::ui::smoke::disk_preparation as route;
use dlms_learning_lab::engine::Goal;
use std::collections::BTreeSet;
const IDS: [&str; 4] = [
    "lpic1-disk-safe-gap",
    "lpic1-disk-hints-versus-content",
    "lpic1-disk-eligible-media",
    "lpic1-disk-delivery",
];
fn expected_captures() -> BTreeSet<String> {
    let mut names = BTreeSet::from(["disk-fixture-library".to_owned()]);
    for id in IDS {
        for suffix in [
            "introduction",
            "whole-assignment",
            "separate-practice",
            "filled-not-applied",
            "practice-experiment",
            "practice-inspection",
            "atomic-refusal",
            "wrong-applied-result",
            "partial-result",
            "reference-complete",
            "assignment-inspection",
            "alternate-1",
            "alternate-2",
            "success-invalidated",
            "challenge",
        ] {
            assert!(names.insert(format!("{id}-{suffix}")));
        }
    }
    for suffix in ["requested-hint", "requested-worked", "requested-reference"] {
        assert!(names.insert(format!("{}-{suffix}", IDS[0])));
    }
    for scale in [100, 115, 125, 150, 200] {
        for theme in ["light", "dark"] {
            for width in ["wide", "narrow"] {
                assert!(names.insert(format!("disk-delivery-layout-{scale}-{theme}-{width}")));
            }
        }
    }
    names
}
fn flatten<'a>(
    shape: &'a egui::Shape,
    clip: egui::Rect,
    out: &mut Vec<(egui::Rect, &'a egui::Shape)>,
) {
    if let egui::Shape::Vec(items) = shape {
        for s in items {
            flatten(s, clip, out);
        }
    } else {
        out.push((clip, shape));
    }
}
fn painted(
    items: &[(egui::Rect, &egui::Shape)],
    root: egui::Rect,
    tol: f32,
    wanted: &str,
) -> egui::Rect {
    let (at, bounds) = items
        .iter()
        .enumerate()
        .find_map(|(at, (clip, shape))| {
            let egui::Shape::Text(t) = shape else {
                return None;
            };
            let bounds = t.visual_bounding_rect();
            (t.galley.text() == wanted
                && t.opacity_factor > 0.
                && bounds.is_positive()
                && clip.expand(tol).contains_rect(bounds)
                && root.expand(tol).contains_rect(bounds)
                && t.galley.rows.iter().any(|r| {
                    !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
                })
                && t.galley.rows.iter().all(|r| {
                    r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                        .iter()
                        .all(|v| {
                            t.override_text_color
                                .unwrap_or_else(|| {
                                    if v.color == egui::Color32::PLACEHOLDER {
                                        t.fallback_color
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
        .unwrap_or_else(|| panic!("Actual requested text not fully painted: {wanted:?}"));
    assert!(!items[at+1..].iter().any(|(clip,s)|matches!(s,egui::Shape::Rect(r)if r.fill.a()==255&&clip.intersect(r.rect).contains_rect(bounds))),"Later opaque paint covers requested text");
    bounds
}
fn paint(ctx: &egui::Context, a: &LabApp, out: &egui::FullOutput, name: &str) {
    assert!(out.pixels_per_point.is_finite() && out.pixels_per_point > 0.);
    let tol = 1. / out.pixels_per_point;
    let mut items = vec![];
    for s in &out.shapes {
        flatten(&s.shape, s.clip_rect, &mut items);
    }
    let root = ctx.content_rect();
    let scale = format!("{:.0}%", a.scale * 100.);
    let theme = theme::find(&a.theme_id).name.clone();
    for text in [
        "DLMS Learning Lab",
        "Offline simulation",
        "Text size",
        scale.as_str(),
        theme.as_str(),
        "Settings",
    ] {
        painted(&items, root, tol, text);
    }
    if name == "disk-fixture-library" {
        assert!(a.progress.attempts.is_empty());
        assert!(a.metrics.library_search.unwrap().is_positive());
        return;
    }
    let source = &a.scenarios[a.selected.unwrap()];
    let State::PartitionFs { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    let Goal::PartitionFs { definition } = &source.goal else {
        panic!()
    };
    if name.ends_with("-introduction") {
        assert_eq!(
            simulation.view,
            dlms_learning_lab::script_teaching::View::Introduction
        );
        painted(&items, root, tol, &definition.lesson.prerequisite_note);
    }
    if name.ends_with("-whole-assignment") {
        painted(&items, root, tol, &source.objective);
    }
    if name.ends_with("-separate-practice") {
        painted(&items, root, tol, &definition.lesson.description);
        assert_eq!(simulation.current, definition.initial);
    }
    if name.ends_with("-filled-not-applied") {
        assert_eq!(simulation.example, definition.lesson.fixture);
        assert!(simulation.example_transcript.is_empty() && !simulation.example_checked);
        assert_eq!(simulation.current, definition.initial);
    }
    if name.ends_with("-atomic-refusal") {
        let notice = a.notice.as_ref().expect("Actual refused operation");
        painted(&items, root, tol, &format!("Needs attention · {notice}"));
    }
    if [
        "-wrong-applied-result",
        "-partial-result",
        "-practice-experiment",
        "-success-invalidated",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
    {
        let target = a
            .metrics
            .partition_fs
            .rows
            .iter()
            .find(|r| !r.0)
            .expect("An actual failed objective must exist")
            .1;
        assert!(items.iter().any(|(clip,s)|matches!(s,egui::Shape::Text(t)if t.galley.text().starts_with("Needs attention ·")&&t.opacity_factor>0.&&target.intersects(t.visual_bounding_rect())&&clip.expand(tol).contains_rect(t.visual_bounding_rect())&&root.expand(tol).contains_rect(t.visual_bounding_rect()))));
    }
    if name.ends_with("-practice-inspection") || name.ends_with("-assignment-inspection") {
        let practice = name.ends_with("-practice-inspection");
        let world = if practice {
            &simulation.example
        } else {
            &simulation.current
        };
        let retained = world.disks.iter().find_map(|(path, d)| {
            d.table.as_ref().and_then(|table| {
                table.partitions.iter().find_map(|part| {
                    if let dlms_learning_lab::partition_fs::Content::Ext4 { files, .. } =
                        &part.content
                    {
                        files.iter().next().map(|(file, data)| {
                            format!("{path}{} · {file}: {}", part.number, data.text)
                        })
                    } else {
                        None
                    }
                })
            })
        });
        let text = retained.unwrap_or_else(|| {
            let row = if practice {
                &simulation.example_transcript[0]
            } else {
                &simulation.transcript[0]
            };
            format!("{}\n{}", row.input, row.output)
        });
        painted(&items, root, tol, &text);
    }
    if name.ends_with("-requested-hint") {
        assert_eq!(a.progress.attempts[&source.id].hints, 1);
        painted(&items, root, tol, &source.hints[0]);
        assert!(simulation.transcript.is_empty() && simulation.example_transcript.is_empty());
    }
    if name.ends_with("-requested-worked") {
        painted(&items, root, tol, &definition.reference[0].description());
    }
    if name.ends_with("-requested-reference") {
        painted(
            &items,
            root,
            tol,
            dlms_learning_lab::partition_fs_workspace::HELP,
        );
    }
    if name.starts_with("disk-delivery-layout-") {
        let (_, field, id, clip) = a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == "name")
            .unwrap();
        assert!(clip.expand(tol).contains_rect(*field));
        assert_eq!(ctx.memory(|m| m.focused()), Some(*id));
        let text =
            &simulation.assignment_editor.drafts[&simulation.assignment_editor.selected].name;
        assert_eq!(text, "resumed");
        let bounds = painted(&items, root, tol, text);
        assert!(field.expand(tol).contains_rect(bounds));
        let font = egui::TextStyle::Body.resolve(&ctx.style());
        let row = ctx.fonts_mut(|f| f.row_height(&font));
        assert!(
            (bounds.center().y - field.center().y).abs() <= row * 0.25 + tol,
            "Shared centered field layout must survive every scale"
        );
    }
}
#[test]
fn conditional_disk_route_has_fixed_closed_inventory_without_broadening_existing_routes() {
    assert_eq!(
        LabApp::disk_stage(false, Some("disk-preparation-fixture")),
        None
    );
    assert_eq!(
        LabApp::disk_stage(true, Some("disk-preparation-fixture")),
        Some(90000)
    );
    for name in [
        None,
        Some("ntp-lessons"),
        Some("lvm-lessons"),
        Some("disk-preparation"),
        Some("one-off-lessons"),
    ] {
        assert_eq!(LabApp::disk_stage(true, name), None);
    }
    let (first, last) = route::phase_bounds();
    assert_eq!(first, 90000);
    assert_eq!(last - first + 1, 2517);
    assert!(last < 94000);
    let captures = route::planned_captures();
    assert_eq!(captures.len(), 84);
    assert_eq!(
        captures.into_iter().collect::<BTreeSet<_>>(),
        expected_captures()
    );
}
#[test]
fn fixed_private_source_route_uses_real_controls_preserves_prefix_and_paints_every_capture() {
    run_disk_route(true);
}
#[test]
fn main_registered_four_tasks_use_real_controls_without_catalog_or_attempt_injection() {
    run_disk_route(false);
}
fn run_disk_route(private_fixture: bool) {
    use crate::ui::one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as registration;
    let (ctx, mut a) = app();
    assert!(!a.synthetic && !a.developer);
    assert!(a.progress.attempts.is_empty());
    assert_eq!(a.scenarios.len(), 197);
    assert_eq!(a.units.len(), 52);
    // Both modes begin with the genuine normal catalog. Only the private
    // predecessor exercise peels the independently checked exact append, while
    // there are no Attempts. The normal route never installs source or work.
    let old_scenarios = registration::before_scenarios(&a.scenarios);
    let old_units = registration::before_typed_units(&a.units);
    assert_eq!(old_scenarios.len(), 193);
    assert_eq!(old_units.len(), 51);
    if private_fixture {
        let units = old_units.to_vec();
        let scenarios = old_scenarios.to_vec();
        a.scenarios = scenarios;
        a.units = units;
    }
    let before = serde_json::to_value(&a.scenarios).unwrap();
    let before_units = serde_json::to_value(&a.units).unwrap();
    let old_count = a.scenarios.len();
    let old_units = a.units.len();
    assert_eq!(
        IDS.iter()
            .filter(|id| a.scenarios.iter().any(|s| s.id == **id))
            .count(),
        if private_fixture { 0 } else { 4 },
        "Private bootstrap must begin without the four sources; MAIN must already contain them"
    );
    a.smoke = true;
    a.smoke_stage = 90000;
    let mut pixels = egui::vec2(1920., 1080.);
    let mut fullscreen = true;
    let mut time = 0.;
    let mut shots = BTreeSet::new();
    let mut closed = 0;
    let mut previous = a.smoke_stage;
    let mut frames = 0;
    for _ in 0..120_000 {
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
        assert!(
            frames < 1000,
            "Bounded actual native stage{}",
            a.smoke_stage
        );
        if closed >= 3 {
            break;
        }
    }
    assert!(closed >= 3);
    assert_eq!(shots, expected_captures());
    assert_eq!(route::recency_observations(&ctx), Some((1, 4)));
    assert_eq!(a.progress.attempts.len(), 4);
    assert_eq!(
        serde_json::to_value(&a.scenarios[..old_count]).unwrap(),
        before
    );
    assert_eq!(
        serde_json::to_value(&a.units[..old_units]).unwrap(),
        before_units
    );
    assert_eq!(
        a.scenarios.len(),
        old_count + if private_fixture { 4 } else { 0 }
    );
    assert_eq!(a.units.len(), old_units + usize::from(private_fixture));
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
    }
}
