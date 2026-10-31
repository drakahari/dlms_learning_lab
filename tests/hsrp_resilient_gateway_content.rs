//! Prospective content/source gates only. No IOS, packets or host network is run.
use dlms_learning_lab::{
    backup, course, developer_digest,
    engine::{self, Goal, State},
    hsrp as core, hsrp_workspace as work,
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::{Value, json};
#[path = "support/ntp_registration.rs"]
mod registration;
use registration::ere_registration::original190::before_ntp as original;
use registration::ere_registration::partition_fs_registration as disk_registration;
const DATA: &str = include_str!("../scenarios/hsrp-resilient-gateway-v1.json");
const REVIEW: &str = include_str!("fixtures/hsrp-resilient-gateway-review.json");
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn labs() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(DATA).unwrap().scenarios
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::Hsrp { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &State) -> &work::Session {
    let State::Hsrp { simulation } = s else {
        panic!()
    };
    simulation
}
fn session_mut(s: &mut State) -> &mut work::Session {
    let State::Hsrp { simulation } = s else {
        panic!()
    };
    simulation
}
fn rows(s: &State, source: &Scenario) -> Vec<bool> {
    engine::checks(s, &source.goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
fn bits(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn commands(state: &mut State, source: &Scenario, commands: &Value) {
    for c in commands.as_array().unwrap() {
        engine::execute_for_goal(state, &source.goal, c.as_str().unwrap()).unwrap();
    }
}
fn solved(source: &Scenario) -> State {
    let mut state = source.initial.clone();
    for c in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &source.goal));
    state
}
// Retain the complete old193 proofs only after validating every current197
// field and the canonical whole disk append; never recover by unchecked slice.
fn current193() -> Vec<Scenario> {
    let normal199 = pack::bundled().unwrap();
    assert_eq!(normal199.len(), 199);
    let current = disk_registration::literal_registration::before_scenarios(&normal199).to_vec();
    assert_eq!(current.len(), 197);
    let units53 = course::bundled(&normal199).unwrap();
    assert_eq!(units53.len(), 53);
    let units = disk_registration::literal_registration::before_typed_units(&units53).to_vec();
    assert_eq!(units.len(), 52);
    let reopened = course::load(&json!({"version":1,"units":units}).to_string(), &current).unwrap();
    assert_eq!(value(&reopened), value(&units));
    let old = disk_registration::before_scenarios(&current);
    assert_eq!(old.len(), 193);
    old.to_vec()
}
fn baseline() -> Vec<Scenario> {
    let r = review();
    let ntp = include_str!("../scenarios/ntp-client-server-v1.json");
    assert_eq!(
        developer_digest::hash(ntp.as_bytes()),
        r["dependencies"]["ntp_pack_sha256"].as_str().unwrap()
    );
    let mut all = original::load();
    assert_eq!(all.len(), 186);
    all.extend(serde_json::from_str::<Pack>(ntp).unwrap().scenarios);
    assert_eq!(all.len(), 190);
    let chunks = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let all = pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let current = current193();
    assert_eq!(current.len(), 193);
    let checked190 = registration::ere_registration::before_scenarios(&current);
    assert_eq!(checked190.len(), 190);
    assert_eq!(value(&all), value(checked190));
    all
}
fn combined() -> Vec<Scenario> {
    let mut all = baseline();
    all.extend(labs());
    let chunks = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn profile(all: &[Scenario]) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (i, s) in all.iter().enumerate() {
        let mut a = Attempt::new(s, i as u64 + 1);
        workspace::prepare(&mut a);
        p.attempts.insert(s.id.clone(), a);
    }
    p.clock = all.len() as u64;
    p
}

#[test]
fn four_distinct_private_assignments_preserve_all190_full_specs_and_fifty_units() {
    let old = baseline();
    let all = combined();
    assert_eq!(all.len(), 194);
    assert_eq!(value(&all[..190]), value(&old));
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let checked51 = disk_registration::before_units(current_raw).unwrap();
    let checked50 = registration::ere_registration::before_units(&checked51).unwrap();
    let raw = checked50.as_slice();
    let r = review();
    assert_eq!(
        developer_digest::hash(raw),
        r["dependencies"]["units_sha256"].as_str().unwrap()
    );
    let mut units = course::load(std::str::from_utf8(raw).unwrap(), &old).unwrap();
    assert_eq!(units.len(), 50);
    let before = value(&units);
    units.push(serde_json::from_value(r["unit_proposal"].clone()).unwrap());
    let checked = course::load(&json!({"version":1,"units":units}).to_string(), &all).unwrap();
    assert_eq!(value(&checked[..50]), before);
    assert_eq!(
        checked[50].labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    for (i, s) in labs().iter().enumerate() {
        let d = definition(s);
        work::source(s).unwrap();
        d.validate().unwrap();
        assert_eq!(session(&s.initial), &work::Session::new(d).unwrap());
        assert_eq!(s.prerequisites[..2], ["ccna-ipv4", "ccna-gateway"]);
        if i > 0 {
            assert_eq!(s.prerequisites[2], labs()[i - 1].id);
        }
        assert_eq!(d.lesson.concepts.len(), 6);
        assert!(!s.guide.iter().any(|g| !g.commands.is_empty()));
        assert!(s.simplifications.contains("No IOS parser"));
        assert!(rows(&s.initial, s).contains(&false));
        let a = Attempt::new(s, 1);
        work::context(&a, s).unwrap();
    }
}

#[test]
fn references_and_two_distinct_alternatives_satisfy_every_client_and_every_checkpoint() {
    let r = review();
    for (s, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        assert_eq!(value(&s.solution), r["reference"]);
        let mut finals = Vec::new();
        for cs in std::iter::once(&r["reference"]).chain(r["alternatives"].as_array().unwrap()) {
            let mut state = s.initial.clone();
            commands(&mut state, s, cs);
            let checks = rows(&state, s);
            assert_eq!(
                checks,
                vec![true; r["initial_expected"].as_array().unwrap().len()]
            );
            assert!(engine::solved(&state, &s.goal));
            let now = &session(&state).current;
            let d = definition(s);
            assert!(work::same_fixed_facts(&d.policy.initial, now).unwrap());
            for c in now.clients() {
                assert_eq!(c.gateway, now.virtual_ip());
                assert!(now.client_status(c.id).unwrap().forwarding_available());
            }
            for id in core::RouterId::ALL {
                if !d.policy.editable_routers.contains(&id) {
                    assert_eq!(now.router(id), d.policy.initial.router(id));
                }
            }
            finals.push(value(now));
        }
        assert_eq!(finals.len(), 3);
        assert_ne!(finals[0], finals[1]);
        assert_ne!(finals[0], finals[2]);
        assert_ne!(finals[1], finals[2]);
    }
}

#[test]
fn exact_initial_partial_and_eight_plausible_wrong_vectors_have_specific_failure_rows() {
    let r = review();
    let mut wrong_count = 0;
    for (s, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let initial = rows(&s.initial, s);
        assert_eq!(initial, bits(&r["initial_expected"]));
        assert!(initial.contains(&true) && initial.contains(&false));
        for v in std::iter::once(&r["partial"]).chain(r["wrong"].as_array().unwrap()) {
            let mut state = s.initial.clone();
            commands(&mut state, s, &v["commands"]);
            let checks = rows(&state, s);
            assert_eq!(checks, bits(&v["expected"]));
            assert!(checks.contains(&true) && checks.contains(&false));
            assert!(!engine::solved(&state, &s.goal));
        }
        wrong_count += r["wrong"].as_array().unwrap().len();
    }
    assert_eq!(wrong_count, 8);
}

#[test]
fn return_checkpoint_sources_are_real_settled_history_not_priority_resorting() {
    let source = &labs()[1];
    let d = definition(source);
    for (p, failed, incumbent) in [
        (&d.policy, core::RouterId::A, core::RouterId::B),
        (&d.lesson.policy, core::RouterId::B, core::RouterId::A),
    ] {
        let w = &p.initial;
        let mut replay = core::State::new(
            w.group(),
            w.prefix(),
            w.virtual_ip(),
            core::RouterId::ALL.map(|id| w.router(id).clone()),
            w.clients().to_vec(),
        )
        .unwrap();
        assert_eq!(replay.active(), Some(failed));
        replay
            .fault(core::FaultEvent::RouterPower {
                router: failed,
                up: false,
            })
            .unwrap();
        replay
            .fault(core::FaultEvent::RouterPower {
                router: failed,
                up: true,
            })
            .unwrap();
        assert_eq!(replay, *w);
        assert_eq!(w.revision(), 2);
        assert_eq!(w.active(), Some(incumbent));
    }
    let a = session(&source.initial);
    assert!(a.history.is_empty());
    let raw = a.to_json(d).unwrap();
    assert_eq!(work::Session::from_json(&raw, d).unwrap(), *a);
}

#[test]
fn separate_practice_insert_is_inert_then_apply_experiment_repair_reset_preserve_assignment() {
    let r = review();
    for (source, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let d = definition(source);
        let mut a = work::Session::new(d).unwrap();
        let assignment = a.current.clone();
        let rows = |a: &work::Session| {
            d.practice_checks(a)
                .into_iter()
                .map(|r| r.0)
                .collect::<Vec<_>>()
        };
        assert_eq!(rows(&a), bits(&r["practice"]["initial_expected"]));
        assert!(!a.checked && !a.example_checked);
        let source_addresses = [
            d.policy.initial.virtual_ip(),
            d.policy.initial.router(core::RouterId::A).address,
            d.policy.initial.router(core::RouterId::B).address,
        ]
        .into_iter()
        .chain(d.policy.initial.clients().iter().map(|c| c.address))
        .collect::<std::collections::BTreeSet<_>>();
        for address in [
            a.example.virtual_ip(),
            a.example.router(core::RouterId::A).address,
            a.example.router(core::RouterId::B).address,
        ]
        .into_iter()
        .chain(a.example.clients().iter().map(|c| c.address))
        {
            assert!(!source_addresses.contains(&address));
        }
        for i in 0..d.lesson.worked.len() {
            let before = a.example.clone();
            let history = a.example_history.clone();
            a.insert_example(d, i).unwrap();
            assert_eq!(a.example, before);
            assert_eq!(a.example_history, history);
            assert!(!rows(&a)[0]);
            a.apply_draft(d, true).unwrap();
            assert_eq!(a.current, assignment);
            assert!(a.history.is_empty());
        }
        assert_eq!(rows(&a), bits(&r["practice"]["worked_expected"]));
        assert!(rows(&a).into_iter().all(|b| b));
        a.apply_action(d, &d.lesson.experiment, true).unwrap();
        assert_eq!(rows(&a), bits(&r["practice"]["experiment_expected"]));
        assert!(rows(&a).contains(&false));
        a.apply_action(d, &d.lesson.repair, true).unwrap();
        assert_eq!(rows(&a), bits(&r["practice"]["repair_expected"]));
        let raw = a.to_json(d).unwrap();
        assert_eq!(work::Session::from_json(&raw, d).unwrap(), a);
        a.reset_example(d).unwrap();
        assert_eq!(a.example, d.lesson.policy.initial);
        assert!(a.example_history.is_empty());
        assert_eq!(a.current, assignment);
        assert!(a.history.is_empty());
    }
}

#[test]
fn post_success_damage_invalidates_current_credit_then_repair_recovers_without_erasing_history() {
    let r = review();
    for (source, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let mut state = solved(source);
        let count = session(&state).history.len();
        engine::execute_for_goal(
            &mut state,
            &source.goal,
            r["damage"]["command"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(rows(&state, source), bits(&r["damage"]["expected"]));
        assert!(!engine::solved(&state, &source.goal));
        assert_eq!(session(&state).history.len(), count + 1);
        engine::execute_for_goal(
            &mut state,
            &source.goal,
            r["damage"]["repair_command"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(rows(&state, source), bits(&r["damage"]["repair_expected"]));
        assert!(engine::solved(&state, &source.goal));
        assert_eq!(session(&state).history.len(), count + 2);
    }
}

#[test]
fn protected_handover_settings_and_transient_faults_cannot_change_persistent_authority() {
    let source = &labs()[3];
    let d = definition(source);
    let mut a = session(&solved(source)).clone();
    let before = a.clone();
    let mut config = a.current.router(core::RouterId::A).config.clone();
    config.participating = false;
    for action in [
        work::Action::Router {
            router: core::RouterId::A,
            config,
        },
        work::Action::Gateway {
            client: 2,
            gateway: a.current.router(core::RouterId::B).address,
        },
    ] {
        assert!(a.apply_action(d, &action, false).is_err());
        assert_eq!(a, before);
    }
    let observed = a
        .inspect(
            d,
            &[core::FaultEvent::WanLink {
                router: core::RouterId::B,
                up: false,
            }],
            false,
        )
        .unwrap();
    assert_eq!(observed.active(), Some(core::RouterId::A));
    assert_eq!(a, before);
    let mut fake = a.clone();
    fake.current = observed;
    assert!(d.bound(&fake).is_err());
    assert_eq!(a, before);
}

#[test]
fn copied_example_values_fake_badges_and_pending_drafts_never_substitute_for_current_outcomes() {
    for s in labs() {
        let d = definition(&s);
        let mut state = s.initial.clone();
        // Every practice gateway is foreign to its assignment; authorization does
        // not convert an admitted but wrong address into a passing virtual gateway.
        for w in &d.lesson.worked {
            if let work::Action::Gateway { client, .. } = &w.action {
                let before = value(&state);
                let result = session_mut(&mut state).apply_action(d, &w.action, false);
                if d.policy.editable_clients.contains(client) {
                    result.unwrap();
                } else {
                    assert!(result.is_err());
                    assert_eq!(value(&state), before);
                }
            }
        }
        session_mut(&mut state).checked = true;
        assert!(!engine::solved(&state, &s.goal));
        let mut a = Attempt::new(&s, 1);
        a.state = state;
        a.step = s.steps.len() - 1;
        a.hints = s.hints.len();
        a.challenge = true;
        workspace::prepare(&mut a);
        assert!(!progress::current_solved(&a, &s));
        a.state = solved(&s);
        let mut draft = session(&a.state).draft.clone();
        draft.routers[0].priority.clear();
        session_mut(&mut a.state)
            .set_draft(d, draft, false)
            .unwrap();
        assert!(!rows(&a.state, &s)[0]);
        assert!(!progress::current_solved(&a, &s));
    }
}

#[test]
fn save_reopen_selected_reset_and_portable_restore_keep_all_other_work_and_real_receipts() {
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let all = combined();
    let source = &all[190];
    let mut p = profile(&all);
    p.attempts.get_mut(&source.id).unwrap().state = solved(source);
    p.favorites.insert(all[193].id.clone());
    let d = definition(source);
    let a = session_mut(&mut p.attempts.get_mut(&source.id).unwrap().state);
    let mut draft = a.example_draft.clone();
    draft.routers[1].priority.clear();
    a.set_draft(d, draft, true).unwrap();
    let peer = &all[193];
    let a = session_mut(&mut p.attempts.get_mut(&peer.id).unwrap().state);
    let mut incomplete = a.draft.clone();
    incomplete.routers[1].priority.clear();
    a.set_draft(definition(peer), incomplete, false).unwrap();
    let caller = value(&p);
    assert!(p.migration_receipts.is_none());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(value(&p), caller);
    assert!(p.migration_receipts.is_none());
    let raw = std::fs::read(&path).unwrap();
    let mut reopened = progress::load(&path, &all).unwrap();
    assert!(reopened.migration_receipts.is_some());
    assert_eq!(value(&reopened), caller); // receipts are not serialized as learner data
    assert!(
        session(&reopened.attempts[&source.id].state)
            .example_draft
            .routers[1]
            .priority
            .is_empty()
    );
    assert!(
        session(&reopened.attempts[&peer.id].state).draft.routers[1]
            .priority
            .is_empty()
    );
    let receipt = value(&reopened.migration_receipts);
    let old = value(
        &reopened
            .attempts
            .iter()
            .filter(|(id, _)| *id != &source.id)
            .collect::<Vec<_>>(),
    );
    let favorites = reopened.favorites.clone();
    let portable = dir.path().join("portable.json");
    backup::export(&portable, &reopened, &all).unwrap();
    let candidate = backup::inspect(
        &portable,
        &all,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert!(progress::current_solved(
        &candidate.progress.attempts[&source.id],
        source
    ));
    reopened
        .attempts
        .insert(source.id.clone(), Attempt::new(source, p.clock + 1));
    assert_eq!(
        value(
            &reopened
                .attempts
                .iter()
                .filter(|(id, _)| *id != &source.id)
                .collect::<Vec<_>>()
        ),
        old
    );
    assert_eq!(reopened.favorites, favorites);
    assert_eq!(value(&reopened.migration_receipts), receipt);
    progress::save_with_catalog(&path, &reopened, &all).unwrap();
    assert!(!progress::current_solved(
        &reopened.attempts[&source.id],
        source
    ));
    let reset_raw = std::fs::read(&path).unwrap();
    let restored = backup::restore(&path, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        reset_raw
    );
    assert!(progress::current_solved(
        &progress::load(&path, &all).unwrap().attempts[&source.id],
        source
    ));
    assert_ne!(std::fs::read(&path).unwrap(), reset_raw);
    assert!(!raw.is_empty());
}

#[test]
fn inactive_missing_foreign_and_jointly_forged_sources_refuse_save_export_and_restore_atomically() {
    let all = combined();
    let good = profile(&all);
    let peer = &all[193]; // nonselected peer must be checked too
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save_with_catalog(&path, &good, &all).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let receipt = value(&progress::load(&path, &all).unwrap().migration_receipts);
    for mode in 0..5 {
        let mut bad: Progress = serde_json::from_value(value(&good)).unwrap();
        let mut sources = all.clone();
        match mode {
            0 => sources.retain(|s| s.id != peer.id),
            1 => bad.attempts.get_mut(&peer.id).unwrap().state = all[0].initial.clone(),
            2 => {
                let mut foreign = all[190].clone();
                foreign.id = peer.id.clone();
                *sources.last_mut().unwrap() = foreign;
            }
            3 => {
                let a = session_mut(&mut bad.attempts.get_mut(&peer.id).unwrap().state);
                let mut v = value(a);
                v["initial"]["group"] = json!(45);
                v["current"]["group"] = json!(45);
                *a = serde_json::from_value(v).unwrap();
                a.validate().unwrap(); // internally consistent still is not source authority
            }
            _ => {
                let a = session_mut(&mut bad.attempts.get_mut(&peer.id).unwrap().state);
                let observed = a
                    .inspect(
                        definition(peer),
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
                a.current = observed;
            }
        }
        let caller = value(&bad);
        assert!(progress::save_with_catalog(&path, &bad, &sources).is_err());
        assert!(progress::save(&path, &bad).is_err());
        let portable = dir.path().join(format!("refused-{mode}.json"));
        assert!(backup::export(&portable, &bad, &sources).is_err());
        assert!(!portable.exists());
        let candidate = backup::Candidate {
            progress: bad,
            rechecked: 0,
        };
        assert!(backup::restore(&path, &candidate, &sources).is_err());
        assert_eq!(value(&candidate.progress), caller);
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        assert_eq!(
            value(&progress::load(&path, &all).unwrap().migration_receipts),
            receipt
        );
    }
    let mut smuggled = value(&good);
    smuggled["catalog"] = json!(all);
    assert!(progress::decode(&serde_json::to_vec(&smuggled).unwrap(), &all).is_err());
    let mut smuggled = value(&good);
    smuggled["goal"] = value(&peer.goal);
    assert!(progress::decode(&serde_json::to_vec(&smuggled).unwrap(), &all).is_err());
}

#[test]
fn invalid_pack_goal_practice_return_coverage_unknown_fields_and_raw_limits_are_refused() {
    let source = &labs()[0];
    for mode in 0..6 {
        let mut bad = value(source);
        match mode {
            0 => bad["goal"]["definition"]["policy"]["probes"][0]["events"]
                .as_array_mut()
                .unwrap()
                .truncate(1),
            1 => {
                bad["goal"]["definition"]["lesson"]["policy"]["initial"] =
                    bad["goal"]["definition"]["policy"]["initial"].clone()
            }
            2 => bad["initial"]["simulation"]["view"] = json!("Introduction"),
            3 => bad["goal"]["definition"]["lesson"]["worked"] = json!([]),
            4 => bad["prerequisites"] = json!(["missing-hsrp-prerequisite"]),
            _ => {
                bad["goal"]["definition"]["policy"]["initial"]["routers"][0]["facts"]["wan_link_up"] =
                    json!(false)
            }
        }
        let mut catalog = value(&baseline()).as_array().unwrap().clone();
        let mut proposed = value(&labs()).as_array().unwrap().clone();
        proposed[0] = bad;
        catalog.extend(proposed);
        let chunks = catalog
            .chunks(16)
            .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
            .collect::<Vec<_>>();
        assert!(chunks.iter().all(|c| c.len() <= pack::MAX_PACK_BYTES));
        assert!(
            pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).is_err()
        );
    }
    let d = definition(source);
    assert!(work::Session::from_json(&vec![b' '; work::MAX_SESSION + 1], d).is_err());
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_DEFINITION + 1]).is_err());
    let mut unknown = value(session(&source.initial));
    unknown["shortcut_pass"] = json!(true);
    assert!(work::Session::from_json(&serde_json::to_vec(&unknown).unwrap(), d).is_err());
    let mut a = session(&solved(source)).clone();
    let before = a.clone();
    assert!(a.execute(d, "standby 11 priority 255", false).is_err());
    assert_eq!(a, before);
    assert!(a.execute(d, &"x".repeat(1025), false).is_err());
    assert_eq!(a, before);
}

#[test]
fn current193_and_fifty_one_units_remain_exact_in_full197_private_content_exercise() {
    let current = current193();
    assert_eq!(current.len(), 193);
    let original190 = baseline();
    assert_eq!(
        value(registration::ere_registration::before_scenarios(&current)),
        value(&original190)
    );
    let mut all = current.clone();
    all.extend(labs());
    let chunks = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let all = pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(all.len(), 197);
    assert_eq!(value(&all[..193]), value(&current));
    assert_eq!(value(&all[193..]), value(&labs()));
    let raw51 =
        disk_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let units = course::load(std::str::from_utf8(&raw51).unwrap(), &current).unwrap();
    assert_eq!(units.len(), 51);
    let old_typed = registration::ere_registration::before_typed_units(&units);
    assert_eq!(old_typed.len(), 50);
    let before = value(&units);
    let mut proposed = units;
    proposed.push(serde_json::from_value(review()["unit_proposal"].clone()).unwrap());
    let checked = course::load(&json!({"version":1,"units":proposed}).to_string(), &all).unwrap();
    assert_eq!(checked.len(), 52);
    assert_eq!(value(&checked[..51]), before);
    let mut p = profile(&all);
    for source in &all[193..] {
        let a = p.attempts.get_mut(&source.id).unwrap();
        assert!(!progress::current_solved(a, source));
        for action in &source.solution {
            engine::execute_for_goal(&mut a.state, &source.goal, action).unwrap();
        }
        assert!(progress::current_solved(a, source));
    }
    for source in &current {
        assert_eq!(value(&p.attempts[&source.id].state), value(&source.initial));
        assert!(!progress::current_solved(&p.attempts[&source.id], source));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("current-profile.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let saved = progress::load(&path, &all).unwrap();
    assert_eq!(value(&saved.attempts), value(&p.attempts));
    assert_eq!(value(&current193()), value(&current));

    // Keep all preceding exact193/51→190/50 and private197 proofs above. Also
    // exercise the actual fullcurrent197 with all four private HSRP sources;
    // this201/53 fixture is unregistered and grants no curriculum credit.
    let normal199 = pack::bundled().unwrap();
    assert_eq!(normal199.len(), 199);
    let current197 = disk_registration::literal_registration::before_scenarios(&normal199).to_vec();
    assert_eq!(current197.len(), 197);
    assert_eq!(
        value(disk_registration::before_scenarios(&current197)),
        value(&current)
    );
    let mut full = current197.clone();
    full.extend(labs());
    let chunks = full
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    assert!(chunks.iter().all(|c| c.len() <= pack::MAX_PACK_BYTES));
    let full = pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(full.len(), 201);
    assert_eq!(value(&full[..197]), value(&current197));
    assert_eq!(value(&full[197..]), value(&labs()));
    let units53 = course::bundled(&normal199).unwrap();
    assert_eq!(units53.len(), 53);
    let units52 = disk_registration::literal_registration::before_typed_units(&units53).to_vec();
    assert_eq!(units52.len(), 52);
    assert_eq!(
        value(disk_registration::before_typed_units(&units52)),
        before
    );
    let before52 = value(&units52);
    let mut full_units = units52;
    full_units.push(serde_json::from_value(review()["unit_proposal"].clone()).unwrap());
    let checked53 =
        course::load(&json!({"version":1,"units":full_units}).to_string(), &full).unwrap();
    assert_eq!(checked53.len(), 53);
    assert_eq!(value(&checked53[..52]), before52);
    assert_eq!(
        checked53[52].labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let mut full_profile = profile(&full);
    for source in &full[197..] {
        let a = full_profile.attempts.get_mut(&source.id).unwrap();
        assert!(!progress::current_solved(a, source));
        for action in &source.solution {
            engine::execute_for_goal(&mut a.state, &source.goal, action).unwrap();
        }
        assert!(progress::current_solved(a, source));
    }
    for source in &current197 {
        assert_eq!(
            value(&full_profile.attempts[&source.id].state),
            value(&source.initial)
        );
        assert!(!progress::current_solved(
            &full_profile.attempts[&source.id],
            source
        ));
    }
    let full_path = dir.path().join("full-current197-profile.json");
    let full_before = value(&full_profile);
    let receipts = value(&full_profile.migration_receipts);
    progress::save_with_catalog(&full_path, &full_profile, &full).unwrap();
    assert_eq!(value(&full_profile), full_before);
    assert_eq!(value(&full_profile.migration_receipts), receipts);
    let saved_full = progress::load(&full_path, &full).unwrap();
    assert_eq!(value(&saved_full), full_before);
    assert_eq!(
        value(disk_registration::literal_registration::before_scenarios(
            &pack::bundled().unwrap()
        )),
        value(&current197)
    );
    assert_eq!(value(&pack::bundled().unwrap()), value(&normal199));
    let mut actual = normal199.clone();
    actual.extend(labs());
    let chunks = actual
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let actual =
        pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(actual.len(), 203);
    assert_eq!(value(&actual[..199]), value(&normal199));
    assert_eq!(value(&actual[199..]), value(&labs()));
    let units53 = course::bundled(&normal199).unwrap();
    assert_eq!(units53.len(), 53);
    let before53 = value(&units53);
    let mut proposed54 = units53;
    proposed54.push(serde_json::from_value(review()["unit_proposal"].clone()).unwrap());
    let units54 = course::load(
        &json!({"version":1,"units":proposed54}).to_string(),
        &actual,
    )
    .unwrap();
    assert_eq!(units54.len(), 54);
    assert_eq!(value(&units54[..53]), before53);
    assert_eq!(
        units54[53].labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let mut actual_profile = profile(&actual);
    for source in &actual[199..] {
        let a = actual_profile.attempts.get_mut(&source.id).unwrap();
        for action in &source.solution {
            engine::execute_for_goal(&mut a.state, &source.goal, action).unwrap();
        }
        assert!(progress::current_solved(a, source));
    }
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
    let actual_path = dir.path().join("full-current199-profile.json");
    let before = value(&actual_profile);
    let receipts = value(&actual_profile.migration_receipts);
    progress::save_with_catalog(&actual_path, &actual_profile, &actual).unwrap();
    assert_eq!(value(&actual_profile), before);
    assert_eq!(value(&actual_profile.migration_receipts), receipts);
    assert_eq!(
        value(&progress::load(&actual_path, &actual).unwrap()),
        before
    );
}
