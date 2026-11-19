//! Authored pair integration: real catalog/units, live/future grading and progress.
use dlms_learning_lab::{
    autofs::runtime::Text,
    autofs_source as binding,
    autofs_workspace::{self as w, Operation},
    course,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-autofs-indirect-repair",
    "lpic2-autofs-direct-conversion",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted337().unwrap())
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    binding::context(&a, s).unwrap();
    a
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::Autofs { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::Autofs { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = fresh(s);
    let d = definition(s);
    for step in &d.reference {
        assert_eq!(
            session(&mut a)
                .apply(d, &step.operation, false)
                .unwrap()
                .status,
            step.status
        );
    }
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn current_catalog_has_two_distinct_tasks_and_retains_every_prior_lab_and_unit() {
    let now = catalog();
    assert_eq!(now.len(), 337);
    let old = pack::accepted335().unwrap();
    assert_eq!(old.len(), 335);
    let retained = now
        .iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .map(|s| serde_json::to_value(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        retained,
        old.iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>()
    );
    let units = course::accepted337(now).unwrap();
    assert_eq!(units.len(), 116);
    let old_units = course::accepted335(&old).unwrap();
    assert_eq!(old_units.len(), 115);
    assert_eq!(
        units
            .iter()
            .filter(|u| u.id != "autofs-local-maps")
            .map(|u| serde_json::to_value(u).unwrap())
            .collect::<Vec<_>>(),
        old_units
            .iter()
            .map(|u| serde_json::to_value(u).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(now.iter().filter(|s| s.track == "CCNA").count(), 74);
    assert_eq!(now.iter().filter(|s| s.track == "LPIC-1").count(), 114);
    assert_eq!(now.iter().filter(|s| s.track == "LPIC-2").count(), 72);
    assert_eq!(now.iter().filter(|s| s.track == "Bash/Shell").count(), 77);
    let at = units
        .iter()
        .position(|u| u.id == "autofs-local-maps")
        .unwrap();
    assert_eq!(units[at - 1].id, "lvm-filesystem-mounting");
    assert_eq!(units[at].labs, IDS);
    assert_eq!(
        now.iter().find(|s| s.id == IDS[1]).unwrap().prerequisites,
        vec![IDS[0]]
    );
}
#[test]
fn taught_methods_wrong_change_and_retry_reach_all_real_outcomes() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = definition(s);
        d.validate_reachability().unwrap();
        assert!(!progress::current_solved(&fresh(s), s));
        solve(s);
        assert!(
            d.example_steps
                .iter()
                .any(|s| matches!(s.operation, Operation::Save { .. }))
        );
        assert_eq!(
            d.reference.iter().filter(|s| s.status == 1).count(),
            usize::from(id == IDS[1])
        );
    }
}
#[test]
fn alternate_supported_option_order_whitespace_and_extra_inspection_pass() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = definition(s);
        let mut a = fresh(s);
        for step in &d.reference {
            let mut op = step.operation.clone();
            if let Operation::Save { text, .. } = &mut op {
                text.0 = format!(
                    "# learner notes\n{}",
                    text.0
                        .replace("-fstype=ext4,rw", "-rw,fstype=ext4")
                        .replace("-fstype=ext4,ro", "-ro,fstype=ext4")
                        .replace(' ', "\t")
                );
            }
            session(&mut a).apply(d, &op, false).unwrap();
        }
        session(&mut a).run(d, "findmnt -t ext4", false).unwrap();
        session(&mut a).run(d, w::CHECK, false).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn saved_only_no_refresh_no_expiry_and_one_wrong_target_are_partial_not_success() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = definition(s);
        let mut a = fresh(s);
        for step in d
            .reference
            .iter()
            .filter(|s| matches!(s.operation, Operation::Save { .. }))
        {
            session(&mut a).apply(d, &step.operation, false).unwrap();
        }
        session(&mut a).run(d, w::CHECK, false).unwrap();
        assert!(!progress::current_solved(&a, s));
        let rows = d.checks(session(&mut a));
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0));
        let mut a = solve(s);
        let file = d.assignment.context.map_files[0].clone();
        let correct = d
            .reference
            .iter()
            .find_map(|s| {
                if let Operation::Save { file: f, text } = &s.operation {
                    (f == &file).then_some(text.0.clone())
                } else {
                    None
                }
            })
            .unwrap();
        let wrong = correct.replace(",ro", ",rw");
        session(&mut a)
            .apply(
                d,
                &Operation::Save {
                    file,
                    text: Text(wrong),
                },
                false,
            )
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        session(&mut a).run(d, w::CHECK, false).unwrap();
        assert!(
            d.checks(session(&mut a))
                .iter()
                .any(|r| !r.0 && r.1.contains("Saved configuration"))
        );
    }
}
#[test]
fn malformed_map_reports_specific_feedback_and_correct_retry_remains_possible() {
    let s = catalog().iter().find(|s| s.id == IDS[0]).unwrap();
    let d = definition(s);
    let mut a = fresh(s);
    let file = d.assignment.context.map_files[0].clone();
    let text = "reports -fstype=ext4 /dev/ledger/reports\n".into();
    session(&mut a)
        .apply(
            d,
            &Operation::Save {
                file,
                text: Text(text),
            },
            false,
        )
        .unwrap();
    session(&mut a)
        .apply(d, &Operation::Idle { seconds: 30 }, false)
        .unwrap();
    let result = session(&mut a)
        .run(d, "cat /srv/teams/reports/keep.txt", false)
        .unwrap();
    assert!(result.contains("Map syntax error"));
    assert!(result.contains("colon"));
    assert!(!progress::current_solved(&a, s));
    for step in &d.reference {
        session(&mut a).apply(d, &step.operation, false).unwrap();
    }
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, s));
}
#[test]
fn native_progress_reopen_keeps_old_attempt_and_two_new_drafts_without_fabricating_credit() {
    let labs = catalog();
    let first = labs.iter().find(|s| s.id == IDS[0]).unwrap();
    let second = labs.iter().find(|s| s.id == IDS[1]).unwrap();
    let old = labs
        .iter()
        .find(|s| s.id == "lpic2-lvm-mount-repair")
        .unwrap();
    let old_attempt = Attempt::new(old, 1);
    let old_value = serde_json::to_value(&old_attempt).unwrap();
    let mut a = fresh(second);
    let d = definition(second);
    session(&mut a).assignment.selected = d.assignment.context.map_files[0].clone();
    session(&mut a)
        .edit(d, "# incomplete assignment\n".into(), false)
        .unwrap();
    session(&mut a).practice.selected = d.example.context.map_files[0].clone();
    session(&mut a)
        .edit(d, "# independent practice\n".into(), true)
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.favorites.insert(second.id.clone());
    p.attempts.insert(old.id.clone(), old_attempt);
    p.attempts.insert(first.id.clone(), solve(first));
    p.attempts.insert(second.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, labs).unwrap();
    let mut q = progress::load(&path, labs).unwrap();
    assert_eq!(q.attempts.len(), 3);
    assert_eq!(q.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&q.attempts[&old.id]).unwrap(),
        old_value
    );
    assert!(progress::current_solved(&q.attempts[&first.id], first));
    assert!(!progress::current_solved(&q.attempts[&second.id], second));
    let restored = session(q.attempts.get_mut(&second.id).unwrap());
    assert_eq!(restored.assignment.draft(), "# incomplete assignment\n");
    assert_eq!(restored.practice.draft(), "# independent practice\n");
    q.attempts
        .insert(second.id.clone(), Attempt::new(second, 4));
    assert_eq!(
        serde_json::to_value(&q.attempts[&old.id]).unwrap(),
        old_value
    );
    assert!(progress::current_solved(&q.attempts[&first.id], first));
}
