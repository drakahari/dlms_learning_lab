//! Native widget regressions for source-bound index editor/terminal, not a novice study.
use super::*;
use dlms_learning_lab::file_index_workspace as work;
fn setup(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-file-index-handover")
        .unwrap();
    a.launch(i);
    let State::FileIndex { simulation } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation.view = view;
    (ctx, a, i)
}
fn full(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1500., 7000., vec![]);
    }
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .file_index
        .controls
        .iter()
        .find(|c| c.0 == key)
        .unwrap()
        .1
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = a
        .metrics
        .file_index
        .fields
        .iter()
        .find(|c| c.0 == key)
        .unwrap()
        .1;
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1500.,
        7000.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    full(ctx, a);
}
#[test]
fn actual_editor_save_and_terminal_enter_repair_partial_results_and_invalidate_after_edit() {
    let (ctx, mut a, i) = setup(teaching::View::Assignment);
    full(&ctx, &mut a);
    let s = a.scenarios[i].clone();
    replace(
        &ctx,
        &mut a,
        "config",
        "PRUNEPATHS=\"/srv/intake/archive\"\n",
    );
    let r = rect(&a, "save");
    click(&ctx, &mut a, r);
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    for c in &s.solution[1..] {
        replace(&ctx, &mut a, "command", c);
        frame(
            &ctx,
            &mut a,
            1500.,
            7000.,
            vec![key(egui::Key::Enter, false)],
        );
        full(&ctx, &mut a);
    }
    let r = rect(&a, "check");
    click(&ctx, &mut a, r);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(a.metrics.completion);
    assert_eq!(
        a.metrics.file_index.result.map(|r| (r.0, r.1)),
        Some((5, 5))
    );
    replace(&ctx, &mut a, "config", "PRUNEPATHS=invalid");
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(!a.metrics.completion);
    assert!(a.metrics.file_index.result.is_none());
    let old = a.progress.attempts[&s.id].state.clone();
    let r = rect(&a, "save");
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts[&s.id].state, old);
    assert!(a.notice.as_ref().is_some_and(|e| e.contains("quoted")));
}
#[test]
fn three_views_keep_independent_work_and_native_controls_with_wide_and_narrow_large_text() {
    for width in [760., 1500.] {
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                let (ctx, mut a, i) = setup(teaching::View::Introduction);
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                if light {
                    a.theme_id = theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone();
                    theme::apply(&ctx, &a.theme_id);
                }
                let source = a.scenarios[i].clone();
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let State::FileIndex { simulation } =
                        &mut a.progress.attempts.get_mut(&source.id).unwrap().state
                    else {
                        panic!()
                    };
                    simulation.view = view;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 12000., vec![]);
                    }
                    assert_eq!(
                        a.metrics.file_index.fields.is_empty(),
                        view == teaching::View::Introduction
                    );
                    for (_, r, _, clip) in &a.metrics.file_index.fields {
                        assert!(
                            r.min.x >= clip.min.x - 1. && r.max.x <= clip.max.x + 1.,
                            "clipped control {r:?}/{clip:?},width{width},scale{scale}"
                        );
                    }
                    assert!(!progress::current_solved(
                        &a.progress.attempts[&source.id],
                        &source
                    ));
                }
                let State::FileIndex { simulation } = &a.progress.attempts[&source.id].state else {
                    panic!()
                };
                let engine::Goal::FileIndex { definition } = &source.goal else {
                    panic!()
                };
                let mut expected = work::Session::new(definition).unwrap();
                expected.view = teaching::View::Assignment;
                assert_eq!(**simulation, expected);
            }
        }
    }
}

#[test]
fn requested_learn_solution_does_not_open_by_default_in_challenge_or_after_reset() {
    fn painted(ctx: &egui::Context, a: &mut LabApp) -> String {
        fn append(shape: &egui::Shape, out: &mut String) {
            match shape {
                egui::Shape::Text(t) => {
                    out.push_str(&t.galley.job.text);
                    out.push('\n');
                }
                egui::Shape::Vec(items) => {
                    for s in items {
                        append(s, out)
                    }
                }
                _ => {}
            }
        }
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 7000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut text = String::new();
        for shape in output.shapes {
            append(&shape.shape, &mut text)
        }
        text
    }
    let (ctx, mut a, _) = setup(teaching::View::Assignment);
    full(&ctx, &mut a);
    let marker = "Compare locate -d /indexes/intake.db runbook";
    assert!(!painted(&ctx, &mut a).contains(marker));
    let r = rect(&a, "help");
    click(&ctx, &mut a, r);
    let r = rect(&a, "worked-help");
    click(&ctx, &mut a, r);
    assert!(painted(&ctx, &mut a).contains(marker));
    let r = a.metrics.mode_challenge.unwrap();
    click(&ctx, &mut a, r);
    assert!(!painted(&ctx, &mut a).contains(marker));
    let r = rect(&a, "help");
    click(&ctx, &mut a, r);
    assert!(!painted(&ctx, &mut a).contains(marker));
    let r = a.metrics.mode_learn.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.reset.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    assert!(!painted(&ctx, &mut a).contains(marker));
}

#[test]
fn worked_commands_wrap_without_horizontal_clipping_and_latest_terminal_output_stays_visible() {
    let worked = "updatedb -l 0 -o /indexes/demo.db -U /gallery --config-file /practice/index.conf";
    for width in [760., 1500.] {
        for scale in [1., 2.] {
            let (ctx, mut a, _) = setup(teaching::View::Introduction);
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 12000., vec![]);
            }
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width / scale, 12000. / scale),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            fn inspect(
                shape: &egui::Shape,
                clip: egui::Rect,
                needle: &str,
                out: &mut Vec<(usize, egui::Rect, egui::Rect)>,
            ) {
                match shape {
                    egui::Shape::Text(t) if t.galley.job.text == needle => out.push((
                        t.galley.rows.len(),
                        t.galley.rect.translate(t.pos.to_vec2()),
                        clip,
                    )),
                    egui::Shape::Vec(v) => {
                        for s in v {
                            inspect(s, clip, needle, out)
                        }
                    }
                    _ => {}
                }
            }
            let mut found = vec![];
            for s in output.shapes {
                inspect(&s.shape, s.clip_rect, worked, &mut found);
            }
            assert_eq!(found.len(), 1);
            let (rows, r, clip) = found[0];
            assert!(
                r.min.x >= clip.min.x - 1. && r.max.x <= clip.max.x + 1.,
                "worked line clipped {width}/{scale}: {r:?}/{clip:?}"
            );
            if width == 760. && scale == 2. {
                assert!(rows > 1);
            }
        }
    }
    let (ctx, mut a, i) = setup(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let attempt = a.progress.attempts.get_mut(&source.id).unwrap();
    for _ in 0..15 {
        engine::execute_for_goal(&mut attempt.state, &source.goal, "cat /work/notes.txt").unwrap();
    }
    engine::execute_for_goal(
        &mut attempt.state,
        &source.goal,
        "cat /srv/intake/current/runbook.txt",
    )
    .unwrap();
    full(&ctx, &mut a);
    let (r, clip) = a.metrics.file_index.last_output.unwrap();
    assert!(
        r.min.y >= clip.min.y - 1. && r.max.y <= clip.max.y + 1.,
        "latest output hidden {r:?}/{clip:?}"
    );
}
