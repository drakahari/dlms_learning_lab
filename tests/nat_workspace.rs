//! Prospective bounded backend tests. No host commands or solved-state injection.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    nat::{self, Action},
    nat_workspace::{
        self as w, Definition, Draft, Expected, Global, Kind, Requirement, Step, Trial,
    },
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use std::net::Ipv4Addr;
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn scenario() -> Scenario {
    pack::load(include_str!("fixtures/nat-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn def(s: &Scenario) -> &Definition {
    let Goal::BasicNat { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(s: &State) -> &nat::State {
    let State::BasicNat { simulation } = s else {
        panic!()
    };
    simulation
}
fn sim_mut(s: &mut State) -> &mut nat::State {
    let State::BasicNat { simulation } = s else {
        panic!()
    };
    simulation
}
fn good(s: &Scenario) -> State {
    let mut current = s.initial.clone();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut current, &s.goal, cmd).unwrap();
    }
    assert!(engine::solved(&current, &s.goal));
    current
}
#[track_caller]
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    assert!(
        a == b,
        "Bounded whole-object equality failed: {}",
        evidence::difference(&a, &b).unwrap_or_default()
    );
}
fn attempt(s: &Scenario, state: State) -> Attempt {
    let mut a = Attempt::new(s, 0);
    a.state = state;
    workspace::prepare(&mut a);
    a
}
fn policy(s: &Scenario) -> nat::Configuration {
    let a = Action::from_json(s.solution[0].strip_prefix(w::PREFIX).unwrap().as_bytes()).unwrap();
    let Action::Configure { configuration } = a else {
        panic!()
    };
    configuration
}
#[test]
fn roundtrip_strict_and_no_old_wire_field() {
    let s = scenario();
    let d = def(&s);
    let raw = serde_json::to_vec(d).unwrap();
    assert_eq!(Definition::from_json(&raw).unwrap().trials, d.trials);
    let draft = Draft::new(sim(&s.initial)).unwrap();
    exact(
        &draft,
        &Draft::from_json(&serde_json::to_vec(&draft).unwrap()).unwrap(),
    );
    let old = workspace::WorkspaceData::new();
    assert!(
        serde_json::to_value(old)
            .unwrap()
            .get("nat_draft")
            .is_none()
    );
}
#[test]
fn current_mapping_never_allocates_and_breaking_it_invalidates() {
    let s = scenario();
    let mut state = s.initial.clone();
    w::apply(
        sim(&s.initial),
        sim_mut(&mut state),
        &Action::Configure {
            configuration: policy(&s),
        },
    )
    .unwrap();
    let before = state.clone();
    let report = def(&s).report(sim(&state));
    assert!(!report.rows[0].0 && !report.rows[1].0);
    assert!(report.rows.iter().rev().take(2).all(|r| r.0));
    exact(&state, &before);
    let mut state = good(&s);
    w::apply(
        sim(&s.initial),
        sim_mut(&mut state),
        &Action::ClearDynamic {},
    )
    .unwrap();
    assert!(!engine::solved(&state, &s.goal));
    assert!(sim(&state).bindings().is_empty());
}
#[test]
fn policy_only_goal_no_unasked_current_binding_and_static_shortcut_fails() {
    let s = scenario();
    let mut d = def(&s).clone();
    d.requirements
        .retain(|r| !matches!(r, Requirement::CurrentMapping { .. }));
    let mut state = sim(&s.initial).clone();
    w::apply(
        &d.initial,
        &mut state,
        &Action::Configure {
            configuration: policy(&s),
        },
    )
    .unwrap();
    assert!(d.checks(&state).iter().all(|r| r.0));
    assert!(state.bindings().is_empty());
    let mut c = policy(&s);
    c.statics.insert("alpha".into(), ip("198.51.100.13"));
    w::apply(
        &d.initial,
        &mut state,
        &Action::Configure { configuration: c },
    )
    .unwrap();
    assert!(d.checks(&state).iter().any(|r| !r.0));
}
#[test]
fn alternate_prefix_pool_and_host_orders_are_not_one_hidden_answer() {
    let s = scenario();
    let mut current = sim(&s.initial).clone();
    let mut c = policy(&s);
    c.dynamic.as_mut().unwrap().source = nat::Prefix {
        network: ip("10.0.0.10"),
        length: 31,
    };
    c.dynamic.as_mut().unwrap().pool = nat::Pool {
        first: ip("198.51.100.12"),
        last: ip("198.51.100.15"),
    };
    w::apply(
        sim(&s.initial),
        &mut current,
        &Action::Configure { configuration: c },
    )
    .unwrap();
    for h in ["beta", "alpha"] {
        w::apply(
            sim(&s.initial),
            &mut current,
            &Action::Outbound {
                inside: h.into(),
                peer: "peer-two".into(),
            },
        )
        .unwrap();
    }
    let before = current.clone();
    assert!(def(&s).checks(&current).iter().all(|r| r.0));
    exact(&current, &before);
    assert_eq!(current.bindings()["beta"], ip("198.51.100.12"));
    assert_eq!(current.bindings()["alpha"], ip("198.51.100.13"));
}
#[test]
fn static_first_reverse_and_clear_preservation_are_real_outcomes() {
    let s = scenario();
    let mut current = sim(&good(&s)).clone();
    w::apply(sim(&s.initial), &mut current, &Action::ClearDynamic {}).unwrap();
    let e = current
        .apply(&Action::Inbound {
            peer: "peer".into(),
            global: ip("198.51.100.50"),
        })
        .unwrap();
    assert!(matches!(
        e,
        nat::Effect::Probe {
            result: nat::ProbeResult::Translated {
                kind: nat::MappingKind::Static,
                ..
            }
        }
    ));
    let mut d = def(&s).clone();
    d.requirements = vec![Requirement::StaticPreserved {
        inside: "observer".into(),
    }];
    assert!(d.checks(&current).iter().all(|r| r.0));
    let mut c = current.configuration().clone();
    c.statics.remove("observer");
    w::apply(
        &d.initial,
        &mut current,
        &Action::Configure { configuration: c },
    )
    .unwrap();
    assert!(!d.current_checks(&current)[0].0);
}
#[test]
fn independently_limited_capacity_exhaustion_is_distinct_from_no_match() {
    let s = scenario();
    let mut state = sim(&s.initial).clone();
    let mut c = policy(&s);
    c.dynamic.as_mut().unwrap().pool.last = ip("198.51.100.10");
    w::apply(
        sim(&s.initial),
        &mut state,
        &Action::Configure { configuration: c },
    )
    .unwrap();
    let mut d = def(&s).clone();
    d.requirements = vec![Requirement::DynamicCapacity { minimum: 1 }];
    d.trials.truncate(1);
    d.trials[0].steps = vec![
        Step::Outbound {
            inside: "alpha".into(),
            peer: "peer".into(),
            expected: Expected::Translated {
                mapping_kind: Kind::Dynamic,
                global: Global::Any {},
            },
        },
        Step::Reverse {
            inside: "alpha".into(),
            peer: "peer".into(),
            mapping_kind: Kind::Dynamic,
            global: Global::Any {},
        },
        Step::Outbound {
            inside: "beta".into(),
            peer: "peer".into(),
            expected: Expected::PoolExhausted {},
        },
        Step::Outbound {
            inside: "unused".into(),
            peer: "peer".into(),
            expected: Expected::Untranslated {},
        },
    ];
    assert!(d.checks(&state).iter().all(|r| r.0));
    assert!(state.bindings().is_empty());
}
#[test]
fn disabled_invalid_fields_retained_enabled_invalid_pending_and_atomic() {
    let s = scenario();
    let mut current = sim(&s.initial).clone();
    let mut draft = Draft::new(&current).unwrap();
    draft.dynamic_enabled = false;
    draft.network = "not IPv4".into();
    draft.prefix = "bad".into();
    draft.first = "bad pool".into();
    draft.last = "\0\n".into();
    draft.apply(sim(&s.initial), &mut current).unwrap();
    assert!(draft.matches(&current));
    let raw = serde_json::to_vec(&draft).unwrap();
    exact(&Draft::from_json(&raw).unwrap(), &draft);
    draft.dynamic_enabled = true;
    assert!(!draft.matches(&current));
    let before = current.clone();
    assert!(draft.apply(sim(&s.initial), &mut current).is_err());
    exact(&current, &before);
    draft.dynamic_enabled = false;
    draft.statics[0].global = "invalid".into();
    assert!(draft.apply(sim(&s.initial), &mut current).is_err());
    exact(&current, &before);
}
#[test]
fn readonly_requests_do_not_block_credit_pending_policy_does() {
    let s = scenario();
    let mut a = attempt(&s, good(&s));
    assert!(progress::current_solved(&a, &s));
    let d = a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap();
    d.inbound_global = "unfinished.invalid".into();
    d.peer = Some("peer-two".into());
    d.notice = Some("Last input was incomplete".into());
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .nat_draft
        .as_mut()
        .unwrap()
        .network = "bad".into();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn incompatible_apply_and_raw_actions_reject_without_losing_draft_or_bindings() {
    let s = scenario();
    let mut current = sim(&good(&s)).clone();
    let mut draft = Draft::new(&current).unwrap();
    draft.first = "198.51.100.12".into();
    let old = current.clone();
    let raw = serde_json::to_vec(&draft).unwrap();
    assert!(draft.apply(sim(&s.initial), &mut current).is_err());
    exact(&current, &old);
    assert_eq!(serde_json::to_vec(&draft).unwrap(), raw);
    let mut e = State::BasicNat {
        simulation: current,
    };
    assert!(engine::execute(&mut e, &s.solution[0]).is_err());
    let before = e.clone();
    assert!(
        engine::execute_for_goal(
            &mut e,
            &s.goal,
            &" ".repeat(nat::MAX_ACTION_JSON + w::PREFIX.len() + 1)
        )
        .is_err()
    );
    exact(&e, &before);
}
#[test]
fn exact_context_origin_and_other_kind_cannot_be_imported() {
    let s = scenario();
    let mut state = sim(&good(&s)).clone();
    let mut value = serde_json::to_value(&state).unwrap();
    value["inventory"]["fixture"] = "another-origin".into();
    let other: nat::State = serde_json::from_value(value).unwrap();
    assert!(!def(&s).same_assignment(&other));
    let d = Draft::new(&other).unwrap();
    assert!(!d.valid_for(&state));
    let before = state.clone();
    assert!(d.apply(sim(&s.initial), &mut state).is_err());
    exact(&state, &before);
}
#[test]
fn negative_only_absent_preserved_duplicate_and_missing_positive_reverse_reject() {
    let s = scenario();
    for req in [
        vec![Requirement::NoCurrentMapping {
            inside: "alpha".into(),
        }],
        vec![Requirement::StaticPreserved {
            inside: "alpha".into(),
        }],
        vec![Requirement::DynamicCapacity { minimum: 0 }],
    ] {
        let mut d = def(&s).clone();
        d.requirements = req;
        assert!(d.validate().is_err());
    }
    let mut d = def(&s).clone();
    d.requirements.push(d.requirements[0].clone());
    assert!(d.validate().is_err());
    let mut d = def(&s).clone();
    d.trials[0]
        .steps
        .retain(|s| !matches!(s, Step::Reverse { .. }));
    assert!(d.validate().is_err());
}
#[test]
fn direct_nested_bounds_precede_serialization_and_clone() {
    let s = scenario();
    let mut d = def(&s).clone();
    d.requirements = vec![d.requirements[0].clone(); 33];
    assert!(d.validate().is_err());
    let mut d = def(&s).clone();
    d.trials = vec![d.trials[0].clone(); 5];
    assert!(d.validate().is_err());
    let mut d = def(&s).clone();
    d.trials[0].steps = vec![d.trials[0].steps[0].clone(); 17];
    assert!(d.validate().is_err());
    let mut d = def(&s).clone();
    if let Requirement::CurrentMapping { inside, .. } = &mut d.requirements[0] {
        *inside = "x".repeat(1024 * 1024);
    }
    assert!(d.validate().is_err());
    let mut draft = Draft::new(sim(&s.initial)).unwrap();
    draft.statics = vec![
        w::StaticRow {
            inside: "a".into(),
            global: "x".into()
        };
        33
    ];
    assert!(!draft.valid());
    draft.statics.clear();
    draft.network = " ".repeat(1024 * 1024);
    assert!(!draft.valid());
}
#[test]
fn strict_empty_variants_unknown_duplicate_kind_and_raw_cap() {
    let valid = r#"{"kind":"DynamicRulePreserved"}"#;
    let bad = r#"{"kind":"DynamicRulePreserved","extra":1}"#;
    assert!(serde_json::from_str::<Requirement>(valid).is_ok());
    assert!(serde_json::from_str::<Requirement>(bad).is_err());
    for raw in [
        r#"{"kind":"Any","extra":1}"#,
        r#"{"kind":"Any","kind":"Any"}"#,
    ] {
        assert!(serde_json::from_str::<Global>(raw).is_err());
    }
    for raw in [
        r#"{"kind":"Untranslated","extra":1}"#,
        r#"{"kind":"PoolExhausted","extra":1}"#,
    ] {
        assert!(serde_json::from_str::<Expected>(raw).is_err());
    }
    assert!(
        Definition::from_json(&vec![b' '; w::MAX_GOAL + 1])
            .unwrap_err()
            .contains("raw")
    );
    assert!(
        Draft::from_json(&vec![b' '; w::MAX_DRAFT + 1])
            .unwrap_err()
            .contains("raw")
    );
}
#[test]
fn max_escaped_draft_and_max_trial_work_stay_bounded() {
    let s = scenario();
    let mut draft = Draft::new(sim(&s.initial)).unwrap();
    draft.statics = vec![
        w::StaticRow {
            inside: "\0".repeat(32),
            global: "\0".repeat(64)
        };
        32
    ];
    draft.network = "\0".repeat(64);
    draft.prefix = "\0".repeat(16);
    draft.first = "\0".repeat(64);
    draft.last = "\0".repeat(64);
    draft.inbound_global = "\0".repeat(64);
    draft.notice = Some("\0".repeat(512));
    assert!(draft.valid());
    let raw = serde_json::to_vec(&draft).unwrap();
    assert!(raw.len() < w::MAX_DRAFT);
    exact(&draft, &Draft::from_json(&raw).unwrap());
    let mut d = def(&s).clone();
    for t in &mut d.trials {
        while t.steps.len() < 16 {
            t.steps.push(t.steps[2].clone());
        }
    }
    let a = d.trials[0].clone();
    let b = d.trials[1].clone();
    d.trials.extend([
        Trial {
            name: "third order".into(),
            ..a
        },
        Trial {
            name: "fourth order".into(),
            ..b
        },
    ]);
    let current = good(&s);
    let before = current.clone();
    let started = std::time::Instant::now();
    let report = d.report(sim(&current));
    eprintln!(
        "NAT maximum trial shape: charged_calls={} clones={} elapsed_us={}",
        report.calls,
        report.clones,
        started.elapsed().as_micros()
    );
    assert_eq!(report.calls, 68);
    assert_eq!(report.clones, 4);
    assert!(report.rows.iter().all(|r| r.0));
    assert!(serde_json::to_vec(&report.rows).unwrap().len() <= 32768);
    exact(&current, &before);
}
#[test]
fn foreign_metadata_fails_all_success_paths_and_never_overwrites_good_file() {
    let s = scenario();
    let mut all = pack::bundled().unwrap();
    assert_eq!(all.len(), 143);
    let old = all
        .iter()
        .find(|s| matches!(s.initial, State::ApacheStatic { .. }))
        .unwrap()
        .clone();
    let mut old_state = old.initial.clone();
    for cmd in &old.solution {
        engine::execute_for_goal(&mut old_state, &old.goal, cmd).unwrap();
    }
    let mut old_a = attempt(&old, old_state);
    assert!(progress::current_solved(&old_a, &old));
    let old_before = serde_json::to_vec(&old_a).unwrap();
    let nat = attempt(&s, good(&s));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    all.push(s.clone());
    p.attempts.insert(s.id.clone(), nat.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let good_disk = std::fs::read(&path).unwrap();
    old_a.workspace.as_mut().unwrap().nat_draft = nat.workspace.as_ref().unwrap().nat_draft.clone();
    let foreign = serde_json::to_vec(&old_a).unwrap();
    assert!(!progress::current_solved(&old_a, &old));
    assert!(
        progress::current_apache_static_checks(&old_a, &old)
            .iter()
            .any(|r| !r.0)
    );
    p.attempts.insert(old.id.clone(), old_a.clone());
    assert!(progress::save(&path, &p).is_err());
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), good_disk);
    assert_eq!(serde_json::to_vec(&old_a).unwrap(), foreign);
    let mut bad = nat.clone();
    let State::ApacheStatic { simulation } = &old.initial else {
        panic!()
    };
    bad.workspace.as_mut().unwrap().apache_static_draft =
        Some(dlms_learning_lab::apache_static_workspace::Draft::new(simulation).unwrap());
    let before = serde_json::to_vec(&bad).unwrap();
    assert!(!progress::current_solved(&bad, &s));
    assert!(progress::current_nat_checks(&bad, &s).iter().any(|r| !r.0));
    p.attempts.remove(&old.id);
    p.attempts.insert(s.id.clone(), bad.clone());
    assert!(progress::save(&path, &p).is_err());
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
    assert_eq!(serde_json::to_vec(&bad).unwrap(), before);
    assert_eq!(std::fs::read(&path).unwrap(), good_disk);
    let restored_old: Attempt = serde_json::from_slice(&old_before).unwrap();
    assert!(progress::current_solved(&restored_old, &old));
}
#[test]
fn current_contexts_pending_nat_backup_restore_and_selected_reset_retention() {
    use dlms_learning_lab::{backup, shell};
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let s = scenario();
    let mut all = pack::bundled().unwrap();
    assert_eq!(all.len(), 143);
    let old_specs = serde_json::to_vec(&all).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for old in &all {
        let mut a = Attempt::new(old, 7);
        workspace::prepare(&mut a);
        if let State::Script { source, run } = &mut a.state {
            *source = "printf 'unfinished old draft".into();
            *run = None;
        }
        p.attempts.insert(old.id.clone(), a);
    }
    // Include actual frozen writer history, never outputs generated then relabeled.
    let historical = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    for (id, mut a) in historical.attempts {
        // Decode validates without rewriting. Entering the real workspace moves
        // its actual earlier main Record to history before the retention snapshot.
        workspace::prepare(&mut a);
        p.attempts.insert(id, a);
    }
    // Independently pin full record contents to the genuine immutable fixtures:
    // v8 carries6/7 history and40 actual8 main runs, not8 history until prepare.
    fn has_exact_history(p: &Progress, id: &str, expected: &serde_json::Value) {
        let w = p.attempts[id].workspace.as_ref().unwrap();
        assert!(
            w.script_history
                .iter()
                .chain(&w.script_history_more)
                .any(|r| { serde_json::to_value(r).unwrap() == *expected }),
            "Missing unchanged historical record for {id}"
        );
    }
    let frozen6: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/shell-v6-authored-records.json")).unwrap();
    let rows6 = frozen6.as_array().unwrap();
    assert_eq!(rows6.len(), 33);
    for row in rows6 {
        assert_eq!(row["record"]["engine_revision"], 6);
        has_exact_history(&p, row["id"].as_str().unwrap(), &row["record"]);
    }
    for (raw, revision, count) in [
        (
            include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json").as_slice(),
            7,
            37,
        ),
        (
            include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
            8,
            40,
        ),
    ] {
        let frozen: serde_json::Value = serde_json::from_slice(raw).unwrap();
        let mut observed = 0;
        for (id, a) in frozen["attempts"].as_object().unwrap() {
            if let Some(record) = a["state"].get("run").filter(|r| !r.is_null()) {
                assert_eq!(record["engine_revision"], revision);
                has_exact_history(&p, id, record);
                observed += 1;
            }
        }
        assert_eq!(observed, count, "Exact immutable fixture record count");
    }
    let history_versions = p
        .attempts
        .values()
        .filter_map(|a| a.workspace.as_ref())
        .flat_map(|w| w.script_history.iter().chain(&w.script_history_more))
        .map(|r| r.engine_revision)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(history_versions.is_superset(&std::collections::BTreeSet::from([6, 7, 8])));
    let mut history_counts = std::collections::BTreeMap::new();
    for record in p
        .attempts
        .values()
        .filter_map(|a| a.workspace.as_ref())
        .flat_map(|w| w.script_history.iter().chain(&w.script_history_more))
    {
        *history_counts
            .entry(record.engine_revision)
            .or_insert(0usize) += 1;
    }
    assert_eq!(
        history_counts,
        std::collections::BTreeMap::from([(6, 33), (7, 37), (8, 40)])
    );
    let before = serde_json::to_value(&p.attempts).unwrap();
    let mut a = attempt(&s, good(&s));
    let d = a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap();
    d.network = "unfinished policy".into();
    d.inbound_global = "different unfinished observation".into();
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    all.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let original8 = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let original7 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(&path, original8).unwrap();
    std::fs::write(path.with_extension("before-shell-v8.json"), original7).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(
            path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION))
        )
        .unwrap(),
        original8
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        original7
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, value) in before.as_object().unwrap() {
        exact(&loaded.attempts[id], value);
    }
    exact(&loaded.attempts[&s.id], &selected);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    assert!(backup::inspect(&export, &all, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    // Backup inspection intentionally rechecks only these two practice fields.
    // Compute their expected fresh values independently from saved source/fixture;
    // compare every other field, including complete old histories, unchanged.
    let mut expected_backup = before.clone();
    for (id, old) in &loaded.attempts {
        if let Some(l) = old
            .workspace
            .as_ref()
            .and_then(|w| w.script_learning.as_ref())
            && l.example_result.is_some()
        {
            let lesson = all
                .iter()
                .find(|s| &s.id == id)
                .unwrap()
                .teaching
                .as_ref()
                .unwrap();
            let result = shell::run(&l.example_source, &lesson.fixture);
            expected_backup[id]["workspace"]["script_learning"]["example_result"] =
                serde_json::to_value(result).unwrap();
            expected_backup[id]["workspace"]["script_learning"]["example_revision"] =
                shell::EVALUATOR_REVISION.into();
        }
    }
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    exact(&candidate.progress.attempts[&s.id], &selected);
    for (id, value) in expected_backup.as_object().unwrap() {
        exact(&candidate.progress.attempts[id], value);
    }
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    exact(&restored.attempts[&s.id], &selected);
    let favorites = restored.favorites.clone();
    restored
        .attempts
        .insert(s.id.clone(), attempt(&s, s.initial.clone()));
    for (id, value) in expected_backup.as_object().unwrap() {
        exact(&restored.attempts[id], value);
    }
    assert_eq!(restored.favorites, favorites);
    assert_eq!(restored.attempts[&s.id].state, s.initial);
    all.pop();
    assert_eq!(serde_json::to_vec(&all).unwrap(), old_specs);
}
#[test]
fn raw_typed_collection_limit_and_future_state_reject_before_use() {
    let s = scenario();
    let d = Draft::new(sim(&s.initial)).unwrap();
    let mut v = serde_json::to_value(&d).unwrap();
    let rows = (0..32)
        .map(|_| serde_json::json!({"inside":"","global":""}))
        .chain([serde_json::json!({"unknown":true})])
        .collect::<Vec<_>>();
    v["statics"] = rows.into();
    let error = serde_json::from_value::<Draft>(v).unwrap_err().to_string();
    assert!(
        error.contains("collection bound"),
        "Reject33rd element before typed body, outer JSON already parsed"
    );
    let mut v = serde_json::to_value(sim(&s.initial)).unwrap();
    v["schema_version"] = 2.into();
    assert!(serde_json::from_value::<nat::State>(v).is_err());
    let mut goal = serde_json::to_value(def(&s)).unwrap();
    goal["future"] = true.into();
    assert!(serde_json::from_value::<Definition>(goal).is_err());
}
#[path = "support/actual9_evidence.rs"]
mod evidence;
#[test]
#[ignore = "Requires exact retained private public9 COMPLETE; only explicit guarded replay, no fixture generation"]
fn actual9_private_profile_nat_addition_preserves_full_old_histories_target_bytes_and_receipts() {
    let raw = evidence::original9();
    let original: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let mut all = pack::bundled().unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    let before = serde_json::to_value(&p.attempts).unwrap();
    assert_eq!(before.as_object().unwrap().len(), 133);
    for s in &all {
        if !before.as_object().unwrap().contains_key(&s.id) {
            assert!(!p.attempts.contains_key(&s.id));
        }
    }
    let s = scenario();
    p.attempts.insert(s.id.clone(), attempt(&s, good(&s)));
    p.favorites.insert(s.id.clone());
    let selected = serde_json::to_value(&p.attempts[&s.id]).unwrap();
    all.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    let old7 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(path.with_extension("before-shell-v8.json"), old7).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v10.json")).unwrap(),
        raw
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        old7
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, value) in before.as_object().unwrap() {
        evidence::exact(&loaded.attempts[id], value);
    }
    evidence::exact(&loaded.attempts[&s.id], &selected);
    let mut expected = original["migration_receipts"].clone();
    let versions = expected["shell"].as_array_mut().unwrap();
    if versions.last() != Some(&serde_json::json!(10)) {
        versions.push(serde_json::json!(10));
    }
    evidence::exact(&loaded.migration_receipts.as_ref().unwrap(), &expected);
    assert_eq!(loaded.attempts.len(), 134);
    assert_eq!(evidence::original9(), raw);
}
#[test]
fn maximum_inventory_escaped_editor_and_definition_fit_their_declared_closure() {
    use std::collections::{BTreeMap, BTreeSet};
    let inside = (0..64)
        .map(|n| (format!("inside-{n:025}"), Ipv4Addr::new(10, 0, 0, n)))
        .collect::<BTreeMap<_, _>>();
    let outside = (0..64)
        .map(|n| (format!("outside-{n:024}"), Ipv4Addr::new(203, 0, 113, n)))
        .collect();
    let globals = (0..128)
        .map(|n| Ipv4Addr::new(198, 51, 100, n))
        .collect::<BTreeSet<_>>();
    let inventory = nat::Inventory {
        fixture: "f".repeat(32),
        inside,
        outside,
        globals,
    };
    let statics = inventory
        .inside
        .keys()
        .take(32)
        .enumerate()
        .map(|(n, id)| (id.clone(), Ipv4Addr::new(198, 51, 100, n as u8)))
        .collect();
    let config = nat::Configuration {
        statics,
        dynamic: Some(nat::DynamicRule {
            source: nat::Prefix {
                network: ip("10.0.0.0"),
                length: 24,
            },
            pool: nat::Pool {
                first: ip("198.51.100.32"),
                last: ip("198.51.100.95"),
            },
        }),
    };
    let mut state = nat::State::new(&inventory, &config).unwrap();
    let peer = inventory.outside.keys().next().unwrap();
    for host in inventory.inside.keys().skip(32) {
        state
            .apply(&Action::Outbound {
                inside: host.clone(),
                peer: peer.clone(),
            })
            .unwrap();
    }
    assert_eq!(state.bindings().len(), 32);
    let raw = serde_json::to_vec(&state).unwrap();
    assert!(raw.len() <= nat::MAX_STATE_JSON);
    let mut draft = Draft::new(&state).unwrap();
    draft.statics = vec![
        w::StaticRow {
            inside: "\0".repeat(32),
            global: "\0".repeat(64)
        };
        32
    ];
    draft.network = "\0".repeat(64);
    draft.prefix = "\0".repeat(16);
    draft.first = "\0".repeat(64);
    draft.last = "\0".repeat(64);
    draft.inbound_global = "\0".repeat(64);
    draft.notice = Some("\0".repeat(512));
    let pretty = serde_json::to_vec_pretty(&draft).unwrap();
    assert!(pretty.len() <= w::MAX_DRAFT);
    exact(&Draft::from_json(&pretty).unwrap(), &draft);
    let host = inventory.inside.keys().next_back().unwrap().clone();
    let pair = [
        Step::Outbound {
            inside: host.clone(),
            peer: peer.clone(),
            expected: Expected::Translated {
                mapping_kind: Kind::Dynamic,
                global: Global::Any {},
            },
        },
        Step::Reverse {
            inside: host,
            peer: peer.clone(),
            mapping_kind: Kind::Dynamic,
            global: Global::Any {},
        },
    ];
    let trials = (0..4)
        .map(|n| Trial {
            name: format!("{n}{}", "t".repeat(63)),
            steps: (0..16).map(|n| pair[n % 2].clone()).collect(),
        })
        .collect();
    let mut requirements = inventory
        .inside
        .keys()
        .take(31)
        .map(|inside| Requirement::CurrentMapping {
            inside: inside.clone(),
            mapping_kind: Kind::Dynamic,
            global: Global::Any {},
        })
        .collect::<Vec<_>>();
    requirements.push(Requirement::DynamicCapacity { minimum: 1 });
    let definition = Definition {
        initial: state,
        requirements,
        trials,
    };
    definition.validate().unwrap();
    let raw = serde_json::to_vec_pretty(&definition).unwrap();
    assert!(raw.len() <= w::MAX_GOAL);
    Definition::from_json(&raw).unwrap();
}
#[test]
fn excluded_host_requirement_needs_an_applied_rule_independent_of_capacity() {
    let s = scenario();
    let mut d = def(&s).clone();
    let g = Global::Exact {
        address: ip("198.51.100.50"),
    };
    d.requirements = vec![
        Requirement::CurrentMapping {
            inside: "observer".into(),
            mapping_kind: Kind::Static,
            global: g,
        },
        Requirement::DynamicPrefixMatches {
            inside: "alpha".into(),
            matches: false,
        },
    ];
    d.trials = vec![Trial {
        name: "static-only reverse before outbound".into(),
        steps: vec![
            Step::Reverse {
                inside: "observer".into(),
                peer: "peer".into(),
                mapping_kind: Kind::Static,
                global: g,
            },
            Step::Outbound {
                inside: "observer".into(),
                peer: "peer".into(),
                expected: Expected::Translated {
                    mapping_kind: Kind::Static,
                    global: g,
                },
            },
        ],
    }];
    d.validate().unwrap();
    let mut current = sim(&s.initial).clone();
    let before = current.clone();
    let rows = d.checks(&current);
    assert!(rows.iter().all(|r| r.0));
    assert!(rows[1].1.contains("excludes alpha"));
    exact(&current, &before);
    let mut c = current.configuration().clone();
    c.dynamic = None;
    w::apply(
        &d.initial,
        &mut current,
        &Action::Configure { configuration: c },
    )
    .unwrap();
    let before = current.clone();
    let rows = d.checks(&current);
    assert!(
        rows[0].0 && rows[2].0,
        "Static positive current/fresh controls remain healthy"
    );
    assert!(!rows[1].0, "Absent dynamic rule cannot prove exclusion");
    assert!(rows[1].1.contains("excludes alpha"));
    exact(&current, &before);
    w::apply(
        &d.initial,
        &mut current,
        &Action::Configure {
            configuration: policy(&s),
        },
    )
    .unwrap();
    let rows = d.checks(&current);
    assert!(rows[0].0 && rows[2].0);
    assert!(
        !rows[1].0,
        "Applied matching rule violates required exclusion"
    );
}
#[test]
fn individually_valid_normal_family_metadata_is_rejected_only_on_nat() {
    use dlms_learning_lab::{
        ipv6_teaching, ipv6_workspace, script_teaching, switching_teaching, switching_workspace,
        terminal_teaching,
    };
    fn attach(
        name: &str,
        target: &mut workspace::WorkspaceData,
        source: &workspace::WorkspaceData,
    ) {
        match name {
            "script_learning" => target.script_learning = source.script_learning.clone(),
            "terminal_learning" => target.terminal_learning = source.terminal_learning.clone(),
            "switching_draft" => target.switching_draft = source.switching_draft.clone(),
            "switching_learning" => target.switching_learning = source.switching_learning.clone(),
            "ipv6_draft" => target.ipv6_draft = source.ipv6_draft.clone(),
            "ipv6_learning" => target.ipv6_learning = source.ipv6_learning.clone(),
            _ => panic!("closed test metadata list"),
        }
    }
    let s = scenario();
    let mut all = pack::bundled().unwrap();
    let script = all.iter().find(|s| s.teaching.is_some()).unwrap().clone();
    let terminal = all
        .iter()
        .find(|s| s.terminal_teaching.is_some())
        .unwrap()
        .clone();
    let switching = all
        .iter()
        .find(|s| s.switching_teaching.is_some())
        .unwrap()
        .clone();
    let ipv6 = all
        .iter()
        .find(|s| s.ipv6_teaching.is_some())
        .unwrap()
        .clone();
    let mut controls = Vec::new();
    for (name, lab) in [
        ("script_learning", &script),
        ("terminal_learning", &terminal),
        ("switching_draft", &switching),
        ("switching_learning", &switching),
        ("ipv6_draft", &ipv6),
        ("ipv6_learning", &ipv6),
    ] {
        let mut control = attempt(lab, lab.initial.clone());
        let w = control.workspace.as_mut().unwrap();
        match name {
            "script_learning" => {
                let l = script_teaching::Learning::new(lab.teaching.as_ref().unwrap());
                assert!(l.valid());
                w.script_learning = Some(l);
            }
            "terminal_learning" => {
                let l = terminal_teaching::Learning::new(lab.terminal_teaching.as_ref().unwrap());
                assert!(l.valid());
                w.terminal_learning = Some(l);
            }
            "switching_draft" => {
                let State::Switching { fabric } = &lab.initial else {
                    panic!()
                };
                let d = switching_workspace::Draft::new(fabric);
                assert!(d.valid());
                w.switching_draft = Some(d);
            }
            "switching_learning" => {
                let l = switching_teaching::Learning::new(lab.switching_teaching.as_ref().unwrap());
                assert!(l.valid());
                w.switching_learning = Some(l);
            }
            "ipv6_draft" => {
                let State::Ipv6 { topology } = &lab.initial else {
                    panic!()
                };
                let d = ipv6_workspace::Draft::new(topology);
                assert!(d.valid());
                w.ipv6_draft = Some(d);
            }
            "ipv6_learning" => {
                let l = ipv6_teaching::Learning::new(lab.ipv6_teaching.as_ref().unwrap());
                assert!(l.valid());
                w.ipv6_learning = Some(l);
            }
            _ => unreachable!(),
        }
        assert!(w.valid());
        controls.push((name, lab.clone(), control));
    }
    all.push(s.clone());
    let nat = attempt(&s, good(&s));
    assert!(progress::current_solved(&nat, &s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), nat.clone());
    progress::save(&path, &p).unwrap();
    let good_bytes = std::fs::read(&path).unwrap();
    for (name, lab, control) in controls {
        let mut normal = Progress {
            schema_version: 1,
            ..Default::default()
        };
        normal.attempts.insert(lab.id.clone(), control.clone());
        progress::save(&dir.path().join("normal-control.json"), &normal).unwrap();
        let decoded = progress::decode(&serde_json::to_vec(&normal).unwrap(), &all).unwrap();
        exact(&decoded.attempts[&lab.id], &control);
        let mut bad = nat.clone();
        attach(
            name,
            bad.workspace.as_mut().unwrap(),
            control.workspace.as_ref().unwrap(),
        );
        assert!(
            bad.workspace.as_ref().unwrap().valid(),
            "{name} must be individually valid"
        );
        let before = serde_json::to_vec(&bad).unwrap();
        let rows = progress::current_nat_checks(&bad, &s);
        assert!(
            rows.iter().any(|r| !r.0),
            "foreign {name} must fail fresh checks"
        );
        assert!(!progress::current_solved(&bad, &s));
        p.attempts.insert(s.id.clone(), bad.clone());
        assert!(
            progress::save(&path, &p).is_err(),
            "foreign {name} must not save"
        );
        assert!(
            progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err(),
            "foreign {name} must not decode"
        );
        assert_eq!(std::fs::read(&path).unwrap(), good_bytes);
        assert_eq!(
            serde_json::to_vec(&bad).unwrap(),
            before,
            "helper/save/decode must not rewrite caller"
        );
        assert!(progress::current_solved(&nat, &s));
    }
}
#[test]
fn shared_command_history_and_legacy_scalar_fields_remain_retainable_without_credit() {
    let s = scenario();
    let mut all = pack::bundled().unwrap();
    all.push(s.clone());
    let mut a = attempt(&s, good(&s));
    let old: serde_json::Value = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    let record = old["attempts"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|a| a.get("workspace").and_then(|w| w.get("script_history")))
        .unwrap()
        .clone();
    let record: dlms_learning_lab::shell::Record = serde_json::from_value(record).unwrap();
    assert!(record.valid());
    let w = a.workspace.as_mut().unwrap();
    w.script_history = Some(record);
    w.command = "saved unrelated command input".into();
    w.service_port = Some("unrelated raw service text".into());
    w.dns_address = Some("unrelated raw DNS text".into());
    w.dns_serial = Some("unrelated raw serial text".into());
    w.mount_device = Some("unrelated raw device text".into());
    w.ipv4_check = Some(std::array::from_fn(|_| "retained IPv4 view".into()));
    w.practice_check = Some(std::array::from_fn(|_| "retained practice view".into()));
    assert!(w.valid());
    assert!(progress::current_solved(&a, &s));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    exact(&loaded.attempts[&s.id], &a);
    let State::BasicNat { simulation } = &mut a.state else {
        panic!()
    };
    w::apply(sim(&s.initial), simulation, &Action::ClearDynamic {}).unwrap();
    assert!(
        !progress::current_solved(&a, &s),
        "Shared fields/history cannot replace actual current mappings"
    );
}
