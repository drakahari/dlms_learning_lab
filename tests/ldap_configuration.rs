use dlms_learning_lab::{
    ldap_access::{Level, Request, Subject},
    ldap_configuration::{Configuration, DatabaseSource, Error, Source},
    ldap_ldif::{self, Change, Edit, EditKind, configuration as codec},
    ldap_query::Dn,
};
#[path = "support/ldap_directory.rs"]
mod fixture;
const DATA: &str = "olcDatabase={1}mdb,cn=config";
const AUDIT: &str = "olcDatabase={2}mdb,cn=config";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
fn dn(s: &str) -> Dn {
    Dn::parse(s).unwrap()
}
fn values(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|s| (*s).into()).collect()
}
fn source() -> Source {
    Source {
        version: 1,
        databases: vec![
            DatabaseSource {
                name: "olcDatabase={-1}frontend,cn=config".into(),
                suffix: None,
                root_dn: None,
                access: vec![],
            },
            DatabaseSource {
                name: "olcDatabase={0}config,cn=config".into(),
                suffix: None,
                root_dn: None,
                access: values(&["to * by * none"]),
            },
            DatabaseSource {
                name: DATA.into(),
                suffix: Some("dc=example,dc=test".into()),
                root_dn: None,
                access: values(&["to * by users write by * none"]),
            },
            DatabaseSource {
                name: AUDIT.into(),
                suffix: Some("dc=audit,dc=test".into()),
                root_dn: None,
                access: values(&["to * by * read"]),
            },
        ],
    }
}
fn document(name: &str, rules: &[&str]) -> String {
    format!(
        "version: 1\n\ndn: {name}\nchangetype: modify\nreplace: olcAccess\n{}-\n",
        rules
            .iter()
            .map(|s| format!("olcAccess: {s}\n"))
            .collect::<String>()
    )
}
fn can(c: &Configuration, actor: &str, target: &str, attr: &str, request: Request) -> bool {
    let (policy, root) = c.policy_for(&dn(target)).unwrap();
    policy
        .evaluate(
            &Subject::Authenticated(dn(actor)),
            &dn(target),
            attr,
            request,
            root.as_ref(),
        )
        .unwrap()
        .allowed
}
#[test]
fn config_names_are_separate_from_unchanged_data_dn_grammar_and_use_actual_indices() {
    assert!(Dn::parse(DATA).is_err());
    assert_eq!(
        codec::Name::parse("OLCDATABASE={1}MDB,CN=CONFIG").unwrap(),
        codec::Name::parse(DATA).unwrap()
    );
    assert_eq!(codec::Name::parse(DATA).unwrap().text(), DATA);
    assert_eq!(
        codec::Name::parse("cn=config").unwrap(),
        codec::Name::Global
    );
    for name in [
        "olcDatabase=mdb,cn=config",
        "olcDatabase={01}mdb,cn=config",
        "olcDatabase={32}mdb,cn=config",
        "olcDatabase={1}hdb,cn=config",
        "olcDatabase={0}mdb,cn=config",
        "olcDatabase={1}config,cn=config",
        "cn=schema,cn=config",
        "olcDatabase={1}mdb,dc=example",
        "olcDatabase={1}mdb, cn=config",
    ] {
        assert!(codec::Name::parse(name).is_err(), "{name}");
    }
}
#[test]
fn config_codec_reads_real_modify_records_with_folding_crlf_case_and_base64() {
    let ordinary = document(
        DATA,
        &["to attrs=mail by self write by * none", "to * by * read"],
    );
    let expected = codec::parse(&ordinary).unwrap();
    let folded = ordinary.replace(
        "to attrs=mail by self write by * none",
        "to attrs=mail\n  by self write\n  by * none",
    );
    assert_eq!(
        codec::parse(&folded.replace('\n', "\r\n")).unwrap(),
        expected
    );
    assert_eq!(
        codec::parse(&ordinary.replace(
            "olcAccess: to * by * read",
            "olcAccess:: dG8gKiBieSAqIHJlYWQ="
        ))
        .unwrap(),
        expected
    );
    assert_eq!(
        codec::parse(
            &ordinary
                .replace("changetype: modify", "CHANGETYPE: MODIFY")
                .replace("replace: olcAccess", "REPLACE: OLCACCESS")
        )
        .unwrap(),
        expected
    );
    // Old contact codec remains restricted; it cannot silently handle config.
    assert!(ldap_ldif::parse(&ordinary).is_err());
}
#[test]
fn unsupported_directives_urls_controls_and_multiple_edits_are_honest_noops() {
    let valid = document(DATA, &["to * by * read"]);
    for bad in [
        valid.replace("replace: olcAccess", "replace: olcSuffix"),
        valid.replace("changetype: modify", "changetype: add"),
        valid.replace("replace: olcAccess", "delete: olcAccess"),
        valid.replace(
            "olcAccess: to * by * read",
            "olcAccess:< file:///etc/passwd",
        ),
        valid.replace("olcAccess: to * by * read", "control: 1.2.3"),
        valid.replace("-\n", "-\nreplace: olcRootPW\nolcRootPW: fictional\n-\n"),
        valid.replace("olcAccess: to * by * read", "olcAccess;binary: ignored"),
    ] {
        let mut c = Configuration::new(&source()).unwrap();
        let before = c.source();
        let result = c.apply_document(&bad);
        assert!(result.error.is_some() && result.applied.is_empty());
        assert_eq!(c.source(), before);
    }
}
#[test]
fn source_and_codec_bounds_and_unknown_fields_refuse_without_unbounded_work() {
    let valid = document(DATA, &["to * by * read"]);
    assert!(codec::parse(&valid.replace("to * by * read", &"x".repeat(1025))).is_err());
    assert!(
        codec::parse(&format!(
            "version: 1\n\n{}",
            valid.split_once("\n\n").unwrap().1.repeat(17)
        ))
        .is_err()
    );
    assert!(codec::parse(&format!("#{}", "x".repeat(64 * 1024))).is_err());
    let mut s = serde_json::to_value(source()).unwrap();
    s["databases"][2]["access"] = serde_json::json!(vec!["x".repeat(1025)]);
    assert!(serde_json::from_value::<Source>(s).is_err());
    let mut s = serde_json::to_value(source()).unwrap();
    s["databases"][2]["suffix"] = serde_json::json!("x".repeat(1025));
    assert!(serde_json::from_value::<Source>(s).is_err());
    let mut s = serde_json::to_value(source()).unwrap();
    s["databases"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({}));
    assert!(serde_json::from_value::<Source>(s).is_err());
    let mut s = serde_json::to_value(source()).unwrap();
    s["command"] = serde_json::json!("ignored");
    assert!(serde_json::from_value::<Source>(s).is_err());
    let mut s = source();
    s.version = 2;
    assert!(Configuration::new(&s).is_err());
    let mut s = source();
    s.databases[3].suffix = Some("ou=Archive,dc=example,dc=test".into());
    assert!(Configuration::new(&s).is_err());
}
#[test]
fn complete_replace_changes_actual_permissions_and_preserves_all_other_settings() {
    let mut c = Configuration::new(&source()).unwrap();
    let before = c.source();
    assert!(can(&c, SAM, ALEX, "mail", Request::Write));
    let result = c.apply_document(&document(
        DATA,
        &[
            "to attrs=mail,description by self write by * none",
            "to * by * read",
        ],
    ));
    assert!(result.error.is_none() && result.applied.len() == 1);
    assert!(can(&c, ALEX, ALEX, "mail", Request::Write));
    assert!(!can(&c, SAM, ALEX, "mail", Request::Write));
    let after = c.source();
    for n in [0, 1, 3] {
        assert_eq!(after.databases[n], before.databases[n]);
    }
    assert_eq!(after.databases[2].suffix, before.databases[2].suffix);
    assert_eq!(after.databases[2].root_dn, before.databases[2].root_dn);
    assert_eq!(
        after.databases[2].access[0],
        "{0}to attrs=mail,description by self write by * none"
    );
}
#[test]
fn explicit_value_indices_preserve_policy_order_even_when_serialized_out_of_order() {
    let mut a = Configuration::new(&source()).unwrap();
    let mut b = a.clone();
    let plain = [
        "to attrs=mail by self write by * none",
        "to * by users read by * none",
    ];
    assert!(a.apply_document(&document(DATA, &plain)).error.is_none());
    assert!(
        b.apply_document(&document(
            DATA,
            &[
                "{1}to * by users read by * none",
                "{0}to attrs=mail by self write by * none"
            ]
        ))
        .error
        .is_none()
    );
    assert_eq!(a, b);
    assert!(can(&b, ALEX, ALEX, "mail", Request::Write));
    assert!(!can(&b, SAM, ALEX, "mail", Request::Write));
}
#[test]
fn ambiguous_indices_and_unsupported_policy_cannot_partially_commit() {
    for rules in [
        vec!["{0}to * by * read", "to * by * none"],
        vec!["{0}to * by * read", "{0}to * by * none"],
        vec!["{1}to * by * read"],
        vec!["{00}to * by * read"],
        vec![
            "to attrs=mail by self write by * none",
            "to filter=(uid=*) by * read",
        ],
        vec!["access to * by * read"],
        vec!["to * by * read\naccess to * by * write"],
    ] {
        let mut c = Configuration::new(&source()).unwrap();
        let before = c.source();
        let result = c.replace(&codec::Modification {
            name: codec::Name::parse(DATA).unwrap(),
            values: values(&rules),
        });
        assert!(result.is_err(), "{rules:?}");
        assert_eq!(c.source(), before);
    }
}
#[test]
fn backend_rules_precede_frontend_and_empty_combined_rules_use_read_only_default() {
    let mut s = source();
    s.databases[0].access = values(&["to * by users write by * none"]);
    s.databases[2].access = values(&["to attrs=description by * none"]);
    let c = Configuration::new(&s).unwrap();
    assert!(!can(&c, ALEX, ALEX, "description", Request::Write));
    assert!(can(&c, ALEX, ALEX, "mail", Request::Write));
    let mut c = Configuration::new(&source()).unwrap();
    assert!(c.apply_document(&document(DATA, &[])).error.is_none());
    let (policy, _) = c.policy_for(&dn(ALEX)).unwrap();
    assert!(
        policy
            .evaluate(&Subject::Anonymous, &dn(ALEX), "mail", Request::Read, None)
            .unwrap()
            .allowed
    );
    assert!(
        !policy
            .evaluate(&Subject::Anonymous, &dn(ALEX), "mail", Request::Write, None)
            .unwrap()
            .allowed
    );
}
#[test]
fn supplied_rootdn_bypass_is_preserved_without_inventing_authentication() {
    let mut s = source();
    s.databases[2].root_dn = Some(SAM.into());
    let mut c = Configuration::new(&s).unwrap();
    assert!(
        c.apply_document(&document(DATA, &["to * by * none"]))
            .error
            .is_none()
    );
    let (policy, root) = c.policy_for(&dn(ALEX)).unwrap();
    let decision = policy
        .evaluate(
            &Subject::Authenticated(dn(SAM)),
            &dn(ALEX),
            "mail",
            Request::Write,
            root.as_ref(),
        )
        .unwrap();
    assert!(decision.allowed);
    assert_eq!(decision.granted, Level::Manage);
    assert!(!can(&c, ALEX, ALEX, "mail", Request::Write));
    assert_eq!(c.source().databases[2].root_dn, s.databases[2].root_dn);
}
#[test]
fn wrong_known_database_changes_only_that_database_and_unknown_or_protected_targets_do_not() {
    let mut c = Configuration::new(&source()).unwrap();
    let before = c.source();
    assert!(
        c.apply_document(&document(AUDIT, &["to * by * none"]))
            .error
            .is_none()
    );
    assert_eq!(c.source().databases[2], before.databases[2]);
    assert_ne!(c.source().databases[3], before.databases[3]);
    assert!(can(&c, SAM, ALEX, "mail", Request::Write));
    for name in [
        "olcDatabase={3}mdb,cn=config",
        "olcDatabase={0}config,cn=config",
        "olcDatabase={-1}frontend,cn=config",
        "cn=config",
    ] {
        let before = c.source();
        assert!(
            c.apply_document(&document(name, &["to * by * read"]))
                .error
                .is_some()
        );
        assert_eq!(c.source(), before);
    }
}
#[test]
fn effective_policy_limits_are_checked_before_a_single_request_commits() {
    let mut s = source();
    s.databases[0].access = (0..15)
        .map(|n| format!("to dn.exact=\"uid=reader{n},ou=People,dc=example,dc=test\" by * read"))
        .collect();
    let mut c = Configuration::new(&s).unwrap();
    let before = c.source();
    let result = c.apply_document(&document(
        DATA,
        &["to attrs=mail by self write by * none", "to * by * read"],
    ));
    assert!(matches!(result.error, Some(Error::Policy(_))));
    assert!(result.applied.is_empty());
    assert_eq!(c.source(), before);
}
#[test]
fn later_semantic_failure_retains_earlier_commits_but_reports_document_failure() {
    let mut c = Configuration::new(&source()).unwrap();
    let before = c.source();
    let first = document(
        DATA,
        &["to attrs=mail by self write by * none", "to * by * read"],
    );
    let second = document("olcDatabase={7}mdb,cn=config", &["to * by * none"]);
    let run = c.apply_document(&(first + "\n" + second.split_once("\n\n").unwrap().1));
    assert_eq!(run.applied.len(), 1);
    assert_eq!(run.error, Some(Error::NoSuchObject));
    assert_ne!(c.source().databases[2], before.databases[2]);
    assert_eq!(c.source().databases[3], before.databases[3]);
    assert!(!can(&c, SAM, ALEX, "mail", Request::Write));
}
#[test]
fn malformed_document_syntax_preflight_is_distinct_from_later_request_failure() {
    let mut c = Configuration::new(&source()).unwrap();
    let before = c.source();
    let bad = document(DATA, &["to * by * read"])
        + "\ndn: not-a-config-name\nchangetype: modify\nreplace: olcAccess\n-\n";
    let run = c.apply_document(&bad);
    assert!(matches!(run.error, Some(Error::Ldif(_))));
    assert!(run.applied.is_empty());
    assert_eq!(c.source(), before);
}
fn realm() -> dlms_learning_lab::ldap_identity::protected::ProtectedRealm {
    use dlms_learning_lab::{
        ldap_identity::{self as id, protected as p},
        ldap_protection as tls,
    };
    let endpoint = tls::Endpoint::new(
        "ldap://config.example.test",
        true,
        Some(tls::Certificate::new(&["config.example.test"], "sample-root", 0, 100, true).unwrap()),
    )
    .unwrap();
    let identities = [ALEX, SAM].map(|n| {
        id::Identity::new(
            dn(n),
            "Fictional-only-83",
            dn("dc=example,dc=test"),
            false,
            false,
            &[],
        )
        .unwrap()
    });
    p::ProtectedRealm::new(
        endpoint,
        &fixture::fixture(),
        &identities,
        p::Policy {
            supports_password_modify: false,
            allow_self_change: false,
            require_old_password: false,
        },
    )
    .unwrap()
}
fn change(fields: &[(&str, &str)]) -> Change {
    Change::Modify {
        dn: dn(ALEX),
        edits: fields
            .iter()
            .map(|(name, value)| Edit {
                kind: EditKind::Replace,
                attribute: fixture::attr(name, &[*value]),
            })
            .collect(),
    }
}
#[test]
fn deployed_policy_drives_actual_protected_modify_and_preserves_denied_peer_changes() {
    use dlms_learning_lab::ldap_protection as tls;
    let mut c = Configuration::new(&source()).unwrap();
    assert!(
        c.apply_document(&document(
            DATA,
            &[
                "to attrs=mail,description by self write by * none",
                "to * by * read"
            ]
        ))
        .error
        .is_none()
    );
    let (policy, root) = c.policy_for(&dn(ALEX)).unwrap();
    let mut r = realm();
    let connection = r
        .open(
            "ldap://config.example.test",
            tls::Request::RequireStartTls,
            &tls::Trust::new(&["sample-root"]).unwrap(),
            50,
        )
        .unwrap();
    let own = r
        .authenticate(&connection, &dn(ALEX), "Fictional-only-83")
        .unwrap();
    let peer = r
        .authenticate(&connection, &dn(SAM), "Fictional-only-83")
        .unwrap();
    let before = r.directory().source().clone();
    r.modify_with_acl(
        &own,
        &policy,
        root.as_ref(),
        &change(&[("mail", "alex.new@example.test")]),
    )
    .unwrap();
    assert_ne!(r.directory().source(), &before);
    let updated = r.directory().source().clone();
    assert!(
        r.modify_with_acl(
            &peer,
            &policy,
            root.as_ref(),
            &change(&[("mail", "forbidden@example.test")])
        )
        .is_err()
    );
    assert_eq!(r.directory().source(), &updated);
}
#[test]
fn deployed_policy_denial_of_a_later_field_keeps_actual_directory_atomically_unchanged() {
    use dlms_learning_lab::ldap_protection as tls;
    let mut c = Configuration::new(&source()).unwrap();
    assert!(
        c.apply_document(&document(
            DATA,
            &["to attrs=mail by self write by * none", "to * by * read"]
        ))
        .error
        .is_none()
    );
    let (policy, root) = c.policy_for(&dn(ALEX)).unwrap();
    let mut r = realm();
    let connection = r
        .open(
            "ldap://config.example.test",
            tls::Request::RequireStartTls,
            &tls::Trust::new(&["sample-root"]).unwrap(),
            50,
        )
        .unwrap();
    let own = r
        .authenticate(&connection, &dn(ALEX), "Fictional-only-83")
        .unwrap();
    let before = r.directory().source().clone();
    assert!(
        r.modify_with_acl(
            &own,
            &policy,
            root.as_ref(),
            &change(&[
                ("mail", "alex.new@example.test"),
                ("description", "must not partly change")
            ])
        )
        .is_err()
    );
    assert_eq!(r.directory().source(), &before);
}
