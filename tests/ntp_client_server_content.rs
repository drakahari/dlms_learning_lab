//! Prospective typed/source/persistence gates. Source author did not execute NTP.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    ntp, ntp_workspace as work,
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use registration::ere_registration::original190::before_ntp as original;
use registration::ere_registration::partition_fs_registration as disk_registration;
use serde_json::{Value, json};
#[path = "support/ntp_registration.rs"]
mod registration;
const DATA: &str = include_str!("../scenarios/ntp-client-server-v1.json");
const REVIEW: &str = include_str!("fixtures/ntp-client-server-review.json");
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn labs() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(DATA).unwrap().scenarios
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::Ntp { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &State) -> &work::Session {
    let State::Ntp { simulation } = s else {
        panic!()
    };
    simulation
}
fn session_mut(s: &mut State) -> &mut work::Session {
    let State::Ntp { simulation } = s else {
        panic!()
    };
    simulation
}
fn rows(state: &State, s: &Scenario) -> Vec<bool> {
    engine::checks(state, &s.goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
fn commands(state: &mut State, s: &Scenario, cs: &Value) {
    for c in cs.as_array().unwrap() {
        engine::execute_for_goal(state, &s.goal, c.as_str().unwrap()).unwrap();
    }
}
fn bits(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn combined() -> Vec<Scenario> {
    let mut all = original::load();
    assert_eq!(all.len(), 186);
    all.extend(labs());
    let packs = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let expected =
        pack::load_catalog(&packs.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let current = pack::bundled().unwrap();
    assert_eq!(disk_registration::before_scenarios(&current).len(), 193);
    assert_eq!(current.len(), 197);
    let checked190 = registration::ere_registration::before_scenarios(&current);
    assert_eq!(checked190.len(), 190);
    assert_eq!(value(&expected), value(checked190));
    expected
}
fn profile(s: &Scenario, state: State) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    a.state = state;
    workspace::prepare(&mut a);
    p.attempts.insert(s.id.clone(), a);
    p
}
#[test]
fn four_distinct_tasks_have_taught_prerequisites_separate_practice_and_exact_old_prefix() {
    let old = original::load();
    let all = combined();
    assert_eq!(value(&all[..old.len()]), value(&old));
    let r: Value = serde_json::from_str(REVIEW).unwrap();
    let raw49 = registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let mut units =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw49).unwrap(), &old).unwrap();
    let prefix = value(&units);
    units.push(serde_json::from_value(r["unit_proposal"].clone()).unwrap());
    let units =
        dlms_learning_lab::course::load(&json!({"version":1,"units":units}).to_string(), &all)
            .unwrap();
    assert_eq!(value(&units[..units.len() - 1]), prefix);
    assert_eq!(units.last().unwrap().labs.len(), 4);
    for s in &all[old.len()..] {
        let d = definition(s);
        work::source(s).unwrap();
        d.validate().unwrap();
        assert_eq!(s.prerequisites[0], "ccna-ipv4");
        assert_eq!(d.lesson.concepts.len(), 6);
        assert!(
            d.initial
                .nodes
                .keys()
                .all(|ip| !d.lesson.fixture.nodes.contains_key(ip))
        );
        assert!(rows(&s.initial, s).contains(&false));
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(s.simplifications.contains("no real UTC"));
        let a = Attempt::new(s, 1);
        work::context(&a, s).unwrap();
    }
}
#[test]
fn all_references_and_two_distinct_allowed_alternatives_satisfy_every_current_node() {
    let r: Value = serde_json::from_str(REVIEW).unwrap();
    for (s, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let mut finals = Vec::new();
        for cs in std::iter::once(&r["reference"]).chain(r["alternatives"].as_array().unwrap()) {
            let mut state = s.initial.clone();
            commands(&mut state, s, cs);
            assert!(engine::solved(&state, &s.goal));
            let d = definition(s);
            let now = &session(&state).current;
            assert!(ntp::environment_matches(now, &d.initial).unwrap());
            for t in &d.targets {
                assert!(now.status(t.client).unwrap().synchronized);
            }
            for ip in &d.protected {
                assert_eq!(now.nodes[ip].role, d.initial.nodes[ip].role);
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
fn exact_initial_partial_wrong_and_report_only_vectors_are_nonvacuous() {
    let r: Value = serde_json::from_str(REVIEW).unwrap();
    for (s, r) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        assert_eq!(rows(&s.initial, s), bits(&r["initial_expected"]));
        let mut state = s.initial.clone();
        commands(&mut state, s, &r["partial"]["commands"]);
        let actual = rows(&state, s);
        assert_eq!(actual, bits(&r["partial"]["expected"]));
        assert!(actual.contains(&false) && actual.contains(&true));
        let mut state = s.initial.clone();
        commands(&mut state, s, &r["report_only"]);
        assert_eq!(rows(&state, s), bits(&r["initial_expected"]));
        assert_eq!(session(&state).current, definition(s).initial);
        let mut state = s.initial.clone();
        for c in r["wrong"]["commands"].as_array().unwrap() {
            let before = value(&state);
            let result = engine::execute_for_goal(&mut state, &s.goal, c.as_str().unwrap());
            if c.as_str().unwrap().starts_with("SIM select ") {
                result.unwrap();
            } else if r["wrong"]["expect_error"] == true {
                assert!(result.is_err());
                assert_eq!(value(&state), before);
            } else {
                result.unwrap();
            }
        }
        assert_eq!(rows(&state, s), bits(&r["wrong"]["expected"]));
    }
}
#[test]
fn worked_insert_is_inert_and_practice_dependency_break_repair_is_separate_and_persistable() {
    for s in labs() {
        let d = definition(&s);
        let mut a = session(&s.initial).clone();
        let initial = a.current.clone();
        assert!(!a.example_checked);
        assert!(a.example_transcript.is_empty());
        assert_eq!(
            d.practice_checks(&a)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            vec![false, false, true]
        );
        for (i, c) in d.lesson.worked.iter().enumerate() {
            let before = a.example.clone();
            let output = a.example_transcript.clone();
            a.insert(d, i).unwrap();
            assert_eq!(a.example, before);
            assert_eq!(a.example_transcript, output);
            assert_eq!(a.example_selected, c.device);
            assert_eq!(a.example_command, c.text);
            a.execute(d, &c.text, true).unwrap();
            assert_eq!(a.current, initial);
        }
        assert_eq!(
            d.practice_checks(&a)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            vec![true, true, true]
        );
        let c = &d.lesson.experiment;
        a.select_device(d, c.device, true).unwrap();
        a.execute(d, &c.text, true).unwrap();
        assert_eq!(
            d.practice_checks(&a)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            vec![false, false, true]
        );
        let c = &d.lesson.repair;
        a.select_device(d, c.device, true).unwrap();
        a.execute(d, &c.text, true).unwrap();
        assert_eq!(
            d.practice_checks(&a)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            vec![true, true, true]
        );
        let restored: work::Session = serde_json::from_value(value(&a)).unwrap();
        d.bound(&restored).unwrap();
        assert_eq!(restored, a);
        a.reset_example(d).unwrap();
        assert_eq!(a.example, d.lesson.fixture);
        assert_eq!(a.current, initial);
        assert!(a.example_transcript.is_empty());
    }
}
#[test]
fn configured_cycle_reply_failure_source_policy_and_protected_peer_have_specific_feedback() {
    let ls = labs();
    let cycle = &session(&ls[1].initial).current;
    assert_eq!(
        cycle
            .status("10.72.0.1".parse().unwrap())
            .unwrap()
            .failure
            .unwrap()
            .reason,
        ntp::FailureReason::Cycle
    );
    let asymmetric = &session(&ls[2].initial).current;
    assert_eq!(
        asymmetric
            .status("10.73.0.1".parse().unwrap())
            .unwrap()
            .failure
            .unwrap()
            .reason,
        ntp::FailureReason::ReplyUnreachable
    );
    let mut state = ls[0].initial.clone();
    for c in &ls[0].solution {
        engine::execute_for_goal(&mut state, &ls[0].goal, c).unwrap();
    }
    let a = session_mut(&mut state);
    a.current
        .configure(
            "10.71.0.2".parse().unwrap(),
            Some("10.71.0.101".parse().unwrap()),
        )
        .unwrap();
    assert!(
        a.current
            .status("10.71.0.2".parse().unwrap())
            .unwrap()
            .synchronized
    );
    assert_eq!(rows(&state, &ls[0]), vec![true, false, true, true]);
    let mut state = ls[3].initial.clone();
    for c in &ls[3].solution {
        engine::execute_for_goal(&mut state, &ls[3].goal, c).unwrap();
    }
    let a = session_mut(&mut state);
    a.current
        .configure(
            "10.74.0.5".parse().unwrap(),
            Some("10.74.0.101".parse().unwrap()),
        )
        .unwrap();
    assert!(
        a.current
            .status("10.74.0.5".parse().unwrap())
            .unwrap()
            .synchronized
    );
    assert_eq!(
        rows(&state, &ls[3]),
        vec![true, true, true, true, true, true, false]
    );
}
#[test]
fn current_configuration_damage_invalidates_credit_and_explicit_repair_or_selected_reset_recovers()
{
    for s in labs() {
        let d = definition(&s);
        let mut state = s.initial.clone();
        for c in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let target = d.targets[0].client;
        let ntp::Role::Device {
            upstream: Some(upstream),
        } = session(&state).current.nodes[&target].role
        else {
            panic!()
        };
        let a = session_mut(&mut state);
        a.select_device(d, target, false).unwrap();
        a.execute(d, &format!("no ntp server {upstream}"), false)
            .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        let p = profile(&s, state.clone());
        assert!(!progress::current_solved(&p.attempts[&s.id], &s));
        session_mut(&mut state)
            .execute(d, &format!("ntp server {upstream}"), false)
            .unwrap();
        assert!(engine::solved(&state, &s.goal));
        let mut p = profile(&s, state);
        p.attempts.insert(s.id.clone(), Attempt::new(&s, 2));
        assert_eq!(p.attempts[&s.id].state, s.initial);
        assert!(!progress::current_solved(&p.attempts[&s.id], &s));
    }
}
#[test]
fn environment_health_delivery_and_foreign_source_or_progress_metadata_are_refused_atomically() {
    let all = combined();
    let s = labs().remove(0);
    let d = definition(&s);
    let original = profile(&s, s.initial.clone());
    for field in ["health", "online", "label", "delivery", "foreign"] {
        let mut p: Progress = serde_json::from_value(value(&original)).unwrap();
        let a = p.attempts.get_mut(&s.id).unwrap();
        let session = session_mut(&mut a.state);
        match field {
            "health" => {
                session
                    .current
                    .nodes
                    .get_mut(&"10.71.0.101".parse().unwrap())
                    .unwrap()
                    .role = ntp::Role::Reference {
                    stratum: 1,
                    healthy: false,
                };
            }
            "online" => {
                session
                    .current
                    .nodes
                    .get_mut(&"10.71.0.1".parse().unwrap())
                    .unwrap()
                    .online = false
            }
            "label" => {
                session
                    .current
                    .nodes
                    .get_mut(&"10.71.0.1".parse().unwrap())
                    .unwrap()
                    .label = "Changed".into()
            }
            "delivery" => session.current.deliveries[0].reply_delivered = false,
            _ => {
                a.workspace.as_mut().unwrap().command = "foreign old terminal".into();
            }
        }
        assert!(work::context(&p.attempts[&s.id], &s).is_err());
        assert!(!progress::current_solved(&p.attempts[&s.id], &s));
        assert!(progress::decode(value(&p).to_string().as_bytes(), &all).is_err());
    }
    let mut a = session(&s.initial).clone();
    let before = value(&a);
    assert!(a.set_command(d, "x".repeat(513), false).is_err());
    assert_eq!(value(&a), before);
    assert!(a.execute(d, "ntp master 1", false).is_err());
    assert_eq!(value(&a), before);
    assert!(a.execute(d, "SIM select 203.0.113.5", false).is_err());
    assert_eq!(value(&a), before);
    let mut foreign = s.clone();
    foreign.teaching = pack::bundled()
        .unwrap()
        .into_iter()
        .find_map(|s| s.teaching);
    assert!(foreign.teaching.is_some());
    assert!(work::source(&foreign).is_err());
}
#[test]
fn save_reopen_and_selected_reset_keep_exact_other_attempts_favorites_receipts_and_existing_files()
{
    let all = combined();
    let labs = labs();
    let s = &labs[0];
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    let mut p = profile(s, state);
    let peer = &labs[1];
    p.attempts.insert(peer.id.clone(), Attempt::new(peer, 2));
    p.favorites.insert(peer.id.clone());
    let old = pack::bundled().unwrap().remove(0);
    p.attempts.insert(old.id.clone(), Attempt::new(&old, 3));
    let d = definition(s);
    let a = session_mut(&mut p.attempts.get_mut(&s.id).unwrap().state);
    a.set_command(d, "show ntp status".into(), true).unwrap();
    let other = value(&p.attempts[&peer.id]);
    let historical = value(&p.attempts[&old.id]);
    let favorites = value(&p.favorites);
    let dir = std::env::temp_dir().join(format!("ntp-unit-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("profile.json");
    let raw = value(&p).to_string();
    std::fs::write(&path, &raw).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), raw);
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let mut reopened = progress::decode(&saved, &all).unwrap();
    assert_eq!(value(&reopened), value(&p));
    // Observe real main-save provenance; never equate fresh None with the decoded receipt.
    assert!(reopened.migration_receipts.is_some());
    let receipts = value(&reopened.migration_receipts);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    assert!(progress::current_solved(&reopened.attempts[&s.id], s));
    reopened.attempts.insert(s.id.clone(), Attempt::new(s, 4));
    assert_eq!(reopened.attempts[&s.id].state, s.initial);
    assert_eq!(value(&reopened.attempts[&peer.id]), other);
    assert_eq!(value(&reopened.attempts[&old.id]), historical);
    assert_eq!(value(&reopened.favorites), favorites);
    assert_eq!(value(&reopened.migration_receipts), receipts);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    progress::save_with_catalog(&path, &reopened, &all).unwrap();
    let reset = progress::decode(&std::fs::read(&path).unwrap(), &all).unwrap();
    assert_eq!(reset.attempts[&s.id].state, s.initial);
    assert_eq!(value(&reset.attempts[&peer.id]), other);
    assert_eq!(value(&reset.attempts[&old.id]), historical);
    assert_eq!(value(&reset.migration_receipts), receipts);
}
#[test]
fn invalid_packs_changed_goals_fake_reports_and_bounded_collections_never_grant_credit() {
    let mut s = labs().remove(0);
    let mut p = profile(&s, s.initial.clone());
    let a = session_mut(&mut p.attempts.get_mut(&s.id).unwrap().state);
    a.checked = true;
    a.transcript.push(work::Row {
        device: a.selected,
        input: "show ntp status".into(),
        output: "synchronized / all checks pass".into(),
    });
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
    let Goal::Ntp { definition: d } = &mut s.goal else {
        panic!()
    };
    d.initial.deliveries[0].reply_delivered = false;
    assert!(work::source(&s).is_err());
    let mut all = combined();
    let index = all.iter().position(|v| v.id == s.id).unwrap();
    all[index] = s;
    let packs = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    assert!(pack::load_catalog(&packs.iter().map(String::as_str).collect::<Vec<_>>()).is_err());
    let s = labs().remove(0);
    let mut raw = value(&s.initial);
    raw["simulation"]["transcript"] =
        json!([{"device":"10.71.0.1","input":"show ntp status","output":"x"}]);
    raw["simulation"]["command"] = Value::String("x".repeat(513));
    assert!(serde_json::from_value::<State>(raw).is_err());
    let mut raw = value(&s.initial);
    raw["simulation"]["transcript"] = Value::Array(
        (0..33)
            .map(|_| json!({"device":"10.71.0.1","input":"show ntp status","output":"x"}))
            .collect(),
    );
    assert!(serde_json::from_value::<State>(raw).is_err());
    assert!(pack::load_catalog(&[&" ".repeat(2_000_001)]).is_err());
    assert!(progress::decode(&vec![b' '; 16_000_001], &combined()).is_err());
}

#[test]
fn source_aware_save_rejects_jointly_forged_environment_markers_before_file_or_receipt_changes() {
    let all = combined();
    let s = labs().remove(0);
    let mut p = profile(&s, s.initial.clone());
    let dir = std::env::temp_dir().join(format!("ntp-source-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("profile.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let caller = value(&p);
    let receipt = value(&p.migration_receipts);
    let a = session_mut(&mut p.attempts.get_mut(&s.id).unwrap().state);
    for world in [&mut a.initial, &mut a.current] {
        world
            .nodes
            .get_mut(&"10.71.0.101".parse().unwrap())
            .unwrap()
            .role = ntp::Role::Reference {
            stratum: 1,
            healthy: false,
        };
        world.deliveries[0].reply_delivered = false;
    }
    // A locally self-consistent forged baseline is not the authored source.
    a.validate().unwrap();
    assert!(definition(&s).bound(a).is_err());
    assert!(progress::save_with_catalog(&path, &p, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(value(&p.migration_receipts), receipt);
    assert!(progress::decode(value(&p).to_string().as_bytes(), &all).is_err());
    assert_ne!(value(&p), caller); // The test's explicit damage remains visible; no normalization.
}

#[test]
fn copied_practice_addresses_never_resolve_in_assignment_and_excess_row_body_is_not_read() {
    for s in labs() {
        let d = definition(&s);
        let mut a = session(&s.initial).clone();
        let reference = d
            .lesson
            .fixture
            .nodes
            .iter()
            .find(|(_, n)| matches!(n.role, ntp::Role::Reference { healthy: true, .. }))
            .unwrap()
            .0;
        for t in &d.targets {
            a.current.configure(t.client, Some(*reference)).unwrap();
        }
        assert!(d.checks(&a).iter().any(|r| !r.0));
        for t in &d.targets {
            assert_eq!(
                a.current.status(t.client).unwrap().failure.unwrap().reason,
                ntp::FailureReason::UnknownServer
            );
        }
    }
    let s = labs().remove(0);
    let raw = serde_json::to_string(session(&s.initial)).unwrap();
    let row = "{\"device\":\"10.71.0.1\",\"input\":\"show ntp status\",\"output\":\"display\"}";
    let replacement = format!(
        "\"transcript\":[{},POISON_MUST_NOT_BE_READ]",
        std::iter::repeat_n(row, 32).collect::<Vec<_>>().join(",")
    );
    let raw = raw.replacen("\"transcript\":[]", &replacement, 1);
    let error = serde_json::from_str::<work::Session>(&raw).unwrap_err();
    assert!(error.to_string().contains("item limit"));
}

#[test]
fn all_attempt_source_authority_including_nonselected_missing_foreign_or_wrong_kind_is_mandatory_on_save()
 {
    let all = combined();
    let ls = labs();
    let s = &ls[0];
    let peer = &ls[1];
    let dir = std::env::temp_dir().join(format!("ntp-all-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("profile.json");
    let mut good = profile(s, s.initial.clone());
    good.attempts.insert(peer.id.clone(), Attempt::new(peer, 2));
    progress::save_with_catalog(&path, &good, &all).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for mode in 0..4 {
        let mut p: Progress = serde_json::from_value(value(&good)).unwrap();
        let mut catalog = all.clone();
        match mode {
            0 => {
                let a = session_mut(&mut p.attempts.get_mut(&peer.id).unwrap().state);
                for world in [&mut a.initial, &mut a.current] {
                    world
                        .nodes
                        .get_mut(&"10.72.0.104".parse().unwrap())
                        .unwrap()
                        .role = ntp::Role::Reference {
                        stratum: 2,
                        healthy: true,
                    };
                }
                a.validate().unwrap();
            }
            1 => catalog.retain(|v| v.id != peer.id),
            2 => {
                let index = catalog.iter().position(|v| v.id == peer.id).unwrap();
                let mut other = ls[2].clone();
                other.id = peer.id.clone();
                catalog[index] = other;
            }
            _ => {
                p.attempts.get_mut(&peer.id).unwrap().state =
                    pack::bundled().unwrap()[0].initial.clone()
            }
        }
        let caller = value(&p);
        let receipt = value(&p.migration_receipts);
        assert!(progress::save_with_catalog(&path, &p, &catalog).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(value(&p), caller);
        assert_eq!(value(&p.migration_receipts), receipt);
    }
}

#[test]
fn portable_export_inspect_restore_and_summary_use_the_owned_catalog_not_saved_world_markers() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    let all = combined();
    let s = labs().remove(0);
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    let p = profile(&s, state);
    let dir = std::env::temp_dir().join(format!("ntp-backup-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main = dir.join("profile.json");
    let portable = dir.join("portable.json");
    let rejected = dir.join("refused.json");
    let untrusted = dir.join("untrusted.json");
    progress::save_with_catalog(&main, &p, &all).unwrap();
    let original = std::fs::read(&main).unwrap();
    let loaded = progress::decode(&original, &all).unwrap();
    assert!(loaded.migration_receipts.is_some());
    backup::export(&portable, &loaded, &all).unwrap();
    let checked = AtomicUsize::new(0);
    let cancel = AtomicBool::new(false);
    let candidate = backup::inspect(&portable, &all, &cancel, &checked).unwrap();
    assert_eq!(checked.load(Ordering::Relaxed), 0);
    assert_eq!(candidate.rechecked, 0);
    assert!(candidate.progress.migration_receipts.is_none()); // Portable format deliberately strips main-file provenance.
    assert!(candidate.summary(&all).contains("1 currently complete"));
    let mut forged: Progress = serde_json::from_value(value(&candidate.progress)).unwrap();
    let a = session_mut(&mut forged.attempts.get_mut(&s.id).unwrap().state);
    for world in [&mut a.initial, &mut a.current] {
        world
            .nodes
            .get_mut(&"10.71.0.104".parse().unwrap())
            .unwrap()
            .role = ntp::Role::Reference {
            stratum: 2,
            healthy: true,
        };
    }
    a.validate().unwrap();
    let caller = value(&forged);
    let receipt = value(&forged.migration_receipts);
    assert!(backup::export(&rejected, &forged, &all).is_err());
    assert!(!rejected.exists());
    std::fs::write(&untrusted, value(&forged).to_string()).unwrap();
    checked.store(0, Ordering::Relaxed);
    assert!(backup::inspect(&untrusted, &all, &cancel, &checked).is_err());
    assert_eq!(checked.load(Ordering::Relaxed), 0);
    let forged = backup::Candidate {
        progress: forged,
        rechecked: 0,
    };
    assert!(forged.summary(&all).contains("0 currently complete"));
    let mut before = std::fs::read_dir(&dir)
        .unwrap()
        .map(|v| v.unwrap().path())
        .collect::<Vec<_>>();
    before.sort();
    assert!(backup::restore(&main, &forged, &all).is_err());
    assert_eq!(std::fs::read(&main).unwrap(), original);
    assert_eq!(value(&forged.progress), caller);
    assert_eq!(value(&forged.progress.migration_receipts), receipt);
    let mut after = std::fs::read_dir(&dir)
        .unwrap()
        .map(|v| v.unwrap().path())
        .collect::<Vec<_>>();
    after.sort();
    assert_eq!(after, before);
    let restored = backup::restore(&main, &candidate, &all).unwrap();
    let safety = restored.safety_copy.unwrap();
    assert_eq!(std::fs::read(safety).unwrap(), original);
    let observed = progress::decode(&std::fs::read(&main).unwrap(), &all).unwrap();
    assert!(observed.migration_receipts.is_some());
    assert!(progress::current_solved(&observed.attempts[&s.id], &s));
    // An import cannot supply or replace the application's catalog.
    let mut extra = value(&candidate.progress);
    extra["catalog"] = json!({"source":"untrusted"});
    std::fs::write(&untrusted, extra.to_string()).unwrap();
    assert!(backup::inspect(&untrusted, &all, &cancel, &checked).is_err());
}

#[test]
fn separate_cycle_example_teaches_breaks_and_repairs_the_loop_and_names_protected_upstream_damage()
{
    let s = labs().remove(1);
    let d = definition(&s);
    let l = &d.lesson;
    assert!(l.concepts[2].text.contains("A asks B while B asks A"));
    let mut a = session(&s.initial).clone();
    let assignment = a.current.clone();
    let relay = "10.82.0.11".parse().unwrap();
    let client = "10.82.0.12".parse().unwrap();
    assert_eq!(
        a.example.status(relay).unwrap().failure.unwrap().reason,
        ntp::FailureReason::Cycle
    );
    assert_eq!(
        a.example.status(client).unwrap().failure.unwrap().reason,
        ntp::FailureReason::Cycle
    );
    for (n, c) in l.worked.iter().enumerate() {
        let before = a.example.clone();
        let transcript = a.example_transcript.clone();
        a.insert(d, n).unwrap();
        assert_eq!(a.example, before);
        assert_eq!(a.example_transcript, transcript);
        a.execute(d, &c.text, true).unwrap();
        assert_eq!(a.current, assignment);
    }
    assert_eq!(
        d.practice_checks(&a)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
    let peer = "10.82.0.13".parse().unwrap();
    a.example
        .configure(peer, Some("10.82.0.102".parse().unwrap()))
        .unwrap();
    let rows = d.practice_checks(&a);
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true, true, false]
    );
    assert!(
        rows[2]
            .1
            .contains("supplied 10.82.0.101; current 10.82.0.102; currently synchronized")
    );
    a.select_device(d, peer, true).unwrap();
    a.execute(d, "no ntp server 10.82.0.102", true).unwrap();
    a.execute(d, "ntp server 10.82.0.101", true).unwrap();
    assert_eq!(
        d.practice_checks(&a)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
    a.select_device(d, l.experiment.device, true).unwrap();
    a.execute(d, &l.experiment.text, true).unwrap();
    assert!(d.practice_checks(&a).iter().any(|r| !r.0));
    a.select_device(d, l.repair.device, true).unwrap();
    a.execute(d, &l.repair.text, true).unwrap();
    assert!(d.practice_checks(&a).iter().all(|r| r.0));
    assert_eq!(a.current, assignment);
}
