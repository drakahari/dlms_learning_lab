//! Source-held task behavior, alternate methods, role and saved-shape checks.
#[path = "support/recursive_tree_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    recursive_tree_workspace::{self as work, Role, Session},
    script_teaching::View,
};
fn passed(d: &work::Definition, s: &Session) -> bool {
    d.checks(s).iter().all(|r| r.0)
}
fn solve(d: &work::Definition) -> Session {
    let mut s = Session::new(d).unwrap();
    for c in &d.reference {
        s.run(d, c, false).unwrap();
    }
    assert!(!passed(d, &s));
    assert!(
        s.check(d, false).unwrap().iter().all(|r| r.0),
        "{:?}",
        d.checks(&s)
    );
    s
}
#[test]
fn typed_sources_reach_complete_outcomes_and_examples_stay_separate() {
    for d in fixture::definitions() {
        d.validate_reachability().unwrap();
        let mut s = Session::new(&d).unwrap();
        assert!(!passed(&d, &s));
        let original = s.assignment.clone();
        for c in &d.example_commands {
            s.run(&d, c, true).unwrap();
        }
        assert!(s.check(&d, true).unwrap().iter().all(|r| r.0));
        assert_eq!(s.assignment, original);
        assert!(!passed(&d, &s));
        solve(&d);
    }
}
#[test]
fn private_copy_accepts_individual_mkdir_cp_steps_but_permissive_masks_and_missing_entries_fail() {
    let d = fixture::definitions().remove(0);
    let mut s = Session::new(&d).unwrap();
    for c in [
        "umask 077",
        "mkdir /work/private-copy",
        "mkdir /work/private-copy/docs",
        "mkdir /work/private-copy/empty-cache",
        "cp /work/incoming/README.txt /work/private-copy/README.txt",
        "cp /work/incoming/.index /work/private-copy/.index",
        "cp /work/incoming/docs/runbook.txt /work/private-copy/docs/runbook.txt",
    ] {
        s.run(&d, c, false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    s.run(&d, "umask 022", false).unwrap();
    assert!(!s.assignment.checked);
    let results = s.check(&d, false).unwrap();
    assert_eq!(results.iter().filter(|r| !r.0).count(), 1);
    assert!(results.last().unwrap().1.contains("077"));
    let mut s = Session::new(&d).unwrap();
    s.run(&d, "cp -R incoming private-copy", false).unwrap();
    assert!(s.check(&d, false).unwrap().iter().any(|r| !r.0));
    assert!(!d.assignment.checks(&s.assignment.tree)[3].0);
    s.run(&d, "umask 077", false).unwrap();
    assert!(
        !s.check(&d, false).unwrap().iter().all(|r| r.0),
        "Changing mask does not fix an existing permissive copy"
    );
}
#[test]
fn retirement_accepts_rebuild_or_individual_repairs_but_never_an_incomplete_handover() {
    let d = fixture::definitions().remove(1);
    let mut s = Session::new(&d).unwrap();
    for c in [
        "rm -r handover",
        "cp -R old-project handover",
        "rm -r old-project",
    ] {
        s.run(&d, c, false).unwrap();
    }
    assert!(s.check(&d, false).unwrap().iter().all(|r| r.0));
    assert_eq!(
        s.assignment.tree.files["/work/old-project-current/live.txt"],
        "Live project: keep this tree\n"
    );
    let mut early = Session::new(&d).unwrap();
    early.run(&d, "rm -r old-project", false).unwrap();
    let results = early.check(&d, false).unwrap();
    assert!(results[4].0, "Old names are retired");
    assert!(
        !results[0].0 && !results[1].0,
        "Missing transferred data must still fail"
    );
    assert!(results[0].1.contains(".index"));
    assert!(!passed(&d, &early));
}
#[test]
fn current_success_invalidates_after_breakage_but_read_queries_and_navigation_keep_it() {
    for d in fixture::definitions() {
        let mut s = solve(&d);
        s.run(&d, "ls -la", false).unwrap();
        s.run(&d, "cd /home/alex", false).unwrap();
        assert!(passed(&d, &s));
        let dest = d.assignment.task.destination();
        s.run(&d, &format!("chmod 000 {dest}/README.txt"), false)
            .unwrap();
        assert!(!s.assignment.checked);
        assert!(!passed(&d, &s));
        assert!(s.check(&d, false).unwrap().iter().any(|r| !r.0));
    }
}
#[test]
fn source_and_role_guards_refuse_swapped_worlds_foreign_context_and_outside_mutations_atomically() {
    for d in fixture::definitions() {
        let s = Session::new(&d).unwrap();
        let mut wrong = s.clone();
        wrong.practice.role = Role::Assignment;
        assert!(d.bound(&wrong).is_err());
        let mut wrong = s.clone();
        wrong.assignment.tree = s.practice.tree.clone();
        assert!(d.bound(&wrong).is_err());
        let mut wrong = s.clone();
        wrong.origin = "other-source".into();
        assert!(d.bound(&wrong).is_err());
        let mut s = s;
        let old = s.clone();
        assert!(s.run(&d, "mkdir /home/alex/outside", false).is_err());
        assert_eq!(s, old);
        let mut wrong = s.clone();
        wrong.assignment.tree.user = "bea".into();
        assert!(d.bound(&wrong).is_err());
        assert!(wrong.run(&d, "help", false).is_err());
    }
}
#[test]
fn state_codec_preserves_both_unfinished_drafts_but_never_grants_credit_from_navigation_or_check_flags()
 {
    for d in fixture::definitions() {
        let mut s = Session::new(&d).unwrap();
        s.assignment.command = "unfinished assignment".into();
        s.practice.command = "unfinished example".into();
        s.view = View::Example;
        s.example_command = 1;
        let decoded = Session::decode(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(s, decoded);
        d.bound(&decoded).unwrap();
        let mut fake = s.clone();
        fake.assignment.checked = true;
        assert!(!passed(&d, &fake));
        assert_eq!(s.assignment.command, "unfinished assignment");
        assert_eq!(s.practice.command, "unfinished example");
    }
    assert!(Session::decode(&vec![b' '; work::MAX_SESSION + 1]).is_err());
}
#[test]
fn metadata_forgery_and_unsupported_commands_do_not_escape_or_rewrite_valid_work() {
    let d = fixture::definitions().remove(0);
    let mut s = Session::new(&d).unwrap();
    let old = s.clone();
    for c in [
        "curl example.test",
        "sh /work/incoming/README.txt",
        "cp -a incoming private-copy",
        "rm -rf incoming",
    ] {
        assert!(s.run(&d, c, false).is_err());
        assert_eq!(s, old);
    }
    let mut wrong = d.clone();
    wrong.version = 999;
    assert!(Session::new(&wrong).is_err());
    let mut wrong = s;
    wrong.assignment.tree.nodes.get_mut("/").unwrap().mode = 0o777;
    assert!(d.bound(&wrong).is_err());
}
