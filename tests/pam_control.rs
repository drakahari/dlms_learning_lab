//! Fixed synthetic return codes only. Never calls host PAM or reads credentials.
use dlms_learning_lab::pam_control::{self as p, Action, Code, ErrorKind, Facts};
fn run(text: &str, code: Code) -> p::Outcome {
    p::evaluate(&p::parse(text).unwrap(), Facts { unix_result: code })
}
#[test]
fn required_failure_runs_later_rules_and_keeps_the_first_error() {
    let r = run(
        "auth required pam_unix.so\nauth required pam_deny.so\nauth sufficient pam_permit.so",
        Code::UserUnknown,
    );
    assert_eq!(r.result, Code::UserUnknown);
    assert!(!r.accepted());
    assert_eq!(r.trace.len(), 3);
    assert_eq!(r.trace[0].action, Action::RememberFailure);
    assert_eq!(r.trace[1].returned, Code::AuthError);
    assert_eq!(r.trace[2].remembered_failure, Some(Code::UserUnknown));
    assert_ne!(r.trace[2].action, Action::ReturnSuccess);
    assert_eq!(r.trace[2].action, Action::KeepFailure);
}
#[test]
fn requisite_stops_and_preserves_a_prior_required_failure() {
    let r = run(
        "# supplied stack\nauth required pam_unix.so\nauth requisite pam_deny.so\nauth required pam_permit.so",
        Code::AuthInfoUnavailable,
    );
    assert_eq!(r.result, Code::AuthInfoUnavailable);
    assert_eq!(r.trace.len(), 2);
    assert_eq!(r.trace[1].line, 3);
    assert_eq!(r.trace[1].action, Action::ReturnFailure);
    let r = run(
        "auth requisite pam_deny.so\nauth required pam_unix.so",
        Code::Success,
    );
    assert_eq!(r.result, Code::AuthError);
    assert_eq!(r.trace.len(), 1);
}
#[test]
fn sufficient_success_short_circuits_only_when_no_failure_is_remembered() {
    let text = "auth sufficient pam_unix.so\nauth required pam_deny.so";
    let good = run(text, Code::Success);
    assert!(good.accepted());
    assert_eq!(good.trace.len(), 1);
    assert_eq!(good.trace[0].action, Action::ReturnSuccess);
    for code in [
        Code::AuthError,
        Code::UserUnknown,
        Code::AuthInfoUnavailable,
        Code::Ignore,
    ] {
        let r = run(text, code);
        assert_eq!(r.result, Code::AuthError);
        assert_eq!(r.trace.len(), 2);
        assert_eq!(r.trace[0].action, Action::Ignore);
    }
    let r = run(
        "auth required pam_deny.so\nauth sufficient pam_permit.so\nauth optional pam_unix.so",
        Code::Success,
    );
    assert_eq!(r.result, Code::AuthError);
    assert_eq!(r.trace.len(), 3);
}
#[test]
fn optional_success_contributes_but_failure_does_not_override_other_results() {
    assert!(run("auth optional pam_permit.so", Code::Success).accepted());
    let only_failed = run("auth optional pam_deny.so", Code::Success);
    assert_eq!(only_failed.result, Code::PermissionDenied);
    assert_eq!(only_failed.trace[0].action, Action::Ignore);
    assert!(
        run(
            "auth required pam_unix.so\nauth optional pam_deny.so",
            Code::Success
        )
        .accepted()
    );
    assert!(
        run(
            "auth optional pam_deny.so\nauth required pam_unix.so",
            Code::Success
        )
        .accepted()
    );
    assert_eq!(
        run(
            "auth required pam_deny.so\nauth optional pam_permit.so",
            Code::Success
        )
        .result,
        Code::AuthError
    );
    // Two optional successes do not become failures merely because there is more than one.
    assert!(
        run(
            "auth optional pam_permit.so\nauth optional pam_permit.so",
            Code::Success
        )
        .accepted()
    );
}
#[test]
fn ignored_authentication_is_not_a_success_or_a_remembered_failure() {
    for control in ["required", "requisite", "sufficient", "optional"] {
        let r = run(&format!("auth {control} pam_unix.so"), Code::Ignore);
        assert_eq!(r.result, Code::PermissionDenied);
        assert_eq!(r.trace[0].remembered_failure, None);
        assert_eq!(r.trace[0].action, Action::Ignore);
    }
    assert!(
        run(
            "auth required pam_unix.so\nauth required pam_permit.so",
            Code::Ignore
        )
        .accepted()
    );
}
#[test]
fn alternate_controls_accept_good_and_reject_bad_supplied_results() {
    for text in [
        "auth required pam_unix.so",
        "auth requisite pam_unix.so",
        "auth sufficient pam_unix.so\nauth required pam_deny.so",
        "auth required pam_unix.so\nauth optional pam_deny.so",
        "auth requisite pam_unix.so\nauth required pam_permit.so",
    ] {
        assert!(run(text, Code::Success).accepted(), "{text}");
        for code in [
            Code::AuthError,
            Code::UserUnknown,
            Code::AuthInfoUnavailable,
        ] {
            assert!(!run(text, code).accepted(), "{text}: {code:?}");
        }
    }
    // Plausible dangerous edits: all-permit, optional credential checking and early permit bypass.
    for text in [
        "auth required pam_permit.so",
        "auth optional pam_unix.so\nauth required pam_permit.so",
        "auth sufficient pam_permit.so\nauth required pam_unix.so",
    ] {
        assert!(
            run(text, Code::AuthError).accepted(),
            "negative teaching case must expose the bypass: {text}"
        );
    }
    assert!(!run("auth required pam_deny.so", Code::Success).accepted());
}
#[test]
fn comments_spacing_case_and_crlf_preserve_real_line_numbers() {
    let s = p::parse("# Example\r\n\r\n AUTH\tReQuIrEd pam_unix.so # result supplied by case\r\n")
        .unwrap();
    assert_eq!(s.rules().len(), 1);
    assert_eq!(s.rules()[0].line, 3);
    assert!(
        p::evaluate(
            &s,
            Facts {
                unix_result: Code::Success
            }
        )
        .accepted()
    );
}
#[test]
fn syntax_errors_are_separate_from_unsupported_behavior() {
    for text in [
        "",
        "# no rules",
        "auth required",
        "auth\0 required pam_unix.so",
    ] {
        assert_eq!(p::parse(text).unwrap_err().kind, ErrorKind::Syntax);
    }
    for text in [
        "account required pam_unix.so",
        "password required pam_unix.so",
        "session required pam_permit.so",
        "-auth required pam_unix.so",
        "auth include common-auth",
        "auth substack common-auth",
        "auth [success=1 default=ignore] pam_unix.so",
        "auth required pam_unix.so nullok",
        "auth required pam_exec.so /bin/sh",
        "auth required /lib/security/pam_unix.so",
        "auth required ../../pam_unix.so",
        "auth required PAM_UNIX.SO",
        "auth required pam_permit.so ignored-argument",
        "auth required pam_unix.so\\\n auth required pam_deny.so",
    ] {
        let e = p::parse(text).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unsupported, "{text}");
        assert!(e.to_string().contains("Unsupported in this simulator"));
        assert_eq!(e.line, Some(1));
    }
}
#[test]
fn bounded_text_lines_and_rules_refuse_excess_before_evaluation() {
    let rule = "auth optional pam_permit.so\n";
    assert_eq!(p::parse(&rule.repeat(32)).unwrap().rules().len(), 32);
    assert_eq!(
        p::parse(&rule.repeat(33)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        p::parse(&("#\n".repeat(127) + "auth required pam_unix.so"))
            .unwrap()
            .rules()
            .len(),
        1
    );
    assert_eq!(
        p::parse(&("#\n".repeat(128) + "auth required pam_unix.so"))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(
        p::parse(&"#".repeat(257)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        p::parse(&" ".repeat(8193)).unwrap_err().kind,
        ErrorKind::Limit
    );
    let r = run(&rule.repeat(32), Code::Success);
    assert_eq!(r.trace.len(), 32);
}
#[test]
fn return_codes_and_visited_rules_survive_retries_without_shared_state() {
    let s = p::parse("auth sufficient pam_unix.so\nauth requisite pam_deny.so").unwrap();
    let a = p::evaluate(
        &s,
        Facts {
            unix_result: Code::Success,
        },
    );
    let b = p::evaluate(
        &s,
        Facts {
            unix_result: Code::UserUnknown,
        },
    );
    let c = p::evaluate(
        &s,
        Facts {
            unix_result: Code::Success,
        },
    );
    assert_eq!(a, c);
    assert_ne!(a, b);
    assert_eq!(b.trace.len(), 2);
    assert_eq!(b.result, Code::AuthError);
    assert!(
        serde_json::from_str::<Facts>(r#"{"unix_result":"Success","host_path":"/etc/shadow"}"#)
            .is_err()
    );
}
