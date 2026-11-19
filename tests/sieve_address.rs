use dlms_learning_lab::{sieve_address::mailboxes, sieve_lex::Kind};
#[test]
fn parses_mailbox_lists_without_treating_display_names_or_comments_as_addresses() {
    let v = mailboxes(r#""security@trusted.test" <visitor@outside.test>, Agent (trusted.test) <ops@trusted.test>"#).unwrap();
    assert_eq!(
        v.iter().map(|a| a.all()).collect::<Vec<_>>(),
        ["visitor@outside.test", "ops@trusted.test"]
    );
    assert_eq!(
        mailboxes("ops@trusted.test (not@elsewhere.test)").unwrap()[0].domain,
        "trusted.test"
    );
    assert_eq!(
        mailboxes("Ops@TRUSTED.TEST").unwrap()[0].all(),
        "Ops@TRUSTED.TEST"
    );
}

#[test]
fn component_edge_comments_and_spaces_are_preserved_without_joining_words() {
    for value in [
        "Ops (sender) @ (domain) Example.Test",
        "Ada < Ops (sender) @ (domain) Example.Test >",
        "(before) Ops\t@\tExample.Test (after)",
    ] {
        let addresses = mailboxes(value).unwrap();
        assert_eq!(addresses.len(), 1, "{value}");
        assert_eq!(addresses[0].all(), "Ops@Example.Test");
    }
    assert!(mailboxes("op(comment)s@Example.Test").unwrap().is_empty());
    assert!(matches(
        r#"address :domain "From" "example.test""#,
        &[("From", "Ops (sender) @ (domain) Example.Test")]
    ));
    for value in [
        "op . s@Example.Test",
        "ops@Example . Test",
        "op.\"s\"@Example.Test",
    ] {
        assert_eq!(
            mailboxes(value).unwrap_err().kind,
            Kind::Unsupported,
            "{value}"
        );
    }
}

#[test]
fn an_at_sign_only_inside_a_quoted_phrase_cannot_create_a_mailbox() {
    for value in [r#""ops@trusted.test""#, r#""Not an address""#, "a@b@c"] {
        assert!(mailboxes(value).unwrap().is_empty(), "{value}");
        assert!(!matches(
            r#"address :domain "From" "trusted.test""#,
            &[("From", value)]
        ));
    }
    assert_eq!(
        mailboxes(r#""first@last"@trusted.test"#).unwrap_err().kind,
        Kind::Unsupported
    );
}
#[test]
fn invalid_mailboxes_never_turn_into_substring_matches() {
    for raw in [
        "",
        "no-at",
        "a@@trusted.test",
        ".a@trusted.test",
        "a..b@trusted.test",
        "a@.trusted.test",
        "a@trusted.test.",
        "a b@trusted.test",
        "Name ops@trusted.test",
        "<a@trusted.test",
        "a@trusted.test>",
        "A <a@trusted.test> trailing",
        "a(comment)b@trusted.test",
    ] {
        assert!(mailboxes(raw).unwrap().is_empty(), "{raw}");
    }
    let v = mailboxes("broken-address, correct@trusted.test").unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].all(), "correct@trusted.test");
}
#[test]
fn unsupported_valid_address_forms_are_diagnosed_instead_of_approximated() {
    for raw in [
        r#""first last"@trusted.test"#,
        "Team: ops@trusted.test;",
        "ops@[127.0.0.1]",
        "<@old.test:ops@trusted.test>",
        "nonascii@é.test",
    ] {
        assert_eq!(mailboxes(raw).unwrap_err().kind, Kind::Unsupported, "{raw}");
    }
}
#[test]
fn address_input_and_comment_depth_and_list_limits_are_bounded() {
    assert_eq!(mailboxes(&"x".repeat(1025)).unwrap_err().kind, Kind::Limit);
    let nested = format!("{}x{} ops@trusted.test", "(".repeat(17), ")".repeat(17));
    assert_eq!(mailboxes(&nested).unwrap_err().kind, Kind::Limit);
    let many = ["a@b"; 33].join(",");
    assert_eq!(mailboxes(&many).unwrap_err().kind, Kind::Limit);
    assert_eq!(mailboxes(&["a@b"; 32].join(",")).unwrap().len(), 32);
}

use dlms_learning_lab::{
    sieve_eval::Message,
    sieve_parse::{parse, parse_address},
};
fn matches(test: &str, headers: &[(&str, &str)]) -> bool {
    let message = Message::new(
        headers
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        "sample",
    )
    .unwrap();
    let program = parse_address(&format!("if {test} {{ discard; }}")).unwrap();
    let result = program.evaluate(&message, &[String::from("INBOX")].into_iter().collect());
    assert_eq!(result.fault, None);
    result.discarded
}
#[test]
fn legacy_profile_refuses_address_tests_and_advanced_profile_validates_options() {
    let script = r#"if address :domain "From" "trusted.test" { keep; }"#;
    assert_eq!(parse(script).unwrap_err().kind, Kind::Unsupported);
    assert!(parse_address(script).is_ok());
    for script in [
        r#"if address :all :domain "From" "x" {keep;}"#,
        r#"if address :is :contains "From" "x" {keep;}"#,
    ] {
        assert_eq!(parse_address(script).unwrap_err().kind, Kind::Syntax);
    }
    assert_eq!(
        parse_address(r#"if address :user "From" "x" {keep;}"#)
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
}
#[test]
fn domain_filter_uses_parsed_domain_not_display_name_or_suffix_text() {
    let condition = r#"address :domain :is "From" "trusted.test""#;
    assert!(matches(condition, &[("From", "Agent <ops@TRUSTED.TEST>")]));
    for input in [
        r#""ops@trusted.test" <visitor@elsewhere.test>"#,
        "ops@trusted.test.evil.test",
        "ops@not-trusted.test",
        "ops@sub.trusted.test",
        "ops(comment @trusted.test)@elsewhere.test",
        "broken",
        "",
    ] {
        assert!(!matches(condition, &[("From", input)]), "{input}");
    }
    assert!(!matches(condition, &[("Subject", "ops@trusted.test")]));
    assert!(!matches(condition, &[]));
}
#[test]
fn localpart_all_comparator_and_header_lists_use_any_real_address() {
    assert!(matches(
        r#"address :localpart :is ["To","Cc"] "help""#,
        &[("Cc", "Other <a@outside.test>, Help <help@work.test>")]
    ));
    assert!(matches(
        r#"address "From" "help@work.test""#,
        &[("From", "HELP@WORK.TEST")]
    ));
    assert!(!matches(
        r#"address :comparator "i;octet" :all "From" "help@work.test""#,
        &[("From", "HELP@WORK.TEST")]
    ));
    assert!(!matches(
        r#"address :localpart "To" "help""#,
        &[("To", "help+alerts@work.test")]
    ));
    assert!(matches(
        r#"address :is "From" "help@work.test""#,
        &[("From", "other@outside.test"), ("From", "help@work.test")]
    ));
    assert!(!matches(
        r#"address :is "Subject" "help@work.test""#,
        &[("Subject", "help@work.test")]
    ));
    assert!(!matches(
        r#"address :contains "From:" """#,
        &[("From", "help@work.test")]
    ));
}
#[test]
fn unsupported_message_address_syntax_fails_atomically_without_success_credit() {
    let message =
        Message::new(vec![("From".into(), "Team: ops@trusted.test;".into())], "").unwrap();
    let result = parse_address(r#"discard; if address :domain "From" "trusted.test" {keep;}"#)
        .unwrap()
        .evaluate(&message, &[String::from("INBOX")].into_iter().collect());
    assert_eq!(result.fault.unwrap().kind, Kind::Unsupported);
    assert_eq!(
        result.mailboxes,
        [String::from("INBOX")].into_iter().collect()
    );
    assert!(!result.discarded);
}

use dlms_learning_lab::sieve_workspace::{Action, Definition, Scope, Session, Task};
fn address_task(key: &str) -> Task {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-filter-lessons-v1.json")).unwrap();
    let mut task: Task =
        serde_json::from_value(source["scenarios"][0]["goal"]["definition"]["assignment"].clone())
            .unwrap();
    task.address_tests = true;
    task.reference = format!(
        "require \"fileinto\"; if address :is :domain \"From\" \"{key}\" {{ fileinto \"News\"; }}"
    );
    let inputs = [
        format!("Alex <ops@{key}>"),
        format!("ops@{}", key.to_uppercase()),
        format!("\"ops@{key}\" <visitor@outside.test>"),
        format!("ops@{key}.outside.test"),
        format!("ops@dept.{key}"),
        String::from(""),
        String::from("not-an-address"),
        format!("other@outside.test, ops@{key}"),
    ];
    task.cases = inputs
        .into_iter()
        .enumerate()
        .map(|(i, value)| dlms_learning_lab::sieve_workspace::Case {
            id: format!("case-{i}"),
            label: format!("Address sample {i}"),
            headers: vec![("From".into(), value)],
            body: "sample".into(),
            envelope: None,
            expected_redirects: std::collections::BTreeSet::new(),
            expected_rejection: false,
            arrival_seconds: None,
            expected_reply: None,
            expected: [String::from(if matches!(i, 0 | 1 | 7) {
                "News"
            } else {
                "INBOX"
            })]
            .into_iter()
            .collect(),
        })
        .collect();
    task
}
#[test]
fn legacy_tasks_serialize_without_new_fields_and_address_profiles_run_all_cases() {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-filter-lessons-v1.json")).unwrap();
    let raw = source["scenarios"][0]["goal"]["definition"]["assignment"].clone();
    let legacy: Task = serde_json::from_value(raw.clone()).unwrap();
    assert!(!legacy.address_tests);
    assert_eq!(serde_json::to_value(legacy).unwrap(), raw);
    let task = address_task("trusted.test");
    task.validate().unwrap();
    assert!(task.complete(&task.reference));
    assert!(!task.complete(
        r#"require "fileinto"; if header :contains "From" "trusted.test" {fileinto "News";}"#
    ));
    assert!(!task.complete(r#"require "fileinto"; fileinto "News";"#));
    let mut scope = Scope::new(&task);
    scope.edit(task.reference.clone()).unwrap();
    scope.start_for(&task);
    for _ in 0..task.cases.len() {
        scope.tick(&task);
    }
    assert!(scope.checked);
    assert!(scope.results.iter().all(|r| r.passed));
    scope.bound(&task).unwrap();
    scope.edit(String::from("keep;")).unwrap();
    assert!(!scope.checked);
    assert!(scope.results.is_empty());
}
#[test]
fn source_bound_profile_preserves_two_drafts_and_partial_stop_resume() {
    let definition = Definition {
        schema: 1,
        source_id: "address-filter-fixture".into(),
        assignment: address_task("trusted.test"),
        example: address_task("guide.test"),
    };
    definition.validate().unwrap();
    let mut session = Session::new(&definition).unwrap();
    session
        .action(
            &definition,
            &Action::Edit {
                script: definition.assignment.reference.clone(),
            },
            false,
        )
        .unwrap();
    session.action(&definition, &Action::Run, false).unwrap();
    session.action(&definition, &Action::Tick, false).unwrap();
    session.action(&definition, &Action::Stop, false).unwrap();
    assert!(session.assignment.stopped);
    assert_eq!(session.assignment.results.len(), 1);
    assert_eq!(session.practice.draft, definition.example.starter);
    let mut restored: Session =
        serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
    restored.assignment.bound(&definition.assignment).unwrap();
    restored.action(&definition, &Action::Run, false).unwrap();
    for _ in 0..definition.assignment.cases.len() {
        restored.action(&definition, &Action::Tick, false).unwrap();
    }
    assert!(definition.checks(&restored).iter().all(|c| c.passed));
    restored
        .action(
            &definition,
            &Action::Edit {
                script: String::from("keep;"),
            },
            false,
        )
        .unwrap();
    assert!(definition.checks(&restored).iter().any(|c| !c.passed));
    let mut mismatch = definition.clone();
    mismatch.example.address_tests = false;
    assert!(mismatch.validate().is_err());
}

#[test]
fn all_three_legacy_sieve_goals_and_initial_scopes_keep_their_original_serialized_values() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-filter-lessons-v1.json")).unwrap();
    let current = dlms_learning_lab::pack::accepted470().unwrap();
    assert_eq!(current.len(), 470);
    for row in raw["scenarios"].as_array().unwrap() {
        let lab = current
            .iter()
            .find(|s| Some(s.id.as_str()) == row["id"].as_str())
            .unwrap();
        assert_eq!(serde_json::to_value(&lab.goal).unwrap(), row["goal"]);
        assert_eq!(serde_json::to_value(&lab.initial).unwrap(), row["initial"]);
    }
}
