use dlms_learning_lab::{linux_v6_admin as core, linux_v6_saved_settings as file};
#[path = "support/host_ipv6.rs"]
mod fixture;
fn state() -> core::State {
    core::State::new(fixture::fixture()).unwrap()
}
#[test]
fn whitespace_comments_duplicate_keys_and_decimal_forms_apply_in_file_order() {
    let mut s = state();
    let text = "  # first comment\n; second\nnet.ipv6.conf.all.forwarding = 0\nnet.ipv6.conf.ens3.accept_ra=02\nnet.ipv6.conf.all.forwarding=01\n";
    let assignments = file::parse(&s.settings, text).unwrap();
    assert_eq!(assignments.len(), 3);
    let out = file::apply(&mut s, text).unwrap();
    assert!(s.settings.global_forwarding);
    assert_eq!(s.settings.interfaces[0].accept_ra, 2);
    assert!(out.contains("net.ipv6.conf.all.forwarding = 0\n"));
    assert!(out.ends_with("net.ipv6.conf.all.forwarding = 1\n"));
    let old = s.clone();
    assert_eq!(file::apply(&mut s, "# comments only\n\n; end").unwrap(), "");
    assert_eq!(s, old);
}
#[test]
fn invalid_or_unsupported_files_are_refused_before_any_simulated_application() {
    for bad in [
        "net.ipv6.conf.all.forwarding = 1\nvm.swappiness=0\n",
        "net.ipv6.conf.all.forwarding=1\nnet.ipv6.conf.ens3.accept_ra=3\n",
        "include /etc/sysctl.conf",
        "net.ipv6.conf.*.accept_ra=2",
        "-net.ipv6.conf.absent.accept_ra=2",
        "net.ipv6.conf.all.forwarding=$(id)",
        "net.ipv6.conf.all.forwarding=1 # inline",
        "net.ipv6.conf.all.forwarding=\"1\"",
    ] {
        let mut s = state();
        let old = s.clone();
        let e = file::apply(&mut s, bad).unwrap_err();
        assert!(e.line.is_some(), "{e}");
        assert_eq!(s, old, "{bad}");
    }
}
#[test]
fn file_size_lines_controls_and_values_stay_bounded() {
    let s = state();
    for text in [
        "x".repeat(4097),
        "#\n".repeat(25),
        format!("#{}", "x".repeat(256)),
        "#bad\u{0}".into(),
        "net.ipv6.conf.all.forwarding=256".into(),
    ] {
        assert!(file::parse(&s.settings, &text).is_err());
    }
}
#[test]
fn fresh_settings_do_not_copy_live_flags_learned_routes_or_assign_a_new_address() {
    let mut s = state();
    s.command("sysctl -w net.ipv6.conf.all.forwarding=1")
        .unwrap();
    s.command("sysctl -w net.ipv6.conf.ens3.accept_ra=2")
        .unwrap();
    s.observe_ra(&core::Advertisement {
        device: "ens3".into(),
        gateway: "fe80::1".parse().unwrap(),
        router_lifetime: 600,
    })
    .unwrap();
    let old = s.clone();
    let fresh = file::fresh(&s, "# no persistent settings yet\n").unwrap();
    assert!(!fresh.settings.global_forwarding);
    assert!(fresh.learned.is_empty());
    assert_eq!(fresh.host, s.host);
    assert_eq!(s, old);
    let configured = file::fresh(
        &s,
        "net.ipv6.conf.all.forwarding=1\nnet.ipv6.conf.ens3.accept_ra=2\n",
    )
    .unwrap();
    assert!(configured.settings.global_forwarding);
    assert!(
        configured.learned.is_empty(),
        "a saved flag still needs a new supplied advertisement"
    );
    assert!(configured.settings.accepts_ra("ens3").unwrap());
}
