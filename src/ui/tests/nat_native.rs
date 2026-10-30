//! Event-protocol tests, not native-window or cold-process acceptance.
use super::super::smoke::nat::{FIRST, ID, LAST};
use super::*;
use dlms_learning_lab::nat as core;
#[derive(Default)]
struct Witness {
    shots: Vec<String>,
    inner_wheel: bool,
    overflowing_inner_wheel: bool,
    child_offset_moved: bool,
    viewport_height: Option<f32>,
    frames: usize,
    wheel_frames: Vec<usize>,
    fullscreen_requests: Vec<bool>,
    focused: Vec<(String, String, String, bool)>,
}
fn app_native() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = FIRST;
    assert_eq!(
        (
            one_off_boundary::before_len(&a.scenarios),
            one_off_boundary::before_len(&a.units)
        ),
        (176, 46)
    );
    assert!(a.progress.attempts.is_empty());
    (ctx, a)
}
fn raw(a: &LabApp) -> &dlms_learning_lab::nat_workspace::Draft {
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
fn actual_tick(
    ctx: &egui::Context,
    a: &mut LabApp,
    age: bool,
    w: &mut Witness,
) -> egui::FullOutput {
    let events = std::mem::take(&mut a.pending_events);
    w.frames += 1;
    if events
        .iter()
        .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
    {
        w.wheel_frames.push(w.frames);
    }
    if events
        .iter()
        .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        && let Some(point) = events.iter().find_map(|e| {
            if let egui::Event::PointerMoved(p) = e {
                Some(*p)
            } else {
                None
            }
        })
        && a.metrics
            .nat_static_viewport
            .is_some_and(|r| r.contains(point))
    {
        w.inner_wheel = true;
        w.overflowing_inner_wheel |= a.metrics.nat_static_scroll.is_some_and(|(_, _, content)| {
            content.y
                > a.metrics.nat_static_viewport.unwrap().height() + 1. / ctx.pixels_per_point()
        });
    }
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
    let width = if a.scale == 1.5 {
        1400.
    } else if a.scale == 2. {
        946.
    } else {
        1200.
    };
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, w.viewport_height.unwrap_or(950.) / a.scale),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            w.child_offset_moved |= a
                .metrics
                .nat_static_scroll
                .is_some_and(|(_, offset, _)| offset.y > 1. / ctx.pixels_per_point());
            a.save();
            if age {
                a.started = Instant::now() - std::time::Duration::from_secs(1);
            }
            a.smoke_tick(ctx);
        },
    );
    let mut screenshots = 0;
    for v in out.viewport_output.values() {
        for cmd in &v.commands {
            if let egui::ViewportCommand::Fullscreen(value) = cmd {
                w.fullscreen_requests.push(*value);
            }
            if let egui::ViewportCommand::Screenshot(data) = cmd {
                let label = data
                    .data
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<String>()
                    .unwrap()
                    .clone();
                screenshots += 1;
                if label.contains("focused-input") {
                    let field = a.metrics.nat_fields.iter().find(|r| r.0 == "last").unwrap();
                    w.focused.push((
                        label.clone(),
                        raw(a).last.clone(),
                        live(a)
                            .configuration()
                            .dynamic
                            .unwrap()
                            .pool
                            .last
                            .to_string(),
                        ctx.memory(|m| m.has_focus(field.2)),
                    ));
                }
                w.shots.push(label);
            }
        }
    }
    if a.capture_pending {
        assert!(
            screenshots > 0,
            "Only acknowledge an actual Screenshot request in this fake renderer"
        );
        a.capture_pending = false;
    }
    out
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, w: &mut Witness) {
    let mut phase = a.smoke_stage;
    let mut frames = 0;
    for _ in 0..8000 {
        if a.smoke_stage == target {
            return;
        }
        assert!((FIRST..=LAST).contains(&a.smoke_stage));
        if phase == a.smoke_stage {
            frames += 1;
        } else {
            phase = a.smoke_stage;
            frames = 0;
        }
        assert!(
            frames <= 400,
            "Bounded actual SmokeTick at phase{}",
            a.smoke_stage
        );
        actual_tick(ctx, a, true, w);
    }
    panic!("NAT bounded total native protocol frames")
}
#[test]
fn nat_closed_selector_actual_smoke_tick_opens_only_one_unsolved_private_fixture_via_real_start() {
    assert_eq!(LabApp::nat_stage(true, Some("nat-workspace")), Some(FIRST));
    for name in [
        None,
        Some("nat"),
        Some("nat-workspace-extra"),
        Some("safe-automation-lessons"),
    ] {
        assert_eq!(LabApp::nat_stage(true, name), None);
    }
    assert_eq!(LabApp::nat_stage(false, Some("nat-workspace")), None);
    const {
        assert!(FIRST == 38000 && LAST == 38071 && FIRST > 37504 && LAST < 39000);
    }
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    a.started = Instant::now();
    actual_tick(&ctx, &mut a, false, &mut w);
    assert_eq!(a.smoke_stage, FIRST);
    assert_eq!(one_off_boundary::before_len(&a.scenarios), 176);
    assert!(a.progress.attempts.is_empty());
    until(&ctx, &mut a, FIRST + 2, &mut w);
    assert_eq!(
        (
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(
                &a.scenarios[..190]
            )
            .len()
                + 1,
            one_off_boundary::before_len(&a.units)
        ),
        (183, 46)
    );
    assert_eq!(
        (a.scenarios.len(), one_off_boundary::before_len(&a.units)),
        (191, 46)
    );
    assert_eq!(
        serde_json::to_value(&a.scenarios[..190]).unwrap(),
        serde_json::to_value(pack::bundled().unwrap()).unwrap(),
        "The single private NAT fixture must preserve every normal assignment"
    );
    assert_eq!(a.scenarios[190].id, ID);
    assert!(
        w.fullscreen_requests.contains(&true),
        "Initial wide view must issue a real fullscreen request; this fake renderer does not claim WM acceptance"
    );
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(a.progress.attempts.contains_key(ID));
    assert!(a.progress.favorites.contains(ID));
    let i = a.selected.unwrap();
    assert_eq!(a.scenarios[i].id, ID);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert!(live(&a).bindings().is_empty());
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn nat_actual_first_flow_scrolls_real_static_child_and_runs_partial_then_current_plus_fresh() {
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + 6, &mut w);
    // The ordinary two-row layout intentionally fits in the normal window.
    // During the existing real Add/edit/Remove phase, give the root one and a
    // half measured row heights: one complete row fits, while the second must
    // be reached through the actual child scroller. No fixture/state injection.
    let row_height = a.metrics.nat_static_viewport.unwrap().height();
    let chrome = ctx.content_rect().height() - a.metrics.main_viewport.unwrap().height();
    w.viewport_height = Some(chrome + row_height * 1.5);
    until(&ctx, &mut a, FIRST + 7, &mut w);
    assert!(
        w.overflowing_inner_wheel,
        "A real wheel must reach a child whose content exceeds its viewport"
    );
    assert!(
        w.child_offset_moved,
        "The real child scroll offset must move before edit/Remove acknowledgement"
    );
    w.viewport_height = None;
    until(&ctx, &mut a, FIRST + 22, &mut w);
    assert!(
        w.inner_wheel,
        "Actual wheel events must reach the child, not its neighbor column"
    );
    assert!(
        w.wheel_frames
            .windows(2)
            .all(|frames| frames[1] - frames[0] >= 3),
        "Actual wheel corrections must be separated by real rendered settling frames"
    );
    assert_eq!(raw(&a).statics.len(), 1);
    assert_eq!(raw(&a).statics[0].inside, "observer");
    assert_eq!(live(&a).bindings().len(), 2);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[a.selected.unwrap()]
    ));
    assert!(w.shots.iter().any(|s| s.contains("partial-beta-missing")));
    assert!(w.shots.iter().any(|s| s.contains("current-and-fresh-pass")));
    assert!(matches!(
        a.nat_presentation.observed_effect(),
        Some(core::Effect::Probe {
            result: core::ProbeResult::Translated {
                kind: core::MappingKind::Dynamic,
                ..
            }
        })
    ));
}
#[test]
fn nat_actual_alternate_modals_invalid_file_cancel_and_warm_continue_preserve_all_fields() {
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + 51, &mut w);
    assert!(!a.settings_open && !a.data_tools.modal_open() && !a.reset_pending);
    assert_eq!(raw(&a).network, "10.0.0.10");
    assert_eq!(raw(&a).prefix, "31");
    assert!(w.shots.iter().any(|s| s.contains("warm-resumed-pending")));
    assert!(a.progress.favorites.contains(ID));
    let before = serde_json::to_value(&a.progress).unwrap();
    let raw_bytes = progress::read_bounded(&a.path, 512 * 1024).unwrap();
    let mut disk: serde_json::Value = serde_json::from_slice(&raw_bytes).unwrap();
    let receipts = disk
        .as_object_mut()
        .unwrap()
        .remove("migration_receipts")
        .unwrap();
    assert!(
        disk == before,
        "All portable clock/Attempt/favorite/other keys retained"
    );
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded).unwrap() == before);
    assert!(serde_json::to_value(&loaded.migration_receipts).unwrap() == receipts);
    assert_eq!(
        progress::read_bounded(&a.path, 512 * 1024).unwrap(),
        raw_bytes
    );
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[a.selected.unwrap()]
    ));
}
#[test]
fn nat_actual_scaled_capture_phases_prove_focused_raw_text_before_apply_and_unchanged_holds() {
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    until(&ctx, &mut a, FIRST + 64, &mut w);
    assert!(w.fullscreen_requests.contains(&true) && w.fullscreen_requests.contains(&false));
    let light = w
        .focused
        .iter()
        .find(|r| r.0.contains("light150"))
        .expect("actual Light screenshot command");
    assert_eq!(
        (light.1.as_str(), light.2.as_str()),
        ("198.51.100.14", "198.51.100.15")
    );
    assert!(light.3);
    let dark = w
        .focused
        .iter()
        .find(|r| r.0.contains("dark200"))
        .expect("actual Dark screenshot command");
    assert_eq!(
        (dark.1.as_str(), dark.2.as_str()),
        ("198.51.100.15", "198.51.100.14")
    );
    assert!(dark.3);
    assert_eq!(
        live(&a)
            .configuration()
            .dynamic
            .unwrap()
            .pool
            .last
            .to_string(),
        "198.51.100.15"
    );
    assert_eq!(live(&a).bindings().len(), 2);
    assert!(w.shots.iter().any(|s| s.contains("light150-current-pass")));
    assert!(w.shots.iter().any(|s| s.contains("dark200-current-pass")));
}
#[test]
fn nat_actual_finish_holds_and_normal_smoke_pacing_is_unchanged() {
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    until(&ctx, &mut a, LAST, &mut w);
    for _ in 0..2 {
        let out = actual_tick(&ctx, &mut a, true, &mut w);
        assert_eq!(a.smoke_stage, LAST);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }));
        assert!(a.pending_events.is_empty());
        assert!(!a.capture_pending);
    }
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(a.selected.is_none());
    assert!(a.progress.favorites.contains(ID));
    let (ctx, mut a) = app_native();
    let mut w = Witness::default();
    a.started = Instant::now();
    actual_tick(&ctx, &mut a, false, &mut w);
    assert_eq!(a.smoke_stage, FIRST, "No NAT pacing exception");
}
fn event_frame(ctx: &egui::Context, a: &mut LabApp, width: f32, events: Vec<egui::Event>) {
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
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 950. / a.scale),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            a.save();
        },
    );
}
fn modal_click(
    ctx: &egui::Context,
    a: &mut LabApp,
    width: f32,
    target: fn(&Metrics) -> Option<egui::Rect>,
    file: bool,
) {
    for _ in 0..30 {
        let r = target(&a.metrics).expect("real modal control");
        let v = if file {
            a.metrics
                .data_file_viewport
                .unwrap()
                .intersect(ctx.content_rect())
        } else if a.settings_open && Some(r) != a.metrics.settings {
            a.metrics
                .settings_tabs
                .first()
                .unwrap()
                .2
                .intersect(ctx.content_rect())
        } else {
            ctx.content_rect()
        };
        let tol = 1. / ctx.pixels_per_point();
        if v.contains(r.center()) && v.expand(tol).contains_rect(r) {
            for pressed in [true, false] {
                event_frame(
                    ctx,
                    a,
                    width,
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
            event_frame(ctx, a, width, vec![]);
            return;
        }
        let delta = if r.bottom() > v.bottom() {
            v.bottom() - r.bottom() - 4.
        } else {
            v.top() - r.top() + 4.
        };
        event_frame(
            ctx,
            a,
            width,
            vec![
                egui::Event::PointerMoved(v.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-300., 300.)),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Modal control not actually visible")
}
fn files(a: &LabApp) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    let mut total = 0usize;
    for (n, item) in std::fs::read_dir(a.path.parent().unwrap())
        .unwrap()
        .enumerate()
    {
        assert!(n < 16);
        let item = item.unwrap();
        assert!(item.file_type().unwrap().is_file());
        let bytes = progress::read_bounded(&item.path(), 512 * 1024).unwrap();
        total = total.checked_add(bytes.len()).unwrap();
        assert!(total <= 1024 * 1024);
        files.insert(item.file_name().into_string().unwrap(), bytes);
    }
    files
}

#[test]
fn real_file_cancel_invalid_name_preserves_files_progress_and_background_access_at_every_scale() {
    for scale in [1., 1.5, 2.] {
        for width in [650., 1400.] {
            let (ctx, mut a) = app_native();
            let mut witness = Witness::default();
            until(&ctx, &mut a, FIRST + 2, &mut witness);
            a.scale = scale;
            for _ in 0..3 {
                event_frame(&ctx, &mut a, width, vec![]);
            }
            let before = serde_json::to_value(&a.progress).unwrap();
            let bytes = files(&a);
            modal_click(&ctx, &mut a, width, |m| m.settings, false);
            modal_click(
                &ctx,
                &mut a,
                width,
                |m| m.settings_tabs.iter().find(|r| r.0 == "data").map(|r| r.1),
                false,
            );
            modal_click(&ctx, &mut a, width, |m| m.data_export, false);
            assert!(a.data_tools.modal_open());
            modal_click(&ctx, &mut a, width, |m| m.data_filename, true);
            event_frame(
                &ctx,
                &mut a,
                width,
                vec![
                    acl_ctrl(egui::Key::A),
                    egui::Event::Text("../invalid-name".into()),
                ],
            );
            assert_eq!(a.data_tools.restore_filename(), Some("../invalid-name"));
            assert!(a.metrics.nat_controls.iter().all(|r| !r.2));
            let (_, close_id, enabled, _) = a
                .metrics
                .settings_close
                .expect("real Settings close remains painted");
            assert!(!enabled, "Settings Close disabled while chooser active");
            ctx.memory_mut(|m| m.request_focus(close_id));
            event_frame(&ctx, &mut a, width, vec![key(egui::Key::Enter, false)]);
            assert!(
                a.settings_open && a.data_tools.modal_open(),
                "Focused disabled Close cannot close a pending chooser"
            );
            let r = a.metrics.data_file_cancel.unwrap();
            let font = egui::TextStyle::Button.resolve(&ctx.style());
            let text =
                ctx.fonts_mut(|f| f.layout_no_wrap("Cancel".into(), font, egui::Color32::WHITE));
            assert!(r.width() + 1. >= text.size().x + 2. * ctx.style().spacing.button_padding.x);
            modal_click(&ctx, &mut a, width, |m| m.data_file_cancel, true);
            assert!(!a.data_tools.modal_open());
            let close = a.metrics.settings_close.unwrap();
            let font = egui::TextStyle::Button.resolve(&ctx.style());
            let glyphs = ctx.fonts_mut(|f| {
                f.layout_no_wrap("Close settings".into(), font, egui::Color32::WHITE)
            });
            assert!(
                close.0.width() + 1. >= glyphs.size().x + 2. * ctx.style().spacing.button_padding.x
            );
            modal_click(
                &ctx,
                &mut a,
                width,
                |m| m.settings_close.map(|r| r.0),
                false,
            );
            assert!(!a.settings_open);
            assert!(a.metrics.nat_controls.iter().all(|r| r.2));
            assert!(serde_json::to_value(&a.progress).unwrap() == before);
            assert!(
                files(&a) == bytes,
                "Invalid-name Cancel must write zero files"
            );
            let applied = live(&a).clone();
            a.smoke_stage = FIRST + 11;
            let mut access = Witness::default();
            until(&ctx, &mut a, FIRST + 12, &mut access);
            assert_eq!(
                raw(&a).network,
                "10.0.0.8",
                "Actual background editing restored after Cancel/settings close"
            );
            assert_eq!(
                live(&a),
                &applied,
                "Edits remain pending until explicit Apply"
            );
        }
    }
}

#[test]
fn single_static_map_shows_heading_inside_and_global_without_inner_scrolling_at_every_scale() {
    for scale in [1., 1.5, 2.] {
        for width in [650., 1400.] {
            let (ctx, mut a) = app_native();
            let mut w = Witness::default();
            until(&ctx, &mut a, FIRST + 2, &mut w);
            a.scale = scale;
            for _ in 0..3 {
                event_frame(&ctx, &mut a, width, vec![]);
            }
            let mut visible = false;
            for _ in 0..40 {
                let child = a.metrics.nat_static_viewport.unwrap();
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let inside = a
                    .metrics
                    .nat_fields
                    .iter()
                    .find(|r| r.0.starts_with("static:") && r.0.ends_with(":inside"))
                    .unwrap()
                    .1;
                let global = a
                    .metrics
                    .nat_fields
                    .iter()
                    .find(|r| r.0.starts_with("static:") && r.0.ends_with(":global"))
                    .unwrap()
                    .1;
                let heading = a
                    .metrics
                    .nat_controls
                    .iter()
                    .find(|r| r.0.starts_with("remove:"))
                    .unwrap()
                    .1;
                let v = child.intersect(root);
                let tolerance = 1. / ctx.pixels_per_point();
                if [inside, global, heading]
                    .iter()
                    .all(|r| v.contains(r.center()) && v.expand(tolerance).contains_rect(*r))
                {
                    assert!(
                        (inside.left() - global.left()).abs() <= tolerance,
                        "Both fields use aligned native value columns"
                    );
                    assert!(
                        a.metrics.nat_static_scroll.unwrap().1.y.abs() <= tolerance,
                        "A single map needs no inner scroll"
                    );
                    visible = true;
                    break;
                }
                // Only the outer page is revealed. Never move the child to hide a
                // missing minimum viewport; both fields must fit simultaneously.
                let target = inside.union(global).union(heading);
                let delta = if target.bottom() > root.bottom() {
                    root.bottom() - target.bottom() - 4.
                } else {
                    root.top() - target.top() + 4.
                };
                event_frame(
                    &ctx,
                    &mut a,
                    width,
                    vec![
                        egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., delta.clamp(-350., 350.)),
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            assert!(
                visible,
                "One complete static map must be visible at scale{scale} width{width}"
            );
        }
    }
}
#[test]
fn native_feedback_uses_one_known_prefix_and_observation_arrows_are_ascii_only() {
    use super::super::nat_workspace::{check_text, observation_lines};
    assert_eq!(
        check_text(true, "Satisfied: Keep observer"),
        "Verified · Keep observer"
    );
    assert_eq!(
        check_text(false, "Needs attention: Missing beta"),
        "Needs attention · Missing beta"
    );
    assert_eq!(
        check_text(false, "Apply pending policy"),
        "Needs attention · Apply pending policy"
    );
    assert_eq!(
        check_text(true, "Unknown wording stays exact"),
        "Verified · Unknown wording stays exact"
    );
    let before = core::Addresses {
        source: "10.0.0.10".parse().unwrap(),
        destination: "203.0.113.1".parse().unwrap(),
    };
    let after = core::Addresses {
        source: "198.51.100.10".parse().unwrap(),
        destination: before.destination,
    };
    let effect = core::Effect::Probe {
        result: core::ProbeResult::Translated {
            before,
            after,
            kind: core::MappingKind::Dynamic,
        },
    };
    let original = serde_json::to_value(&effect).unwrap();
    let lines = observation_lines(&effect);
    assert_eq!(
        lines,
        vec![
            "Dynamic rewrite",
            "Before: 10.0.0.10 -> 203.0.113.1",
            "After: 198.51.100.10 -> 203.0.113.1"
        ]
    );
    assert!(lines.iter().all(|line| line.is_ascii()));
    assert_eq!(
        serde_json::to_value(&effect).unwrap(),
        original,
        "Presentation cannot change the core observation contract"
    );
}
