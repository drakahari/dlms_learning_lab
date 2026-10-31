//! Headless authoring regressions. Fixture append is confined to this test module;
//! the production controller requires the registered133 catalogue and normal unit.
use super::*;
use serde::Deserialize;
const IDS: [&str; 3] = [
    "shell-options-report",
    "shell-options-jobs",
    "shell-options-publication",
];
fn proposed_app() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    if !a.scenarios.iter().any(|s| s.id == IDS[0]) {
        #[derive(Deserialize)]
        struct AuthoringPack {
            scenarios: Vec<Scenario>,
        }
        let extra: AuthoringPack =
            serde_json::from_str(include_str!("../../../scenarios/shell-getopts-v1.json")).unwrap();
        a.scenarios.extend(extra.scenarios);
        a.units.push(
            serde_json::from_str(include_str!(
                "../../../tests/fixtures/shell-getopts-unit-proposal.json"
            ))
            .unwrap(),
        );
    }
    assert_eq!(one_off_boundary::before_len(&a.scenarios), 176);
    assert_eq!(one_off_boundary::before_len(&a.units), 46);
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, kind: ScriptLessonFlow) -> Option<String> {
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .rev()
        .find_map(|event| match event {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or_default();
    let mut shot = None;
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
            let (advance, capture) = a.case_lessons_native(ctx, a.smoke_stage, kind);
            if advance {
                a.smoke_stage += 1;
            }
            shot = capture;
        },
    );
    shot
}
fn until(ctx: &egui::Context, a: &mut LabApp, phase: usize) {
    // A span contains multiple12-case jobs. Bound each actual phase, not all
    // of those jobs together; production15s waits and VM limits stay unchanged.
    let mut previous = a.smoke_stage;
    let mut phase_frames = 0;
    let mut transitions = 0;
    loop {
        if a.smoke_stage == phase {
            return;
        }
        assert!(
            phase_frames < 2000 && transitions < 100,
            "native source phase {} did not reach {phase}; job pending: {}",
            a.smoke_stage,
            a.script_job.is_some()
        );
        tick(ctx, a, ScriptLessonFlow::Getopts);
        if a.smoke_stage == previous {
            phase_frames += 1;
        } else {
            transitions += 1;
            previous = a.smoke_stage;
            phase_frames = 0;
        }
    }
}
#[test]
fn getopts_native_route_is_exact_and_three_fixed_plans_fit_the_existing_range() {
    assert_eq!(
        ScriptLessonFlow::from_stage(Some("getopts-lessons")),
        Some(ScriptLessonFlow::Getopts)
    );
    assert_eq!(
        ScriptLessonFlow::from_stage(Some("case-lessons")),
        Some(ScriptLessonFlow::Case)
    );
    for text in [
        None,
        Some("getopts"),
        Some("getopts-lessons-extra"),
        Some("capture-lessons"),
    ] {
        assert_eq!(ScriptLessonFlow::from_stage(text), None);
    }
    let (_, a) = proposed_app();
    for (n, id) in IDS.iter().enumerate() {
        let s = a.scenarios.iter().find(|s| s.id == *id).unwrap();
        assert_eq!(s.unit, "shell-option-interfaces");
        assert!(s.solution[0].len() + 64 < shell::MAX_SOURCE);
        assert!(s.teaching.as_ref().unwrap().example.len() + 64 < shell::MAX_SOURCE);
        assert!(7200 + n * 100 + 79 <= 7499);
    }
}
#[test]
fn getopts_native_actual_library_start_then_example_run_preserves_assignment() {
    let (ctx, mut a) = proposed_app();
    a.smoke = true;
    a.smoke_stage = 7200;
    until(&ctx, &mut a, 7204);
    let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.script_view(i), teaching::View::Example);
    let lesson = a.scenarios[i].teaching.as_ref().unwrap();
    let learning = a.progress.attempts[IDS[0]]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(learning.example_source, lesson.example);
    assert!(learning.example_result.is_none());
    assert_eq!(a.progress.attempts[IDS[0]].state, a.scenarios[i].initial);
    until(&ctx, &mut a, 7206);
    let learning = a.progress.attempts[IDS[0]]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert!(
        a.scenarios[i]
            .teaching
            .as_ref()
            .unwrap()
            .fixture
            .accepts(learning.example_result.as_ref().unwrap())
    );
    assert_eq!(a.progress.attempts[IDS[0]].state, a.scenarios[i].initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
}
#[test]
fn getopts_native_real_editor_partial_reference_edit_retry_and_manual_alternative_all_three() {
    for (n, id) in IDS.iter().enumerate() {
        let (ctx, mut a) = proposed_app();
        let i = a.scenarios.iter().position(|s| s.id == *id).unwrap();
        a.smoke = true;
        a.smoke_stage = 7200 + n * 100;
        // Establish the real Start/practice/repair/Assignment preconditions.
        until(&ctx, &mut a, 7200 + n * 100 + 17);
        assert_eq!(a.selected, Some(i));
        assert!(a.progress.favorites.contains(*id));
        a.smoke_stage = 7200 + n * 100 + 21;
        until(&ctx, &mut a, 7200 + n * 100 + 39);
        assert!(progress::current_solved(
            &a.progress.attempts[*id],
            &a.scenarios[i]
        ));
        let State::Script {
            source,
            run: Some(record),
        } = &a.progress.attempts[*id].state
        else {
            panic!()
        };
        assert_eq!(record.source, *source);
        assert_eq!(record.engine_revision, shell::EVALUATOR_REVISION);
        if n < 2 {
            assert!(
                !source.contains("getopts "),
                "actual manual alternative was typed and run"
            );
        }
    }
}
#[test]
fn getopts_native_actual_unfinished_example_and_assignment_continue_then_reset_cancel_confirm() {
    let (ctx, mut a) = proposed_app();
    let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
    a.launch(i);
    a.set_script_view(i, teaching::View::Assignment);
    a.progress.favorites.insert(IDS[0].into());
    a.smoke = true;
    a.smoke_stage = 7255;
    until(&ctx, &mut a, 7263);
    assert_eq!(a.progress.attempts[IDS[0]].state, a.scenarios[i].initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    assert!(!a.reset_pending);
    assert!(a.progress.favorites.contains(IDS[0]));
}
#[test]
fn getopts_native_and_original_case_finish_hold_until_backend_closes() {
    for kind in [ScriptLessonFlow::Case, ScriptLessonFlow::Getopts] {
        let (ctx, mut a) = proposed_app();
        a.smoke = true;
        a.smoke_stage = 7465;
        for _ in 0..2 {
            assert!(tick(&ctx, &mut a, kind).is_none());
            assert_eq!(a.smoke_stage, 7465);
            assert!(a.pending_events.is_empty());
            assert!(a.script_job.is_none());
        }
    }
}

#[test]
fn case_and_getopts_recheck_full_sources_and_units_on_later_geometry_callbacks() {
    fn callback(
        ctx: &egui::Context,
        a: &mut LabApp,
        kind: ScriptLessonFlow,
    ) -> std::thread::Result<(bool, Option<String>)> {
        let mut result = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                ..Default::default()
            },
            |ctx| {
                // Catch inside the pass so a refused source still ends the
                // actual egui frame cleanly. No learner input or job is run.
                result = Some(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || a.case_lessons_native(ctx, 7241, kind),
                )));
            },
        );
        result.unwrap()
    }
    fn full(p: &progress::Progress) -> serde_json::Value {
        serde_json::json!({"portable":p,"receipts":p.migration_receipts})
    }
    for kind in [ScriptLessonFlow::Case, ScriptLessonFlow::Getopts] {
        let (ctx, mut a) = proposed_app();
        a.smoke = true;
        a.smoke_stage = 7241;
        assert_eq!((a.scenarios.len(), a.units.len()), (199, 53));
        let before = full(&a.progress);
        let sources = a.scenarios.as_ptr();
        let units = a.units.as_ptr();
        let mut settled = false;
        for _ in 0..4 {
            let (advance, shot) = callback(&ctx, &mut a, kind).unwrap();
            settled |= advance;
            assert!(shot.is_none());
            assert_eq!(full(&a.progress), before);
        }
        assert!(settled, "The ordinary geometry phase must actually settle");
        for mutation in 0..4 {
            match mutation {
                0 => a.scenarios[0].objective.push('!'),
                1 => {
                    let engine::Goal::Script { cases } = &mut a.scenarios[197].goal else {
                        panic!()
                    };
                    cases[0].stdout.push('!');
                }
                2 => a.units[0].summary.push('!'),
                _ => a.units[52].labs.push("unreviewed-source".into()),
            }
            assert_eq!(a.scenarios.as_ptr(), sources);
            assert_eq!(a.units.as_ptr(), units);
            let error = callback(&ctx, &mut a, kind)
                .expect_err("Every later callback must reject same-ID/version field corruption");
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("");
            let expected = if mutation < 2 {
                "Complete source differs"
            } else {
                "Complete unit differs"
            };
            assert!(
                message.contains(expected),
                "Real full-field guard required for {kind:?}/{mutation}: {message}"
            );
            assert_eq!(full(&a.progress), before);
            match mutation {
                0 => {
                    a.scenarios[0].objective.pop();
                }
                1 => {
                    let engine::Goal::Script { cases } = &mut a.scenarios[197].goal else {
                        panic!()
                    };
                    cases[0].stdout.pop();
                }
                2 => {
                    a.units[0].summary.pop();
                }
                _ => {
                    a.units[52].labs.pop();
                }
            }
            assert_eq!(callback(&ctx, &mut a, kind).unwrap(), (true, None));
            assert_eq!(full(&a.progress), before);
            assert!(
                a.progress.attempts.is_empty()
                    && a.pending_events.is_empty()
                    && a.script_job.is_none()
            );
        }
    }
}
