mod envelope {
    use dlms_learning_lab::{
        sieve_envelope::{Input, bare, same_mailbox},
        sieve_lex::Kind,
        sieve_parse::AddressPart,
    };
    #[test]
    fn envelope_parts_do_not_substitute_header_text_and_null_sender_matches_empty_in_all_parts() {
        let data = Input {
            from: "".into(),
            to: "Help@service.test".into(),
        }
        .validate()
        .unwrap();
        for part in [AddressPart::All, AddressPart::Local, AddressPart::Domain] {
            assert_eq!(data.part("FROM", part).unwrap(), "");
        }
        assert_eq!(
            data.part("to", AddressPart::All).unwrap(),
            "Help@service.test"
        );
        assert_eq!(data.part("to", AddressPart::Local).unwrap(), "Help");
        assert_eq!(
            data.part("to", AddressPart::Domain).unwrap(),
            "service.test"
        );
        assert_eq!(
            data.part("sender", AddressPart::All).unwrap_err().kind,
            Kind::Unsupported
        );
    }
    #[test]
    fn represented_envelope_mailboxes_are_bounded_bare_values_not_display_names() {
        for value in ["", "not-a-mailbox", "a@b,c@d"] {
            assert!(bare(value).is_err());
        }
        for value in ["Name <a@b>", "<a@b>", " a@b", "a (comment) @ b"] {
            assert_eq!(bare(value).unwrap_err().kind, Kind::Unsupported);
        }
        assert_eq!(
            bare(&format!("{}@b", "x".repeat(254))).unwrap_err().kind,
            Kind::Limit
        );
        assert!(same_mailbox(
            &bare("Help@Service.Test").unwrap(),
            &bare("Help@service.test").unwrap()
        ));
        assert!(!same_mailbox(
            &bare("Help@service.test").unwrap(),
            &bare("help@service.test").unwrap()
        ));
    }
}
mod policy {
    use dlms_learning_lab::{
        sieve_delivery_policy::Policy, sieve_envelope::Input, sieve_lex::Kind,
    };
    #[test]
    fn virtual_redirect_targets_are_explicit_bounded_and_direct_self_loops_fail() {
        let envelope = Input {
            from: "sender@outside.test".into(),
            to: "inbox@service.test".into(),
        }
        .validate()
        .unwrap();
        let policy = Policy {
            targets: [
                String::from("archive@service.test"),
                String::from("inbox@service.test"),
            ]
            .into_iter()
            .collect(),
        };
        policy.validate().unwrap();
        assert_eq!(
            policy.redirect("archive@SERVICE.TEST", &envelope).unwrap(),
            "archive@service.test"
        );
        assert_eq!(
            policy
                .redirect("inbox@service.test", &envelope)
                .unwrap_err()
                .kind,
            Kind::Runtime
        );
        assert_eq!(
            policy
                .redirect("external@elsewhere.test", &envelope)
                .unwrap_err()
                .kind,
            Kind::Runtime
        );
        let unknown = Policy {
            targets: [String::from("../../personal")].into_iter().collect(),
        };
        assert!(unknown.validate().is_err());
        let oversized = Policy {
            targets: (0..17).map(|n| format!("mail{n}@service.test")).collect(),
        };
        assert_eq!(oversized.validate().unwrap_err().kind, Kind::Limit);
    }
}
mod evaluation {
    use dlms_learning_lab::{
        sieve_delivery_policy::Policy,
        sieve_envelope::Input,
        sieve_eval::{Message, Outcome},
        sieve_lex::Kind,
        sieve_parse::{parse, parse_address, parse_delivery},
    };
    use std::collections::BTreeSet;
    fn folders() -> BTreeSet<String> {
        ["INBOX", "Review"].into_iter().map(String::from).collect()
    }
    fn run(code: &str, from: &str, to: &str) -> Outcome {
        let message = Message::new(
            vec![
                ("From".into(), "visible@headers.test".into()),
                ("To".into(), "other@headers.test".into()),
            ],
            "body",
        )
        .unwrap();
        let envelope = Input {
            from: from.into(),
            to: to.into(),
        }
        .validate()
        .unwrap();
        let policy = Policy {
            targets: ["archive@service.test", "review@service.test"]
                .into_iter()
                .map(String::from)
                .collect(),
        };
        parse_delivery(code)
            .unwrap()
            .evaluate_delivery(&message, &folders(), &envelope, &policy)
    }
    #[test]
    fn existing_profiles_refuse_new_capabilities_and_delivery_declarations_are_required() {
        for code in [
            r#"require "envelope";keep;"#,
            r#"require "reject";reject "No";"#,
            "redirect \"archive@service.test\";",
        ] {
            assert_eq!(parse(code).unwrap_err().kind, Kind::Unsupported);
            assert_eq!(parse_address(code).unwrap_err().kind, Kind::Unsupported);
        }
        for code in [r#"if envelope "from" "x@a" {keep;}"#, r#"reject "No";"#] {
            assert_eq!(parse_delivery(code).unwrap_err().kind, Kind::Syntax);
        }
        assert_eq!(
            parse_delivery(r#"redirect :copy "archive@service.test";"#)
                .unwrap_err()
                .kind,
            Kind::Unsupported
        );
        assert_eq!(
            parse_delivery(r#"require "envelope";if envelope "sender" "x@a" {keep;}"#)
                .unwrap_err()
                .kind,
            Kind::Unsupported
        );
    }
    #[test]
    fn envelope_tests_read_actual_delivery_values_not_visible_headers() {
        let out = run(
            r#"require ["envelope","fileinto"];if envelope :is "from" "return@delivery.test" {fileinto "Review";}"#,
            "return@delivery.test",
            "inbox@service.test",
        );
        assert_eq!(out.fault, None);
        assert_eq!(
            out.mailboxes,
            [String::from("Review")].into_iter().collect()
        );
        let out = run(
            r#"require ["envelope","fileinto"];if envelope :is "from" "visible@headers.test" {fileinto "Review";}"#,
            "return@delivery.test",
            "inbox@service.test",
        );
        assert_eq!(out.mailboxes, [String::from("INBOX")].into_iter().collect());
        for part in [":all", ":domain", ":localpart"] {
            let out = run(
                &format!(
                    "require [\"envelope\",\"fileinto\"];if envelope {part} :is \"from\" \"\" {{fileinto \"Review\";}}"
                ),
                "",
                "inbox@service.test",
            );
            assert_eq!(out.fault, None);
            assert_eq!(
                out.mailboxes,
                [String::from("Review")].into_iter().collect()
            );
        }
    }
    #[test]
    fn redirects_cancel_implicit_keep_continue_and_coalesce_duplicate_targets() {
        let out = run(
            r#"redirect "archive@service.test";redirect "archive@SERVICE.TEST";"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.fault, None);
        assert_eq!(
            out.redirects,
            [String::from("archive@service.test")].into_iter().collect()
        );
        assert!(out.mailboxes.is_empty());
        assert!(!out.implicit_keep && !out.discarded);
        let out = run(
            r#"redirect "archive@service.test";keep;"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.redirects.len(), 1);
        assert_eq!(out.mailboxes, [String::from("INBOX")].into_iter().collect());
        let out = run(
            r#"redirect "archive@service.test";stop;keep;"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.redirects.len(), 1);
        assert!(out.mailboxes.is_empty());
    }
    #[test]
    fn direct_loops_unknown_targets_and_forwarding_limits_fail_atomically() {
        for code in [
            r#"redirect "archive@service.test";"#,
            r#"redirect "unknown@elsewhere.test";"#,
        ] {
            let out = run(code, "sender@outside.test", "archive@service.test");
            assert_eq!(out.fault.unwrap().kind, Kind::Runtime);
            assert!(out.redirects.is_empty());
            assert_eq!(out.mailboxes, [String::from("INBOX")].into_iter().collect());
            assert!(out.implicit_keep);
        }
        let code = r#"redirect "archive@service.test";"#.repeat(5);
        let out = run(&code, "sender@outside.test", "inbox@service.test");
        assert_eq!(out.fault.unwrap().kind, Kind::Limit);
        assert!(out.redirects.is_empty());
        assert!(!out.discarded);
    }
    #[test]
    fn a_refusal_is_not_a_discard_and_does_not_automatically_stop_later_rules() {
        let out = run(
            r#"require "reject";reject "This address is not accepting that message.";"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.fault, None);
        assert!(out.mailboxes.is_empty() && out.redirects.is_empty());
        assert!(out.rejection.is_some());
        assert!(!out.discarded && !out.implicit_keep);
        for code in [
            r#"require "reject";reject "No";keep;"#,
            r#"require ["reject","fileinto"];fileinto "Review";reject "No";"#,
            r#"require "reject";redirect "archive@service.test";reject "No";"#,
            r#"require "reject";reject "First";reject "Second";"#,
        ] {
            let out = run(code, "sender@outside.test", "inbox@service.test");
            assert_eq!(out.fault.unwrap().kind, Kind::Runtime);
            assert!(out.rejection.is_none() && out.redirects.is_empty());
            assert_eq!(out.mailboxes, [String::from("INBOX")].into_iter().collect());
        }
        let out = run(
            r#"require "reject";reject "No";stop;keep;"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.fault, None);
        assert!(out.rejection.is_some());
        let out = run(
            r#"require "reject";reject "No";discard;"#,
            "sender@outside.test",
            "inbox@service.test",
        );
        assert_eq!(out.fault, None);
        assert!(out.rejection.is_some());
        assert!(!out.discarded);
    }
}
