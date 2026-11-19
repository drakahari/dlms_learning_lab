#[path = "support/samba_share_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    filesystem::Access,
    samba_share::{Account, Credential, FaultKind, World},
};
use fixture::*;
#[test]
fn share_policy_and_actual_unix_read_write_are_both_enforced() {
    let mut w = world();
    w.validate().unwrap();
    let editor = w.connect("TEAM", &valid("alex")).unwrap();
    let reviewer = w.connect("team", &valid("blair")).unwrap();
    assert_eq!(w.read(&editor, "draft.txt").unwrap(), "draft\n");
    assert_eq!(w.read(&reviewer, "draft.txt").unwrap(), "draft\n");
    w.replace(&editor, "draft.txt", "edited\n").unwrap();
    assert_eq!(w.read(&reviewer, "draft.txt").unwrap(), "edited\n");
    assert_eq!(
        w.replace(&reviewer, "draft.txt", "wrong").unwrap_err().kind,
        FaultKind::ReadOnly
    );
    assert_eq!(
        w.read(&editor, "private.txt").unwrap_err().kind,
        FaultKind::FilePermission
    );
    assert_eq!(w.files.files["/srv/team/private.txt"], "private fixture\n");
}
#[test]
fn write_list_overrides_read_list_and_read_only_but_never_filesystem_rights() {
    let mut w = world();
    w.save(
        w.saved_config()
            .unwrap()
            .replace("write list = @writers", "write list = @writers @reviewers"),
    )
    .unwrap();
    w.publish().unwrap();
    let c = w.connect("team", &valid("blair")).unwrap();
    w.replace(&c, "draft.txt", "permitted").unwrap();
    w.files.nodes.get_mut("/srv/team/draft.txt").unwrap().mode = 0o640;
    assert_eq!(
        w.replace(&c, "draft.txt", "denied").unwrap_err().kind,
        FaultKind::FilePermission
    );
    assert_eq!(w.files.files["/srv/team/draft.txt"], "permitted");
    w.files.nodes.get_mut("/srv/team").unwrap().mode = 0o700;
    assert_eq!(
        w.connect("team", &valid("blair")).unwrap_err().kind,
        FaultKind::DirectorySearch
    );
    assert_eq!(
        w.read(&c, "draft.txt").unwrap_err().kind,
        FaultKind::FilePermission
    );
}
#[test]
fn invalid_users_precedes_valid_users_and_guest_does_not_rescue_bad_named_login() {
    let mut w = world();
    w.save(w.saved_config().unwrap().replace(
        "valid users = @team",
        "valid users = alex @team\ninvalid users = @writers",
    ))
    .unwrap();
    w.publish().unwrap();
    assert_eq!(
        w.connect("team", &valid("alex")).unwrap_err().kind,
        FaultKind::SharePolicy
    );
    assert_eq!(
        w.connect(
            "public",
            &Credential::Invalid {
                user: "alex".into()
            }
        )
        .unwrap_err()
        .kind,
        FaultKind::Authentication
    );
    let c = w.connect("public", &Credential::Anonymous).unwrap();
    assert_eq!(c.user(), "guest");
    assert_eq!(w.read(&c, "notice.txt").unwrap(), "notice\n");
    assert_eq!(
        w.replace(&c, "notice.txt", "wrong").unwrap_err().kind,
        FaultKind::ReadOnly
    );
    assert_eq!(
        w.connect("team", &Credential::Anonymous).unwrap_err().kind,
        FaultKind::Authentication
    );
}
#[test]
fn missing_disabled_orphaned_and_invalid_credentials_are_distinct_and_nonmutating() {
    let mut w = world();
    let before = w.clone();
    assert_eq!(
        w.connect("team", &valid("casey")).unwrap_err().kind,
        FaultKind::Authentication
    );
    assert_eq!(w, before);
    w.set_account("alex", false).unwrap();
    assert_eq!(
        w.connect("team", &valid("alex")).unwrap_err().kind,
        FaultKind::AccountDisabled
    );
    w.accounts.push(Account {
        user: "orphan".into(),
        enabled: true,
    });
    assert_eq!(
        w.connect("team", &valid("orphan")).unwrap_err().kind,
        FaultKind::UnixIdentity
    );
    w.set_account("casey", true).unwrap();
    assert_eq!(
        w.connect("team", &valid("casey")).unwrap_err().kind,
        FaultKind::SharePolicy
    );
    assert_eq!(
        w.set_account("missing", true).unwrap_err().kind,
        FaultKind::UnixIdentity
    );
    assert_eq!(
        w.connect("team", &valid("root")).unwrap_err().kind,
        FaultKind::Unsupported
    );
}
#[test]
fn saved_applied_and_existing_connection_policy_are_not_conflated() {
    let mut w = world();
    let old = w.connect("team", &valid("blair")).unwrap();
    assert_eq!(old.generation(), 1);
    w.save(
        w.saved_config()
            .unwrap()
            .replace("write list = @writers", "write list = @writers @reviewers"),
    )
    .unwrap();
    let before_apply = w.connect("team", &valid("blair")).unwrap();
    assert_eq!(
        w.replace(&before_apply, "draft.txt", "not yet")
            .unwrap_err()
            .kind,
        FaultKind::ReadOnly
    );
    w.publish().unwrap();
    assert_eq!(w.generation, 2);
    let new = w.connect("team", &valid("blair")).unwrap();
    assert_eq!(new.generation(), 2);
    assert_eq!(
        w.replace(&old, "draft.txt", "still old").unwrap_err().kind,
        FaultKind::ReadOnly
    );
    w.replace(&new, "draft.txt", "new settings").unwrap();
    let published = w.published.clone();
    w.save("[team\npath=/srv/team".into()).unwrap();
    assert_eq!(w.publish().unwrap_err().kind, FaultKind::Syntax);
    assert_eq!(w.published, published);
    assert_eq!(w.generation, 2);
    assert_eq!(w.read(&old, "draft.txt").unwrap(), "new settings");
}
#[test]
fn config_success_does_not_prove_path_availability_and_connections_are_server_bound() {
    let mut w = world();
    let c = w.connect("team", &valid("alex")).unwrap();
    let mut other = w.clone();
    other.server = "other-server".into();
    assert_eq!(
        other.read(&c, "draft.txt").unwrap_err().kind,
        FaultKind::InvalidState
    );
    w.save(
        w.saved_config()
            .unwrap()
            .replace("path = /srv/team", "path = /missing"),
    )
    .unwrap();
    w.publish().unwrap();
    assert_eq!(
        w.connect("team", &valid("alex")).unwrap_err().kind,
        FaultKind::MissingFile
    );
    assert_eq!(w.read(&c, "draft.txt").unwrap(), "draft\n");
}
#[test]
fn group_changes_require_a_new_trial_and_chmod_is_checked_on_each_new_file_operation() {
    let mut w = world();
    let c = w.connect("team", &valid("alex")).unwrap();
    w.files
        .users
        .get_mut("alex")
        .unwrap()
        .retain(|g| g != "writers");
    assert_eq!(
        w.read(&c, "draft.txt").unwrap_err().kind,
        FaultKind::Unsupported
    );
    let c = w.connect("team", &valid("alex")).unwrap();
    assert_eq!(
        w.replace(&c, "draft.txt", "wrong").unwrap_err().kind,
        FaultKind::ReadOnly
    );
    w.files.nodes.get_mut("/srv/team/draft.txt").unwrap().mode = 0o600;
    assert_eq!(
        w.read(&c, "draft.txt").unwrap_err().kind,
        FaultKind::FilePermission
    );
}
#[test]
fn owner_class_precedence_is_the_existing_unix_model_not_a_second_smb_mode_rule() {
    let mut w = world();
    w.files.nodes.get_mut("/srv/team/draft.txt").unwrap().owner = "alex".into();
    w.files.nodes.get_mut("/srv/team/draft.txt").unwrap().mode = 0o060;
    let c = w.connect("team", &valid("alex")).unwrap();
    assert!(
        !w.files
            .access("alex", "/srv/team/draft.txt", Access::WriteFile)
    );
    assert_eq!(
        w.replace(&c, "draft.txt", "wrong").unwrap_err().kind,
        FaultKind::FilePermission
    );
}
#[test]
fn traversal_paths_size_limits_and_failed_writes_leave_all_state_intact() {
    let mut w = world();
    let c = w.connect("team", &valid("alex")).unwrap();
    let before = w.clone();
    for path in [
        "/srv/team/draft.txt",
        "../public/notice.txt",
        "./draft.txt",
        "sub/../draft.txt",
        "a\\b",
        "a//b",
        "",
        "a\0b",
    ] {
        assert_eq!(
            w.replace(&c, path, "wrong").unwrap_err().kind,
            FaultKind::Unsupported
        );
        assert_eq!(w, before);
    }
    assert_eq!(
        w.replace(&c, "missing.txt", "wrong").unwrap_err().kind,
        FaultKind::MissingFile
    );
    assert_eq!(w, before);
    assert_eq!(
        w.replace(&c, "draft.txt", &"x".repeat(32769))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    assert_eq!(w, before);
    assert_eq!(
        w.replace(&c, "draft.txt", "bad\0text").unwrap_err().kind,
        FaultKind::Unsupported
    );
    assert_eq!(w, before);
}
#[test]
fn root_share_join_is_canonical_and_state_roundtrip_does_not_serialize_privileged_handles() {
    let mut w = world();
    w.save("[public]\npath=/\nguest ok=yes\nread only=yes\n".into())
        .unwrap();
    w.publish().unwrap();
    let c = w.connect("public", &Credential::Anonymous).unwrap();
    assert_eq!(c.share(), "public");
    assert_eq!(w.read(&c, "public/notice.txt").unwrap(), "notice\n");
    let raw = serde_json::to_string(&w).unwrap();
    assert!(!raw.contains("policy"));
    let restored: World = serde_json::from_str(&raw).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, w);
    let mut raw = serde_json::to_value(&w).unwrap();
    raw["accounts"] = serde_json::json!(vec![
        Account {
            user: "alex".into(),
            enabled: true
        };
        17
    ]);
    assert!(serde_json::from_value::<World>(raw).is_err());
    let mut raw = serde_json::to_value(&w).unwrap();
    raw["exec"] = "anything".into();
    assert!(serde_json::from_value::<World>(raw).is_err());
}

#[test]
fn editor_and_virtual_terminal_share_one_saved_file_and_publish_reports_typed_failures() {
    let mut w = world();
    let changed = w
        .saved_config()
        .unwrap()
        .replace("write list = @writers", "write list = @writers @reviewers");
    w.save(changed.clone()).unwrap();
    assert_eq!(w.files.execute("cat /etc/samba/smb.conf").unwrap(), changed);
    let applied = w.published.clone();
    w.save("[team]\npath=/srv/team\nvfs objects=acl_xattr\n".into())
        .unwrap();
    assert_eq!(w.check_saved().unwrap_err().kind, FaultKind::Unsupported);
    assert_eq!(w.publish().unwrap_err().kind, FaultKind::Unsupported);
    assert_eq!(w.published, applied);
    w.save("[team\npath=/srv/team\n".into()).unwrap();
    assert_eq!(w.check_saved().unwrap_err().kind, FaultKind::Syntax);
    assert_eq!(w.published, applied);
    w.files.nodes.get_mut("/etc/samba/smb.conf").unwrap().mode = 0o400;
    let before = w.clone();
    assert_eq!(
        w.save(config()).unwrap_err().kind,
        FaultKind::FilePermission
    );
    assert_eq!(w, before);
    w.files.nodes.get_mut("/etc/samba/smb.conf").unwrap().mode = 0;
    assert_eq!(w.check_saved().unwrap_err().kind, FaultKind::FilePermission);
}
#[test]
fn ordinary_group_modes_and_share_policy_match_an_independent_boolean_oracle() {
    for read_only in [false, true] {
        for read_list in [false, true] {
            for write_list in [false, true] {
                for file_bits in 0..8u16 {
                    for directory_bits in 0..8u16 {
                        let mut w = world();
                        w.files.nodes.get_mut("/srv/team").unwrap().mode =
                            0o700 | (directory_bits << 3);
                        w.files.nodes.get_mut("/srv/team/draft.txt").unwrap().mode =
                            0o600 | (file_bits << 3);
                        w.save(format!(
                            "[team]\npath=/srv/team\nread only={}\nread list={}\nwrite list={}\n",
                            if read_only { "yes" } else { "no" },
                            if read_list { "alex" } else { "" },
                            if write_list { "alex" } else { "" }
                        ))
                        .unwrap();
                        w.publish().unwrap();
                        let connection = w.connect("team", &valid("alex"));
                        if directory_bits & 1 == 0 {
                            assert_eq!(connection.unwrap_err().kind, FaultKind::DirectorySearch);
                            continue;
                        }
                        let connection = connection.unwrap();
                        assert_eq!(w.read(&connection, "draft.txt").is_ok(), file_bits & 4 != 0);
                        let expected =
                            (write_list || (!read_only && !read_list)) && file_bits & 2 != 0;
                        let before = w.clone();
                        assert_eq!(
                            w.replace(&connection, "draft.txt", "probe").is_ok(),
                            expected,
                            "ro={read_only} read-list={read_list} write-list={write_list} file={file_bits:o} dir={directory_bits:o}"
                        );
                        if !expected {
                            assert_eq!(w, before);
                        } else {
                            assert_eq!(w.files.files["/srv/team/draft.txt"], "probe");
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn oversized_configuration_and_clock_limits_do_not_partially_modify_the_virtual_file() {
    let mut w = world();
    let before = w.clone();
    assert_eq!(
        w.save("x".repeat(16385)).unwrap_err().kind,
        FaultKind::Limit
    );
    assert_eq!(w, before);
    w.files.clock = 1_000_000;
    let before = w.clone();
    assert_eq!(w.save(config()).unwrap_err().kind, FaultKind::Limit);
    assert_eq!(w, before);
    let mut raw = serde_json::to_value(world()).unwrap();
    raw["configuration_path"] = "x".repeat(513).into();
    assert!(serde_json::from_value::<World>(raw).is_err());
}
