//! Actual SMTP widgets; no native, physical-input or novice acceptance implied.
use super::*;
use dlms_learning_lab::{smtp_source as binding, smtp_workspace as work};
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
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic2-smtp-destination-role")
        .unwrap();
    a.launch(i);
    let State::SmtpRouting { simulation } = &mut a
        .progress
        .attempts
        .get_mut("lpic2-smtp-destination-role")
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
    let State::SmtpRouting { simulation } =
        &a.progress.attempts["lpic2-smtp-destination-role"].state
    else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    settle(ctx, a);
    let (_, rect, clip) = a
        .metrics
        .smtp
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
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
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
#[test]
fn editor_save_reload_retry_and_checks_distinguish_partial_work_and_broken_again() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.smtp.result.unwrap().1 > 0);
    let work::Action::Edit { text, .. } = &d.reference[0] else {
        panic!()
    };
    edit(&ctx, &mut a, "editor", text);
    assert!(!session(&a).assignment.checked);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "reload");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    edit(&ctx, &mut a, "command", "postqueue -f");
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert_eq!(a.metrics.smtp.result.unwrap().1, 0);
    edit(
        &ctx,
        &mut a,
        "editor",
        &d.assignment.facts.initial_files["edge"],
    );
    assert!(!a.metrics.completion);
    assert!(!session(&a).assignment.checked);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.smtp.result.unwrap().1 > 0);
}
#[test]
fn practice_insert_never_runs_and_enter_runs_once_without_assignment_credit_or_draft_changes() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    let original = session(&a).assignment.clone();
    let before = session(&a).practice.runtime.clone();
    hit(&ctx, &mut a, "insert-command");
    assert_eq!(session(&a).practice.input, "postconf -n");
    assert!(session(&a).practice.output.is_empty());
    assert_eq!(session(&a).practice.runtime, before);
    hit(&ctx, &mut a, "run");
    assert!(
        session(&a)
            .practice
            .output
            .contains("guide-edge.guide.test")
    );
    assert_eq!(session(&a).assignment, original);
    edit(&ctx, &mut a, "command", "postqueue -p");
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
    assert_eq!(
        session(&a)
            .practice
            .output
            .matches("$ postqueue -p")
            .count(),
        1
    );
    assert!(!a.metrics.completion);
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
            .smtp
            .controls
            .iter()
            .any(|r| matches!(r.0.as_str(), "insert-command" | "introduction" | "example"))
    );
}
#[test]
fn introduction_and_optional_depth_preserve_the_independent_task_and_real_workspace() {
    let (ctx, mut a, _) = opened(teaching::View::Introduction);
    let original = session(&a).assignment.clone();
    assert!(a.metrics.smtp.controls.iter().any(|r| r.0 == "worked"));
    assert!(!a.metrics.smtp.controls.iter().any(|r| r.0 == "editor"));
    hit(&ctx, &mut a, "try");
    assert!(a.metrics.smtp.controls.iter().any(|r| r.0 == "editor"));
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
}
#[test]
fn server_selector_field_and_buttons_align_at_large_text_and_escape_retains_keyboard_focus() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    hit(&ctx, &mut a, "server");
    let id = a
        .metrics
        .smtp
        .focus
        .iter()
        .find(|r| r.0 == "server")
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
    for text in ["", "postconf -n"] {
        edit(&ctx, &mut a, "command", text);
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
                .smtp
                .fields
                .iter()
                .find(|r| r.0 == "command")
                .unwrap();
            for key in ["run", "server", "save", "reload"] {
                let (_, r, clip) = a.metrics.smtp.controls.iter().find(|r| r.0 == key).unwrap();
                assert!(
                    (r.right() - clip.right()) * ctx.pixels_per_point() <= 1.,
                    "{key} clipped at {scale}"
                );
                assert!(
                    (r.height() - input.height()).abs() <= 1.,
                    "{key} height at {scale}"
                );
            }
            assert!((input.right() - ctx.content_rect().right()) * ctx.pixels_per_point() <= 1.);
        }
    }
}
