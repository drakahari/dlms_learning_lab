//! Actual native widgets; injected input is technical evidence, not a novice study.
use super::*;
use dlms_learning_lab::{vpn_point_source as binding, vpn_point_workspace as work};
const ID: &str = "lpic2-vpn-peer-profile";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 12000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::VpnPoint { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::VpnPoint { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .vpn_point
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {r:?} {clip:?}"
    );
    let p = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
    let id = a
        .metrics
        .vpn_point
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    paint(
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
fn visible_text(ctx: &egui::Context, a: &mut LabApp) -> Vec<String> {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, text);
                }
            }
            egui::epaint::Shape::Text(t) => text.push(t.galley.job.text.clone()),
            _ => {}
        }
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1450., 12000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in out.shapes {
        collect(&s.shape, &mut text);
    }
    text
}

fn perform(ctx: &egui::Context, a: &mut LabApp, action: &work::Action) {
    match action {
        work::Action::Edit { peer, text } => edit(ctx, a, &format!("editor-{peer}"), text),
        work::Action::Save { peer } => hit(ctx, a, &format!("save-{peer}")),
        work::Action::Apply => hit(ctx, a, "apply"),
        work::Action::Check => hit(ctx, a, "check"),
        // Actual address edits and popup selection are covered by the native fixed plan.
        work::Action::Query { .. } | work::Action::Probe => {}
    }
}
#[test]
fn vpn_real_edit_save_apply_check_and_keyboard_focus_preserve_current_outcomes() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("apply saved profiles to evaluate"))
    );
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let e: serde_json::Value =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        perform(
            &ctx,
            &mut a,
            &serde_json::from_value(e["action"].clone()).unwrap(),
        );
    }
    assert!(a.metrics.completion);
    let saved = session(&a).assignment.world.saved.clone();
    edit(
        &ctx,
        &mut a,
        "editor-1",
        &(saved[1].clone() + "script-security 2\n"),
    );
    assert!(!session(&a).assignment.checked && !a.metrics.completion);
    assert_eq!(session(&a).assignment.world.saved, saved);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "save-1");
    hit(&ctx, &mut a, "apply");
    assert!(
        session(&a)
            .assignment
            .visible_notice()
            .unwrap()
            .kind
            .contains("Unsupported")
    );
    assert_ne!(
        session(&a).assignment.world.applied_source.as_ref(),
        Some(&session(&a).assignment.world.saved)
    );
    edit(&ctx, &mut a, "editor-1", &saved[1]);
    hit(&ctx, &mut a, "save-1");
    hit(&ctx, &mut a, "apply");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    edit(&ctx, &mut a, "source", "not an address");
    hit(&ctx, &mut a, "probe");
    assert!(session(&a).assignment.visible_notice().is_some());
    assert!(
        a.metrics.completion,
        "A malformed investigation query must not change a correct configuration"
    );
}
#[test]
fn vpn_practice_is_separate_and_three_views_teach_before_assessment() {
    let (ctx, mut a, i) = open(teaching::View::Introduction);
    assert!(a.metrics.vpn_point.controls.iter().any(|r| r.0 == "worked"));
    let original = session(&a).assignment.clone();
    hit(&ctx, &mut a, "example");
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    for action in &d.worked {
        perform(&ctx, &mut a, action);
    }
    assert!(session(&a).example.solved(&d.example));
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, original);
    hit(&ctx, &mut a, "introduction");
    assert!(a.metrics.vpn_point.controls.iter().any(|r| r.0 == "worked"));
}
#[test]
fn vpn_challenge_keeps_editors_without_unsolicited_solutions() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in [
        "introduction",
        "example",
        "try-example",
        "worked",
        "fill-worked-0",
        "fill-worked-1",
    ] {
        assert!(!a.metrics.vpn_point.controls.iter().any(|r| r.0 == key));
    }
    for key in [
        "assignment-content",
        "editor-0",
        "editor-1",
        "save-0",
        "save-1",
        "apply",
        "source",
        "destination",
        "probe",
        "check",
    ] {
        assert!(a.metrics.vpn_point.controls.iter().any(|r| r.0 == key));
    }
    let text = visible_text(&ctx, &mut a);
    assert!(!text.iter().any(|s| s.contains("Show worked answer")));
    assert!(
        session(&a).assignment.editors[1].contains(" 1195"),
        "Preserve the meaningful starter fault"
    );
}
#[test]
fn vpn_editors_save_rows_and_fields_reflow_without_scrollbar_displacement() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for (width, scale) in [
        (1450., 1.),
        (1450., 1.5),
        (760., 1.),
        (760., 1.5),
        (760., 2.),
    ] {
        ctx.set_zoom_factor(scale);
        a.scale = scale;
        for light in [false, true] {
            a.theme_id = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap()
                .id
                .clone();
            theme::apply(&ctx, &a.theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 16000., vec![]);
            }
            let rect = |a: &LabApp, key: &str| {
                a.metrics
                    .vpn_point
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap()
                    .1
            };
            let left = rect(&a, "editor-fixed-0");
            let right = rect(&a, "editor-fixed-1");
            if width == 1450. && scale == 1. {
                assert!(right.left() > left.right());
                assert!((rect(&a, "save-0").top() - rect(&a, "save-1").top()).abs() < 1.);
            } else if width == 760. {
                assert!(right.top() > left.bottom());
            }
            for key in [
                "save-0",
                "save-1",
                "source",
                "destination",
                "from-choice",
                "apply",
                "probe",
                "check",
            ] {
                let r = rect(&a, key);
                assert!(
                    r.left() >= -1. && r.right() <= width + 1.,
                    "{key}: {r:?} at {width}/{scale}"
                );
                assert!(r.height() > 15.);
            }
            let before = rect(&a, "save-0");
            // Increase both horizontal and vertical overflow without altering the outer editor.
            let source = a.scenarios[a.selected.unwrap()].clone();
            let d = binding::source(&source).unwrap();
            let State::VpnPoint { simulation } =
                &mut a.progress.attempts.get_mut(ID).unwrap().state
            else {
                panic!()
            };
            simulation
                .assignment
                .apply(
                    &d.assignment,
                    &work::Action::Edit {
                        peer: 0,
                        text: ("# ".to_owned() + &"x".repeat(300) + "\n").repeat(20),
                    },
                )
                .unwrap();
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 16000., vec![]);
            }
            let after = rect(&a, "save-0");
            assert!(
                (before.top() - after.top()).abs() < 1.
                    && (before.left() - after.left()).abs() < 1.,
                "Scrollbar moved Save: {before:?}->{after:?}"
            );
        }
    }
}

#[test]
fn vpn_guidance_uses_an_aligned_row_and_reference_expansion_belongs_to_each_lesson() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    for light in [false, true] {
        a.theme_id = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap()
            .id
            .clone();
        theme::apply(&ctx, &a.theme_id);
        for (width, scale) in [(1450., 1.), (1600., 1.5), (760., 2.)] {
            ctx.set_zoom_factor(scale);
            a.scale = scale;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 16000., vec![]);
            }
            let rect = |key: &str| {
                a.metrics
                    .vpn_point
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap()
                    .1
            };
            let left = rect("editor-fixed-0");
            let right = rect("editor-fixed-1");
            if width > 1000. {
                assert!(right.left() > left.right());
                assert!(
                    (right.top() - left.top()).abs() < 1.,
                    "Different guidance lengths must not offset editors"
                );
                assert!((rect("save-1").top() - rect("save-0").top()).abs() < 1.);
            } else {
                assert!(right.top() > left.bottom());
            }
        }
    }
    ctx.set_zoom_factor(1.);
    a.scale = 1.;
    settle(&ctx, &mut a);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("Private-key reference"))
    );
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic2-vpn-lan-return")
        .unwrap();
    a.launch(i);
    let id = a.scenarios[i].id.clone();
    let State::VpnPoint { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    settle(&ctx, &mut a);
    assert!(
        !visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("Private-key reference")),
        "The route lesson's optional file reference must not inherit another lesson's expanded panel"
    );
}

#[test]
fn vpn_related_workspaces_keep_shared_navigation_modes_and_selected_reset() {
    fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
        let p = r.center();
        for pressed in [true, false] {
            paint(
                ctx,
                a,
                vec![
                    egui::Event::PointerMoved(p),
                    egui::Event::PointerButton {
                        pos: p,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        settle(ctx, a);
    }
    for id in [ID, "lpic2-exposure-client-policy", "lpic2-ftp-local-jail"] {
        let (ctx, mut a, _) = open(teaching::View::Assignment);
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        settle(&ctx, &mut a);
        assert!(
            a.metrics.back.is_some()
                && a.metrics.reset.is_some()
                && a.metrics.mode_challenge.is_some()
                && a.metrics.mode_learn.is_some(),
            "Missing common controls: {id}"
        );
        let r = a.metrics.mode_challenge.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.progress.attempts[id].challenge);
        let r = a.metrics.mode_learn.unwrap();
        click(&ctx, &mut a, r);
        assert!(!a.progress.attempts[id].challenge);
        if id == ID {
            edit(
                &ctx,
                &mut a,
                "editor-1",
                "# Preserve this unfinished draft\n",
            );
        }
        a.progress.favorites.insert(id.into());
        let before = a.progress.attempts[id].clone();
        let r = a.metrics.back.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.selected.is_none());
        assert_eq!(
            serde_json::to_value(&a.progress.attempts[id]).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        a.launch(i);
        settle(&ctx, &mut a);
        assert_eq!(
            a.progress.attempts[id].state, before.state,
            "Return must retain actual draft/state"
        );
        let others = a
            .progress
            .attempts
            .iter()
            .filter(|(key, _)| key.as_str() != id)
            .map(|(key, v)| (key.clone(), serde_json::to_value(v).unwrap()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let r = a.metrics.reset.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.reset_pending);
        let r = a.metrics.confirm_reset.unwrap();
        click(&ctx, &mut a, r);
        assert!(!a.reset_pending);
        assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
        assert!(a.progress.favorites.contains(id));
        for (key, value) in others {
            assert_eq!(
                serde_json::to_value(&a.progress.attempts[&key]).unwrap(),
                value,
                "Other work must survive selected reset"
            );
        }
    }
}
