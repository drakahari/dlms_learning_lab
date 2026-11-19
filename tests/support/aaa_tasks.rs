//! Synthetic tasks only; not registered or counted as authored curriculum.
#[path = "local_access_tasks.rs"]
pub mod local_fixture;
use dlms_learning_lab::{
    aaa_methods as a, aaa_workspace as w, local_access as l, local_access_workspace as old,
};
use std::collections::BTreeMap;
fn from_local(t: old::Task, role: bool) -> w::Task {
    let service = a::Method::TrainingService {
        name: "Primary".into(),
    };
    let reference = t.reference.replace(
        "aaa authentication login default local-case",
        "aaa authentication login default group Primary local-case",
    );
    let admin = t.cases[0].username.clone();
    let operator = t.cases[1].username.clone();
    let admin_word = t.cases[0].training_token.clone();
    let operator_word = t.cases[1].training_token.clone();
    let central_admin = format!("Central-{admin_word}");
    let central_operator = format!("Central-{operator_word}");
    let services = a::Services {
        version: 1,
        named: BTreeMap::from([(
            "Primary".into(),
            a::TrainingService {
                available: true,
                identities: vec![
                    a::Identity {
                        name: admin.clone(),
                        training_token: central_admin.clone(),
                    },
                    a::Identity {
                        name: operator.clone(),
                        training_token: central_operator.clone(),
                    },
                    a::Identity {
                        name: "CentralOnly".into(),
                        training_token: "Central-Only-91".into(),
                    },
                ],
            },
        )]),
    };
    let auth = |user: &str, p, via| w::Expected::Authenticated {
        user: user.into(),
        privilege: p,
        via,
    };
    let reject = || w::Expected::AuthenticationRejected {
        via: service.clone(),
    };
    let case = |label: &str, client: &str, user: &str, word: &str, available, expected| w::Case {
        label: label.into(),
        client: client.into(),
        username: user.into(),
        training_token: word.into(),
        service_states: vec![w::Availability {
            service: "Primary".into(),
            available,
        }],
        expected,
    };
    let admin_client = t.cases[0].client.clone();
    let operator_client = t.cases[1].client.clone();
    let cases = vec![
        case(
            "Central administrator identity and local privilege",
            &admin_client,
            &admin,
            &central_admin,
            true,
            auth(&admin, l::Privilege::Administrator, service.clone()),
        ),
        case(
            "Central operator identity and restricted local privilege",
            &operator_client,
            &operator,
            &central_operator,
            true,
            auth(&operator, l::Privilege::User, service.clone()),
        ),
        case(
            "Unavailable primary permits actual local fallback",
            &operator_client,
            &operator,
            &operator_word,
            false,
            auth(&operator, l::Privilege::User, a::Method::LocalCase),
        ),
        case(
            "A returned denial must not try the working local word",
            &admin_client,
            &admin,
            &admin_word,
            true,
            reject(),
        ),
        case(
            "Unknown identity is denied by the responding primary",
            &admin_client,
            "Stranger",
            "Unknown-91",
            true,
            reject(),
        ),
        case(
            "Authentication is separate from a missing local profile",
            &admin_client,
            "CentralOnly",
            "Central-Only-91",
            true,
            w::Expected::AuthorizationDenied {
                user: "CentralOnly".into(),
                via: service.clone(),
            },
        ),
        case(
            "Account letter case remains significant in the supplied directory",
            &operator_client,
            &admin.to_lowercase(),
            &central_admin,
            true,
            reject(),
        ),
        case(
            "Administrator local fallback retains actual authorization",
            &admin_client,
            &admin,
            &admin_word,
            false,
            auth(&admin, l::Privilege::Administrator, a::Method::LocalCase),
        ),
    ];
    let starter = if role {
        reference.replace(
            &format!("username {admin} privilege 15"),
            &format!("username {admin} privilege 1"),
        )
    } else {
        reference.replace("group Primary local-case", "local-case group Primary")
    };
    w::Task{version:1,id:format!("aaa-{}-{}",if role{"authorization"}else{"methods"},t.id),instructions:"Repair the supplied authentication methods/local profiles while retaining incoming policy and the required outcomes.".into(),network:t.network,supplied:t.supplied,services,permitted_users:t.permitted_users,entry_cases:t.entry_cases,cases,starter,reference}
}
pub fn definition(role: bool) -> w::Definition {
    let old = local_fixture::definition(false);
    w::Definition {
        version: 1,
        initial: from_local(old.initial, role),
        example: from_local(old.example, role),
    }
}
