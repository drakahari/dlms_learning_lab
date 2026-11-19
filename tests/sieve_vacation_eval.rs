use dlms_learning_lab::{
    sieve_delivery_policy::Policy as Forwarding,
    sieve_envelope::Input,
    sieve_eval::{Message, Outcome, VacationContext},
    sieve_lex::Kind,
    sieve_parse::{parse, parse_address, parse_delivery, parse_vacation},
    sieve_vacation_history::{History, SECONDS_PER_DAY},
    sieve_vacation_policy::{Policy, RunResult, Suppression},
};
use std::collections::BTreeSet;
fn boxes() -> BTreeSet<String> {
    ["INBOX", "Review"].into_iter().map(String::from).collect()
}
fn message(extra: &[(&str, &str)]) -> Message {
    Message::new(
        std::iter::once(("To", "owner@site.test"))
            .chain(extra.iter().copied())
            .map(|(n, v)| (n.into(), v.into()))
            .collect(),
        "Body",
    )
    .unwrap()
}
fn run(code: &str, at: u64, from: &str, m: &Message, h: &mut History) -> Outcome {
    let env = Input {
        from: from.into(),
        to: "owner@site.test".into(),
    }
    .validate()
    .unwrap();
    let forward = Forwarding {
        targets: ["archive@site.test".into()].into(),
    };
    let replies = Policy {
        primary: "owner@site.test".into(),
        aliases: Default::default(),
    };
    parse_vacation(code).unwrap().evaluate_vacation(
        m,
        &boxes(),
        VacationContext {
            envelope: &env,
            forwarding: &forward,
            replies: &replies,
            at_seconds: at,
        },
        h,
    )
}
fn sent(out: &Outcome) -> bool {
    matches!(out.vacation, Some(RunResult::Sent { .. }))
}
#[test]
fn new_syntax_requires_its_profile_declaration_and_honest_supported_options() {
    let script = r#"require "vacation";vacation "Away";"#;
    for parser in [parse, parse_address, parse_delivery] {
        assert_eq!(parser(script).unwrap_err().kind, Kind::Unsupported);
    }
    assert_eq!(
        parse_vacation(r#"vacation "Away";"#).unwrap_err().kind,
        Kind::Syntax
    );
    for code in [
        r#"require "vacation";vacation :mime "Away";"#,
        r#"require "vacation";vacation :seconds 1 "Away";"#,
        r#"require ["vacation","variables"];vacation "Away";"#,
        r#"require "vacation";if false {vacation :mime "Away";}"#,
    ] {
        assert_eq!(parse_vacation(code).unwrap_err().kind, Kind::Unsupported);
    }
    for code in [
        r#"require "vacation";vacation :days 1 :days 2 "Away";"#,
        r#"require "vacation";vacation :subject "Away";"#,
        r#"require "vacation";vacation :days "seven" "Away";"#,
    ] {
        assert_eq!(parse_vacation(code).unwrap_err().kind, Kind::Syntax);
    }
}
#[test]
fn real_ordered_messages_share_reply_history_and_exact_expiry() {
    let code = r#"require "vacation";vacation "Away; contact the team desk.";"#;
    let m = message(&[("From", "visible@elsewhere.test")]);
    let mut h = History::default();
    let first = run(code, 0, "actual@return.test", &m, &mut h);
    assert_eq!(first.fault, None);
    assert!(sent(&first));
    assert!(first.implicit_keep);
    assert_eq!(first.mailboxes, ["INBOX".into()].into());
    let second = run(code, SECONDS_PER_DAY, "actual@return.test", &m, &mut h);
    assert!(
        matches!(second.vacation,Some(RunResult::Recent{last_sent:0,remaining_seconds})if remaining_seconds==6*SECONDS_PER_DAY)
    );
    assert_eq!(second.fault, None);
    let third = run(code, 7 * SECONDS_PER_DAY, "actual@return.test", &m, &mut h);
    assert!(sent(&third));
    assert_eq!(h.len(), 1);
    let Some(RunResult::Sent { reply, .. }) = third.vacation else {
        panic!()
    };
    assert_eq!(reply.to, "actual@return.test");
}
#[test]
fn zero_days_are_clamped_and_different_senders_get_their_own_response() {
    let code = r#"require "vacation";vacation :days 0 "Away";"#;
    let m = message(&[]);
    let mut h = History::default();
    let first = run(code, 0, "one@outside.test", &m, &mut h);
    assert!(matches!(
        first.vacation,
        Some(RunResult::Sent {
            effective_days: 1,
            ..
        })
    ));
    assert!(sent(&run(code, 1, "two@outside.test", &m, &mut h)));
    assert!(!sent(&run(
        code,
        SECONDS_PER_DAY - 1,
        "one@outside.test",
        &m,
        &mut h
    )));
    assert!(sent(&run(
        code,
        SECONDS_PER_DAY,
        "one@outside.test",
        &m,
        &mut h
    )));
}
#[test]
fn handles_and_raw_content_arguments_control_tracking_in_the_real_evaluator() {
    let m = message(&[]);
    let mut h = History::default();
    assert!(sent(&run(
        r#"require "vacation";vacation :handle "absence" "First text";"#,
        0,
        "one@outside.test",
        &m,
        &mut h
    )));
    assert!(!sent(&run(
        r#"require "vacation";vacation :handle "absence" "Changed text";"#,
        1,
        "one@outside.test",
        &m,
        &mut h
    )));
    assert!(sent(&run(
        r#"require "vacation";vacation "Changed text";"#,
        2,
        "one@outside.test",
        &m,
        &mut h
    )));
    assert!(sent(&run(
        r#"require "vacation";vacation "Another text";"#,
        3,
        "one@outside.test",
        &m,
        &mut h
    )));
}
#[test]
fn suppression_keeps_original_mail_and_does_not_add_response_entries() {
    let code = r#"require "vacation";vacation "Away";"#;
    for (from, extra, why) in [
        ("", vec![], Suppression::NullSender),
        (
            "one@outside.test",
            vec![("List-Id", "team.site.test")],
            Suppression::MailingList,
        ),
        (
            "one@outside.test",
            vec![("Auto-Submitted", "auto-replied")],
            Suppression::Automated,
        ),
    ] {
        let mut h = History::default();
        let out = run(code, 0, from, &message(&extra), &mut h);
        assert_eq!(out.fault, None);
        assert_eq!(out.vacation, Some(RunResult::Suppressed { reason: why }));
        assert!(out.implicit_keep);
        assert!(h.is_empty());
    }
}
#[test]
fn vacation_coexists_with_filing_forwarding_keep_and_discard_without_cancelling_keep_by_itself() {
    let cases = [
        (
            r#"require "vacation";vacation "Away";"#,
            "INBOX",
            false,
            false,
        ),
        (
            r#"require ["vacation","fileinto"];vacation "Away";fileinto "Review";"#,
            "Review",
            false,
            false,
        ),
        (
            r#"require "vacation";vacation "Away";redirect "archive@site.test";"#,
            "",
            true,
            false,
        ),
        (
            r#"require "vacation";vacation "Away";discard;"#,
            "",
            false,
            true,
        ),
        (
            r#"require "vacation";discard;vacation "Away";"#,
            "",
            false,
            true,
        ),
        (
            r#"require "vacation";vacation "Away";keep;"#,
            "INBOX",
            false,
            false,
        ),
    ];
    for (code, folder, forward, discarded) in cases {
        let out = run(
            code,
            0,
            "one@outside.test",
            &message(&[]),
            &mut History::default(),
        );
        assert_eq!(out.fault, None, "{code}");
        assert!(sent(&out));
        assert_eq!(
            out.mailboxes,
            if folder.is_empty() {
                BTreeSet::new()
            } else {
                [folder.into()].into()
            }
        );
        assert_eq!(!out.redirects.is_empty(), forward);
        assert_eq!(out.discarded, discarded);
    }
}
#[test]
fn double_vacation_or_rejection_conflicts_roll_back_even_if_the_first_reply_was_suppressed() {
    for code in [
        r#"require "vacation";vacation "One";vacation "Two";"#,
        r#"require ["vacation","reject"];vacation "Away";reject "No";"#,
        r#"require ["vacation","reject"];reject "No";vacation "Away";"#,
    ] {
        for from in ["one@outside.test", ""] {
            let mut h = History::default();
            let before = h.clone();
            let out = run(code, 10, from, &message(&[]), &mut h);
            assert_eq!(out.fault.unwrap().kind, Kind::Runtime);
            assert!(out.vacation.is_none() && out.rejection.is_none() && out.redirects.is_empty());
            assert_eq!(out.mailboxes, ["INBOX".into()].into());
            assert_eq!(h, before);
        }
    }
    let code = r#"require "vacation";if true {vacation "One";} else {vacation "Two";}"#;
    assert!(sent(&run(
        code,
        0,
        "one@outside.test",
        &message(&[]),
        &mut History::default()
    )));
}
#[test]
fn late_action_error_restores_existing_history_and_cancels_every_proposed_effect() {
    let good = r#"require "vacation";vacation "Away";"#;
    let m = message(&[]);
    let mut h = History::default();
    run(good, 0, "old@outside.test", &m, &mut h);
    let before = h.clone();
    for code in [
        r#"require ["vacation","fileinto"];fileinto "Review";vacation "Away";redirect "unknown@elsewhere.test";"#,
        r#"require ["vacation","fileinto"];vacation "Away";fileinto "Missing";"#,
    ] {
        let out = run(code, 100, "new@outside.test", &m, &mut h);
        assert!(out.fault.is_some());
        assert!(out.vacation.is_none());
        assert_eq!(out.mailboxes, ["INBOX".into()].into());
        assert_eq!(h, before);
    }
}
#[test]
fn action_limit_and_backward_time_cannot_commit_a_response() {
    let mut h = History::default();
    let before = h.clone();
    let code = format!(
        "require \"vacation\";{}vacation \"Away\";",
        "keep;".repeat(32)
    );
    let out = run(&code, 10, "one@outside.test", &message(&[]), &mut h);
    assert_eq!(out.fault.unwrap().kind, Kind::Limit);
    assert!(out.vacation.is_none());
    assert_eq!(h, before);
    run(
        r#"require "vacation";vacation "Away";"#,
        100,
        "one@outside.test",
        &message(&[]),
        &mut h,
    );
    let before = h.clone();
    let out = run(
        r#"require "vacation";vacation "Away";"#,
        99,
        "two@outside.test",
        &message(&[]),
        &mut h,
    );
    assert_eq!(out.fault.unwrap().kind, Kind::Runtime);
    assert_eq!(h, before);
}
#[test]
fn absence_of_a_timeline_context_never_pretends_to_send_a_reply() {
    let p = parse_vacation(r#"require "vacation";vacation "Away";"#).unwrap();
    let out = p.evaluate(&message(&[]), &boxes());
    assert_eq!(out.fault.unwrap().kind, Kind::Unsupported);
    assert!(out.vacation.is_none());
    assert_eq!(out.mailboxes, ["INBOX".into()].into());
}
