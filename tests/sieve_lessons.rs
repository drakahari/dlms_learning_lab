use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    sieve_source as binding,
    sieve_workspace::{Action, Definition, Session},
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/sieve-filter-lessons-v1.json")).unwrap()
}
fn run(s: &mut Session, d: &Definition, script: &str, practice: bool) {
    s.action(
        d,
        &Action::Edit {
            script: script.into(),
        },
        practice,
    )
    .unwrap();
    s.action(d, &Action::Run, practice).unwrap();
    let n = if practice {
        d.example.cases.len()
    } else {
        d.assignment.cases.len()
    };
    for _ in 0..n {
        s.action(d, &Action::Tick, practice).unwrap();
    }
}
#[test]
fn authored_assignments_and_separate_worked_filters_reach_all_cases_without_shared_credit() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    for source in &labs {
        let d = binding::source(source).unwrap();
        assert_eq!(source.guide.len(), 3);
        assert_eq!(source.steps.len(), 3);
        assert!(d.assignment.cases.len() >= 8);
        assert!(d.example.cases.len() >= 4);
        let mut s = Session::new(d).unwrap();
        assert!(!d.solved(&s));
        let original = s.assignment.clone();
        run(&mut s, d, &d.example.reference, true);
        assert!(s.practice.solved(&d.example));
        assert_eq!(s.assignment, original);
        assert!(!d.solved(&s));
        let mut state = State::SieveFiltering {
            simulation: Box::new(s),
        };
        for line in &source.solution {
            engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
        }
        assert!(
            engine::solved(&state, &source.goal),
            "{}: {:?}",
            source.id,
            engine::checks(&state, &source.goal)
        );
        let State::SieveFiltering { simulation } = &mut state else {
            panic!()
        };
        simulation
            .action(
                d,
                &Action::Edit {
                    script: "keep;".into(),
                },
                false,
            )
            .unwrap();
        assert!(!engine::solved(&state, &source.goal));
    }
}
#[test]
fn alternate_supported_control_flow_passes_for_each_distinct_assignment() {
    let alternatives = [
        "require \"fileinto\"; if not header :is \"X-List\" \"staff-updates\" {keep;} else {fileinto \"News\";}",
        "require \"fileinto\"; if header :is \"X-Project\" \"atlas\" { if anyof (size :over 1024, header :is \"X-Review\" \"pending\") {fileinto \"Review\";} }",
        "require \"fileinto\"; if header :is \"X-Priority\" \"urgent\" {fileinto \"Review\";stop;} if header :is \"X-List\" \"staff-updates\" {fileinto \"News\";stop;} keep;",
    ];
    for (source, script) in labs().iter().zip(alternatives) {
        let d = binding::source(source).unwrap();
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, script, false);
        assert!(d.solved(&s), "{}", source.id);
    }
}
#[test]
fn overbroad_exact_size_and_priority_errors_fail_named_cases_with_useful_feedback() {
    let sources = labs();
    let wrong = [
        "require \"fileinto\"; if header :contains \"X-List\" \"staff-updates\" {fileinto \"News\";}",
        "require \"fileinto\"; if allof (header :is \"X-Project\" \"atlas\", anyof (header :is \"X-Review\" \"pending\", not size :under 1K)) {fileinto \"Review\";}",
        "require \"fileinto\"; if header :is \"X-Priority\" \"urgent\" {fileinto \"Review\";} if header :is \"X-List\" \"staff-updates\" {fileinto \"News\";}",
    ];
    for (source, script) in sources.iter().zip(wrong) {
        let d = binding::source(source).unwrap();
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, script, false);
        let rows = d.checks(&s);
        assert!(rows.iter().any(|c| c.passed));
        assert!(rows.iter().any(|c| !c.passed));
        assert!(!d.solved(&s));
        for r in rows.iter().filter(|c| !c.passed) {
            assert!(
                r.detail.contains("Expected:") && r.detail.contains("Actual:"),
                "{}",
                r.detail
            );
        }
    }
    let d = binding::source(&sources[1]).unwrap();
    let mut s = Session::new(d).unwrap();
    run(&mut s, d, wrong[1], false);
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.passed && r.label.contains("exactly 1024"))
    );
    for source in &sources {
        let d = binding::source(source).unwrap();
        for script in [
            "keep;",
            "discard;",
            "require \"fileinto\"; fileinto \"INBOX\";",
        ] {
            assert!(!d.assignment.complete(script));
        }
    }
}
#[test]
fn source_bound_safety_invalid_pack_and_partial_run_do_not_claim_success() {
    for source in labs() {
        let d = binding::source(&source).unwrap();
        let mut state = source.initial.clone();
        let before = serde_json::to_value(&state).unwrap();
        assert!(engine::execute(&mut state, "rm -rf /").is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        assert!(engine::execute_for_goal(&mut state, &source.goal, "sh learner.sh").is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        let State::SieveFiltering { simulation } = &mut state else {
            panic!()
        };
        simulation
            .action(
                d,
                &Action::Edit {
                    script: d.assignment.reference.clone(),
                },
                false,
            )
            .unwrap();
        simulation.action(d, &Action::Run, false).unwrap();
        simulation.action(d, &Action::Tick, false).unwrap();
        assert!(!engine::solved(&state, &source.goal));
    }
    let text = include_str!("../scenarios/sieve-filter-lessons-v1.json");
    let original: serde_json::Value = serde_json::from_str(text).unwrap();
    let mut bad = original.clone();
    bad["scenarios"][0]["goal"]["definition"]["source_id"] = "different".into();
    assert!(pack::load(&bad.to_string()).is_err());
    let mut bad = original.clone();
    bad["scenarios"][0]["goal"]["definition"]["assignment"]["cases"][0]["headers"] =
        serde_json::json!([["Subject", "=?UTF-8?Q?encoded?="]]);
    assert!(pack::load(&bad.to_string()).is_err());
    let mut bad = original;
    bad["scenarios"][0]["initial"]["simulation"]["host_path"] = "/etc/passwd".into();
    assert!(pack::load(&bad.to_string()).is_err());
}
#[test]
fn ordinary_progress_preserves_partial_scripts_and_selected_reset_only() {
    let sources = labs();
    let source = &sources[2];
    let d = binding::source(source).unwrap();
    let mut attempt = Attempt::new(source, 1);
    let State::SieveFiltering { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("# assignment draft\nif header".into())
        .unwrap();
    simulation
        .practice
        .edit("# independent example draft\nkeep;".into())
        .unwrap();
    simulation.practice.selected = 2;
    let expected = simulation.clone();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), attempt);
    p.attempts
        .insert(sources[0].id.clone(), Attempt::new(&sources[0], 2));
    p.favorites.insert(source.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &sources).unwrap();
    let mut restored = progress::load(&path, &sources).unwrap();
    let other = serde_json::to_value(&restored.attempts[&sources[0].id]).unwrap();
    let State::SieveFiltering { simulation } =
        &mut restored.attempts.get_mut(&source.id).unwrap().state
    else {
        panic!()
    };
    assert_eq!(*simulation, expected);
    let example = simulation.practice.clone();
    simulation.action(d, &Action::Reset, false).unwrap();
    assert_eq!(simulation.practice, example);
    assert!(!d.solved(simulation));
    assert!(restored.favorites.contains(&source.id));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&sources[0].id]).unwrap(),
        other
    );
}
