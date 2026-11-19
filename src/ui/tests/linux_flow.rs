//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{linux_flow_source as binding, linux_flow_workspace as work};
const ID: &str = "lpic2-nat-published-service-policy";
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
    let State::LinuxFlow { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::LinuxFlow { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .flow
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
    let id = a.metrics.flow.fields.iter().find(|r| r.0 == key).unwrap().2;
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
fn flow_insert_is_inert_enter_runs_practice_only_and_challenge_hides_guidance() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let assignment = session(&a).assignment.clone();
    hit(&ctx, &mut a, "insert-command");
    assert!(session(&a).practice.output.is_empty());
    assert!(!session(&a).practice.input.is_empty());
    assert!(!session(&a).practice.checked);
    edit(&ctx, &mut a, "command", "iptables -t nat -S");
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
    assert!(session(&a).practice.output.contains("PREROUTING"));
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["example", "introduction", "insert-command", "worked"] {
        assert!(!a.metrics.flow.controls.iter().any(|r| r.0 == key));
    }
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn flow_current_success_requires_check_readonly_trial_preserves_it_and_edit_invalidates_it() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.flow.controls.iter().any(|r| r.0 == "failed-case"));
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let action: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        match action {
            work::Action::Command { line } => {
                edit(&ctx, &mut a, "command", &line);
                hit(&ctx, &mut a, "run");
            }
            work::Action::Check => hit(&ctx, &mut a, "check"),
            _ => panic!("Unexpected recipe action"),
        }
    }
    assert!(a.metrics.completion);
    hit(&ctx, &mut a, "run-traffic");
    assert!(a.metrics.completion);
    assert!(session(&a).assignment.last_trial.is_some());
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "command", "iptables -P FORWARD ACCEPT");
    assert!(!a.metrics.completion);
    assert!(session(&a).assignment.last_trial.is_none());
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "run-traffic");
    assert!(
        a.metrics
            .flow
            .controls
            .iter()
            .any(|r| r.0 == "trial-result")
    );
    assert!(!a.metrics.completion);
}
#[test]
fn flow_fields_and_actions_fit_supported_scales_themes_and_widths() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for width in [1450., 760.] {
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                ctx.set_zoom_factor(scale);
                a.scale = scale;
                let selected = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &selected);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 9000., vec![]);
                }
                for key in ["command", "run", "case", "run-traffic", "check"] {
                    let (_, r, clip) = a.metrics.flow.controls.iter().find(|r| r.0 == key).unwrap();
                    assert!(
                        r.left() >= clip.left() - 1.,
                        "{width}/{scale}/{light}/{key}"
                    );
                    assert!(
                        r.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale}/{light}/{key}: {r:?}"
                    );
                    assert!(
                        r.height() >= 20.,
                        "Readable control height at {width}/{scale}"
                    );
                }
                assert_eq!(ctx.zoom_factor(), scale);
            }
        }
    }
}
#[test]
fn flow_tab_navigation_reaches_command_actions_without_executing() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "command");
    let before = session(&a).assignment.clone();
    for (key, shift) in [("run", false), ("case", false), ("run", true)] {
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    shift,
                    ..Default::default()
                },
            }],
        );
        settle(&ctx, &mut a);
        let expected = a.metrics.flow.focus.iter().find(|r| r.0 == key).unwrap().1;
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(expected),
            "Native focus should reach {key}"
        );
        assert_eq!(session(&a).assignment, before);
    }
}
