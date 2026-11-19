use super::*;
use dlms_learning_lab::rest_workspace as work;
const IDS: [&str; 2] = ["ccna-rest-change-request", "ccna-rest-profile-handover"];
fn view(app: &mut LabApp, id: &str, view: teaching::View) {
    let State::RestController { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    app.lesson_scroll_reset = true;
}

#[test]
fn rest_request_fields_and_actions_reflow_at_all_supported_scales_in_both_themes() {
    for id in IDS {
        let (ctx, mut app, _) = super::nm_lessons::opened(id);
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for light in [false, true] {
                for width in [760., 1600.] {
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
                    for mode in [
                        teaching::View::Introduction,
                        teaching::View::Example,
                        teaching::View::Assignment,
                    ] {
                        view(&mut app, id, mode);
                        for _ in 0..3 {
                            frame(&ctx, &mut app, width, 7000., vec![]);
                        }
                        assert!((ctx.zoom_factor() - scale).abs() < 0.01);
                        if mode == teaching::View::Introduction {
                            assert!(
                                !app.metrics
                                    .rest_controller
                                    .controls
                                    .iter()
                                    .any(|r| r.0 == "body")
                            );
                            continue;
                        }
                        for key in [
                            "method",
                            "path",
                            "authorization",
                            "content-type",
                            "body",
                            "send",
                            "check",
                        ] {
                            let (_, r, c) = app
                                .metrics
                                .rest_controller
                                .controls
                                .iter()
                                .find(|r| r.0 == key)
                                .unwrap();
                            assert!(
                                r.width() > 0.
                                    && r.left() >= c.left() - 1.
                                    && r.right() <= c.right() + 1.,
                                "{id} {scale} {width} {light} {mode:?} {key} {r:?} {c:?}"
                            );
                        }
                        let rows = &app.metrics.rest_controller.controls;
                        let path = rows.iter().find(|r| r.0 == "path").unwrap().1;
                        for key in ["authorization", "content-type", "send", "check"] {
                            let r = rows.iter().find(|r| r.0 == key).unwrap().1;
                            assert!(
                                (r.height() - path.height()).abs() < 1.1,
                                "{key} {r:?} {path:?}"
                            );
                        }
                        assert!(
                            !rows.iter().any(|r| r.0 == "result"),
                            "Unrun work must not expose an invented result"
                        );
                    }
                }
            }
        }
        app.progress.attempts.get_mut(id).unwrap().challenge = true;
        frame(&ctx, &mut app, 1200., 7000., vec![]);
        assert!(
            !app.metrics
                .rest_controller
                .controls
                .iter()
                .any(|r| matches!(r.0.as_str(), "example" | "introduction"))
        );
    }
}

#[test]
fn rest_current_results_fail_clearly_and_editing_invalidates_a_real_success() {
    for id in IDS {
        let (ctx, mut app, index) = super::nm_lessons::opened(id);
        let source = app.scenarios[index].clone();
        let def = work::source(&source).unwrap();
        view(&mut app, id, teaching::View::Assignment);
        let State::RestController { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.assignment.test(&def.initial).unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, 1600., 7000., vec![]);
        }
        assert!(!app.metrics.completion);
        assert!(!app.metrics.rest_controller.shown_outcomes.is_empty());
        assert!(
            app.metrics
                .rest_controller
                .shown_outcomes
                .iter()
                .all(|c| !c.0)
        );
        let State::RestController { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        for request in &def.initial.reference {
            simulation.assignment.edit(request.clone()).unwrap();
            simulation.assignment.send(&def.initial).unwrap();
        }
        simulation.assignment.test(&def.initial).unwrap();
        app.completion_index.forget(id);
        frame(&ctx, &mut app, 1600., 7000., vec![]);
        assert!(app.metrics.completion);
        assert!(app.metrics.rest_controller.shown_outcomes.is_empty());
        let State::RestController { simulation } =
            &mut app.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        let mut request = simulation.assignment.request.clone();
        request.body = "{ unfinished".into();
        simulation.assignment.edit(request).unwrap();
        app.completion_index.forget(id);
        frame(&ctx, &mut app, 1600., 7000., vec![]);
        assert!(!app.metrics.completion);
        assert!(
            !app.metrics
                .rest_controller
                .controls
                .iter()
                .any(|r| r.0 == "result")
        );
    }
}

#[test]
fn rest_body_tab_reaches_send_and_enter_runs_only_the_simulated_request() {
    let id = IDS[0];
    let (ctx, mut app, _) = super::nm_lessons::opened(id);
    view(&mut app, id, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    let body = app
        .metrics
        .rest_controller
        .ids
        .iter()
        .find(|r| r.0 == "body")
        .unwrap()
        .1;
    let send = app
        .metrics
        .rest_controller
        .ids
        .iter()
        .find(|r| r.0 == "send")
        .unwrap()
        .1;
    let viewport = app
        .metrics
        .rest_controller
        .controls
        .iter()
        .find(|r| r.0 == "body")
        .unwrap()
        .1;
    ctx.memory_mut(|m| m.request_focus(body));
    frame(&ctx, &mut app, 1600., 7000., vec![]);
    assert_eq!(ctx.memory(|m| m.focused()), Some(body));
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
            7000.,
            vec![key(egui::Key::Tab, true), key(egui::Key::Tab, false)],
        );
        frame(&ctx, &mut app, 1600., 7000., vec![]);
        let focused = ctx.memory(|m| m.focused());
        assert_ne!(
            focused,
            Some(body),
            "Tab must leave the editor rather than indenting"
        );
        if focused == Some(send) {
            reached = true;
            break;
        }
        let response = ctx
            .read_response(focused.expect("keyboard target"))
            .expect("actual widget");
        assert!(
            viewport.expand(2.).intersects(response.rect),
            "Only editor scrollbars may intervene before Send request"
        );
    }
    assert!(reached, "Send request is keyboard reachable");
    frame(
        &ctx,
        &mut app,
        1600.,
        7000.,
        vec![key(egui::Key::Enter, true), key(egui::Key::Enter, false)],
    );
    frame(&ctx, &mut app, 1600., 7000., vec![]);
    assert!(
        app.notice
            .as_ref()
            .unwrap()
            .starts_with("Actual response · 403 Forbidden")
    );
    let State::RestController { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    assert!(simulation.assignment.changes.is_empty());
    assert!(!app.metrics.completion);
}

#[test]
fn rest_guidance_teaches_json_before_a_success_and_retirement_before_deletion() {
    for id in IDS {
        let (_, app, index) = super::nm_lessons::opened(id);
        let task = &work::source(&app.scenarios[index]).unwrap().example;
        let mut draft = work::Draft::new(task);
        assert_eq!(rest_controller_page::practice_level(task, &draft), 0);
        if id == IDS[0] {
            let mut pending = draft.request.clone();
            pending.authorization = "bearer lab-operator".into();
            pending.content_type = "Application/Merge-Patch+Json".into();
            draft.edit(pending).unwrap();
            assert!(draft.changes.is_empty());
            assert!(!draft.checked);
            assert_eq!(
                rest_controller_page::practice_level(task, &draft),
                1,
                "Teach the boolean while the quoted value is still wrong"
            );
            assert_eq!(draft.send(task).unwrap().status, 422);
            assert_eq!(rest_controller_page::practice_level(task, &draft), 1);
            draft.edit(task.reference[0].clone()).unwrap();
            draft.send(task).unwrap();
            assert_eq!(
                rest_controller_page::practice_level(task, &draft),
                1,
                "Guide checking before suggesting another failure"
            );
            draft.test(task).unwrap();
            assert_eq!(rest_controller_page::practice_level(task, &draft), 2);
        } else {
            for (i, r) in task.reference.iter().enumerate() {
                draft.edit(r.clone()).unwrap();
                draft.send(task).unwrap();
                if i < 2 {
                    assert_eq!(rest_controller_page::practice_level(task, &draft), 1);
                }
                if i == 2 {
                    assert!(draft.store(task).unwrap().profiles.contains_key("guests"));
                    assert_eq!(
                        rest_controller_page::practice_level(task, &draft),
                        2,
                        "Teach retirement while the old unused profile still exists"
                    );
                }
            }
            draft.test(task).unwrap();
            assert_eq!(rest_controller_page::practice_level(task, &draft), 2);
        }
    }
}

#[test]
fn rest_type_errors_remain_readable_without_opening_or_scrolling_raw_json() {
    fn painted(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),
            egui::Shape::Vec(v) => {
                for shape in v {
                    painted(shape, out)
                }
            }
            _ => {}
        }
    }
    let id = IDS[0];
    let (ctx, mut app, index) = super::nm_lessons::opened(id);
    view(&mut app, id, teaching::View::Example);
    let task = &work::source(&app.scenarios[index]).unwrap().example;
    let State::RestController { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    let before = simulation.practice.store(task).unwrap();
    let mut request = simulation.practice.request.clone();
    request.authorization = "Bearer lab-operator".into();
    request.content_type = "application/merge-patch+json".into();
    simulation.practice.edit(request).unwrap();
    let response = simulation.practice.send(task).unwrap();
    assert_eq!(response.status, 422);
    assert!(response.body.contains("expected a boolean"));
    assert_eq!(simulation.practice.store(task).unwrap(), before);
    app.notice = Some(rest_controller_page::response_notice(response));
    for scale in [1., 2.] {
        for light in [false, true] {
            for width in [760., 1600.] {
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
                for _ in 0..3 {
                    frame(&ctx, &mut app, width, 7000., vec![]);
                }
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 7000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.render(ctx),
                );
                let mut words = Vec::new();
                for shape in &out.shapes {
                    painted(&shape.shape, &mut words);
                }
                assert!(
                    words
                        .iter()
                        .any(|s| s.contains("true or false without quotes")),
                    "The cause must be painted without raw-response expansion"
                );
                assert!(
                    !words.iter().any(|s| s.contains("invalid type:")),
                    "Raw encoded JSON is optional detail"
                );
                let (_, rect, clip) = app
                    .metrics
                    .rest_controller
                    .controls
                    .iter()
                    .find(|r| r.0 == "diagnostic")
                    .unwrap();
                assert!(
                    clip.expand(1.).contains_rect(*rect),
                    "{width} {scale} {light}: {rect:?} {clip:?}"
                );
                assert!(!app.metrics.completion);
            }
        }
    }
    let State::RestController { simulation } =
        &mut app.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    let mut oversized = simulation.practice.request.clone();
    oversized.body = "x".repeat(4097);
    app.notice = Some(simulation.practice.edit(oversized).unwrap_err());
    frame(&ctx, &mut app, 760., 7000., vec![]);
    let (_, rect, clip) = app
        .metrics
        .rest_controller
        .controls
        .iter()
        .find(|r| r.0 == "diagnostic")
        .unwrap();
    assert!(clip.expand(1.).contains_rect(*rect));
    assert!(
        app.metrics
            .rest_controller
            .shown_diagnostic
            .as_ref()
            .unwrap()
            .contains("4KiB")
    );
}
