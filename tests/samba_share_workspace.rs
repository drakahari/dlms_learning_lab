#[path = "support/samba_share_task_fixture.rs"]
mod fixture;
use dlms_learning_lab::{samba_share_config, samba_share_workspace::*};
use fixture::*;
fn repair(a: &mut Attempt, t: &Task) {
    a.account(t, "alex", true).unwrap();
    a.account(t, "blair", true).unwrap();
    a.command(t, "chmod g+rx /srv/team").unwrap();
}
#[test]
fn independent_behavior_cases_preserve_actual_state_and_accept_symbolic_or_octal_repairs() {
    let t = task();
    t.validate().unwrap();
    let initial = t.initial.clone();
    let mut a = Attempt::new(&t).unwrap();
    a.check(&t).unwrap();
    assert!(!a.solved(&t));
    assert!(a.checks(&t).iter().any(|r| r.passed));
    assert!(a.checks(&t).iter().any(|r| !r.passed));
    repair(&mut a, &t);
    a.check(&t).unwrap();
    assert!(a.solved(&t));
    assert_eq!(
        a.world.files.files, initial.files.files,
        "grading clones must not replace the learner's files with case inputs or write payloads"
    );
    let mut b = Attempt::new(&t).unwrap();
    b.account(&t, "blair", true).unwrap();
    b.command(&t, "chmod 750 /srv/team").unwrap();
    b.account(&t, "alex", true).unwrap();
    b.check(&t).unwrap();
    assert!(b.solved(&t));
    a.command(&t, "chmod g-x /srv/team").unwrap();
    assert!(!a.checked);
    assert!(!a.solved(&t));
    a.checked = true;
    assert!(
        !a.solved(&t),
        "imported checked flags do not grant stale success"
    );
}
#[test]
fn partial_accounts_or_modes_are_not_completion_and_examples_cannot_award_assignment_credit() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    repair(&mut s.practice, &d.example);
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.solved(&d.example));
    assert!(!s.assignment.solved(&d.assignment));
    s.assignment.account(&d.assignment, "alex", true).unwrap();
    s.assignment.check(&d.assignment).unwrap();
    assert!(!s.assignment.solved(&d.assignment));
    s.assignment.account(&d.assignment, "blair", true).unwrap();
    s.assignment.check(&d.assignment).unwrap();
    assert!(!s.assignment.solved(&d.assignment));
    s.assignment
        .command(&d.assignment, "chmod g+x /srv/team")
        .unwrap();
    s.assignment.check(&d.assignment).unwrap();
    assert!(s.assignment.solved(&d.assignment));
    let raw = serde_json::to_vec(&s).unwrap();
    let resumed: Session = serde_json::from_slice(&raw).unwrap();
    d.context(&resumed).unwrap();
    assert!(resumed.assignment.solved(&d.assignment));
    assert!(resumed.practice.solved(&d.example));
    let mut wrong = resumed.clone();
    wrong.practice.world.server = d.assignment.initial.server.clone();
    assert!(d.context(&wrong).is_err());
}
#[test]
fn editor_save_publication_and_runtime_write_are_distinct_and_edit_invalidates_checked_success() {
    let t = task();
    let mut a = Attempt::new(&t).unwrap();
    repair(&mut a, &t);
    a.check(&t).unwrap();
    assert!(a.solved(&t));
    let editor = a.editor.replace("read only = yes", "read only = no");
    a.edit(&t, editor).unwrap();
    assert!(!a.checked);
    assert_ne!(a.editor, a.world.saved_config().unwrap());
    a.check(&t).unwrap();
    assert!(!a.solved(&t));
    a.save(&t).unwrap();
    assert!(samba_share_config::parse(a.world.saved_config().unwrap()).is_ok());
    assert_ne!(a.world.saved_config().unwrap(), a.world.published);
    a.publish(&t).unwrap();
    a.check(&t).unwrap();
    assert!(
        a.solved(&t),
        "supported equivalent policy, not exact reference text, determines outcomes"
    );
    a.request.operation = Operation::Replace {
        path: "draft.txt".into(),
        text: "learner trial\n".into(),
    };
    a.run(&t).unwrap();
    assert_eq!(
        a.world.files.files["/srv/team/draft.txt"],
        "learner trial\n"
    );
    assert!(!a.checked);
    assert!(a.last_trial.as_ref().unwrap().result.is_ok());
    let raw = serde_json::to_vec(&a).unwrap();
    let resumed: Attempt = serde_json::from_slice(&raw).unwrap();
    assert_eq!(resumed.world, a.world);
    assert_eq!(resumed.editor, a.editor);
    assert!(
        resumed.last_trial.is_none(),
        "resume must not invent a runtime trial result"
    );
}
#[test]
fn missing_files_wrong_share_paths_or_plain_world_writable_modes_are_not_denial_shortcuts() {
    let t = task();
    let mut a = Attempt::new(&t).unwrap();
    repair(&mut a, &t);
    a.command(&t, "chmod 777 /srv/team/draft.txt").unwrap();
    a.command(&t, "chmod 777 /srv/team").unwrap();
    assert!(a.world.files.access(
        "casey",
        "/srv/team/draft.txt",
        dlms_learning_lab::filesystem::Access::ReadFile
    ));
    a.check(&t).unwrap();
    assert!(
        !a.solved(&t),
        "local non-team access requirement rejects chmod777"
    );
    a.command(&t, "chmod 660 /srv/team/draft.txt").unwrap();
    a.command(&t, "chmod 770 /srv/team").unwrap();
    a.edit(&t, a.editor.replace("path = /srv/team", "path = /missing"))
        .unwrap();
    a.save(&t).unwrap();
    a.publish(&t).unwrap();
    a.check(&t).unwrap();
    assert!(!a.solved(&t));
    let private = a
        .checks(&t)
        .into_iter()
        .find(|c| c.label == "Private file stays unreadable to editor")
        .unwrap();
    assert!(!private.passed);
    assert!(private.detail.contains("MissingFile"));
    let mut removed = t.reference.clone();
    removed.files.nodes.remove("/srv/team/private.txt");
    removed.files.files.remove("/srv/team/private.txt");
    assert!(t.owned(&removed).is_err());
    assert!(t.checks(&removed).iter().any(|c| !c.passed));
}
#[test]
fn invalid_source_matrix_unreachable_references_and_json_resource_escapes_fail_closed() {
    let t = task();
    let mut bad = t.clone();
    bad.reference = bad.initial.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.initial = bad.reference.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].inputs[0].path = bad.initial.configuration_path.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[1].id = bad.cases[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].expected = Expected::Read {
        path: "/srv/team/draft.txt".into(),
        text: "hardcoded wrong answer".into(),
    };
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases
        .retain(|c| !matches!(c.expected, Expected::Denied { .. }));
    assert!(bad.validate().is_err());
    let mut raw = serde_json::to_value(&t).unwrap();
    raw["cases"] = serde_json::json!(vec![&t.cases[0]; 17]);
    assert!(serde_json::from_value::<Task>(raw).is_err());
    let mut raw = serde_json::to_value(definition()).unwrap();
    raw["execute"] = "host".into();
    assert!(serde_json::from_value::<Definition>(raw).is_err());
}
#[test]
fn unsupported_terminal_forms_and_foreign_account_controls_do_not_mutate_any_state() {
    let t = task();
    let mut a = Attempt::new(&t).unwrap();
    let before = a.clone();
    for command in [
        "smbclient //host/team",
        "sudo anything",
        "sh -c anything",
        "rm /srv/team/private.txt",
        "cat /personal/file",
    ] {
        assert!(a.command(&t, command).is_err());
        assert_eq!(a, before);
    }
    assert!(a.account(&t, "casey", true).is_err());
    assert_eq!(a, before);
    a.edit(&t, "[team\npath=/srv/team".into()).unwrap();
    let draft = a.editor.clone();
    a.save(&t).unwrap();
    assert!(a.publish(&t).is_err());
    assert_eq!(a.editor, draft);
    assert_eq!(a.world.published, t.initial.published);
    a.terminal = "cat /etc/samba/smb.conf".into();
    let raw = serde_json::to_string(&a).unwrap();
    let resumed: Attempt = serde_json::from_str(&raw).unwrap();
    assert_eq!(resumed.editor, draft);
    assert_eq!(resumed.terminal, a.terminal);
    resumed.validate(&t).unwrap();
}

#[test]
fn wrong_file_write_cannot_pass_because_expected_file_already_contains_target_text() {
    let mut t = task();
    for w in [&mut t.initial, &mut t.reference] {
        w.files.nodes.insert(
            "/public/draft.txt".into(),
            w.files.nodes["/srv/team/draft.txt"].clone(),
        );
        w.files
            .files
            .insert("/public/draft.txt".into(), "wrong location\n".into());
    }
    t.validate().unwrap();
    let mut w = t.reference.clone();
    w.files
        .files
        .insert("/srv/team/draft.txt".into(), "replacement\n".into());
    let changed = w
        .saved_config()
        .unwrap()
        .replace("path = /srv/team", "path = /public");
    w.save(changed).unwrap();
    w.publish().unwrap();
    let writer = t
        .checks(&w)
        .into_iter()
        .find(|r| r.label == "Editor replaces the shared draft")
        .unwrap();
    assert!(!writer.passed);
    assert!(writer.detail.contains("Actual write: /public/draft.txt"));
    assert_eq!(
        w.files.files["/public/draft.txt"], "wrong location\n",
        "grading keeps test transactions separate"
    );
}
#[test]
fn source_cannot_replace_behavior_grading_with_an_exact_config_file_answer() {
    let mut t = task();
    t.filesystem
        .push(dlms_learning_lab::filesystem::Requirement::Contents {
            path: t.initial.configuration_path.clone(),
            text: t.initial.saved_config().unwrap().into(),
        });
    assert!(
        t.validate()
            .unwrap_err()
            .contains("not an exact configuration-file answer")
    );
}

#[test]
fn wrong_file_read_does_not_pass_when_output_happens_to_match_expected_text() {
    let mut t = task();
    for w in [&mut t.initial, &mut t.reference] {
        w.files.nodes.insert(
            "/public/draft.txt".into(),
            w.files.nodes["/srv/team/draft.txt"].clone(),
        );
        w.files
            .files
            .insert("/public/draft.txt".into(), "first sample\n".into());
    }
    t.validate().unwrap();
    let mut w = t.reference.clone();
    let changed = w
        .saved_config()
        .unwrap()
        .replace("path = /srv/team", "path = /public");
    w.save(changed).unwrap();
    w.publish().unwrap();
    let read = t
        .checks(&w)
        .into_iter()
        .find(|r| r.label == "Editor reads read-first")
        .unwrap();
    assert!(!read.passed);
    assert!(read.detail.contains("Actual file: /public/draft.txt"));
}
#[test]
fn default_read_only_rule_rejects_write_exceptions_and_history_is_bounded_honestly() {
    let mut t = task();
    t.policy.push(PolicyGoal::ReadOnly {
        share: "public".into(),
    });
    t.validate().unwrap();
    let mut w = t.reference.clone();
    w.save(format!(
        "{}\nwrite list = alex\n",
        w.saved_config().unwrap()
    ))
    .unwrap();
    w.publish().unwrap();
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| c.label == "public: default read-only policy" && !c.passed)
    );
    let mut a = Attempt::new(&t).unwrap();
    for _ in 0..40 {
        a.record("small output".into());
    }
    assert_eq!(a.output.len(), 32);
    a.record("é".repeat(17000));
    a.validate(&t).unwrap();
    assert_eq!(a.output.len(), 1);
    assert!(a.output[0].contains("History preview shortened"));
    assert!(a.output[0].len() <= 32768);
    assert!(a.command(&t, "groups alex").unwrap_err().contains("id -Gn"));
    assert!(a.command(&t, "id -Gn alex").unwrap().contains("team"));
    assert!(
        a.command(&t, "ls -ld /srv/team")
            .unwrap()
            .contains("/srv/team")
    );
}
#[test]
fn incomplete_or_malformed_trial_path_is_a_preservable_draft_but_never_executable() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    for path in ["", "../outside", "/personal/files"] {
        s.assignment.request.operation = Operation::Read { path: path.into() };
        d.context(&s).unwrap();
        let before = s.assignment.world.clone();
        assert!(s.assignment.run(&d.assignment).is_err());
        assert_eq!(s.assignment.world, before);
        let raw = serde_json::to_vec(&s).unwrap();
        let restored: Session = serde_json::from_slice(&raw).unwrap();
        d.context(&restored).unwrap();
        assert_eq!(restored, s);
    }
}
#[test]
fn failed_new_trial_clears_an_older_observation_without_changing_files() {
    let t = task();
    let mut a = Attempt::new(&t).unwrap();
    repair(&mut a, &t);
    a.run(&t).unwrap();
    assert!(a.last_trial.as_ref().unwrap().result.is_ok());
    let before = a.world.clone();
    a.request.operation = Operation::Read {
        path: "../outside".into(),
    };
    assert!(a.run(&t).unwrap_err().contains("inside the selected share"));
    assert_eq!(a.world, before);
    assert!(a.last_trial.is_none());
}
