use dlms_learning_lab::ssh_policy::{Kind, Method, Policy, RootLogin};
#[test]
fn method_offers_are_separate_from_credential_acceptance_and_root_defaults() {
    let p = Policy::parse("").unwrap();
    assert!(p.offers("alex", Method::PublicKey).unwrap());
    assert!(p.offers("alex", Method::Password).unwrap());
    assert!(p.offers("alex", Method::KeyboardInteractive).unwrap());
    assert!(p.offers("root", Method::PublicKey).unwrap());
    assert!(!p.offers("root", Method::Password).unwrap());
    assert!(!p.offers("root", Method::KeyboardInteractive).unwrap());
    assert!(p.strict_modes);
    assert_eq!(
        p.authorized_files,
        vec![".ssh/authorized_keys", ".ssh/authorized_keys2"]
    );
}
#[test]
fn ordinary_first_value_wins_but_repeated_allow_users_appends() {
    let p=Policy::parse("PasswordAuthentication no\npasswordauthentication yes\nAllowUsers alex\nallowusers blair alex # append\n").unwrap();
    assert!(!p.password);
    assert_eq!(
        p.allow_users,
        Some(vec!["alex".into(), "blair".into(), "alex".into()])
    );
    assert!(p.offers("blair", Method::PublicKey).unwrap());
    assert!(!p.offers("carol", Method::PublicKey).unwrap());
    let p=Policy::parse("PermitRootLogin no\nPermitRootLogin yes\nAuthorizedKeysFile none\nAuthorizedKeysFile .ssh/authorized_keys\n").unwrap();
    assert_eq!(p.root, RootLogin::No);
    assert!(p.authorized_files.is_empty());
    assert!(!p.offers("root", Method::PublicKey).unwrap());
}
#[test]
fn key_only_and_method_disabling_compose_without_silently_accepting_password_or_keyboard() {
    let p = Policy::parse("AuthenticationMethods publickey\nAllowUsers alex\n").unwrap();
    assert!(p.password && p.keyboard_interactive);
    assert!(p.offers("alex", Method::PublicKey).unwrap());
    assert!(!p.offers("alex", Method::Password).unwrap());
    assert!(!p.offers("alex", Method::KeyboardInteractive).unwrap());
    let p = Policy::parse("AuthenticationMethods publickey\nPubkeyAuthentication no").unwrap();
    for m in [
        Method::PublicKey,
        Method::Password,
        Method::KeyboardInteractive,
    ] {
        assert!(!p.offers("alex", m).unwrap());
    }
    let p = Policy::parse("PasswordAuthentication no").unwrap();
    assert!(
        p.offers("alex", Method::KeyboardInteractive).unwrap(),
        "Disabling password alone is not a claim all interactive methods are disabled"
    );
}
#[test]
fn supported_alternatives_keep_real_directive_case_and_argument_rules() {
    let a = Policy::parse("PermitRootLogin prohibit-password\nStrictModes yes\n").unwrap();
    let b = Policy::parse("# comment\r\npErMiTrOoTlOgIn\twithout-password\r\nSTRICTMODES yes\r\n")
        .unwrap();
    assert_eq!(a, b);
    let p =
        Policy::parse("AuthorizedKeysFile .ssh/authorized_keys2 .ssh/authorized_keys\n").unwrap();
    assert_eq!(p.authorized_files.len(), 2);
    assert_eq!(
        Policy::parse("PasswordAuthentication YES")
            .unwrap_err()
            .kind,
        Kind::Malformed
    );
}
#[test]
fn malformed_and_unsupported_are_distinct_and_never_silently_ignored() {
    for text in [
        "PasswordAuthentication",
        "StrictModes maybe",
        "PermitRootLogin root",
        "PubkeyAuthentication yes no",
    ] {
        let e = Policy::parse(text).unwrap_err();
        assert_eq!(e.kind, Kind::Malformed, "{text}");
        assert_eq!(e.line, 1);
    }
    for text in [
        "Match User alex",
        "Include /etc/ssh/other",
        "UsePAM yes",
        "AllowUsers *@host",
        "AuthenticationMethods publickey,password",
        "PermitRootLogin forced-commands-only",
        "AuthorizedKeysFile ../keys",
        "AuthorizedKeysFile /etc/passwd",
        "PasswordAuthentication=yes",
        "AllowUsers \"alex\"",
        "X11Forwarding no",
    ] {
        assert_eq!(
            Policy::parse(text).unwrap_err().kind,
            Kind::Unsupported,
            "{text}"
        );
    }
    // A later unsupported/invalid value is diagnosed even when an earlier supported
    // value won; unsupported source is never made to look fully evaluated.
    assert_eq!(
        Policy::parse("StrictModes yes\nStrictModes maybe")
            .unwrap_err()
            .line,
        2
    );
}
#[test]
fn documents_and_names_are_bounded_and_only_in_memory() {
    for text in [
        "x".repeat(16_385),
        format!("#{}", "a".repeat(1024)),
        "StrictModes yes\n".repeat(129),
        format!("AllowUsers {}", vec!["alex"; 33].join(" ")),
    ] {
        assert_eq!(Policy::parse(&text).unwrap_err().kind, Kind::Limit);
    }
    let p = Policy::default();
    for name in ["a;id", "$(id)", "../../root", "UPPER", ""] {
        assert!(p.offers(name, Method::PublicKey).is_err());
    }
    assert!(Policy::parse("StrictModes yes\0").is_err());
}

#[test]
fn comments_require_a_separator_and_do_not_turn_unsupported_arguments_into_valid_policy() {
    let p = Policy::parse("PasswordAuthentication no # an ignored \"comment\"\n").unwrap();
    assert!(!p.password);
    assert_eq!(
        Policy::parse("PasswordAuthentication yes#not-a-separated-comment")
            .unwrap_err()
            .kind,
        Kind::Malformed
    );
    assert_eq!(
        Policy::parse("AllowUsers alex#not-a-comment")
            .unwrap_err()
            .kind,
        Kind::Unsupported
    );
    assert_eq!(
        Policy::parse("AllowUsers \"alex#name\"").unwrap_err().kind,
        Kind::Unsupported
    );
    let e = Policy::parse("StrictModes yes\0").unwrap_err();
    assert_eq!(e.kind, Kind::Unsupported);
    assert!(!e.to_string().contains("Line 0"));
}
