use super::*;
const IDS: [&str; 3] = ["lpic1-permissions", "lpic1-traversal", "lpic1-logs"];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    a.set_legacy_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn practice(a: &LabApp, id: &str) -> serde_json::Value {
    let w = a.progress.attempts[id].workspace.as_ref().unwrap();
    if let Some(l) = &w.legacy_access_learning {
        serde_json::to_value(l).unwrap()
    } else {
        serde_json::to_value(w.legacy_logs_learning.as_ref().unwrap()).unwrap()
    }
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    accounts_ui_click(ctx, a, |m| m.terminal);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    accounts_ui_click(ctx, a, |m| m.terminal_run);
}
#[test]
fn legacy_native_new_introductions_and_old_first_open_assignment_preserve_work() {
    for id in IDS {
        let (ctx, mut a) = app();
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.legacy_view(i), teaching::View::Introduction);
        assert!(a.metrics.terminal.is_none());
        accounts_ui_click(&ctx, &mut a, |m| m.assignment_action);
        assert_eq!(a.legacy_view(i), teaching::View::Assignment);
        assert!(a.metrics.terminal.is_some() && a.metrics.editor_save.is_none());
        let (ctx, mut old) = app();
        let s = old.scenarios[i].clone();
        let mut attempt = Attempt::new(&s, 1);
        model::prepare(&mut attempt);
        attempt.workspace.as_mut().unwrap().command = "unfinished older command".into();
        let before = attempt.state.clone();
        old.progress.attempts.insert(id.into(), attempt);
        old.launch(i);
        draw(&ctx, &mut old);
        assert_eq!(old.legacy_view(i), teaching::View::Assignment);
        assert_eq!(old.progress.attempts[id].state, before);
        assert_eq!(old.input, "unfinished older command");
    }
}
#[test]
fn legacy_native_real_insert_run_reference_experiment_and_repair_are_independent() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Example);
        let s = a.scenarios[i].clone();
        let initial = a.progress.attempts[id].state.clone();
        let before = practice(&a, id);
        accounts_ui_click(&ctx, &mut a, |m| m.insert);
        let filled = practice(&a, id);
        assert_eq!(filled["current"], before["current"]);
        assert_eq!(filled["transcript"], before["transcript"]);
        assert!(!filled["command"].as_str().unwrap().is_empty());
        accounts_ui_click(&ctx, &mut a, |m| m.terminal_run);
        assert_eq!(practice(&a, id)["command"], "");
        assert_eq!(practice(&a, id)["transcript"].as_array().unwrap().len(), 1);
        let (reference, experiment, repair) = if let Some(l) = &s.legacy_access_teaching {
            (&l.reference, &l.try_commands, &l.repair_commands)
        } else {
            let l = s.legacy_logs_teaching.as_ref().unwrap();
            (&l.reference, &l.try_commands, &l.repair_commands)
        };
        for c in reference {
            command(&ctx, &mut a, c);
        }
        accounts_ui_click(&ctx, &mut a, |m| m.check);
        assert!(a.metrics.seed_feedback.iter().all(|r| r.0));
        assert!(!a.metrics.seed_feedback.is_empty());
        for c in experiment {
            command(&ctx, &mut a, c);
        }
        accounts_ui_click(&ctx, &mut a, |m| m.check);
        assert!(!a.metrics.seed_feedback[0].0);
        for c in repair {
            command(&ctx, &mut a, c);
        }
        accounts_ui_click(&ctx, &mut a, |m| m.check);
        assert!(a.metrics.seed_feedback.iter().all(|r| r.0));
        assert_eq!(a.progress.attempts[id].state, initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        command(&ctx, &mut a, "unfinished practice");
        let old = practice(&a, id);
        accounts_ui_click(&ctx, &mut a, |m| m.example_reset);
        let reset = practice(&a, id);
        assert!(reset["transcript"].as_array().unwrap().is_empty());
        assert_ne!(old["current"], reset["current"]);
        assert_eq!(a.progress.attempts[id].state, initial);
    }
}
#[test]
fn legacy_native_commands_cannot_run_behind_confirmation_settings_or_real_chooser() {
    for id in [IDS[0], IDS[2]] {
        for view in [teaching::View::Example, teaching::View::Assignment] {
            for blocked in ["reset", "settings", "data"] {
                let (ctx, mut a, _) = opened(id, view);
                accounts_ui_click(&ctx, &mut a, |m| m.terminal);
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    vec![
                        acl_ctrl(egui::Key::A),
                        egui::Event::Text(
                            if id == IDS[0] {
                                "chmod 777 /srv/brief.txt"
                            } else {
                                "cat /var/log/worker.log"
                            }
                            .into(),
                        ),
                    ],
                );
                let input = a.metrics.focus.unwrap();
                match blocked {
                    "reset" => a.reset_pending = true,
                    "settings" => a.settings_open = true,
                    "data" => {
                        a.smoke = true;
                        a.settings_open = true;
                        a.settings_data = true;
                        draw(&ctx, &mut a);
                        accounts_ui_click(&ctx, &mut a, |m| m.data_export);
                        assert!(a.data_tools.modal_open());
                        a.settings_open = false;
                    }
                    _ => unreachable!(),
                }
                let before = serde_json::to_value(&a.progress.attempts[id]).unwrap();
                ctx.memory_mut(|m| m.request_focus(input));
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    vec![key(egui::Key::Enter, false)],
                );
                assert_eq!(
                    serde_json::to_value(&a.progress.attempts[id]).unwrap(),
                    before
                );
                assert_eq!(a.metrics.terminal_run_enabled, Some(false));
            }
        }
    }
}
#[test]
fn legacy_native_views_and_challenge_keep_controls_inside_at_100_150_200_percent() {
    for id in IDS {
        for scale in [1., 1.5, 2.] {
            for (width, height) in [(1200., 850.), (720., 560.)] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let (ctx, mut a, i) = opened(id, view);
                    a.scale = scale;
                    for _ in 0..5 {
                        frame(&ctx, &mut a, width, height, vec![]);
                    }
                    let rect = ctx.content_rect();
                    for input in &a.metrics.inputs {
                        assert!(
                            input.left() >= rect.left() && input.right() <= rect.right() + 1.,
                            "{id}/{view:?}/{scale}"
                        );
                    }
                    assert!(a.metrics.terminal.is_some() || view == teaching::View::Introduction);
                    a.progress.attempts.get_mut(id).unwrap().challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.legacy_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty() && a.metrics.insert.is_none());
                    assert!(!a.help_open);
                }
            }
        }
    }
}
