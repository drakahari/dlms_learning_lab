//! Real egui events over the unregistered four-task authoring fixture.
use super::*;
#[path = "../../../tests/support/file_backup_authored.rs"]
mod fixture;
use dlms_learning_lab::{file_backup_workspace as work, script_teaching::View};
fn opened(n: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (171, 44));
    let authored = fixture::catalog();
    assert_eq!(
        serde_json::to_value(&a.scenarios[..164]).unwrap(),
        serde_json::to_value(&authored[..164]).unwrap(),
        "Complete prior164 prefix"
    );
    // The historical authoring fixture appends its four tasks after all other
    // registered content. Keep its complete prior164 and compare both appended
    // canonical packs in their actual normal-catalog order.
    let numeric: dlms_learning_lab::pack::Pack = serde_json::from_str(include_str!(
        "../../../scenarios/shell-numeric-validation-v1.json"
    ))
    .unwrap();
    let backup: dlms_learning_lab::pack::Pack = serde_json::from_str(fixture::DATA).unwrap();
    assert_eq!(numeric.scenarios.len(), 3);
    assert_eq!(backup.scenarios.len(), 4);
    let expected = authored[..164]
        .iter()
        .chain(&backup.scenarios)
        .chain(&numeric.scenarios)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&a.scenarios).unwrap(),
        serde_json::to_value(expected).unwrap(),
        "Entire actual normal catalog: all prior164, canonical backup4 and canonical numeric3"
    );
    let canonical: dlms_learning_lab::pack::Pack = serde_json::from_str(fixture::DATA).unwrap();
    assert_eq!(
        serde_json::to_value(&a.scenarios[164..168]).unwrap(),
        serde_json::to_value(canonical.scenarios).unwrap(),
        "Exactly four canonical registered tasks"
    );
    let at = a
        .scenarios
        .iter()
        .position(|s| s.id == fixture::review()["labs"][n]["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        serde_json::to_value(&a.scenarios[at]).unwrap(),
        serde_json::to_value(fixture::lab(&a.scenarios[at].id)).unwrap()
    );
    a.launch(at);
    for _ in 0..8 {
        frame(&ctx, &mut a, 1400., 1000., vec![]);
    }
    (ctx, a, at)
}
fn draw_frame(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1400., 1000., events);
}
fn backup_click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    draw_frame(
        ctx,
        a,
        vec![
            egui::Event::PointerMoved(r.center()),
            egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    draw_frame(
        ctx,
        a,
        vec![
            egui::Event::PointerMoved(r.center()),
            egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
    );
}
fn reveal(
    ctx: &egui::Context,
    a: &mut LabApp,
    mut target: impl FnMut(&LabApp) -> egui::Rect,
) -> egui::Rect {
    let mut stable = None;
    for _ in 0..32 {
        let r = target(a);
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if stable == Some((r, clip)) {
                return r;
            }
            stable = Some((r, clip));
            draw_frame(ctx, a, vec![egui::Event::PointerMoved(r.center())]);
            continue;
        }
        stable = None;
        let delta = if r.center().y > clip.center().y {
            -200.
        } else {
            200.
        };
        draw_frame(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Backup control did not become visible");
}
fn tab(ctx: &egui::Context, a: &mut LabApp, v: View) {
    let r = reveal(ctx, a, |a| {
        a.metrics
            .backup
            .tabs
            .iter()
            .find(|(view, _)| *view == v)
            .unwrap()
            .1
    });
    backup_click(ctx, a, r);
    for _ in 0..4 {
        draw_frame(ctx, a, vec![]);
    }
}
fn action(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, |a| {
        a.metrics
            .backup
            .controls
            .iter()
            .find(|(name, _)| name == key)
            .expect("Actual backup control")
            .1
    });
    backup_click(ctx, a, r);
}
fn enter(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = reveal(ctx, a, |a| a.metrics.backup.terminal.input.unwrap().0);
    backup_click(ctx, a, r);
    draw_frame(
        ctx,
        a,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Paste(text.into()),
        ],
    );
    draw_frame(ctx, a, vec![key(egui::Key::Enter, false)]);
}
#[test]
fn backup_actual_terminal_taught_commands_complete_all_four_and_wrong_command_keeps_retry_draft() {
    for n in 0..4 {
        let (ctx, mut a, i) = opened(n);
        tab(&ctx, &mut a, View::Assignment);
        let id = a.scenarios[i].id.clone();
        let before = serde_json::to_value(&a.progress.attempts[&id].state).unwrap();
        enter(&ctx, &mut a, "rm -rf /");
        assert_eq!(
            serde_json::to_value(&a.progress.attempts[&id].state).unwrap(),
            before
        );
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
        for command in a.scenarios[i].solution.clone() {
            enter(&ctx, &mut a, &command);
        }
        action(&ctx, &mut a, "check");
        for _ in 0..96 {
            draw_frame(&ctx, &mut a, vec![]);
            if a.metrics.backup.complete {
                break;
            }
        }
        assert!(a.metrics.backup.complete);
        assert!(progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
        enter(&ctx, &mut a, "rm /backups/no-such-archive.tar");
        assert!(
            !a.metrics.backup.complete,
            "Editing clears displayed checked result even when a refused command retains valid files"
        );
    }
}
#[test]
fn backup_insert_is_unexecuted_and_example_enter_does_not_change_assignment_or_credit() {
    let (ctx, mut a, i) = opened(0);
    let id = a.scenarios[i].id.clone();
    tab(&ctx, &mut a, View::Example);
    let before = serde_json::to_value(&a.progress.attempts[&id].state).unwrap();
    let r = reveal(&ctx, &mut a, |a| a.metrics.backup.inserts[0].1);
    backup_click(&ctx, &mut a, r);
    let w = a.progress.attempts[&id].workspace.as_ref().unwrap();
    let l = w.file_backup_learning.as_ref().unwrap();
    assert!(!l.command.is_empty());
    assert!(l.transcript.is_empty());
    draw_frame(&ctx, &mut a, vec![key(egui::Key::Enter, false)]);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&id].state).unwrap(),
        before
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    tab(&ctx, &mut a, View::Assignment);
    assert!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .command
            .is_empty()
    );
}
#[test]
fn backup_rejected_native_selected_paste_preserves_whole_attempt_and_does_not_run() {
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    let id = a.scenarios[i].id.clone();
    enter(&ctx, &mut a, "pwd");
    let r = reveal(&ctx, &mut a, |a| a.metrics.backup.terminal.input.unwrap().0);
    backup_click(&ctx, &mut a, r);
    let before = serde_json::to_value(&a.progress).unwrap();
    draw_frame(
        &ctx,
        &mut a,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Paste("ü".repeat(4097)),
            key(egui::Key::Enter, false),
        ],
    );
    assert!(a.metrics.backup.terminal.rejected);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert!(!a.metrics.backup.complete);
    assert_eq!(
        a.progress.attempts[&id].workspace.as_ref().unwrap().command,
        "pwd"
    );
}
// Additional event-protocol gates. Controlled scale is geometry setup, not an
// acceptance claim for the global picker. No model/grade/progress installation.
fn qa_wire(a: &LabApp) -> serde_json::Value {
    serde_json::to_value((&a.progress, &a.progress.migration_receipts)).unwrap()
}
fn qa_ctrl(key: egui::Key, shift: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            shift,
            ..Default::default()
        },
    }
}
fn qa_pointer(r: egui::Rect, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(r.center()),
        egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        },
    ]
}
fn qa_command(a: &LabApp, practice: bool) -> &str {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let w = a.progress.attempts[id].workspace.as_ref().unwrap();
    if practice {
        &w.file_backup_learning.as_ref().unwrap().command
    } else {
        &w.command
    }
}
fn qa_focus(ctx: &egui::Context, a: &mut LabApp) -> egui::Id {
    let r = reveal(ctx, a, |a| a.metrics.backup.terminal.input.unwrap().0);
    backup_click(ctx, a, r);
    let (rect, id) = a.metrics.backup.terminal.input.unwrap();
    let clip = a
        .metrics
        .backup
        .terminal
        .input_clip
        .unwrap()
        .intersect(a.metrics.main_viewport.unwrap())
        .intersect(ctx.content_rect());
    assert!(
        smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()),
        "The actual focused field must fit its measured clip"
    );
    assert!(
        ctx.memory(|m| m.has_focus(id)),
        "Actual command widget receives focus"
    );
    id
}
fn qa_type(ctx: &egui::Context, a: &mut LabApp, practice: bool, text: &str) {
    qa_focus(ctx, a);
    draw_frame(
        ctx,
        a,
        vec![
            qa_ctrl(egui::Key::A, false),
            egui::Event::Paste(text.into()),
        ],
    );
    assert!(
        a.metrics.backup.terminal.received
            && !a.metrics.backup.terminal.rejected
            && !a.metrics.backup.terminal.dropped
    );
    assert_eq!(
        qa_command(a, practice),
        text,
        "Actual accepted draft, not execution"
    );
}
fn qa_settle_undo(ctx: &egui::Context, a: &mut LabApp) {
    let before = ctx.input(|i| i.time);
    for _ in 0..100 {
        draw_frame(ctx, a, vec![]);
        if ctx.input(|i| i.time) - before > 1.1 {
            return;
        }
    }
    panic!("Natural egui frames did not reach its documented stable Undo interval");
}
fn qa_native(ctx: &egui::Context, id: egui::Id) -> (Option<egui::text::CCursorRange>, String) {
    let state = egui::text_edit::TextEditState::load(ctx, id).expect("Actual native edit state");
    (state.cursor.char_range(), format!("{:?}", state.undoer()))
}
fn qa_undo_to(ctx: &egui::Context, a: &mut LabApp, practice: bool, expected: &str, redo: bool) {
    // Pointer/selection positions can form a native cursor-only Undo point. Keep
    // a strict small bound and prove actual text history in both directions.
    for _ in 0..3 {
        draw_frame(ctx, a, vec![qa_ctrl(egui::Key::Z, redo)]);
        if qa_command(a, practice) == expected {
            return;
        }
    }
    assert_eq!(
        qa_command(a, practice),
        expected,
        "Actual native Undo/Redo text history"
    );
}
fn qa_frontier(a: &mut LabApp, i: usize) {
    let s = &a.scenarios[i];
    let attempt = a.progress.attempts.get_mut(&s.id).unwrap();
    // Deliberate valid metadata-only fixture: no State, answer, checked flag,
    // simulated file, record or result is installed by the test.
    attempt.transcript = vec![String::new(); 120];
    let remaining = work::MAX_ATTEMPT - work::size(attempt, work::MAX_ATTEMPT).unwrap();
    for (n, text) in attempt.transcript.iter_mut().enumerate() {
        *text = "x".repeat(remaining / 120 + usize::from(n < remaining % 120));
    }
    assert_eq!(
        serde_json::to_vec(attempt).unwrap().len(),
        work::MAX_ATTEMPT
    );
    work::context(attempt, s).unwrap();
}
fn qa_header(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    reveal(ctx, a, |a| {
        a.metrics
            .workspace_header_controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap()
            .1
    })
}
#[test]
fn backup_stop_stays_cancelled_through_idle_library_and_controlled_zoom_without_work_loss() {
    let (ctx, mut a, i) = opened(3);
    tab(&ctx, &mut a, View::Assignment);
    qa_type(&ctx, &mut a, false, "pwd");
    action(&ctx, &mut a, "check");
    draw_frame(&ctx, &mut a, vec![]);
    assert!(
        a.backup_presentation.observed().3,
        "Real Check starts a bounded live job"
    );
    action(&ctx, &mut a, "stop");
    let stopped = a.backup_presentation.observed();
    assert!(!stopped.3);
    assert!(
        a.backup_presentation
            .last_notice()
            .is_some_and(|n| n.contains("cancelled"))
    );
    let before = qa_wire(&a);
    assert!(!a.metrics.backup.complete);
    for _ in 0..10 {
        draw_frame(&ctx, &mut a, vec![]);
    }
    let back = qa_header(&ctx, &mut a, "back");
    backup_click(&ctx, &mut a, back);
    assert!(
        a.selected.is_none(),
        "Actual Back action returns to the library"
    );
    for (scale, width) in [(1., 1400.), (1.5, 946.), (2., 946.)] {
        a.scale = scale; // Controlled presentation geometry, not picker acceptance.
        for _ in 0..10 {
            frame(&ctx, &mut a, width, 1000., vec![]);
        }
        let now = a.backup_presentation.observed();
        assert_eq!(
            (now.0, now.1, now.2, now.3),
            (stopped.0, stopped.1, stopped.2, false),
            "Stop must not silently reschedule during idle/library/zoom"
        );
        assert_eq!(qa_wire(&a), before);
    }
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn backup_rejected_selected_paste_preserves_cursor_and_real_native_undo_redo_history() {
    let (ctx, mut a, _) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    qa_type(&ctx, &mut a, false, "pwd");
    qa_settle_undo(&ctx, &mut a);
    qa_type(&ctx, &mut a, false, "ls");
    qa_settle_undo(&ctx, &mut a);
    let id = qa_focus(&ctx, &mut a);
    draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::A, false)]);
    let native = qa_native(&ctx, id);
    let before = qa_wire(&a);
    draw_frame(
        &ctx,
        &mut a,
        vec![
            egui::Event::Paste("ü".repeat(4097)),
            key(egui::Key::Enter, false),
        ],
    );
    assert!(a.metrics.backup.terminal.rejected);
    assert_eq!(qa_wire(&a), before);
    assert_eq!(qa_native(&ctx, id), native);
    qa_focus(&ctx, &mut a);
    qa_undo_to(&ctx, &mut a, false, "pwd", false);
    qa_undo_to(&ctx, &mut a, false, "ls", true);
    assert!(
        a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
            .transcript
            .is_empty(),
        "Undo/Redo never executes a command"
    );
}
#[test]
fn backup_whole_attempt_capacity_refusal_vetoes_received_paste_run_and_retains_native_history() {
    for full in [false, true] {
        let (ctx, mut a, i) = opened(0);
        tab(&ctx, &mut a, View::Assignment);
        qa_type(&ctx, &mut a, false, "ls");
        qa_settle_undo(&ctx, &mut a);
        qa_type(&ctx, &mut a, false, "pwd");
        qa_settle_undo(&ctx, &mut a);
        if full {
            qa_frontier(&mut a, i);
            for _ in 0..16 {
                draw_frame(&ctx, &mut a, vec![]);
            }
        }
        let id = qa_focus(&ctx, &mut a);
        draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::A, false)]);
        let before = qa_wire(&a);
        let native = qa_native(&ctx, id);
        let state = a.progress.attempts[&a.scenarios[i].id].state.clone();
        // This is valid one-line text, genuinely delivered to the real editor.
        // At the frontier only whole-Attempt admission rejects its extra byte.
        draw_frame(
            &ctx,
            &mut a,
            vec![
                egui::Event::Paste("pwd ".into()),
                key(egui::Key::Enter, false),
            ],
        );
        assert!(
            a.metrics.backup.terminal.received
                && !a.metrics.backup.terminal.rejected
                && !a.metrics.backup.terminal.dropped
        );
        assert_eq!(a.progress.attempts[&a.scenarios[i].id].state, state);
        if full {
            assert_eq!(
                qa_wire(&a),
                before,
                "Rejected save+Run must retain every field/history/receipt"
            );
            assert_eq!(qa_native(&ctx, id), native);
            assert!(
                a.backup_presentation
                    .last_notice()
                    .is_some_and(|n| n.contains("saved-work allowance"))
            );
            qa_focus(&ctx, &mut a);
            qa_undo_to(&ctx, &mut a, false, "ls", false);
            qa_undo_to(&ctx, &mut a, false, "pwd", true);
            assert_eq!(
                qa_wire(&a),
                before,
                "Undo then Redo returns exactly to the admitted whole caller"
            );
            // A smaller real retry is accepted and executes in the simulator.
            qa_type(&ctx, &mut a, false, "ls");
            draw_frame(&ctx, &mut a, vec![key(egui::Key::Enter, false)]);
            assert!(
                a.progress.attempts[&a.scenarios[i].id]
                    .transcript
                    .last()
                    .unwrap()
                    .starts_with("simulated root $ ls\n")
            );
        } else {
            assert_eq!(qa_command(&a, false), "pwd ");
            assert!(
                a.progress.attempts[&a.scenarios[i].id]
                    .transcript
                    .last()
                    .unwrap()
                    .starts_with("simulated root $ pwd \n"),
                "The same focused packet genuinely Runs when capacity permits"
            );
        }
    }
}
#[test]
fn backup_dropped_same_frame_paste_run_is_vetoed_but_the_later_real_run_still_works() {
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    qa_type(&ctx, &mut a, false, "pwd");
    let run = reveal(&ctx, &mut a, |a| a.metrics.backup.terminal.run.unwrap());
    draw_frame(&ctx, &mut a, qa_pointer(run, true));
    let field = a.metrics.backup.terminal.input.unwrap().1;
    assert!(
        ctx.memory(|m| m.has_focus(field)),
        "Entry focus must be observed, not forced by the test"
    );
    let before = qa_wire(&a);
    let mut release = vec![
        qa_ctrl(egui::Key::A, false),
        egui::Event::Paste("pwd ".into()),
    ];
    release.extend(qa_pointer(run, false));
    draw_frame(&ctx, &mut a, release);
    assert!(
        a.metrics.backup.terminal.dropped && !a.metrics.backup.terminal.received,
        "Pinned egui's actual dropped Paste is required for this negative vector"
    );
    assert_eq!(qa_wire(&a), before);
    // The terminal draws its dropped-delivery diagnostic directly. The readonly
    // presentation notice accessor does not own that terminal-local message.
    let run = reveal(&ctx, &mut a, |a| a.metrics.backup.terminal.run.unwrap());
    backup_click(&ctx, &mut a, run);
    assert!(
        a.progress.attempts[&a.scenarios[i].id]
            .transcript
            .last()
            .unwrap()
            .starts_with("simulated root $ pwd\n"),
        "An ordinary later Run is not blocked by the dropped-text guard"
    );
}
#[test]
fn backup_pending_text_defers_header_and_reset_confirmation_blocks_terminal_dispatch() {
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    qa_type(&ctx, &mut a, false, "pwd");
    let back = qa_header(&ctx, &mut a, "back");
    draw_frame(&ctx, &mut a, qa_pointer(back, true));
    let old = qa_wire(&a);
    let mut release = vec![
        qa_ctrl(egui::Key::A, false),
        egui::Event::Paste("ls".into()),
    ];
    release.extend(qa_pointer(back, false));
    draw_frame(&ctx, &mut a, release);
    assert_eq!(
        a.selected,
        Some(i),
        "Header navigation is deferred while its command field owns pending text"
    );
    let mut normalized = qa_wire(&a);
    normalized[0]["attempts"][&a.scenarios[i].id]["workspace"]["command"] =
        old[0]["attempts"][&a.scenarios[i].id]["workspace"]["command"].clone();
    assert_eq!(
        normalized, old,
        "Only actually delivered text may change; navigation cannot discard or execute work"
    );
    assert!(a.metrics.backup.terminal.received || a.metrics.backup.terminal.dropped);
    let reset = qa_header(&ctx, &mut a, "reset");
    backup_click(&ctx, &mut a, reset);
    assert!(a.reset_pending);
    let before = qa_wire(&a);
    let run = reveal(&ctx, &mut a, |a| a.metrics.backup.terminal.run.unwrap());
    draw_frame(&ctx, &mut a, qa_pointer(run, true));
    let mut release = vec![
        egui::Event::Paste("rm /backups/project-full.tar".into()),
        key(egui::Key::Enter, false),
    ];
    release.extend(qa_pointer(run, false));
    draw_frame(&ctx, &mut a, release);
    assert!(a.reset_pending);
    assert_eq!(
        qa_wire(&a),
        before,
        "A reset confirmation makes the underlying editor/Run inert"
    );
    let cancel = reveal(&ctx, &mut a, |a| a.metrics.reset_cancel.unwrap());
    backup_click(&ctx, &mut a, cancel);
    assert!(!a.reset_pending);
    assert_eq!(qa_wire(&a), before);
}
#[test]
fn backup_same_value_insert_begins_a_new_native_generation_without_executing_or_losing_assignment()
{
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Example);
    let assignment = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    qa_type(&ctx, &mut a, true, "pwd");
    qa_settle_undo(&ctx, &mut a);
    let insert = reveal(&ctx, &mut a, |a| a.metrics.backup.inserts[0].1);
    backup_click(&ctx, &mut a, insert);
    for _ in 0..3 {
        draw_frame(&ctx, &mut a, vec![]);
    }
    let worked = qa_command(&a, true).to_owned();
    assert!(!worked.is_empty());
    qa_type(&ctx, &mut a, true, "ls");
    qa_settle_undo(&ctx, &mut a);
    qa_undo_to(&ctx, &mut a, true, &worked, false);
    let before_generation = a.backup_presentation.observed().4;
    let insert = reveal(&ctx, &mut a, |a| a.metrics.backup.inserts[0].1);
    backup_click(&ctx, &mut a, insert);
    for _ in 0..3 {
        draw_frame(&ctx, &mut a, vec![]);
    }
    let after = a.backup_presentation.observed();
    assert_ne!(after.4, before_generation);
    assert_eq!(after.4, after.5);
    assert_eq!(qa_command(&a, true), worked);
    qa_focus(&ctx, &mut a);
    let before = qa_wire(&a);
    draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::Z, false)]);
    draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::Z, true)]);
    assert_eq!(
        qa_wire(&a),
        before,
        "A same-value external Insert cannot resurrect previous draft text"
    );
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        assignment
    );
    assert!(
        a.progress.attempts[&a.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .transcript
            .is_empty()
    );
}
#[test]
fn backup_insert_then_observed_focus_enter_really_runs_only_the_separate_example() {
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Example);
    let before = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    let insert = reveal(&ctx, &mut a, |a| a.metrics.backup.inserts[0].1);
    backup_click(&ctx, &mut a, insert);
    for _ in 0..3 {
        draw_frame(&ctx, &mut a, vec![]);
    }
    let id = a.metrics.backup.terminal.input.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    let command = qa_command(&a, true).to_owned();
    draw_frame(&ctx, &mut a, vec![key(egui::Key::Enter, false)]);
    let l = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .file_backup_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.transcript.len(),
        1,
        "Actual Enter produced one simulated example command, not a no-op test"
    );
    assert!(l.transcript[0].starts_with(&format!("simulated root $ {command}\n")));
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        before
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn backup_ordered_edit_enter_dropped_paste_restores_whole_caller_shadow_cursor_and_undo() {
    for prefix in ["cut", "undo", "selection"] {
        let (ctx, mut a, i) = opened(0);
        tab(&ctx, &mut a, View::Assignment);
        qa_type(&ctx, &mut a, false, "pwd");
        qa_settle_undo(&ctx, &mut a);
        qa_type(&ctx, &mut a, false, "ls");
        qa_settle_undo(&ctx, &mut a);
        let field = qa_focus(&ctx, &mut a);
        // A repeated physical click can select the whole short word. Establish
        // an actual collapsed cursor with End before testing Select-all rollback.
        draw_frame(&ctx, &mut a, vec![key(egui::Key::End, false)]);
        assert!(qa_native(&ctx, field).0.unwrap().is_empty());
        if prefix == "cut" {
            draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::A, false)]);
        }
        let before = qa_wire(&a);
        let native = qa_native(&ctx, field);
        let shadow = a.backup_presentation.document_value().to_owned();
        assert_eq!(shadow, "ls");
        let event = match prefix {
            "cut" => egui::Event::Cut,
            "undo" => qa_ctrl(egui::Key::Z, false),
            "selection" => qa_ctrl(egui::Key::A, false),
            _ => unreachable!(),
        };
        // Pinned egui processes the first edit/selection, then single-line
        // Enter surrenders focus and breaks before the later Paste. A dropped
        // delivery must roll back the WHOLE native transaction, not just Run.
        draw_frame(
            &ctx,
            &mut a,
            vec![
                event.clone(),
                key(egui::Key::Enter, false),
                egui::Event::Paste("not-delivered".into()),
            ],
        );
        assert!(
            a.metrics.backup.terminal.dropped
                && !a.metrics.backup.terminal.received
                && !a.metrics.backup.terminal.rejected,
            "Require actual ordered dropped delivery for {prefix}"
        );
        assert_eq!(qa_wire(&a), before, "Full caller retained after {prefix}");
        assert_eq!(a.backup_presentation.document_value(), shadow);
        assert_eq!(qa_native(&ctx, field), native);
        assert!(
            a.progress.attempts[&a.scenarios[i].id]
                .transcript
                .is_empty()
        );
        assert!(!a.metrics.backup.complete);
        // A later frame must not silently detect divergent shadow text and
        // replace the native generation (which would erase actual Undo).
        draw_frame(&ctx, &mut a, vec![]);
        assert_eq!(a.backup_presentation.document_value(), shadow);
        assert_eq!(qa_wire(&a), before);
        qa_focus(&ctx, &mut a);
        qa_undo_to(&ctx, &mut a, false, "pwd", false);
        qa_undo_to(&ctx, &mut a, false, "ls", true);
        assert_eq!(qa_wire(&a), before);
        // Positive control: the same first event genuinely edits/selects when
        // not followed by Enter + dropped Paste; no input-delivery fabrication.
        qa_focus(&ctx, &mut a);
        if prefix == "selection" {
            draw_frame(&ctx, &mut a, vec![key(egui::Key::End, false)]);
            assert!(qa_native(&ctx, field).0.unwrap().is_empty());
        }
        if prefix == "cut" {
            draw_frame(&ctx, &mut a, vec![qa_ctrl(egui::Key::A, false)]);
        }
        let prior_cursor = qa_native(&ctx, field).0;
        draw_frame(&ctx, &mut a, vec![event]);
        match prefix {
            "cut" => assert_eq!(qa_command(&a, false), ""),
            "undo" => {
                // Re-focusing after the negative test and Undo/Redo can add a
                // cursor-only undo point. Still require a real bounded text undo.
                qa_undo_to(&ctx, &mut a, false, "pwd", false);
                assert_eq!(qa_command(&a, false), "pwd");
            }
            "selection" => assert_ne!(qa_native(&ctx, field).0, prior_cursor),
            _ => unreachable!(),
        }
        assert!(
            a.progress.attempts[&a.scenarios[i].id]
                .transcript
                .is_empty()
        );
        // A normal later retry still executes in the bounded simulator.
        qa_type(&ctx, &mut a, false, "pwd");
        draw_frame(&ctx, &mut a, vec![key(egui::Key::Enter, false)]);
        let transcript = &a.progress.attempts[&a.scenarios[i].id].transcript;
        assert_eq!(transcript.len(), 1);
        assert!(transcript[0].starts_with("simulated root $ pwd\n"));
        assert!(!progress::current_solved(
            &a.progress.attempts[&a.scenarios[i].id],
            &a.scenarios[i]
        ));
    }
}
