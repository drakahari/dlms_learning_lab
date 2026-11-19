use dlms_learning_lab::mailbox_config::{Config, Kind, MAX_TEXT, Protocol, Ssl, virtual_path};
const CONFIG: &str = "dovecot_config_version = 2.4.0\ndovecot_storage_version = 2.4.0\nprotocols {\n imap = yes\n pop3 = yes\n}\nssl = required\nauth_allow_cleartext = no\nauth_mechanisms = plain login\nssl_server_cert_file = /etc/dovecot/mail.pem\nssl_server_key_file = /etc/dovecot/mail.key\nmail_driver = maildir\nmail_path = /srv/mail/%{user}/Maildir\n";
#[test]
fn actual_typed_configuration_normalizes_without_losing_protocol_or_listener_state() {
    let c = Config::parse(CONFIG).unwrap();
    assert_eq!(c.ssl, Ssl::Required);
    assert_eq!(c.protocols, [Protocol::Imap, Protocol::Pop3].into());
    assert!(!c.auth_allow_cleartext);
    assert_eq!(c.active_listeners().count(), 4);
    assert_eq!(Config::parse(&c.normalized()).unwrap(), c);
}
#[test]
fn alternate_boolean_spellings_lists_comments_and_quotes_have_documented_semantics() {
    let alternate = CONFIG
        .replace("imap = yes", "imap = Y")
        .replace("pop3 = yes", "pop3 = 1")
        .replace(
            "auth_allow_cleartext = no",
            "auth_allow_cleartext = No # remote safety",
        )
        .replace("ssl = required", "ssl = \"required\"");
    assert_eq!(
        Config::parse(&alternate).unwrap(),
        Config::parse(CONFIG).unwrap()
    );
    for value in ["imap pop3", "imap,pop3", "\"imap\" \"pop3\""] {
        let s = CONFIG.replace(
            "protocols {\n imap = yes\n pop3 = yes\n}",
            &format!("protocols = {value}"),
        );
        assert_eq!(Config::parse(&s).unwrap(), Config::parse(CONFIG).unwrap());
    }
    let s = CONFIG.replace(
        "protocols {\n imap = yes\n pop3 = yes\n}",
        "protocols = imap pop3\nprotocols {\n imap = no\n}",
    );
    assert_eq!(
        Config::parse(&s).unwrap().protocols,
        [Protocol::Pop3].into()
    );
    for value in ["false", "0", "true"] {
        let s = CONFIG.replace(
            "auth_allow_cleartext = no",
            &format!("auth_allow_cleartext = {value}"),
        );
        assert_eq!(Config::parse(&s).unwrap_err().kind, Kind::Invalid);
    }
}
#[test]
fn boolean_list_replacement_and_updates_do_not_silently_merge_or_split_quoted_names() {
    let mut s = CONFIG.to_owned();
    s.push_str("protocols = imap\n");
    assert_eq!(
        Config::parse(&s).unwrap().protocols,
        [Protocol::Imap].into()
    );
    s.push_str("protocols {\n pop3 = yes\n}\n");
    assert_eq!(Config::parse(&s).unwrap().protocols.len(), 2);
    let s = CONFIG.replace(
        "protocols {\n imap = yes\n pop3 = yes\n}",
        "protocols = \"imap pop3\"",
    );
    assert_eq!(Config::parse(&s).unwrap_err().kind, Kind::Unsupported);
    let s = CONFIG.replace("protocols {\n imap = yes\n pop3 = yes\n}", "protocols =");
    assert!(Config::parse(&s).unwrap().protocols.is_empty());
}
#[test]
fn port_zero_protocol_disable_and_global_ssl_derive_the_actual_listener_set() {
    let base = Config::parse(CONFIG).unwrap();
    let s = format!(
        "{CONFIG}service imap-login {{\n inet_listener imap {{\n  port = 0\n }}\n inet_listener imaps {{\n  inet_listener_port = 1993\n  inet_listener_ssl = yes\n }}\n}}\n"
    );
    let c = Config::parse(&s).unwrap();
    assert_eq!(c.active_listeners().count(), 3);
    assert_eq!(c.listeners["imaps"].port, 1993);
    assert_eq!(base.listeners["imap"].port, 143);
    assert_eq!(Config::parse(&c.normalized()).unwrap(), c);
    let c = Config::parse(&CONFIG.replace("ssl = required", "ssl = no")).unwrap();
    assert_eq!(
        c.active_listeners().map(|(n, _)| n).collect::<Vec<_>>(),
        ["imap", "pop3"]
    );
}
#[test]
fn remote_password_policy_and_explicit_secured_connection_exception_are_not_confused() {
    let required = Config::parse(CONFIG).unwrap();
    assert!(!required.permits_password(false, false));
    assert!(required.permits_password(true, false));
    assert!(required.permits_password(false, true));
    let yes = Config::parse(&CONFIG.replace("ssl = required", "ssl = yes")).unwrap();
    assert!(!yes.permits_password(false, false));
    assert!(yes.permits_password(true, false));
    let allow = Config::parse(
        &CONFIG
            .replace("ssl = required", "ssl = yes")
            .replace("auth_allow_cleartext = no", "auth_allow_cleartext = yes"),
    )
    .unwrap();
    assert!(allow.permits_password(false, false));
    let still_required =
        Config::parse(&CONFIG.replace("auth_allow_cleartext = no", "auth_allow_cleartext = yes"))
            .unwrap();
    assert!(!still_required.permits_password(false, false));
}
#[test]
fn syntax_errors_unsupported_features_and_limits_have_different_honest_diagnostics() {
    for changed in [
        CONFIG.replace("protocols {\n", "protocols { imap = yes }\n"),
        CONFIG.replace("ssl = required", "ssl = \"required"),
        CONFIG.to_owned() + "}\n",
    ] {
        assert_eq!(Config::parse(&changed).unwrap_err().kind, Kind::Syntax);
    }
    for suffix in [
        "!include /etc/shadow\n",
        "passdb sql {\n}\n",
        "service auth {\n}\n",
        "ssl_cert = </private/key\n",
        "unknown_option = yes\n",
    ] {
        let e = Config::parse(&(CONFIG.to_owned() + suffix)).unwrap_err();
        assert_eq!(e.kind, Kind::Unsupported, "{e}");
        assert!(e.line.is_some());
    }
    assert_eq!(
        Config::parse(&"x".repeat(MAX_TEXT + 1)).unwrap_err().kind,
        Kind::Limit
    );
    assert_eq!(
        Config::parse(&(CONFIG.to_owned() + &"\n".repeat(161)))
            .unwrap_err()
            .kind,
        Kind::Limit
    );
    assert_eq!(
        Config::parse(&CONFIG.replace("2.4.0", "2.3"))
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
    assert!(
        Config::parse(&CONFIG.replace("dovecot_config_version = 2.4.0\n", ""))
            .unwrap_err()
            .message
            .contains("first setting")
    );
}
#[test]
fn virtual_paths_never_expand_arbitrary_expressions_or_escape_the_supplied_namespace() {
    for bad in [
        "/etc/../shadow",
        "/../../private",
        "~/../secret",
        "/srv/%{env:HOME}",
        "/srv/$(touch)",
        "/srv//mail",
        "~//Maildir",
        "relative/path",
        "/srv/mail\0",
    ] {
        assert!(!virtual_path(bad, true), "{bad}");
    }
    for good in [
        "/etc/dovecot/mail.pem",
        "/srv/mail/%{user}/Maildir",
        "~/Maildir",
    ] {
        assert!(virtual_path(good, true), "{good}");
    }
    let literal = CONFIG.replace("/etc/dovecot/mail.pem", "/definitely-not-opened/server.pem");
    assert!(Config::parse(&literal).is_ok());
    assert_eq!(
        Config::parse(&CONFIG.replace("/etc/dovecot/mail.pem", "inline:PRIVATE-KEY"))
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
}
#[test]
fn named_filters_require_linebreaks_and_mismatched_listener_families_do_not_apply() {
    let bad =
        format!("{CONFIG}service imap-login {{\n inet_listener pop3 {{\n port = 110\n }}\n}}\n");
    assert_eq!(Config::parse(&bad).unwrap_err().kind, Kind::Unsupported);
    let bad =
        format!("{CONFIG}service imap-login {{\n inet_listener imap {{\n port = 65536\n }}\n}}\n");
    assert_eq!(Config::parse(&bad).unwrap_err().kind, Kind::Invalid);
    let bad = format!(
        "{CONFIG}service imap-login {{\n inet_listener imap {{\n port = $HOSTPORT\n }}\n}}\n"
    );
    assert_eq!(Config::parse(&bad).unwrap_err().kind, Kind::Invalid);
    let unknown =
        format!("{CONFIG}service imap-login {{\n inet_listener custom {{\n port = 9999\n }}\n}}\n");
    assert_eq!(Config::parse(&unknown).unwrap_err().kind, Kind::Unsupported);
}

#[test]
fn blank_input_explains_how_to_begin_instead_of_reporting_an_unrelated_missing_protocol() {
    let error = Config::parse("  \n# no settings yet\n").unwrap_err();
    assert_eq!(error.kind, Kind::Syntax);
    assert!(error.message.contains("blank"));
    assert!(error.message.contains("dovecot_config_version"));
}
