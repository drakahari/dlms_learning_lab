use dlms_learning_lab::{
    ldap_access as a,
    ldap_identity::{
        self as i,
        protected::{self as p, ProtectedRealm, access::Error},
    },
    ldap_ldif::{Change, Edit, EditKind},
    ldap_protection as tls,
    ldap_query::Dn,
};
#[path = "support/ldap_directory.rs"]
mod fixture;
const URI: &str = "ldap://directory.example.test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const OLD: &str = "Fictional-original-only-81";
const NEW: &str = "Fictional-replacement-only-82";
fn dn(s: &str) -> Dn {
    Dn::parse(s).unwrap()
}
fn realm() -> ProtectedRealm {
    let endpoint = tls::Endpoint::new(
        URI,
        true,
        Some(
            tls::Certificate::new(&["directory.example.test"], "training-root", 100, 200, true)
                .unwrap(),
        ),
    )
    .unwrap();
    let people = [ALEX, SAM].map(|name| {
        i::Identity::new(
            dn(name),
            OLD,
            dn("ou=People,dc=example,dc=test"),
            false,
            false,
            &[],
        )
        .unwrap()
    });
    ProtectedRealm::new(
        endpoint,
        &fixture::fixture(),
        &people,
        p::Policy {
            supports_password_modify: true,
            allow_self_change: true,
            require_old_password: false,
        },
    )
    .unwrap()
}
fn connection(r: &ProtectedRealm) -> tls::Connection {
    r.open(
        URI,
        tls::Request::RequireStartTls,
        &tls::Trust::new(&["training-root"]).unwrap(),
        150,
    )
    .unwrap()
}
fn auth(r: &ProtectedRealm, who: &str) -> p::Authenticated {
    r.authenticate(&connection(r), &dn(who), OLD).unwrap()
}
fn change(target: &str, fields: &[(&str, &str)]) -> Change {
    Change::Modify {
        dn: dn(target),
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
fn actual_self_modify_uses_ordered_acl_and_preserves_other_people_and_credentials() {
    let mut r = realm();
    let before = r.directory().source().clone();
    let me = auth(&r, ALEX);
    let policy =
        a::Policy::parse("access to attrs=mail,description by self write by * read").unwrap();
    let applied = r
        .modify_with_acl(
            &me,
            &policy,
            None,
            &change(
                ALEX,
                &[("mail", "alex@updated.test"), ("description", "New desk")],
            ),
        )
        .unwrap();
    assert_eq!(applied.decisions.len(), 2);
    assert!(applied.decisions.iter().all(|(_, d)| d.allowed));
    let after = r.directory().source();
    assert_ne!(after, &before);
    for old in before
        .entries
        .iter()
        .filter(|e| !dn(&e.dn).equivalent(&dn(ALEX)))
    {
        assert!(after.entries.contains(old));
    }
    assert!(r.authenticate(&connection(&r), &dn(ALEX), OLD).is_ok());
    assert!(r.authenticate(&connection(&r), &dn(SAM), OLD).is_ok());
    assert!(r.authenticate(&connection(&r), &dn(ALEX), NEW).is_err());
}
#[test]
fn an_early_allowed_attribute_and_later_denial_leave_the_entire_entry_unchanged() {
    let mut r = realm();
    let before = r.directory().source().clone();
    let me = auth(&r, ALEX);
    let policy =
        a::Policy::parse("access to attrs=mail by self write\naccess to * by * read").unwrap();
    let result = r.modify_with_acl(
        &me,
        &policy,
        None,
        &change(
            ALEX,
            &[("mail", "alex@updated.test"), ("description", "New desk")],
        ),
    );
    let Error::Denied {
        attribute,
        decision,
    } = result.unwrap_err()
    else {
        panic!()
    };
    assert_eq!(attribute, "description");
    assert_eq!(decision.rule, Some(2));
    assert_eq!(r.directory().source(), &before);
}
#[test]
fn another_person_cannot_use_self_rights_and_removing_the_policy_defaults_to_read_not_write() {
    let mut r = realm();
    let before = r.directory().source().clone();
    let me = auth(&r, ALEX);
    for text in ["access to * by self write by * read", ""] {
        let policy = a::Policy::parse(text).unwrap();
        assert!(matches!(
            r.modify_with_acl(
                &me,
                &policy,
                None,
                &change(SAM, &[("mail", "wrong@updated.test")])
            ),
            Err(Error::Denied { .. })
        ));
        assert_eq!(r.directory().source(), &before);
    }
}
#[test]
fn a_foreign_established_context_is_rejected_before_mutation_even_for_an_allow_all_policy() {
    let other = realm();
    let foreign = auth(&other, ALEX);
    let mut r = realm();
    let before = r.directory().source().clone();
    let p = a::Policy::parse("access to * by * manage").unwrap();
    let error = r
        .modify_with_acl(
            &foreign,
            &p,
            None,
            &change(ALEX, &[("mail", "wrong@updated.test")]),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Authentication(p::Error::Protection(tls::Error::ForeignConnection))
    ));
    assert_eq!(r.directory().source(), &before);
}
#[test]
fn schema_and_unsupported_operations_are_not_bypassed_by_manage_or_rootdn() {
    let mut r = realm();
    let before = r.directory().source().clone();
    let me = auth(&r, ALEX);
    let policy = a::Policy::parse("access to * by * manage").unwrap();
    for c in [
        Change::Delete(dn(SAM)),
        Change::Add(before.entries[0].clone()),
    ] {
        assert_eq!(
            r.modify_with_acl(&me, &policy, Some(&dn(ALEX)), &c),
            Err(Error::Unsupported)
        );
    }
    let oversized = "value".repeat(300);
    for fields in [
        vec![("uid", "changed")],
        vec![("userPassword", "not-a-password-write")],
        vec![("mail", oversized.as_str())],
    ] {
        assert!(matches!(
            r.modify_with_acl(&me, &policy, Some(&dn(ALEX)), &change(ALEX, &fields)),
            Err(Error::Request(_))
        ));
        assert_eq!(r.directory().source(), &before);
    }
}
#[test]
fn acl_modifications_do_not_reset_rotated_credentials_or_invalidate_existing_bound_contexts() {
    let mut r = realm();
    let me = auth(&r, ALEX);
    r.change_password(&me, None, None, Some(NEW)).unwrap();
    let policy = a::Policy::parse("access to * by self write by * none").unwrap();
    r.modify_with_acl(
        &me,
        &policy,
        None,
        &change(ALEX, &[("description", "New desk")]),
    )
    .unwrap();
    assert!(r.authenticate(&connection(&r), &dn(ALEX), OLD).is_err());
    assert!(r.authenticate(&connection(&r), &dn(ALEX), NEW).is_ok());
    assert!(r.authenticate(&connection(&r), &dn(SAM), OLD).is_ok());
}
#[test]
fn reordered_equivalent_attribute_requests_have_the_same_actual_result() {
    let mut first = realm();
    let mut second = realm();
    let a = auth(&first, ALEX);
    let b = auth(&second, ALEX);
    let policy = a::Policy::parse("access to * by self write by * none").unwrap();
    first
        .modify_with_acl(
            &a,
            &policy,
            None,
            &change(
                ALEX,
                &[("mail", "alex@updated.test"), ("description", "New desk")],
            ),
        )
        .unwrap();
    second
        .modify_with_acl(
            &b,
            &policy,
            None,
            &change(
                ALEX,
                &[("description", "New desk"), ("mail", "alex@updated.test")],
            ),
        )
        .unwrap();
    assert_eq!(first.directory().source(), second.directory().source());
}
