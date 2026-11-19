use super::*;
use dlms_learning_lab::device_path_workspace as work;

fn view(app: &mut LabApp, index: usize, view: teaching::View) {
    let State::DevicePath { simulation } = &mut app
        .progress
        .attempts
        .get_mut(&app.scenarios[index].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation.view = view;
    app.lesson_scroll_reset = true;
}

#[test]
fn cable_controls_reflow_and_keep_consistent_heights_at_supported_text_scales() {
    for id in [
        "ccna-device-path-patching",
        "ccna-device-path-broadcast-boundary",
    ] {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
        for (width, scale, light) in [(2200., 1., false), (2200., 1.5, false), (1268., 2., true)] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|theme| theme.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &app.theme_id);
            for screen in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                view(&mut app, index, screen);
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1308., vec![]);
                }
                if screen == teaching::View::Introduction {
                    continue;
                }
                let (_, topology, _) = app
                    .metrics
                    .device_path
                    .controls
                    .iter()
                    .find(|row| row.0 == "topology")
                    .unwrap();
                assert!(
                    topology.width() > 0. && topology.right() <= ctx.content_rect().right() + 1.
                );
                let fields: Vec<_> = app
                    .metrics
                    .device_path
                    .controls
                    .iter()
                    .filter(|row| row.0.ends_with("-a") || row.0.ends_with("-b"))
                    .collect();
                assert_eq!(fields.len(), if id.ends_with("patching") { 10 } else { 12 });
                let height = fields[0].1.height();
                for (_, rect, clip) in fields {
                    assert!(
                        rect.left() >= clip.left() - 1. && rect.right() <= clip.right() + 1.,
                        "{id} {scale} {rect:?} {clip:?}"
                    );
                    assert!((rect.height() - height).abs() < 1.);
                }
            }
        }
    }
}

#[test]
fn challenge_hides_worked_wiring_and_broken_edits_clear_current_success() {
    let id = "ccna-device-path-broadcast-boundary";
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let source = app.scenarios[index].clone();
    engine::execute_for_goal(
        &mut app.progress.attempts.get_mut(id).unwrap().state,
        &source.goal,
        &source.solution[0],
    )
    .unwrap();
    view(&mut app, index, teaching::View::Assignment);
    app.progress.attempts.get_mut(id).unwrap().challenge = true;
    draw(&ctx, &mut app);
    assert!(app.metrics.completion);
    assert!(
        !app.metrics
            .device_path
            .controls
            .iter()
            .any(|row| matches!(row.0.as_str(), "example" | "introduction" | "fill"))
    );
    let definition = work::source(&source).unwrap();
    let State::DevicePath { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
        .assignment
        .edit(&definition.initial, definition.initial.initial.clone())
        .unwrap();
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    assert!(!progress::current_solved(
        &app.progress.attempts[id],
        &source
    ));
}

fn target(ctx: &egui::Context, app: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, app, 1200., 950., vec![]);
        let (_, rect, clip) = app
            .metrics
            .device_path
            .controls
            .iter()
            .find(|row| row.0 == key)
            .unwrap();
        let rect = *rect;
        let clip = clip
            .intersect(app.metrics.main_viewport.unwrap())
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            return rect;
        }
        frame(
            ctx,
            app,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - rect.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, app, 1200., 950., vec![]);
        }
    }
    panic!("Device-path control not visible: {key}")
}
fn tap(ctx: &egui::Context, app: &mut LabApp, key: &str) {
    let rect = target(ctx, app, key);
    super::debian_contents::point_click(ctx, app, 1200., 950., rect);
    for _ in 0..3 {
        frame(ctx, app, 1200., 950., vec![]);
    }
}

#[test]
fn actual_cable_dropdown_click_changes_only_that_end_and_invalidates_feedback() {
    let id = "ccna-device-path-patching";
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    tap(&ctx, &mut app, "example");
    tap(&ctx, &mut app, "check");
    tap(&ctx, &mut app, "workstation-b");
    tap(&ctx, &mut app, "workstation-b-choice-5");
    let State::DevicePath { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(
        simulation.practice.cables[0].b,
        dlms_learning_lab::switching::Jack::Switch {
            switch: "office".into(),
            port: "p1".into()
        }
    );
    assert!(!simulation.practice.checked);
    assert_ne!(simulation.practice.cables, simulation.assignment.cables);
    frame(
        &ctx,
        &mut app,
        1200.,
        950.,
        vec![key(egui::Key::Tab, false)],
    );
    assert!(ctx.memory(|memory| memory.focused()).is_some());
    tap(&ctx, &mut app, "check");
    let State::DevicePath { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    let definition = work::source(&app.scenarios[app.selected.unwrap()]).unwrap();
    assert!(
        simulation
            .practice
            .checks(&definition.example)
            .iter()
            .any(|row| !row.0 && row.1.contains("jack"))
    );
}
