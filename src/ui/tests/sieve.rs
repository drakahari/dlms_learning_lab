//! Real Sieve widgets; rendered/physical/novice acceptance is separate.
use super::*;
use dlms_learning_lab::{sieve_source as binding, sieve_workspace as work};
const ID: &str = "lpic2-sieve-header-filing";
fn painted(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 7000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        painted(ctx, a, vec![]);
    }
}
fn finish(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..work::MAX_CASES + 3 {
        painted(ctx, a, vec![]);
    }
}
fn session(a: &LabApp) -> &work::Session {
    let State::SieveFiltering { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::SieveFiltering { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn click_raw(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, rect, clip) = a
        .metrics
        .sieve
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            rect,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {rect:?} in {clip:?}"
    );
    let pos = rect.center();
    painted(
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
    painted(
        ctx,
        a,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    settle(ctx, a);
    click_raw(ctx, a, key);
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    hit(ctx, a, "editor");
    painted(
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
fn sieve_editor_runs_real_cases_and_breaking_a_good_filter_clears_current_success() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert_eq!(a.metrics.sieve.result.unwrap().0, 5);
    assert_eq!(a.metrics.sieve.result.unwrap().1, 8);
    let failure = a
        .metrics
        .sieve
        .controls
        .iter()
        .find(|r| r.0 == "failed-case")
        .unwrap()
        .1;
    let line = egui::TextStyle::Body.resolve(&ctx.style()).size;
    assert!(
        failure.height() > line * 2.,
        "Failure capture must include expected/actual explanation, not only its heading"
    );
    edit(&ctx, &mut a, &d.assignment.reference);
    assert!(!session(&a).assignment.checked);
    assert!(session(&a).assignment.results.is_empty());
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert_eq!(a.metrics.sieve.result.unwrap().0, 8);
    edit(&ctx, &mut a, &format!("{}\nkeep;", d.assignment.reference));
    assert!(!a.metrics.completion);
    assert!(!session(&a).assignment.checked);
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .sieve
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
}
#[test]
fn sieve_fill_is_inert_and_practice_and_challenge_preserve_independent_work() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    let original = session(&a).assignment.clone();
    hit(&ctx, &mut a, "fill-example");
    assert!(!session(&a).practice.checked);
    assert!(session(&a).practice.results.is_empty());
    assert!(!session(&a).practice.running);
    assert_eq!(session(&a).assignment, original);
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    let d = binding::source(&a.scenarios[i]).unwrap();
    assert!(session(&a).practice.solved(&d.example));
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, original);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(!a.metrics.sieve.controls.iter().any(|r| matches!(
        r.0.as_str(),
        "introduction" | "example" | "fill-example" | "worked"
    )));
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "editor"));
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "run"));
}
#[test]
fn sieve_stop_interrupts_between_samples_and_retry_finishes_the_same_draft() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let script = binding::source(&a.scenarios[i])
        .unwrap()
        .assignment
        .reference
        .clone();
    edit(&ctx, &mut a, &script);
    click_raw(&ctx, &mut a, "run");
    assert!(session(&a).assignment.running);
    click_raw(&ctx, &mut a, "stop");
    assert!(!session(&a).assignment.running);
    assert!(session(&a).assignment.stopped);
    assert!(!session(&a).assignment.checked);
    assert!(!a.metrics.completion);
    let count = session(&a).assignment.results.len();
    assert!(count > 0 && count < 8);
    assert_eq!(session(&a).assignment.draft, script);
    finish(&ctx, &mut a);
    assert_eq!(session(&a).assignment.results.len(), count);
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn sieve_introduction_and_example_navigation_keep_the_assignment_intact() {
    let (ctx, mut a, _) = opened(teaching::View::Introduction);
    let before = session(&a).assignment.clone();
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "worked"));
    assert!(!a.metrics.sieve.controls.iter().any(|r| r.0 == "editor"));
    hit(&ctx, &mut a, "try");
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "editor"));
    assert_eq!(session(&a).assignment, before);
    assert!(!a.metrics.completion);
}
#[test]
fn sieve_controls_reflow_at_large_text_and_popup_escape_retains_keyboard_focus() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    hit(&ctx, &mut a, "sample");
    let id = a
        .metrics
        .sieve
        .focus
        .iter()
        .find(|r| r.0 == "sample")
        .unwrap()
        .1;
    assert!(egui::ComboBox::is_open(&ctx, id));
    painted(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert!(!egui::ComboBox::is_open(&ctx, id));
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    for (scale, width, theme) in [
        (1., 1450., "dlms-navy"),
        (1.5, 840., "dlms-light"),
        (2., 840., "dlms-navy"),
    ] {
        a.scale = scale;
        a.theme_id = theme.into();
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 7000., vec![]);
        }
        let controls = &a.metrics.sieve.controls;
        let run = controls.iter().find(|r| r.0 == "run").unwrap().1;
        for key in ["run", "stop", "sample"] {
            let (_, r, clip) = controls.iter().find(|r| r.0 == key).unwrap();
            assert!(
                (r.right() - clip.right()) * ctx.pixels_per_point() <= 1.,
                "{key} clipped at {scale}"
            );
            assert!(
                (run.height() - r.height()).abs() <= 1.,
                "{key} height at {scale}"
            );
        }
        let (_, r, clip) = controls.iter().find(|r| r.0 == "editor-viewport").unwrap();
        assert!((r.right() - clip.right()) * ctx.pixels_per_point() <= 1.);
    }
}

#[test]
fn address_tasks_use_their_real_opt_in_parser_and_keep_both_drafts_separate() {
    for id in [
        "lpic2-sieve-sender-domain",
        "lpic2-sieve-recipient-recovery",
    ] {
        let (ctx, mut a, _) = opened(teaching::View::Assignment);
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        let State::SieveFiltering { simulation } =
            &mut a.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        let d = binding::source(&a.scenarios[i]).unwrap().clone();
        assert!(d.assignment.address_tests);
        assert!(dlms_learning_lab::sieve_parse::parse(&d.assignment.reference).is_err());
        settle(&ctx, &mut a);
        edit(&ctx, &mut a, &d.assignment.reference);
        hit(&ctx, &mut a, "run");
        finish(&ctx, &mut a);
        assert!(
            progress::current_solved(&a.progress.attempts[id], &a.scenarios[i]),
            "{id}"
        );
        edit(&ctx, &mut a, "keep;");
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        hit(&ctx, &mut a, "example");
        hit(&ctx, &mut a, "fill-example");
        let State::SieveFiltering { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.draft, "keep;");
        assert!(!simulation.practice.checked);
        hit(&ctx, &mut a, "run");
        finish(&ctx, &mut a);
        let State::SieveFiltering { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(simulation.practice.solved(&d.example));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}

#[test]
fn delivery_tasks_use_real_editor_run_grading_envelope_and_isolated_practice() {
    for id in [
        "lpic2-sieve-envelope-recipient",
        "lpic2-sieve-redirect-recovery",
        "lpic2-sieve-rejection-priority",
    ] {
        let (ctx, mut a, _) = opened(teaching::View::Assignment);
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        let State::SieveFiltering { simulation } =
            &mut a.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        let d = binding::source(&a.scenarios[i]).unwrap().clone();
        assert!(d.assignment.address_tests && d.assignment.delivery_actions);
        assert!(dlms_learning_lab::sieve_parse::parse(&d.assignment.reference).is_err());
        settle(&ctx, &mut a);
        assert!(a.metrics.sieve.controls.iter().any(|c| c.0 == "envelope"));
        edit(&ctx, &mut a, &d.assignment.reference);
        hit(&ctx, &mut a, "run");
        finish(&ctx, &mut a);
        assert!(
            progress::current_solved(&a.progress.attempts[id], &a.scenarios[i]),
            "{id}"
        );
        edit(&ctx, &mut a, "keep;");
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        hit(&ctx, &mut a, "example");
        hit(&ctx, &mut a, "fill-example");
        let State::SieveFiltering { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.draft, "keep;");
        assert!(!simulation.practice.checked);
        hit(&ctx, &mut a, "run");
        finish(&ctx, &mut a);
        let State::SieveFiltering { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(simulation.practice.solved(&d.example));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}

#[test]
fn vacation_widgets_show_a_shared_timeline_only_for_the_opt_in_profile() {
    use dlms_learning_lab::{sieve_envelope, sieve_vacation_policy, sieve_workspace::VacationTask};
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    assert!(!a.metrics.sieve.controls.iter().any(|r| r.0 == "timeline"));
    let mut d = binding::source(&a.scenarios[i]).unwrap().clone();
    for (task, site) in [
        (&mut d.assignment, "away.test"),
        (&mut d.example, "practice.test"),
    ] {
        task.address_tests = true;
        task.delivery_actions = true;
        let owner = format!("learner@{site}");
        task.vacation = Some(VacationTask {
            user: sieve_vacation_policy::Policy {
                primary: owner.clone(),
                aliases: Default::default(),
            },
            required_reply_text: "contact the team".into(),
        });
        task.reference =
            "require \"vacation\";vacation :days 1 \"I am away; contact the team.\";".into();
        for (index, c) in task.cases.iter_mut().enumerate() {
            c.headers = vec![("To".into(), owner.clone())];
            c.envelope = Some(sieve_envelope::Input {
                from: "visitor@outside.test".into(),
                to: owner.clone(),
            });
            c.expected = [String::from("INBOX")].into();
            c.arrival_seconds =
                Some([0, 86399, 86400, 86401, 172800, 172801, 259200, 259201][index]);
            c.expected_reply = Some(matches!(index, 0 | 2 | 4 | 6));
        }
    }
    assert_eq!((d.assignment.cases.len(), d.example.cases.len()), (8, 4));
    d.validate().unwrap();
    a.scenarios[i].goal = engine::Goal::SieveFiltering {
        definition: Box::new(d.clone()),
    };
    a.scenarios[i].initial = State::SieveFiltering {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    a.progress.attempts.remove(ID);
    a.launch(i);
    let State::SieveFiltering { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    settle(&ctx, &mut a);
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "timeline"));
    edit(&ctx, &mut a, &d.assignment.reference);
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(d.solved(session(&a)));
    assert_eq!(
        a.metrics.sieve.result.as_ref().map(|r| (r.0, r.1)),
        Some((8, 8))
    );
    hit(
        &ctx,
        &mut a,
        &format!("outcome:{}", d.assignment.cases[0].id),
    );
    hit(
        &ctx,
        &mut a,
        &format!("reply-toggle:{}", d.assignment.cases[0].id),
    );
    settle(&ctx, &mut a);
    assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "reply-body"));
    for (scale, width, theme) in [
        (1., 1450., "dlms-navy"),
        (1.5, 840., "dlms-light"),
        (2., 840., "dlms-navy"),
    ] {
        a.scale = scale;
        a.theme_id = theme.into();
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 7000., vec![]);
        }
        for key in ["timeline", "run", "stop", "sample", "editor-viewport"] {
            let (_, r, clip) = a
                .metrics
                .sieve
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap();
            assert!(
                (r.right() - clip.right()) * ctx.pixels_per_point() <= 1.,
                "{key} clipped at {scale}"
            );
        }
    }
    edit(&ctx, &mut a, "keep;");
    assert!(!d.solved(session(&a)));
    assert!(session(&a).assignment.results.is_empty());
    hit(&ctx, &mut a, "example");
    hit(&ctx, &mut a, "fill-example");
    hit(&ctx, &mut a, "run");
    finish(&ctx, &mut a);
    assert!(session(&a).practice.solved(&d.example));
    assert!(!d.solved(session(&a)));
    assert_eq!(session(&a).assignment.draft, "keep;");
}

#[test]
fn vacation_timeline_stays_inside_actual_fractional_density_content_edges() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    let id = "lpic2-sieve-vacation-safety";
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::SieveFiltering { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    a.scale = 2.;
    for _ in 0..4 {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946. / 2.3, 929. / 2.3),
            )),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let _ = ctx.run(input, |ctx| a.render(ctx));
    }
    assert!((ctx.pixels_per_point() - 2.3).abs() < 0.0001);
    let (_, rect, _) = a
        .metrics
        .sieve
        .controls
        .iter()
        .find(|r| r.0 == "timeline")
        .unwrap();
    let parent = a.metrics.main_viewport.unwrap();
    assert!(
        (rect.right() - parent.right()) * ctx.pixels_per_point() <= 1.,
        "Timeline {rect:?} exceeds actual content {parent:?} at fractional density"
    );
    assert!((parent.left() - rect.left()) * ctx.pixels_per_point() <= 1.);
}

#[test]
fn migration_context_is_readonly_and_scope_correct_in_all_three_views_and_challenge() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let id = "lpic2-procmail-sieve-migration";
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    for view in [teaching::View::Introduction, teaching::View::Example, teaching::View::Assignment] {
        let State::SieveFiltering { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state else { panic!() };
        simulation.view = view;
        settle(&ctx, &mut a);
        assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == "legacy-recipe"));
        assert!(!a.metrics.sieve.fields.iter().any(|r| r.0 == "legacy-recipe"));
        if view != teaching::View::Introduction {
            assert_eq!(a.metrics.sieve.fields.iter().filter(|r| r.0 == "editor").count(), 1);
        }
    }
    a.progress.attempts.get_mut(id).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["legacy-recipe", "editor", "run"] { assert!(a.metrics.sieve.controls.iter().any(|r| r.0 == key)); }
    for key in ["worked", "fill-example", "example"] { assert!(!a.metrics.sieve.controls.iter().any(|r| r.0 == key)); }
}

#[test]
fn migration_legacy_context_keeps_full_width_at_fractional_density_and_enlarged_text() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == "lpic2-procmail-sieve-migration").unwrap();
    a.launch(i);
    for zoom in [1., 1.5, 2.] {
        ctx.set_zoom_factor(zoom);
        for _ in 0..3 { frame(&ctx, &mut a, 823., 8000., vec![]); }
        let (_, r, clip) = a.metrics.sieve.controls.iter().find(|r| r.0 == "legacy-recipe").unwrap();
        assert!(r.left() >= clip.left() && r.right() <= clip.right() + 1. / ctx.pixels_per_point(), "{zoom}: {r:?} in {clip:?}");
    }
}

#[test]
fn migration_entire_readonly_recipe_can_fit_below_the_toolbar_at_actual_navy200_size() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    a.scale = 2.;
    let id = "lpic2-procmail-sieve-migration";
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::SieveFiltering { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state else { panic!() };
    simulation.view = teaching::View::Assignment;
    for _ in 0..4 {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(946. / 2.3, 929. / 2.3))),
            ..Default::default()
        };
        input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().native_pixels_per_point = Some(1.15);
        let _ = ctx.run(input, |ctx| a.render(ctx));
    }
    assert!((ctx.pixels_per_point() - 2.3).abs() < 0.0001);
    let (_, recipe, _) = a.metrics.sieve.controls.iter().find(|r| r.0 == "legacy-recipe").unwrap();
    let visible = a.metrics.main_viewport.unwrap().intersect(ctx.content_rect());
    assert!(recipe.height() <= visible.height(), "Whole policy cannot fit below the fixed toolbar: {recipe:?}, viewport {visible:?}");
}
