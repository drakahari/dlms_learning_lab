//! Real native events against the authored LACP workspaces, still unregistered in this stage.
use super::*;
use dlms_learning_lab::{etherchannel_workspace as work, script_teaching::View};
pub(in crate::ui) fn sources() -> Vec<Scenario> {
    pack::load_catalog(&[
        include_str!("../../../scenarios/starter-v1.json"),
        include_str!("../../../scenarios/vlan-data-v1.json"),
        include_str!("../../../scenarios/etherchannel-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "ccna-lacp")
    .collect()
}
pub(in crate::ui) fn admitted_sources(a: &mut LabApp) -> usize {
    let expected = sources();
    let present = expected
        .iter()
        .filter(|s| a.scenarios.iter().any(|old| old.id == s.id))
        .count();
    assert!(
        present == 0 || present == 3,
        "Never hide partial normal registration with extra fixtures"
    );
    if present == 0 {
        a.scenarios.extend(expected.iter().cloned());
    }
    let first = a
        .scenarios
        .iter()
        .position(|s| s.id == expected[0].id)
        .unwrap();
    for (n, expected) in expected.iter().enumerate() {
        assert_eq!(a.scenarios[first + n].id, expected.id);
        assert_eq!(
            serde_json::to_value(&a.scenarios[first + n]).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "Normal registration must be the reviewed authored source"
        );
    }
    first
}
fn opened(n: usize, view: View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let at = admitted_sources(&mut a);
    a.launch(at + n);
    let s = &a.scenarios[at + n];
    work::set_practice_presentation(
        a.progress.attempts.get_mut(&s.id).unwrap(),
        s,
        Some(view),
        None,
        None,
    )
    .unwrap();
    for _ in 0..4 {
        draw(&ctx, &mut a);
    }
    (ctx, a, at + n)
}
fn current(a: &LabApp) -> &Attempt {
    &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
}
fn native_frame(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1600., 1100., events);
}
fn tap(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    for pressed in [true, false] {
        native_frame(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    for _ in 0..4 {
        native_frame(ctx, a, vec![]);
    }
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for _ in 0..45 {
        let (r, clip) = if let Some(n) = key
            .strip_prefix("objective-detail:")
            .and_then(|n| n.parse::<usize>().ok())
        {
            (
                a.metrics
                    .ether
                    .detail_rows
                    .iter()
                    .find(|r| r.0 == n)
                    .expect("Actual requested detail row")
                    .2,
                a.metrics.main_viewport.unwrap(),
            )
        } else if key == "library-start" {
            (
                a.metrics
                    .card_actions
                    .first()
                    .expect("Actual single filtered card")
                    .0,
                a.metrics.main_viewport.unwrap(),
            )
        } else if key == "tab-assignment" {
            (
                a.metrics
                    .ether
                    .tabs
                    .iter()
                    .find(|r| r.0 == View::Assignment)
                    .expect("Actual Assignment tab")
                    .1,
                a.metrics.main_viewport.unwrap(),
            )
        } else if key == "next-lab" {
            (
                a.metrics
                    .ether
                    .next_lab
                    .as_ref()
                    .expect("Actual normal Next action")
                    .1,
                a.metrics.main_viewport.unwrap(),
            )
        } else if key == "mode-challenge" {
            (
                a.metrics.mode_challenge.expect("Actual Challenge action"),
                a.metrics.main_viewport.unwrap(),
            )
        } else if field {
            let f = a
                .metrics
                .ether
                .fields
                .iter()
                .find(|f| f.0 == key)
                .expect("Actual LACP field");
            (f.1, f.4)
        } else {
            let c = a
                .metrics
                .ether
                .controls
                .iter()
                .find(|c| c.0 == key)
                .expect("Actual LACP control");
            (c.1, c.3)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect())
            .intersect(clip);
        if root.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        let delta = if r.bottom() > root.bottom() {
            root.bottom() - r.bottom() - 8.
        } else {
            root.top() - r.top() + 8.
        };
        native_frame(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(root.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-350., 350.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            native_frame(ctx, a, vec![]);
        }
    }
    panic!("Actual LACP control clipped: {key}")
}
fn control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false);
    tap(ctx, a, r);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, label: &str) {
    control(ctx, a, key);
    let target = format!("{key}:{label}");
    let r = a
        .metrics
        .ether
        .choices
        .iter()
        .find(|c| c.0 == target)
        .expect("Actual supplied option")
        .1;
    assert!(
        ctx.content_rect().contains_rect(r),
        "Supplied choice must be on screen: {target}"
    );
    tap(ctx, a, r);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = reveal(ctx, a, key, true);
    tap(ctx, a, r);
    let id = a
        .metrics
        .ether
        .fields
        .iter()
        .find(|f| f.0 == key)
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(id)));
    native_frame(
        ctx,
        a,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste(text.into())],
    );
    native_frame(ctx, a, vec![]);
}
fn assign(a: &mut LabApp, i: usize) {
    let s = &a.scenarios[i];
    work::set_practice_presentation(
        a.progress.attempts.get_mut(&s.id).unwrap(),
        s,
        Some(View::Assignment),
        None,
        None,
    )
    .unwrap();
}
#[test]
fn two_members_join_by_native_modes_and_a_later_raw_edit_invalidates_success() {
    let (ctx, mut a, i) = opened(0, View::Assignment);
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    control(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.ether.rows.iter().any(|r| !r.1));
    choose(&ctx, &mut a, "campus-a:Port(\"p1\"):Lacp", "Active");
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "check");
    assert!(!a.metrics.completion, "One pair is partial work");
    choose(&ctx, &mut a, "campus-a:Port(\"p2\"):Lacp", "Active");
    control(&ctx, &mut a, "check");
    assert!(!a.metrics.completion, "Check must not apply saved fields");
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(progress::current_solved(current(&a), &a.scenarios[i]));
    edit(&ctx, &mut a, "campus-a:Device:Vlans", "1,10,99,100");
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(current(&a), &a.scenarios[i]));
    let applied = current(&a).state.clone();
    control(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert_eq!(
        serde_json::to_value(&current(&a).state).unwrap(),
        serde_json::to_value(applied).unwrap()
    );
}
#[test]
fn practice_fill_is_fields_only_apply_is_real_and_assignment_draft_stays_isolated() {
    for n in 0..3 {
        let (ctx, mut a, i) = opened(n, View::Example);
        for _ in 0..4 {
            native_frame(&ctx, &mut a, vec![]);
        }
        let before = serde_json::to_value(&current(&a).state).unwrap();
        let draft = current(&a)
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_draft
            .clone();
        control(&ctx, &mut a, "fill");
        assert_eq!(serde_json::to_value(&current(&a).state).unwrap(), before);
        let l = current(&a)
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap();
        assert!(!l.draft.matches(&l.simulation));
        control(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(a.metrics.ether.rows.iter().any(|r| !r.1));
        control(&ctx, &mut a, "apply");
        control(&ctx, &mut a, "check");
        assert!(!a.metrics.completion, "Practice never completes assignment");
        let l = current(&a)
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap();
        assert!(
            l.checks(a.scenarios[i].etherchannel_teaching.as_ref().unwrap())
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(
            current(&a).workspace.as_ref().unwrap().etherchannel_draft,
            draft
        );
        assert_eq!(serde_json::to_value(&current(&a).state).unwrap(), before);
        assign(&mut a, i);
        for _ in 0..4 {
            native_frame(&ctx, &mut a, vec![]);
        }
        assert!(!a.metrics.completion);
    }
}
#[test]
fn overlong_native_paste_rolls_back_and_vetoes_check_without_mutating_canonical_attempt() {
    let (ctx, mut a, _) = opened(0, View::Assignment);
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    let key = "campus-a:Device:Vlans";
    let r = reveal(&ctx, &mut a, key, true);
    tap(&ctx, &mut a, r);
    let id = a
        .metrics
        .ether
        .fields
        .iter()
        .find(|f| f.0 == key)
        .unwrap()
        .2;
    let before = serde_json::to_value(current(&a)).unwrap();
    native_frame(
        &ctx,
        &mut a,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste("9".repeat(385))],
    );
    assert!(a.metrics.ether.rejected);
    assert!(!a.metrics.ether.checked);
    assert_eq!(serde_json::to_value(current(&a)).unwrap(), before);
    assert!(
        egui::text_edit::TextEditState::load(&ctx, id).is_none(),
        "Rejected generation must release native Undo history"
    );
}
#[test]
fn exact_idle_frames_reuse_memo_and_all_five_text_sizes_keep_native_fields_inside_width() {
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for width in [946., 1920.] {
            let (ctx, mut a, _) = opened(0, View::Assignment);
            a.scale = scale;
            for _ in 0..6 {
                frame(&ctx, &mut a, width, 1100., vec![]);
            }
            let before = a.ether_presentation.memo.counters;
            for _ in 0..60 {
                frame(&ctx, &mut a, width, 1100., vec![]);
            }
            let after = a.ether_presentation.memo.counters;
            assert_eq!(after.evaluations, before.evaluations);
            assert_eq!(after.observations, before.observations);
            assert_eq!(after.meters, before.meters);
            assert_eq!(after.snapshots, before.snapshots);
            let screen = ctx.content_rect();
            for (key, r, _, _, _) in &a.metrics.ether.fields {
                assert!(
                    r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                    "{key} clips horizontally at width{width} scale{scale}: {r:?} {screen:?}"
                );
                assert!(
                    r.height() >= 28.,
                    "Native field should not shrink text to fit"
                );
            }
        }
    }
}

#[test]
fn wrapped_native_etherchannel_disclosure_preserves_full_accessible_text_and_toggle() {
    let ctx = egui::Context::default();
    let title = "Inspect a supplied cable failure or VLAN · Optional";
    let mut rectangle = egui::Rect::NOTHING;
    let mut body_opened = false;
    let mut clicked_info = false;
    for n in 0..8 {
        let events = match n {
            3 | 4 => vec![
                egui::Event::PointerMoved(rectangle.center()),
                egui::Event::PointerButton {
                    pos: rectangle.center(),
                    button: egui::PointerButton::Primary,
                    pressed: n == 3,
                    modifiers: Default::default(),
                },
            ],
            _ => vec![],
        };
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(220., 400.),
                )),
                time: Some(n as f64 / 20.),
                events,
                ..Default::default()
            },
            |ctx| {
                controls::configure(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    let result = controls::collapsing_header(ui, title)
                        .id_salt("wrapped-native-title")
                        .show(ui, |ui| ui.label("Actual optional body"));
                    rectangle = result.header_response.rect;
                    assert!(rectangle.right() <= ui.clip_rect().right());
                    assert!(rectangle.left() >= ui.clip_rect().left());
                    assert!(rectangle.height() > controls::height(ui));
                    body_opened |= result.body_response.is_some();
                });
            },
        );
        clicked_info |= output.platform_output.events.iter().any(|event| {
            matches!(event, egui::output::OutputEvent::Clicked(info)
                if info.typ == egui::WidgetType::CollapsingHeader
                && info.label.as_deref() == Some(title))
        });
    }
    assert!(
        body_opened,
        "The unchanged native header must really toggle its body"
    );
    assert!(
        clicked_info,
        "The native accessibility event must keep the full untruncated title"
    );
}

#[test]
fn etherchannel_applied_notice_yields_to_current_paste_failure_and_valid_retry() {
    let (ctx, mut a, i) = opened(0, View::Assignment);
    for port in ["p1", "p2"] {
        choose(
            &ctx,
            &mut a,
            &format!("campus-a:Port({port:?}):Lacp"),
            "Active",
        );
    }
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    // Applying the same valid fields creates an actual applied notice again.
    control(&ctx, &mut a, "apply");
    assert!(
        a.ether_presentation
            .notice()
            .unwrap()
            .starts_with("Configuration applied")
    );
    let key = "campus-a:Device:Vlans";
    let r = reveal(&ctx, &mut a, key, true);
    tap(&ctx, &mut a, r);
    let before = serde_json::to_value(current(&a)).unwrap();
    let was_solved = progress::current_solved(current(&a), &a.scenarios[i]);
    let original = current(&a)
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .devices["campus-a"]
        .vlans
        .clone();
    native_frame(
        &ctx,
        &mut a,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste("9".repeat(385))],
    );
    assert!(a.metrics.ether.rejected);
    assert_eq!(serde_json::to_value(current(&a)).unwrap(), before);
    assert_eq!(
        progress::current_solved(current(&a), &a.scenarios[i]),
        was_solved
    );
    let notice = a.ether_presentation.notice().unwrap();
    assert!(notice.contains("too long"));
    assert!(!notice.contains("Configuration applied"));
    // Retry edits only this actual field. Persisted state remains unapplied,
    // unrelated data survives, and a formerly valid outcome becomes pending.
    let applied_state = serde_json::to_value(&current(&a).state).unwrap();
    // Rejection deliberately clears the old native field/Undo generation.
    // Observe its replacement before sending the learner's next click/paste.
    native_frame(&ctx, &mut a, vec![]);
    edit(&ctx, &mut a, key, "1,10,99,100");
    assert!(a.ether_presentation.notice().is_none());
    assert!(!a.metrics.ether.rejected);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(current(&a), &a.scenarios[i]));
    assert_eq!(
        serde_json::to_value(&current(&a).state).unwrap(),
        applied_state
    );
    let mut expected = before;
    expected["workspace"]["etherchannel_draft"]["devices"]["campus-a"]["vlans"] =
        serde_json::json!("1,10,99,100");
    assert_eq!(serde_json::to_value(current(&a)).unwrap(), expected);
    // Returning to the learner's exact original text restores the complete
    // draft through native editing; checking still remains an explicit action.
    edit(&ctx, &mut a, key, &original);
    assert!(a.ether_presentation.notice().is_none());
    control(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}

#[test]
fn compact_etherchannel_results_keep_every_outcome_and_original_details_are_requested() {
    let (ctx, mut a, i) = opened(2, View::Assignment);
    control(&ctx, &mut a, "check");
    let expected = progress::current_etherchannel_checks(current(&a), &a.scenarios[i]);
    assert_eq!(expected.len(), 21);
    let original = serde_json::to_value(current(&a)).unwrap();
    let canonical: Vec<_> = expected.iter().enumerate().map(|(n, r)| (n, r.0)).collect();
    assert_eq!(
        a.metrics
            .ether
            .rows
            .iter()
            .map(|r| (r.0, r.1))
            .collect::<Vec<_>>(),
        canonical
    );
    assert!(
        a.metrics.ether.detail_rows.is_empty(),
        "Full diagnostics must be optional by default"
    );
    assert_eq!(
        a.metrics.ether.first_failure.unwrap().0,
        expected.iter().position(|r| !r.0).unwrap()
    );
    control(&ctx, &mut a, "objective-details");
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(
        a.metrics
            .ether
            .detail_rows
            .iter()
            .map(|r| (r.0, r.1))
            .collect::<Vec<_>>(),
        canonical
    );
    assert_eq!(
        a.metrics
            .ether
            .rows
            .iter()
            .map(|r| (r.0, r.1))
            .collect::<Vec<_>>(),
        canonical,
        "Original details must not duplicate compact-row identities"
    );
    assert_eq!(serde_json::to_value(current(&a)).unwrap(), original);
    // Read each original diagnostic from actual painted text after revealing
    // its real row with wheel events. Metrics alone cannot stand in for text.
    fn text(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(s) => {
                out.push_str(&s.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| text(s, out)),
            _ => {}
        }
    }
    for (n, (ok, diagnostic)) in expected.iter().enumerate() {
        reveal(&ctx, &mut a, &format!("objective-detail:{n}"), false);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600. / a.scale, 1100. / a.scale),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut painted = String::new();
        for shape in output.shapes {
            text(&shape.shape, &mut painted);
        }
        let original_line = format!("{} {diagnostic}", if *ok { "Pass ·" } else { "Check ·" });
        assert_eq!(
            painted.matches(&original_line).count(),
            1,
            "Original detail{n} must be painted exactly once: {original_line}"
        );
    }
    assert_eq!(serde_json::to_value(current(&a)).unwrap(), original);
    control(&ctx, &mut a, "objective-details");
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert!(a.metrics.ether.detail_rows.is_empty());
    let cached = a.ether_presentation.memo.counters;
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for width in [946., 1920.] {
            a.scale = scale;
            for _ in 0..6 {
                frame(&ctx, &mut a, width, 1100., vec![]);
            }
            assert_eq!(
                a.metrics
                    .ether
                    .rows
                    .iter()
                    .map(|r| (r.0, r.1))
                    .collect::<Vec<_>>(),
                canonical
            );
            let screen = ctx.content_rect();
            for (n, _, r) in &a.metrics.ether.rows {
                assert!(
                    r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                    "Compact outcome{n} clips at{scale}/{width}: {r:?} {screen:?}"
                );
            }
            assert!(a.metrics.ether.detail_rows.is_empty());
            assert_eq!(serde_json::to_value(current(&a)).unwrap(), original);
        }
    }
    assert_eq!(
        a.ether_presentation.memo.counters.observations, cached.observations,
        "Appearance must not evaluate pair observations"
    );
    // Challenge keeps the same assignment and compact outcomes; opening full
    // diagnostics is still a deliberate action and never reveals a solution.
    control(&ctx, &mut a, "mode-challenge");
    assert!(current(&a).challenge);
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(
        a.metrics
            .ether
            .rows
            .iter()
            .map(|r| (r.0, r.1))
            .collect::<Vec<_>>(),
        canonical
    );
    assert!(a.metrics.ether.detail_rows.is_empty());
    assert!(a.metrics.ether.reasoning_text.is_none());
    control(&ctx, &mut a, "objective-details");
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(
        a.metrics
            .ether
            .detail_rows
            .iter()
            .map(|r| (r.0, r.1))
            .collect::<Vec<_>>(),
        canonical
    );
    // The first two tasks have no retired-members requirement. Removing that
    // empty row must restore exactly the shared spacing, at real font sizes.
    for n in 0..2 {
        let (ctx, mut a, _) = opened(n, View::Assignment);
        control(&ctx, &mut a, "check");
        let retained = serde_json::to_value(current(&a)).unwrap();
        for scale in [1., 2.] {
            for light in [false, true] {
                a.scale = scale;
                a.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
                for width in [946., 1920.] {
                    for _ in 0..6 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    assert!(a.metrics.ether.compact_retired.is_none());
                    let top = a.metrics.ether.compact_current.unwrap();
                    let bottom = a.metrics.ether.compact_preserved.unwrap();
                    let gap = bottom.top() - top.bottom();
                    assert!(
                        (gap - ctx.style().spacing.item_spacing.y).abs()
                            <= 2. / ctx.pixels_per_point(),
                        "No empty row: {n}/{scale}/{light}/{width}, gap{gap}"
                    );
                    assert_eq!(a.metrics.ether.rows.len(), 7);
                    assert_eq!(serde_json::to_value(current(&a)).unwrap(), retained);
                }
            }
        }
    }
}

#[test]
fn registered_lacp_unit_starts_from_library_and_real_next_keeps_completed_work() {
    let (ctx, mut a) = app();
    let ids = [
        "ccna-lacp-initiation",
        "ccna-lacp-member-service",
        "ccna-lacp-resilient-handover",
    ];
    let unit = a
        .units
        .iter()
        .find(|u| u.id == "ccna-lacp")
        .expect("Normal bundled unit, not injected fixture")
        .clone();
    assert_eq!(unit.labs, ids);
    assert_eq!(unit.prerequisites, ["vlan-data-foundations"]);
    let original = serde_json::to_value(&a.scenarios).unwrap();
    a.filter = Filter {
        track: "CCNA".into(),
        unit: unit.id.clone(),
        ..Default::default()
    };
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(
        a.metrics.card_actions.len(),
        3,
        "Normal unit filter exposes exactly its three meaningful assignments"
    );
    a.filter.search = a
        .scenarios
        .iter()
        .find(|s| s.id == ids[0])
        .unwrap()
        .title
        .clone();
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(a.metrics.card_actions.len(), 1);
    control(&ctx, &mut a, "library-start");
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    assert_eq!(a.scenarios[a.selected.unwrap()].id, ids[0]);
    control(&ctx, &mut a, "tab-assignment");
    for _ in 0..4 {
        native_frame(&ctx, &mut a, vec![]);
    }
    for port in ["p1", "p2"] {
        choose(
            &ctx,
            &mut a,
            &format!("campus-a:Port({port:?}):Lacp"),
            "Active",
        );
    }
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let first = serde_json::to_value(&a.progress.attempts[ids[0]]).unwrap();
    assert_eq!(a.metrics.ether.next_lab.as_ref().unwrap().0, ids[1]);
    control(&ctx, &mut a, "next-lab");
    assert_eq!(a.scenarios[a.selected.unwrap()].id, ids[1]);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ids[0]]).unwrap(),
        first
    );
    let second = a.scenarios[a.selected.unwrap()].clone();
    assert_eq!(a.progress.attempts[ids[1]].state, second.initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[ids[1]],
        &second
    ));
    assert_eq!(
        a.progress.attempts[ids[1]]
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap()
            .view,
        View::Introduction
    );
    assert_eq!(
        serde_json::to_value(&a.scenarios).unwrap(),
        original,
        "Normal navigation does not install or rewrite authored sources"
    );
}
