//! Native controls exercised headlessly; actual Fedora images remain separate evidence.
use super::*;
use dlms_learning_lab::{poe_source as binding, poe_workspace as work};
const ID: &str = "ccna-poe-compatible-ports";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 16000., events)
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
    let State::Poe { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::Poe { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .poe
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
    let pos = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, device: usize, port: Option<usize>) {
    hit(ctx, a, &format!("port-choice:{device}"));
    hit(
        ctx,
        a,
        &format!(
            "port-option:{device}:{}",
            port.map(|p| p.to_string()).unwrap_or_else(|| "none".into())
        ),
    );
}
#[test]
fn introduction_is_skippable_and_does_not_fill_an_assignment() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    let assignment = session(&a).assignment.clone();
    for key in [
        "introduction-content",
        "worked",
        "try-example",
        "assignment",
    ] {
        assert!(a.metrics.poe.controls.iter().any(|r| r.0 == key));
    }
    assert!(!a.metrics.poe.controls.iter().any(|r| r.0 == "fill-example"));
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .poe
            .controls
            .iter()
            .any(|r| r.0 == "assignment-content")
    );
}
#[test]
fn explicit_worked_fill_and_practice_check_preserve_the_assignment_draft() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let assignment = session(&a).assignment.clone();
    assert!(!a.metrics.poe.controls.iter().any(|r| r.0 == "fill-example"));
    hit(&ctx, &mut a, "worked-answer");
    hit(&ctx, &mut a, "fill-example");
    let d = binding::source(&a.scenarios[i]).unwrap();
    assert_eq!(session(&a).example.plan, d.example.reference);
    assert!(!session(&a).example.checked);
    hit(&ctx, &mut a, "check");
    assert!(
        session(&a)
            .example
            .solved(&binding::source(&a.scenarios[i]).unwrap().example)
    );
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "assignment");
    hit(&ctx, &mut a, "example");
    assert_eq!(session(&a).assignment, assignment);
    assert!(session(&a).example.checked);
}
#[test]
fn actual_port_and_power_controls_repair_partial_work_and_invalidate_success() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(
        a.metrics
            .poe
            .result
            .is_some_and(|(n, total, _)| n > 0 && n < total)
    );
    assert!(!a.metrics.completion);
    choose(&ctx, &mut a, 0, None);
    choose(&ctx, &mut a, 2, Some(0));
    choose(&ctx, &mut a, 0, Some(1));
    assert!(!session(&a).assignment.checked);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    hit(&ctx, &mut a, "enabled:1");
    assert!(!session(&a).assignment.checked && !a.metrics.completion);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.poe.controls.iter().any(|r| r.0 == "failed-case"));
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "enabled:1");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}
#[test]
fn source_hardware_is_not_an_invented_toggle_and_keyboard_can_edit_power() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    assert!(!a.metrics.poe.controls.iter().any(|r| r.0 == "enabled:2"));
    let id = a
        .metrics
        .poe
        .focus
        .iter()
        .find(|(k, _)| k == "enabled:1")
        .unwrap()
        .1;
    ctx.memory_mut(|m| m.request_focus(id));
    for pressed in [true, false] {
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
    }
    settle(&ctx, &mut a);
    assert!(!session(&a).assignment.plan.enabled[1]);
    assert!(!a.metrics.completion);
}
#[test]
fn power_controls_reflow_at_supported_scales_and_light_dark_themes() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    let saved = session(&a).assignment.clone();
    for (width, scale) in [
        (1450., 1.),
        (1450., 1.5),
        (760., 1.),
        (760., 1.5),
        (760., 2.),
    ] {
        ctx.set_zoom_factor(scale);
        a.scale = scale;
        for light in [false, true] {
            a.theme_id = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap()
                .id
                .clone();
            theme::apply(&ctx, &a.theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 20000., vec![]);
            }
            for key in [
                "port-choice:0",
                "port-choice:1",
                "port-choice:2",
                "enabled:0",
                "enabled:1",
                "enabled:3",
                "check",
            ] {
                let (_, r, clip) = a
                    .metrics
                    .poe
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap_or_else(|| panic!("Missing {key}"));
                assert!(
                    smoke::smoke_control_visible(
                        *r,
                        clip.intersect(ctx.content_rect()),
                        ctx.pixels_per_point()
                    ),
                    "Clipped {key} at{width}/{scale}: {r:?} {clip:?}"
                );
            }
        }
    }
    assert_eq!(session(&a).assignment, saved);
}
#[test]
fn challenge_retains_the_full_independent_assignment_without_worked_controls() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(
        a.metrics
            .poe
            .controls
            .iter()
            .any(|r| r.0 == "assignment-content")
    );
    for key in [
        "introduction",
        "example",
        "worked",
        "worked-answer",
        "fill-example",
    ] {
        assert!(
            !a.metrics.poe.controls.iter().any(|r| r.0 == key),
            "Challenge exposed {key}"
        );
    }
    assert!(!a.metrics.completion);
}

#[test]
fn worked_help_stays_opt_in_when_moving_to_the_next_power_lesson() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    hit(&ctx, &mut a, "worked-answer");
    assert!(a.metrics.poe.controls.iter().any(|r| r.0 == "fill-example"));
    let next = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-poe-budget-recovery")
        .unwrap();
    a.launch(next);
    settle(&ctx, &mut a);
    hit(&ctx, &mut a, "example");
    assert!(
        !a.metrics.poe.controls.iter().any(|r| r.0 == "fill-example"),
        "A prior lesson must not reveal this lesson's worked controls"
    );
    hit(&ctx, &mut a, "worked-answer");
    assert!(a.metrics.poe.controls.iter().any(|r| r.0 == "fill-example"));
}
