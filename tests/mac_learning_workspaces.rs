//! Unbundled MAC workspace acceptance; no reference packet/host execution.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    mac_learning as mac, mac_learning_teaching as teach, mac_learning_workspace as work, pack,
    progress::{self, Progress},
    workspace,
};
use std::collections::BTreeMap;
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/mac-learning-preview.json"))
        .unwrap()
        .remove(0)
}
fn initial(s: &pack::Scenario) -> mac::State {
    let State::MacLearning { simulation } = &s.initial else {
        panic!()
    };
    simulation.clone()
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute(&mut state, input).unwrap();
    }
    state
}
fn profile(s: &pack::Scenario) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts
        .insert(s.id.clone(), progress::Attempt::new(s, 1));
    workspace::prepare(p.attempts.get_mut(&s.id).unwrap());
    teach::prepare(
        p.attempts.get_mut(&s.id).unwrap(),
        s.mac_teaching.as_ref().unwrap(),
        true,
    );
    p
}
#[test]
fn private_fixture_reachable_and_distinct_native_sends_accept_an_alternate_order() {
    let s = scenario();
    assert!(!pack::bundled().unwrap().iter().any(|x| x.id == s.id));
    assert!(!engine::solved(&s.initial, &s.goal));
    assert!(engine::solved(&solve(&s), &s.goal));
    let mut alternate = s.initial.clone();
    for action in s.solution.iter().rev() {
        engine::execute(&mut alternate, action).unwrap();
    }
    assert!(engine::solved(&alternate, &s.goal));
    let before = serde_json::to_vec(&alternate).unwrap();
    assert!(engine::checks(&alternate, &s.goal).iter().all(|c| c.0));
    assert_eq!(serde_json::to_vec(&alternate).unwrap(), before);
}
#[test]
fn checks_do_not_learn_missing_required_state_or_allow_partial_or_stopped_shortcuts() {
    let s = scenario();
    for action in &s.solution {
        let mut state = s.initial.clone();
        engine::execute(&mut state, action).unwrap();
        let c = engine::checks(&state, &s.goal);
        assert!(c.iter().any(|c| c.0) && c.iter().any(|c| !c.0));
    }
    let mut state = solve(&s);
    let before = state.clone();
    assert!(
        engine::execute(
            &mut state,
            "@mac {\"kind\":\"Send\",\"source\":\"missing\",\"destination\":\"ff:ff:ff:ff:ff:ff\"}"
        )
        .is_err()
    );
    assert_eq!(state, before);
    engine::execute(
        &mut state,
        "@mac {\"kind\":\"Settings\",\"operation\":\"port s1 uplink down\"}",
    )
    .unwrap();
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn malformed_native_drafts_are_preserved_atomic_and_pending_configuration_clears_success() {
    let s = scenario();
    let mut p = profile(&s);
    p.attempts.get_mut(&s.id).unwrap().state = solve(&s);
    let State::MacLearning { simulation } = &p.attempts[&s.id].state else {
        panic!()
    };
    let mut d = work::Draft::new(simulation).unwrap();
    assert!(progress::current_solved(&p.attempts[&s.id], &s));
    d.configuration
        .switches
        .get_mut("s1")
        .unwrap()
        .ports
        .get_mut("uplink")
        .unwrap()
        .allowed = "bad".into();
    let mut candidate = simulation.clone();
    let before = candidate.clone();
    assert!(d.apply(&mut candidate).is_err());
    assert_eq!(candidate, before);
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mac_draft = Some(d);
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
    let bytes = serde_json::to_vec(&p).unwrap();
    let loaded = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
}
#[test]
fn clear_and_expiry_freshly_invalidate_then_relearning_repairs_without_event_credit() {
    let s = scenario();
    let mut state = solve(&s);
    engine::execute(
        &mut state,
        "@mac {\"kind\":\"Clear\",\"switch\":\"s1\",\"vlan\":10}",
    )
    .unwrap();
    assert!(!engine::solved(&state, &s.goal));
    for a in &s.solution {
        engine::execute(&mut state, a).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    engine::execute(&mut state, "@mac {\"kind\":\"Advance\",\"seconds\":300}").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    for a in &s.solution {
        engine::execute(&mut state, a).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn examples_fill_and_insert_never_execute_or_touch_assignment_and_experiment_is_nontrivial() {
    let s = scenario();
    let l = s.mac_teaching.as_ref().unwrap();
    let mut p = profile(&s);
    let original = serde_json::to_value(&p.attempts[&s.id].state).unwrap();
    let mut learning = teach::Learning::new(l).unwrap();
    let before = learning.simulation.clone();
    learning.fill_example(l).unwrap();
    assert_eq!(learning.simulation, before);
    assert!(learning.simulation.entries().is_empty());
    assert!(!learning.checked);
    for a in &l.example_solution {
        let before = learning.simulation.clone();
        a.insert(&learning.simulation, &mut learning.draft).unwrap();
        assert_eq!(learning.simulation, before);
        a.apply(&mut learning.simulation).unwrap();
    }
    learning.checked = true;
    assert!(learning.checks(l).iter().all(|c| c.0));
    let before = learning.simulation.clone();
    for a in &l.try_actions {
        a.apply(&mut learning.simulation).unwrap();
    }
    assert_ne!(learning.simulation, before);
    assert!(
        work::checks(&l.fixture, &learning.simulation, &l.try_outcomes)
            .iter()
            .all(|c| c.0)
    );
    assert!(learning.checks(l).iter().any(|c| !c.0));
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mac_learning = Some(learning);
    assert_eq!(
        serde_json::to_value(&p.attempts[&s.id].state).unwrap(),
        original
    );
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
}
#[test]
fn imported_assignment_identity_and_exact_example_origin_are_pinned_with_positive_controls() {
    let s = scenario();
    let p = profile(&s);
    let good = serde_json::to_value(&p).unwrap();
    assert!(
        progress::decode(
            &serde_json::to_vec(&good).unwrap(),
            std::slice::from_ref(&s)
        )
        .is_ok()
    );
    for field in ["state", "practice"] {
        let mut v = good.clone();
        let target = if field == "state" {
            &mut v["attempts"][&s.id]["state"]["simulation"]
        } else {
            &mut v["attempts"][&s.id]["workspace"]["mac_learning"]["fixture"]
        };
        target["host_addresses"][if field == "state" { "a" } else { "x" }] =
            serde_json::json!("02:00:00:00:88:99");
        assert!(
            progress::decode(&serde_json::to_vec(&v).unwrap(), std::slice::from_ref(&s)).is_err()
        );
    }
    // Same topology/MACs but changed authored example aging must not inherit practice.
    let mut changed = s.clone();
    changed.mac_teaching.as_mut().unwrap().fixture = serde_json::from_value({
        let mut v = serde_json::to_value(&changed.mac_teaching.as_ref().unwrap().fixture).unwrap();
        v["aging_seconds"] = serde_json::json!(200);
        v
    })
    .unwrap();
    assert!(
        progress::decode(
            &serde_json::to_vec(&good).unwrap(),
            std::slice::from_ref(&changed)
        )
        .is_err()
    );
}
#[test]
fn wrong_kind_and_oversized_metadata_save_errors_retain_original_exact_bytes() {
    let s = scenario();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let p = profile(&s);
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut broken: Progress = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    broken.attempts.get_mut(&s.id).unwrap().state = pack::bundled().unwrap()[0].initial.clone();
    assert!(progress::save(&path, &broken).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut broken: Progress = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    broken
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mac_draft
        .as_mut()
        .unwrap()
        .destination = "é".repeat(100_000);
    assert!(progress::save(&path, &broken).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn older_bundled_progress_histories_favorites_and_unrelated_drafts_survive_selected_reset_and_restore()
 {
    let s = scenario();
    let old = pack::bundled().unwrap();
    assert!(old.len() >= 106);
    let catalog_len = old.len();
    let mut labs = old.clone();
    labs.push(s.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for lab in &old {
        p.attempts
            .insert(lab.id.clone(), progress::Attempt::new(lab, 1));
        workspace::prepare(p.attempts.get_mut(&lab.id).unwrap());
    }
    p.favorites.insert(old[0].id.clone());
    let before = serde_json::to_value(&p.attempts).unwrap();
    p.attempts
        .insert(s.id.clone(), progress::Attempt::new(&s, 1));
    workspace::prepare(p.attempts.get_mut(&s.id).unwrap());
    teach::prepare(
        p.attempts.get_mut(&s.id).unwrap(),
        s.mac_teaching.as_ref().unwrap(),
        true,
    );
    let old7: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    for (id, a) in &old7.attempts {
        if matches!(a.state, State::Script { .. }) {
            p.attempts.insert(id.clone(), a.clone());
        }
    }
    let old_attempts = p
        .attempts
        .iter()
        .filter(|(id, _)| *id != &s.id)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<BTreeMap<_, _>>();
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(loaded.attempts.len(), catalog_len + 1);
    for (id, a) in old_attempts {
        assert_eq!(serde_json::to_value(&loaded.attempts[&id]).unwrap(), a);
    }
    loaded
        .attempts
        .insert(s.id.clone(), progress::Attempt::new(&s, 0));
    workspace::prepare(loaded.attempts.get_mut(&s.id).unwrap());
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(before.as_object().unwrap().len(), catalog_len);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    dlms_learning_lab::backup::export(&path, &loaded, &labs).unwrap();
    let pending = dlms_learning_lab::backup::inspect(
        &path,
        &labs,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(pending.progress.attempts.len(), catalog_len + 1);
    let target = dir.path().join("restored.json");
    progress::save(&target, &loaded).unwrap();
    let target_bytes = std::fs::read(&target).unwrap();
    let result = dlms_learning_lab::backup::restore(&target, &pending, &labs).unwrap();
    assert_eq!(
        std::fs::read(result.safety_copy.unwrap()).unwrap(),
        target_bytes
    );
    let restored = progress::load(&target, &labs).unwrap();
    assert_eq!(restored.attempts[&s.id].state, loaded.attempts[&s.id].state);
}
#[test]
fn direct_public_bounds_precede_lesson_replays_and_keep_practice_unchanged() {
    let s = scenario();
    let l = s.mac_teaching.as_ref().unwrap();
    let mut learning = teach::Learning::new(l).unwrap();
    let before = serde_json::to_value(&learning).unwrap();
    let mut bad = l.clone();
    bad.example_solution = vec![work::Action::Advance { seconds: 1 }; 10000];
    assert!(bad.validate().is_err());
    assert!(learning.fill_example(&bad).is_err());
    assert_eq!(serde_json::to_value(&learning).unwrap(), before);
    let mut d = work::Draft::new(&initial(&s)).unwrap();
    d.aging = "9".repeat(100000);
    let mut state = initial(&s);
    let before = state.clone();
    assert!(d.apply(&mut state).is_err());
    assert_eq!(state, before);
    let malformed = work::Requirement::Port {
        switch: "x".repeat(100_000),
        port: "bad".into(),
        settings: initial(&s).fabric().switches["s1"].ports["a"].clone(),
    };
    let (passed, message) = malformed.check(&initial(&s));
    assert!(!passed && message.len() <= mac::MAX_ERROR_BYTES);
    let action = work::Action::Send {
        source: "x".repeat(100_000),
        destination: mac::Mac::BROADCAST,
    };
    assert!(!action.valid());
    assert!(action.label().len() <= mac::MAX_ERROR_BYTES);
}
#[test]
fn typed_goals_unknown_fields_excessive_counts_and_foreign_teaching_are_rejected() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/mac-learning-preview.json")).unwrap();
    for attack in [0, 1, 2, 3] {
        let mut bad = v.clone();
        match attack {
            0 => bad["scenarios"][0]["goal"]["initial"]["aging_seconds"] = serde_json::json!(200),
            1 => {
                bad["scenarios"][0]["goal"]["requirements"] = serde_json::json!(vec![
                    serde_json::json!({"kind":"ActiveVlans","switch":"s1","vlans":[10]});
                    65
                ])
            }
            2 => bad["scenarios"][0]["mac_teaching"]["unknown"] = true.into(),
            _ => {
                bad["scenarios"][0]["guide"] = serde_json::json!(vec![
                    serde_json::json!({"title":"Native guide","why":"Inspect the table","action":"Send a frame","commands":[{"command":"@mac fake","explanation":"Must never be taught as a terminal command","simulator_only":true}]});
                    3
                ])
            }
        }
        assert!(pack::load(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let s = scenario();
    let Goal::MacLearning {
        initial,
        requirements,
    } = s.goal
    else {
        panic!()
    };
    assert!(work::valid_requirements(&requirements, &initial));
}
