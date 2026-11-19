use dlms_learning_lab::{
    ldap_client,
    ldap_protection::client::{Error, MAX_LINE, MAX_LINES, Settings},
};
const CA: &str = "/work/training-ca.pem";
#[test]
fn default_demand_and_explicit_equivalents_preserve_the_virtual_input_identifier() {
    let expected = Settings::parse(&format!("TLS_CACERT {CA}\n")).unwrap();
    assert_eq!(expected.ca_input(), CA);
    for level in ["demand", "hard", "DEMAND", "Hard"] {
        assert_eq!(
            Settings::parse(&format!("tls_reqcert {level}\ntls_cacert {CA}\n")).unwrap(),
            expected
        );
    }
}
#[test]
fn ordinary_whitespace_comments_and_crlf_are_supported_without_shell_or_file_loading() {
    let s = Settings::parse(&format!(
        "# supplied inputs\r\n\r\n\tTLS_CACERT\t{CA} \r\n TLS_REQCERT demand\r\n"
    ))
    .unwrap();
    assert_eq!(s.ca_input(), CA);
    assert_eq!(
        Settings::parse("# comment only\n").unwrap_err(),
        Error::MissingCaInput
    );
}
#[test]
fn duplicate_options_are_an_explicit_subset_limit_with_a_specific_line() {
    for raw in [
        format!("TLS_CACERT {CA}\ntls_cacert /work/other.pem"),
        format!("TLS_CACERT {CA}\nTLS_REQCERT demand\ntls_reqcert hard"),
    ] {
        let e = Settings::parse(&raw).unwrap_err();
        assert!(matches!(e, Error::Duplicate { .. }));
        assert!(e.to_string().contains("subset"));
    }
}
#[test]
fn other_real_client_levels_and_unmodeled_options_are_not_misreported_as_linux_syntax_errors() {
    for level in ["never", "allow", "try", "bogus"] {
        let e = Settings::parse(&format!("TLS_CACERT {CA}\nTLS_REQCERT {level}")).unwrap_err();
        assert!(matches!(e, Error::Unsupported { line: 2, .. }));
        assert!(e.to_string().contains("outside this verified-peer subset"));
    }
    for name in [
        "URI",
        "BASE",
        "TLS_CACERTDIR",
        "TLS_REQSAN",
        "SASL_MECH",
        "LDAPCONF",
        "BINDPW",
        "echo",
    ] {
        assert!(matches!(
            Settings::parse(&format!("TLS_CACERT {CA}\n{name} something")),
            Err(Error::Unsupported { line: 2, .. })
        ));
    }
}
#[test]
fn host_paths_traversal_expansion_multiple_paths_quotes_and_inline_comments_never_select_inputs() {
    for value in [
        "/etc/ssl/ca.pem",
        "../ca.pem",
        "/work/../ca.pem",
        "/work/./ca.pem",
        "/work//ca.pem",
        "https://ca.example.test/roots",
        "$HOME/ca.pem",
        "$(echo wrong)",
        "/work/a.pem /work/b.pem",
        "\"/work/a.pem\"",
        "/work/a.pem # explanation",
        "/work/ünsupported.pem",
    ] {
        assert!(
            matches!(
                Settings::parse(&format!("TLS_CACERT {value}")),
                Err(Error::Unsupported { line: 1, .. })
            ),
            "{value}"
        );
    }
}
#[test]
fn aggregate_line_and_control_bounds_are_enforced_before_any_setting_is_returned() {
    assert_eq!(
        Settings::parse(&"x".repeat(ldap_client::MAX_CONFIG + 1)).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Settings::parse(&"#\n".repeat(MAX_LINES + 1)).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Settings::parse(&"x".repeat(MAX_LINE + 1)).unwrap_err(),
        Error::Limit
    );
    for control in ['\0', '\u{1}', '\u{b}', '\u{1b}', '\r'] {
        let e = Settings::parse(&format!("TLS_CACERT {CA}{control}more")).unwrap_err();
        assert_eq!(e, Error::Control { line: 1 });
    }
}
#[test]
fn missing_values_and_missing_ca_are_distinct_and_retry_uses_the_corrected_text() {
    for raw in ["TLS_CACERT", "TLS_CACERT   ", "TLS_REQCERT\t"] {
        assert_eq!(
            Settings::parse(raw).unwrap_err(),
            Error::MissingValue { line: 1 }
        );
    }
    assert_eq!(
        Settings::parse("TLS_REQCERT demand").unwrap_err(),
        Error::MissingCaInput
    );
    assert!(Settings::parse(&format!("TLS_REQCERT demand\nTLS_CACERT {CA}")).is_ok());
}
#[test]
fn identifiers_have_independent_byte_component_and_case_boundaries() {
    let component = "A".repeat(64);
    let path = format!("/work/{component}");
    assert_eq!(
        Settings::parse(&format!("TLS_CACERT {path}"))
            .unwrap()
            .ca_input(),
        path
    );
    assert!(Settings::parse(&format!("TLS_CACERT /work/{}", "a".repeat(65))).is_err());
    let s = Settings::parse("TLS_CACERT /work/Root.pem").unwrap();
    assert_eq!(s.ca_input(), "/work/Root.pem");
    assert_ne!(s.ca_input(), "/work/root.pem");
}
#[test]
fn unknown_supplied_input_is_only_a_virtual_identifier_not_implicit_host_trust() {
    let s = Settings::parse("TLS_CACERT /work/not-supplied.pem").unwrap();
    assert_eq!(s.ca_input(), "/work/not-supplied.pem");
    // Resolution to source-held symbolic CA facts is a separate workspace step.
    // The parser never substitutes system trust or silently chooses another input.
}
