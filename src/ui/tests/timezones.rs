use super::*;
use dlms_learning_lab::{engine::Goal, timezone_workspace as work};
const IDS: [&str; 2] = [
    "lpic1-timezone-system-display",
    "lpic1-timezone-process-override",
];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    assert_eq!((app.scenarios.len(), app.units.len()), (421, 162));
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut app);
    (ctx, app, IDS[n].into())
}
fn session<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::Timezone { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn control(app: &LabApp, key: &str) -> egui::Rect {
    app.metrics
        .timezone
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice={:?}", app.notice))
        .1
}
fn has(app: &LabApp, key: &str) -> bool {
    app.metrics.timezone.controls.iter().any(|r| r.0 == key)
}
#[test]
fn actual_registered_lessons_keep_teaching_and_independent_terminal_scopes() {
    let (ctx, mut app, _) = opened(0);
    for id in IDS {
        app.launch(app.scenarios.iter().position(|s| s.id == id).unwrap());
        draw(&ctx, &mut app);
        assert_eq!(app.metrics.lesson_tabs.len(), 3);
        assert!(has(&app, "worked-value-0"));
        assert!(!has(&app, "command"));
        session(&mut app, id).view = teaching::View::Example;
        draw(&ctx, &mut app);
        assert!(has(&app, "insert") && has(&app, "worked-menu") && has(&app, "command"));
        session(&mut app, id).view = teaching::View::Assignment;
        draw(&ctx, &mut app);
        assert!(!has(&app, "insert") && !has(&app, "worked-menu") && !has(&app, "worked-value-0"));
        assert!(session(&mut app, id).assignment.input.is_empty());
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut app);
        assert!(app.metrics.lesson_tabs.is_empty());
        assert!(has(&app, "command"));
        assert!(!has(&app, "insert") && !has(&app, "worked-menu"));
    }
}
#[test]
fn command_insertion_is_inert_and_keyboard_return_runs_only_the_example() {
    let (ctx, mut app, id) = opened(0);
    session(&mut app, &id).view = teaching::View::Example;
    draw(&ctx, &mut app);
    let independent = session(&mut app, &id).assignment.clone();
    let r = control(&app, "insert");
    click(&ctx, &mut app, r);
    assert!(session(&mut app, &id).practice.history.is_empty());
    assert_eq!(
        session(&mut app, &id).practice.input,
        "timedatectl show -p Timezone --value"
    );
    let r = control(&app, "command");
    click(&ctx, &mut app, r);
    frame(
        &ctx,
        &mut app,
        1920.,
        1080.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut app);
    let i = app.selected.unwrap();
    let Goal::Timezone { definition } = &app.scenarios[i].goal else {
        panic!()
    };
    let t = definition.example.clone();
    assert_eq!(
        session(&mut app, &id)
            .practice
            .last_output(&t)
            .unwrap()
            .unwrap()
            .unwrap()
            .stdout,
        "UTC\n"
    );
    assert_eq!(session(&mut app, &id).assignment, independent);
    assert!(!session(&mut app, &id).assignment.checked);
}
#[test]
fn partial_cases_are_named_and_rebroken_success_is_not_reported_complete() {
    let (ctx, mut app, id) = opened(0);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Timezone { definition } = &source.goal else {
        panic!()
    };
    let draft = &mut session(&mut app, &id).assignment;
    draft.input = "export TZ=Europe/Berlin".into();
    draft.run(&definition.initial).unwrap().unwrap();
    draft.check(&definition.initial).unwrap();
    draw(&ctx, &mut app);
    assert!(
        app.metrics
            .timezone
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
    assert!(!app.metrics.completion);
    assert!(has(&app, "feedback-1"));
    assert!(
        session(&mut app, &id)
            .assignment
            .checks(&definition.initial)
            .unwrap()
            .iter()
            .any(|r| r.correct && r.label.starts_with("Current process"))
    );
    session(&mut app, &id).assignment.input = "unset TZ".into();
    session(&mut app, &id)
        .assignment
        .run(&definition.initial)
        .unwrap()
        .unwrap();
    for raw in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(&id).unwrap().state,
            &source.goal,
            raw,
        )
        .unwrap();
    }
    draw(&ctx, &mut app);
    assert!(app.metrics.completion);
    assert_eq!(
        app.metrics.timezone.summary.as_deref(),
        Some("Assignment complete")
    );
    session(&mut app, &id).assignment.input = "timedatectl set-timezone UTC".into();
    session(&mut app, &id)
        .assignment
        .run(&definition.initial)
        .unwrap()
        .unwrap();
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    assert!(app.metrics.timezone.summary.is_none());
    let r = control(&app, "check");
    click(&ctx, &mut app, r);
    assert!(!app.metrics.completion);
    assert!(
        app.metrics
            .timezone
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
}
#[test]
fn hardcoded_today_override_exposes_future_case_failures_and_preserves_correct_rows() {
    let (ctx, mut app, id) = opened(1);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Timezone { definition } = &source.goal else {
        panic!()
    };
    let d = &mut session(&mut app, &id).assignment;
    d.input = "export TZ=Europe/Berlin".into();
    d.run(&definition.initial).unwrap().unwrap();
    d.check(&definition.initial).unwrap();
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    let rows = session(&mut app, &id)
        .assignment
        .checks(&definition.initial)
        .unwrap();
    assert_eq!(rows.iter().filter(|r| !r.correct).count(), 2);
    assert!(
        rows.iter()
            .filter(|r| !r.correct)
            .all(|r| r.label.starts_with("After the supplied zone handover"))
    );
    assert_eq!(
        app.metrics.timezone.summary.as_deref(),
        Some("Needs attention · 2 requirements fail")
    );
    assert!(has(&app, "passing-checks"));
    let ids = app.metrics.timezone.disclosure_ids.clone();
    assert_eq!(ids.len(), 2, "Both failed cases need their own disclosure");
    assert_ne!(
        ids[0], ids[1],
        "Repeated labels must not alias widget state"
    );
    draw(&ctx, &mut app);
    assert_eq!(
        ids, app.metrics.timezone.disclosure_ids,
        "IDs remain stable across frames"
    );
}
#[test]
fn singleline_and_run_alignment_survive_history_scrolling_scaling_and_theme_changes() {
    let (ctx, mut app, id) = opened(0);
    session(&mut app, &id).view = teaching::View::Assignment;
    let source = app.scenarios[app.selected.unwrap()].clone();
    let Goal::Timezone { definition } = &source.goal else {
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
            "timedatectl show --property=Timezone --value".into();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        let mono_height = egui::TextStyle::Monospace.resolve(&ctx.style()).size * 1.4;
        for n in 0..2 {
            assert!(control(&app, &format!("moment-{n}-utc")).height() <= mono_height);
            assert!(control(&app, &format!("moment-{n}-epoch")).height() <= mono_height);
        }
        let field = control(&app, "command");
        let run = control(&app, "run");
        assert!((field.height() - run.height()).abs() < 1.5);
        if (field.center().y - run.center().y).abs() < field.height() {
            assert!((field.center().y - run.center().y).abs() < 1.);
        }
        assert!(run.right() <= app.metrics.main_scroll_region.unwrap().right() + 1.);
        for _ in 0..12 {
            let d = &mut session(&mut app, &id).assignment;
            d.input = "date '+%F %T %z %Z'".into();
            d.run(&definition.initial).unwrap().unwrap();
        }
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        assert_eq!(control(&app, "command"), field);
        assert_eq!(control(&app, "run"), run);
    }
}
