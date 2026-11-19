use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::dhcp_snooping_workspace as work;
const IDS: [&str; 2] = ["ccna-dhcp-server-boundary", "ccna-dhcp-client-mac-check"];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(id)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, input: bool) -> egui::Rect {
    for _ in 0..24 {
        frame(ctx, a, 1200., 950., vec![]);
        let r = if key == "challenge" {
            a.metrics.mode_challenge.unwrap()
        } else if input {
            a.metrics
                .dhcp_snooping
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        } else {
            a.metrics
                .dhcp_snooping
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("DHCP observer control never became visible: {key}");
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key, true);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = a
        .metrics
        .dhcp_snooping
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(id),
        "Field {key} must receive focus in the same measured viewport"
    );
    frame(
        ctx,
        a,
        1200.,
        950.,
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
            if text.is_empty() {
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }
            } else {
                egui::Event::Paste(text.into())
            },
        ],
    );
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}

#[test]
fn dhcp_native_policy_edit_replay_check_retry_and_stale_success_use_real_widgets() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "vlans", "ten");
    tap(&ctx, &mut a, "apply");
    assert!(a.notice.as_deref().unwrap().contains("VLANs"));
    assert!(!a.metrics.completion);
    field(&ctx, &mut a, "vlans", "10");
    tap(&ctx, &mut a, "enabled");
    tap(&ctx, &mut a, "trust-server");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "replay");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    let State::DhcpSnooping { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.world.bindings().len(), 1);
    assert!(!simulation.assignment.output.is_empty());
    field(&ctx, &mut a, "vlans", "");
    let State::DhcpSnooping { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(simulation.assignment.draft.vlans.is_empty());
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    field(&ctx, &mut a, "vlans", "10");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    field(&ctx, &mut a, "vlans", "20");
    assert!(!a.metrics.completion);
    assert!(a.metrics.dhcp_snooping.result.is_none());
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    tap(&ctx, &mut a, "apply");
    assert!(a.notice.is_some());
    field(&ctx, &mut a, "vlans", "10");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    tap(&ctx, &mut a, "enabled");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    let (p, t, _) = a.metrics.dhcp_snooping.result.unwrap();
    assert!(p < t);
}
#[test]
fn dhcp_native_partial_results_and_mac_fields_are_specific_and_visible_without_optional_detail() {
    let (ctx, mut a, i) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Ethernet source MAC"));
    assert!(text.contains("DHCP client MAC"));
    tap(&ctx, &mut a, "check");
    let (p, t, _) = a.metrics.dhcp_snooping.result.unwrap();
    assert!(p > 0 && p < t);
    tap(&ctx, &mut a, "replay");
    let State::DhcpSnooping { simulation } = &a.progress.attempts[IDS[1]].state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .output
            .iter()
            .any(|l| l.text.contains("inspected"))
    );
    tap(&ctx, &mut a, "verify-source");
    tap(&ctx, &mut a, "apply");
    let State::DhcpSnooping { simulation } = &a.progress.attempts[IDS[1]].state else {
        panic!()
    };
    assert!(simulation.assignment.output.is_empty());
    tap(&ctx, &mut a, "replay");
    let State::DhcpSnooping { simulation } = &a.progress.attempts[IDS[1]].state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .output
            .iter()
            .any(|l| l.text.contains("rejected"))
    );
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[1]],
        &a.scenarios[i]
    ));
    tap(&ctx, &mut a, "verify-source");
    assert!(!a.metrics.completion);
    assert!(a.metrics.dhcp_snooping.result.is_none());
}
#[test]
fn dhcp_introduction_separate_worked_practice_and_independent_assignment_preserve_drafts() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let d = work::source(&source).unwrap();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("WHAT YOU’LL LEARN"));
        assert!(text.contains("Worked"));
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "worked-example");
        tap(&ctx, &mut a, "fill-example");
        tap(&ctx, &mut a, "replay");
        tap(&ctx, &mut a, "check");
        let State::DhcpSnooping { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(
            d.example
                .checks(&simulation.practice.world)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(simulation.assignment.world, d.initial.initial);
        let practice = simulation.practice.clone();
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains(work::PREFIX));
        assert!(!text.contains(&source.explanation));
        let State::DhcpSnooping { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, practice);
        assert!(!simulation.assignment.checked);
    }
}
#[test]
fn dhcp_controls_reflow_five_scales_light_dark_populated_and_disabled() {
    let (ctx, mut a, _) = opened(IDS[0]);
    let State::DhcpSnooping { simulation } =
        &mut a.progress.attempts.get_mut(IDS[0]).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    simulation.assignment.draft.vlans = "10, 20, 30".into();
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        for width in [760., 1600.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(&ctx, &a.theme_id);
                for disabled in [false, true] {
                    a.reset_pending = disabled;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for (key, r, _, _) in &a.metrics.dhcp_snooping.fields {
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "{key} {scale} {width} {light}: {r:?} {clip:?}"
                        );
                        assert!(r.height() > 20.);
                    }
                    let viewport = a.metrics.dhcp_snooping.replay_output.unwrap();
                    let font = egui::TextStyle::Body.resolve(&ctx.style());
                    let height = ctx.fonts_mut(|f| f.row_height(&font));
                    assert!(
                        (viewport.height() - height * 8.).abs() < 1.,
                        "Replay reserves readable scrolling space at {scale}/{width}"
                    );
                    assert!(
                        viewport.right() <= clip.right() + 1.,
                        "Replay viewport {viewport:?} outside {clip:?} at {scale}/{width}/{light}"
                    );
                    for key in [
                        "apply",
                        "replay",
                        "check",
                        "sample",
                        "trust-server",
                        "trust-client",
                    ] {
                        let r = a
                            .metrics
                            .dhcp_snooping
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap()
                            .1;
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "{key} {scale} {width} {light}: {r:?} {clip:?}"
                        );
                    }
                }
                a.reset_pending = false;
            }
        }
    }
}
#[test]
fn dhcp_stable_field_focus_survives_reflow_and_tab_reaches_the_next_control() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "vlans", "10");
    let id = a.metrics.dhcp_snooping.fields[0].2;
    for width in [760., 1600., 760.] {
        frame(&ctx, &mut a, width, 950., vec![]);
        assert_eq!(a.metrics.dhcp_snooping.fields[0].2, id);
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    }
    frame(
        &ctx,
        &mut a,
        760.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
}
