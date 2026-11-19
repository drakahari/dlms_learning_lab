//! Focused virtual transport/client tests; no real network or utility runs.
use dlms_learning_lab::{
    linux_filter::Class,
    service_exposure::PortState,
    telnet_client::{self as client, Mode},
};
#[path = "support/service_exposure_fixture.rs"]
mod fixture;
#[test]
fn explicit_numeric_forms_and_port_bounds_are_clear() {
    let expected = client::parse("telnet -c 203.0.113.10 443").unwrap();
    assert_eq!(client::parse("telnet 203.0.113.10 443").unwrap(), expected);
    for p in [1, 65535] {
        assert_eq!(
            client::parse(&format!("telnet -c 203.0.113.10 {p}"))
                .unwrap()
                .1,
            p
        );
    }
    for cmd in [
        "telnet",
        "telnet -c 203.0.113.10",
        "telnet -c 203.0.113.10 443 80",
    ] {
        assert_eq!(client::parse(cmd).unwrap_err().class, Class::Syntax);
    }
}
#[test]
fn unsafe_unsupported_or_malformed_forms_never_reach_a_host() {
    for cmd in [
        "telnet -n /etc/private 203.0.113.10 443",
        "telnet -e ! 203.0.113.10 443",
        "telnet -cc 203.0.113.10 443",
        "telnet -c example.org 443",
        "telnet -c ::1 443",
        "telnet -c 203.0.113.10 https",
        "telnet -c 203.0.113.10 0443",
        "telnet -c 203.0.113.10 443; id",
        "telnet -c $(cat target) 443",
        "telnet -c 203.0.113.10 443 >/tmp/private",
    ] {
        assert_eq!(
            client::parse(cmd).unwrap_err().class,
            Class::Unsupported,
            "{cmd}"
        );
    }
    for cmd in [
        "telnet -c 203.0.113.10 0".into(),
        "telnet -c 203.0.113.10 65536".into(),
        "x".repeat(1025),
        "telnet -c 203.0.113.10 443\n".into(),
    ] {
        assert_eq!(
            client::parse(&cmd).unwrap_err().class,
            Class::Limit,
            "{cmd}"
        );
    }
}
#[test]
fn open_transport_is_an_ongoing_client_not_a_fabricated_exit0() {
    let world = fixture::fixture();
    let original = world.clone();
    let r = client::run(&world, 0, Mode::Idle, "telnet -c 203.0.113.10 443").unwrap();
    assert_eq!(r.mode, Mode::Connected { port: 443 });
    assert_eq!(r.exit_status, None);
    assert_eq!(r.observations.unwrap()[0].state, PortState::Open);
    assert!(r.output.contains("Simulated"));
    assert!(r.output.contains("application data are not modeled"));
    assert_eq!(world, original);
}
#[test]
fn refusal_and_drop_are_distinct_completed_negative_attempts() {
    let mut w = fixture::fixture();
    let refused = client::run(&w, 0, Mode::Idle, "telnet -c 203.0.113.10 80").unwrap();
    assert_eq!(refused.mode, Mode::Idle);
    assert_eq!(refused.exit_status, Some(1));
    assert_eq!(refused.observations.unwrap()[0].state, PortState::Closed);
    assert!(refused.output.contains("Connection refused"));
    w.filter
        .command("iptables -A INPUT -s 198.51.100.20 -p tcp --dport 443 -j DROP")
        .unwrap();
    let dropped = client::run(&w, 0, Mode::Idle, "telnet -c 203.0.113.10 443").unwrap();
    assert_eq!(dropped.mode, Mode::Idle);
    assert_eq!(dropped.exit_status, Some(1));
    assert_eq!(dropped.observations.unwrap()[0].state, PortState::Filtered);
    assert!(dropped.output.contains("modeled timeout"));
    assert_eq!(
        client::run(&w, 1, Mode::Idle, "telnet -c 203.0.113.10 443")
            .unwrap()
            .mode,
        Mode::Connected { port: 443 }
    );
}
#[test]
fn escape_then_local_quit_closes_the_client_without_new_port_evidence() {
    let w = fixture::fixture();
    let connected = client::run(&w, 0, Mode::Idle, "telnet -c 203.0.113.10 443")
        .unwrap()
        .mode;
    assert_eq!(
        client::run(&w, 0, connected, "quit").unwrap_err().class,
        Class::Unsupported
    );
    assert_eq!(
        client::run(&w, 0, connected, "^]").unwrap_err().class,
        Class::Unsupported
    );
    let local = client::escape(connected).unwrap();
    assert_eq!(local.mode, Mode::Command { port: 443 });
    assert_eq!(local.exit_status, None);
    assert!(local.observations.is_none());
    let quit = client::run(&w, 0, local.mode, "quit").unwrap();
    assert_eq!(quit.mode, Mode::Idle);
    assert_eq!(quit.exit_status, Some(0));
    assert!(quit.observations.is_none());
    assert!(client::escape(Mode::Idle).is_err());
    assert!(client::escape(local.mode).is_err());
}
#[test]
fn payload_and_local_shell_commands_do_not_silently_succeed() {
    let w = fixture::fixture();
    for text in [
        "GET / HTTP/1.0",
        "!sh",
        "cat /etc/passwd",
        "send escape",
        "open 203.0.113.10 80",
    ] {
        assert!(client::run(&w, 0, Mode::Connected { port: 443 }, text).is_err());
        assert!(client::run(&w, 0, Mode::Command { port: 443 }, text).is_err());
    }
}
#[test]
fn only_the_supplied_server_and_observers_can_be_used() {
    let w = fixture::fixture();
    assert_eq!(
        client::run(&w, 0, Mode::Idle, "telnet -c 192.0.2.99 443")
            .unwrap_err()
            .class,
        Class::Context
    );
    assert!(client::run(&w, 999, Mode::Idle, "telnet -c 203.0.113.10 443").is_err());
    assert_eq!(
        client::parse("telnet -c 203.0.113.10 23")
            .unwrap_err()
            .class,
        Class::Unsupported
    );
    for mode in [Mode::Connected { port: 0 }, Mode::Command { port: 23 }] {
        assert!(mode.validate().is_err());
        assert!(client::escape(mode).is_err());
        assert!(client::run(&w, 0, mode, "quit").is_err());
    }
}
#[test]
fn stopped_listener_transport_differs_from_an_ended_owner() {
    let mut w = fixture::fixture();
    w.admin_command("kill -STOP 101").unwrap();
    assert_eq!(
        client::run(&w, 0, Mode::Idle, "telnet -c 203.0.113.10 9000")
            .unwrap()
            .mode,
        Mode::Connected { port: 9000 }
    );
    w.admin_command("kill -KILL 101").unwrap();
    let r = client::run(&w, 0, Mode::Idle, "telnet -c 203.0.113.10 9000").unwrap();
    assert_eq!(r.mode, Mode::Idle);
    assert_eq!(r.exit_status, Some(1));
}
