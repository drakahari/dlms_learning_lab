//! Synthetic source-grading fixtures, not authored or registered lessons.
#[path = "management_tasks.rs"]
pub mod management;
use dlms_learning_lab::{local_access::Privilege, local_access_workspace as w};
pub fn task(handover: bool) -> w::Task {
    let entry = management::task(true);
    let name = if handover { "NewAdmin" } else { "Admin" };
    let word = if handover {
        "Practice-New-68"
    } else {
        "Practice-Admin-42"
    };
    let aaa = "aaa new-model\naaa authentication login default local-case\naaa authorization exec default local\n";
    let records = format!(
        "username {name} privilege 15 secret {word}\nusername Operator privilege 1 secret Practice-Operator-93\n"
    );
    let reference = format!("{aaa}{records}{}", entry.reference);
    let starter = if handover {
        format!(
            "{aaa}username Former privilege 15 secret Practice-Former-51\nusername Operator privilege 15 secret Practice-Operator-93\n{}",
            entry.reference
        )
    } else {
        entry.reference.clone()
    };
    let case =
        |label: &str, client: &str, username: &str, training_token: &str, expected| w::Case {
            label: label.into(),
            client: client.into(),
            username: username.into(),
            training_token: training_token.into(),
            expected,
        };
    let cases = vec![
        case(
            "Administrator authenticates with privileged access",
            "admin",
            name,
            word,
            w::Expected::Authenticated {
                user: name.into(),
                privilege: Privilege::Administrator,
            },
        ),
        case(
            "Operator authenticates with user access",
            "operator",
            "Operator",
            "Practice-Operator-93",
            w::Expected::Authenticated {
                user: "Operator".into(),
                privilege: Privilege::User,
            },
        ),
        case(
            "Wrong administrator training word",
            "admin",
            name,
            "Wrong-Training",
            w::Expected::RejectedCredential,
        ),
        case(
            "Unknown local name",
            "operator",
            "Unknown",
            "Practice-Unknown-37",
            w::Expected::RejectedCredential,
        ),
        case(
            "Letter case remains significant",
            "admin",
            &name.to_lowercase(),
            word,
            w::Expected::RejectedCredential,
        ),
        case(
            "Retired or unapproved name",
            "admin",
            "Former",
            "Practice-Former-51",
            w::Expected::RejectedCredential,
        ),
    ];
    w::Task{version:1,id:if handover{"synthetic-local-handover"}else{"synthetic-local-records"}.into(),instructions:if handover{"Keep only NewAdmin and Operator with their supplied training words and required levels. Retire Former and preserve every incoming access case."}else{"Configure only Admin and Operator with their supplied training words, explicit local-case/local EXEC methods, required access levels and all incoming access requirements."}.into(),network:entry.network,supplied:entry.supplied,permitted_users:vec![name.into(),"Operator".into()],entry_cases:entry.cases,cases,starter,reference}
}
pub fn definition(handover: bool) -> w::Definition {
    let initial = task(handover);
    let raw = serde_json::to_string(&initial)
        .unwrap()
        .replace("192.0.2.", "172.18.40.")
        .replace("Practice-", "Sample-");
    let mut example: w::Task = serde_json::from_str(&raw).unwrap();
    example.id.push_str("-example");
    let entry_example = management::definition(true).example;
    example.network = entry_example.network;
    example.supplied = entry_example.supplied;
    example.entry_cases = entry_example.cases;
    example.starter = example.starter.replace("line vty 0 15", "line vty 0 2");
    example.reference = example.reference.replace("line vty 0 15", "line vty 0 2");
    w::Definition {
        version: 1,
        initial,
        example,
    }
}
