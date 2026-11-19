use dlms_learning_lab::{
    ldap_identity::{
        self as i,
        protected::{self as p, Policy, ProtectedRealm},
    },
    ldap_protection::{self as t, Certificate, Endpoint, Request, Trust},
    ldap_query as q,
};
#[path = "support/ldap_directory.rs"]
mod fixture;
const URI: &str = "ldap://directory.example.test";
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const OLD: &str = "Fictional-original-only-81";
const NEW: &str = "Fictional-replacement-only-82";
fn dn(raw: &str) -> q::Dn {
    q::Dn::parse(raw).unwrap()
}
fn policy() -> Policy {
    Policy {
        supports_password_modify: true,
        allow_self_change: true,
        require_old_password: false,
    }
}
fn realm_with(policy: Policy) -> ProtectedRealm {
    let endpoint = Endpoint::new(
        URI,
        true,
        Some(
            Certificate::new(&["directory.example.test"], "training-root", 100, 200, true).unwrap(),
        ),
    )
    .unwrap();
    let identities = [
        i::Identity::new(
            dn(ALEX),
            OLD,
            dn(BASE),
            true,
            true,
            &["mail", "description"],
        )
        .unwrap(),
        i::Identity::new(dn(SAM), OLD, dn(BASE), false, false, &[]).unwrap(),
    ];
    ProtectedRealm::new(endpoint, &fixture::fixture(), &identities, policy).unwrap()
}
fn connection(r: &ProtectedRealm) -> t::Connection {
    r.open(
        URI,
        Request::RequireStartTls,
        &Trust::new(&["training-root"]).unwrap(),
        150,
    )
    .unwrap()
}
#[test]
fn successful_change_is_verified_by_fresh_authentication_not_directory_output_or_command_text() {
    let mut r = realm_with(policy());
    let before = r.directory().source().clone();
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    r.change_password(&auth, None, None, Some(NEW)).unwrap();
    let fresh = connection(&r);
    assert!(matches!(
        r.authenticate(&fresh, &dn(ALEX), OLD),
        Err(p::Error::Identity(i::Error::InvalidCredentials))
    ));
    assert!(r.authenticate(&fresh, &dn(ALEX), NEW).is_ok());
    assert!(r.authenticate(&fresh, &dn(SAM), OLD).is_ok());
    assert!(r.authenticate(&fresh, &dn(SAM), NEW).is_err());
    assert_eq!(r.directory().source(), &before);
}
#[test]
fn optional_and_explicit_self_identity_and_old_password_are_legitimate_alternatives() {
    for explicit in [false, true] {
        for old in [None, Some(OLD)] {
            let mut r = realm_with(policy());
            let auth = r.authenticate(&connection(&r), &dn(ALEX), OLD).unwrap();
            let target = dn("UID=ALEX,OU=people,DC=EXAMPLE,DC=TEST");
            r.change_password(
                &auth,
                if explicit { Some(&target) } else { None },
                old,
                Some(NEW),
            )
            .unwrap();
            assert!(r.authenticate(&connection(&r), &dn(ALEX), NEW).is_ok());
        }
    }
}
#[test]
fn wrong_old_target_password_never_changes_credentials_even_with_a_valid_bound_identity() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    for old in ["wrong", NEW] {
        assert_eq!(
            r.change_password(&auth, None, Some(old), Some(NEW))
                .unwrap_err(),
            p::Error::OldPasswordMismatch
        );
        assert!(r.authenticate(&c, &dn(ALEX), OLD).is_ok());
        assert!(r.authenticate(&c, &dn(ALEX), NEW).is_err());
    }
}
#[test]
fn already_bound_identity_remains_distinct_from_fresh_login_after_rotation() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    r.change_password(&auth, None, None, Some(NEW)).unwrap();
    assert!(r.authenticate(&c, &dn(ALEX), OLD).is_err());
    assert!(r.authenticate(&c, &dn(ALEX), NEW).is_ok());
    assert_eq!(
        r.change_password(&auth, None, Some(OLD), Some("Another-fictional-83"))
            .unwrap_err(),
        p::Error::OldPasswordMismatch
    );
    r.change_password(&auth, None, Some(NEW), Some("Another-fictional-83"))
        .unwrap();
    assert!(
        r.authenticate(&c, &dn(ALEX), "Another-fictional-83")
            .is_ok()
    );
}
#[test]
fn server_support_and_self_service_policy_fail_without_mutating_credentials() {
    for (policy, error) in [
        (
            Policy {
                supports_password_modify: false,
                ..policy()
            },
            p::Error::ExtensionUnavailable,
        ),
        (
            Policy {
                allow_self_change: false,
                ..policy()
            },
            p::Error::PolicyDenied,
        ),
    ] {
        let mut r = realm_with(policy);
        let c = connection(&r);
        let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
        assert_eq!(
            r.change_password(&auth, None, None, Some(NEW)).unwrap_err(),
            error
        );
        assert!(r.authenticate(&c, &dn(ALEX), OLD).is_ok());
        assert!(r.authenticate(&c, &dn(ALEX), NEW).is_err());
    }
}
#[test]
fn required_old_password_is_a_supplied_policy_and_not_a_universal_assumption() {
    let mut r = realm_with(Policy {
        require_old_password: true,
        ..policy()
    });
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    assert_eq!(
        r.change_password(&auth, None, None, Some(NEW)).unwrap_err(),
        p::Error::OldPasswordRequired
    );
    assert!(r.authenticate(&c, &dn(ALEX), OLD).is_ok());
    r.change_password(&auth, None, Some(OLD), Some(NEW))
        .unwrap();
    assert!(r.authenticate(&c, &dn(ALEX), NEW).is_ok());
}
#[test]
fn ordinary_write_grants_do_not_make_unsupported_administrative_password_resets_succeed() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    for target in [SAM, "uid=absent,ou=People,dc=example,dc=test"] {
        assert_eq!(
            r.change_password(&auth, Some(&dn(target)), None, Some(NEW))
                .unwrap_err(),
            p::Error::OtherIdentityUnsupported
        );
        assert!(r.authenticate(&c, &dn(SAM), OLD).is_ok());
    }
    let read_only = r.authenticate(&c, &dn(SAM), OLD).unwrap();
    r.change_password(&read_only, None, None, Some(NEW))
        .unwrap();
    assert!(r.authenticate(&c, &dn(SAM), NEW).is_ok());
}
#[test]
fn plaintext_and_foreign_connections_are_rejected_before_credentials_are_evaluated() {
    let a = realm_with(policy());
    let b = realm_with(policy());
    let plain = a.open(URI, Request::Plain, &Trust::default(), 150).unwrap();
    assert_eq!(
        a.authenticate(&plain, &dn(ALEX), "").unwrap_err(),
        p::Error::Protection(t::Error::ProtectionRequired)
    );
    assert_eq!(
        b.authenticate(&connection(&a), &dn(ALEX), OLD).unwrap_err(),
        p::Error::Protection(t::Error::ForeignConnection)
    );
}
#[test]
fn a_protected_authenticated_context_cannot_be_replayed_into_a_new_attempt() {
    let a = realm_with(policy());
    let mut b = realm_with(policy());
    let auth = a.authenticate(&connection(&a), &dn(ALEX), OLD).unwrap();
    assert_eq!(
        b.change_password(&auth, None, None, Some(NEW)).unwrap_err(),
        p::Error::Protection(t::Error::ForeignConnection)
    );
    assert!(b.authenticate(&connection(&b), &dn(ALEX), OLD).is_ok());
}
#[test]
fn malformed_or_missing_replacement_and_old_values_are_rejected_before_mutation() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    assert_eq!(
        r.change_password(&auth, None, None, None).unwrap_err(),
        p::Error::NewPasswordRequired
    );
    for raw in [
        "".into(),
        "x".repeat(129),
        "two\nlines".into(),
        "ünsupported".into(),
    ] {
        assert!(r.change_password(&auth, None, None, Some(&raw)).is_err());
        assert!(
            r.change_password(&auth, None, Some(&raw), Some(NEW))
                .is_err()
        );
        assert!(r.authenticate(&c, &dn(ALEX), OLD).is_ok());
        assert!(r.authenticate(&c, &dn(ALEX), NEW).is_err());
    }
}
#[test]
fn new_credentials_are_exact_and_the_declared_maximum_is_accepted() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    let maximum = "A".repeat(i::MAX_SAMPLE_PASSWORD);
    r.change_password(&auth, None, None, Some(&maximum))
        .unwrap();
    assert!(r.authenticate(&c, &dn(ALEX), &maximum).is_ok());
    assert!(
        r.authenticate(&c, &dn(ALEX), &maximum.to_ascii_lowercase())
            .is_err()
    );
    assert!(
        r.authenticate(&c, &dn(ALEX), &format!("{maximum} "))
            .is_err()
    );
}
#[test]
fn setting_the_same_value_is_not_arbitrarily_called_a_successful_rotation() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    r.change_password(&auth, None, None, Some(OLD)).unwrap();
    assert!(r.authenticate(&c, &dn(ALEX), OLD).is_ok());
    assert!(r.authenticate(&c, &dn(ALEX), NEW).is_err());
    // A future rotation assignment must grade both fresh-login outcomes, not just
    // the operation result. This source policy has no password-history rejection.
}
#[test]
fn diagnostics_and_debug_do_not_echo_current_replacement_or_submitted_passwords() {
    let mut r = realm_with(policy());
    let c = connection(&r);
    let auth = r.authenticate(&c, &dn(ALEX), OLD).unwrap();
    r.change_password(&auth, None, None, Some(NEW)).unwrap();
    let error = r
        .change_password(
            &auth,
            None,
            Some("Private-test-only-84"),
            Some("Private-test-only-85"),
        )
        .unwrap_err();
    let debug = format!("{r:?} {auth:?} {error:?} {error}");
    for secret in [OLD, NEW, "Private-test-only-84", "Private-test-only-85"] {
        assert!(!debug.contains(secret), "redaction must hold");
    }
}
