//! Prospective capability/source/codec gates only; no catalog or host networking.
#[path = "support/ere_registration.rs"]
mod ere_registration;
#[path = "support/hsrp_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    hsrp as core, hsrp_workspace as work, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use ere_registration::partition_fs_registration as disk_registration;
use serde_json::{Value, json};
fn value<T: serde::Serialize + ?Sized>(s: &T) -> Value {
    serde_json::to_value(s).unwrap()
}
fn solved() -> (work::Definition, work::Session) {
    let d = fixture::definition();
    let mut s = work::Session::new(&d).unwrap();
    for a in fixture::actions(&d, false) {
        s.apply_action(&d, &a, false).unwrap();
    }
    (d, s)
}
// Keep all historical193/190 proofs only after checking the complete current197
// and canonical full disk source/unit append; no unchecked tail or numeric bump.
fn current193() -> Vec<pack::Scenario> {
    let normal199 = pack::bundled().unwrap();
    assert_eq!(normal199.len(), 199);
    let current = disk_registration::literal_registration::before_scenarios(&normal199).to_vec();
    assert_eq!(current.len(), 197);
    let old = disk_registration::before_scenarios(&current);
    assert_eq!(old.len(), 193);
    let units53 = dlms_learning_lab::course::bundled(&normal199).unwrap();
    assert_eq!(units53.len(), 53);
    let units52 = disk_registration::literal_registration::before_typed_units(&units53).to_vec();
    assert_eq!(units52.len(), 52);
    let raw51 =
        disk_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let units51 =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw51).unwrap(), old).unwrap();
    assert_eq!(
        value(disk_registration::before_typed_units(&units52)),
        value(&units51)
    );
    old.to_vec()
}
// Keep the complete old190 fixture scope, but admit the actual current197 first.
fn baseline190() -> Vec<pack::Scenario> {
    let all = current193();
    assert_eq!(all.len(), 193);
    let old = ere_registration::before_scenarios(&all);
    assert_eq!(old.len(), 190);
    old.to_vec()
}
fn catalog() -> Vec<pack::Scenario> {
    let mut all = baseline190();
    assert_eq!(all.len(), 190);
    all.push(fixture::scenario());
    let chunks = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
#[test]
fn private_fixture_preserves_normal190_without_any_registered_hsrp_or_fake_coverage() {
    let old = baseline190();
    let all = catalog();
    assert_eq!(all.len(), 191);
    assert_eq!(value(&all[..190]), value(&old));
    let s = all.last().unwrap();
    work::source(s).unwrap();
    assert!(!engine::solved(&s.initial, &s.goal));
    let mut state = s.initial.clone();
    let before = value(&state);
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    assert_eq!(value(&state), before);
    for a in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, a).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn exact_nonvacuous_initial_and_partial_rows_require_two_peers_every_client_and_each_checkpoint() {
    let d = fixture::definition();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    assert_eq!(
        d.checks(&s).iter().map(|r| r.0).collect::<Vec<_>>(),
        [
            true, true, false, false, false, false, false, false, false, true, false, false, true,
            false, false, true, false, false
        ]
    );
    let actions = fixture::actions(&d, false);
    s.apply_action(&d, &actions[0], false).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
    s.apply_action(&d, &actions[1], false).unwrap();
    assert!(!d.checks(&s).iter().all(|r| r.0));
    s.apply_action(&d, &actions[2], false).unwrap();
    assert_eq!(
        d.checks(&s).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true; 18]
    );
}
#[test]
fn alternate_priorities_and_preempt_choices_satisfy_behavior_without_one_canned_configuration() {
    let (d, base) = solved();
    let mut a = base.clone();
    a.apply_action(
        &d,
        &work::Action::Router {
            router: core::RouterId::B,
            config: fixture::config(140, true),
        },
        false,
    )
    .unwrap();
    assert_eq!(a.current.active(), Some(core::RouterId::B));
    assert!(d.checks(&a).iter().all(|r| r.0));
    let mut b = base.clone();
    b.apply_action(
        &d,
        &work::Action::Router {
            router: core::RouterId::A,
            config: fixture::config(150, false),
        },
        false,
    )
    .unwrap();
    assert_eq!(b.current.active(), Some(core::RouterId::A));
    assert!(d.checks(&b).iter().all(|r| r.0));
    assert_ne!(value(&a.current), value(&b.current));
}
#[test]
fn saved_incumbency_follows_real_configuration_history_and_cannot_be_sorted_or_forged() {
    let (d, mut s) = solved();
    let mut off = fixture::config(110, false);
    off.participating = false;
    s.apply_action(
        &d,
        &work::Action::Router {
            router: core::RouterId::A,
            config: off,
        },
        false,
    )
    .unwrap();
    s.apply_action(
        &d,
        &work::Action::Router {
            router: core::RouterId::A,
            config: fixture::config(110, false),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.current.active(), Some(core::RouterId::B));
    assert!(d.checks(&s).iter().all(|r| r.0));
    let raw = s.to_json(&d).unwrap();
    let resumed = work::Session::from_json(&raw, &d).unwrap();
    assert_eq!(resumed, s);
    assert_eq!(resumed.current.active(), Some(core::RouterId::B));
    let (d, base) = solved();
    let fake = base
        .inspect(
            &d,
            &[
                core::FaultEvent::RouterPower {
                    router: core::RouterId::A,
                    up: false,
                },
                core::FaultEvent::RouterPower {
                    router: core::RouterId::A,
                    up: true,
                },
            ],
            false,
        )
        .unwrap();
    fake.validate().unwrap();
    assert_eq!(fake.active(), Some(core::RouterId::B));
    let mut forged = base;
    forged.current = fake;
    forged.validate().unwrap();
    assert!(d.bound(&forged).is_err());
}
#[test]
fn temporary_fault_inspection_never_publishes_assignment_facts_roles_or_grade() {
    let (d, s) = solved();
    let before = value(&s);
    for event in [
        core::FaultEvent::RouterPower {
            router: core::RouterId::A,
            up: false,
        },
        core::FaultEvent::WanLink {
            router: core::RouterId::A,
            up: false,
        },
        core::FaultEvent::UpstreamRoute {
            router: core::RouterId::A,
            available: false,
        },
        core::FaultEvent::ClientLink {
            client: 1,
            up: false,
        },
    ] {
        let observed = s.inspect(&d, &[event], false).unwrap();
        assert_ne!(observed, s.current);
        assert_eq!(value(&s), before);
    }
    assert!(
        s.inspect(&d, &[core::FaultEvent::PeerCommunicationLost], false)
            .is_err()
    );
    assert_eq!(value(&s), before);
    assert_eq!(
        d.checks(&s).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true; 18]
    );
}
#[test]
fn peer_disable_real_gateway_and_pending_drafts_cannot_bypass_resilience() {
    let (d, s) = solved();
    let mut off = s.clone();
    let mut c = fixture::config(100, false);
    c.participating = false;
    off.apply_action(
        &d,
        &work::Action::Router {
            router: core::RouterId::B,
            config: c,
        },
        false,
    )
    .unwrap();
    assert!(d.checks(&off).iter().any(|r| !r.0));
    let mut real = s.clone();
    real.apply_action(
        &d,
        &work::Action::Gateway {
            client: 1,
            gateway: d.policy.initial.router(core::RouterId::A).address,
        },
        false,
    )
    .unwrap();
    assert!(
        real.current
            .client_status(1)
            .unwrap()
            .forwarding_available()
    );
    assert!(!d.checks(&real).iter().all(|r| r.0));
    let mut draft = s.clone();
    let mut pending = draft.draft.clone();
    pending.routers[0].priority = "".into();
    draft.set_draft(&d, pending, false).unwrap();
    assert!(draft.current == s.current);
    assert!(!d.checks(&draft)[0].0);
}
#[test]
fn source_faults_and_config_authority_remain_owned_even_with_jointly_changed_markers() {
    let (d, s) = solved();
    let mut raw = value(&s);
    for key in ["initial", "current"] {
        raw[key]["routers"][1]["facts"]["power_on"] = Value::Bool(false);
        raw[key]["standby"] = Value::Null;
    }
    let forged: work::Session = serde_json::from_value(raw).unwrap();
    forged.validate().unwrap();
    assert!(d.bound(&forged).is_err());
    let mut protected = d.clone();
    protected
        .policy
        .editable_routers
        .retain(|id| *id != core::RouterId::B);
    assert!(protected.bound(&s).is_err());
    let mut forged_goal = d.clone();
    forged_goal.policy.probes.pop();
    assert!(forged_goal.shape().is_err());
    let mut impossible = d;
    impossible.policy.probes[0].events.insert(
        1,
        work::Event::Power {
            router: core::RouterId::B,
            up: false,
        },
    );
    impossible.policy.probes[0].expected_active.insert(1, None);
    assert!(impossible.shape().is_err());
}
#[test]
fn wan_tracking_needs_peer_preemption_and_checks_failure_before_return() {
    let (mut d, s) = solved();
    d.policy.probes.push(work::Probe {
        title: "WAN down and return".into(),
        events: vec![
            work::Event::Wan {
                router: core::RouterId::A,
                up: false,
            },
            work::Event::Wan {
                router: core::RouterId::A,
                up: true,
            },
        ],
        expected_active: vec![Some(core::RouterId::B), None],
    });
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let mut configured = s;
    let mut a = fixture::config(110, false);
    a.track_wan_decrement = Some(30);
    configured
        .apply_action(
            &d,
            &work::Action::Router {
                router: core::RouterId::A,
                config: a,
            },
            false,
        )
        .unwrap();
    assert!(d.checks(&configured).iter().any(|r| !r.0));
    configured
        .apply_action(
            &d,
            &work::Action::Router {
                router: core::RouterId::B,
                config: fixture::config(100, true),
            },
            false,
        )
        .unwrap();
    assert!(d.checks(&configured).iter().all(|r| r.0));
}
#[test]
fn separate_example_insert_is_inert_then_real_apply_experiment_repair_and_reset_keep_assignment() {
    let d = fixture::definition();
    let mut s = work::Session::new(&d).unwrap();
    let assignment = s.current.clone();
    let histories = s.history.clone();
    for (i, w) in d.lesson.worked.iter().enumerate() {
        let world = s.example.clone();
        s.insert_example(&d, i).unwrap();
        assert_eq!(s.example, world);
        assert!(s.example_draft != work::Draft::from(&s.example));
        s.apply_draft(&d, true).unwrap();
        assert_eq!(s.current, assignment);
        assert_eq!(s.history, histories);
        assert_eq!(s.example_history.last(), Some(&w.action));
    }
    assert!(d.practice_checks(&s).iter().all(|r| r.0));
    s.apply_action(&d, &d.lesson.experiment, true).unwrap();
    assert!(d.practice_checks(&s).iter().any(|r| !r.0));
    s.apply_action(&d, &d.lesson.repair, true).unwrap();
    assert!(d.practice_checks(&s).iter().all(|r| r.0));
    s.reset_example(&d).unwrap();
    assert_eq!(s.example, d.lesson.policy.initial);
    assert_eq!(s.current, assignment);
    assert!(s.example_history.is_empty());
}
#[test]
fn bounded_history_and_invalid_apply_are_atomic_without_pruning_or_rebasing_incumbency() {
    let (d, mut s) = solved();
    while s.history.len() < work::MAX_ACTIONS {
        let mut config = s.current.router(core::RouterId::B).config.clone();
        config.preempt = !config.preempt;
        s.apply_action(
            &d,
            &work::Action::Router {
                router: core::RouterId::B,
                config,
            },
            false,
        )
        .unwrap();
    }
    let before = value(&s);
    let mut config = s.current.router(core::RouterId::B).config.clone();
    config.priority += 1;
    assert!(
        s.apply_action(
            &d,
            &work::Action::Router {
                router: core::RouterId::B,
                config
            },
            false
        )
        .is_err()
    );
    assert_eq!(value(&s), before);
    let mut invalid = s.draft.clone();
    invalid.routers[1].priority = "0".into();
    s.set_draft(&d, invalid, false).unwrap();
    let before = value(&s);
    assert!(s.apply_draft(&d, false).is_err());
    assert_eq!(value(&s), before);
}
#[test]
fn raw_streaming_unknown_duplicate_and_excess_bodies_are_refused_before_semantic_use() {
    let (d, s) = solved();
    let raw = s.to_json(&d).unwrap();
    assert_eq!(work::Session::from_json(&raw, &d).unwrap(), s);
    assert!(work::Session::from_json(&vec![b' '; work::MAX_SESSION + 1], &d).is_err());
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_DEFINITION + 1]).is_err());
    let mut extra = value(&s);
    extra["history"] = json!([null]);
    assert!(work::Session::from_json(&serde_json::to_vec(&extra).unwrap(), &d).is_err());
    let entry = serde_json::to_string(&s.history[0]).unwrap();
    let bad = format!(
        "{{\"version\":1,\"history\":[{},{{\"do-not-read\":true}}]}}",
        vec![entry; 128].join(",")
    );
    assert!(
        serde_json::from_str::<work::Session>(&bad)
            .unwrap_err()
            .to_string()
            .contains("item limit")
    );
    let duplicate =
        String::from_utf8(raw)
            .unwrap()
            .replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(work::Session::from_json(duplicate.as_bytes(), &d).is_err());
}
#[test]
fn all_attempts_save_export_inspect_restore_and_completion_require_catalog_authority() {
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let all = catalog();
    let source = all.last().unwrap();
    let mut good = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(source, 1);
    for command in &source.solution {
        engine::execute_for_goal(&mut a.state, &source.goal, command).unwrap();
    }
    workspace::prepare(&mut a);
    good.attempts.insert(source.id.clone(), a);
    let old = &all[0];
    good.attempts.insert(old.id.clone(), Attempt::new(old, 2));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let caller = value(&good);
    progress::save_with_catalog(&path, &good, &all).unwrap();
    assert_eq!(value(&good), caller);
    let raw = std::fs::read(&path).unwrap();
    let saved = progress::load(&path, &all).unwrap();
    assert!(saved.migration_receipts.is_some());
    assert!(progress::current_solved(
        &saved.attempts[&source.id],
        source
    ));
    let receipt = value(&saved.migration_receipts);
    let portable = dir.path().join("portable.json");
    backup::export(&portable, &good, &all).unwrap();
    let cancel = AtomicBool::new(false);
    let count = AtomicUsize::new(0);
    let inspected = backup::inspect(&portable, &all, &cancel, &count).unwrap();
    assert!(progress::current_solved(
        &inspected.progress.attempts[&source.id],
        source
    ));
    let mut bad = good;
    let State::Hsrp { simulation } = &mut bad.attempts.get_mut(&source.id).unwrap().state else {
        panic!()
    };
    let wrong = simulation
        .inspect(
            if let Goal::Hsrp { definition } = &source.goal {
                definition
            } else {
                panic!()
            },
            &[
                core::FaultEvent::RouterPower {
                    router: core::RouterId::A,
                    up: false,
                },
                core::FaultEvent::RouterPower {
                    router: core::RouterId::A,
                    up: true,
                },
            ],
            false,
        )
        .unwrap();
    simulation.current = wrong;
    simulation.validate().unwrap();
    assert!(!progress::current_solved(&bad.attempts[&source.id], source));
    let before = value(&bad);
    assert!(progress::save_with_catalog(&path, &bad, &all).is_err());
    assert!(progress::save(&path, &bad).is_err());
    let forbidden = dir.path().join("bad-export.json");
    assert!(backup::export(&forbidden, &bad, &all).is_err());
    let candidate = backup::Candidate {
        progress: bad,
        rechecked: 0,
    };
    assert!(backup::restore(&path, &candidate, &all).is_err());
    assert_eq!(value(&candidate.progress), before);
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(
        value(&progress::load(&path, &all).unwrap().migration_receipts),
        receipt
    );
}
#[test]
fn inactive_missing_wrong_kind_and_joint_world_markers_cannot_supply_save_authority() {
    let mut all = catalog();
    let first = all.last().unwrap().clone();
    let mut peer = first.clone();
    peer.id = "private-hsrp-peer-context".into();
    let Goal::Hsrp { definition } = &mut peer.goal else {
        panic!()
    };
    let mut world = value(&definition.policy.initial);
    world["group"] = json!(8);
    definition.policy.initial =
        core::State::from_json(&serde_json::to_vec(&world).unwrap()).unwrap();
    peer.initial = State::Hsrp {
        simulation: Box::new(work::Session::new(definition).unwrap()),
    };
    work::source(&peer).unwrap();
    all.push(peer.clone());
    let mut good = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in [&first, &peer] {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        good.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save_with_catalog(&path, &good, &all).unwrap();
    let raw = std::fs::read(&path).unwrap();
    for mode in 0..4 {
        let mut bad: Progress = serde_json::from_value(value(&good)).unwrap();
        let mut sources = all.clone();
        match mode {
            0 => sources.retain(|s| s.id != peer.id),
            1 => bad.attempts.get_mut(&peer.id).unwrap().state = all[0].initial.clone(),
            2 => {
                let mut foreign = first.clone();
                foreign.id = peer.id.clone();
                *sources.last_mut().unwrap() = foreign;
            }
            _ => {
                let a = bad.attempts.get_mut(&peer.id).unwrap();
                let State::Hsrp { simulation } = &mut a.state else {
                    panic!()
                };
                let mut encoded = value(simulation);
                encoded["initial"]["group"] = json!(9);
                encoded["current"]["group"] = json!(9);
                **simulation = serde_json::from_value(encoded).unwrap();
                simulation.validate().unwrap();
            }
        }
        let before = value(&bad);
        let receipt = value(&bad.migration_receipts);
        assert!(progress::save_with_catalog(&path, &bad, &sources).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        assert_eq!(value(&bad), before);
        assert_eq!(value(&bad.migration_receipts), receipt);
    }
    let mut smuggled = value(&good);
    smuggled["catalog"] = json!([peer]);
    assert!(progress::decode(&serde_json::to_vec(&smuggled).unwrap(), &all).is_err());
    let mut smuggled = value(&good);
    smuggled["attempts"][&first.id]["state"]["definition"] = value(&first.goal);
    assert!(progress::decode(&serde_json::to_vec(&smuggled).unwrap(), &all).is_err());
}

#[test]
fn source_probes_require_both_real_failure_returns_in_the_same_fact_and_sequence() {
    let d = fixture::definition();
    d.policy.validate().unwrap();
    for index in 0..2 {
        let mut missing = d.policy.clone();
        assert_eq!(missing.probes[index].events.len(), 2);
        missing.probes[index].events.pop();
        missing.probes[index].expected_active.pop();
        assert!(
            missing.validate().is_err(),
            "Missing return for router index {index}"
        );

        let mut wrong_fact = d.policy.clone();
        let router = if index == 0 {
            core::RouterId::A
        } else {
            core::RouterId::B
        };
        wrong_fact.probes[index].events[1] = work::Event::Lan { router, up: true };
        assert!(
            wrong_fact.validate().is_err(),
            "An already-up LAN cannot restore power for {router:?}"
        );

        let mut wrong_router = d.policy.clone();
        let other = if index == 0 {
            core::RouterId::B
        } else {
            core::RouterId::A
        };
        wrong_router.probes[index].events[1] = work::Event::Power {
            router: other,
            up: true,
        };
        assert!(
            wrong_router.validate().is_err(),
            "Another router cannot restore {router:?}"
        );
    }
    let mut neither_return = d.policy.clone();
    for probe in &mut neither_return.probes {
        probe.events.truncate(1);
        probe.expected_active.truncate(1);
    }
    assert!(neither_return.validate().is_err());
    // Both supported failure types have a legitimate same-fact return.
    let mut lan = d.policy.clone();
    for probe in &mut lan.probes {
        for event in &mut probe.events {
            let work::Event::Power { router, up } = *event else {
                panic!()
            };
            *event = work::Event::Lan { router, up };
        }
    }
    lan.validate().unwrap();
    let (_, solved) = solved();
    assert_eq!(
        lan.rows(&solved.current)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true; 17]
    );
    assert_eq!(
        d.checks(&solved).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true; 18]
    );
}

#[test]
fn actual193_full_catalog_roundtrip_and_private_hsrp_solve_preserve_every_current_object() {
    let current = current193();
    assert_eq!(current.len(), 193);
    let old = baseline190();
    assert_eq!(
        value(ere_registration::before_scenarios(&current)),
        value(&old)
    );
    let mut all = current.clone();
    all.push(fixture::scenario());
    let chunks = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let all = pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(all.len(), 194);
    assert_eq!(value(&all[..193]), value(&current));
    let s = &all[193];
    work::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::context(&a, s).unwrap();
    assert!(!progress::current_solved(&a, s));
    for action in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    assert_eq!(value(&current193()), value(&current));

    // The unchanged predecessor assertions above remain193/194. Separately
    // retain the entire live197 catalog around the new private capability.
    let normal199 = pack::bundled().unwrap();
    assert_eq!(normal199.len(), 199);
    let live = disk_registration::literal_registration::before_scenarios(&normal199).to_vec();
    assert_eq!(live.len(), 197);
    assert_eq!(
        value(disk_registration::before_scenarios(&live)),
        value(&current)
    );
    let mut full = live.clone();
    full.push(fixture::scenario());
    let chunks = full
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    assert!(chunks.iter().all(|s| s.len() <= pack::MAX_PACK_BYTES));
    let full = pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(full.len(), 198);
    assert_eq!(value(&full[..197]), value(&live));
    assert_eq!(value(&full[197]), value(&fixture::scenario()));
    let source = &full[197];
    let mut attempt = Attempt::new(source, 1);
    workspace::prepare(&mut attempt);
    work::context(&attempt, source).unwrap();
    assert!(!progress::current_solved(&attempt, source));
    for action in &source.solution {
        engine::execute_for_goal(&mut attempt.state, &source.goal, action).unwrap();
    }
    assert!(progress::current_solved(&attempt, source));
    assert_eq!(
        value(disk_registration::literal_registration::before_scenarios(
            &pack::bundled().unwrap()
        )),
        value(&live)
    );
    assert_eq!(value(&pack::bundled().unwrap()), value(&normal199));
    let mut actual = normal199.clone();
    actual.push(fixture::scenario());
    let chunks = actual
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let actual =
        pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(actual.len(), 200);
    assert_eq!(value(&actual[..199]), value(&normal199));
    let mut actual_profile = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &actual {
        actual_profile
            .attempts
            .insert(source.id.clone(), Attempt::new(source, 1));
    }
    let source = &actual[199];
    let attempt = actual_profile.attempts.get_mut(&source.id).unwrap();
    workspace::prepare(attempt);
    for action in &source.solution {
        engine::execute_for_goal(&mut attempt.state, &source.goal, action).unwrap();
    }
    assert!(progress::current_solved(attempt, source));
    for source in &actual[..199] {
        assert_eq!(
            value(&actual_profile.attempts[&source.id].state),
            value(&source.initial)
        );
        assert!(!progress::current_solved(
            &actual_profile.attempts[&source.id],
            source
        ));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("full-current199-profile.json");
    let before = value(&actual_profile);
    let receipts = value(&actual_profile.migration_receipts);
    progress::save_with_catalog(&path, &actual_profile, &actual).unwrap();
    assert_eq!(value(&actual_profile), before);
    assert_eq!(value(&actual_profile.migration_receipts), receipts);
    assert_eq!(value(&progress::load(&path, &actual).unwrap()), before);
}

#[path = "support/hsrp_capacity.rs"]
mod capacity;
#[test]
fn complete_attempt_admission_is_atomic_for_draft_view_hint_mode_and_wire_growth() {
    use dlms_learning_lab::hsrp_workspace::binding::{self, Change};
    let source = capacity::source(fixture::scenario());
    let Goal::Hsrp { definition: d } = &source.goal else {
        panic!()
    };
    let mut a = Attempt::new(&source, 1);
    workspace::prepare(&mut a);
    for action in fixture::actions(d, false) {
        let State::Hsrp { simulation } = &mut a.state else {
            panic!()
        };
        simulation.apply_action(d, &action, false).unwrap();
    }
    binding::change(&mut a, &source, Change::Check { practice: false }).unwrap();
    binding::change(
        &mut a,
        &source,
        Change::View(dlms_learning_lab::script_teaching::View::Example),
    )
    .unwrap();
    a.hints = 9;
    a.challenge = true;
    capacity::fill(&source, &mut a, work::MAX_CONTEXT);
    let wire_before = value(&a);
    binding::change(&mut a, &source, Change::Check { practice: false }).unwrap();
    assert_eq!(
        value(&a),
        wire_before,
        "Exact unchanged full wire must fit at the limit"
    );
    let before = value(&a);
    let State::Hsrp { simulation } = &a.state else {
        panic!()
    };
    let mut draft = simulation.draft.clone();
    draft.routers[0].priority.push('x');
    assert!(
        binding::change(
            &mut a,
            &source,
            Change::Draft {
                draft,
                practice: false
            }
        )
        .is_err()
    );
    assert_eq!(value(&a), before);
    let mut wire: Attempt = serde_json::from_value(before.clone()).unwrap();
    let State::Hsrp { simulation } = &mut wire.state else {
        panic!()
    };
    simulation.view = dlms_learning_lab::script_teaching::View::Introduction;
    assert_eq!(capacity::size(&source, &wire), work::MAX_CONTEXT + 5);
    assert!(
        binding::change(
            &mut a,
            &source,
            Change::View(dlms_learning_lab::script_teaching::View::Introduction)
        )
        .is_err()
    );
    assert_eq!(value(&a), before);
    assert!(binding::request_hint(&mut a, &source).is_err());
    assert_eq!(value(&a), before);
    assert!(binding::set_challenge(&mut a, &source, false).is_err());
    assert_eq!(value(&a), before);
    assert!(binding::admit_launch(Some(&a), &source, u64::MAX).is_err());
    assert_eq!(value(&a), before);
    assert!(progress::current_solved(&a, &source));
    a.transcript.remove(0);
    binding::request_hint(&mut a, &source).unwrap();
    binding::set_challenge(&mut a, &source, false).unwrap();
    binding::change(
        &mut a,
        &source,
        Change::View(dlms_learning_lab::script_teaching::View::Introduction),
    )
    .unwrap();
    assert_eq!(a.hints, 10);
    assert!(!a.challenge);
    assert!(progress::current_solved(&a, &source));
}
#[test]
fn candidate_policy_history_and_workspace_preparation_never_publish_past_complete_context_cap() {
    use dlms_learning_lab::hsrp_workspace::binding::{self, Change};
    let source = capacity::source(fixture::scenario());
    let mut a = Attempt::new(&source, 1);
    let State::Hsrp { simulation } = &mut a.state else {
        panic!()
    };
    simulation.draft.routers[1].participating = true;
    capacity::fill(&source, &mut a, work::MAX_CONTEXT);
    let before = value(&a);
    assert!(binding::change(&mut a, &source, Change::Apply { practice: false }).is_err());
    assert_eq!(value(&a), before);
    assert!(binding::prepare(&mut a, &source).is_err());
    assert_eq!(value(&a), before);
    assert!(binding::admit_launch(Some(&a), &source, 1).is_err());
    assert_eq!(value(&a), before);
    a.transcript.remove(0);
    binding::prepare(&mut a, &source).unwrap();
    binding::change(&mut a, &source, Change::Apply { practice: false }).unwrap();
    let State::Hsrp { simulation } = &a.state else {
        panic!()
    };
    assert_eq!(simulation.history.len(), 1);
    assert!(
        simulation
            .current
            .router(core::RouterId::B)
            .config
            .participating
    );
    assert!(
        !progress::current_solved(&a, &source),
        "Both clients still need the virtual gateway"
    );
}
#[test]
fn fresh_source_preflights_widest_attempt_without_answer_replay_and_below_pack_cap() {
    let source = capacity::oversized_fresh(fixture::scenario());
    let before = value(&source);
    let Goal::Hsrp { definition } = &source.goal else {
        panic!()
    };
    definition.shape().unwrap();
    assert!(serde_json::to_vec(&source).unwrap().len() < work::MAX_CONTEXT);
    let future = Attempt::new(&source, u64::MAX);
    assert!(capacity::size(&source, &future) > work::MAX_CONTEXT);
    assert!(work::source(&source).is_err());
    assert_eq!(value(&source), before);
    let text = json!({"schema_version":1,"scenarios":[source]}).to_string();
    assert!(text.len() < pack::MAX_PACK_BYTES);
    assert!(pack::load(&text).is_err());
}
