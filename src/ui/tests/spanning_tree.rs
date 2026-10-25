//! Prospective native-egui interactions; no actual renderer/cold-open claim.
use super::*;
use dlms_learning_lab::{spanning_tree as core, spanning_tree_workspace as work};
const ID: &str = "test-spanning-tree-workspace";
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let s = pack::load(include_str!(
        "../../../tests/fixtures/spanning-tree-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    assert_eq!(a.spanning_tree_view(i), teaching::View::Introduction);
    a.set_spanning_tree_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn draft(a: &LabApp) -> &work::Draft {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .spanning_tree_draft
        .as_ref()
        .unwrap()
}
fn live(a: &LabApp) -> &core::State {
    let State::SpanningTree { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn locate(
    ctx: &egui::Context,
    a: &mut LabApp,
    f: impl Fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for _ in 0..30 {
        let r = f(&a.metrics).expect("existing STP control");
        let v = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if v.contains(r.center()) && v.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        let delta = if r.bottom() > v.bottom() {
            v.bottom() - r.bottom()
        } else {
            v.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(v.left() + 1., v.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("STP target never became actually visible")
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = locate(ctx, a, |m| {
        m.stp_fields.iter().find(|x| x.0 == key).map(|x| x.1)
    });
    click(ctx, a, r);
    a.focus_input = false;
    let id = a.metrics.stp_fields.iter().find(|x| x.0 == key).unwrap().2;
    assert!(
        ctx.memory(|m| m.has_focus(id)),
        "actual clicked field owns focus"
    );
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(value.into())],
    );
    draw(ctx, a);
}
fn apply(ctx: &egui::Context, a: &mut LabApp) {
    let r = locate(ctx, a, |m| m.stp_apply);
    click(ctx, a, r);
    draw(ctx, a);
}
fn check(ctx: &egui::Context, a: &mut LabApp) {
    let r = locate(ctx, a, |m| m.check);
    click(ctx, a, r);
    draw(ctx, a);
}
fn choose_bridge(ctx: &egui::Context, a: &mut LabApp, b: &str) {
    let r = locate(ctx, a, |m| m.stp_bridge);
    click(ctx, a, r);
    draw(ctx, a);
    let r = a
        .metrics
        .stp_bridge_items
        .iter()
        .find(|x| x.0 == b)
        .unwrap()
        .1;
    click(ctx, a, r);
    draw(ctx, a);
    assert_eq!(draft(a).selected_bridge, b);
}
fn solved(ctx: &egui::Context, a: &mut LabApp, i: usize, priority: &str) {
    choose_bridge(ctx, a, "core");
    field(ctx, a, "core:priority", priority);
    apply(ctx, a);
    check(ctx, a);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn spanning_tree_native_fill_only_practice_experiment_repair_keeps_assignment_exact() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    let before = a.progress.attempts[ID].state.clone();
    let raw = draft(&a).clone();
    let r = locate(&ctx, &mut a, |m| m.insert);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .spanning_tree_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, l.fixture);
    assert!(!l.draft.matches(&l.simulation));
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    let lesson = a.scenarios[i].spanning_tree_teaching.clone().unwrap();
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .spanning_tree_learning
            .as_ref()
            .unwrap()
            .checks(&lesson)
            .iter()
            .all(|r| r.0)
    );
    // The example initially selects sample-core; edit actual independent fields.
    field(&ctx, &mut a, "sample-core:priority", "61440");
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .spanning_tree_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.checks(&lesson)[1..]
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        lesson.try_change.expected
    );
    field(&ctx, &mut a, "sample-core:priority", "0");
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .spanning_tree_learning
            .as_ref()
            .unwrap()
            .checks(&lesson)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, before);
    assert_eq!(draft(&a), &raw);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn spanning_tree_native_alternate_current_pending_wrong_invalid_and_repair() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    solved(&ctx, &mut a, i, "4096"); // Tied configured priority; core's lower MAC wins.
    let good = live(&a).clone();
    field(&ctx, &mut a, "core:priority", "32768");
    check(&ctx, &mut a);
    assert_eq!(live(&a), &good);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "core:priority", "not-a-number");
    let before = live(&a).clone();
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    assert_eq!(live(&a), &before);
    assert!(draft(&a).notice.is_some());
    field(&ctx, &mut a, "core:priority", "0");
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "core:priority", &"7".repeat(65));
    assert_eq!(draft(&a).bridges["core"].priority, "0");
}
#[test]
fn spanning_tree_native_isolation_shortcut_fails_availability_and_connected_rows() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    solved(&ctx, &mut a, i, "0");
    choose_bridge(&ctx, &mut a, "west");
    let r = locate(&ctx, &mut a, |m| {
        m.stp_toggles
            .iter()
            .find(|x| x.0 == "west:online")
            .map(|x| x.1)
    });
    click(&ctx, &mut a, r);
    apply(&ctx, &mut a);
    check(&ctx, &mut a);
    let rows = progress::current_spanning_tree_checks(&a.progress.attempts[ID], &a.scenarios[i]);
    assert!(!rows[1].0);
    assert!(!rows[2].0);
    assert!(!a.metrics.completion);
}
#[test]
fn spanning_tree_native_modal_guards_assignment_and_practice_keep_raw_and_applied() {
    for view in [teaching::View::Assignment, teaching::View::Example] {
        for modal in 0..4 {
            let (ctx, mut a, i) = opened(view);
            let key = if view == teaching::View::Example {
                "sample-core:priority"
            } else {
                "core:priority"
            };
            if view == teaching::View::Assignment {
                choose_bridge(&ctx, &mut a, "core");
            }
            field(&ctx, &mut a, key, "8192");
            let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
            match modal {
                0 => a.reset_pending = true,
                1 => a.settings_open = true,
                2 => a.theme_preview = true,
                _ => {
                    a.settings_open = true;
                    a.settings_data = true;
                    draw(&ctx, &mut a);
                    let r = a
                        .metrics
                        .data_export
                        .expect("actual Settings export control");
                    click(&ctx, &mut a, r);
                    draw(&ctx, &mut a);
                    assert!(a.data_tools.modal_open());
                    a.settings_open = false;
                }
            }
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    acl_ctrl(egui::Key::A),
                    egui::Event::Text("0".into()),
                    acl_ctrl(egui::Key::Enter),
                ],
            );
            draw(&ctx, &mut a);
            assert_eq!(
                serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
                before,
                "modal{modal} view{view:?}"
            );
            assert!(!a.focus_input);
            let _ = i;
        }
    }
}
#[test]
fn spanning_tree_native_namespaced_undo_reflow_and_deferred_focus() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    choose_bridge(&ctx, &mut a, "core");
    field(&ctx, &mut a, "core:priority", "4096");
    let core_id = a
        .metrics
        .stp_fields
        .iter()
        .find(|x| x.0 == "core:priority")
        .unwrap()
        .2;
    choose_bridge(&ctx, &mut a, "west");
    field(&ctx, &mut a, "west:priority", "8192");
    let west_id = a
        .metrics
        .stp_fields
        .iter()
        .find(|x| x.0 == "west:priority")
        .unwrap()
        .2;
    assert_ne!(core_id, west_id);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).bridges["core"].priority, "4096");
    choose_bridge(&ctx, &mut a, "core");
    for _ in 0..4 {
        frame(&ctx, &mut a, 720., 700., vec![]);
    }
    assert_eq!(
        a.metrics
            .stp_fields
            .iter()
            .find(|x| x.0 == "core:priority")
            .unwrap()
            .2,
        core_id,
        "column reflow preserves edit ID"
    );
    a.set_spanning_tree_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let example_id = a.metrics.stp_fields[0].2;
    assert_ne!(example_id, core_id);
    a.focus_input = true;
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerButton {
            pos: egui::pos2(0., 0.),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }],
    );
    assert!(a.focus_input, "pointer press must defer focus");
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerButton {
            pos: egui::pos2(0., 0.),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut a);
    assert!(!a.focus_input);
}
#[test]
fn spanning_tree_native_saved_drafts_practice_favorite_resume_selected_reset_challenge() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    solved(&ctx, &mut a, i, "0");
    field(&ctx, &mut a, "core:priority", "pending");
    a.set_spanning_tree_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    field(&ctx, &mut a, "sample-core:priority", "unapplied");
    a.progress.favorites.insert(ID.into());
    let j = 0;
    a.launch(j);
    let other = serde_json::to_value(&a.progress.attempts[&a.scenarios[j].id]).unwrap();
    a.launch(i);
    let expected = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    a.progress = progress::decode(&serde_json::to_vec(&a.progress).unwrap(), &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
        expected,
        "exact full Attempt decode before navigation"
    );
    let mut after_navigation = expected.clone();
    after_navigation["visited"] = serde_json::json!(a.progress.clock + 1);
    a.launch(i);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
        after_navigation,
        "launch changes only visited clock"
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    assert_eq!(a.spanning_tree_view(i), teaching::View::Assignment);
    a.help_open = false;
    draw(&ctx, &mut a);
    assert!(!a.help_open);
    assert_eq!(draft(&a).bridges["core"].priority, "pending");
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert!(a.progress.favorites.contains(ID));
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[j].id]).unwrap(),
        other
    );
}
#[test]
fn spanning_tree_native_width_scale_mode_matrix_has_bounded_fields_shared_height() {
    for width in [720., 1268.] {
        for scale in [1., 1.5, 2.] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                let (ctx, mut a, _) = opened(view);
                a.scale = scale;
                for _ in 0..5 {
                    frame(&ctx, &mut a, width, 850., vec![]);
                }
                let rect = ctx.content_rect();
                for (_, r, _) in &a.metrics.stp_fields {
                    assert!(
                        r.left() >= rect.left() - 1. && r.right() <= rect.right() + 1.,
                        "{width}/{scale}/{view:?}: {r:?} outside {rect:?}"
                    );
                    assert!(r.height() > 0.);
                }
                if view != teaching::View::Introduction {
                    let h = a.metrics.stp_fields[0].1.height();
                    for (_, r, _) in &a.metrics.stp_fields {
                        assert!((r.height() - h).abs() < 1.);
                    }
                }
            }
        }
    }
}

#[test]
fn spanning_tree_native_mode_switch_retains_current_completion_and_closes_learn_help() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    solved(&ctx, &mut a, i, "0");
    a.help_open = true;
    let before = a.progress.attempts[ID].state.clone();
    let r = locate(&ctx, &mut a, |m| m.mode_challenge);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(a.progress.attempts[ID].challenge);
    assert!(!a.help_open);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    let r = locate(&ctx, &mut a, |m| m.mode_learn);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(!a.progress.attempts[ID].challenge);
    assert_eq!(a.progress.attempts[ID].state, before);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn spanning_tree_native_port_picker_edits_only_selected_port_and_enter_does_not_apply() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    choose_bridge(&ctx, &mut a, "core");
    let before = live(&a).clone();
    let r = locate(&ctx, &mut a, |m| m.stp_port);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = a
        .metrics
        .stp_port_items
        .iter()
        .find(|x| x.0 == "west")
        .unwrap()
        .1;
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).selected_port.as_deref(), Some("west"));
    field(&ctx, &mut a, "core/west:cost", "20");
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert_eq!(live(&a), &before);
    assert_eq!(draft(&a).bridges["core"].ports["west"].cost, "20");
    assert_eq!(draft(&a).bridges["core"].ports["east"].cost, "4");
    apply(&ctx, &mut a);
    assert_eq!(
        live(&a).configuration().bridges["core"].ports["west"].cost,
        20
    );
    assert_eq!(
        live(&a).configuration().bridges["core"].ports["east"].cost,
        4
    );
}

#[test]
fn spanning_tree_native_second_apply_marks_dirty_even_with_identical_notice_and_persists() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    solved(&ctx, &mut a, i, "0");
    let notice = draft(&a).notice.clone();
    field(&ctx, &mut a, "core:priority", "4096");
    a.save();
    assert!(!a.dirty, "normal autosave clears pending-draft dirty flag");
    let old = progress::load(&a.path, &a.scenarios).unwrap();
    let State::SpanningTree { simulation } = &old.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.configuration().bridges["core"].priority, 0);
    assert_eq!(
        old.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .spanning_tree_draft
            .as_ref()
            .unwrap()
            .bridges["core"]
            .priority,
        "4096"
    );
    let before = draft(&a).clone();
    apply(&ctx, &mut a);
    assert_eq!(
        draft(&a),
        &before,
        "same successful notice/raw fields; only simulation changes"
    );
    assert_eq!(draft(&a).notice, notice);
    assert!(a.dirty, "second actual Apply must request persistence");
    assert_eq!(live(&a).configuration().bridges["core"].priority, 4096);
    a.save();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(loaded.attempts[ID].state, a.progress.attempts[ID].state);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[ID]).unwrap(),
        serde_json::to_value(&a.progress.attempts[ID]).unwrap()
    );
}

// Exercise the same real render/input/SmokeTick path as native. The test only
// acknowledges requested screenshot events; it does not fabricate rendered PNGs.
fn capture_tick(ctx: &egui::Context, a: &mut LabApp) {
    capture_tick_sized(ctx, a, 1200., 850., 1.);
}
fn capture_tick_sized(ctx: &egui::Context, a: &mut LabApp, width: f32, height: f32, density: f32) {
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| match e {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or_default();
    a.capture_pending = false;
    let scale = a.scale;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / (scale * density), height / (scale * density)),
        )),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(density);
    let _ = ctx.run(input, |ctx| {
        a.render(ctx);
        // Match native update order: autosave precedes the controller.
        a.save();
        a.started = Instant::now() - std::time::Duration::from_secs(1);
        a.smoke_tick(ctx);
    });
}
#[test]
fn spanning_tree_native_measured_tiled_density_reveals_clipped_field_for_example_repair() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 33000;
    for _ in 0..1000 {
        capture_tick_sized(&ctx, &mut a, 946., 929., 1.15);
        if a.smoke_stage == 33020 {
            break;
        }
    }
    assert_eq!(
        a.smoke_stage, 33020,
        "real events must complete changed-example repair and reach assignment Check"
    );
    let s = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let practice = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .spanning_tree_learning
        .as_ref()
        .unwrap();
    assert!(
        practice
            .checks(s.spanning_tree_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, s.initial);
}

#[test]
fn spanning_tree_native_checked_example_does_not_pull_scroll_back_during_idle_frames() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    {
        let lesson = a.scenarios[i].spanning_tree_teaching.as_ref().unwrap();
        let learning = a
            .progress
            .attempts
            .get_mut(ID)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .spanning_tree_learning
            .as_mut()
            .unwrap();
        learning.fill_example(lesson).unwrap();
        learning.apply(lesson).unwrap();
        learning.checked = true;
    }
    a.lesson_scroll_reset = true;
    a.focus_input = false;
    for _ in 0..20 {
        frame(&ctx, &mut a, 822.6087, 807.8261, vec![]);
    }
    assert!(
        a.metrics.main_scroll_offset.y.abs() < 1.,
        "showing already checked feedback must not force a new scroll: {:?}",
        a.metrics.main_scroll_offset
    );
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .spanning_tree_learning
            .as_ref()
            .unwrap()
            .checked
    );
}
#[test]
fn spanning_tree_native_actual_capture_open_settle_and_first_example_flow() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 33000;
    let original = a.scenarios.len();
    capture_tick(&ctx, &mut a);
    assert_eq!(a.smoke_stage, 33001);
    assert_eq!(a.scenarios.len(), original + 1);
    assert_eq!(
        a.path.parent(),
        Some(a._smoke_profile.as_ref().unwrap().path())
    );
    assert_eq!(
        a.progress.attempts[ID].state,
        a.scenarios.last().unwrap().initial
    );
    for _ in 0..250 {
        capture_tick(&ctx, &mut a);
        if a.smoke_stage == 33009 {
            break;
        }
    }
    assert_eq!(
        a.smoke_stage, 33009,
        "actual Insert/Apply/Check queue reaches first example proof/capture"
    );
    let s = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .spanning_tree_learning
        .as_ref()
        .unwrap();
    assert!(l.checked);
    assert!(
        l.checks(s.spanning_tree_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, s.initial);
}
#[test]
fn spanning_tree_native_actual_finish_holds_when_close_is_delayed() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 33000;
    capture_tick(&ctx, &mut a);
    // LAST is a closed developer controller constant, never a learner mode.
    a.smoke_stage = super::super::smoke::spanning_tree::LAST;
    for _ in 0..2 {
        let stage = a.smoke_stage;
        capture_tick(&ctx, &mut a);
        assert_eq!(
            a.smoke_stage, stage,
            "delayed Close must not advance into an unhandled phase"
        );
    }
}

#[test]
fn spanning_tree_native_full_controller_autosaves_before_warm_resume_and_selected_reset() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 33000;
    let last = super::super::smoke::spanning_tree::LAST;
    for _ in 0..1500 {
        capture_tick(&ctx, &mut a);
        if a.smoke_stage == last {
            break;
        }
    }
    assert_eq!(
        a.smoke_stage, last,
        "entire actual-event controller must reach held Finish"
    );
    let s = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    assert_eq!(a.progress.attempts[ID].state, s.initial);
    assert!(a.progress.favorites.contains(ID));
    assert!(!progress::current_solved(&a.progress.attempts[ID], s));
}
