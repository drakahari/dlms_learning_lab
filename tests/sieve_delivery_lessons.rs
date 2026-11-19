use dlms_learning_lab::{
    engine, pack,
    progress::{self, Attempt, Progress},
    sieve_source as binding,
    sieve_workspace::{Action, Definition, Session},
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/sieve-delivery-lessons-v1.json")).unwrap()
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
fn three_distinct_authored_methods_and_examples_pass_without_shared_credit() {
    let sources = labs();
    assert_eq!(sources.len(), 3);
    for (source, n) in sources.iter().zip([8, 8, 11]) {
        let d = binding::source(source).unwrap();
        assert!(d.assignment.delivery_actions);
        assert_eq!(d.assignment.cases.len(), n);
        assert_eq!(d.example.cases.len(), 4);
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
fn equivalent_supported_branching_forwarding_and_refusal_wording_pass() {
    let alternatives = [
        r#"require ["envelope","fileinto"];if not envelope :is "to" "tickets@dispatch.test" {keep;} else {fileinto "Tickets";}"#,
        r#"require "envelope";if envelope :matches "to" "alerts@dispatch.test" {redirect "archive@DISPATCH.TEST";stop;}keep;"#,
        r#"require ["envelope","reject","fileinto"];if envelope "to" "retired@dispatch.test" {reject "Please contact the new desk.";stop;}if header "Subject" "Dispatch update" {fileinto "Updates";}"#,
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
        r#"require ["envelope","fileinto"];if envelope :contains "to" "tickets" {fileinto "Tickets";}"#,
        r#"redirect "archive@dispatch.test";"#,
        r#"require ["envelope","fileinto"];if envelope "to" "retired@dispatch.test" {discard;stop;}if header "Subject" "Dispatch update" {fileinto "Updates";}"#,
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
    for (source, experiment) in labs().iter().zip([
        ("envelope :is \"to\"", "envelope :is \"from\""),
        (
            "redirect \"album@studio.guide.test\"",
            "redirect \"photos@studio.guide.test\"",
        ),
        ("elsif header", "if header"),
    ]) {
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
