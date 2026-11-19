use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::arp_inspection_workspace as work;
const IDS: [&str; 3] = [
    "ccna-arp-sender-binding",
    "ccna-arp-static-policy",
    "ccna-arp-field-consistency",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(id)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, input: bool) -> egui::Rect {
    for _ in 0..24 {
        frame(ctx, a, 1200., 950., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, child) = if key == "challenge" {
            (a.metrics.mode_challenge.unwrap(), main)
        } else if input {
            let row = a
                .metrics
                .arp_inspection
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap();
            (row.1, row.3)
        } else {
            let row = a
                .metrics
                .arp_inspection
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap();
            (row.1, row.2)
        };
        let clip = main.intersect(child);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(main.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (main.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("ARP control never became visible: {key}");
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
        .arp_inspection
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
fn arp_boundary_actual_widgets_edit_apply_replay_check_and_clear_stale_success() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "vlans", "ten");
    tap(&ctx, &mut a, "apply");
    assert!(a.notice.as_deref().unwrap().contains("VLAN"));
    field(&ctx, &mut a, "vlans", "10");
    tap(&ctx, &mut a, "trust-server");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "replay");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    let State::ArpInspection { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.world.dhcp().bindings().len(), 1);
    field(&ctx, &mut a, "vlans", "");
    assert!(!a.metrics.completion);
    assert!(a.metrics.arp_inspection.result.is_none());
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    field(&ctx, &mut a, "vlans", "10");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}
#[test]
fn arp_static_rule_editor_reports_partial_work_and_requires_dhcp_fallback() {
    let (ctx, mut a, i) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "list-0-rule-0-mac", "invalid");
    tap(&ctx, &mut a, "apply");
    assert!(a.notice.as_deref().unwrap().contains("MAC pairs"));
    let mac = work::source(&a.scenarios[i])
        .unwrap()
        .initial
        .reference
        .filters[0]
        .rules[0]
        .sender_mac
        .text();
    field(&ctx, &mut a, "list-0-rule-0-mac", &mac);
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    let (p, t, _) = a.metrics.arp_inspection.result.unwrap();
    assert_eq!((p, t), (4, 5));
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "list-0-static");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "replay");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    field(&ctx, &mut a, "list-0-rule-0-mac", "02:00:00:00:00:09");
    assert!(!a.metrics.completion);
    assert!(a.metrics.arp_inspection.result.is_none());
}
#[test]
fn arp_field_comparisons_show_specific_cases_and_do_not_reject_unknown_request_targets() {
    let (ctx, mut a, i) = opened(IDS[2]);
    tap(&ctx, &mut a, "assignment");
    target(&ctx, &mut a, "replay", false);
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Ethernet source MAC"));
    assert!(text.contains("ARP sender MAC"));
    tap(&ctx, &mut a, "check");
    let (p, t, _) = a.metrics.arp_inspection.result.unwrap();
    assert_eq!((p, t), (3, 6));
    tap(&ctx, &mut a, "check-source");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    let (p, t, _) = a.metrics.arp_inspection.result.unwrap();
    assert_eq!((p, t), (5, 6));
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check-destination");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "replay");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
    tap(&ctx, &mut a, "check-source");
    assert!(!a.metrics.completion);
    assert!(a.metrics.arp_inspection.result.is_none());
}
#[test]
fn arp_worked_practice_is_isolated_and_challenge_has_no_unrequested_solution() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "worked-example");
        tap(&ctx, &mut a, "fill-example");
        tap(&ctx, &mut a, "replay");
        tap(&ctx, &mut a, "check");
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let State::ArpInspection { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "challenge");
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Apply worked example"));
        assert!(!text.contains("Worked practice policy"));
        let State::ArpInspection { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, practice);
    }
}
#[test]
fn arp_all_three_tools_reflow_five_scales_light_dark_and_disabled_without_clipping() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        let State::ArpInspection { simulation } =
            &mut a.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
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
                        for (key, r, _, _) in &a.metrics.arp_inspection.fields {
                            assert!(
                                r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                                "{id} {key} {scale}/{width}/{light}: {r:?} {clip:?}"
                            );
                            assert!(r.height() > 20.);
                        }
                        let viewport = a.metrics.arp_inspection.replay_output.unwrap();
                        let font = egui::TextStyle::Body.resolve(&ctx.style());
                        let h = ctx.fonts_mut(|f| f.row_height(&font));
                        assert!((viewport.height() - 8. * h).abs() < 1.);
                        assert!(
                            viewport.right() <= clip.right() + 1.,
                            "{id} output{viewport:?} outside{clip:?}"
                        );
                        for key in ["sample", "apply", "replay", "check"] {
                            let r = a
                                .metrics
                                .arp_inspection
                                .controls
                                .iter()
                                .find(|r| r.0 == key)
                                .unwrap()
                                .1;
                            assert!(
                                r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                                "{id} {key} {scale}/{width}/{light}: {r:?} {clip:?}"
                            );
                        }
                    }
                    a.reset_pending = false;
                }
            }
        }
    }
}
#[test]
fn arp_rule_field_focus_is_stable_through_reflow_and_tab_moves_on() {
    let (ctx, mut a, _) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "list-0-rule-0-ip", "10.30.0.99");
    let id = a
        .metrics
        .arp_inspection
        .fields
        .iter()
        .find(|r| r.0 == "list-0-rule-0-ip")
        .unwrap()
        .2;
    for width in [760., 1600., 760.] {
        frame(&ctx, &mut a, width, 950., vec![]);
        let current = a
            .metrics
            .arp_inspection
            .fields
            .iter()
            .find(|r| r.0 == "list-0-rule-0-ip")
            .unwrap()
            .2;
        assert_eq!(id, current);
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
