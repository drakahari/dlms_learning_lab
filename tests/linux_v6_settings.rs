use dlms_learning_lab::linux_v6_settings::{Interface, State};
fn state() -> State {
    State {
        version: 1,
        global_forwarding: false,
        interfaces: ["lan0", "wan0"]
            .into_iter()
            .map(|name| Interface {
                name: name.into(),
                forwarding: false,
                force_forwarding: false,
                accept_ra: 1,
            })
            .collect(),
    }
}
fn write(s: &mut State, name: &str, value: u8) {
    let o = s
        .command(&format!("sysctl -w net.ipv6.conf.{name}={value}"))
        .unwrap();
    assert_eq!(o.text, format!("net.ipv6.conf.{name} = {value}\n"));
}
#[test]
fn global_forwarding_changes_host_router_flags_and_ra_acceptance() {
    let mut s = state();
    assert!(s.accepts_ra("wan0").unwrap());
    write(&mut s, "all.forwarding", 1);
    for i in ["lan0", "wan0"] {
        assert!(s.forwards_from(i).unwrap());
        assert!(!s.accepts_ra(i).unwrap());
    }
    assert!(s.interfaces.iter().all(|i| i.forwarding));
    write(&mut s, "wan0.accept_ra", 2);
    assert!(s.accepts_ra("wan0").unwrap());
    assert!(!s.accepts_ra("lan0").unwrap());
    write(&mut s, "all.forwarding", 0);
    assert!(s.interfaces.iter().all(|i| !i.forwarding));
    assert!(s.accepts_ra("lan0").unwrap());
    assert_eq!(s.interfaces[1].accept_ra, 2);
}
#[test]
fn per_interface_host_router_flag_is_not_a_global_transit_gate() {
    let mut s = state();
    write(&mut s, "lan0.forwarding", 1);
    assert!(!s.forwards_from("lan0").unwrap());
    assert!(!s.accepts_ra("lan0").unwrap());
    write(&mut s, "all.forwarding", 1);
    write(&mut s, "lan0.forwarding", 0);
    assert!(s.forwards_from("lan0").unwrap());
    assert!(s.accepts_ra("lan0").unwrap());
    assert!(!s.accepts_ra("wan0").unwrap());
}
#[test]
fn force_forwarding_is_ingress_scoped_and_global_off_resets_it_even_if_already_off() {
    let mut s = state();
    write(&mut s, "wan0.force_forwarding", 1);
    assert!(s.forwards_from("wan0").unwrap());
    assert!(!s.forwards_from("lan0").unwrap());
    assert!(s.accepts_ra("wan0").unwrap());
    let o = s
        .command("sysctl -w net.ipv6.conf.all.forwarding=0")
        .unwrap();
    assert!(o.changed);
    assert!(!s.interfaces[1].force_forwarding);
    assert!(!s.forwards_from("wan0").unwrap());
    write(&mut s, "wan0.force_forwarding", 1);
    write(&mut s, "all.forwarding", 1);
    assert!(s.interfaces[1].force_forwarding);
    write(&mut s, "all.forwarding", 0);
    assert!(s.interfaces.iter().all(|i| !i.force_forwarding));
}
#[test]
fn ra_matrix_uses_stored_accept_ra_and_local_flag_without_inventing_routes() {
    for local in [false, true] {
        for ra in 0..=2 {
            let mut s = state();
            s.interfaces[1].forwarding = local;
            s.interfaces[1].accept_ra = ra;
            let old = s.clone();
            assert_eq!(
                s.accepts_ra("wan0").unwrap(),
                ra == 2 || (ra == 1 && !local)
            );
            assert_eq!(s, old);
        }
    }
}
#[test]
fn reads_and_idempotent_writes_are_literal_bounded_and_do_not_change_state() {
    let mut s = state();
    let old = s.clone();
    assert_eq!(
        s.command("cat '/proc/sys/net/ipv6/conf/wan0/accept_ra'")
            .unwrap()
            .text,
        "1\n"
    );
    let o = s.command("sysctl net.ipv6.conf.wan0.accept_ra").unwrap();
    assert_eq!(o.text, "net.ipv6.conf.wan0.accept_ra = 1\n");
    assert!(!o.changed);
    assert!(
        !s.command("sysctl -w 'net.ipv6.conf.wan0.accept_ra=1'")
            .unwrap()
            .changed
    );
    assert_eq!(s, old);
}
#[test]
fn unsupported_syntax_paths_and_invalid_ranges_are_atomic() {
    for cmd in [
        "sysctl -w net.ipv6.conf.all.forwarding=2",
        "sysctl -w net.ipv6.conf.wan0.accept_ra=3",
        "sysctl -w net.ipv6.conf.wan0.accept_ra=-1",
        "sysctl -w net.ipv6.conf.default.forwarding=1",
        "sysctl -w net.ipv6.conf.all.accept_ra=2",
        "sysctl -w net.ipv6.conf.absent.forwarding=1",
        "sysctl -w net.ipv6.conf.wan0.disable_ipv6=1",
        "sysctl -w net.ipv6.conf.wan0.accept_ra=2 extra",
        "sysctl -a",
        "cat /etc/passwd",
        "cat /proc/sys/net/ipv6/conf/../wan0/accept_ra",
        "cat /proc/sys/net/ipv6/conf/wan0/../../etc/passwd",
        "cat /proc/sys/net/ipv6/conf/wan0/accept_ra > /etc/x",
        "sysctl -w net.ipv6.conf.wan0.accept_ra=$(id)",
        "sysctl -w net.ipv6.conf.wan0.accept_ra=2; id",
    ] {
        let mut s = state();
        let old = s.clone();
        assert!(s.command(cmd).is_err(), "{cmd}");
        assert_eq!(s, old, "{cmd}");
    }
    let mut s = state();
    let old = s.clone();
    assert!(s.command(&"x".repeat(2049)).is_err());
    assert_eq!(s, old);
}
#[test]
fn unknown_ingress_is_rejected_even_when_global_forwarding_is_on() {
    let mut s = state();
    write(&mut s, "all.forwarding", 1);
    assert!(s.forwards_from("absent").is_err());
    assert!(s.accepts_ra("absent").is_err());
}
#[test]
fn model_and_serialization_reject_unknown_fields_invalid_names_and_excess_interfaces() {
    let s = state();
    let text = serde_json::to_string(&s).unwrap();
    let restored: State = serde_json::from_str(&text).unwrap();
    assert_eq!(restored, s);
    restored.validate().unwrap();
    let mut value = serde_json::to_value(&s).unwrap();
    value["host_path"] = serde_json::json!("/proc/sys");
    assert!(serde_json::from_value::<State>(value).is_err());
    for name in ["all", "default", "../wan0", "eth.1", ""] {
        let mut bad = s.clone();
        bad.interfaces[0].name = name.into();
        assert!(bad.validate().is_err());
    }
    let mut bad = s.clone();
    bad.interfaces[0].accept_ra = 3;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.interfaces[1].name = "lan0".into();
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.interfaces = vec![s.interfaces[0].clone(); 9];
    assert!(bad.validate().is_err());
    let value = serde_json::to_value(bad).unwrap();
    assert!(serde_json::from_value::<State>(value).is_err());
}

#[test]
fn validated_forwarding_write_events_are_distinct_from_state_changes() {
    let mut s = state();
    let o = s
        .command("sysctl -w net.ipv6.conf.all.forwarding=0")
        .unwrap();
    assert!(!o.changed);
    assert_eq!(o.forwarding_written, Some(false));
    write(&mut s, "all.forwarding", 1);
    let o = s
        .command("sysctl -w net.ipv6.conf.all.forwarding=1")
        .unwrap();
    assert!(!o.changed);
    assert_eq!(o.forwarding_written, Some(true));
    let o = s
        .command("sysctl -w net.ipv6.conf.wan0.forwarding=1")
        .unwrap();
    assert!(!o.changed);
    assert_eq!(o.forwarding_written, Some(true));
    assert_eq!(
        s.command("sysctl -w net.ipv6.conf.wan0.force_forwarding=1")
            .unwrap()
            .forwarding_written,
        None
    );
    assert_eq!(
        s.command("sysctl -w net.ipv6.conf.wan0.accept_ra=2")
            .unwrap()
            .forwarding_written,
        None
    );
    assert_eq!(
        s.command("sysctl net.ipv6.conf.all.forwarding")
            .unwrap()
            .forwarding_written,
        None
    );
}
