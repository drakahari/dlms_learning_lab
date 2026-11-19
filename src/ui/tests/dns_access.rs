//! Native capability checks; authored lessons and rendered acceptance are separate.
use super::*;
use dlms_learning_lab::dns_access_workspace::{self as work, Action};
#[path = "../../../tests/support/dns_access_task.rs"]
mod fixture;
fn opened() -> (egui::Context, LabApp, String) {
    let def = fixture::definition();
    let mut solution = def
        .initial
        .reference
        .iter()
        .map(|a| format!("dns-policy {}", serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    solution.push("dns-check".into());
    let value = serde_json::json!({"schema_version":1,"scenarios":[{"id":"dns-ui-capability-fixture","version":1,"track":"LPIC-2","topic":"DNS access boundaries","title":"Verify a supplied DNS policy","summary":"Technical fixture only, not authored curriculum.","objective":"Keep public DNS while restricting recursive/cache clients and synchronizing only the approved secondary.","difficulty":"Intermediate","minutes":25,"steps":["Inspect the supplied client ranges and saved/live rules.","Edit, save, reload and test the current outcomes."],"hints":["The cache has its own client access list."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical capability fixture, not independent novice acceptance.","simplifications":"Positive IPv4 ACLs and supplied DNS data; no real server or network.","initial":{"kind":"DnsAccess","simulation":work::Session::new(&def).unwrap()},"goal":{"kind":"DnsAccess","definition":def},"solution":solution,"prerequisites":[]}]});
    let source = pack::load(&serde_json::to_string(&value).unwrap())
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
fn state<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::DnsAccess { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn control(app: &LabApp, key: &str) -> egui::Rect {
    app.metrics
        .dns_access
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
#[test]
fn editor_viewport_keeps_save_rows_stable_and_controls_reflow_without_shrinking_type() {
    let (ctx, mut app, id) = opened();
    state(&mut app, &id).view = teaching::View::Assignment;
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
        state(&mut app, &id)
            .assignment
            .set_editor("# short pending config".into())
            .unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        let save = control(&app, "save");
        let reload = control(&app, "reload");
        let viewport = control(&app, "editor-viewport");
        state(&mut app, &id)
            .assignment
            .set_editor(format!(
                "{}{}",
                "# long pending configuration line\n".repeat(180),
                fixture::alternate_config()
            ))
            .unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, width, 1080., vec![]);
        }
        assert!((control(&app, "save").top() - save.top()).abs() < 1.);
        assert!((control(&app, "reload").top() - reload.top()).abs() < 1.);
        assert_eq!(
            control(&app, "editor-viewport"),
            viewport,
            "{width}/{scale}/{light}: editor viewport shifted after internal scrolling"
        );
        assert_eq!(
            control(&app, "save"),
            save,
            "{width}/{scale}: Save must retain its complete geometry"
        );
        assert_eq!(
            control(&app, "reload"),
            reload,
            "{width}/{scale}: Reload must retain its complete geometry"
        );
        assert!((save.height() - reload.height()).abs() < 1.);
        for key in ["save", "reload", "query", "check"] {
            let (_, r, clip) = app
                .metrics
                .dns_access
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap();
            assert!(
                r.width() > 0. && r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                "{width}/{scale}/{key}: {r:?}/{clip:?}"
            );
        }
    }
}
#[test]
fn fill_worked_configuration_is_inert_and_keyboard_runs_only_the_practice_query() {
    let (ctx, mut app, id) = opened();
    state(&mut app, &id).view = teaching::View::Example;
    app.lesson_scroll_reset = true;
    for _ in 0..3 {
        frame(&ctx, &mut app, 2200., 1080., vec![]);
    }
    let before = state(&mut app, &id).clone();
    let fill = control(&app, "fill");
    click(&ctx, &mut app, fill);
    assert_eq!(
        state(&mut app, &id).practice.history,
        before.practice.history
    );
    assert_eq!(state(&mut app, &id).assignment, before.assignment);
    assert_ne!(state(&mut app, &id).practice.editor, before.practice.editor);
    state(&mut app, &id).practice.query_client.0 = "192.0.2.7".into();
    state(&mut app, &id).practice.query_name.0 = "www.demo.example".into();
    for _ in 0..3 {
        frame(&ctx, &mut app, 2200., 1080., vec![]);
    }
    let name = app
        .metrics
        .dns_access
        .fields
        .iter()
        .find(|f| f.0 == "query-name")
        .unwrap()
        .1;
    click(&ctx, &mut app, name);
    frame(
        &ctx,
        &mut app,
        2200.,
        1080.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut app);
    assert!(matches!(
        state(&mut app, &id).practice.history.last(),
        Some(Action::Query { .. })
    ));
    assert_eq!(state(&mut app, &id).assignment, before.assignment);
    assert!(!app.metrics.completion);
}
#[test]
fn challenge_hides_worked_content_and_successful_cases_do_not_hide_a_failed_scope() {
    let (ctx, mut app, id) = opened();
    let def = fixture::definition();
    assert_eq!(def.initial, fixture::task());
    let scope = fixture::alternate_config()
        .replace("192.0.2.0/25; 192.0.2.128/25;", "192.0.2.7; 192.0.2.8;");
    let d = &mut state(&mut app, &id).assignment;
    d.set_editor(scope.clone()).unwrap();
    for action in [
        Action::Save {
            config: work::Config(scope),
        },
        Action::Reload {},
        def.initial.reference[2].clone(),
    ] {
        d.run(&def.initial, action).unwrap().unwrap();
    }
    let results = d.check(&def.initial).unwrap();
    assert!(
        results
            .iter()
            .filter(|r| r.label.starts_with("Query case"))
            .all(|r| r.correct)
    );
    assert!(results.iter().any(|r| !r.correct));
    state(&mut app, &id).view = teaching::View::Assignment;
    for _ in 0..3 {
        frame(&ctx, &mut app, 2200., 1080., vec![]);
    }
    assert!(
        app.metrics
            .dns_access
            .summary
            .as_deref()
            .unwrap()
            .starts_with("Needs attention")
    );
    assert!(!app.metrics.completion);
    app.progress.attempts.get_mut(&id).unwrap().challenge = true;
    draw(&ctx, &mut app);
    assert!(
        !app.metrics
            .dns_access
            .controls
            .iter()
            .any(|c| c.0 == "fill" || c.0 == "introduction")
    );
    assert!(
        fixture::task()
            .grade(&fixture::reached(&def.initial))
            .iter()
            .all(|r| r.correct)
    );
}
#[test]
fn actual_authored_catalog_lessons_have_three_screens_and_assignment_keeps_answers_hidden() {
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("dns-access-batch")
    ));
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    assert!(app._smoke_profile.is_some());
    assert_eq!((app.scenarios.len(), app.units.len()), (412, 158));
    for id in [
        "lpic2-dns-recursion-boundary",
        "lpic2-dns-secondary-transfer-policy",
        "lpic2-dns-access-handover",
    ] {
        let i = app.scenarios.iter().position(|s| s.id == id).unwrap();
        app.launch(i);
        let entry = lesson_presentation::entry(&app.scenarios[i]).unwrap();
        assert!(entry.concepts.len() >= 4);
        assert_eq!(entry.practice.len(), 3);
        assert!(entry.worked[0].value.contains("allow-query"));
        for (width, scale, light) in [(2200., 1., false), (946., 2., true)] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            if light {
                app.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone();
            } else {
                app.theme_id = theme::Preferences::default().theme;
            }
            theme::apply(&ctx, &app.theme_id);
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                state(&mut app, id).view = view;
                app.lesson_scroll_reset = true;
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                let m = &app.metrics.dns_access;
                if view == teaching::View::Introduction {
                    assert!(m.controls.iter().any(|r| r.0 == "worked-value-0"));
                    assert!(!m.controls.iter().any(|r| r.0 == "fill"));
                } else {
                    assert_eq!(
                        m.controls.iter().any(|r| r.0 == "fill"),
                        view == teaching::View::Example
                    );
                    for key in ["editor-viewport", "save", "reload", "query", "check"] {
                        let (_, r, clip) = m.controls.iter().find(|r| r.0 == key).unwrap();
                        assert!(
                            r.width() > 0.
                                && r.left() >= clip.left() - 1.
                                && r.right() <= clip.right() + 1.,
                            "{id}/{view:?}/{width}/{key}: {r:?}/{clip:?}"
                        );
                    }
                }
                assert!(!app.metrics.completion);
            }
        }
    }
}

#[test]
fn offscreen_field_metrics_keep_real_geometry_separate_from_visibility() {
    let (ctx, mut app, id) = opened();
    state(&mut app, &id).view = teaching::View::Assignment;
    app.scale = 2.;
    ctx.set_zoom_factor(2.);
    frame(&ctx, &mut app, 640., 500., vec![]);
    let fields = &app.metrics.dns_access.fields;
    assert!(
        fields
            .iter()
            .any(|(key, rect, _, clip)| key != "editor" && !clip.contains_rect(*rect))
    );
    for (key, rect, _, _) in fields {
        assert!(
            rect.is_finite() && rect.width() > 0. && rect.height() > 0.,
            "{key}: retain the actual response rectangle, including offscreen fields"
        );
    }
}
