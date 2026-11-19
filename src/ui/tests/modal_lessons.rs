use super::*;
const IDS: [&str; 3] = [
    "lpic1-modal-setting-repair",
    "lpic1-modal-record-maintenance",
    "lpic1-modal-recovery",
];
fn opened(id: &str) -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    (ctx, a)
}
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let State::ModalEditing { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn paint(ctx: &egui::Context, a: &mut LabApp, width: f32) -> String {
    fn collect(s: &egui::Shape, v: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => v.push(t.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, v);
                }
            }
            _ => {}
        }
    }
    for _ in 0..3 {
        frame(ctx, a, width, 7000., vec![]);
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in out.shapes {
        collect(&s.shape, &mut text);
    }
    text.join("\n")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    paint(ctx, a, 1600.);
    let r = a
        .metrics
        .modal
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1600.,
            7000.,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
}
#[test]
fn modal_lessons_render_all_three_real_views_without_default_assignment_solutions() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        let intro = paint(&ctx, &mut a, 1600.);
        assert!(intro.contains("Worked example"));
        assert!(intro.contains("Try this example"));
        assert!(!intro.contains("Check saved files") || intro.contains("NORMAL"));
        view(&mut a, id, teaching::View::Example);
        let practice = paint(&ctx, &mut a, 1600.);
        assert!(practice.contains("Show worked answer"));
        assert!(practice.contains("Check example"));
        assert!(practice.contains("Actual saved file"));
        view(&mut a, id, teaching::View::Assignment);
        let assignment = paint(&ctx, &mut a, 1600.);
        assert!(assignment.contains("Check saved files"));
        assert!(assignment.contains("NORMAL"));
        assert!(!assignment.contains("Show worked answer"));
        assert!(!assignment.contains("9090"));
        assert!(!assignment.contains("5050"));
        assert!(!assignment.contains("smoke-check"));
    }
}
#[test]
fn actual_worked_button_changes_only_practice_and_failing_checks_explain_saved_values() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        view(&mut a, id, teaching::View::Example);
        let before = serde_json::to_value(&a.progress.attempts[id].state).unwrap();
        tap(&ctx, &mut a, "worked");
        let State::ModalEditing { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(!simulation.practice.checked);
        assert_eq!(
            serde_json::to_value(&simulation.assignment).unwrap(),
            before["simulation"]["assignment"]
        );
        tap(&ctx, &mut a, "check");
        let result = paint(&ctx, &mut a, 1600.);
        assert!(result.contains("Example checks pass"));
        assert!(!a.metrics.completion);
        view(&mut a, id, teaching::View::Assignment);
        tap(&ctx, &mut a, "check");
        let result = paint(&ctx, &mut a, 1600.);
        assert!(result.contains("Needs attention"));
        assert!(result.contains("saved") || result.contains("Unsaved buffer"));
        assert!(!a.metrics.completion);
    }
}
#[test]
fn modal_assignment_buttons_and_editor_align_at_all_supported_scales_and_two_real_themes() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        view(&mut a, id, teaching::View::Assignment);
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &a.theme_id);
                assert_eq!(ctx.style().visuals.dark_mode, !light);
                for width in [760., 1600.] {
                    paint(&ctx, &mut a, width);
                    let bounds = ctx.content_rect();
                    for key in ["file", "open", "check"] {
                        let r = a
                            .metrics
                            .modal
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap()
                            .1;
                        assert!(
                            r.left() >= bounds.left() && r.right() <= bounds.right() + 1.,
                            "{id} {scale} {width} {key} {r:?}"
                        );
                    }
                    let (_, clip, _) = a.metrics.modal.editor.unwrap();
                    assert!(clip.left() >= bounds.left() && clip.right() <= bounds.right() + 1.);
                    assert!(a.metrics.modal.cursor.is_some());
                }
            }
        }
    }
}
#[test]
fn challenge_button_keeps_the_full_assignment_and_hides_unsolicited_guidance() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        paint(&ctx, &mut a, 1600.);
        let before = serde_json::to_value(&a.progress.attempts[id].state).unwrap();
        let rect = a.metrics.mode_challenge.unwrap();
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut a,
                1600.,
                7000.,
                vec![
                    egui::Event::PointerMoved(rect.center()),
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        let text = paint(&ctx, &mut a, 1600.);
        assert!(a.progress.attempts[id].challenge);
        assert!(text.contains("Check saved files"));
        assert!(text.contains("Virtual file"));
        assert!(!text.contains("Show worked answer"));
        assert!(!text.contains("Try this example"));
        assert!(!text.contains("9090"));
        assert!(!text.contains("5050"));
        assert_eq!(
            serde_json::to_value(&a.progress.attempts[id].state).unwrap(),
            before,
            "Mode selection must preserve both activity drafts"
        );
    }
}
