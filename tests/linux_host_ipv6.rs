//! Source-only IPv6 extension proof. No new authored lab, packet or host command.
use dlms_learning_lab::{ipv6::Prefix, linux_host as host};
use serde_json::{Value, json};
use std::net::Ipv6Addr;
fn ip(s: &str) -> Ipv6Addr {
    s.parse().unwrap()
}
#[path = "support/host_ipv6.rs"]
mod f;
use f::fixture;
fn run(w: &mut host::State, c: &str) -> String {
    w.execute(c).unwrap_or_else(|e| panic!("{c}: {e}"))
}
fn frozen_v4(w: &host::State) -> Value {
    let mut v = serde_json::to_value(&w.configuration).unwrap();
    v.as_object_mut().unwrap().remove("ipv6");
    v
}
#[test]
fn existing_ipv4_sources_serialize_identically_and_do_not_enable_ipv6_from_saved_data() {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    for s in p["scenarios"].as_array().unwrap() {
        for v in [
            &s["goal"]["definition"]["initial"],
            &s["goal"]["definition"]["example"],
            &s["initial"]["simulation"]["assignment"]["world"],
            &s["initial"]["simulation"]["practice"]["world"],
        ] {
            let mut w: host::State = serde_json::from_value(v.clone()).unwrap();
            w.validate().unwrap();
            assert!(w.profile.ipv6.is_none() && w.configuration.ipv6.is_none());
            assert_eq!(serde_json::to_value(&w).unwrap(), *v);
            let before = w.clone();
            assert!(
                w.execute("ip -6 address show")
                    .unwrap_err()
                    .contains("Unsupported command/flags")
            );
            assert_eq!(w, before);
            assert_eq!(w.command_help(), host::HELP);
            let v6 = fixture();
            assert!(host::State::decode(&serde_json::to_vec(&v6).unwrap(), &w.profile).is_err());
        }
    }
    let mut w = fixture();
    w.profile.ipv6 = None;
    assert!(w.validate().is_err());
    let mut w = fixture();
    w.configuration.ipv6 = None;
    assert!(w.validate().is_err());
}
#[test]
fn connected_ipv6_and_explicit_scoped_gateway_routes_preserve_all_ipv4_configuration() {
    let mut w = fixture();
    let before = frozen_v4(&w);
    let generation = w.generation;
    assert!(w.probe_ipv6(ip("2001:db8:10::20")).pass);
    assert!(w.probe_ipv6(ip("2001:db8:20::20")).pass);
    assert!(!w.probe_ipv6(ip("2001:db8:90::80")).pass);
    assert!(
        run(&mut w, "ip -6 address show dev ens3")
            .contains("inet6 2001:db8:10::40/64 scope global")
    );
    assert!(run(&mut w, "ip -6 address show dev ens3").contains("inet6 fe80::a/64 scope link"));
    assert_eq!(w.generation, generation, "queries never mutate");
    run(
        &mut w,
        "ip -6 route add default via fe80::1 dev ens3 metric 100",
    );
    assert!(w.probe_ipv6(ip("2001:db8:90::80")).pass);
    let routes = run(&mut w, "ip -6 route show");
    assert!(routes.contains("default via fe80::1 dev ens3 metric 100"));
    assert!(routes.contains("2001:db8:10::/64 dev ens3 metric 256"));
    assert_eq!(frozen_v4(&w), before);
    let saved = serde_json::to_vec(&w).unwrap();
    assert_eq!(host::State::decode(&saved, &w.profile).unwrap(), w);
}
#[test]
fn longest_prefix_precedes_metric_and_unavailable_gateway_never_invents_failover() {
    let mut w = fixture();
    run(
        &mut w,
        "ip -6 route add default via fe80::1 dev ens3 metric 100",
    );
    run(
        &mut w,
        "ip -6 route add 2001:db8:90::80/128 via fe80::1 dev ens4 metric 9999",
    );
    let p = w.probe_ipv6(ip("2001:db8:90::80"));
    assert!(!p.pass);
    assert!(p.reason.contains("unavailable"));
    assert_eq!(p.selection.as_ref().unwrap().device, "ens4");
    assert_eq!(p.selection.as_ref().unwrap().destination.length, 128);
    run(
        &mut w,
        "ip -6 route del 2001:db8:90::80/128 via fe80::1 dev ens4 metric 9999",
    );
    assert!(w.probe_ipv6(ip("2001:db8:90::80")).pass);
    run(
        &mut w,
        "ip -6 route add default via fe80::1 dev ens4 metric 50",
    );
    assert!(!w.probe_ipv6(ip("2001:db8:90::80")).pass);
    run(
        &mut w,
        "ip -6 route replace default via fe80::1 dev ens3 metric 50",
    );
    assert!(w.probe_ipv6(ip("2001:db8:90::80")).pass);
    run(
        &mut w,
        "ip -6 route replace default via fe80::2 dev ens3 metric 50",
    );
    let p = w.probe_ipv6(ip("2001:db8:90::80"));
    assert!(!p.pass && p.reason.contains("scoped"));
}
#[test]
fn ipv6_does_not_apply_ipv4_broadcast_rules_and_mask_errors_explain_the_actual_path() {
    for address in [
        "2001:db8:10::41",
        "2001:db8:10::f00",
        "2001:db8:10:0:ffff:ffff:ffff:ffff",
    ] {
        let mut w = fixture();
        run(&mut w, "ip -6 address del 2001:db8:10::40/64 dev ens3");
        run(&mut w, &format!("ip -6 address add {address}/64 dev ens3"));
        assert!(
            w.probe_ipv6(ip("2001:db8:10::20")).pass,
            "IPv6 has no broadcast: {address}"
        );
    }
    let mut w = fixture();
    run(&mut w, "ip -6 address del 2001:db8:10::40/64 dev ens3");
    run(&mut w, "ip -6 address add 2001:db8:10::40/60 dev ens3");
    let p = w.probe_ipv6(ip("2001:db8:10:8::80"));
    assert!(!p.pass);
    assert!(p.reason.contains("physical LAN"), "{p:?}");
    assert!(
        w.probe_ipv6(ip("2001:db8:10::20")).pass,
        "A broad mask does not magically break every local path"
    );
    run(&mut w, "ip -6 address del 2001:db8:10::40/60 dev ens3");
    run(&mut w, "ip -6 address add 2001:db8:10::20/64 dev ens3");
    assert!(
        w.probe_ipv6(ip("2001:db8:10::20"))
            .reason
            .contains("conflicts")
    );
    for (a, prefix, want) in [
        ("2001:db8:1234:abcd:ef00::1", 48, "2001:db8:1234::/48"),
        ("2001:db8:10:3::1", 63, "2001:db8:10:2::/63"),
        ("2001:db8:10::ffff", 128, "2001:db8:10::ffff/128"),
    ] {
        assert_eq!(Prefix::new(ip(a), prefix).unwrap().text(), want);
    }
}
#[test]
fn unsupported_syntax_resource_limits_and_mutation_failures_are_atomic() {
    let commands = [
        "ip -6 address flush dev ens3",
        "ip -6 address replace 2001:db8:10::41/64 dev ens3",
        "ip -6 route add default via fe80::1 dev ens3",
        "ip -6 route add default via fe80::1 dev ens3 metric 0",
        "ip -6 route add default via fe80::1 dev ens3 metric -1",
        "ip -6 route add default via fe80::1 dev ens3 metric 4294967296",
        "ip -6 route add 2001:db8:90::1/64 via fe80::1 dev ens3 metric 100",
        "ip -6 route add ff00::/8 via fe80::1 dev ens3 metric 100",
        "ip -6 route add default via fe80::1%ens3 dev ens3 metric 100",
        "ip -6 route add default via 2001:db8:ff::1 dev ens3 metric 100",
        "ip -6 address show; touch /tmp/escape",
        "ip -6 address add $(anything) dev ens3",
        "ip -6 route get 2001:db8:90::80",
        "ip -6 address show dev ../../etc/passwd",
    ];
    for c in commands {
        let mut w = fixture();
        let before = w.clone();
        assert!(w.execute(c).is_err(), "{c}");
        assert_eq!(w, before, "{c}");
    }
    let mut w = fixture();
    w.generation = u64::MAX;
    let before = w.clone();
    assert!(
        w.execute("ip -6 route add default via fe80::1 dev ens3 metric 100")
            .is_err()
    );
    assert_eq!(w, before);
    let mut w = fixture();
    run(
        &mut w,
        "ip -6 route add default via fe80::1 dev ens3 metric 100",
    );
    let before = w.clone();
    assert!(
        w.execute("ip -6 address del 2001:db8:10::40/64 dev ens3")
            .is_err()
    );
    assert_eq!(w, before);
    let mut w = fixture();
    run(&mut w, "ip -6 address del 2001:db8:10::40/64 dev ens3");
    let before = w.clone();
    for cidr in [
        "2001:db8:10::/64",
        "::1/64",
        "ff02::1/64",
        "fe80::2/64",
        "2001:db8:10::41/127",
        "2001:db8:10::41/129",
    ] {
        assert!(
            w.execute(&format!("ip -6 address add {cidr} dev ens3"))
                .is_err()
        );
        assert_eq!(w, before);
    }
    let mut v = serde_json::to_value(fixture()).unwrap();
    let node = v["profile"]["ipv6"]["nodes"][0].clone();
    v["profile"]["ipv6"]["nodes"] = json!(vec![node; 9]);
    assert!(serde_json::from_value::<host::State>(v).is_err());
}
#[test]
fn admin_and_carrier_state_are_shared_but_new_ipv6_commands_cannot_change_ipv4() {
    let mut w = fixture();
    let before = frozen_v4(&w);
    run(
        &mut w,
        "ip -6 route add default via fe80::1 dev ens3 metric 4294967295",
    );
    assert_eq!(frozen_v4(&w), before);
    run(&mut w, "ip link set dev ens3 down");
    assert!(
        w.probe_ipv6(ip("2001:db8:90::80"))
            .reason
            .contains("administratively down")
    );
    run(&mut w, "ip link set dev ens3 up");
    assert!(w.probe_ipv6(ip("2001:db8:90::80")).pass);
    w.profile.links[0].carrier = false;
    assert!(
        w.probe_ipv6(ip("2001:db8:90::80"))
            .reason
            .contains("carrier")
    );
    assert!(w.execute("help").unwrap().contains("IPv6 extension"));
    assert!(w.execute("help").unwrap().contains("positive u32 metric"));
}
