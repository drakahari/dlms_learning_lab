use dlms_learning_lab::{
    engine, pack,
    progress::{self, Attempt, Progress},
    sieve_source as binding,
    sieve_workspace::{Action, Definition, Session},
};
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/sieve-filter-lessons-v1.json"),
        include_str!("../scenarios/sieve-address-lessons-v1.json"),
        include_str!("../scenarios/sieve-delivery-lessons-v1.json"),
        include_str!("../scenarios/sieve-vacation-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "linux-sieve-vacation")
    .collect()
}
fn run(s: &mut Session, d: &Definition, code: &str, practice: bool) {
    s.action(
        d,
        &Action::Edit {
            script: code.into(),
        },
        practice,
    )
    .unwrap();
    s.action(d, &Action::Run, practice).unwrap();
    for _ in 0..12 {
        s.action(d, &Action::Tick, practice).unwrap();
    }
}
#[test]
fn two_distinct_authored_methods_and_examples_pass_without_shared_credit() {
    let sources = labs();
    assert_eq!(sources.len(), 2);
    for (source, n) in sources.iter().zip([11, 10]) {
        let d = binding::source(source).unwrap();
        assert!(d.assignment.delivery_actions);
        assert_eq!(d.assignment.cases.len(), n);
        assert_eq!(d.example.cases.len(), n);
        let mut s = Session::new(d).unwrap();
        let original = s.assignment.clone();
        run(&mut s, d, &d.example.reference, true);
        assert!(s.practice.solved(&d.example));
        assert_eq!(s.assignment, original);
        assert!(!d.solved(&s));
        let mut state = engine::State::SieveFiltering {
            simulation: Box::new(s),
        };
        for action in &source.solution {
            engine::execute_for_goal(&mut state, &source.goal, action).unwrap();
        }
        assert!(
            engine::solved(&state, &source.goal),
            "{} {:?}",
            source.id,
            engine::checks(&state, &source.goal)
        );
    }
}
#[test]
fn equivalent_supported_reply_wording_and_reachable_branching_pass() {
    let alternatives = [
        r#"require "vacation"; if exists "To" {vacation :days 3 :handle "absence" "For help please email cover@absence.test.";} keep;"#,
        r#"require ["vacation","envelope","fileinto"]; if envelope :is "to" "desk@dispatch.away.test" {fileinto "Desk"; vacation :days 1 :handle "away" "Email cover@dispatch.away.test for help.";} else {vacation :days 1 :handle "away" "Email cover@dispatch.away.test for help.";}"#,
    ];
    for (source, code) in labs().iter().zip(alternatives) {
        let d = binding::source(source).unwrap();
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, code, false);
        assert!(d.solved(&s), "{} {:?}", source.id, d.checks(&s));
    }
}
#[test]
fn plausible_wrong_filters_and_every_starter_fail_usefully_then_retry_and_edit_invalidation_work() {
    let wrong = [
        r#"require "vacation";vacation "Please contact cover@absence.test.";"#,
        r#"require ["vacation","envelope","fileinto"];vacation :days 1 "Please contact cover@dispatch.away.test.";if envelope :is "to" "desk@dispatch.away.test" {fileinto "Desk";}keep;"#,
    ];
    for (source, code) in labs().iter().zip(wrong) {
        let d = binding::source(source).unwrap();
        for bad in [&d.assignment.starter, code] {
            let mut s = Session::new(d).unwrap();
            run(&mut s, d, bad, false);
            assert!(!d.solved(&s));
            assert!(
                d.checks(&s)
                    .iter()
                    .any(|c| !c.passed && c.detail.contains("Expected:"))
            );
            assert!(s.assignment.results.iter().any(|r| r.passed));
        }
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, &d.assignment.reference, false);
        assert!(d.solved(&s));
        s.action(
            d,
            &Action::Edit {
                script: "keep;".into(),
            },
            false,
        )
        .unwrap();
        assert!(!d.solved(&s));
        assert!(s.assignment.results.is_empty());
    }
}
#[test]
fn each_guided_experiment_produces_its_taught_failure_without_touching_assignment() {
    for (source, experiment) in labs()
        .iter()
        .zip([(":days 2", ":days 3"), ("}\n", "}\nkeep;\n")])
    {
        let d = binding::source(source).unwrap();
        let mut s = Session::new(d).unwrap();
        let assignment = s.assignment.clone();
        run(&mut s, d, &d.example.starter, true);
        assert!(!s.practice.solved(&d.example));
        let changed = d.example.reference.replace(experiment.0, experiment.1);
        assert_ne!(changed, d.example.reference);
        run(&mut s, d, &changed, true);
        assert!(!s.practice.solved(&d.example));
        assert!(s.practice.results.iter().any(|r| !r.passed));
        assert_eq!(s.assignment, assignment);
        run(&mut s, d, &d.example.reference, true);
        assert!(s.practice.solved(&d.example));
        assert_eq!(s.assignment, assignment);
    }
}
#[test]
fn ordinary_progress_restores_distinct_partial_drafts_and_current_credit_without_fabrication() {
    for source in labs() {
        let d = binding::source(&source).unwrap();
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, &d.example.reference, true);
        run(&mut s, d, &d.assignment.starter, false);
        s.assignment.selected = d.assignment.cases.len() - 1;
        let before = s.clone();
        let mut a = Attempt::new(&source, 1);
        a.state = engine::State::SieveFiltering {
            simulation: Box::new(s),
        };
        let mut p = Progress {
            schema_version: 1,
            clock: 2,
            ..Default::default()
        };
        p.favorites.insert(source.id.clone());
        p.attempts.insert(source.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&source)).unwrap();
        let restored = progress::load(&path, std::slice::from_ref(&source)).unwrap();
        assert!(restored.favorites.contains(&source.id));
        let a = &restored.attempts[&source.id];
        binding::context(a, &source).unwrap();
        assert!(!progress::current_solved(a, &source));
        let engine::State::SieveFiltering { simulation } = &a.state else {
            panic!()
        };
        assert_eq!(**simulation, before);
    }
}

#[test]
fn cold_reopen_fixture_retains_exact_nine_of_ten_real_timeline_results() {
    let source = labs().pop().unwrap();
    let d = binding::source(&source).unwrap();
    let mut s = Session::new(d).unwrap();
    for practice in [false, true] {
        let t = if practice { &d.example } else { &d.assignment };
        let text = if practice {
            t.reference.replace(":days 2", ":days 3")
        } else {
            t.reference.replace(":days 1", ":days 2")
        };
        assert_ne!(text, t.reference);
        run(&mut s, d, &text, practice);
        let scope = if practice { &s.practice } else { &s.assignment };
        assert_eq!(scope.results.len(), 10);
        assert_eq!(scope.results.iter().filter(|r| r.passed).count(), 9);
        assert!(!scope.results[3].passed);
        assert!(matches!(
            scope.results[1].vacation,
            Some(dlms_learning_lab::sieve_vacation_policy::RunResult::Recent { last_sent: 0, .. })
        ));
    }
    d.bound(&s).unwrap();
    assert!(!d.solved(&s));
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
}
