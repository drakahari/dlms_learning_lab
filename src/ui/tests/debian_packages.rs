//! Prospective actual-egui checks; private fixture appended only in these tests.
use super::*;
use dlms_learning_lab::{debian_packages_terminal::Session, script_teaching::View};
const ID: &str = "development-debian-packages";
fn opened(view: View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let fixture = pack::load(include_str!(
        "../../../tests/fixtures/debian-packages-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    assert!(!a.scenarios.iter().any(|s| s.id == ID));
    let i = a.scenarios.len();
    a.scenarios.push(fixture);
    a.launch(i);
    a.set_debian_packages_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn terminal(a: &LabApp, practice: bool) -> &Session {
    let w = a.progress.attempts[ID].workspace.as_ref().unwrap();
    if practice {
        w.debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
    } else {
        w.debian_packages_terminal.as_ref().unwrap()
    }
}
fn find(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
    f: fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for _ in 0..30 {
        let r = f(&a.metrics).expect("actual package control exists");
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let t = 1. / ctx.pixels_per_point();
        if clip.contains(r.center()) && clip.expand(t).contains_rect(r) {
            return r;
        }
        let dy = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom()
        } else {
            clip.top() - r.top()
        };
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..4 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("actual package control must become visible");
}
fn point_click(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, r: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            w,
            h,
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
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn command(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, text: &str) {
    a.focus_input = false;
    let r = find(ctx, a, w, h, |m| m.package_command.map(|(r, _, _)| r));
    point_click(ctx, a, w, h, r);
    let id = a.metrics.package_command.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        w,
        h,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn run(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    command(ctx, a, 1200., 850., text);
    frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
    draw(ctx, a);
}
fn check_work(ctx: &egui::Context, a: &mut LabApp) {
    let r = find(ctx, a, 1200., 850., |m| m.check);
    point_click(ctx, a, 1200., 850., r);
}
#[test]
fn package_intro_and_actual_insert_only_keep_the_example_and_assignment_separate() {
    let (ctx, mut a, i) = opened(View::Introduction);
    assert!(a.metrics.package_command.is_none());
    let original = a.progress.attempts[ID].state.clone();
    let r = find(&ctx, &mut a, 1200., 850., |m| m.example_action);
    point_click(&ctx, &mut a, 1200., 850., r);
    assert_eq!(a.debian_packages_view(i), View::Example);
    let before = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap()
        .simulation
        .clone();
    let r = find(&ctx, &mut a, 1200., 850., |m| m.command_insert);
    point_click(&ctx, &mut a, 1200., 850., r);
    assert!(
        terminal(&a, true)
            .command
            .starts_with("apt-get -y --reinstall")
    );
    assert!(terminal(&a, true).entries.is_empty());
    assert_eq!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .simulation,
        before
    );
    assert_eq!(a.progress.attempts[ID].state, original);
    let r = find(&ctx, &mut a, 1200., 850., |m| m.package_run.map(|(r, _)| r));
    point_click(&ctx, &mut a, 1200., 850., r);
    check_work(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap();
    assert!(
        l.checks(a.scenarios[i].debian_packages_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert_eq!(a.progress.attempts[ID].state, original);
}
#[test]
fn package_actual_terminal_checks_are_fresh_query_preserves_then_remove_invalidates_and_repair_restores()
 {
    let (ctx, mut a, i) = opened(View::Assignment);
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    run(&ctx, &mut a, "dpkg-query -W museum-report");
    assert_eq!(terminal(&a, false).entries.last().unwrap().status, 0);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    run(
        &ctx,
        &mut a,
        "apt-get -y --reinstall install museum-report=1.0-1",
    );
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
    run(&ctx, &mut a, "dpkg-query -S /usr/bin/museum-report");
    assert!(a.progress.attempts[ID].workspace.as_ref().unwrap().checked);
    assert!(a.metrics.completion);
    run(&ctx, &mut a, "dpkg -r museum-report");
    assert!(!a.progress.attempts[ID].workspace.as_ref().unwrap().checked);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    run(&ctx, &mut a, "apt-get -y install museum-report=1.0-1");
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn package_syntax_retry_retains_actual_input_status_and_state_then_real_run_recovers() {
    let (ctx, mut a, _) = opened(View::Assignment);
    let initial = a.progress.attempts[ID].state.clone();
    run(&ctx, &mut a, "apt-get -y install 'unfinished");
    assert_eq!(a.progress.attempts[ID].state, initial);
    assert!(terminal(&a, false).command.ends_with("'unfinished"));
    assert!(
        terminal(&a, false)
            .entries
            .last()
            .unwrap()
            .diagnostic
            .is_some()
    );
    run(
        &ctx,
        &mut a,
        "apt-get -y --reinstall install museum-report=1.0-1",
    );
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn package_context_ids_keep_mode_undo_and_pending_command_independent_across_reflow() {
    let (ctx, mut a, i) = opened(View::Assignment);
    command(&ctx, &mut a, 1200., 850., "assignment query");
    let aid = a.metrics.package_command.unwrap().1;
    a.set_debian_packages_view(i, View::Example);
    draw(&ctx, &mut a);
    command(&ctx, &mut a, 1200., 850., "example query");
    let eid = a.metrics.package_command.unwrap().1;
    assert_ne!(aid, eid);
    for _ in 0..5 {
        frame(&ctx, &mut a, 720., 850., vec![]);
    }
    assert_eq!(a.metrics.package_command.unwrap().1, eid);
    assert!(ctx.memory(|m| m.has_focus(eid)));
    assert_eq!(terminal(&a, true).command, "example query");
    assert_eq!(terminal(&a, false).command, "assignment query");
    // egui may retain a cursor-only snapshot above the edit. Use bounded real
    // Undo events, asserting the other mode after each, without changing history.
    for _ in 0..3 {
        if terminal(&a, true).command != "example query" {
            break;
        }
        frame(&ctx, &mut a, 720., 850., vec![acl_ctrl(egui::Key::Z)]);
        assert_eq!(terminal(&a, false).command, "assignment query");
    }
    assert_ne!(terminal(&a, true).command, "example query");
}
#[test]
fn package_all_four_modals_block_focused_text_enter_and_actual_run_in_both_modes() {
    for view in [View::Assignment, View::Example] {
        for modal in 0..4 {
            for click_run in [false, true] {
                let (ctx, mut a, _) = opened(view);
                let practice = view == View::Example;
                command(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    if practice {
                        "dpkg -r gallery-report"
                    } else {
                        "dpkg -r museum-report"
                    },
                );
                let id = a.metrics.package_command.unwrap().1;
                let run = find(&ctx, &mut a, 1200., 850., |m| m.package_run.map(|(r, _)| r));
                a.focus_input = false;
                let before = serde_json::to_value(&a.progress.attempts).unwrap();
                match modal {
                    0 => a.reset_pending = true,
                    1 => a.settings_open = true,
                    2 => a.theme_preview = true,
                    _ => {
                        a.smoke = true;
                        a.settings_open = true;
                        a.settings_data = true;
                        draw(&ctx, &mut a);
                        let r = a.metrics.data_export.unwrap();
                        point_click(&ctx, &mut a, 1200., 850., r);
                        assert!(a.data_tools.modal_open());
                        a.settings_open = false;
                    }
                }
                draw(&ctx, &mut a);
                ctx.memory_mut(|m| m.request_focus(id));
                if click_run {
                    point_click(&ctx, &mut a, 1200., 850., run);
                } else {
                    frame(
                        &ctx,
                        &mut a,
                        1200.,
                        850.,
                        vec![
                            acl_ctrl(egui::Key::A),
                            egui::Event::Text("blocked replacement".into()),
                            acl_ctrl(egui::Key::Enter),
                        ],
                    );
                }
                assert_eq!(
                    serde_json::to_value(&a.progress.attempts).unwrap(),
                    before,
                    "{view:?}/{modal}/{click_run}"
                );
                assert!(!a.metrics.package_command.unwrap().2);
                assert!(!a.metrics.package_run.unwrap().1);
            }
        }
    }
}
#[test]
fn package_all_views_challenge_and_real_run_text_fit_at100_150_200_wide_and_narrow() {
    for scale in [1., 1.5, 2.] {
        for width in [1920., 720.] {
            for view in [View::Introduction, View::Example, View::Assignment] {
                let (ctx, mut a, i) = opened(view);
                a.scale = scale;
                for _ in 0..6 {
                    frame(&ctx, &mut a, width, 1080., vec![]);
                }
                let bounds = a.metrics.main_viewport.unwrap();
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .copied()
                    .chain(a.metrics.package_run.map(|(r, _)| r))
                    .chain(a.metrics.package_example)
                    .chain(a.metrics.command_insert)
                {
                    assert!(
                        r.left() >= bounds.left() - 1. && r.right() <= bounds.right() + 1.,
                        "{view:?}/{scale}/{width} {r:?} {bounds:?}"
                    );
                }
                if view != View::Introduction {
                    find(&ctx, &mut a, width, 1080., |m| {
                        m.package_run.map(|(r, _)| r)
                    });
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / scale, 1080. / scale),
                            )),
                            ..Default::default()
                        },
                        |ctx| a.render(ctx),
                    );
                    fn texts(s: &egui::Shape, out: &mut Vec<egui::Rect>) {
                        match s {
                            egui::Shape::Text(t) if t.galley.text() == "Run command" => {
                                out.push(egui::Rect::from_min_size(t.pos, t.galley.size()))
                            }
                            egui::Shape::Vec(v) => {
                                for s in v {
                                    texts(s, out);
                                }
                            }
                            _ => (),
                        }
                    }
                    let mut text = Vec::new();
                    for shape in &output.shapes {
                        texts(&shape.shape, &mut text);
                    }
                    assert_eq!(text.len(), 1);
                    // Smooth scrolling can move the row between renders.
                    // Compare painted text and its button from the SAME frame.
                    let r = a.metrics.package_run.unwrap().0;
                    assert!(
                        r.contains_rect(text[0]),
                        "{view:?}/{scale}/{width}: painted Run {:?} must fit actual button {r:?}",
                        text[0]
                    );
                }
                a.progress.attempts.get_mut(ID).unwrap().challenge = true;
                a.set_debian_packages_view(i, View::Assignment);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 1080., vec![]);
                }
                assert!(a.metrics.lesson_tabs.is_empty());
                assert!(a.metrics.command_insert.is_none());
                assert_eq!(a.debian_packages_view(i), View::Assignment);
            }
        }
    }
}
#[test]
fn package_long_diagnostic_wraps_in_actual_output_viewport_at_large_narrow_text() {
    for scale in [1.5, 2.] {
        let (ctx, mut a, _) = opened(View::Assignment);
        a.scale = scale;
        for _ in 0..4 {
            frame(&ctx, &mut a, 720., 1000., vec![]);
        }
        command(
            &ctx,
            &mut a,
            720.,
            1000.,
            "apt-get --unsupported install museum-report",
        );
        frame(
            &ctx,
            &mut a,
            720.,
            1000.,
            vec![key(egui::Key::Enter, false)],
        );
        for _ in 0..3 {
            frame(&ctx, &mut a, 720., 1000., vec![]);
        }
        assert!(terminal(&a, false).entries.last().unwrap().stderr.len() > 40);
        assert!(!a.metrics.package_diagnostics.is_empty());
        for (r, width) in &a.metrics.package_diagnostics {
            assert!(
                r.width() <= *width + 1.,
                "diagnostic should wrap, not force horizontal prose scrolling"
            );
        }
    }
}
#[test]
fn package_warm_resume_and_selected_reset_preserve_both_command_drafts_and_other_attempts() {
    let (ctx, mut a, i) = opened(View::Assignment);
    run(&ctx, &mut a, "dpkg-query -W museum-report");
    command(&ctx, &mut a, 1200., 850., "assignment malformed '");
    a.set_debian_packages_view(i, View::Example);
    draw(&ctx, &mut a);
    run(&ctx, &mut a, "dpkg-query -W gallery-report");
    command(&ctx, &mut a, 1200., 850., "practice malformed '");
    let snapshot = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-package-progress.json");
    progress::save(&path, &a.progress).unwrap();
    let original = progress::read_bounded(&path, 16_000_000).unwrap();
    let loaded = progress::load(&path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[ID]).unwrap(),
        snapshot
    );
    assert_eq!(progress::read_bounded(&path, 16_000_000).unwrap(), original);
    let (ctx2, mut b) = app();
    b.scenarios = a.scenarios.clone();
    b.progress = loaded;
    b.launch(i);
    draw(&ctx2, &mut b);
    let mut visited = snapshot.clone();
    visited["visited"] = b.progress.clock.into();
    assert_eq!(
        serde_json::to_value(&b.progress.attempts[ID]).unwrap(),
        visited
    );
    assert_eq!(terminal(&b, false).command, "assignment malformed '");
    assert_eq!(terminal(&b, true).command, "practice malformed '");
    let others = b
        .progress
        .attempts
        .iter()
        .filter(|(id, _)| id.as_str() != ID)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<std::collections::BTreeMap<_, _>>();
    b.reset_attempt(i);
    draw(&ctx2, &mut b);
    assert_eq!(b.progress.attempts[ID].state, b.scenarios[i].initial);
    assert!(
        b.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .debian_packages_terminal
            .is_none()
    );
    for (id, old) in others {
        assert_eq!(
            serde_json::to_value(&b.progress.attempts[&id]).unwrap(),
            old
        );
    }
}
