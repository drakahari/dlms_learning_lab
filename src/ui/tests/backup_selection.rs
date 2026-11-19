use super::*;
use dlms_learning_lab::file_backup_workspace::presentation::{Edit, present};
const ID: &str = "lpic2-backup-directory-selection";
fn key_event(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    present(
        a.progress.attempts.get_mut(ID).unwrap(),
        &a.scenarios[i],
        false,
        Edit {
            view: Some(view),
            ..Default::default()
        },
    )
    .unwrap();
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn view(ctx: &egui::Context, a: &mut LabApp, i: usize, v: teaching::View) {
    present(
        a.progress.attempts.get_mut(ID).unwrap(),
        &a.scenarios[i],
        false,
        Edit {
            view: Some(v),
            ..Default::default()
        },
    )
    .unwrap();
    a.lesson_scroll_reset = true;
    draw(ctx, a);
}
#[test]
fn backup_selection_three_views_keep_real_terminal_and_challenge_has_no_default_worked_commands() {
    let (ctx, mut a, i) = opened(teaching::View::Introduction);
    assert!(
        a.metrics
            .backup
            .controls
            .iter()
            .any(|r| r.0 == "try-example")
    );
    assert!(a.metrics.backup.terminal.input.is_none());
    view(&ctx, &mut a, i, teaching::View::Example);
    assert!(a.metrics.backup.terminal.input.is_some());
    assert_eq!(a.metrics.backup.inserts.len(), 1);
    assert!(
        a.metrics
            .backup
            .controls
            .iter()
            .any(|r| r.0 == "example-picker")
    );
    view(&ctx, &mut a, i, teaching::View::Assignment);
    assert!(a.metrics.backup.terminal.input.is_some());
    assert!(a.metrics.backup.inserts.is_empty());
    assert!(a.metrics.check.is_some());
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.backup.tabs.is_empty());
    assert!(a.metrics.backup.inserts.is_empty());
    assert!(a.metrics.backup.terminal.input.is_some());
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn backup_selection_actual_tab_focus_and_paste_preserve_separate_drafts_without_execution() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let id = a.metrics.backup.terminal.input.unwrap().1;
    ctx.memory_mut(|m| m.surrender_focus(m.focused().unwrap_or(id)));
    let mut reached = false;
    for _ in 0..80 {
        frame(&ctx, &mut a, 1200., 850., vec![key_event(egui::Key::Tab)]);
        if ctx.memory(|m| m.focused()) == Some(id) {
            reached = true;
            break;
        }
    }
    assert!(reached, "Tab must reach native command field");
    let text = "tar -tf /backups/unfinished.tar";
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Paste(text.into())],
    );
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[ID].workspace.as_ref().unwrap().command,
        text
    );
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert!(a.progress.attempts[ID].transcript.is_empty());
    view(&ctx, &mut a, i, teaching::View::Example);
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .command
            .is_empty()
    );
    view(&ctx, &mut a, i, teaching::View::Assignment);
    assert_eq!(
        a.progress.attempts[ID].workspace.as_ref().unwrap().command,
        text
    );
}
#[test]
fn backup_selection_native_controls_reflow_at_all_supported_text_sizes_light_dark_and_widths() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            a.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &a.theme_id);
            for width in [760., 1600.] {
                for v in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    present(
                        a.progress.attempts.get_mut(ID).unwrap(),
                        &a.scenarios[i],
                        false,
                        Edit {
                            view: Some(v),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 950., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for (key, r) in &a.metrics.backup.controls {
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{key} scale{scale} width{width} {r:?} {clip:?}"
                        );
                    }
                    if let Some((r, _)) = a.metrics.backup.terminal.input {
                        assert!(r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.);
                    }
                }
            }
        }
    }
}

#[test]
fn backup_selection_picker_exposes_all_commands_and_selection_or_insertion_never_executes() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    let picker = a
        .metrics
        .backup
        .controls
        .iter()
        .find(|r| r.0 == "example-picker")
        .unwrap()
        .1;
    click(&ctx, &mut a, picker);
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics
            .backup
            .controls
            .iter()
            .filter(|r| r.0.starts_with("example:"))
            .count(),
        10
    );
    let choice = a
        .metrics
        .backup
        .controls
        .iter()
        .find(|r| r.0 == "example:2")
        .unwrap()
        .1;
    click(&ctx, &mut a, choice);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .file_backup_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.example_command, 2);
    assert!(l.command.is_empty() && l.transcript.is_empty());
    let insert = a.metrics.backup.inserts[0].1;
    click(&ctx, &mut a, insert);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .file_backup_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.command,
        a.scenarios[i]
            .file_backup_teaching
            .as_ref()
            .unwrap()
            .example_commands[2]
            .command
    );
    assert!(l.transcript.is_empty());
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
