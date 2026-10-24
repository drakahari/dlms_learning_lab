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

#[test]
fn authored_mac_lessons_keep_intro_example_assignment_and_challenge_controls_inside_width() {
    for name in [
        "ccna-mac-learn-paths",
        "ccna-mac-stale-location",
        "ccna-mac-source-refresh",
        "ccna-mac-handover",
    ] {
        for scale in [1., 1.5, 2.] {
            for (width, height) in [(1200., 850.), (850., 600.)] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let (ctx, mut a) = app();
                    let i = a.scenarios.iter().position(|s| s.id == name).unwrap();
                    a.launch(i);
                    a.set_mac_view(i, view);
                    a.scale = scale;
                    ctx.set_zoom_factor(scale);
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, height, vec![]);
                    }
                    let actual = ctx.content_rect();
                    for rect in &a.metrics.inputs {
                        assert!(
                            rect.left() >= actual.left() - 1.
                                && rect.right() <= actual.right() + 1.,
                            "{name} {scale} {view:?}: {rect:?} vs {actual:?}"
                        );
                    }
                    if view == teaching::View::Assignment {
                        assert!(a.metrics.mac_send.is_some());
                        a.progress.attempts.get_mut(name).unwrap().challenge = true;
                        for _ in 0..3 {
                            frame(&ctx, &mut a, width, height, vec![]);
                        }
                        assert!(
                            a.metrics.lesson_tabs.is_empty() && a.metrics.command_insert.is_none()
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn authored_mac_capstone_metrics_identify_real_port_fields_without_replacing_widget_ids() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-mac-handover")
        .unwrap();
    a.launch(i);
    a.set_mac_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.vlan_switch_picker.is_some());
    for (switch, port, kind, rect) in &a.metrics.vlan_named_fields {
        assert_eq!(switch, "core");
        assert!(a.metrics.inputs.contains(rect));
        if kind == &"vlans" {
            assert!(port.is_empty());
        } else {
            assert!(a.metrics.vlan_fields.iter().any(|(_, r)| r == rect));
        }
    }
    assert!(
        a.metrics
            .vlan_named_fields
            .iter()
            .any(|(s, p, k, _)| s == "core" && p == "right" && k == &"allowed")
    );
}

#[test]
fn mac_popup_last_of_six_sources_scrolls_inside_the_actual_popup_before_clicking() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-mac-handover")
        .unwrap();
    a.launch(i);
    a.set_mac_view(i, teaching::View::Assignment);
    // Deliberately small real popup: all six choices are painted but the last
    // cannot be clicked until its own ScrollArea has actually scrolled.
    ctx.style_mut(|style| style.spacing.combo_height = 70.);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.mac_source);
    // Keep the real ComboBox well left of the window centre: a full-window
    // inherited clip would direct wheel events outside this actual popup.
    for _ in 0..4 {
        frame(&ctx, &mut a, 1600., 850., vec![]);
    }
    let before = a.progress.attempts["ccna-mac-handover"].state.clone();
    let target = a
        .metrics
        .mac_sources
        .iter()
        .find(|(id, _)| id == "visitor-right")
        .unwrap()
        .1;
    let clip = a
        .metrics
        .mac_source_popup
        .unwrap()
        .intersect(ctx.content_rect());
    assert!(
        clip.width() < ctx.content_rect().width() * 0.6,
        "actual offcentre ComboBox pane {clip:?}"
    );
    // The app centres its form; the dedicated real-ComboBox fixture below
    // guarantees window-centre X outside the pane. This actual-app case
    // retains its six-source selection, pane-width and wheel-routing checks.
    assert!(
        target.bottom() > clip.bottom() + 1. / ctx.pixels_per_point()
            || target.top() < clip.top() - 1. / ctx.pixels_per_point()
    );
    a.smoke_stage = 19500;
    let mut scrolled = false;
    let mut done = false;
    for _ in 0..24 {
        let target = a
            .metrics
            .mac_sources
            .iter()
            .find(|(id, _)| id == "visitor-right")
            .map(|(_, r)| *r);
        if a.mac_popup_choice(&ctx, target, a.metrics.mac_source_popup) {
            done = true;
            break;
        }
        let events = std::mem::take(&mut a.pending_events);
        if events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            scrolled = true;
            let actual = a
                .metrics
                .mac_source_popup
                .unwrap()
                .intersect(ctx.content_rect());
            let choice_x = a
                .metrics
                .mac_sources
                .iter()
                .map(|(_, r)| *r)
                .reduce(|a, b| a.union(b))
                .unwrap();
            let pointer = events
                .iter()
                .find_map(|e| {
                    if let egui::Event::PointerMoved(p) = e {
                        Some(*p)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert!(
                actual.contains(pointer),
                "wheel must route inside actual clipped popup: {pointer:?} vs{actual:?}"
            );
            assert!(
                pointer.x >= choice_x.left() && pointer.x <= choice_x.right(),
                "wheelX must be inside actual option content, not windowclip"
            );
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, egui::Event::PointerButton { .. }))
            );
        }
        frame(&ctx, &mut a, 1600., 850., events);
        for _ in 0..4 {
            frame(&ctx, &mut a, 1600., 850., vec![]);
        }
    }
    assert!(
        done && scrolled,
        "actual six-choice popup must scroll and finish its click"
    );
    assert_eq!(
        a.progress.attempts["ccna-mac-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .mac_draft
            .as_ref()
            .unwrap()
            .source,
        "visitor-right"
    );
    assert_eq!(a.progress.attempts["ccna-mac-handover"].state, before);
}
#[test]
fn mac_popup_hidden_choice_never_queues_a_click_and_scroll_attempts_are_bounded() {
    let (ctx, mut a, _) = mac_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    a.smoke_stage = 19501;
    let popup = egui::Rect::from_min_max(egui::pos2(50., 100.), egui::pos2(200., 200.));
    let target = egui::Rect::from_min_size(egui::pos2(300., 120.), egui::vec2(40., 25.));
    for _ in 0..20 {
        assert!(!a.mac_popup_choice(&ctx, Some(target), Some(popup)));
        assert!(
            a.pending_events
                .iter()
                .all(|e| !matches!(e, egui::Event::PointerButton { .. }))
        );
        a.pending_events.clear();
    }
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.mac_popup_choice(
            &ctx,
            Some(target),
            Some(popup)
        )))
        .is_err()
    );
    assert!(a.pending_events.is_empty());
}

#[test]
fn mac_rejected_send_notice_is_distinct_from_true_current_completion() {
    let (ctx, mut a, i) = mac_app(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    for input in &s.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(&s.id).unwrap().state,
            &s.goal,
            input,
        )
        .unwrap();
    }
    a.checked = true;
    draw(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    let applied = a.progress.attempts[&s.id].state.clone();
    accounts_ui_click(&ctx, &mut a, |m| m.mac_destination);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Text("not-a-mac".into()),
        ],
    );
    accounts_ui_click(&ctx, &mut a, |m| m.mac_send);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&s.id].state, applied);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    // Reveal the actual diagnostic/completion block without clicking its
    // labels or any Next action. The grade and exact applied State above
    // already prove that rejected input did not change objective truth.
    for _ in 0..20 {
        let rect = a.metrics.feedback.expect("actual feedback block");
        let visible = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if rect.top() >= visible.top() && rect.bottom() <= visible.bottom() {
            break;
        }
        let delta = if rect.bottom() > visible.bottom() {
            visible.bottom() - rect.bottom()
        } else {
            visible.top() - rect.top()
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(visible.left() + 1., visible.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..4 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
    }
    let text = mac_rendered_text(&ctx, &mut a);
    assert!(
        text.contains("Frame not sent: Use six hexadecimal pairs"),
        "{text}"
    );
    assert!(
        text.contains("Objective complete · All current checks pass."),
        "{text}"
    );
    assert!(a.metrics.completion);
}

#[test]
fn mac_related_native_form_controls_share_columns_at_all_scales_and_widths() {
    for scale in [1., 1.5, 2.] {
        for (width, height) in [(1200., 850.), (850., 600.)] {
            let (ctx, mut a, _) = mac_app(teaching::View::Assignment);
            draw(&ctx, &mut a);
            accounts_ui_click(&ctx, &mut a, |m| m.mac_tools);
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for _ in 0..4 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            let source = a.metrics.mac_source.unwrap();
            let destination = a.metrics.mac_destination.unwrap();
            let inspect = a.metrics.mac_inspect.unwrap();
            assert!(
                (source.left() - destination.left()).abs() <= 1.,
                "From/Destination {scale} {width}: {source:?}/{destination:?}"
            );
            assert!(
                (inspect.left() - destination.left()).abs() <= 1.,
                "Inspect/Destination {scale} {width}: {inspect:?}/{destination:?}"
            );
            let clear = a.metrics.mac_clear_switch.unwrap();
            let time = a.metrics.mac_time.unwrap();
            assert!(
                (clear.left() - time.left()).abs() <= 1.,
                "Clear/Advance in tools {scale} {width}: {clear:?}/{time:?}"
            );
        }
    }
}

#[test]
fn mac_real_offcentre_combo_routes_wheels_inside_its_narrow_pane_and_selects_sixth() {
    fn popup_frame(
        ctx: &egui::Context,
        a: &mut LabApp,
        values: &[String],
        events: Vec<egui::Event>,
    ) -> (egui::Rect, Vec<(String, egui::Rect)>, Option<egui::Rect>) {
        let mut button = None;
        let mut choices = Vec::new();
        let mut pane = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 850.),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::Area::new(egui::Id::new("fixed-real-six-source-combo"))
                    .fixed_pos(egui::pos2(40., 80.))
                    .movable(false)
                    .show(ctx, |ui| {
                        let source = &mut a
                            .progress
                            .attempts
                            .get_mut("ccna-mac-handover")
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .mac_draft
                            .as_mut()
                            .unwrap()
                            .source;
                        let r = egui::ComboBox::from_id_salt("real-six-source-popup")
                            .width(160.)
                            .height(70.)
                            .selected_text(source.as_str())
                            .show_ui(ui, |ui| {
                                for value in values {
                                    let row = ui.selectable_value(source, value.clone(), value);
                                    choices.push((value.clone(), row.rect));
                                }
                                pane = Some(controls::popup_content_clip(ui));
                            });
                        button = Some(r.response.rect);
                    });
            },
        );
        (button.unwrap(), choices, pane)
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-mac-handover")
        .unwrap();
    a.launch(i);
    let State::MacLearning { simulation } = &a.progress.attempts["ccna-mac-handover"].state else {
        panic!()
    };
    let values = simulation
        .fabric()
        .hosts
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 6);
    let before = a.progress.attempts["ccna-mac-handover"].state.clone();
    let (button, _, _) = popup_frame(&ctx, &mut a, &values, vec![]);
    popup_frame(
        &ctx,
        &mut a,
        &values,
        vec![egui::Event::PointerMoved(button.center())],
    );
    let click = [true, false]
        .into_iter()
        .map(|pressed| egui::Event::PointerButton {
            pos: button.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        });
    popup_frame(&ctx, &mut a, &values, click.collect());
    let (_, mut choices, mut pane) = popup_frame(&ctx, &mut a, &values, vec![]);
    let clip = pane
        .expect("real popup is open")
        .intersect(ctx.content_rect());
    assert!(clip.width() < ctx.content_rect().width() / 2.);
    assert!(
        ctx.content_rect().center().x > clip.right() + 1.,
        "window centreX must miss this real offcentre popup"
    );
    let sixth = values.last().unwrap();
    let row = choices.iter().find(|(x, _)| x == sixth).unwrap().1;
    assert!(
        row.bottom() > clip.bottom() + 1. / ctx.pixels_per_point(),
        "sixth actualrow starts clipped"
    );
    a.smoke_stage = 19502;
    let mut done = false;
    let mut scrolled = false;
    for _ in 0..24 {
        let target = choices.iter().find(|(x, _)| x == sixth).map(|(_, r)| *r);
        if a.mac_popup_choice(&ctx, target, pane) {
            done = true;
            break;
        }
        let events = std::mem::take(&mut a.pending_events);
        if events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            scrolled = true;
            let current = pane.unwrap().intersect(ctx.content_rect());
            let pointer = events
                .iter()
                .find_map(|e| {
                    if let egui::Event::PointerMoved(p) = e {
                        Some(*p)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert!(
                current.contains(pointer),
                "wheel must route inside actual narrow popup"
            );
            assert!(ctx.content_rect().center().x > current.right() + 1.);
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, egui::Event::PointerButton { .. })),
                "no clipped-row click"
            );
        }
        let result = popup_frame(&ctx, &mut a, &values, events);
        choices = result.1;
        pane = result.2;
        for _ in 0..4 {
            let result = popup_frame(&ctx, &mut a, &values, vec![]);
            choices = result.1;
            pane = result.2;
        }
    }
    assert!(
        done && scrolled,
        "actual sixth selection must follow popup scrolling"
    );
    assert_eq!(
        &a.progress.attempts["ccna-mac-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .mac_draft
            .as_ref()
            .unwrap()
            .source,
        sixth
    );
    assert_eq!(a.progress.attempts["ccna-mac-handover"].state, before);
}
