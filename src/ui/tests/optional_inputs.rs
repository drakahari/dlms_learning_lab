//! Prospective real-event regressions; no solved State/Record installation.
use super::*;
const BASE: usize = 64000;
const STRIDE: usize = 250;
const LAST: usize = 176;
const IDS: [&str; 3] = [
    "shell-optional-policy",
    "shell-optional-report",
    "shell-optional-handover",
];
fn native_app() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = BASE;
    // Prospective declarative fixture only; no Attempt, draft, result or credit is injected.
    // Normal registration remains a later source grant after content/runtime gates.
    let old = serde_json::to_value(&a.scenarios).unwrap();
    let added: pack::Pack = serde_json::from_str(include_str!(
        "../../../scenarios/shell-optional-inputs-v1.json"
    ))
    .unwrap();
    let present = a
        .scenarios
        .iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .count();
    assert!(
        present == 0 || present == 3,
        "Closed unregistered/registered boundary"
    );
    if present == 0 {
        let prefix = a.scenarios.len();
        a.scenarios.extend(added.scenarios);
        a.scenarios = pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: std::mem::take(&mut a.scenarios),
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(serde_json::to_value(&a.scenarios[..prefix]).unwrap(), old);
        let review: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-optional-inputs-review.json"
        ))
        .unwrap();
        a.units
            .push(serde_json::from_value(review["unit_proposal"].clone()).unwrap());
        a.units = dlms_learning_lab::course::load(
            &serde_json::to_string(&serde_json::json!({"version":1,"units":a.units})).unwrap(),
            &a.scenarios,
        )
        .unwrap();
    }
    assert!(a.progress.attempts.is_empty());
    assert!(a._smoke_profile.is_some());
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, width: f32) -> Option<String> {
    tick_with_output(ctx, a, width).0
}
#[derive(Clone, Default)]
struct WindowProvider {
    fullscreen: bool,
    requests: Vec<(usize, bool)>,
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
    let stage_before = a.smoke_stage;
    let phase = (stage_before - BASE) % STRIDE;
    let provider_key = egui::Id::new("optional-observed-window-provider");
    let mut provider = ctx
        .data(|d| d.get_temp::<WindowProvider>(provider_key))
        .unwrap_or_default();
    let (width, height) = if (105..=176).contains(&phase) {
        if provider.fullscreen {
            (1920., 1080.)
        } else {
            (850., 850.)
        }
    } else {
        (width, 950.)
    };
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / a.scale, height / a.scale),
        )),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .fullscreen = Some(provider.fullscreen);
    let output = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        let (advance, capture) = a.optional_inputs_lessons_native(ctx, a.smoke_stage);
        if advance {
            a.smoke_stage += 1;
        }
        shot = capture;
    });
    for viewport in output.viewport_output.values() {
        for command in &viewport.commands {
            if let egui::ViewportCommand::Fullscreen(value) = command {
                provider.fullscreen = *value;
                provider.requests.push((stage_before, *value));
            }
        }
    }
    ctx.data_mut(|d| d.insert_temp(provider_key, provider));
    if shot
        .as_deref()
        .is_some_and(|name| name.contains("-goal200-") && name.ends_with("-bottom"))
    {
        // Inspect the actual captured frame, independently of the controller's
        // row-union cache. The old narrow endpoint passed clip containment but
        // left the final line behind a floating horizontal scrollbar.
        let objective = &a.scenarios[a.selected.unwrap()].objective;
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        fn rows(shape: &egui::Shape, objective: &str) -> Option<Vec<egui::Rect>> {
            match shape {
                egui::Shape::Text(text) if text.galley.text() == objective => Some(
                    text.galley
                        .rows
                        .iter()
                        .filter(|row| !row.text().trim().is_empty())
                        .map(|row| row.rect().translate(text.pos.to_vec2()))
                        .collect(),
                ),
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| rows(s, objective)),
                _ => None,
            }
        }
        let (actual_rows, clip) = output
            .shapes
            .iter()
            .find_map(|shape| {
                rows(&shape.shape, objective)
                    .map(|rows| (rows, shape.clip_rect.intersect(viewport)))
            })
            .expect("The actual bottom capture must paint its task galley");
        let tolerance = 1. / ctx.pixels_per_point();
        let fully_visible = actual_rows
            .iter()
            .all(|row| clip.expand(tolerance).contains_rect(*row));
        let last = *actual_rows.last().unwrap();
        assert!(clip.expand(tolerance).contains_rect(last));
        assert!(
            last.bottom() + last.height() <= clip.bottom() + tolerance,
            "The actual final row needs a full painted text-row clearance above edge scrollbars"
        );
        if !fully_visible {
            assert!(
                (last.center().y - clip.center().y).abs() <= tolerance,
                "The long-task final row must be centered, not merely inside its clip edge"
            );
            assert!(
                last.height() * 2. < clip.height(),
                "The final row needs observed clearance from edge scrollbar overpaint"
            );
        }
    }
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
            frames < 3000 && transitions < 540,
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
fn optional_inputs_route_is_smoke_only_and_closed_without_other_stage_overlap() {
    assert_eq!(
        LabApp::optional_inputs_stage(true, Some("optional-inputs-lessons")),
        Some(BASE)
    );
    for name in [
        None,
        Some("optional-inputs"),
        Some("optional-inputs-lessons-extra"),
        Some("getopts-lessons"),
        Some("apache-static-lessons"),
    ] {
        assert_eq!(LabApp::optional_inputs_stage(true, name), None);
    }
    assert_eq!(
        LabApp::optional_inputs_stage(false, Some("optional-inputs-lessons")),
        None
    );
    const {
        assert!(
            BASE > 63999 && BASE + 2 * STRIDE + LAST == 64676 && BASE + 2 * STRIDE + LAST < 64680
        );
    }
    let (_, a) = native_app();
    let unit = a
        .units
        .iter()
        .find(|u| u.id == "shell-optional-inputs")
        .unwrap();
    assert_eq!(
        unit.labs.iter().map(String::as_str).collect::<Vec<_>>(),
        IDS
    );
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shell-optional-inputs-native-review.json"
    ))
    .unwrap();
    let plans = value["plans"].as_array().unwrap();
    assert_eq!(plans.len(), 3);
    for (id, plan) in IDS.iter().zip(plans) {
        assert_eq!(*id, plan["id"].as_str().unwrap());
        let source = a.scenarios.iter().find(|s| &s.id == id).unwrap();
        assert_eq!(plan["repair"], source.teaching.as_ref().unwrap().example);
        assert_eq!(plan["alternates"].as_array().unwrap().len(), 2);
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
fn twenty_real_editor_settings_per_lab_cover_every_scale_theme_and_width() {
    for (n, id) in IDS.iter().enumerate() {
        let (ctx, mut a) = native_app();
        a.smoke_stage = BASE + n * STRIDE;
        until(&ctx, &mut a, BASE + n * STRIDE + 39, 1200.);
        let shots = until(&ctx, &mut a, BASE + n * STRIDE + 55, 1200.);
        let i = a.selected.unwrap();
        assert_eq!(a.scenarios[i].id, *id);
        assert!(a.progress.attempts[*id].challenge);
        assert!(!progress::current_solved(
            &a.progress.attempts[*id],
            &a.scenarios[i]
        ));
        let layouts = shots
            .iter()
            .filter(|s| s.contains("-layout-"))
            .collect::<Vec<_>>();
        assert_eq!(layouts.len(), 20);
        let unique = layouts.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 20);
        let provider = ctx
            .data(|d| {
                d.get_temp::<WindowProvider>(egui::Id::new("optional-observed-window-provider"))
            })
            .unwrap();
        for k in 0usize..20 {
            let stage = BASE + n * STRIDE + 105 + 3 * k;
            assert_eq!(
                provider
                    .requests
                    .iter()
                    .filter(|(s, _)| *s == stage)
                    .map(|(_, flag)| *flag)
                    .collect::<Vec<_>>(),
                vec![k.is_multiple_of(2)],
                "Actual emitted fullscreen/windowed branch for every settled setting"
            );
        }
        if n == 2 {
            for setting in 0usize..4 {
                let stage = BASE + n * STRIDE + 165 + 3 * setting;
                assert_eq!(
                    provider
                        .requests
                        .iter()
                        .filter(|(s, _)| *s == stage)
                        .map(|(_, flag)| *flag)
                        .collect::<Vec<_>>(),
                    vec![setting.is_multiple_of(2)],
                    "Actual goal-slice viewport branch"
                );
            }
        }
        for scale in [100, 115, 125, 150, 200] {
            for theme in ["light", "dark"] {
                for width in ["wide", "narrow"] {
                    assert!(
                        layouts
                            .iter()
                            .any(|s| s.ends_with(&format!("layout-{scale}-{theme}-{width}")))
                    );
                }
            }
        }
    }
}
#[test]
fn full_actual_flow_warm_raw_receipts_continue_reset_and_asynchronous_finish_hold() {
    let (ctx, mut a) = native_app();
    let shots = until(&ctx, &mut a, BASE + 2 * STRIDE + 104, 1200.);
    assert!(a.selected.is_none());
    assert_eq!(shots.iter().filter(|s| s.contains("-layout-")).count(), 60);
    assert!(shots.len() <= 136);
    let goals = shots
        .iter()
        .filter(|s| s.contains("-goal200-"))
        .collect::<Vec<_>>();
    assert_eq!(goals.len(), 12);
    for theme in ["light", "dark"] {
        for width in ["wide", "narrow"] {
            for slice in ["top", "middle", "bottom"] {
                assert!(
                    goals
                        .iter()
                        .any(|s| s.ends_with(&format!("goal200-{theme}-{width}-{slice}")))
                );
            }
        }
    }
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
    a.capture_pending = true;
    for _ in 0..2 {
        let (_, output) = tick_with_output(&ctx, &mut a, 1200.);
        assert!(!output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }));
        assert_eq!(a.smoke_stage, BASE + 2 * STRIDE + 104);
    }
    a.capture_pending = false;
    for _ in 0..2 {
        assert!(tick(&ctx, &mut a, 1200.).is_none());
        assert_eq!(a.smoke_stage, BASE + 2 * STRIDE + 104);
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
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 950.),
                )),
                events,
                modifiers,
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .fullscreen = Some(false);
            let _ = ctx.run(input, |ctx| {
                a.render(ctx);
                a.save();
                observed_running |= a.script_job.is_some();
                // Keep ordinary pacing closed. The real Run/Stop phases must
                // proceed through smoke_tick without an elapsed-time escape.
                a.started = Instant::now();
                a.smoke_tick(ctx);
            });
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

#[test]
fn optional_inputs_immediate_stop_and_capture_budgets_are_scoped() {
    for n in 0..3 {
        for phase in [66, 67, 68] {
            assert!(super::super::smoke::optional_inputs::immediate_stop(
                BASE + n * STRIDE + phase
            ));
        }
    }
    for stage in [0, 37066, 43066, BASE - 1, BASE + 65, BASE + 69, 64700] {
        assert!(!super::super::smoke::optional_inputs::immediate_stop(stage));
    }
}

#[test]
fn capstone_essential_assignment_rows_remain_reachable_with_real_bounded_scrolling_at200() {
    fn visible_rows(
        shape: &egui::Shape,
        clip: egui::Rect,
        task: &str,
        seen: &mut std::collections::BTreeSet<usize>,
        total: &mut Option<usize>,
    ) {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == task => {
                let rows = text
                    .galley
                    .rows
                    .iter()
                    .filter(|r| !r.text().trim().is_empty())
                    .count();
                if let Some(previous) = total {
                    assert_eq!(*previous, rows);
                } else {
                    *total = Some(rows);
                }
                for (i, row) in text
                    .galley
                    .rows
                    .iter()
                    .filter(|r| !r.text().trim().is_empty())
                    .enumerate()
                {
                    let rect = row.rect().translate(text.pos.to_vec2());
                    if clip.expand(0.5).contains_rect(rect) {
                        seen.insert(i);
                    }
                }
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    visible_rows(s, clip, task, seen, total);
                }
            }
            _ => {}
        }
    }
    for width in [850., 1280.] {
        for theme_id in ["dlms-light", "dlms-dark"] {
            let (ctx, mut a) = native_app();
            a.smoke_stage = BASE + 2 * STRIDE;
            until(&ctx, &mut a, BASE + 2 * STRIDE + 17, width);
            a.scale = 2.;
            a.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            a.lesson_scroll_reset = true;
            let id = IDS[2];
            let i = a.selected.unwrap();
            let task = a.scenarios[i].objective.clone();
            let before =
                serde_json::to_value((&a.progress.attempts, &a.progress.migration_receipts))
                    .unwrap();
            let mut total = None;
            let mut seen = std::collections::BTreeSet::new();
            // Twenty real wheel pulses, with ordinary settling frames. No ScrollArea state writes.
            for pulse in 0..=20 {
                for settle in 0..8 {
                    let mut events = vec![];
                    if pulse > 0 && settle == 0 {
                        let clip = a.metrics.main_viewport.unwrap();
                        events.push(egui::Event::PointerMoved(egui::pos2(
                            clip.left() + 2.,
                            clip.center().y,
                        )));
                        events.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., -100.),
                            modifiers: Default::default(),
                        });
                    }
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / a.scale, 850. / a.scale),
                            )),
                            events,
                            ..Default::default()
                        },
                        |ctx| a.render(ctx),
                    );
                    for shape in out.shapes {
                        visible_rows(&shape.shape, shape.clip_rect, &task, &mut seen, &mut total);
                    }
                }
                if total.is_some_and(|n| n == seen.len()) {
                    break;
                }
            }
            assert_eq!(
                Some(seen.len()),
                total,
                "Every essential task row must actually paint inside a clip at200% {width}/{theme_id}"
            );
            assert!(total.is_some_and(|n| n > 0));
            assert_eq!(
                serde_json::to_value((&a.progress.attempts, &a.progress.migration_receipts))
                    .unwrap(),
                before
            );
            assert_eq!(a.scenarios[i].objective, task);
            assert!(!progress::current_solved(
                &a.progress.attempts[id],
                &a.scenarios[i]
            ));
        }
    }
}
