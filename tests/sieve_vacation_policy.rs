use dlms_learning_lab::{
    sieve_envelope::Input,
    sieve_eval::Message,
    sieve_lex::Kind,
    sieve_vacation_policy::{AutoSubmitted, Policy, Preparation, Request, Suppression},
};
fn policy() -> Policy {
    Policy {
        primary: "owner@site.test".into(),
        aliases: ["alias@site.test".into()].into(),
    }
}
fn message(extra: &[(&str, &str)]) -> Message {
    let fields = std::iter::once(("To", "owner@site.test"))
        .chain(extra.iter().copied())
        .map(|(n, v)| (n.into(), v.into()))
        .collect();
    Message::new(fields, "Body").unwrap()
}
fn envelope(from: &str) -> dlms_learning_lab::sieve_envelope::Envelope {
    Input {
        from: from.into(),
        to: "delivered@site.test".into(),
    }
    .validate()
    .unwrap()
}
fn request() -> Request {
    Request {
        reason: "I am away; use the team desk for urgent work.".into(),
        ..Default::default()
    }
}
#[test]
fn logical_reply_uses_actual_return_address_owner_and_real_thread_metadata() {
    let m = message(&[
        ("From", "visible@elsewhere.test"),
        ("Subject", "A question"),
        ("Message-ID", "<new@message.test>"),
        ("References", "<first@message.test> <second@message.test>"),
    ]);
    let Preparation::Ready { reply, key } = policy()
        .prepare(&m, &envelope("actual@return.test"), &request())
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(reply.to, "actual@return.test");
    assert_eq!(reply.from, "owner@site.test");
    assert_eq!(reply.subject, "Auto: A question");
    assert_eq!(reply.body, request().reason);
    assert_eq!(reply.auto_submitted, AutoSubmitted::AutoReplied);
    assert_eq!(reply.in_reply_to.as_deref(), Some("<new@message.test>"));
    assert_eq!(
        reply.references,
        [
            "<first@message.test>",
            "<second@message.test>",
            "<new@message.test>"
        ]
    );
    assert_eq!(key, request().response_key());
}
#[test]
fn null_and_configured_unattended_senders_are_suppressed_but_postmaster_is_not_assumed_unattended()
{
    for (from, why) in [
        ("", Suppression::NullSender),
        ("MAILER-DAEMON@outside.test", Suppression::DaemonSender),
        ("owner-team@outside.test", Suppression::DaemonSender),
        ("team-request@outside.test", Suppression::DaemonSender),
        ("listserv@outside.test", Suppression::DaemonSender),
        ("majordomo@outside.test", Suppression::DaemonSender),
    ] {
        assert_eq!(
            policy()
                .prepare(&message(&[]), &envelope(from), &request())
                .unwrap(),
            Preparation::Suppressed(why)
        );
    }
    assert!(matches!(
        policy()
            .prepare(
                &message(&[]),
                &envelope("postmaster@outside.test"),
                &request()
            )
            .unwrap(),
        Preparation::Ready { .. }
    ));
}
#[test]
fn every_standard_list_field_and_automatic_keyword_suppresses_the_reply() {
    for name in [
        "List-Id",
        "List-Help",
        "List-Subscribe",
        "List-Unsubscribe",
        "List-Post",
        "List-Owner",
        "List-Archive",
    ] {
        assert_eq!(
            policy()
                .prepare(
                    &message(&[(name, "")]),
                    &envelope("writer@outside.test"),
                    &request()
                )
                .unwrap(),
            Preparation::Suppressed(Suppression::MailingList)
        );
    }
    for value in ["auto-replied", "auto-generated", "extension"] {
        assert_eq!(
            policy()
                .prepare(
                    &message(&[("Auto-Submitted", value)]),
                    &envelope("writer@outside.test"),
                    &request()
                )
                .unwrap(),
            Preparation::Suppressed(Suppression::Automated)
        );
    }
    assert!(matches!(
        policy()
            .prepare(
                &message(&[("Auto-Submitted", " NO ")]),
                &envelope("writer@outside.test"),
                &request()
            )
            .unwrap(),
        Preparation::Ready { .. }
    ));
    assert_eq!(
        policy()
            .prepare(
                &message(&[("Auto-Submitted", "no; parameter=value")]),
                &envelope("writer@outside.test"),
                &request()
            )
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
}
#[test]
fn personal_recipient_fields_and_actual_aliases_count_but_mentions_do_not() {
    for field in ["To", "Cc", "Bcc", "Resent-To", "Resent-Cc", "Resent-Bcc"] {
        let m = Message::new(
            vec![(field.into(), "Name <alias@SITE.TEST>".into())],
            "body",
        )
        .unwrap();
        assert!(
            matches!(
                policy()
                    .prepare(&m, &envelope("writer@outside.test"), &request())
                    .unwrap(),
                Preparation::Ready { .. }
            ),
            "{field}"
        );
    }
    for (field, value) in [
        ("From", "owner@site.test"),
        ("Subject", "owner@site.test"),
        ("To", "\"owner@site.test\" <other@outside.test>"),
    ] {
        let m = Message::new(vec![(field.into(), value.into())], "body").unwrap();
        assert_eq!(
            policy()
                .prepare(&m, &envelope("writer@outside.test"), &request())
                .unwrap(),
            Preparation::Suppressed(Suppression::NotPersonal)
        );
    }
    let m = Message::new(vec![("To".into(), "delivered@site.test".into())], "body").unwrap();
    assert!(matches!(
        policy()
            .prepare(&m, &envelope("writer@outside.test"), &request())
            .unwrap(),
        Preparation::Ready { .. }
    ));
}
#[test]
fn additional_addresses_are_meaningful_and_replying_to_self_is_suppressed() {
    let m = Message::new(vec![("To".into(), "other-alias@site.test".into())], "body").unwrap();
    let mut r = request();
    assert_eq!(
        policy()
            .prepare(&m, &envelope("writer@outside.test"), &r)
            .unwrap(),
        Preparation::Suppressed(Suppression::NotPersonal)
    );
    r.addresses.push("other-alias@site.test".into());
    assert!(matches!(
        policy()
            .prepare(&m, &envelope("writer@outside.test"), &r)
            .unwrap(),
        Preparation::Ready { .. }
    ));
    for sender in [
        "owner@site.test",
        "alias@SITE.TEST",
        "delivered@site.test",
        "other-alias@site.test",
    ] {
        assert_eq!(
            policy().prepare(&m, &envelope(sender), &r).unwrap(),
            Preparation::Suppressed(Suppression::SelfReply)
        );
    }
}
#[test]
fn explicit_subject_and_from_override_defaults_without_changing_the_input() {
    let m = message(&[("Subject", "Incoming")]);
    let before = m.clone();
    let mut r = request();
    r.subject = Some("Temporarily away".into());
    r.from = Some("alias@site.test".into());
    r.handle = Some("absence".into());
    let Preparation::Ready { reply, key } = policy()
        .prepare(&m, &envelope("writer@outside.test"), &r)
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(reply.subject, "Temporarily away");
    assert_eq!(reply.from, "alias@site.test");
    assert_eq!(key, r.response_key());
    assert_eq!(m, before);
    let Preparation::Ready { reply, .. } = policy()
        .prepare(&message(&[]), &envelope("writer@outside.test"), &request())
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(reply.subject, "Automated reply");
    assert!(reply.in_reply_to.is_none() && reply.references.is_empty());
}
#[test]
fn unsupported_encoded_subjects_and_malformed_parameters_do_not_pretend_success() {
    // The existing validated Message boundary rejects encoded headers before
    // a policy can receive them; retain that earlier honest diagnostic.
    assert_eq!(
        Message::new(
            vec![("Subject".into(), "=?UTF-8?Q?Example?=".into())],
            "body"
        )
        .unwrap_err()
        .kind,
        Kind::Unsupported
    );
    let mut r = request();
    r.subject = Some("Injected\nSubject".into());
    assert_eq!(
        policy()
            .prepare(&message(&[]), &envelope("writer@outside.test"), &r)
            .unwrap_err()
            .kind,
        Kind::Syntax
    );
    let mut r = request();
    r.from = Some("Name <alias@site.test>".into());
    assert_eq!(
        policy()
            .prepare(&message(&[]), &envelope("writer@outside.test"), &r)
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
    let mut r = request();
    r.addresses = vec!["alias@site.test".into(); 33];
    assert_eq!(
        policy()
            .prepare(&message(&[]), &envelope("writer@outside.test"), &r)
            .unwrap_err()
            .kind,
        Kind::Limit
    );
    let invalid = Policy {
        primary: "../../personal".into(),
        aliases: Default::default(),
    };
    assert!(
        invalid
            .prepare(&message(&[]), &envelope("writer@outside.test"), &request())
            .is_err()
    );
}

#[test]
fn default_generated_subject_does_not_change_the_response_identity() {
    let p = policy();
    let e = envelope("writer@outside.test");
    let r = request();
    let Preparation::Ready {
        reply: first,
        key: a,
    } = p
        .prepare(&message(&[("Subject", "First question")]), &e, &r)
        .unwrap()
    else {
        panic!()
    };
    let Preparation::Ready {
        reply: second,
        key: b,
    } = p
        .prepare(&message(&[("Subject", "Another question")]), &e, &r)
        .unwrap()
    else {
        panic!()
    };
    assert_ne!(first.subject, second.subject);
    assert_eq!(a, b);
    let sender = dlms_learning_lab::sieve_envelope::bare("writer@outside.test").unwrap();
    let mut history = dlms_learning_lab::sieve_vacation_history::History::default();
    history.consider(&sender, &a, 0, r.days).unwrap();
    assert!(matches!(
        history.consider(&sender, &b, 1, r.days).unwrap(),
        dlms_learning_lab::sieve_vacation_history::Decision::Suppressed { .. }
    ));
}
