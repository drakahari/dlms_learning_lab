//! Source-only UI capability fixtures; no curriculum-completion claim.
use super::*;
use dlms_learning_lab::maintenance_notice_workspace as work;
fn opened(restart: bool) -> (egui::Context, LabApp, String) {
    let mut def: work::Definition = serde_json::from_str(include_str!(
        "../../../tests/fixtures/maintenance-notice-workspace-v1.json"
    ))
    .unwrap();
    if !restart {
        for task in [&mut def.initial, &mut def.example] {
            task.requires_restart = false;
            task.minimum_lead = 0;
            task.starter.clear();
            task.reference.retain(|a| {
                matches!(
                    a,
                    dlms_learning_lab::maintenance_notice::Action::Save { .. }
                        | dlms_learning_lab::maintenance_notice::Action::Broadcast { .. }
                )
            });
        }
    }
    let mut solution = def
        .initial
        .reference
        .iter()
        .map(|a| {
            format!(
                "notice-action {}",
                serde_json::to_string(&work::Reference::Apply { action: a.clone() }).unwrap()
            )
        })
        .collect::<Vec<_>>();
    solution.push("notice-action {\"kind\":\"check\"}".into());
    let raw = serde_json::json!({"schema_version":1,"scenarios":[{"id":"notice-ui-capability-fixture","version":1,"track":"LPIC-2","topic":"Maintenance notices","unit":"notice-ui-fixture","title":"Notice workspace model fixture","summary":"UI capability fixture, not authored curriculum.","objective":"Repair notices and the supplied virtual maintenance plan.","difficulty":"Intermediate","minutes":20,"steps":["Inspect notices and sessions.","Repair files, announce the work and verify outcomes."],"hints":["Preserve the pre-login notice."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical fixture; no independent novice acceptance.","simplifications":"Bounded local notices and virtual time only.","initial":{"kind":"MaintenanceNotice","simulation":work::Session::new(&def).unwrap()},"goal":{"kind":"MaintenanceNotice","definition":def},"solution":solution,"prerequisites":[]}]});
    let scenario = pack::load(&serde_json::to_string(&raw).unwrap())
        .unwrap()
        .remove(0);
    let id = scenario.id.clone();
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    let i = app.scenarios.len();
    app.scenarios.push(scenario);
    app.launch(i);
    draw(&ctx, &mut app);
    (ctx, app, id)
}
fn session<'a>(app: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::MaintenanceNotice { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
#[test]
fn native_notice_controls_reflow_and_preserve_editor_save_geometry_across_scales() {
    for restart in [false, true] {
        let (ctx, mut app, id) = opened(restart);
        for (width, scale, light) in [(2200., 1., false), (2200., 1.5, false), (946., 2., true)] {
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
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                let rows = &app.metrics.maintenance_notice.controls;
                if view == teaching::View::Introduction {
                    assert!(!rows.iter().any(|r| r.0 == "editor"));
                    continue;
                }
                for key in ["document", "editor-viewport", "command", "run", "check"] {
                    let (_, rect, clip) = rows.iter().find(|r| r.0 == key).unwrap();
                    assert!(
                        rect.width() > 0.
                            && rect.left() >= clip.left() - 1.
                            && rect.right() <= clip.right() + 1.,
                        "{key} {scale}: {rect:?} {clip:?}"
                    );
                }
                let viewport = rows.iter().find(|r| r.0 == "editor-viewport").unwrap().1;
                let save = rows.iter().find(|r| r.0 == "save").unwrap().1;
                let target = if view == teaching::View::Example {
                    &mut session(&mut app, &id).practice
                } else {
                    &mut session(&mut app, &id).assignment
                };
                target.selected = work::Editor::Motd;
                target.edit("Long draft line\n".repeat(80)).unwrap();
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                let rows = &app.metrics.maintenance_notice.controls;
                let next = rows.iter().find(|r| r.0 == "editor-viewport").unwrap().1;
                let next_save = rows.iter().find(|r| r.0 == "save").unwrap().1;
                assert!((viewport.height() - next.height()).abs() < 1.);
                assert!(
                    (save.top() - next_save.top()).abs() < 1.,
                    "Scrolling editor must not move Save"
                );
                let command = rows.iter().find(|r| r.0 == "command").unwrap().1;
                let run = rows.iter().find(|r| r.0 == "run").unwrap().1;
                assert!(
                    (command.height() - run.height()).abs() < 1.,
                    "Related controls share font-derived height"
                );
            }
        }
    }
}
#[test]
fn challenge_hides_unsolicited_worked_guidance_and_practice_never_awards_completion() {
    let (ctx, mut app, id) = opened(true);
    let index = app.selected.unwrap();
    let engine::Goal::MaintenanceNotice { definition } = &app.scenarios[index].goal else {
        panic!()
    };
    let def = (**definition).clone();
    let draft = &mut session(&mut app, &id).practice;
    for a in &def.example.reference {
        draft.apply_action(&def.example, a.clone()).unwrap();
    }
    draft.check(&def.example).unwrap();
    session(&mut app, &id).view = teaching::View::Example;
    draw(&ctx, &mut app);
    assert!(!app.metrics.completion);
    app.progress.attempts.get_mut(&id).unwrap().challenge = true;
    draw(&ctx, &mut app);
    assert!(
        !app.metrics
            .maintenance_notice
            .controls
            .iter()
            .any(|r| r.0 == "example" || r.0 == "introduction" || r.0 == "reset-example")
    );
    assert!(!app.metrics.completion);
    assert!(
        app.metrics
            .maintenance_notice
            .controls
            .iter()
            .any(|r| r.0 == "check")
    );
}
