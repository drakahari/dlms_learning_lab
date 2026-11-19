//! Composition fixtures only; no new catalog lab or host networking operation.
use dlms_learning_lab::{
    linux_v6_admin::{Advertisement, State},
    linux_v6_connections,
    linux_v6_filter::{Protocol, Verdict},
    linux_v6_flow,
    linux_v6_path::{Gate, Header},
    linux_v6_state_filter,
};
#[path = "support/host_ipv6.rs"]
mod host_fixture;
fn state() -> State {
    let mut host = host_fixture::fixture();
    for link in &mut host.profile.links {
        link.carrier = true;
    }
    for i in &mut host.configuration.interfaces {
        i.up = true;
    }
    for g in &mut host.profile.ipv6.as_mut().unwrap().gateways {
        g.available = true;
    }
    State::new(host).unwrap()
}
fn advertisement(device: &str, lifetime: u16) -> Advertisement {
    Advertisement {
        device: device.into(),
        gateway: "fe80::1".parse().unwrap(),
        router_lifetime: lifetime,
    }
}
fn run(s: &mut State, command: &str) {
    s.command(command)
        .unwrap_or_else(|e| panic!("{command}: {e}"));
}
fn v4(s: &State) -> serde_json::Value {
    let mut value = serde_json::to_value(&s.host.configuration).unwrap();
    value.as_object_mut().unwrap().remove("ipv6");
    value
}
#[test]
fn real_supported_host_parser_drives_router_addresses_routes_and_actual_transit() {
    let mut s = state();
    let old = v4(&s);
    run(
        &mut s,
        "ip -6 route add 2001:db8:90::/64 via fe80::1 dev ens3 metric 10",
    );
    let h = Header {
        source: "2001:db8:20::20".parse().unwrap(),
        destination: "2001:db8:90::80".parse().unwrap(),
        protocol: Protocol::Tcp,
        source_port: Some(40000),
        destination_port: Some(443),
        icmp_type: None,
        icmp_code: None,
    };
    let mut flow = linux_v6_flow::State {
        filter: linux_v6_state_filter::Table::new(Verdict::Accept),
        connections: linux_v6_connections::Table::default(),
    };
    assert_eq!(
        flow.event(
            &s.facts().unwrap(),
            s.settings.forwards_from("ens4").unwrap(),
            "ens4",
            &h
        )
        .unwrap()
        .path
        .gate,
        Gate::ForwardingDisabled
    );
    run(&mut s, "sysctl -w net.ipv6.conf.all.forwarding=1");
    let actual = flow
        .event(
            &s.facts().unwrap(),
            s.settings.forwards_from("ens4").unwrap(),
            "ens4",
            &h,
        )
        .unwrap();
    assert_eq!(actual.path.gate, Gate::Allowed);
    assert_eq!(actual.path.route.unwrap().device, "ens3");
    let before = s.clone();
    assert!(
        s.command("ip -6 address del 2001:db8:10::40/64 dev ens3")
            .is_err()
    );
    assert_eq!(s, before);
    run(
        &mut s,
        "ip -6 route del 2001:db8:90::/64 via fe80::1 dev ens3 metric 10",
    );
    run(&mut s, "ip -6 address del 2001:db8:10::40/64 dev ens3");
    s.validate().unwrap();
    assert!(
        s.facts()
            .unwrap_err()
            .contains("Configure the primary IPv6 address")
    );
    let restored: State = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    restored.validate().unwrap();
    run(&mut s, "ip -6 address add 2001:db8:10::77/64 dev ens3");
    s.facts().unwrap();
    assert_eq!(v4(&s), old);
}
#[test]
fn forwarding_writes_purge_learned_defaults_unless_receiving_interface_accept_ra_is_two() {
    let mut s = state();
    let a = advertisement("ens3", 600);
    assert!(s.observe_ra(&a).unwrap().accepted);
    assert_eq!(s.learned.len(), 1);
    run(&mut s, "sysctl -w net.ipv6.conf.all.forwarding=1");
    assert!(s.learned.is_empty());
    let old = s.clone();
    let ignored = s.observe_ra(&a).unwrap();
    assert!(!ignored.accepted && !ignored.changed);
    assert_eq!(s, old);
    run(&mut s, "sysctl -w net.ipv6.conf.ens3.accept_ra=2");
    assert!(s.learned.is_empty());
    assert!(s.observe_ra(&a).unwrap().accepted);
    run(&mut s, "sysctl -w net.ipv6.conf.all.forwarding=1");
    assert_eq!(s.learned.len(), 1);
    run(&mut s, "sysctl -w net.ipv6.conf.ens3.accept_ra=0");
    assert_eq!(
        s.learned.len(),
        1,
        "a changed setting is not an invented withdrawal/timeout"
    );
    let o = s
        .command("sysctl -w net.ipv6.conf.ens4.forwarding=1")
        .unwrap();
    assert!(o.changed);
    assert!(
        s.learned.is_empty(),
        "even an idempotent local forwarding write triggers the handler's purge"
    );
}
#[test]
fn observing_a_supplied_ra_does_not_assign_addresses_or_invent_slaac_and_timers() {
    let mut s = state();
    run(&mut s, "ip -6 address del 2001:db8:10::40/64 dev ens3");
    let before = s.host.configuration.clone();
    let o = s.observe_ra(&advertisement("ens3", 600)).unwrap();
    assert!(o.accepted && o.changed);
    assert_eq!(s.host.configuration, before);
    assert!(s.facts().is_err());
    s.validate().unwrap();
    assert!(
        s.command("ip -6 route show")
            .unwrap()
            .text
            .contains("proto ra metric 1024 pref medium")
    );
    run(&mut s, "ip -6 address add 2001:db8:10::88/64 dev ens3");
    assert_eq!(s.facts().unwrap().routes.len(), 1);
    let old = s.clone();
    s.command("sysctl net.ipv6.conf.ens3.accept_ra").unwrap();
    assert_eq!(s, old, "read operations do not advance an invented clock");
}
#[test]
fn zero_lifetime_withdraws_only_a_matching_accepted_learned_router() {
    let mut s = state();
    run(
        &mut s,
        "ip -6 route add 2001:db8:90::/64 via fe80::1 dev ens3 metric 10",
    );
    let static_routes = s.host.configuration.ipv6.as_ref().unwrap().routes.clone();
    s.observe_ra(&advertisement("ens3", 600)).unwrap();
    run(&mut s, "sysctl -w net.ipv6.conf.ens3.accept_ra=0");
    let old = s.clone();
    assert!(!s.observe_ra(&advertisement("ens3", 0)).unwrap().accepted);
    assert_eq!(s, old);
    run(&mut s, "sysctl -w net.ipv6.conf.ens3.accept_ra=2");
    let actual = s.observe_ra(&advertisement("ens3", 0)).unwrap();
    assert!(actual.accepted && actual.changed);
    assert!(s.learned.is_empty());
    assert_eq!(
        s.host.configuration.ipv6.as_ref().unwrap().routes,
        static_routes
    );
    let unchanged = s.observe_ra(&advertisement("ens3", 0)).unwrap();
    assert!(unchanged.accepted && !unchanged.changed);
    assert!(unchanged.detail.contains("no matching learned default"));
}
#[test]
fn wrong_sources_down_links_and_unsupported_equal_defaults_never_invent_a_route() {
    let mut s = state();
    let old = s.clone();
    let mut a = advertisement("ens3", 600);
    a.gateway = "fe80::bad".parse().unwrap();
    assert!(s.observe_ra(&a).is_err());
    assert_eq!(s, old);
    let mut a = advertisement("../ens3", 600);
    assert!(s.observe_ra(&a).is_err());
    assert_eq!(s, old);
    a.device = "ens3".into();
    a.gateway = "ff02::1".parse().unwrap();
    assert!(s.observe_ra(&a).is_err());
    assert_eq!(s, old);
    s.host.configuration.interfaces[0].up = false;
    s.validate().unwrap();
    let before = s.clone();
    assert!(!s.observe_ra(&advertisement("ens3", 600)).unwrap().accepted);
    assert_eq!(s, before);
    let mut s = state();
    s.observe_ra(&advertisement("ens3", 600)).unwrap();
    let old = s.clone();
    let e = s.observe_ra(&advertisement("ens4", 600)).unwrap_err();
    assert!(e.contains("Simulator route-profile limit"));
    assert_eq!(s, old);
}
#[test]
fn manual_and_learned_route_edit_limits_are_explicit_and_atomic() {
    let mut s = state();
    run(
        &mut s,
        "ip -6 route add default via fe80::1 dev ens3 metric 1024",
    );
    let old = s.clone();
    assert!(
        s.observe_ra(&advertisement("ens3", 600))
            .unwrap_err()
            .contains("Simulator limit")
    );
    assert_eq!(s, old);
    run(
        &mut s,
        "ip -6 route del default via fe80::1 dev ens3 metric 1024",
    );
    s.observe_ra(&advertisement("ens3", 600)).unwrap();
    for command in [
        "ip -6 route del default via fe80::1 dev ens3 metric 1024",
        "ip -6 route replace default via fe80::1 dev ens4 metric 1024",
    ] {
        let old = s.clone();
        assert!(s.command(command).unwrap_err().contains("Simulator limit"));
        assert_eq!(s, old);
    }
    run(
        &mut s,
        "ip -6 route add default via fe80::1 dev ens3 metric 2000",
    );
    assert_eq!(s.learned.len(), 1);
    s.facts().unwrap();
    run(
        &mut s,
        "ip -6 route del default via fe80::1 dev ens3 metric 2000",
    );
    assert_eq!(s.learned.len(), 1);
}
#[test]
fn serialization_and_untrusted_commands_cannot_enable_an_unrelated_host_operation() {
    let s = state();
    let raw = serde_json::to_vec(&s).unwrap();
    let roundtrip: State = serde_json::from_slice(&raw).unwrap();
    assert_eq!(s, roundtrip);
    roundtrip.validate().unwrap();
    for command in [
        "ip -4 address flush dev ens3",
        "ip link set dev ens3 down",
        "getent ahostsv4 example.com",
        "bash -c id",
        "cat /etc/passwd",
        "cat /proc/sys/net/ipv6/conf/../../etc/passwd",
        "sysctl -w net.ipv6.conf.all.forwarding=$(id)",
        "ip -6 address replace 2001:db8:10::77/64 dev ens3",
    ] {
        let mut next = s.clone();
        assert!(next.command(command).is_err(), "{command}");
        assert_eq!(next, s, "{command}");
    }
    let mut next = s.clone();
    assert!(next.command(&"x".repeat(513)).is_err());
    assert_eq!(next, s);
    let mut wrong = s.clone();
    wrong.version = 2;
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong.settings.interfaces[0].name = "absent".into();
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong.learned = vec![advertisement("ens3", 0)];
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong.learned = vec![advertisement("ens3", 600); 5];
    assert!(serde_json::from_value::<State>(serde_json::to_value(wrong).unwrap()).is_err());
}

#[test]
fn a_numeric_route_does_not_invent_a_supplied_neighbor_or_physical_lan() {
    let target = "2001:db8:90::80".parse().unwrap();
    let mut s = state();
    run(
        &mut s,
        "ip -6 route add 2001:db8:90::/64 via fe80::bad dev ens3 metric 10",
    );
    s.facts().unwrap();
    let actual = s.supplied_path(target).unwrap();
    assert!(!actual.pass);
    assert!(actual.reason.contains("No supplied IPv6 gateway"));
    run(
        &mut s,
        "ip -6 route replace 2001:db8:90::/64 via fe80::1 dev ens3 metric 10",
    );
    assert!(s.supplied_path(target).unwrap().pass);
    s.host.profile.ipv6.as_mut().unwrap().gateways[0].available = false;
    assert!(!s.supplied_path(target).unwrap().pass);
    assert!(
        s.supplied_path(target)
            .unwrap()
            .reason
            .contains("unavailable")
    );
    let mut s = state();
    run(&mut s, "ip -6 address del 2001:db8:10::40/64 dev ens3");
    run(&mut s, "ip -6 address add 2001:db8:10:8::40/64 dev ens3");
    run(
        &mut s,
        "ip -6 route add 2001:db8:90::/64 via fe80::1 dev ens3 metric 10",
    );
    s.facts().unwrap();
    let actual = s.supplied_path(target).unwrap();
    assert!(!actual.pass);
    assert!(actual.reason.contains("outside the supplied"));
}

#[test]
fn learned_routes_participate_in_real_supplied_path_checks_without_changing_manual_routes() {
    let target = "2001:db8:90::80".parse().unwrap();
    let mut s = state();
    assert!(!s.supplied_path(target).unwrap().pass);
    s.observe_ra(&advertisement("ens3", 600)).unwrap();
    assert!(s.supplied_path(target).unwrap().pass);
    assert!(
        !s.host.probe_ipv6(target).pass,
        "the effective view must include actual learned state"
    );
    assert!(
        s.host
            .configuration
            .ipv6
            .as_ref()
            .unwrap()
            .routes
            .is_empty()
    );
    run(&mut s, "sysctl -w net.ipv6.conf.all.forwarding=1");
    assert!(!s.supplied_path(target).unwrap().pass);
    run(&mut s, "sysctl -w net.ipv6.conf.ens3.accept_ra=2");
    assert!(
        !s.supplied_path(target).unwrap().pass,
        "a flag change is not a fabricated new packet"
    );
    s.observe_ra(&advertisement("ens3", 600)).unwrap();
    assert!(s.supplied_path(target).unwrap().pass);
    let before = s.clone();
    s.supplied_path(target).unwrap();
    assert_eq!(s, before);
}
