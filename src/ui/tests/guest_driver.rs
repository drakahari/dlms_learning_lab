//! Actual synthetic widgets; not authored-content, physical-input or novice acceptance.
use super::*;
use dlms_learning_lab::{guest_driver_source as binding, guest_driver_workspace as work};
#[path = "../../../tests/support/guest_driver_source_fixture.rs"]
mod setup;
fn painted(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 7000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        painted(ctx, a, vec![]);
    }
}
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let s = setup::scenario();
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    let State::GuestDrivers { simulation } = &mut a
        .progress
        .attempts
        .get_mut("test-guest-driver")
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::GuestDrivers { simulation } = &a.progress.attempts["test-guest-driver"].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    settle(ctx, a);
    let (_, rect, clip) = a
        .metrics
        .guest_driver
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            rect,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {rect:?} in {clip:?}"
    );
    let pos = rect.center();
    painted(
        ctx,
        a,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    painted(
        ctx,
        a,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    hit(ctx, a, "command");
    painted(
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
fn choose(ctx: &egui::Context, a: &mut LabApp, archive: &str, operation: &str) {
    hit(ctx, a, "archive");
    hit(ctx, a, &format!("archive:{archive}"));
    hit(ctx, a, "operation");
    hit(ctx, a, &format!("operation:{operation}"));
    hit(ctx, a, "apply-package");
}
#[test]
fn guest_native_package_and_terminal_repair_grades_partial_correct_and_broken_again() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.guest_driver.result.unwrap().1 > 0);
    choose(&ctx, &mut a, "support-current", "Install");
    assert!(!session(&a).assignment.checked);
    assert!(
        !session(&a)
            .assignment
            .runtime
            .modules
            .loaded
            .contains_key("lab_sensor")
    );
    edit(&ctx, &mut a, "depmod -a");
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    edit(&ctx, &mut a, "modprobe lab_sensor");
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert_eq!(a.metrics.guest_driver.result.unwrap().1, 0);
    binding::context(&a.progress.attempts[&source.id], &source).unwrap();
    choose(&ctx, &mut a, "support-current", "Remove");
    assert!(!a.metrics.completion && !session(&a).assignment.checked);
    assert!(
        session(&a)
            .assignment
            .runtime
            .modules
            .loaded
            .contains_key("lab_sensor")
    );
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.guest_driver.result.unwrap().1 > 0);
    assert!(
        a.metrics
            .guest_driver
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
}
#[test]
fn guest_example_insert_is_inert_run_uses_example_only_and_enter_executes_once() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    let original = session(&a).assignment.clone();
    let runtime = session(&a).practice.runtime.clone();
    hit(&ctx, &mut a, "insert-command");
    assert_eq!(session(&a).practice.input, "lspci -k");
    assert_eq!(session(&a).practice.runtime, runtime);
    assert!(session(&a).practice.output.is_empty());
    hit(&ctx, &mut a, "run");
    assert!(
        session(&a)
            .practice
            .output
            .contains("Guide reader controller")
    );
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    edit(&ctx, &mut a, "uname -r");
    painted(
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
    assert_eq!(session(&a).practice.output.matches("$ uname -r").count(), 1);
    assert!(session(&a).practice.input.is_empty());
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, original);
    a.progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .challenge = true;
    settle(&ctx, &mut a);
    assert!(
        !a.metrics
            .guest_driver
            .controls
            .iter()
            .any(|r| r.0 == "insert-command")
    );
}
#[test]
fn guest_controls_fit_populated_and_empty_fields_at_large_text_and_keep_escape_focus() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    hit(&ctx, &mut a, "archive");
    let id = a
        .metrics
        .guest_driver
        .focus
        .iter()
        .find(|r| r.0 == "archive")
        .unwrap()
        .1;
    assert!(egui::ComboBox::is_open(&ctx, id));
    painted(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert!(!egui::ComboBox::is_open(&ctx, id));
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    painted(
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
    assert!(ctx.memory(|m| m.focused()).is_some());
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    edit(
        &ctx,
        &mut a,
        "modinfo /lib/modules/6.8.0-lab/kernel/drivers/lab_sensor.ko",
    );
    for text in [
        "",
        "modinfo /lib/modules/6.8.0-lab/kernel/drivers/lab_sensor.ko",
    ] {
        edit(&ctx, &mut a, text);
        for (scale, width, theme) in [
            (1., 1450., "dlms-navy"),
            (1.5, 840., "dlms-light"),
            (2., 840., "dlms-navy"),
        ] {
            a.scale = scale;
            a.theme_id = theme.into();
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 7000., vec![]);
            }
            let (_, input, _) = a
                .metrics
                .guest_driver
                .fields
                .iter()
                .find(|r| r.0 == "command")
                .unwrap();
            let (_, run, clip) = a
                .metrics
                .guest_driver
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .unwrap();
            assert!((input.height() - run.height()).abs() <= 1.);
            assert!(
                (input.right() - clip.right()) * ctx.pixels_per_point() <= 1.,
                "Input exceeds clip at {scale}"
            );
            assert!((run.right() - clip.right()) * ctx.pixels_per_point() <= 1.);
            for key in ["archive", "operation"] {
                let (_, rect, clip) = a
                    .metrics
                    .guest_driver
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap();
                assert!((rect.right() - clip.right()) * ctx.pixels_per_point() <= 1.);
                assert!((rect.height() - input.height()).abs() <= 1.);
            }
        }
    }
    assert!(
        session(&a)
            .assignment
            .input
            .starts_with("modinfo /lib/modules")
    );
}
