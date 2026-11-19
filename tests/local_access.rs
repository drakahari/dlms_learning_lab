use dlms_learning_lab::{local_access as local, management_policy as incoming};
#[path = "support/management_fixture.rs"]
mod fixture;
use fixture::{ip, network, settings};
fn user(name: &str, token: &str, privilege: local::Privilege) -> local::User {
    local::User {
        name: name.into(),
        training_token: token.into(),
        privilege,
    }
}
fn policy() -> local::Policy {
    local::Policy {
        version: 1,
        method: local::Method::LocalCase,
        exec_authorization: local::Authorization::Local,
        users: vec![
            user(
                "Admin",
                "Practice-Admin-42",
                local::Privilege::Administrator,
            ),
            user("Operator", "Practice-Operator-93", local::Privilege::User),
        ],
    }
}
fn probe(protocol: incoming::Protocol) -> incoming::Attempt {
    incoming::Attempt {
        line: 0,
        source: ip("192.0.2.10"),
        protocol,
    }
}
#[test]
fn actual_named_records_determine_identity_and_privilege_without_an_exact_script() {
    let p = policy();
    for u in &p.users {
        assert_eq!(
            p.authenticate(&u.name, &u.training_token).unwrap(),
            local::Decision::Authenticated {
                user: u.name.clone(),
                privilege: u.privilege
            }
        );
    }
    let mut reverse = p.clone();
    reverse.users.reverse();
    for u in &p.users {
        assert_eq!(
            p.authenticate(&u.name, &u.training_token),
            reverse.authenticate(&u.name, &u.training_token)
        );
    }
    assert_eq!(local::Privilege::User.level(), 1);
    assert_eq!(local::Privilege::Administrator.level(), 15);
}
#[test]
fn explicit_local_case_distinguishes_name_case_token_case_and_unknown_identity() {
    let p = policy();
    assert_eq!(
        p.authenticate("admin", "Practice-Admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::UnknownUser)
    );
    assert_eq!(
        p.authenticate("Admin", "practice-admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::WrongToken)
    );
    assert_eq!(
        p.authenticate("Missing", "Practice-Admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::UnknownUser)
    );
    let mut separate = p.clone();
    separate
        .users
        .push(user("admin", "Practice-Other-51", local::Privilege::User));
    separate.validate().unwrap();
    assert_eq!(
        separate.authenticate("admin", "Practice-Other-51").unwrap(),
        local::Decision::Authenticated {
            user: "admin".into(),
            privilege: local::Privilege::User
        }
    );
    assert_eq!(
        separate.authenticate("admin", "Practice-Admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::WrongToken)
    );
}
#[test]
fn token_spaces_are_preserved_and_empty_submission_is_not_a_bypass() {
    let mut p = policy();
    p.users[0].training_token = " Training words ".into();
    assert!(matches!(
        p.authenticate("Admin", " Training words ").unwrap(),
        local::Decision::Authenticated { .. }
    ));
    for wrong in ["Training words", "", " Training words"] {
        assert_eq!(
            p.authenticate("Admin", wrong).unwrap(),
            local::Decision::Rejected(local::Rejection::WrongToken)
        );
    }
    assert!(
        p.authenticate("", " Training words ")
            .unwrap_err()
            .contains("account name")
    );
}
#[test]
fn replacing_tokens_or_privileges_immediately_changes_outcomes_without_cached_success() {
    let mut p = policy();
    assert!(matches!(
        p.authenticate("Admin", "Practice-Admin-42").unwrap(),
        local::Decision::Authenticated { .. }
    ));
    let mut users = p.users.clone();
    users[0].training_token = "Practice-Replaced-18".into();
    users[0].privilege = local::Privilege::User;
    p.replace_users(users).unwrap();
    assert_eq!(
        p.authenticate("Admin", "Practice-Admin-42").unwrap(),
        local::Decision::Rejected(local::Rejection::WrongToken)
    );
    assert_eq!(
        p.authenticate("Admin", "Practice-Replaced-18").unwrap(),
        local::Decision::Authenticated {
            user: "Admin".into(),
            privilege: local::Privilege::User
        }
    );
    p.replace_users(vec![]).unwrap();
    assert_eq!(
        p.authenticate("Admin", "Practice-Replaced-18").unwrap(),
        local::Decision::Rejected(local::Rejection::UnknownUser)
    );
}
#[test]
fn invalid_replacements_are_atomic_and_do_not_infer_no_password_defaults() {
    let mut p = policy();
    let before = p.clone();
    for users in [
        vec![user("Bad Name", "Training", local::Privilege::User)],
        vec![user("Admin", "", local::Privilege::User)],
        vec![before.users[0].clone(), before.users[0].clone()],
        vec![user("Admin", "Bad\nToken", local::Privilege::User)],
    ] {
        assert!(p.replace_users(users).is_err());
        assert_eq!(p, before);
    }
    let mut raw = serde_json::to_value(&p).unwrap();
    raw["users"][0]["training_token"] = serde_json::Value::Null;
    assert!(local::Policy::from_json(&raw.to_string()).is_err());
}
#[test]
fn malformed_unknown_or_unsupported_policy_fields_do_not_silently_authenticate() {
    let original = serde_json::to_value(policy()).unwrap();
    for (field, value) in [
        ("version", serde_json::json!(2)),
        ("method", serde_json::json!("local")),
        ("method", serde_json::json!("none")),
        ("exec_authorization", serde_json::json!("default")),
        ("authenticated", serde_json::json!(true)),
    ] {
        let mut v = original.clone();
        v[field] = value;
        assert!(local::Policy::from_json(&v.to_string()).is_err());
    }
    for level in [
        serde_json::json!(0),
        serde_json::json!(14),
        serde_json::json!("root"),
    ] {
        let mut v = original.clone();
        v["users"][0]["privilege"] = level;
        assert!(local::Policy::from_json(&v.to_string()).is_err());
    }
    assert!(local::Policy::from_json("{broken").is_err());
    assert!(local::Policy::from_json(&" ".repeat(local::MAX_POLICY_BYTES + 1)).is_err());
    assert_eq!(
        local::Policy::from_json(&original.to_string()).unwrap(),
        policy()
    );
}
#[test]
fn explicit_resource_bounds_reject_overlarge_input_before_evaluation() {
    let mut p = policy();
    p.users = (0..local::MAX_RECORDS)
        .map(|i| user(&format!("User{i}"), "Training", local::Privilege::User))
        .collect();
    p.validate().unwrap();
    p.users
        .push(user("Extra", "Training", local::Privilege::User));
    assert!(p.validate().is_err());
    assert!(local::Policy::from_json(&serde_json::to_string(&p).unwrap()).is_err());
    let mut p = policy();
    p.users[0].name = "A".repeat(32);
    p.users[0].training_token = "X".repeat(128);
    p.validate().unwrap();
    p.users[0].training_token.push('X');
    assert!(p.validate().is_err());
    assert!(local::Policy::from_json(&serde_json::to_string(&p).unwrap()).is_err());
    let p = policy();
    for (name, token) in [
        ("A".repeat(33), "Training".into()),
        ("Admin".into(), "X".repeat(129)),
        ("Admin".into(), "Bad\0token".into()),
        ("Admin".into(), "Träining".into()),
    ] {
        assert!(p.authenticate(&name, &token).is_err());
    }
}
#[test]
fn permitted_actual_ip_path_reaches_local_check_and_retains_path_evidence() {
    let n = network();
    let in_ = settings();
    let p = policy();
    let result = local::inspect(
        &n,
        &in_,
        &p,
        probe(incoming::Protocol::Ssh2),
        "admin",
        "Admin",
        "Practice-Admin-42",
    )
    .unwrap();
    let local::Observation::Local { path, decision } = result else {
        panic!()
    };
    assert_eq!(path.source, ip("192.0.2.10"));
    assert_eq!(path.forward.destination_device, "R2");
    assert_eq!(path.reply.destination_device, "Client");
    assert_eq!(
        decision,
        local::Decision::Authenticated {
            user: "Admin".into(),
            privilege: local::Privilege::Administrator
        }
    );
    let result = local::inspect(
        &n,
        &in_,
        &p,
        probe(incoming::Protocol::Ssh2),
        "admin",
        "Admin",
        "wrong",
    )
    .unwrap();
    assert!(matches!(
        result,
        local::Observation::Local {
            decision: local::Decision::Rejected(local::Rejection::WrongToken),
            ..
        }
    ));
}
#[test]
fn protocol_version_source_and_identity_gates_are_not_reported_as_password_failures() {
    let n = network();
    let p = policy();
    let in_ = settings();
    for protocol in [incoming::Protocol::Ssh1, incoming::Protocol::Telnet] {
        assert!(matches!(
            local::inspect(&n, &in_, &p, probe(protocol), "admin", "Missing", "wrong").unwrap(),
            local::Observation::PolicyRejected { .. }
        ));
    }
    let guest = incoming::Attempt {
        source: ip("192.0.2.130"),
        ..probe(incoming::Protocol::Ssh2)
    };
    assert!(
        matches!(local::inspect(&n,&in_,&p,guest,"guest","Admin","Practice-Admin-42").unwrap(),local::Observation::PolicyRejected{conditions,..}if conditions.source_decision.as_ref().unwrap().action==dlms_learning_lab::acl::Action::Deny)
    );
    let mut in_ = settings();
    in_.host_identity = None;
    assert!(
        matches!(local::inspect(&n,&in_,&p,probe(incoming::Protocol::Ssh2),"admin","Admin","Practice-Admin-42").unwrap(),local::Observation::PolicyRejected{conditions,..}if conditions.identity_available==Some(false))
    );
}
#[test]
fn path_failure_cannot_satisfy_a_local_authentication_denial_and_source_cannot_be_spoofed() {
    let mut n = network();
    let p = policy();
    let in_ = settings();
    n.topology.devices.get_mut("R2").unwrap().routes.clear();
    assert!(
        matches!(local::inspect(&n,&in_,&p,probe(incoming::Protocol::Ssh2),"admin","Admin","wrong").unwrap(),local::Observation::PathUnavailable{reason}if reason.contains("reply"))
    );
    assert!(
        local::inspect(
            &network(),
            &in_,
            &p,
            probe(incoming::Protocol::Ssh2),
            "guest",
            "Admin",
            "Practice-Admin-42"
        )
        .unwrap_err()
        .contains("source")
    );
    let unknown = incoming::Attempt {
        line: 98,
        ..probe(incoming::Protocol::Ssh2)
    };
    assert!(
        local::inspect(
            &network(),
            &in_,
            &p,
            unknown,
            "admin",
            "Admin",
            "Practice-Admin-42"
        )
        .is_err()
    );
    assert!(
        local::inspect(
            &network(),
            &in_,
            &p,
            probe(incoming::Protocol::Ssh2),
            "unknown",
            "Admin",
            "Practice-Admin-42"
        )
        .is_err()
    );
}

#[test]
fn shell_looking_training_tokens_and_path_strings_remain_inert_comparison_data() {
    let mut p = policy();
    for token in [
        "$(not-a-command);../outside",
        "`not-a-command` > /outside",
        "quoted 'training' \"words\"",
    ] {
        let mut users = p.users.clone();
        users[0].training_token = token.into();
        p.replace_users(users).unwrap();
        assert_eq!(
            p.authenticate("Admin", token).unwrap(),
            local::Decision::Authenticated {
                user: "Admin".into(),
                privilege: local::Privilege::Administrator
            }
        );
        let restored = local::Policy::from_json(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(restored, p);
        assert_eq!(
            restored.authenticate("Admin", "not the token").unwrap(),
            local::Decision::Rejected(local::Rejection::WrongToken)
        );
    }
    assert!(p.authenticate("../outside", "Training").is_err());
}
