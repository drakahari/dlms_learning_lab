use super::*;
const IDS: [&str; 3] = [
    "shell-counter-rates",
    "shell-cpu-share",
    "shell-measurement-recovery",
];
#[test]
fn measurement_screens_keep_real_editors_and_actions_inside_their_columns() {
    for id in IDS {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
        for (width, scale, light) in [(1920., 1., false), (1920., 1.5, false), (946., 2., true)] {
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
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                app.set_script_view(index, view);
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                assert_eq!(app.metrics.lesson_tabs.len(), 3);
                assert!(!app.metrics.completion);
                if view == teaching::View::Introduction {
                    assert!(app.metrics.editor_viewport.is_none());
                    continue;
                }
                let editor = app.metrics.editor_viewport.unwrap();
                let main = app.metrics.main_viewport.unwrap();
                assert!(
                    editor.width() > 100.
                        && editor.left() >= main.left() - 1.
                        && editor.right() <= main.right() + 1.,
                    "{id} {view:?} {scale}: {editor:?} / {main:?}"
                );
                for r in [app.metrics.check.unwrap(), app.metrics.stop.unwrap()] {
                    assert!(r.left() >= main.left() - 1. && r.right() <= main.right() + 1.);
                }
                assert!(
                    (app.metrics.check.unwrap().height() - app.metrics.stop.unwrap().height())
                        .abs()
                        < 1.
                );
            }
        }
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        frame(&ctx, &mut app, 1200., 950., vec![]);
        assert!(app.metrics.lesson_tabs.is_empty());
        assert_eq!(app.script_view(index), teaching::View::Assignment);
    }
}

#[test]
fn measurement_worked_calculations_remain_visible_without_horizontal_scrolling() {
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
