use dlms_learning_lab::{aaa_methods as a, local_access as l};
use std::collections::BTreeMap;
fn local() -> l::Policy {
    l::Policy {
        version: 1,
        method: l::Method::LocalCase,
        exec_authorization: l::Authorization::Local,
        users: vec![l::User {
            name: "Operator".into(),
            training_token: "Local-Word-19".into(),
            privilege: l::Privilege::User,
        }],
    }
}
fn services() -> a::Services {
    a::Services {
        version: 1,
        named: BTreeMap::from([
            (
                "Primary".into(),
                a::TrainingService {
                    available: true,
                    identities: vec![
                        a::Identity {
                            name: "Operator".into(),
                            training_token: "Central-Word-27".into(),
                        },
                        a::Identity {
                            name: "CentralOnly".into(),
                            training_token: "Central-Word-31".into(),
                        },
                    ],
                },
            ),
            (
                "Secondary".into(),
                a::TrainingService {
                    available: false,
                    identities: vec![],
                },
            ),
        ]),
    }
}
fn policy() -> a::Policy {
    a::Policy {
        version: 1,
        methods: vec![
            a::Method::TrainingService {
                name: "Primary".into(),
            },
            a::Method::LocalCase,
        ],
    }
}
#[test]
fn an_explicit_denial_does_not_try_a_working_local_credential() {
    let result = a::authenticate(
        &policy(),
        &services(),
        &local(),
        "Operator",
        "Local-Word-19",
    )
    .unwrap();
    assert_eq!(result.outcome, a::Reply::Rejected(l::Rejection::WrongToken));
    assert_eq!(result.steps.len(), 1);
    assert!(matches!(&result.steps[0].method,a::Method::TrainingService{name} if name=="Primary"));
}
#[test]
fn unavailability_advances_to_local_but_local_rejection_is_terminal() {
    let mut s = services();
    s.named.get_mut("Primary").unwrap().available = false;
    for (token, expected) in [
        (
            "Local-Word-19",
            a::Reply::Authenticated {
                user: "Operator".into(),
            },
        ),
        (
            "Central-Word-27",
            a::Reply::Rejected(l::Rejection::WrongToken),
        ),
        ("", a::Reply::Rejected(l::Rejection::WrongToken)),
    ] {
        let result = a::authenticate(&policy(), &s, &local(), "Operator", token).unwrap();
        assert_eq!(result.steps.len(), 2);
        assert_eq!(result.steps[0].reply, a::Reply::Unavailable);
        assert_eq!(result.outcome, expected);
    }
}
#[test]
fn a_valid_primary_reply_stops_and_keeps_identity_separate_from_privilege() {
    let result = a::authenticate(
        &policy(),
        &services(),
        &local(),
        "Operator",
        "Central-Word-27",
    )
    .unwrap();
    assert_eq!(result.steps.len(), 1);
    assert_eq!(
        result.outcome,
        a::Reply::Authenticated {
            user: "Operator".into()
        }
    );
    assert_eq!(
        a::authorize_local_exec(&local(), &result).unwrap(),
        a::ExecAuthorization::Authorized {
            user: "Operator".into(),
            privilege: l::Privilege::User
        }
    );
    let mut higher = local();
    higher.users[0].privilege = l::Privilege::Administrator;
    assert_eq!(
        a::authorize_local_exec(&higher, &result).unwrap(),
        a::ExecAuthorization::Authorized {
            user: "Operator".into(),
            privilege: l::Privilege::Administrator
        }
    );
}
#[test]
fn authenticated_identity_missing_from_explicit_local_authorization_is_denied() {
    let result = a::authenticate(
        &policy(),
        &services(),
        &local(),
        "CentralOnly",
        "Central-Word-31",
    )
    .unwrap();
    assert_eq!(
        a::authorize_local_exec(&local(), &result).unwrap(),
        a::ExecAuthorization::MissingLocalRecord {
            user: "CentralOnly".into()
        }
    );
    let rejected = a::authenticate(&policy(), &services(), &local(), "Operator", "wrong").unwrap();
    assert_eq!(
        a::authorize_local_exec(&local(), &rejected).unwrap(),
        a::ExecAuthorization::NotAuthenticated
    );
}
#[test]
fn unknown_and_differently_cased_names_are_real_returned_rejections() {
    for name in ["Stranger", "operator"] {
        let r = a::authenticate(&policy(), &services(), &local(), name, "Central-Word-27").unwrap();
        assert_eq!(r.outcome, a::Reply::Rejected(l::Rejection::UnknownUser));
        assert_eq!(r.steps.len(), 1);
    }
}
#[test]
fn method_order_changes_which_actual_database_can_authenticate() {
    let mut p = policy();
    p.methods.reverse();
    for (token, expected) in [
        (
            "Local-Word-19",
            a::Reply::Authenticated {
                user: "Operator".into(),
            },
        ),
        (
            "Central-Word-27",
            a::Reply::Rejected(l::Rejection::WrongToken),
        ),
    ] {
        let r = a::authenticate(&p, &services(), &local(), "Operator", token).unwrap();
        assert_eq!(r.outcome, expected);
        assert_eq!(r.steps.len(), 1);
        assert_eq!(r.steps[0].method, a::Method::LocalCase);
    }
}
#[test]
fn exhaustion_is_unavailable_and_never_an_authenticated_or_denied_identity() {
    let mut s = services();
    s.named.get_mut("Primary").unwrap().available = false;
    let p = a::Policy {
        version: 1,
        methods: vec![
            a::Method::TrainingService {
                name: "Primary".into(),
            },
            a::Method::TrainingService {
                name: "Secondary".into(),
            },
        ],
    };
    let r = a::authenticate(&p, &s, &local(), "Operator", "Local-Word-19").unwrap();
    assert_eq!(r.outcome, a::Reply::Unavailable);
    assert_eq!(r.steps.len(), 2);
    assert!(r.steps.iter().all(|s| s.reply == a::Reply::Unavailable));
    assert_eq!(
        a::authorize_local_exec(&local(), &r).unwrap(),
        a::ExecAuthorization::NotAuthenticated
    );
}
#[test]
fn missing_or_duplicate_methods_are_configuration_errors_not_fallback() {
    for p in [
        a::Policy {
            version: 1,
            methods: vec![],
        },
        a::Policy {
            version: 2,
            ..policy()
        },
        a::Policy {
            version: 1,
            methods: vec![a::Method::LocalCase, a::Method::LocalCase],
        },
        a::Policy {
            version: 1,
            methods: vec![a::Method::TrainingService {
                name: "Undefined".into(),
            }],
        },
    ] {
        assert!(a::authenticate(&p, &services(), &local(), "Operator", "Local-Word-19").is_err());
    }
}
#[test]
fn malformed_unknown_and_oversized_policies_fail_honestly() {
    let s = services();
    for raw in [
        r#"{"version":1,"methods":[{"kind":"none"}]}"#.into(),
        r#"{"version":1,"methods":[{"kind":"local_case"}],"bypass":true}"#.into(),
        "x".repeat(a::MAX_INPUT_BYTES + 1),
    ] {
        assert!(a::Policy::from_json(&raw, &s).is_err());
    }
    let raw = serde_json::to_string(&policy()).unwrap();
    assert_eq!(a::Policy::from_json(&raw, &s).unwrap(), policy());
}
#[test]
fn service_directory_bounds_records_versions_and_unknown_fields_are_validated() {
    let mut s = services();
    let duplicate = s.named["Primary"].identities[0].clone();
    s.named
        .get_mut("Primary")
        .unwrap()
        .identities
        .push(duplicate);
    assert!(s.validate().is_err());
    let mut s = services();
    s.version = 2;
    assert!(s.validate().is_err());
    let mut s = services();
    s.named.get_mut("Primary").unwrap().identities[0]
        .training_token
        .clear();
    assert!(s.validate().is_err());
    let raw = serde_json::to_string(&services()).unwrap();
    assert_eq!(a::Services::from_json(&raw).unwrap(), services());
    assert!(
        a::Services::from_json(&raw.replacen("\"version\":1", "\"version\":1,\"host\":true", 1))
            .is_err()
    );
    assert!(a::Services::from_json(&"x".repeat(a::MAX_INPUT_BYTES + 1)).is_err());
}
#[test]
fn bad_attempts_are_diagnostics_and_never_truncated_or_executed() {
    for (name, token) in [
        ("../outside", "word"),
        ("Operator", "bad\nword"),
        ("Operator", "bad\0word"),
    ] {
        assert!(a::authenticate(&policy(), &services(), &local(), name, token).is_err());
    }
    assert!(
        a::authenticate(
            &policy(),
            &services(),
            &local(),
            "Operator",
            &"x".repeat(129)
        )
        .is_err()
    );
}

#[test]
fn duplicate_service_keys_and_excess_entries_fail_before_reading_their_bodies() {
    for raw in [
        r#"{"version":1,"named":{"One":{"available":false,"identities":[]},"One":null}}"#,
        r#"{"version":1,"named":{"One":{"available":false,"identities":[]},"Two":{"available":false,"identities":[]},"Three":{"available":false,"identities":[]},"Four":{"available":false,"identities":[]},"Five":null}}"#,
    ] {
        let e = serde_json::from_str::<a::Services>(raw)
            .unwrap_err()
            .to_string();
        assert!(e.contains("Too many or duplicate training services"), "{e}");
    }
    let raw = r#"{"version":1,"named":{"ThisNameIsFarLongerThanThe32CharacterLimit":null}}"#;
    assert!(a::Services::from_json(raw).is_err());
}
