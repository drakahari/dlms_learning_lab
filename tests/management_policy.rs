use dlms_learning_lab::{acl, management_policy as m};
use std::net::Ipv4Addr;

fn policy() -> m::Policy {
    m::Policy {
        version: m::VERSION,
        host_identity: Some("supplied-review-identity".into()),
        versions: m::Versions::Version2Only,
        lines: vec![m::Line {
            number: 0,
            ssh: true,
            telnet: false,
            source_list: None,
        }],
    }
}
fn attempt(protocol: m::Protocol) -> m::Attempt {
    m::Attempt {
        line: 0,
        protocol,
        source: Ipv4Addr::new(192, 0, 2, 17),
    }
}
fn rule(id: &str, action: acl::Action, base: [u8; 4], wildcard: [u8; 4]) -> acl::Rule {
    acl::Rule {
        id: id.into(),
        action,
        selector: acl::Selector::Standard {
            source: acl::Wildcard {
                base: Ipv4Addr::from(base),
                wildcard: Ipv4Addr::from(wildcard),
            },
        },
    }
}
fn list(rules: Vec<acl::Rule>) -> acl::AccessList {
    acl::AccessList {
        kind: acl::Kind::Standard,
        rules,
    }
}

#[test]
fn identity_version_and_line_transport_remain_independent_conditions() {
    let mut p = policy();
    assert!(
        p.assess(attempt(m::Protocol::Ssh2))
            .unwrap()
            .allowed_to_authenticate()
    );
    p.host_identity = None;
    let r = p.assess(attempt(m::Protocol::Ssh2)).unwrap();
    assert!(r.protocol_allowed);
    assert_eq!(r.identity_available, Some(false));
    assert_eq!(r.version_allowed, Some(true));
    assert!(!r.allowed_to_authenticate());
    p.lines[0].ssh = false;
    p.host_identity = Some("different-supplied-name".into());
    let r = p.assess(attempt(m::Protocol::Ssh2)).unwrap();
    assert_eq!(r.identity_available, Some(true));
    assert!(!r.protocol_allowed);
    assert!(!r.allowed_to_authenticate());
}

#[test]
fn telnet_is_neither_disabled_by_missing_ssh_identity_nor_secured_by_version2() {
    let mut p = policy();
    p.host_identity = None;
    p.lines[0].telnet = true;
    let r = p.assess(attempt(m::Protocol::Telnet)).unwrap();
    assert_eq!((r.identity_available, r.version_allowed), (None, None));
    assert!(r.allowed_to_authenticate());
    p.lines[0].telnet = false;
    assert!(
        !p.assess(attempt(m::Protocol::Telnet))
            .unwrap()
            .allowed_to_authenticate()
    );
}

#[test]
fn ssh_selection_alone_does_not_exclude_version1_compatibility() {
    let mut p = policy();
    assert!(
        !p.assess(attempt(m::Protocol::Ssh1))
            .unwrap()
            .allowed_to_authenticate()
    );
    p.versions = m::Versions::Compatibility;
    assert!(
        p.assess(attempt(m::Protocol::Ssh1))
            .unwrap()
            .allowed_to_authenticate()
    );
    assert!(
        p.assess(attempt(m::Protocol::Ssh2))
            .unwrap()
            .allowed_to_authenticate()
    );
    p.lines[0].ssh = false;
    assert!(
        !p.assess(attempt(m::Protocol::Ssh1))
            .unwrap()
            .allowed_to_authenticate()
    );
}

#[test]
fn each_supplied_line_is_independent_and_unknown_line_is_an_error_not_a_denial() {
    let mut p = policy();
    p.lines.push(m::Line {
        number: 9,
        ssh: false,
        telnet: true,
        source_list: None,
    });
    let mut a = attempt(m::Protocol::Ssh2);
    assert!(p.assess(a).unwrap().allowed_to_authenticate());
    a.line = 9;
    assert!(!p.assess(a).unwrap().allowed_to_authenticate());
    a.protocol = m::Protocol::Telnet;
    assert!(p.assess(a).unwrap().allowed_to_authenticate());
    a.line = 1;
    assert_eq!(p.assess(a).unwrap_err().kind, m::ErrorKind::Invalid);
}

#[test]
fn source_lists_use_real_ordered_standard_matching_and_implicit_deny() {
    let mut p = policy();
    p.lines[0].source_list = Some(list(vec![
        rule("exception", acl::Action::Deny, [192, 0, 2, 17], [0; 4]),
        rule(
            "remaining",
            acl::Action::Permit,
            [192, 0, 2, 0],
            [0, 0, 0, 255],
        ),
    ]));
    let a = attempt(m::Protocol::Ssh2);
    let r = p.assess(a).unwrap();
    assert!(!r.allowed_to_authenticate());
    assert!(
        matches!(r.source_decision.unwrap().cause, acl::Cause::Matched { rule, row: 0 } if rule == "exception")
    );
    let allowed = m::Attempt {
        source: Ipv4Addr::new(192, 0, 2, 18),
        ..a
    };
    assert!(p.assess(allowed).unwrap().allowed_to_authenticate());
    let outside = m::Attempt {
        source: Ipv4Addr::new(198, 51, 100, 17),
        ..a
    };
    let r = p.assess(outside).unwrap();
    assert!(!r.allowed_to_authenticate());
    assert_eq!(r.source_decision.unwrap().cause, acl::Cause::ImplicitDeny);
    p.lines[0].source_list.as_mut().unwrap().rules.swap(0, 1);
    assert!(p.assess(a).unwrap().allowed_to_authenticate());
}

#[test]
fn equivalent_supported_source_rule_decompositions_produce_identical_outcomes() {
    let mut whole = policy();
    whole.lines[0].source_list = Some(list(vec![rule(
        "all",
        acl::Action::Permit,
        [192, 0, 2, 0],
        [0, 0, 0, 255],
    )]));
    let mut split = whole.clone();
    split.lines[0].source_list = Some(list(vec![
        rule("lower", acl::Action::Permit, [192, 0, 2, 0], [0, 0, 0, 127]),
        rule(
            "upper",
            acl::Action::Permit,
            [192, 0, 2, 128],
            [0, 0, 0, 127],
        ),
    ]));
    for third in [1, 2, 3] {
        for last in 0..=255 {
            let a = m::Attempt {
                source: Ipv4Addr::new(192, 0, third, last),
                ..attempt(m::Protocol::Ssh2)
            };
            let expected = third == 2;
            assert_eq!(whole.assess(a).unwrap().allowed_to_authenticate(), expected);
            assert_eq!(split.assess(a).unwrap().allowed_to_authenticate(), expected);
        }
    }
}

#[test]
fn malformed_and_unsupported_lists_cannot_satisfy_negative_policy_cases() {
    let mut p = policy();
    p.lines[0].source_list = Some(list(vec![]));
    assert_eq!(
        p.assess(attempt(m::Protocol::Ssh2)).unwrap_err().kind,
        m::ErrorKind::Unsupported
    );
    p.lines[0].source_list = Some(acl::AccessList {
        kind: acl::Kind::Extended,
        rules: vec![],
    });
    assert_eq!(
        p.assess(attempt(m::Protocol::Ssh2)).unwrap_err().kind,
        m::ErrorKind::Unsupported
    );
    let one = rule("same", acl::Action::Permit, [192, 0, 2, 0], [0, 0, 0, 255]);
    p.lines[0].source_list = Some(list(vec![one.clone(), one]));
    assert_eq!(
        p.assess(attempt(m::Protocol::Ssh2)).unwrap_err().kind,
        m::ErrorKind::Invalid
    );
}

#[test]
fn replacement_is_atomic_and_reassessment_reflects_broken_policy() {
    let mut p = policy();
    let original = p.clone();
    let mut invalid = p.clone();
    invalid.lines.push(invalid.lines[0].clone());
    assert_eq!(p.replace(invalid).unwrap_err().kind, m::ErrorKind::Invalid);
    assert_eq!(p, original);
    assert!(
        p.assess(attempt(m::Protocol::Ssh2))
            .unwrap()
            .allowed_to_authenticate()
    );
    let mut broken = p.clone();
    broken.lines[0].ssh = false;
    p.replace(broken).unwrap();
    assert!(
        !p.assess(attempt(m::Protocol::Ssh2))
            .unwrap()
            .allowed_to_authenticate()
    );
    p.replace(original).unwrap();
    assert!(
        p.assess(attempt(m::Protocol::Ssh2))
            .unwrap()
            .allowed_to_authenticate()
    );
}

#[test]
fn unknown_schema_fields_protocols_and_oversize_input_fail_before_use() {
    assert!(serde_json::from_str::<m::Protocol>("\"rlogin\"").is_err());
    let raw = serde_json::to_vec(&policy()).unwrap();
    assert_eq!(m::Policy::decode(&raw).unwrap(), policy());
    for raw in [
        b"{}".as_slice(),
        b"not JSON",
        br#"{"version":1,"version":1}"#,
    ] {
        assert_eq!(
            m::Policy::decode(raw).unwrap_err().kind,
            m::ErrorKind::Invalid
        );
    }
    let mut v = serde_json::to_value(policy()).unwrap();
    v["host_path"] = "/etc/ssh/ssh_host_rsa_key".into();
    assert_eq!(
        m::Policy::decode(&serde_json::to_vec(&v).unwrap())
            .unwrap_err()
            .kind,
        m::ErrorKind::Invalid
    );
    v.as_object_mut().unwrap().remove("host_path");
    v["version"] = 2.into();
    assert_eq!(
        m::Policy::decode(&serde_json::to_vec(&v).unwrap())
            .unwrap_err()
            .kind,
        m::ErrorKind::Unsupported
    );
    assert_eq!(
        m::Policy::decode(&vec![b' '; m::MAX_JSON_BYTES + 1])
            .unwrap_err()
            .kind,
        m::ErrorKind::Limit
    );
}

#[test]
fn maximum_valid_typed_policy_roundtrips_with_bounded_json_and_larger_counts_fail() {
    let mut p = policy();
    p.host_identity = Some("a".repeat(64));
    let rules = (0..64)
        .map(|i| {
            rule(
                &format!("rule-{i:025}"),
                acl::Action::Permit,
                [192, 0, 2, i],
                [0; 4],
            )
        })
        .collect::<Vec<_>>();
    p.lines = (0..16)
        .map(|number| m::Line {
            number,
            ssh: true,
            telnet: false,
            source_list: (number < 2).then(|| list(rules.clone())),
        })
        .collect();
    p.lines[15].number = 98;
    p.validate().unwrap();
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(raw.len() <= m::MAX_JSON_BYTES);
    assert_eq!(m::Policy::decode(&raw).unwrap(), p);
    let mut invalid = p.clone();
    invalid.lines[2].source_list = Some(list(vec![rules[0].clone()]));
    assert_eq!(invalid.validate().unwrap_err().kind, m::ErrorKind::Limit);
    let mut invalid = p.clone();
    invalid.lines[15].number = 99;
    assert_eq!(invalid.validate().unwrap_err().kind, m::ErrorKind::Invalid);
    p.lines.push(m::Line {
        number: 97,
        ssh: true,
        telnet: false,
        source_list: None,
    });
    assert_eq!(p.validate().unwrap_err().kind, m::ErrorKind::Limit);
}

#[test]
fn identities_cannot_be_host_paths_and_unsupported_source_addresses_are_errors() {
    let mut p = policy();
    for identity in [
        "",
        "../../identity",
        "/etc/ssh/host",
        "$(whoami)",
        "bad\nname",
    ] {
        p.host_identity = Some(identity.into());
        assert_eq!(p.validate().unwrap_err().kind, m::ErrorKind::Invalid);
    }
    p.host_identity = None;
    for ip in [
        Ipv4Addr::UNSPECIFIED,
        Ipv4Addr::BROADCAST,
        Ipv4Addr::new(224, 0, 0, 1),
    ] {
        assert_eq!(
            p.assess(m::Attempt {
                source: ip,
                ..attempt(m::Protocol::Ssh2)
            })
            .unwrap_err()
            .kind,
            m::ErrorKind::Unsupported
        );
    }
    assert_eq!(p.lines, policy().lines);
}
