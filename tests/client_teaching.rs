use dlms_learning_lab::{
    client_teaching::{self as client, Action, Client, Draft, Learning},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
fn scenario() -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "ccna-gateway")
        .unwrap()
}
fn prepared(new: bool) -> (pack::Scenario, Attempt) {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    client::prepare(&mut a, s.client_teaching.as_ref().unwrap(), new).unwrap();
    (s, a)
}
fn solved(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
#[test]
fn original_nine_definitions_and_existing_client_terminal_solutions_remain_exact() {
    let labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), 164);
    let old: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/client-original-seeds.json")).unwrap();
    for row in old["scenarios"].as_array().unwrap() {
        let s = labs
            .iter()
            .find(|s| s.id == row["id"].as_str().unwrap())
            .unwrap();
        assert_eq!(s.version, row["version"].as_u64().unwrap() as u32);
        assert_eq!(serde_json::to_value(&s.initial).unwrap(), row["initial"]);
        // Decode original omission/default semantics without rewriting the captured source.
        let original_goal: engine::Goal = serde_json::from_value(row["goal"].clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&s.goal).unwrap(),
            serde_json::to_value(original_goal).unwrap()
        );
        assert_eq!(serde_json::to_value(&s.solution).unwrap(), row["solution"]);
    }
    let s = scenario();
    let reference = solved(&s);
    let mut alternate = s.initial.clone();
    for input in [
        "ip route replace default via 192.168.40.65",
        "ip addr set 192.168.40.70/26",
    ] {
        engine::execute(&mut alternate, input).unwrap();
    }
    assert_eq!(alternate, reference);
}
#[test]
fn separate_example_inputs_never_execute_or_credit_and_real_gateway_experiment_repairs() {
    let (s, a) = prepared(true);
    let lesson = s.client_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let initial = l.client;
    l.fill(lesson).unwrap();
    assert_eq!(l.client, initial);
    assert!(!l.checked);
    assert!(!l.draft.matches(&l.client));
    assert!(!progress::current_solved(&a, &s));
    for (i, action) in lesson.example_actions.iter().enumerate() {
        l.selected_action = i;
        let before = l.client;
        l.insert(lesson).unwrap();
        assert_eq!(l.client, before);
        match &action.action {
            Action::Apply { .. } => l.apply().unwrap(),
            Action::Command { .. } => l.submit().unwrap(),
        };
    }
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!(a.state, s.initial);
    l.draft = lesson.try_settings.clone();
    l.apply().unwrap();
    let c = l.checks(lesson);
    assert_eq!(
        c.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, false]
    );
    assert!(
        l.client
            .execute(&format!("ping {}", l.client.router))
            .unwrap()
            .contains("Reply")
    );
    assert!(
        l.client
            .execute(&format!("ping {}", l.client.target))
            .unwrap()
            .contains("unreachable")
    );
    l.draft = lesson.repair_settings.clone();
    l.apply().unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
}
#[test]
fn native_apply_is_atomic_preserves_wrong_raw_text_and_accepts_a_real_partial_state() {
    let s = scenario();
    let c = Client::from_state(&s.initial).unwrap();
    for (field, value) in [
        ("address", "bad"),
        ("prefix", "31"),
        ("prefix", "oops"),
        ("gateway", "999.1.1.1"),
    ] {
        let mut d = Draft::new(&c);
        match field {
            "address" => d.address = value.into(),
            "prefix" => d.prefix = value.into(),
            _ => d.gateway = value.into(),
        };
        let before = d.clone();
        let mut state = s.initial.clone();
        assert!(client::apply_assignment(&mut state, &d).is_err());
        assert_eq!(state, s.initial);
        assert_eq!(d, before);
    }
    let mut d = Draft::new(&c);
    d.prefix = "26".into();
    let mut state = s.initial.clone();
    client::apply_assignment(&mut state, &d).unwrap();
    assert_eq!(
        engine::checks(&state, &s.goal)
            .iter()
            .map(|c| c.0)
            .collect::<Vec<_>>(),
        [true, true, false]
    );
    d.gateway = "192.168.40.65".into();
    client::apply_assignment(&mut state, &d).unwrap();
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn absent_legacy_drafts_never_invalidate_credit_and_old_attempts_open_assignment() {
    let s = scenario();
    let mut a = Attempt::new(&s, 7);
    a.state = solved(&s);
    a.hints = 1;
    a.step = 1;
    a.transcript.push("retained query".into());
    assert!(progress::current_solved(&a, &s));
    let before = a.state.clone();
    workspace::prepare(&mut a);
    assert!(progress::current_solved(&a, &s));
    assert!(a.workspace.as_ref().unwrap().client_draft.is_none());
    client::prepare(&mut a, s.client_teaching.as_ref().unwrap(), false).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .client_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    assert_eq!(a.state, before);
    assert_eq!(a.hints, 1);
    assert_eq!(a.step, 1);
    assert_eq!(a.transcript, ["retained query"]);
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .client_draft
        .as_mut()
        .unwrap()
        .prefix = "oops".into();
    assert!(!progress::current_solved(&a, &s));
    a.workspace.as_mut().unwrap().client_draft = None;
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn supplied_assignment_context_is_pinned_without_restricting_editable_settings() {
    let labs = pack::bundled().unwrap();
    let s = scenario();
    let mut a = Attempt::new(&s, 7);
    a.state = solved(&s);
    // Missing legacy drafts retain truthful credit and are accepted on decode.
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a.clone());
    assert!(progress::current_solved(&a, &s));
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_ok());
    for field in ["router", "lan_prefix", "target"] {
        let mut foreign = a.clone();
        let State::Network {
            router,
            gateway,
            prefix,
            lan_prefix,
            target,
            ..
        } = &mut foreign.state
        else {
            panic!()
        };
        match field {
            "router" => {
                *router = "192.168.40.66".parse().unwrap();
                *gateway = *router;
            }
            "lan_prefix" => {
                *lan_prefix = 25;
                *prefix = 25;
            }
            _ => {
                *target = "203.0.113.91".parse().unwrap();
            }
        }
        assert!(
            engine::solved(&foreign.state, &s.goal),
            "the original outcome alone cannot pin {field}"
        );
        assert!(!progress::current_solved(&foreign, &s));
        p.attempts.insert(s.id.clone(), foreign);
        assert!(
            progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_err(),
            "foreign {field}"
        );
    }
    // Real editable mistakes remain loadable for repair, independent of optional drafts.
    for field in ["address", "prefix", "gateway"] {
        let mut editable = a.clone();
        let State::Network {
            address,
            prefix,
            gateway,
            ..
        } = &mut editable.state
        else {
            panic!()
        };
        match field {
            "address" => *address = "192.168.40.71".parse().unwrap(),
            "prefix" => *prefix = 24,
            _ => *gateway = "192.168.40.1".parse().unwrap(),
        }
        p.attempts.insert(s.id.clone(), editable);
        assert!(
            progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_ok(),
            "editable {field}"
        );
    }
}
#[test]
fn exact_practice_origin_is_pinned_but_evolving_values_can_coincide_assignment() {
    let labs = pack::bundled().unwrap();
    let (s, mut a) = prepared(false);
    let current = Client::from_state(&a.state).unwrap();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .client_learning
        .as_mut()
        .unwrap();
    l.client.address = current.address;
    l.draft = Draft::new(&l.client);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let good = serde_json::to_value(&p).unwrap();
    assert!(progress::decode(&serde_json::to_vec(&good).unwrap(), &labs).is_ok());
    let mut foreign = good;
    for key in ["fixture", "client"] {
        foreign["attempts"][&s.id]["workspace"]["client_learning"][key]["target"] =
            "203.0.113.91".into();
    }
    assert!(progress::decode(&serde_json::to_vec(&foreign).unwrap(), &labs).is_err());
    // Existing assignment may also contain a legitimate wrong address equal to the example.
    let a = p.attempts.get_mut(&s.id).unwrap();
    if let State::Network { address, .. } = &mut a.state {
        *address = s.client_teaching.as_ref().unwrap().fixture.address;
    }
    client::prepare(a, s.client_teaching.as_ref().unwrap(), false).unwrap();
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_ok());
}
#[test]
fn bounded_readers_reject_excess_bodies_and_direct_oversize_before_any_mutation() {
    let lesson = scenario().client_teaching.unwrap();
    let mut l = Learning::new(&lesson).unwrap();
    let original = l.client;
    l.draft.address = "é".repeat(1000);
    let draft = l.draft.clone();
    assert!(l.apply().is_err());
    assert_eq!(l.client, original);
    assert_eq!(l.draft, draft);
    let mut raw = serde_json::to_value(Learning::new(&lesson).unwrap()).unwrap();
    raw["transcript"] = serde_json::json!(vec!["query"; 32]);
    assert!(serde_json::from_value::<Learning>(raw.clone()).is_ok());
    raw["transcript"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"not":"decoded"}));
    let err = serde_json::from_value::<Learning>(raw)
        .unwrap_err()
        .to_string();
    assert!(!err.contains("expected a string"));
    let mut raw = serde_json::to_value(&lesson).unwrap();
    let first = raw["example_actions"][0].clone();
    raw["example_actions"] = serde_json::Value::Array(vec![first; 12]);
    assert!(serde_json::from_value::<client::Lesson>(raw.clone()).is_ok());
    raw["example_actions"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"invalid":"excess action"}));
    assert!(serde_json::from_value::<client::Lesson>(raw).is_err());
    let mut future = serde_json::to_value(Learning::new(&lesson).unwrap()).unwrap();
    future["version"] = 2.into();
    let future: Learning = serde_json::from_value(future).unwrap();
    assert!(!future.valid());
    let mut unknown = serde_json::to_value(Learning::new(&lesson).unwrap()).unwrap();
    unknown["future_key"] = true.into();
    assert!(serde_json::from_value::<Learning>(unknown).is_err());
    let (scenario, mut attempt) = prepared(false);
    attempt
        .workspace
        .as_mut()
        .unwrap()
        .client_draft
        .as_mut()
        .unwrap()
        .prefix = "x".repeat(17);
    let untouched = serde_json::to_value(&attempt).unwrap();
    assert!(
        client::prepare(
            &mut attempt,
            scenario.client_teaching.as_ref().unwrap(),
            false
        )
        .is_err()
    );
    assert_eq!(serde_json::to_value(attempt).unwrap(), untouched);
    let mut invalid = lesson.clone();
    invalid.example_actions = vec![invalid.example_actions[0].clone(); 13];
    let mut l = Learning::new(&lesson).unwrap();
    let before = l.clone();
    assert!(l.fill(&invalid).is_err());
    assert_eq!(l, before);
}
#[test]
fn commands_preserve_unapplied_native_edits_and_queries_keep_current_example_checks() {
    let lesson = scenario().client_teaching.unwrap();
    let mut l = Learning::new(&lesson).unwrap();
    l.fill(&lesson).unwrap();
    l.apply().unwrap();
    l.checked = true;
    l.command = "ip route".into();
    l.submit().unwrap();
    assert!(l.checked && l.checks(&lesson).iter().all(|c| c.0));
    l.draft.gateway = "unfinished".into();
    let before = l.draft.clone();
    l.command = "ip addr set 198.51.100.150/27".into();
    l.submit().unwrap();
    assert_eq!(l.draft, before);
    assert!(!l.checks(&lesson).iter().all(|c| c.0));
}
#[test]
fn load_is_read_only_optional_fields_stay_absent_and_invalid_save_keeps_original_bytes() {
    let labs = pack::bundled().unwrap();
    let s = scenario();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&s, 3);
    a.state = solved(&s);
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let raw = format!("  {}\n", serde_json::to_string(&p).unwrap());
    std::fs::write(&path, &raw).unwrap();
    let read = progress::load(&path, &labs).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), raw.as_bytes());
    assert!(read.attempts[&s.id].workspace.is_none());
    assert!(progress::current_solved(&read.attempts[&s.id], &s));
    let mut bad = read;
    let a = bad.attempts.get_mut(&s.id).unwrap();
    workspace::prepare(a);
    client::prepare(a, s.client_teaching.as_ref().unwrap(), false).unwrap();
    a.workspace
        .as_mut()
        .unwrap()
        .client_draft
        .as_mut()
        .unwrap()
        .address = "x".repeat(65);
    assert!(progress::save(&path, &bad).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw.as_bytes());
    let (_, a) = prepared(true);
    let learning = a.workspace.unwrap().client_learning;
    let other = &labs[3];
    let mut wrong = Attempt::new(other, 1);
    workspace::prepare(&mut wrong);
    wrong.workspace.as_mut().unwrap().client_learning = learning;
    bad.attempts.clear();
    bad.attempts.insert(other.id.clone(), wrong);
    assert!(progress::save(&path, &bad).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw.as_bytes());
}
#[test]
fn saved_practice_and_raw_assignment_resume_separately_and_selected_reset_keeps_others() {
    let labs = pack::bundled().unwrap();
    let (s, mut a) = prepared(false);
    let w = a.workspace.as_mut().unwrap();
    w.client_draft.as_mut().unwrap().prefix = "oops".into();
    let l = w.client_learning.as_mut().unwrap();
    l.command = "ip ro".into();
    l.view = View::Example;
    l.draft.gateway = "unfinished".into();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for lab in &labs {
        p.attempts.insert(lab.id.clone(), Attempt::new(lab, 1));
    }
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let raw = serde_json::to_vec(&p).unwrap();
    let loaded = progress::decode(&raw, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let others = loaded
        .attempts
        .iter()
        .filter(|(id, _)| *id != &s.id)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut reset = loaded;
    let (_, a) = prepared(true);
    reset.attempts.insert(s.id.clone(), a);
    for (id, expected) in others {
        assert_eq!(
            serde_json::to_value(&reset.attempts[&id]).unwrap(),
            expected
        );
    }
}
#[test]
fn backup_recheck_preserves_genuine_six_seven_and_current_eight_records_without_relabeling() {
    let labs = pack::bundled().unwrap();
    let original: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    let mut p: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    for s in &labs {
        p.attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
    }
    let (s, mut a) = prepared(false);
    a.workspace
        .as_mut()
        .unwrap()
        .client_draft
        .as_mut()
        .unwrap()
        .gateway = "incomplete".into();
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("export.json");
    dlms_learning_lab::backup::export(&backup, &p, &labs).unwrap();
    let candidate = dlms_learning_lab::backup::inspect(
        &backup,
        &labs,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let (mut six, mut seven) = (0, 0);
    for (id, a) in &original.attempts {
        let State::Script {
            run: Some(record), ..
        } = &a.state
        else {
            continue;
        };
        let w = candidate.progress.attempts[id].workspace.as_ref().unwrap();
        let all = w
            .script_history
            .iter()
            .chain(&w.script_history_more)
            .collect::<Vec<_>>();
        assert!(all.contains(&record));
        let State::Script {
            source,
            run: Some(current),
        } = &candidate.progress.attempts[id].state
        else {
            panic!("rechecked script must have a current run")
        };
        assert!(current.current());
        assert_eq!(&current.source, source);
        seven += 1;
        if let Some(old) = a.workspace.as_ref().and_then(|w| w.script_history.as_ref()) {
            assert_eq!(w.script_history.as_ref(), Some(old));
            six += 1;
        }
    }
    assert_eq!((six, seven), (33, 37));
    assert_eq!(
        candidate.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .client_draft
            .as_ref()
            .unwrap()
            .gateway,
        "incomplete"
    );
    let path = dir.path().join("profile.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let restored = dlms_learning_lab::backup::restore(&path, &candidate, &labs).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        before
    );
    let loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        serde_json::to_value(&candidate.progress.attempts[&s.id]).unwrap()
    );
}
