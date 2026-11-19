use dlms_learning_lab::{qos as c, qos_workspace as w};
use std::collections::{BTreeMap, BTreeSet};
fn task(name: &str) -> w::Task {
    let bulk = format!("{name}-bulk");
    let voice = format!("{name}-voice");
    let flows = BTreeSet::from([bulk.clone(), voice.clone()]);
    let cases = (0..2)
        .map(|n| w::Workload {
            id: format!("{name}-case{n}"),
            label: format!("Burst case {n}"),
            fixture: c::Fixture {
                flows: flows.clone(),
                buffer_budget: 8,
                packets: vec![
                    c::Packet {
                        id: format!("{name}-b1"),
                        flow: bulk.clone(),
                        arrival: 0,
                        dscp: 0,
                    },
                    c::Packet {
                        id: format!("{name}-b2"),
                        flow: bulk.clone(),
                        arrival: 0,
                        dscp: 0,
                    },
                    c::Packet {
                        id: format!("{name}-v1"),
                        flow: voice.clone(),
                        arrival: n,
                        dscp: 46,
                    },
                ],
            },
        })
        .collect();
    w::Task{id:name.into(),task:"Keep the supplied voice packets labeled 46, deliver every bulk packet, and give voice zero queue waiting in both provided workloads.".into(),streams:vec![w::Stream{id:bulk.clone(),label:"Bulk stream".into(),description:"A supplied equal-size report burst.".into()},w::Stream{id:voice.clone(),label:"Voice stream".into(),description:"A supplied equal-size interactive packet.".into()}],workloads:cases,requirements:vec![w::Requirement::Mark{flow:voice.clone(),dscp:46},w::Requirement::MaxWait{flow:voice.clone(),slots:0},w::Requirement::DeliverAll{flow:bulk.clone()}],starter:c::Policy{marks:BTreeMap::from([(bulk,c::Mark::Keep),(voice,c::Mark::Keep)]),scheduler:c::Scheduler::Fifo,priority_dscp:BTreeSet::new(),high_buffer:0,standard_buffer:8}}
}
fn worked(t: &w::Task) -> c::Policy {
    let mut p = t.starter.clone();
    p.scheduler = c::Scheduler::StrictPriority;
    p.priority_dscp = BTreeSet::from([46]);
    p.high_buffer = 2;
    p.standard_buffer = 6;
    p
}
fn definition() -> w::Definition {
    let initial = task("assignment");
    let example = task("practice");
    let example_worked = worked(&example);
    let d = w::Definition {
        initial,
        example,
        example_worked,
    };
    d.validate().unwrap();
    d
}
#[test]
fn multiple_real_cases_pass_for_supported_alternative_policies() {
    let d = definition();
    for code in [0, 10, 63] {
        let mut p = worked(&d.initial);
        p.marks.insert("assignment-bulk".into(), c::Mark::Set(code));
        let results = d.initial.run(&p).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(w::CaseResult::passed));
    }
}
#[test]
fn wrong_configuration_has_specific_actual_wait_feedback_and_preserves_correct_checks() {
    let d = definition();
    let r = d.initial.run(&d.initial.starter).unwrap();
    assert!(r.iter().all(|r| !r.passed()));
    for case in r {
        assert!(case.checks[0].0);
        assert!(!case.checks[1].0);
        assert!(case.checks[1].1.contains("actual largest wait"));
        assert!(case.checks[2].0);
    }
}
#[test]
fn overall_failure_does_not_relabel_a_correct_individual_case() {
    let mut d = definition();
    d.initial.requirements[1] = w::Requirement::MaxWait {
        flow: "assignment-voice".into(),
        slots: 1,
    };
    d.validate().unwrap();
    let r = d.initial.run(&d.initial.starter).unwrap();
    assert!(!r[0].passed());
    assert!(r[1].passed());
    assert!(!r.iter().all(w::CaseResult::passed));
}
#[test]
fn separate_practice_never_earns_assignment_credit_and_changes_clear_current_success() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let original = s.assignment.clone();
    s.practice
        .replace(&d.example, d.example_worked.clone())
        .unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert_eq!(s.assignment, original);
    assert!(!d.solved(&s));
    s.assignment
        .replace(&d.initial, worked(&d.initial))
        .unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    s.assignment
        .replace(&d.initial, d.initial.starter.clone())
        .unwrap();
    assert!(s.assignment.tested.is_none());
    assert!(!d.solved(&s));
    assert!(s.practice.passed(&d.example));
}
#[test]
fn partial_invalid_budget_and_failure_diagnostics_resume_then_repair() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let mut p = worked(&d.initial);
    p.standard_buffer = 8;
    s.assignment.replace(&d.initial, p).unwrap();
    assert!(s.assignment.run(&d.initial).is_err());
    assert!(
        s.assignment
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("buffer budget")
    );
    assert!(!d.solved(&s));
    let text = serde_json::to_string(&s).unwrap();
    let mut r = w::Session::from_json(&text, &d).unwrap();
    assert_eq!(s, r);
    r.assignment
        .replace(&d.initial, worked(&d.initial))
        .unwrap();
    r.assignment.run(&d.initial).unwrap();
    assert!(r.assignment.diagnostic.is_none());
    assert!(d.solved(&r));
}
#[test]
fn foreign_context_and_oversized_or_fabricated_state_refuse_without_replacing_drafts() {
    let d = definition();
    let s = w::Session::new(&d).unwrap();
    for n in 0..5 {
        let mut v = serde_json::to_value(&s).unwrap();
        match n {
            0 => v["origin"] = serde_json::json!("different-task"),
            1 => v["assignment"]["diagnostic"] = serde_json::json!("x".repeat(513)),
            2 => v["assignment"]["passed"] = serde_json::json!(true),
            3 => v["assignment"]["draft"]["marks"]["foreign"] = serde_json::json!("Keep"),
            _ => {
                v["assignment"]["tested"] = serde_json::json!({"marks":{},"scheduler":"Fifo","priority_dscp":[],"high_buffer":0,"standard_buffer":8})
            }
        }
        assert!(w::Session::from_json(&v.to_string(), &d).is_err(), "{n}");
    }
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION_BYTES + 1), &d).is_err());
    let mut direct = s.clone();
    direct.assignment.tested = Some(d.initial.starter.clone());
    assert!(!d.solved(&direct));
}
#[test]
fn invalid_authored_tasks_and_unreachable_worked_example_refuse() {
    let good = definition();
    for n in 0..7 {
        let mut d = good.clone();
        match n {
            0 => d.initial.workloads[1].id = d.initial.workloads[0].id.clone(),
            1 => d.initial.workloads[0]
                .fixture
                .packets
                .pop()
                .map(|_| ())
                .unwrap(),
            2 => d
                .initial
                .requirements
                .push(d.initial.requirements[0].clone()),
            3 => {
                d.initial.requirements[0] = w::Requirement::Mark {
                    flow: "assignment-voice".into(),
                    dscp: 64,
                }
            }
            4 => {
                d.initial.requirements[2] = w::Requirement::MinDelivered {
                    flow: "assignment-bulk".into(),
                    count: 3,
                }
            }
            5 => d.example_worked = d.example.starter.clone(),
            _ => d.example = d.initial.clone(),
        }
        assert!(d.validate().is_err(), "{n}");
    }
}
#[test]
fn reset_of_one_session_preserves_other_work_and_has_no_stale_credit() {
    let d = definition();
    let mut chosen = w::Session::new(&d).unwrap();
    chosen
        .assignment
        .replace(&d.initial, worked(&d.initial))
        .unwrap();
    chosen.assignment.run(&d.initial).unwrap();
    let untouched = chosen.clone();
    assert!(d.solved(&chosen));
    chosen = w::Session::new(&d).unwrap();
    assert!(!d.solved(&chosen));
    assert!(d.solved(&untouched));
    let restored = w::Session::from_json(&serde_json::to_string(&untouched).unwrap(), &d).unwrap();
    assert_eq!(restored, untouched);
    assert!(d.solved(&restored));
}
