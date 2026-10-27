//! Event-driven technical fixture only. All execution is the offline NAT model.
use super::*;
use dlms_learning_lab::{nat as core, nat_workspace as work};
const ID: &str = "nat-workspace-preview";
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let s = pack::load(include_str!(
        "../../../tests/fixtures/nat-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn draft(a: &LabApp) -> &work::Draft {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .nat_draft
        .as_ref()
        .unwrap()
}
fn live(a: &LabApp) -> &core::State {
    let State::BasicNat { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn locate(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    let mut last = std::collections::VecDeque::with_capacity(3);
    for attempt in 0..40 {
        let (r, clip) = if field {
            let x = a
                .metrics
                .nat_fields
                .iter()
                .find(|x| x.0 == key)
                .expect("NAT field");
            (x.1, x.4)
        } else {
            let x = a
                .metrics
                .nat_controls
                .iter()
                .find(|x| x.0 == key)
                .expect("NAT control");
            (x.1, x.3)
        };
        // Text/Remove responses inherit a broad painter clip in columns. Wheel
        // events must instead target the actual static ScrollArea child.
        let child =
            if (field && key.starts_with("static:")) || (!field && key.starts_with("remove:")) {
                Some(
                    a.metrics
                        .nat_static_viewport
                        .expect("actual static-list viewport"),
                )
            } else {
                None
            };
        let clip = child.map_or(clip, |child| clip.intersect(child));
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let v = root.intersect(clip);
        let tol = 1. / ctx.pixels_per_point();
        if v.contains(r.center()) && v.expand(tol).contains_rect(r) {
            return r;
        }
        // Route to the actual inner list only when root already contains it.
        let inner = root.contains(clip.center()) && clip.height() < root.height() - 4.;
        let viewport = if inner { v } else { root };
        let y = if r.bottom() > viewport.bottom() {
            viewport.bottom() - r.bottom()
        } else {
            viewport.top() - r.top()
        };
        let pointer = if inner {
            viewport.center()
        } else {
            egui::pos2(root.left() + 1., root.center().y)
        };
        if last.len() == 3 {
            last.pop_front();
        }
        last.push_back(format!("try={attempt} target={r:?} clip={clip:?} child={child:?} root={root:?} intersection={v:?} inner={inner} pointer={pointer:?} wheel={:.2} main_offset={:?} ppp={:.3}",y.clamp(-350.,350.),a.metrics.main_scroll_offset,ctx.pixels_per_point()));
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-350., 350.)),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("NAT target {key} was not actually visible; last geometry: {last:?}")
}
// Header controls are in the actual outer scroller too. Raw retained rectangles
// may be offscreen after editing a policy field, so reveal before any real click.
fn header(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    target: fn(&Metrics) -> Option<egui::Rect>,
) {
    let mut last = None;
    for attempt in 0..40 {
        let r = target(&a.metrics).expect("existing NAT header control");
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let tolerance = 1. / ctx.pixels_per_point();
        if root.contains(r.center()) && root.expand(tolerance).contains_rect(r) {
            click(ctx, a, r);
            draw(ctx, a);
            return;
        }
        let delta = if r.bottom() > root.bottom() {
            root.bottom() - r.bottom()
        } else {
            root.top() - r.top()
        };
        last = Some(format!(
            "try={attempt} target={r:?} root={root:?} main_offset={:?} wheel={:.2}",
            a.metrics.main_scroll_offset,
            delta.clamp(-350., 350.)
        ));
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-350., 350.)),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("NAT header {key} never became actually visible: {last:?}")
}
fn ctrl(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = locate(ctx, a, key, false);
    click(ctx, a, r);
    draw(ctx, a);
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = locate(ctx, a, key, true);
    click(ctx, a, r);
    a.focus_input = false;
    let id = a.metrics.nat_fields.iter().find(|x| x.0 == key).unwrap().2;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(value.into())],
    );
    draw(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    ctrl(ctx, a, key);
    for _ in 0..20 {
        let r = a
            .metrics
            .nat_choices
            .iter()
            .find(|x| x.0 == format!("{key}:{value}"))
            .unwrap()
            .1;
        let clip = a.metrics.nat_popup.unwrap();
        if clip.contains_rect(r) {
            click(ctx, a, r);
            draw(ctx, a);
            return;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        if r.bottom() > clip.bottom() {
                            -160.
                        } else {
                            160.
                        },
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("NAT option never visible")
}
fn solve_events(ctx: &egui::Context, a: &mut LabApp) {
    field(ctx, a, "network", "10.0.0.8");
    field(ctx, a, "prefix", "30");
    ctrl(ctx, a, "apply");
    choose(ctx, a, "inside", "alpha");
    ctrl(ctx, a, "outbound");
    choose(ctx, a, "inside", "beta");
    ctrl(ctx, a, "outbound");
    ctrl(ctx, a, "check");
}
#[test]
fn nat_actual_policy_events_allocate_only_on_outbound_check_is_readonly_and_clear_breaks_again() {
    let (ctx, mut a, i) = opened();
    let before = live(&a).clone();
    ctrl(&ctx, &mut a, "check");
    assert_eq!(live(&a), &before);
    field(&ctx, &mut a, "network", "10.0.0.8");
    field(&ctx, &mut a, "prefix", "30");
    ctrl(&ctx, &mut a, "apply");
    assert!(live(&a).bindings().is_empty());
    choose(&ctx, &mut a, "inside", "alpha");
    ctrl(&ctx, &mut a, "outbound");
    assert_eq!(live(&a).bindings().len(), 1);
    let g = live(&a).bindings()["alpha"].to_string();
    field(&ctx, &mut a, "inbound-global", &g);
    let before = live(&a).clone();
    ctrl(&ctx, &mut a, "inbound");
    assert_eq!(live(&a), &before);
    choose(&ctx, &mut a, "inside", "beta");
    ctrl(&ctx, &mut a, "outbound");
    ctrl(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    let c = a.nat_presentation.memo.counters;
    draw(&ctx, &mut a);
    assert_eq!(a.nat_presentation.memo.counters.evaluations, c.evaluations);
    assert_eq!(a.nat_presentation.memo.counters.meters, c.meters);
    assert_eq!(a.nat_presentation.memo.counters.snapshots, c.snapshots);
    ctrl(&ctx, &mut a, "clear");
    assert!(live(&a).bindings().is_empty());
    assert_eq!(live(&a).configuration().statics.len(), 1);
    assert_eq!(a.nat_presentation.memo.prepared(ID), Some(false));
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn nat_pending_malformed_edits_atomic_apply_and_readonly_bad_request_preserve_truth() {
    let (ctx, mut a, i) = opened();
    solve_events(&ctx, &mut a);
    let before = live(&a).clone();
    field(&ctx, &mut a, "inbound-global", "not an address");
    ctrl(&ctx, &mut a, "inbound");
    assert_eq!(live(&a), &before);
    assert_eq!(a.nat_presentation.memo.prepared(ID), Some(true));
    field(&ctx, &mut a, "prefix", "bad");
    assert_eq!(a.nat_presentation.memo.prepared(ID), Some(false));
    ctrl(&ctx, &mut a, "apply");
    assert_eq!(live(&a), &before);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "prefix", "30");
    assert_eq!(a.nat_presentation.memo.prepared(ID), Some(true));
    // A same-success-notice Apply still declares save intent after an earlier save.
    a.save();
    assert!(!a.dirty);
    let r = locate(&ctx, &mut a, "apply", false);
    click(&ctx, &mut a, r);
    assert!(a.dirty);
}
#[test]
fn nat_library_cards_counts_filters_and_readiness_reuse_one_prepass_and_exact_replacements_miss() {
    let (ctx, mut a, i) = opened();
    solve_events(&ctx, &mut a);
    a.scenarios[i].unit = "test-nat-unit".into();
    a.units.push(course::Unit {
        id: "test-nat-unit".into(),
        track: "CCNA".into(),
        title: "Technical NAT".into(),
        summary: "Test fixture".into(),
        prerequisites: vec![],
        labs: vec![ID.into()],
    });
    a.selected = None;
    a.filter.unit = "test-nat-unit".into();
    a.filter.status = "Complete".into();
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    let counters = a.nat_presentation.memo.counters;
    for _ in 0..8 {
        draw(&ctx, &mut a);
        let attempt = &a.progress.attempts[ID];
        assert!(a.visible_solved(attempt, &a.scenarios[i]));
    }
    assert_eq!(
        a.nat_presentation.memo.counters.evaluations,
        counters.evaluations
    );
    assert_eq!(a.nat_presentation.memo.counters.meters, counters.meters);
    assert_eq!(
        a.nat_presentation.memo.counters.snapshots,
        counters.snapshots
    );
    a.progress
        .attempts
        .get_mut(ID)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .version = 2;
    draw(&ctx, &mut a);
    assert!(a.metrics.card_actions.is_empty());
    assert_eq!(a.nat_presentation.memo.prepared(ID), Some(false));
    a.progress
        .attempts
        .get_mut(ID)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .version = 1;
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    let engine::Goal::BasicNat { definition } = &mut a.scenarios[i].goal else {
        panic!()
    };
    definition
        .requirements
        .push(work::Requirement::NoCurrentMapping {
            inside: "alpha".into(),
        });
    draw(&ctx, &mut a);
    assert!(a.metrics.card_actions.is_empty());
}
#[test]
fn nat_modal_keyboard_and_controls_do_not_mutate_the_background_or_steal_focus() {
    for modal in 0..4 {
        let (ctx, mut a, _) = opened();
        field(&ctx, &mut a, "network", "10.0.0.8");
        let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
        match modal {
            0 => a.reset_pending = true,
            1 => a.settings_open = true,
            2 => a.theme_preview = true,
            _ => {
                a.settings_open = true;
                a.settings_data = true;
                draw(&ctx, &mut a);
                let r = a.metrics.data_export.unwrap();
                click(&ctx, &mut a, r);
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
                egui::Event::Text("hidden edit".into()),
                acl_ctrl(egui::Key::Enter),
            ],
        );
        draw(&ctx, &mut a);
        assert!(a.metrics.nat_fields.iter().all(|r| !r.3));
        assert!(a.metrics.nat_controls.iter().all(|r| !r.2));
        assert!(
            serde_json::to_value(&a.progress.attempts[ID]).unwrap() == before,
            "modal {modal} mutated saved attempt"
        );
    }
}
#[test]
fn nat_real_paste_undo_and_row_tokens_survive_reflow_and_isolate_removed_rows() {
    let (ctx, mut a, _) = opened();
    field(&ctx, &mut a, "network", "10.0.0.8");
    let old = draft(&a).network.clone();
    let r = locate(&ctx, &mut a, "network", true);
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste("é".repeat(40))],
    );
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).network, old);
    let id = a
        .metrics
        .nat_fields
        .iter()
        .find(|r| r.0 == "network")
        .unwrap()
        .2;
    frame(&ctx, &mut a, 650., 850., vec![]);
    assert_eq!(
        a.metrics
            .nat_fields
            .iter()
            .find(|r| r.0 == "network")
            .unwrap()
            .2,
        id
    );
    assert!(ctx.memory(|m| m.has_focus(id)));
    draw(&ctx, &mut a);
    ctrl(&ctx, &mut a, "add-static");
    let tokens: Vec<_> = a
        .metrics
        .nat_fields
        .iter()
        .filter(|r| r.0.ends_with(":inside"))
        .map(|r| r.0.clone())
        .collect();
    assert_eq!(tokens.len(), 2);
    let second = tokens[1].clone();
    field(&ctx, &mut a, &second, "unused");
    let second_id = a
        .metrics
        .nat_fields
        .iter()
        .find(|r| r.0 == second)
        .unwrap()
        .2;
    ctrl(&ctx, &mut a, "remove:0");
    assert_eq!(draft(&a).statics.len(), 1);
    assert_eq!(draft(&a).statics[0].inside, "unused");
    assert_eq!(
        a.metrics
            .nat_fields
            .iter()
            .find(|r| r.0 == second)
            .unwrap()
            .2,
        second_id
    );
    ctrl(&ctx, &mut a, "add-static");
    let new = a
        .metrics
        .nat_fields
        .iter()
        .rfind(|r| r.0.ends_with(":inside"))
        .unwrap();
    assert_ne!(new.2, second_id);
    let new_key = new.0.clone();
    field(&ctx, &mut a, &new_key, "alpha");
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).statics[0].inside, "unused");
}
#[test]
fn nat_scale_controls_use_measured_heights_and_painted_text_widths_without_field_id_churn() {
    let (ctx, mut a, _) = opened();
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        for width in [650., 1200.] {
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 850., vec![]);
            }
            assert!(a.metrics.nat_fields.iter().all(|r| r.1.is_finite()
                && r.1.width() > 0.
                && r.1.height() >= ctx.style().spacing.interact_size.y - 1.));
            let texts = [
                ("apply", "Apply policy"),
                ("outbound", "Outbound"),
                ("inbound", "Reverse"),
                ("clear", "Clear dynamic maps"),
                ("check", "Check work"),
            ];
            for (key, text) in texts {
                let r = a
                    .metrics
                    .nat_controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap()
                    .1;
                let font = egui::TextStyle::Button.resolve(&ctx.style());
                let galley =
                    ctx.fonts_mut(|f| f.layout_no_wrap(text.into(), font, egui::Color32::WHITE));
                assert!(
                    r.width() + 1. >= galley.size().x + 2. * ctx.style().spacing.button_padding.x
                );
            }
        }
    }
}
#[test]
fn nat_save_resume_reset_and_challenge_keep_real_state_pending_draft_favorites_and_old140_intact() {
    let (ctx, mut a, i) = opened();
    let baseline: Vec<_> = a.scenarios[..i]
        .iter()
        .map(|s| serde_json::to_value(s).unwrap())
        .collect();
    solve_events(&ctx, &mut a);
    field(&ctx, &mut a, "network", "pending invalid text");
    a.progress.favorites.insert(ID.into());
    let expected = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    a.dirty = true;
    a.save();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded.attempts[ID]).unwrap() == expected);
    assert!(loaded.favorites.contains(ID));
    a.progress = loaded;
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).network, "pending invalid text");
    assert_eq!(live(&a).bindings().len(), 2);
    // Real header controls, not assignment/state replacement.
    header(&ctx, &mut a, "challenge", |m| m.mode_challenge);
    assert!(a.progress.attempts[ID].challenge);
    assert_eq!(live(&a).bindings().len(), 2);
    header(&ctx, &mut a, "reset", |m| m.reset);
    header(&ctx, &mut a, "cancel-reset", |m| m.reset_cancel);
    assert_eq!(live(&a).bindings().len(), 2);
    header(&ctx, &mut a, "reset", |m| m.reset);
    header(&ctx, &mut a, "confirm-reset", |m| m.confirm_reset);
    assert!(live(&a).bindings().is_empty());
    assert!(a.progress.favorites.contains(ID));
    assert!(
        a.scenarios[..i]
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>()
            == baseline,
        "Complete original140 typed specs must remain equal"
    );
}

#[test]
fn nat_disabled_invalid_text_is_retained_and_only_enabled_fields_are_applied() {
    let (ctx, mut a, i) = opened();
    field(&ctx, &mut a, "prefix", "bad");
    ctrl(&ctx, &mut a, "dynamic");
    assert!(!draft(&a).dynamic_enabled);
    assert!(
        a.metrics
            .nat_fields
            .iter()
            .all(|r| !["network", "prefix", "first", "last"].contains(&r.0.as_str()))
    );
    ctrl(&ctx, &mut a, "apply");
    assert!(live(&a).configuration().dynamic.is_none());
    assert_eq!(draft(&a).prefix, "bad");
    let rows = progress::current_nat_checks(&a.progress.attempts[ID], &a.scenarios[i]);
    assert!(
        rows[0].0,
        "Disabled invalid raw text must not make a pending policy"
    );
    assert!(
        !rows.iter().all(|r| r.0),
        "The assignment's requested dynamic outcomes are still unmet"
    );
    let before = live(&a).clone();
    ctrl(&ctx, &mut a, "dynamic");
    assert!(draft(&a).dynamic_enabled);
    assert_eq!(
        draft(&a).prefix,
        "bad",
        "Re-enable restores exact retained invalid text"
    );
    assert!(
        ["network", "prefix", "first", "last"].iter().all(|key| a
            .metrics
            .nat_fields
            .iter()
            .any(|r| r.0 == *key && r.3))
    );
    ctrl(&ctx, &mut a, "apply");
    assert_eq!(live(&a), &before);
    assert!(!progress::current_nat_checks(&a.progress.attempts[ID], &a.scenarios[i])[0].0);
}
