//! Authored transport unit admission, source-held grading and retained progress.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack, progress, transport_workspace as work,
};
fn authored() -> Vec<pack::Scenario> {
    let all = pack::accepted294().unwrap();
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/transport-lessons-v1.json")).unwrap();
    let mut texts = Vec::new();
    for chunk in all.chunks(8) {
        texts.push(
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":chunk}))
                .unwrap(),
        );
    }
    texts.push(serde_json::to_string(&data).unwrap());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
        .unwrap()
        .into_iter()
        .filter(|s| s.id.starts_with("ccna-transport-"))
        .collect()
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::Transport { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &pack::Scenario) -> &work::Session {
    let State::Transport { simulation } = &s.initial else {
        panic!()
    };
    simulation
}
#[test]
fn both_taught_units_admit_and_worked_settings_reach_multiple_complete_outcomes() {
    for s in authored() {
        let d = def(&s);
        d.validate().unwrap();
        assert_eq!(session(&s), &work::Session::new(d).unwrap());
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(d.assignment.requirements.len() >= 3);
        let mut state = s.initial.clone();
        for action in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        let mut p = work::Session::new(d).unwrap();
        let before = p.assignment.clone();
        p.practice
            .edit(&d.example, d.worked_example.clone())
            .unwrap();
        assert!(p.practice.check(&d.example).unwrap().iter().all(|r| r.0));
        assert_eq!(p.assignment, before);
        assert!(!d.checks(&p).iter().all(|r| r.0));
    }
}
#[test]
fn independent_pairings_allow_real_alternatives_but_partial_and_wrong_protocol_fail() {
    let list = authored();
    let s = &list[0];
    let d = def(s);
    for port in [8000, 8443, 65535] {
        let mut draft = d.worked_assignment.clone();
        draft.listeners[0].port = port.to_string();
        for e in &mut draft.exchanges[..2] {
            e.server_port = port.to_string();
        }
        assert!(d.assignment.checks(&draft).iter().all(|r| r.0));
    }
    let mut draft = d.worked_assignment.clone();
    draft.exchanges[1].server_port = "9999".into();
    let c = d.assignment.checks(&draft);
    assert_eq!(
        c.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![true, false, true]
    );
    assert!(c[1].1.contains("no listener"));
    let mut draft = d.worked_assignment.clone();
    draft.listeners[0].protocol = dlms_learning_lab::transport_core::Protocol::Udp;
    for e in &mut draft.exchanges[..2] {
        e.protocol = dlms_learning_lab::transport_core::Protocol::Udp;
    }
    assert!(d.assignment.checks(&draft).iter().any(|r| !r.0));
}
#[test]
fn reverse_policy_wrong_side_broad_permits_and_deny_all_never_get_credit() {
    let list = authored();
    let s = &list[1];
    let d = def(s);
    let starter = work::Draft::new(&d.assignment).unwrap();
    let results = d.assignment.checks(&starter);
    assert!(!results[0].0);
    assert!(results[0].1.contains("SYN-ACK"));
    assert_eq!(
        results.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![false, false, true, true, false, false]
    );
    // Reverse restrictions require a working positive crossing of that same boundary.
    for bad in [
        "10 permit ip any any",
        "10 permit tcp any any",
        "10 deny ip any any",
    ] {
        let mut draft = d.worked_assignment.clone();
        let raw = draft.documents.routers.get_mut("R1").unwrap();
        let old = "10 permit tcp host 198.51.100.20 eq 8443 host 192.0.2.10 range 49152 65535";
        assert!(raw.contains(old));
        *raw = raw.replace(old, bad);
        let checks = d.assignment.checks(&draft);
        assert!(checks.iter().any(|r| !r.0), "{bad} {checks:?}");
    }
    // Equivalent two explicit destination-port permits are allowed; no string matching.
    let mut alt = d.worked_assignment.clone();
    let raw = alt.documents.routers.get_mut("R1").unwrap();
    *raw=raw.replace("10 permit tcp host 198.51.100.20 eq 8443 host 192.0.2.10 range 49152 65535","10 permit tcp host 198.51.100.20 eq 8443 host 192.0.2.10 eq 50123\n 20 permit tcp host 198.51.100.20 eq 8443 host 192.0.2.10 eq 50124");
    assert!(d.assignment.checks(&alt).iter().all(|r| r.0));
}
#[test]
fn source_guards_and_actual_safe_save_load_preserve_drafts_favorites_and_check_invalidation() {
    let labs = authored();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = progress::Attempt::new(s, 1);
        if let State::Transport { simulation } = &mut a.state {
            simulation
                .assignment
                .edit(&def(s).assignment, def(s).worked_assignment.clone())
                .unwrap();
            simulation.assignment.check(&def(s).assignment).unwrap();
            if def(s).example.editable_exchanges.is_empty() {
                simulation
                    .practice
                    .draft
                    .documents
                    .routers
                    .insert("R1".into(), "unfinished practice edit\n".into());
            } else {
                simulation.practice.draft.exchanges[0].client_port = "unfinished".into();
            }
        }
        p.favorites.insert(s.id.clone());
        assert!(progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
    }
    let root = std::env::temp_dir().join(format!("transport-progress-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    assert!(progress::save(&path, &p).is_err());
    let mut loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(p.favorites, loaded.favorites);
    for s in &labs {
        let a = loaded.attempts.get_mut(&s.id).unwrap();
        work::context(a, s).unwrap();
        assert!(progress::current_solved(a, s));
        if let State::Transport { simulation } = &mut a.state {
            if def(s).example.editable_exchanges.is_empty() {
                assert_eq!(
                    simulation.practice.draft.documents.routers["R1"],
                    "unfinished practice edit\n"
                );
            } else {
                assert_eq!(
                    simulation.practice.draft.exchanges[0].client_port,
                    "unfinished"
                );
            }
            let mut changed = simulation.assignment.draft.clone();
            if !def(s).assignment.editable_listeners.is_empty() {
                changed.listeners[0].port = "1".into();
            } else {
                changed.documents.routers.insert("R1".into(), String::new());
            }
            simulation
                .assignment
                .edit(&def(s).assignment, changed)
                .unwrap();
        }
        assert!(!progress::current_solved(a, s));
    }
    let s = &labs[0];
    let mut foreign = loaded.attempts[&s.id].clone();
    if let State::Transport { simulation } = &mut foreign.state {
        simulation.origin = labs[1].id.clone();
    }
    assert!(work::context(&foreign, s).is_err());
    assert!(!progress::current_solved(&foreign, s));
    let mut wrong = labs[0].initial.clone();
    assert!(
        engine::execute_for_goal(&mut wrong, &Goal::Script { cases: Vec::new() }, "help").is_err()
    );
}

#[test]
fn current_catalog_preserves_all_previous_scenario_and_unit_values_and_adds_only_two_real_tasks() {
    let before = pack::accepted294().unwrap();
    let after = pack::accepted296().unwrap();
    assert_eq!((before.len(), after.len()), (294, 296));
    for old in &before {
        let new = after.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(new).unwrap()
        );
    }
    for expected in authored() {
        let actual = after.iter().find(|s| s.id == expected.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
    let prior = dlms_learning_lab::course::accepted294(&before).unwrap();
    let current = dlms_learning_lab::course::accepted296(&after).unwrap();
    assert_eq!((prior.len(), current.len()), (95, 96));
    for old in &prior {
        let new = current.iter().find(|u| u.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(new).unwrap()
        );
    }
    let retained: Vec<_> = current
        .iter()
        .filter(|u| u.id != "ccna-transport-conversations")
        .collect();
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&retained).unwrap()
    );
    let unit = current
        .iter()
        .position(|u| u.id == "ccna-transport-conversations")
        .unwrap();
    assert!(
        unit > current
            .iter()
            .position(|u| u.id == "ipv4-access-policy")
            .unwrap()
    );
    assert_eq!(
        current[unit].labs,
        vec![
            "ccna-transport-port-pairing",
            "ccna-transport-return-policy"
        ]
    );
    for (track, total) in [
        ("CCNA", 74),
        ("LPIC-1", 101),
        ("LPIC-2", 45),
        ("Bash/Shell", 76),
    ] {
        assert_eq!(after.iter().filter(|s| s.track == track).count(), total);
    }
}
