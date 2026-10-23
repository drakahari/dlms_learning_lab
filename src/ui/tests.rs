use super::*;
use engine::State;
fn app() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context(&ctx, true, false);
    a.smoke = false;
    (ctx, a)
}
fn open_assignment(a: &mut LabApp, i: usize) {
    a.launch(i);
    if a.scenarios[i].address_teaching.is_some() {
        a.set_address_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].service_teaching.is_some() {
        a.set_service_view(i, teaching::View::Assignment);
    }
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
                open_assignment(&mut a, i);
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
    open_assignment(&mut a, 3);
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
    open_assignment(&mut a, 0);
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
        open_assignment(&mut a, i);
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
            State::Switching { .. }
            | State::Ipv6 { .. }
            | State::Ipv4Routing { .. }
            | State::Filesystem { .. } => {
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
    open_assignment(&mut a, 6);
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
    open_assignment(&mut a, 6);
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
    open_assignment(&mut a, 7);
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
    open_assignment(&mut a, 8);
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
    open_assignment(&mut a, 5);
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
    open_assignment(&mut a, 0);
    solve(&mut a, 0);
    a.load_ipv4_view(0);
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    let original = a.progress.attempts["ccna-ipv4"].state.clone();
    open_assignment(&mut a, 5);
    a.input = "unfinished command".into();
    a.progress
        .attempts
        .get_mut("lpic1-logs")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    open_assignment(&mut a, 0);
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
    open_assignment(&mut a, 5);
    assert_eq!(a.input, "unfinished command");
    a.progress.favorites.insert("lpic1-logs".into());
    open_assignment(&mut a, 0);
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
        open_assignment(&mut a, i);
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
                    open_assignment(&mut a, i);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 0);
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
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
                    open_assignment(&mut a, i);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 0);
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
    open_assignment(&mut a, 0);
    solve(&mut a, 0);
    let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
    attempt.workspace.as_mut().unwrap().migrated_host = true;
    attempt.workspace.as_mut().unwrap().ipv4_check = Some(model::snapshot(
        &ipv4::Exercise::from_state(&attempt.state).unwrap(),
    ));
    let before = attempt.state.clone();
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, before);
    assert!(!a.metrics.completion);
    assert!(a.ipv4_view.as_ref().unwrap().checked_answers.is_none());
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    open_assignment(&mut a, 0);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, i);
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
                    open_assignment(&mut a, i);
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
    open_assignment(&mut a, 9);
    let old = a.progress.attempts["shell-handover"].state.clone();
    let indexes: Vec<_> = a
        .scenarios
        .iter()
        .enumerate()
        .filter(|(_, s)| s.terminal_teaching.is_some())
        .map(|(i, _)| i)
        .collect();
    for i in indexes {
        open_assignment(&mut a, i);
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
    open_assignment(&mut a, 24);
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
    open_assignment(&mut a, 9);
    let earlier = a.progress.attempts["shell-handover"].clone();
    open_assignment(&mut a, 30);
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
    open_assignment(&mut a, 31);
    open_assignment(&mut a, 30);
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
        open_assignment(&mut a, i);
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
                    open_assignment(&mut a, i);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 9);
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
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
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
            open_assignment(&mut a, i);
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
    open_assignment(&mut a, 46);
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
        open_assignment(&mut a, i);
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
            open_assignment(&mut a, i);
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
    open_assignment(&mut a, i);
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
                open_assignment(&mut a, i);
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
        open_assignment(&mut a, i);
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

#[test]
fn routing_v4_native_typing_check_retry_focus_and_selected_reset() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    let earlier = a.progress.attempts["shell-handover"].clone();
    let i = a.add_routing_fixture();
    open_assignment(&mut a, i);
    assert_eq!(a.routing_v4_view(i), teaching::View::Introduction);
    draw(&ctx, &mut a);
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .selected_device,
        "Client"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
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
            egui::Event::Text("192.0.2.99/24".into()),
        ],
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
        &a.scenarios[i]
    ));
    check(&ctx, &mut a);
    assert!(a.notice.as_ref().unwrap().contains("Malformed"));
    open_assignment(&mut a, 31);
    open_assignment(&mut a, i);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .devices["Client"]
            .interfaces["lan"]
            .address,
        "malformed"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
    assert_eq!(a.progress.attempts["shell-handover"].state, earlier.state);
}

#[test]
fn routing_v4_native_views_keep_controls_in_width_with_large_text_and_challenge_is_independent() {
    for _ in 0..1 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for routes in [false, true] {
                for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                    let (ctx, mut a) = app();
                    let i = a.add_routing_fixture();
                    open_assignment(&mut a, i);
                    a.scale = scale;
                    a.set_routing_v4_view(i, view);
                    let workspace = a
                        .progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap();
                    workspace.routing_v4_draft.as_mut().unwrap().show_routes = routes;
                    workspace
                        .routing_v4_learning
                        .as_mut()
                        .unwrap()
                        .draft
                        .show_routes = routes;
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
                    assert_eq!(a.routing_v4_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}

#[test]
fn routing_v4_single_interface_router_is_editable_and_guideless_help_is_safe() {
    let (ctx, mut a) = app();
    let i = a.add_routing_fixture();
    if let engine::Goal::Ipv4Routing { requirements } = &mut a.scenarios[i].goal {
        requirements.push(dlms_learning_lab::ipv4_routing::Requirement::Forwarding {
            device: "Client".into(),
            enabled: true,
        });
    }
    open_assignment(&mut a, i);
    a.set_routing_v4_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let r = a.metrics.routing_router_options.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.routing_forward.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        a.progress.attempts[&a.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .devices["Client"]
            .forwarding
    );
    let r = a.metrics.help.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.help_open);
    assert!(!a.metrics.guide); // Empty optional walkthrough is valid authoring data.
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
            egui::Event::Text("192.0.2.99/24".into()),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
}

#[test]
fn ipv4_authored_route_entry_supports_keyboard_tab_and_default_distance_without_reference_fill() {
    for keyboard_open in [true, false] {
        let (ctx, mut a) = app();
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-ipv4-network-route")
            .unwrap();
        open_assignment(&mut a, i);
        draw(&ctx, &mut a);
        let r = a.metrics.assignment_action.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.metrics.inputs.is_empty());
        let r = a.metrics.routing_add.unwrap();
        click(&ctx, &mut a, r);
        let focus = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(focus)));
        let select = egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![select, egui::Event::Text("10.40.1.0/24".into())],
        );
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("10.0.0.2".into())],
        );
        draw(&ctx, &mut a);
        if keyboard_open {
            frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
            assert!(ctx.memory(|m| m.has_focus(a.metrics.routing_picker_id.unwrap())));
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![key(egui::Key::Enter, false)],
            );
            draw(&ctx, &mut a);
        } else {
            let r = a.metrics.routing_route_interface.unwrap();
            click(&ctx, &mut a, r);
        }
        let target = a
            .metrics
            .routing_port_options
            .iter()
            .find(|(port, _)| port == "transit")
            .unwrap()
            .1;
        let mut tabs = 0;
        while !ctx.memory(|m| m.has_focus(target)) && tabs < 8 {
            frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
            tabs += 1;
            let mut release = key(egui::Key::Tab, false);
            if let egui::Event::Key { pressed, .. } = &mut release {
                *pressed = false;
            }
            frame(&ctx, &mut a, 1200., 850., vec![release]);
        }
        assert!(
            ctx.memory(|m| m.has_focus(target)),
            "Outgoing interface option reachable by Tab"
        );
        assert_eq!(tabs, 1, "Selected menu option receives focus when opened");
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(
            a.metrics.completion,
            "notice {:?} draft {:?}",
            a.notice,
            a.progress.attempts[&a.scenarios[i].id]
                .workspace
                .as_ref()
                .unwrap()
                .routing_v4_draft
        );
        let id = a.scenarios[i].id.clone();
        let draft = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap();
        assert_eq!(draft.devices["R1"].routes[0].distance, "1");
        let r = a.metrics.inputs[1];
        click(&ctx, &mut a, r);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("x".into())],
        );
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn six_authored_ipv4_workspaces_keep_full_tasks_tools_and_guidance_separate_at_each_size() {
    for id in [
        "ccna-ipv4-network-route",
        "ccna-ipv4-return-route",
        "ccna-ipv4-specific-route",
        "ccna-ipv4-host-exception",
        "ccna-ipv4-floating-route",
        "ccna-ipv4-branch-handover",
    ] {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [
                (1200., 850., 1.),
                (1200., 850., 1.5),
                (1000., 800., 1.),
                (940., 700., 1.),
                (960., 700., 1.5),
                (720., 560., 1.),
                (720., 560., 1.5),
            ] {
                let (ctx, mut a) = app();
                let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
                open_assignment(&mut a, i);
                a.set_routing_v4_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.guide);
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                    .chain(a.metrics.routing_add.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{id} {view:?} {w}/{scale}: {r:?}"
                    );
                }
                if w == 1200. && scale == 1. && view == teaching::View::Assignment {
                    assert!(
                        a.metrics.check.unwrap().bottom() < 850.,
                        "Desktop Check below workspace {id}: {:?}",
                        a.metrics.check
                    );
                }
                a.progress.attempts.get_mut(id).unwrap().challenge = true;
                draw(&ctx, &mut a);
                assert_eq!(a.routing_v4_view(i), teaching::View::Assignment);
                assert!(a.metrics.lesson_tabs.is_empty());
                assert!(!a.metrics.help_visible);
            }
        }
    }
}

#[test]
fn addressing_introduction_examples_and_assignments_preserve_drafts_and_credit() {
    for i in [0, 1] {
        let (ctx, mut a) = app();
        let dir = tempfile::tempdir().unwrap();
        a.path = dir.path().join("progress.json");
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.address_view(i), teaching::View::Introduction);
        assert!(a.metrics.check.is_none());
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(!a.metrics.help_visible);
        let id = a.scenarios[i].id.clone();
        let original = a.progress.attempts[&id].state.clone();
        let r = a.metrics.example_action.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(a.address_view(i), teaching::View::Example);
        let example_focus = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(example_focus)));
        let r = a.metrics.insert.unwrap();
        click(&ctx, &mut a, r);
        check(&ctx, &mut a);
        draw(&ctx, &mut a);
        let example = &a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .address_learning
            .as_ref()
            .unwrap();
        assert!(example.solved());
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
        assert_eq!(a.progress.attempts[&id].state, original);
        let saved_example = serde_json::to_value(example).unwrap();
        let r = a.metrics.assignment_action.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(
            a.address_view(i),
            teaching::View::Assignment,
            "lab{i} action {r:?}"
        );
        assert_eq!(a.progress.attempts[&id].state, original);
        let actions = a.scenarios[i].solution.clone();
        for action in actions {
            engine::execute(
                &mut a.progress.attempts.get_mut(&id).unwrap().state,
                &action,
            )
            .unwrap();
        }
        a.load_ipv4_view(i);
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(a.metrics.completion);
        let solved = a.progress.attempts[&id].state.clone();
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, solved);
        assert_eq!(
            serde_json::to_value(
                a.progress.attempts[&id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .address_learning
                    .as_ref()
                    .unwrap()
            )
            .unwrap()["state"],
            saved_example["state"]
        );
        let r = a.metrics.example_reset.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(a.progress.attempts[&id].state, solved);
        assert!(
            !a.progress.attempts[&id]
                .workspace
                .as_ref()
                .unwrap()
                .address_learning
                .as_ref()
                .unwrap()
                .solved()
        );
        a.save();
        a.progress = progress::load(&a.path, &a.scenarios).unwrap();
        a.launch(i);
        assert_eq!(a.address_view(i), teaching::View::Example);
        assert_eq!(a.progress.attempts[&id].state, solved);
        a.reset_attempt(i);
        assert_eq!(a.address_view(i), teaching::View::Introduction);
        assert_eq!(a.progress.attempts[&id].state, original);
        // Pre-teaching attempts reopen their intact assignment rather than forcing a lesson.
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .address_learning = None;
        a.launch(i);
        assert_eq!(a.address_view(i), teaching::View::Assignment);
    }
}
#[test]
fn addressing_fields_namespace_keyboard_undo_between_example_and_assignment() {
    for i in [0, 1] {
        let (ctx, mut a) = app();
        a.launch(i);
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        let example_id = a.metrics.focus.unwrap();
        let command = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("example-draft".into())],
        );
        draw(&ctx, &mut a);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                command(egui::Key::A),
                egui::Event::Text("example-edit".into()),
            ],
        );
        draw(&ctx, &mut a);
        let id = a.scenarios[i].id.clone();
        let example = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .address_learning
            .as_ref()
            .unwrap()
            .state
            .clone();
        a.set_address_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let assignment_id = a.metrics.focus.unwrap();
        assert_ne!(assignment_id, example_id);
        assert!(ctx.memory(|m| m.has_focus(assignment_id)));
        let original = a.progress.attempts[&id].state.clone();
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, original);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("assignment-draft".into())],
        );
        draw(&ctx, &mut a);
        let draft = a.progress.attempts[&id].state.clone();
        assert_ne!(draft, original);
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        assert_eq!(
            a.progress.attempts[&id]
                .workspace
                .as_ref()
                .unwrap()
                .address_learning
                .as_ref()
                .unwrap()
                .state,
            example
        );
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, draft);
    }
}
#[test]
fn addressing_views_wrap_and_challenge_keeps_complete_independent_assignments() {
    for i in [0, 1] {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [
                (1200., 850., 1.),
                (960., 740., 1.5),
                (720., 560., 1.),
                (720., 560., 1.5),
            ] {
                let (ctx, mut a) = app();
                a.launch(i);
                a.scale = scale;
                a.set_address_view(i, view);
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{i} {view:?} {w}/{scale} {r:?}"
                    );
                }
                if let Some(r) = a.metrics.address_diagram {
                    assert!(
                        r.height() < 400. && r.right() <= ctx.content_rect().right() + 1.,
                        "diagram {w}/{scale} {r:?}"
                    );
                }
                if w == 1200. && view != teaching::View::Introduction {
                    assert!(a.metrics.check.unwrap().bottom() < 850.);
                }
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert_eq!(a.address_view(i), teaching::View::Assignment);
                assert!(a.metrics.lesson_tabs.is_empty());
                assert!(a.metrics.insert.is_none());
                assert_eq!(a.metrics.inputs.len(), if i == 0 { 5 } else { 3 });
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.help.is_none());
            }
        }
    }
}

#[test]
fn service_fresh_learning_teaches_runs_and_preserves_both_editor_terminal_drafts() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.launch(6);
    draw(&ctx, &mut a);
    assert_eq!(a.service_view(6), teaching::View::Introduction);
    assert!(a.metrics.check.is_none());
    let original = a.progress.attempts["lpic2-service"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.service_view(6), teaching::View::Example);
    let example_focus = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(example_focus)));
    let r = a.metrics.command_insert.unwrap();
    click(&ctx, &mut a, r);
    let l = a.progress.attempts["lpic2-service"]
        .workspace
        .as_ref()
        .unwrap()
        .service_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, "systemctl status atlas");
    assert!(l.transcript.is_empty());
    assert_eq!(
        l.state,
        a.scenarios[6].service_teaching.as_ref().unwrap().fixture
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert!(
        !a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .transcript
            .is_empty()
    );
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .state,
        a.scenarios[6].service_teaching.as_ref().unwrap().fixture
    );
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for command in ["systemctl restart atlas", "systemctl enable atlas"] {
        a.focus_input = true;
        draw(&ctx, &mut a);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text(command.into())],
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        draw(&ctx, &mut a);
    }
    check(&ctx, &mut a);
    assert!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .solved(a.scenarios[6].service_teaching.as_ref().unwrap())
    );
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts["lpic2-service"].state, original);
    a.set_service_view(6, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_ne!(example_focus, a.metrics.focus.unwrap());
    a.input = "original terminal draft".into();
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some("original file draft".into());
    a.set_service_view(6, teaching::View::Example);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_learning
        .as_mut()
        .unwrap()
        .port_draft = "9091".into();
    draw(&ctx, &mut a);
    assert!(
        !a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .solved(a.scenarios[6].service_teaching.as_ref().unwrap())
    );
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    a.launch(6);
    assert_eq!(a.service_view(6), teaching::View::Example);
    assert_eq!(a.input, "original terminal draft");
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("original file draft")
    );
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .port_draft,
        "9091"
    );
    a.reset_attempt(6);
    assert_eq!(a.service_view(6), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["lpic2-service"].state, original);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_learning = None;
    a.launch(6);
    assert_eq!(a.service_view(6), teaching::View::Assignment);
}
#[test]
fn service_views_wrap_and_challenge_keeps_the_full_editor_and_terminal() {
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        for (w, h, scale) in [
            (1200., 850., 1.),
            (960., 740., 1.5),
            (720., 560., 1.),
            (720., 560., 1.5),
        ] {
            let (ctx, mut a) = app();
            a.launch(6);
            a.scale = scale;
            a.set_service_view(6, view);
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.check.iter())
                .chain(a.metrics.terminal.iter())
                .chain(a.metrics.lesson_tabs.iter())
                .chain(a.metrics.example_action.iter())
                .chain(a.metrics.assignment_action.iter())
                .chain(a.metrics.command_insert.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?} {w}/{scale} {r:?}"
                );
            }
            if w == 1200. {
                if view == teaching::View::Introduction {
                    assert!(a.metrics.example_action.unwrap().bottom() < 850.);
                } else {
                    assert!(a.metrics.check.unwrap().bottom() < 850.);
                }
            }
            a.progress
                .attempts
                .get_mut("lpic2-service")
                .unwrap()
                .challenge = true;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            assert_eq!(a.service_view(6), teaching::View::Assignment);
            assert!(a.metrics.lesson_tabs.is_empty());
            assert!(a.metrics.command_insert.is_none());
            assert!(a.metrics.terminal.is_some());
            assert!(a.metrics.editor_save.is_some());
            assert!(!a.metrics.help_visible);
        }
    }
}
#[test]
fn service_example_and_assignment_editor_terminal_undo_histories_are_isolated() {
    for terminal in [false, true] {
        let (ctx, mut a) = app();
        a.launch(6);
        a.set_service_view(6, teaching::View::Example);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        let command = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        for text in ["example-draft", "example-edit"] {
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![command(egui::Key::A), egui::Event::Text(text.into())],
            );
            draw(&ctx, &mut a);
        }
        let example = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .clone()
            .unwrap();
        assert_eq!(
            if terminal {
                &example.command
            } else {
                &example.port_draft
            },
            "example-edit"
        );
        a.set_service_view(6, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        let before = serde_json::to_value(&a.progress.attempts["lpic2-service"]).unwrap();
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(
            serde_json::to_value(&a.progress.attempts["lpic2-service"]).unwrap(),
            before
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                command(egui::Key::A),
                egui::Event::Text("assignment-draft".into()),
            ],
        );
        draw(&ctx, &mut a);
        let draft = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap();
        let draft = if terminal {
            draft.command.clone()
        } else {
            draft.service_port.clone().unwrap()
        };
        assert_eq!(draft, "assignment-draft");
        a.set_service_view(6, teaching::View::Example);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        let w = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap();
        assert_eq!(
            if terminal {
                &w.command
            } else {
                w.service_port.as_ref().unwrap()
            },
            "assignment-draft"
        );
    }
}

fn accessibility_tree(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
) -> egui::accesskit::TreeUpdate {
    ctx.enable_accesskit();
    let mut result = None;
    for _ in 0..3 {
        result = ctx
            .run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(w / a.scale, h / a.scale),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            )
            .platform_output
            .accesskit_update;
    }
    result.expect("native accessibility tree enabled")
}
fn accessible_named<'a>(
    tree: &'a egui::accesskit::TreeUpdate,
    name: &str,
) -> &'a egui::accesskit::Node {
    &tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(name))
        .unwrap_or_else(|| {
            panic!(
                "Missing native name {name}; names: {:?}",
                tree.nodes
                    .iter()
                    .filter_map(|(_, n)| n.label())
                    .collect::<Vec<_>>()
            )
        })
        .1
}
#[test]
fn native_accessibility_shared_assignments_expose_names_without_losing_values_or_roles() {
    use egui::accesskit::Role;
    let (_, inventory) = app();
    let labs: Vec<_> = inventory
        .scenarios
        .iter()
        .enumerate()
        .filter(|(i, s)| {
            *i < 9 || matches!(s.initial, State::Script { .. } | State::Filesystem { .. })
        })
        .map(|(i, _)| i)
        .collect();
    for i in labs {
        for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            a.launch(i);
            a.scale = scale;
            let id = a.scenarios[i].id.clone();
            a.progress.attempts.get_mut(&id).unwrap().challenge = true;
            if let State::Script { source, .. } =
                &mut a.progress.attempts.get_mut(&id).unwrap().state
            {
                *source = "printf '%s\\n' 'Editable source'\n".into();
            }
            let tree = accessibility_tree(&ctx, &mut a, w, h);
            let mut count = 0;
            for (_, node) in &tree.nodes {
                if matches!(
                    node.role(),
                    Role::TextInput | Role::MultilineTextInput | Role::ComboBox
                ) {
                    assert!(
                        node.label().is_some_and(|s| !s.trim().is_empty()),
                        "lab{i} {w}/{scale}: unnamed {:?},value={:?},placeholder={:?}",
                        node.role(),
                        node.value(),
                        node.placeholder()
                    );
                    if matches!(node.role(), Role::TextInput | Role::MultilineTextInput) {
                        count += 1;
                    }
                }
            }
            assert!(count > 0, "lab{i} fields absent");
            let size = accessible_named(&tree, "Text size");
            assert_eq!(size.role(), Role::ComboBox);
            assert_eq!(
                size.value(),
                Some(if scale == 1. { "100%" } else { "150%" })
            );
            if matches!(a.scenarios[i].initial, State::Script { .. }) {
                let source = accessible_named(&tree, "Assignment script source");
                assert_eq!(source.role(), Role::MultilineTextInput);
                assert_eq!(source.value(), Some("printf '%s\\n' 'Editable source'\n"));
                assert_eq!(
                    accessible_named(&tree, "Inspect script test inputs").role(),
                    Role::ComboBox
                );
            }
            if i == 1 {
                for (name, hosts) in [("design", 60), ("support", 28), ("lab", 12)] {
                    assert_eq!(
                        accessible_named(
                            &tree,
                            &format!("{name} — network/prefix for {hosts} required hosts")
                        )
                        .role(),
                        Role::TextInput
                    );
                }
            }
        }
    }
}
#[test]
fn native_accessibility_examples_name_their_scope_and_preserve_editor_and_terminal_values() {
    use egui::accesskit::Role;
    for i in [1, 6, 9, 24] {
        let (ctx, mut a) = app();
        a.launch(i);
        match i {
            1 => a.set_address_view(i, teaching::View::Example),
            6 => a.set_service_view(i, teaching::View::Example),
            9 => a.set_script_view(i, teaching::View::Example),
            24 => a.set_terminal_view(i, teaching::View::Example),
            _ => unreachable!(),
        }
        let id = a.scenarios[i].id.clone();
        let w = a
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.service_learning {
            l.port_draft = "9091".into();
            l.command = "systemctl status atlas".into();
        }
        if let Some(l) = &mut w.script_learning {
            l.example_source = "printf '%s\\n' 'Example source'\n".into();
        }
        if let Some(l) = &mut w.terminal_learning {
            l.command = "pwd".into();
        }
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        let checks = match i {
            1 => vec![("staff — network/prefix for 30 required hosts", "")],
            6 => vec![
                (
                    "Practice example service configuration port — ungraded",
                    "9091",
                ),
                (
                    "Service practice terminal command — ungraded",
                    "systemctl status atlas",
                ),
            ],
            9 => vec![(
                "Practice example script source — ungraded",
                "printf '%s\\n' 'Example source'\n",
            )],
            24 => vec![("Practice example terminal command — ungraded", "pwd")],
            _ => unreachable!(),
        };
        for (name, value) in checks {
            let n = accessible_named(&tree, name);
            assert_eq!(
                n.role(),
                if i == 9 {
                    Role::MultilineTextInput
                } else {
                    Role::TextInput
                }
            );
            assert_eq!(n.value(), Some(value));
        }
        for (_, n) in &tree.nodes {
            if n.role() == Role::ComboBox {
                assert!(
                    n.label().is_some(),
                    "example{i} unnamed picker {:?}",
                    n.value()
                );
            }
        }
    }
}
#[test]
fn native_accessibility_library_and_appearance_keep_filter_values_card_context_and_disabled_states()
{
    use egui::accesskit::Role;
    let (ctx, mut a) = app();
    let mut tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert!(tree.nodes.iter().any(|(_, n)| n.role() == Role::Button
        && n.label().is_some_and(|s| s.starts_with("Start lab: "))));
    assert!(tree.nodes.iter().any(|(_, n)| n.role() == Role::Button
        && n.label().is_some_and(|s| s.starts_with("Add favorite: "))));
    a.filter.search = "repair".into();
    a.filter.topic = "Services".into();
    a.filter.unit = "linux-service-recovery".into();
    a.filter.track = "LPIC-2".into();
    a.filter.difficulty = "Beginner".into();
    a.filter.status = "Complete".into();
    a.filter.max_minutes = 8;
    a.extra_filters = true;
    a.scale = 1.5;
    tree = accessibility_tree(&ctx, &mut a, 960., 740.);
    for (name, value) in [
        ("Search labs by title, goal or topic", "repair"),
        ("Topic filter", "Services"),
        ("Difficulty filter", "Beginner"),
        ("Completion status filter", "Complete"),
        ("Estimated time filter", "Up to 8 min"),
    ] {
        let n = accessible_named(&tree, name);
        assert_eq!(n.value(), Some(value));
    }
    assert_eq!(
        accessible_named(&tree, "Course unit filter").role(),
        Role::ComboBox
    );
    assert_eq!(
        accessible_named(&tree, "Application theme").role(),
        Role::ComboBox
    );
    a.theme_preview = true;
    tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert_eq!(
        accessible_named(&tree, "Theme sample editor — appearance example only").value(),
        Some("Select this editor text")
    );
    assert!(accessible_named(&tree, "Unavailable (disabled)").is_disabled());
}
#[test]
fn native_accessibility_backup_filename_has_a_name_and_retains_the_proposed_value() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
    let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    let filename = accessible_named(&tree, "Progress backup filename");
    assert_eq!(filename.role(), egui::accesskit::Role::TextInput);
    assert_eq!(filename.value(), Some("dlms-learning-backup.json"));
    assert!(!filename.is_disabled());
}

#[test]
fn shared_singleline_centers_real_galleys_and_placeholders_at_font_metric_heights() {
    for body_size in [16., 24., 32.] {
        for text in [
            "",
            "192.168.10.77",
            "a very long value that scrolls within this bounded single-line field",
        ] {
            let (ctx, _) = app();
            ctx.style_mut(|s| {
                s.text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(body_size));
            });
            let mut value = text.to_owned();
            let mut geometry = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., 600.),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    controls::configure(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            let h = controls::height(ui);
                            let field = controls::singleline(ui, &mut value)
                                .hint_text("IPv4 address")
                                .desired_width(240.)
                                .show(ui);
                            let button = ui.add(button("Check"));
                            let combo = egui::ComboBox::from_id_salt("alignment-test")
                                .selected_text("Selected")
                                .show_ui(ui, |_| {})
                                .response;
                            assert!((field.response.rect.height() - h).abs() < 1.);
                            assert!((button.rect.height() - h).abs() < 1.);
                            assert!((combo.rect.height() - h).abs() < 1.);
                            assert!(
                                (field.response.rect.center().y - button.rect.center().y).abs()
                                    < 1.
                            );
                            assert!(
                                (field.response.rect.center().y - combo.rect.center().y).abs() < 1.
                            );
                            let galley =
                                egui::Rect::from_min_size(field.galley_pos, field.galley.size());
                            assert!(
                                (galley.center().y - field.response.rect.center().y).abs() < 1.
                            );
                            assert!(field.text_clip_rect.left() >= field.response.rect.left());
                            assert!(field.text_clip_rect.right() <= field.response.rect.right());
                            geometry = Some(field.response.rect);
                        });
                    });
                },
            );
            fn find_hint(s: &egui::Shape) -> Option<egui::Rect> {
                match s {
                    egui::Shape::Text(t) if t.galley.job.text == "IPv4 address" => {
                        Some(egui::Rect::from_min_size(t.pos, t.galley.size()))
                    }
                    egui::Shape::Vec(v) => v.iter().find_map(find_hint),
                    _ => None,
                }
            }
            if text.is_empty() {
                let hint = output
                    .shapes
                    .iter()
                    .find_map(|s| find_hint(&s.shape))
                    .expect("real painted placeholder");
                assert!((hint.center().y - geometry.unwrap().center().y).abs() < 1.);
            }
            assert_eq!(value, text);
        }
    }
}

#[test]
fn all_authored_assignment_controls_fit_at_two_hundred_percent_and_vlandrafts_survive_reflow() {
    let (_, inventory) = app();
    for i in 0..inventory.scenarios.len() {
        for (w, h) in [(1920., 1080.), (720., 700.)] {
            let (ctx, mut a) = app();
            open_assignment(&mut a, i);
            a.scale = 2.;
            a.progress
                .attempts
                .get_mut(&a.scenarios[i].id)
                .unwrap()
                .challenge = true;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.terminal.iter())
                .chain(a.metrics.check.iter())
                .chain(a.metrics.editor_save.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "lab{i} {w}/200%: {r:?} vs {:?}",
                    ctx.content_rect()
                );
            }
        }
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| matches!(s.initial, State::Switching { .. }))
        .unwrap();
    open_assignment(&mut a, i);
    a.progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .challenge = true;
    for _ in 0..4 {
        frame(&ctx, &mut a, 1920., 1080., vec![]);
    }
    let id = a.metrics.vlan_fields[0].0;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        &ctx,
        &mut a,
        1920.,
        1080.,
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
            egui::Event::Text("30".into()),
        ],
    );
    for _ in 0..4 {
        frame(&ctx, &mut a, 720., 700., vec![]);
    }
    assert_eq!(
        a.metrics.vlan_fields[0].0, id,
        "same actual focused editor across wide/stacked grid"
    );
    assert!(ctx.memory(|m| m.has_focus(id)));
    let draft = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .switching_draft
        .as_ref()
        .unwrap();
    assert_eq!(
        draft.switches[&draft.selected_switch]
            .ports
            .values()
            .next()
            .unwrap()
            .access,
        "30"
    );
    frame(
        &ctx,
        &mut a,
        720.,
        700.,
        vec![egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        }],
    );
    let draft = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .switching_draft
        .as_ref()
        .unwrap();
    assert_eq!(
        draft.switches[&draft.selected_switch]
            .ports
            .values()
            .next()
            .unwrap()
            .access,
        "20"
    );
}

#[test]
fn two_hundred_percent_appearance_selection_round_trips_without_changing_progress() {
    let temp = tempfile::tempdir().unwrap();
    let p = temp.path().join("preferences-v1.json");
    let prefs = theme::Preferences {
        version: 1,
        theme: "dlms-light".into(),
        text_scale: 2.,
    };
    prefs.save(&p).unwrap();
    assert_eq!(theme::Preferences::load(&p).unwrap().text_scale, 2.);
}

#[test]
fn shared_native_field_clicks_select_and_edit_the_character_under_the_cursor() {
    fn edit_frame(
        ctx: &egui::Context,
        value: &mut String,
        events: Vec<egui::Event>,
    ) -> egui::text_edit::TextEditOutput {
        let mut output = None;
        let modifiers = events
            .iter()
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
                    egui::vec2(800., 600.),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| {
                controls::configure(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    output = Some(
                        controls::singleline(ui, value)
                            .id(egui::Id::new("aligned-hit-editor"))
                            .desired_width(240.)
                            .show(ui),
                    );
                });
            },
        );
        output.unwrap()
    }
    for size in [16., 32.] {
        let (ctx, _) = app();
        ctx.style_mut(|s| {
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(size));
        });
        let mut value = "ABCD".to_owned();
        let o = edit_frame(&ctx, &mut value, vec![]);
        let cursor = o.galley.pos_from_cursor(egui::text::CCursor::new(2));
        let point = egui::pos2(
            o.galley_pos.x + cursor.center().x,
            o.response.rect.center().y,
        );
        for pressed in [true, false] {
            let o = edit_frame(
                &ctx,
                &mut value,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
            assert_eq!(o.cursor_range.unwrap().primary.index, 2);
        }
        let o = edit_frame(
            &ctx,
            &mut value,
            vec![egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::SHIFT,
            }],
        );
        let selection = o.cursor_range.unwrap();
        assert_eq!(
            selection.primary.index.abs_diff(selection.secondary.index),
            1
        );
        edit_frame(&ctx, &mut value, vec![egui::Event::Text("X".into())]);
        assert_eq!(value, "ABXD");
    }
}
