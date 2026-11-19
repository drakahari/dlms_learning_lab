//! Actual widgets for the opt-in advisory workspace; no host package operation.
use super::*;
const PATCH_IDS: [&str; 2] = ["lpic2-vendor-backport", "lpic2-vendor-patch-set"];
#[test]
fn patch_origin_table_uses_outer_width_and_never_collapses_to_character_columns() {
    for id in PATCH_IDS {
        for (w, h, scale, theme_id) in [
            (1280., 900., 1., "dlms-dark"),
            (800., 750., 1.5, "dlms-light"),
            (720., 760., 2., "dlms-dark"),
        ] {
            let (ctx, mut app, _) = super::super::debug_recovery::opened(id);
            draw(&ctx, &mut app);
            tab(&ctx, &mut app, View::Assignment);
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut app, w, h, vec![]);
            }
            let rows = &app.metrics.rpm.origin_rows;
            assert_eq!(rows.len(), if id == PATCH_IDS[0] { 4 } else { 7 });
            let font = ctx.style().text_styles[&egui::TextStyle::Body].size;
            let first = rows[0];
            for (path, origin) in rows {
                assert!(
                    path.width() >= font * 10. && origin.width() >= font * 10.,
                    "Readable widths, not one-character columns: {path:?} {origin:?}"
                );
                assert!((path.left() - first.0.left()).abs() < 1.);
                assert!((origin.left() - first.1.left()).abs() < 1.);
                assert!(
                    path.height() < font * 10. && origin.height() < font * 10.,
                    "No vertical character dump"
                );
                if origin.left() > path.left() + 1. {
                    assert!(origin.left() >= path.right());
                } else {
                    assert!(origin.top() >= path.bottom());
                }
                assert!(origin.right() <= ctx.content_rect().right() + 1.);
                assert!(path.right() <= ctx.content_rect().right() + 1.);
            }
        }
    }
}
#[test]
fn patch_real_three_views_insert_run_partial_and_current_invalidation() {
    for id in PATCH_IDS {
        let (ctx, mut app, i) = super::super::debug_recovery::opened(id);
        draw(&ctx, &mut app);
        assert_eq!(learning(&app).view, View::Introduction);
        let initial = chosen(&app).state.clone();
        let source = app.scenarios[i].rpm_packages_teaching.clone().unwrap();
        tab(&ctx, &mut app, View::Example);
        let insert = reveal(&ctx, &mut app, 1200., 850., |m| m.rpm.insert);
        tap(&ctx, &mut app, 1200., 850., insert);
        assert!(!console(&app, true).command.is_empty());
        assert!(
            console(&app, true).entries.is_empty(),
            "Insert does not run or grade"
        );
        for c in &source.example_commands {
            submit(&ctx, &mut app, &c.command, true);
        }
        assert!(learning(&app).checks(&source).iter().all(|r| r.0));
        for c in &source.try_change.commands {
            submit(&ctx, &mut app, &c.command, false);
        }
        assert_eq!(
            learning(&app)
                .checks(&source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            source.try_change.expected
        );
        for c in &source.repair {
            submit(&ctx, &mut app, &c.command, true);
        }
        assert_eq!(chosen(&app).state, initial);
        assert!(!progress::current_solved(chosen(&app), &app.scenarios[i]));
        tab(&ctx, &mut app, View::Assignment);
        assert!(console(&app, false).command.is_empty() && console(&app, false).entries.is_empty());
        if id == PATCH_IDS[1] {
            submit(
                &ctx,
                &mut app,
                "rpm -U /media/dispatch/dispatch-agent-1.0-4.train8.noarch.rpm",
                true,
            );
            assert!(matches!(
                &console(&app, false).entries.last().unwrap().output,
                terminal::Outcome::SimulatorUnsupported { .. }
            ));
            assert_eq!(chosen(&app).state, initial);
        }
        for c in app.scenarios[i].solution.clone() {
            submit(&ctx, &mut app, &c, true);
        }
        let check = reveal(&ctx, &mut app, 1200., 850., |m| m.rpm.check);
        tap(&ctx, &mut app, 1200., 850., check);
        assert!(app.metrics.completion);
        assert!(progress::current_solved(chosen(&app), &app.scenarios[i]));
        let prefix = if id == PATCH_IDS[0] {
            "relay"
        } else {
            "dispatch"
        };
        submit(&ctx, &mut app, &format!("rpm -e {prefix}-notes"), true);
        assert!(!progress::current_solved(chosen(&app), &app.scenarios[i]));
        assert!(!app.metrics.completion);
        let check = reveal(&ctx, &mut app, 1200., 850., |m| m.rpm.check);
        tap(&ctx, &mut app, 1200., 850., check);
        assert!(app.metrics.rpm.result_rows.iter().any(|r| !r.1));
    }
}
#[test]
fn patch_fields_focus_populated_drafts_and_actions_reflow_at_supported_scales() {
    for id in PATCH_IDS {
        for (w, h, scale, theme_id) in [
            (1280., 900., 1., "dlms-dark"),
            (800., 750., 1.5, "dlms-light"),
            (720., 760., 2., "dlms-dark"),
        ] {
            let (ctx, mut app, i) = super::super::debug_recovery::opened(id);
            draw(&ctx, &mut app);
            tab(&ctx, &mut app, View::Assignment);
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut app, w, h, vec![]);
            }
            type_command(&ctx, &mut app, w, h, "rpm -q unfinished-learning-draft");
            assert_eq!(
                console(&app, false).command,
                "rpm -q unfinished-learning-draft"
            );
            assert!(console(&app, false).entries.is_empty());
            let field = reveal(&ctx, &mut app, w, h, |m| m.rpm.command.map(|r| r.0));
            assert!(field.width() > 120. && field.height() > 15.);
            let run = reveal(&ctx, &mut app, w, h, |m| m.rpm.run.map(|r| r.0));
            assert!(run.height() > 15.);
            assert!(!progress::current_solved(chosen(&app), &app.scenarios[i]));
        }
    }
}
#[test]
fn patch_challenge_keeps_the_complete_task_without_unsolicited_worked_commands() {
    for id in PATCH_IDS {
        let (ctx, mut app, i) = super::super::debug_recovery::opened(id);
        draw(&ctx, &mut app);
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut app);
        assert!(app.metrics.rpm.tabs.is_empty());
        assert!(app.metrics.rpm.command.is_some() && app.metrics.rpm.check.is_some());
        assert!(console(&app, false).command.is_empty() && console(&app, false).entries.is_empty());
        assert_eq!(chosen(&app).state, app.scenarios[i].initial);
        assert!(!progress::current_solved(chosen(&app), &app.scenarios[i]));
    }
}

#[test]
fn patch_readonly_inspector_shows_actual_retained_and_staged_configuration() {
    for id in PATCH_IDS {
        let (ctx, mut app, i) = super::super::debug_recovery::opened(id);
        draw(&ctx, &mut app);
        tab(&ctx, &mut app, View::Assignment);
        for c in app.scenarios[i].solution.clone() {
            submit(&ctx, &mut app, &c, true);
        }
        let before = serde_json::to_value(chosen(&app)).unwrap();
        let inspector = reveal(&ctx, &mut app, 1200., 850., |m| m.rpm.inspector);
        tap(&ctx, &mut app, 1200., 850., inspector);
        let prefix = if id == PATCH_IDS[0] {
            "relay"
        } else {
            "dispatch"
        };
        choose_inspector(&ctx, &mut app, &format!("{prefix}-agent-fixed"), |m| {
            m.rpm.inspect_header
        });
        choose_inspector(&ctx, &mut app, &format!("/srv/{prefix}/agent.conf"), |m| {
            m.rpm.inspect_path
        });
        for text in ["zone=site-east", "zone=vendor-v2", "agent.conf.rpmnew"] {
            assert!(
                app.metrics
                    .rpm
                    .inspection_text
                    .iter()
                    .any(|s| s.contains(text)),
                "{text}"
            );
        }
        assert_eq!(
            serde_json::to_value(chosen(&app)).unwrap(),
            before,
            "Inspection changes no state, draft or grade"
        );
    }
}
