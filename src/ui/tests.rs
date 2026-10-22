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
    assert_eq!(a.metrics.filters.len(), 2);
    assert!(a.metrics.pagination);
    a.filter.track = "CCNA".into();
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics.pagination,
        a.scenarios.iter().filter(|s| s.track == "CCNA").count() > 12
    );
    a.filter.unit = "ipv4-foundations".into();
    draw(&ctx, &mut a);
    assert!(!a.metrics.pagination);
    let r = a.metrics.extras.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.metrics.filters.len(), 6);
    a.filter.track = "LPIC-1".into();
    a.filter.unit.clear();
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
            State::Switching { .. } | State::Ipv6 { .. } | State::Filesystem { .. } => {
                unreachable!("This regression targets the original nine workspaces")
            }
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
    for (i, _) in pack::bundled()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.teaching.is_some())
    {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                for challenge in [false, true] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.set_script_view(i, teaching::View::Assignment);
                    a.scale = scale;
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = challenge;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    assert!(
                        a.metrics.editor_viewport.unwrap().height() >= 175.,
                        "usable script editor collapsed"
                    );
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
    a.set_script_view(9, teaching::View::Assignment);
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
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
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
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
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

#[test]
fn learn_examples_are_editable_ungraded_and_drafts_survive_navigation_save_and_retry() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.launch(9);
    draw(&ctx, &mut a);
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    assert!(a.metrics.check.is_none());
    assert!(!a.metrics.help_visible);
    let before = a.progress.attempts["shell-handover"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.script_view(9), teaching::View::Example);
    let editor = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(editor));
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    let l = a.progress.attempts["shell-handover"]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.example_result.as_ref().unwrap().stdout,
        "Welcome\nHello, Mina!\nRoom: Library\n"
    );
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
    assert!(!a.metrics.completion);
    let edited = l.example_source.replace("Mina", "Rowan");
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
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Text(edited.clone()),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .as_ref()
            .unwrap()
            .stdout
            .contains("Rowan")
    );
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.script_view(9), teaching::View::Assignment);
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
    let solution = a.scenarios[9].solution[0].clone();
    if let State::Script { source, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = solution;
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    let solved = a.progress.attempts["shell-handover"].state.clone();
    a.set_script_view(9, teaching::View::Example);
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.progress.attempts["shell-handover"].state, solved);
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    a.launch(10);
    a.launch(9);
    assert_eq!(a.script_view(9), teaching::View::Example);
    assert_eq!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        edited
    );
    a.set_script_view(9, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.reset_attempt(9);
    draw(&ctx, &mut a);
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
}

#[test]
fn all_introductions_and_examples_wrap_at_large_text_and_challenge_stays_independent() {
    for (i, _) in pack::bundled()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.teaching.is_some())
    {
        for view in [teaching::View::Introduction, teaching::View::Example] {
            for scale in [1., 1.5] {
                for width in [720., 1200.] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.set_script_view(i, view);
                    a.scale = scale;
                    if view == teaching::View::Example {
                        let lesson = a.scenarios[i].teaching.as_ref().unwrap();
                        let result = shell::run(&lesson.example, &lesson.fixture);
                        a.progress
                            .attempts
                            .get_mut(&a.scenarios[i].id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .script_learning
                            .as_mut()
                            .unwrap()
                            .example_result = Some(result);
                    }
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.lesson_tabs.iter())
                        .chain(a.metrics.inputs.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.assignment_action.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "lab {i} {view:?} {width}/{scale} {r:?}"
                        );
                    }
                    if view == teaching::View::Example {
                        assert!(
                            a.metrics.editor_viewport.unwrap().height() >= 175.,
                            "example editor collapsed"
                        );
                    }
                    if view == teaching::View::Example {
                        assert!(
                            a.metrics.output.unwrap().height() >= 180.,
                            "example output collapsed"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.completion);
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.script_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                    assert!(a.metrics.check.is_some());
                    assert!(a.metrics.example_action.is_none());
                }
            }
        }
    }
}

#[test]
fn settings_moves_samples_off_toolbar_and_theme_changes_persist() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    draw(&ctx, &mut a);
    assert!(!a.settings_open);
    assert!(!a.theme_preview);
    let r = a.metrics.settings.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.settings_open);
    a.theme_id = "dlms-light".into();
    a.sync_appearance(&ctx, ("lab-navy".into(), a.scale));
    let prefs = theme::Preferences::load(&a.path.with_file_name("preferences-v1.json")).unwrap();
    assert_eq!(prefs.theme, "dlms-light");
    assert_eq!(theme::inventory().len(), 28);
}

#[test]
fn example_keyboard_run_syntax_retry_and_stop_do_not_grade_the_assignment() {
    let (ctx, mut a) = app();
    a.launch(9);
    a.set_script_view(9, teaching::View::Example);
    draw(&ctx, &mut a);
    let original = a.progress.attempts["shell-handover"].state.clone();
    let editor = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(editor)));
    let replace = |a: &mut LabApp, text: &str| {
        a.focus_input = true;
        draw(&ctx, a);
        frame(
            &ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Text(text.into()),
            ],
        );
        draw(&ctx, a);
    };
    let run = |a: &mut LabApp| {
        frame(
            &ctx,
            a,
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
    };
    replace(&mut a, "printf 'not closed");
    run(&mut a);
    for _ in 0..5 {
        draw(&ctx, &mut a);
    }
    let error = a.progress.attempts["shell-handover"]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap()
        .example_result
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    assert_eq!(teaching::diagnostic(error), teaching::Diagnostic::Syntax);
    let example = a.scenarios[9].teaching.as_ref().unwrap().example.clone();
    replace(&mut a, &example);
    run(&mut a);
    for _ in 0..5 {
        draw(&ctx, &mut a);
    }
    assert!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .as_ref()
            .unwrap()
            .error
            .is_none()
    );
    a.focus_input = true;
    draw(&ctx, &mut a);
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
    replace(&mut a, "while true; do :; done");
    run(&mut a);
    assert!(a.script_job.is_some());
    draw(&ctx, &mut a);
    let r = a.metrics.stop.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts["shell-handover"].state, original);
}

#[test]
fn legacy_subnet_credit_does_not_invent_a_host_and_requires_new_check() {
    let (ctx, mut a) = app();
    let s = &a.scenarios[0];
    let mut old = Attempt::new(s, 1);
    if let State::Subnet {
        answers, second, ..
    } = &mut old.state
    {
        *answers = [
            "192.168.10.64",
            "192.168.10.95",
            "192.168.10.65",
            "192.168.10.94",
        ]
        .map(String::from);
        *second = None;
    }
    a.progress.attempts.insert(s.id.clone(), old);
    a.launch(0);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(progress::legacy_credit(&a.progress.attempts["ccna-ipv4"]));
    assert!(!progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.81".into());
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert!(
        a.progress.attempts["ccna-ipv4"]
            .workspace
            .as_ref()
            .unwrap()
            .legacy_host_reviewed
    );
    assert!(progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
}

#[test]
fn historical_auto_host_snapshot_is_reviewed_resumed_and_invalidated() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.launch(0);
    solve(&mut a, 0);
    let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
    attempt.workspace.as_mut().unwrap().migrated_host = true;
    attempt.workspace.as_mut().unwrap().ipv4_check = Some(model::snapshot(
        &ipv4::Exercise::from_state(&attempt.state).unwrap(),
    ));
    let before = attempt.state.clone();
    a.launch(0);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, before);
    assert!(!a.metrics.completion);
    assert!(a.ipv4_view.as_ref().unwrap().checked_answers.is_none());
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    a.launch(0);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.77".into());
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
}

fn data_preview(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.data_restore.unwrap();
    click(ctx, a, r);
    let r = a.metrics.data_filename.unwrap();
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![egui::Event::Text("dlms-learning-backup.json".into())],
    );
    draw(ctx, a);
    let r = a.metrics.data_file_action.unwrap();
    click(ctx, a, r);
    for _ in 0..100 {
        draw(ctx, a);
        if a.metrics.data_confirm.is_some() {
            // Async inspection can first show the window on this frame. Let egui
            // settle its measured geometry before using a synthetic pointer target.
            draw(ctx, a);
            draw(ctx, a);
            assert!(a.metrics.data_confirm.is_some());
            return;
        }
        std::thread::yield_now();
    }
    panic!("Restore preview did not finish");
}
#[test]
fn data_ui_exports_previews_cancels_then_restores_without_losing_current_drafts() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    a.launch(9);
    if let State::Script { source, .. } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "saved='draft'".into();
    }
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.data_file_action.unwrap();
    click(&ctx, &mut a, r);
    assert!(dir.path().join("dlms-learning-backup.json").exists());
    a.reset_attempt(9);
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_cancel.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(
        a.progress.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        matches!(&a.progress.attempts["shell-handover"].state,State::Script{source,..}if source=="saved='draft'")
    );
    let safety = progress::load(
        &dir.path().join("progress-before-restore-1.json"),
        &a.scenarios,
    )
    .unwrap();
    assert_eq!(
        safety.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
    assert!(!a.dirty);
}
#[test]
fn restoring_corrupt_profile_protects_unsaved_read_only_session_work() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    a.launch(9);
    dlms_learning_lab::backup::export(
        &dir.path().join("dlms-learning-backup.json"),
        &a.progress,
        &a.scenarios,
    )
    .unwrap();
    std::fs::write(&a.path, b"corrupt original").unwrap();
    a.read_only = true;
    a.storage_error = Some("controlled invalid profile".into());
    if let State::Script { source, .. } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "keep='unsaved session'".into();
    }
    a.dirty = true;
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    assert!(!a.read_only, "{}", a.data_tools.restore_diagnostic());
    assert_eq!(
        std::fs::read(dir.path().join("progress-before-restore-1.json")).unwrap(),
        b"corrupt original"
    );
    let session = dlms_learning_lab::backup::inspect(
        &dir.path().join("session-before-restore-1.json"),
        &a.scenarios,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        matches!(&session.progress.attempts["shell-handover"].state,State::Script{source,..}if source=="keep='unsaved session'")
    );
}

#[test]
fn backup_dialog_does_not_allow_background_editor_shortcuts_to_run_labs() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    a.launch(9);
    a.set_script_view(9, teaching::View::Assignment);
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
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
    draw(&ctx, &mut a);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
}

#[test]
fn course_start_and_unit_filters_keep_search_favorites_and_resume_navigation() {
    let (ctx, mut a) = app();
    a.filter.track = "Bash/Shell".into();
    draw(&ctx, &mut a);
    let r = a.metrics.course_start.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.selected, Some(9));
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.filter.track, "Bash/Shell");
    a.filter.unit = "shell-foundations".into();
    a.filter.search = "handover".into();
    a.progress.favorites.insert("shell-handover".into());
    a.filter.favorites = true;
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    a.launch(9);
    draw(&ctx, &mut a);
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.filter.unit, "shell-foundations");
    assert_eq!(a.filter.search, "handover");
    assert!(a.filter.favorites);
}
#[test]
fn unit_filter_controls_wrap_at_narrow_widths_and_large_text() {
    for (track, unit) in [
        ("LPIC-2", "linux-service-recovery"),
        ("LPIC-1", "linux-access-policy"),
    ] {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                let (ctx, mut a) = app();
                a.filter.track = track.into();
                a.filter.unit = unit.into();
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 850., vec![]);
                }
                for r in &a.metrics.filters {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale}: {r:?}, screen {:?}",
                        ctx.content_rect()
                    );
                }
            }
        }
    }
}
#[test]
fn function_scripts_run_cooperatively_in_the_existing_workspace_and_preserve_drafts() {
    let (ctx, mut a) = app();
    a.launch(9);
    a.set_script_view(9, teaching::View::Assignment);
    let source = "handover() { printf 'Site: %s\\nTask: %s\\n' \"$1\" \"$2\"; }\nhandover \"$SITE\" \"$TASK\"";
    if let State::Script { source: draft, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *draft = source.into();
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    if let State::Script { source, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "spin() { while true; do :; done; }\nspin".into();
        *run = None;
    }
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let stop = a.metrics.stop.unwrap();
    click(&ctx, &mut a, stop);
    assert!(a.script_job.is_none());
    let draft = a.progress.attempts["shell-handover"].state.clone();
    a.launch(10);
    a.launch(9);
    assert_eq!(a.progress.attempts["shell-handover"].state, draft);
    assert!(!a.metrics.completion);
}

#[test]
fn terminal_introduction_practice_and_assignment_keep_drafts_focus_and_credit_separate() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-file-navigation")
        .unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Introduction);
    assert!(a.metrics.terminal.is_none());
    let original = a.progress.attempts["lpic1-file-navigation"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Example);
    assert!(ctx.memory(|m| m.has_focus(a.metrics.focus.unwrap())));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Text("pwd".into()),
            key(egui::Key::Enter, false),
        ],
    );
    draw(&ctx, &mut a);
    let l = a.progress.attempts["lpic1-file-navigation"]
        .workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.transcript.len(), 1);
    assert!(l.transcript[0].contains("/home/alex"));
    assert_eq!(a.progress.attempts["lpic1-file-navigation"].state, original);
    assert!(!a.metrics.completion);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("practice draft".into())],
    );
    draw(&ctx, &mut a);
    a.set_terminal_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("assignment draft".into())],
    );
    draw(&ctx, &mut a);
    let assignment = a.input.clone();
    a.set_terminal_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts["lpic1-file-navigation"]
            .workspace
            .as_ref()
            .unwrap()
            .terminal_learning
            .as_ref()
            .unwrap()
            .command,
        "practice draft"
    );
    assert_eq!(a.input, assignment);
    let directory = tempfile::tempdir().unwrap();
    a.path = directory.path().join("progress.json");
    a.save();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    a.progress
        .attempts
        .get_mut("lpic1-file-navigation")
        .unwrap()
        .challenge = true;
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Assignment);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    a.reset_attempt(i);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["lpic1-file-navigation"].state, original);
}
#[test]
fn file_lesson_layouts_wrap_and_challenge_keeps_full_assignment_at_large_text() {
    let labs = pack::bundled().unwrap();
    for (i, s) in labs
        .iter()
        .enumerate()
        .filter(|(_, s)| s.terminal_teaching.is_some())
    {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.scale = scale;
                    a.set_terminal_view(i, view);
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.lesson_tabs.iter())
                        .chain(a.metrics.terminal.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.assignment_action.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "{} {width}/{scale} {view:?} {r:?}",
                            s.id
                        );
                    }
                    if view != teaching::View::Introduction {
                        assert!(a.metrics.output.unwrap().height() >= 140.);
                        assert!(a.metrics.terminal.is_some());
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.terminal_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}
#[test]
fn file_assignments_grade_then_invalidate_and_selected_reset_preserves_other_labs() {
    let (ctx, mut a) = app();
    a.launch(9);
    let old = a.progress.attempts["shell-handover"].state.clone();
    let indexes: Vec<_> = a
        .scenarios
        .iter()
        .enumerate()
        .filter(|(_, s)| s.terminal_teaching.is_some())
        .map(|(i, _)| i)
        .collect();
    for i in indexes {
        a.launch(i);
        a.set_terminal_view(i, teaching::View::Assignment);
        for cmd in a.scenarios[i].solution.clone() {
            a.input = cmd;
            a.submit(i);
        }
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        draw(&ctx, &mut a);
        assert!(a.metrics.completion, "{}", a.scenarios[i].id);
        let id = a.scenarios[i].id.clone();
        let State::Filesystem { tree } = &mut a.progress.attempts.get_mut(&id).unwrap().state
        else {
            panic!()
        };
        // Each assignment protects at least one supplied source; change all for this UI regression.
        for text in tree.files.values_mut() {
            text.push_str("changed");
        }
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion, "{id}");
        a.reset_attempt(i);
        assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
        assert_eq!(a.progress.attempts["shell-handover"].state, old);
    }
}

#[test]
fn new_linux_learners_start_with_file_foundations_without_reordering_saved_ids() {
    let (ctx, mut a) = app();
    a.filter.track = "LPIC-1".into();
    draw(&ctx, &mut a);
    let r = a.metrics.course_start.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, "lpic1-file-navigation");
    assert_eq!(
        a.terminal_view(a.selected.unwrap()),
        teaching::View::Introduction
    );
    assert_eq!(a.scenarios[3].id, "lpic1-permissions");
}

#[test]
fn terminal_example_insert_only_populates_input_and_tab_releases_the_field() {
    let (ctx, mut a) = app();
    a.launch(24);
    a.set_terminal_view(24, teaching::View::Example);
    draw(&ctx, &mut a);
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let l = a.progress.attempts["lpic1-file-navigation"]
        .workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, "pwd");
    assert!(l.transcript.is_empty());
    assert_eq!(
        l.tree,
        a.scenarios[24].terminal_teaching.as_ref().unwrap().fixture
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(
        !ctx.memory(|m| m.has_focus(field)),
        "Tab leaves the one-line terminal input"
    );
}

#[test]
fn ipv6_native_typing_check_retry_focus_and_selected_reset() {
    let (ctx, mut a) = app();
    a.launch(9);
    let earlier = a.progress.attempts["shell-handover"].clone();
    a.launch(30);
    assert_eq!(a.ipv6_view(30), teaching::View::Introduction);
    draw(&ctx, &mut a);
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let id = a.scenarios[30].id.clone();
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .as_ref()
            .unwrap()
            .selected_device,
        "NewPC"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("2001:db8:44::23/64".into())],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(
        a.metrics.completion,
        "Manual entry should work without advanced settings"
    );
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
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
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
            },
            egui::Event::Text("malformed".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[30]
    ));
    check(&ctx, &mut a);
    assert!(a.notice.as_ref().unwrap().contains("Malformed"));
    a.launch(31);
    a.launch(30);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .as_ref()
            .unwrap()
            .devices["NewPC"]
            .interfaces["lan"]
            .address,
        "malformed"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    a.reset_attempt(30);
    assert_eq!(a.progress.attempts[&id].state, a.scenarios[30].initial);
    assert_eq!(a.progress.attempts["shell-handover"].state, earlier.state);
}

#[test]
fn ipv6_examples_apply_without_credit_and_open_assignment_without_overwriting_draft() {
    let (ctx, mut a) = app();
    for i in 30..36 {
        a.launch(i);
        let id = a.scenarios[i].id.clone();
        let assignment = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .clone();
        a.set_ipv6_view(i, teaching::View::Example);
        let lesson = a.scenarios[i].ipv6_teaching.clone().unwrap();
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .ipv6_learning
            .as_mut()
            .unwrap()
            .fill_example(&lesson)
            .unwrap();
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        let attempt = &a.progress.attempts[&id];
        let learning = attempt
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_learning
            .as_ref()
            .unwrap();
        assert!(learning.checks(&lesson).iter().all(|(pass, _)| *pass));
        assert_eq!(attempt.state, a.scenarios[i].initial);
        assert_eq!(attempt.workspace.as_ref().unwrap().ipv6_draft, assignment);
        assert!(!progress::current_solved(attempt, &a.scenarios[i]));
        let r = a.metrics.lesson_tabs[2];
        click(&ctx, &mut a, r);
        assert_eq!(a.ipv6_view(i), teaching::View::Assignment);
    }
}

#[test]
fn ipv6_native_views_keep_controls_in_width_with_large_text_and_challenge_is_independent() {
    for i in 30..36 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for routes in [false, true] {
                for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.scale = scale;
                    a.set_ipv6_view(i, view);
                    let workspace = a
                        .progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap();
                    workspace.ipv6_draft.as_mut().unwrap().show_routes = routes;
                    workspace.ipv6_learning.as_mut().unwrap().draft.show_routes = routes;
                    for _ in 0..4 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    for r in a
                        .metrics
                        .inputs
                        .iter()
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.lesson_tabs.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "lab{i} {view:?} {w}/{scale} {r:?}"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    if w == 1200. && view == teaching::View::Assignment && !routes {
                        assert!(
                            a.metrics.check.unwrap().bottom() < 850.,
                            "check below desktop lab{i}"
                        );
                    }
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.ipv6_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}

#[test]
fn earlier_script_runs_are_labeled_legacy_until_a_current_native_run_and_remain_saved() {
    let (ctx, mut a) = app();
    a.launch(9);
    let lab = a.scenarios[9].clone();
    let engine::Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let original = lab.solution[0].clone();
    let source = format!("IFS=,\n{original}");
    a.progress.attempts.get_mut(&lab.id).unwrap().state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: 0,
            source,
            results: cases.iter().map(|c| shell::run(&original, c)).collect(),
        }),
    };
    a.launch(9);
    a.set_script_view(9, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(!a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
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
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
            },
            egui::Event::Text(original),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..15 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    let history = a.progress.attempts[&lab.id]
        .workspace
        .as_ref()
        .unwrap()
        .script_history
        .clone();
    a.launch(10);
    a.launch(9);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert_eq!(
        a.progress.attempts[&lab.id]
            .workspace
            .as_ref()
            .unwrap()
            .script_history,
        history
    );
    a.reset_attempt(9);
    assert!(
        a.progress.attempts[&lab.id]
            .workspace
            .as_ref()
            .unwrap()
            .script_history
            .is_none()
    );
}

#[test]
fn count_assignments_keep_the_editor_and_actions_in_width_and_stop_file_loops() {
    for i in 43..47 {
        for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            a.launch(i);
            a.set_script_view(i, teaching::View::Assignment);
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a.metrics.inputs.iter().chain(a.metrics.check.iter()) {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "lab{i} {w}/{scale}: {r:?}"
                );
            }
            assert!(a.metrics.editor_viewport.unwrap().height() >= 175.);
            assert!(!a.metrics.help_visible);
        }
    }
    let (ctx, mut a) = app();
    a.launch(46);
    a.set_script_view(46, teaching::View::Assignment);
    if let State::Script { source, run } = &mut a
        .progress
        .attempts
        .get_mut("shell-count-capstone")
        .unwrap()
        .state
    {
        *source = "while true; do :; done < alpha.txt".into();
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let r = a.metrics.stop.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
}

#[test]
fn switching_fixture_keeps_practice_separate_and_wraps_with_visible_keyboard_focus() {
    for (width, height, scale) in [
        (1200., 850., 1.),
        (1120., 850., 1.),
        (1080., 850., 1.),
        (720., 560., 1.),
        (720., 560., 1.5),
    ] {
        let (ctx, mut a) = app();
        a.scenarios.push(
            pack::load(include_str!("../../tests/fixtures/switching-preview.json"))
                .unwrap()
                .remove(0),
        );
        let i = a.scenarios.len() - 1;
        a.launch(i);
        a.scale = scale;
        assert_eq!(a.switching_view(i), teaching::View::Introduction);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.set_switching_view(i, view);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.check.iter())
                .chain(a.metrics.lesson_tabs.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?} {width}/{scale}: {r:?}"
                );
            }
            assert!(!a.metrics.help_visible);
            assert!(!a.metrics.completion);
        }
        assert!(a.metrics.inputs.len() >= 5);
        let input = a.metrics.inputs[0];
        if ctx.content_rect().contains(input.center()) {
            for pressed in [true, false] {
                frame(
                    &ctx,
                    &mut a,
                    width,
                    height,
                    vec![
                        egui::Event::PointerMoved(input.center()),
                        egui::Event::PointerButton {
                            pos: input.center(),
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            assert!(ctx.memory(|m| m.focused()).is_some());
        }
        frame(
            &ctx,
            &mut a,
            width,
            height,
            vec![key(egui::Key::Tab, false)],
        );
        assert!(ctx.memory(|m| m.focused()).is_some());
        let s = &a.scenarios[i];
        a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
        frame(&ctx, &mut a, width, height, vec![]);
        assert_eq!(a.switching_view(i), teaching::View::Assignment);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.help_visible);
    }
}

#[test]
fn authored_vlan_assignments_keep_tools_in_width_and_guidance_optional() {
    for name in [
        "ccna-vlan-access",
        "ccna-vlan-trunk",
        "ccna-vlan-native",
        "ccna-vlan-handover",
    ] {
        for (width, height, scale) in [(1200., 850., 1.), (1080., 850., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            let i = a.scenarios.iter().position(|s| s.id == name).unwrap();
            a.launch(i);
            a.scale = scale;
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                a.set_switching_view(i, view);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{name} {view:?} {width}/{scale}: {r:?}"
                    );
                }
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.completion);
            }
            a.progress.attempts.get_mut(name).unwrap().challenge = true;
            frame(&ctx, &mut a, width, height, vec![]);
            assert_eq!(a.switching_view(i), teaching::View::Assignment);
            assert!(!a.metrics.inputs.is_empty());
            assert!(a.metrics.lesson_tabs.is_empty());
        }
    }
}

#[test]
fn resumed_switching_draft_never_shows_applied_state_as_current_completion() {
    fn shape_text(shape: &egui::epaint::Shape, result: &mut String) {
        match shape {
            egui::epaint::Shape::Text(t) => {
                result.push_str(&t.galley.job.text);
                result.push('\n');
            }
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    shape_text(s, result);
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-vlan-access")
        .unwrap();
    a.launch(i);
    a.set_switching_view(i, teaching::View::Assignment);
    let s = &a.scenarios[i];
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute(&mut state, cmd).unwrap();
    }
    let attempt = a.progress.attempts.get_mut(&s.id).unwrap();
    attempt.state = state;
    let w = attempt.workspace.as_mut().unwrap();
    w.checked = true;
    w.switching_draft
        .as_mut()
        .unwrap()
        .switches
        .get_mut("office")
        .unwrap()
        .ports
        .get_mut("staff-b")
        .unwrap()
        .access = "bad".into();
    assert!(engine::solved(&attempt.state, &s.goal));
    assert!(!progress::current_solved(attempt, s));
    let raw = serde_json::to_vec(&a.progress).unwrap();
    a.progress = progress::decode(&raw, &a.scenarios).unwrap();
    a.checked = true;
    draw(&ctx, &mut a);
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
    let mut text = String::new();
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(!a.metrics.completion);
    assert!(!text.contains("Objective complete"), "{text}");
    assert!(text.contains("Draft not verified"), "{text}");
}

#[test]
fn file_batch_lessons_keep_native_tools_in_width_and_challenge_guidance_optional() {
    for i in 55..58 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.5)] {
                let (ctx, mut a) = app();
                a.launch(i);
                a.set_script_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{i} {view:?} {w}/{scale}: {r:?}"
                    );
                }
                if view != teaching::View::Introduction {
                    assert!(a.metrics.editor_viewport.unwrap().height() >= 175.);
                }
                assert!(!a.metrics.help_visible);
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..3 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(a.metrics.lesson_tabs.is_empty());
                assert_eq!(a.script_view(i), teaching::View::Assignment);
                assert!(!a.metrics.help_visible);
            }
        }
    }
}

#[test]
fn file_batch_editors_support_keyboard_run_and_release_focus_for_navigation() {
    for i in 55..58 {
        let (ctx, mut a) = app();
        a.launch(i);
        a.set_script_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let editor = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(editor)));
        let source = a.scenarios[i].solution[0].clone();
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
                    modifiers: egui::Modifiers {
                        command: true,
                        ..egui::Modifiers::CTRL
                    },
                },
                egui::Event::Text(source),
            ],
        );
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
        for _ in 0..40 {
            draw(&ctx, &mut a);
            if a.script_job.is_none() {
                break;
            }
        }
        assert!(a.metrics.completion, "{}", a.scenarios[i].id);
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
        assert!(ctx.memory(|m| m.focused()).is_some());
    }
}
