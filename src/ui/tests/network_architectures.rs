//! Actual egui controls on synthetic profiles, not independent learner acceptance.
use super::debug_recovery::{settle, tap};
use super::*;
use dlms_learning_lab::{
    network_architecture_source as binding, network_architecture_workspace as work,
};
const IDS: [&str; 3] = [
    "ccna-collapsed-campus-uplinks",
    "ccna-distribution-core-paths",
    "ccna-leaf-spine-rack-expansion",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn state(a: &LabApp) -> &work::Session {
    let State::CampusArchitecture { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    for _ in 0..20 {
        settle(ctx, a);
        let (_, r, clip) = a
            .metrics
            .network_architecture
            .controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap_or_else(|| panic!("missing {key}"))
            .clone();
        let clip = clip.intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            tap(ctx, a, r);
            return;
        }
        super::debug_recovery::paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Campus control did not become visible: {key}");
}

fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    click(ctx, a, key);
    click(ctx, a, &format!("{key}:{value}"));
}
fn repair(ctx: &egui::Context, a: &mut LabApp) {
    let s = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(s).unwrap().clone();
    for c in &d.assignment.reference.connections {
        if !d.assignment.editable.iter().any(|e| e.id == c.wire) {
            continue;
        }
        for (end, jack) in [("a", &c.a), ("b", &c.b)] {
            let value = jack
                .as_ref()
                .map(|j| format!("{}/{}", j.node, j.port))
                .unwrap_or_else(|| "unplugged".into());
            choose(ctx, a, &format!("{}:{end}", c.wire), &value);
        }
    }
}
#[test]
fn guided_practice_is_separate_real_cable_changes_invalidate_success_and_header_reset_restores_this_lab()
 {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let initial = state(&a).assignment.clone();
        click(&ctx, &mut a, "example");
        click(&ctx, &mut a, "check");
        assert!(
            !state(&a)
                .practice
                .solved(&binding::source(&a.scenarios[i]).unwrap().example)
        );
        click(&ctx, &mut a, "worked");
        click(&ctx, &mut a, "check");
        assert!(
            state(&a)
                .practice
                .solved(&binding::source(&a.scenarios[i]).unwrap().example)
        );
        assert_eq!(state(&a).assignment, initial);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        click(&ctx, &mut a, "assignment");
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        repair(&ctx, &mut a);
        click(&ctx, &mut a, "check");
        assert!(
            a.metrics.completion,
            "{id}: {:?}",
            state(&a).checks(binding::source(&a.scenarios[i]).unwrap(), false)
        );
        let d = binding::source(&a.scenarios[i]).unwrap().clone();
        let cable = d.assignment.editable[0].id.clone();
        choose(&ctx, &mut a, &format!("{cable}:b"), "unplugged");
        assert!(!state(&a).assignment.checked);
        assert!(!a.metrics.completion);
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        let practice = binding::source(&a.scenarios[i])
            .unwrap()
            .new_session()
            .unwrap()
            .practice;
        let reset = a.metrics.reset.unwrap();
        tap(&ctx, &mut a, reset);
        let confirm = a.metrics.confirm_reset.unwrap();
        tap(&ctx, &mut a, confirm);
        assert_eq!(state(&a).assignment, initial);
        assert_eq!(state(&a).practice, practice);
    }
}
#[test]
fn assignments_hide_worked_controls_and_challenge_keeps_the_meaningful_cable_task() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        click(&ctx, &mut a, "assignment");
        assert!(
            a.metrics
                .network_architecture
                .controls
                .iter()
                .any(|c| c.0 == "check")
        );
        assert!(
            !a.metrics
                .network_architecture
                .controls
                .iter()
                .any(|c| c.0 == "worked")
        );
        assert_eq!(
            state(&a).assignment.draft,
            binding::source(&a.scenarios[i]).unwrap().assignment.initial
        );
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        settle(&ctx, &mut a);
        assert!(
            !a.metrics
                .network_architecture
                .controls
                .iter()
                .any(|c| c.0 == "example")
        );
        assert!(
            a.metrics
                .network_architecture
                .controls
                .iter()
                .any(|c| c.0 == "check")
        );
    }
}
#[test]
fn responsive_cable_columns_and_topology_cards_keep_geometry_without_mutating_work() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        click(&ctx, &mut a, "assignment");
        let before = serde_json::to_value(&a.progress).unwrap();
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for light in [false, true] {
                a.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &a.theme_id);
                for width in [620., 1500.] {
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 12000., vec![]);
                    }
                    let m = &a.metrics.network_architecture;
                    let endpoints: Vec<_> = m
                        .controls
                        .iter()
                        .filter(|c| c.0.ends_with(":a") || c.0.ends_with(":b"))
                        .collect();
                    assert!(!endpoints.is_empty());
                    for r in &endpoints {
                        assert!(
                            r.1.left() >= 0. && r.1.right() <= width + 0.1,
                            "{} {:?}",
                            r.0,
                            r.1
                        );
                        assert!((r.1.height() - endpoints[0].1.height()).abs() < 0.5);
                    }
                    for pair in m.cards.windows(2) {
                        assert!((pair[0].width() - pair[1].width()).abs() < 0.1);
                        assert!((pair[0].height() - pair[1].height()).abs() < 0.1);
                    }
                    assert!(a.storage_error.is_none());
                    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn keyboard_can_leave_endpoint_popup_and_move_between_native_controls_without_changing_drafts() {
    let (ctx, mut a, i) = opened(IDS[0]);
    click(&ctx, &mut a, "assignment");
    let before = state(&a).assignment.clone();
    click(&ctx, &mut a, "ne:a");
    let key = |key, pressed| egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    };
    super::debug_recovery::paint(
        &ctx,
        &mut a,
        vec![key(egui::Key::Escape, true), key(egui::Key::Escape, false)],
    );
    settle(&ctx, &mut a);
    let first = ctx.memory(|m| m.focused());
    assert!(first.is_some());
    super::debug_recovery::paint(
        &ctx,
        &mut a,
        vec![key(egui::Key::Tab, true), key(egui::Key::Tab, false)],
    );
    settle(&ctx, &mut a);
    let second = ctx.memory(|m| m.focused());
    assert!(second.is_some());
    assert_ne!(first, second);
    let candidates = &a.metrics.network_architecture.focus;
    assert!(
        candidates.iter().any(|(_, id)| Some(*id) == second),
        "Tab must reach another real endpoint selector"
    );
    assert_eq!(state(&a).assignment, before);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
}

#[test]
fn narrow_enlarged_endpoint_values_are_complete_centered_native_text_not_ellipses() {
    fn texts<'a>(shape: &'a egui::Shape, items: &mut Vec<&'a egui::epaint::TextShape>) {
        match shape {
            egui::Shape::Text(t) => items.push(t),
            egui::Shape::Vec(v) => {
                for s in v {
                    texts(s, items)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, i) = opened(IDS[1]);
    for practice in [false, true] {
        click(
            &ctx,
            &mut a,
            if practice { "example" } else { "assignment" },
        );
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [820., 1500.] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 12000., vec![]);
                }
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width / scale, 12000. / scale),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                let mut rendered = vec![];
                for s in &output.shapes {
                    texts(&s.shape, &mut rendered);
                }
                let d = binding::source(&a.scenarios[i]).unwrap();
                let t = if practice { &d.example } else { &d.assignment };
                let attempt = state(&a).attempt(practice);
                for c in attempt
                    .draft
                    .connections
                    .iter()
                    .filter(|c| t.editable.iter().any(|w| w.id == c.wire))
                {
                    for (end, j) in [("a", &c.a), ("b", &c.b)] {
                        let Some(j) = j else {
                            continue;
                        };
                        let expected =
                            format!("{} / {}", t.facts.node(&j.node).unwrap().label, j.port);
                        let rect = a
                            .metrics
                            .network_architecture
                            .controls
                            .iter()
                            .find(|r| r.0 == format!("{}:{end}", c.wire))
                            .unwrap()
                            .1;
                        let text = rendered
                            .iter()
                            .find(|r| {
                                r.galley.job.text == expected
                                    && r.pos.y >= rect.top()
                                    && r.pos.y <= rect.bottom()
                            })
                            .unwrap_or_else(|| {
                                panic!("Missing endpoint text {expected} at {width}/{scale}")
                            });
                        assert!(
                            !text.galley.elided,
                            "Truncated {expected} at {width}/{scale}"
                        );
                        assert_eq!(text.galley.rows.len(), 1);
                        assert!(
                            text.pos.x >= rect.left()
                                && text.pos.x + text.galley.size().x <= rect.right() + 0.5
                        );
                        assert!(
                            (text.pos.y + text.galley.size().y / 2. - rect.center().y).abs() < 0.6
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn focused_endpoint_keeps_its_identity_and_keyboard_focus_across_reflow() {
    let (ctx, mut a, _) = opened(IDS[0]);
    click(&ctx, &mut a, "assignment");
    click(&ctx, &mut a, "ne:a");
    let escape = |pressed| egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    };
    super::debug_recovery::paint(&ctx, &mut a, vec![escape(true), escape(false)]);
    settle(&ctx, &mut a);
    let first = a
        .metrics
        .network_architecture
        .focus
        .iter()
        .find(|r| r.0 == "ne:a")
        .unwrap()
        .1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(first));
    for (width, scale) in [(620., 1.), (1500., 2.), (1500., 1.)] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for _ in 0..4 {
            frame(&ctx, &mut a, width, 9000., vec![]);
        }
        let now = a
            .metrics
            .network_architecture
            .focus
            .iter()
            .find(|r| r.0 == "ne:a")
            .unwrap()
            .1;
        assert_eq!(
            now, first,
            "Endpoint identity changed with width{width}/scale{scale}"
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(first),
            "Focus lost on reflow at width{width}/scale{scale}"
        );
    }
}

#[test]
fn explicitly_opened_learn_reasoning_does_not_open_challenge_guidance() {
    let (ctx, mut a, i) = opened(IDS[0]);
    click(&ctx, &mut a, "assignment");
    let before = state(&a).assignment.clone();
    click(&ctx, &mut a, "help");
    click(&ctx, &mut a, "reasoning");
    assert!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|r| r.0 == "reasoning")
    );
    let challenge = a.metrics.mode_challenge.unwrap();
    tap(&ctx, &mut a, challenge);
    assert!(a.progress.attempts[IDS[0]].challenge);
    assert!(
        !a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|r| r.0 == "reasoning")
    );
    assert!(
        !a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|r| r.0 == "worked")
    );
    assert_eq!(state(&a).assignment, before);
    let learn = a.metrics.mode_learn.unwrap();
    tap(&ctx, &mut a, learn);
    assert!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|r| r.0 == "reasoning")
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
}

#[test]
fn leaf_spine_pair_choice_and_case_summaries_do_not_change_graded_work() {
    let (ctx, mut a, i) = opened(IDS[2]);
    click(&ctx, &mut a, "assignment");
    click(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .filter(|c| c.0 == "case-summary")
            .count(),
        4
    );
    assert!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|c| c.0 == "failed-row")
    );
    let before = state(&a).clone();
    choose(&ctx, &mut a, "pair", "2");
    assert_eq!(*state(&a), before);
    assert!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|c| c.0 == "spine-count")
    );
    repair(&ctx, &mut a);
    click(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
    assert!(
        !a.metrics
            .network_architecture
            .controls
            .iter()
            .any(|c| c.0 == "failed-row")
    );
    assert_eq!(
        a.metrics
            .network_architecture
            .controls
            .iter()
            .filter(|c| c.0 == "case-summary")
            .count(),
        4
    );
    choose(&ctx, &mut a, "r4-s1:b", "unplugged");
    assert!(!state(&a).assignment.checked);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
}

#[test]
fn concise_failed_pair_feedback_uses_actual_counts_without_repeating_full_paths() {
    let s = pack::load(include_str!(
        "../../../scenarios/leaf-spine-lessons-v1.json"
    ))
    .unwrap()
    .remove(0);
    let d = binding::source(&s).unwrap();
    let t = &d.assignment;
    let mut a = work::Attempt {
        draft: t.reference.clone(),
        checked: false,
    };
    a.draft.connections[9].b = None;
    let normal =
        super::super::network_architecture_page::pair_failure(t, &a, &t.cases[0], &t.pairs[0]);
    assert!(normal.contains("2 of 3 required internal spines available"));
    assert!(!normal.contains("Hierarchy path"));
    assert!(normal.len() < 120);
    let lost =
        super::super::network_architecture_page::pair_failure(t, &a, &t.cases[2], &t.pairs[0]);
    assert!(lost.contains("1 of 2 required internal spines available"));
    a.draft.connections[10].b = None;
    a.draft.connections[11].b = None;
    let none =
        super::super::network_architecture_page::pair_failure(t, &a, &t.cases[0], &t.pairs[0]);
    assert!(none.contains("0 of 3 required internal spines available"));
}
