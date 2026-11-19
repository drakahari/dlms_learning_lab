use dlms_learning_lab::{ldap_client as c, ldap_query as q};
#[path = "support/ldap_directory.rs"]
mod fixture;
fn settings() -> c::Settings {
    c::Settings {
        version: 1,
        uri: "ldap://directory.example.test".into(),
        base: "ou=People,dc=example,dc=test".into(),
    }
}
fn env() -> c::Environment {
    c::Environment::new(&[(settings().uri, fixture::fixture())]).unwrap()
}
fn run(command: &str) -> c::Outcome {
    env().run(&settings(), command).unwrap()
}
fn count(command: &str) -> usize {
    let c::Outcome::Entries(e) = run(command) else {
        panic!("Expected an actual search result");
    };
    e.len()
}
#[test]
fn ldap_conf_options_are_case_insensitive_values_are_unquoted_and_defaults_are_real() {
    let s=c::Settings::parse("# supplied client\r\nuri\tldap://DIRECTORY.example.test:389/\r\n  BaSe ou=People,dc=example,dc=test  \r\n").unwrap();
    assert_eq!(c::uri_key(&s.uri).unwrap(), settings().uri);
    assert_eq!(s.base, settings().base);
    let e = env();
    assert_eq!(
        e.run(&s, "ldapsearch -x -LLL '(uid=alex)' uid").unwrap(),
        run("ldapsearch -x -LLL '(uid=alex)' uid")
    );
}
#[test]
fn search_uses_configured_uri_base_and_actual_supplied_entries() {
    assert_eq!(count("ldapsearch -x -LLL '(objectClass=person)'"), 3);
    let mut s = settings();
    s.base = "ou=Teams,dc=example,dc=test".into();
    assert_eq!(
        env()
            .run(&s, "ldapsearch -x -LLL '(objectClass=person)'")
            .unwrap(),
        c::Outcome::Entries(vec![])
    );
    s.base = "ou=Absent,dc=example,dc=test".into();
    assert_eq!(
        env().run(&s, "ldapsearch -x -LLL '(uid=alex)'").unwrap(),
        c::Outcome::NoSuchObject
    );
    s.uri = "ldap://offline.example.test".into();
    assert_eq!(
        env().run(&s, "ldapsearch -x -LLL '(uid=alex)'").unwrap(),
        c::Outcome::Unavailable { uri: s.uri }
    );
}
#[test]
fn explicit_options_override_defaults_and_scope_changes_the_actual_result() {
    assert_eq!(
        count("ldapsearch -LLL -x -b 'dc=example,dc=test' -s one '(objectClass=*)' 1.1"),
        2
    );
    assert_eq!(
        count("ldapsearch -x -LLL -s one '(objectClass=person)' uid"),
        2
    );
    assert_eq!(count("ldapsearch -x -LLL -s base '(objectClass=*)' ou"), 1);
    let s = settings();
    let a=c::Query::parse(&s,"ldapsearch -s sub -H ldap://DIRECTORY.example.test:389/ -b 'ou=People,dc=example,dc=test' -LLL -x '(uid=alex)' uid").unwrap();
    assert_eq!(
        env().search(&a).unwrap(),
        run("ldapsearch -x -LLL '(uid=alex)' uid")
    );
}
#[test]
fn named_attributes_star_and_dn_only_are_distinct_from_filtering() {
    let actual = run("ldapsearch -x -LLL '(uid=alex)' UID mail uid unknownAttribute");
    let c::Outcome::Entries(e) = &actual else {
        panic!()
    };
    assert_eq!(e.len(), 1);
    assert_eq!(
        e[0].attributes
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>(),
        ["uid", "mail"]
    );
    assert_eq!(e[0].attributes[1].values.len(), 2);
    let text = actual.output().unwrap();
    assert!(text.starts_with("dn: uid=alex,ou=People,dc=example,dc=test\nuid: alex\n"));
    assert!(!text.contains("cn:"));
    let dn = run("ldapsearch -x -LLL '(uid=alex)' 1.1").output().unwrap();
    assert_eq!(dn, "dn: uid=alex,ou=People,dc=example,dc=test\n\n");
    let star = run("ldapsearch -x -LLL '(uid=alex)' '*'");
    assert_eq!(star, run("ldapsearch -x -LLL '(uid=alex)'"));
    assert_eq!(
        run("ldapsearch -x -LLL '(uid=alex)' 1.1 uid"),
        run("ldapsearch -x -LLL '(uid=alex)' uid")
    );
}
#[test]
fn literal_shell_quoting_preserves_ldap_escapes_without_a_real_shell() {
    for command in [
        r"ldapsearch -x -LLL '(cn=Sam \2a Visitor)' uid",
        r#"ldapsearch -x -LLL "(cn=Sam \2a Visitor)" uid"#,
        r#"ldapsearch -x -LLL "(cn=Sam \\2a Visitor)" uid"#,
    ] {
        assert_eq!(count(command), 1, "{command}");
    }
    assert_eq!(count(r"ldapsearch -x -LLL \(uid=alex\) uid"), 1);
}
#[test]
fn empty_search_is_successful_but_missing_base_and_unavailable_are_not() {
    let empty = run("ldapsearch -x -LLL '(uid=nobody)' uid");
    assert!(empty.succeeded());
    assert_eq!(empty.output().unwrap(), "");
    let absent = run("ldapsearch -x -LLL -b 'ou=absent,dc=example,dc=test' '(uid=alex)'");
    assert!(!absent.succeeded());
    assert!(absent.output().unwrap().contains("(32)"));
    let unavailable = run("ldapsearch -x -LLL -H ldap://absent.example.test '(uid=alex)'");
    assert!(!unavailable.succeeded());
    assert!(unavailable.output().unwrap().contains("No host connection"));
}
#[test]
fn search_reports_unknown_filter_attributes_without_matching_everyone() {
    assert_eq!(count("ldapsearch -x -LLL '(!(shoeSize=12))'"), 0);
    assert_eq!(count("ldapsearch -x -LLL '(|(uid=alex)(shoeSize=12))'"), 1);
}
#[test]
fn unsupported_commands_shell_features_and_credentials_are_never_executed() {
    for raw in [
        "id",
        "curl https://example.test",
        "ldapsearch -x -LLL '(uid=alex)'; id",
        "ldapsearch -x -LLL '(uid=alex)' | cat",
        "ldapsearch -x -LLL '(uid=alex)' > /etc/hosts",
        "ldapsearch -x -LLL \"(uid=$USER)\"",
        "ldapsearch -x -LLL '(uid=alex)' $(id)",
        "ldapsearch -x -LLL (uid=alex)",
        "ldapsearch -x -LLL '(uid=alex)' *",
        "ldapsearch -x -LLL '(uid=alex)'\nid",
        "ldapsearch -x -LLL -D cn=admin '(uid=alex)'",
        "ldapsearch -x -LLL -w secret '(uid=alex)'",
        "ldapsearch -LLL '(uid=alex)'",
        "ldapsearch -x '(uid=alex)'",
        "ldapsearch -x -LLL -f /etc/passwd '(uid=alex)'",
        "ldapsearch -x -LLL '(uid=alex)' '+'",
    ] {
        assert!(env().run(&settings(), raw).is_err(), "{raw:?}");
    }
}
#[test]
fn malformed_options_and_unsupported_forms_are_specific_diagnostics() {
    for raw in [
        "ldapsearch -x -LLL",
        "ldapsearch -x -LLL -b",
        "ldapsearch -x -LLL -s children '(uid=alex)'",
        "ldapsearch -x -x -LLL '(uid=alex)'",
        "ldapsearch -x -LLL '(uid=alex)' ;",
        "ldapsearch -x -LLL '(uid=alex)",
    ] {
        assert!(env().run(&settings(), raw).is_err(), "{raw:?}");
    }
    assert!(matches!(
        env().run(&settings(), "ldapsearch -x -LLL '(uid=alex'"),
        Err(q::Error::Syntax(_))
    ));
    assert!(matches!(
        env().run(&settings(), "ldapsearch -x -LLL '(uid=al*)'"),
        Err(q::Error::Unsupported(_))
    ));
}
#[test]
fn uri_validation_selects_only_supplied_virtual_identifiers() {
    assert_eq!(
        c::uri_key("ldap://Directory.example.test:389/").unwrap(),
        "ldap://directory.example.test"
    );
    assert_eq!(
        c::uri_key("ldap://directory.example.test:1389").unwrap(),
        "ldap://directory.example.test:1389"
    );
    for bad in [
        "ldap://",
        "ldaps://example.test",
        "ldapi:///run/ldap",
        "file:///etc/passwd",
        "ldap://user:word@example.test",
        "ldap://example.test/path",
        "ldap://example.test?base",
        "ldap://example.test:0",
        "ldap://example.test:65536",
        "ldap://-bad.example.test",
        "ldap://example..test",
        "ldap://example.test:389:390",
        "ldap://[::1]",
        "ldap://example.test/#fragment",
    ] {
        assert!(c::uri_key(bad).is_err(), "{bad}");
    }
}
#[test]
fn configuration_rejects_unimplemented_options_duplicate_fields_and_limits() {
    for raw in [
        "BASE dc=example,dc=test",
        "URI ldap://example.test",
        "URI ldap://example.test\nBASE\n",
        "URI ldap://example.test\nURI ldap://other.test\nBASE dc=example,dc=test",
        "URI ldap://example.test\nBASE dc=example,dc=test\nTLS_REQCERT never",
        "URI \"ldap://example.test\"\nBASE dc=example,dc=test",
        "URI ldap://one.test ldap://two.test\nBASE dc=example,dc=test",
        "URI ldap://example.test\nBASE cn=René,dc=test",
    ] {
        assert!(c::Settings::parse(raw).is_err(), "{raw}");
    }
    assert!(matches!(
        c::Settings::parse(&" ".repeat(8193)),
        Err(q::Error::Limit(_))
    ));
    assert!(matches!(
        c::Settings::parse(&"\n".repeat(65)),
        Err(q::Error::Limit(_))
    ));
    assert!(matches!(
        c::Query::parse(&settings(), &"x".repeat(4097)),
        Err(q::Error::Limit(_))
    ));
    assert!(
        env()
            .run(
                &settings(),
                &format!(
                    "ldapsearch -x -LLL '(uid=alex)' {}",
                    vec!["uid"; 65].join(" ")
                )
            )
            .is_err()
    );
}
#[test]
fn multiple_directories_are_distinct_and_duplicate_endpoint_names_fail() {
    let mut other = fixture::fixture();
    other.entries[2]
        .attributes
        .iter_mut()
        .find(|a| a.name == "cn")
        .unwrap()
        .values = vec!["Alex Staging".into()];
    let e = c::Environment::new(&[
        (settings().uri.clone(), fixture::fixture()),
        ("ldap://staging.example.test".into(), other),
    ])
    .unwrap();
    let a = e
        .run(&settings(), "ldapsearch -x -LLL '(uid=alex)' cn")
        .unwrap();
    let b = e
        .run(
            &settings(),
            "ldapsearch -x -LLL -H ldap://staging.example.test '(uid=alex)' cn",
        )
        .unwrap();
    assert_ne!(a, b);
    assert!(b.output().unwrap().contains("Alex Staging"));
    assert!(
        c::Environment::new(&[
            (settings().uri, fixture::fixture()),
            (
                "ldap://DIRECTORY.example.test:389/".into(),
                fixture::fixture()
            )
        ])
        .is_err()
    );
    assert!(c::Environment::new(&[]).is_err());
    assert!(
        c::Environment::new(&vec![("ldap://example.test".into(), fixture::fixture()); 5]).is_err()
    );
}
#[test]
fn ldif_base64_preserves_unsafe_initial_and_trailing_spaces() {
    let mut s = fixture::fixture();
    s.entries[2].attributes.push(fixture::attr(
        "description",
        &[" leading", ":colon", "<url", "tail "],
    ));
    let e = c::Environment::new(&[(settings().uri, s)]).unwrap();
    let text = e
        .run(&settings(), "ldapsearch -x -LLL '(uid=alex)' description")
        .unwrap()
        .output()
        .unwrap();
    for line in [
        "description:: IGxlYWRpbmc=",
        "description:: OmNvbG9u",
        "description:: PHVybA==",
        "description:: dGFpbCA=",
    ] {
        assert!(text.contains(line), "{text}");
    }
}
#[test]
fn output_limit_fails_explicitly_without_a_truncated_success() {
    let mut s = fixture::fixture();
    for e in &mut s.entries {
        e.attributes.push(fixture::attr(
            "description",
            &(0..16).map(|_| "x").collect::<Vec<_>>(),
        ));
        let a = e.attributes.last_mut().unwrap();
        a.values = (0..16)
            .map(|i| format!("{i:02}{}", "z".repeat(254)))
            .collect();
    }
    // Seven entries remain under32KiB; add distinct sibling people to exceed it.
    let template = s.entries[2].clone();
    for i in 0..6 {
        let mut e = template.clone();
        e.dn = format!("uid=extra{i},ou=People,dc=example,dc=test");
        e.attributes
            .iter_mut()
            .find(|a| a.name == "uid")
            .unwrap()
            .values = vec![format!("extra{i}")];
        s.entries.push(e);
    }
    let e = c::Environment::new(&[(settings().uri, s)]).unwrap();
    assert!(matches!(
        e.run(&settings(), "ldapsearch -x -LLL '(objectClass=*)'"),
        Err(q::Error::Limit(_))
    ));
    assert!(
        e.run(&settings(), "ldapsearch -x -LLL '(objectClass=*)' uid")
            .is_ok()
    );
}
#[test]
fn public_result_and_query_boundaries_reject_unbounded_or_control_text() {
    let mut q = c::Query::parse(&settings(), "ldapsearch -x -LLL '(uid=alex)' uid").unwrap();
    q.attributes = vec!["uid".into(); 65];
    assert!(env().search(&q).is_err());
    q.attributes = vec!["uid;bad".into()];
    assert!(env().search(&q).is_err());
    let out = c::Outcome::Entries(vec![c::ResultEntry {
        dn: "uid=alex\ninjected: yes".into(),
        attributes: vec![],
    }]);
    assert!(out.output().is_err());
    let out = c::Outcome::Entries(vec![c::ResultEntry {
        dn: "uid=alex,dc=test".into(),
        attributes: vec![fixture::attr("cn", &["Alex\ninjected"])],
    }]);
    assert!(out.output().is_err());
}
