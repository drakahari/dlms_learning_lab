use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    sieve_source as binding,
    sieve_workspace::{Action, Definition, Session},
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/sieve-address-lessons-v1.json")).unwrap()
}
fn run(s: &mut Session, d: &Definition, text: &str, practice: bool) {
    s.action(
        d,
        &Action::Edit {
            script: text.into(),
        },
        practice,
    )
    .unwrap();
    s.action(d, &Action::Run, practice).unwrap();
    for _ in 0..if practice {
        d.example.cases.len()
    } else {
        d.assignment.cases.len()
    } {
        s.action(d, &Action::Tick, practice).unwrap();
    }
}
#[test]
fn both_authored_methods_and_separate_examples_reach_every_case_without_shared_credit() {
    let sources = labs();
    assert_eq!(sources.len(), 2);
    for source in &sources {
        let d = binding::source(source).unwrap();
        assert!(d.assignment.address_tests && d.example.address_tests);
        assert_eq!(source.guide.len(), 3);
        assert!(d.assignment.cases.len() >= 11);
        assert_eq!(d.example.cases.len(), 4);
        let mut session = Session::new(d).unwrap();
        let original = session.assignment.clone();
        run(&mut session, d, &d.example.reference, true);
        assert!(session.practice.solved(&d.example));
        assert_eq!(session.assignment, original);
        assert!(!d.solved(&session));
        let mut state = State::SieveFiltering {
            simulation: Box::new(session),
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
fn genuinely_equivalent_domain_and_full_recipient_rules_pass() {
    let sources = labs();
    let alternatives = [
        vec![
            r#"require "fileinto"; if not address :domain "From" "partners.dispatch.test" {keep;} else {fileinto "Partners";}"#,
            r#"require "fileinto"; if address :matches "From" "*@partners.dispatch.test" {fileinto "Partners"; stop;}"#,
        ],
        vec![
            r#"require "fileinto"; if anyof (address :is "To" "helpdesk@service.dispatch.test", address :is "Cc" "helpdesk@service.dispatch.test") {fileinto "Support";}"#,
            r#"require "fileinto"; if address :all :matches ["To","Cc"] "helpdesk@service.dispatch.test" {fileinto "Support";}"#,
        ],
    ];
    for (source, options) in sources.iter().zip(alternatives) {
        let d = binding::source(source).unwrap();
        for text in options {
            let mut s = Session::new(d).unwrap();
            run(&mut s, d, text, false);
            assert!(d.solved(&s), "{}: {:?}", source.id, d.checks(&s));
        }
    }
}
#[test]
fn plausible_broad_or_cross_mailbox_conditions_fail_specific_cases_with_feedback() {
    let sources = labs();
    let wrong = [
        r#"require "fileinto"; if header :contains "From" "partners.dispatch.test" {fileinto "Partners";}"#,
        r#"require "fileinto"; if allof (address :localpart ["To","Cc"] "helpdesk",address :domain ["To","Cc"] "service.dispatch.test") {fileinto "Support";}"#,
    ];
    for (source, text) in sources.iter().zip(wrong) {
        let d = binding::source(source).unwrap();
        let mut session = Session::new(d).unwrap();
        run(&mut session, d, text, false);
        let rows = d.checks(&session);
        assert!(rows.iter().any(|c| c.passed));
        assert!(rows.iter().any(|c| !c.passed));
        assert!(!d.solved(&session));
        for row in rows.iter().filter(|c| !c.passed) {
            assert!(
                row.detail.contains("Expected:")
                    && row.detail.contains("Actual:")
                    && row.detail.contains("mailbox"),
                "{}",
                row.detail
            );
        }
    }
    let recipient = binding::source(&sources[1]).unwrap();
    let mut s = Session::new(recipient).unwrap();
    run(&mut s, recipient, wrong[1], false);
    let index = recipient
        .assignment
        .cases
        .iter()
        .position(|c| c.id == "split-parts")
        .unwrap();
    assert!(!s.assignment.results[index].passed);
    for source in &sources {
        let d = binding::source(source).unwrap();
        for wrong in [
            "keep;",
            "discard;",
            &format!(
                "require \"fileinto\"; fileinto \"{}\";",
                if source.id.contains("sender") {
                    "Partners"
                } else {
                    "Support"
                }
            ),
            &format!("{}\nkeep;", d.assignment.reference),
        ] {
            assert!(!d.assignment.complete(wrong), "{}: {wrong}", source.id);
        }
        assert!(!d.assignment.complete(&d.assignment.starter));
    }
}
#[test]
fn each_guided_experiment_produces_the_taught_observation_and_can_be_repaired() {
    let sources = labs();
    for source in &sources {
        let d = binding::source(source).unwrap();
        let reference = &d.example.reference;
        assert!(d.example.complete(reference));
        let wrong = if source.id.contains("sender") {
            reference.replace("address :domain :is", "header :contains")
        } else {
            reference
                .replace(":is", ":contains")
                .replace("visits@clinic.guide.test", "visits")
        };
        let mut s = Session::new(d).unwrap();
        run(&mut s, d, &wrong, true);
        assert!(s.practice.results.iter().any(|r| !r.passed));
        assert!(!d.solved(&s));
        run(&mut s, d, reference, true);
        assert!(s.practice.solved(&d.example));
        assert!(!d.solved(&s));
        let changed_field = if source.id.contains("sender") {
            reference.replace("\"From\"", "\"To\"")
        } else {
            reference.replace("[\"To\", \"Cc\"]", "[\"To\"]")
        };
        assert!(!d.example.complete(&changed_field));
    }
}
#[test]
fn normal_progress_retains_separate_partial_runs_and_reset_only_changes_selected_scope() {
    let sources = labs();
    let source = &sources[1];
    let d = binding::source(source).unwrap();
    let mut attempt = Attempt::new(source, 1);
    let State::SieveFiltering { simulation } = &mut attempt.state else {
        panic!()
    };
    run(simulation, d, &d.assignment.starter, false);
    simulation
        .practice
        .edit(format!("# guided draft\n{}", d.example.reference))
        .unwrap();
    simulation.practice.selected = 3;
    simulation.assignment.selected = 8;
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
    assert!(restored.favorites.contains(&source.id));
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
    assert_eq!(
        serde_json::to_value(&restored.attempts[&sources[0].id]).unwrap(),
        other
    );
}
#[test]
fn invalid_profiles_and_unsupported_message_forms_fail_before_registration() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-address-lessons-v1.json")).unwrap();
    let mut bad = raw.clone();
    bad["scenarios"][0]["goal"]["definition"]["example"]["address_tests"] = false.into();
    assert!(pack::load(&bad.to_string()).is_err());
    let mut bad = raw.clone();
    bad["scenarios"][0]["goal"]["definition"]["assignment"]["cases"][0]["headers"] =
        serde_json::json!([["From", "Team: staff@partners.dispatch.test;"]]);
    assert!(pack::load(&bad.to_string()).is_err());
    let mut bad = raw;
    bad["scenarios"][1]["initial"]["simulation"]["address_tests"] = true.into();
    assert!(pack::load(&bad.to_string()).is_err());
    for source in labs() {
        let mut state = source.initial.clone();
        let before = serde_json::to_value(&state).unwrap();
        assert!(engine::execute(&mut state, "sh learner.sh").is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
}
