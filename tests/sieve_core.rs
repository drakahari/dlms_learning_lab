//! Focused Sieve syntax, filtering, failure and resource-bound checks.
use dlms_learning_lab::{
    sieve_eval::{Message, Outcome},
    sieve_lex::Kind,
    sieve_parse::parse,
};
use std::collections::BTreeSet;
fn message(subject: &str) -> Message {
    Message::new(
        vec![
            ("Subject".into(), subject.into()),
            ("From".into(), "ida@example.test".into()),
        ],
        "sample\n",
    )
    .unwrap()
}
fn run(script: &str, subject: &str) -> Outcome {
    parse(script).unwrap().evaluate(
        &message(subject),
        &["INBOX", "Notices", "Review"]
            .into_iter()
            .map(String::from)
            .collect(),
    )
}
fn boxes(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| String::from(*s)).collect()
}
#[test]
fn implicit_keep_stop_and_discard_have_distinct_effects() {
    for script in ["", "if false {discard;}", "stop; discard;"] {
        let r = run(script, "Hello");
        assert_eq!(r.mailboxes, boxes(&["INBOX"]));
        assert!(r.implicit_keep);
        assert!(!r.discarded);
        assert_eq!(r.fault, None);
    }
    let r = run("discard;", "Hello");
    assert!(r.discarded);
    assert!(!r.implicit_keep);
    assert!(r.mailboxes.is_empty());
    let r = run("keep; discard;", "Hello");
    assert!(!r.discarded);
    assert!(!r.implicit_keep);
    assert_eq!(r.mailboxes, boxes(&["INBOX"]));
}
#[test]
fn explicit_actions_accumulate_until_stop_and_same_mailbox_is_not_duplicated() {
    let r = run(
        "require \"fileinto\"; fileinto \"Notices\"; discard; fileinto \"Notices\"; keep; fileinto \"inbox\";",
        "Hi",
    );
    assert_eq!(r.mailboxes, boxes(&["INBOX", "Notices"]));
    assert_eq!(r.actions, 5);
    assert_eq!(r.fault, None);
    let r = run(
        "require \"fileinto\"; fileinto \"Notices\"; stop; fileinto \"Review\";",
        "Hi",
    );
    assert_eq!(r.mailboxes, boxes(&["Notices"]));
}
#[test]
fn elsif_selects_one_branch_while_separate_if_rules_can_both_file() {
    let shared =
        "require \"fileinto\"; if header :contains \"Subject\" \"notice\" {fileinto \"Notices\";}";
    let r = run(
        &format!("{shared} elsif header :contains \"Subject\" \"review\" {{fileinto \"Review\";}}"),
        "Notice: review",
    );
    assert_eq!(r.mailboxes, boxes(&["Notices"]));
    let r = run(
        &format!("{shared} if header :contains \"Subject\" \"review\" {{fileinto \"Review\";}}"),
        "Notice: review",
    );
    assert_eq!(r.mailboxes, boxes(&["Notices", "Review"]));
}
#[test]
fn header_tests_distinguish_missing_empty_trimmed_and_duplicate_values() {
    let mail = Message::new(
        vec![
            ("Subject".into(), "  HELLO\t".into()),
            ("Subject".into(), "Later".into()),
            ("X-Empty".into(), "\t".into()),
        ],
        "",
    )
    .unwrap();
    let allowed = boxes(&["INBOX"]);
    for test in [
        "header :is \"sUbJeCt\" \"hello\"",
        "header :is \"Subject\" \"Later\"",
        "header :is \"X-Empty\" \"\"",
        "header :contains \"X-Empty\" \"\"",
        "not header :contains \"X-Missing\" \"\"",
        "not exists [\"Subject\",\"X-Missing\"]",
        "not exists \"Subject:\"",
    ] {
        let out = parse(&format!("if {test} {{discard;}}"))
            .unwrap()
            .evaluate(&mail, &allowed);
        assert!(out.discarded, "{test}: {out:?}");
        assert_eq!(out.fault, None);
    }
}
#[test]
fn comparators_globs_quoted_escapes_and_tag_order_are_evaluated() {
    for test in [
        r#"header :matches "Subject" "no?ice*""#,
        r#"header :comparator "i;ascii-casemap" :is "subject" "NOTICE""#,
    ] {
        assert!(run(&format!("if {test} {{discard;}}"), "notice").discarded);
    }
    assert!(
        !run(
            r#"if header :comparator "i;octet" :is "Subject" "NOTICE" {discard;}"#,
            "notice"
        )
        .discarded
    );
    assert!(run(r#"if header :matches "Subject" "\\*" {discard;}"#, "*").discarded);
    assert!(run(r#"if header :is "Subject" "\n" {discard;}"#, "n").discarded);
    assert!(
        !run(
            r#"if header :matches "Subject" "no?ice" {discard;}"#,
            "notices"
        )
        .discarded
    );
}
#[test]
fn size_uses_canonical_full_message_and_strict_boundaries() {
    let mail = Message::new(vec![("Subject".into(), "Hi".into())], "x").unwrap();
    assert_eq!(mail.octets(), 16);
    for (test, matched) in [
        ("size :over 16", false),
        ("size :under 16", false),
        ("size :over 15", true),
        ("size :under 17", true),
        ("size :under 1K", true),
    ] {
        let r = parse(&format!("if {test} {{discard;}}"))
            .unwrap()
            .evaluate(&mail, &boxes(&["INBOX"]));
        assert_eq!(r.discarded, matched, "{test}");
    }
    let lf = Message::new(vec![], "a\nb\n").unwrap();
    let crlf = Message::new(vec![], "a\r\nb\r\n").unwrap();
    assert_eq!(lf.octets(), 8);
    assert_eq!(lf.octets(), crlf.octets());
}
#[test]
fn requires_and_unreachable_branches_are_checked_before_any_actions() {
    for script in [
        "fileinto \"Notices\";",
        "keep; require \"fileinto\";",
        "if false {require \"fileinto\";}",
        "if true {keep;} else if false {discard;}",
    ] {
        assert_eq!(parse(script).unwrap_err().kind, Kind::Syntax, "{script}");
    }
    for script in [
        "require \"FILEINTO\";",
        "require \"vacation\";",
        "require \"fileinto\"; fileinto :copy \"Notices\";",
        "if false {redirect \"other@example.test\";}",
        "if address :is \"From\" \"a@b.test\" {keep;}",
        "if header :regex \"Subject\" \".*\" {discard;}",
    ] {
        assert_eq!(
            parse(script).unwrap_err().kind,
            Kind::Unsupported,
            "{script}"
        );
    }
}
#[test]
fn syntax_errors_and_unsupported_shell_input_are_not_canned_success() {
    for script in [
        "keep",
        "if true {keep;",
        "if true keep;",
        "discard;;",
        "if header :is :contains \"Subject\" \"x\" {keep;}",
        "/* unfinished",
        "if true {fileinto \"unfinished}",
        "if size :over -1 {keep;}",
    ] {
        assert_eq!(parse(script).unwrap_err().kind, Kind::Syntax, "{script}");
    }
    for script in ["$(touch /tmp/x)", "keep | cat", "text:\nhello\n.\n"] {
        assert_eq!(
            parse(script).unwrap_err().kind,
            Kind::Unsupported,
            "{script}"
        );
    }
    assert!(
        run(
            r#"if header :is "Subject" "$(touch /tmp/x)" {discard;}"#,
            "$(touch /tmp/x)"
        )
        .discarded
    );
}
#[test]
fn runtime_errors_are_atomic_and_explain_recovery_keep() {
    for name in ["Missing", "../private", "/etc/passwd"] {
        let r = run(
            &format!("require \"fileinto\"; fileinto \"Notices\"; fileinto \"{name}\";"),
            "Hi",
        );
        assert_eq!(r.mailboxes, boxes(&["INBOX"]));
        assert_eq!(r.fault.as_ref().unwrap().kind, Kind::Runtime);
        assert!(r.implicit_keep);
        assert!(!r.discarded);
    }
}
#[test]
fn bounded_source_tokens_nesting_actions_and_work_recover_without_host_io() {
    assert_eq!(
        parse(&" ".repeat(16 * 1024 + 1)).unwrap_err().kind,
        Kind::Limit
    );
    assert_eq!(parse(&"keep;".repeat(2100)).unwrap_err().kind, Kind::Limit);
    let nested = |n| format!("{}keep;{}", "if true {".repeat(n), "}".repeat(n));
    assert!(parse(&nested(15)).is_ok());
    assert_eq!(parse(&nested(17)).unwrap_err().kind, Kind::Limit);
    let r = run(&"keep;".repeat(33), "Hi");
    assert_eq!(r.fault.as_ref().unwrap().kind, Kind::Limit);
    assert_eq!(r.mailboxes, boxes(&["INBOX"]));
    let keys = (0..32)
        .map(|_| format!("\"{}\"", "y".repeat(1024)))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!("if header :contains \"Subject\" [{keys}] {{discard;}}");
    assert_eq!(parse(&script).unwrap_err().kind, Kind::Limit);
    let one = format!(
        "if header :contains \"Subject\" \"{}\" {{discard;}}",
        "y".repeat(1024)
    );
    let r = run(&format!("{one}{one}{one}"), &"x".repeat(1024));
    assert_eq!(r.fault.as_ref().unwrap().kind, Kind::Limit);
    assert!(r.implicit_keep);
}
