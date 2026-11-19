use dlms_learning_lab::ldap_protection::{
    self as p, Certificate, Endpoint, Error, PlainReason, Protection, Request, Trust,
};
const URI: &str = "ldap://directory.example.test";
fn certificate() -> Certificate {
    Certificate::new(&["directory.example.test"], "training-root", 100, 200, true).unwrap()
}
fn endpoint() -> Endpoint {
    Endpoint::new(URI, true, Some(certificate())).unwrap()
}
fn trust() -> Trust {
    Trust::new(&["training-root"]).unwrap()
}
#[test]
fn required_and_opportunistic_starttls_validate_supplied_facts_before_issuing_proof() {
    let e = endpoint();
    for request in [Request::RequireStartTls, Request::TryStartTls] {
        let c = e.open(URI, request, &trust(), 150).unwrap();
        assert_eq!(c.protection(), Protection::VerifiedConfidential);
        let proof = e.require_protected(&c).unwrap();
        e.accepts(&proof).unwrap();
        e.accepts(&proof.clone()).unwrap();
    }
}
#[test]
fn no_request_or_unavailable_optional_upgrade_cannot_supply_protected_authorization() {
    let e = Endpoint::new(URI, false, None).unwrap();
    for (request, reason) in [
        (Request::Plain, PlainReason::NotRequested),
        (Request::TryStartTls, PlainReason::StartTlsUnavailable),
    ] {
        let c = e.open(URI, request, &Trust::default(), 150).unwrap();
        assert_eq!(c.protection(), Protection::Plain(reason));
        assert!(matches!(
            e.require_protected(&c),
            Err(Error::ProtectionRequired)
        ));
    }
    assert!(matches!(
        e.open(URI, Request::RequireStartTls, &trust(), 150),
        Err(Error::StartTlsUnavailable)
    ));
    let e = endpoint();
    let plain = e.open(URI, Request::Plain, &trust(), 150).unwrap();
    assert!(matches!(
        e.require_protected(&plain),
        Err(Error::ProtectionRequired)
    ));
}
#[test]
fn certificate_failures_do_not_silently_downgrade_either_upgrade_request() {
    let cases = [
        (None, Error::MissingCertificate),
        (
            Some(
                Certificate::new(&["directory.example.test"], "other-root", 100, 200, true)
                    .unwrap(),
            ),
            Error::UntrustedIssuer,
        ),
        (
            Some(
                Certificate::new(&["wrong.example.test"], "training-root", 100, 200, true).unwrap(),
            ),
            Error::NameMismatch,
        ),
        (
            Some(
                Certificate::new(&["directory.example.test"], "training-root", 151, 200, true)
                    .unwrap(),
            ),
            Error::NotYetValid,
        ),
        (
            Some(
                Certificate::new(&["directory.example.test"], "training-root", 100, 149, true)
                    .unwrap(),
            ),
            Error::Expired,
        ),
        (
            Some(
                Certificate::new(
                    &["directory.example.test"],
                    "training-root",
                    100,
                    200,
                    false,
                )
                .unwrap(),
            ),
            Error::NoConfidentiality,
        ),
    ];
    for (cert, error) in cases {
        let e = Endpoint::new(URI, true, cert).unwrap();
        for request in [Request::TryStartTls, Request::RequireStartTls] {
            assert_eq!(e.open(URI, request, &trust(), 150).unwrap_err(), error);
        }
    }
}
#[test]
fn validity_uses_fixed_sample_time_with_inclusive_bounds_without_reading_host_clock() {
    let e = endpoint();
    for now in [100, 150, 200] {
        let c = e
            .open(URI, Request::RequireStartTls, &trust(), now)
            .unwrap();
        e.require_protected(&c).unwrap();
    }
    assert_eq!(
        e.open(URI, Request::RequireStartTls, &trust(), 99)
            .unwrap_err(),
        Error::NotYetValid
    );
    assert_eq!(
        e.open(URI, Request::RequireStartTls, &trust(), 201)
            .unwrap_err(),
        Error::Expired
    );
    let exact = Endpoint::new(
        URI,
        true,
        Some(Certificate::new(&["directory.example.test"], "training-root", 5, 5, true).unwrap()),
    )
    .unwrap();
    assert!(
        exact
            .open(URI, Request::RequireStartTls, &trust(), 5)
            .is_ok()
    );
}
#[test]
fn names_issuers_and_endpoints_have_separate_matching_rules_and_no_network_lookup() {
    let e = endpoint();
    let c = e
        .open(
            "ldap://DIRECTORY.EXAMPLE.TEST:389/",
            Request::RequireStartTls,
            &trust(),
            150,
        )
        .unwrap();
    e.require_protected(&c).unwrap();
    let e = Endpoint::new(
        "ldap://directory.example.test:1389",
        true,
        Some(
            Certificate::new(
                &["backup.example.test", "DIRECTORY.EXAMPLE.TEST"],
                "training-root",
                0,
                u64::MAX,
                true,
            )
            .unwrap(),
        ),
    )
    .unwrap();
    assert!(
        e.open(
            "ldap://directory.example.test:1389",
            Request::RequireStartTls,
            &trust(),
            u64::MAX
        )
        .is_ok()
    );
    for target in [
        "ldap://localhost",
        "ldap://other.example.test",
        "ldap://directory.example.test:389",
    ] {
        assert_eq!(
            e.open(target, Request::RequireStartTls, &trust(), 150)
                .unwrap_err(),
            Error::UnknownEndpoint
        );
    }
    assert_eq!(
        endpoint()
            .open(
                URI,
                Request::RequireStartTls,
                &Trust::new(&["other-root"]).unwrap(),
                150
            )
            .unwrap_err(),
        Error::UntrustedIssuer
    );
}
#[test]
fn a_connection_or_proof_cannot_be_replayed_into_an_identical_new_endpoint_attempt() {
    let a = endpoint();
    let b = endpoint();
    let c = a
        .open(URI, Request::RequireStartTls, &trust(), 150)
        .unwrap();
    let proof = a.require_protected(&c).unwrap();
    assert_eq!(
        b.require_protected(&c).unwrap_err(),
        Error::ForeignConnection
    );
    assert_eq!(b.accepts(&proof).unwrap_err(), Error::ForeignConnection);
    assert!(a.accepts(&proof).is_ok());
    drop(a);
    let replacement = endpoint();
    assert_eq!(
        replacement.accepts(&proof).unwrap_err(),
        Error::ForeignConnection
    );
}
#[test]
fn failed_new_attempts_do_not_create_success_or_mutate_old_connections() {
    let e = endpoint();
    let old = e
        .open(URI, Request::RequireStartTls, &trust(), 150)
        .unwrap();
    assert!(
        e.open(URI, Request::RequireStartTls, &Trust::default(), 150)
            .is_err()
    );
    let plain = e.open(URI, Request::Plain, &trust(), 150).unwrap();
    assert_eq!(
        plain.protection(),
        Protection::Plain(PlainReason::NotRequested)
    );
    assert!(e.require_protected(&plain).is_err());
    assert!(e.require_protected(&old).is_ok());
}
#[test]
fn unsupported_names_schemes_controls_and_oversize_sources_are_honest_diagnostics() {
    for name in [
        "*.example.test",
        "192.0.2.1",
        "::1",
        "directory.example.test:389",
        "diréctory.example.test",
        "bad..test",
        "-bad.test",
        "bad.test.",
        "dir/test",
        "dir@host",
        "dir\nname",
    ] {
        assert!(
            Certificate::new(&[name], "training-root", 0, 1, true).is_err(),
            "{name}"
        );
    }
    for target in [
        "ldaps://directory.example.test",
        "ldapi:///work/socket",
        "ldap://127.0.0.1",
        "ldap://[::1]",
        "ldap://host/path",
        "ldap://host?query",
        "ldap://host;echo",
    ] {
        assert!(
            Endpoint::new(target, true, Some(certificate())).is_err(),
            "{target}"
        );
    }
    assert_eq!(
        Certificate::new(&["a.test"; p::MAX_NAMES + 1], "training-root", 0, 1, true).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Trust::new(&["root"; p::MAX_ROOTS + 1]).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Certificate::new(&["a.test"], &"a".repeat(65), 0, 1, true).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Certificate::new(&[&"a".repeat(254)], "root", 0, 1, true).unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        Endpoint::new(&"a".repeat(257), true, None).unwrap_err(),
        Error::Limit
    );
}
#[test]
fn inconsistent_duplicate_and_empty_facts_fail_while_an_empty_trust_set_never_authorizes() {
    for roots in [
        vec!["root", "root"],
        vec![""],
        vec!["ROOT"],
        vec!["/etc/ssl/ca.pem"],
        vec!["root\n"],
    ] {
        assert!(Trust::new(&roots).is_err());
    }
    assert_eq!(
        Certificate::new(&[], "root", 0, 1, true).unwrap_err(),
        Error::InvalidFacts
    );
    assert_eq!(
        Certificate::new(&["a.test", "A.TEST"], "root", 0, 1, true).unwrap_err(),
        Error::InvalidFacts
    );
    assert_eq!(
        Certificate::new(&["a.test"], "root", 2, 1, true).unwrap_err(),
        Error::InvalidFacts
    );
    assert!(Certificate::new(&["a.test"], "", 0, 1, true).is_err());
    assert_eq!(
        endpoint()
            .open(URI, Request::RequireStartTls, &Trust::default(), 150)
            .unwrap_err(),
        Error::UntrustedIssuer
    );
}
#[test]
fn old_plain_ldap_parsers_do_not_gain_tls_flags_or_schemes_from_this_opt_in_model() {
    use dlms_learning_lab::{ldap_client as c, ldap_utility as u};
    assert!(c::uri_key("ldaps://directory.example.test").is_err());
    for command in [
        "ldapadd -x -W -ZZ -H ldap://directory.example.test -D uid=alex,dc=example,dc=test",
        "ldapdelete -x -W -ZZ -H ldap://directory.example.test -D uid=alex,dc=example,dc=test",
    ] {
        assert!(u::parse(command).is_err());
        assert!(u::parse_delete(command).is_err());
    }
}

#[test]
fn certificate_dns_name_bound_is_independent_of_the_shorter_uri_identifier_budget() {
    let certificate_name = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61),
    ]
    .join(".");
    assert_eq!(certificate_name.len(), 253);
    let cert = Certificate::new(
        &[&certificate_name, "directory.example.test"],
        "training-root",
        100,
        200,
        true,
    )
    .unwrap();
    let e = Endpoint::new(URI, true, Some(cert)).unwrap();
    assert!(e.open(URI, Request::RequireStartTls, &trust(), 150).is_ok());
    let host = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(57),
    ]
    .join(".");
    let target = format!("ldap://{host}");
    assert_eq!(target.len(), 256);
    let e = Endpoint::new(
        &target,
        true,
        Some(Certificate::new(&[&host], "training-root", 100, 200, true).unwrap()),
    )
    .unwrap();
    assert!(
        e.open(&target, Request::RequireStartTls, &trust(), 150)
            .is_ok()
    );
    assert_eq!(
        Endpoint::new(&format!("{target}:1"), true, None).unwrap_err(),
        Error::Limit
    );
}
