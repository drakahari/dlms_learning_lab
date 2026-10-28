//! Source-paired controller/capture regression. Headless events do not prove Fedora paint.
use super::*;
use dlms_learning_lab::{etherchannel_workspace as work, script_teaching::View};
#[test]
fn etherchannel_capture_route_is_closed_and_all_three_real_control_flows_finish_before_native_run()
{
    assert_eq!(
        LabApp::etherchannel_stage(false, Some("etherchannel-lessons")),
        None
    );
    assert_eq!(
        LabApp::etherchannel_stage(true, Some("etherchannel-lessons-extra")),
        None
    );
    assert_eq!(
        LabApp::etherchannel_stage(true, Some("etherchannel-lessons")),
        Some(49000)
    );
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 49000;
    let old_count = a.scenarios.len();
    let old_unit_count = a.units.len();
    let old = serde_json::to_value(&a.scenarios).unwrap();
    let units = serde_json::to_value(&a.units).unwrap();
    let mut captures = std::collections::BTreeSet::new();
    let mut closed = false;
    let mut last = 49000;
    let mut same_stage = 0;
    for n in 0..25_000 {
        let events = std::mem::take(&mut a.pending_events);
        assert!(events.len() <= 300);
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
        let align = a.smoke_stage.saturating_sub(49600);
        let narrow = a.smoke_stage >= 49600 && align / 4 >= 10;
        let width = if narrow { 946. } else { 1920. };
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / (a.scale * 1.15), 1080. / (a.scale * 1.15)),
            )),
            events,
            modifiers,
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.native_pixels_per_point = Some(1.15);
        viewport.fullscreen = Some(!narrow);
        let output = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        });
        for viewport in output.viewport_output.values() {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::Screenshot(data) => {
                        let tag = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(
                            captures.insert(tag),
                            "Actual capture-phase tags must be unique"
                        );
                        assert!(captures.len() <= 128);
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        if a.smoke_stage == last {
            same_stage += 1;
        } else {
            same_stage = 0;
            last = a.smoke_stage;
        }
        assert!(
            same_stage <= 900,
            "Controller stage{} failed to complete inside its actual event/scroll budget",
            a.smoke_stage
        );
        if closed {
            break;
        }
    }
    assert!(
        closed,
        "Actual closed native-capture route must terminate before native launch"
    );
    assert_eq!(
        captures
            .iter()
            .filter(|s| s.starts_with("lacp-alignment-"))
            .count(),
        60
    );
    for id in [
        "ccna-lacp-initiation",
        "ccna-lacp-member-service",
        "ccna-lacp-resilient-handover",
    ] {
        for suffix in [
            "introduction",
            "example-initial",
            "example-verified",
            "example-adverse-retry",
            "assignment-initial",
            "wrong-specific-feedback",
            "partial-feedback",
            "reference-verified",
            "pending-edit-invalidates-success",
            "native-undo-restores-current-success",
            "alternate-one-verified",
            "alternate-two-verified",
            "challenge-keeps-full-task",
            "reset-prepared-introduction",
            "objective-details-requested",
        ] {
            assert!(
                captures.contains(&format!("{id}-{suffix}")),
                "Missing actual capture phase {id}/{suffix}"
            );
        }
    }
    assert!(captures.contains("ccna-lacp-resilient-handover-independent-cable-loss-observation"));
    assert_eq!(captures.len(), 106);
    assert_eq!(
        serde_json::to_value(&a.scenarios[..old_count]).unwrap(),
        old
    );
    assert_eq!(
        serde_json::to_value(&a.units[..old_unit_count]).unwrap(),
        units
    );
    assert!(
        a.progress
            .attempts
            .keys()
            .all(|id| id.starts_with("ccna-lacp-"))
    );

    // Exercise the actual short follow-up capture branch before any desktop run.
    assert_eq!(
        LabApp::etherchannel_stage(true, Some("etherchannel-spacing")),
        Some(49700)
    );
    assert_eq!(
        LabApp::etherchannel_stage(true, Some("etherchannel-spacing-extra")),
        None
    );
    assert_eq!(
        LabApp::etherchannel_stage(false, Some("etherchannel-spacing")),
        None
    );
    let (ctx, mut spacing) = app();
    spacing.smoke = true;
    spacing.smoke_stage = 49700;
    let old = serde_json::to_value(&spacing.scenarios).unwrap();
    let units = serde_json::to_value(&spacing.units).unwrap();
    let old_count = spacing.scenarios.len();
    let old_units = spacing.units.len();
    let mut shots = std::collections::BTreeSet::new();
    let mut closed = false;
    let mut last = 49700;
    let mut same = 0;
    for n in 0..3000 {
        let phase = (spacing.smoke_stage - 49700) % 12;
        let narrow = (8..=11).contains(&phase);
        let width = if narrow { 946. } else { 1920. };
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(
                    width / (spacing.scale * 1.15),
                    1080. / (spacing.scale * 1.15),
                ),
            )),
            events: std::mem::take(&mut spacing.pending_events),
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.native_pixels_per_point = Some(1.15);
        viewport.fullscreen = Some(!narrow);
        let output = ctx.run(input, |ctx| {
            spacing.render(ctx);
            spacing.save();
            spacing.started = Instant::now() - std::time::Duration::from_secs(1);
            spacing.smoke_tick(ctx);
        });
        for viewport in output.viewport_output.values() {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::Screenshot(data) => {
                        let tag = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(shots.insert(tag));
                        assert!(shots.len() <= 8);
                        spacing.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        assert!(spacing.pending_events.len() <= 300);
        if spacing.smoke_stage == last {
            same += 1;
        } else {
            same = 0;
            last = spacing.smoke_stage;
        }
        assert!(same <= 900, "Short real capture stage{last} cannot stall");
        if closed {
            break;
        }
    }
    assert!(closed);
    assert_eq!(shots.len(), 8);
    for id in ["ccna-lacp-initiation", "ccna-lacp-member-service"] {
        for suffix in [
            "wide-dark-100",
            "wide-light-100",
            "narrow-dark-200",
            "narrow-light-200",
        ] {
            assert!(shots.contains(&format!("lacp-spacing-{id}-{suffix}")));
        }
    }
    assert_eq!(
        serde_json::to_value(&spacing.scenarios[..old_count]).unwrap(),
        old
    );
    assert_eq!(
        serde_json::to_value(&spacing.units[..old_units]).unwrap(),
        units
    );
    for (id, a) in &spacing.progress.attempts {
        let source = spacing.scenarios.iter().find(|s| s.id == *id).unwrap();
        assert!(!progress::current_solved(a, source));
    }
}
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let at = super::etherchannel_workspace::admitted_sources(&mut a);
    a.launch(at);
    let s = &a.scenarios[at];
    work::set_practice_presentation(
        a.progress.attempts.get_mut(&s.id).unwrap(),
        s,
        Some(View::Assignment),
        None,
        None,
    )
    .unwrap();
    draw(&ctx, &mut a);
    (ctx, a, at)
}
#[test]
fn etherchannel_actual_reset_confirm_prepares_new_intro_draft_and_separate_example() {
    let (ctx, mut a, i) = opened();
    let id = a.scenarios[i].id.clone();
    a.progress.favorites.insert(id.clone());
    let reset = a.metrics.reset.unwrap();
    click(&ctx, &mut a, reset);
    assert!(a.reset_pending);
    let confirm = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, confirm);
    let attempt = &a.progress.attempts[&id];
    assert_eq!(attempt.state, a.scenarios[i].initial);
    assert!(
        attempt
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_draft
            .is_some()
    );
    let l = attempt
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.view, View::Introduction);
    assert!(l.draft.matches(&l.simulation));
    assert!(a.progress.favorites.contains(&id));
    assert!(!a.metrics.completion);
}
#[test]
fn etherchannel_whole_attempt_capacity_rejects_native_growth_without_erasing_or_grading_old_data() {
    let (ctx, mut a, i) = opened();
    let id = a.scenarios[i].id.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    let mut lo = 0;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        attempt.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(&*attempt).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    attempt.transcript = vec!["x".repeat(lo)];
    assert_eq!(
        serde_json::to_vec(&*attempt).unwrap().len(),
        work::MAX_ATTEMPT
    );
    draw(&ctx, &mut a);
    let f = a
        .metrics
        .ether
        .fields
        .iter()
        .find(|r| r.0 == "campus-a:Device:Vlans")
        .unwrap();
    let rect = f.1;
    click(&ctx, &mut a, rect);
    let before = serde_json::to_value(&a.progress.attempts[&id]).unwrap();
    let old = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .devices["campus-a"]
        .vlans
        .clone();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Paste(format!("{old} ")),
        ],
    );
    assert!(a.metrics.ether.rejected);
    assert!(!a.metrics.ether.checked);
    assert!(!a.metrics.completion);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&id]).unwrap(),
        before
    );
}
#[test]
fn etherchannel_context_foreign_presence_cannot_reuse_a_warm_renderer_or_mutate_saved_work() {
    let (ctx, mut a, i) = opened();
    let id = a.scenarios[i].id.clone();
    let nfs = a
        .scenarios
        .iter()
        .find(|s| s.id == "lpic2-nfs-reference")
        .unwrap();
    let mut other = Attempt::new(nfs, 1);
    dlms_learning_lab::nfs_teaching::prepare(&mut other, nfs, true).unwrap();
    let foreign = other.workspace.unwrap().nfs_session.unwrap();
    let before = a.ether_presentation.memo.counters;
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .nfs_session = Some(foreign);
    let retained = serde_json::to_value(&a.progress.attempts[&id]).unwrap();
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(a.metrics.ether.fields.is_empty());
    assert!(a.metrics.ether.controls.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&id]).unwrap(),
        retained
    );
    assert_eq!(
        a.ether_presentation.memo.counters.snapshots,
        before.snapshots
    );
    assert_eq!(a.ether_presentation.memo.counters.meters, before.meters);
}

fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) -> String {
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
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600., 2400.),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut printed = String::new();
    for clipped in output.shapes {
        text(&clipped.shape, &mut printed);
    }
    printed
}
fn painted_click(ctx: &egui::Context, a: &mut LabApp, rect: egui::Rect) {
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    for _ in 0..12 {
        paint(ctx, a, vec![]);
    }
}
#[test]
fn etherchannel_worked_reasoning_stays_in_completion_and_challenge_requires_explicit_review() {
    let (ctx, mut a, i) = opened();
    let id = a.scenarios[i].id.clone();
    let source = a.scenarios[i].clone();
    let paragraph = source.explanation.clone();
    let before = paint(&ctx, &mut a, vec![]);
    assert!(!before.contains(&paragraph));
    assert!(a.metrics.ether.reasoning_header.is_none());
    // Backend fixture establishes only the visibility precondition; the closed
    // controller above separately proves real configuration paths and grading.
    for action in &source.solution {
        let action = dlms_learning_lab::etherchannel::Action::from_json(
            action.strip_prefix(work::PREFIX).unwrap().as_bytes(),
        )
        .unwrap();
        let dlms_learning_lab::etherchannel::Action::Configure { configuration } = action;
        let attempt = a.progress.attempts.get_mut(&id).unwrap();
        let State::EtherChannel { simulation } = &attempt.state else {
            panic!()
        };
        let d = work::Draft::from_configuration(&configuration, simulation.topology()).unwrap();
        work::set_assignment_draft(attempt, &source, &d).unwrap();
        work::apply_assignment(attempt, &source).unwrap();
    }
    paint(&ctx, &mut a, vec![]);
    assert!(
        a.metrics.ether.reasoning_header.is_none(),
        "No completion card before actual Check"
    );
    let check = a
        .metrics
        .ether
        .controls
        .iter()
        .find(|r| r.0 == "check")
        .unwrap()
        .1;
    painted_click(&ctx, &mut a, check);
    assert!(a.metrics.completion);
    let closed = paint(&ctx, &mut a, vec![]);
    assert!(!closed.contains(&paragraph));
    assert!(a.metrics.ether.reasoning_text.is_none());
    let header = a.metrics.ether.reasoning_header.unwrap();
    painted_click(&ctx, &mut a, header);
    let opened = paint(&ctx, &mut a, vec![]);
    assert!(opened.contains(&paragraph));
    assert!(a.metrics.ether.reasoning_text.is_some());
    let challenge = a.metrics.mode_challenge.unwrap();
    painted_click(&ctx, &mut a, challenge);
    assert!(a.progress.attempts[&id].challenge);
    assert!(a.metrics.ether.tabs.is_empty());
    let independent = paint(&ctx, &mut a, vec![]);
    assert!(
        !independent.contains(&paragraph),
        "Learn review cannot automatically open Challenge reasoning"
    );
    assert!(a.metrics.ether.reasoning_text.is_none());
    let header = a.metrics.ether.reasoning_header.unwrap();
    painted_click(&ctx, &mut a, header);
    assert!(
        paint(&ctx, &mut a, vec![]).contains(&paragraph),
        "Challenge permits explicitly requested review after a pass"
    );
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    let mut d = attempt
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .clone();
    d.devices.get_mut("campus-a").unwrap().vlans = "1,10,99,100".into();
    work::set_assignment_draft(attempt, &source, &d).unwrap();
    let broken = paint(&ctx, &mut a, vec![]);
    assert!(!a.metrics.completion);
    assert!(a.metrics.ether.reasoning_header.is_none());
    assert!(a.metrics.ether.reasoning_text.is_none());
    assert!(
        !broken.contains(&paragraph),
        "Broken current work cannot retain the passed completion explanation"
    );
}

#[test]
fn etherchannel_capture_reentry_scrolls_after_the_completed_selector_without_reopening_it() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 49000;
    let mut completed_frames = 0;
    let mut wheel_after_completion = false;
    let mut captured = false;
    for n in 0..6000 {
        let stage = a.smoke_stage;
        assert!(stage <= 49016);
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
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920. / 1.15, 650. / 1.15),
            )),
            events,
            modifiers,
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        let v = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        v.native_pixels_per_point = Some(1.15);
        v.fullscreen = Some(true);
        let output = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        });
        if stage == 49015
            && a.smoke_stage == 49015
            && ctx
                .data(|d| {
                    d.get_temp::<bool>(egui::Id::new((
                        "etherchannel-native-single-done",
                        49015usize,
                    )))
                })
                .unwrap_or(false)
        {
            completed_frames += 1;
            wheel_after_completion |= a
                .pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }));
            assert!(
                !a.pending_events
                    .iter()
                    .any(|e| matches!(e, egui::Event::PointerButton { .. })),
                "Completed selector cannot emit another click while its capture scrolls"
            );
        }
        for viewport in output.viewport_output.values() {
            for command in &viewport.commands {
                if let egui::ViewportCommand::Screenshot(data) = command {
                    let tag = data
                        .data
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap();
                    if tag == "ccna-lacp-initiation-example-adverse-retry" {
                        captured = true;
                    }
                    a.capture_pending = false;
                }
            }
        }
        if a.smoke_stage == 49016 {
            break;
        }
    }
    assert!(
        captured && wheel_after_completion && completed_frames >= 2,
        "Repeated actual capture-phase frames must retain selector completion and perform real scrolling"
    );
    assert_eq!(a.smoke_stage, 49016);
    assert!(
        ctx.data(|d| d.get_temp::<bool>(egui::Id::new((
            "etherchannel-native-single-done",
            49015usize
        ))))
        .is_none(),
        "Advance clears the phase-local latch"
    );
}

#[test]
fn etherchannel_actual_admission_counter_covers_live_pending_display_and_completion_navigation() {
    let (ctx, mut a) = app();
    let start = super::etherchannel_workspace::admitted_sources(&mut a);
    let ids: Vec<_> = a.scenarios[start..start + 3]
        .iter()
        .map(|s| s.id.clone())
        .collect();
    if let Some(unit) = a.units.iter().find(|u| u.id == "ccna-lacp") {
        assert_eq!(unit.labs, ids);
        assert_eq!(unit.prerequisites, ["vlan-data-foundations"]);
    } else {
        a.units.push(course::Unit {
            id: "ccna-lacp".into(),
            track: "CCNA".into(),
            title: "LACP regression unit".into(),
            summary: "Admitted cache navigation regression".into(),
            prerequisites: vec!["vlan-data-foundations".into()],
            labs: ids.clone(),
        });
    }
    for i in start..start + 3 {
        a.launch(i);
        let source = a.scenarios[i].clone();
        let attempt = a.progress.attempts.get_mut(&source.id).unwrap();
        work::set_practice_presentation(attempt, &source, Some(View::Assignment), None, None)
            .unwrap();
        for action in &source.solution {
            let action = dlms_learning_lab::etherchannel::Action::from_json(
                action.strip_prefix(work::PREFIX).unwrap().as_bytes(),
            )
            .unwrap();
            let dlms_learning_lab::etherchannel::Action::Configure { configuration } = action;
            let State::EtherChannel { simulation } = &attempt.state else {
                panic!()
            };
            let d = work::Draft::from_configuration(&configuration, simulation.topology()).unwrap();
            work::set_assignment_draft(attempt, &source, &d).unwrap();
            work::apply_assignment(attempt, &source).unwrap();
        }
        work::set_saved_check(attempt, &source, true).unwrap();
    }
    for _ in 0..6 {
        frame(&ctx, &mut a, 1600., 1000., vec![]);
    }
    assert!(a.metrics.completion);
    assert!(a.metrics.ether.next_lab.is_none());
    let before = work::json_admission_calls();
    assert!(
        before > 0,
        "Actual JSON sizing must be instrumented, not inferred from cache counters"
    );
    let cached = a.ether_presentation.memo.counters;
    for n in 0..80 {
        let events = if n < 20 {
            let r = a.metrics.main_viewport.unwrap();
            vec![
                egui::Event::PointerMoved(egui::pos2(r.left() + 1., r.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., if n < 10 { 80. } else { -80. }),
                    modifiers: Default::default(),
                },
            ]
        } else {
            vec![]
        };
        frame(&ctx, &mut a, 1600., 1000., events);
    }
    assert_eq!(
        work::json_admission_calls(),
        before,
        "Live rendering and completed-course navigation must not serialize/admit unchanged LACP data"
    );
    assert_eq!(
        a.ether_presentation.memo.counters.evaluations,
        cached.evaluations
    );
    assert_eq!(
        a.ether_presentation.memo.counters.observations,
        cached.observations
    );
    assert_eq!(a.ether_presentation.memo.counters.meters, cached.meters);
    assert_eq!(
        a.ether_presentation.memo.counters.snapshots,
        cached.snapshots
    );
    // The same navigation must stop calling another attempt complete as soon as
    // a real, source-aware pending edit breaks its current outcome.
    let source = a.scenarios[start].clone();
    let attempt = a.progress.attempts.get_mut(&source.id).unwrap();
    let mut pending = attempt
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .clone();
    pending.devices.get_mut("campus-a").unwrap().vlans = "1,10,99,100".into();
    work::set_assignment_draft(attempt, &source, &pending).unwrap();
    frame(&ctx, &mut a, 1600., 1000., vec![]);
    assert!(
        a.metrics.completion,
        "Selected capstone remains independently correct"
    );
    assert_eq!(
        a.metrics.ether.next_lab.as_ref().map(|r| r.0.as_str()),
        Some(ids[0].as_str())
    );
    let renewed = work::json_admission_calls();
    assert!(
        renewed > before,
        "Changed complete input requires fresh actual admission"
    );
    for _ in 0..20 {
        frame(&ctx, &mut a, 1600., 1000., vec![]);
    }
    assert_eq!(work::json_admission_calls(), renewed);
    let first = a.progress.attempts[&ids[0]].clone();
    a.progress
        .attempts
        .get_mut(&ids[0])
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .nfs_session = Some(Default::default());
    frame(&ctx, &mut a, 1600., 1000., vec![]);
    assert_eq!(a.ether_presentation.memo.prepared(&ids[0]), Some(false));
    assert_eq!(
        a.metrics.ether.next_lab.as_ref().map(|r| r.0.as_str()),
        Some(ids[0].as_str()),
        "Foreign context cannot be skipped as completed by navigation"
    );
    a.progress.attempts.insert(ids[0].clone(), first);
}

#[test]
fn etherchannel_real_capture_branch_waits_for_deferred_zoom_and_still_rejects_settled_clipping() {
    let (ctx, mut a) = app();
    let start = super::etherchannel_workspace::admitted_sources(&mut a);
    a.launch(start + 2);
    let source = a.scenarios[start + 2].clone();
    work::set_practice_presentation(
        a.progress.attempts.get_mut(&source.id).unwrap(),
        &source,
        Some(View::Assignment),
        None,
        None,
    )
    .unwrap();
    a.scale = 1.5;
    for _ in 0..4 {
        paint(&ctx, &mut a, vec![]);
    }
    let disclosure = a
        .metrics
        .ether
        .expanders
        .iter()
        .find(|r| r.0 == "inspect-options")
        .unwrap()
        .1;
    painted_click(&ctx, &mut a, disclosure);
    let tick = |a: &mut LabApp, n: usize, drive: bool, inject_clip: bool| {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946. / (a.scale * 1.15), 1080. / (a.scale * 1.15)),
            )),
            events: std::mem::take(&mut a.pending_events),
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        let v = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        v.native_pixels_per_point = Some(1.15);
        v.fullscreen = Some(false);
        ctx.run(input, |ctx| {
            a.render(ctx);
            if inject_clip {
                let right = ctx.content_rect().right();
                let r = &mut a.metrics.ether.controls[0].1;
                r.min.x = right + 5.;
                r.max.x = right + 65.;
            }
            if drive {
                a.started = Instant::now() - std::time::Duration::from_secs(1);
                a.smoke_tick(ctx);
            }
        })
    };
    for n in 100..106 {
        tick(&mut a, n, false, false);
    }
    a.smoke = true;
    a.smoke_stage = 49656;
    let preserved = serde_json::to_value(&a.progress).unwrap();
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("ether-alignment-portable"), preserved.clone()));
    tick(&mut a, 106, true, false);
    assert_eq!(a.smoke_stage, 49657);
    assert_eq!(a.scale, 2.);
    let first = tick(&mut a, 107, true, false);
    assert_eq!(
        a.smoke_stage, 49657,
        "First changed-zoom frame must not advance/capture before actual geometry settles"
    );
    assert!(
        first
            .viewport_output
            .values()
            .flat_map(|v| &v.commands)
            .all(|c| !matches!(c, egui::ViewportCommand::Screenshot(_)))
    );
    let mut captured = false;
    for n in 108..168 {
        let output = tick(&mut a, n, true, false);
        for v in output.viewport_output.values() {
            for command in &v.commands {
                if let egui::ViewportCommand::Screenshot(_) = command {
                    captured = true;
                    a.capture_pending = false;
                }
            }
        }
        if captured {
            break;
        }
    }
    assert!(
        captured,
        "The real capture branch must accept fully settled200% geometry within its unchanged60-frame budget"
    );
    assert_eq!(a.smoke_stage, 49658);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), preserved);
    a.smoke_stage = 49657;
    a.capture_pending = false;
    let mut rejected = None;
    for n in 169..229 {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick(&mut a, n, true, true)))
        {
            Err(failure) => {
                rejected = Some(failure);
                break;
            }
            Ok(_) => assert_eq!(
                a.smoke_stage, 49657,
                "A clipped control cannot advance the actual capture phase"
            ),
        }
    }
    let failed=rejected.expect("Actual settled200% clipped control must still be rejected within its bounded geometry sequence");
    let message = failed
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| failed.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        message.contains("LACP control horizontal clipping"),
        "Keep the real all-control clipping assertion, not a weakened geometry or duplicate-tag failure: {message}"
    );
}
