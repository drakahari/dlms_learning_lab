use dlms_learning_lab::dhcp_snooping_workspace::*;
#[path = "support/snoop_tasks.rs"]
mod common;
use common::{definition, task};
#[test]
fn structured_case_reports_retain_every_observation_and_grade_text() {
    for kind in [1, 2] {
        let d = definition(kind);
        for w in [&d.initial.initial, &d.initial.reference_world().unwrap()] {
            let reports = d.initial.reports(w);
            assert_eq!(reports.len(), d.initial.cases.len());
            assert_eq!(
                reports.iter().map(CaseReport::check).collect::<Vec<_>>(),
                d.initial.checks(w)
            );
            for (report, case) in reports.iter().zip(&d.initial.cases) {
                assert_eq!(report.label.as_deref(), Some(case.label.as_str()));
                assert_eq!(report.passed(), report.failures.is_empty());
                for failure in &report.failures {
                    assert!(!failure.trim().is_empty());
                    assert!(report.check().1.contains(failure));
                }
            }
        }
    }
    let t = task(1, false);
    let reports = t.reports(&t.initial);
    assert_eq!(reports[0].failures.len(), 5);
    for i in 1..=4 {
        assert!(reports[0].failures[i - 1].starts_with(&format!("Message {i}: expected")));
    }
    assert!(reports[0].failures[4].starts_with("Observed leases: expected 1, actual 0."));
    assert!(reports.iter().any(|r| r.failures.len() > 1));
    assert!(
        reports
            .iter()
            .flat_map(|r| &r.failures)
            .any(|s| s.contains("expected") && s.contains("actual"))
    );
}
#[test]
fn reachable_reference_and_unfinished_starters_use_multiple_actual_behavior_cases() {
    for kind in [1, 2] {
        let d = definition(kind);
        d.validate().unwrap();
        assert_eq!(d.initial.cases.len(), 4);
        assert!(d.initial.checks(&d.initial.initial).iter().any(|r| !r.0));
        assert!(
            d.initial
                .checks(&d.initial.reference_world().unwrap())
                .iter()
                .all(|r| r.0)
        );
    }
}
#[test]
fn assignment_and_example_drafts_runs_checks_and_resume_are_isolated() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    s.practice.fill(&d.example).unwrap();
    s.practice.run_sample(&d.example).unwrap();
    assert!(s.practice.output.iter().all(|l| !l.error));
    assert_eq!(s.practice.world.bindings().len(), 1);
    assert!(s.practice.check(&d.example).unwrap().iter().all(|r| r.0));
    assert_eq!(s.assignment, original);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    s.assignment.draft.vlans = "10,".into();
    s.view = dlms_learning_lab::script_teaching::View::Assignment;
    let restored = Session::from_json(&serde_json::to_string(&s).unwrap(), &d).unwrap();
    assert_eq!(s, restored);
    assert_eq!(restored.assignment.draft.vlans, "10,");
}
#[test]
fn equivalent_supported_policies_pass_and_false_checked_flags_cannot_hide_broken_work() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let mut c = d.initial.reference.clone();
    c.ports.reverse();
    c.vlans = vec![20, 10];
    s.assignment.draft = Draft::new(&c);
    s.assignment.apply().unwrap();
    assert!(s.assignment.check(&d.initial).unwrap().iter().all(|r| r.0));
    assert!(d.checks(&s).iter().all(|r| r.0));
    s.assignment.draft.enabled = false;
    assert!(d.checks(&s).iter().any(|r| !r.0));
    s.assignment.apply().unwrap();
    s.assignment.checked = true;
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let identity = definition(2);
    let mut s = Session::new(&identity).unwrap();
    s.assignment.fill(&identity.initial).unwrap();
    s.assignment.draft.verify_source = false;
    s.assignment.apply().unwrap();
    s.assignment.checked = true;
    assert!(identity.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn trusting_a_user_port_and_disabling_everything_fail_specific_cases_without_hiding_good_rows() {
    let d = definition(2);
    let mut s = Scope::new(&d.initial);
    s.fill(&d.initial).unwrap();
    s.draft
        .ports
        .iter_mut()
        .find(|p| p.port == "peer")
        .unwrap()
        .trusted = true;
    s.apply().unwrap();
    let rows = s.check(&d.initial).unwrap();
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("Unexpected server reply"))
    );
    assert!(
        rows.iter()
            .any(|r| r.0 && r.1.contains("Client identifiers disagree"))
    );
    s.draft.enabled = false;
    s.apply().unwrap();
    let rows = s.check(&d.initial).unwrap();
    assert!(rows.iter().all(|r| !r.0));
    assert!(rows.iter().any(|r| r.1.contains("not inspected")));
}
#[test]
fn educational_feedback_and_actual_run_distinguish_unsupported_followup_from_rejection() {
    let d = definition(2);
    let mut s = Scope::new(&d.initial);
    let rows = s.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| {
        !r.0 && r
            .1
            .contains("Ethernet source and DHCP client MAC addresses disagree")
    }));
    assert!(rows.iter().any(|r| r.0));
    s.sample = 2;
    s.run_sample(&d.initial).unwrap();
    assert!(s.output.iter().any(|r| r.text.contains("inspected")));
    s.fill(&d.initial).unwrap();
    s.sample = 2;
    s.run_sample(&d.initial).unwrap();
    assert!(s.output.iter().any(|r| r.text.contains("rejected")));
    assert!(s.output.iter().all(|r| !r.error));
    s.sample = 0;
    s.run_sample(&d.initial).unwrap();
    assert_eq!(s.world.bindings().len(), 1);
    assert!(s.output.iter().any(|r| r.text.contains("observed")));
    // Deliberately invalid author fixture: unlike a security rejection, an
    // unsupported reply stops replay and must never display a fabricated lease.
    let mut invalid = task(1, false);
    invalid.cases[0]
        .steps
        .insert(1, Step::Advance { seconds: 61 });
    assert!(invalid.validate().is_err());
    let mut run = Scope::new(&invalid);
    run.fill(&invalid).unwrap();
    run.run_sample(&invalid).unwrap();
    assert!(
        run.output
            .iter()
            .any(|r| r.error && r.text.contains("Not supported"))
    );
    assert!(run.world.bindings().is_empty());
}
#[test]
fn checking_is_deterministic_and_never_trains_or_overwrites_displayed_state() {
    let d = definition(1);
    let mut s = Scope::new(&d.initial);
    s.fill(&d.initial).unwrap();
    s.sample = 0;
    s.run_sample(&d.initial).unwrap();
    let before = s.world.clone();
    let output = s.output.clone();
    let first = s.check(&d.initial).unwrap();
    let second = s.check(&d.initial).unwrap();
    assert_eq!(first, second);
    assert_eq!(s.world, before);
    assert_eq!(s.output, output);
    assert_eq!(s.world.bindings().len(), 1);
    s.sample = 3;
    s.run_sample(&d.initial).unwrap();
    assert!(s.world.bindings().is_empty());
    assert!(!s.checked);
    assert!(
        s.check(&d.initial).unwrap().iter().all(|r| r.0),
        "Configuration trials are separate deterministic inputs, not stale runtime lease credit"
    );
}
#[test]
fn invalid_drafts_stay_editable_without_applying_them_and_clear_previous_success() {
    let d = definition(1);
    let mut s = Scope::new(&d.initial);
    s.fill(&d.initial).unwrap();
    s.check(&d.initial).unwrap();
    let world = s.world.clone();
    for input in ["10,,20", "ten", "4095", "10,10"] {
        s.draft.vlans = input.into();
        assert!(s.apply().is_err());
        assert_eq!(s.world, world);
        assert_eq!(s.draft.vlans, input);
        assert!(!s.checked);
        assert!(s.check(&d.initial).is_err());
    }
}
#[test]
fn wrong_source_or_practice_equipment_cannot_resume_as_assignment_and_reset_is_selected() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    s.assignment.fill(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.practice.draft.vlans = "unfinished".into();
    let practice = s.practice.clone();
    s.assignment = Scope::new(&d.initial);
    assert_eq!(s.practice, practice);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let mut wrong = s.clone();
    wrong.assignment.world = d.example.initial.clone();
    assert!(d.bound(&wrong).is_err());
    assert!(Session::from_json(&serde_json::to_string(&s).unwrap(), &definition(2)).is_err());
}
#[test]
fn invalid_task_data_requires_reachable_real_delivery_rejection_and_observed_leases() {
    let mut t = task(1, false);
    t.cases[0].leases_after = 2;
    assert!(t.validate().is_err());
    let mut t = task(1, false);
    t.reference.enabled = false;
    assert!(t.validate().is_err());
    let mut t = task(1, false);
    if let Step::Message {
        expected: Expected::Forwarded { accepted_by, .. },
        ..
    } = &mut t.cases[0].steps[0]
    {
        accepted_by.clear();
    }
    assert!(t.validate().is_err());
    let mut d = definition(1);
    d.example = d.initial.clone();
    assert!(d.validate().is_err());
}
#[test]
fn bounded_restore_and_internal_policy_adapter_never_treat_input_as_a_command() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let original = s.clone();
    assert!(s.execute_reference(&d, "sh -c anything").is_err());
    assert_eq!(s, original);
    let solution = format!(
        "{PREFIX}{}",
        serde_json::to_string(&d.initial.reference).unwrap()
    );
    assert!(s.execute_reference(&d, &solution).unwrap().contains("4/4"));
    assert!(d.checks(&s).iter().all(|r| r.0));
    assert_eq!(s.practice, original.practice);
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["version"] = serde_json::json!(2);
    assert!(Session::from_json(&raw.to_string(), &d).is_err());
    assert!(Session::from_json(&" ".repeat(MAX_SESSION + 1), &d).is_err());
}
