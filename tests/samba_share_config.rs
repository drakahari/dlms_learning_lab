//! Focused supported-configuration checks; no host Samba execution.
use dlms_learning_lab::samba_share_config::{Kind, Principal, parse};
fn text(extra: &str) -> String {
    format!(
        "[global]\nsecurity = user\nserver role = standalone server\nmap to guest = Never\n[team]\npath = /srv/team\n{extra}\n"
    )
}
#[test]
fn share_defaults_and_supported_smb_booleans_are_explicit() {
    let c = parse(&text("")).unwrap();
    let s = &c.shares["team"];
    assert!(s.read_only);
    assert!(!s.guest_ok);
    assert!(s.valid_users.is_empty());
    for yes in ["yes", "YES", "true", "1"] {
        assert!(parse(&text(&format!("guest ok={yes}"))).unwrap().shares["team"].guest_ok);
    }
    for no in ["no", "NO", "false", "0"] {
        assert!(!parse(&text(&format!("read only={no}"))).unwrap().shares["team"].read_only);
    }
}
#[test]
fn settings_and_share_names_ignore_case_without_changing_unix_principals() {
    let c =
        parse("[TEAM]\nPATH=/srv/team\nREAD   ONLY = No\nvalid users = Alex, @Editors\n").unwrap();
    let s = &c.shares["team"];
    assert!(!s.read_only);
    assert_eq!(
        s.valid_users,
        vec![
            Principal::User("Alex".into()),
            Principal::Group("Editors".into())
        ]
    );
}
#[test]
fn writable_aliases_invert_read_only_and_last_setting_wins() {
    for key in ["writeable", "writable"] {
        assert!(!parse(&text(&format!("{key}=yes"))).unwrap().shares["team"].read_only);
        assert!(
            parse(&text(&format!("{key}=yes\nread only=yes")))
                .unwrap()
                .shares["team"]
                .read_only
        );
        assert!(
            !parse(&text(&format!("read only=yes\n{key}=yes")))
                .unwrap()
                .shares["team"]
                .read_only
        );
    }
}
#[test]
fn policy_lists_keep_user_group_distinction_and_unresolved_names_for_access_diagnosis() {
    let c=parse(&text("valid users=alex,@editors unknown\ninvalid users=blocked\nread list=@reviewers\nwrite list=@editors")).unwrap();
    let s = &c.shares["team"];
    assert_eq!(s.valid_users.len(), 3);
    assert_eq!(s.invalid_users, vec![Principal::User("blocked".into())]);
    assert_eq!(s.read_list, vec![Principal::Group("reviewers".into())]);
    assert_eq!(s.write_list, vec![Principal::Group("editors".into())]);
}
#[test]
fn config_parse_is_not_filesystem_availability_or_access_acceptance() {
    // The future virtual trial must diagnose missing/unsearchable roots. A parser
    // must not pretend to perform testparm's service/access validation.
    assert!(parse("[team]\npath=/supplied-but-missing\n").is_ok());
    assert_eq!(parse("[team]\npath=/\n").unwrap().shares["team"].path, "/");
}
#[test]
fn comments_and_crlf_do_not_turn_into_executable_actions() {
    let c = parse(";example\r\n# note\r\n[team]\r\npath=/srv/team\r\nread only=no\r\n").unwrap();
    assert!(!c.shares["team"].read_only);
    assert_eq!(
        parse(&text("include=/personal/file")).unwrap_err().kind,
        Kind::Unsupported
    );
    assert_eq!(
        parse(&text("root preexec=anything")).unwrap_err().kind,
        Kind::Unsupported
    );
}
#[test]
fn unsupported_real_samba_syntax_is_not_silently_approximated() {
    for extra in [
        "vfs objects=acl_xattr",
        "valid users=\"named user\"",
        "valid users=DOMAIN\\alex",
        "valid users=+nisgroup",
        "path=/srv/%U",
        "read only=maybe",
        "write list=@",
        "guest only=yes",
    ] {
        let e = parse(&text(extra)).unwrap_err();
        assert_eq!(e.kind, Kind::Unsupported, "{extra}");
        assert!(e.line > 0);
    }
    assert_eq!(
        parse("[homes]\npath=/srv/team\n").unwrap_err().kind,
        Kind::Unsupported
    );
    assert_eq!(
        parse("[team]\npath=/srv/team\n[TEAM]\nread only=no\n")
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
}
#[test]
fn syntax_diagnostics_and_canonical_virtual_path_exclusions_are_separate() {
    for s in [
        "path=/srv/team\n",
        "[team\npath=/srv/team\n",
        "[team]\npath /srv/team\n",
        "\0",
    ] {
        assert_eq!(parse(s).unwrap_err().kind, Kind::Syntax);
    }
    for path in [
        "srv/team",
        "/srv/../private",
        "/srv/./team",
        "//srv/team",
        "/srv/team/",
    ] {
        assert_eq!(
            parse(&format!("[team]\npath={path}\n")).unwrap_err().kind,
            Kind::Unsupported
        );
    }
}
#[test]
fn resource_limits_are_enforced_without_large_allocation_or_host_io() {
    assert_eq!(
        parse(&"x".repeat(16 * 1024 + 1)).unwrap_err().kind,
        Kind::Limit
    );
    assert_eq!(
        parse(&format!("[team]\npath=/srv/team\n{}", "#\n".repeat(129)))
            .unwrap_err()
            .kind,
        Kind::Limit
    );
    assert_eq!(
        parse(&text(&format!(
            "valid users={}",
            (0..17)
                .map(|n| format!("user{n}"))
                .collect::<Vec<_>>()
                .join(" ")
        )))
        .unwrap_err()
        .kind,
        Kind::Limit
    );
    let shares = (0..9)
        .map(|n| format!("[s{n}]\npath=/srv/team\n"))
        .collect::<String>();
    assert_eq!(parse(&shares).unwrap_err().kind, Kind::Limit);
}

#[test]
fn principal_order_and_duplicate_names_do_not_change_effective_policy() {
    let a = parse(&text(
        "valid users=blair alex @team alex\nwrite list=@writers alex",
    ))
    .unwrap();
    let b = parse(&text(
        "valid users=@team alex blair\nwrite list=alex @writers",
    ))
    .unwrap();
    assert_eq!(a, b);
}
