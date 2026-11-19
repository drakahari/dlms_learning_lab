use super::*;
use dlms_learning_lab::{partition_fs_workspace as work, script_teaching::View};
const IDS: [&str; 2] = ["lpic1-layout-log-growth", "lpic1-layout-media-policy"];
fn view(app: &mut LabApp, id: &str, v: View) {
    let s = app.scenarios[app.selected.unwrap()].clone();
    work::binding::set_view(app.progress.attempts.get_mut(id).unwrap(), &s, v).unwrap();
    app.lesson_scroll_reset = true;
}
#[test]
fn disk_layout_fields_actions_and_checklists_reflow_without_answers() {
    for id in IDS {
        let (ctx, mut app, _) = super::nm_lessons::opened(id);
        for (width, scale, light) in [(2200., 1., false), (2200., 1.5, false), (946., 2., true)] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &app.theme_id);
            for v in [View::Introduction, View::Example, View::Assignment] {
                view(&mut app, id, v);
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                let m = &app.metrics.partition_fs;
                if v == View::Introduction {
                    assert!(m.introduction.is_some());
                    assert!(m.fields.is_empty());
                    continue;
                }
                for key in ["name", "start", "end", "label", "uuid"] {
                    let (_, r, _, clip) = m.fields.iter().find(|r| r.0 == key).unwrap();
                    assert!(
                        r.width() > 0.
                            && r.left() >= clip.left() - 1.
                            && r.right() <= clip.right() + 1.,
                        "{id} {v:?} {scale} {key}: {r:?} / {clip:?}"
                    );
                }
                let heights: Vec<_> = m.fields.iter().map(|r| r.1.height()).collect();
                assert!(heights.iter().all(|h| (h - heights[0]).abs() < 1.));
                for key in ["disk", "add", "format", "check"] {
                    let (_, r, clip) = m.controls.iter().find(|r| r.0 == key).unwrap();
                    assert!(
                        r.right() <= clip.right() + 1. && r.left() >= clip.left() - 1.,
                        "{id} {key} {r:?} {clip:?}"
                    );
                }
                assert!(!app.metrics.completion);
                assert!(!m.controls.iter().any(|r| r.0 == "results"));
            }
        }
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        for _ in 0..3 {
            frame(&ctx, &mut app, 1200., 1000., vec![]);
        }
        assert!(
            !app.metrics
                .partition_fs
                .controls
                .iter()
                .any(|r| matches!(r.0.as_str(), "practice" | "introduction"))
        );
        assert!(!app.metrics.completion);
    }
}
#[test]
fn partial_layout_feedback_is_failed_first_and_current_state_can_invalidate_success() {
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let s = app.scenarios[index].clone();
    let d = work::binding::source(&s).unwrap();
    view(&mut app, id, View::Assignment);
    {
        let a = app.progress.attempts.get_mut(id).unwrap();
        work::binding::apply(a, &s, &d.reference[0], false).unwrap();
        work::binding::check(a, &s, false).unwrap();
    }
    frame(&ctx, &mut app, 1600., 1400., vec![]);
    assert!(!app.metrics.completion);
    assert!(!app.metrics.partition_fs.rows.is_empty());
    assert!(app.metrics.partition_fs.rows.iter().all(|r| !r.0));
    assert!(
        app.metrics
            .partition_fs
            .controls
            .iter()
            .any(|r| r.0 == "all-results")
    );
    {
        let a = app.progress.attempts.get_mut(id).unwrap();
        for op in &d.reference[1..] {
            work::binding::apply(a, &s, op, false).unwrap();
        }
        work::binding::check(a, &s, false).unwrap();
    }
    app.completion_index.forget(id);
    frame(&ctx, &mut app, 1600., 1400., vec![]);
    assert!(app.metrics.completion);
    assert!(
        app.metrics
            .partition_fs
            .controls
            .iter()
            .any(|r| r.0 == "results")
    );
    {
        let a = app.progress.attempts.get_mut(id).unwrap();
        work::binding::apply(
            a,
            &s,
            &work::Operation::AddPartition {
                disk: "/dev/vda".into(),
                name: "intruder".into(),
                start: 40 * 2048,
                end: 41 * 2048 - 1,
                type_hint: dlms_learning_lab::partition_fs::TypeHint::LinuxData,
            },
            false,
        )
        .unwrap();
    }
    app.completion_index.forget(id);
    frame(&ctx, &mut app, 1600., 1400., vec![]);
    assert!(!app.metrics.completion);
    assert!(
        !app.metrics
            .partition_fs
            .controls
            .iter()
            .any(|r| r.0 == "results")
    );
}
#[test]
fn tab_moves_between_real_single_line_fields_and_preserves_draft_input() {
    let id = IDS[0];
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    view(&mut app, id, View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 2200., 2200., vec![]);
    }
    let fields = &app.metrics.partition_fs.fields;
    let name = fields.iter().find(|r| r.0 == "name").unwrap().2;
    let start = fields.iter().find(|r| r.0 == "start").unwrap().2;
    ctx.memory_mut(|m| m.request_focus(name));
    frame(
        &ctx,
        &mut app,
        2200.,
        2200.,
        vec![egui::Event::Text("draft-logs".into())],
    );
    let key = |pressed| egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    };
    frame(&ctx, &mut app, 2200., 2200., vec![key(true), key(false)]);
    frame(&ctx, &mut app, 2200., 2200., vec![]);
    assert_eq!(ctx.memory(|m| m.focused()), Some(start));
    let State::PartitionFs { simulation: w } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(
        w.assignment_editor.drafts[&w.assignment_editor.selected].name,
        "draft-logs"
    );
    assert!(!progress::current_solved(
        &app.progress.attempts[id],
        &app.scenarios[app.selected.unwrap()]
    ));
}

#[test]
fn worked_calculation_lines_are_visible_without_horizontal_scrolling() {
    fn visit(
        shape: &egui::Shape,
        clip: egui::Rect,
        wanted: &[String],
        found: &mut std::collections::BTreeSet<String>,
        tolerance: f32,
    ) {
        match shape {
            egui::Shape::Vec(items) => {
                for s in items {
                    visit(s, clip, wanted, found, tolerance)
                }
            }
            egui::Shape::Text(t) if wanted.iter().any(|w| w == t.galley.text()) => {
                let b = t.visual_bounding_rect();
                assert!(
                    b.left() >= clip.left() - tolerance && b.right() <= clip.right() + tolerance,
                    "Worked calculation hidden horizontally: {:?}: {b:?} / {clip:?}",
                    t.galley.text()
                );
                found.insert(t.galley.text().into());
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
        let s = app.scenarios[index].clone();
        let wanted: Vec<_> = lesson_presentation::entry(&s)
            .unwrap()
            .worked
            .iter()
            .map(|r| r.value.clone())
            .collect();
        for (width, scale) in [(1300f32, 1f32), (1600., 1.5), (946., 2.)] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            let mut found = std::collections::BTreeSet::new();
            for _ in 0..3 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2((width / scale - 32.).min(1220.), 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            lesson_presentation::worked_rows(ui, &s);
                        });
                    },
                );
                for clipped in &output.shapes {
                    visit(
                        &clipped.shape,
                        clipped.clip_rect,
                        &wanted,
                        &mut found,
                        1. / output.pixels_per_point,
                    );
                }
            }
            assert_eq!(
                found.len(),
                wanted.len(),
                "Every actual value must be painted at{width}px/{scale}zoom; do not merely test string lengths"
            );
        }
    }
}
