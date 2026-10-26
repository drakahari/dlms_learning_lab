//! Prospective egui controller checks. No native renderer or cold-process claim.
use super::*;

const ID: &str = "shell-handover";

// Deliver the controller's queued events to actual widgets, then execute the
// same SmokeTick dispatcher as the native app. Wall time alone is accelerated;
// VM execution, input, visibility and stage predicates remain unchanged.
fn tick(ctx: &egui::Context, a: &mut LabApp) {
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .rev()
        .find_map(|event| match event {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or_default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 850.),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        },
    );
}
fn opened(mode: HistorySource) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.history_source = Some(mode);
    a.smoke_stage = 15000;
    tick(&ctx, &mut a);
    assert_eq!(a.smoke_stage, 15001, "actual opening phase advanced");
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    (ctx, a, i)
}
fn reference_through_editor(ctx: &egui::Context, a: &mut LabApp, i: usize) {
    a.smoke = false;
    a.set_script_view(i, teaching::View::Assignment);
    a.focus_input = true;
    draw(ctx, a);
    let editor = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(editor)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Text(HistorySource::Writer8.solution().into()),
        ],
    );
    frame(ctx, a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    for _ in 0..240 {
        frame(ctx, a, 1200., 850., vec![]);
        if a.script_job.is_none()
            && progress::current_solved(&a.progress.attempts[ID], &a.scenarios[i])
        {
            a.smoke = true;
            return;
        }
    }
    panic!("real editor Run must produce a current passing reference");
}
#[test]
fn getopts_history_mode_is_closed_and_smoke_only() {
    for name in ["case-history", "getopts-history"] {
        assert_eq!(HistorySource::from_stage(false, Some(name)), None);
    }
    assert_eq!(
        HistorySource::from_stage(true, Some("case-history")),
        Some(HistorySource::Writer7)
    );
    assert_eq!(
        HistorySource::from_stage(true, Some("getopts-history")),
        Some(HistorySource::Writer8)
    );
    for name in [None, Some("getopts-history-extra"), Some("shell-case")] {
        assert_eq!(HistorySource::from_stage(true, name), None);
    }
}
#[test]
fn getopts_history_exact_smoke_tick_opening_preserves_both_frozen_profiles_without_credit() {
    const {
        assert!(dlms_learning_lab::shell::EVALUATOR_REVISION >= 9);
    }
    for (mode, contexts, revisions) in [
        (HistorySource::Writer7, 102, vec![7]),
        (HistorySource::Writer8, 116, vec![7, 8]),
    ] {
        let (ctx, mut a, i) = opened(mode);
        assert_eq!(a.scenarios.len(), 140);
        assert_eq!(a.progress.attempts.len(), contexts);
        assert_eq!(a.selected, Some(i));
        assert_eq!(
            a.path.parent(),
            Some(a._smoke_profile.as_ref().unwrap().path())
        );
        assert_eq!(std::fs::read(&a.path).unwrap(), mode.bytes());
        assert_eq!(
            a.progress.migration_receipts.as_ref().unwrap().shell,
            revisions
        );
        let backup = a.path.with_extension(format!(
            "before-shell-v{}.json",
            dlms_learning_lab::shell::EVALUATOR_REVISION
        ));
        assert!(
            !backup.exists(),
            "opening alone does not manufacture a migration save"
        );
        for attempt in a.progress.attempts.values() {
            if let State::Script { run: Some(run), .. } = &attempt.state {
                assert!(run.engine_revision < 9);
            }
        }
        let learning = a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap();
        assert_eq!(learning.example_revision, 7);
        assert!(!progress::current_solved(
            &a.progress.attempts[ID],
            &a.scenarios[i]
        ));
        // Exercise the real migration-save phase after the exact opening phase.
        a.smoke_stage = 15002;
        tick(&ctx, &mut a);
        assert_eq!(a.smoke_stage, 15003);
        assert_eq!(std::fs::read(backup).unwrap(), mode.bytes());
        let expected = if mode == HistorySource::Writer7 {
            vec![7, dlms_learning_lab::shell::EVALUATOR_REVISION]
        } else {
            vec![7, 8, dlms_learning_lab::shell::EVALUATOR_REVISION]
        };
        // The writer stamps only its on-disk copy; caller data stays unchanged.
        assert_eq!(
            a.progress.migration_receipts.as_ref().unwrap().shell,
            revisions
        );
        let persisted = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            persisted.migration_receipts.as_ref().unwrap().shell,
            expected
        );
    }
}
#[test]
fn getopts_history_opening_rejects_a_path_outside_its_private_profile_before_writing() {
    let (ctx, mut a) = app();
    let outside = tempfile::tempdir().unwrap();
    let path = outside.path().join("progress.json");
    std::fs::write(&path, b"untouched synthetic sentinel").unwrap();
    a.path = path.clone();
    a.smoke = true;
    a.history_source = Some(HistorySource::Writer8);
    a.smoke_stage = 15000;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick(&ctx, &mut a)));
    assert!(result.is_err());
    assert_eq!(
        std::fs::read(path).unwrap(),
        b"untouched synthetic sentinel"
    );
}
#[test]
fn getopts_history_new_example_branch_uses_actual_input_run() {
    let (ctx, mut a, i) = opened(HistorySource::Writer8);
    reference_through_editor(&ctx, &mut a, i);
    a.set_script_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let old = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(old.example_revision, 7);
    a.smoke_stage = 15011;
    tick(&ctx, &mut a);
    assert_eq!(
        a.smoke_stage, 15100,
        "new branch must be reachable through SmokeTick"
    );
    for _ in 0..300 {
        tick(&ctx, &mut a);
        if a.smoke_stage == 15104 {
            break;
        }
    }
    assert_eq!(a.smoke_stage, 15104);
    assert!(
        a.capture_pending,
        "actual example-success capture phase executed"
    );
    let learning = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        learning.example_revision,
        dlms_learning_lab::shell::EVALUATOR_REVISION
    );
    assert_eq!(
        learning.example_source,
        HistorySource::Writer8.example_source()
    );
    assert_ne!(learning.example_source, HistorySource::Writer8.solution());
    assert!(
        a.scenarios[i]
            .teaching
            .as_ref()
            .unwrap()
            .fixture
            .accepts(learning.example_result.as_ref().unwrap())
    );
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn getopts_history_stop_phase_observes_inherited_child_scanner_before_actual_cancel() {
    let (ctx, mut a, i) = opened(HistorySource::Writer8);
    reference_through_editor(&ctx, &mut a, i);
    a.smoke_stage = 15012;
    let mut reached = false;
    for _ in 0..200 {
        tick(&ctx, &mut a);
        reached |= a
            .script_job
            .as_ref()
            .is_some_and(|job| job.has_case_native_child_marker());
        if a.smoke_stage == 15017 {
            break;
        }
    }
    assert!(
        reached,
        "child emits marker only after inherited scanner returns b"
    );
    assert_eq!(a.smoke_stage, 15017);
    assert!(
        a.capture_pending,
        "actual stopped-state capture phase executed"
    );
    assert!(a.script_job.is_none());
    assert!(
        matches!(&a.progress.attempts[ID].state, State::Script {run: None, source} if source.contains("getopts ab child -ab"))
    );
    assert!(
        a.notice
            .as_deref()
            .is_some_and(|n| n.starts_with("Stopped"))
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}

#[test]
fn getopts_history_example_return_routes_through_actual_assignment_tab_to_original_stop_queue() {
    let (ctx, mut a, i) = opened(HistorySource::Writer8);
    a.set_script_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    a.smoke_stage = 15104;
    for _ in 0..80 {
        tick(&ctx, &mut a);
        if a.smoke_stage == 15012 {
            break;
        }
    }
    assert_eq!(a.smoke_stage, 15012);
    assert_eq!(a.script_view(i), teaching::View::Assignment);
    assert!(!a.capture_pending);
    assert!(a.script_job.is_none());
}

#[test]
fn getopts_history_finish_holds_both_modes_until_asynchronous_close() {
    for mode in [HistorySource::Writer7, HistorySource::Writer8] {
        let (ctx, mut a, _) = opened(mode);
        a.smoke_stage = 15075;
        for _ in 0..2 {
            tick(&ctx, &mut a);
            assert_eq!(
                a.smoke_stage, 15075,
                "a delayed close cannot dispatch an unhandled phase"
            );
            assert!(a.pending_events.is_empty());
            assert!(!a.capture_pending);
        }
    }
}
