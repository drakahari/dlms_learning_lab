//! Real widgets and synthetic progress only; native/device acceptance is separate.
use super::*;
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
const CONFIG: &str = "lpic2-dovecot-secure-access";
const IMAP: &str = "lpic2-imap-mailbox-recovery";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 7000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::MailboxAccess { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let State::MailboxAccess { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, rect, clip) = a
        .metrics
        .mailbox
        .controls
        .iter()
        .find(|c| c.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            rect,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key} {rect:?} {clip:?}"
    );
    let pos = rect.center();
    paint(
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
    paint(
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
    let field = a
        .metrics
        .mailbox
        .fields
        .iter()
        .find(|(k, _, _)| k == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(field));
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
fn mailbox_guided_fill_and_insert_are_inert_and_cannot_change_the_assignment() {
    let (ctx, mut a, i) = opened(CONFIG, teaching::View::Example);
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    let original = session(&a).assignment.clone();
    let saved = session(&a).practice.runtime.clone();
    hit(&ctx, &mut a, "fill");
    assert_eq!(session(&a).practice.draft, d.worked_config.clone().unwrap());
    assert_eq!(session(&a).practice.runtime, saved);
    assert!(!session(&a).practice.checked);
    hit(&ctx, &mut a, "insert-command");
    assert!(session(&a).practice.input.starts_with("a1 LOGIN mira"));
    assert!(session(&a).practice.connection.is_none());
    assert_eq!(session(&a).assignment, original);
    for key in ["save", "restart", "enable", "check"] {
        hit(&ctx, &mut a, key);
    }
    let stats = a.metrics.mailbox.result.unwrap();
    assert_eq!(stats.0, stats.1);
    assert!(!a.metrics.completion);
    assert_eq!(session(&a).assignment, original);
}
#[test]
fn mailbox_config_buttons_grade_real_edits_and_clear_stale_success() {
    let (ctx, mut a, i) = opened(CONFIG, teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let text = source
        .solution
        .iter()
        .filter_map(|s| s.strip_prefix(binding::PREFIX))
        .filter_map(|s| serde_json::from_str::<work::Action>(s).ok())
        .find_map(|a| {
            if let work::Action::Edit { text } = a {
                Some(text)
            } else {
                None
            }
        })
        .unwrap();
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .mailbox
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
    edit(&ctx, &mut a, "editor", &text);
    for key in ["save", "restart", "enable", "check"] {
        hit(&ctx, &mut a, key);
    }
    assert!(a.metrics.completion);
    assert!(progress::current_solved(
        &a.progress.attempts[CONFIG],
        &source
    ));
    edit(&ctx, &mut a, "editor", "bad_setting = yes");
    assert!(!session(&a).assignment.checked);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.mailbox.result.unwrap().0 < a.metrics.mailbox.result.unwrap().1);
    assert!(!progress::current_solved(
        &a.progress.attempts[CONFIG],
        &source
    ));
}
#[test]
fn mailbox_protocol_workspace_has_no_config_editor_and_enter_sends_one_parsed_command() {
    let (ctx, mut a, _) = opened(IMAP, teaching::View::Assignment);
    for key in [
        "editor",
        "save",
        "restart",
        "enable",
        "fill",
        "insert-command",
    ] {
        assert!(
            !a.metrics.mailbox.controls.iter().any(|r| r.0 == key),
            "Unexpected {key}"
        );
    }
    hit(&ctx, &mut a, "connect");
    edit(&ctx, &mut a, "command", "a1 CAPABILITY");
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
        session(&a).assignment.connection.as_ref().unwrap().commands,
        1
    );
    assert!(session(&a).assignment.output.contains("LOGINDISABLED"));
    assert!(session(&a).assignment.input.is_empty());
    assert!(!a.metrics.completion);
}
#[test]
fn mailbox_challenge_preserves_the_assignment_and_hides_worked_controls() {
    let (ctx, mut a, i) = opened(CONFIG, teaching::View::Introduction);
    let before = session(&a).assignment.clone();
    a.progress.attempts.get_mut(CONFIG).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in [
        "introduction",
        "example",
        "fill",
        "worked",
        "insert-command",
    ] {
        assert!(
            !a.metrics.mailbox.controls.iter().any(|r| r.0 == key),
            "Challenge exposed {key}"
        );
    }
    assert_eq!(session(&a).assignment, before);
    assert_eq!(a.scenarios[i].id, CONFIG);
    assert!(a.metrics.mailbox.controls.iter().any(|r| r.0 == "editor"));
}
#[test]
fn mailbox_empty_output_stays_compact_until_real_results_need_more_room() {
    let (ctx, mut a, _) = opened(IMAP, teaching::View::Assignment);
    let rect = |a: &LabApp| {
        a.metrics
            .mailbox
            .controls
            .iter()
            .find(|c| c.0 == "output-viewport")
            .unwrap()
            .1
    };
    let empty = rect(&a);
    hit(&ctx, &mut a, "connect");
    assert!((rect(&a).height() - empty.height()).abs() < 1.);
    edit(&ctx, &mut a, "command", "a1 CAPABILITY");
    hit(&ctx, &mut a, "send");
    assert!(rect(&a).height() > empty.height());
    assert!(session(&a).assignment.output.contains("CAPABILITY"));
}
#[test]
fn mailbox_fields_reflow_at_large_text_without_clipping_horizontal_click_targets() {
    for id in [CONFIG, IMAP] {
        let (ctx, mut a, _) = opened(id, teaching::View::Assignment);
        for (width, scale) in [(1450., 1.), (780., 1.), (1450., 1.5), (780., 2.)] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 9000., vec![]);
            }
            let (_, r, clip) = a
                .metrics
                .mailbox
                .controls
                .iter()
                .find(|c| c.0 == "command")
                .unwrap();
            assert!(
                r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                "{id} {width} {scale}: {r:?} {clip:?}"
            );
            assert!(r.height() >= egui::TextStyle::Body.resolve(&ctx.style()).size);
            assert!(!a.metrics.mailbox.focus.is_empty());
        }
    }
}
