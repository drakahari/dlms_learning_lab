//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
const ID: &str = "lpic2-mailbox-administration-recovery";
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
    let State::MailboxAdministration { simulation } =
        &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::MailboxAdministration { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .mailbox
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
        .mailbox
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
fn admin_fill_and_insert_are_inert_and_saved_queries_do_not_read_unsaved_text() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let d = binding::source(&a.scenarios[i]).unwrap().clone();
    let assignment = session(&a).assignment.clone();
    let runtime = session(&a).practice.runtime.clone();
    hit(&ctx, &mut a, "fill");
    hit(&ctx, &mut a, "insert-command");
    assert_eq!(session(&a).practice.runtime, runtime);
    assert!(session(&a).practice.output.is_empty());
    assert!(!session(&a).practice.checked);
    hit(&ctx, &mut a, "run-admin");
    assert!(
        session(&a)
            .practice
            .output
            .ends_with("/srv/practice/elin/Maildir\n")
    );
    assert_eq!(session(&a).practice.draft, d.worked_config.unwrap());
    hit(&ctx, &mut a, "save");
    edit(&ctx, &mut a, "command", "doveconf -h mail_path");
    hit(&ctx, &mut a, "run-admin");
    assert!(
        session(&a)
            .practice
            .output
            .ends_with("/srv/practice/%{user}/Maildir\n")
    );
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
    assert!(
        !a.metrics
            .mailbox
            .controls
            .iter()
            .any(|r| r.0 == "connect" || r.0 == "client")
    );
}
#[test]
fn admin_actual_controls_complete_the_task_and_clear_broken_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    for raw in &source.solution {
        let act: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        match act {
            work::Action::Edit { text } => edit(&ctx, &mut a, "editor", &text),
            work::Action::Admin { line } => {
                edit(&ctx, &mut a, "command", &line);
                hit(&ctx, &mut a, "run-admin");
            }
            work::Action::Save => hit(&ctx, &mut a, "save"),
            work::Action::Restart => hit(&ctx, &mut a, "restart"),
            work::Action::Enable => hit(&ctx, &mut a, "enable"),
            work::Action::Check => hit(&ctx, &mut a, "check"),
            _ => panic!("Unreviewed reference action"),
        }
    }
    assert!(a.metrics.completion);
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "editor", "unknown_setting = yes");
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.mailbox.result.is_some_and(|(p, n, _)| p < n));
}
#[test]
fn admin_enter_runs_once_and_unsupported_input_remains_retryable() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    edit(&ctx, &mut a, "command", "doveconf -n");
    hit(&ctx, &mut a, "run-admin");
    assert!(session(&a).assignment.output.contains("[Unsupported]"));
    assert_eq!(session(&a).assignment.input, "doveconf -n");
    edit(&ctx, &mut a, "command", "doveconf -h ssl");
    let before = session(&a)
        .assignment
        .output
        .matches("$ doveconf -h ssl")
        .count();
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
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert_eq!(
        session(&a)
            .assignment
            .output
            .matches("$ doveconf -h ssl")
            .count(),
        before + 1
    );
    assert!(session(&a).assignment.input.is_empty());
}
#[test]
fn admin_controls_reflow_without_horizontal_clipping_at_supported_text_sizes() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for (width, scale) in [(1450., 1.), (780., 1.), (1450., 1.5), (780., 2.)] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 10000., vec![]);
        }
        for key in ["command", "run-admin", "save", "restart", "enable", "check"] {
            let (_, r, clip) = a
                .metrics
                .mailbox
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap();
            assert!(
                r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                "{width}/{scale}: {key} {r:?} {clip:?}"
            );
            assert!(r.height() >= egui::TextStyle::Body.resolve(&ctx.style()).size);
        }
        assert!(!a.metrics.mailbox.focus.is_empty());
    }
}

#[test]
fn admin_challenge_keeps_full_task_and_hides_worked_controls() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    let original = session(&a).assignment.clone();
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["introduction", "example", "fill", "worked", "insert-command", "example-command"] {
        assert!(!a.metrics.mailbox.controls.iter().any(|r| r.0 == key), "Challenge exposed {key}");
    }
    assert_eq!(session(&a).assignment, original);
    for key in ["editor", "command", "run-admin", "check"] {
        assert!(a.metrics.mailbox.controls.iter().any(|r| r.0 == key));
    }
}
