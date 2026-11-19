#[path = "support/removable_workspace.rs"]
mod fixture;
use dlms_learning_lab::{removable_mount as m, removable_workspace as w, script_teaching::View};
use fixture::*;
#[test]
fn state_and_dependency_outcomes_accept_equivalent_supported_policy() {
    let d = definition();
    d.validate().unwrap();
    let canonical = configuration(10);
    let alternatives = [
        canonical.clone(),
        canonical
            .replace("rw,auto", "defaults,rw")
            .replace("ro,auto,nofail", "nofail,ro"),
        format!(
            "# Same supported settings\n\n{}",
            canonical.replace(' ', "\t")
        ),
    ];
    for text in alternatives {
        let rows = d.initial.run(&text).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(w::CaseResult::passed));
        let missing = rows[2].projection.as_ref().unwrap();
        assert!(!missing.required_attempts_satisfied());
        assert!(missing.state.live().contains_key("/media/audit"));
    }
}
#[test]
fn wrong_uuid_has_a_passing_absent_case_but_cannot_hide_two_failed_present_cases() {
    let t = task("optional-assignment", 10);
    let wrong = configuration(10).replace("000000000011", "000000000099");
    let rows = t.run(&wrong).unwrap();
    assert!(!rows[0].passed());
    assert!(rows[1].passed());
    assert!(!rows[2].passed());
    assert!(rows[0].checks.iter().any(|r| r.0));
    assert!(
        rows[0]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("Not mounted"))
    );
    assert!(rows.iter().all(|r| r.diagnostic.is_none()));
}
#[test]
fn nofail_everywhere_noauto_empty_table_or_wrong_type_never_bypass_the_outcomes() {
    let t = task("optional-assignment", 10);
    let text = configuration(10);
    for wrong in [
        text.replace("rw,auto", "rw,auto,nofail"),
        text.replace("rw,auto", "rw,noauto")
            .replace("ro,auto,nofail", "ro,noauto,nofail"),
        String::new(),
        text.replace("/media/audit ext4", "/media/audit xfs"),
    ] {
        let rows = t.run(&wrong).unwrap();
        assert!(rows.iter().any(|r| !r.passed()), "{wrong}");
    }
    let all_optional = text.replace("rw,auto", "rw,auto,nofail");
    let rows = t.run(&all_optional).unwrap();
    assert!(
        rows[2]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("Required-storage"))
    );
}
#[test]
fn malformed_or_unsupported_text_is_not_a_successful_expected_failure() {
    let t = task("optional-assignment", 10);
    for wrong in [
        "UUID=bad /project xfs rw 0 2",
        "$(cat /etc/shadow)",
        "include /etc/fstab",
        "UUID=00000000-0000-4000-8000-000000000010 /project xfs user,nofail 0 2",
    ] {
        let rows = t.run(wrong).unwrap();
        assert!(
            rows.iter()
                .all(|r| !r.passed() && r.diagnostic.is_some() && r.projection.is_none())
        );
        assert!(!rows[2].checks[0].0);
    }
}
#[test]
fn practice_assignment_edits_retry_and_serialized_resume_keep_current_credit_honest() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    s.view = View::Example;
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert_eq!(s.assignment, assignment);
    assert!(!d.solved(&s));
    s.assignment.replace(&configuration(10)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let saved = serde_json::to_string(&s).unwrap();
    let got = w::Session::from_json(&saved, &d).unwrap();
    assert_eq!(got, s);
    assert!(d.solved(&got));
    s.assignment
        .replace(&configuration(10).replace("nofail", "noauto"))
        .unwrap();
    assert!(s.assignment.tested.is_none());
    assert!(!d.solved(&s));
    s.assignment.replace(&configuration(10)).unwrap();
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let fresh = w::Session::new(&d).unwrap();
    assert!(!d.solved(&fresh));
    assert!(fresh.assignment.tested.is_none());
    assert_eq!(got.practice.draft, d.example_worked);
}
#[test]
fn stopped_unfinished_changed_or_foreign_runs_cannot_publish_current_credit() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&configuration(10)).unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    run.cancel();
    assert!(!run.tick());
    assert!(run.results().is_none());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut run = s.assignment.begin(&d.initial).unwrap();
    for _ in 0..w::MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    assert!(run.done());
    s.assignment.replace("# changed").unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut bad = s.clone();
    bad.origin = "foreign".into();
    assert!(w::Session::from_json(&serde_json::to_string(&bad).unwrap(), &d).is_err());
    assert!(!d.solved(&bad));
}
#[test]
fn sources_definitions_and_imports_validate_bounds_references_and_reachability() {
    let d = definition();
    assert!(w::Definition::from_json(&serde_json::to_string(&d).unwrap()).is_ok());
    let mut bad = d.clone();
    bad.initial.cases[0].requirements.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[1]
        .requirements
        .push(w::Requirement::Mounted {
            target: "/media/audit".into(),
            volume: "media".into(),
            read_only: true,
        });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0]
        .requirements
        .push(w::Requirement::Policy {
            target: "/home/private".into(),
            mode: m::Dependency::Wanted,
        });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example_worked = "# not solved".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example = d.initial.clone();
    assert!(bad.validate().is_err());
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(s.assignment.replace(&"x".repeat(16385)).is_err());
    assert_eq!(s, before);
    assert!(w::Definition::from_json(&" ".repeat(w::MAX_DEFINITION + 1)).is_err());
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["version"] = 2.into();
    assert!(w::Session::from_json(&value.to_string(), &d).is_err());
}
