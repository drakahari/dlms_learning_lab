//! Private capability-fixture widget tests, not normal curriculum/native acceptance.
use super::*;
use dlms_learning_lab::{engine::Goal, hsrp_workspace as work};
#[path = "../../../tests/support/hsrp_fixture.rs"]
mod fixture;
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.len();
    assert_eq!(i, 201);
    let old199=one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::literal_registration::snapshot_registration::before_scenarios(&a.scenarios);
    assert_eq!(old199.len(), 199);
    let old197=one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::literal_registration::before_scenarios(&a.scenarios);
    assert_eq!(old197.len(), 197);
    let checked193 = one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::before_scenarios(&a.scenarios);
    assert_eq!(checked193.len(), 193);
    let before = one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(checked193);
    assert_eq!(before.len(), 190);
    a.scenarios.push(fixture::scenario());
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::Hsrp { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn target(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    input: bool,
    w: f32,
    h: f32,
) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = if input {
            let r = a.metrics.hsrp.fields.iter().find(|r| r.0 == key).unwrap();
            (r.1, r.3)
        } else {
            let r = a.metrics.hsrp.controls.iter().find(|r| r.0 == key).unwrap();
            (r.1, r.2)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let clip = clip.intersect(root);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
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
        for _ in 0..10 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("Rendered HSRP {key} did not become visible within20 real wheels")
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, w: f32, h: f32) {
    frame(ctx, a, w, h, vec![egui::Event::PointerMoved(r.center())]);
    frame(
        ctx,
        a,
        w,
        h,
        [true, false]
            .map(|pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            })
            .to_vec(),
    );
    for _ in 0..2 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false, 1200., 929.);
    click(ctx, a, r, 1200., 929.);
}
fn text(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str, w: f32, h: f32) {
    let r = target(ctx, a, key, true, w, h);
    click(ctx, a, r, w, h);
    let id = a.metrics.hsrp.fields.iter().find(|r| r.0 == key).unwrap().2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let mut events = [true, false]
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .to_vec();
    events.push(egui::Event::Paste(value.into()));
    frame(ctx, a, w, h, events);
    frame(ctx, a, w, h, vec![]);
}
fn solve(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, "assignment");
    tap(ctx, a, "router-B-participating");
    for key in ["client-1", "client-2"] {
        text(ctx, a, key, "192.0.2.254", 1200., 929.);
    }
    assert!(!a.metrics.completion);
    tap(ctx, a, "apply");
    tap(ctx, a, "check");
    let (total, failed, _) = a.metrics.hsrp.summary.unwrap();
    assert_eq!((total, failed), (18, 0));
    assert!(a.metrics.completion);
}
#[test]
fn real_router_client_edits_apply_then_check_and_separate_example_insert_require_actual_application()
 {
    let (ctx, mut a, i) = opened();
    assert_eq!(session(&a).view, teaching::View::Introduction);
    let before = session(&a).current.clone();
    tap(&ctx, &mut a, "example");
    for n in 0..3 {
        tap(&ctx, &mut a, "worked-picker");
        tap(&ctx, &mut a, &format!("worked-choice-{n}"));
        let unchanged = session(&a).example.clone();
        tap(&ctx, &mut a, "insert");
        assert_eq!(session(&a).example, unchanged);
        tap(&ctx, &mut a, "apply");
        assert_eq!(session(&a).current, before);
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.hsrp.summary.map(|x| (x.0, x.1)), Some((18, 0)));
    assert!(!a.metrics.completion);
    let practice = session(&a).example.clone();
    solve(&ctx, &mut a);
    assert_eq!(session(&a).example, practice);
    assert_eq!(session(&a).history.len(), 3);
    assert!(progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn ascii_utf8_or_ignored_packet_with_apply_keeps_exact_draft_model_file_and_native_undo() {
    for mode in 0..3 {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        text(&ctx, &mut a, "router-A-priority", "110", 1200., 929.);
        let field = target(&ctx, &mut a, "router-A-priority", true, 1200., 929.);
        click(&ctx, &mut a, field, 1200., 929.);
        let id = a
            .metrics
            .hsrp
            .fields
            .iter()
            .find(|x| x.0 == "router-A-priority")
            .unwrap()
            .2;
        if mode == 2 {
            tap(&ctx, &mut a, "goal");
        }
        let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        let cursor = native.cursor.char_range();
        let undo = format!("{:?}", native.undoer());
        a.save();
        let raw = std::fs::read(&a.path).unwrap();
        let before = serde_json::to_value(&a.progress).unwrap();
        let apply = target(&ctx, &mut a, "apply", false, 1200., 929.);
        frame(
            &ctx,
            &mut a,
            1200.,
            929.,
            vec![egui::Event::PointerMoved(apply.center())],
        );
        let mut events = vec![egui::Event::Paste(if mode == 0 {
            format!("110{} invalid", " ".repeat(16))
        } else if mode == 1 {
            "é".repeat(16)
        } else {
            "9".repeat(17)
        })];
        events.extend([true, false].map(|pressed| egui::Event::PointerButton {
            pos: apply.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }));
        frame(&ctx, &mut a, 1200., 929., events);
        frame(&ctx, &mut a, 1200., 929., vec![]);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        let retained = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        assert_eq!(retained.cursor.char_range(), cursor);
        assert_eq!(format!("{:?}", retained.undoer()), undo);
        a.save();
        assert_eq!(std::fs::read(&a.path).unwrap(), raw);
        assert!(a.notice.is_some());
        assert!(session(&a).history.is_empty());
    }
}
#[test]
fn shared_topology_and_aligned_native_fields_remain_editable_without_geometry_changes_at_all20_settings()
 {
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for name in ["dlms-light", "dlms-dark"] {
            for width in [760., 1600.] {
                let (ctx, mut a, _) = opened();
                tap(&ctx, &mut a, "assignment");
                a.scale = scale;
                a.theme_id = name.into();
                theme::apply(&ctx, name);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let (_, clip) = a.metrics.hsrp.topology.unwrap();
                assert!(clip.is_positive());
                text(&ctx, &mut a, "router-A-priority", "130", width, 929.);
                let (_, r, id, clip) = a
                    .metrics
                    .hsrp
                    .fields
                    .iter()
                    .find(|x| x.0 == "router-A-priority")
                    .unwrap();
                let tolerance = 1. / ctx.pixels_per_point();
                assert!(clip.expand(tolerance).contains_rect(*r));
                let font = egui::TextStyle::Body.resolve(&ctx.style());
                let line = ctx.fonts_mut(|f| f.row_height(&font));
                assert!(r.height() >= line);
                assert_eq!(ctx.memory(|m| m.focused()), Some(*id));
                assert_eq!(session(&a).draft.routers[0].priority, "130");
                assert_eq!(
                    session(&a)
                        .current
                        .router(dlms_learning_lab::hsrp::RouterId::A)
                        .config
                        .priority,
                    110
                );
                for (_, r, _, clip) in &a.metrics.hsrp.fields {
                    assert!(
                        r.left() >= clip.left() - tolerance
                            && r.right() <= clip.right() + tolerance
                    );
                }
            }
        }
    }
}
#[test]
fn temporary_failure_return_never_replace_configured_history_and_selected_reset_keeps_other_work() {
    let (ctx, mut a, i) = opened();
    solve(&ctx, &mut a);
    let before = serde_json::to_value(&a.progress).unwrap();
    tap(&ctx, &mut a, "inspection");
    tap(&ctx, &mut a, "probe-A-power-false");
    tap(&ctx, &mut a, "probe-A-power-true");
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert!(a.metrics.hsrp.probe.is_some());
    assert_eq!(
        session(&a).current.active(),
        Some(dlms_learning_lab::hsrp::RouterId::A)
    );
    let Goal::Hsrp { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let observed = session(&a)
        .inspect(definition, &a.hsrp_presentation.events, false)
        .unwrap();
    assert_eq!(
        observed.active(),
        Some(dlms_learning_lab::hsrp::RouterId::B)
    );
    assert!(a.metrics.completion);
    a.launch(182);
    draw(&ctx, &mut a);
    let other = serde_json::to_value(&a.progress.attempts[&a.scenarios[182].id]).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    let before = serde_json::to_value(&a.progress).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    accounts_ui_click(&ctx, &mut a, |m| m.reset_cancel);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    accounts_ui_click(&ctx, &mut a, |m| m.confirm_reset);
    assert_eq!(session(&a).current, session(&a).initial);
    assert!(session(&a).history.is_empty());
    assert!(a.hsrp_presentation.events.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[182].id]).unwrap(),
        other
    );
    assert!(!a.metrics.completion);
}

#[test]
fn raw_control_post_enter_and_aggregate_overflow_packets_roll_back_before_clean_apply() {
    for same_frame_apply in [false, true] {
        for mode in 0..6 {
            let (ctx, mut a, _) = opened();
            tap(&ctx, &mut a, "assignment");
            text(&ctx, &mut a, "router-A-priority", "130", 1200., 929.);
            let field = target(&ctx, &mut a, "router-A-priority", true, 1200., 929.);
            click(&ctx, &mut a, field, 1200., 929.);
            let id = a
                .metrics
                .hsrp
                .fields
                .iter()
                .find(|x| x.0 == "router-A-priority")
                .unwrap()
                .2;
            let modifiers = egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            };
            frame(
                &ctx,
                &mut a,
                1200.,
                929.,
                [true, false]
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    })
                    .to_vec(),
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
            let cursor = native.cursor.char_range();
            let undo = format!("{:?}", native.undoer());
            a.save();
            let raw = std::fs::read(&a.path).unwrap();
            let before = serde_json::to_value(&a.progress).unwrap();
            let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
            let apply = target(&ctx, &mut a, "apply", false, 1200., 929.);
            frame(
                &ctx,
                &mut a,
                1200.,
                929.,
                vec![egui::Event::PointerMoved(apply.center())],
            );
            let enter = egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            };
            let mut events = match mode {
                0 => vec![egui::Event::Paste(" 110\n".into())],
                1 => vec![egui::Event::Paste(" 110\r".into())],
                2 => vec![egui::Event::Text(" 110\t".into())],
                3 => vec![
                    egui::Event::Text("1".into()),
                    enter.clone(),
                    egui::Event::Text("2".into()),
                ],
                4 => vec![enter.clone(), egui::Event::Paste("2".into())],
                _ => vec![
                    egui::Event::Paste("12345678".into()),
                    egui::Event::Paste("123456789".into()),
                ],
            };
            if mode == 3 || mode == 4 {
                events.push(egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
            if same_frame_apply {
                events.extend([true, false].map(|pressed| egui::Event::PointerButton {
                    pos: apply.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                }));
            } else {
                // No PointerButton: the focused field must receive this packet.
                // A same-frame click would surrender focus before text delivery,
                // letting the older any-receipt guard pass this test vacuously.
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            }
            frame(&ctx, &mut a, 1200., 929., events);
            frame(&ctx, &mut a, 1200., 929., vec![]);
            assert_eq!(
                serde_json::to_value(&a.progress).unwrap(),
                before,
                "Packet mode {mode}"
            );
            assert_eq!(
                serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                receipts
            );
            let retained = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
            assert_eq!(retained.cursor.char_range(), cursor);
            assert_eq!(format!("{:?}", retained.undoer()), undo);
            a.save();
            assert_eq!(std::fs::read(&a.path).unwrap(), raw);
            assert!(a.notice.is_some());
            assert!(session(&a).history.is_empty());
            assert_eq!(session(&a).draft.routers[0].priority, "130");
            // Refusal must not trap the learner: a later clean native gesture works.
            text(&ctx, &mut a, "router-A-priority", "140", 1200., 929.);
            tap(&ctx, &mut a, "apply");
            assert_eq!(
                session(&a)
                    .current
                    .router(dlms_learning_lab::hsrp::RouterId::A)
                    .config
                    .priority,
                140
            );
            assert_eq!(session(&a).history.len(), 1);
        }
    }
}

#[path = "../../../tests/support/hsrp_capacity.rs"]
mod capacity;
fn portable(a: &LabApp) -> serde_json::Value {
    serde_json::json!({"progress":a.progress,"receipts":a.progress.migration_receipts})
}
#[test]
fn native_complete_context_refusal_preserves_full_work_cursor_undo_success_and_hint_digits() {
    let (ctx, mut a, i) = opened();
    solve(&ctx, &mut a);
    let source = capacity::source(a.scenarios[i].clone());
    let id = source.id.clone();
    a.scenarios[i] = source.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    attempt.hints = 9;
    capacity::fill(&source, attempt, work::MAX_CONTEXT);
    a.dirty = false;
    frame(&ctx, &mut a, 1200., 929., vec![]);
    let r = target(&ctx, &mut a, "client-1", true, 1200., 929.);
    click(&ctx, &mut a, r, 1200., 929.);
    let field = a
        .metrics
        .hsrp
        .fields
        .iter()
        .find(|r| r.0 == "client-1")
        .unwrap()
        .2;
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        929.,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            })
            .collect(),
    );
    let before = portable(&a);
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    let cursor = native.cursor.char_range();
    type UndoTrace = (
        Vec<(egui::text::CCursorRange, String)>,
        Vec<(egui::text::CCursorRange, String)>,
        bool,
    );
    fn trace(state: &egui::text_edit::TextEditState) -> UndoTrace {
        let current = (state.cursor.char_range().unwrap(), "192.0.2.254".to_owned());
        let mut past = vec![];
        let mut undo = state.undoer();
        let mut now = current.clone();
        while let Some(v) = undo.undo(&now).cloned() {
            now = v.clone();
            past.push(v);
            assert!(past.len() <= 100);
        }
        let mut future = vec![];
        let mut redo = state.undoer();
        let mut now = current;
        while let Some(v) = redo.redo(&now).cloned() {
            now = v.clone();
            future.push(v);
            assert!(future.len() <= 100);
        }
        (past, future, state.undoer().is_in_flux())
    }
    let undo = trace(&native);
    frame(
        &ctx,
        &mut a,
        1200.,
        929.,
        vec![egui::Event::Paste("192.0.2.254x".into())],
    );
    frame(&ctx, &mut a, 1200., 929., vec![]);
    assert_eq!(portable(&a), before);
    assert!(a.metrics.completion && session(&a).checked);
    assert!(!a.dirty);
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    assert_eq!(native.cursor.char_range(), cursor);
    assert_eq!(trace(&native), undo);
    assert!(a.notice.as_ref().unwrap().contains("cannot fit"));
    tap(&ctx, &mut a, "optional-help");
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(portable(&a), before);
    assert_eq!(a.progress.attempts[&id].hints, 9);
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .transcript
        .remove(0);
    text(&ctx, &mut a, "client-1", "192.0.2.254x", 1200., 929.);
    assert!(!a.metrics.completion);
    assert!(a.dirty);
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.progress.attempts[&id].hints, 10);
}
#[test]
fn native_mode_refusal_keeps_prior_dirty_and_later_clean_retry_succeeds() {
    fn mode(ctx: &egui::Context, a: &mut LabApp, challenge: bool) {
        for _ in 0..20 {
            let r = if challenge {
                a.metrics.mode_challenge.unwrap()
            } else {
                a.metrics.mode_learn.unwrap()
            };
            let root = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
                click(ctx, a, r, 1200., 929.);
                return;
            }
            frame(
                ctx,
                a,
                1200.,
                929.,
                vec![egui::Event::PointerMoved(egui::pos2(
                    root.left() + 2.,
                    root.center().y,
                ))],
            );
            frame(
                ctx,
                a,
                1200.,
                929.,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                }],
            );
            for _ in 0..10 {
                frame(ctx, a, 1200., 929., vec![]);
            }
        }
        panic!("Bounded real header wheels must reach mode");
    }
    for dirty in [false, true] {
        let (ctx, mut a, i) = opened();
        solve(&ctx, &mut a);
        mode(&ctx, &mut a, true);
        let source = capacity::source(a.scenarios[i].clone());
        let id = source.id.clone();
        a.scenarios[i] = source.clone();
        capacity::fill(
            &source,
            a.progress.attempts.get_mut(&id).unwrap(),
            work::MAX_CONTEXT,
        );
        a.dirty = dirty;
        frame(&ctx, &mut a, 1200., 929., vec![]);
        let before = portable(&a);
        mode(&ctx, &mut a, false);
        assert_eq!(portable(&a), before);
        assert_eq!(a.dirty, dirty);
        assert!(a.progress.attempts[&id].challenge && a.metrics.completion);
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .transcript
            .remove(0);
        let mut expected = portable(&a);
        expected["progress"]["attempts"][&id]["challenge"] = false.into();
        mode(&ctx, &mut a, false);
        assert_eq!(portable(&a), expected);
        assert!(a.dirty && a.metrics.completion);
    }
}
#[test]
fn real_guidance_press_click_and_held_release_veto_old_apply_check_and_probe_intents() {
    fn focus(ctx: &egui::Context, a: &mut LabApp, key: &str) {
        for _ in 0..96 {
            let r = a
                .metrics
                .hsrp
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1;
            if ctx
                .memory(|m| m.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|response| response.rect == r)
            {
                return;
            }
            frame(
                ctx,
                a,
                1200.,
                929.,
                [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    })
                    .collect(),
            );
        }
        panic!("Real Tab must reach {key}");
    }
    for header in [
        "inspection",
        "optional-help",
        "worked-approach",
        "references",
    ] {
        for action in ["apply", "check", "probe-A-power-false"] {
            if header == "inspection" && action == "probe-A-power-false" {
                continue;
            }
            for held in [false, true] {
                let (ctx, mut a, i) = opened();
                tap(&ctx, &mut a, "assignment");
                tap(&ctx, &mut a, "router-B-participating");
                for key in ["client-1", "client-2"] {
                    text(&ctx, &mut a, key, "192.0.2.254", 1200., 929.);
                }
                if action == "probe-A-power-false" {
                    tap(&ctx, &mut a, "inspection");
                }
                if matches!(header, "worked-approach" | "references") {
                    tap(&ctx, &mut a, "optional-help");
                }
                focus(&ctx, &mut a, action);
                let r = target(&ctx, &mut a, header, false, 1200., 929.);
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    929.,
                    vec![egui::Event::PointerMoved(r.center())],
                );
                let before = portable(&a);
                let events_before = a.hsrp_presentation.events.clone();
                let key = |pressed| egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                };
                let pointer = |pressed| egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                };
                let mut events = vec![key(true), pointer(true)];
                if !held {
                    events.extend([pointer(false), key(false)]);
                }
                assert!(
                    !events
                        .iter()
                        .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
                );
                frame(&ctx, &mut a, 1200., 929., events);
                assert!(a.metrics.hsrp.guidance);
                let raw = a.metrics.hsrp.intents;
                assert!(
                    match action {
                        "apply" => raw.0,
                        "check" => raw.1,
                        _ => raw.2,
                    },
                    "Actual focused {action} intent must be observed before {header} veto"
                );
                assert_eq!(portable(&a), before);
                assert_eq!(a.hsrp_presentation.events, events_before);
                assert_eq!(a.selected, Some(i));
                if held {
                    frame(&ctx, &mut a, 1200., 929., vec![pointer(false), key(false)]);
                    assert_eq!(portable(&a), before);
                    assert_eq!(a.hsrp_presentation.events, events_before);
                }
                frame(&ctx, &mut a, 1200., 929., vec![]);
                tap(&ctx, &mut a, action);
                if action == "apply" {
                    assert!(a.metrics.completion);
                } else if action == "check" {
                    assert!(session(&a).checked);
                } else {
                    assert_eq!(a.hsrp_presentation.events.len(), 1);
                }
            }
        }
    }
}
#[test]
fn repeated_native_frames_keep_field_ids_bounded_and_separate_practice_undo_and_data() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    text(&ctx, &mut a, "router-A-priority", "130", 1200., 929.);
    let before = portable(&a);
    let fields = a.hsrp_presentation.fields.clone();
    let source_field = a
        .metrics
        .hsrp
        .fields
        .iter()
        .find(|r| r.0 == "router-A-priority")
        .unwrap()
        .2;
    for _ in 0..160 {
        frame(&ctx, &mut a, 1200., 929., vec![]);
        assert_eq!(a.hsrp_presentation.fields, fields);
        assert!(a.hsrp_presentation.fields.len() <= 4 + dlms_learning_lab::hsrp::MAX_CLIENTS);
    }
    assert_eq!(portable(&a), before);
    tap(&ctx, &mut a, "example");
    let example_field = target(&ctx, &mut a, "router-A-priority", true, 1200., 929.);
    click(&ctx, &mut a, example_field, 1200., 929.);
    let other_id = a
        .metrics
        .hsrp
        .fields
        .iter()
        .find(|r| r.0 == "router-A-priority")
        .unwrap()
        .2;
    assert_ne!(source_field, other_id);
    let snapshot = portable(&a);
    frame(
        &ctx,
        &mut a,
        1200.,
        929.,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            })
            .collect(),
    );
    assert_eq!(portable(&a), snapshot);
    tap(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).draft.routers[0].priority, "130");
    assert!(a.hsrp_presentation.fields.contains(&source_field));
    // This verifies the existing clear-per-render lifecycle; no claimed Vec leak.
}

#[test]
fn requested_guidance_suppresses_genuinely_available_next_until_a_clean_later_action() {
    fn focus_next(ctx: &egui::Context, a: &mut LabApp) {
        for _ in 0..96 {
            let r = a.metrics.next_unit.unwrap();
            if ctx
                .memory(|m| m.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|response| response.rect == r)
            {
                return;
            }
            frame(
                ctx,
                a,
                1200.,
                929.,
                [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    })
                    .collect(),
            );
        }
        panic!("Real Tab must reach actual Next");
    }
    for header in [
        "inspection",
        "optional-help",
        "worked-approach",
        "references",
    ] {
        for held in [false, true] {
            let (ctx, mut a, i) = opened();
            solve(&ctx, &mut a);
            let id = a.scenarios[i].id.clone();
            a.scenarios[i].unit = "dev-hsrp-guidance-unit".into();
            let mut peer = a.scenarios[i].clone();
            peer.id = "dev-hsrp-guidance-peer".into();
            let peer_i = a.scenarios.len();
            a.scenarios.push(peer.clone());
            a.units.push(course::Unit {
                id: "dev-hsrp-guidance-unit".into(),
                track: "CCNA".into(),
                title: "Private guidance route".into(),
                summary: "Private fixture only".into(),
                prerequisites: vec![],
                labs: vec![id.clone(), peer.id.clone()],
            });
            frame(&ctx, &mut a, 1200., 929., vec![]);
            assert!(a.metrics.next_unit.is_some());
            if matches!(header, "worked-approach" | "references") {
                tap(&ctx, &mut a, "optional-help");
            }
            focus_next(&ctx, &mut a);
            let r = target(&ctx, &mut a, header, false, 1200., 929.);
            frame(
                &ctx,
                &mut a,
                1200.,
                929.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            let before = portable(&a);
            let key = |pressed| egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            };
            let pointer = |pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            let mut events = vec![key(true), pointer(true)];
            if !held {
                events.extend([pointer(false), key(false)]);
            }
            frame(&ctx, &mut a, 1200., 929., events);
            assert!(a.metrics.hsrp.guidance);
            assert!(a.metrics.next_unit.is_none());
            assert_eq!(portable(&a), before);
            assert_eq!(a.selected, Some(i));
            if held {
                frame(&ctx, &mut a, 1200., 929., vec![pointer(false), key(false)]);
                assert_eq!(portable(&a), before);
                assert_eq!(a.selected, Some(i));
            }
            frame(&ctx, &mut a, 1200., 929., vec![]);
            assert!(a.metrics.next_unit.is_some());
            let old_clock = a.progress.clock;
            let old_attempt = serde_json::to_value(&a.progress.attempts[&id]).unwrap();
            tap(&ctx, &mut a, "next");
            assert_eq!(a.selected, Some(peer_i));
            assert_eq!(a.progress.clock, old_clock + 1);
            assert_eq!(
                serde_json::to_value(&a.progress.attempts[&id]).unwrap(),
                old_attempt
            );
            assert!(!progress::current_solved(
                &a.progress.attempts[&peer.id],
                &peer
            ));
        }
    }
}
#[test]
fn unicode_line_separators_refuse_whole_native_packet_and_each_clean_action_requests_repaint() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let r = target(&ctx, &mut a, "router-A-priority", true, 1200., 929.);
    click(&ctx, &mut a, r, 1200., 929.);
    let before = portable(&a);
    for value in ["1\u{2028}2", "1\u{2029}2"] {
        frame(
            &ctx,
            &mut a,
            1200.,
            929.,
            vec![egui::Event::Paste(value.into())],
        );
        frame(&ctx, &mut a, 1200., 929., vec![]);
        assert_eq!(portable(&a), before);
    }
    for key in ["apply", "check", "show-hint"] {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        if key == "show-hint" {
            tap(&ctx, &mut a, "optional-help");
        }
        let r = target(&ctx, &mut a, key, false, 1200., 929.);
        frame(
            &ctx,
            &mut a,
            1200.,
            929.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 929.),
                )),
                events: [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    })
                    .collect(),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        assert_eq!(
            out.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::ZERO
        );
        frame(&ctx, &mut a, 1200., 929., vec![]);
        assert!(
            ctx.repaint_causes()
                .iter()
                .any(|c| c.file.ends_with("src/ui/hsrp_workspace.rs")),
            "{key} must request its own followup frame"
        );
    }
}

#[test]
fn ordinary_split_frame_hint_click_stays_enabled_through_press_idle_and_release() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "optional-help");
    let r = target(&ctx, &mut a, "show-hint", false, 1200., 929.);
    frame(
        &ctx,
        &mut a,
        1200.,
        929.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let before = portable(&a);
    let pointer = |pressed| egui::Event::PointerButton {
        pos: r.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(&ctx, &mut a, 1200., 929., vec![pointer(true)]);
    assert!(a.metrics.hsrp.guidance);
    assert_eq!(portable(&a), before);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 929., vec![]);
        assert_eq!(portable(&a), before);
    }
    frame(&ctx, &mut a, 1200., 929., vec![pointer(false)]);
    frame(&ctx, &mut a, 1200., 929., vec![]);
    let mut expected = before;
    expected["progress"]["attempts"][&a.scenarios[i].id]["hints"] = 1.into();
    assert_eq!(portable(&a), expected);
    assert!(!a.metrics.completion);
}

#[test]
fn ordinary_disclosure_split_frame_clicks_open_actual_bodies_without_lab_side_effects() {
    for header in [
        "optional-help",
        "worked-approach",
        "references",
        "inspection",
    ] {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        if matches!(header, "worked-approach" | "references") {
            tap(&ctx, &mut a, "optional-help");
        }
        let r = target(&ctx, &mut a, header, false, 1200., 929.);
        frame(
            &ctx,
            &mut a,
            1200.,
            929.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let before = portable(&a);
        let pointer = |pressed| egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&ctx, &mut a, 1200., 929., vec![pointer(true)]);
        assert_eq!(portable(&a), before);
        for _ in 0..3 {
            frame(&ctx, &mut a, 1200., 929., vec![]);
            assert_eq!(portable(&a), before);
        }
        frame(&ctx, &mut a, 1200., 929., vec![pointer(false)]);
        for _ in 0..3 {
            frame(&ctx, &mut a, 1200., 929., vec![]);
        }
        assert!(
            a.metrics.hsrp.disclosures.contains(&header),
            "Actual {header} body must open on split release"
        );
        assert_eq!(portable(&a), before);
        assert!(!a.metrics.completion);
        assert!(a.hsrp_presentation.events.is_empty());
    }
}

// R7 uses actual keyboard focus and pointer events. No request, focus, scroll,
// Attempt, grade or expected profile is installed by these gesture helpers.
fn r7_rect(a: &LabApp, key: &str) -> egui::Rect {
    match key {
        "mode-learn" => a.metrics.mode_learn.unwrap(),
        "mode-challenge" => a.metrics.mode_challenge.unwrap(),
        _ => {
            a.metrics
                .hsrp
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        }
    }
}
fn r7_focus(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    for _ in 0..96 {
        let r = r7_rect(a, key);
        if ctx
            .memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|response| response.rect == r)
        {
            return;
        }
        frame(
            ctx,
            a,
            1200.,
            929.,
            [true, false]
                .into_iter()
                .map(|pressed| egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                })
                .collect(),
        );
    }
    panic!("Real Tab must reach saved-scope action {key}");
}
fn r7_tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    if !key.starts_with("mode-") {
        tap(ctx, a, key);
        return;
    }
    for _ in 0..20 {
        let r = r7_rect(a, key);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
            click(ctx, a, r, 1200., 929.);
            return;
        }
        frame(
            ctx,
            a,
            1200.,
            929.,
            vec![egui::Event::PointerMoved(egui::pos2(
                root.left() + 2.,
                root.center().y,
            ))],
        );
        frame(
            ctx,
            a,
            1200.,
            929.,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (root.center().y - r.center().y)
                        .clamp(-root.height() * 0.75, root.height() * 0.75),
                ),
                modifiers: Default::default(),
            }],
        );
        for _ in 0..10 {
            frame(ctx, a, 1200., 929., vec![]);
        }
    }
    panic!("Bounded real header wheels must reveal {key}");
}
type R7NativeTrace = (
    egui::Id,
    Option<egui::text::CCursorRange>,
    Vec<(egui::text::CCursorRange, String)>,
    Vec<(egui::text::CCursorRange, String)>,
    bool,
);
fn r7_native(ctx: &egui::Context, a: &LabApp) -> Vec<R7NativeTrace> {
    let attempt = &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id];
    let s = session(a);
    let draft = if !attempt.challenge && s.view == dlms_learning_lab::script_teaching::View::Example
    {
        &s.example_draft
    } else {
        &s.draft
    };
    a.metrics
        .hsrp
        .fields
        .iter()
        .filter_map(|f| {
            let state = egui::text_edit::TextEditState::load(ctx, f.2)?;
            let range = state.cursor.char_range();
            let mut past = vec![];
            let mut future = vec![];
            if let Some(cursor) = range {
                let text = if let Some(client) = f.0.strip_prefix("client-") {
                    &draft
                        .clients
                        .iter()
                        .find(|c| c.id.to_string() == client)
                        .unwrap()
                        .gateway
                } else {
                    let index = if f.0.starts_with("router-A-") { 0 } else { 1 };
                    if f.0.ends_with("-priority") {
                        &draft.routers[index].priority
                    } else {
                        &draft.routers[index].decrement
                    }
                };
                let current = (cursor, text.clone());
                let mut undo = state.undoer();
                let mut now = current.clone();
                while let Some(v) = undo.undo(&now).cloned() {
                    now = v.clone();
                    past.push(v);
                    assert!(past.len() <= 100);
                }
                let mut redo = state.undoer();
                let mut now = current;
                while let Some(v) = redo.redo(&now).cloned() {
                    now = v.clone();
                    future.push(v);
                    assert!(future.len() <= 100);
                }
            }
            Some((f.2, range, past, future, state.undoer().is_in_flux()))
        })
        .collect()
}
#[test]
fn all_saved_scope_actions_wait_for_complete_guidance_and_later_clean_native_retry() {
    for action in [
        "introduction",
        "example",
        "assignment",
        "insert",
        "reset-example",
        "mode-challenge",
        "mode-learn",
    ] {
        let practice = matches!(action, "assignment" | "insert" | "reset-example");
        let headers: &[&str] = if practice {
            &["inspection", "optional-help"]
        } else {
            &[
                "inspection",
                "optional-help",
                "worked-approach",
                "references",
            ]
        };
        for &header in headers {
            for held in [false, true] {
                let (ctx, mut a, i) = opened();
                solve(&ctx, &mut a);
                if practice {
                    tap(&ctx, &mut a, "example");
                    if action == "reset-example" {
                        tap(&ctx, &mut a, "insert");
                        tap(&ctx, &mut a, "apply");
                        tap(&ctx, &mut a, "check");
                        assert!(session(&a).example_checked);
                        assert!(!session(&a).example_history.is_empty());
                    }
                }
                if action == "mode-learn" {
                    r7_tap(&ctx, &mut a, "mode-challenge");
                }
                if matches!(header, "worked-approach" | "references") {
                    tap(&ctx, &mut a, "optional-help");
                }
                r7_focus(&ctx, &mut a, action);
                let r = target(&ctx, &mut a, header, false, 1200., 929.);
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    929.,
                    vec![egui::Event::PointerMoved(r.center())],
                );
                let before = portable(&a);
                let before_dirty = a.dirty;
                let events_before = a.hsrp_presentation.events.clone();
                let native = r7_native(&ctx, &a);
                let key = |pressed| egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                };
                let pointer = |pressed| egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                };
                let mut events = vec![key(true), pointer(true)];
                if !held {
                    events.extend([pointer(false), key(false)]);
                }
                assert!(
                    !events
                        .iter()
                        .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
                );
                frame(&ctx, &mut a, 1200., 929., events);
                assert!(
                    a.metrics.hsrp.guidance,
                    "Actual {header} intent must be observed"
                );
                assert_eq!(
                    a.metrics.hsrp.scope_intents,
                    if action.starts_with("mode-") {
                        (false, true)
                    } else {
                        (true, false)
                    },
                    "Actual focused {action} must be queued before {header} veto"
                );
                assert_eq!(portable(&a), before);
                assert_eq!(a.dirty, before_dirty);
                assert_eq!(a.selected, Some(i));
                assert_eq!(a.hsrp_presentation.events, events_before);
                assert!(a.hsrp_presentation.pending_mode.is_none());
                assert_eq!(r7_native(&ctx, &a), native);
                if held {
                    for _ in 0..3 {
                        frame(&ctx, &mut a, 1200., 929., vec![]);
                        assert_eq!(portable(&a), before);
                    }
                    frame(&ctx, &mut a, 1200., 929., vec![pointer(false), key(false)]);
                    assert_eq!(portable(&a), before);
                    assert_eq!(a.dirty, before_dirty);
                }
                for _ in 0..3 {
                    frame(&ctx, &mut a, 1200., 929., vec![]);
                    assert_eq!(portable(&a), before);
                }
                let id = a.scenarios[i].id.clone();
                let mut expected = before.clone();
                let attempt = &mut expected["progress"]["attempts"][&id];
                match action {
                    "mode-challenge" => attempt["challenge"] = true.into(),
                    "mode-learn" => attempt["challenge"] = false.into(),
                    "introduction" | "example" | "assignment" => {
                        attempt["state"]["simulation"]["view"] = action.into()
                    }
                    "insert" => {
                        attempt["state"]["simulation"]["example_draft"]["routers"][1]["participating"] =
                            true.into()
                    }
                    "reset-example" => {
                        let s = &mut attempt["state"]["simulation"];
                        s["example"] = s["example_initial"].clone();
                        s["example_history"] = serde_json::json!([]);
                        s["example_draft"]["routers"][1]["participating"] = false.into();
                        s["example_checked"] = false.into();
                        s["selected_example"] = 0.into();
                    }
                    _ => unreachable!(),
                }
                r7_tap(&ctx, &mut a, action);
                assert_eq!(
                    portable(&a),
                    expected,
                    "Clean later {action} must apply only its documented change"
                );
                assert!(a.dirty);
                assert!(a.hsrp_presentation.pending_mode.is_none());
                assert_eq!(a.selected, Some(i));
                assert!(progress::current_solved(
                    &a.progress.attempts[&id],
                    &a.scenarios[i]
                ));
            }
        }
    }
}
#[test]
fn selected_worked_example_is_saved_only_after_all_help_and_real_later_picker_selection() {
    for header in ["inspection", "optional-help"] {
        for held in [false, true] {
            let (ctx, mut a, i) = opened();
            solve(&ctx, &mut a);
            tap(&ctx, &mut a, "example");
            tap(&ctx, &mut a, "worked-picker");
            r7_focus(&ctx, &mut a, "worked-choice-1");
            let r = target(&ctx, &mut a, header, false, 1200., 929.);
            frame(
                &ctx,
                &mut a,
                1200.,
                929.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            let before = portable(&a);
            let dirty = a.dirty;
            let key = |pressed| egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            };
            let pointer = |pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            let mut events = vec![key(true), pointer(true)];
            if !held {
                events.extend([pointer(false), key(false)]);
            }
            frame(&ctx, &mut a, 1200., 929., events);
            assert!(a.metrics.hsrp.guidance);
            assert_eq!(
                a.metrics.hsrp.scope_intents,
                (true, false),
                "Real keyboard selection must be queued before help"
            );
            assert_eq!(portable(&a), before);
            assert_eq!(a.dirty, dirty);
            if held {
                frame(&ctx, &mut a, 1200., 929., vec![pointer(false), key(false)]);
                assert_eq!(portable(&a), before);
            }
            for _ in 0..3 {
                frame(&ctx, &mut a, 1200., 929., vec![]);
                assert_eq!(portable(&a), before);
            }
            if !a
                .metrics
                .hsrp
                .controls
                .iter()
                .any(|r| r.0 == "worked-choice-1")
            {
                tap(&ctx, &mut a, "worked-picker");
            }
            tap(&ctx, &mut a, "worked-choice-1");
            let mut expected = before;
            expected["progress"]["attempts"][&a.scenarios[i].id]["state"]["simulation"]["selected_example"] =
                1.into();
            assert_eq!(portable(&a), expected);
            assert_eq!(session(&a).selected_example, 1);
            assert!(
                a.dirty
                    && progress::current_solved(
                        &a.progress.attempts[&a.scenarios[i].id],
                        &a.scenarios[i]
                    )
            );
            assert_eq!(a.selected, Some(i));
            assert!(a.hsrp_presentation.pending_mode.is_none());
        }
    }
}
