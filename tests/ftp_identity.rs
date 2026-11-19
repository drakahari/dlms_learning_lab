//! Exercise the actual production FTP capability and existing virtual file model.
mod filesystem {
    pub use dlms_learning_lab::filesystem::Access;
}
use dlms_learning_lab::{ftp_identity, ftp_identity_config};
use ftp_identity::{Credential, Kind, World};
use ftp_identity_config as config;
#[path = "support/ftp_identity_fixture.rs"]
mod fixture;
use fixture::{settings, world};
fn login(w: &World, u: &str) -> ftp_identity::Session {
    w.login(&Credential::Valid { user: u.into() }).unwrap()
}
#[test]
fn parser_defaults_and_real_boolean_whitespace_duplicate_and_octal_semantics() {
    let d = config::parse("").unwrap();
    assert!(
        !d.local_enable
            && d.anonymous_enable
            && !d.write_enable
            && !d.chroot_local_user
            && !d.chroot_list_enable
            && !d.userlist_enable
            && d.userlist_deny
            && d.download_enable
            && d.dirlist_enable
    );
    assert_eq!(d.local_umask, 0o077);
    let c = config::parse(
        "# comment\n\tlocal_enable = true\r\nlocal_enable=0\nlocal_enable=YeS\nlocal_umask=022\n",
    )
    .unwrap();
    assert!(c.local_enable);
    assert_eq!(c.local_umask, 0o022);
    assert_eq!(config::parse("local_umask=22").unwrap().local_umask, 22);
    assert_eq!(
        config::parse("local_umask=0777").unwrap().local_umask,
        0o777
    );
}
#[test]
fn unsupported_and_malformed_configuration_cannot_silently_succeed() {
    for s in [
        "ssl_enable=YES",
        "local_root=/srv/alex",
        "allow_writeable_chroot=YES",
        "userlist_file=$HOME/list",
        "userlist_file=/etc/../list",
    ] {
        assert_eq!(
            config::parse(s).unwrap_err().kind,
            config::Kind::Unsupported,
            "{s}"
        );
    }
    for s in [
        "local_enable",
        "local_enable=maybe",
        "local_enable=YES # comment",
        " # indented comment",
        "local_umask=08",
    ] {
        assert!(config::parse(s).is_err(), "{s}");
    }
    assert_eq!(
        config::parse(&"#".repeat(config::MAX_CONFIG + 1))
            .unwrap_err()
            .kind,
        config::Kind::Limit
    );
    assert_eq!(
        config::parse(&"\n".repeat(config::MAX_LINES + 1))
            .unwrap_err()
            .kind,
        config::Kind::Limit
    );
    assert!(config::parse("local_enable=YES\0").is_err());
}
#[test]
fn outside_search_is_required_at_login_but_not_after_entering_the_jail() {
    let mut w = world();
    w.validate().unwrap();
    let mut s = login(&w, "alex");
    w.files.nodes.get_mut("/srv").unwrap().mode = 0o700;
    assert!(!w
        .files
        .access("alex", "/srv/alex/readme", filesystem::Access::ReadFile));
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Permission
    );
    assert!(s.confined());
    assert_eq!(s.user(), "alex");
    assert_eq!(s.pwd(), "/");
    assert_eq!(w.get(&s, "/readme").unwrap(), "hello Alex\n");
    w.cd(&mut s, "inbox").unwrap();
    assert_eq!(s.pwd(), "/inbox");
    w.cd(&mut s, "../../../../").unwrap();
    assert_eq!(s.pwd(), "/");
    assert_eq!(w.get(&s, "/etc/secret").unwrap_err().kind, Kind::Missing);
    assert_eq!(
        w.get(&s, "../../srv/blair/readme").unwrap_err().kind,
        Kind::Missing
    );
}
#[test]
fn component_walk_does_not_lexically_skip_missing_or_unsearchable_directories() {
    let mut w = world();
    let s = login(&w, "alex");
    w.files.nodes.get_mut("/srv/alex/inbox").unwrap().mode = 0o600;
    assert_eq!(
        w.get(&s, "inbox/../readme").unwrap_err().kind,
        Kind::Permission
    );
    assert_eq!(
        w.get(&s, "missing/../readme").unwrap_err().kind,
        Kind::Missing
    );
    assert_eq!(
        w.get(&s, "readme/../readme").unwrap_err().kind,
        Kind::Permission
    );
    assert!(w.get(&s, "readme/").is_err());
}
#[test]
fn all_four_chroot_list_meanings_and_exact_line_matching() {
    for (all, listed, expected) in [
        (false, false, false),
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        let mut w = world();
        w.files.nodes.get_mut("/srv").unwrap().mode = 0o755;
        w.saved = format!(
            "local_enable=YES\nchroot_local_user={}\nchroot_list_enable=YES\n",
            if all { "YES" } else { "NO" }
        );
        w.files.files.insert(
            "/etc/vsftpd.chroot_list".into(),
            if listed {
                "alex\n"
            } else {
                "alexander\n alex\nalex \nalex\r\n"
            }
            .into(),
        );
        w.restart().unwrap();
        let s = login(&w, "alex");
        assert_eq!(s.confined(), expected);
        assert_eq!(
            w.get(
                &s,
                if expected {
                    "/readme"
                } else {
                    "/srv/alex/readme"
                }
            )
            .unwrap(),
            "hello Alex\n"
        );
        if !expected {
            assert_eq!(w.get(&s, "/etc/secret").unwrap(), "outside\n");
        }
    }
}
#[test]
fn authentication_lists_and_unix_permissions_are_distinct_failures() {
    let mut w = world();
    assert_eq!(
        w.login(&Credential::Invalid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Authentication
    );
    assert_eq!(
        w.login(&Credential::Valid {
            user: "unknown".into()
        })
        .unwrap_err()
        .kind,
        Kind::Authentication
    );
    assert_eq!(
        w.login(&Credential::Anonymous).unwrap_err().kind,
        Kind::Authentication
    );
    w.saved.push_str("userlist_enable=YES\nuserlist_deny=NO\n");
    w.restart().unwrap();
    assert!(w
        .login(&Credential::Valid {
            user: "alex".into()
        })
        .is_ok());
    assert_eq!(
        w.login(&Credential::Valid {
            user: "blair".into()
        })
        .unwrap_err()
        .kind,
        Kind::Authentication
    );
    w.saved.push_str("userlist_deny=YES\n");
    w.restart().unwrap();
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Authentication
    );
    assert!(w
        .login(&Credential::Valid {
            user: "blair".into()
        })
        .is_ok());
    w.accounts[1].enabled = false;
    assert_eq!(
        w.login(&Credential::Valid {
            user: "blair".into()
        })
        .unwrap_err()
        .kind,
        Kind::Authentication
    );
}
#[test]
fn missing_account_lists_fail_instead_of_disabling_confinement() {
    let mut w = world();
    w.saved
        .push_str("chroot_list_enable=YES\nchroot_list_file=/etc/absent\n");
    w.restart().unwrap();
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Policy
    );
    w.saved = settings() + "userlist_enable=YES\nuserlist_file=/etc/absent\n";
    w.restart().unwrap();
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Policy
    );
}
#[test]
fn writable_jail_root_is_rejected_but_writable_child_is_useful() {
    let mut w = world();
    w.files.nodes.get_mut("/srv/alex").unwrap().mode = 0o777;
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Policy
    );
    w.files.nodes.get_mut("/srv/alex").unwrap().mode = 0o755;
    let s = login(&w, "alex");
    w.put(&s, "inbox/new", "a payload\n").unwrap();
    assert_eq!(w.get(&s, "inbox/new").unwrap(), "a payload\n");
    assert_eq!(w.files.nodes["/srv/alex/inbox/new"].mode, 0o644);
    assert_eq!(w.files.nodes["/srv/alex/inbox/new"].owner, "alex");
    assert!(w.files.valid());
}
#[test]
fn actual_content_alternatives_and_negative_operations_are_atomic() {
    for text in ["one\n", "a different deterministic input\n", ""] {
        let mut w = world();
        let s = login(&w, "alex");
        let before = w.clone();
        assert_eq!(w.put(&s, "/new", text).unwrap_err().kind, Kind::Permission);
        assert_eq!(w, before);
        assert_eq!(
            w.put(&s, "/private", text).unwrap_err().kind,
            Kind::Permission
        );
        assert_eq!(w, before);
        w.put(&s, "/inbox/own", text).unwrap();
        assert_eq!(w.get(&s, "inbox/./own").unwrap(), text);
        assert_eq!(w.files.files["/etc/secret"], "outside\n");
        assert_eq!(w.files.files["/srv/blair/readme"], "hello Blair\n");
    }
}
#[test]
fn saved_changes_wait_for_restart_and_failed_restart_invalidates_old_connections() {
    let mut w = world();
    let s = login(&w, "alex");
    w.save("unsupported=YES".into()).unwrap();
    assert!(w.get(&s, "readme").is_ok());
    assert_eq!(w.restart().unwrap_err().kind, Kind::Unsupported);
    assert!(w.published.is_none());
    assert_eq!(w.get(&s, "readme").unwrap_err().kind, Kind::Stale);
    assert_eq!(
        w.login(&Credential::Valid {
            user: "alex".into()
        })
        .unwrap_err()
        .kind,
        Kind::Service
    );
    w.save(settings()).unwrap();
    w.restart().unwrap();
    let fresh = login(&w, "alex");
    assert!(w.get(&fresh, "readme").is_ok());
    assert_eq!(w.get(&s, "readme").unwrap_err().kind, Kind::Stale);
    w.stop().unwrap();
    assert_eq!(w.get(&fresh, "readme").unwrap_err().kind, Kind::Stale);
}
#[test]
fn write_download_and_directory_gates_do_not_replace_file_permissions() {
    let mut w = world();
    w.saved = settings() + "write_enable=NO\ndownload_enable=NO\ndirlist_enable=NO\n";
    w.restart().unwrap();
    let s = login(&w, "alex");
    assert_eq!(w.put(&s, "inbox/a", "x").unwrap_err().kind, Kind::Policy);
    assert_eq!(w.get(&s, "readme").unwrap_err().kind, Kind::Policy);
    assert_eq!(w.list(&s, "/").unwrap_err().kind, Kind::Policy);
    w.save(settings()).unwrap();
    w.restart().unwrap();
    let s = login(&w, "alex");
    assert_eq!(w.get(&s, "private").unwrap_err().kind, Kind::Permission);
    assert_eq!(w.list(&s, "/").unwrap(), vec!["inbox", "private", "readme"]);
}
#[test]
fn anonymous_is_read_only_confined_and_needs_world_readable_files() {
    let mut w = world();
    w.saved = settings() + "anonymous_enable=YES\n";
    w.restart().unwrap();
    let s = w.login(&Credential::Anonymous).unwrap();
    assert_eq!(s.user(), "ftp");
    assert_eq!(w.get(&s, "readme").unwrap(), "public\n");
    assert_eq!(
        w.get(&s, "not-world-readable").unwrap_err().kind,
        Kind::Permission
    );
    assert_eq!(w.put(&s, "new", "x").unwrap_err().kind, Kind::Policy);
    assert!(w.get(&s, "../etc/secret").is_err());
}
#[test]
fn malformed_world_and_bounded_paths_transfers_and_data_fail_honestly() {
    let mut w = world();
    let s = login(&w, "alex");
    let before = w.clone();
    for path in [
        "",
        "$(id)",
        "file;id",
        "~/.ssh/key",
        "a\0b",
        &"a".repeat(513),
    ] {
        assert_eq!(w.get(&s, path).unwrap_err().kind, Kind::Unsupported);
    }
    assert_eq!(
        w.put(&s, "inbox/new", &"x".repeat(4097)).unwrap_err().kind,
        Kind::Limit
    );
    assert_eq!(w, before);
    assert_eq!(
        w.save("x".repeat(config::MAX_CONFIG + 1)).unwrap_err().kind,
        Kind::Limit
    );
    assert_eq!(w, before);
    assert_eq!(
        w.get(&s, &("./".repeat(65) + "readme")).unwrap_err().kind,
        Kind::Limit
    );
    w.version = 2;
    assert_eq!(w.validate().unwrap_err().kind, Kind::Invalid);
    w.version = 1;
    w.accounts[0].home = "/srv/alex/../blair".into();
    assert!(w.validate().is_err());
}
#[test]
fn saved_world_roundtrip_preserves_files_and_never_serializes_privileged_sessions() {
    let mut w = world();
    let s = login(&w, "alex");
    w.put(&s, "inbox/report", "report\n").unwrap();
    w.save("local_enable=MAYBE".into()).unwrap();
    let restored: World = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(restored, w);
    assert!(restored.validate().is_ok());
    assert_eq!(restored.get(&s, "inbox/report").unwrap(), "report\n");
}
