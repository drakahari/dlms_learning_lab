//! Actual egui interactions on fresh private profiles; not native-device acceptance.
use super::debug_recovery::{paint, settle, tap};
use super::*;
use dlms_learning_lab::{
    network_transfer as core, network_transfer_source as binding,
    network_transfer_workspace as work,
};
const IDS: [&str; 3] = [
    "ccna-tftp-recovery-copy",
    "ccna-ftp-archive-channel",
    "ccna-transfer-file-handover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    assert_eq!(a.scenarios.len(), 458);
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn simulation(a: &LabApp) -> &work::Session {
    let State::NetworkTransfer { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    settle(ctx, a);
    let r = a
        .metrics
        .network_transfer
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1;
    tap(ctx, a, r);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, label: &str) {
    click(ctx, a, key);
    click(ctx, a, &format!("{key}:{label}"));
}
fn type_at(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, text: &str) {
    tap(ctx, a, r);
    paint(
        ctx,
        a,
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
            if text.is_empty() {
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }
            } else {
                egui::Event::Text(text.into())
            },
        ],
    );
    settle(ctx, a);
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    settle(ctx, a);
    let r = a
        .metrics
        .network_transfer
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    type_at(ctx, a, r, text);
}
fn recipe(ctx: &egui::Context, a: &mut LabApp, d: &work::Draft) {
    choose(
        ctx,
        a,
        "protocol",
        if d.request.protocol == core::Protocol::Tftp {
            "TFTP"
        } else {
            "FTP · passive"
        },
    );
    choose(
        ctx,
        a,
        "direction",
        if d.request.direction == core::Direction::Download {
            "Download · server to device"
        } else {
            "Upload · device to server"
        },
    );
    choose(ctx, a, "server", &d.request.server);
    if let Some(login) = &d.request.login {
        choose(ctx, a, "login", login);
    }
    field(ctx, a, "local-path", &d.request.local_path);
    field(ctx, a, "remote-path", &d.request.remote_path);
    for (name, text) in &d.documents.routers {
        if let Some(r) = a
            .metrics
            .network_transfer
            .controls
            .iter()
            .find(|r| r.0 == format!("policy-{name}"))
            .map(|r| r.1)
        {
            type_at(ctx, a, r, text);
        }
    }
}
fn recipes(s: &Scenario) -> Vec<work::Draft> {
    s.solution
        .iter()
        .filter_map(|v| {
            match serde_json::from_str::<binding::Event>(v.strip_prefix(binding::PREFIX).unwrap())
                .unwrap()
            {
                binding::Event::Edit { draft } => Some(draft),
                _ => None,
            }
        })
        .collect()
}
fn visible_text(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 9000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in output.shapes {
        collect(&s.shape, &mut text);
    }
    text
}
#[test]
fn guided_fill_is_inert_isolated_and_runtime_copy_success_is_distinct_from_task_success() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let text = visible_text(&ctx, &mut a);
    assert!(text.contains("TFTP has no directory-listing command"));
    assert!(text.contains("Worked example"));
    click(&ctx, &mut a, "example");
    let original = simulation(&a).assignment.clone();
    click(&ctx, &mut a, "transfer");
    assert!(
        simulation(&a)
            .practice
            .last_report(&binding::source(&a.scenarios[i]).unwrap().example)
            .unwrap()
            .unwrap()
            .success
    );
    click(&ctx, &mut a, "check");
    let text = visible_text(&ctx, &mut a);
    assert!(text.contains("File handover needs attention"));
    assert!(text.contains("Needs attention · Current supplied files"));
    assert!(!text.contains("Assignment complete"));
    assert!(!a.metrics.completion);
    let history = simulation(&a).practice.history.clone();
    click(&ctx, &mut a, "worked");
    assert_eq!(simulation(&a).practice.history, history);
    assert!(!simulation(&a).practice.checked);
    assert!(simulation(&a).practice.last.is_none());
    assert_eq!(simulation(&a).assignment, original);
    click(&ctx, &mut a, "transfer");
    click(&ctx, &mut a, "check");
    assert!(visible_text(&ctx, &mut a).contains("Example checks pass"));
    assert_eq!(simulation(&a).assignment, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    click(&ctx, &mut a, "reset-example");
    assert!(simulation(&a).practice.history.is_empty());
    assert_eq!(simulation(&a).assignment, original);
}
#[test]
fn all_three_assignments_accept_actual_native_controls_then_invalidate_after_broken_copies() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        click(&ctx, &mut a, "assignment");
        let s = a.scenarios[i].clone();
        let plan = recipes(&s);
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        for d in &plan {
            recipe(&ctx, &mut a, d);
            click(&ctx, &mut a, "transfer");
        }
        click(&ctx, &mut a, "check");
        assert!(
            progress::current_solved(&a.progress.attempts[id], &s),
            "{}: {:?}",
            id,
            engine::checks(&a.progress.attempts[id].state, &s.goal)
        );
        assert!(a.metrics.completion);
        assert!(visible_text(&ctx, &mut a).contains("Assignment complete"));
        let unchanged = simulation(&a).assignment.draft.request.local_path.clone();
        let history = simulation(&a).assignment.history.clone();
        field(&ctx, &mut a, "local-path", &unchanged);
        assert!(
            progress::current_solved(&a.progress.attempts[id], &s),
            "Retyping an identical pending request does not break already-correct files"
        );
        assert_eq!(
            simulation(&a).assignment.history,
            history,
            "Typing alone never transfers files"
        );
        if id == IDS[1] {
            field(&ctx, &mut a, "local-path", "note.txt");
        } else {
            field(&ctx, &mut a, "remote-path", "public/outdated.cfg");
        }
        assert!(!a.metrics.completion);
        click(&ctx, &mut a, "transfer");
        click(&ctx, &mut a, "check");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(!a.metrics.completion);
        assert!(visible_text(&ctx, &mut a).contains("File handover needs attention"));
        a.save();
        let loaded = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        assert!(!progress::current_solved(&loaded.attempts[id], &s));
    }
}
#[test]
fn request_fields_keep_focus_tab_and_do_not_run_on_enter_and_challenge_keeps_guidance_optional() {
    let (ctx, mut a, i) = opened(IDS[1]);
    click(&ctx, &mut a, "assignment");
    field(
        &ctx,
        &mut a,
        "local-path",
        "a-valid-long-relative-source-file.cfg",
    );
    let local = a
        .metrics
        .network_transfer
        .fields
        .iter()
        .find(|r| r.0 == "local-path")
        .unwrap()
        .2;
    let remote = a
        .metrics
        .network_transfer
        .fields
        .iter()
        .find(|r| r.0 == "remote-path")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(local));
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(remote));
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(simulation(&a).assignment.history.is_empty());
    let draft = simulation(&a).assignment.draft.clone();
    let policy = a.metrics.network_transfer.editor.unwrap();
    tap(&ctx, &mut a, policy.1);
    assert_eq!(ctx.memory(|m| m.focused()), Some(policy.2));
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert_ne!(
        ctx.memory(|m| m.focused()),
        Some(policy.2),
        "The configuration editor must not trap Tab navigation"
    );
    assert_eq!(
        simulation(&a).assignment.draft,
        draft,
        "Navigating out of the editor does not insert a tab or change policy"
    );
    a.progress.attempts.get_mut(IDS[1]).unwrap().challenge = true;
    settle(&ctx, &mut a);
    let text = visible_text(&ctx, &mut a);
    assert!(!text.contains("Fill worked transfer"));
    assert!(!text.contains("Worked example ·"));
    assert!(!text.contains("Reproduce the data failure"));
    assert!(text.contains("YOUR TASK"));
    assert!(text.contains("Transfer request"));
    assert_eq!(simulation(&a).assignment.draft, draft);
    assert_eq!(a.scenarios[i].id, IDS[1]);
}
#[test]
fn request_text_and_placeholders_are_centered_and_reflow_with_readable_native_controls() {
    let (ctx, mut a, _) = opened(IDS[2]);
    click(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "local-path", "");
    assert!(
        simulation(&a)
            .assignment
            .draft
            .request
            .local_path
            .is_empty()
    );
    field(
        &ctx,
        &mut a,
        "remote-path",
        "archive/a-long-configuration-file-name.cfg",
    );
    let remote_id = a
        .metrics
        .network_transfer
        .fields
        .iter()
        .find(|r| r.0 == "remote-path")
        .unwrap()
        .2;
    for light in [false, true] {
        let theme = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap();
        a.theme_id = theme.id.clone();
        theme::apply(&ctx, &a.theme_id);
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [1100., 1900.] {
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 15000., vec![]);
                }
                assert_eq!(a.metrics.network_transfer.fields.len(), 2);
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    Some(remote_id),
                    "Reflow must preserve the active input and its cursor"
                );
                let local = &a.metrics.network_transfer.fields[0];
                let remote = &a.metrics.network_transfer.fields[1];
                assert!((local.1.width() - remote.1.width()).abs() < 1.);
                assert!((local.1.height() - remote.1.height()).abs() < 1.);
                for (_, rect, _, clip) in &a.metrics.network_transfer.fields {
                    assert!(rect.width() > 50.);
                    assert!(
                        rect.min.x >= clip.min.x - 1. && rect.max.x <= clip.max.x + 1.,
                        "{width} {scale} {rect:?} {clip:?}"
                    );
                }
                let (_, viewport, _) = a.metrics.network_transfer.editor.unwrap();
                let policy = a
                    .metrics
                    .network_transfer
                    .controls
                    .iter()
                    .find(|r| r.0 == "policy-R1")
                    .unwrap()
                    .1;
                assert_eq!(
                    policy, viewport,
                    "Capture/click geometry uses the nested viewport rather than the full scrolling document"
                );
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width / scale, 15000. / scale),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                fn endpoint_rows(shape: &egui::Shape, rows: &mut Vec<usize>) {
                    match shape {
                        egui::Shape::Text(t)
                            if t.galley.job.text.contains("Transfer port:")
                                || t.galley.job.text.contains("Passive data:") =>
                        {
                            rows.push(t.galley.rows.len())
                        }
                        egui::Shape::Vec(v) => {
                            for s in v {
                                endpoint_rows(s, rows)
                            }
                        }
                        _ => {}
                    }
                }
                let mut rows = vec![];
                for s in &output.shapes {
                    endpoint_rows(&s.shape, &mut rows);
                }
                assert_eq!(
                    rows.len(),
                    6,
                    "All three samples retain both server endpoints at {width} {scale}"
                );
                assert!(
                    rows.iter().all(|n| *n == 2),
                    "Endpoint columns must not split port labels into tiny fragments: {rows:?} at {width} {scale}"
                );
                fn center(shape: &egui::Shape, rect: egui::Rect, expected: &str) -> Option<f32> {
                    match shape {
                        egui::Shape::Text(t)
                            if t.galley.job.text == expected
                                && t.pos.y >= rect.top()
                                && t.pos.y <= rect.bottom() =>
                        {
                            Some((t.pos.y + t.galley.size().y / 2. - rect.center().y).abs())
                        }
                        egui::Shape::Vec(v) => v.iter().find_map(|s| center(s, rect, expected)),
                        _ => None,
                    }
                }
                for (key, expected) in [
                    ("local-path", "Relative file name"),
                    ("remote-path", "archive/a-long-configuration-file-name.cfg"),
                ] {
                    let rect = a
                        .metrics
                        .network_transfer
                        .fields
                        .iter()
                        .find(|r| r.0 == key)
                        .unwrap()
                        .1;
                    let delta = output
                        .shapes
                        .iter()
                        .find_map(|s| center(&s.shape, rect, expected))
                        .unwrap_or_else(|| {
                            panic!("Missing actual single-line galley for {key} {width} {scale}")
                        });
                    assert!(
                        delta <= 2.,
                        "Single-line text center {key}: {delta} at {width} {scale}"
                    );
                }
            }
        }
    }
}
#[test]
fn header_reset_and_disk_resume_keep_other_labs_and_favorites_untouched() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let other = a.scenarios.iter().position(|s| s.id == IDS[1]).unwrap();
    a.launch(other);
    let other_before = serde_json::to_value(&a.progress.attempts[IDS[1]]).unwrap();
    a.launch(i);
    a.progress.favorites.insert(IDS[0].into());
    click(&ctx, &mut a, "assignment");
    let s = a.scenarios[i].clone();
    for d in recipes(&s) {
        recipe(&ctx, &mut a, &d);
        click(&ctx, &mut a, "transfer");
    }
    click(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    a.save();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(progress::current_solved(&loaded.attempts[IDS[0]], &s));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[IDS[1]]).unwrap(),
        other_before
    );
    assert!(loaded.favorites.contains(IDS[0]));
    settle(&ctx, &mut a);
    let reset = a.metrics.reset.expect("actual header Reset button");
    tap(&ctx, &mut a, reset);
    settle(&ctx, &mut a);
    let confirm = a.metrics.confirm_reset.expect("actual reset confirmation");
    tap(&ctx, &mut a, confirm);
    settle(&ctx, &mut a);
    assert_eq!(a.progress.attempts[IDS[0]].state, s.initial);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[1]]).unwrap(),
        other_before
    );
    assert!(a.progress.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], &s));
}

#[test]
fn native_editor_reveal_uses_the_whole_viewport_and_can_scroll_it_fully_onscreen() {
    for scale in [1.5, 2.] {
        let (ctx, mut a, _) = opened(IDS[1]);
        click(&ctx, &mut a, "assignment");
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for _ in 0..4 {
            frame(&ctx, &mut a, 946., 929., vec![]);
        }
        let target = |a: &LabApp| {
            a.metrics
                .network_transfer
                .controls
                .iter()
                .find(|r| r.0 == "policy-R1")
                .unwrap()
                .1
        };
        let before = target(&a);
        let fragment = a.metrics.network_transfer.editor.unwrap().1;
        assert!(before.height() > 0.);
        assert!(
            before.height() > fragment.height() + 1.,
            "Offscreen editor must retain its logical viewport rather than only its visible fragment"
        );
        let mut visible = false;
        for _ in 0..20 {
            if a.reveal_smoke_control(&ctx, target(&a)) {
                visible = true;
                break;
            }
            let clip = a.metrics.main_viewport.unwrap();
            let region = a.metrics.main_scroll_region.unwrap();
            let gutter = egui::Rect::from_min_max(
                egui::pos2(clip.right(), clip.top()),
                egui::pos2(region.right(), clip.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(gutter.width() * ctx.pixels_per_point() > 1.);
            let mut events = std::mem::take(&mut a.pending_events);
            for event in &mut events {
                if let egui::Event::PointerMoved(p) = event {
                    *p = gutter.center();
                }
            }
            frame(&ctx, &mut a, 946., 929., events);
            for _ in 0..4 {
                frame(&ctx, &mut a, 946., 929., vec![]);
            }
        }
        assert!(
            visible,
            "Actual editor viewport must become fully visible at {scale}"
        );
        let rect = target(&a);
        assert!(super::super::smoke::smoke_control_visible(
            rect,
            a.metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ));
        assert!(
            (rect.height() - before.height()).abs() < 1.,
            "Revealing cannot shrink the capture target to force a pass"
        );
    }
}
