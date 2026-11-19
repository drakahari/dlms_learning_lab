use dlms_learning_lab::{
    poe::{Class, Device, Facts, Plan, Port, Source},
    poe_task::{Target, Task},
    poe_workspace::{Action, Definition, Session},
};
fn task(id: &str) -> Task {
    Task {
        id: id.into(),
        instructions:
            "Power every supplied device using compatible ports within the shared source budget."
                .into(),
        facts: Facts {
            budget_mw: 60_000,
            ports: vec![
                Port {
                    id: "Gi1/0/1".into(),
                    source: Source::Ieee8023af,
                },
                Port {
                    id: "Gi1/0/2".into(),
                    source: Source::Ieee8023at,
                },
                Port {
                    id: "Gi1/0/3".into(),
                    source: Source::DataOnly,
                },
                Port {
                    id: "Gi1/0/4".into(),
                    source: Source::Ieee8023at,
                },
            ],
            devices: vec![
                Device {
                    id: "staff-ap".into(),
                    class: Class::Four,
                },
                Device {
                    id: "phone".into(),
                    class: Class::Two,
                },
                Device {
                    id: "camera".into(),
                    class: Class::Three,
                },
            ],
        },
        initial: Plan {
            attachments: vec![Some(0), Some(3), Some(2)],
            enabled: vec![true; 4],
        },
        reference: Plan {
            attachments: vec![Some(1), Some(3), Some(0)],
            enabled: vec![true; 4],
        },
        targets: vec![Target::Powered; 3],
    }
}
fn definition() -> Definition {
    let assignment = task("ccna-power-fixture");
    let mut example = task("ccna-power-example");
    example.facts.budget_mw = 55_000;
    example.facts.devices[0].id = "example-ap".into();
    Definition {
        version: 1,
        source_id: assignment.id.clone(),
        assignment,
        example,
    }
}
#[test]
fn actual_power_outcomes_accept_an_alternate_port_plan() {
    let t = task("ccna-power-fixture");
    t.validate().unwrap();
    assert!(!t.solved(&t.initial));
    assert!(t.solved(&t.reference));
    let alt = Plan {
        attachments: vec![Some(3), Some(0), Some(1)],
        enabled: vec![true; 4],
    };
    assert_ne!(alt, t.reference);
    assert!(t.solved(&alt));
    let mut wrong = alt;
    wrong.enabled[3] = false;
    assert!(!t.solved(&wrong));
    assert_eq!(t.outcomes(&wrong).iter().filter(|r| r.passed).count(), 2);
}
#[test]
fn feedback_names_per_port_budget_and_data_only_mistakes() {
    let t = task("ccna-power-fixture");
    let c = t.outcomes(&t.initial);
    assert!(
        c[0].detail.contains("30 W")
            && c[0].detail.contains("15.4 W")
            && c[0].detail.contains("per-port")
    );
    assert!(c[2].detail.contains("data-only"));
    let mut t = t;
    t.facts.budget_mw = 30_000;
    let c = t.outcomes(&t.reference);
    assert!(c.iter().any(|r| r.detail.contains("shared source budget")));
}
#[test]
fn budget_starvation_does_not_count_as_deliberate_retirement() {
    let mut t = task("ccna-power-fixture");
    t.facts.budget_mw = 37_000;
    t.initial = t.reference.clone();
    t.targets[2] = Target::DisconnectedOrDisabled;
    t.reference.attachments[2] = None;
    t.validate().unwrap();
    assert!(!t.solved(&t.initial));
    assert!(t.outcomes(&t.initial)[2].detail.contains("not retirement"));
    assert!(t.solved(&t.reference));
    let mut alt = t.initial.clone();
    alt.enabled[0] = false;
    assert!(t.solved(&alt));
}
#[test]
fn practice_edits_checks_and_resets_never_award_assignment_credit() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    s.action(
        &d,
        &Action::Edit {
            plan: d.example.reference.clone(),
        },
        true,
    )
    .unwrap();
    s.action(&d, &Action::Check, true).unwrap();
    assert!(s.example.solved(&d.example));
    assert_eq!(s.assignment, assignment);
    assert!(!s.assignment.solved(&d.assignment));
    s.action(
        &d,
        &Action::Edit {
            plan: d.assignment.reference.clone(),
        },
        false,
    )
    .unwrap();
    assert!(!s.assignment.checked);
    s.action(&d, &Action::Check, false).unwrap();
    assert!(s.assignment.solved(&d.assignment));
    let example = s.example.clone();
    s.action(&d, &Action::Reset, false).unwrap();
    assert_eq!(s.example, example);
    assert_eq!(s.assignment, assignment);
}
#[test]
fn editing_a_success_invalidates_it_and_partial_work_resumes_exactly() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::Edit {
            plan: d.assignment.reference.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(s.assignment.solved(&d.assignment));
    let mut broken = s.assignment.plan.clone();
    broken.enabled[1] = false;
    s.action(&d, &Action::Edit { plan: broken }, false).unwrap();
    assert!(!s.assignment.checked && !s.assignment.solved(&d.assignment));
    s.view = dlms_learning_lab::script_teaching::View::Assignment;
    let raw = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(s, restored);
    assert_eq!(raw, serde_json::to_vec(&restored).unwrap());
}
#[test]
fn changed_sources_and_forged_claims_do_not_invent_mastery() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment.checked = true;
    assert!(!s.assignment.solved(&d.assignment));
    let mut changed = d.clone();
    changed.assignment.facts.budget_mw += 1;
    assert!(changed.bound(&s).is_err());
    let mut forged = s.clone();
    forged.assignment.source_task.clear();
    assert!(d.bound(&forged).is_err());
    forged = s;
    forged.source_id = "another-task".into();
    assert!(d.bound(&forged).is_err());
}
#[test]
fn invalid_tasks_cannot_be_already_solved_unreachable_or_underspecified() {
    let mut bad = task("ccna-power-fixture");
    bad.targets.pop();
    assert!(bad.validate().is_err());
    let mut bad = task("ccna-power-fixture");
    bad.targets.push(Target::Powered);
    assert!(bad.validate().is_err());
    let mut bad = task("ccna-power-fixture");
    bad.id = "not a stable identifier".into();
    assert!(bad.validate().is_err());
    let mut bad = task("ccna-power-fixture");
    bad.instructions = "x".repeat(1601);
    assert!(bad.validate().is_err());
    let mut t = task("ccna-power-fixture");
    t.initial = t.reference.clone();
    assert!(t.validate().is_err());
    let mut t = task("ccna-power-fixture");
    t.reference = t.initial.clone();
    assert!(t.validate().is_err());
    let mut d = definition();
    d.example = d.assignment.clone();
    assert!(d.validate().is_err());
}
#[test]
fn malformed_or_oversized_imports_and_double_occupied_ports_fail_atomically() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let before = s.clone();
    let mut bad = s.assignment.plan.clone();
    bad.attachments[1] = bad.attachments[0];
    assert!(s.action(&d, &Action::Edit { plan: bad }, false).is_err());
    assert_eq!(s, before);
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]["plan"]["enabled"] =
        serde_json::json!([true, true, true, true, true, true, true, true, true]);
    assert!(serde_json::from_value::<Session>(v).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]["source_task"] = serde_json::Value::String("x".repeat(16385));
    assert!(serde_json::from_value::<Session>(v).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]["plan"]["host_command"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<Session>(v).is_err());
}

#[test]
fn standalone_saved_shape_rejects_garbage_and_inconsistent_embedded_sources() {
    let d = definition();
    let s = Session::new(&d).unwrap();
    s.validate().unwrap();
    let mut bad = s.clone();
    bad.assignment.source_task = "not JSON".into();
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.example.source_task = bad.assignment.source_task.clone();
    assert!(bad.validate().is_err());
    let mut bad = s;
    bad.assignment.plan.attachments[1] = bad.assignment.plan.attachments[0];
    assert!(bad.validate().is_err());
}
