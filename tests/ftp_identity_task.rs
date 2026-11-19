use dlms_learning_lab::{
    ftp_identity::World,
    ftp_identity_task::{Expected, Task},
};
#[path = "support/ftp_identity_fixture.rs"]
mod fixture;
#[path = "support/ftp_identity_task_fixture.rs"]
mod task_fixture;
use task_fixture::task;
fn passed(t: &Task, w: &World) -> bool {
    t.checks(w).iter().all(|c| c.passed)
}
#[test]
fn distinct_inputs_and_real_transfer_effects_pass_without_mutating_assignment() {
    let t = task();
    t.validate().unwrap();
    let w = fixture::world();
    assert!(passed(&t, &w), "{:?}", t.checks(&w));
    assert_eq!(w, t.initial);
    assert!(!w.files.files.contains_key("/srv/alex/inbox/report"));
    // An expected constant that ignores the changed input fails the second trial.
    let mut wrong = t.clone();
    wrong.cases[1].steps[1].expected = Expected::Download {
        text: "first supplied input\n".into(),
    };
    assert!(!passed(&wrong, &w));
}
#[test]
fn equivalent_configuration_and_supported_permission_alternatives_pass() {
    let t = task();
    let mut w = t.initial.clone();
    w.save(
        fixture::settings() + "# alternative formatting\nlocal_enable = true\nlocal_umask=027\n",
    )
    .unwrap();
    w.restart().unwrap();
    assert!(passed(&t, &w));
    w.files.nodes.get_mut("/srv/alex/inbox").unwrap().mode = 0o750;
    assert!(passed(&t, &w));
    w.saved.push_str("# not a policy change\n");
    assert!(passed(&t, &w));
}
#[test]
fn blocked_everything_is_not_confinement_or_transfer_success() {
    let t = task();
    let mut w = t.initial.clone();
    w.saved.push_str("local_enable=NO\n");
    w.restart().unwrap();
    let rows = t.checks(&w);
    assert!(!rows.iter().all(|r| r.passed));
    assert!(rows.iter().any(|r| r.passed));
    assert!(rows
        .iter()
        .any(|r| !r.passed && r.detail.contains("Local account login is disabled")));
    w.save(fixture::settings()).unwrap();
    w.restart().unwrap();
    w.stop().unwrap();
    assert!(!passed(&t, &w));
}
#[test]
fn wrong_root_bad_write_gate_permissions_and_anonymous_policy_fail_specifically() {
    let t = task();
    for line in [
        "chroot_local_user=NO\n",
        "write_enable=NO\n",
        "anonymous_enable=YES\n",
        "download_enable=NO\n",
    ] {
        let mut w = t.initial.clone();
        w.saved.push_str(line);
        w.restart().unwrap();
        let checks = t.checks(&w);
        assert!(checks.iter().any(|r| !r.passed), "{line}");
    }
    let mut w = t.initial.clone();
    w.files.nodes.get_mut("/srv/alex/inbox").unwrap().mode = 0o500;
    let checks = t.checks(&w);
    assert!(checks
        .iter()
        .any(|r| !r.passed && r.detail.contains("write and search")));
    let mut w = t.initial.clone();
    w.files.nodes.get_mut("/srv/alex/private").unwrap().mode = 0o644;
    assert!(!passed(&t, &w));
}
#[test]
fn chroot_exception_inversion_and_partial_configuration_remain_visible() {
    let t = task();
    let mut w = t.initial.clone();
    w.saved.push_str("chroot_list_enable=YES\n");
    w.files
        .files
        .insert("/etc/vsftpd.chroot_list".into(), "alex\n".into());
    w.restart().unwrap();
    let rows = t.checks(&w);
    assert!(rows
        .iter()
        .any(|r| !r.passed && r.detail.contains("wrong root boundary")));
    assert!(rows.iter().any(|r| r.passed));
}
#[test]
fn success_clears_when_saved_policy_changes_or_restart_fails() {
    let t = task();
    let mut w = t.initial.clone();
    assert!(passed(&t, &w));
    w.saved.push_str("write_enable=NO\n");
    assert!(!passed(&t, &w));
    w.restart().unwrap();
    assert!(!passed(&t, &w));
    w.save(fixture::settings()).unwrap();
    w.restart().unwrap();
    assert!(passed(&t, &w));
    w.save("wrong_option=YES".into()).unwrap();
    assert!(w.restart().is_err());
    assert!(!passed(&t, &w));
}
#[test]
fn tampered_fixtures_and_outside_file_changes_are_refused() {
    let t = task();
    let mut w = t.initial.clone();
    w.accounts[0].home = "/srv/blair".into();
    assert!(t.bound(&w).is_err());
    let mut w = t.initial.clone();
    w.files
        .files
        .insert("/etc/secret".into(), "tampered".into());
    assert!(t.bound(&w).is_err());
    let mut w = t.initial.clone();
    w.files
        .files
        .insert("/etc/vsftpd.user_list".into(), "alex\nblair\n".into());
    assert!(t.bound(&w).is_ok());
}
#[test]
fn invalid_case_contracts_and_oversized_imports_fail_before_grading() {
    let t = task();
    for mutate in [0, 1, 2, 3] {
        let mut bad = t.clone();
        match mutate {
            0 => bad.cases[1].id = bad.cases[0].id.clone(),
            1 => {
                bad.cases[0].steps[0].expected = Expected::Download {
                    text: "wrong result type".into(),
                }
            }
            2 => bad.cases[0].inputs[0].path = "/missing".into(),
            _ => bad.cases = vec![bad.cases[0].clone()],
        };
        assert!(bad.validate().is_err());
    }
    let mut json = serde_json::to_value(&t).unwrap();
    json["initial"]["saved"] = serde_json::json!("x".repeat(16385));
    assert!(serde_json::from_value::<Task>(json).is_err());
}

#[test]
fn resulting_upload_mode_is_graded_and_equivalent_masks_are_accepted() {
    let mut t = task();
    for c in &mut t.cases {
        for s in &mut c.steps {
            if let Expected::Uploaded { mode, .. } = &mut s.expected {
                *mode = Some(0o600);
            }
        }
    }
    assert!(!passed(&t, &t.initial));
    for mask in ["077", "0177"] {
        let mut w = t.initial.clone();
        w.saved.push_str(&format!("local_umask={mask}\n"));
        w.restart().unwrap();
        assert!(passed(&t, &w), "{mask}: {:?}", t.checks(&w));
    }
}
