//! Prospective real-event regressions; no solved State/Record installation.
use super::*;
const BASE: usize = 37000;
const STRIDE: usize = 200;
const LAST: usize = 104;
const IDS: [&str; 3] = [
    "shell-preserve-lines",
    "shell-pipeline-status",
    "shell-safe-handover",
];
fn native_app() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = BASE;
    assert_eq!((a.scenarios.len(), a.units.len()), (164, 42));
    assert!(a._smoke_profile.is_some());
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, width: f32) -> Option<String> {
    tick_with_output(ctx, a, width).0
}
fn tick_with_output(
    ctx: &egui::Context,
    a: &mut LabApp,
    width: f32,
) -> (Option<String>, egui::FullOutput) {
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| {
            if let egui::Event::Key { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let mut shot = None;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 950.),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            a.save();
            let (advance, capture) = a.safe_automation_lessons_native(ctx, a.smoke_stage);
            if advance {
                a.smoke_stage += 1;
            }
            shot = capture;
        },
    );
    (shot, output)
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, width: f32) -> Vec<String> {
    let mut previous = a.smoke_stage;
    let mut frames = 0;
    let mut transitions = 0;
    let mut shots = Vec::new();
    for _ in 0..30_000 {
        if a.smoke_stage == target {
            return shots;
        }
        assert!(
            frames < 3000 && transitions < 400,
            "Native phase{} did not reach{target}; actual job={}",
            a.smoke_stage,
            a.script_job.is_some()
        );
        if let Some(shot) = tick(ctx, a, width) {
            shots.push(shot);
        }
        if a.smoke_stage == previous {
            frames += 1;
        } else {
            transitions += 1;
            previous = a.smoke_stage;
            frames = 0;
        }
    }
    panic!("Native source controller total frame ceiling")
}
#[test]
fn safe_automation_route_is_smoke_only_and_closed_without_other_stage_overlap() {
    assert_eq!(
        LabApp::safe_automation_stage(true, Some("safe-automation-lessons")),
        Some(BASE)
    );
    for name in [
        None,
        Some("safe-automation"),
        Some("safe-automation-lessons-extra"),
        Some("getopts-lessons"),
        Some("apache-static-lessons"),
    ] {
        assert_eq!(LabApp::safe_automation_stage(true, name), None);
    }
    assert_eq!(
        LabApp::safe_automation_stage(false, Some("safe-automation-lessons")),
        None
    );
    const {
        assert!(
            BASE > 36999 && BASE + 2 * STRIDE + LAST == 37504 && BASE + 2 * STRIDE + LAST < 38000
        );
    }
    let (_, a) = native_app();
    let unit = a
        .units
        .iter()
        .find(|u| u.id == "shell-safe-automation")
        .unwrap();
    assert_eq!(
        unit.labs.iter().map(String::as_str).collect::<Vec<_>>(),
        IDS
    );
    let plans: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shell-safe-automation-native-review.json"
    ))
    .unwrap();
    assert_eq!(plans.as_array().unwrap().len(), 3);
    for (id, plan) in IDS.iter().zip(plans.as_array().unwrap()) {
        assert_eq!(*id, plan["id"].as_str().unwrap());
        assert_eq!(plan["partial"], plan["incorrect"]);
        let s = a.scenarios.iter().find(|s| &s.id == id).unwrap();
        let l = s.teaching.as_ref().unwrap();
        assert_eq!(plan["repair"], l.example);
        for key in ["alternate", "partial", "wrong", "repair"] {
            assert!(plan[key].as_str().unwrap().len() <= shell::MAX_SOURCE);
        }
        assert_ne!(l.example, s.solution[0]);
    }
}
#[test]
fn first_actual_library_start_and_practice_run_experiment_repair_are_isolated_all_three() {
    for (n, id) in IDS.iter().enumerate() {
        let (ctx, mut a) = native_app();
        a.smoke_stage = BASE + n * STRIDE;
        let shots = until(&ctx, &mut a, BASE + n * STRIDE + 17, 1200.);
        let i = a.scenarios.iter().position(|s| &s.id == id).unwrap();
        assert_eq!(a.selected, Some(i));
        assert!(a.progress.favorites.contains(*id));
        assert_eq!(a.script_view(i), teaching::View::Assignment);
        assert_eq!(a.progress.attempts[*id].state, a.scenarios[i].initial);
        assert!(!progress::current_solved(
            &a.progress.attempts[*id],
            &a.scenarios[i]
        ));
        let l = a.progress.attempts[*id]
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
                .accepts(l.example_result.as_ref().unwrap())
        );
        for label in [
            "introduction",
            "separate-example-ready-not-run",
            "ungraded-example-run",
            "ungraded-example-experiment",
            "ungraded-example-repaired",
            "full-assignment",
        ] {
            assert!(
                shots.iter().any(|s| s == &format!("{id}-{label}")),
                "Capture-phase {label} not actually exercised"
            );
        }
    }
}
#[test]
fn real_wrong_partial_reference_current_edit_and_genuine_alternate_run_for_each() {
    for (n, id) in IDS.iter().enumerate() {
        let (ctx, mut a) = native_app();
        a.smoke_stage = BASE + n * STRIDE;
        let shots = until(&ctx, &mut a, BASE + n * STRIDE + 39, 1200.);
        let i = a.selected.unwrap();
        assert_eq!(a.scenarios[i].id, *id);
        assert!(progress::current_solved(
            &a.progress.attempts[*id],
            &a.scenarios[i]
        ));
        assert!(
            shots
                .iter()
                .any(|s| s.ends_with("hardcoded-wrong-feedback"))
        );
        assert!(
            shots
                .iter()
                .any(|s| s.ends_with("partial-misconception-feedback"))
        );
        assert!(
            shots
                .iter()
                .any(|s| s.ends_with("genuine-alternate-complete"))
        );
    }
}
#[test]
fn real_run_stop_syntax_unsupported_limit_and_retry_preserve_current_truth() {
    let (ctx, mut a) = native_app();
    until(&ctx, &mut a, BASE + 17, 1200.);
    a.smoke_stage = BASE + 55;
    let shots = until(&ctx, &mut a, BASE + 77, 1200.);
    let i = a.selected.unwrap();
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    assert!(a.script_job.is_none());
    for label in [
        "syntax-feedback",
        "unsupported-feedback",
        "real-run-stopped-without-credit",
        "bounded-resource-feedback",
        "after-stop-limit-repaired",
    ] {
        assert!(
            shots.iter().any(|s| s.ends_with(label)),
            "Actual event/capture stage {label} missing"
        );
    }
}
#[test]
fn actual_large_scale_focus_typing_and_run_use_real_geometry_at_two_widths() {
    for width in [760., 1280.] {
        let (ctx, mut a) = native_app();
        until(&ctx, &mut a, BASE + 39, width);
        let shots = until(&ctx, &mut a, BASE + 55, width);
        let i = a.selected.unwrap();
        assert!(a.progress.attempts[IDS[0]].challenge);
        assert!(progress::current_solved(
            &a.progress.attempts[IDS[0]],
            &a.scenarios[i]
        ));
        assert!((a.scale - 2.).abs() < 0.01);
        for label in [
            "light150-focused-input",
            "light150-typed-and-run",
            "dark200-focused-input",
            "dark200-typed-and-run",
        ] {
            assert!(shots.iter().any(|s| s.ends_with(label)));
        }
    }
}
#[test]
fn full_actual_flow_warm_raw_receipts_continue_reset_and_asynchronous_finish_hold() {
    let (ctx, mut a) = native_app();
    let shots = until(&ctx, &mut a, BASE + 2 * STRIDE + LAST, 1200.);
    assert!(a.selected.is_none());
    for id in IDS {
        assert!(a.progress.favorites.contains(id));
        for label in [
            "separate-unfinished-drafts-saved",
            "real-continue-retains-both-drafts",
            "reset-cancel-keeps-drafts",
            "normal-three-lesson-unit-after-reset",
        ] {
            assert!(shots.iter().any(|s| s == &format!("{id}-{label}")));
        }
    }
    let before = serde_json::to_vec(&a.progress.attempts).unwrap();
    for _ in 0..2 {
        assert!(tick(&ctx, &mut a, 1200.).is_none());
        assert_eq!(a.smoke_stage, BASE + 2 * STRIDE + LAST);
        assert!(serde_json::to_vec(&a.progress.attempts).unwrap() == before);
    }
    let raw = progress::read_bounded(&a.path, 16_000_000).unwrap();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_vec(&loaded.attempts).unwrap() == before);
    assert!(std::fs::read(&a.path).unwrap() == raw);
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![shell::EVALUATOR_REVISION]
    );
}

#[test]
fn first_wrong_capture_really_scrolls_inner_output_and_paints_full_expected_actual_pair() {
    let (ctx, mut a) = native_app();
    until(&ctx, &mut a, BASE + 20, 950.);
    // Phase 20 first waits for the real asynchronous Run. Reaching its entry
    // does not mean the full multi-case Record has already been published.
    for _ in 0..1000 {
        if a.script_job.is_none() {
            break;
        }
        assert!(tick(&ctx, &mut a, 950.).is_none());
        assert_eq!(a.smoke_stage, BASE + 20);
    }
    assert!(a.script_job.is_none(), "Bounded actual Run must finish");
    let i = a.selected.unwrap();
    let case = a.script_case;
    let engine::Goal::Script { cases } = &a.scenarios[i].goal else {
        panic!()
    };
    let engine::State::Script {
        run: Some(record), ..
    } = &a.progress.attempts[IDS[0]].state
    else {
        panic!()
    };
    let expected_pair = format!(
        "Expected: {:?}\nActual:   {:?}",
        cases[case].stdout, record.results[case].stdout
    );
    assert!(!cases[case].accepts(&record.results[case]));
    let before = serde_json::to_vec(&a.progress.attempts).unwrap();
    let mut wheel = false;
    let mut captured = false;
    for _ in 0..100 {
        let point = a.pending_events.iter().find_map(|e| {
            if let egui::Event::PointerMoved(point) = e {
                Some(*point)
            } else {
                None
            }
        });
        if a.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { delta, .. } if delta.y < 0.))
        {
            assert!(
                a.metrics
                    .output
                    .unwrap()
                    .shrink(12.)
                    .contains(point.unwrap())
            );
            wheel = true;
        }
        let (shot, output) = tick_with_output(&ctx, &mut a, 950.);
        if let Some(shot) = shot {
            assert_eq!(shot, "shell-preserve-lines-hardcoded-wrong-feedback");
            fn fully_painted(
                shape: &egui::Shape,
                clip: egui::Rect,
                expected: &str,
                tolerance: f32,
            ) -> bool {
                match shape {
                    egui::Shape::Text(text) => {
                        text.galley.text() == expected
                            && clip
                                .expand(tolerance)
                                .contains_rect(text.visual_bounding_rect())
                    }
                    egui::Shape::Vec(shapes) => shapes
                        .iter()
                        .any(|s| fully_painted(s, clip, expected, tolerance)),
                    _ => false,
                }
            }
            assert!(
                output.shapes.iter().any(|s| fully_painted(
                    &s.shape,
                    s.clip_rect,
                    &expected_pair,
                    1. / ctx.pixels_per_point()
                )),
                "Capture must show the whole measured Expected/Actual pair inside its actual paint clip"
            );
            captured = true;
            break;
        }
        assert_eq!(
            a.smoke_stage,
            BASE + 20,
            "No capture-phase advance before evidence"
        );
    }
    assert!(
        wheel && captured,
        "A real inner-wheel frame must precede the actual capture"
    );
    assert!(
        serde_json::to_vec(&a.progress.attempts).unwrap() == before,
        "Scrolling changes no Attempt field"
    );
    assert_eq!(a.smoke_stage, BASE + 21);
}

#[test]
fn actual_smoke_pacer_acknowledges_run_then_clicks_stop_before_waiting() {
    for (n, id) in IDS.iter().enumerate() {
        let (ctx, mut a) = native_app();
        let start = BASE + n * STRIDE;
        a.smoke_stage = start;
        until(&ctx, &mut a, start + 66, 1200.);
        let mut observed_running = false;
        for _ in 0..16 {
            let events = std::mem::take(&mut a.pending_events);
            let modifiers = events
                .iter()
                .rev()
                .find_map(|event| {
                    if let egui::Event::Key { modifiers, .. } = event {
                        Some(*modifiers)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1200., 950.),
                    )),
                    events,
                    modifiers,
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    a.save();
                    observed_running |= a.script_job.is_some();
                    // Keep ordinary pacing closed. The real Run/Stop phases must
                    // proceed through smoke_tick without an elapsed-time escape.
                    a.started = Instant::now();
                    a.smoke_tick(ctx);
                },
            );
            if a.smoke_stage == start + 69 {
                break;
            }
        }
        assert_eq!(
            a.smoke_stage,
            start + 69,
            "Run acknowledgement and Stop must use consecutive actual frames"
        );
        assert!(
            observed_running,
            "The real Run button must create the job before Stop"
        );
        assert!(a.script_job.is_none());
        assert!(
            a.notice
                .as_deref()
                .is_some_and(|s| s.starts_with("Stopped"))
        );
        assert!(
            matches!(&a.progress.attempts[*id].state,engine::State::Script{source,run:None} if source=="while true; do :; done\n")
        );
        assert!(!progress::current_solved(
            &a.progress.attempts[*id],
            &a.scenarios[a.selected.unwrap()]
        ));
    }
    for offset in [65, 69] {
        let (ctx, mut a) = native_app();
        a.smoke_stage = BASE + offset;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            a.started = Instant::now();
            a.smoke_tick(ctx);
        });
        assert_eq!(
            a.smoke_stage,
            BASE + offset,
            "Neighboring phases retain normal pacing"
        );
        assert!(a.pending_events.is_empty() && !a.capture_pending);
    }
}
