//! Authored DHCP unit acceptance. No host DHCP/resolver/commands.
use dlms_learning_lab::{
    backup,
    dhcpv4::{self, Outcome, State as Simulation},
    dhcpv4_teaching::{self, Learning},
    dhcpv4_workspace::{self as ws, Action, Draft, Requirement},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize};
const IDS: [&str; 3] = [
    "ccna-dhcp-held-options",
    "ccna-dhcp-relay-scope",
    "ccna-dhcp-lease-handover",
];
fn labs() -> Vec<Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn catalog() -> Vec<Scenario> {
    pack::bundled().unwrap()
}
fn plan(s: &Scenario) -> Value {
    serde_json::from_str::<Vec<Value>>(include_str!("fixtures/dhcpv4-content-review.json"))
        .unwrap()
        .into_iter()
        .find(|v| v["id"] == s.id)
        .unwrap()
}
fn actions(v: &Value) -> Vec<Action> {
    serde_json::from_value(v.clone()).unwrap()
}
fn sim(s: &State) -> &Simulation {
    let State::DhcpV4 { simulation } = s else {
        panic!()
    };
    simulation
}
fn sim_mut(s: &mut State) -> &mut Simulation {
    let State::DhcpV4 { simulation } = s else {
        panic!()
    };
    simulation
}
fn parts(s: &Scenario) -> (&Simulation, &[Requirement], &[ws::Probe]) {
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
fn checks(s: &Scenario, state: &State) -> Vec<(bool, String)> {
    let (i, r, p) = parts(s);
    ws::checks(i, sim(state), r, p)
}
fn run(s: &Scenario, state: &mut State, aa: &[Action]) {
    for a in aa {
        let command = format!("dhcpv4 {}", serde_json::to_string(a).unwrap());
        engine::execute_for_goal(state, &s.goal, &command).unwrap();
    }
}
fn passed(s: &Scenario) -> State {
    let mut st = s.initial.clone();
    run(s, &mut st, &actions(&plan(s)["reference"]));
    assert!(
        engine::solved(&st, &s.goal),
        "{} {:?}",
        s.id,
        checks(s, &st)
    );
    st
}
fn reject_with_feedback(s: &Scenario, state: &State) {
    let c = checks(s, state);
    assert!(c.iter().any(|r| !r.0), "{}", s.id);
    assert!(
        c.iter()
            .filter(|r| !r.0)
            .all(|r| !r.1.trim().is_empty() && r.1.len() <= 512)
    );
}
fn acquire(s: &mut Simulation, c: &str) {
    let Outcome::Offered(o) = s.discover(c).unwrap() else {
        panic!("{c}")
    };
    assert!(matches!(
        s.request(c, o.token).unwrap(),
        Outcome::Ack { .. }
    ));
}
fn edit(s: &mut State, f: impl FnOnce(&mut dhcpv4::Policy)) {
    let mut p = sim(s).policy().clone();
    f(&mut p);
    sim_mut(s).replace_policy(p).unwrap();
}
#[test]
fn registered_unit_has_real_prerequisites_three_distinct_tasks_and_reachable_references() {
    let l = labs();
    assert_eq!(l.len(), 3);
    assert_ne!(l[0].objective, l[1].objective);
    assert_ne!(l[1].objective, l[2].objective);
    for s in l {
        assert!(pack::bundled().unwrap().iter().any(|x| x.id == s.id));
        assert_eq!(s.unit, "dhcpv4-client-foundations");
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(!s.prerequisites.is_empty());
        let st = passed(&s);
        assert_eq!(
            checks(&s, &st).len(),
            plan(&s)["expected_current_rows"].as_u64().unwrap() as usize
                + plan(&s)["expected_fresh_rows"].as_u64().unwrap() as usize
        );
        for command in &s.solution {
            assert!(command.starts_with("dhcpv4 "));
        }
        let mut authored = s.initial.clone();
        for command in &s.solution {
            engine::execute_for_goal(&mut authored, &s.goal, command).unwrap();
        }
        assert_eq!(authored, st);
    }
}
#[test]
fn two_alternates_accept_different_orders_lease_lengths_and_retirement_without_action_credentials()
{
    for s in labs() {
        let p = plan(&s);
        for alternative in p["alternatives"].as_array().unwrap() {
            let mut st = s.initial.clone();
            run(&s, &mut st, &actions(alternative));
            assert!(
                engine::solved(&st, &s.goal),
                "{} {:?}",
                s.id,
                checks(&s, &st)
            );
        }
    }
}
#[test]
fn read_only_partial_offer_only_and_plainly_wrong_parameters_do_not_earn_success() {
    for s in labs() {
        let p = plan(&s);
        let mut partial = s.initial.clone();
        run(&s, &mut partial, &actions(&p["partial"]));
        reject_with_feedback(&s, &partial);
        let mut readonly = s.initial.clone();
        engine::execute_for_goal(&mut readonly, &s.goal, "show").unwrap();
        assert_eq!(readonly, s.initial);
        reject_with_feedback(&s, &readonly);
        let mut offered = s.initial.clone();
        let good: dhcpv4::Policy = serde_json::from_value(p["desired_policy"].clone()).unwrap();
        run(&s, &mut offered, &[Action::ApplyPolicy { policy: good }]);
        let first = p["first_client"].as_str().unwrap();
        if sim(&offered).leases().contains_key(first) {
            run(
                &s,
                &mut offered,
                &[Action::Release {
                    client: first.into(),
                }],
            );
        }
        run(
            &s,
            &mut offered,
            &[Action::Discover {
                client: first.into(),
            }],
        );
        assert!(sim(&offered).offers().contains_key(first));
        assert!(!sim(&offered).leases().contains_key(first));
        reject_with_feedback(&s, &offered);
        let mut wrong = passed(&s);
        let c = p["first_client"].as_str().unwrap();
        let work = p["work_pool"].as_str().unwrap();
        sim_mut(&mut wrong).release(c).unwrap();
        edit(&mut wrong, |p| {
            p.pools.get_mut(work).unwrap().dns = vec!["203.0.113.200".parse().unwrap()]
        });
        acquire(sim_mut(&mut wrong), c);
        assert!(!checks(&s, &wrong)[0].0);
        reject_with_feedback(&s, &wrong);
    }
}
#[test]
fn direct_apply_early_renew_and_declined_extension_have_distinct_outcomes() {
    let s = labs().remove(0);
    let p = plan(&s);
    let c = p["first_client"].as_str().unwrap();
    let mut st = s.initial.clone();
    let before = sim(&st).leases()[c].clone();
    run(&s, &mut st, &actions(&p["partial"]));
    assert_eq!(sim(&st).now(), 0);
    assert!(sim(&st).now() < before.renew_at);
    assert_eq!(
        sim(&st).leases()[c],
        before,
        "Apply alone does not change held options"
    );
    assert!(!checks(&s, &st)[0].0);
    let exact = serde_json::to_vec(&st).unwrap();
    assert!(
        sim_mut(&mut st).discover(c).is_err(),
        "Discover while holding a lease is rejected, not a current-offer workaround"
    );
    assert_eq!(serde_json::to_vec(&st).unwrap(), exact);
    let outcome = sim_mut(&mut st).renew(c).unwrap();
    assert!(matches!(
        outcome,
        Outcome::Ack {
            kind: dhcpv4::AckKind::Renewed,
            ..
        }
    ));
    assert_eq!(
        sim(&st).now(),
        0,
        "no clock advance or T1 gate was required"
    );
    assert_ne!(sim(&st).leases()[c].parameters, before.parameters);
    assert!(
        checks(&s, &st).iter().all(|r| r.0),
        "eligible early renewal is a valid current and new-client solution"
    );
    assert!(engine::solved(&st, &s.goal));

    let mut declined = s.initial.clone();
    let mut policy: dhcpv4::Policy = serde_json::from_value(p["desired_policy"].clone()).unwrap();
    let work = p["work_pool"].as_str().unwrap();
    let pool = policy.pools.get_mut(work).unwrap();
    assert_eq!(
        pool.first,
        "192.0.2.10".parse::<std::net::Ipv4Addr>().unwrap()
    );
    assert_eq!(
        pool.last,
        "192.0.2.13".parse::<std::net::Ipv4Addr>().unwrap()
    );
    // Narrow this independent test policy to exactly two eligible new addresses
    // after excluding the held address. The authored/reference range stays intact.
    pool.last = "192.0.2.12".parse().unwrap();
    pool.excluded.insert(before.address);
    // .11 and .12 still provide both new clients with correct current policy.
    run(&s, &mut declined, &[Action::ApplyPolicy { policy }]);
    let outcome = sim_mut(&mut declined).renew(c).unwrap();
    let Outcome::Ack {
        binding,
        kind: dhcpv4::AckKind::CurrentLease,
    } = outcome
    else {
        panic!("declined extension retains the old grant")
    };
    assert_eq!(binding, before);
    assert_eq!(sim(&declined).leases()[c], before);
    assert_eq!(sim(&declined).bindings()[c], before);
    let rows = checks(&s, &declined);
    assert!(
        !rows[0].0,
        "old held options are still wrong despite the acknowledgement"
    );
    assert!(
        rows[1..].iter().all(|r| r.0),
        "preservation and all independent new-client checks still pass: {rows:?}"
    );
    assert!(!engine::solved(&declined, &s.goal));
}
#[test]
fn relay_only_repair_exposes_broad_mask_and_correct_pool_without_delivery_is_incomplete() {
    let s = labs().remove(1);
    let p = plan(&s);
    let work = p["work_pool"].as_str().unwrap();
    let c = p["first_client"].as_str().unwrap();
    let mut st = s.initial.clone();
    let target = sim(&st).topology().server;
    edit(&mut st, |p| p.relays.get_mut(work).unwrap().target = target);
    acquire(sim_mut(&mut st), c);
    assert_eq!(sim(&st).leases()[c].parameters.subnet.prefix, 16);
    assert!(!checks(&s, &st)[0].0);
    let mut st = s.initial.clone();
    let desired: dhcpv4::Policy = serde_json::from_value(p["desired_policy"].clone()).unwrap();
    edit(&mut st, |p| p.pools = desired.pools.clone());
    assert!(matches!(
        sim_mut(&mut st).discover(c).unwrap(),
        Outcome::NoReply(_)
    ));
    assert!(!checks(&s, &st)[0].0);
}
#[test]
fn each_required_current_predicate_fails_its_own_row_after_passing() {
    for s in labs() {
        let (_, requirements, _) = parts(&s);
        for (i, r) in requirements.iter().enumerate() {
            let mut st = passed(&s);
            match r {
                Requirement::ClientConfiguration { expected } => {
                    sim_mut(&mut st).release(&expected.client).unwrap();
                }
                Requirement::NoClientLease { client } => {
                    let work = plan(&s)["work_pool"].as_str().unwrap().to_owned();
                    edit(&mut st, |p| {
                        p.pools.get_mut(&work).unwrap().excluded.clear()
                    });
                    acquire(sim_mut(&mut st), client);
                }
                Requirement::PoolPreserved { pool } => edit(&mut st, |p| {
                    p.pools.get_mut(pool).unwrap().lease_seconds += 8
                }),
                Requirement::BindingPreserved { client } => {
                    sim_mut(&mut st).release(client).unwrap();
                }
                Requirement::RelayPreserved { segment } => {
                    let t = &sim(&st).topology().segments[segment];
                    let giaddr = t.server_interface.or(t.relay_interface).unwrap();
                    let target = sim(&st).topology().server;
                    edit(&mut st, |p| {
                        p.relays.insert(
                            segment.clone(),
                            dhcpv4::Relay {
                                enabled: false,
                                target,
                                giaddr,
                            },
                        );
                    });
                }
                Requirement::ServerEnabled { enabled } => {
                    edit(&mut st, |p| p.server_enabled = !*enabled)
                }
                Requirement::Excluded { pool, address } => edit(&mut st, |p| {
                    p.pools.get_mut(pool).unwrap().excluded.remove(address);
                }),
                Requirement::Reserved { pool, client, .. } => edit(&mut st, |p| {
                    p.pools.get_mut(pool).unwrap().reservations.remove(client);
                }),
            }
            let c = checks(&s, &st);
            assert!(!c[i].0, "{} target{i}: {}", s.id, c[i].1);
            assert!(!c[i].1.is_empty());
        }
    }
}
#[test]
fn new_client_checks_fail_independently_while_every_current_row_still_passes() {
    for s in labs() {
        let p = plan(&s);
        let work = p["work_pool"].as_str().unwrap();
        let mut st = passed(&s);
        edit(&mut st, |p| {
            p.pools.get_mut(work).unwrap().dns = vec!["203.0.113.201".parse().unwrap()]
        });
        let n = parts(&s).1.len();
        let rows = checks(&s, &st);
        assert!(rows[..n].iter().all(|r| r.0), "{} {:?}", s.id, rows);
        assert!(rows[n..].iter().any(|r| !r.0));
        assert!(!engine::solved(&st, &s.goal));
        let mut capacity = passed(&s);
        edit(&mut capacity, |p| {
            let pool = p.pools.get_mut(work).unwrap();
            pool.last = pool.first;
        });
        let rows = checks(&s, &capacity);
        assert!(rows[..n].iter().all(|r| r.0), "{} {:?}", s.id, rows);
        assert!(rows[n..].iter().any(|r| !r.0), "{}", s.id);
    }
}
#[test]
fn exact_remaining_time_boundary_and_expiry_are_current_not_cached_credit() {
    for s in labs() {
        let p = plan(&s);
        let c = p["first_client"].as_str().unwrap();
        let mut st = passed(&s);
        let remaining = sim(&st).leases()[c].remaining(sim(&st).now());
        sim_mut(&mut st).advance(remaining - 40).unwrap();
        assert!(checks(&s, &st)[0].0);
        sim_mut(&mut st).advance(1).unwrap();
        assert!(!checks(&s, &st)[0].0);
        sim_mut(&mut st).advance(39).unwrap();
        assert!(!sim(&st).leases().contains_key(c));
        assert!(!checks(&s, &st)[0].0);
    }
}
#[test]
fn separate_examples_fill_only_then_explicit_work_experiment_and_repair() {
    for s in labs() {
        let lesson = s.dhcpv4_teaching.as_ref().unwrap();
        assert!(lesson.separate_from(sim(&s.initial)));
        let mut l = Learning::new(lesson).unwrap();
        let original = l.simulation.clone();
        l.fill(lesson).unwrap();
        assert_eq!(l.simulation, original);
        assert!(!l.draft.matches(&l.simulation));
        assert!(l.checks(lesson).iter().any(|r| !r.0));
        for a in &lesson.example_actions {
            l.execute(lesson, &a.action).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        let assignment = s.initial.clone();
        for a in &lesson.try_change.actions {
            l.execute(lesson, a).unwrap();
        }
        assert_eq!(
            l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
            lesson.try_change.expected
        );
        for a in &lesson.repair {
            l.execute(lesson, a).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        assert_eq!(s.initial, assignment);
    }
}
#[test]
fn pending_invalid_drafts_save_resume_restore_and_selected_reset_preserve_other_work() {
    let labs = labs();
    let s = &labs[0];
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (n, s) in labs.iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        workspace::prepare(&mut a);
        dhcpv4_teaching::prepare(&mut a, s.dhcpv4_teaching.as_ref().unwrap(), true).unwrap();
        a.state = passed(s);
        a.workspace.as_mut().unwrap().dhcpv4_draft = Some(Draft::new(sim(&a.state)).unwrap());
        p.attempts.insert(s.id.clone(), a);
    }
    let before_other = serde_json::to_value(&p.attempts[&labs[1].id]).unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let w = a.workspace.as_mut().unwrap();
    let pool = plan(s)["work_pool"].as_str().unwrap().to_owned();
    w.dhcpv4_draft
        .as_mut()
        .unwrap()
        .pools
        .get_mut(&pool)
        .unwrap()
        .lease_seconds = "unfinished".into();
    let l = w.dhcpv4_learning.as_mut().unwrap();
    l.view = View::Example;
    let key = l.draft.pools.keys().next().unwrap().clone();
    l.draft.pools.get_mut(&key).unwrap().router = "pending".into();
    a.challenge = true;
    p.favorites.insert(s.id.clone());
    assert!(!progress::current_solved(&p.attempts[&s.id], s));
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let export = d.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let source = std::fs::read(&export).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 0);
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        serde_json::to_value(&loaded.attempts).unwrap()
    );
    let old = std::fs::read(&path).unwrap();
    let r = backup::restore(&path, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(r.safety_copy.unwrap()).unwrap(), old);
    assert_eq!(std::fs::read(&export).unwrap(), source);
    let mut reopened = progress::load(&path, &all).unwrap();
    reopened.attempts.insert(s.id.clone(), Attempt::new(s, 9));
    assert_eq!(
        serde_json::to_value(&reopened.attempts[&labs[1].id]).unwrap(),
        before_other
    );
    assert!(reopened.favorites.contains(&s.id));
    assert_eq!(reopened.attempts[&s.id].state, s.initial);
}
#[test]
fn old122_specs_units_and_actual6_7_8_records_remain_exact_without_rerunning() {
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/dhcpv4-original122.json")).unwrap();
    let original = baseline["scenarios"].as_array().unwrap();
    assert_eq!(original.len(), 122);
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 140);
    assert_eq!(
        current[..122]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        original
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .collect::<Vec<_>>()
    );
    for raw in original {
        let s = current
            .iter()
            .find(|s| s.id == raw["id"].as_str().unwrap())
            .unwrap();
        assert_eq!(s.version, raw["version"].as_u64().unwrap() as u32);
        assert_eq!(s.unit, raw["unit"].as_str().unwrap_or(""));
        assert_eq!(
            s.initial,
            serde_json::from_value::<State>(raw["initial"].clone()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&s.goal).unwrap(),
            serde_json::to_value(serde_json::from_value::<Goal>(raw["goal"].clone()).unwrap())
                .unwrap()
        );
        assert_eq!(
            s.solution,
            serde_json::from_value::<Vec<String>>(raw["solution"].clone()).unwrap()
        );
    }
    let units: Value = serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    assert_eq!(units["units"].as_array().unwrap().len(), 35);
    assert_eq!(
        &units["units"].as_array().unwrap()[..30],
        baseline["units"]["units"].as_array().unwrap().as_slice()
    );
    for old in baseline["units"]["units"].as_array().unwrap() {
        assert!(units["units"].as_array().unwrap().contains(old));
    }
    let all = catalog();
    let old = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let p = progress::decode(old, &all).unwrap();
    let old_attempts = serde_json::to_value(&p.attempts).unwrap();
    let mut p = progress::decode(old, &all).unwrap();
    for s in labs() {
        let mut a = Attempt::new(&s, 9);
        workspace::prepare(&mut a);
        dhcpv4_teaching::prepare(&mut a, s.dhcpv4_teaching.as_ref().unwrap(), true).unwrap();
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let q = progress::decode(&bytes, &all).unwrap();
    let mut new = serde_json::to_value(&q.attempts).unwrap();
    for id in IDS {
        new.as_object_mut().unwrap().remove(id);
    }
    assert_eq!(new, old_attempts);
    let mut revisions = std::collections::BTreeSet::new();
    for a in q.attempts.values() {
        if let State::Script { run: Some(r), .. } = &a.state {
            revisions.insert(r.engine_revision);
        }
        if let Some(w) = &a.workspace {
            for r in w.script_history.iter().chain(&w.script_history_more) {
                revisions.insert(r.engine_revision);
            }
        }
    }
    assert!(revisions.contains(&6) && revisions.contains(&7) && revisions.contains(&8));
}
#[test]
fn context_tampering_wrong_kind_and_malformed_policy_fail_without_changing_saved_bytes() {
    let s = labs().remove(0);
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    dhcpv4_teaching::prepare(&mut a, s.dhcpv4_teaching.as_ref().unwrap(), true).unwrap();
    p.attempts.insert(s.id.clone(), a);
    let original = serde_json::to_vec(&p).unwrap();
    let mut raw: Value = serde_json::from_slice(&original).unwrap();
    raw["attempts"][&s.id]["state"]["simulation"]["topology"]["server"] = "192.0.2.200".into();
    assert!(progress::decode(&serde_json::to_vec(&raw).unwrap(), &all).is_err());
    assert_eq!(serde_json::to_vec(&p).unwrap(), original);
    let mut bad = serde_json::to_value(&s.initial).unwrap();
    bad["simulation"]["policy"]["pools"][plan(&s)["work_pool"].as_str().unwrap()]["lease_seconds"] =
        0.into();
    assert!(serde_json::from_value::<State>(bad).is_err());
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let prior = std::fs::read(&path).unwrap();
    p.attempts.get_mut(&s.id).unwrap().state = all[0].initial.clone();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(path).unwrap(), prior);
}

#[test]
fn capstone_additional_valid_reservations_are_an_equivalent_strategy_without_dynamic_capacity() {
    let s = labs().remove(2);
    let row = plan(&s);
    let work = row["work_pool"].as_str().unwrap();
    let clients: Vec<String> = serde_json::from_value(row["clients"].clone()).unwrap();
    let mut st = passed(&s);
    let b = sim(&st).policy().pools[work].subnet.network.octets();
    let addresses = [
        std::net::Ipv4Addr::new(b[0], b[1], b[2], 11),
        std::net::Ipv4Addr::new(b[0], b[1], b[2], 12),
        std::net::Ipv4Addr::new(b[0], b[1], b[2], 13),
    ];
    for address in addresses {
        assert!(sim(&st).topology().segments.values().all(|segment| {
            !segment.occupied.contains(&address)
                && segment.server_interface != Some(address)
                && segment.relay_interface != Some(address)
        }));
        assert_ne!(sim(&st).topology().server, address);
    }
    edit(&mut st, |p| {
        let pool = p.pools.get_mut(work).unwrap();
        pool.first = std::net::Ipv4Addr::new(b[0], b[1], b[2], 10);
        pool.last = pool.first;
        assert!(pool.excluded.contains(&pool.first));
        for (client, address) in clients.iter().take(3).zip(addresses) {
            pool.reservations.insert(client.clone(), address);
        }
    });
    assert!(sim(&st).policy().validate(sim(&st).topology()).is_ok());
    assert!(checks(&s, &st).iter().all(|row| row.0));
    assert!(engine::solved(&st, &s.goal));
}
