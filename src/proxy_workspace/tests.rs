use super::*;
fn task(id: &str, network: &str, client: &str) -> Task {
    let cases = vec![
        Case {
            id: "staff".into(),
            label: "Staff web request".into(),
            request: Request {
                client: client.parse().unwrap(),
                method: "GET".into(),
                destination_port: 80,
            },
            expected: DecisionAction::Allow,
        },
        Case {
            id: "visitor".into(),
            label: "Visitor refused".into(),
            request: Request {
                client: "198.51.100.7".parse().unwrap(),
                method: "GET".into(),
                destination_port: 80,
            },
            expected: DecisionAction::Deny,
        },
        Case {
            id: "unsafe-port".into(),
            label: "Unsafe port refused".into(),
            request: Request {
                client: client.parse().unwrap(),
                method: "GET".into(),
                destination_port: 25,
            },
            expected: DecisionAction::Deny,
        },
    ];
    Task {
        version: 1,
        id: id.into(),
        initial_config: "http_access allow all".into(),
        required_policy: format!(
            "acl staff src {network}\nacl web port 80 443\nhttp_access allow staff web\nhttp_access deny all"
        ),
        cases,
    }
}
fn definition() -> Definition {
    Definition {
        initial: task("proxy-assignment", "192.0.2.0/24", "192.0.2.7"),
        example: task("proxy-practice", "203.0.113.0/24", "203.0.113.8"),
    }
}
#[test]
fn editor_save_and_application_are_meaningful_distinct_requirements() {
    let def = definition();
    let mut d = Draft::new(&def.initial).unwrap();
    let r = d.check(&def.initial).unwrap();
    assert!(!r.passed());
    assert!(r.cases[0].passed);
    assert!(!r.cases[1].passed);
    d.edit(def.initial.required_policy.clone()).unwrap();
    assert!(!d.checked());
    assert!(
        d.check(&def.initial)
            .unwrap()
            .configuration_error
            .unwrap()
            .contains("unsaved")
    );
    d.save(&def.initial).unwrap();
    let r = d.check(&def.initial).unwrap();
    assert!(!r.passed());
    assert!(!r.policies[0].passed);
    assert!(r.policies[1].passed);
    d.apply(&def.initial).unwrap();
    assert!(d.check(&def.initial).unwrap().passed());
    assert!(d.checked());
    d.edit("http_access allow all".into()).unwrap();
    assert!(!d.checked());
    assert!(d.check(&def.initial).unwrap().configuration_error.is_some());
}
#[test]
fn valid_alternative_subnets_pass_and_hardcoded_case_clients_fail_full_policy() {
    let def = definition();
    let mut d = Draft::new(&def.initial).unwrap();
    let alternate = "acl staff src 192.0.2.0/25 192.0.2.128/25\nacl web port 80 443\nhttp_access deny !staff\nhttp_access deny !web\nhttp_access allow all";
    d.edit(alternate.into()).unwrap();
    d.save(&def.initial).unwrap();
    d.apply(&def.initial).unwrap();
    assert!(d.check(&def.initial).unwrap().passed());
    d.edit("acl one src 192.0.2.7\nacl web port 80 443\nhttp_access allow one web\nhttp_access deny all".into()).unwrap();
    d.save(&def.initial).unwrap();
    d.apply(&def.initial).unwrap();
    let r = d.check(&def.initial).unwrap();
    assert!(r.cases.iter().all(|c| c.passed));
    assert!(!r.passed());
    assert!(r.policies.iter().all(|p| p.counterexample.is_some()));
}
#[test]
fn separate_practice_and_request_inspection_do_not_award_assignment_credit() {
    let def = definition();
    let mut s = Session::new(&def).unwrap();
    let original = s.assignment.clone();
    s.practice
        .edit(def.example.required_policy.clone())
        .unwrap();
    s.practice.save(&def.example).unwrap();
    s.practice.apply(&def.example).unwrap();
    assert!(s.practice.check(&def.example).unwrap().passed());
    assert!(def.checks(&s, true).iter().all(|c| c.0));
    assert_eq!(s.assignment, original);
    assert!(def.checks(&s, false).iter().any(|c| !c.0));
    s.assignment.inspect(&def.initial).unwrap();
    assert_eq!(s.assignment, original);
}
#[test]
fn syntax_failure_invalid_activation_and_bad_save_preserve_effective_state_and_draft() {
    let def = definition();
    let mut d = Draft::new(&def.initial).unwrap();
    let old = d.world.applied().to_owned();
    d.edit("include /etc/passwd".into()).unwrap();
    d.save(&def.initial).unwrap();
    let saved = d.clone();
    assert!(
        d.validate_saved(&def.initial)
            .unwrap_err()
            .contains("Only acl")
    );
    assert!(d.apply(&def.initial).is_err());
    assert_eq!(d, saved);
    assert_eq!(d.world.applied(), old);
    assert!(
        d.check(&def.initial)
            .unwrap()
            .configuration_error
            .unwrap()
            .contains("Saved configuration")
    );
    assert!(d.edit("x".repeat(8193)).is_err());
    // A permitted character count can still exceed the smaller saved-byte budget.
    d.edit("é".repeat(4097)).unwrap();
    let before = d.clone();
    assert!(d.save(&def.initial).is_err());
    assert_eq!(d, before);
}
#[test]
fn source_validation_rejects_pre_solved_contradictory_negative_only_and_oversized_tasks() {
    let d = definition();
    d.validate().unwrap();
    let mut bad = d.clone();
    bad.initial.initial_config = bad.initial.required_policy.clone();
    assert!(bad.validate().unwrap_err().contains("starter"));
    let mut bad = d.clone();
    bad.initial.cases[0].expected = DecisionAction::Deny;
    assert!(bad.validate().unwrap_err().contains("disagrees"));
    let mut bad = d.clone();
    bad.initial.cases.remove(0);
    assert!(bad.validate().unwrap_err().contains("both"));
    let mut bad = d.clone();
    bad.initial.cases[1].id = bad.initial.cases[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases = vec![bad.initial.cases[0].clone(); 9];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example = bad.initial.clone();
    assert!(bad.validate().is_err());
}
#[test]
fn serde_resume_keeps_drafts_and_success_is_recomputed_from_current_state() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment
        .edit(d.initial.required_policy.clone())
        .unwrap();
    s.assignment.save(&d.initial).unwrap();
    s.assignment.apply(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.practice
        .edit("# unfinished\nhttp_access deny all".into())
        .unwrap();
    let json = serde_json::to_string(&s).unwrap();
    let mut restored: Session = serde_json::from_str(&json).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    assert!(d.checks(&restored, false).iter().all(|c| c.0));
    restored
        .assignment
        .world
        .save("http_access allow all")
        .unwrap();
    restored.assignment.world.apply_saved().unwrap();
    restored.assignment.editor = "http_access allow all".into();
    assert!(!restored.assignment.checked());
    assert!(d.checks(&restored, false).iter().any(|c| !c.0));
    restored.assignment.checked_stamp = Some(restored.assignment.stamp());
    assert!(d.checks(&restored, false).iter().any(|c| !c.0));
    restored.assignment.selected_case = "foreign".into();
    assert!(restored.validate(&d).is_err());
}

fn scenario() -> crate::pack::Scenario {
    static BASE: std::sync::OnceLock<crate::pack::Scenario> = std::sync::OnceLock::new();
    let mut s = BASE
        .get_or_init(|| {
            crate::pack::current()
                .unwrap()
                .into_iter()
                .find(|s| matches!(s.initial, crate::engine::State::SnmpMonitor { .. }))
                .unwrap()
        })
        .clone();
    let d = definition();
    s.id = d.initial.id.clone();
    s.version = 1;
    s.prerequisites.clear();
    s.initial = crate::engine::State::ProxyPolicy {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.goal = crate::engine::Goal::ProxyPolicy {
        definition: Box::new(d.clone()),
    };
    s.solution = vec![
        Action::Edit {
            config: d.initial.required_policy,
        },
        Action::Save {},
        Action::Validate {},
        Action::Apply {},
        Action::Check {},
    ]
    .into_iter()
    .map(|a| format!("{PREFIX}{}", serde_json::to_string(&a).unwrap()))
    .collect();
    s
}
#[test]
fn engine_and_catalog_binding_require_actual_state_not_source_command_spelling() {
    let s = scenario();
    source(&s).unwrap();
    let mut state = s.initial.clone();
    assert!(!crate::engine::solved(&state, &s.goal));
    for a in &s.solution {
        crate::engine::execute_for_goal(&mut state, &s.goal, a).unwrap();
    }
    assert!(crate::engine::solved(&state, &s.goal));
    let crate::engine::State::ProxyPolicy { simulation } = &mut state else {
        panic!()
    };
    simulation
        .assignment
        .edit("http_access allow all".into())
        .unwrap();
    assert!(!crate::engine::solved(&state, &s.goal));
    let snapshot = state.clone();
    assert!(crate::engine::execute_for_goal(&mut state, &s.goal, "squid -k reconfigure").is_err());
    assert_eq!(state, snapshot);
    for bad in [
        format!("{PREFIX}{{\"action\":\"apply\",\"unknown\":true}}"),
        format!("{PREFIX}{{\"action\":\"exec\",\"command\":\"cat /etc/passwd\"}}"),
        "@proxy-policy {}\n".into(),
    ] {
        assert!(crate::engine::execute_for_goal(&mut state, &s.goal, &bad).is_err());
        assert_eq!(state, snapshot);
    }
}
#[test]
fn pack_rejects_prechecked_sources_wrong_goal_identity_and_unknown_fields() {
    let s = scenario();
    let text =
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s.clone()]}))
            .unwrap();
    crate::pack::load_catalog(&[&text]).unwrap();
    let mut bad = s.clone();
    let crate::engine::State::ProxyPolicy { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.checked_stamp = Some(simulation.assignment.stamp());
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    let crate::engine::Goal::ProxyPolicy { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.id = "foreign".into();
    assert!(source(&bad).is_err());
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["initial"]["simulation"]["assignment"]["world"]["host_path"] =
        serde_json::json!("/etc/passwd");
    assert!(serde_json::from_value::<crate::pack::Scenario>(raw).is_err());
}
#[test]
fn actual_disk_progress_preserves_separate_drafts_favorite_reset_and_failed_save_bytes() {
    let s = scenario();
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = crate::progress::Attempt::new(&s, 7);
    crate::workspace::prepare(&mut a);
    for action in &s.solution {
        crate::engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    let crate::engine::State::ProxyPolicy { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .practice
        .edit("# unfinished practice\nhttp_access deny all".into())
        .unwrap();
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(
        crate::progress::save(&path, &p)
            .unwrap_err()
            .contains("authored catalog")
    );
    crate::progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let original = std::fs::read(&path).unwrap();
    let loaded = crate::progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(crate::progress::current_solved(&loaded.attempts[&s.id], &s));
    assert!(loaded.favorites.contains(&s.id));
    let mut bad: crate::progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    bad.attempts.get_mut(&s.id).unwrap().scenario_version += 1;
    assert!(crate::progress::save_with_catalog(&path, &bad, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut reset = loaded;
    reset
        .attempts
        .insert(s.id.clone(), crate::progress::Attempt::new(&s, 8));
    assert!(!crate::progress::current_solved(&reset.attempts[&s.id], &s));
    assert!(reset.favorites.contains(&s.id));
}
#[test]
fn source_references_are_atomic_and_example_success_does_not_satisfy_engine_goal() {
    let s = scenario();
    let d = source(&s).unwrap();
    let mut session = Session::new(d).unwrap();
    session
        .practice
        .edit(d.example.required_policy.clone())
        .unwrap();
    session.practice.save(&d.example).unwrap();
    session.practice.apply(&d.example).unwrap();
    session.practice.check(&d.example).unwrap();
    let mut state = crate::engine::State::ProxyPolicy {
        simulation: Box::new(session),
    };
    assert!(!crate::engine::solved(&state, &s.goal));
    let before = state.clone();
    let raw = format!(
        "{PREFIX}{}",
        serde_json::to_string(&Action::Edit {
            config: "bad\0text".into()
        })
        .unwrap()
    );
    assert!(crate::engine::execute_for_goal(&mut state, &s.goal, &raw).is_err());
    assert_eq!(state, before);
}
#[test]
fn full_task_snapshot_binding_detects_changed_source_even_with_reused_identifiers() {
    let d = definition();
    let s = Session::new(&d).unwrap();
    let mut changed = d.clone();
    changed
        .initial
        .required_policy
        .push_str("\n# authored source changed\n");
    changed.validate().unwrap();
    assert!(s.validate(&changed).unwrap_err().contains("task changed"));
    let mut bad = s.clone();
    bad.source.example.cases[0].label = "Changed supplied request".into();
    assert!(bad.validate(&d).is_err());
}
