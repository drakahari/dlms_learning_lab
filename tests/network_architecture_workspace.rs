use dlms_learning_lab::{network_architecture::*, network_architecture_workspace::*};
#[path = "support/network_architecture_fixture.rs"]
mod fixture;
use fixture::*;
fn task(three_tier: bool) -> Task {
    let (facts, reference) = if three_tier { three() } else { two() };
    let mut initial = reference.clone();
    initial.connections[0].a = None;
    let uplinks = if three_tier {
        vec!["north", "south", "dn1", "dn2", "ds1", "ds2"]
    } else {
        vec!["north", "south"]
    };
    Task {
        id: if three_tier { "three" } else { "two" }.into(),
        editable: facts
            .wires
            .iter()
            .map(|w| Named { id: w.id.clone() })
            .collect(),
        facts,
        initial,
        reference,
        uplinks: uplinks
            .into_iter()
            .map(|node| Uplinks {
                node: node.into(),
                minimum: 2,
            })
            .collect(),
        pairs: vec![Pair {
            from: "north".into(),
            to: "south".into(),
        }],
        cases: vec![
            Case {
                minimum_paths: None,
                id: "normal".into(),
                label: "All devices available".into(),
                experiment: Experiment::default(),
            },
            Case {
                minimum_paths: None,
                id: "west-failed".into(),
                label: "West device unavailable".into(),
                experiment: failure("west"),
            },
            Case {
                minimum_paths: None,
                id: "east-failed".into(),
                label: "East device unavailable".into(),
                experiment: failure("east"),
            },
        ],
    }
}
fn definition() -> Definition {
    Definition {
        schema_version: 1,
        version: 1,
        id: "campus".into(),
        assignment: task(false),
        example: task(true),
    }
}
#[test]
fn references_and_equivalent_ports_and_cable_orders_pass_both_designs() {
    for tier in [false, true] {
        let t = task(tier);
        t.validate().unwrap();
        assert!(t.checks(&t.reference).iter().all(|c| c.passed));
        let mut alt = t.reference.clone();
        alt.connections.reverse();
        for wire in &mut alt.connections {
            std::mem::swap(&mut wire.a, &mut wire.b);
        }
        assert!(t.checks(&alt).iter().all(|c| c.passed));
        if !tier {
            for wire in &mut alt.connections {
                for jack in [&mut wire.a, &mut wire.b].into_iter().flatten() {
                    if jack.port == "p1" {
                        jack.port = "p3".into();
                    }
                }
            }
            assert_ne!(alt, t.reference);
            assert!(t.checks(&alt).iter().all(|c| c.passed));
        }
    }
}
#[test]
fn partial_normal_connectivity_cannot_substitute_for_redundancy() {
    let t = task(false);
    let rows = t.checks(&t.initial);
    assert!(rows.iter().any(|r| r.passed));
    assert!(rows.iter().any(|r| !r.passed));
    assert!(
        rows.iter()
            .any(|r| r.passed && r.detail.contains("All devices available"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.passed && r.detail.contains("East device unavailable"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.passed && r.detail.contains("distinct upstream"))
    );
    let mut parallel = t.reference.clone();
    parallel.connections[1] = link("ne", "north", 2, "west", 3);
    assert!(
        t.checks(&parallel)
            .iter()
            .any(|r| !r.passed && r.detail.contains("1 distinct upstream"))
    );
}
#[test]
fn wrong_hierarchy_physical_paths_and_conflicts_explain_the_mistake() {
    let t = task(false);
    let mut wrong = t.reference.clone();
    wrong.connections[0] = link("nw", "north", 1, "south", 3);
    wrong.connections[1].a = None;
    wrong.connections[1].b = None;
    let rows = t.checks(&wrong);
    assert!(
        rows.iter()
            .any(|r| !r.passed && r.detail.contains("physical path exists"))
    );
    let mut conflict = t.reference.clone();
    conflict.connections[1].a = conflict.connections[0].a.clone();
    assert!(t.facts.validate_draft(&conflict).is_ok());
    let rows = t.checks(&conflict);
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].passed);
    assert!(rows[0].detail.contains("more than one cable"));
}
#[test]
fn editing_and_resume_regrade_state_not_a_claimed_success_bit() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    s.assignment
        .edit(&d.assignment, d.assignment.reference.clone())
        .unwrap();
    assert!(!s.assignment.solved(&d.assignment));
    assert!(
        s.assignment
            .check(&d.assignment)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert!(s.assignment.solved(&d.assignment));
    s.assignment
        .edit(&d.assignment, d.assignment.reference.clone())
        .unwrap();
    assert!(s.assignment.solved(&d.assignment));
    let raw = serde_json::to_vec(&s).unwrap();
    let mut resumed: Session = serde_json::from_slice(&raw).unwrap();
    d.context(&resumed).unwrap();
    assert!(resumed.assignment.solved(&d.assignment));
    resumed
        .assignment
        .edit(&d.assignment, d.assignment.initial.clone())
        .unwrap();
    assert!(!resumed.assignment.checked);
    assert!(!resumed.assignment.solved(&d.assignment));
    resumed.assignment.checked = true;
    assert!(!resumed.assignment.solved(&d.assignment));
    assert!(
        resumed
            .assignment
            .check(&d.assignment)
            .unwrap()
            .iter()
            .any(|r| !r.passed)
    );
}
#[test]
fn practice_is_separate_and_reset_only_changes_selected_attempt() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    let original = s.assignment.clone();
    s.practice
        .edit(&d.example, d.example.reference.clone())
        .unwrap();
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.solved(&d.example));
    assert_eq!(s.assignment, original);
    assert!(!s.assignment.solved(&d.assignment));
    let practice = s.practice.clone();
    s.reset(&d, false).unwrap();
    assert_eq!(s.practice, practice);
    s.reset(&d, true).unwrap();
    assert!(!s.practice.checked);
    assert_eq!(s.practice.draft, d.example.initial);
    let mut unfinished = s.clone();
    unfinished.assignment.draft.connections[0].b = None;
    let restored: Session =
        serde_json::from_slice(&serde_json::to_vec(&unfinished).unwrap()).unwrap();
    d.context(&restored).unwrap();
    assert_eq!(restored, unfinished);
}
#[test]
fn protected_cables_are_source_bound_not_a_reference_layout_lock() {
    let mut t = task(false);
    t.editable.retain(|w| w.id != "sw");
    t.validate().unwrap();
    let mut wrong = t.initial.clone();
    wrong
        .connections
        .iter_mut()
        .find(|w| w.wire == "sw")
        .unwrap()
        .b = None;
    assert!(
        t.checks(&wrong)
            .iter()
            .any(|r| !r.passed && r.detail.contains("protected cable"))
    );
    let mut a = Attempt {
        draft: t.initial.clone(),
        checked: false,
    };
    assert!(a.edit(&t, wrong).is_err());
    assert_eq!(a.draft, t.initial);
    assert!(a.edit(&t, t.reference.clone()).is_ok());
}
#[test]
fn invalid_authored_tasks_and_unreachable_solutions_are_rejected() {
    let t = task(false);
    let mut cases = vec![];
    let mut x = t.clone();
    x.initial = x.reference.clone();
    cases.push(x);
    let mut x = t.clone();
    x.reference = x.initial.clone();
    cases.push(x);
    let mut x = t.clone();
    x.editable[0].id = "foreign".into();
    cases.push(x);
    let mut x = t.clone();
    x.uplinks[0].node = "east".into();
    cases.push(x);
    let mut x = t.clone();
    x.uplinks[0].minimum = 0;
    cases.push(x);
    let mut x = t.clone();
    x.pairs[0].to = "east".into();
    cases.push(x);
    let mut x = t.clone();
    x.cases[1].experiment = Experiment::default();
    cases.push(x);
    let mut x = t.clone();
    x.cases.truncate(1);
    cases.push(x);
    let mut x = t.clone();
    x.cases[1].experiment = failure("foreign");
    cases.push(x);
    let mut x = t.clone();
    x.cases[1].id = x.cases[0].id.clone();
    cases.push(x);
    let mut x = t.clone();
    x.cases[1].label = "bad\nlabel".into();
    cases.push(x);
    let mut x = t.clone();
    x.editable = vec![x.editable[0].clone(); 33];
    cases.push(x);
    let mut x = t.clone();
    x.id = "../bad".into();
    cases.push(x);
    for (i, x) in cases.iter().enumerate() {
        assert!(x.validate().is_err(), "mutation{i}");
    }
}
#[test]
fn unknown_json_fields_oversized_lists_and_foreign_sessions_fail_closed() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    s.id = "foreign".into();
    assert!(d.context(&s).is_err());
    let mut s = d.new_session().unwrap();
    s.schema_version = 2;
    assert!(d.context(&s).is_err());
    let mut s = d.new_session().unwrap();
    s.practice.draft.connections[0].wire = "unknown".into();
    assert!(d.context(&s).is_err());
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["assignment"]["fake_pass"] = true.into();
    assert!(serde_json::from_value::<Definition>(raw).is_err());
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["assignment"]["cases"] =
        serde_json::json!([null, null, null, null, null, null, null, null, null]);
    assert!(serde_json::from_value::<Definition>(raw).is_err());
    let mut same = d.clone();
    same.example = same.assignment.clone();
    assert!(same.validate().is_err());
    let mut wrong = d.clone();
    wrong.schema_version = 2;
    assert!(wrong.validate().is_err());
}

fn leaf_task() -> Task {
    let (facts, reference) = leaf_spine();
    let mut initial = reference.clone();
    initial.connections[0].b = None;
    Task {
        id: "clos-counts".into(),
        editable: facts
            .wires
            .iter()
            .map(|w| Named { id: w.id.clone() })
            .collect(),
        facts,
        initial,
        reference,
        uplinks: (0..4)
            .map(|n| Uplinks {
                node: format!("leaf-{n}"),
                minimum: 1,
            })
            .collect(),
        pairs: vec![Pair {
            from: "leaf-0".into(),
            to: "leaf-3".into(),
        }],
        cases: vec![
            Case {
                id: "normal".into(),
                label: "All spines available".into(),
                experiment: Experiment::default(),
                minimum_paths: Some(3),
            },
            Case {
                id: "spine-zero-lost".into(),
                label: "Spine zero unavailable".into(),
                experiment: failure("spine-0"),
                minimum_paths: Some(2),
            },
        ],
    }
}
#[test]
fn one_working_path_is_not_the_required_number_of_independent_alternatives() {
    let t = leaf_task();
    t.validate().unwrap();
    let mut c = t.reference.clone();
    for index in [0, 1] {
        c.connections[index].a = None;
        c.connections[index].b = None;
    }
    assert!(
        t.facts
            .inspect(&c, &Experiment::default(), "leaf-0", "leaf-3")
            .unwrap()
            .hierarchy
            .is_some()
    );
    let checks = t.checks(&c);
    assert!(checks.iter().any(|r| !r.passed
        && r.detail.contains("1 distinct internal spine alternatives")
        && r.detail.contains("All spines available")));
    assert!(t.checks(&t.reference).iter().all(|r| r.passed));
}
#[test]
fn count_goals_are_bounded_reachable_and_never_silently_applied_to_other_designs() {
    let t = leaf_task();
    for n in [0, 4, 9] {
        let mut bad = t.clone();
        bad.cases[0].minimum_paths = Some(n);
        assert!(bad.validate().is_err(), "count{n}");
    }
    let mut old = task(false);
    old.cases[0].minimum_paths = Some(1);
    assert!(old.validate().is_err());
    let mut long = t.clone();
    long.cases[0].id = "a".repeat(49);
    assert!(long.validate().is_err());
}
#[test]
fn legacy_case_json_and_saved_sessions_do_not_acquire_new_fields_or_credit() {
    let d = definition();
    let raw = serde_json::to_value(&d).unwrap();
    assert!(raw["assignment"]["cases"][0].get("minimum_paths").is_none());
    let fresh = d.new_session().unwrap();
    let text = serde_json::to_string(&fresh).unwrap();
    assert!(!text.contains("minimum_paths"));
    assert!(!text.contains("spine_alternatives"));
    let new = Definition {
        schema_version: 1,
        version: 1,
        id: "clos".into(),
        assignment: leaf_task(),
        example: task(false),
    };
    let mut s = new.new_session().unwrap();
    s.assignment
        .edit(&new.assignment, new.assignment.reference.clone())
        .unwrap();
    s.assignment.check(&new.assignment).unwrap();
    assert!(s.assignment.solved(&new.assignment));
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    new.context(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(restored.assignment.solved(&new.assignment));
    s.assignment
        .edit(&new.assignment, new.assignment.initial.clone())
        .unwrap();
    assert!(!s.assignment.solved(&new.assignment));
}
