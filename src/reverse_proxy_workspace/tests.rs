use super::*;
fn task(id: &str, prefix: &str, address: &str) -> Task {
    let endpoint = model::Endpoint {
        address: address.parse().unwrap(),
        port: 9000,
    };
    Task {
        version: 1,
        id: id.into(),
        port: 8080,
        initial_config: format!(
            "server {{ listen 8080; location / {{ return 404; }} location /{prefix}/ {{ proxy_pass http://{address}:9000; }} }}"
        ),
        required_config: format!(
            "server {{ listen 8080; location / {{ return 404; }} location /{prefix}/ {{ proxy_pass http://{address}:9000/; }} }}"
        ),
        cases: vec![
            Case {
                id: "document".into(),
                label: "Document path".into(),
                path: format!("/{prefix}/daily.csv"),
                expected: Expected::Forward {
                    endpoint,
                    path: "/daily.csv".into(),
                },
            },
            Case {
                id: "bare-prefix".into(),
                label: "Bare path".into(),
                path: format!("/{prefix}"),
                expected: Expected::Redirect {
                    path: format!("/{prefix}/"),
                },
            },
            Case {
                id: "fallback".into(),
                label: "Unrelated public path".into(),
                path: "/other".into(),
                expected: Expected::Return {
                    status: 404,
                    body: None,
                },
            },
        ],
    }
}
pub(super) fn fixture() -> Definition {
    Definition {
        initial: task("routing-assignment", "reports", "192.0.2.20"),
        example: task("routing-example", "charts", "198.51.100.30"),
    }
}
fn solve(d: &mut Draft, t: &Task) {
    d.edit(t.required_config.clone()).unwrap();
    d.save(t).unwrap();
    d.validate_saved(t).unwrap();
    d.apply(t).unwrap();
    assert!(d.check(t).unwrap().passed());
}
#[test]
fn source_reachability_examples_and_assignment_are_independent() {
    let def = fixture();
    def.validate().unwrap();
    let mut s = Session::new(&def).unwrap();
    let assignment = s.assignment.clone();
    assert!(def.checks(&s, false).iter().any(|r| !r.0));
    solve(&mut s.practice, &def.example);
    assert!(def.checks(&s, true).iter().all(|r| r.0));
    assert_eq!(s.assignment, assignment);
    assert!(def.checks(&s, false).iter().any(|r| !r.0));
    solve(&mut s.assignment, &def.initial);
    assert!(def.checks(&s, false).iter().all(|r| r.0));
    let practice = s.practice.clone();
    s.assignment = Draft::new(&def.initial).unwrap();
    assert_eq!(s.practice, practice);
    assert!(def.checks(&s, false).iter().any(|r| !r.0));
}
#[test]
fn unsaved_saved_applied_and_checked_are_distinct() {
    let def = fixture();
    let t = &def.initial;
    let mut d = Draft::new(t).unwrap();
    d.edit(t.required_config.clone()).unwrap();
    let r = d.check(t).unwrap();
    assert!(!r.passed());
    assert!(r.configuration_error.unwrap().contains("unsaved"));
    d.save(t).unwrap();
    d.validate_saved(t).unwrap();
    assert!(!d.checked());
    let r = d.check(t).unwrap();
    assert!(!r.passed());
    assert!(!r.policies[0].passed);
    assert!(r.policies[1].passed);
    assert!(r.cases.iter().any(|c| !c.passed));
    d.apply(t).unwrap();
    assert!(!d.checked());
    assert!(d.check(t).unwrap().passed());
    let stamp = d.checked_stamp.clone();
    d.selected_case = t.cases[1].id.clone();
    assert!(d.checked());
    assert_eq!(stamp, d.checked_stamp);
    d.edit(t.initial_config.clone()).unwrap();
    assert!(!d.checked());
}
#[test]
fn correct_case_results_survive_overall_failure_and_feedback_explains_path() {
    let t = fixture().initial;
    let mut d = Draft::new(&t).unwrap();
    let r = d.check(&t).unwrap();
    assert!(!r.passed());
    assert!(!r.cases[0].passed);
    assert!(r.cases[0].explanation.contains("omitting the URI keeps"));
    assert!(r.cases[1].passed && r.cases[2].passed);
    assert!(r.cases[0].explanation.contains("/reports/daily.csv"));
}
#[test]
fn invalid_saved_text_does_not_replace_applied_routes_and_retry_works() {
    let t = fixture().initial;
    let mut d = Draft::new(&t).unwrap();
    let applied = d.world.applied().to_owned();
    for (text, kind) in [
        ("server { location / { return 404 } }", "Syntax"),
        ("http { server {} }", "Unsupported"),
    ] {
        d.edit(text.into()).unwrap();
        d.save(&t).unwrap();
        assert!(d.validate_saved(&t).is_err());
        assert!(d.apply(&t).is_err());
        assert_eq!(d.world.applied(), applied);
        let r = d.check(&t).unwrap();
        assert!(!r.passed());
        assert!(r.configuration_error.unwrap().contains(kind));
    }
    solve(&mut d, &t);
}
#[test]
fn full_path_requirement_rejects_hidden_leaks_even_when_all_named_cases_pass() {
    let t = fixture().initial;
    let mut d = Draft::new(&t).unwrap();
    let text = t.required_config.replace(
        "location / { return 404; }",
        "location / { return 404; } location /secret { return 200 'leaked'; }",
    );
    d.edit(text).unwrap();
    d.save(&t).unwrap();
    d.apply(&t).unwrap();
    let r = d.check(&t).unwrap();
    assert!(r.cases.iter().all(|c| c.passed));
    assert!(!r.passed());
    assert!(r.policies.iter().all(|p| !p.passed));
    assert!(r.policies[0].difference.is_some());
}
#[test]
fn equivalent_supported_configurations_and_unconstrained_error_text_pass() {
    let t = fixture().initial;
    let mut d = Draft::new(&t).unwrap();
    let text = t
        .required_config
        .replace("return 404;", "return 404 'Not here';")
        .replace("location /reports/", "location ^~ /reports/")
        .replace("192.0.2.20:9000/;", "192.0.2.20:09000/;");
    assert_ne!(text, t.required_config);
    d.edit(text).unwrap();
    d.save(&t).unwrap();
    d.apply(&t).unwrap();
    assert!(d.check(&t).unwrap().passed());
}
#[test]
fn changed_work_and_forged_checked_stamp_never_preserve_current_success() {
    let def = fixture();
    let mut s = Session::new(&def).unwrap();
    solve(&mut s.assignment, &def.initial);
    s.assignment.editor = def.initial.initial_config.clone();
    assert!(!s.assignment.checked());
    assert!(def.checks(&s, false).iter().any(|r| !r.0));
    s.assignment.world = model::World::new(&def.initial.initial_config).unwrap();
    s.assignment.checked_stamp = Some(s.assignment.stamp());
    assert!(s.assignment.checked());
    assert!(def.checks(&s, false).iter().any(|r| !r.0));
}
#[test]
fn real_serialized_roundtrip_retains_unfinished_separate_drafts_without_credit() {
    let def = fixture();
    let mut s = Session::new(&def).unwrap();
    solve(&mut s.practice, &def.example);
    solve(&mut s.assignment, &def.initial);
    s.assignment
        .edit("# unfinished assignment\nserver {".into())
        .unwrap();
    s.assignment.save(&def.initial).unwrap();
    s.practice
        .edit("# different unfinished example\nserver".into())
        .unwrap();
    s.view = View::Assignment;
    let bytes = serde_json::to_vec(&s).unwrap();
    let read: Session = serde_json::from_slice(&bytes).unwrap();
    read.validate(&def).unwrap();
    assert_eq!(s, read);
    assert!(def.checks(&read, false).iter().any(|r| !r.0));
    let mut changed = def.clone();
    changed.initial.cases[0].label.push_str(" changed");
    assert!(read.validate(&changed).is_err());
    assert_eq!(serde_json::to_vec(&read).unwrap(), bytes);
}
#[test]
fn invalid_sources_and_progress_shapes_fail_without_changing_original() {
    let def = fixture();
    let s = Session::new(&def).unwrap();
    let mut bad = def.clone();
    bad.initial.cases[0].expected = Expected::Return {
        status: 200,
        body: None,
    };
    assert!(bad.validate().is_err());
    let mut bad = def.clone();
    bad.initial.initial_config = bad.initial.required_config.clone();
    assert!(bad.validate().is_err());
    let mut bad = def.clone();
    bad.example = bad.initial.clone();
    assert!(bad.validate().is_err());
    let mut bad = def.clone();
    bad.initial.cases[1].path = bad.initial.cases[0].path.clone();
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.assignment.selected_case = "absent".into();
    assert!(bad.validate(&def).is_err());
    let mut bad = s.clone();
    bad.practice.task_id = def.initial.id.clone();
    assert!(bad.validate(&def).is_err());
    let mut bad = s.clone();
    bad.version = 2;
    assert!(bad.validate(&def).is_err());
    let before = serde_json::to_vec(&s).unwrap();
    let mut bad = s.clone();
    assert!(bad.assignment.edit("x".repeat(32769)).is_err());
    assert_eq!(serde_json::to_vec(&bad).unwrap(), before);
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["editor"] = serde_json::json!("x".repeat(32769));
    assert!(serde_json::from_value::<Session>(value).is_err());
    let mut value = serde_json::to_value(&def).unwrap();
    value["initial"]["cases"] = serde_json::json!(vec![&def.initial.cases[0]; 9]);
    assert!(serde_json::from_value::<Definition>(value).is_err());
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
    let d = fixture();
    s.id = d.initial.id.clone();
    s.version = 1;
    s.prerequisites.clear();
    s.initial = crate::engine::State::ReverseProxy {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.goal = crate::engine::Goal::ReverseProxy {
        definition: Box::new(d.clone()),
    };
    s.solution = vec![
        Action::Edit {
            config: d.initial.required_config,
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
    let crate::engine::State::ReverseProxy { simulation } = &mut state else {
        panic!()
    };
    simulation
        .assignment
        .edit("server { listen 8080; location / { return 403; } }".into())
        .unwrap();
    assert!(!crate::engine::solved(&state, &s.goal));
    let snapshot = state.clone();
    assert!(crate::engine::execute_for_goal(&mut state, &s.goal, "nginx -s reload").is_err());
    assert_eq!(state, snapshot);
    for bad in [
        format!("{PREFIX}{{\"action\":\"apply\",\"unknown\":true}}"),
        format!("{PREFIX}{{\"action\":\"exec\",\"command\":\"cat /etc/passwd\"}}"),
        "@reverse-proxy {}\n".into(),
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
    let crate::engine::State::ReverseProxy { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.checked_stamp = Some(simulation.assignment.stamp());
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    let crate::engine::Goal::ReverseProxy { definition } = &mut bad.goal else {
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
    let crate::engine::State::ReverseProxy { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .practice
        .edit("# unfinished practice\nserver {".into())
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
        .edit(d.example.required_config.clone())
        .unwrap();
    session.practice.save(&d.example).unwrap();
    session.practice.apply(&d.example).unwrap();
    session.practice.check(&d.example).unwrap();
    let mut state = crate::engine::State::ReverseProxy {
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
