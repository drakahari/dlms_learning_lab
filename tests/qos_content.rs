//! Authored source gate only: no registration or native learning acceptance yet.
use dlms_learning_lab::{qos as c, qos_workspace as w};
fn rows() -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(include_str!("../scenarios/qos-lessons-v1.json"))
        .unwrap()["scenarios"]
        .as_array()
        .unwrap()
        .clone()
}
fn definition(row: &serde_json::Value) -> w::Definition {
    serde_json::from_value(row["goal"]["definition"].clone()).unwrap()
}
#[test]
fn three_authored_assignments_and_different_examples_reach_every_behavioral_case() {
    let rows = rows();
    assert_eq!(rows.len(), 3);
    for row in rows {
        let d = definition(&row);
        d.validate().unwrap();
        assert_eq!(d.initial.id, row["id"]);
        assert_eq!(d.initial.task, row["objective"]);
        let mut s: w::Session =
            serde_json::from_value(row["initial"]["simulation"].clone()).unwrap();
        assert_eq!(s, w::Session::new(&d).unwrap());
        assert!(!d.solved(&s));
        let assignment = s.assignment.clone();
        s.practice
            .replace(&d.example, d.example_worked.clone())
            .unwrap();
        let cases = s.practice.run(&d.example).unwrap();
        assert_eq!(cases.len(), 3);
        assert!(cases.iter().all(w::CaseResult::passed));
        assert_eq!(s.assignment, assignment);
        assert!(!d.solved(&s));
        let answer = row["solution"][0].as_str().unwrap();
        let text = s.execute_reference(&d, answer).unwrap();
        assert!(text.contains("3/3"));
        assert!(d.solved(&s));
        assert!(d.checks(&s).iter().all(|r| r.0));
    }
}
#[test]
fn plausible_wrong_marking_fifo_capacity_and_dropping_shortcuts_fail() {
    for row in rows() {
        let d = definition(&row);
        let mut s = w::Session::new(&d).unwrap();
        let good = c::Policy::from_json(
            row["solution"][0]
                .as_str()
                .unwrap()
                .strip_prefix(w::PREFIX)
                .unwrap(),
            &d.initial.workloads[0].fixture,
        )
        .unwrap();
        for n in 0..4 {
            let mut p = good.clone();
            match n {
                0 => p.marks.values_mut().for_each(|m| *m = c::Mark::Set(0)),
                1 => {
                    p.scheduler = c::Scheduler::Fifo;
                    p.priority_dscp.clear();
                    p.high_buffer = 0;
                    p.standard_buffer = 1;
                }
                2 => {
                    p.standard_buffer = 64;
                }
                _ => p = d.initial.starter.clone(),
            }
            s.assignment.replace(&d.initial, p).unwrap();
            let _ = s.assignment.run(&d.initial);
            assert!(!d.solved(&s), "{} bad{n}", d.initial.id);
            assert!(d.checks(&s).iter().any(|r| !r.0));
        }
    }
}
#[test]
fn supported_queue_allocations_and_equivalent_guest_labels_are_accepted() {
    for row in rows().into_iter().skip(1) {
        let d = definition(&row);
        let good = c::Policy::from_json(
            row["solution"][0]
                .as_str()
                .unwrap()
                .strip_prefix(w::PREFIX)
                .unwrap(),
            &d.initial.workloads[0].fixture,
        )
        .unwrap();
        for (high, standard) in [(1, 7), (2, 6)] {
            let mut p = good.clone();
            p.high_buffer = high;
            p.standard_buffer = standard;
            if let Some(guest) = d.initial.streams.iter().find(|s| s.id.ends_with("-guest")) {
                p.marks.insert(guest.id.clone(), c::Mark::Set(63));
            }
            let r = d.initial.run(&p).unwrap();
            assert!(r.iter().all(w::CaseResult::passed), "{}", d.initial.id);
        }
    }
}
#[test]
fn reference_adapter_rejects_arbitrary_commands_and_later_bad_state_revokes_success() {
    for row in rows() {
        let d = definition(&row);
        let mut s = w::Session::new(&d).unwrap();
        s.execute_reference(&d, row["solution"][0].as_str().unwrap())
            .unwrap();
        assert!(d.solved(&s));
        let before = s.clone();
        for text in ["sh -c id", "ping example.com", "QOS POLICY\n{broken}"] {
            assert!(s.execute_reference(&d, text).is_err());
            assert_eq!(s, before);
        }
        s.assignment
            .replace(&d.initial, d.initial.starter.clone())
            .unwrap();
        assert!(!d.solved(&s));
        assert!(d.checks(&s)[0].1.contains("Not checked"));
    }
}
