//! Prospective native-egui interaction checks. No actual renderer/cold-start claim.
use super::*;
use dlms_learning_lab::{dhcpv4::State as Simulation, dhcpv4_workspace as work};
const ID: &str = "dhcpv4-workspace-preview";
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.add_dhcpv4_fixture();
    a.launch(i);
    a.set_dhcpv4_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn locate(
    ctx: &egui::Context,
    a: &mut LabApp,
    find: impl Fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for _ in 0..30 {
        let r = find(&a.metrics).expect("actual DHCP control exists");
        let v = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let tolerance = 1. / ctx.pixels_per_point();
        if v.contains(r.center()) && v.expand(tolerance).contains_rect(r) {
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
    panic!("DHCP control must become actually visible");
}
fn control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = locate(ctx, a, |m| {
        m.dhcp_controls
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, r, _)| *r)
    });
    click(ctx, a, r);
    draw(ctx, a);
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = locate(ctx, a, |m| {
        m.dhcp_fields
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, r, _)| *r)
    });
    click(ctx, a, r);
    let id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(k, _, _)| k == key)
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    draw(ctx, a);
}
fn check_work(ctx: &egui::Context, a: &mut LabApp) {
    let r = locate(ctx, a, |m| m.check);
    click(ctx, a, r);
    draw(ctx, a);
}
fn live(a: &LabApp) -> &Simulation {
    let State::DhcpV4 { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn draft(a: &LabApp) -> &work::Draft {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_draft
        .as_ref()
        .unwrap()
}
fn repair_through_controls(ctx: &egui::Context, a: &mut LabApp) {
    field(ctx, a, "relay-target", "192.0.2.1");
    field(ctx, a, "router", "10.20.0.1");
    if !a.metrics.dhcp_fields.iter().any(|(k, _, _)| k == "dns-0") {
        control(ctx, a, "dns-header");
    }
    field(ctx, a, "dns-0", "203.0.113.53");
    control(ctx, a, "apply");
    control(ctx, a, "discover");
    assert!(!live(a).leases().contains_key("branch"));
    assert!(live(a).offers().contains_key("branch"));
    control(ctx, a, "request");
    check_work(ctx, a);
}
#[test]
fn dhcp_native_introduction_separate_fill_apply_offer_request_experiment_and_repair() {
    let (ctx, mut a, i) = opened(teaching::View::Introduction);
    let original = a.progress.attempts[ID].state.clone();
    assert!(a.metrics.dhcp_fields.is_empty());
    let r = locate(&ctx, &mut a, |m| m.example_action);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.dhcpv4_view(i), teaching::View::Example);
    let before = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_learning
        .as_ref()
        .unwrap()
        .simulation
        .clone();
    let r = locate(&ctx, &mut a, |m| m.insert);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, before);
    assert!(!l.draft.matches(&l.simulation));
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "discover");
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_learning
        .as_ref()
        .unwrap();
    assert!(l.simulation.offers().contains_key("museum-branch"));
    assert!(!l.simulation.leases().contains_key("museum-branch"));
    control(&ctx, &mut a, "request");
    check_work(&ctx, &mut a);
    let lesson = a.scenarios[i].dhcpv4_teaching.as_ref().unwrap().clone();
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .dhcpv4_learning
            .as_ref()
            .unwrap()
            .checks(&lesson)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, original);
    assert!(!a.metrics.completion);
    control(&ctx, &mut a, "release");
    check_work(&ctx, &mut a);
    assert!(
        !a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .dhcpv4_learning
            .as_ref()
            .unwrap()
            .checks(&lesson)[1]
            .0
    );
    control(&ctx, &mut a, "discover");
    control(&ctx, &mut a, "request");
    check_work(&ctx, &mut a);
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .dhcpv4_learning
            .as_ref()
            .unwrap()
            .checks(&lesson)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, original);
}
#[test]
fn dhcp_native_wrong_relay_offer_only_full_repair_and_wellformed_wrong_options_are_distinct() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    control(&ctx, &mut a, "discover");
    assert!(!live(&a).offers().contains_key("branch"));
    assert!(
        a.notice
            .as_deref()
            .is_some_and(|s| s.contains("relay targets another server"))
    );
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    repair_through_controls(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    let held = live(&a).leases()["branch"].clone();
    field(&ctx, &mut a, "dns-0", "203.0.113.88");
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    control(&ctx, &mut a, "apply");
    assert_eq!(live(&a).leases()["branch"], held);
    assert_eq!(
        live(&a).policy().pools["remote"].dns,
        vec!["203.0.113.88".parse::<std::net::Ipv4Addr>().unwrap()]
    );
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let rows = engine::checks(&a.progress.attempts[ID].state, &a.scenarios[i].goal);
    assert!(rows[0].0);
    assert!(rows[5..].iter().any(|r| !r.0));
    control(&ctx, &mut a, "renew");
    assert_ne!(live(&a).leases()["branch"].parameters, held.parameters);
    field(&ctx, &mut a, "dns-0", "203.0.113.53");
    control(&ctx, &mut a, "apply");
    control(&ctx, &mut a, "renew");
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn dhcp_native_invalid_raw_draft_save_resume_and_byte_bound_never_partially_apply() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    repair_through_controls(&ctx, &mut a);
    let before = live(&a).clone();
    field(&ctx, &mut a, "lease-seconds", "not a duration");
    control(&ctx, &mut a, "apply");
    assert_eq!(live(&a), &before);
    assert_eq!(draft(&a).pools["remote"].lease_seconds, "not a duration");
    assert!(a.notice.is_some());
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let prior = serde_json::to_value(&a.progress.attempts).unwrap();
    a.save();
    assert!(a.storage_error.is_none());
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(serde_json::to_value(&a.progress.attempts).unwrap(), prior);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "lease-seconds", "96");
    let prior = draft(&a).clone();
    field(&ctx, &mut a, "lease-seconds", &"é".repeat(33));
    assert_eq!(draft(&a), &prior);
    assert!(a.notice.as_deref().is_some_and(|n| n.contains("64 UTF8")));
    assert_eq!(live(&a), &before);
}
#[test]
fn dhcp_native_all_modal_guards_block_assignment_and_practice_input_and_actions() {
    for view in [teaching::View::Assignment, teaching::View::Example] {
        for blocked in 0..4 {
            for input in 0..2 {
                let (ctx, mut a, _) = opened(view);
                // Keep the real data chooser inside this tool-created profile;
                // normal backup_folder otherwise uses the user's Documents.
                a.smoke = true;
                let action = if input == 0 {
                    let r = locate(&ctx, &mut a, |m| {
                        m.dhcp_fields
                            .iter()
                            .find(|(k, _, _)| k == "relay-target")
                            .map(|(_, r, _)| *r)
                    });
                    click(&ctx, &mut a, r);
                    None
                } else {
                    Some(locate(&ctx, &mut a, |m| {
                        m.dhcp_controls
                            .iter()
                            .find(|(k, _, _)| k == "advance")
                            .map(|(_, r, _)| *r)
                    }))
                };
                let before = serde_json::to_value(&a.progress.attempts).unwrap();
                match blocked {
                    0 => a.reset_pending = true,
                    1 => a.settings_open = true,
                    2 => {
                        a.settings_open = true;
                        a.settings_data = true;
                        draw(&ctx, &mut a);
                        let export = a
                            .metrics
                            .data_export
                            .expect("actual Settings export control");
                        click(&ctx, &mut a, export);
                        draw(&ctx, &mut a);
                        assert!(a.data_tools.modal_open());
                        a.settings_open = false;
                    }
                    _ => a.theme_preview = true,
                }
                draw(&ctx, &mut a);
                if let Some(r) = action {
                    click(&ctx, &mut a, r);
                } else {
                    frame(
                        &ctx,
                        &mut a,
                        1200.,
                        850.,
                        vec![
                            acl_ctrl(egui::Key::A),
                            egui::Event::Text("192.0.2.1".into()),
                            acl_ctrl(egui::Key::Enter),
                        ],
                    );
                }
                draw(&ctx, &mut a);
                assert_eq!(
                    serde_json::to_value(&a.progress.attempts).unwrap(),
                    before,
                    "background {view:?}/{blocked}/{input}"
                );
                assert!(!a.metrics.dhcp_controls.is_empty());
                assert!(
                    a.metrics
                        .dhcp_controls
                        .iter()
                        .all(|(_, _, enabled)| !*enabled)
                );
                assert!(!a.focus_input);
            }
        }
    }
}

#[test]
fn dhcp_native_pending_drafts_survive_mode_and_selected_reset_keeps_other_attempts_and_favorite() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    repair_through_controls(&ctx, &mut a);
    let original_count = a.scenarios.len();
    let mut second = a.scenarios[i].clone();
    second.id = "dhcpv4-second-private-fixture".into();
    let j = a.scenarios.len();
    a.scenarios.push(second);
    a.launch(j);
    a.launch(i);
    a.set_dhcpv4_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let other = serde_json::to_value(&a.progress.attempts[&a.scenarios[j].id]).unwrap();
    a.progress.favorites.insert(ID.into());
    field(&ctx, &mut a, "lease-seconds", "pending");
    let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    let r = locate(&ctx, &mut a, |m| m.mode_challenge);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    assert_eq!(draft(&a).pools["remote"].lease_seconds, "pending");
    let r = locate(&ctx, &mut a, |m| m.mode_learn);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
        before
    );
    let r = locate(&ctx, &mut a, |m| m.reset);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = locate(&ctx, &mut a, |m| m.reset_cancel);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
        before
    );
    let r = locate(&ctx, &mut a, |m| m.reset);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = locate(&ctx, &mut a, |m| m.confirm_reset);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[j].id]).unwrap(),
        other
    );
    assert!(a.progress.favorites.contains(ID));
    assert_eq!(a.scenarios.len(), original_count + 1);
}
#[test]
fn dhcp_native_field_ids_keep_undo_separate_between_examples_and_labs_and_defer_auto_focus() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    field(&ctx, &mut a, "router", "10.20.0.2");
    let assignment_id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(k, _, _)| k == "router")
        .unwrap()
        .2;
    a.set_dhcpv4_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let example_id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(k, _, _)| k == "router")
        .unwrap()
        .2;
    assert_ne!(assignment_id, example_id);
    let mut second = a.scenarios[i].clone();
    second.id = "dhcpv4-undo-private-fixture".into();
    let j = a.scenarios.len();
    a.scenarios.push(second);
    a.launch(j);
    a.set_dhcpv4_view(j, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let second_id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(k, _, _)| k == "router")
        .unwrap()
        .2;
    assert_ne!(assignment_id, second_id);
    let r = locate(&ctx, &mut a, |m| {
        m.dhcp_fields
            .iter()
            .find(|(k, _, _)| k == "router")
            .map(|(_, r, _)| *r)
    });
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    let d = a.progress.attempts[&a.scenarios[j].id]
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_draft
        .as_ref()
        .unwrap();
    assert_eq!(d.pools["remote"].router, "10.20.0.254");
    a.focus_input = true;
    let p = a.metrics.main_viewport.unwrap().left_top() + egui::vec2(2., 2.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    assert!(a.focus_input);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut a);
    assert!(!a.focus_input);
}
#[test]
fn dhcp_native_control_grid_and_truthful_topology_fit_narrow_and_large_text_views() {
    for width in [720., 1200.] {
        for scale in [1., 1.5, 2.] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                let (ctx, mut a, _) = opened(view);
                a.scale = scale;
                for _ in 0..6 {
                    frame(&ctx, &mut a, width, 850., vec![]);
                }
                if view != teaching::View::Introduction {
                    let clip = a.metrics.main_viewport.unwrap();
                    let fields = &a.metrics.dhcp_fields;
                    assert!(!fields.is_empty());
                    let height = fields[0].1.height();
                    for (_, r, _) in fields {
                        assert!((r.height() - height).abs() < 1.1);
                        assert!(r.left() >= clip.left() - 1.);
                        assert!(
                            r.right() <= clip.right() + 1.,
                            "field overflows {width}/{scale}: {r:?}/{clip:?}"
                        );
                    }
                    for r in a.metrics.dhcp_client.into_iter().chain(a.metrics.dhcp_pool) {
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "dropdown overflows {width}/{scale}: {r:?}/{clip:?}"
                        );
                    }
                    assert!(a.metrics.dhcp_topology.is_some());
                    assert!(a.metrics.dhcp_status.is_some());
                }
            }
        }
    }
}
#[test]
fn dhcp_native_policy_and_client_tools_share_a_row_when_wide_and_stack_when_narrow() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    let network = |a: &LabApp| {
        a.metrics
            .dhcp_fields
            .iter()
            .find(|(key, _, _)| key == "network")
            .unwrap()
            .1
    };
    let status = a.metrics.dhcp_status.unwrap();
    assert!(network(&a).right() < status.left());
    assert!(network(&a).top() - status.top() < network(&a).height() * 5.);
    for _ in 0..6 {
        frame(&ctx, &mut a, 720., 850., vec![]);
    }
    assert!(network(&a).top() > a.metrics.dhcp_status.unwrap().bottom());
}
#[test]
fn dhcp_native_reflow_keeps_pending_text_focus_and_real_undo_in_the_same_field() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    let before = live(&a).clone();
    field(&ctx, &mut a, "router", "unfinished router");
    let router_id = |a: &LabApp| {
        a.metrics
            .dhcp_fields
            .iter()
            .find(|(key, _, _)| key == "router")
            .unwrap()
            .2
    };
    let id = router_id(&a);
    a.scale = 1.5;
    for _ in 0..6 {
        frame(&ctx, &mut a, 720., 850., vec![]);
    }
    assert_eq!(router_id(&a), id);
    assert!(ctx.memory(|m| m.has_focus(id)));
    assert_eq!(draft(&a).pools["remote"].router, "unfinished router");
    a.scale = 1.;
    draw(&ctx, &mut a);
    assert_eq!(router_id(&a), id);
    assert!(ctx.memory(|m| m.has_focus(id)));
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
        draw(&ctx, &mut a);
        if draft(&a).pools["remote"].router == "10.20.0.254" {
            break;
        }
    }
    assert_eq!(draft(&a).pools["remote"].router, "10.20.0.254");
    assert_eq!(live(&a), &before);
}
#[test]
fn dhcp_native_long_selected_pool_and_raw_values_fit_without_truncating_saved_text() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let name = format!("remote-{}", "p".repeat(41));
    let raw = "9".repeat(64);
    {
        let d = a
            .progress
            .attempts
            .get_mut(ID)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .dhcpv4_draft
            .as_mut()
            .unwrap();
        let mut pool = d.pools.remove("remote").unwrap();
        pool.router = raw.clone();
        pool.lease_seconds = raw.clone();
        d.pools.insert(name.clone(), pool);
        d.selected_pool = Some(name.clone());
        assert!(d.valid());
    }
    for width in [720., 1200.] {
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            for _ in 0..6 {
                frame(&ctx, &mut a, width, 850., vec![]);
            }
            let clip = a.metrics.main_viewport.unwrap();
            for r in a
                .metrics
                .dhcp_fields
                .iter()
                .map(|(_, r, _)| *r)
                .chain(a.metrics.dhcp_pool)
            {
                assert!(r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.);
            }
            assert_eq!(draft(&a).selected_pool.as_deref(), Some(name.as_str()));
            assert_eq!(draft(&a).pools[&name].router, raw);
            assert_eq!(draft(&a).pools[&name].lease_seconds, raw);
        }
    }
    a.scale = 1.;
    draw(&ctx, &mut a);
    field(&ctx, &mut a, "router", "192.0.2.5");
    assert_eq!(draft(&a).pools[&name].router, "192.0.2.5");
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn dhcp_native_added_rows_stay_open_focus_new_text_and_survive_reflow_without_applying() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    let before = live(&a).clone();
    control(&ctx, &mut a, "excluded-header");
    control(&ctx, &mut a, "add-exclusion");
    let field_id = |a: &LabApp, key: &str| {
        a.metrics
            .dhcp_fields
            .iter()
            .find(|(k, _, _)| k == key)
            .unwrap()
            .2
    };
    assert!(ctx.memory(|m| m.has_focus(field_id(&a, "excluded-0"))));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("10.20.0.18".into())],
    );
    draw(&ctx, &mut a);
    control(&ctx, &mut a, "add-exclusion");
    let id = field_id(&a, "excluded-1");
    assert!(ctx.memory(|m| m.has_focus(id)));
    a.scale = 1.5;
    for _ in 0..6 {
        frame(&ctx, &mut a, 720., 850., vec![]);
    }
    assert_eq!(field_id(&a, "excluded-1"), id);
    assert!(ctx.memory(|m| m.has_focus(id)));
    assert_eq!(draft(&a).pools["remote"].excluded, ["10.20.0.18", ""]);
    a.scale = 1.;
    draw(&ctx, &mut a);
    control(&ctx, &mut a, "reservations-header");
    control(&ctx, &mut a, "add-reservation");
    let id = field_id(&a, "reserved-client-0");
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("branch".into())],
    );
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).pools["remote"].reservations[0].client, "branch");
    assert_eq!(live(&a), &before, "adding and typing rows is not Apply");
}
// Exercise actual capture-phase code with a real egui Context. Do not call any
// Context API while another Context data guard is held.
fn native_tick(ctx: &egui::Context, a: &mut LabApp) {
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
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 850.),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.render(ctx);
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        },
    );
}
#[test]
fn dhcp_native_exact_open_geometry_and_delayed_close_phases_are_bounded_and_private() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 30000;
    let count = a.scenarios.len();
    native_tick(&ctx, &mut a);
    assert_eq!(a.scenarios.len(), count + 1);
    assert_eq!(a.smoke_stage, 30001);
    assert_eq!(
        a.path.parent(),
        Some(a._smoke_profile.as_ref().unwrap().path())
    );
    for _ in 0..5 {
        native_tick(&ctx, &mut a);
        if a.smoke_stage == 30002 {
            break;
        }
    }
    assert_eq!(a.smoke_stage, 30002);
    assert!(!a.capture_pending);
    a.smoke_stage = 30095;
    for _ in 0..2 {
        native_tick(&ctx, &mut a);
        assert_eq!(a.smoke_stage, 30095);
        assert!(a.pending_events.is_empty());
    }
}
#[test]
fn dhcp_native_selected_client_queries_show_independent_current_state_without_exchange() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    let before = live(&a).clone();
    let r = locate(&ctx, &mut a, |m| m.dhcp_client);
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = a
        .metrics
        .dhcp_clients
        .iter()
        .find(|(id, _)| id == "peer")
        .unwrap()
        .1;
    assert!(ctx.content_rect().contains(r.center()));
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).selected_client, "peer");
    assert_eq!(draft(&a).selected_pool.as_deref(), Some("desk"));
    assert_eq!(live(&a), &before);
    assert!(
        a.metrics
            .dhcp_controls
            .iter()
            .any(|(key, _, enabled)| key == "renew" && *enabled)
    );
    assert!(
        a.metrics
            .dhcp_controls
            .iter()
            .any(|(key, _, enabled)| key == "request" && !*enabled)
    );
    assert!(
        a.metrics
            .dhcp_controls
            .iter()
            .any(|(key, _, enabled)| key == "discover" && !*enabled)
    );
    control(&ctx, &mut a, "request");
    assert_eq!(live(&a), &before);
}
#[test]
fn dhcp_native_clock_requires_explicit_button_and_window_controls_never_auto_renew() {
    let (ctx, mut a, _) = opened(teaching::View::Assignment);
    repair_through_controls(&ctx, &mut a);
    let held = live(&a).leases()["branch"].clone();
    assert!(
        a.metrics
            .dhcp_controls
            .iter()
            .any(|(key, _, enabled)| key == "rebind" && !*enabled)
    );
    control(&ctx, &mut a, "rebind");
    assert_eq!(live(&a).leases()["branch"], held);
    field(&ctx, &mut a, "time", "32");
    assert_eq!(live(&a).now(), 0);
    control(&ctx, &mut a, "advance");
    assert_eq!(live(&a).now(), 32);
    assert_eq!(live(&a).leases()["branch"], held);
    assert_eq!(
        live(&a).window("branch").unwrap(),
        dlms_learning_lab::dhcpv4::Window::Renewal
    );
    field(&ctx, &mut a, "time", "24");
    control(&ctx, &mut a, "advance");
    assert_eq!(
        live(&a).window("branch").unwrap(),
        dlms_learning_lab::dhcpv4::Window::Rebinding
    );
    assert_eq!(live(&a).leases()["branch"], held);
    control(&ctx, &mut a, "rebind");
    assert_eq!(live(&a).leases()["branch"].acquired_at, 56);
    assert_eq!(live(&a).leases()["branch"].remaining(56), 64);
}

#[test]
fn dhcp_native_relay_segment_ids_keep_real_switch_edit_and_undo_scopes_independent() {
    let (ctx, mut a) = app();
    let mut fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/dhcpv4-workspace-preview.json"
    ))
    .unwrap();
    let s = &mut fixture["scenarios"][0];
    fn add_relay(state: &mut serde_json::Value) {
        state["topology"]["segments"]["archive"] = serde_json::json!({"subnet":{"network":"172.16.0.0","prefix":24},"server_interface":null,"relay_interface":"172.16.0.1","relay_to_server":true,"client_unicast_to_server":true,"occupied":[]});
        state["topology"]["clients"]["archive-client"] = serde_json::json!({"segment":"archive"});
        state["policy"]["relays"]["archive"] =
            serde_json::json!({"enabled":true,"target":"192.0.2.77","giaddr":"172.16.0.1"});
    }
    add_relay(&mut s["initial"]["simulation"]);
    add_relay(&mut s["goal"]["initial"]);
    let scenario = pack::load(&fixture.to_string()).unwrap().remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(scenario);
    a.launch(i);
    a.set_dhcpv4_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).selected_client, "branch");
    let before = live(&a).clone();
    field(&ctx, &mut a, "relay-target", "192.0.2.88");
    let remote_id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(key, _, _)| key == "relay-target")
        .unwrap()
        .2;
    fn choose(ctx: &egui::Context, a: &mut LabApp, client: &str) {
        let r = locate(ctx, a, |m| m.dhcp_client);
        click(ctx, a, r);
        draw(ctx, a);
        let r = a
            .metrics
            .dhcp_clients
            .iter()
            .find(|(key, _)| key == client)
            .unwrap()
            .1;
        assert!(ctx.content_rect().contains(r.center()));
        click(ctx, a, r);
        draw(ctx, a);
    }
    choose(&ctx, &mut a, "archive-client");
    let archive_id = a
        .metrics
        .dhcp_fields
        .iter()
        .find(|(key, _, _)| key == "relay-target")
        .unwrap()
        .2;
    assert_ne!(remote_id, archive_id);
    let r = locate(&ctx, &mut a, |m| {
        m.dhcp_fields
            .iter()
            .find(|(key, _, _)| key == "relay-target")
            .map(|(_, r, _)| *r)
    });
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_eq!(
        draft(&a).relays["archive"].target,
        "192.0.2.77",
        "new record must not inherit the first relay's Undo stack"
    );
    field(&ctx, &mut a, "relay-target", "192.0.2.66");
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).relays["archive"].target, "192.0.2.77");
    choose(&ctx, &mut a, "branch");
    assert_eq!(
        a.metrics
            .dhcp_fields
            .iter()
            .find(|(key, _, _)| key == "relay-target")
            .unwrap()
            .2,
        remote_id
    );
    assert_eq!(draft(&a).relays["remote"].target, "192.0.2.88");
    let r = locate(&ctx, &mut a, |m| {
        m.dhcp_fields
            .iter()
            .find(|(key, _, _)| key == "relay-target")
            .map(|(_, r, _)| *r)
    });
    click(&ctx, &mut a, r);
    // egui checkpoints the cursor as well as the text. Revisiting this field
    // can put a cursor-only snapshot above the edit; exercise real Undo events
    // through that snapshot, while checking the other relay on every frame.
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(draft(&a).relays["archive"].target, "192.0.2.77");
        if draft(&a).relays["remote"].target == "192.0.2.99" {
            break;
        }
    }
    assert_eq!(draft(&a).relays["remote"].target, "192.0.2.99");
    assert_eq!(
        live(&a),
        &before,
        "editing and Undo must not apply either relay"
    );
}
