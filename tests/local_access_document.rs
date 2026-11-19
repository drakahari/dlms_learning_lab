use dlms_learning_lab::{
    acl_editor::Category, local_access as local, local_access_document as doc,
    management_policy as policy,
};
#[path = "support/management_fixture.rs"]
mod fixture;
const AAA: &str = "aaa new-model\naaa authentication login default local-case\naaa authorization exec default local\n";
const USERS: &str = "username Admin privilege 15 secret Practice-Admin-42\nusername Operator privilege 1 secret Practice-Operator-93\n";
const INCOMING: &str = "ip ssh version 2\nline vty 0\n transport input ssh\n access-class 10 in\nexit\nip access-list standard 10\n permit host 192.0.2.10\nexit\n";
fn reference() -> String {
    format!("{AAA}{USERS}{INCOMING}")
}
fn parse(raw: &str) -> Result<doc::Settings, dlms_learning_lab::acl_editor::Diagnostic> {
    doc::parse(&fixture::network(), &fixture::settings(), raw)
}
#[test]
fn complete_documents_produce_real_named_record_authentication_behind_actual_ip_policy() {
    let p = parse(&reference()).unwrap();
    assert_eq!(p.local.users.len(), 2);
    assert_eq!(p.local.users[0].privilege.level(), 15);
    for (u, token, level) in [
        (
            "Admin",
            "Practice-Admin-42",
            local::Privilege::Administrator,
        ),
        ("Operator", "Practice-Operator-93", local::Privilege::User),
    ] {
        let probe = policy::Attempt {
            line: 0,
            source: fixture::ip("192.0.2.10"),
            protocol: policy::Protocol::Ssh2,
        };
        let r = local::inspect(
            &fixture::network(),
            &p.incoming,
            &p.local,
            probe,
            "admin",
            u,
            token,
        )
        .unwrap();
        assert!(
            matches!(r,local::Observation::Local{decision:local::Decision::Authenticated{privilege,..},..}if privilege==level)
        );
    }
    assert_eq!(
        p.local.authenticate("Admin", "wrong").unwrap(),
        local::Decision::Rejected(local::Rejection::WrongToken)
    );
}
#[test]
fn legitimate_order_case_spacing_and_explicit_default_selection_preserve_behavior() {
    let a = parse(&reference()).unwrap();
    let raw = format!(
        "! Explicit complete configuration\r\n{}{}{}",
        USERS.replace("username", "USERNAME"),
        AAA.replace("aaa", "AAA"),
        INCOMING.replace(
            " transport input ssh",
            " login authentication default\n transport input ssh"
        )
    );
    let b = parse(&raw).unwrap();
    for u in &a.local.users {
        assert_eq!(
            a.local.authenticate(&u.name, &u.training_token),
            b.local.authenticate(&u.name, &u.training_token)
        );
    }
    for protocol in [
        policy::Protocol::Ssh2,
        policy::Protocol::Ssh1,
        policy::Protocol::Telnet,
    ] {
        let n = fixture::network();
        assert_eq!(
            n.inspect(&a.incoming, "admin", 0, protocol)
                .unwrap()
                .allowed_to_authenticate(),
            n.inspect(&b.incoming, "admin", 0, protocol)
                .unwrap()
                .allowed_to_authenticate()
        );
    }
}
#[test]
fn incomplete_supported_settings_are_syntax_diagnostics_at_the_actual_line() {
    for bad in [
        "aaa",
        "aaa authentication login default",
        "aaa authorization exec default",
        "username Admin privilege 15 secret",
    ] {
        let text = format!("! Read the setting\n{bad}\n{}", reference());
        let e = parse(&text).unwrap_err();
        assert_eq!(e.category, Category::Syntax);
        assert_eq!(e.line, Some(2));
    }
    let r = reference().replace(
        " transport input ssh",
        " login authentication\n transport input ssh",
    );
    let e = parse(&r).unwrap_err();
    assert_eq!(e.category, Category::Syntax);
    assert_eq!(
        e.line,
        r.lines()
            .position(|s| s.contains("login authentication"))
            .map(|i| i + 1)
    );
}
#[test]
fn missing_global_methods_never_acquire_inferred_device_defaults() {
    for remove in [
        "aaa new-model\n",
        "aaa authentication login default local-case\n",
        "aaa authorization exec default local\n",
    ] {
        let r = reference().replace(remove, "");
        let e = parse(&r).unwrap_err();
        assert_eq!(e.category, Category::Syntax);
        assert_eq!(e.line, None);
        assert!(e.message.contains("explicitly"));
    }
}
#[test]
fn remote_fallback_hashes_other_privileges_and_bypass_syntax_are_explicitly_unsupported() {
    for command in [
        "aaa authentication login default local",
        "aaa authentication login default none",
        "aaa authentication login default group radius local-case",
        "aaa authentication login named local-case",
        "username Admin privilege 14 secret Training",
        "username Admin privilege 15 secret 9 EncodedValue",
        "username Admin algorithm-type scrypt secret Training",
        "username Admin privilege 15 nopassword",
        "username Admin privilege 15 password Training",
        "username Admin privilege 15 secret \"Two Words\"",
        "username Admin privilege 15 secret $encoded",
    ] {
        let e = parse(&format!("{command}\n{}", reference())).unwrap_err();
        assert_eq!(e.category, Category::Unsupported, "{command}: {e}");
    }
    for bad in ["login local", "login authentication named", "no login"] {
        let raw = reference().replace(
            " transport input ssh",
            &format!(" {bad}\n transport input ssh"),
        );
        assert_eq!(parse(&raw).unwrap_err().category, Category::Unsupported);
    }
}
#[test]
fn filtered_aaa_and_user_rows_preserve_original_acl_diagnostic_locations() {
    let text = reference().replace("permit host 192.0.2.10", "permit host invalid-address");
    let line = text
        .lines()
        .position(|s| s.contains("invalid-address"))
        .unwrap()
        + 1;
    let e = parse(&text).unwrap_err();
    assert_eq!(e.line, Some(line));
    assert!(e.message.to_lowercase().contains("address"));
}
#[test]
fn duplicate_settings_are_not_incremental_overrides_but_case_distinct_names_remain_distinct() {
    let raw = format!("{AAA}{}", reference());
    assert_eq!(parse(&raw).unwrap_err().category, Category::Unsupported);
    let raw = format!("{USERS}{}", reference());
    assert_eq!(parse(&raw).unwrap_err().category, Category::Unsupported);
    let raw = format!(
        "username admin privilege 1 secret Another-Training\n{}",
        reference()
    );
    let p = parse(&raw).unwrap();
    assert_eq!(p.local.users.len(), 3);
    assert_eq!(
        p.local.authenticate("admin", "Practice-Admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::WrongToken)
    );
    assert!(matches!(
        p.local.authenticate("admin", "Another-Training").unwrap(),
        local::Decision::Authenticated {
            privilege: local::Privilege::User,
            ..
        }
    ));
}
#[test]
fn complete_document_and_record_bounds_fail_honestly_before_any_success() {
    for raw in [
        "X".repeat(dlms_learning_lab::acl_editor::MAX_TEXT + 1),
        format!("!{}\n{}", "X".repeat(512), reference()),
        format!("{}{}", "\n".repeat(1024), reference()),
        format!(
            "{}\n{}",
            std::iter::repeat_n("word", 17)
                .collect::<Vec<_>>()
                .join(" "),
            reference()
        ),
    ] {
        assert_eq!(parse(&raw).unwrap_err().category, Category::Limit);
    }
    let users = (0..17)
        .map(|i| format!("username Person{i} privilege 1 secret Training\n"))
        .collect::<String>();
    assert_eq!(
        parse(&format!("{AAA}{users}{INCOMING}"))
            .unwrap_err()
            .category,
        Category::Limit
    );
    let raw = reference().replace("Practice-Admin-42", "Bad\0Token");
    assert_eq!(parse(&raw).unwrap_err().category, Category::Unsupported);
}
#[test]
fn parsing_never_changes_supplied_identity_routes_or_applied_context_on_error() {
    let n = fixture::network();
    let supplied = fixture::settings();
    let before_n = n.clone();
    let before_p = supplied.clone();
    doc::parse(&n, &supplied, &reference()).unwrap();
    assert_eq!(n, before_n);
    assert_eq!(supplied, before_p);
    for bad in [
        "hostname Different",
        "username ../outside privilege 15 secret Training",
        "username Admin privilege 15 secret Training; reboot",
    ] {
        assert!(doc::parse(&n, &supplied, &format!("{bad}\n{}", reference())).is_err());
        assert_eq!(n, before_n);
        assert_eq!(supplied, before_p);
    }
}
#[test]
fn user_and_aaa_settings_inside_a_line_or_list_are_not_promoted_to_global_authority() {
    for bad in [
        "username Extra privilege 15 secret Training",
        "aaa authentication login default local-case",
    ] {
        for marker in [" transport input ssh", " permit host 192.0.2.10"] {
            let raw = reference().replace(marker, &format!(" {bad}\n{marker}"));
            assert!(
                parse(&raw).is_err(),
                "No hidden global record can be created inside a different section: {raw}"
            );
        }
    }
}
