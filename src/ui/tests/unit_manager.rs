//! Native-egui interaction proposals; actual Fedora render acceptance is separate.
use super::*;
use dlms_learning_lab::{unit_manager_terminal::Session, unit_manager_workspace as manager};
fn manager_app(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let s = pack::load(include_str!(
        "../../../tests/fixtures/unit-manager-pack-preview.json"
    ))
    .unwrap()
    .remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    a.set_unit_manager_view(i, view);
    (ctx, a, i)
}
fn manager_command(ctx: &egui::Context, a: &mut LabApp, i: usize, practice: bool, text: &str) {
    let id = egui::Id::new(("manager-command", (a.scenarios[i].id.as_str(), practice)));
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
    draw(ctx, a);
}
#[test]
fn unit_manager_intro_fill_only_practice_keyboard_commands_and_assignment_stay_separate() {
    let (ctx, mut a, i) = manager_app(teaching::View::Introduction);
    draw(&ctx, &mut a);
    assert!(a.metrics.example_action.is_some() && a.metrics.assignment_action.is_some());
    let scenario = a.scenarios[i].clone();
    let original = a.progress.attempts[&scenario.id].state.clone();
    a.set_unit_manager_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let before = a.progress.attempts[&scenario.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_learning
        .as_ref()
        .unwrap()
        .state
        .clone();
    let fill = a.metrics.insert.unwrap();
    click(&ctx, &mut a, fill);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[&scenario.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.state, before);
    assert!(!l.draft.matches(&l.state));
    assert!(!l.checked);
    let save = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, save);
    draw(&ctx, &mut a);
    for command in [
        "systemctl daemon-reload",
        "systemctl enable sample-application.service",
        "systemctl set-default sample.target",
        "systemctl start sample.target",
    ] {
        manager_command(&ctx, &mut a, i, true, command);
    }
    let l = a.progress.attempts[&scenario.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_learning
        .as_ref()
        .unwrap();
    assert!(manager::solved(
        &scenario.unit_manager_teaching.as_ref().unwrap().definition,
        &l.state,
        &l.draft
    ));
    assert_eq!(a.progress.attempts[&scenario.id].state, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[&scenario.id],
        &scenario
    ));
    a.set_unit_manager_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(
        a.progress.attempts[&scenario.id]
            .workspace
            .as_ref()
            .unwrap()
            .unit_manager_terminal
            .as_ref()
            .unwrap()
            .entries
            .is_empty()
    );
}
#[test]
fn unit_manager_keyboard_queries_keep_completion_and_editor_changes_invalidate_with_undo_isolation()
{
    let (ctx, mut a, i) = manager_app(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    for text in &s.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(&s.id).unwrap().state,
            &s.goal,
            text,
        )
        .unwrap();
    }
    let State::UnitManager { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    let draft = manager::Draft::new(simulation).unwrap();
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_draft = Some(draft);
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .checked = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    manager_command(
        &ctx,
        &mut a,
        i,
        false,
        "systemctl status work-application.service",
    );
    assert!(a.metrics.completion);
    let selected = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_draft
        .as_ref()
        .unwrap()
        .selected_unit
        .clone();
    let editor = egui::Id::new((
        "manager-editor",
        (s.id.as_str(), false),
        false,
        selected.as_str(),
    ));
    ctx.memory_mut(|m| m.request_focus(editor));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Text("# unsaved assignment\\n".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    let assignment = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_draft
        .clone();
    a.set_unit_manager_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    assert_eq!(
        a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .unit_manager_draft,
        assignment
    );
}
#[test]
fn unit_manager_background_enter_cannot_execute_during_reset_or_settings_and_challenge_has_no_walkthrough()
 {
    let (ctx, mut a, i) = manager_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    let s = a.scenarios[i].clone();
    let command_id = egui::Id::new(("manager-command", (s.id.as_str(), false)));
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_terminal = Some(Session {
        command: "systemctl start work-observer.service".into(),
        ..Default::default()
    });
    let original = a.progress.attempts[&s.id].state.clone();
    for settings in [false, true] {
        a.reset_pending = !settings;
        a.settings_open = settings;
        ctx.memory_mut(|m| m.request_focus(command_id));
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
        assert_eq!(a.progress.attempts[&s.id].state, original);
        assert!(
            a.progress.attempts[&s.id]
                .workspace
                .as_ref()
                .unwrap()
                .unit_manager_terminal
                .as_ref()
                .unwrap()
                .entries
                .is_empty()
        );
    }
    a.settings_open = false;
    a.reset_pending = false;
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(a.metrics.insert.is_none() && a.metrics.command_insert.is_none());
    assert!(a.metrics.manager_run.is_some());
}
#[test]
fn unit_manager_controls_reflow_at_readable_scales_and_keep_contextual_focus_ids() {
    for view in [teaching::View::Example, teaching::View::Assignment] {
        let (ctx, mut a, i) = manager_app(view);
        let s = a.scenarios[i].clone();
        let practice = view == teaching::View::Example;
        let id = egui::Id::new(("manager-command", (s.id.as_str(), practice)));
        for (width, height, scale) in [
            (1200., 850., 1.),
            (1920., 1080., 1.5),
            (960., 900., 2.),
            (720., 560., 1.5),
        ] {
            a.scale = scale;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            let content = ctx.content_rect();
            for r in [
                a.metrics.manager_slot,
                a.metrics.manager_file,
                a.metrics.terminal,
                a.metrics.manager_run,
                a.metrics.check,
            ]
            .into_iter()
            .flatten()
            {
                assert!(
                    r.left() >= content.left() - 1. && r.right() <= content.right() + 1.,
                    "{view:?}/{width}/{scale}: {r:?} outside {content:?}"
                );
            }
            ctx.memory_mut(|m| m.request_focus(id));
            frame(
                &ctx,
                &mut a,
                width,
                height,
                vec![egui::Event::Text("x".into())],
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            frame(
                &ctx,
                &mut a,
                width,
                height,
                vec![key(egui::Key::Tab, false)],
            );
            assert_ne!(ctx.memory(|m| m.focused()), Some(id));
        }
    }
}

#[test]
fn unit_manager_long_diagnostic_prose_wraps_to_finite_viewport_despite_wide_command_and_table() {
    use dlms_learning_lab::unit_manager_terminal::{DiagnosticKind, session::Entry};
    let (ctx, mut a, i) = manager_app(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    let message="This command is outside the supported simulation. Keep the supplied unit name, inspect Help for the accepted options, then correct the command and retry. ".repeat(5);
    assert!(message.len() <= 1024);
    let session = Session {
        command: String::new(),
        entries: vec![
            Entry {
                command: "systemctl ".repeat(120),
                stdout: "Wide table column ".repeat(120),
                stderr: String::new(),
                status: Some(0),
                diagnostic: None,
                display_escaped: false,
            },
            Entry {
                command: "systemctl unsupported-option".into(),
                stdout: String::new(),
                stderr: message,
                status: None,
                diagnostic: Some(DiagnosticKind::Unsupported),
                display_escaped: false,
            },
        ],
        ..Default::default()
    };
    assert!(session.valid());
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_terminal = Some(session);
    for (width, height, scale) in [(720., 800., 1.5), (960., 1000., 2.)] {
        a.scale = scale;
        for _ in 0..3 {
            frame(&ctx, &mut a, width, height, vec![]);
        }
        assert_eq!(a.metrics.manager_diagnostics.len(), 1);
        let (rect, limit) = a.metrics.manager_diagnostics[0];
        assert!(
            rect.width() <= limit + 1.,
            "{scale}: diagnostic {rect:?} exceeds finite{limit}"
        );
        assert!(
            rect.height() > ctx.style().text_styles[&egui::TextStyle::Body].size * 2.,
            "long explanation should wrap"
        );
        assert!(limit < ctx.content_rect().width());
    }
}
