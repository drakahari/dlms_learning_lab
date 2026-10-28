//! Fixed native lesson controller interaction checks.
//! Headless controller returns are inspected; no renderer/screenshot/native claim.
use super::*;
const FIRST: usize = 31000;
const LAST: usize = 31246;
const IDS: [&str; 3] = [
    "ccna-dhcp-held-options",
    "ccna-dhcp-relay-scope",
    "ccna-dhcp-lease-handover",
];
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.check_dhcpv4_lesson_catalog();
    (ctx, a)
}
fn controller_frame(ctx: &egui::Context, a: &mut LabApp, router: bool) -> Option<String> {
    let (width, height, density) = ctx.data(|d| {
        d.get_temp::<(f32, f32, f32)>(egui::Id::new("dhcp-test-physical-viewport"))
            .unwrap_or((1200., 850., 1.))
    });
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .find_map(|e| match e {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or_default();
    let mut shot = None;
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale / density, height / a.scale / density),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            if router {
                a.smoke_tick(ctx);
            } else {
                let (advance, capture) = a.dhcpv4_lessons_native(ctx, a.smoke_stage);
                if advance {
                    a.smoke_stage += 1;
                }
                shot = capture;
            }
        },
    );
    shot
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, frames: usize) -> Vec<String> {
    let mut shots = Vec::new();
    for _ in 0..frames {
        if a.smoke_stage == target {
            return shots;
        }
        assert!(
            a.smoke_stage < target,
            "closed controller cannot skip target"
        );
        if let Some(s) = controller_frame(ctx, a, false) {
            shots.push(s);
        }
    }
    panic!(
        "fixed native phase did not finish: {} -> {target}",
        a.smoke_stage
    )
}
#[test]
fn dhcp_lessons_native_exact_smoketick_open_geometry_and_repeated_finish_hold() {
    let (ctx, mut a) = opened();
    a.smoke_stage = FIRST;
    controller_frame(&ctx, &mut a, true);
    assert_eq!(a.smoke_stage, FIRST + 1);
    assert_eq!(a.scenarios.len(), 164);
    assert_eq!(
        a.dhcpv4_view(a.selected.unwrap()),
        teaching::View::Introduction
    );
    for _ in 0..6 {
        controller_frame(&ctx, &mut a, true);
        if a.smoke_stage == FIRST + 2 {
            break;
        }
    }
    assert_eq!(a.smoke_stage, FIRST + 2);
    assert!(!a.capture_pending);
    a.smoke_stage = LAST;
    for _ in 0..2 {
        controller_frame(&ctx, &mut a, true);
        assert_eq!(a.smoke_stage, LAST);
        assert!(a.pending_events.is_empty());
    }
}
#[test]
fn dhcp_lessons_native_actual_first_example_fill_actions_experiment_repair_and_isolation() {
    let (ctx, mut a) = opened();
    a.smoke_stage = FIRST;
    let captures = until(&ctx, &mut a, FIRST + 16, 900);
    let s = a.scenarios.iter().find(|s| s.id == IDS[0]).unwrap();
    let attempt = &a.progress.attempts[IDS[0]];
    let l = attempt
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_learning
        .as_ref()
        .unwrap();
    assert_eq!(attempt.state, s.initial);
    assert!(
        l.checks(s.dhcpv4_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert!(!progress::current_solved(attempt, s));
    assert!(captures.iter().any(|s| s.ends_with("fill-only")));
    assert!(captures.iter().any(|s| s.ends_with("example-experiment")));
    assert!(captures.iter().any(|s| s.ends_with("example-repaired")));
}
#[test]
fn dhcp_lessons_native_all_three_full_flows_at_observed_fedora_window_and_density() {
    let (ctx, mut a) = opened();
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("dhcp-test-physical-viewport"),
            (946_f32, 929_f32, 1.15_f32),
        );
    });
    for (n, id) in IDS.iter().enumerate() {
        a.smoke_stage = FIRST + n * 82;
        let captures = until(&ctx, &mut a, FIRST + (n + 1) * 82, 3600);
        let s = a.scenarios.iter().find(|s| s.id == *id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[*id], s));
        for suffix in [
            "light150-typed",
            "dark200-typed",
            "pending-warm-resume",
            "alternate-complete",
            "light150-focused-policy-field",
            "dark200-focused-policy-field",
        ] {
            assert!(
                captures.iter().any(|c| c.ends_with(suffix)),
                "{id}: {suffix}"
            );
        }
    }
}
#[test]
fn dhcp_lessons_native_capstone_full_controller_lists_fields_request_scale_resume_reset_and_alternate()
 {
    let (ctx, mut a) = opened();
    a.smoke_stage = FIRST + 82 * 2;
    let captures = until(&ctx, &mut a, LAST, 3600);
    let s = a.scenarios.iter().find(|s| s.id == IDS[2]).unwrap();
    let attempt = &a.progress.attempts[IDS[2]];
    assert!(progress::current_solved(attempt, s));
    assert!(attempt.challenge);
    assert!(a.progress.favorites.contains(IDS[2]));
    assert!(a.selected.is_none());
    for suffix in [
        "fill-only",
        "example-experiment",
        "example-repaired",
        "wrong-feedback",
        "partial-feedback",
        "reference-complete",
        "pending-invalidates",
        "new-client-policy-fails",
        "light150-typed",
        "dark200-typed",
        "pending-warm-resume",
        "alternate-complete",
        "challenge",
    ] {
        assert!(captures.iter().any(|c| c.ends_with(suffix)), "{suffix}");
    }
    let engine::State::DhcpV4 { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(
        simulation.policy().pools["handover-scope"].reservations["handover-kiosk"],
        "172.20.5.12".parse::<std::net::Ipv4Addr>().unwrap()
    );
    assert!(
        simulation.policy().pools["handover-scope"]
            .excluded
            .contains(&"172.20.5.10".parse().unwrap())
    );
    assert!(!simulation.leases().contains_key("handover-retired"));
}
#[test]
fn dhcp_lessons_native_relay_flow_reveals_wrong_mask_then_current_new_client_and_correct_alternate()
{
    let (ctx, mut a) = opened();
    a.smoke_stage = FIRST + 82;
    let captures = until(&ctx, &mut a, FIRST + 82 * 2, 3000);
    let s = a.scenarios.iter().find(|s| s.id == IDS[1]).unwrap();
    let attempt = &a.progress.attempts[IDS[1]];
    assert!(progress::current_solved(attempt, s));
    let engine::State::DhcpV4 { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(
        simulation.leases()["annex-branch"].parameters.subnet.prefix,
        24
    );
    assert_eq!(
        simulation.leases()["annex-visitor"]
            .parameters
            .subnet
            .prefix,
        24
    );
    assert!(captures.iter().any(|c| c.ends_with("wrong-feedback")));
    assert!(captures.iter().any(|c| c.ends_with("alternate-complete")));
}
