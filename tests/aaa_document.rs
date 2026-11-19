use dlms_learning_lab::{aaa_document as doc, aaa_methods as a, acl_editor::Category};
use std::collections::BTreeMap;
#[path = "support/management_fixture.rs"]
mod fixture;
const LIST: &str = "aaa authentication login default group Primary local-case";
fn reference() -> String {
    format!(
        "aaa new-model\n{LIST}\naaa authorization exec default local\nusername Admin privilege 15 secret Local-Admin-42\nusername Operator privilege 1 secret Local-Operator-93\nip ssh version 2\nline vty 0\n transport input ssh\n access-class 10 in\nexit\nip access-list standard 10\n permit host 192.0.2.10\nexit\n"
    )
}
fn services() -> a::Services {
    a::Services {
        version: 1,
        named: BTreeMap::from([(
            "Primary".into(),
            a::TrainingService {
                available: true,
                identities: vec![a::Identity {
                    name: "Admin".into(),
                    training_token: "Central-Admin-27".into(),
                }],
            },
        )]),
    }
}
fn parse(text: &str) -> Result<doc::Settings, dlms_learning_lab::acl_editor::Diagnostic> {
    doc::parse(&fixture::network(), &fixture::settings(), &services(), text)
}
#[test]
fn complete_document_parses_actual_method_order_and_separate_local_records() {
    let settings = parse(&reference()).unwrap();
    assert_eq!(
        settings.authentication.methods,
        vec![
            a::Method::TrainingService {
                name: "Primary".into()
            },
            a::Method::LocalCase
        ]
    );
    let result = a::authenticate(
        &settings.authentication,
        &services(),
        &settings.local,
        "Admin",
        "Central-Admin-27",
    )
    .unwrap();
    assert_eq!(
        result.outcome,
        a::Reply::Authenticated {
            user: "Admin".into()
        }
    );
    assert_eq!(
        a::authorize_local_exec(&settings.local, &result).unwrap(),
        a::ExecAuthorization::Authorized {
            user: "Admin".into(),
            privilege: dlms_learning_lab::local_access::Privilege::Administrator
        }
    );
    let result = a::authenticate(
        &settings.authentication,
        &services(),
        &settings.local,
        "Admin",
        "Local-Admin-42",
    )
    .unwrap();
    assert_eq!(
        result.outcome,
        a::Reply::Rejected(dlms_learning_lab::local_access::Rejection::WrongToken)
    );
    assert_eq!(result.steps.len(), 1);
}
#[test]
fn changing_list_order_changes_actual_evaluation_and_supported_spacing_is_accepted() {
    let text = reference().replace(
        LIST,
        "  AAA\tAUTHENTICATION LOGIN DEFAULT local-case GROUP Primary",
    );
    let settings = parse(&text).unwrap();
    assert_eq!(settings.authentication.methods[0], a::Method::LocalCase);
    let result = a::authenticate(
        &settings.authentication,
        &services(),
        &settings.local,
        "Admin",
        "Local-Admin-42",
    )
    .unwrap();
    assert!(matches!(result.outcome, a::Reply::Authenticated { .. }));
    assert_eq!(result.steps.len(), 1);
    assert!(
        parse(&reference().replace(LIST, "aaa authentication login default group Primary")).is_ok()
    );
    assert!(
        parse(&reference().replace(LIST, "aaa authentication login default local-case")).is_ok()
    );
}
#[test]
fn undefined_services_other_methods_protocols_and_bypass_fail_honestly() {
    for list in [
        "aaa authentication login default group Missing local-case",
        "aaa authentication login default none",
        "aaa authentication login default local",
        "aaa authentication login default enable",
        "aaa authentication login default group radius local-case",
    ] {
        let e = parse(&reference().replace(LIST, list)).unwrap_err();
        assert_eq!(e.category, Category::Unsupported);
        assert_eq!(e.line, Some(2));
    }
    let e = parse(&format!("radius server Primary\n{}", reference())).unwrap_err();
    assert_eq!(e.category, Category::Unsupported);
}
#[test]
fn missing_arguments_are_syntax_and_duplicates_are_not_incremental_overrides() {
    for list in [
        "aaa authentication login default",
        "aaa authentication login default group",
    ] {
        let e = parse(&reference().replace(LIST, list)).unwrap_err();
        assert_eq!(e.category, Category::Syntax);
        assert_eq!(e.line, Some(2));
    }
    let e = parse(&reference().replace(LIST, &format!("{LIST}\n{LIST}"))).unwrap_err();
    assert_eq!(e.category, Category::Unsupported);
    assert_eq!(e.line, Some(3));
    assert!(parse(&reference().replace(LIST, "")).is_err());
    let e = parse(&reference().replace(
        LIST,
        "aaa authentication login default local-case local-case",
    ))
    .unwrap_err();
    assert_eq!(e.category, Category::Unsupported);
}
#[test]
fn method_normalization_preserves_original_line_diagnostics_and_scope() {
    let e = parse(&reference().replace(
        "username Admin privilege 15 secret Local-Admin-42",
        "username Admin privilege 15 secret",
    ))
    .unwrap_err();
    assert_eq!(e.category, Category::Syntax);
    assert_eq!(e.line, Some(4));
    let raw = reference()
        .replace(&format!("{LIST}\n"), "")
        .replace("line vty 0\n", &format!("line vty 0\n{LIST}\n"));
    let e = parse(&raw).unwrap_err();
    assert_eq!(e.category, Category::Unsupported);
    let raw = reference().replace(&format!("{LIST}\n"), "").replace(
        "ip access-list standard 10\n",
        &format!("ip access-list standard 10\n{LIST}\n"),
    );
    assert_eq!(parse(&raw).unwrap_err().category, Category::Unsupported);
}
#[test]
fn original_control_characters_lengths_and_limits_are_not_hidden_by_normalization() {
    for text in [
        reference().replace(LIST, &format!("{LIST}\0")),
        reference().replace(LIST, &format!("{LIST}\u{7f}")),
    ] {
        assert_eq!(parse(&text).unwrap_err().category, Category::Unsupported);
    }
    assert_eq!(
        parse(&"x".repeat(65537)).unwrap_err().category,
        Category::Limit
    );
    assert_eq!(
        parse(&reference().replace(LIST, &format!("{LIST}{}", " ".repeat(513))))
            .unwrap_err()
            .category,
        Category::Limit
    );
    assert_eq!(parse(&reference().replace(LIST,"aaa authentication login default group Primary group Primary group Primary group Primary local-case")).unwrap_err().category,Category::Limit);
    assert_eq!(
        parse(&format!("{}{}", "\n".repeat(1025), reference()))
            .unwrap_err()
            .category,
        Category::Limit
    );
}
#[test]
fn settings_comments_line_endings_and_input_rejection_leave_supplied_state_unchanged() {
    let network = fixture::network();
    let supplied = fixture::settings();
    let service = services();
    let pin = serde_json::to_value((&network, &supplied, &service)).unwrap();
    let text = format!("! supplied example only\n{}", reference()).replace('\n', "\r\n");
    assert!(doc::parse(&network, &supplied, &service, &text).is_ok());
    assert!(doc::parse(&network, &supplied, &service, "username bad").is_err());
    assert_eq!(
        serde_json::to_value((&network, &supplied, &service)).unwrap(),
        pin
    );
}
