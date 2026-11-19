use dlms_learning_lab::{port_security::ViolationMode as V, port_security_workspace::*};
#[path = "support/port_security_tasks.rs"]
mod common;
use common::{definition, mac};
#[test]
fn all_three_distinct_tasks_have_reachable_work_and_meaningful_initial_failures() {
    for n in 1..=3 {
        let d = definition(n);
        d.validate().unwrap();
        let current = d.initial.checks(&d.initial.initial);
        assert!(current.iter().any(|r| !r.0));
        assert!(
            d.initial
                .checks(&d.initial.reference_world().unwrap())
                .iter()
                .all(|r| r.0)
        );
    }
}
#[test]
fn equivalent_modes_and_address_order_pass_while_shortcuts_do_not() {
    for n in [1, 3] {
        let d = definition(n);
        for mode in [V::Protect, V::Restrict] {
            let mut w = d.initial.reference_world().unwrap();
            w.configure("desk", true, 1, mode, vec![mac(1)]).unwrap();
            assert!(d.initial.checks(&w).iter().all(|r| r.0));
        }
    }
    let d = definition(2);
    let mut w = d.initial.reference_world().unwrap();
    w.configure("desk", true, 2, V::Restrict, vec![mac(8), mac(1)])
        .unwrap();
    assert!(d.initial.checks(&w).iter().all(|r| r.0));
    for (enabled, max, mode, ids) in [
        (false, 1, V::Restrict, vec![]),
        (true, 1, V::Restrict, vec![mac(1)]),
        (true, 2, V::Protect, vec![mac(1), mac(8)]),
        (true, 2, V::Shutdown, vec![mac(1), mac(8)]),
        (true, 3, V::Restrict, vec![mac(1), mac(8)]),
    ] {
        let mut w = d.initial.reference_world().unwrap();
        w.configure("desk", enabled, max, mode, ids).unwrap();
        assert!(d.initial.checks(&w).iter().any(|r| !r.0));
    }
}
#[test]
fn partial_feedback_identifies_affected_cases_and_unavailable_is_not_a_security_rejection() {
    let d = definition(2);
    let rows = d.initial.checks(&d.initial.initial);
    assert!(!rows[0].0);
    assert!(rows.iter().any(|r| !r.0
        && r.1.contains("Unexpected sender arrives first")
        && r.1.contains("actual")));
    assert!(
        rows.iter().any(|r| r.0 && r.1.contains("Both directions")),
        "Correct cases stay correct when overall configuration fails"
    );
    let mut w = d.initial.reference_world().unwrap();
    w.set_admin("desk", false).unwrap();
    let rows = d.initial.checks(&w);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("unavailable attachment"))
    );
}
#[test]
fn examples_drafts_and_checks_are_isolated_and_edits_cannot_reuse_old_success() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let untouched = s.assignment.clone();
    s.practice.fill(&d.example).unwrap();
    s.practice.check(&d.example).unwrap();
    assert_eq!(s.assignment, untouched);
    assert!(!d.checks(&s).iter().all(|r| r.0));
    s.assignment.fill(&d.initial).unwrap();
    assert!(!d.checks(&s).iter().all(|r| r.0));
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    s.assignment.drafts[0].maximum = "3".into();
    assert!(
        !d.checks(&s).iter().all(|r| r.0),
        "Even imported stale checked=true must not ignore changed drafts"
    );
    s.assignment.apply().unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
    s.assignment.drafts[0].maximum = "1".into();
    s.assignment.apply().unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    s.assignment.admin("desk", false).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn invalid_edits_are_retained_for_retry_without_partially_applying_settings() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let before = s.assignment.world.clone();
    s.assignment.drafts[0].configured = mac(1).text();
    s.assignment.drafts[1].maximum = "not a number".into();
    assert!(s.assignment.apply().is_err());
    assert_eq!(s.assignment.world, before);
    assert_eq!(s.assignment.drafts[1].maximum, "not a number");
    assert!(!s.assignment.checked);
    s.assignment.drafts[1].maximum = "1".into();
    s.assignment.apply().unwrap();
    assert_eq!(
        s.assignment.world.port("desk").unwrap().configured,
        vec![mac(1)]
    );
    let before = s.assignment.world.clone();
    s.assignment.source = "not a mac".into();
    assert!(s.assignment.send().is_err());
    assert_eq!(s.assignment.world, before);
    assert!(s.assignment.output.last().unwrap().error);
    assert!(
        s.assignment
            .output
            .last()
            .unwrap()
            .text
            .contains("Input needs attention")
    );
}
#[test]
fn explicit_recovery_and_protected_ports_are_actual_requirements() {
    let d = definition(3);
    let mut w = d.initial.initial.clone();
    d.initial.reference[0].apply(&mut w).unwrap();
    assert!(d.initial.checks(&w).iter().any(|r| !r.0));
    w.set_admin("desk", true).unwrap();
    assert!(w.port("desk").unwrap().error_disabled);
    w.set_admin("desk", false).unwrap();
    w.set_admin("desk", true).unwrap();
    assert!(d.initial.checks(&w).iter().all(|r| r.0));
    w.configure("ops", false, 1, V::Protect, vec![]).unwrap();
    assert!(
        d.initial
            .checks(&w)
            .iter()
            .any(|r| !r.0 && r.1.contains("neighboring"))
    );
}
#[test]
fn versioned_resume_keeps_both_scopes_and_validates_the_authored_source() {
    let d = definition(3);
    let mut s = Session::new(&d).unwrap();
    s.assignment.drafts[0].maximum = "pending draft".into();
    s.practice.fill(&d.example).unwrap();
    s.practice.check(&d.example).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let loaded = Session::from_json(&raw, &d).unwrap();
    assert_eq!(loaded, s);
    assert!(Session::from_json(&raw, &definition(2)).is_err());
    let mut wrong = serde_json::to_value(&s).unwrap();
    wrong["version"] = 2.into();
    assert!(Session::from_json(&wrong.to_string(), &d).is_err());
    assert!(Session::from_json(&" ".repeat(MAX_SESSION + 1), &d).is_err());
    s.assignment = Scope::new(&d.initial);
    assert_eq!(s.practice, loaded.practice);
    assert_eq!(s.assignment.world, d.initial.initial);
}
#[test]
fn malformed_unreachable_or_answerless_sources_are_rejected() {
    let d = definition(1);
    for kind in 0..6 {
        let mut bad = d.clone();
        match kind {
            0 => bad.initial.reference.clear(),
            1 => bad.initial.cases.clear(),
            2 => bad.example = bad.initial.clone(),
            3 => bad.initial.required[0].settings.maximum = 2,
            4 => {
                let Expected::Admitted { accepted_by } =
                    &mut bad.initial.cases[0].frames[1].expected
                else {
                    panic!()
                };
                accepted_by.clear();
            }
            _ => {
                bad.initial.cases[0].frames[0].source =
                    dlms_learning_lab::mac_learning::Mac::BROADCAST
            }
        };
        assert!(bad.validate().is_err(), "Accepted malformed source {kind}");
    }
}

#[test]
fn counted_trials_do_not_mistake_a_saturated_protect_counter_for_counted_restriction() {
    use dlms_learning_lab::port_security::{MAX_COUNTER, World};
    let d = definition(2);
    let mut w = d.initial.reference_world().unwrap();
    w.configure("desk", true, 2, V::Protect, vec![mac(1), mac(8)])
        .unwrap();
    let mut saved = serde_json::to_value(w).unwrap();
    saved["ports"][0]["violations"] = MAX_COUNTER.into();
    let mut w = World::from_json(&saved.to_string()).unwrap();
    let rows = d.initial.checks(&w);
    let rejected = rows
        .iter()
        .find(|r| r.1.starts_with("Unexpected sender arrives first:"))
        .unwrap();
    assert!(
        !rejected.0,
        "A saturated numerical counter is not evidence that Protect counted the rejected source"
    );
    w.configure("desk", true, 2, V::Restrict, vec![mac(1), mac(8)])
        .unwrap();
    assert!(
        d.initial.checks(&w).iter().all(|r| r.0),
        "Actual Restrict behavior remains valid when the display counter saturates"
    );
}

#[test]
fn pretraining_the_right_source_does_not_replace_the_required_explicit_binding() {
    let d = definition(1);
    let mut w = d.initial.initial.clone();
    w.send("assigned-desk", mac(1), mac(3)).unwrap();
    let rows = d.initial.checks(&w);
    assert!(
        !rows[0].0,
        "Learned identity is not the requested configured reservation"
    );
    assert!(
        rows.iter().skip(3).all(|r| r.0),
        "Actual traffic cases can pass while the explicit configuration requirement is still unfinished"
    );
}
#[test]
fn wrong_first_learning_has_a_taught_atomic_retry_instead_of_an_unexplained_dead_end() {
    let d = definition(1);
    let mut s = Scope::new(&d.initial);
    s.send().unwrap();
    assert_eq!(s.world.port("desk").unwrap().dynamic, vec![mac(9)]);
    assert!(s.output.last().unwrap().text.contains("assigned-desk"));
    assert!(s.output.last().unwrap().text.contains(&mac(9).text()));
    s.drafts[0].configured = mac(1).text();
    let before = s.world.clone();
    let error = s.apply().unwrap_err();
    assert!(
        error.contains("learned secure")
            && error.contains("Shut down")
            && error.contains("Bring up")
    );
    assert_eq!(s.world, before);
    s.admin("desk", false).unwrap();
    s.apply().unwrap();
    s.admin("desk", true).unwrap();
    assert!(s.check(&d.initial).unwrap().iter().all(|r| r.0));
    s.source.clear();
    assert!(s.send().is_err());
    assert!(
        s.output
            .last()
            .unwrap()
            .text
            .contains("Source MAC is blank")
    );
    s.source = mac(1).text();
    s.destination.clear();
    assert!(s.send().is_err());
    assert!(
        s.output
            .last()
            .unwrap()
            .text
            .contains("Destination MAC is blank")
    );
}
