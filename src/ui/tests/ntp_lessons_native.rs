//! Actual route/paint-phase regression, not a Fedora pixel or novice acceptance claim.
use super::*;
use crate::ui::smoke::ntp_lessons as route;
use std::collections::BTreeSet;
fn expected_captures() -> BTreeSet<String> {
    let mut names = BTreeSet::from(["ntp-normal-library-search-placeholder".to_owned()]);
    for id in [
        "ccna-ntp-client-chain",
        "ccna-ntp-cycle-repair",
        "ccna-ntp-asymmetric-delivery",
        "ccna-ntp-policy-handover",
    ] {
        for suffix in [
            "introduction",
            "whole-assignment",
            "example-inserted-not-run",
            "example-experiment",
            "command-refusal",
            "wrong-retry",
            "partial-configuration",
            "dependency-evidence",
            "reference-complete",
            "alternate-1",
            "alternate-2",
            "challenge",
        ] {
            assert!(names.insert(format!("{id}-{suffix}")));
        }
    }
    names.insert("ccna-ntp-client-chain-success-invalidated".into());
    for suffix in [
        "requested-progressive-hint",
        "requested-worked-approach",
        "requested-references",
        "requested-supported-commands",
    ] {
        assert!(names.insert(format!("ccna-ntp-client-chain-{suffix}")));
    }
    for scale in [100, 115, 125, 150, 200] {
        for theme in ["light", "dark"] {
            for width in ["wide", "narrow"] {
                assert!(names.insert(format!("ntp-handover-layout-{scale}-{theme}-{width}")));
                assert!(names.insert(format!("ntp-handover-dependency-{scale}-{theme}-{width}")));
            }
        }
    }
    names
}
fn flatten<'a>(s: &'a egui::Shape, clip: egui::Rect, out: &mut Vec<(egui::Rect, &'a egui::Shape)>) {
    if let egui::Shape::Vec(items) = s {
        for s in items {
            flatten(s, clip, out);
        }
    } else {
        out.push((clip, s));
    }
}
fn paint(ctx: &egui::Context, a: &LabApp, out: &egui::FullOutput, name: &str) {
    let mut items = Vec::new();
    for s in &out.shapes {
        flatten(&s.shape, s.clip_rect, &mut items);
    }
    let tolerance = 1. / out.pixels_per_point;
    assert!(out.pixels_per_point > 0. && out.pixels_per_point.is_finite());
    let theme_name = theme::find(&a.theme_id).name.clone();
    let scale = format!("{:.0}%", a.scale * 100.);
    for label in [
        "DLMS Learning Lab",
        "Offline simulation",
        "Text size",
        scale.as_str(),
        theme_name.as_str(),
        "Settings",
    ] {
        let (at, bounds) = items
            .iter()
            .enumerate()
            .find_map(|(at, (clip, shape))| {
                let egui::Shape::Text(text) = shape else {
                    return None;
                };
                let bounds = text.visual_bounding_rect();
                (text.galley.text() == label
                    && text.opacity_factor > 0.
                    && bounds.is_positive()
                    && clip.expand(tolerance).contains_rect(bounds)
                    && ctx.content_rect().expand(tolerance).contains_rect(bounds)
                    && text.galley.rows.iter().any(|r| {
                        !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
                    })
                    && text.galley.rows.iter().all(|r| {
                        r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                            .iter()
                            .all(|v| {
                                text.override_text_color
                                    .unwrap_or_else(|| {
                                        if v.color == egui::Color32::PLACEHOLDER {
                                            text.fallback_color
                                        } else {
                                            v.color
                                        }
                                    })
                                    .a()
                                    > 0
                            })
                    }))
                .then_some((at, bounds))
            })
            .unwrap_or_else(|| {
                panic!("Actual capture{name} lacks visible unclipped header{label:?}")
            });
        assert!(!items[at+1..].iter().any(|(clip,shape)|matches!(shape,egui::Shape::Rect(r)if r.fill.a()==255&&clip.intersect(r.rect).contains_rect(bounds))),"Later opaque paint covers header{label:?}");
    }
    if name.contains("-requested-") {
        let source = &a.scenarios[a.selected.unwrap()];
        let wanted = if name.ends_with("requested-progressive-hint") {
            format!("Hint 1: {}", source.hints[0])
        } else if name.ends_with("requested-worked-approach") {
            source.explanation.clone()
        } else if name.ends_with("requested-references") {
            source.sources[0].clone()
        } else {
            dlms_learning_lab::ntp::HELP.to_owned()
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(
            items.iter().any(|(clip, shape)| match shape {
                egui::Shape::Text(text) if text.galley.text() == wanted => {
                    let bounds = text.visual_bounding_rect();
                    text.opacity_factor > 0.
                        && !text.galley.elided
                        && bounds.is_positive()
                        && clip.intersect(root).expand(tolerance).contains_rect(bounds)
                        && text.galley.rows.iter().any(|r| {
                            !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                                .is_empty()
                        })
                }
                _ => false,
            }),
            "Actualrequestedhelpbodymustpaintunclipped:{name}"
        );
        assert_eq!(a.progress.attempts[&source.id].hints, 1);
        assert!(!progress::current_solved(
            &a.progress.attempts[&source.id],
            source
        ));
        let engine::State::Ntp {
            simulation: session,
        } = &a.progress.attempts[&source.id].state
        else {
            panic!()
        };
        assert!(
            session.transcript.is_empty() && session.example_transcript.is_empty(),
            "Requestinghelpneversendscommands"
        );
    }
    if name.starts_with("ntp-handover-layout-") {
        let (_, field, id, clip) = &a.metrics.ntp.fields[0];
        assert!(clip.expand(tolerance).contains_rect(*field));
        assert_eq!(ctx.memory(|m| m.focused()), Some(*id));
        let text = items
            .iter()
            .find_map(|(clip, s)| match s {
                egui::Shape::Text(t)
                    if t.galley.text() == a.ntp_session_for_test_command()
                        && field.intersects(t.visual_bounding_rect()) =>
                {
                    Some((clip, t))
                }
                _ => None,
            })
            .expect("Populated command is actually painted");
        let bounds = text.1.visual_bounding_rect();
        assert!(text.0.expand(tolerance).contains_rect(bounds));
        assert!(field.expand(tolerance).contains_rect(bounds));
        let font = egui::TextStyle::Body.resolve(&ctx.style());
        let row = ctx.fonts_mut(|f| f.row_height(&font));
        assert!(
            (bounds.center().y - field.center().y).abs() <= row * 0.25 + tolerance,
            "Field text should use shared font-centered layout, no top-border alignment"
        );
    }
}
impl LabApp {
    fn ntp_session_for_test_command(&self) -> &str {
        let s = &self.scenarios[self.selected.unwrap()];
        let State::Ntp { simulation } = &self.progress.attempts[&s.id].state else {
            panic!()
        };
        &simulation.command
    }
}
#[test]
fn ntp_native_route_has_closed_inventory_and_exact_selection_without_weakening_other_routes() {
    assert_eq!(LabApp::ntp_stage(false, Some("ntp-lessons")), None);
    assert_eq!(LabApp::ntp_stage(true, Some("ntp-lessons")), Some(80000));
    for name in [
        None,
        Some("ntp"),
        Some("loop-recovery-lessons"),
        Some("one-off-lessons"),
    ] {
        assert_eq!(LabApp::ntp_stage(true, name), None);
    }
    let (first, last) = route::phase_bounds();
    assert_eq!(first, 80000);
    assert_eq!(
        last - first + 1,
        1369 + 20,
        "Original1369 actions plus20 explicit native help/disclosure/progress-law actions"
    );
    assert!(last < 82000);
    let shots = route::planned_captures();
    assert_eq!(shots.len(), 94);
    assert_eq!(
        shots
            .iter()
            .filter(|name| !name.contains("-requested-"))
            .count(),
        90,
        "Alloriginal90capture names remain"
    );
    assert_eq!(
        shots.iter().cloned().collect::<BTreeSet<_>>(),
        expected_captures()
    );
}
#[test]
fn ntp_normal190_real_events_complete_all_tasks_and_every_capture_phase_without_installed_success()
{
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 80000;
    assert!(!a.synthetic && !a.developer);
    assert_eq!((a.scenarios.len(), a.units.len()), (190, 50));
    assert!(a.progress.attempts.is_empty());
    let catalog = serde_json::to_vec(&a.scenarios).unwrap();
    let units = serde_json::to_vec(&a.units).unwrap();
    let mut pixels = egui::vec2(1920., 1080.);
    let mut fullscreen = true;
    let mut time = 0.;
    let mut shots = BTreeSet::new();
    let mut closed = 0;
    let mut previous = a.smoke_stage;
    let mut frames = 0;
    // Reproduce a newly requested body initially below the visible viewport.
    // Use only real RawInput window geometry/hover/wheels; never install scroll,
    // focus, progress or a fabricated paint rectangle.
    let mut help_offscreen_proved = false;
    let mut help_seek_frames = 0usize;
    let mut help_wheels = 0usize;
    let mut help_restore_pixels = None;
    let mut help_prepare_progress = None;
    for _ in 0..60_000 {
        time += 0.05;
        let mut events = std::mem::take(&mut a.pending_events);
        let prepare_hidden =
            a.smoke_stage == route::first_hint_capture_stage() && !help_offscreen_proved;
        if prepare_hidden {
            if help_restore_pixels.is_none() {
                help_restore_pixels = Some(pixels);
                help_prepare_progress = Some(
                    serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
                );
                pixels = egui::vec2(760., 600.) * 1.15; // a native-user resize in the test provider
            }
            if help_seek_frames == 0 {
                let root = a.metrics.main_viewport.unwrap();
                events.push(egui::Event::PointerMoved(egui::pos2(
                    root.left() + 2.,
                    root.center().y,
                )));
            } else if help_seek_frames.is_multiple_of(10) {
                assert!(
                    help_wheels < 20,
                    "Bounded real outerwheel preparation for offscreen guidance"
                );
                let root = a.metrics.main_viewport.unwrap();
                events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., root.height() * 0.75),
                    modifiers: Default::default(),
                });
                help_wheels += 1;
            }
            help_seek_frames += 1;
            assert!(help_seek_frames <= 201);
        }
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
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                pixels / (a.scale * 1.15),
            )),
            time: Some(time),
            events,
            modifiers,
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.native_pixels_per_point = Some(1.15);
        viewport.fullscreen = Some(fullscreen);
        let out = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            if !prepare_hidden {
                a.smoke_tick(ctx);
            }
        });
        if prepare_hidden {
            assert_eq!(
                &serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
                help_prepare_progress.as_ref().unwrap(),
                "Only realviewgeometry/hover/wheels preparedtheoffscreenbody; fullprogress/receiptsmustremainexact"
            );
            let source = &a.scenarios[a.selected.unwrap()];
            let wanted = format!("Hint 1: {}", source.hints[0]);
            let (_, response, clip) = a
                .metrics
                .ntp
                .help_bodies
                .iter()
                .find(|(text, _, _)| *text == wanted)
                .expect("Real response exists even before its text paints");
            let root = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if !root.intersect(*clip).intersects(*response) {
                let mut painted = Vec::new();
                for shape in &out.shapes {
                    flatten(&shape.shape, shape.clip_rect, &mut painted);
                }
                assert!(!painted.iter().any(|(_,shape)|matches!(shape,egui::Shape::Text(text)if text.galley.text()==wanted)),"The reproduced body is genuinely unpainted, not only nominally clipped");
                assert_eq!(a.progress.attempts[&source.id].hints, 1);
                help_offscreen_proved = true;
            }
        }
        for v in out.viewport_output.values() {
            for command in &v.commands {
                match command {
                    egui::ViewportCommand::Fullscreen(value) => {
                        fullscreen = *value;
                        if *value {
                            pixels = egui::vec2(1920., 1080.);
                        }
                    }
                    egui::ViewportCommand::InnerSize(size) => {
                        if !fullscreen {
                            pixels = *size * 1.15;
                        }
                    }
                    egui::ViewportCommand::Screenshot(data) => {
                        let name = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(expected_captures().contains(&name));
                        assert!(shots.insert(name.clone()));
                        paint(&ctx, &a, &out, &name);
                        if name.ends_with("-requested-progressive-hint") {
                            assert!(
                                help_offscreen_proved,
                                "Capture must recover from a genuinely unpainted body using response rects and real wheels"
                            );
                            assert_eq!(
                                &serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
                                help_prepare_progress.as_ref().unwrap()
                            );
                            pixels = help_restore_pixels
                                .take()
                                .expect("Observed pre-reproduction window size");
                        }
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => {
                        assert!(route::completed(&ctx));
                        closed += 1;
                    }
                    _ => {}
                }
            }
        }
        if a.smoke_stage == previous {
            frames += 1;
        } else {
            previous = a.smoke_stage;
            frames = 0;
        }
        assert!(frames < 1000, "Finite native stage{}", a.smoke_stage);
        if closed >= 3 {
            break;
        }
    }
    assert!(closed >= 3, "Actual idempotent close reached");
    assert!(help_offscreen_proved && help_restore_pixels.is_none());
    assert_eq!(
        route::recency_observations(&ctx),
        Some((1, 4)),
        "The real Next/return and all four real Back/Continue recency laws were exercised, not skipped"
    );
    assert_eq!(shots, expected_captures());
    assert_eq!(a.progress.attempts.len(), 4);
    assert_eq!(serde_json::to_vec(&a.scenarios).unwrap(), catalog);
    assert_eq!(serde_json::to_vec(&a.units).unwrap(), units);
    for id in [
        "ccna-ntp-client-chain",
        "ccna-ntp-cycle-repair",
        "ccna-ntp-asymmetric-delivery",
        "ccna-ntp-policy-handover",
    ] {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
    }
}

/// Isolate native cadence from learner data: only real normal Start/assignment
/// clicks and outer wheels prepare an actually clipped Back control. The test
/// selects an existing QA Back phase, never installs learner state or progress.
#[test]
fn ntp_native_wheel_wait_does_not_spend_work_polls_at_fast_frame_cadence() {
    fn cadence_frame(
        ctx: &egui::Context,
        a: &mut LabApp,
        time: f64,
        events: Vec<egui::Event>,
        drive: bool,
    ) {
        let mut events = events;
        events.append(&mut a.pending_events);
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(760., 560.),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                a.render(ctx);
                if drive {
                    a.started = Instant::now() - std::time::Duration::from_secs(1);
                    a.smoke_tick(ctx);
                }
            },
        );
    }
    fn click_cadence(ctx: &egui::Context, a: &mut LabApp, time: &mut f64, r: egui::Rect) {
        *time += 0.05;
        cadence_frame(
            ctx,
            a,
            *time,
            vec![egui::Event::PointerMoved(r.center())],
            false,
        );
        *time += 0.05;
        cadence_frame(
            ctx,
            a,
            *time,
            [true, false]
                .into_iter()
                .map(|pressed| egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                })
                .collect(),
            false,
        );
        for _ in 0..3 {
            *time += 0.05;
            cadence_frame(ctx, a, *time, vec![], false);
        }
    }
    fn reveal_cadence(
        ctx: &egui::Context,
        a: &mut LabApp,
        time: &mut f64,
        target: fn(&LabApp) -> egui::Rect,
    ) -> egui::Rect {
        for _ in 0..20 {
            let r = target(a);
            let root = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
                return r;
            }
            let point = egui::pos2(root.left() + 2., root.center().y);
            *time += 0.05;
            cadence_frame(ctx, a, *time, vec![egui::Event::PointerMoved(point)], false);
            *time += 0.05;
            cadence_frame(
                ctx,
                a,
                *time,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                }],
                false,
            );
            for _ in 0..10 {
                *time += 0.05;
                cadence_frame(ctx, a, *time, vec![], false);
            }
        }
        panic!("The actual cadence setup target must be reachable within20 real wheels");
    }
    let (ctx, mut a) = app();
    a.filter.track = "CCNA".into();
    a.filter.unit = "ntp-client-server".into();
    let mut time = 0.;
    for _ in 0..3 {
        time += 0.05;
        cadence_frame(&ctx, &mut a, time, vec![], false);
    }
    let start = reveal_cadence(&ctx, &mut a, &mut time, |a| {
        let at = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == "ccna-ntp-client-chain")
            .unwrap();
        a.metrics.card_actions[at].0
    });
    click_cadence(&ctx, &mut a, &mut time, start);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, "ccna-ntp-client-chain");
    let assignment = reveal_cadence(&ctx, &mut a, &mut time, |a| {
        a.metrics
            .ntp
            .controls
            .iter()
            .find(|r| r.0 == "assignment")
            .unwrap()
            .1
    });
    click_cadence(&ctx, &mut a, &mut time, assignment);
    let full =
        serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts});
    let mut clipped = false;
    for _ in 0..20 {
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if a.metrics.back.unwrap().bottom() < root.top() {
            clipped = true;
            break;
        }
        time += 0.05;
        cadence_frame(
            &ctx,
            &mut a,
            time,
            vec![egui::Event::PointerMoved(egui::pos2(
                root.left() + 2.,
                root.center().y,
            ))],
            false,
        );
        time += 0.05;
        cadence_frame(
            &ctx,
            &mut a,
            time,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., -root.height() * 0.75),
                modifiers: Default::default(),
            }],
            false,
        );
        for _ in 0..10 {
            time += 0.05;
            cadence_frame(&ctx, &mut a, time, vec![], false);
        }
    }
    assert!(
        clipped,
        "Preparation must genuinely move Back above its visible clip using real wheels"
    );
    assert_eq!(
        serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
        full
    );
    a.smoke = true;
    a.smoke_stage = route::first_back_stage();
    let stage = a.smoke_stage;
    let mut observed = None;
    for _ in 0..20 {
        time += 0.000_001;
        cadence_frame(&ctx, &mut a, time, vec![], true);
        if let Some((wheels, until, polls)) = route::wheel_wait_observation(&ctx)
            && wheels > 0
            && until > time
        {
            observed = Some((until, polls));
            break;
        }
    }
    let (until, polls) =
        observed.expect("The real driver must issue a reveal wheel and enter its150ms wait");
    for _ in 0..400 {
        time += 0.000_001;
        assert!(
            time < until,
            "All400 extra root frames intentionally precede the observed wheel deadline"
        );
        cadence_frame(&ctx, &mut a, time, vec![], true);
        assert_eq!(a.smoke_stage, stage);
        assert_eq!(
            route::wheel_wait_observation(&ctx).unwrap().2,
            polls,
            "Waiting is not another work attempt"
        );
        assert_eq!(
            serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
            full
        );
    }
    for _ in 0..240 {
        time += 0.05;
        cadence_frame(&ctx, &mut a, time, vec![], true);
        if a.selected.is_none() {
            break;
        }
    }
    assert!(
        a.selected.is_none(),
        "After real time advances, the unchanged bounded driver must reveal and actually click Back"
    );
    assert_eq!(
        serde_json::json!({"portable":a.progress,"caller_receipts":a.progress.migration_receipts}),
        full
    );
    assert_eq!(a.progress.attempts.len(), 1);
}
