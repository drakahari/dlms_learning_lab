//! Unbundled DHCP backend acceptance. The technical fixture is not curriculum.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    backup,
    dhcpv4::{self, Outcome, State as Simulation, Subnet},
    dhcpv4_teaching::{self, Learning},
    dhcpv4_workspace::{self as ws, Action, ClientConfiguration, Draft, Probe, Requirement},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    net::Ipv4Addr,
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn fixture() -> Scenario {
    pack::load(include_str!("fixtures/dhcpv4-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn all() -> Vec<Scenario> {
    let mut v = pack::bundled().unwrap();
    v.push(fixture());
    v
}
fn initial(s: &Scenario) -> &Simulation {
    let State::DhcpV4 { simulation } = &s.initial else {
        panic!()
    };
    simulation
}
fn parts(s: &Scenario) -> (&Simulation, &[Requirement], &[Probe]) {
    let Goal::DhcpV4 {
        initial,
        requirements,
        probes,
    } = &s.goal
    else {
        panic!()
    };
    (initial, requirements, probes)
}
fn simulation(s: &State) -> &Simulation {
    let State::DhcpV4 { simulation } = s else {
        panic!()
    };
    simulation
}
fn repaired(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for a in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, a).unwrap();
    }
    state
}
fn corrected(s: &Scenario) -> dhcpv4::Policy {
    let a: Action = serde_json::from_str(s.solution[0].strip_prefix("dhcpv4 ").unwrap()).unwrap();
    let Action::ApplyPolicy { policy } = a else {
        panic!()
    };
    policy
}
fn acquire(s: &mut Simulation, client: &str) {
    let Outcome::Offered(o) = s.discover(client).unwrap() else {
        panic!()
    };
    assert!(matches!(
        s.request(client, o.token).unwrap(),
        Outcome::Ack { .. }
    ));
}
fn check(s: &Scenario, state: &Simulation) -> Vec<(bool, String)> {
    let (i, r, p) = parts(s);
    ws::checks(i, state, r, p)
}
fn profile(s: &Scenario) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    dhcpv4_teaching::prepare(&mut a, s.dhcpv4_teaching.as_ref().unwrap(), true).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p
}
#[test]
fn shared_current_results_recheck_state_draft_and_authored_context_without_cached_credit() {
    let s = fixture();
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    let results = |a: &Attempt, s: &Scenario| {
        let rows = progress::current_dhcpv4_checks(a, s);
        assert!(!rows.is_empty());
        assert_eq!(rows.iter().all(|r| r.0), progress::current_solved(a, s));
        rows
    };
    assert!(results(a, &s).iter().any(|r| !r.0));
    a.state = repaired(&s);
    a.workspace.as_mut().unwrap().dhcpv4_draft = Some(Draft::new(simulation(&a.state)).unwrap());
    assert!(results(a, &s).iter().all(|r| r.0));
    let good = a.clone();
    a.workspace.as_mut().unwrap().dhcpv4_draft = None;
    assert!(results(a, &s).iter().all(|r| r.0));
    *a = good.clone();
    let d = a.workspace.as_mut().unwrap().dhcpv4_draft.as_mut().unwrap();
    d.pools.get_mut("remote").unwrap().router = "10.20.0.2".into();
    let rows = results(a, &s);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].1.contains("Apply the pending"));
    a.workspace
        .as_mut()
        .unwrap()
        .dhcpv4_draft
        .as_mut()
        .unwrap()
        .pools
        .get_mut("remote")
        .unwrap()
        .router = "not-an-address".into();
    let rows = results(a, &s);
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].0);
    assert!(rows[0].1.contains("Apply the pending"));
    *a = good.clone();
    let State::DhcpV4 { simulation: live } = &mut a.state else {
        panic!()
    };
    live.release("branch").unwrap();
    assert!(results(a, &s).iter().any(|r| !r.0));
    *a = good.clone();
    a.workspace
        .as_mut()
        .unwrap()
        .dhcpv4_learning
        .as_mut()
        .unwrap()
        .context = simulation(&good.state).clone();
    assert!(!results(a, &s)[0].0);
    let other = pack::bundled().unwrap().remove(0);
    let mut foreign = s.clone();
    foreign.initial = other.initial.clone();
    assert!(!results(&good, &foreign)[0].0);
    let wrong = Attempt::new(&other, 1);
    assert!(!progress::current_dhcpv4_checks(&wrong, &s)[0].0);
}
#[test]
fn private_fixture_reaches_current_and_fresh_goals_without_bundling_or_host_commands() {
    let s = fixture();
    let (i, r, p) = parts(&s);
    assert!(ws::valid_goal(i, r, p));
    assert!(!engine::solved(&s.initial, &s.goal));
    let good = repaired(&s);
    assert!(engine::solved(&good, &s.goal));
    assert!(!pack::bundled().unwrap().iter().any(|old| old.id == s.id));
    assert!(engine::execute(&mut s.initial.clone(), &s.solution[0]).is_err());
    assert!(engine::help(&good).contains("not routed connectivity"));
    let mut alternate = s.initial.clone();
    let State::DhcpV4 { simulation: sim } = &mut alternate else {
        panic!()
    };
    let mut policy = corrected(&s);
    policy.pools.get_mut("remote").unwrap().first = "10.20.0.11".parse().unwrap();
    policy.pools.get_mut("remote").unwrap().lease_seconds = 96;
    sim.replace_policy(policy).unwrap();
    acquire(sim, "visitor");
    acquire(sim, "branch");
    assert!(engine::solved(&alternate, &s.goal));
    assert_ne!(
        simulation(&good).leases()["branch"].address,
        simulation(&alternate).leases()["branch"].address
    );
}
#[test]
fn offers_policy_edits_and_stale_held_options_cannot_earn_current_credit() {
    let s = fixture();
    let i = initial(&s);
    let mut live = i.clone();
    let mut wrong = i.policy().clone();
    wrong.relays.get_mut("remote").unwrap().target = i.topology().server;
    live.replace_policy(wrong).unwrap();
    let Outcome::Offered(offer) = live.discover("branch").unwrap() else {
        panic!()
    };
    assert!(!check(&s, &live)[0].0);
    assert!(!live.leases().contains_key("branch"));
    live.replace_policy(corrected(&s)).unwrap();
    live.request("branch", offer.token).unwrap();
    let rows = check(&s, &live);
    assert!(!rows[0].0);
    assert!(rows[5..].iter().all(|r| r.0));
    let before = live.leases()["branch"].clone();
    ws::apply(
        i,
        &mut live,
        &Action::Renew {
            client: "branch".into(),
        },
    )
    .unwrap();
    assert_ne!(live.leases()["branch"].parameters, before.parameters);
    assert!(check(&s, &live).iter().all(|r| r.0));
}
#[test]
fn current_lease_ack_is_not_a_renewed_grant_or_fresh_policy_proof() {
    let s = fixture();
    let state = repaired(&s);
    let mut live = simulation(&state).clone();
    let before = live.leases()["branch"].clone();
    let mut p = live.policy().clone();
    p.pools.remove("remote");
    live.replace_policy(p).unwrap();
    let text = ws::apply(
        initial(&s),
        &mut live,
        &Action::Renew {
            client: "branch".into(),
        },
    )
    .unwrap();
    assert!(text.contains("held options and expiry unchanged"));
    assert_eq!(live.leases()["branch"], before);
    let rows = check(&s, &live);
    assert!(rows[..5].iter().all(|r| r.0));
    assert!(rows[5..].iter().any(|r| !r.0));
}
#[test]
fn wrong_mask_and_wrong_peer_policy_remain_editable_but_fail_their_current_rows() {
    let s = fixture();
    let mut live = initial(&s).clone();
    let mut p = corrected(&s);
    p.pools.get_mut("remote").unwrap().subnet.prefix = 16;
    live.replace_policy(p).unwrap();
    acquire(&mut live, "branch");
    assert!(live.validate().is_ok());
    assert!(!check(&s, &live)[0].0);
    let state = repaired(&s);
    let mut live = simulation(&state).clone();
    let mut p = live.policy().clone();
    p.pools.get_mut("desk").unwrap().lease_seconds = 96;
    live.replace_policy(p).unwrap();
    assert!(!check(&s, &live)[2].0);
    let mut live = simulation(&state).clone();
    live.release("peer").unwrap();
    assert!(!check(&s, &live)[1].0);
}
#[test]
fn fresh_contexts_reject_exhaustion_even_when_all_current_rows_still_pass() {
    let s = fixture();
    let good = repaired(&s);
    let mut live = simulation(&good).clone();
    let mut p = live.policy().clone();
    p.pools.get_mut("remote").unwrap().excluded = ["10.20.0.10", "10.20.0.11", "10.20.0.12"]
        .map(|s| s.parse().unwrap())
        .into();
    live.replace_policy(p).unwrap();
    let before = live.clone();
    let rows = check(&s, &live);
    assert!(rows[..5].iter().all(|r| r.0));
    assert!(rows[5..].iter().all(|r| !r.0));
    assert_eq!(
        live, before,
        "fresh probes cannot mutate live leases or policy"
    );
}
#[test]
fn exact_lease_expiry_and_offer_expiry_are_current_not_cached_success() {
    let s = fixture();
    let good = repaired(&s);
    let mut live = simulation(&good).clone();
    live.advance(63).unwrap();
    assert!(check(&s, &live)[0].0);
    live.advance(1).unwrap();
    assert!(!check(&s, &live)[0].0);
    assert!(!live.leases().contains_key("branch"));
    assert!(check(&s, &live)[5..].iter().all(|r| r.0));
    let mut live = initial(&s).clone();
    live.replace_policy(corrected(&s)).unwrap();
    let Outcome::Offered(o) = live.discover("branch").unwrap() else {
        panic!()
    };
    live.advance(dhcpv4::OFFER_HOLD).unwrap();
    assert!(matches!(
        live.request("branch", o.token).unwrap(),
        Outcome::NoReply(dhcpv4::Reason::NoOffer)
    ));
    assert!(!check(&s, &live)[0].0);
}
#[test]
fn invalid_text_and_pending_valid_drafts_are_saveable_and_apply_atomically() {
    let s = fixture();
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = repaired(&s);
    let State::DhcpV4 { simulation } = &mut a.state else {
        panic!()
    };
    let mut d = Draft::new(simulation).unwrap();
    d.pools.get_mut("remote").unwrap().lease_seconds = "not a duration".into();
    assert!(d.valid());
    assert!(!d.matches(simulation));
    let before = simulation.clone();
    assert!(d.apply(simulation).is_err());
    assert_eq!(*simulation, before);
    a.workspace.as_mut().unwrap().dhcpv4_draft = Some(d);
    assert!(!progress::current_solved(a, &s));
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &all()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let a = loaded.attempts.get_mut(&s.id).unwrap();
    a.workspace
        .as_mut()
        .unwrap()
        .dhcpv4_draft
        .as_mut()
        .unwrap()
        .pools
        .get_mut("remote")
        .unwrap()
        .lease_seconds = "96".into();
    assert!(!progress::current_solved(a, &s));
    let d = a
        .workspace
        .as_ref()
        .unwrap()
        .dhcpv4_draft
        .as_ref()
        .unwrap()
        .clone();
    let State::DhcpV4 { simulation } = &mut a.state else {
        panic!()
    };
    d.apply(simulation).unwrap();
    assert!(progress::current_solved(a, &s));
}
#[test]
fn draft_clock_text_is_independent_of_policy_and_cannot_auto_advance() {
    let s = fixture();
    let good = repaired(&s);
    let live = simulation(&good);
    let mut d = Draft::new(live).unwrap();
    d.advance_seconds = "999999".into();
    assert!(d.valid());
    assert!(d.matches(live));
    assert!(d.seconds().is_err());
    assert_eq!(live.now(), 0);
    d.advance_seconds = "64".into();
    assert_eq!(d.seconds().unwrap(), 64);
}
#[test]
fn immutable_topology_and_foreign_draft_imports_reject_with_exact_original_bytes() {
    let s = fixture();
    let good = repaired(&s);
    let mut raw = serde_json::to_value(simulation(&good)).unwrap();
    raw["topology"]["clients"]["branch"]["segment"] = "desk".into();
    assert!(serde_json::from_value::<Simulation>(raw.clone()).is_err());
    raw = serde_json::to_value(initial(&s)).unwrap();
    raw["topology"]["server"] = "192.0.2.88".into();
    let foreign: Simulation = serde_json::from_value(raw).unwrap();
    let mut target = State::DhcpV4 {
        simulation: foreign.clone(),
    };
    assert!(engine::execute_for_goal(&mut target, &s.goal, &s.solution[1]).is_err());
    assert_eq!(simulation(&target), &foreign);
    let mut p = profile(&s);
    p.attempts.get_mut(&s.id).unwrap().state = target;
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &all()).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let valid = profile(&s);
    progress::save(&path, &valid).unwrap();
    let prior = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), prior);
}
#[test]
fn practice_fill_is_draft_only_and_actual_actions_experiment_repair_stay_separate() {
    let s = fixture();
    let lesson = s.dhcpv4_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let untouched = l.simulation.clone();
    l.fill(lesson).unwrap();
    assert_eq!(l.simulation, untouched);
    assert!(!l.draft.matches(&l.simulation));
    assert!(!l.checked);
    l.apply_draft(lesson).unwrap();
    for a in &lesson.example_actions[1..] {
        l.execute(lesson, &a.action).unwrap();
    }
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!(initial(&s), parts(&s).0);
    for a in &lesson.try_change.actions {
        l.execute(lesson, a).unwrap();
    }
    assert_eq!(
        l.checks(lesson).iter().map(|c| c.0).collect::<Vec<_>>(),
        lesson.try_change.expected
    );
    for a in &lesson.repair {
        l.execute(lesson, a).unwrap();
    }
    assert!(l.checks(lesson).iter().all(|c| c.0));
    l.draft
        .pools
        .get_mut("museum-remote")
        .unwrap()
        .lease_seconds = "invalid".into();
    let raw = l.draft.clone();
    l.execute(lesson, &Action::Advance { seconds: 0 }).unwrap();
    assert_eq!(l.draft, raw);
    assert!(!l.checks(lesson)[0].0);
}
#[test]
fn exact_authored_practice_copy_is_pinned_but_evolving_policy_names_may_overlap() {
    let s = fixture();
    let lesson = s.dhcpv4_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let mut other = lesson.clone();
    let mut p = other.fixture.policy().clone();
    p.server_enabled = false;
    other.fixture.replace_policy(p).unwrap();
    assert!(!l.valid_for(&other));
    let mut policy = corrected(&s);
    let sample = l.simulation.policy().pools.values().next().unwrap().clone();
    policy.pools.insert("museum-extra".into(), sample);
    assert!(
        policy.validate(initial(&s).topology()).is_err(),
        "overlapping pool remains intrinsically rejected, not a namespace policy"
    );
    let mut raw = serde_json::to_value(&l).unwrap();
    raw["context"]["policy"]["server_enabled"] = false.into();
    let changed: Learning = serde_json::from_value(raw).unwrap();
    assert!(!changed.valid_for(lesson));
    let mut p = l.simulation.policy().clone();
    let remote = p.pools.remove("museum-remote").unwrap();
    p.pools.insert("remote".into(), remote);
    l.execute(lesson, &Action::ApplyPolicy { policy: p })
        .unwrap();
    assert!(
        l.valid_for(lesson),
        "mutable names may coincide with the separate assignment"
    );
}
#[test]
fn goal_validation_rejects_duplicates_empty_positives_and_probe_identity_overflow() {
    let s = fixture();
    let (i, r, p) = parts(&s);
    let mut dup = r.to_vec();
    dup.push(r[0].clone());
    assert!(!ws::valid_goal(i, &dup, p));
    assert!(!ws::valid_goal(
        i,
        &[Requirement::NoClientLease {
            client: "branch".into()
        }],
        p
    ));
    assert!(!ws::valid_goal(i, r, &[]));
    let mut probes = p.to_vec();
    let repeated = probes[0].clients[0].clone();
    probes[0].clients.push(repeated);
    assert!(!ws::valid_goal(i, r, &probes));
    let probes = vec![p[0].clone(); 5];
    assert!(!ws::valid_goal(i, r, &probes));
    assert!(!ws::valid_goal(i, r, &[p[0].clone(), p[0].clone()]));
    let mut malformed = serde_json::to_value(&s).unwrap();
    let repeated = malformed["goal"]["requirements"][0].clone();
    malformed["goal"]["requirements"]
        .as_array_mut()
        .unwrap()
        .push(repeated);
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[malformed]}).to_string())
            .is_err()
    );
}
#[test]
fn readers_reject_oversized_fields_unknown_variants_duplicates_and_excess_items() {
    let s = fixture();
    let d = Draft::new(initial(&s)).unwrap();
    let mut value = serde_json::to_value(&d).unwrap();
    value["advance_seconds"] = "x".repeat(65).into();
    assert!(serde_json::from_value::<Draft>(value).is_err());
    let json = serde_json::to_string(&d).unwrap();
    let duplicate = json.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(serde_json::from_str::<Draft>(&duplicate).is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    value["pools"]["remote"]["excluded"] =
        serde_json::json!((0..257).map(|_| "x").collect::<Vec<_>>());
    assert!(serde_json::from_value::<Draft>(value).is_err());
    let long = "é".repeat(10000);
    let err = ws::execute(
        initial(&s),
        &mut initial(&s).clone(),
        &format!("dhcpv4 {{\"kind\":\"{long}\"}}"),
    )
    .unwrap_err();
    assert!(err.len() <= 512);
    assert!(!err.contains(&long));
    let n = ws::notice("é".repeat(1000));
    assert!(n.len() <= 512);
    assert_eq!(n.capacity(), n.len());
}

fn maximal_state() -> Simulation {
    let mut segments = BTreeMap::new();
    let mut clients = BTreeMap::new();
    let mut pools = BTreeMap::new();
    for n in 1u8..=8 {
        let id = format!("segment-{n}-{}", "s".repeat(38));
        let subnet = Subnet {
            network: Ipv4Addr::new(10, n, 0, 0),
            prefix: 16,
        };
        segments.insert(
            id.clone(),
            dhcpv4::Segment {
                subnet,
                server_interface: Some(Ipv4Addr::new(10, n, 0, 1)),
                relay_interface: Some(Ipv4Addr::new(10, n, 0, 2)),
                relay_to_server: true,
                client_unicast_to_server: true,
                occupied: (1..=16).map(|x| Ipv4Addr::new(10, n, 4, x)).collect(),
            },
        );
        let mut reservations = BTreeMap::new();
        for c in 0..4 {
            let key = format!("client-{n}-{c}-{}", "c".repeat(37));
            clients.insert(
                key.clone(),
                dhcpv4::Client {
                    segment: id.clone(),
                },
            );
            reservations.insert(key, Ipv4Addr::new(10, n, 3, 10 + c));
        }
        pools.insert(
            id,
            dhcpv4::Pool {
                subnet,
                first: Ipv4Addr::new(10, n, 0, 10),
                last: Ipv4Addr::new(10, n, 1, 9),
                excluded: (0..=255).map(|x| Ipv4Addr::new(10, n, 2, x)).collect(),
                reservations,
                router: Some(Ipv4Addr::new(10, n, 0, 254)),
                dns: (1..=4).map(|x| Ipv4Addr::new(203, 0, 113, x)).collect(),
                lease_seconds: 86400,
            },
        );
    }
    let t = dhcpv4::Topology {
        server: "198.51.100.254".parse().unwrap(),
        segments,
        clients,
    };
    let p = dhcpv4::Policy {
        server_enabled: true,
        pools,
        relays: BTreeMap::new(),
    };
    let mut s = Simulation::new(t, p).unwrap();
    let ids: Vec<_> = s.topology().clients.keys().cloned().collect();
    for c in ids {
        acquire(&mut s, &c);
    }
    s
}
fn large_raw_draft(s: &Simulation, excluded_bytes: usize) -> Draft {
    let mut d = Draft::new(s).unwrap();
    let escaped = "\\\"".repeat(32);
    assert_eq!(escaped.len(), 64);
    for p in d.pools.values_mut() {
        p.network = escaped.clone();
        p.prefix = escaped.clone();
        p.first = escaped.clone();
        p.last = escaped.clone();
        p.router = escaped.clone();
        p.lease_seconds = escaped.clone();
        p.dns = vec![escaped.clone(); 4];
        p.excluded = vec![escaped[..excluded_bytes].into(); 256];
        p.reservations = (0..32)
            .map(|_| ws::ReservationDraft {
                client: escaped.clone(),
                address: escaped.clone(),
            })
            .collect();
    }
    d.advance_seconds = escaped;
    d
}
#[test]
fn maximum_core_shape_and_large_escaped_admitted_drafts_roundtrip_compact_and_pretty() {
    let s = maximal_state();
    assert_eq!(s.bindings().len(), 32);
    assert_eq!(s.topology().segments.len(), 8);
    for bytes in [
        serde_json::to_vec(&s).unwrap(),
        serde_json::to_vec_pretty(&s).unwrap(),
    ] {
        assert_eq!(Simulation::decode(&bytes).unwrap(), s);
    }
    let d = large_raw_draft(&s, 32);
    assert!(d.valid());
    assert!(d.snapshot().is_err());
    assert!(serde_json::to_vec(&d).unwrap().len() <= ws::MAX_DRAFT_BYTES);
    for bytes in [
        serde_json::to_vec(&d).unwrap(),
        serde_json::to_vec_pretty(&d).unwrap(),
    ] {
        assert_eq!(serde_json::from_slice::<Draft>(&bytes).unwrap(), d);
    }
    let oversized = large_raw_draft(&s, 64);
    assert!(
        !oversized.valid(),
        "cartesian field maxima honestly exceed aggregate admission; never truncate"
    );
    let json = serde_json::to_vec(&oversized).unwrap();
    assert!(json.len() > ws::MAX_DRAFT_BYTES);
    assert!(serde_json::from_slice::<Draft>(&json).is_err());
}
#[test]
fn maximum_saved_practice_copy_is_bounded_and_exact_before_authored_context_pinning() {
    let s = maximal_state();
    let l = Learning {
        version: 1,
        context: s.clone(),
        simulation: s.clone(),
        draft: large_raw_draft(&s, 32),
        view: View::Example,
        selected_action: 15,
        checked: true,
        notice: Some("n".repeat(512)),
    };
    assert!(l.valid());
    assert!(serde_json::to_vec(&l).unwrap().len() <= dhcpv4_teaching::MAX_LEARNING_BYTES);
    for bytes in [
        serde_json::to_vec(&l).unwrap(),
        serde_json::to_vec_pretty(&l).unwrap(),
    ] {
        assert_eq!(serde_json::from_slice::<Learning>(&bytes).unwrap(), l);
    }
}
#[test]
fn bounded_four_contexts_exercise_32_clients_and_full_256_address_scans_without_mutation() {
    let mut s = maximal_state();
    let mut policy = s.policy().clone();
    for p in policy.pools.values_mut() {
        p.reservations.clear();
        let first = u32::from(p.first);
        p.excluded = (0..252).map(|n| Ipv4Addr::from(first + n)).collect();
    }
    s.replace_policy(policy).unwrap();
    let t = s.topology();
    let expected: Vec<_> = t
        .clients
        .iter()
        .map(|(id, c)| {
            let subnet = t.segments[&c.segment].subnet;
            ClientConfiguration {
                client: id.clone(),
                subnet,
                router: Some(Ipv4Addr::from(u32::from(subnet.network) + 254)),
                dns: (1..=4).map(|x| Ipv4Addr::new(203, 0, 113, x)).collect(),
                minimum_remaining: 1,
                address: None,
            }
        })
        .collect();
    let r = vec![Requirement::ClientConfiguration {
        expected: expected[0].clone(),
    }];
    let probes: Vec<_> = expected
        .chunks(8)
        .map(|c| Probe {
            clients: c.to_vec(),
        })
        .collect();
    assert_eq!(probes.len(), 4);
    assert!(ws::valid_goal(&s, &r, &probes));
    // One ordinary maximum supported fixture, not a stress loop. Record the
    // clone+grade cost before deciding whether the native frame can run it.
    let started = std::time::Instant::now();
    let before = s.clone();
    let checks = ws::checks(&s, &s, &r, &probes);
    let elapsed = started.elapsed();
    eprintln!(
        "DHCP_MAX_PROBE_OBSERVATION clone_and_check_ms={:.3} contexts=4 total_clients=32 concurrent_per_context=8 core_json_bytes={}",
        elapsed.as_secs_f64() * 1000.,
        serde_json::to_vec(&s).unwrap().len()
    );
    assert_eq!(checks.len(), 33);
    assert!(checks.iter().all(|r| r.0));
    assert_eq!(s, before);
}
#[test]
fn save_resume_reset_and_explicit_backup_restore_preserve_both_raw_drafts_and_peers() {
    let s = fixture();
    let all = all();
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = repaired(&s);
    a.workspace.as_mut().unwrap().dhcpv4_draft = Some(Draft::new(simulation(&a.state)).unwrap());
    let w = a.workspace.as_mut().unwrap();
    w.dhcpv4_draft
        .as_mut()
        .unwrap()
        .pools
        .get_mut("remote")
        .unwrap()
        .lease_seconds = "unfinished".into();
    let l = w.dhcpv4_learning.as_mut().unwrap();
    l.view = View::Example;
    l.draft.pools.get_mut("museum-remote").unwrap().router = "pending".into();
    let old = &all[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 2));
    p.favorites.insert(old.id.clone());
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let raw_export = std::fs::read(&export).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 0);
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        serde_json::to_value(&loaded.attempts).unwrap()
    );
    let prior = std::fs::read(&path).unwrap();
    let restored = backup::restore(&path, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), prior);
    assert_eq!(std::fs::read(&export).unwrap(), raw_export);
    let mut reopened = progress::load(&path, &all).unwrap();
    let old_before = serde_json::to_value(&reopened.attempts[&old.id]).unwrap();
    let favorites = reopened.favorites.clone();
    reopened.attempts.insert(s.id.clone(), Attempt::new(&s, 9));
    assert_eq!(
        serde_json::to_value(&reopened.attempts[&old.id]).unwrap(),
        old_before
    );
    assert_eq!(reopened.favorites, favorites);
    assert_eq!(reopened.attempts[&s.id].state, s.initial);
}
#[test]
fn wrong_kind_and_foreign_authored_practice_reject_before_replacing_saved_bytes() {
    let s = fixture();
    let all = all();
    let mut p = profile(&s);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let prior = std::fs::read(&path).unwrap();
    let mut a = p.attempts[&s.id].clone();
    let other = all[0].clone();
    a.state = other.initial.clone();
    p.attempts.insert(other.id.clone(), a);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), prior);
    let mut p = profile(&s);
    let l = p
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dhcpv4_learning
        .as_mut()
        .unwrap();
    let mut policy = l.context.policy().clone();
    policy.server_enabled = false;
    l.context.replace_policy(policy).unwrap();
    assert!(l.valid());
    let bytes = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&bytes, &all).is_err());
}
#[derive(Deserialize)]
struct Original {
    id: String,
    version: u32,
    unit: String,
    initial: State,
    goal: Goal,
    solution: Vec<String>,
}
#[test]
fn original_119_specs_and_genuine_6_7_8_history_remain_exact_without_reruns() {
    let baseline: Vec<Original> =
        serde_json::from_str(include_str!("fixtures/dhcpv4-original119.json")).unwrap();
    assert_eq!(baseline.len(), 119);
    let current = pack::bundled().unwrap();
    for b in baseline {
        let s = current.iter().find(|s| s.id == b.id).unwrap();
        assert_eq!(s.version, b.version);
        assert_eq!(s.unit, b.unit);
        assert_eq!(s.initial, b.initial);
        assert_eq!(
            serde_json::to_value(&s.goal).unwrap(),
            serde_json::to_value(&b.goal).unwrap()
        );
        assert_eq!(s.solution, b.solution);
    }
    let bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let raw: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let p = progress::decode(bytes, &all()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap()["attempts"],
        raw["attempts"]
    );
    assert!(!p.attempts.contains_key(&fixture().id));
    let mut counts = BTreeMap::new();
    for a in p.attempts.values() {
        if let State::Script { run: Some(r), .. } = &a.state {
            *counts.entry(r.engine_revision).or_insert(0usize) += 1;
        }
        if let Some(w) = &a.workspace {
            for r in w.script_history.iter().chain(&w.script_history_more) {
                *counts.entry(r.engine_revision).or_insert(0) += 1;
            }
        }
    }
    assert_eq!(counts, BTreeMap::from([(6, 33), (7, 37), (8, 40)]));
}
#[test]
fn lost_release_removes_client_use_without_inventing_server_notification() {
    let s = fixture();
    let mut raw = serde_json::to_value(initial(&s)).unwrap();
    raw["topology"]["segments"]["remote"]["client_unicast_to_server"] = false.into();
    let context: Simulation = serde_json::from_value(raw).unwrap();
    let mut live = context.clone();
    live.replace_policy(corrected(&s)).unwrap();
    acquire(&mut live, "branch");
    let binding = live.bindings()["branch"].clone();
    let text = ws::apply(
        &context,
        &mut live,
        &Action::Release {
            client: "branch".into(),
        },
    )
    .unwrap();
    assert!(text.contains("binding retained"));
    assert!(!live.leases().contains_key("branch"));
    assert_eq!(live.bindings()["branch"], binding);
    let (_, r, p) = parts(&s);
    let req = vec![
        r[1].clone(),
        Requirement::NoClientLease {
            client: "branch".into(),
        },
    ];
    assert!(ws::checks(&context, &live, &req, p).iter().all(|c| c.0));
}
#[test]
fn timer_actions_reject_early_rebinding_atomically_and_do_not_send_on_advance() {
    let s = fixture();
    let good = repaired(&s);
    let mut live = simulation(&good).clone();
    let before = live.clone();
    assert!(
        ws::apply(
            initial(&s),
            &mut live,
            &Action::Rebind {
                client: "branch".into()
            }
        )
        .is_err()
    );
    assert_eq!(live, before);
    ws::apply(initial(&s), &mut live, &Action::Advance { seconds: 32 }).unwrap();
    assert_eq!(live.window("branch").unwrap(), dhcpv4::Window::Renewal);
    assert_eq!(live.leases()["branch"], before.leases()["branch"]);
    ws::apply(initial(&s), &mut live, &Action::Advance { seconds: 24 }).unwrap();
    assert_eq!(live.window("branch").unwrap(), dhcpv4::Window::Rebinding);
    assert_eq!(live.leases()["branch"], before.leases()["branch"]);
    ws::apply(
        initial(&s),
        &mut live,
        &Action::Rebind {
            client: "branch".into(),
        },
    )
    .unwrap();
    assert_eq!(live.leases()["branch"].acquired_at, 56);
    let before = live.clone();
    assert!(
        ws::apply(
            initial(&s),
            &mut live,
            &Action::Advance {
                seconds: dhcpv4::MAX_ADVANCE + 1
            }
        )
        .is_err()
    );
    assert_eq!(live, before);
}
#[test]
fn probe_field_count_rejects_a_ninth_item_before_constructing_its_typed_body() {
    let s = fixture();
    let (_, _, probes) = parts(&s);
    let first = serde_json::to_value(&probes[0].clients[0]).unwrap();
    let mut items = vec![first; 8];
    items.push(serde_json::json!({"not_a_client_configuration":"x".repeat(1024)}));
    let error = serde_json::from_value::<Probe>(serde_json::json!({"clients":items}))
        .unwrap_err()
        .to_string();
    assert!(error.contains("count limit"));
    assert!(!error.contains("missing field"));
}
#[test]
fn maximum_admitted_assignment_and_practice_profile_roundtrip_with_actual_legacy_histories() {
    let mut assignment = maximal_state();
    let target = assignment.topology().clients.keys().next().unwrap().clone();
    assignment.release(&target).unwrap();
    let mut raw = serde_json::to_value(&assignment).unwrap();
    fn rename(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                *s = s
                    .replace("segment-", "gallery-")
                    .replace("client-", "museum-")
            }
            serde_json::Value::Array(v) => v.iter_mut().for_each(rename),
            serde_json::Value::Object(m) => {
                let old = std::mem::take(m);
                for (k, mut v) in old {
                    rename(&mut v);
                    m.insert(
                        k.replace("segment-", "gallery-")
                            .replace("client-", "museum-"),
                        v,
                    );
                }
            }
            _ => {}
        }
    }
    rename(&mut raw);
    raw["topology"]["server"] = "203.0.113.254".into();
    let practice: Simulation = serde_json::from_value(raw).unwrap();
    let example_client = target.replace("client-", "museum-");
    fn configured(s: &Simulation, c: &str) -> ClientConfiguration {
        let segment = &s.topology().clients[c].segment;
        let p = s.policy().pools.get(segment).unwrap();
        ClientConfiguration {
            client: c.into(),
            subnet: p.subnet,
            router: p.router,
            dns: p.dns.clone(),
            minimum_remaining: 1,
            address: None,
        }
    }
    let mut scenario = fixture();
    scenario.id = "dhcpv4-max-profile-preview".into();
    let wanted = configured(&assignment, &target);
    let requirements = vec![Requirement::ClientConfiguration {
        expected: wanted.clone(),
    }];
    let probes = vec![Probe {
        clients: vec![wanted],
    }];
    scenario.initial = State::DhcpV4 {
        simulation: assignment.clone(),
    };
    scenario.goal = Goal::DhcpV4 {
        initial: assignment.clone(),
        requirements,
        probes,
    };
    scenario.solution = [
        Action::Discover {
            client: target.clone(),
        },
        Action::Request {
            client: target.clone(),
            token: 33,
        },
    ]
    .iter()
    .map(|a| format!("dhcpv4 {}", serde_json::to_string(a).unwrap()))
    .collect();
    let mut lesson = scenario.dhcpv4_teaching.take().unwrap();
    let wanted = configured(&practice, &example_client);
    lesson.fixture = practice.clone();
    lesson.requirements = vec![Requirement::ClientConfiguration {
        expected: wanted.clone(),
    }];
    lesson.probes = vec![Probe {
        clients: vec![wanted],
    }];
    lesson.example_actions = vec![
        dhcpv4_teaching::ExampleAction {
            action: Action::Discover {
                client: example_client.clone(),
            },
            explanation: "Discover the distinct private client offer.".into(),
        },
        dhcpv4_teaching::ExampleAction {
            action: Action::Request {
                client: example_client.clone(),
                token: 33,
            },
            explanation: "Request its delivered offer.".into(),
        },
    ];
    lesson.try_change.actions = vec![Action::Release {
        client: example_client.clone(),
    }];
    lesson.try_change.expected = vec![true, false, true];
    lesson.repair = vec![
        Action::Discover {
            client: example_client.clone(),
        },
        Action::Request {
            client: example_client,
            token: 34,
        },
    ];
    lesson.validate().unwrap();
    assert!(lesson.separate_from(&assignment));
    scenario.dhcpv4_teaching = Some(lesson);
    // The catalog is a collection of bounded packs, not one growing artificial pack.
    let mut catalog = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&catalog), 176);
    let normal = serde_json::to_value(&catalog).unwrap();
    assert!(scenario.prerequisites.is_empty());
    assert!(catalog.iter().all(|s| s.id != scenario.id));
    let fixture_pack =
        serde_json::json!({"schema_version":1,"scenarios":[scenario.clone()]}).to_string();
    let validated = pack::load(&fixture_pack).unwrap();
    assert_eq!(validated.len(), 1);
    assert_eq!(
        serde_json::to_value(&validated[0]).unwrap(),
        serde_json::to_value(&scenario).unwrap()
    );
    catalog.extend(validated);
    assert_eq!(
        one_off_boundary::loop_registration::lvm_registration::before_scenarios(&catalog[..197])
            .len()
            + 1,
        183
    );
    assert_eq!(
        one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(
            &catalog[..197]
        )
        .len()
            + 1,
        187
    );
    assert_eq!(catalog.len(), 198);
    assert_eq!(serde_json::to_value(&catalog[..197]).unwrap(), normal);
    let oversized = format!("{}{}", fixture_pack, " ".repeat(2 * 1024 * 1024));
    assert!(
        pack::load(&oversized)
            .unwrap_err()
            .contains("Pack exceeds 2 MB")
    );
    let old_bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let mut p = progress::decode(old_bytes, &catalog).unwrap();
    let old_attempts = serde_json::to_value(&p.attempts).unwrap();
    let mut a = Attempt::new(&scenario, 7);
    workspace::prepare(&mut a);
    a.state = repaired(&scenario);
    a.workspace.as_mut().unwrap().dhcpv4_draft = Some(large_raw_draft(simulation(&a.state), 32));
    let lesson = scenario.dhcpv4_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    for a in &lesson.example_actions {
        l.execute(lesson, &a.action).unwrap();
    }
    l.draft = large_raw_draft(&l.simulation, 32);
    l.view = View::Example;
    assert!(l.valid_for(lesson));
    a.workspace.as_mut().unwrap().dhcpv4_learning = Some(l);
    p.attempts.insert(scenario.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let saved = progress::read_bounded(&path, 16_000_000).unwrap();
    let decoded = progress::decode(&saved, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let mut actual = serde_json::to_value(&decoded.attempts).unwrap();
    actual.as_object_mut().unwrap().remove(&scenario.id);
    assert_eq!(actual, old_attempts);
    let pretty = serde_json::to_vec_pretty(&p).unwrap();
    assert!(pretty.len() < 16_000_000);
    assert_eq!(
        serde_json::to_value(&progress::decode(&pretty, &catalog).unwrap().attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
}
#[test]
fn each_explicit_safety_and_grant_preservation_predicate_checks_its_own_current_row() {
    let s = fixture();
    let good = repaired(&s);
    let live = simulation(&good);
    let (i, base, p) = parts(&s);
    let mut requirements = base.to_vec();
    requirements.extend([
        Requirement::BindingPreserved {
            client: "peer".into(),
        },
        Requirement::Excluded {
            pool: "desk".into(),
            address: "192.0.2.11".parse().unwrap(),
        },
        Requirement::Reserved {
            pool: "desk".into(),
            client: "peer".into(),
            address: "192.0.2.12".parse().unwrap(),
        },
    ]);
    assert!(ws::checks(i, live, &requirements, p).iter().all(|r| r.0));
    let mut changed = live.clone();
    let mut policy = changed.policy().clone();
    policy.pools.get_mut("desk").unwrap().excluded.clear();
    changed.replace_policy(policy).unwrap();
    assert!(!ws::checks(i, &changed, &requirements, p)[6].0);
    let mut changed = live.clone();
    let mut policy = changed.policy().clone();
    policy.pools.get_mut("desk").unwrap().reservations.clear();
    changed.replace_policy(policy).unwrap();
    assert!(!ws::checks(i, &changed, &requirements, p)[7].0);
    let mut changed = live.clone();
    changed.advance(1).unwrap();
    changed.renew("peer").unwrap();
    let rows = ws::checks(i, &changed, &requirements, p);
    assert!(rows[1].0);
    assert!(!rows[5].0);
    let mut exact = base.to_vec();
    let Requirement::ClientConfiguration { expected } = &mut exact[0] else {
        panic!()
    };
    expected.address = Some("10.20.0.12".parse().unwrap());
    assert!(!ws::checks(i, live, &exact, p)[0].0);
    let mut remaining = base.to_vec();
    let Requirement::ClientConfiguration { expected } = &mut remaining[0] else {
        panic!()
    };
    expected.minimum_remaining = 64;
    let mut aged = live.clone();
    aged.advance(1).unwrap();
    assert!(!ws::checks(i, &aged, &remaining, p)[0].0);
    assert!(ws::checks(i, &aged, base, p)[0].0);
}
