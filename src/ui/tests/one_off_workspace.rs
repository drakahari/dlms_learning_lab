//! Prospective actual-egui tests for a synthetic one-off activity only.
//! No normal catalog registration or native/novice/accessibility acceptance.
use super::*;
use dlms_learning_lab::{one_off as core, one_off_workspace as work, script_teaching::View};
pub(in crate::ui) fn fixture() -> Scenario {
    let (mut s, _) = super::one_off_admission::prepared();
    s.title = "Synthetic one-off workspace fixture".into();
    s.objective="As Alex, queue /work/job.sh for five minutes from now. Run through its due time and verify /work/output.txt copies the live input, while preserving unrelated files.".into();
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    pack::load(&raw).unwrap().remove(0)
}
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.len();
    let source = fixture();
    assert!(!a.progress.attempts.contains_key(&source.id));
    a.scenarios.push(source);
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn state(a: &LabApp) -> &Attempt {
    &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
}
fn session(a: &LabApp, practice: bool) -> &work::Session {
    let w = state(a).workspace.as_ref().unwrap();
    if practice {
        &w.one_off_learning.as_ref().unwrap().session
    } else {
        w.one_off_session.as_ref().unwrap()
    }
}
fn live(a: &LabApp) -> &work::Assignment {
    let State::OneOff { simulation } = &state(a).state else {
        panic!("One-off fixture")
    };
    simulation
}
fn snapshot(a: &LabApp) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::to_value(&a.progress).unwrap(),
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
    )
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    click_at_size(ctx, a, r, 1200., 850.);
}
fn click_at_size(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, w: f32, h: f32) {
    // Arrive over the target in its own frame before pressing. Otherwise the
    // parent drag-only ScrollArea sees the entire prior pointer movement as a
    // drag on the press frame. This matches the native smoke cursor hover phase.
    frame(ctx, a, w, h, vec![egui::Event::PointerMoved(r.center())]);
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
// Match the native smoke cursor: hover first, then one real press/release
// event batch. A finite cooperative Check must not finish during generic
// post-click settling before the learner's immediate Stop can be delivered.
fn click_immediate(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let mut events = vec![egui::Event::PointerMoved(r.center())];
    for pressed in [true, false] {
        events.push(egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        });
    }
    frame(ctx, a, 1200., 850., events);
}
fn reveal(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    field: bool,
    w: f32,
    h: f32,
) -> egui::Rect {
    let mut stable = None;
    let mut trace = Vec::new();
    for _ in 0..20 {
        let (r, clip) = if field {
            let f = a
                .metrics
                .one_off
                .fields
                .iter()
                .find(|f| f.0 == key)
                .expect("Painted one-off field");
            (f.1, f.4)
        } else {
            let c = a
                .metrics
                .one_off
                .controls
                .iter()
                .find(|f| f.0 == key)
                .expect("Painted one-off control");
            (c.1, c.3)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let clip = clip.intersect(root);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if stable == Some((r, clip)) {
                return r;
            }
            stable = Some((r, clip));
            frame(ctx, a, w, h, vec![]);
            continue;
        }
        stable = None;
        trace.push((r, clip));
        let delta =
            (root.center().y - r.center().y).clamp(-root.height() * 0.75, root.height() * 0.75);
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(
                    root.left() + 1. / ctx.pixels_per_point(),
                    root.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        // Let this real wheel event settle before measuring the next displacement.
        // The outer loop still allows at most20 wheel events, with no forced scroll.
        for _ in 0..10 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("Cannot reveal {key} in20 real wheel steps: {trace:?}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false, 1200., 850.);
    click(ctx, a, r);
    draw(ctx, a);
}
fn keys(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
fn type_field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = reveal(ctx, a, key, true, 1200., 850.);
    click(ctx, a, r);
    let id = a
        .metrics
        .one_off
        .fields
        .iter()
        .find(|f| f.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let mut e = keys(egui::Key::A);
    e.push(egui::Event::Paste(text.into()));
    frame(ctx, a, 1200., 850., e);
    draw(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, choice: &str) {
    tap(ctx, a, key);
    let r = a
        .metrics
        .one_off
        .choices
        .iter()
        .find(|(k, _)| k == choice)
        .expect("Actual popup choice")
        .1;
    click(ctx, a, r);
    draw(ctx, a);
}
fn assignment(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, "assignment");
    assert_eq!(
        state(a)
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    type_field(ctx, a, "command", text);
    let count = a.one_off_presentation.command_executions();
    tap(ctx, a, "command-run");
    assert_eq!(
        a.one_off_presentation.command_executions(),
        count + 1,
        "A new actual admitted command must execute, even if an older output is identical"
    );
    assert_eq!(session(a, false).command, text);
}
fn wait(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..300 {
        if !a.one_off_presentation.running()
            && !a
                .one_off_presentation
                .memo
                .pending(state(a), &a.scenarios[a.selected.unwrap()])
        {
            return;
        }
        draw(ctx, a);
    }
    panic!("Bounded continuation did not settle in300 frames");
}
fn solve(ctx: &egui::Context, a: &mut LabApp) {
    choose(ctx, a, "actor", "actor:alex");
    command(ctx, a, "at -f /work/job.sh now + 5 minutes");
    type_field(ctx, a, "end", "2026-01-06 10:06");
    tap(ctx, a, "run");
    wait(ctx, a);
    tap(ctx, a, "check");
    wait(ctx, a);
    assert!(
        a.one_off_presentation
            .memo
            .rows(state(a), &a.scenarios[a.selected.unwrap()], false)
            .unwrap()
            .iter()
            .all(|r| r.0)
    );
}
#[test]
fn actual_editor_save_queue_inspect_remove_run_and_current_checks_use_real_state() {
    let (ctx, mut a, _) = opened();
    assignment(&ctx, &mut a);
    choose(&ctx, &mut a, "actor", "actor:alex");
    type_field(
        &ctx,
        &mut a,
        "document",
        "cat -- /work/input.txt > /work/output.txt\n",
    );
    assert_ne!(
        session(&a, false).drafts["/work/job.sh"],
        live(&a).simulation.files["/work/job.sh"]
    );
    tap(&ctx, &mut a, "save");
    assert_eq!(
        session(&a, false).drafts["/work/job.sh"],
        live(&a).simulation.files["/work/job.sh"]
    );
    command(&ctx, &mut a, "at -f /work/job.sh now + 5 minutes");
    assert_eq!(live(&a).simulation.jobs.len(), 1);
    command(&ctx, &mut a, "atq");
    assert!(session(&a, false).output.contains("alex"));
    command(&ctx, &mut a, "at -c 1");
    assert!(session(&a, false).output.contains("cat --"));
    command(&ctx, &mut a, "atrm 1");
    assert!(live(&a).simulation.jobs.is_empty());
    command(&ctx, &mut a, "at -f /work/job.sh now + 5 minutes");
    type_field(&ctx, &mut a, "end", "2026-01-06 10:06");
    tap(&ctx, &mut a, "run");
    wait(&ctx, &mut a);
    assert_eq!(
        live(&a).simulation.files["/work/output.txt"],
        "live first\n"
    );
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(a.metrics.completion);
    type_field(
        &ctx,
        &mut a,
        "document",
        "printf 'wrong\\n' > /work/output.txt\n",
    );
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        state(&a),
        &a.scenarios[a.selected.unwrap()]
    ));
    tap(&ctx, &mut a, "save");
    assert!(live(&a).last_run.is_none());
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(a.metrics.one_off.rows.iter().any(|r| !r.1));
}
#[test]
fn introduction_insert_example_experiment_and_repair_do_not_install_assignment_credit() {
    let (ctx, mut a, _) = opened();
    assert!(
        a.metrics
            .one_off
            .labels
            .iter()
            .any(|(t, _)| t.contains("captur"))
    );
    let original_state = serde_json::to_value(&state(&a).state).unwrap();
    let original_session = serde_json::to_value(session(&a, false)).unwrap();
    tap(&ctx, &mut a, "try-example");
    tap(&ctx, &mut a, "insert");
    assert_eq!(session(&a, true).actor, "sam");
    assert!(
        state(&a)
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .assignment
            .simulation
            .jobs
            .is_empty()
    );
    choose(&ctx, &mut a, "example", "example:1");
    tap(&ctx, &mut a, "insert");
    assert_eq!(
        session(&a, true).command,
        "at -f /work/example-job.sh now + 5 minutes"
    );
    assert!(
        state(&a)
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .assignment
            .simulation
            .jobs
            .is_empty(),
        "Insert must be inert"
    );
    tap(&ctx, &mut a, "command-run");
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    for text in ["atrm 1", "at -f /work/example-job.sh now + 6 minutes"] {
        type_field(&ctx, &mut a, "command", text);
        tap(&ctx, &mut a, "command-run");
    }
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(a.metrics.one_off.rows.iter().any(|r| !r.1));
    for text in ["atrm 2", "at -f /work/example-job.sh now + 5 minutes"] {
        type_field(&ctx, &mut a, "command", text);
        tap(&ctx, &mut a, "command-run");
    }
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(
        a.one_off_presentation
            .memo
            .rows(state(&a), &a.scenarios[a.selected.unwrap()], true)
            .unwrap()
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(
        serde_json::to_value(&state(&a).state).unwrap(),
        original_state
    );
    assert_eq!(
        serde_json::to_value(session(&a, false)).unwrap(),
        original_session
    );
    assert!(!a.metrics.completion);
}
#[test]
fn failed_command_and_same_frame_rejected_text_packet_keep_full_attempt_and_native_cursor() {
    let (ctx, mut a, _) = opened();
    assignment(&ctx, &mut a);
    choose(&ctx, &mut a, "actor", "actor:alex");
    type_field(
        &ctx,
        &mut a,
        "command",
        "at -f /work/missing.sh now + 5 minutes",
    );
    let before = serde_json::to_value(&state(&a).state).unwrap();
    tap(&ctx, &mut a, "command-run");
    assert_eq!(serde_json::to_value(&state(&a).state).unwrap(), before);
    assert!(a.one_off_presentation.notice().unwrap().contains("file"));
    type_field(&ctx, &mut a, "document", "echo retained\n");
    let r = reveal(&ctx, &mut a, "document", true, 1200., 850.);
    click(&ctx, &mut a, r);
    let before = snapshot(&a);
    let id = a
        .metrics
        .one_off
        .fields
        .iter()
        .find(|f| f.0 == "document")
        .unwrap()
        .2;
    let cursor = egui::text_edit::TextEditState::load(&ctx, id)
        .unwrap()
        .cursor
        .char_range();
    let mut e = keys(egui::Key::A);
    e.push(egui::Event::Paste("x".repeat(core::MAX_FILES_BYTES + 1)));
    e.extend(keys(egui::Key::Enter));
    frame(&ctx, &mut a, 1200., 850., e);
    assert_eq!(snapshot(&a), before);
    assert_eq!(
        egui::text_edit::TextEditState::load(&ctx, id)
            .unwrap()
            .cursor
            .char_range(),
        cursor
    );
    assert!(!a.one_off_presentation.running());
}
#[test]
fn real_keyboard_undo_redo_tab_and_save_preserve_editor_behavior() {
    let (ctx, mut a, _) = opened();
    assignment(&ctx, &mut a);
    type_field(&ctx, &mut a, "document", "echo typed\n");
    let saved = session(&a, false).drafts["/work/job.sh"].clone();
    // Actual public egui time/event packets establish stable checkpoints;
    // no cursor, undo stack, document state or focus is installed.
    let timed = |at: f64, events: Vec<egui::Event>, a: &mut LabApp| {
        let _ = ctx.run(
            egui::RawInput {
                time: Some(at),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200. / a.scale, 850. / a.scale),
                )),
                events,
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
    };
    timed(100., vec![], &mut a);
    timed(101.3, vec![], &mut a);
    timed(101.4, vec![egui::Event::Paste("# more\n".into())], &mut a);
    timed(102.8, vec![], &mut a);
    let changed = session(&a, false).drafts["/work/job.sh"].clone();
    assert_ne!(changed, saved);
    frame(&ctx, &mut a, 1200., 850., keys(egui::Key::Z));
    draw(&ctx, &mut a);
    assert_eq!(session(&a, false).drafts["/work/job.sh"], saved);
    frame(&ctx, &mut a, 1200., 850., keys(egui::Key::Y));
    draw(&ctx, &mut a);
    assert_eq!(session(&a, false).drafts["/work/job.sh"], changed);
    let focused = ctx.memory(|m| m.focused());
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut a);
    assert_ne!(ctx.memory(|m| m.focused()), focused);
    tap(&ctx, &mut a, "save");
    assert_eq!(live(&a).simulation.files["/work/job.sh"], changed);
}
#[test]
fn pending_run_stop_and_modal_cancel_keep_all_saved_state_with_bounded_frame_work() {
    let (ctx, mut a, _) = opened();
    assignment(&ctx, &mut a);
    choose(&ctx, &mut a, "actor", "actor:alex");
    type_field(&ctx, &mut a, "document", &"printf '%s' ''\n".repeat(32));
    tap(&ctx, &mut a, "save");
    for _ in 0..16 {
        command(&ctx, &mut a, "at -f /work/job.sh now + 5 minutes");
    }
    type_field(&ctx, &mut a, "end", "2026-01-06 10:06");
    let before = snapshot(&a);
    tap(&ctx, &mut a, "run");
    assert!(a.one_off_presentation.running());
    tap(&ctx, &mut a, "stop");
    assert!(!a.one_off_presentation.running());
    assert_eq!(snapshot(&a), before);
    tap(&ctx, &mut a, "run");
    assert!(a.one_off_presentation.running());
    a.settings_open = true;
    draw(&ctx, &mut a);
    assert!(!a.one_off_presentation.running());
    assert_eq!(snapshot(&a), before);
    a.settings_open = false;
}
#[test]
fn reset_cancel_confirm_and_save_resume_preserve_full_global_and_other_work() {
    let (ctx, mut a, i) = opened();
    assignment(&ctx, &mut a);
    solve(&ctx, &mut a);
    let old_other = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-subnet-basics")
        .unwrap_or(0);
    a.launch(old_other);
    draw(&ctx, &mut a);
    a.launch(i);
    draw(&ctx, &mut a);
    a.progress.favorites.insert(a.scenarios[i].id.clone());
    let before = snapshot(&a);
    let visited = state(&a).visited;
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    let cancel = a.metrics.reset_cancel.unwrap();
    click(&ctx, &mut a, cancel);
    draw(&ctx, &mut a);
    assert_eq!(snapshot(&a), before);
    let path = a.path.clone();
    progress::save(&path, &a.progress).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::decode(&raw, &a.scenarios).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), before.0);
    let receipts = loaded
        .migration_receipts
        .as_ref()
        .expect("Actual normal writer receipts");
    assert!(receipts.workspace);
    assert!(receipts.shell.contains(&shell::EVALUATOR_REVISION));
    assert!(progress::current_solved(
        &loaded.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
    // Actual Reset dialog and selected reset law, never installing loaded Progress.
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    let confirm = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, confirm);
    draw(&ctx, &mut a);
    let mut fresh = Attempt::new(&a.scenarios[i], visited);
    work::budget::prepare(&mut fresh, &a.scenarios[i]).unwrap();
    let mut expected = before.0.clone();
    expected["attempts"][&a.scenarios[i].id] = serde_json::to_value(&fresh).unwrap();
    assert_eq!(snapshot(&a).0, expected);
    assert_eq!(snapshot(&a).1, before.1);
    assert_eq!(state(&a).visited, visited);
    assert!(!progress::current_solved(state(&a), &a.scenarios[i]));
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn all_supported_text_scales_light_dark_and_two_widths_keep_shared_controls_editable() {
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for theme in ["dlms-dark", "dlms-light"] {
            for width in [760., 1600.] {
                let (ctx, mut a, _) = opened();
                assignment(&ctx, &mut a);
                a.scale = scale;
                a.theme_id = theme.into();
                theme::apply(&ctx, theme);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                assert!(a.metrics.one_off.goal.is_some());
                let field = reveal(&ctx, &mut a, "command", true, width, 929.);
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(field.left() >= root.left() - 1. && field.right() <= root.right() + 1.);
                let button = a
                    .metrics
                    .one_off
                    .controls
                    .iter()
                    .find(|c| c.0 == "command-run")
                    .unwrap()
                    .1;
                assert!(
                    (field.height() - button.height()).abs() < 1.,
                    "Related single-line field/button heights differ at {scale}/{theme}/{width}"
                );
                click_at_size(&ctx, &mut a, field, width, 929.);
                let mut e = keys(egui::Key::A);
                e.push(egui::Event::Paste("atq".into()));
                frame(&ctx, &mut a, width, 929., e);
                assert_eq!(session(&a, false).command, "atq");
                assert!(ctx.memory(|m| m.focused()).is_some());
            }
        }
    }
}
#[test]
fn challenge_keeps_full_assignment_and_optional_help_without_unsolicited_worked_content() {
    let (ctx, mut a, _) = opened();
    let original = serde_json::to_value(&state(&a).state).unwrap();
    let r = a.metrics.mode_challenge.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(state(&a).challenge);
    assert!(a.metrics.one_off.goal.is_some());
    assert!(
        !a.metrics
            .one_off
            .controls
            .iter()
            .any(|c| c.0 == "insert" || c.0 == "try-example")
    );
    assert_eq!(serde_json::to_value(&state(&a).state).unwrap(), original);
    assert!(live(&a).simulation.jobs.is_empty());
    assert!(!a.metrics.completion);
    let learn = a.metrics.mode_learn.unwrap();
    click(&ctx, &mut a, learn);
    draw(&ctx, &mut a);
    assert!(!state(&a).challenge);
    assert_eq!(serde_json::to_value(&state(&a).state).unwrap(), original);
}

#[test]
fn rejected_held_primary_release_and_discard_do_not_run_old_command_or_change_cursor() {
    let (ctx, mut a, _) = opened();
    assignment(&ctx, &mut a);
    type_field(&ctx, &mut a, "command", "atq");
    let run = reveal(&ctx, &mut a, "command-run", false, 1200., 850.);
    let field = a
        .metrics
        .one_off
        .fields
        .iter()
        .find(|f| f.0 == "command")
        .unwrap()
        .1;
    click(&ctx, &mut a, field);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::PointerMoved(run.center()),
            egui::Event::PointerButton {
                pos: run.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    let id = a
        .metrics
        .one_off
        .fields
        .iter()
        .find(|f| f.0 == "command")
        .unwrap()
        .2;
    for _ in 0..20 {
        if ctx.memory(|m| m.focused()) == Some(id) {
            break;
        }
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, true)]);
        frame(&ctx, &mut a, 1200., 850., vec![]);
    }
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let old = snapshot(&a);
    let executions = a.one_off_presentation.command_executions();
    let cursor = egui::text_edit::TextEditState::load(&ctx, id)
        .unwrap()
        .cursor
        .char_range();
    let mut events = keys(egui::Key::A);
    events.push(egui::Event::Paste("x".repeat(2049)));
    frame(&ctx, &mut a, 1200., 850., events);
    assert!(a.one_off_presentation.gesture_blocked());
    assert_eq!(snapshot(&a), old);
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 850.),
            )),
            events: vec![
                egui::Event::PointerMoved(run.center()),
                egui::Event::PointerButton {
                    pos: run.center(),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            if ctx.current_pass_index() == 0 {
                ctx.request_discard("held rejected release must remain atomic");
            }
        },
    );
    assert_eq!(out.platform_output.num_completed_passes, 2);
    assert_eq!(snapshot(&a), old);
    assert_eq!(a.one_off_presentation.command_executions(), executions);
    assert_eq!(
        egui::text_edit::TextEditState::load(&ctx, id)
            .unwrap()
            .cursor
            .char_range(),
        cursor
    );
    assert_eq!(a.one_off_presentation.command_value(), "atq");
    draw(&ctx, &mut a);
    type_field(&ctx, &mut a, "command", "atq");
    tap(&ctx, &mut a, "command-run");
    assert_eq!(a.one_off_presentation.command_executions(), executions + 1);
}

fn deep_check_opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let mut s = fixture();
    let engine::Goal::OneOff { definition } = &mut s.goal else {
        panic!("one-off source")
    };
    let sample = definition.probes[0].clone();
    definition.probes = (0..work::MAX_CASES)
        .map(|n| {
            let mut p = sample.clone();
            p.name = format!("Bounded slow replay sample {n}");
            let input = format!("independent bounded sample {n}\n");
            p.files.insert("/work/input.txt".into(), input.clone());
            p.after.insert("/work/input.txt".into(), input.clone());
            p.after.insert("/work/output.txt".into(), input);
            p
        })
        .collect();
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    let s = pack::load(&raw).unwrap().remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s); // Trusted initial synthetic source only, never an Attempt/result.
    a.launch(i);
    draw(&ctx, &mut a);
    assignment(&ctx, &mut a);
    choose(&ctx, &mut a, "actor", "actor:alex");
    let source = "printf '%s' ''\n".repeat(31) + "cat /work/input.txt > /work/output.txt\n";
    type_field(&ctx, &mut a, "document", &source);
    tap(&ctx, &mut a, "save");
    command(&ctx, &mut a, "at -f /work/job.sh now + 5 minutes");
    type_field(&ctx, &mut a, "end", "2026-01-06 10:06");
    tap(&ctx, &mut a, "run");
    wait(&ctx, &mut a);
    (ctx, a, i)
}
#[test]
fn actual_check_stop_idle_back_resume_keeps_full_work_and_needs_explicit_restart() {
    let (ctx, mut a, i) = deep_check_opened();
    let before = snapshot(&a);
    tap(&ctx, &mut a, "check");
    assert!(
        a.one_off_presentation
            .memo
            .pending(state(&a), &a.scenarios[i])
    );
    tap(&ctx, &mut a, "stop");
    assert!(
        a.one_off_presentation
            .memo
            .paused(state(&a), &a.scenarios[i])
    );
    let stopped = a.one_off_presentation.memo.counters.steps;
    assert_eq!(snapshot(&a), before);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped);
    accounts_ui_click(&ctx, &mut a, |m| m.back);
    assert!(a.selected.is_none());
    assert_eq!(snapshot(&a), before);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped);
    a.filter.search = a.scenarios[i].title.clone(); // Presentation-only controlled filter.
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    let continue_button = a.metrics.card_actions[0].0;
    click(&ctx, &mut a, continue_button);
    draw(&ctx, &mut a);
    assert_eq!(a.selected, Some(i));
    let mut expected = before.0.clone();
    let clock = before.0["clock"].as_u64().unwrap() + 1;
    expected["clock"] = clock.into();
    expected["attempts"][&a.scenarios[i].id]["visited"] = clock.into();
    assert_eq!(snapshot(&a), (expected, before.1));
    assert!(
        a.one_off_presentation
            .memo
            .paused(state(&a), &a.scenarios[i])
    );
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped);
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(
        a.one_off_presentation
            .memo
            .rows(state(&a), &a.scenarios[i], false)
            .unwrap()
            .iter()
            .all(|r| r.0)
    );
}
#[test]
fn actual_settings_modal_cancels_check_without_ticking_or_restarting_on_close() {
    let (ctx, mut a, i) = deep_check_opened();
    let before = snapshot(&a);
    tap(&ctx, &mut a, "check");
    assert!(
        a.one_off_presentation
            .memo
            .pending(state(&a), &a.scenarios[i])
    );
    let settings = a.metrics.settings.unwrap();
    click(&ctx, &mut a, settings);
    draw(&ctx, &mut a);
    assert!(a.settings_open);
    assert!(
        a.one_off_presentation
            .memo
            .paused(state(&a), &a.scenarios[i])
    );
    let stopped = a.one_off_presentation.memo.counters.steps;
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped);
    assert_eq!(snapshot(&a), before);
    let close = a.metrics.settings_close.unwrap().0;
    click(&ctx, &mut a, close);
    draw(&ctx, &mut a);
    assert!(!a.settings_open);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped);
    assert_eq!(snapshot(&a), before);
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert!(
        a.one_off_presentation
            .memo
            .rows(state(&a), &a.scenarios[i], false)
            .unwrap()
            .iter()
            .all(|r| r.0)
    );
}

// Only trusted, local scenario/unit source setup. All Attempts below are created
// by actual library Start/Continue and edited/run through native controls.
fn next_unit_sources() -> (egui::Context, LabApp, [usize; 2]) {
    let (ctx, mut a) = app();
    let first = a.scenarios.len();
    let mut source = fixture();
    source.id = "synthetic-oneoff-next-first".into();
    source.title = "Synthetic OneOff Next first assignment".into();
    source.unit = "synthetic-oneoff-next-unit".into();
    let mut sibling = source.clone();
    sibling.id = "synthetic-oneoff-next-stopped".into();
    sibling.title = "Synthetic OneOff Next stopped sibling".into();
    let source_raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![source, sibling],
    })
    .unwrap();
    let sources = pack::load(&source_raw).unwrap();
    let unit = course::Unit {
        id: "synthetic-oneoff-next-unit".into(),
        track: "LPIC-1".into(),
        title: "Synthetic OneOff Next source fixture".into(),
        summary: "Two unchanged one-off assignments for native navigation".into(),
        prerequisites: vec![],
        labs: sources.iter().map(|s| s.id.clone()).collect(),
    };
    assert!(
        sources
            .iter()
            .all(|s| !a.progress.attempts.contains_key(&s.id))
    );
    a.scenarios.extend(sources);
    a.units.push(unit); // Local catalog data, never bundled registration/progress.
    draw(&ctx, &mut a);
    (ctx, a, [first, first + 1])
}
fn next_start(ctx: &egui::Context, a: &mut LabApp, i: usize) {
    assert!(a.selected.is_none());
    a.filter.search = a.scenarios[i].title.clone(); // Presentation-only search.
    a.page = 0;
    draw(ctx, a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    let rect = a.metrics.card_actions[0].0;
    let clip = a
        .metrics
        .main_viewport
        .unwrap()
        .intersect(ctx.content_rect());
    assert!(smoke::smoke_control_visible(
        rect,
        clip,
        ctx.pixels_per_point()
    ));
    click(ctx, a, rect);
    draw(ctx, a);
    assert_eq!(a.selected, Some(i));
    assignment(ctx, a);
}
fn next_counts(a: &LabApp) -> (usize, usize, usize, usize, usize, usize) {
    let memo = a.one_off_presentation.memo.counters;
    let index = a.completion_index.observed().2;
    (
        memo.snapshots,
        memo.meters,
        memo.steps,
        index.contexts,
        index.key_copies,
        index.certificates,
    )
}
#[test]
fn actual_unit_next_keeps_stopped_sibling_pending_without_synchronous_replay_or_idle_copies() {
    use crate::ui::completion_index::Status;
    let (ctx, mut a, [first, sibling]) = next_unit_sources();
    next_start(&ctx, &mut a, sibling);
    choose(&ctx, &mut a, "actor", "actor:alex");
    // A legitimate32-command script within the unchanged subset makes an actual
    // Check observable. It changes no Goal, queue model or execution ceiling.
    let script = "printf '%s' ''\n".repeat(31) + "cat /work/input.txt > /work/output.txt\n";
    type_field(&ctx, &mut a, "document", &script);
    tap(&ctx, &mut a, "save");
    command(&ctx, &mut a, "at -f /work/job.sh now + 5 minutes");
    type_field(&ctx, &mut a, "end", "2026-01-06 10:06");
    tap(&ctx, &mut a, "run");
    wait(&ctx, &mut a);
    // Check/Stop are ordinary clicks on actually painted, visible controls.
    // Do not spend a generic reveal sequence advancing a short finite Check.
    let check = reveal(&ctx, &mut a, "check", false, 1200., 850.);
    let check_layout_before = (
        a.metrics.main_scroll_offset,
        a.metrics.one_off.documents.clone(),
        a.metrics.one_off.goal,
    );
    click_immediate(&ctx, &mut a, check);
    frame(&ctx, &mut a, 1200., 850., vec![]); // Paint the actual Stop control.
    assert!(
        (a.metrics.main_scroll_offset.y - check_layout_before.0.y).abs()
            <= 1. / ctx.pixels_per_point(),
        "Arriving then pressing Check must not drag the main viewport: before {:?}, after {:?}",
        check_layout_before.0,
        a.metrics.main_scroll_offset,
    );
    assert!(
        a.one_off_presentation
            .memo
            .pending(state(&a), &a.scenarios[sibling])
    );
    let stop = a
        .metrics
        .one_off
        .controls
        .iter()
        .find(|c| c.0 == "stop")
        .unwrap();
    assert!(stop.2);
    let stop_rect = stop.1;
    let clip = stop
        .3
        .intersect(a.metrics.main_viewport.unwrap())
        .intersect(ctx.content_rect());
    assert!(
        smoke::smoke_control_visible(stop_rect, clip, ctx.pixels_per_point()),
        "Check {check:?}, Stop {stop_rect:?}, clip {clip:?}, offset {:?}, focus {:?}, before {check_layout_before:?}, documents after {:?}, goal after {:?}",
        a.metrics.main_scroll_offset,
        ctx.memory(|m| m.focused()),
        a.metrics.one_off.documents,
        a.metrics.one_off.goal
    );
    let before_stop = snapshot(&a);
    click_immediate(&ctx, &mut a, stop_rect);
    draw(&ctx, &mut a);
    assert!(
        a.one_off_presentation
            .memo
            .paused(state(&a), &a.scenarios[sibling])
    );
    assert!(
        !a.one_off_presentation
            .memo
            .pending(state(&a), &a.scenarios[sibling])
    );
    assert_eq!(snapshot(&a), before_stop);
    assert_eq!(
        a.completion_index.status(state(&a), &a.scenarios[sibling]),
        Some(Status::Pending)
    );
    let stopped_steps = a.one_off_presentation.memo.counters.steps;
    for _ in 0..8 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.one_off_presentation.memo.counters.steps, stopped_steps);
    accounts_ui_click(&ctx, &mut a, |m| m.back);
    draw(&ctx, &mut a);
    next_start(&ctx, &mut a, first);
    solve(&ctx, &mut a);
    assert_eq!(
        a.completion_index.status(state(&a), &a.scenarios[first]),
        Some(Status::Complete)
    );
    let sibling_id = a.scenarios[sibling].id.clone();
    assert_eq!(
        a.completion_index
            .status(&a.progress.attempts[&sibling_id], &a.scenarios[sibling]),
        Some(Status::Pending)
    );
    // This is the before/after discriminator: the old course::next fallback
    // evaluates the correct sibling state synchronously and wrongly skips it.
    assert!(
        a.metrics.next_unit.is_some(),
        "Stopped sibling must remain the next independent assignment"
    );
    assert!(a.metrics.unit_complete.is_none());
    let before_idle = snapshot(&a);
    let counts = next_counts(&a);
    for _ in 0..20 {
        draw(&ctx, &mut a);
        assert!(a.metrics.next_unit.is_some());
        assert!(a.metrics.unit_complete.is_none());
    }
    assert_eq!(
        next_counts(&a),
        counts,
        "Painting Next/idle must not grade, snapshot, size or copy more inputs"
    );
    assert_eq!(snapshot(&a), before_idle);
    let before_navigation = snapshot(&a);
    let before_work = next_counts(&a);
    accounts_ui_click(&ctx, &mut a, |m| m.next_unit);
    draw(&ctx, &mut a);
    assert_eq!(a.selected, Some(sibling));
    let clock = before_navigation.0["clock"].as_u64().unwrap() + 1;
    let mut expected = before_navigation.0;
    expected["clock"] = clock.into();
    expected["attempts"][&sibling_id]["visited"] = clock.into();
    assert_eq!(snapshot(&a), (expected, before_navigation.1));
    assert!(
        a.one_off_presentation
            .memo
            .paused(state(&a), &a.scenarios[sibling])
    );
    assert!(!a.metrics.completion);
    assert_eq!(
        a.completion_index.status(state(&a), &a.scenarios[sibling]),
        Some(Status::Pending)
    );
    let after_work = next_counts(&a);
    assert_eq!(
        after_work.2, before_work.2,
        "Actual Next must not restart or synchronously replay the stopped check"
    );
    // Recency changes the complete key. Exactly one bounded admission is allowed
    // for that visit; requiring zero copies would weaken full-input identity.
    assert!(after_work.0 - before_work.0 <= 1);
    assert!(after_work.1 - before_work.1 <= 1);
    assert!(after_work.3 - before_work.3 <= 1);
    assert!(after_work.4 - before_work.4 <= 1);
    assert_eq!(after_work.5, before_work.5);
    let paused_counts = next_counts(&a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(next_counts(&a), paused_counts);
    tap(&ctx, &mut a, "check");
    wait(&ctx, &mut a);
    assert_eq!(
        a.completion_index.status(state(&a), &a.scenarios[sibling]),
        Some(Status::Complete)
    );
    assert!(a.metrics.unit_complete.is_some());
    assert!(a.metrics.next_unit.is_none());
}
