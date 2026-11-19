//! Size-selected evidence outcomes; this is not a registered curriculum pack.
#[path = "support/recursive_tree_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    filesystem::size_predicate::Criterion,
    recursive_tree_workspace::{self as work, Session, Task},
};

fn task(case: usize) -> work::Definition {
    let mut d = fixture::definitions().remove(0);
    let operand = match case {
        0 => "+40c",
        1 => "-1k",
        _ => "+1k",
    };
    for a in [&mut d.assignment, &mut d.example] {
        let src = a.task.source().to_string();
        let dest = a.task.destination().to_string();
        a.initial
            .files
            .insert(format!("{src}/.index"), String::new());
        a.initial.files.insert(
            format!("{src}/README.txt"),
            "a".repeat(if case == 2 { 1024 } else { 64 }),
        );
        a.initial.files.insert(
            format!("{src}/docs/runbook.txt"),
            "b".repeat(if case == 2 { 1025 } else { 128 }),
        );
        let mut node = a.initial.nodes[&src].clone();
        node.owner = "alex".into();
        node.group = "staff".into();
        node.mode = 0o700;
        a.initial.nodes.insert(dest.clone(), node);
        a.task = Task::Collect {
            source: src,
            destination: dest,
            criterion: Criterion::parse(operand).unwrap(),
            private_defaults: true,
        };
    }
    fn recipe(a: &work::Activity, case: usize) -> Vec<String> {
        let src = a.task.source();
        let dst = a.task.destination();
        let mut c = vec!["umask 077".into()];
        if case != 1 {
            c.push(format!("mkdir {dst}/docs"));
        }
        for p in match case {
            0 => vec!["README.txt", "docs/runbook.txt"],
            1 => vec![".index"],
            _ => vec!["docs/runbook.txt"],
        } {
            c.push(format!("cp {src}/{p} {dst}/{p}"));
        }
        c
    }
    d.reference = recipe(&d.assignment, case);
    d.example_commands = recipe(&d.example, case);
    d
}

#[test]
fn multiple_byte_and_rounded_unit_tasks_reach_state_outcomes_with_independent_practice() {
    for case in 0..3 {
        let d = task(case);
        d.validate_reachability().unwrap();
        let mut s = Session::new(&d).unwrap();
        let assignment = s.assignment.clone();
        for c in &d.example_commands {
            s.run(&d, c, true).unwrap();
        }
        assert!(s.check(&d, true).unwrap().iter().all(|r| r.0));
        assert_eq!(s.assignment, assignment);
        assert!(!d.checks(&s).iter().all(|r| r.0));
        // Query output is diagnostic; commands themselves do not award completion.
        let operand = match case {
            0 => "+40c",
            1 => "-1k",
            _ => "+1k",
        };
        let out = s
            .run(
                &d,
                &format!(
                    "find {} -type f -size {operand}",
                    d.assignment.task.source()
                ),
                false,
            )
            .unwrap();
        assert!(!out.is_empty());
        assert!(!d.checks(&s).iter().all(|r| r.0));
        for c in &d.reference {
            s.run(&d, c, false).unwrap();
        }
        assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
        let source = d.assignment.task.source();
        let dest = d.assignment.task.destination();
        let excluded = if case == 1 { "README.txt" } else { ".index" };
        s.run(
            &d,
            &format!("cp {source}/{excluded} {dest}/{excluded}"),
            false,
        )
        .unwrap();
        assert!(!s.assignment.checked);
        let rows = s.check(&d, false).unwrap();
        assert!(!rows[0].0);
        assert!(
            rows[0]
                .1
                .contains(if case == 0 { "UTF-8 bytes" } else { "round up" })
        );
        assert!(
            rows[5].0,
            "An extra evidence member must not falsely report altered unrelated entries"
        );
        s.run(&d, &format!("rm {dest}/{excluded}"), false).unwrap();
        assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    }
}

#[test]
fn wrong_units_copied_all_missing_parents_and_changed_sources_never_pass() {
    let d = task(1);
    let mut s = Session::new(&d).unwrap();
    s.run(&d, "umask 077", false).unwrap();
    s.run(
        &d,
        "cp /work/incoming/README.txt /work/private-copy/README.txt",
        false,
    )
    .unwrap();
    let c = s.check(&d, false).unwrap();
    assert!(!c[0].0 && c[0].1.contains("Missing") && c[0].1.contains("Unexpected"));
    let mut all = Session::new(&d).unwrap();
    all.run(&d, "cp -R /work/incoming /work/private-copy", false)
        .unwrap();
    assert!(all.check(&d, false).unwrap().iter().any(|r| !r.0));
    let d = task(0);
    let mut s = Session::new(&d).unwrap();
    let old = s.clone();
    assert!(
        s.run(
            &d,
            "cp /work/incoming/docs/runbook.txt /work/private-copy/docs/runbook.txt",
            false
        )
        .is_err()
    );
    assert_eq!(s, old);
    for c in &d.reference {
        s.run(&d, c, false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    // Imported valid-but-altered source data must fail even with a stored check flag.
    s.assignment
        .tree
        .files
        .get_mut("/work/incoming/README.txt")
        .unwrap()
        .push('x');
    assert!(s.assignment.checked);
    assert!(!d.checks(&s).iter().all(|r| r.0));
    assert!(!d.checks(&s)[4].0);
    let mut mixed = d.clone();
    mixed.example = fixture::definitions().remove(0).example;
    assert!(Session::new(&mixed).is_err());
}

#[test]
fn earlier_recursive_assignments_do_not_silently_gain_size_queries() {
    for d in fixture::definitions() {
        let mut s = Session::new(&d).unwrap();
        let before = s.clone();
        assert!(s.run(&d, "find . -type f -size +1c", false).is_err());
        assert_eq!(s, before);
    }
}

#[test]
fn membership_and_unrelated_scratch_feedback_are_independent_without_any_false_success() {
    let d = task(0);
    let mut s = Session::new(&d).unwrap();
    for c in &d.reference {
        s.run(&d, c, false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    fn failures(rows: Vec<(bool, String)>) -> Vec<usize> {
        rows.iter()
            .enumerate()
            .filter(|(_, r)| !r.0)
            .map(|(i, _)| i)
            .collect()
    }
    s.run(
        &d,
        "cp /work/incoming/.index /work/private-copy/.index",
        false,
    )
    .unwrap();
    assert_eq!(failures(s.check(&d, false).unwrap()), vec![0]);
    s.run(&d, "touch /work/scratch.tmp", false).unwrap();
    assert_eq!(failures(s.check(&d, false).unwrap()), vec![0, 5]);
    s.run(&d, "rm /work/private-copy/.index", false).unwrap();
    assert_eq!(failures(s.check(&d, false).unwrap()), vec![5]);
    s.run(&d, "rm /work/scratch.tmp", false).unwrap();
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    s.assignment
        .tree
        .files
        .get_mut("/work/incoming/README.txt")
        .unwrap()
        .push('x');
    assert_eq!(failures(d.checks(&s)), vec![4]);
    assert!(
        s.assignment.checked,
        "Stored flag alone never preserves success after source damage"
    );
}
