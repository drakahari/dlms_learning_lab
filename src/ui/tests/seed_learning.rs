use super::*;
const IDS: [&str; 2] = ["lpic2-dns", "lpic2-storage"];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    a.set_seed_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
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
fn choose_action(ctx: &egui::Context, a: &mut LabApp, index: usize) {
    accounts_ui_click(ctx, a, |m| m.terminal_example_picker);
    a.smoke_stage = 24400 + index;
    for _ in 0..24 {
        let target = a
            .metrics
            .terminal_example_choices
            .iter()
            .find(|(n, _)| *n == index)
            .map(|(_, r)| *r);
        if a.mac_popup_choice(ctx, target, a.metrics.terminal_example_popup) {
            return;
        }
        let events = std::mem::take(&mut a.pending_events);
        frame(ctx, a, 1200., 850., events);
        for _ in 0..4 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    panic!("practice action must be reached through the real clipped popup");
}
fn field(ctx: &egui::Context, a: &mut LabApp, n: usize, text: &str) {
    let r = a.metrics.inputs[n];
    for _ in 0..20 {
        let target = a.metrics.inputs[n];
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if viewport.contains_rect(target) {
            click(ctx, a, target);
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
            );
            draw(ctx, a);
            return;
        }
        let dy = if target.bottom() > viewport.bottom() {
            viewport.bottom() - target.bottom()
        } else {
            viewport.top() - target.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(viewport.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("field stays offscreen {r:?}");
}
#[test]
fn seed_native_new_introductions_and_upgraded_assignment_keep_the_original_native_fields() {
    for id in IDS {
        let (ctx, mut a) = app();
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.seed_view(i), teaching::View::Introduction);
        assert!(a.metrics.terminal.is_none());
        accounts_ui_click(&ctx, &mut a, |m| m.assignment_action);
        assert_eq!(a.seed_view(i), teaching::View::Assignment);
        assert!(a.metrics.editor_save.is_some() && a.metrics.terminal.is_some());
    }
}
#[test]
fn seed_native_actual_example_insert_save_run_experiment_and_repair_are_ungraded() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Example);
        let s = a.scenarios[i].clone();
        let actions: Vec<bool> = if let Some(l) = &s.seed_dns_teaching {
            l.example_actions
                .iter()
                .map(|a| {
                    matches!(
                        a.action,
                        dlms_learning_lab::seed_dns_teaching::Action::Save { .. }
                    )
                })
                .collect()
        } else {
            s.seed_mount_teaching
                .as_ref()
                .unwrap()
                .example_actions
                .iter()
                .map(|a| {
                    matches!(
                        a.action,
                        dlms_learning_lab::seed_mount_teaching::Action::Save { .. }
                    )
                })
                .collect()
        };
        for (index, save) in actions.into_iter().enumerate() {
            choose_action(&ctx, &mut a, index);
            let w = a.progress.attempts[id].workspace.as_ref().unwrap();
            let selected = if let Some(l) = &w.seed_dns_learning {
                l.selected_action
            } else {
                w.seed_mount_learning.as_ref().unwrap().selected_action
            };
            assert_eq!(selected, index, "actual popup selection for {id}");
            let before = serde_json::to_value(&a.progress.attempts[id].state).unwrap();
            let practice_before = if id == IDS[0] {
                serde_json::to_value(
                    &a.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .seed_dns_learning
                        .as_ref()
                        .unwrap()
                        .current,
                )
                .unwrap()
            } else {
                serde_json::to_value(
                    &a.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .seed_mount_learning
                        .as_ref()
                        .unwrap()
                        .current,
                )
                .unwrap()
            };
            accounts_ui_click(&ctx, &mut a, |m| m.insert);
            assert_eq!(
                serde_json::to_value(&a.progress.attempts[id].state).unwrap(),
                before
            );
            let current = if id == IDS[0] {
                serde_json::to_value(
                    &a.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .seed_dns_learning
                        .as_ref()
                        .unwrap()
                        .current,
                )
                .unwrap()
            } else {
                serde_json::to_value(
                    &a.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .seed_mount_learning
                        .as_ref()
                        .unwrap()
                        .current,
                )
                .unwrap()
            };
            assert_eq!(current, practice_before);
            if save {
                accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
            } else {
                accounts_ui_click(&ctx, &mut a, |m| m.terminal_run);
            }
        }
        accounts_ui_click(&ctx, &mut a, |m| m.check);
        if id == IDS[0] {
            let lesson = s.seed_dns_teaching.as_ref().unwrap();
            assert!(
                a.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .seed_dns_learning
                    .as_ref()
                    .unwrap()
                    .checks(lesson)
                    .iter()
                    .all(|c| c.0)
            );
            field(&ctx, &mut a, 0, "198.51.100.45");
            field(&ctx, &mut a, 1, "43");
            accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
            command(&ctx, &mut a, "dig mirror.lab.test");
            let l = a.progress.attempts[id]
                .workspace
                .as_ref()
                .unwrap()
                .seed_dns_learning
                .as_ref()
                .unwrap();
            assert_eq!(l.current.served.unwrap().to_string(), "198.51.100.44");
            assert!(l.checks(lesson).iter().all(|c| !c.0));
            field(&ctx, &mut a, 0, "198.51.100.44");
            field(&ctx, &mut a, 1, "44");
            accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
            command(&ctx, &mut a, "systemctl reload named");
            assert!(
                a.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .seed_dns_learning
                    .as_ref()
                    .unwrap()
                    .checks(lesson)
                    .iter()
                    .all(|c| c.0)
            );
        } else {
            let lesson = s.seed_mount_teaching.as_ref().unwrap();
            let learning = a.progress.attempts[id]
                .workspace
                .as_ref()
                .unwrap()
                .seed_mount_learning
                .as_ref()
                .unwrap();
            assert!(
                learning.checks(lesson).iter().all(|c| c.0),
                "worked example incomplete: {learning:?}"
            );
            command(&ctx, &mut a, "umount /data");
            assert_eq!(
                a.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .seed_mount_learning
                    .as_ref()
                    .unwrap()
                    .checks(lesson)
                    .iter()
                    .map(|c| c.0)
                    .collect::<Vec<_>>(),
                [true, true, false]
            );
            command(&ctx, &mut a, "mount /data");
            assert!(
                a.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .seed_mount_learning
                    .as_ref()
                    .unwrap()
                    .checks(lesson)
                    .iter()
                    .all(|c| c.0)
            );
        }
        assert_eq!(a.progress.attempts[id].state, s.initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        accounts_ui_click(&ctx, &mut a, |m| m.example_reset);
        assert_eq!(a.progress.attempts[id].state, s.initial);
    }
}
#[test]
fn seed_native_assignment_save_keeps_invalid_raw_input_atomic_and_undo_is_separate() {
    let (ctx, mut a, i) = opened(IDS[0], teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    let old = a.progress.attempts[IDS[0]].state.clone();
    field(&ctx, &mut a, 0, "invalid address");
    let assignment_id = ctx.memory(|m| m.focused()).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    assert_eq!(a.progress.attempts[IDS[0]].state, old);
    assert_eq!(
        a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .dns_address
            .as_deref(),
        Some("invalid address")
    );
    a.set_seed_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    field(&ctx, &mut a, 0, "198.51.100.99");
    let example_id = ctx.memory(|m| m.focused()).unwrap();
    assert_ne!(assignment_id, example_id);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    assert_eq!(
        a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .dns_address
            .as_deref(),
        Some("invalid address")
    );
    assert_eq!(a.progress.attempts[IDS[0]].state, s.initial);
    a.set_seed_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .dns_address
            .as_deref(),
        Some("invalid address")
    );
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], &s));
}
#[test]
fn seed_native_inactive_enter_does_not_execute_or_take_focus_and_challenge_hides_worked_help() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Example);
        accounts_ui_click(&ctx, &mut a, |m| m.terminal);
        let text = if id == IDS[0] {
            "dns serial 99"
        } else {
            "umount /data"
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
        );
        let input = a.metrics.focus.unwrap();
        for settings in [false, true] {
            a.settings_open = settings;
            a.reset_pending = !settings;
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
        a.settings_open = false;
        a.reset_pending = false;
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert_eq!(a.seed_view(i), teaching::View::Assignment);
        assert!(a.metrics.lesson_tabs.is_empty() && a.metrics.insert.is_none());
        assert!(a.metrics.editor_save.is_some());
    }
}
#[test]
fn seed_native_views_wrap_and_keep_fields_inside_at_all_supported_scales() {
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
                            input.right() <= rect.right() + 1.,
                            "{id}/{view:?}/{scale} {input:?} {rect:?}"
                        );
                        assert!(input.left() >= rect.left());
                    }
                    assert!(a.metrics.terminal.is_some() || view == teaching::View::Introduction);
                    a.progress.attempts.get_mut(id).unwrap().challenge = true;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, height, vec![]);
                    }
                    assert!(a.metrics.lesson_tabs.is_empty());
                    assert_eq!(a.seed_view(i), teaching::View::Assignment);
                }
            }
        }
    }
}
