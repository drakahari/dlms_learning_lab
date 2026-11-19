use dlms_learning_lab::{
    ldap_identity::{
        self as i,
        protected::{Policy, ProtectedRealm},
    },
    ldap_protection::{Certificate, Endpoint, Request, Trust},
    ldap_query::Dn,
    ldap_utility as u,
};
#[path = "support/ldap_directory.rs"]
mod fixture;
const URI: &str = "ldap://directory.example.test";
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const OLD: &str = "Fictional-only-original-91";
const NEW: &str = "Fictional-only-replacement-92";
fn dn(s: &str) -> Dn {
    Dn::parse(s).unwrap()
}
fn realm(old_required: bool) -> ProtectedRealm {
    let endpoint = Endpoint::new(
        URI,
        true,
        Some(
            Certificate::new(&["directory.example.test"], "training-root", 100, 200, true).unwrap(),
        ),
    )
    .unwrap();
    ProtectedRealm::new(
        endpoint,
        &fixture::fixture(),
        &[
            i::Identity::new(dn(ALEX), OLD, dn(BASE), true, true, &[]).unwrap(),
            i::Identity::new(dn(SAM), OLD, dn(BASE), false, false, &[]).unwrap(),
        ],
        Policy {
            supports_password_modify: true,
            allow_self_change: true,
            require_old_password: old_required,
        },
    )
    .unwrap()
}
fn trust() -> Trust {
    Trust::new(&["training-root"]).unwrap()
}
fn command(extra: &str) -> String {
    format!("ldappasswd -x -W -S -ZZ -H {URI} -D '{ALEX}' {extra}")
}
fn prompts() -> u::PasswordPrompts<'static> {
    u::PasswordPrompts {
        bind: OLD,
        old_target: Some(OLD),
        replacement: Some(NEW),
    }
}
fn can_auth(r: &ProtectedRealm, name: &str, value: &str) -> bool {
    let c = r
        .open(URI, Request::RequireStartTls, &trust(), 150)
        .unwrap();
    r.authenticate(&c, &dn(name), value).is_ok()
}
#[test]
fn the_parser_distinguishes_three_prompts_and_optional_target_from_bind_identity() {
    let p = u::parse_password(&command("-A")).unwrap();
    assert_eq!(p.uri(), URI);
    assert!(p.bind().equivalent(&dn(ALEX)));
    assert!(p.target().is_none());
    assert!(p.asks_old_password());
    assert_eq!(p.protection_request(), Request::RequireStartTls);
    let p = u::parse_password(&command(&format!("-- '{SAM}'"))).unwrap();
    assert!(p.target().unwrap().equivalent(&dn(SAM)));
    assert!(!p.asks_old_password());
    let plain = command("").replace(" -ZZ", "");
    assert_eq!(
        u::parse_password(&plain).unwrap().protection_request(),
        Request::Plain
    );
    assert_eq!(
        u::parse_password(&command("").replace("-ZZ", "-Z"))
            .unwrap()
            .protection_request(),
        Request::TryStartTls
    );
}
#[test]
fn taught_command_alternate_order_optional_self_and_prompt_forms_change_real_modeled_credentials() {
    for raw in [
        command(""),
        command(&format!("'{ALEX}'")),
        command(&format!("-A -- '{ALEX}'")),
        format!("ldappasswd -D '{ALEX}' -H ldap://DIRECTORY.EXAMPLE.TEST:389/ -S -A -W -x -Z"),
    ] {
        let mut r = realm(false);
        let before = r.directory().source().clone();
        let result = u::run_password(&raw, &prompts(), &trust(), 150, &mut r);
        assert!(result.succeeded(), "{result:?}");
        assert_eq!(result.exit_status(), 0);
        assert!(!can_auth(&r, ALEX, OLD));
        assert!(can_auth(&r, ALEX, NEW));
        assert!(can_auth(&r, SAM, OLD));
        assert_eq!(r.directory().source(), &before);
    }
}
#[test]
fn server_required_old_target_value_is_not_confused_with_the_bind_prompt() {
    let mut r = realm(true);
    let no_old = u::run_password(&command(""), &prompts(), &trust(), 150, &mut r);
    assert!(matches!(
        no_old.failure,
        Some(u::PasswordFailure::Operation(
            i::protected::Error::OldPasswordRequired
        ))
    ));
    assert!(can_auth(&r, ALEX, OLD));
    let wrong = u::PasswordPrompts {
        old_target: Some("Wrong-old-target"),
        ..prompts()
    };
    let report = u::run_password(&command("-A"), &wrong, &trust(), 150, &mut r);
    assert!(matches!(
        report.failure,
        Some(u::PasswordFailure::Operation(
            i::protected::Error::OldPasswordMismatch
        ))
    ));
    assert!(can_auth(&r, ALEX, OLD));
    assert!(!can_auth(&r, ALEX, NEW));
    assert!(u::run_password(&command("-A"), &prompts(), &trust(), 150, &mut r).succeeded());
    assert!(can_auth(&r, ALEX, NEW));
}
#[test]
fn failed_bind_has_its_own_result_and_all_prompt_data_is_redacted() {
    let mut r = realm(false);
    let wrong = u::PasswordPrompts {
        bind: "Private-bind-test-93",
        old_target: Some("Private-old-test-94"),
        replacement: Some("Private-new-test-95"),
    };
    let result = u::run_password(&command("-A"), &wrong, &trust(), 150, &mut r);
    assert_eq!(result.exit_status(), 49);
    let text = format!("{wrong:?} {result:?} {}", result.failure.as_ref().unwrap());
    for value in [
        wrong.bind,
        wrong.old_target.unwrap(),
        wrong.replacement.unwrap(),
    ] {
        assert!(!text.contains(value));
    }
    assert!(can_auth(&r, ALEX, OLD));
}
#[test]
fn missing_and_invalid_prompt_values_do_not_simulate_success_or_read_any_host_input() {
    let mut r = realm(false);
    let p = u::PasswordPrompts {
        old_target: None,
        ..prompts()
    };
    assert_eq!(
        u::run_password(&command("-A"), &p, &trust(), 150, &mut r).failure,
        Some(u::PasswordFailure::MissingPrompt(
            u::MissingPrompt::OldTarget
        ))
    );
    let p = u::PasswordPrompts {
        replacement: None,
        ..prompts()
    };
    assert_eq!(
        u::run_password(&command(""), &p, &trust(), 150, &mut r).failure,
        Some(u::PasswordFailure::MissingPrompt(
            u::MissingPrompt::Replacement
        ))
    );
    for bad in ["".into(), "x".repeat(129), "bad\nvalue".into()] {
        let p = u::PasswordPrompts {
            replacement: Some(&bad),
            ..prompts()
        };
        assert!(!u::run_password(&command(""), &p, &trust(), 150, &mut r).succeeded());
        assert!(can_auth(&r, ALEX, OLD));
    }
}
#[test]
fn optional_unrequested_old_prompt_data_is_inert_but_cannot_bypass_a_required_policy() {
    let mut r = realm(false);
    let p = u::PasswordPrompts {
        old_target: Some("unused-old-prompt"),
        ..prompts()
    };
    assert!(u::run_password(&command(""), &p, &trust(), 150, &mut r).succeeded());
    let mut r = realm(true);
    assert!(!u::run_password(&command(""), &p, &trust(), 150, &mut r).succeeded());
    assert!(can_auth(&r, ALEX, OLD));
}
#[test]
fn plaintext_wrong_trust_expiry_and_other_targets_fail_without_mutation() {
    for (raw, trust, time) in [
        (command("").replace(" -ZZ", ""), trust(), 150),
        (command(""), Trust::default(), 150),
        (command(""), trust(), 201),
        (command(&format!("'{SAM}'")), trust(), 150),
        (
            command("").replace(URI, "ldap://other.example.test"),
            trust(),
            150,
        ),
    ] {
        let mut r = realm(false);
        let result = u::run_password(&raw, &prompts(), &trust, time, &mut r);
        assert!(!result.succeeded(), "{result:?}");
        assert!(can_auth(&r, ALEX, OLD));
        assert!(!can_auth(&r, ALEX, NEW));
        assert!(can_auth(&r, SAM, OLD));
    }
}
#[test]
fn literal_subset_rejects_password_arguments_files_shell_syntax_and_unsupported_tools() {
    for extra in [
        "-w Private-cli-test-96",
        "-s Private-cli-test-96",
        "-a Private-cli-test-96",
        "-y /etc/passwd",
        "-t /work/old",
        "-T /work/new",
        "-f /work/list",
        "-Y EXTERNAL",
        "-n",
        "-v",
        "-e noop",
        "-r",
        "-c",
        "; echo wrong",
        "| echo wrong",
        "> /work/file",
        "$(echo wrong)",
        "`echo wrong`",
    ] {
        let mut r = realm(false);
        let result = u::run_password(&command(extra), &prompts(), &trust(), 150, &mut r);
        assert!(!result.succeeded(), "{extra}");
        assert!(!format!("{result:?}").contains("Private-cli-test-96"));
        assert!(can_auth(&r, ALEX, OLD));
    }
    for name in ["ldapmodify", "ldapadd", "ldapdelete", "passwd", "bash"] {
        assert!(u::parse_password(&command("").replacen("ldappasswd", name, 1)).is_err());
    }
    assert!(u::parse(&command("")).is_err());
    assert!(u::parse_delete(&command("")).is_err());
}
#[test]
fn duplicate_missing_conflicting_options_and_extra_operands_fail_honestly() {
    for extra in [
        "-x",
        "-W",
        "-S",
        "-A -A",
        "-ZZ",
        "-Z",
        "-H ldap://else.test",
        "-D uid=else,dc=test",
        "-H",
        "-D",
    ] {
        assert!(u::parse_password(&command(extra)).is_err(), "{extra}");
    }
    for required in ["-x ", "-W ", "-S ", "-H ldap://directory.example.test "] {
        assert!(
            u::parse_password(&command("").replace(required, "")).is_err(),
            "{required}"
        );
    }
    assert!(u::parse_password(&command(&format!("'{ALEX}' '{SAM}'"))).is_err());
    assert!(u::parse_password(&command(&format!("'{ALEX}' -A"))).is_err());
    assert!(
        u::parse_password(&command("").replace(URI, "ldaps://directory.example.test")).is_err()
    );
    assert!(u::parse_password(&command("").replace(ALEX, "not-a-dn")).is_err());
}
#[test]
fn existing_resource_limits_apply_before_any_password_request() {
    let mut r = realm(false);
    for raw in [
        "x".repeat(4097),
        format!("{} {}", command(""), "extra ".repeat(40)),
        format!("{}\n{}", command(""), command("")),
    ] {
        let result = u::run_password(&raw, &prompts(), &trust(), 150, &mut r);
        assert!(!result.succeeded());
        assert!(can_auth(&r, ALEX, OLD));
    }
}
#[test]
fn later_invocations_authenticate_against_changed_state_and_cannot_succeed_with_stale_bind_data() {
    let mut r = realm(false);
    assert!(u::run_password(&command(""), &prompts(), &trust(), 150, &mut r).succeeded());
    let stale = u::run_password(&command(""), &prompts(), &trust(), 150, &mut r);
    assert_eq!(stale.exit_status(), 49);
    let p = u::PasswordPrompts {
        bind: NEW,
        old_target: Some(NEW),
        replacement: Some("Fictional-third-only-97"),
    };
    assert!(u::run_password(&command("-A"), &p, &trust(), 150, &mut r).succeeded());
    assert!(can_auth(&r, ALEX, "Fictional-third-only-97"));
    assert!(!can_auth(&r, ALEX, NEW));
    assert!(can_auth(&r, SAM, OLD));
}

#[test]
fn unprotected_and_optional_fallback_are_honest_subset_refusals_not_fake_server_auth_failures() {
    let mut r = realm(false);
    let plain = u::run_password(
        &command("").replace(" -ZZ", ""),
        &prompts(),
        &trust(),
        150,
        &mut r,
    );
    assert_eq!(
        plain.failure,
        Some(u::PasswordFailure::UnsupportedTransport(
            dlms_learning_lab::ldap_protection::PlainReason::NotRequested
        ))
    );
    assert_eq!(plain.exit_status(), 1);
    assert!(can_auth(&r, ALEX, OLD));
    assert!(!can_auth(&r, ALEX, NEW));
    let endpoint = Endpoint::new(URI, false, None).unwrap();
    let mut r = ProtectedRealm::new(
        endpoint,
        &fixture::fixture(),
        &[i::Identity::new(dn(ALEX), OLD, dn(BASE), false, false, &[]).unwrap()],
        Policy {
            supports_password_modify: true,
            allow_self_change: true,
            require_old_password: false,
        },
    )
    .unwrap();
    let before = r.directory().source().clone();
    let optional = u::run_password(
        &command("").replace("-ZZ", "-Z"),
        &prompts(),
        &trust(),
        150,
        &mut r,
    );
    assert_eq!(
        optional.failure,
        Some(u::PasswordFailure::UnsupportedTransport(
            dlms_learning_lab::ldap_protection::PlainReason::StartTlsUnavailable
        ))
    );
    assert!(
        optional
            .failure
            .as_ref()
            .unwrap()
            .to_string()
            .contains("real-client fallback is outside")
    );
    let required = u::run_password(&command(""), &prompts(), &trust(), 150, &mut r);
    assert_eq!(
        required.failure,
        Some(u::PasswordFailure::Operation(
            i::protected::Error::Protection(
                dlms_learning_lab::ldap_protection::Error::StartTlsUnavailable
            )
        ))
    );
    assert_eq!(r.directory().source(), &before);
}
