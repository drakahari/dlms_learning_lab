use super::*;
use dlms_learning_lab::engine::Goal;
use dlms_learning_lab::print_queue_workspace as work;
const IDS: [&str; 3] = [
    "lpic1-print-paused-queue",
    "lpic1-print-held-request",
    "lpic1-print-destination-handover",
];
fn opened() -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    assert_eq!((app.scenarios.len(), app.units.len()), (421, 162));
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let id = IDS[0].to_string();
    draw(&ctx, &mut app);
    (ctx, app, id)
}
fn session<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::PrintQueues { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn control(app: &LabApp, key: &str) -> egui::Rect {
    app.metrics
        .print_queues
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
#[test]
fn actual_authored_three_screen_lessons_keep_assignment_independent_and_challenge_unassisted() {
    let (ctx, mut app, _) = opened();
    for id in IDS {
        app.launch(app.scenarios.iter().position(|s| s.id == id).unwrap());
        draw(&ctx, &mut app);
        assert_eq!(app.metrics.lesson_tabs.len(), 3);
        assert!(
            app.metrics
                .print_queues
                .controls
                .iter()
                .any(|r| r.0 == "worked-value-0")
        );
        session(&mut app, id).view = teaching::View::Example;
        draw(&ctx, &mut app);
        assert!(
            app.metrics
                .print_queues
                .controls
                .iter()
                .any(|r| r.0 == "insert")
        );
        session(&mut app, id).view = teaching::View::Assignment;
        draw(&ctx, &mut app);
        assert!(
            !app.metrics
                .print_queues
                .controls
                .iter()
                .any(|r| r.0 == "insert" || r.0 == "worked-menu")
        );
        assert_eq!(session(&mut app, id).assignment.input.0, "");
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut app);
        assert!(app.metrics.lesson_tabs.is_empty());
        assert!(
            app.metrics
                .print_queues
                .controls
                .iter()
                .any(|r| r.0 == "command")
        );
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[app.selected.unwrap()]
        ));
    }
}
#[test]
fn insert_is_inert_and_return_runs_only_the_separate_example() {
    let (ctx, mut app, id) = opened();
    session(&mut app, &id).view = teaching::View::Example;
    draw(&ctx, &mut app);
    let initial = session(&mut app, &id).assignment.clone();
    let target = control(&app, "insert");
    click(&ctx, &mut app, target);
    assert!(session(&mut app, &id).practice.history.is_empty());
    assert!(!session(&mut app, &id).practice.input.0.is_empty());
    let target = control(&app, "command");
    click(&ctx, &mut app, target);
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut app,
        1920.,
        1080.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text("lpstat -t".into()),
        ],
    );
    frame(
        &ctx,
        &mut app,
        1920.,
        1080.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut app);
    assert_eq!(
        session(&mut app, &id)
            .practice
            .last_run
            .as_ref()
            .unwrap()
            .command
            .0,
        "lpstat -t"
    );
    assert_eq!(session(&mut app, &id).assignment, initial);
    assert!(!session(&mut app, &id).assignment.checked);
}
#[test]
fn completed_sample_cases_do_not_hide_an_overall_wrong_document_failure() {
    let (ctx, mut app, id) = opened();
    session(&mut app, &id).view = teaching::View::Assignment;
    let s = app.scenarios[app.selected.unwrap()].clone();
    let Goal::PrintQueues { definition } = &s.goal else {
        panic!()
    };
    for raw in &s.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(&id).unwrap().state,
            &s.goal,
            raw,
        )
        .unwrap();
    }
    assert!(progress::current_solved(&app.progress.attempts[&id], &s));
    engine::execute_for_goal(
        &mut app.progress.attempts.get_mut(&id).unwrap().state,
        &s.goal,
        "lp stock-labels.txt",
    )
    .unwrap();
    engine::execute_for_goal(
        &mut app.progress.attempts.get_mut(&id).unwrap().state,
        &s.goal,
        "@print-check",
    )
    .unwrap();
    draw(&ctx, &mut app);
    let rows = session(&mut app, &id)
        .assignment
        .checks(&definition.initial)
        .unwrap();
    assert!(
        rows.iter()
            .filter(|r| r.label.starts_with("Fresh request case"))
            .all(|r| r.correct)
    );
    assert!(!progress::current_solved(&app.progress.attempts[&id], &s));
    assert!(
        app.metrics
            .print_queues
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
    assert!(
        app.metrics
            .print_queues
            .controls
            .iter()
            .any(|r| r.0 == "passing-checks")
    );
}
#[test]
fn command_labels_and_actions_reflow_with_populated_fields_and_do_not_shrink_type() {
    let (ctx, mut app, id) = opened();
    session(&mut app, &id).view = teaching::View::Assignment;
    for (width, scale, light) in [
        (2200., 1., false),
        (2200., 1.5, false),
        (946., 2., true),
        (640., 2., true),
    ] {
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
        session(&mut app, &id).assignment.input.0 = "lp -d office -n 2 /work/agenda.txt".into();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        let command = control(&app, "command");
        let run = control(&app, "run");
        assert!(command.is_finite() && command.width() > 0.);
        assert!(command.height() >= run.height() - 1.);
        assert!(run.right() <= app.metrics.main_scroll_region.unwrap().right() + 1.);
        assert!((ctx.zoom_factor() - scale).abs() < 0.01);
        let task = match &app.scenarios[app.selected.unwrap()].goal {
            Goal::PrintQueues { definition } => definition.initial.clone(),
            _ => panic!(),
        };
        let draft = &mut session(&mut app, &id).assignment;
        draft.input.0 = "lpstat -t".into();
        draft.run(&task).unwrap().unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        assert_eq!(control(&app, "command"), command);
        assert_eq!(control(&app, "run"), run);
    }
}
