use super::*;
use dlms_learning_lab::json_inventory_workspace as work;
fn set_view(app: &mut LabApp, id: &str, view: teaching::View) {
    let State::JsonInventory { simulation } = &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    app.lesson_scroll_reset = true;
}
#[test]
fn inventory_editors_actions_and_guidance_reflow_across_supported_scales() {
    for id in ["ccna-json-port-records", "ccna-json-inventory-handover"] {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
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
                set_view(&mut app, id, view);
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 1080., vec![]);
                }
                if view == teaching::View::Introduction {
                    assert!(
                        !app.metrics
                            .json_inventory
                            .controls
                            .iter()
                            .any(|r| r.0 == "editor")
                    );
                    continue;
                }
                let rows = &app.metrics.json_inventory.controls;
                for key in ["editor", "apply", "check"] {
                    let (_, rect, clip) = rows.iter().find(|r| r.0 == key).unwrap();
                    assert!(
                        rect.width() > 0.
                            && rect.left() >= clip.left() - 1.
                            && rect.right() <= clip.right() + 1.,
                        "{id} {view:?} {scale} {key} {rect:?} {clip:?}"
                    );
                }
                let editor = rows.iter().find(|r| r.0 == "editor").unwrap().1;
                let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                let expected = ctx.fonts_mut(|fonts| fonts.row_height(&font)).ceil() * 10.
                    + 2. * f32::from(controls::PAD_Y);
                assert!(
                    editor.height() <= expected + 1.,
                    "Editor must scroll rather than grow: {editor:?} > {expected}"
                );
                let apply = rows.iter().find(|r| r.0 == "apply").unwrap().1;
                let check = rows.iter().find(|r| r.0 == "check").unwrap().1;
                assert!((apply.height() - check.height()).abs() < 1.);
            }
        }
        // Challenge forces the complete assignment without example/reference tabs.
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        for _ in 0..3 {
            frame(&ctx, &mut app, 1200., 950., vec![]);
        }
        assert!(
            !app.metrics
                .json_inventory
                .controls
                .iter()
                .any(|r| matches!(r.0.as_str(), "example" | "introduction"))
        );
        let State::JsonInventory { simulation } = &app.progress.attempts[id].state else {
            panic!()
        };
        let def = work::source(&app.scenarios[index]).unwrap();
        assert_eq!(simulation.assignment, work::Draft::new(&def.initial));
    }
}
#[test]
fn successful_result_is_invalidated_on_current_document_edit() {
    let id = "ccna-json-port-records";
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let source = app.scenarios[index].clone();
    let def = work::source(&source).unwrap();
    {
        let State::JsonInventory { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        simulation
            .assignment
            .edit(def.initial.reference.clone())
            .unwrap();
        simulation.assignment.apply(&def.initial).unwrap();
        simulation.assignment.test(&def.initial).unwrap();
    }
    frame(&ctx, &mut app, 1200., 950., vec![]);
    assert!(app.metrics.completion);
    {
        let State::JsonInventory { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.assignment.edit("{ unfinished".into()).unwrap();
    }
    app.completion_index.forget(id);
    frame(&ctx, &mut app, 1200., 950., vec![]);
    assert!(!app.metrics.completion);
    assert!(
        !app.metrics
            .json_inventory
            .controls
            .iter()
            .any(|r| r.0 == "result")
    );
}

#[test]
fn tab_leaves_the_code_editor_for_the_apply_action() {
    let id = "ccna-json-port-records";
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    set_view(&mut app, id, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1600., 1600., vec![]);
    }
    let editor = app
        .metrics
        .json_inventory
        .ids
        .iter()
        .find(|r| r.0 == "editor")
        .unwrap()
        .1;
    let apply = app
        .metrics
        .json_inventory
        .ids
        .iter()
        .find(|r| r.0 == "apply")
        .unwrap()
        .1;
    ctx.memory_mut(|m| m.request_focus(editor));
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    assert_eq!(ctx.memory(|m| m.focused()), Some(editor));
    // Native scrollbars are keyboard targets between a scrolling editor and
    // Apply. Tab must leave the editor, traverse only its scrollbars, and reach
    // Apply without inserting indentation or trapping focus.
    let key = |key, pressed| egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    };
    let mut reached = false;
    for _ in 0..4 {
        frame(
            &ctx,
            &mut app,
            1600.,
            1600.,
            vec![key(egui::Key::Tab, true), key(egui::Key::Tab, false)],
        );
        frame(&ctx, &mut app, 1600., 1600., vec![]);
        let focused = ctx.memory(|m| m.focused());
        assert_ne!(focused, Some(editor), "Tab must leave the editor");
        if focused == Some(apply) {
            reached = true;
            break;
        }
        let response = ctx
            .read_response(focused.expect("keyboard target"))
            .expect("actual native widget");
        let viewport = app
            .metrics
            .json_inventory
            .controls
            .iter()
            .find(|r| r.0 == "editor")
            .unwrap()
            .1;
        assert!(
            viewport.expand(2.).intersects(response.rect),
            "Unexpected focus stop: {:?}",
            response.rect
        );
    }
    assert!(
        reached,
        "Tab must reach Apply after the native editor scrollbars"
    );
    frame(
        &ctx,
        &mut app,
        1600.,
        1600.,
        vec![key(egui::Key::Enter, true), key(egui::Key::Enter, false)],
    );
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    assert!(
        app.notice
            .as_ref()
            .unwrap()
            .contains("Document field error"),
        "Enter must activate Apply"
    );
    let State::JsonInventory { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    let source = app.scenarios.iter().find(|s| s.id == id).unwrap();
    let def = work::source(source).unwrap();
    assert_eq!(simulation.assignment.source, def.initial.starter);
}

#[test]
fn failures_are_shown_first_and_guidance_tracks_validity_and_actual_outcomes() {
    let id = "ccna-json-inventory-handover";
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    let source = app.scenarios[index].clone();
    let def = work::source(&source).unwrap();
    set_view(&mut app, id, teaching::View::Example);
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    assert_eq!(app.metrics.json_inventory.practice_level, Some(0));
    {
        let State::JsonInventory { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation
            .practice
            .edit(def.example.initial_applied.clone())
            .unwrap();
    }
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    assert_eq!(app.metrics.json_inventory.practice_level, Some(1));
    {
        let State::JsonInventory { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation
            .practice
            .edit(def.example.reference.clone())
            .unwrap();
        simulation.practice.apply(&def.example).unwrap();
        simulation.practice.test(&def.example).unwrap();
    }
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    assert_eq!(app.metrics.json_inventory.practice_level, Some(2));
    assert!(!app.metrics.completion);
    set_view(&mut app, id, teaching::View::Assignment);
    {
        let mut doc: serde_json::Value = serde_json::from_str(&def.initial.reference).unwrap();
        for sw in doc["switches"].as_array_mut().unwrap() {
            sw["ports"][2]["vlan"] = 10.into();
        }
        let State::JsonInventory { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation
            .assignment
            .edit(serde_json::to_string_pretty(&doc).unwrap())
            .unwrap();
        simulation.assignment.apply(&def.initial).unwrap();
        simulation.assignment.test(&def.initial).unwrap();
    }
    frame(&ctx, &mut app, 1600., 1600., vec![]);
    let shown = &app.metrics.json_inventory.shown_outcomes;
    assert_eq!(shown.len(), 3);
    assert!(shown.iter().all(|c| !c.0));
    assert!(shown.iter().any(|c| c.1.contains("99")));
    assert!(!app.metrics.completion);
}

#[test]
fn capture_rechecks_the_real_editor_clip_after_settling_before_requesting_an_image() {
    let id = "ccna-json-port-records";
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    app.scale = 2.;
    ctx.set_zoom_factor(2.);
    set_view(&mut app, id, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 946., 929., vec![]);
    }
    assert!(crate::ui::smoke::json_inventory::test_late_editor_capture(
        &mut app, &ctx
    ));
}
