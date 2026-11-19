use dlms_learning_lab::{
    script_teaching::View,
    sieve_workspace::{Action, Case, Definition, Scope, Session, Task},
};
use std::collections::BTreeSet;
fn boxes(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).into()).collect()
}
fn task(header: &str, key: &str) -> Task {
    let cases = [
        ("matched", key.to_owned(), "Notices"),
        ("mixed-case", key.to_uppercase(), "Notices"),
        ("different", "elsewhere".into(), "INBOX"),
        ("substring", format!("not-{key}"), "INBOX"),
        ("empty", String::new(), "INBOX"),
    ]
    .into_iter()
    .map(|(id, value, expected)| Case {
        id: id.into(),
        label: format!("{id} sample"),
        headers: vec![(header.into(), value)],
        body: "Sample message.\n".into(),
        expected: boxes(&[expected]),
        envelope: None,
        expected_redirects: BTreeSet::new(),
        expected_rejection: false,
        arrival_seconds: None,
        expected_reply: None,
    })
    .collect();
    Task {
        legacy_recipe: None,
        vacation: None,
        address_tests: false,
        delivery_actions: false,
        redirect_targets: BTreeSet::new(),
        starter: "# File selected mail and keep everything else.\nkeep;\n".into(),
        mailboxes: boxes(&["INBOX", "Notices"]),
        cases,
        reference: format!(
            "require \"fileinto\"; if header :is \"{header}\" \"{key}\" {{ fileinto \"Notices\"; }}"
        ),
    }
}
fn definition() -> Definition {
    Definition {
        schema: 1,
        source_id: "filter-fixture".into(),
        assignment: task("X-List", "training"),
        example: task("X-Route", "laboratory"),
    }
}
fn finish(s: &mut Scope, t: &Task) {
    s.start();
    for _ in 0..t.cases.len() {
        s.tick(t);
    }
    assert!(!s.running);
}
#[test]
fn validation_checks_reachability_initial_work_and_separate_inputs() {
    let d = definition();
    d.validate().unwrap();
    let mut bad = d.clone();
    bad.assignment.reference = "discard;".into();
    assert!(bad.validate().unwrap_err().contains("method"));
    let mut bad = d.clone();
    bad.assignment.starter = bad.assignment.reference.clone();
    assert!(bad.validate().unwrap_err().contains("meaningful"));
    let mut bad = d.clone();
    bad.example.cases = bad.assignment.cases.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.cases[1].id = bad.assignment.cases[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.cases[0].expected = boxes(&["Missing"]);
    assert!(bad.validate().is_err());
    let mut bad = d;
    bad.assignment.cases.truncate(1);
    assert!(bad.validate().is_err());
}
#[test]
fn ticks_are_bounded_and_stop_preserves_partial_work_for_resume() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::Edit {
            script: d.assignment.reference.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Run, false).unwrap();
    assert!(s.assignment.running);
    assert!(s.assignment.results.is_empty());
    assert!(!d.solved(&s));
    s.action(&d, &Action::Tick, false).unwrap();
    assert_eq!(s.assignment.results.len(), 1);
    assert!(!d.solved(&s));
    s.action(&d, &Action::Stop, false).unwrap();
    assert!(!s.assignment.running);
    assert!(s.assignment.stopped);
    assert!(!d.solved(&s));
    let before = s.clone();
    s.action(&d, &Action::Tick, false).unwrap();
    assert_eq!(s, before);
    let mut restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    restored.action(&d, &Action::Run, false).unwrap();
    assert!(restored.assignment.results.is_empty());
    for _ in 0..d.assignment.cases.len() {
        restored.action(&d, &Action::Tick, false).unwrap();
    }
    assert!(d.solved(&restored));
    assert!(restored.assignment.checked);
    assert!(!restored.assignment.stopped);
}
#[test]
fn practice_never_awards_credit_and_edits_clear_stale_success() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    s.action(
        &d,
        &Action::Edit {
            script: d.example.reference.clone(),
        },
        true,
    )
    .unwrap();
    finish(&mut s.practice, &d.example);
    assert!(s.practice.solved(&d.example));
    assert_eq!(s.assignment, original);
    assert!(!d.solved(&s));
    s.assignment.edit(d.assignment.reference.clone()).unwrap();
    finish(&mut s.assignment, &d.assignment);
    assert!(d.solved(&s));
    s.assignment.edit("keep;".into()).unwrap();
    assert!(!s.assignment.checked);
    assert!(s.assignment.results.is_empty());
    assert!(!d.solved(&s));
    finish(&mut s.assignment, &d.assignment);
    assert!(s.assignment.checked);
    assert_eq!(s.assignment.results.iter().filter(|r| r.passed).count(), 3);
    assert!(!d.solved(&s));
}
#[test]
fn equivalent_logic_passes_but_hardcoded_or_overbroad_filters_fail() {
    let d = definition();
    for script in [
        "require \"fileinto\"; if not header :is \"X-List\" \"training\" {keep;} else {fileinto \"Notices\";}",
        "require \"fileinto\"; if header :is \"X-List\" \"training\" {fileinto \"Notices\";stop;} keep;",
    ] {
        assert!(d.assignment.complete(script), "{script}");
    }
    for script in [
        "keep;",
        "discard;",
        "require \"fileinto\"; fileinto \"Notices\";",
        "require \"fileinto\"; if header :contains \"X-List\" \"training\" {fileinto \"Notices\";}",
        "require \"fileinto\"; if header :comparator \"i;octet\" :is \"X-List\" \"training\" {fileinto \"Notices\";}",
        "require \"fileinto\"; if header :is \"X-List\" \"training\" {fileinto \"Notices\";} keep;",
    ] {
        assert!(!d.assignment.complete(script), "{script}");
    }
}
#[test]
fn diagnostics_distinguish_syntax_unsupported_and_filing_errors() {
    let d = definition();
    let mut s = Scope::new(&d.assignment);
    for (script, label) in [
        ("# heading\nkeep", "Syntax error"),
        ("vacation \"Away\";", "Unsupported syntax"),
    ] {
        s.edit(script.into()).unwrap();
        s.start();
        assert!(!s.running);
        assert!(!s.checked);
        assert!(s.results.is_empty());
        assert!(s.error.as_ref().unwrap().starts_with(label));
        assert!(!s.solved(&d.assignment));
    }
    s.edit("# heading\nkeep".into()).unwrap();
    s.start();
    assert!(s.error.as_ref().unwrap().contains("line 2, column 5"));
    s.edit("require \"fileinto\"; fileinto \"Missing\";".into())
        .unwrap();
    finish(&mut s, &d.assignment);
    assert!(
        s.results
            .iter()
            .all(|r| r.error.as_ref().unwrap().starts_with("Filing error"))
    );
    assert!(s.results.iter().all(|r| !r.passed));
    assert!(!s.solved(&d.assignment));
}
#[test]
fn resource_failure_stops_the_run_and_roundtrips_the_draft_without_stale_results() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let script = "keep;".repeat(33);
    s.action(
        &d,
        &Action::Edit {
            script: script.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Run, false).unwrap();
    s.action(&d, &Action::Tick, false).unwrap();
    assert!(!s.assignment.running);
    assert!(!s.assignment.checked);
    assert!(!d.solved(&s));
    assert!(s.assignment.results.is_empty());
    assert!(
        s.assignment
            .error
            .as_ref()
            .unwrap()
            .starts_with("Limit reached")
    );
    assert_eq!(s.assignment.draft, script);
    let mut restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    restored
        .action(
            &d,
            &Action::Edit {
                script: d.assignment.reference.clone(),
            },
            false,
        )
        .unwrap();
    restored.action(&d, &Action::Run, false).unwrap();
    for _ in 0..d.assignment.cases.len() {
        restored.action(&d, &Action::Tick, false).unwrap();
    }
    assert!(d.solved(&restored));
    assert!(restored.assignment.error.is_none());
}
#[test]
fn selected_reset_preserves_other_draft_and_restore_rejects_forged_results() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.view = View::Example;
    s.assignment.edit(d.assignment.reference.clone()).unwrap();
    finish(&mut s.assignment, &d.assignment);
    s.practice
        .edit("# unfinished practice\nkeep;".into())
        .unwrap();
    s.practice.selected = 2;
    let assignment = s.assignment.clone();
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
    s.action(&d, &Action::Reset, true).unwrap();
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice, Scope::new(&d.example));
    let practice = s.practice.clone();
    s.action(&d, &Action::Reset, false).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!d.solved(&s));
    let mut forged = restored.clone();
    forged.assignment.results[0].actual = boxes(&["INBOX"]);
    assert!(d.bound(&forged).is_err());
    assert!(!d.solved(&forged));
    let mut forged = restored;
    forged.source_id = "other-filter".into();
    assert!(d.bound(&forged).is_err());
    assert!(
        serde_json::from_str::<Action>(r#"{"kind":"edit","script":"keep;","path":"/etc/passwd"}"#)
            .is_err()
    );
}
