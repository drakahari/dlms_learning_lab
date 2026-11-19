use dlms_learning_lab::{virtual_isolation_workspace::*, virtual_workloads as hosts};
#[path = "support/virtual_isolation_tasks.rs"]
mod setup;
use setup::{definition, task};
#[test]
fn rejects_unreachable_references_and_already_solved_starters() {
    for routing in [false, true] {
        let t = task(routing);
        t.validate().unwrap();
        assert!(t.checks(&t.reference()).iter().all(|c| c.passed));
        assert!(t.checks(&t.initial()).iter().any(|c| !c.passed));
        let mut bad = t.clone();
        match &mut bad.exercise {
            Exercise::Workloads {
                reference, initial, ..
            } => *reference = initial.clone(),
            Exercise::Routing {
                reference, initial, ..
            } => *reference = initial.clone(),
        };
        assert!(bad.validate().is_err());
        let mut bad = t.clone();
        match &mut bad.exercise {
            Exercise::Workloads {
                reference, initial, ..
            } => *initial = reference.clone(),
            Exercise::Routing {
                reference, initial, ..
            } => *initial = reference.clone(),
        };
        assert!(bad.validate().is_err());
    }
}
#[test]
fn accepts_alternate_replica_placement_and_context_labels() {
    for routing in [false, true] {
        let t = task(routing);
        let mut alternative = t.reference();
        match &mut alternative {
            Draft::Workloads(c) => {
                c.placements[2].host = "south".into();
                c.placements[3].host = "north".into();
            }
            Draft::Routing(c) => {
                for b in &mut c.bindings {
                    b.context = if b.context == "blue" { "green" } else { "blue" }.into();
                }
            }
        }
        assert!(t.checks(&alternative).iter().all(|c| c.passed));
    }
}
#[test]
fn checking_and_running_examples_cannot_finish_assignments() {
    for routing in [false, true] {
        let d = definition(routing);
        let mut s = d.new_session().unwrap();
        s.practice.edit(&d.example, d.example.reference()).unwrap();
        s.practice.run(&d.example).unwrap();
        s.practice.check(&d.example).unwrap();
        assert!(s.practice.solved(&d.example));
        assert!(!s.assignment.checked && !s.assignment.solved(&d.assignment));
        assert_eq!(s.assignment.configuration, d.assignment.initial());
    }
}
#[test]
fn edits_invalidate_feedback_and_forged_checked_does_not_override_state() {
    for routing in [false, true] {
        let t = task(routing);
        let mut a = Attempt::new(&t).unwrap();
        a.edit(&t, t.reference()).unwrap();
        a.run(&t).unwrap();
        a.check(&t).unwrap();
        assert!(a.solved(&t));
        a.edit(&t, t.initial()).unwrap();
        assert!(!a.checked && a.actual.is_none() && !a.solved(&t));
        a.checked = true;
        assert!(!a.solved(&t));
    }
}
#[test]
fn disk_format_preserves_both_drafts_and_does_not_fabricate_output() {
    for routing in [false, true] {
        let d = definition(routing);
        let mut s = d.new_session().unwrap();
        s.assignment
            .edit(&d.assignment, d.assignment.reference())
            .unwrap();
        s.assignment.check(&d.assignment).unwrap();
        s.assignment.run(&d.assignment).unwrap();
        let bytes = serde_json::to_vec(&s).unwrap();
        let loaded: Session = serde_json::from_slice(&bytes).unwrap();
        d.context(&loaded).unwrap();
        assert_eq!(loaded.assignment.configuration, s.assignment.configuration);
        assert_eq!(loaded.practice.configuration, s.practice.configuration);
        assert!(loaded.assignment.solved(&d.assignment));
        assert!(loaded.assignment.actual.is_none() && loaded.practice.actual.is_none());
        assert!(!loaded.practice.checked);
    }
}
#[test]
fn same_numbered_reply_is_labeled_as_wrong_endpoint() {
    let t = task(true);
    let mut a = Attempt::new(&t).unwrap();
    a.edit(&t, t.reference()).unwrap();
    a.choose(
        &t,
        Selection::Routing {
            from: "a-client".into(),
            to: "b-server".into(),
        },
    )
    .unwrap();
    a.run(&t).unwrap();
    let text = a.actual.as_ref().unwrap().join("\n");
    assert!(
        text.contains("Actual destination: a-server")
            && text.contains("different endpoint with the same address")
    );
    assert!(!text.contains("vrf-0"));
}
#[test]
fn invalid_new_trial_clears_old_output_without_changing_a_correct_plan() {
    let t = task(true);
    let mut a = Attempt::new(&t).unwrap();
    a.edit(&t, t.reference()).unwrap();
    a.check(&t).unwrap();
    a.run(&t).unwrap();
    a.choose(
        &t,
        Selection::Routing {
            from: "a-client".into(),
            to: "a-client".into(),
        },
    )
    .unwrap();
    assert!(a.actual.is_none());
    assert!(a.run(&t).is_err());
    assert!(a.actual.as_ref().unwrap()[0].contains("different supplied host endpoints"));
    assert!(a.solved(&t));
}
#[test]
fn malformed_goals_and_untrusted_selections_fail_explicitly() {
    let mut t = task(true);
    if let Exercise::Routing { pairs, .. } = &mut t.exercise {
        pairs.retain(|p| p.from != "b-client");
    }
    assert!(t.validate().is_err());
    let mut t = task(false);
    if let Exercise::Workloads { services, .. } = &mut t.exercise {
        services.pop();
    }
    assert!(t.validate().is_err());
    let t = task(false);
    let mut a = Attempt::new(&t).unwrap();
    assert!(
        a.choose(
            &t,
            Selection::Workloads {
                experiment: hosts::Experiment::HostOutage {
                    host: "outside".into()
                }
            }
        )
        .is_err()
    );
    let mut d = definition(false);
    d.example.id = d.assignment.id.clone();
    assert!(d.new_session().is_err());
    let d = definition(false);
    let mut s = d.new_session().unwrap();
    s.version += 1;
    assert!(d.context(&s).is_err());
}
