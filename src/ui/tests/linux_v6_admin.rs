//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{linux_v6_admin_source as binding, linux_v6_admin_workspace as work};
const ID: &str = "lpic2-router-ipv6-saved-settings";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 9000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::LinuxV6Administration { simulation } =
        &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::LinuxV6Administration { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .v6_admin
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {r:?} {clip:?}"
    );
    let p = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
    let id = a
        .metrics
        .v6_admin
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
            egui::Event::Text(text.into()),
        ],
    );
    settle(ctx, a);
}

#[test]
fn v6_admin_insert_is_inert_and_enter_runs_actual_simulated_command() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    let assignment = session(&a).assignment.clone();
    hit(&ctx, &mut a, "insert-command");
    assert!(!session(&a).practice.input.is_empty());
    assert!(session(&a).practice.output.is_empty());
    edit(&ctx, &mut a, "command", "ip -6 route show");
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
    settle(&ctx, &mut a);
    assert!(session(&a).practice.output.contains("$ ip -6 route show"));
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
}
#[test]
fn v6_admin_real_editor_save_check_and_stale_success_follow_native_controls() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let d = binding::source(&a.scenarios[i]).unwrap().clone();
    for action in &d.worked {
        match action {
            work::Action::EditFile { text } => edit(
                &ctx,
                &mut a,
                "editor",
                &text.replace("uplink0", "wan0").replace("edge0", "lan0"),
            ),
            work::Action::SaveFile => hit(&ctx, &mut a, "save"),
            work::Action::Command { input } => {
                edit(&ctx, &mut a, "command", input);
                hit(&ctx, &mut a, "run");
            }
            work::Action::Observe { .. } => hit(&ctx, &mut a, "observe"),
            work::Action::RunRestart => hit(&ctx, &mut a, "restart"),
            work::Action::Check => hit(&ctx, &mut a, "check"),
            _ => panic!("Unexpected reference action"),
        }
    }
    assert!(a.metrics.completion);
    assert!(
        session(&a)
            .assignment
            .last_restart
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .all(|c| c.passed)
    );
    let old = session(&a).assignment.saved_file.clone();
    edit(
        &ctx,
        &mut a,
        "editor",
        &(old + "net.ipv6.conf.all.forwarding=0\n"),
    );
    assert!(!a.metrics.completion && !session(&a).assignment.checked);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.v6_admin.result.is_some_and(|(p, n, _)| p < n));
}
#[test]
fn v6_admin_views_reflow_and_editor_footer_is_stable_across_scrollbars() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for (width, scale, light) in [(1450., 1., false), (760., 1.5, true), (760., 2., false)] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        a.theme_id = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap()
            .id
            .clone();
        theme::apply(&ctx, &a.theme_id);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            let State::LinuxV6Administration { simulation } =
                &mut a.progress.attempts.get_mut(ID).unwrap().state
            else {
                panic!()
            };
            simulation.view = view;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 12000., vec![]);
            }
            assert!(
                a.metrics
                    .v6_admin
                    .controls
                    .iter()
                    .all(|(_, r, _)| r.is_finite())
            );
            if view != teaching::View::Introduction {
                let get = |key: &str| {
                    a.metrics
                        .v6_admin
                        .controls
                        .iter()
                        .find(|r| r.0 == key)
                        .unwrap()
                        .1
                };
                let save = get("save");
                let editor = get("editor-viewport");
                assert!(save.top() >= editor.bottom());
                assert!(save.right() <= width + 1.);
                assert!(get("command").right() <= width + 1.);
            }
        }
    }
}
#[test]
fn v6_admin_challenge_hides_example_and_command_picker() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["example", "introduction", "command-choice", "worked"] {
        assert!(!a.metrics.v6_admin.controls.iter().any(|v| v.0 == key));
    }
    assert!(a.metrics.v6_admin.controls.iter().any(|v| v.0 == "editor"));
}
