//! Technical fixture only. No authored scenario or coverage credit.
use dlms_learning_lab::{
    dns_access_workspace::{self as work, Action, Config, Session},
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
};
#[path = "support/dns_access_task.rs"]
mod fixture;
fn scenario() -> pack::Scenario {
    let definition = fixture::definition();
    let mut solution = definition
        .initial
        .reference
        .iter()
        .map(|a| format!("dns-policy {}", serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    solution.push("dns-check".into());
    let value = serde_json::json!({"schema_version":1,"scenarios":[{"id":"dns-policy-capability-fixture","version":1,"track":"LPIC-2","topic":"DNS access boundaries","unit":"dns-policy-model-fixture","title":"Verify current DNS client policy","summary":"Technical fixture, not registered authored curriculum.","objective":"Preserve public DNS while limiting recursive/cache access and the supplied secondary transfer.","difficulty":"Intermediate","minutes":25,"steps":["Inspect the supplied names, client ranges and current policy.","Save, reload, test permitted and denied clients, then check the handover."],"hints":["Saving does not apply rules.","Cached data needs its own access control.","A zone transfer override replaces its global default."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical source binding only; native teaching and independent acceptance remain pending.","simplifications":"Fixed supplied DNS data and positive IPv4 ACLs; no network, BIND process, TSIG or host access.","initial":{"kind":"DnsAccess","simulation":Session::new(&definition).unwrap()},"goal":{"kind":"DnsAccess","definition":definition},"solution":solution,"prerequisites":[]}]});
    pack::load(&serde_json::to_string(&value).unwrap())
        .unwrap()
        .remove(0)
}
fn session(a: &mut Attempt) -> &mut Session {
    let State::DnsAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn engine_checks_actual_current_state_and_does_not_count_example_credit() {
    let s = scenario();
    let Goal::DnsAccess { definition } = &s.goal else {
        panic!()
    };
    let mut a = Attempt::new(&s, 1);
    for action in &definition.example.reference {
        if let Action::Save { config } = action {
            session(&mut a)
                .practice
                .set_editor(config.0.clone())
                .unwrap();
        }
        session(&mut a)
            .practice
            .run(&definition.example, action.clone())
            .unwrap()
            .unwrap();
    }
    session(&mut a).practice.check(&definition.example).unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "dns-check").unwrap();
    assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    assert_eq!(
        session(&mut a)
            .assignment
            .world(&definition.initial)
            .unwrap()
            .zones(),
        fixture::reached(&definition.initial).zones()
    );
    let action = Action::Save {
        config: definition.initial.initial_config.clone(),
    };
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &format!("dns-policy {}", serde_json::to_string(&action).unwrap()),
    )
    .unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn malformed_source_and_unsafe_or_unknown_helpers_refuse_atomically() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    for command in [
        "sh /etc/named.conf",
        "cat /etc/passwd",
        "dns-policy {}",
        "dns-policy {\"kind\":\"reload\",\"shell\":\"id\"}",
        "dns-policy {\"kind\":\"save\",\"config\":\"include \\\"/etc/passwd\\\";\"}",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, command).is_err(),
            "{command}"
        );
        assert_eq!(a.state, before);
    }
    let before = a.state.clone();
    assert!(engine::execute(&mut a.state, "dns-check").is_err());
    assert_eq!(a.state, before);
    let mut bad = s.clone();
    bad.goal = Goal::Subnet;
    assert!(work::source(&bad).is_err());
    let mut bad = s.clone();
    let Goal::DnsAccess { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.reference.pop();
    assert!(work::source(&bad).is_err());
    let mut bad = s.clone();
    let State::DnsAccess { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.editor = Config("invented starter".into());
    assert!(work::source(&bad).is_err());
}
#[test]
fn save_restore_keeps_two_drafts_favorite_and_original_file_on_refusal() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    session(&mut a)
        .assignment
        .set_editor("# pending assignment\noptions {".into())
        .unwrap();
    session(&mut a)
        .practice
        .set_editor("# pending example".into())
        .unwrap();
    session(&mut a).assignment.query_client.0 = "192.".into();
    let mut p = Progress {
        schema_version: 1,
        attempts: std::collections::BTreeMap::from([(s.id.clone(), a)]),
        ..Progress::default()
    };
    let root = std::env::temp_dir().join(format!(
        "dns-access-bindings-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(
        root.starts_with(
            std::env::current_dir()
                .unwrap()
                .join(".build-artifacts/tmp")
        )
    );
    std::fs::create_dir(&root).unwrap();
    let file = root.join("progress.json");
    p.favorites.insert(s.id.clone());
    progress::save_with_catalog(&file, &p, std::slice::from_ref(&s)).unwrap();
    let bytes = std::fs::read(&file).unwrap();
    let loaded = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    let mut a = loaded.attempts[&s.id].clone();
    assert!(loaded.favorites.contains(&s.id));
    assert_eq!(session(&mut a).assignment.query_client.0, "192.");
    assert_eq!(session(&mut a).practice.editor.0, "# pending example");
    let def = match &s.goal {
        Goal::DnsAccess { definition } => definition,
        _ => panic!(),
    };
    session(&mut a)
        .assignment
        .set_editor(fixture::alternate_config())
        .unwrap();
    for action in [
        Action::Save {
            config: session(&mut a).assignment.editor.clone(),
        },
        Action::Reload {},
        def.initial.reference[2].clone(),
    ] {
        session(&mut a)
            .assignment
            .run(&def.initial, action)
            .unwrap()
            .unwrap();
    }
    session(&mut a).assignment.check(&def.initial).unwrap();
    assert!(progress::current_solved(&a, &s));
    p.attempts.insert(s.id.clone(), a);
    assert!(progress::save(&file, &p).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
    p.attempts.get_mut(&s.id).unwrap().scenario_version += 1;
    assert!(progress::save_with_catalog(&file, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
}
#[test]
fn foreign_metadata_unknown_sources_and_forged_checked_state_do_not_import_as_success() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    session(&mut a).assignment.checked = true;
    session(&mut a).assignment.attempted = true;
    assert!(work::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut a = Attempt::new(&s, 1);
    a.workspace = Some(dlms_learning_lab::workspace::WorkspaceData::new());
    a.workspace.as_mut().unwrap().command = "foreign".into();
    assert!(work::context(&a, &s).is_err());
    let p = Progress {
        schema_version: 1,
        attempts: std::collections::BTreeMap::from([(s.id.clone(), Attempt::new(&s, 1))]),
        ..Progress::default()
    };
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &[]).is_err());
}
