use dlms_learning_lab::{
    ldap_query::{Dn, Error},
    ldap_utility as u,
};
const AUTH: &str =
    "-x -W -H ldap://directory.example.test -D 'uid=operator,ou=People,dc=example,dc=test'";
fn command(verb: &str, extra: &str) -> String {
    format!("{verb} {AUTH} {extra}")
}
#[test]
fn names_and_add_default_are_not_confused_with_executed_operations() {
    assert!(u::parse(&command("ldapadd", "")).unwrap().add_default());
    assert!(!u::parse(&command("ldapmodify", "")).unwrap().add_default());
    assert!(
        u::parse(&command("ldapmodify", "-a"))
            .unwrap()
            .add_default()
    );
    assert!(u::parse(&command("ldapadd", "-a")).unwrap().add_default());
    for verb in [
        "ldapdelete",
        "ldappasswd",
        "sh",
        "sudo ldapadd",
        "/usr/bin/ldapadd",
    ] {
        assert!(matches!(
            u::parse(&command(verb, "")),
            Err(Error::Unsupported(_))
        ));
    }
}
#[test]
fn option_order_quoting_and_uri_aliases_preserve_validated_components() {
    let a = u::parse(&command("ldapadd", "-f /work/new.ldif")).unwrap();
    let b=u::parse("ldapmodify -f '/work/new.ldif' -D \"UID=Operator,OU=people,DC=example,DC=TEST\" -H ldap://DIRECTORY.EXAMPLE.TEST:389/ -W -a -x --").unwrap();
    assert_eq!(a.add_default(), b.add_default());
    assert_eq!(a.uri(), b.uri());
    assert_eq!(a.uri(), "ldap://directory.example.test");
    assert!(a.bind().equivalent(b.bind()));
    assert_eq!(a.input(), b.input());
}
#[test]
fn editor_input_and_virtual_identifiers_are_data_without_any_file_lookup() {
    assert_eq!(
        u::parse(&command("ldapmodify", "")).unwrap().input(),
        &u::Input::Editor
    );
    assert_eq!(
        u::parse(&command("ldapmodify", "-f /work/team/changes.ldif"))
            .unwrap()
            .input(),
        &u::Input::VirtualFile("/work/team/changes.ldif".into())
    );
    // The identifier need not exist here: parsing performs no resolution or I/O.
    assert!(u::parse(&command("ldapadd", "-f /work/not-created.ldif")).is_ok());
}
#[test]
fn empty_anonymous_or_unimplemented_authentication_is_not_inferred_from_a_name() {
    for raw in [
        "",
        "   ",
        "ldapadd",
        "ldapadd -x",
        "ldapadd -x -W -H ldap://directory.example.test",
        "ldapadd -x -H ldap://directory.example.test -D uid=operator,dc=test",
        "ldapadd -W -H ldap://directory.example.test -D uid=operator,dc=test",
    ] {
        assert!(u::parse(raw).is_err(), "{raw}");
    }
    for extra in [
        "-w a-training-value",
        "-y /work/password",
        "-Y EXTERNAL",
        "-Z",
        "-ZZ",
        "-I",
        "-Q",
    ] {
        assert!(
            matches!(
                u::parse(&command("ldapmodify", extra)),
                Err(Error::Unsupported(_))
            ),
            "{extra}"
        );
    }
}
#[test]
fn duplicate_switches_and_missing_option_values_have_syntax_feedback() {
    for extra in [
        "-x",
        "-W",
        "-H ldap://other.test",
        "-D uid=other,dc=test",
        "-f /work/a.ldif -f /work/b.ldif",
        "-a -a",
        "-H",
        "-D",
        "-f",
    ] {
        assert!(
            matches!(
                u::parse(&command("ldapmodify", extra)),
                Err(Error::Syntax(_))
            ),
            "{extra}"
        );
    }
}
#[test]
fn unknown_combined_and_positional_forms_are_honestly_unsupported() {
    for extra in [
        "-c",
        "-n",
        "-v",
        "-P 3",
        "-e manageDSAit",
        "-j 2",
        "-S /work/rejects.ldif",
        "-xW",
        "--help",
        "-Hldap://directory.example.test",
        "stray",
        "-- stray",
    ] {
        assert!(
            matches!(
                u::parse(&command("ldapmodify", extra)),
                Err(Error::Unsupported(_))
            ),
            "{extra}"
        );
    }
}
#[test]
fn traversal_urls_host_paths_and_unsupported_virtual_components_never_become_resources() {
    for file in [
        "/etc/passwd",
        "../file",
        "/work/../private",
        "/work/a/../../private",
        "/work/./changes.ldif",
        "/work//changes.ldif",
        "/work/",
        "/work",
        "file:///etc/passwd",
        "https://example.test/file",
        "/work/name with spaces",
        "/work/a\\b",
    ] {
        let raw = command("ldapadd", &format!("-f '{file}'"));
        assert!(u::parse(&raw).is_err(), "{file}");
    }
    assert!(matches!(
        u::parse(&command(
            "ldapadd",
            &format!("-f /work/{}", "a".repeat(129))
        )),
        Err(Error::Limit(_))
    ));
}
#[test]
fn shell_features_multiline_control_text_and_unicode_cannot_execute() {
    for extra in [
        "; echo no",
        "| cat",
        "&& true",
        "$(id)",
        "`id`",
        "< /work/input",
        "/work/*",
        "# comment",
        "\necho no",
        "\u{0}",
        "é",
    ] {
        let result = u::parse(&command("ldapmodify", extra));
        assert!(result.is_err(), "{extra:?}");
        if let Err(Error::Unsupported(message)) = result {
            assert!(!message.contains("ldapsearch"));
        }
    }
}
#[test]
fn malformed_and_escaped_names_follow_the_existing_dn_parser() {
    let parsed=u::parse("ldapadd -x -W -H ldap://directory.example.test -D 'cn=Reed\\, Ava,ou=People,dc=example,dc=test'").unwrap();
    assert!(
        parsed
            .bind()
            .equivalent(&Dn::parse(r"cn=reed\2c ava,ou=people,dc=EXAMPLE,dc=test").unwrap())
    );
    for name in [
        "",
        "uid=",
        "../People",
        "uid=Operator+cn=Operator,dc=test",
        "uid=René,dc=test",
    ] {
        assert!(
            u::parse(&format!(
                "ldapadd -x -W -H ldap://directory.example.test -D '{name}'"
            ))
            .is_err(),
            "{name}"
        );
    }
    assert!(matches!(
        u::parse("ldapadd -x -W -H ldap://directory.example.test -D 'unfinished"),
        Err(Error::Syntax(_))
    ));
}
#[test]
fn unsupported_schemes_credentials_and_uri_components_do_not_select_connections() {
    for uri in [
        "ldaps://directory.example.test",
        "ldapi:///",
        "https://directory.example.test",
        "ldap://user:pass@directory.example.test",
        "ldap://directory.example.test/dc=test",
        "ldap://directory.example.test??sub",
        "ldap://[::1]",
    ] {
        let raw = format!("ldapadd -x -W -H '{uri}' -D uid=operator,dc=test");
        assert!(u::parse(&raw).is_err(), "{uri}");
    }
}
#[test]
fn command_word_and_component_bounds_are_independent() {
    assert!(matches!(u::parse(&"a".repeat(4097)), Err(Error::Limit(_))));
    assert!(matches!(
        u::parse(&format!("ldapadd {}", "word ".repeat(33))),
        Err(Error::Limit(_))
    ));
    assert!(u::parse(&command("ldapadd", &format!("-f /work/{}", "a".repeat(65)))).is_err());
    assert!(matches!(
        u::parse(&format!(
            "ldapadd -x -W -H ldap://directory.example.test -D {}",
            "a".repeat(1025)
        )),
        Err(Error::Limit(_))
    ));
}
#[test]
fn truncated_and_arbitrary_bounded_prefixes_never_panic_or_claim_execution() {
    let raw = command("ldapmodify", "-a -f '/work/changes.ldif'");
    for end in 0..=raw.len() {
        let _ = u::parse(&raw[..end]);
    }
    for byte in 0..=127u8 {
        let raw = String::from_utf8(vec![byte; 64]).unwrap();
        let _ = u::parse(&raw);
    }
}

#[test]
fn utility_word_limit_diagnostic_never_advertises_the_search_lexers_larger_limit() {
    for n in [33, 65] {
        let e = u::parse(&format!("ldapadd {}", "word ".repeat(n))).unwrap_err();
        assert!(matches!(e, Error::Limit(_)));
        let message = e.to_string();
        assert!(message.contains("32"), "{message}");
        assert!(!message.contains("64"), "{message}");
    }
}
