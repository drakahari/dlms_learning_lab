//! Actual egui editor/dropdown/key/button events. No native-window, novice or
//! screen-reader acceptance is implied by these component tests.
use super::*;
use dlms_learning_lab::{cron_workspace as work, script_teaching::View};
pub(in crate::ui) fn sources() -> Vec<Scenario> {
    crate::ui::smoke::cron_lessons::fixtures()
}
fn opened(n: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!(
        (
            one_off_boundary::before_len(&a.scenarios),
            one_off_boundary::before_len(&a.units)
        ),
        (176, 46)
    );
    let at = 160;
    assert_eq!(
        a.scenarios[at..164]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        crate::ui::smoke::cron_lessons::IDS
    );
    assert_eq!(
        a.scenarios[164..168]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic2-backup-selective-recovery",
            "lpic2-backup-incremental-chain",
            "lpic2-backup-snapshot-branch",
            "lpic2-backup-recovery-handover"
        ],
        "The four canonical backup assignments retain their exact positions"
    );
    assert_eq!(
        a.scenarios[168..171]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-numeric-retry",
            "shell-numeric-intake",
            "shell-numeric-publication"
        ],
        "The entire remaining tail contains exactly the three numeric assignments"
    );
    a.launch(at + n);
    draw(&ctx, &mut a);
    (ctx, a, at + n)
}
fn attempt(a: &LabApp) -> &Attempt {
    &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
}
fn snapshot(a: &LabApp) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::to_value(&a.progress).unwrap(),
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
    )
}
fn session(a: &LabApp, practice: bool) -> &work::Session {
    let w = attempt(a).workspace.as_ref().unwrap();
    if practice {
        &w.cron_learning.as_ref().unwrap().session
    } else {
        w.cron_session.as_ref().unwrap()
    }
}
fn live(a: &LabApp, practice: bool) -> &dlms_learning_lab::cron::State {
    if practice {
        &attempt(a)
            .workspace
            .as_ref()
            .unwrap()
            .cron_learning
            .as_ref()
            .unwrap()
            .simulation
    } else {
        let State::Cron { simulation } = &attempt(a).state else {
            unreachable!()
        };
        simulation
    }
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    let mut trace = Vec::new();
    let mut stable = None;
    for _ in 0..20 {
        let (r, clip) = if field {
            let f = a
                .metrics
                .cron
                .fields
                .iter()
                .find(|f| f.0 == key)
                .expect("Actual Cron field");
            (f.1, f.4)
        } else {
            let c = a
                .metrics
                .cron
                .controls
                .iter()
                .find(|c| c.0 == key)
                .expect("Actual Cron control");
            (c.1, c.3)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let visible = root.intersect(clip);
        if smoke::smoke_control_visible(r, visible, ctx.pixels_per_point()) {
            if stable == Some((r, visible)) {
                return r;
            }
            stable = Some((r, visible));
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            continue;
        }
        stable = None;
        if trace.len() == 3 {
            trace.remove(0);
        }
        trace.push((r, visible));
        let dy =
            (root.center().y - r.center().y).clamp(-root.height() * 0.75, root.height() * 0.75);
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(
                    root.left() + 1. / ctx.pixels_per_point(),
                    root.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Could not reveal {key} within20 actual wheel events: {trace:?}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false);
    click(ctx, a, r);
}
fn command(key: egui::Key) -> Vec<egui::Event> {
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
    let r = reveal(ctx, a, key, true);
    click(ctx, a, r);
    let id = a.metrics.cron.fields.iter().find(|f| f.0 == key).unwrap().2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let mut events = command(egui::Key::A);
    events.push(egui::Event::Paste(text.into()));
    frame(ctx, a, 1200., 850., events);
    draw(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str, practice: bool) {
    tap(ctx, a, key);
    let wanted = format!("{key}:{value}");
    let mut trace = Vec::new();
    for _ in 0..20 {
        let r = a
            .metrics
            .cron
            .choices
            .iter()
            .find(|c| c.0 == wanted)
            .expect("Actual Cron popup option")
            .1;
        let clip = a
            .metrics
            .cron
            .popup
            .as_ref()
            .unwrap()
            .1
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            click(ctx, a, r);
            match key {
                "actor" => assert_eq!(session(a, practice).actor, value),
                "target" => assert_eq!(session(a, practice).selected.key(), value),
                _ => {}
            }
            return;
        }
        if trace.len() == 3 {
            trace.remove(0);
        }
        trace.push((r, clip));
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., clip.center().y - r.center().y),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Popup option {wanted} not fully visible: {trace:?}")
}
fn assignment(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, "assignment");
    draw(ctx, a);
    assert_eq!(view(a), View::Assignment);
}
fn view(a: &LabApp) -> View {
    attempt(a)
        .workspace
        .as_ref()
        .unwrap()
        .cron_learning
        .as_ref()
        .unwrap()
        .view
}
fn apply(ctx: &egui::Context, a: &mut LabApp, action: &work::ReferenceAction, practice: bool) {
    match action {
        work::ReferenceAction::Install {
            actor,
            target,
            text,
        } => {
            choose(ctx, a, "actor", actor, practice);
            choose(ctx, a, "target", &target.key(), practice);
            type_field(ctx, a, "document", text);
            tap(ctx, a, "install");
            assert_eq!(session(a, practice).drafts[&target.key()], *text);
        }
        work::ReferenceAction::Remove { actor, target } => {
            choose(ctx, a, "actor", actor, practice);
            choose(ctx, a, "target", &target.key(), practice);
            tap(ctx, a, "remove");
            assert!(!target.exists(live(a, practice)));
        }
        work::ReferenceAction::Run { end } => {
            type_field(ctx, a, "end", &end.text());
            tap(ctx, a, "run");
            for _ in 0..1000 {
                if !a.cron_presentation.running() {
                    break;
                }
                draw(ctx, a);
            }
            assert!(!a.cron_presentation.running());
            assert_eq!(live(a, practice).clock, *end);
            assert!(session(a, practice).last_run.is_some());
        }
    }
}
fn check_work(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, "check");
    for _ in 0..1000 {
        if !a
            .cron_presentation
            .memo
            .pending(attempt(a), &a.scenarios[a.selected.unwrap()])
        {
            break;
        }
        draw(ctx, a);
    }
    draw(ctx, a);
}
#[test]
fn cron_intro_example_and_assignment_keep_real_work_separate() {
    for n in 0..4 {
        let (ctx, mut a, _) = opened(n);
        let initial = snapshot(&a);
        let assignment_state = attempt(&a).state.clone();
        let assignment_draft = session(&a, false).clone();
        assert_eq!(view(&a), View::Introduction);
        assert!(a.metrics.cron.controls.iter().any(|c| c.0 == "try-example"));
        tap(&ctx, &mut a, "try-example");
        assert_eq!(view(&a), View::Example);
        let before = live(&a, true).clone();
        tap(&ctx, &mut a, "fill");
        assert_eq!(live(&a, true), &before);
        assert_eq!(attempt(&a).state, assignment_state);
        assert_eq!(session(&a, false), &assignment_draft);
        assert!(!a.metrics.completion);
        tap(&ctx, &mut a, "assignment");
        assert_eq!(view(&a), View::Assignment);
        assert_eq!(attempt(&a).state, assignment_state);
        assert_eq!(session(&a, false), &assignment_draft);
        assert_eq!(a.progress.clock, initial.0["clock"].as_u64().unwrap());
        assert_eq!(
            serde_json::to_value(&a.progress.migration_receipts).unwrap(),
            initial.1
        );
    }
}
#[test]
fn cron_actual_editor_install_run_and_check_pass_all_four_full_assignments() {
    let review: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/cron-lessons-review-v1.json"
    ))
    .unwrap();
    for n in 0..4 {
        let (ctx, mut a, i) = opened(n);
        assignment(&ctx, &mut a);
        for action in
            serde_json::from_value::<Vec<work::ReferenceAction>>(review[n]["reference"].clone())
                .unwrap()
        {
            apply(&ctx, &mut a, &action, false);
        }
        check_work(&ctx, &mut a);
        assert!(
            a.metrics.completion,
            "{} actual native widgets",
            a.scenarios[i].id
        );
        assert!(progress::current_solved(attempt(&a), &a.scenarios[i]));
        assert_eq!(attempt(&a).step, 0);
        let log = session(&a, false).last_run.as_ref().unwrap();
        assert!(log.events.iter().all(|e| e.status == 0));
        assert!(!log.events.is_empty());
        // A changed unsaved draft is real saved input, not a different State.
        let state = attempt(&a).state.clone();
        type_field(&ctx, &mut a, "document", "* * * * * echo wrong\n");
        assert_eq!(attempt(&a).state, state);
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(attempt(&a), &a.scenarios[i]));
    }
}
#[test]
fn cron_long_window_stops_without_publishing_private_clock_files_or_history() {
    let (ctx, mut a, _) = opened(0);
    assignment(&ctx, &mut a);
    let end = live(&a, false).clock.after(7 * 24 * 60).unwrap();
    type_field(&ctx, &mut a, "end", &end.text());
    let before = snapshot(&a);
    tap(&ctx, &mut a, "run");
    assert!(a.cron_presentation.running());
    tap(&ctx, &mut a, "stop");
    assert!(!a.cron_presentation.running());
    assert_eq!(snapshot(&a), before);
    assert!(session(&a, false).last_run.is_none());
}
#[test]
fn cron_input_rejection_keeps_whole_progress_and_does_not_dispatch_check() {
    let (ctx, mut a, _) = opened(0);
    assignment(&ctx, &mut a);
    let id = a.scenarios[a.selected.unwrap()].id.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    attempt.transcript = vec![String::new()];
    let n = serde_json::to_vec(attempt).unwrap().len();
    attempt.transcript[0] = "x".repeat(work::MAX_ATTEMPT - n);
    assert_eq!(
        serde_json::to_vec(attempt).unwrap().len(),
        work::MAX_ATTEMPT
    );
    draw(&ctx, &mut a);
    let r = reveal(&ctx, &mut a, "document", true);
    click(&ctx, &mut a, r);
    let before = snapshot(&a);
    let mut events = command(egui::Key::A);
    events.push(egui::Event::Paste("* * * * * echo new\n".into()));
    events.extend(command(egui::Key::Enter));
    frame(&ctx, &mut a, 1200., 850., events);
    draw(&ctx, &mut a);
    assert_eq!(snapshot(&a), before);
    assert!(!a.cron_presentation.observed_checked(false));
    assert!(!a.cron_presentation.running());
    assert!(
        a.cron_presentation
            .notice()
            .is_some_and(|s| s.contains("retained"))
    );
}
#[test]
fn cron_native_undo_tab_and_resize_retain_editing_and_current_draft() {
    let (ctx, mut a, _) = opened(0);
    assignment(&ctx, &mut a);
    type_field(
        &ctx,
        &mut a,
        "document",
        "10 9 * * MON-FRI cat /work/source > /work/report\n",
    );
    let saved = session(&a, false).clone();
    let id = a
        .metrics
        .cron
        .fields
        .iter()
        .find(|f| f.0 == "document")
        .unwrap()
        .2;
    let stroke = |shift| {
        let down = key(egui::Key::Tab, shift);
        let egui::Event::Key {
            key,
            physical_key,
            repeat,
            modifiers,
            ..
        } = down.clone()
        else {
            unreachable!()
        };
        vec![
            down,
            egui::Event::Key {
                key,
                physical_key,
                pressed: false,
                repeat,
                modifiers,
            },
        ]
    };
    frame(&ctx, &mut a, 1200., 850., stroke(false));
    draw(&ctx, &mut a);
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    frame(&ctx, &mut a, 1200., 850., stroke(true));
    draw(&ctx, &mut a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    for scale in [1.15, 1.25, 1.5, 2., 1.] {
        a.scale = scale;
        frame(&ctx, &mut a, 946., 929., vec![]);
        assert_eq!(session(&a, false), &saved);
    }
    let r = reveal(&ctx, &mut a, "document", true);
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., command(egui::Key::Z));
    draw(&ctx, &mut a);
    assert_eq!(session(&a, false).drafts["user:alex"], "");
    frame(&ctx, &mut a, 1200., 850., command(egui::Key::Y));
    draw(&ctx, &mut a);
    assert_eq!(session(&a, false), &saved);
}

#[test]
fn cron_actual_retry_restores_only_supplied_files_and_clock_in_both_modes_and_scopes() {
    for challenge in [false, true] {
        for practice in [false, true] {
            if challenge && practice {
                continue;
            } // Challenge intentionally has no worked example.
            let (ctx, mut a, i) = opened(0);
            if practice {
                tap(&ctx, &mut a, "try-example");
            } else {
                assignment(&ctx, &mut a);
            }
            if challenge {
                let r = a.metrics.mode_challenge.unwrap();
                click(&ctx, &mut a, r);
                draw(&ctx, &mut a);
            }
            let owner = if practice { "mira" } else { "alex" };
            let source = if practice {
                "/work/example-source"
            } else {
                "/work/source"
            };
            let target = work::Document::User {
                owner: owner.into(),
            };
            apply(
                &ctx,
                &mut a,
                &work::ReferenceAction::Install {
                    actor: owner.into(),
                    target,
                    text: format!("* * * * * cat {source} > /work/stray\n"),
                },
                practice,
            );
            let end = live(&a, practice).clock.next().unwrap();
            apply(&ctx, &mut a, &work::ReferenceAction::Run { end }, practice);
            assert!(live(&a, practice).files.contains_key("/work/stray"));
            assert!(session(&a, practice).last_run.is_some());
            let before = snapshot(&a);
            a.save();
            assert!(a.storage_error.is_none());
            let disk = progress::load(&a.path, &a.scenarios).unwrap();
            assert_eq!(
                serde_json::to_value(&disk).unwrap(),
                before.0,
                "Actual normal readback preserves run output and both drafts without installation"
            );
            let id = a.scenarios[i].id.clone();
            let initial = if practice {
                &a.scenarios[i]
                    .cron_teaching
                    .as_ref()
                    .unwrap()
                    .definition
                    .initial
            } else {
                let engine::Goal::Cron { definition } = &a.scenarios[i].goal else {
                    unreachable!()
                };
                &definition.initial
            };
            let initial_files = initial.files.clone();
            let initial_clock = initial.clock;
            tap(&ctx, &mut a, "retry");
            tap(&ctx, &mut a, "restore-samples");
            assert_eq!(live(&a, practice).files, initial_files);
            assert_eq!(live(&a, practice).clock, initial_clock);
            assert!(session(&a, practice).last_run.is_none());
            assert!(!a.cron_presentation.running());
            assert!(!a.metrics.completion);
            let mut expected = before.0;
            if practice {
                expected["attempts"][&id]["workspace"]["cron_learning"]["simulation"]["files"] =
                    serde_json::to_value(&initial_files).unwrap();
                expected["attempts"][&id]["workspace"]["cron_learning"]["simulation"]["clock"] =
                    serde_json::to_value(initial_clock).unwrap();
                expected["attempts"][&id]["workspace"]["cron_learning"]["session"]
                    .as_object_mut()
                    .unwrap()
                    .remove("last_run");
            } else {
                expected["attempts"][&id]["state"]["simulation"]["files"] =
                    serde_json::to_value(&initial_files).unwrap();
                expected["attempts"][&id]["state"]["simulation"]["clock"] =
                    serde_json::to_value(initial_clock).unwrap();
                expected["attempts"][&id]["workspace"]["cron_session"]
                    .as_object_mut()
                    .unwrap()
                    .remove("last_run");
            }
            assert_eq!(
                snapshot(&a),
                (expected, before.1),
                "Recovery changes exactly this scope's files, virtual clock and run log; drafts/tables/policies/other work/receipts remain exact"
            );
        }
    }
}
#[test]
fn cron_actual_install_errors_keep_the_bad_draft_and_diagnose_syntax_vs_unsupported_commands() {
    let (ctx, mut a, _) = opened(0);
    assignment(&ctx, &mut a);
    for (text, term) in [
        (
            "60 9 * * MON-FRI cat /work/source > /work/report\n",
            "range",
        ),
        ("10 9 * * MON-FRI cat /work/source | cat\n", "Unsupported"),
    ] {
        let state = live(&a, false).clone();
        type_field(&ctx, &mut a, "document", text);
        tap(&ctx, &mut a, "install");
        assert_eq!(live(&a, false), &state);
        assert_eq!(session(&a, false).drafts["user:alex"], text);
        assert!(!a.metrics.completion);
        assert!(
            a.cron_presentation
                .notice()
                .is_some_and(|s| s.to_lowercase().contains(&term.to_lowercase())),
            "Educational diagnostic {term}: {:?}",
            a.cron_presentation.notice()
        );
    }
}

// Normal registration exposes course navigation that the old developer-only
// fixture deliberately did not have. Drive it through the existing widgets.
fn route_reveal(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    let mut stable = None;
    let mut trace = Vec::new();
    for _ in 0..20 {
        let r = match key {
            "unit" => a.metrics.unit_picker,
            "start" => a.metrics.card_actions.first().map(|r|r.0),
            "next" => a.metrics.next_unit,
            "complete" => a.metrics.unit_complete,
            "back-unit" => a.metrics.unit_back,
            _ => unreachable!(),
        }
        .unwrap_or_else(||panic!("Existing normal course control {key}; selected={:?}; unit={}; cards={}; completion={}",a.selected,a.filter.unit,a.metrics.card_actions.len(),a.metrics.completion));
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
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(r.center())],
            );
        } else {
            stable = None;
            if trace.len() == 3 {
                trace.remove(0);
            }
            trace.push((r, clip));
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![
                    egui::Event::PointerMoved(egui::pos2(
                        clip.left() + 1. / ctx.pixels_per_point(),
                        clip.center().y,
                    )),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(
                            0.,
                            (clip.center().y - r.center().y)
                                .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                        ),
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }
    panic!("Normal course control {key} not fully visible after20 real wheel frames: {trace:?}")
}
#[test]
fn cron_normal_unit_picker_start_next_and_complete_use_actual_events_and_warm_checks() {
    let (ctx, mut a) = app();
    draw(&ctx, &mut a);
    assert_eq!(
        (
            one_off_boundary::before_len(&a.scenarios),
            one_off_boundary::before_len(&a.units)
        ),
        (176, 46)
    );
    assert!(a.progress.attempts.is_empty());
    let before = snapshot(&a);
    let picker = route_reveal(&ctx, &mut a, "unit");
    click(&ctx, &mut a, picker);
    let mut chosen = false;
    for _ in 0..20 {
        let r = a
            .metrics
            .unit_choices
            .iter()
            .find(|(id, _)| id == "linux-cron-foundations")
            .expect("Real normal unit option")
            .1;
        let clip = a
            .metrics
            .unit_popup
            .expect("Actual popup clip")
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            click(&ctx, &mut a, r);
            chosen = true;
            break;
        }
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., clip.center().y - r.center().y),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    assert!(chosen, "Choose the actual fully visible unit option");
    assert_eq!(a.filter.unit, "linux-cron-foundations");
    assert_eq!(a.metrics.card_actions.len(), 4);
    assert_eq!(
        snapshot(&a),
        before,
        "Unit selection grants no work, visit or credit"
    );
    let r = route_reveal(&ctx, &mut a, "start");
    click(&ctx, &mut a, r);
    let refs: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/cron-lessons-review-v1.json"
    ))
    .unwrap();
    let ids = crate::ui::smoke::cron_lessons::IDS;
    for n in 0..4 {
        assert_eq!(a.scenarios[a.selected.unwrap()].id, ids[n]);
        assert_eq!(attempt(&a).state, a.scenarios[a.selected.unwrap()].initial);
        assert_eq!(view(&a), View::Introduction);
        assignment(&ctx, &mut a);
        for action in
            serde_json::from_value::<Vec<work::ReferenceAction>>(refs[n]["reference"].clone())
                .unwrap()
        {
            apply(&ctx, &mut a, &action, false);
        }
        check_work(&ctx, &mut a);
        assert!(a.metrics.completion);
        // Settle all bounded background cases, then prove completed normal
        // navigation does no JSON metering, snapshotting or VM work while idle.
        for _ in 0..1000 {
            if !a.scenarios.iter().any(|s| {
                a.progress
                    .attempts
                    .get(&s.id)
                    .is_some_and(|p| a.cron_presentation.memo.pending(p, s))
            }) {
                break;
            }
            draw(&ctx, &mut a);
        }
        let counts = a.cron_presentation.memo.counters;
        for _ in 0..10 {
            draw(&ctx, &mut a);
        }
        assert_eq!(a.cron_presentation.memo.counters.meters, counts.meters);
        assert_eq!(
            a.cron_presentation.memo.counters.snapshots,
            counts.snapshots
        );
        assert_eq!(a.cron_presentation.memo.counters.steps, counts.steps);
        if n < 3 {
            let before = snapshot(&a);
            let r = route_reveal(&ctx, &mut a, "next");
            click(&ctx, &mut a, r);
            let mut expected = before.0;
            let clock = expected["clock"].as_u64().unwrap() + 1;
            let source = a.scenarios.iter().find(|s| s.id == ids[n + 1]).unwrap();
            let mut fresh = Attempt::new(source, clock);
            work::budget::prepare(&mut fresh, source).unwrap();
            expected["clock"] = clock.into();
            expected["attempts"][ids[n + 1]] = serde_json::to_value(fresh).unwrap();
            assert_eq!(
                snapshot(&a),
                (expected, before.1),
                "Only next fresh assignment and recency change"
            );
        }
    }
    assert!(a.metrics.next_unit.is_none());
    let _ = route_reveal(&ctx, &mut a, "complete");
    let before = snapshot(&a);
    let r = route_reveal(&ctx, &mut a, "back-unit");
    click(&ctx, &mut a, r);
    assert!(a.selected.is_none());
    assert_eq!(a.filter.unit, "linux-cron-foundations");
    assert_eq!(a.metrics.card_actions.len(), 4);
    assert_eq!(
        snapshot(&a),
        before,
        "Back to unit preserves all drafts, logs and receipts"
    );
}
