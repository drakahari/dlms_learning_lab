use super::*;
use dlms_learning_lab::{engine::Goal, localization_workspace as work};
const IDS: [&str; 2] = [
    "lpic1-locale-report-handover",
    "lpic1-encoding-import-repair",
];
fn opened(id: &str) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    assert_eq!((app.scenarios.len(), app.units.len()), (421, 162));
    app.launch(app.scenarios.iter().position(|s| s.id == id).unwrap());
    draw(&ctx, &mut app);
    (ctx, app, id.into())
}
fn session<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::Localization { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn control(app: &LabApp, key: &str) -> egui::Rect {
    app.metrics
        .localization
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice={:?}", app.notice))
        .1
}
fn has(app: &LabApp, key: &str) -> bool {
    app.metrics.localization.controls.iter().any(|r| r.0 == key)
}
#[test]
fn actual_two_lessons_keep_subject_tools_and_hide_worked_controls_in_assignment_and_challenge() {
    let (ctx, mut app, _) = opened(IDS[0]);
    for (id, reports) in [(IDS[0], true), (IDS[1], false)] {
        app.launch(app.scenarios.iter().position(|s| s.id == id).unwrap());
        draw(&ctx, &mut app);
        assert_eq!(app.metrics.lesson_tabs.len(), 3);
        assert!(has(&app, "worked-value-0"));
        session(&mut app, id).view = teaching::View::Example;
        draw(&ctx, &mut app);
        assert!(has(&app, "insert"));
        assert_eq!(has(&app, "configuration"), reports);
        assert_eq!(has(&app, "run-report"), reports);
        assert_eq!(has(&app, "fill-worked"), reports);
        if !reports {
            assert!(has(&app, "document-menu") && has(&app, "document-body"));
        }
        session(&mut app, id).view = teaching::View::Assignment;
        draw(&ctx, &mut app);
        assert!(!has(&app, "insert") && !has(&app, "fill-worked") && !has(&app, "worked-menu"));
        assert!(session(&mut app, id).assignment.input.is_empty());
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut app);
        assert!(app.metrics.lesson_tabs.is_empty());
        assert!(has(&app, "command"));
        assert!(!has(&app, "insert") && !has(&app, "fill-worked"));
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[app.selected.unwrap()]
        ));
    }
}
#[test]
fn worked_fill_and_command_insertion_are_inert_and_return_runs_only_practice() {
    let (ctx, mut app, id) = opened(IDS[0]);
    session(&mut app, &id).view = teaching::View::Example;
    draw(&ctx, &mut app);
    let independent = session(&mut app, &id).assignment.clone();
    let i = app.selected.unwrap();
    let Goal::Localization { definition } = &app.scenarios[i].goal else {
        panic!()
    };
    let task = definition.example.clone();
    let saved = session(&mut app, &id)
        .practice
        .world(&task)
        .unwrap()
        .saved_locale()
        .to_string();
    let target = control(&app, "fill-worked");
    click(&ctx, &mut app, target);
    assert!(session(&mut app, &id).practice.history.is_empty());
    assert_ne!(session(&mut app, &id).practice.configuration, saved);
    assert_eq!(
        session(&mut app, &id)
            .practice
            .world(&task)
            .unwrap()
            .saved_locale(),
        saved
    );
    let target = control(&app, "save-config");
    click(&ctx, &mut app, target);
    assert_eq!(session(&mut app, &id).practice.history.len(), 1);
    let before = session(&mut app, &id)
        .practice
        .world(&task)
        .unwrap()
        .process()
        .clone();
    assert_eq!(before, task.initial.process);
    let target = control(&app, "insert");
    click(&ctx, &mut app, target);
    assert_eq!(session(&mut app, &id).practice.history.len(), 1);
    assert_eq!(
        session(&mut app, &id).practice.input,
        "locale -k decimal_point"
    );
    let target = control(&app, "command");
    click(&ctx, &mut app, target);
    frame(
        &ctx,
        &mut app,
        1920.,
        1080.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut app);
    assert_eq!(
        session(&mut app, &id)
            .practice
            .last_output(&task)
            .unwrap()
            .unwrap()
            .unwrap()
            .stdout,
        b"decimal_point=\",\"\n"
    );
    assert_eq!(session(&mut app, &id).assignment, independent);
    assert!(!session(&mut app, &id).assignment.checked);
}
#[test]
fn unsaved_configuration_is_a_visible_overall_failure_even_with_all_report_cases_passing() {
    let (ctx, mut app, id) = opened(IDS[0]);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Localization { definition } = &source.goal else {
        panic!()
    };
    let draft = &mut session(&mut app, &id).assignment;
    for action in &definition.initial.reference {
        draft
            .perform(&definition.initial, action.clone())
            .unwrap()
            .unwrap();
    }
    draft.check(&definition.initial).unwrap();
    assert!(progress::current_solved(
        &app.progress.attempts[&id],
        &source
    ));
    session(&mut app, &id)
        .assignment
        .edit_configuration("LC_ALL=C\n".into())
        .unwrap();
    session(&mut app, &id)
        .assignment
        .check(&definition.initial)
        .unwrap();
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    assert!(
        app.metrics
            .localization
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
    let rows = session(&mut app, &id)
        .assignment
        .checks(&definition.initial)
        .unwrap();
    assert!(!rows[0].correct && rows[1..].iter().all(|r| r.correct));
    assert!(has(&app, "passing-checks"));
}
#[test]
fn negative_conversion_status_and_partial_saved_file_invalidate_completion() {
    let (ctx, mut app, id) = opened(IDS[1]);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Localization { definition } = &source.goal else {
        panic!()
    };
    for raw in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(&id).unwrap().state,
            &source.goal,
            raw,
        )
        .unwrap();
    }
    assert!(progress::current_solved(
        &app.progress.attempts[&id],
        &source
    ));
    let draft = &mut session(&mut app, &id).assignment;
    draft.input =
        "iconv -f LATIN1 -t ASCII -o /work/roster-utf8.txt /work/roster-latin1.txt".into();
    assert_eq!(draft.run(&definition.initial).unwrap().unwrap().status, 1);
    draft.check(&definition.initial).unwrap();
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    assert!(
        app.metrics
            .localization
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
    assert_eq!(
        session(&mut app, &id)
            .assignment
            .world(&definition.initial)
            .unwrap()
            .files()
            .read("/work/roster-utf8.txt")
            .unwrap(),
        b"Caf"
    );
}
#[test]
fn populated_singleline_fields_actions_and_scrollbars_preserve_geometry_at_large_text() {
    let (ctx, mut app, id) = opened(IDS[1]);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Localization { definition } = &source.goal else {
        panic!()
    };
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
        session(&mut app, &id).assignment.input =
            "iconv -f ISO-8859-1 -t UTF-8 -o /work/roster-utf8.txt /work/roster-latin1.txt".into();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        let input = control(&app, "command");
        let run = control(&app, "run");
        assert!(input.is_finite() && input.width() > 0.);
        assert!(input.height() >= run.height() - 1.);
        assert!(run.right() <= app.metrics.main_scroll_region.unwrap().right() + 1.);
        session(&mut app, &id).assignment.input = "cat /work/notes-utf8.txt".into();
        session(&mut app, &id)
            .assignment
            .run(&definition.initial)
            .unwrap()
            .unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        assert_eq!(control(&app, "command"), input);
        assert_eq!(control(&app, "run"), run);
        assert!((ctx.zoom_factor() - scale).abs() < 0.01);
    }
}
