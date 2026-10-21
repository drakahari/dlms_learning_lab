use super::*;
fn app() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context(&ctx, true, false);
    app.smoke = false;
    (ctx, app)
}
fn frame(ctx: &egui::Context, app: &mut LabApp, width: f32, height: f32, events: Vec<egui::Event>) {
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / app.scale, height / app.scale),
            )),
            modifiers: events
                .iter()
                .rev()
                .find_map(|e| {
                    if let egui::Event::Key { modifiers, .. } = e {
                        Some(*modifiers)
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            events,
            ..Default::default()
        },
        |ctx| app.render(ctx),
    );
}
fn click(ctx: &egui::Context, app: &mut LabApp, rect: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            1400.,
            1800.,
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
}
#[test]
fn library_controls_share_baselines_heights_and_fit_at_every_scale() {
    for width in [720., 980., 1400.] {
        for scale in [0.85, 1., 1.25, 1.5] {
            let (ctx, mut a) = app();
            a.scale = scale;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 900., vec![]);
            }
            let bounds = ctx.input(|i| i.content_rect());
            for rect in &a.metrics.header {
                assert!(
                    rect.right() <= bounds.right() + 1.,
                    "header overflow {width}/{scale}: {rect:?} in {bounds:?}"
                );
            }
            assert_eq!(a.metrics.filters.len(), 5);
            let h = a.metrics.filters[0].height();
            for r in &a.metrics.filters {
                assert!(
                    (r.height() - h).abs() < 1.,
                    "height {width}/{scale}: {:?}",
                    a.metrics.filters
                );
                assert!(
                    r.right() <= bounds.right() + 1.,
                    "filter overflow {width}/{scale} {r:?} {bounds:?}"
                );
            }
            for pair in a.metrics.filters.windows(2) {
                if (pair[0].top() - pair[1].top()).abs() < h {
                    assert!((pair[0].top() - pair[1].top()).abs() < 1., "baseline");
                }
            }
            assert!(!a.metrics.pagination);
            for (start, fav) in &a.metrics.card_actions {
                assert!((start.top() - fav.top()).abs() < 1.);
                assert!((start.height() - fav.height()).abs() < 1.);
                assert!(fav.right() <= bounds.right() + 1.);
            }
        }
    }
}
#[test]
fn native_workspaces_render_at_narrow_and_wide_widths_without_overflow() {
    for index in [0, 1, 3, 6] {
        for width in [480., 720., 1400.] {
            let (ctx, mut a) = app();
            a.launch(index);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 1600., vec![]);
            }
            for r in &a.metrics.inputs {
                assert!(
                    r.right() <= width + 1.,
                    "native field overflow at {width}: {r:?}"
                );
            }
            if let (Some(output), Some(input)) = (a.metrics.output, a.metrics.terminal) {
                assert!(
                    input.top() - output.bottom() < 25.,
                    "terminal input stays next to output"
                );
                assert!(output.height() >= 200.);
                assert!(input.right() <= width + 1.);
            }
            assert_eq!(a.metrics.split, width >= 996.);
        }
    }
}
#[test]
fn challenge_hides_guide_and_solutions_until_objective_success() {
    let (ctx, mut a) = app();
    a.launch(6);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .challenge = true;
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(!a.metrics.guide);
    assert!(!a.metrics.completion);
    assert!(a.metrics.insert.is_none());
    assert_eq!(a.progress.attempts["lpic2-service"].hints, 0);
}
#[test]
fn insert_populates_but_does_not_execute() {
    let (ctx, mut a) = app();
    a.launch(3);
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    let state = a.progress.attempts["lpic1-permissions"].state.clone();
    let count = a.progress.attempts["lpic1-permissions"].transcript.len();
    let rect = a.metrics.insert.unwrap();
    click(&ctx, &mut a, rect);
    assert_eq!(a.input, "id alex");
    assert_eq!(a.progress.attempts["lpic1-permissions"].state, state);
    assert_eq!(
        a.progress.attempts["lpic1-permissions"].transcript.len(),
        count
    );
}
#[test]
fn wrong_partial_complete_at_first_section_then_broken_again() {
    let (ctx, mut a) = app();
    a.launch(0);
    a.checked = true;
    if let engine::State::Subnet { answers, .. } =
        &mut a.progress.attempts.get_mut("ccna-ipv4").unwrap().state
    {
        answers[0] = "999.1.1.1".into();
        answers[1] = "192.168.10.95".into();
    }
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(!a.metrics.completion);
    let checks = engine::checks(
        &a.progress.attempts["ccna-ipv4"].state,
        &a.scenarios[0].goal,
    );
    assert_eq!(checks.iter().filter(|x| x.0).count(), 1);
    if let engine::State::Subnet { answers, .. } =
        &mut a.progress.attempts.get_mut("ccna-ipv4").unwrap().state
    {
        *answers = [
            "192.168.10.64".into(),
            "192.168.10.95".into(),
            "192.168.10.65".into(),
            "192.168.10.94".into(),
        ];
    }
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(a.metrics.completion);
    assert_eq!(a.progress.attempts["ccna-ipv4"].step, 0);
    if let engine::State::Subnet { answers, .. } =
        &mut a.progress.attempts.get_mut("ccna-ipv4").unwrap().state
    {
        answers[3] = "192.168.10.95".into();
    }
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(!a.metrics.completion);
    assert!(a.invalidated);
}
#[test]
fn config_editor_save_is_separate_from_runtime_and_preserves_failures() {
    let (ctx, mut a) = app();
    a.launch(6);
    for _ in 0..2 {
        frame(&ctx, &mut a, 1400., 1800., vec![]);
    }
    a.service_draft = Some("8080".into());
    let rect = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, rect);
    assert!(
        matches!(&a.progress.attempts["lpic2-service"].state,engine::State::Service{port,running:false,..}if port=="8080")
    );
    assert!(!a.metrics.completion);
}
#[test]
fn synthetic_catalog_requires_developer_option_and_paginated_filters_work() {
    let (ctx, mut a) = app();
    a.synthetic = true;
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(!a.metrics.pagination);
    a.developer = true;
    a.page = 74;
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(a.metrics.pagination);
    assert_eq!(a.page, 74);
    a.filter.track = "CCNA".into();
    a.filter.search = "impossible xyz".into();
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert!(!a.metrics.pagination);
    assert_eq!(a.page, 0);
}

#[test]
fn reset_and_resume_keep_other_labs_favorites_and_native_drafts() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.launch(0);
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    frame(
        &ctx,
        &mut a,
        1400.,
        1800.,
        vec![egui::Event::Text("192.168.10.64".into())],
    );
    a.progress.favorites.insert("ccna-ipv4".into());
    a.progress.attempts.get_mut("ccna-ipv4").unwrap().step = 2;
    a.launch(1);
    if let engine::State::Vlsm { allocations, .. } =
        &mut a.progress.attempts.get_mut("ccna-vlsm").unwrap().state
    {
        allocations[0].cidr = "10.20.".into();
    }
    a.save();
    assert!(a.storage_error.is_none());
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(p.attempts["ccna-ipv4"].step, 2);
    assert!(
        matches!(&p.attempts["ccna-ipv4"].state,engine::State::Subnet{answers,..}if answers[0]=="192.168.10.64")
    );
    let other = p.attempts["ccna-vlsm"].state.clone();
    a.reset_attempt(0);
    a.save();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(p.attempts["ccna-ipv4"].state, a.scenarios[0].initial);
    assert_eq!(p.attempts["ccna-vlsm"].state, other);
    assert!(p.favorites.contains("ccna-ipv4"));
    a.launch(6);
    a.service_draft = Some("8080".into());
    a.reset_attempt(6);
    assert_eq!(a.service_draft, Some("eighty".into()));
}
#[test]
fn native_answer_fields_have_working_tab_focus_and_do_not_submit_commands() {
    let (ctx, mut a) = app();
    a.launch(0);
    for _ in 0..2 {
        frame(&ctx, &mut a, 1400., 1800., vec![]);
    }
    let first = a.metrics.focus;
    assert_eq!(ctx.memory(|m| m.focused()), first);
    frame(
        &ctx,
        &mut a,
        1400.,
        1800.,
        vec![egui::Event::Text("192.168.10.64".into())],
    );
    let key = |shift| egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            ..Default::default()
        },
    };
    frame(&ctx, &mut a, 1400., 1800., vec![key(false)]);
    assert_ne!(ctx.memory(|m| m.focused()), first);
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    frame(&ctx, &mut a, 1400., 1800., vec![key(true)]);
    frame(&ctx, &mut a, 1400., 1800., vec![]);
    assert_eq!(ctx.memory(|m| m.focused()), first);
    assert_eq!(a.progress.attempts["ccna-ipv4"].transcript.len(), 1);
    assert!(
        matches!(&a.progress.attempts["ccna-ipv4"].state,engine::State::Subnet{answers,..}if answers[0]=="192.168.10.64")
    );
}

#[test]
fn existing_library_reflows_after_resizing_and_zoom_changes() {
    let (ctx, mut a) = app();
    for (width, scale) in [
        (1400., 1.),
        (720., 1.5),
        (980., 1.25),
        (1400., 0.85),
        (1268., 1.5),
    ] {
        a.scale = scale;
        for _ in 0..4 {
            frame(&ctx, &mut a, width, 900., vec![]);
        }
        let bounds = ctx.input(|i| i.content_rect());
        for r in &a.metrics.filters {
            assert!(
                r.right() <= bounds.right() + 1.,
                "dynamic filter clip: {width}/{scale} {r:?}, bounds {bounds:?}"
            );
        }
    }
}

#[test]
fn ipv4_worked_results_require_predictions_and_challenge_hides_prior_review() {
    use dlms_learning_lab::ipv4::{Exercise, Section};
    let (ctx, mut a) = app();
    a.launch(0);
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    a.progress
        .attempts
        .get_mut("ccna-ipv4")
        .unwrap()
        .ipv4
        .as_mut()
        .unwrap()
        .section = Section::Network;
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(a.metrics.ipv4_explain);
    assert!(!a.metrics.ipv4_worked);
    let lesson = a.progress.attempts["ccna-ipv4"].ipv4.as_ref().unwrap();
    let e = Exercise::from_state(&a.progress.attempts["ccna-ipv4"].state).unwrap();
    assert!(!lesson.can_reveal(Section::Network, &e));
    if let engine::State::Subnet { answers, .. } =
        &mut a.progress.attempts.get_mut("ccna-ipv4").unwrap().state
    {
        answers[0] = "192.168.10.77".into();
    }
    let e = Exercise::from_state(&a.progress.attempts["ccna-ipv4"].state).unwrap();
    a.progress
        .attempts
        .get_mut("ccna-ipv4")
        .unwrap()
        .ipv4
        .as_mut()
        .unwrap()
        .submit(&e);
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(!a.metrics.ipv4_worked);
    // Model the same explicit review action as the native Reveal button; requests remain per-result.
    assert!(
        a.progress
            .attempts
            .get_mut("ccna-ipv4")
            .unwrap()
            .ipv4
            .as_mut()
            .unwrap()
            .reveal(Section::Network, &e)
    );
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(a.metrics.ipv4_worked);
    assert_eq!(
        a.progress.attempts["ccna-ipv4"]
            .ipv4
            .as_ref()
            .unwrap()
            .revealed,
        [true, false, false]
    );
    a.progress.attempts.get_mut("ccna-ipv4").unwrap().challenge = true;
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(!a.metrics.ipv4_worked);
    assert!(!a.metrics.guide);
    assert!(!a.metrics.ipv4_bits.iter().any(|(_, _, sandbox)| *sandbox));
}
#[test]
fn ipv4_octets_and_native_fields_fit_prefixes_at_narrow_widths_and_scales() {
    for prefix in [8, 16, 20, 23, 27, 30] {
        for (width, scale) in [(480., 1.), (720., 1.5), (1400., 1.), (1400., 1.5)] {
            let (ctx, mut a) = app();
            a.scale = scale;
            a.launch(0);
            if let engine::State::Subnet { ip, prefix: p, .. } =
                &mut a.progress.attempts.get_mut("ccna-ipv4").unwrap().state
            {
                *ip = "10.12.255.254".parse().unwrap();
                *p = prefix;
            }
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 2400., vec![]);
            }
            let bounds = ctx.input(|i| i.content_rect());
            assert!(a.metrics.ipv4_explain);
            assert_eq!(a.metrics.inputs.len(), 4);
            assert!(a.metrics.ipv4_bits.len() >= 32);
            for (r, _, _) in &a.metrics.ipv4_bits {
                assert!(
                    r.right() <= bounds.right() + 1.,
                    "octet clip at /{prefix}, {width}/{scale}: {r:?} vs {bounds:?}"
                );
            }
            for r in &a.metrics.inputs {
                assert!(r.right() <= bounds.right() + 1.);
            }
        }
    }
}
#[test]
fn sandbox_native_click_and_space_change_only_host_bits_and_not_predictions() {
    let (ctx, mut a) = app();
    a.launch(0);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1400., 2400., vec![]);
    }
    let initial = a.progress.attempts["ccna-ipv4"]
        .ipv4
        .as_ref()
        .unwrap()
        .sandbox_host;
    let original = a.progress.attempts["ccna-ipv4"].state.clone();
    // Use a tall test viewport so these real widgets are visible during pointer injection.
    let press = |ctx: &egui::Context, a: &mut LabApp, r: egui::Rect| {
        for pressed in [true, false] {
            frame(
                ctx,
                a,
                1400.,
                2400.,
                vec![
                    egui::Event::PointerMoved(r.center()),
                    egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    };
    let fixed = a
        .metrics
        .ipv4_bits
        .iter()
        .find(|(_, p, s)| *s && *p == 0)
        .unwrap()
        .0;
    press(&ctx, &mut a, fixed);
    assert_eq!(
        a.progress.attempts["ccna-ipv4"]
            .ipv4
            .as_ref()
            .unwrap()
            .sandbox_host,
        initial
    );
    let host = a
        .metrics
        .ipv4_bits
        .iter()
        .find(|(_, p, s)| *s && *p == 31)
        .unwrap()
        .0;
    press(&ctx, &mut a, host);
    assert_eq!(
        a.progress.attempts["ccna-ipv4"]
            .ipv4
            .as_ref()
            .unwrap()
            .sandbox_host,
        initial ^ 1
    );
    frame(
        &ctx,
        &mut a,
        1400.,
        2400.,
        vec![
            egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            },
            egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Default::default(),
            },
        ],
    );
    assert_eq!(
        a.progress.attempts["ccna-ipv4"]
            .ipv4
            .as_ref()
            .unwrap()
            .sandbox_host,
        initial
    );
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, original);
    assert!(!a.metrics.ipv4_worked);
}

#[test]
fn resuming_unsolved_variant_does_not_invalidate_completed_original_or_reset_its_position() {
    use dlms_learning_lab::ipv4::{Exercise, Section};
    let (ctx, mut a) = app();
    a.launch(0);
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    {
        let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
        if let engine::State::Subnet { answers, .. } = &mut attempt.state {
            *answers = [
                "192.168.10.64".into(),
                "192.168.10.95".into(),
                "192.168.10.65".into(),
                "192.168.10.94".into(),
            ];
        }
        attempt.ipv4.as_mut().unwrap().section = Section::Usable;
    }
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(a.metrics.completion);
    assert_eq!(a.progress.attempts["ccna-ipv4"].step, 2);
    {
        let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
        let e = Exercise::from_state(&attempt.state).unwrap();
        let lesson = attempt.ipv4.as_mut().unwrap();
        for section in [Section::Network, Section::Broadcast, Section::Usable] {
            lesson.reveal(section, &e);
        }
        assert!(lesson.fresh());
    }
    a.launch(0);
    frame(&ctx, &mut a, 1400., 2400., vec![]);
    assert!(!a.invalidated);
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts["ccna-ipv4"].step, 2);
    assert_eq!(
        a.progress.attempts["ccna-ipv4"]
            .ipv4
            .as_ref()
            .unwrap()
            .original_section,
        Section::Usable
    );
    assert!(engine::solved(
        &a.progress.attempts["ccna-ipv4"].state,
        &a.scenarios[0].goal
    ));
}
