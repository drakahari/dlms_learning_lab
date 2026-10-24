//! Native-egui source proposals; actual Fedora rendering remains a separate gate.
use super::*;
fn mac_app(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut app) = app();
    let scenario = pack::load(include_str!(
        "../../../tests/fixtures/mac-learning-preview.json"
    ))
    .unwrap()
    .remove(0);
    let i = app.scenarios.len();
    app.scenarios.push(scenario);
    app.launch(i);
    app.set_mac_view(i, view);
    (ctx, app, i)
}
#[test]
fn mac_practice_insert_send_and_assignment_are_distinct_and_no_example_awards_credit() {
    let (ctx, mut app, i) = mac_app(teaching::View::Introduction);
    draw(&ctx, &mut app);
    assert!(app.metrics.example_action.is_some() && app.metrics.assignment_action.is_some());
    let id = app.scenarios[i].id.clone();
    let original = app.progress.attempts[&id].state.clone();
    app.set_mac_view(i, teaching::View::Example);
    draw(&ctx, &mut app);
    let before = app.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .mac_learning
        .as_ref()
        .unwrap()
        .simulation
        .clone();
    accounts_ui_click(&ctx, &mut app, |m| m.command_insert);
    draw(&ctx, &mut app);
    assert_eq!(
        app.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .mac_learning
            .as_ref()
            .unwrap()
            .simulation,
        before
    );
    accounts_ui_click(&ctx, &mut app, |m| m.mac_send);
    draw(&ctx, &mut app);
    assert!(
        !app.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .mac_learning
            .as_ref()
            .unwrap()
            .simulation
            .entries()
            .is_empty()
    );
    assert_eq!(app.progress.attempts[&id].state, original);
    assert!(!progress::current_solved(
        &app.progress.attempts[&id],
        &app.scenarios[i]
    ));
    app.set_mac_view(i, teaching::View::Assignment);
    draw(&ctx, &mut app);
    assert!(app.metrics.mac_send.is_some());
    assert_eq!(app.progress.attempts[&id].state, original);
}
#[test]
fn mac_visible_controls_reflow_without_challenge_walkthrough_at_three_scales() {
    for scale in [1., 1.5, 2.] {
        for (w, h) in [(1200., 850.), (850., 600.)] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                let (ctx, mut app, i) = mac_app(view);
                app.scale = scale;
                ctx.set_zoom_factor(scale);
                for _ in 0..4 {
                    frame(&ctx, &mut app, w, h, vec![]);
                }
                let width = ctx.content_rect().right();
                for rect in &app.metrics.inputs {
                    assert!(
                        rect.left() >= -1. && rect.right() <= width + 1.,
                        "{scale} {w} {view:?}: {rect:?} vs{width}"
                    );
                }
                if view == teaching::View::Assignment {
                    app.progress
                        .attempts
                        .get_mut(&app.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    for _ in 0..3 {
                        frame(&ctx, &mut app, w, h, vec![]);
                    }
                    assert!(app.metrics.lesson_tabs.is_empty());
                    assert!(app.metrics.command_insert.is_none());
                }
            }
        }
    }
}
#[test]
fn mac_port_field_edit_invalidates_success_and_practice_undo_keeps_assignment_drafts() {
    let (ctx, mut app, i) = mac_app(teaching::View::Assignment);
    let s = app.scenarios[i].clone();
    for action in &s.solution {
        engine::execute(
            &mut app.progress.attempts.get_mut(&s.id).unwrap().state,
            action,
        )
        .unwrap();
    }
    app.checked = true;
    draw(&ctx, &mut app);
    assert!(app.metrics.completion);
    assert!(app.metrics.vlan_fields.is_empty());
    accounts_ui_click(&ctx, &mut app, |m| m.mac_settings);
    draw(&ctx, &mut app);
    let (id, rect) = app.metrics.vlan_fields[0];
    click(&ctx, &mut app, rect);
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        &ctx,
        &mut app,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text("9999".into())],
    );
    draw(&ctx, &mut app);
    assert!(!progress::current_solved(&app.progress.attempts[&s.id], &s));
    let assignment = app.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .mac_draft
        .clone()
        .unwrap();
    app.set_mac_view(i, teaching::View::Example);
    draw(&ctx, &mut app);
    frame(&ctx, &mut app, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    assert_eq!(
        app.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .mac_draft
            .as_ref()
            .unwrap(),
        &assignment
    );
}

#[test]
fn mac_focused_edits_and_send_are_disabled_behind_reset_or_settings() {
    let (ctx, mut app, i) = mac_app(teaching::View::Assignment);
    draw(&ctx, &mut app);
    accounts_ui_click(&ctx, &mut app, |m| m.mac_settings);
    draw(&ctx, &mut app);
    let name = app.scenarios[i].id.clone();
    let field = app.metrics.vlan_fields[0].0;
    let before = serde_json::to_value(&app.progress.attempts[&name]).unwrap();
    for settings in [false, true] {
        app.settings_open = settings;
        app.reset_pending = !settings;
        ctx.memory_mut(|m| m.request_focus(field));
        frame(
            &ctx,
            &mut app,
            1200.,
            850.,
            vec![
                acl_ctrl(egui::Key::A),
                egui::Event::Text("9999".into()),
                key(egui::Key::Enter, false),
            ],
        );
        assert_eq!(
            serde_json::to_value(&app.progress.attempts[&name]).unwrap(),
            before
        );
        // Inspect flags captured from the actual current widget Responses.
        // Context::read_response after the completed pass can describe older
        // interaction data even while the current disabled widgets reject input.
        let current_field = app.metrics.vlan_fields[0].0;
        assert_eq!(
            app.metrics
                .vlan_field_enabled
                .iter()
                .find(|(id, _)| *id == current_field)
                .map(|(_, enabled)| *enabled),
            Some(false)
        );
        let send = app.metrics.mac_send_id.expect("measured Send button");
        assert_eq!(app.metrics.mac_send_enabled, Some(false));
        ctx.memory_mut(|m| m.request_focus(send));
        frame(
            &ctx,
            &mut app,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        assert_eq!(
            serde_json::to_value(&app.progress.attempts[&name]).unwrap(),
            before
        );
    }
}

fn mac_rendered_text(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(t.galley.text());
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, text)
                }
            }
            _ => {}
        }
    }
    let mut text = String::new();
    for _ in 0..3 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        text.clear();
        for s in output.shapes {
            collect(&s.shape, &mut text);
        }
    }
    text
}
#[test]
fn mac_pending_settings_teach_apply_while_vlan_keeps_its_actual_check_instruction() {
    let (ctx, mut a, i) = mac_app(teaching::View::Assignment);
    let id = a.scenarios[i].id.clone();
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mac_draft
        .as_mut()
        .unwrap()
        .configuration
        .switches
        .get_mut("s1")
        .unwrap()
        .vlans = "10,77,99".into();
    let text = mac_rendered_text(&ctx, &mut a);
    assert!(
        text.contains("Pending settings · Apply settings before checking work"),
        "{text}"
    );
    assert!(!text.contains("Check configuration applies and tests them"));
    for (view, expected) in [
        (
            teaching::View::Assignment,
            "Draft changes · Check configuration applies and tests them",
        ),
        (
            teaching::View::Example,
            "Practice draft · Apply and test example uses these edits",
        ),
    ] {
        let (ctx, mut a) = app();
        let i = a
            .scenarios
            .iter()
            .position(|s| matches!(s.initial, State::Switching { .. }))
            .unwrap();
        a.launch(i);
        a.set_switching_view(i, view);
        let id = a.scenarios[i].id.clone();
        let w = a
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        let draft = if view == teaching::View::Example {
            &mut w.switching_learning.as_mut().unwrap().draft
        } else {
            w.switching_draft.as_mut().unwrap()
        };
        draft
            .switches
            .values_mut()
            .next()
            .unwrap()
            .vlans
            .push_str(",77");
        let text = mac_rendered_text(&ctx, &mut a);
        assert!(text.contains(expected), "{text}");
    }
}
#[test]
fn mac_practice_selected_action_has_one_caption_and_adjacent_insert_with_persistent_explanation() {
    let (ctx, mut a, i) = mac_app(teaching::View::Example);
    let action = a.scenarios[i]
        .mac_teaching
        .as_ref()
        .unwrap()
        .example_solution[0]
        .label();
    let text = mac_rendered_text(&ctx, &mut a);
    assert_eq!(text.matches(&action).count(), 1, "{text}");
    assert!(text.contains("Insert fills the matching native controls only. Use the labeled action button to perform it."));
    let picker = a.metrics.mac_example_action.unwrap();
    let insert = a.metrics.command_insert.unwrap();
    assert!(
        (picker.center().y - insert.center().y).abs() < 1.5,
        "{picker:?} {insert:?}"
    );
    assert!(insert.left() > picker.right());
}

#[test]
fn mac_preservation_only_tasks_start_with_send_and_table_and_optional_configuration() {
    let (ctx, mut a, _i) = mac_app(teaching::View::Assignment);
    let text = mac_rendered_text(&ctx, &mut a);
    assert!(a.metrics.vlan_fields.is_empty(), "{text}");
    assert!(a.metrics.mac_settings.is_some() && a.metrics.mac_lifetime.is_some());
    assert!(a.metrics.mac_send.is_some());
    assert!(
        a.metrics.mac_time.is_none(),
        "Quiet probes do not require a real wait"
    );
    assert!(!text.contains("Entry lifetime (seconds; 0 disables expiry)"));
    assert!(text.contains("Learned MAC entries · applied state"));
    accounts_ui_click(&ctx, &mut a, |m| m.mac_settings);
    draw(&ctx, &mut a);
    assert!(!a.metrics.vlan_fields.is_empty());
}

#[test]
fn mac_lifetime_policy_is_compact_and_repair_controls_stay_visible_after_solving() {
    use dlms_learning_lab::{mac_learning as net, mac_learning_workspace as work};
    let (ctx, mut a, i) = mac_app(teaching::View::Assignment);
    let engine::Goal::MacLearning { requirements, .. } = &mut a.scenarios[i].goal else {
        unreachable!()
    };
    requirements.push(work::Requirement::Outcome {
        outcome: net::Requirement::Aging {
            minimum: 90,
            maximum: 120,
        },
    });
    let text = mac_rendered_text(&ctx, &mut a);
    assert!(
        text.contains("Entry lifetime (seconds; 0 disables expiry)"),
        "{text}"
    );
    assert!(a.metrics.vlan_fields.is_empty() && a.metrics.mac_time.is_none());
    let engine::Goal::MacLearning {
        initial,
        requirements,
    } = &mut a.scenarios[i].goal
    else {
        unreachable!()
    };
    let mut settings = initial.fabric().switches["s1"].ports["a"].clone();
    settings.up = false;
    requirements.push(work::Requirement::Port {
        switch: "s1".into(),
        port: "a".into(),
        settings,
    });
    mac_rendered_text(&ctx, &mut a);
    assert!(!a.metrics.vlan_fields.is_empty());
    // Applying the requested policy does not change which controls the task needs.
    let id = a.scenarios[i].id.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    let State::MacLearning { simulation } = &mut attempt.state else {
        unreachable!()
    };
    let mut draft = dlms_learning_lab::mac_learning_workspace::Draft::new(simulation).unwrap();
    draft
        .configuration
        .switches
        .get_mut("s1")
        .unwrap()
        .ports
        .get_mut("a")
        .unwrap()
        .up = false;
    draft.aging = "100".into();
    draft.apply(simulation).unwrap();
    attempt.workspace.as_mut().unwrap().mac_draft = Some(draft);
    mac_rendered_text(&ctx, &mut a);
    assert!(!a.metrics.vlan_fields.is_empty());
    assert!(a.metrics.mac_aging.is_some());
    accounts_ui_click(&ctx, &mut a, |m| m.mac_aging);
    let text = mac_rendered_text(&ctx, &mut a);
    assert!(
        text.contains("Entry lifetime (seconds; 0 disables expiry)"),
        "{text}"
    );
}

#[test]
fn mac_pending_optional_settings_reopen_after_exact_resume_without_moving_field_ids() {
    let (ctx, mut a, i) = mac_app(teaching::View::Assignment);
    let id = a.scenarios[i].id.clone();
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.mac_settings);
    draw(&ctx, &mut a);
    let field = a.metrics.vlan_fields[0].0;
    accounts_ui_click(&ctx, &mut a, |m| m.mac_lifetime);
    let w = a
        .progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap();
    let d = w.mac_draft.as_mut().unwrap();
    d.configuration.switches.get_mut("s1").unwrap().vlans = "bad".into();
    d.aging = "oops".into();
    let raw = serde_json::to_vec(&a.progress).unwrap();
    let saved = progress::decode(&raw, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    let (fresh, mut reopened, j) = mac_app(teaching::View::Assignment);
    assert_eq!(i, j);
    reopened.progress = saved;
    reopened.launch(j);
    let text = mac_rendered_text(&fresh, &mut reopened);
    assert!(!reopened.metrics.vlan_fields.is_empty(), "{text}");
    assert_eq!(reopened.metrics.vlan_fields[0].0, field);
    assert!(reopened.metrics.mac_aging.is_some());
    accounts_ui_click(&fresh, &mut reopened, |m| m.mac_aging);
    let text = mac_rendered_text(&fresh, &mut reopened);
    assert!(
        text.contains("Entry lifetime (seconds; 0 disables expiry)"),
        "{text}"
    );
    assert!(text.contains("Pending settings"));
    assert!(!progress::current_solved(
        &reopened.progress.attempts[&id],
        &reopened.scenarios[j]
    ));
    // Repeated frames retain raw work and the same field identity, rather than relocating on edits.
    let before = serde_json::to_value(&reopened.progress.attempts[&id]).unwrap();
    mac_rendered_text(&fresh, &mut reopened);
    assert_eq!(reopened.metrics.vlan_fields[0].0, field);
    assert_eq!(
        serde_json::to_value(&reopened.progress.attempts[&id]).unwrap(),
        before
    );
}

#[test]
fn mac_optional_port_typing_keeps_focus_identity_and_undo_in_the_same_section() {
    let (ctx, mut a, i) = mac_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.mac_settings);
    draw(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    let before = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .mac_draft
        .clone();
    let (field, rect) = a.metrics.vlan_fields[0];
    click(&ctx, &mut a, rect);
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text("9999".into())],
    );
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.vlan_fields[0].0, field);
    assert!(ctx.memory(|m| m.has_focus(field)));
    assert_ne!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .mac_draft,
        before
    );
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .mac_draft,
        before
    );
    assert_eq!(a.metrics.vlan_fields[0].0, field);
}
