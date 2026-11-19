//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{linux_v6_startup_source as binding, linux_v6_startup_workspace as work};
const ID: &str = "lpic2-ipv6-startup-policy-recovery";
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
    let State::LinuxV6Startup { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::LinuxV6Startup { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .v6_startup
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
        .v6_startup
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
fn v6_startup_insert_is_inert_and_enter_runs_actual_simulated_command() {
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
fn v6_startup_native_commands_save_check_and_editor_invalidate_current_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let action: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        match action {
            work::Action::Command { input } => {
                edit(&ctx, &mut a, "command", &input);
                hit(&ctx, &mut a, "run");
            }
            work::Action::RunStartup => hit(&ctx, &mut a, "startup"),
            work::Action::Check => hit(&ctx, &mut a, "check"),
            _ => panic!("Unexpected reference action"),
        }
    }
    assert!(a.metrics.completion);
    assert!(
        session(&a)
            .assignment
            .last_startup
            .as_ref()
            .unwrap()
            .checks
            .iter()
            .all(|c| c.passed)
    );
    let bad = session(&a)
        .assignment
        .editor
        .replace(":FORWARD DROP", ":FORWARD ACCEPT");
    edit(&ctx, &mut a, "editor", &bad);
    assert!(!a.metrics.completion && !session(&a).assignment.checked);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.v6_startup.result.is_some_and(|(p, n, _)| p < n));
}
#[test]
fn v6_startup_views_reflow_and_editor_footer_is_stable_across_scrollbars() {
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
            let State::LinuxV6Startup { simulation } =
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
                    .v6_startup
                    .controls
                    .iter()
                    .all(|(_, r, _)| r.is_finite())
            );
            if view != teaching::View::Introduction {
                let get = |key: &str| {
                    a.metrics
                        .v6_startup
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
fn v6_startup_challenge_hides_example_and_command_picker() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["example", "introduction", "command-choice", "worked"] {
        assert!(!a.metrics.v6_startup.controls.iter().any(|v| v.0 == key));
    }
    assert!(
        a.metrics
            .v6_startup
            .controls
            .iter()
            .any(|v| v.0 == "editor")
    );
}
#[test]
fn v6_startup_editor_scrolls_long_rules_without_soft_wrap_or_moving_save() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let definition = binding::source(&a.scenarios[i]).unwrap().clone();
    for (width, scale) in [(1450., 1.), (760., 1.5), (760., 2.)] {
        ctx.set_zoom_factor(scale);
        a.scale = scale;
        let paint = |a: &mut LabApp| {
            for _ in 0..3 {
                frame(&ctx, a, width, 12000., vec![]);
            }
        };
        let State::LinuxV6Startup { simulation } =
            &mut a.progress.attempts.get_mut(ID).unwrap().state
        else {
            panic!()
        };
        simulation
            .action(
                &definition,
                &work::Action::EditFile {
                    text: String::new(),
                },
                false,
            )
            .unwrap();
        paint(&mut a);
        let get = |a: &LabApp, key: &str| {
            a.metrics
                .v6_startup
                .controls
                .iter()
                .find(|c| c.0 == key)
                .unwrap()
                .1
        };
        let before = get(&a, "save");
        // Actual parser-valid source line is long; comments also exercise very wide drafts.
        let text = format!(
            "{}# {}\n",
            definition.assignment.initial.saved_filter,
            "long-comment ".repeat(90)
        );
        let State::LinuxV6Startup { simulation } =
            &mut a.progress.attempts.get_mut(ID).unwrap().state
        else {
            panic!()
        };
        simulation
            .action(&definition, &work::Action::EditFile { text }, false)
            .unwrap();
        paint(&mut a);
        let viewport = get(&a, "editor-viewport");
        let field = a
            .metrics
            .v6_startup
            .fields
            .iter()
            .find(|f| f.0 == "editor")
            .unwrap()
            .1;
        assert!(
            field.width() > viewport.width() * 2.,
            "long text must extend into horizontal scroll space, not wrap: {field:?} {viewport:?}"
        );
        let after = get(&a, "save");
        assert!(
            (before.top() - after.top()).abs() <= 1.,
            "scrollbars cannot move Save: {before:?} {after:?}"
        );
        assert!(after.top() >= viewport.bottom());
    }
}
