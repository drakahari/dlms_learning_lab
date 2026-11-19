//! Production-linked exposure model checks; no scanner/network/host execution.
use dlms_learning_lab::service_exposure as audit;

#[path = "support/service_exposure_fixture.rs"]
mod fixture;
use fixture::fixture;

fn state(w: &audit::World, view: usize, port: u16) -> audit::PortState {
    w.scan(view, &[port]).unwrap()[0].state
}
#[test]
fn derives_open_closed_and_remote_loopback_distinction_without_mutation() {
    let w = fixture();
    let before = w.clone();
    assert_eq!(state(&w, 0, 22), audit::PortState::Open);
    assert_eq!(state(&w, 0, 443), audit::PortState::Open);
    assert_eq!(state(&w, 0, 80), audit::PortState::Closed);
    assert_eq!(state(&w, 0, 9001), audit::PortState::Closed);
    assert_eq!(w, before);
}
#[test]
fn observations_are_viewpoint_specific_and_filtered_does_not_prove_closed() {
    let mut w = fixture();
    w.admin_command("iptables -A INPUT -s 192.0.2.0/24 -p tcp --dport 22 -j DROP")
        .unwrap();
    assert_eq!(state(&w, 0, 22), audit::PortState::Open);
    assert_eq!(state(&w, 1, 22), audit::PortState::Filtered);
    assert_eq!(state(&w, 1, 443), audit::PortState::Open);
    w.admin_command("kill -TERM 201").unwrap();
    assert_eq!(state(&w, 0, 22), audit::PortState::Closed);
    assert_eq!(state(&w, 1, 22), audit::PortState::Filtered);
}
#[test]
fn dropped_replies_filter_open_and_closed_ports() {
    let mut w = fixture();
    w.admin_command("iptables -P OUTPUT DROP").unwrap();
    assert_eq!(state(&w, 0, 22), audit::PortState::Filtered);
    assert_eq!(state(&w, 0, 80), audit::PortState::Filtered);
}
#[test]
fn stopped_and_pending_owner_keeps_kernel_listener_until_continued_termination() {
    let mut w = fixture();
    w.admin_command("kill -STOP 101").unwrap();
    assert_eq!(state(&w, 0, 9000), audit::PortState::Open);
    w.admin_command("kill -TERM 101").unwrap();
    assert_eq!(state(&w, 0, 9000), audit::PortState::Open);
    w.admin_command("kill -CONT 101").unwrap();
    assert_eq!(state(&w, 0, 9000), audit::PortState::Closed);
    assert_eq!(state(&w, 0, 22), audit::PortState::Open);
    let out = w.admin_command("ss -lntp").unwrap();
    assert!(out.text.contains("127.0.0.1:9001"));
}
#[test]
fn scanner_options_reorder_and_numeric_port_order_is_stable() {
    let w = fixture();
    let a = w
        .scan_command(0, "nmap -n -Pn -sT -p 443,22,80 203.0.113.10")
        .unwrap();
    let b = w
        .scan_command(0, "nmap 203.0.113.10 -sT -p 80,443,22 -Pn -n")
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.iter().map(|x| x.port).collect::<Vec<_>>(), [22, 80, 443]);
    let text = audit::report(&w, 0, &a).unwrap();
    assert!(text.contains("admin (198.51.100.20)"));
    assert!(!text.contains("vulnerable"));
}
#[test]
fn unsupported_scan_modes_and_shell_expressions_fail_honestly() {
    let w = fixture();
    for line in [
        "nmap -sS -p 22 203.0.113.10",
        "nmap -n -Pn -sT -sV -p 22 203.0.113.10",
        "nmap --script safe 203.0.113.10",
        "nmap -n -Pn -sT -p 1-65535 203.0.113.10",
        "nmap -n -Pn -sT -p ssh 203.0.113.10",
        "nmap -n -Pn -sT -p 22 203.0.113.10; touch x",
        "nmap -n -Pn -sT -p $(echo 22) 203.0.113.10",
    ] {
        assert!(w.scan_command(0, line).is_err(), "{line}");
    }
}
#[test]
fn arbitrary_targets_hostnames_observers_and_path_arguments_are_refused() {
    let w = fixture();
    for target in [
        "example.com",
        "127.0.0.1",
        "198.51.100.20",
        "/etc/passwd",
        "../../x",
    ] {
        assert!(
            w.scan_command(0, &format!("nmap -n -Pn -sT -p 22 {target}"))
                .is_err()
        );
    }
    assert!(w.scan(2, &[22]).is_err());
}
#[test]
fn limits_duplicate_flags_bad_ports_and_multiline_inputs_are_not_silently_accepted() {
    let w = fixture();
    for ports in [vec![], vec![0], vec![22, 22], (1..=33).collect()] {
        assert!(w.scan(0, &ports).is_err());
    }
    for line in [
        "nmap -n -n -Pn -sT -p 22 203.0.113.10",
        "nmap -n -Pn -sT -p 65536 203.0.113.10",
        "nmap -n -Pn -sT -p 22 203.0.113.10\n",
    ] {
        assert!(w.scan_command(0, line).is_err());
    }
    assert!(w.scan_command(0, &"x".repeat(1025)).is_err());
}
#[test]
fn source_identity_pins_processes_sockets_server_and_observer_positions() {
    let original = fixture();
    let mut w = original.clone();
    w.admin_command("kill -TERM 101").unwrap();
    assert!(w.same_problem(&original));
    w.processes.listeners[0].port = 9999;
    assert!(!w.same_problem(&original));
    let mut w = original.clone();
    w.server = "203.0.113.11".parse().unwrap();
    assert!(!w.same_problem(&original));
    let mut w = original.clone();
    w.viewpoints[0].address = "198.51.100.21".parse().unwrap();
    assert!(!w.same_problem(&original));
}
#[test]
fn valid_partial_process_failure_preserves_real_effects_and_exit_status() {
    let mut w = fixture();
    let out = w.admin_command("kill -TERM 101 999").unwrap();
    assert_ne!(out.status, 0);
    assert!(out.changed);
    assert_eq!(state(&w, 0, 9000), audit::PortState::Closed);
    assert_eq!(state(&w, 0, 443), audit::PortState::Open);
}
#[test]
fn malformed_world_and_unknown_fields_are_rejected() {
    let w = fixture();
    let mut x = w.clone();
    x.viewpoints.push(x.viewpoints[0].clone());
    assert!(x.validate().is_err());
    let mut x = w.clone();
    x.processes.listeners[0].pid = 999;
    assert!(x.validate().is_err());
    let mut x = w.clone();
    x.device = "../host".into();
    assert!(x.validate().is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["host_command"] = serde_json::json!("touch nope");
    assert!(serde_json::from_value::<audit::World>(value).is_err());
    let wire = serde_json::to_string(&w).unwrap();
    assert_eq!(serde_json::from_str::<audit::World>(&wire).unwrap(), w);
}

#[test]
fn server_commands_cannot_escape_the_supported_context_and_failed_edits_are_atomic() {
    let mut w = fixture();
    for input in [
        "sh -c echo hello",
        "sudo iptables -P INPUT DROP",
        "ss -lntp; touch outside",
        "iptables -A INPUT -i eth9 -j DROP",
        "cat /etc/passwd",
    ] {
        let before = w.clone();
        assert!(w.admin_command(input).is_err(), "{input}");
        assert_eq!(w, before);
    }
    w.processes.user = dlms_learning_lab::processes::Identity {
        name: "alex".into(),
        uid: 1000,
    };
    let before = w.clone();
    assert!(w.admin_command("iptables -P INPUT DROP").is_err());
    assert_eq!(w, before);
    let out = w.admin_command("kill -TERM 201").unwrap();
    assert_ne!(out.status, 0);
    assert!(!out.changed);
    assert_eq!(w, before);
}
