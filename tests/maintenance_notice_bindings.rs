//! Technical capability fixtures, not registered authored curriculum.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    maintenance_notice as core, maintenance_notice_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
fn scenario() -> pack::Scenario {
    let def: work::Definition = serde_json::from_str(include_str!(
        "fixtures/maintenance-notice-workspace-v1.json"
    ))
    .unwrap();
    let mut ops = def
        .initial
        .reference
        .iter()
        .map(|action| {
            format!(
                "notice-action {}",
                serde_json::to_string(&work::Reference::Apply {
                    action: action.clone()
                })
                .unwrap()
            )
        })
        .collect::<Vec<_>>();
    ops.push("notice-action {\"kind\":\"check\"}".into());
    let data = serde_json::json!({"schema_version":1,"scenarios":[{"id":"maintenance-notice-model-fixture","version":1,"track":"LPIC-2","topic":"Maintenance notices","unit":"notice-model-fixture","title":"Verify a supplied maintenance handover","summary":"Technical integration fixture; not registered curriculum.","objective":"Correct the notice, audience and pending virtual restart while preserving the pre-login notice.","difficulty":"Intermediate","minutes":20,"steps":["Review the supplied maintenance request.","Repair notices and the virtual schedule; verify every outcome."],"hints":["A saved login notice does not reach existing sessions."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Model fixture; complete native teaching and authored units follow capability validation.","simplifications":"Supplied local writable terminals, a finite notice format and a bounded virtual clock only.","initial":{"kind":"MaintenanceNotice","simulation":work::Session::new(&def).unwrap()},"goal":{"kind":"MaintenanceNotice","definition":def},"solution":ops,"prerequisites":[]}]});
    pack::load(&serde_json::to_string(&data).unwrap())
        .unwrap()
        .remove(0)
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::MaintenanceNotice { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn engine_uses_real_resulting_state_and_never_credits_practice_or_stale_work() {
    let s = scenario();
    let Goal::MaintenanceNotice { definition } = &s.goal else {
        panic!()
    };
    let mut a = Attempt::new(&s, 1);
    assert!(!progress::current_solved(&a, &s));
    for action in &definition.example.reference {
        session(&mut a)
            .practice
            .apply_action(&definition.example, action.clone())
            .unwrap();
    }
    session(&mut a).practice.check(&definition.example).unwrap();
    assert!(!progress::current_solved(&a, &s));
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    session(&mut a).assignment.selected = work::Editor::Motd;
    session(&mut a)
        .assignment
        .edit("Starts: not a time".into())
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
    session(&mut a)
        .assignment
        .save(&definition.initial)
        .unwrap();
    session(&mut a)
        .assignment
        .check(&definition.initial)
        .unwrap();
    let checks = engine::checks(&a.state, &s.goal);
    assert!(checks.iter().any(|c| !c.0 && c.1.contains("missing")));
    assert!(
        checks
            .iter()
            .any(|c| c.0 && c.1.starts_with("Current session"))
    );
    assert!(!engine::solved(&a.state, &s.goal));
}
#[test]
fn malformed_operations_and_wrong_source_are_atomic_and_fail_closed() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    for raw in [
        "sh -c anything",
        "notice-action {}",
        "notice-action {\"kind\":\"apply\",\"action\":{\"kind\":\"host_shell\"}}",
        "cat ../../personal",
        "wall /etc/passwd",
    ] {
        let before = a.state.clone();
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, raw).is_err());
        assert_eq!(a.state, before);
    }
    let mut bad = s.clone();
    let Goal::MaintenanceNotice { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.reference.pop();
    assert!(work::source(&bad).is_err());
    let mut foreign = a.clone();
    foreign.ipv4 = Some(dlms_learning_lab::ipv4::Lesson::from_legacy(
        "192.0.2.1".parse().unwrap(),
        24,
        0,
    ));
    assert!(work::context(&foreign, &s).is_err());
    assert!(!progress::current_solved(&foreign, &s));
}
#[test]
fn validated_store_preserves_both_drafts_and_rejects_incompatible_saves_without_replacing_bytes() {
    let s = scenario();
    let Goal::MaintenanceNotice { definition } = &s.goal else {
        panic!()
    };
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    let mut a = Attempt::new(&s, 1);
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    session(&mut a).practice.selected = work::Editor::Stdin;
    session(&mut a)
        .practice
        .edit("unfinished example".into())
        .unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notice-progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    let mut bad = p;
    session(bad.attempts.get_mut(&s.id).unwrap())
        .assignment
        .task = "foreign-task".into();
    assert!(progress::save_with_catalog(&path, &bad, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    let mut reset = loaded;
    let practice = session(reset.attempts.get_mut(&s.id).unwrap())
        .practice
        .clone();
    session(reset.attempts.get_mut(&s.id).unwrap())
        .reset_assignment(definition)
        .unwrap();
    assert_eq!(
        session(reset.attempts.get_mut(&s.id).unwrap()).practice,
        practice
    );
    assert!(!progress::current_solved(&reset.attempts[&s.id], &s));
}
#[test]
fn forged_pack_starters_and_incompatible_versions_do_not_become_solutions() {
    let s = scenario();
    let mut bad = s.clone();
    let State::MaintenanceNotice { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.version = 2;
    assert!(work::source(&bad).is_err());
    let mut bad = s.clone();
    let State::MaintenanceNotice { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.history.push(core::Action::Broadcast {
        text: "Invented starter evidence".into(),
    });
    assert!(work::source(&bad).is_err());
    let mut pack = pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    };
    pack.scenarios[0].solution.push("cat /etc/shadow".into());
    assert!(pack::load(&serde_json::to_string(&pack).unwrap()).is_err());
}
