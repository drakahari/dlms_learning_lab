//! Capability fixture only; no teaching/native/novice acceptance is implied.
use super::*;
use dlms_learning_lab::source_install_workspace as work;
fn opened() -> (egui::Context, LabApp, String) {
    let def: work::Definition = serde_json::from_str(include_str!(
        "../../../tests/fixtures/source-install-definition-v1.json"
    ))
    .unwrap();
    let mut solution = def
        .initial
        .reference
        .iter()
        .map(|c| c.0.clone())
        .collect::<Vec<_>>();
    solution.push("source-action {\"kind\":\"check\"}".into());
    let data = serde_json::json!({"schema_version":1,"scenarios":[{"id":"source-ui-capability-fixture","version":1,"track":"LPIC-2","topic":"Supplied source installation","unit":"source-ui-model-fixture","title":"Supplied project workspace fixture","summary":"UI capability fixture, not registered curriculum.","objective":"Repair the supplied configuration, rebuild and stage the required program without replacing the existing package.","difficulty":"Intermediate","minutes":25,"steps":["Inspect the supplied project and its requested settings.","Configure, test and stage the repaired package."],"hints":["Rebuilding does not replace a staged copy."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical fixture only. No independent novice acceptance.","simplifications":"Finite supplied program, typed artifacts and virtual files; no host tools.","initial":{"kind":"SourceInstall","simulation":work::Session::new(&def)},"goal":{"kind":"SourceInstall","definition":def},"solution":solution,"prerequisites":[]}]});
    let source = pack::load(&serde_json::to_string(&data).unwrap())
        .unwrap()
        .remove(0);
    let id = source.id.clone();
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    let i = app.scenarios.len();
    app.scenarios.push(source);
    app.launch(i);
    draw(&ctx, &mut app);
    (ctx, app, id)
}
fn session<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::SourceInstall { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
#[test]
fn source_project_controls_reflow_long_populated_fields_across_widths_scales_and_themes() {
    let (ctx, mut app, id) = opened();
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
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            session(&mut app, &id).view = view;
            app.lesson_scroll_reset = true;
            session(&mut app, &id).assignment.input = "long-pending-command-".repeat(40);
            session(&mut app, &id).practice.input = "long-pending-command-".repeat(40);
            for _ in 0..3 {
                frame(&ctx, &mut app, width, 1080., vec![]);
            }
            let metrics = &app.metrics.source_install;
            if view == teaching::View::Introduction {
                assert!(!metrics.controls.iter().any(|r| r.0 == "command"));
                continue;
            }
            for key in [
                "command",
                "run",
                "check",
                "state-Configured prefix",
                "state-Current directory",
            ] {
                let (_, rect, clip) = metrics.controls.iter().find(|r| r.0 == key).unwrap();
                assert!(
                    rect.width() > 0.
                        && rect.left() >= clip.left() - 1.
                        && rect.right() <= clip.right() + 1.,
                    "{view:?}/{width}/{scale}/{key}: {rect:?}/{clip:?}"
                );
            }
            let command = metrics
                .controls
                .iter()
                .find(|r| r.0 == "command")
                .unwrap()
                .1;
            let run = metrics.controls.iter().find(|r| r.0 == "run").unwrap().1;
            assert!(
                (command.height() - run.height()).abs() < 1.,
                "Shared font-derived control height"
            );
        }
    }
}
#[test]
fn inserting_worked_command_is_inert_and_keyboard_enter_runs_only_that_practice_draft() {
    let (ctx, mut app, id) = opened();
    session(&mut app, &id).view = teaching::View::Example;
    app.lesson_scroll_reset = true;
    for _ in 0..3 {
        frame(&ctx, &mut app, 2200., 1080., vec![]);
    }
    let prior = session(&mut app, &id).clone();
    let insert = app
        .metrics
        .source_install
        .controls
        .iter()
        .find(|r| r.0 == "insert")
        .unwrap()
        .1;
    click(&ctx, &mut app, insert);
    assert_eq!(
        session(&mut app, &id).practice.history,
        prior.practice.history
    );
    assert!(session(&mut app, &id).practice.last_run.is_none());
    assert_eq!(session(&mut app, &id).assignment, prior.assignment);
    assert!(!session(&mut app, &id).practice.input.is_empty());
    let input = app
        .metrics
        .source_install
        .controls
        .iter()
        .find(|r| r.0 == "command")
        .unwrap()
        .1;
    click(&ctx, &mut app, input);
    let modifier = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let select = |pressed| egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: modifier,
    };
    frame(
        &ctx,
        &mut app,
        2200.,
        1080.,
        vec![select(true), select(false), egui::Event::Text("pwd".into())],
    );
    frame(
        &ctx,
        &mut app,
        2200.,
        1080.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut app);
    let state = session(&mut app, &id);
    assert_eq!(state.practice.history.last().unwrap().0, "pwd");
    assert_eq!(state.assignment, prior.assignment);
    assert!(!app.metrics.completion);
}
#[test]
fn challenge_hides_worked_commands_and_global_failure_never_becomes_a_case_success_summary() {
    let (ctx, mut app, id) = opened();
    let i = app.selected.unwrap();
    let source = app.scenarios[i].clone();
    app.progress.attempts.get_mut(&id).unwrap().challenge = true;
    draw(&ctx, &mut app);
    assert!(
        !app.metrics.source_install.controls.iter().any(|r| matches!(
            r.0.as_str(),
            "example" | "introduction" | "worked-menu" | "insert"
        ))
    );
    for command in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(&id).unwrap().state,
            &source.goal,
            command,
        )
        .unwrap();
    }
    app.completion_index.forget(&id);
    draw(&ctx, &mut app);
    assert!(app.metrics.completion);
    assert_eq!(
        app.metrics.source_install.summary.as_deref(),
        Some("All project requirements met")
    );
    engine::execute_for_goal(
        &mut app.progress.attempts.get_mut(&id).unwrap().state,
        &source.goal,
        "./configure --prefix=/opt/wrong --with-greeting=Wrong",
    )
    .unwrap();
    engine::execute_for_goal(
        &mut app.progress.attempts.get_mut(&id).unwrap().state,
        &source.goal,
        "source-action {\"kind\":\"check\"}",
    )
    .unwrap();
    app.completion_index.forget(&id);
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    let summary = app.metrics.source_install.summary.as_deref().unwrap();
    assert!(summary.starts_with("Needs attention"));
    let checks = engine::checks(&app.progress.attempts[&id].state, &source.goal);
    assert!(
        checks
            .iter()
            .any(|r| r.0 && r.1.starts_with("Program case"))
    );
    assert!(
        checks
            .iter()
            .any(|r| !r.0 && r.1.starts_with("Configured project"))
    );
}

#[test]
fn authored_worked_commands_are_full_width_wrapped_and_keep_complete_values() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    for id in [
        "lpic2-source-project-build",
        "lpic2-source-stage-install",
        "lpic2-source-stale-handover",
    ] {
        app.open_source_install_capture(id);
        for (width, scale) in [(2200., 1.), (946., 2.)] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.lesson_scroll_reset = true;
            for _ in 0..3 {
                frame(&ctx, &mut app, width, 1080., vec![]);
            }
            let values = app
                .metrics
                .source_install
                .controls
                .iter()
                .filter(|r| r.0.starts_with("worked-value-"))
                .collect::<Vec<_>>();
            assert!(
                values.len() >= 7,
                "Actual authored worked content must reach the opt-in renderer"
            );
            for (_, rect, clip) in &values {
                assert!(
                    rect.left() >= clip.left() - 1. && rect.right() <= clip.right() + 1.,
                    "{id}/{scale}: {rect:?}/{clip:?}"
                );
            }
            if scale == 2. {
                let line = 14.;
                assert!(
                    values.iter().any(|(_, r, _)| r.height() > line * 1.5),
                    "Long commands must wrap rather than require a horizontal scrollbar"
                );
            }
        }
    }
}
