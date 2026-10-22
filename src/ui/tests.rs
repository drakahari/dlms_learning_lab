use super::*;
use engine::State;
fn app() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context(&ctx, true, false);
    a.smoke = false;
    (ctx, a)
}
fn frame(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, events: Vec<egui::Event>) {
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| {
            if let egui::Event::Key { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / a.scale, h / a.scale),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
}
fn draw(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1200., 850., vec![]);
    }
}
fn key(key: egui::Key, shift: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            ..Default::default()
        },
    }
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1200.,
            850.,
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
    draw(ctx, a);
}
fn check(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.check.unwrap();
    click(ctx, a, r);
}
fn solve(a: &mut LabApp, i: usize) {
    for command in a.scenarios[i].solution.clone() {
        a.input = command;
        a.submit(i);
    }
}
#[test]
fn all_nine_keep_workspace_prominent_and_controls_inside_width_at_both_modes_and_scales() {
    for i in 0..9 {
        for challenge in [false, true] {
            for (width, height, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                let (ctx, mut a) = app();
                a.launch(i);
                a.scale = scale;
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = challenge;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert!(!a.metrics.guide);
                assert!(!a.metrics.help_visible);
                let bounds = ctx.input(|input| input.content_rect());
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.terminal.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.editor_save.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= bounds.right() + 1.,
                        "{i} clips at {width}/{scale}: {r:?} vs {bounds:?}"
                    );
                }
                if width == 1200. {
                    assert!(
                        a.metrics.check.unwrap().bottom() < 850.,
                        "Check hidden on initial desktop lab {i}: {:?}",
                        a.metrics.check
                    );
                }
                if i == 0 {
                    assert_eq!(a.metrics.inputs.len(), 5);
                    for r in &a.metrics.inputs {
                        assert!((r.left() - a.metrics.inputs[0].left()).abs() < 1.);
                    }
                }
            }
        }
    }
}
#[test]
fn library_topic_is_visible_extra_filters_are_optional_and_catalog_navigation_survives_return() {
    let (ctx, mut a) = app();
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.filters.len(), 1);
    assert!(a.metrics.pagination);
    a.filter.track = "CCNA".into();
    draw(&ctx, &mut a);
    assert!(!a.metrics.pagination);
    let r = a.metrics.extras.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.metrics.filters.len(), 5);
    a.filter.track = "LPIC-1".into();
    a.filter.topic = "Permissions".into();
    a.filter.search = "report".into();
    a.progress.favorites.insert("lpic1-permissions".into());
    a.filter.favorites = true;
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    a.launch(3);
    a.selected = None;
    draw(&ctx, &mut a);
    assert_eq!(a.filter.search, "report");
    assert_eq!(a.metrics.card_actions.len(), 1);
    a.filter = Filter::default();
    a.developer = true;
    a.synthetic = true;
    draw(&ctx, &mut a);
    assert_eq!(a.synthetic_catalog.len(), 900);
    assert!(a.metrics.pagination);
    assert_eq!(a.metrics.card_actions.len(), 12);
    for width in [720., 1200.] {
        for scale in [1., 1.5] {
            a.scale = scale;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 850., vec![]);
            }
            let bounds = ctx.input(|input| input.content_rect());
            for r in a.metrics.header.iter().chain(a.metrics.filters.iter()) {
                assert!(
                    r.right() <= bounds.right() + 1.,
                    "library overflow {width}/{scale}: {r:?}"
                );
            }
        }
    }
}
#[test]
fn ipv4_native_form_checks_partial_answers_and_clears_stale_topology_success() {
    let (ctx, mut a) = app();
    a.launch(0);
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.inputs.len(), 5);
    let first = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(first));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("192.168.10.77".into())],
    );
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(first));
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let view = a.ipv4_view.as_mut().unwrap();
    view.exercise.answers = [
        "192.168.10.64",
        "192.168.10.95",
        "192.168.10.65",
        "192.168.10.94",
    ]
    .map(String::from);
    view.exercise.second = Some("192.168.10.80".into());
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.81".into());
    draw(&ctx, &mut a);
    assert!(
        !a.metrics.completion,
        "a different valid host still needs a fresh check"
    );
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.77".into());
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!engine::solved(
        &a.progress.attempts["ccna-ipv4"].state,
        &a.scenarios[0].goal
    ));
}
#[test]
fn every_lab_grades_state_and_invalidates_after_breakage_without_walkthrough_gates() {
    for i in 0..9 {
        let (ctx, mut a) = app();
        a.launch(i);
        if i == 0 {
            solve(&mut a, i);
            a.load_ipv4_view(i);
        } else {
            solve(&mut a, i);
        }
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(a.metrics.completion, "solution not complete {i}");
        assert_eq!(a.progress.attempts[&a.scenarios[i].id].step, 0);
        match &mut a
            .progress
            .attempts
            .get_mut(&a.scenarios[i].id)
            .unwrap()
            .state
        {
            State::Subnet { second, .. } => *second = Some("192.168.10.77".into()),
            State::Vlsm { allocations, .. } => allocations[0].cidr = "10.20.0.1/26".into(),
            State::Network { gateway, .. } => *gateway = "192.168.40.1".parse().unwrap(),
            State::Unix { nodes, .. } => {
                let p = if i == 3 {
                    "/srv/report.txt"
                } else {
                    "/srv/project/notes.txt"
                };
                nodes.get_mut(p).unwrap().mode = 0;
            }
            State::Logs { result, .. } => result.clear(),
            State::Service { enabled, .. } => *enabled = false,
            State::Dns { loaded, .. } => {
                loaded.insert("portal.lab.test".into(), "192.0.2.8".into());
            }
            State::Mount { writable, .. } => *writable = false,
            State::Script { run, .. } => *run = None,
        }
        if i == 0 {
            a.load_ipv4_view(i);
        }
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion, "success remained after breakage {i}");
    }
}
#[test]
fn service_editor_saves_without_starting_and_resume_preserves_unsaved_draft() {
    let (ctx, mut a) = app();
    a.launch(6);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some(" 8080 ".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        matches!(&a.progress.attempts["lpic2-service"].state,State::Service{port,running:false,enabled:false,..}if port=="8080")
    );
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("8080")
    );
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for c in ["systemctl restart atlas", "systemctl enable atlas"] {
        a.input = c.into();
        a.submit(6);
    }
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some("unfinished draft".into());
    a.selected = None;
    a.launch(6);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("unfinished draft")
    );
    a.input = "systemctl stop atlas".into();
    a.submit(6);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
}
#[test]
fn dns_and_mount_editors_stage_configuration_then_terminal_verifies_application() {
    let (ctx, mut a) = app();
    a.launch(7);
    draw(&ctx, &mut a);
    let original = a.progress.attempts["lpic2-dns"].state.clone();
    let w = a
        .progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap();
    w.dns_address = Some("192.0.2.80".into());
    w.dns_serial = Some("bad".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts["lpic2-dns"].state, original);
    a.progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_serial = Some("2026102102".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    a.input = "systemctl reload named".into();
    a.submit(7);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.launch(8);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-storage")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mount_device = Some("/dev/vdb1".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for c in [
        "mount -o remount,rw /data",
        "rm -r /data/cache",
        "umount /data",
        "mount /data",
    ] {
        a.input = c.into();
        a.submit(8);
    }
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn terminal_enter_focus_and_query_invalidation_work_in_normal_app() {
    let (ctx, mut a) = app();
    a.launch(5);
    draw(&ctx, &mut a);
    let focus = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text(
            "cat /var/log/app.log | grep ERROR".into(),
        )],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(focus));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, true)]);
    draw(&ctx, &mut a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("cat /var/log/app.log".into())],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
}
#[test]
fn reset_resume_and_numerical_practice_preserve_other_attempts_and_original_answers() {
    let (ctx, mut a) = app();
    a.launch(0);
    solve(&mut a, 0);
    a.load_ipv4_view(0);
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    let original = a.progress.attempts["ccna-ipv4"].state.clone();
    a.launch(5);
    a.input = "unfinished command".into();
    a.progress
        .attempts
        .get_mut("lpic1-logs")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    a.launch(0);
    let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
    let e = ipv4::Exercise::from_state(&attempt.state).unwrap();
    let mut l = ipv4::Lesson::from_legacy(e.ip, e.prefix, 0);
    l.variant = Some(ipv4::generated(2));
    l.use_variant = true;
    attempt.ipv4 = Some(l);
    a.load_ipv4_view(0);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, original);
    assert!(!a.metrics.completion);
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.dirty = true;
    a.save();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(p.attempts["ccna-ipv4"].state, original);
    a.progress = p;
    a.launch(5);
    assert_eq!(a.input, "unfinished command");
    a.progress.favorites.insert("lpic1-logs".into());
    a.launch(0);
    draw(&ctx, &mut a);
    let before_reset = a.progress.attempts["ccna-ipv4"].state.clone();
    let r = a.metrics.reset.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.reset_pending);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, before_reset);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    assert!(!a.reset_pending);
    assert_eq!(
        a.progress.attempts["lpic1-logs"]
            .workspace
            .as_ref()
            .unwrap()
            .command,
        "unfinished command"
    );
    assert!(a.progress.favorites.contains("lpic1-logs"));
    assert!(a.progress.attempts["ccna-ipv4"].ipv4.is_none());
}

#[test]
fn assistance_is_optional_and_inserting_a_command_never_executes_it() {
    for i in [3, 6] {
        let (ctx, mut a) = app();
        a.launch(i);
        draw(&ctx, &mut a);
        let id = a.scenarios[i].id.clone();
        let original = a.progress.attempts[&id].state.clone();
        assert!(!a.metrics.guide);
        assert!(a.metrics.insert.is_none());
        let r = a.metrics.help.unwrap();
        click(&ctx, &mut a, r);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(700., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -400.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
        let r = a.metrics.walkthrough.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.metrics.guide);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(700., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -500.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
        let r = a.metrics.insert.unwrap();
        assert!(ctx.input(|i| i.content_rect()).contains(r.center()));
        click(&ctx, &mut a, r);
        assert!(!a.input.is_empty());
        assert_eq!(a.progress.attempts[&id].state, original);
        assert!(a.progress.attempts[&id].transcript.is_empty());
        assert_eq!(
            ctx.memory(|m| m.focused()),
            a.metrics.focus,
            "Insert focuses the terminal even beside an editor"
        );
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(!a.metrics.guide);
        assert!(a.metrics.insert.is_none());
    }
}

#[test]
fn scripting_workspaces_keep_tasks_and_controls_at_sizes_scales_and_modes() {
    for i in 9..16 {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                for challenge in [false, true] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.scale = scale;
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = challenge;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    if width == 1200. && scale == 1. {
                        assert!(
                            a.metrics.editor_viewport.unwrap().height() >= 175.,
                            "usable script editor collapsed"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.inputs.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.stop.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "script{i} {width}/{scale} {r:?}"
                        );
                    }
                    assert!(a.metrics.stop.is_some());
                    assert!(a.metrics.check.is_some());
                }
            }
        }
    }
}
#[test]
fn scripting_native_run_stop_focus_resume_and_success_invalidation() {
    let (ctx, mut a) = app();
    a.launch(9);
    draw(&ctx, &mut a);
    let editor = a.metrics.focus.unwrap();
    let solution = a.scenarios[9].solution[0].clone();
    assert!(ctx.memory(|m| m.has_focus(editor)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            },
            egui::Event::Text(solution),
        ],
    );
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        }],
    );
    for _ in 0..20 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    a.launch(10);
    a.launch(9);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Escape, false)],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(editor)));
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            },
            egui::Event::Text("while true; do :; done".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let stop = a.metrics.stop.unwrap();
    click(&ctx, &mut a, stop);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
    let draft = a.progress.attempts["shell-handover"].state.clone();
    a.launch(0);
    a.launch(9);
    assert_eq!(a.progress.attempts["shell-handover"].state, draft);
    a.reset_attempt(9);
    assert_eq!(
        a.progress.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
}
#[test]
fn every_theme_renders_focus_selection_and_disabled_samples() {
    for t in theme::inventory() {
        let (ctx, mut a) = app();
        a.theme_id = t.id.clone();
        theme::apply(&ctx, &t.id);
        a.theme_preview = true;
        draw(&ctx, &mut a);
        assert_eq!(theme::roles(&ctx).text, t.roles.text);
        assert!(ctx.style().visuals.selection.bg_fill == t.roles.accent);
        assert_eq!(
            ctx.style().visuals.selection.stroke.color,
            t.roles.on_accent
        );
        assert_eq!(ctx.style().visuals.disabled_alpha, 1.);
    }
}
