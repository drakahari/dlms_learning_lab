use dlms_learning_lab::{ldap_changes as c, ldap_identity as i, ldap_ldif as l, ldap_query as q};
#[path = "support/ldap_directory.rs"]
mod fixture;
use fixture::{attr, fixture};
const URI: &str = "ldap://directory.example.test";
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const SAMPLE: &str = "Fictional-only-41";
fn dn(raw: &str) -> q::Dn {
    q::Dn::parse(raw).unwrap()
}
fn identity(principal: &str, add: bool, delete: bool, attrs: &[&str]) -> i::Identity {
    i::Identity::new(dn(principal), SAMPLE, dn(BASE), add, delete, attrs).unwrap()
}
fn realm() -> i::Realm {
    i::Realm::new(
        URI,
        &fixture(),
        &[
            identity(ALEX, true, true, &["mail", "description"]),
            identity(SAM, false, false, &["description"]),
        ],
    )
    .unwrap()
}
fn modify(target: &str, name: &str, value: &str) -> l::Change {
    l::Change::Modify {
        dn: dn(target),
        edits: vec![l::Edit {
            kind: l::EditKind::Replace,
            attribute: attr(name, &[value]),
        }],
    }
}
fn code(result: Result<c::Applied, i::Error>, expected: c::Code) {
    assert!(
        matches!(result, Err(i::Error::Request(c::Error::Result{code,..})) if code==expected),
        "{expected:?}"
    );
}
#[test]
fn a_name_alone_or_the_wrong_sample_password_never_authorizes_a_request() {
    let r = realm();
    for (name, password) in [
        (ALEX, "wrong"),
        (SAM, "wrong"),
        ("uid=absent,ou=People,dc=example,dc=test", SAMPLE),
    ] {
        assert!(matches!(
            r.authenticate(&dn(name), password),
            Err(i::Error::InvalidCredentials)
        ));
    }
    assert!(matches!(
        r.authenticate(&dn(ALEX), ""),
        Err(i::Error::Unsupported(_))
    ));
    assert!(r.authenticate(&dn(ALEX), SAMPLE).is_ok());
}
#[test]
fn distinguished_names_use_supported_matching_but_passwords_are_exact() {
    let r = realm();
    assert!(
        r.authenticate(&dn("UID=ALEX,OU=people,DC=EXAMPLE,DC=TEST"), SAMPLE)
            .is_ok()
    );
    for password in [
        SAMPLE.to_ascii_lowercase(),
        format!("{SAMPLE} "),
        format!(" {SAMPLE}"),
    ] {
        assert!(matches!(
            r.authenticate(&dn(ALEX), &password),
            Err(i::Error::InvalidCredentials)
        ));
    }
}
#[test]
fn credentials_and_delegated_rights_are_separate_and_state_really_changes() {
    let mut r = realm();
    let limited = r.authenticate(&dn(SAM), SAMPLE).unwrap();
    let before = r.directory().clone();
    code(
        r.apply(&limited, &modify(ALEX, "mail", "new@example.test")),
        c::Code::InsufficientAccessRights,
    );
    assert_eq!(r.directory(), &before);
    assert_eq!(
        r.apply(&limited, &modify(ALEX, "description", "contact verified"))
            .unwrap()
            .operation,
        c::Operation::Modified
    );
    assert_ne!(r.directory(), &before);
    let full = r.authenticate(&dn(ALEX), SAMPLE).unwrap();
    assert!(
        r.apply(&full, &modify(ALEX, "mail", "new@example.test"))
            .is_ok()
    );
}
#[test]
fn add_and_delete_permissions_are_checked_independently() {
    let mut r = realm();
    let limited = r.authenticate(&dn(SAM), SAMPLE).unwrap();
    let full = r.authenticate(&dn(ALEX), SAMPLE).unwrap();
    let mut entry = fixture().entries[3].clone();
    entry.dn = format!("uid=new,{BASE}");
    entry
        .attributes
        .iter_mut()
        .find(|a| a.name == "uid")
        .unwrap()
        .values = vec!["new".into()];
    let newdn = dn(&entry.dn);
    let change = l::Change::Add(entry);
    code(
        r.apply(&limited, &change),
        c::Code::InsufficientAccessRights,
    );
    assert!(r.apply(&full, &change).is_ok());
    code(
        r.apply(&limited, &l::Change::Delete(newdn.clone())),
        c::Code::InsufficientAccessRights,
    );
    assert!(r.apply(&full, &l::Change::Delete(newdn)).is_ok());
}
#[test]
fn authenticated_identity_cannot_escape_the_delegated_subtree() {
    let mut r = realm();
    let session = r.authenticate(&dn(ALEX), SAMPLE).unwrap();
    let before = r.directory().clone();
    code(
        r.apply(
            &session,
            &modify(
                "ou=Teams,dc=example,dc=test",
                "description",
                "not delegated",
            ),
        ),
        c::Code::InsufficientAccessRights,
    );
    assert_eq!(r.directory(), &before);
}
#[test]
fn sessions_cannot_be_replayed_into_another_attempt_even_at_the_same_uri() {
    let first = realm();
    let session = first.authenticate(&dn(ALEX), SAMPLE).unwrap();
    let mut second = realm();
    let before = second.directory().clone();
    assert!(matches!(
        second.apply(&session, &modify(ALEX, "description", "replay")),
        Err(i::Error::ForeignSession)
    ));
    assert_eq!(second.directory(), &before);
    let valid = second.authenticate(&dn(ALEX), SAMPLE).unwrap();
    assert!(
        second
            .apply(&valid, &modify(ALEX, "description", "current attempt"))
            .is_ok()
    );
}
#[test]
fn retiring_an_identity_blocks_new_authentication_and_its_prior_write_grant() {
    let mut r = realm();
    let full = r.authenticate(&dn(ALEX), SAMPLE).unwrap();
    let limited = r.authenticate(&dn(SAM), SAMPLE).unwrap();
    assert!(r.apply(&full, &l::Change::Delete(dn(SAM))).is_ok());
    assert!(matches!(
        r.authenticate(&dn(SAM), SAMPLE),
        Err(i::Error::InvalidCredentials)
    ));
    code(
        r.apply(&limited, &modify(ALEX, "description", "retired")),
        c::Code::InsufficientAccessRights,
    );
}
#[test]
fn malformed_or_unknown_context_facts_fail_before_an_attempt_is_available() {
    let source = fixture();
    let unknown = identity("uid=absent,ou=People,dc=example,dc=test", true, true, &[]);
    assert!(matches!(
        i::Realm::new(URI, &source, &[unknown]),
        Err(i::Error::Invalid(_))
    ));
    let container = identity(BASE, true, true, &[]);
    assert!(matches!(
        i::Realm::new(URI, &source, &[container]),
        Err(i::Error::Invalid(_))
    ));
    let missing_base = i::Identity::new(
        dn(ALEX),
        SAMPLE,
        dn("ou=Missing,dc=example,dc=test"),
        true,
        true,
        &[],
    )
    .unwrap();
    assert!(matches!(
        i::Realm::new(URI, &source, &[missing_base]),
        Err(i::Error::Invalid(_))
    ));
    let duplicates = [
        identity(ALEX, true, true, &[]),
        identity("UID=ALEX,OU=PEOPLE,DC=EXAMPLE,DC=TEST", true, true, &[]),
    ];
    assert!(matches!(
        i::Realm::new(URI, &source, &duplicates),
        Err(i::Error::Invalid(_))
    ));
}
#[test]
fn empty_excessive_unsupported_and_invalid_inputs_are_bounded_honestly() {
    let source = fixture();
    assert!(matches!(
        i::Realm::new(URI, &source, &[]),
        Err(i::Error::Limit(_))
    ));
    assert!(matches!(
        i::Realm::new(URI, &source, &vec![identity(ALEX, true, true, &[]); 9]),
        Err(i::Error::Limit(_))
    ));
    for value in [
        "".into(),
        "a".repeat(129),
        "line\nfeed".into(),
        "café".into(),
    ] {
        assert!(i::Identity::new(dn(ALEX), &value, dn(BASE), true, true, &[]).is_err());
        assert!(realm().authenticate(&dn(ALEX), &value).is_err());
    }
    assert!(i::Identity::new(dn(ALEX), &"x".repeat(128), dn(BASE), false, false, &[]).is_ok());
    assert!(i::Identity::new(dn(ALEX), SAMPLE, dn(BASE), true, true, &["userPassword"]).is_err());
    assert!(
        i::Realm::new(
            "https://example.test",
            &source,
            &[identity(ALEX, true, true, &[])]
        )
        .is_err()
    );
    let mut malformed = source.clone();
    malformed.entries[2]
        .attributes
        .push(attr("userPassword", &[SAMPLE]));
    assert!(i::Realm::new(URI, &malformed, &[identity(ALEX, true, true, &[])]).is_err());
}
#[test]
fn diagnostics_and_debug_output_do_not_echo_the_sample_password() {
    let identity = identity(ALEX, true, true, &[]);
    assert!(!format!("{identity:?}").contains(SAMPLE));
    let r = realm();
    let session = r.authenticate(&dn(ALEX), SAMPLE).unwrap();
    for rendered in [
        format!("{r:?}"),
        format!("{session:?}"),
        r.authenticate(&dn(ALEX), "wrong-secret-51")
            .unwrap_err()
            .to_string(),
    ] {
        assert!(!rendered.contains(SAMPLE));
        assert!(!rendered.contains("wrong-secret-51"));
    }
}
#[test]
fn a_failed_later_login_has_no_write_effect_and_does_not_change_a_new_invocations_rights() {
    let mut r = realm();
    let before = r.directory().clone();
    assert!(r.authenticate(&dn(ALEX), "wrong").is_err());
    assert_eq!(r.directory(), &before);
    let limited = r.authenticate(&dn(SAM), SAMPLE).unwrap();
    code(
        r.apply(&limited, &modify(ALEX, "mail", "escalated@example.test")),
        c::Code::InsufficientAccessRights,
    );
    assert_eq!(r.directory(), &before);
    assert_eq!(r.uri(), URI);
}
