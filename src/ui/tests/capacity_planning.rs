use super::*;
use dlms_learning_lab::capacity_planning_workspace as work;
use dlms_learning_lab::engine::Goal;
const IDS: [&str; 2] = [
    "lpic2-capacity-trend-plan",
    "lpic2-capacity-import-handover",
];
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let State::CapacityPlanning { simulation } =
        &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
#[test]
fn planning_views_reflow_fields_controls_and_graphs_at_supported_scales() {
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
            for v in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                view(&mut app, id, v);
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                assert_eq!(app.metrics.lesson_tabs.len(), 3);
                assert!(!app.metrics.completion);
                let m = &app.metrics.capacity_planning;
                if v == teaching::View::Introduction {
                    assert!(m.fields.is_empty());
                    continue;
                }
                assert_eq!(m.fields.len(), 2);
                let graph = m.graph.unwrap();
                let clip = app.metrics.main_viewport.unwrap();
                assert!(
                    graph.left() >= clip.left() - 1. && graph.right() <= clip.right() + 1.,
                    "{id}/{scale}: {graph:?}/{clip:?}"
                );
                for (key, r, _, clip) in &m.fields {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "{id}/{v:?}/{scale}/{key}: {r:?}/{clip:?}"
                    );
                }
                for (key, r, clip) in &m.controls {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "{id}/{v:?}/{scale}/{key}: {r:?}/{clip:?}"
                    );
                }
                let field = m.fields[0].1;
                let primary = m.controls.iter().find(|r| r.0 == "test").unwrap().1;
                assert!((field.height() - primary.height()).abs() < 1.);
                assert!((field.height() - m.fields[1].1.height()).abs() < 1.);
                assert!(m.result.is_none());
            }
        }
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        frame(&ctx, &mut app, 1200., 950., vec![]);
        assert!(app.metrics.lesson_tabs.is_empty());
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[index]
        ));
    }
}
fn key_event(key: egui::Key, pressed: bool, command: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: command,
            command,
            ..Default::default()
        },
    }
}
#[test]
fn actual_field_edit_clears_success_and_enter_tab_use_native_focus() {
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let source = app.scenarios[index].clone();
    view(&mut app, id, teaching::View::Assignment);
    for op in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(id).unwrap().state,
            &source.goal,
            op,
        )
        .unwrap();
    }
    for _ in 0..3 {
        frame(&ctx, &mut app, 1920., 2200., vec![]);
    }
    assert!(app.metrics.completion);
    let input = app
        .metrics
        .capacity_planning
        .fields
        .iter()
        .find(|r| r.0 == "reserve")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(input));
    frame(
        &ctx,
        &mut app,
        1920.,
        2200.,
        vec![
            key_event(egui::Key::A, true, true),
            key_event(egui::Key::A, false, true),
            egui::Event::Text("0".into()),
        ],
    );
    assert!(!app.metrics.completion);
    assert!(app.metrics.capacity_planning.result.is_none());
    assert!(!progress::current_solved(
        &app.progress.attempts[id],
        &source
    ));
    let State::CapacityPlanning { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.inputs.reserve_kib, "0");
    assert!(!simulation.assignment.checked);
    frame(
        &ctx,
        &mut app,
        1920.,
        2200.,
        vec![
            key_event(egui::Key::Tab, true, false),
            key_event(egui::Key::Tab, false, false),
        ],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(input));
    let input = app
        .metrics
        .capacity_planning
        .fields
        .iter()
        .find(|r| r.0 == "order-day")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(input));
    frame(
        &ctx,
        &mut app,
        1920.,
        2200.,
        vec![
            key_event(egui::Key::Enter, true, false),
            key_event(egui::Key::Enter, false, false),
        ],
    );
    assert!(app.metrics.capacity_planning.result.unwrap().1 > 0);
    assert!(!app.metrics.completion);
    let input = app
        .metrics
        .capacity_planning
        .fields
        .iter()
        .find(|r| r.0 == "reserve")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(input));
    frame(
        &ctx,
        &mut app,
        1920.,
        2200.,
        vec![
            key_event(egui::Key::A, true, true),
            key_event(egui::Key::A, false, true),
            egui::Event::Text("9".repeat(1000)),
        ],
    );
    let State::CapacityPlanning { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.inputs.reserve_kib.len(), 24);
    assert!(!simulation.assignment.checked);
}
#[test]
fn painted_overall_budget_failure_and_correct_individual_cases_are_unambiguous() {
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let source = app.scenarios[index].clone();
    view(&mut app, id, teaching::View::Assignment);
    for op in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(id).unwrap().state,
            &source.goal,
            op,
        )
        .unwrap();
    }
    let Goal::CapacityPlanning { definition } = &source.goal else {
        panic!()
    };
    let State::CapacityPlanning { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
        .assignment
        .apply(
            &definition.initial,
            &work::Action::Offer {
                id: "costly".into(),
            },
        )
        .unwrap();
    simulation
        .assignment
        .apply(&definition.initial, &work::Action::Test)
        .unwrap();
    for _ in 0..3 {
        frame(&ctx, &mut app, 1920., 2400., vec![]);
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920., 2400.),
            )),
            ..Default::default()
        },
        |ctx| app.render(ctx),
    );
    fn visit(shape: &egui::Shape, words: &mut Vec<String>) {
        match shape {
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    visit(shape, words)
                }
            }
            egui::Shape::Text(text) => words.push(text.galley.text().into()),
            _ => {}
        }
    }
    let mut words = vec![];
    for shape in output.shapes {
        visit(&shape.shape, &mut words);
    }
    let text = words.join("\n");
    assert!(text.contains("Plan needs attention"));
    assert!(text.contains("A passing workload alone does not pass the plan"));
    assert!(text.contains("Needs attention · Budget:"));
    for id in ["steady", "busier", "retention"] {
        assert!(
            text.contains(&format!(
                "Meets requirement · Case {id}: required reserve is retained"
            )),
            "Missing painted case result: {id}"
        );
    }
    let result = app.metrics.capacity_planning.result.unwrap();
    assert_eq!((result.0, result.1), (7, 1));
    assert!(!app.metrics.completion);
}
