//! Source-bound replay, real multiple-target outcome grading, separate drafts.
#[path = "support/autofs_fixture.rs"]
mod fixture;
use dlms_learning_lab::autofs::runtime::{Name, Text};
use dlms_learning_lab::autofs_workspace::{self as work, Operation, Session};
use fixture::*;
use serde_json::json;
fn pass(s: &mut Session, d: &work::Definition, practice: bool) {
    s.run(d, work::CHECK, practice).unwrap();
    assert!(
        s.work(practice)
            .checks(Session::activity(d, practice))
            .iter()
            .all(|r| r.0)
    );
}
#[test]
fn both_complete_methods_and_the_taught_wrong_change_retry_are_reachable() {
    for direct in [false, true] {
        let d = definition(direct);
        d.validate_reachability().unwrap();
        let mut s = Session::new(&d).unwrap();
        assert!(d.checks(&s).iter().any(|r| !r.0));
        for step in &d.reference {
            assert_eq!(
                s.apply(&d, &step.operation, false).unwrap().status,
                step.status
            );
        }
        pass(&mut s, &d, false);
    }
}
#[test]
fn correct_saved_text_is_only_partial_until_live_and_active_state_are_repaired() {
    for direct in [false, true] {
        let d = definition(direct);
        let mut s = Session::new(&d).unwrap();
        s.apply(&d, &d.reference[0].operation, false).unwrap();
        s.run(&d, work::CHECK, false).unwrap();
        let rows = d.checks(&s);
        assert!(
            rows.iter()
                .any(|r| r.0 && r.1.contains("Saved configuration"))
        );
        assert!(rows.iter().any(|r| !r.0));
        assert!(rows.iter().any(|r| !r.0 && r.1.contains("Current mount")));
        if direct {
            assert!(
                rows.iter()
                    .any(|r| !r.0 && r.1.contains("Active lookup policy"))
            );
        }
    }
}
#[test]
fn alternate_semantically_equivalent_maps_and_read_order_pass() {
    for direct in [false, true] {
        let d = definition(direct);
        let mut s = Session::new(&d).unwrap();
        for step in &d.reference {
            let mut op = step.operation.clone();
            if let Operation::Save { text, .. } = &mut op {
                text.0 = format!(
                    "# alternative order\n{}",
                    text.0.lines().rev().collect::<Vec<_>>().join("\n")
                )
                .replace("-fstype=ext4,rw", "-rw,fstype=ext4")
                .replace("-fstype=ext4,ro", "-ro,fstype=ext4");
            }
            s.apply(&d, &op, false).unwrap();
        }
        pass(&mut s, &d, false);
    }
}
#[test]
fn wrong_filesystem_correct_looking_data_and_late_saved_policy_fail() {
    let mut d = definition(false); // Actual distinct identities matter even if file bytes happen to match.
    let bytes = d
        .assignment
        .world
        .read_file("team/draft", "/keep.txt")
        .unwrap()
        .to_owned();
    d.assignment
        .world
        .groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("reference")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = bytes;
    let mut s = Session::new(&d).unwrap();
    s.run(&d, "cat /srv/teams/audit/keep.txt", false).unwrap();
    s.run(&d, work::CHECK, false).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Current mount"))
    );
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    for step in &d.reference {
        s.apply(&d, &step.operation, false).unwrap();
    }
    pass(&mut s, &d, false);
    s.apply(&d, &save(INDIRECT, indirect("/dev/team/reference")), false)
        .unwrap();
    assert!(!s.assignment.checked);
    s.run(&d, work::CHECK, false).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("New lookup after expiry"))
    );
}
#[test]
fn examples_never_change_assignment_draft_or_award_completion() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    s.assignment.selected = Name(INDIRECT.into());
    s.edit(&d, "# unfinished assignment\n".into(), false)
        .unwrap();
    let original = s.assignment.clone();
    for step in &d.example_steps {
        s.apply(&d, &step.operation, true).unwrap();
    }
    pass(&mut s, &d, true);
    assert_eq!(s.assignment, original);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let raw = serde_json::to_string(&s).unwrap();
    let restored: Session = serde_json::from_str(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert_eq!(restored.assignment.draft(), "# unfinished assignment\n");
}
#[test]
fn edits_invalidate_current_success_but_do_not_apply_until_explicit_save() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    for step in &d.reference {
        s.apply(&d, &step.operation, false).unwrap();
    }
    pass(&mut s, &d, false);
    let current = s.assignment.runtime.clone();
    s.assignment.selected = Name(INDIRECT.into());
    s.edit(&d, indirect("/dev/team/reference"), false).unwrap();
    assert!(!s.assignment.checked);
    assert_eq!(s.assignment.runtime, current);
    s.run(&d, work::CHECK, false).unwrap();
    assert!(!d.checks(&s)[0].0);
    s.save(&d, false).unwrap();
    assert!(!s.assignment.checked);
    s.run(&d, work::CHECK, false).unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn failed_read_after_mount_is_replayed_as_failure_with_real_side_effects() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    let out = s
        .apply(&d, &command("cat /data/audit/missing.txt"), false)
        .unwrap();
    assert_eq!(out.status, 1);
    assert!(out.display().contains("Lookup failed"));
    assert!(
        s.assignment
            .runtime
            .mounted
            .storage
            .live()
            .contains_key("/data/audit")
    );
    assert_eq!(s.assignment.history.len(), 1);
    d.bound(&s).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    d.bound(&serde_json::from_str(&raw).unwrap()).unwrap();
}
#[test]
fn unsupported_commands_paths_and_packets_never_execute_or_mutate_state() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    for text in [
        "sh script",
        "cat /etc/passwd",
        "cat /srv/teams/report/../keep.txt",
        "cat /srv/teams/report/keep.txt;id",
        "mount /srv/teams/report",
        "automount -m",
        "systemctl restart sshd",
        "cat /etc/auto.teams | sh",
        "systemctl reload autofs\ncat /etc/auto.master",
    ] {
        let before = s.clone();
        assert!(s.run(&d, text, false).is_err(), "{text}");
        assert_eq!(s, before);
    }
    let before = s.clone();
    assert!(
        s.apply(
            &d,
            &Operation::Save {
                file: Name("/home/personal".into()),
                text: Text("hello".into())
            },
            false
        )
        .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn replay_rejects_forged_live_results_histories_and_changed_source() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    for step in &d.reference {
        s.apply(&d, &step.operation, false).unwrap();
    }
    pass(&mut s, &d, false);
    let mut forged = s.clone();
    forged.assignment.history.clear();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.transcript[0].result.status = 1;
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.runtime.clock += 1;
    assert!(d.bound(&forged).is_err());
    let mut wrong = d.clone();
    wrong.assignment.saved.master.0.clear();
    assert!(wrong.bound(&s).is_err());
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["assignment"]["history"] = json!(vec![command("help"); 65]);
    assert!(serde_json::from_value::<Session>(raw).is_err());
}
#[test]
fn action_limit_retains_all_work_and_selected_reset_restores_only_that_attempt() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    for _ in 0..work::MAX_HISTORY {
        s.run(&d, "help", false).unwrap();
    }
    let before = s.clone();
    assert!(s.run(&d, "help", false).unwrap_err().contains("64 actions"));
    assert_eq!(s, before);
    let reset = Session::new(&d).unwrap();
    assert_eq!(reset.assignment.runtime, d.assignment.initial().unwrap());
    assert!(reset.assignment.history.is_empty());
    assert!(reset.practice.history.is_empty());
}
#[test]
fn missing_prerequisite_source_separation_and_impossible_reference_are_rejected() {
    let mut d = definition(false);
    d.example = d.assignment.clone();
    assert!(d.validate().is_err());
    let mut d = definition(false);
    d.reference = vec![work::Step {
        operation: command("help"),
        explanation: "This cannot repair storage".into(),
        status: 0,
    }];
    assert!(d.validate_reachability().is_err());
    let mut d = definition(false);
    d.experiment.operation = command("help");
    assert!(d.validate_reachability().is_err());
    let mut d = definition(false);
    d.assignment.targets[0].original_volume = u32::MAX;
    assert!(d.validate().is_err());
}
