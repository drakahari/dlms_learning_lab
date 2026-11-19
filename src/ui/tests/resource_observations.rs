use super::*;
const IDS: [&str; 3] = [
    "lpic2-resource-cpu-intervals",
    "lpic2-resource-swap-activity",
    "lpic2-resource-restart-handover",
];
fn view(app: &mut LabApp, id: &str, v: teaching::View) {
    let State::ResourceObservations { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    app.lesson_scroll_reset = true;
}
#[test]
fn recorded_host_controls_reflow_and_independent_fields_start_unanswered() {
    for id in IDS {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
        let engine::Goal::ResourceObservations { definition } = &app.scenarios[index].goal else {
            panic!()
        };
        let rules = definition.initial.rules.clone();
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
                let m = &app.metrics.resource_observation;
                if v == teaching::View::Introduction {
                    assert!(m.fields.is_empty());
                    continue;
                }
                for (key, r, _, clip) in &m.fields {
                    assert!(
                        r.width() > 0.
                            && r.left() >= clip.left() - 1.
                            && r.right() <= clip.right() + 1.,
                        "{id} {v:?} {scale}: {key} {r:?}/{clip:?}"
                    );
                }
                let attach: Vec<_> = m
                    .controls
                    .iter()
                    .filter(|r| r.0.starts_with("attach-"))
                    .map(|r| r.1)
                    .collect();
                assert!(
                    attach
                        .iter()
                        .all(|r| (r.left() - attach[0].left()).abs() < 1.),
                    "Related attachment buttons must share their column at{scale}: {attach:?}"
                );
                let heights: Vec<_> = m.fields.iter().map(|r| r.1.height()).collect();
                assert!(heights.iter().all(|h| (h - heights[0]).abs() < 1.));
                for (key, r, clip) in &m.controls {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "{id} {key} {scale}: {r:?}/{clip:?}"
                    );
                }
            }
        }
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        frame(&ctx, &mut app, 1200., 950., vec![]);
        assert!(app.metrics.lesson_tabs.is_empty());
        let State::ResourceObservations { simulation } = &app.progress.attempts[id].state else {
            panic!()
        };
        assert!(
            simulation
                .assignment
                .claims
                .iter()
                .all(|c| c.value.is_empty() && c.evidence.is_none())
        );
        assert_eq!(simulation.assignment.claims.len(), rules.len());
    }
}
#[test]
fn real_command_enter_and_tab_navigation_run_the_simulator_without_credit() {
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    view(&mut app, id, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1920., 1400., vec![]);
    }
    let input = app
        .metrics
        .resource_observation
        .fields
        .iter()
        .find(|r| r.0 == "command")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(input));
    frame(
        &ctx,
        &mut app,
        1920.,
        1400.,
        vec![egui::Event::Text("vmstat 1 3".into())],
    );
    let key = |key, pressed| egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    };
    frame(
        &ctx,
        &mut app,
        1920.,
        1400.,
        vec![key(egui::Key::Enter, true), key(egui::Key::Enter, false)],
    );
    let State::ResourceObservations { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(
        simulation.assignment.history.last().unwrap().command,
        "vmstat 1 3"
    );
    assert!(!simulation.assignment.checked);
    assert!(!progress::current_solved(
        &app.progress.attempts[id],
        &app.scenarios[index]
    ));
    ctx.memory_mut(|m| m.request_focus(input));
    frame(
        &ctx,
        &mut app,
        1920.,
        1400.,
        vec![key(egui::Key::Tab, true), key(egui::Key::Tab, false)],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(input));
}
#[test]
fn actual_value_edit_removes_current_success_and_keeps_other_report_rows() {
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    view(&mut app, id, teaching::View::Assignment);
    let s = app.scenarios[index].clone();
    for op in &s.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(id).unwrap().state,
            &s.goal,
            op,
        )
        .unwrap();
    }
    for _ in 0..3 {
        frame(&ctx, &mut app, 1920., 1500., vec![]);
    }
    assert!(app.metrics.completion);
    let input = app
        .metrics
        .resource_observation
        .fields
        .iter()
        .find(|r| r.0 == "peak-work")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(input));
    let mut events: Vec<_> = [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect();
    events.push(egui::Event::Text("999".into()));
    frame(&ctx, &mut app, 1920., 1500., events);
    assert!(!app.metrics.completion);
    assert!(!progress::current_solved(&app.progress.attempts[id], &s));
    let State::ResourceObservations { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert!(!simulation.assignment.checked);
    assert_eq!(simulation.assignment.claims[0].value, "999");
    let engine::Goal::ResourceObservations { definition } = &s.goal else {
        panic!()
    };
    let checks = definition.initial.check(&simulation.assignment);
    assert!(!checks[0].0 && checks.iter().skip(1).all(|r| r.0));
}

#[test]
fn observation_popup_scrolls_within_its_actual_columns_to_the_last_sample() {
    let id = IDS[2];
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    view(&mut app, id, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1920., 1400., vec![]);
    }
    let picker = app
        .metrics
        .resource_observation
        .controls
        .iter()
        .find(|r| r.0 == "observation")
        .unwrap()
        .1;
    click(&ctx, &mut app, picker);
    let mut selected = false;
    for _ in 0..20 {
        let (_, row, clip) = app
            .metrics
            .resource_observation
            .choices
            .iter()
            .find(|r| r.0 == "observation-6")
            .unwrap()
            .clone();
        let visible = clip.intersect(ctx.content_rect());
        assert!(
            visible.center().x >= row.left() && visible.center().x <= row.right(),
            "Scrolling must target the popup, not its inherited window clip: {row:?}/{clip:?}"
        );
        if smoke::smoke_control_visible(row, visible, ctx.pixels_per_point()) {
            click(&ctx, &mut app, row);
            selected = true;
            break;
        }
        frame(
            &ctx,
            &mut app,
            1920.,
            1400.,
            vec![
                egui::Event::PointerMoved(visible.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -100.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..5 {
            frame(&ctx, &mut app, 1920., 1400., vec![]);
        }
    }
    assert!(
        selected,
        "Last recorded observation must be reachable by native scrolling"
    );
    let State::ResourceObservations { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.cursor, 6);
    assert!(!simulation.assignment.checked);
}
