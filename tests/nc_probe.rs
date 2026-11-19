use dlms_learning_lab::{linux_filter::Class, nc_probe, service_exposure::PortState};
#[path = "support/service_exposure_fixture.rs"]
mod fixture;

#[test]
fn grouped_and_separate_flags_parse_as_the_same_numeric_query() {
    let expected = nc_probe::parse("nc -n -v -z 203.0.113.10 443").unwrap();
    for command in [
        "nc -nvz 203.0.113.10 443",
        "nc -zvn 203.0.113.10 443",
        "nc -zn -v 203.0.113.10 443",
        "  nc\t-v -z -n 203.0.113.10 443  ",
    ] {
        assert_eq!(nc_probe::parse(command).unwrap(), expected);
    }
    assert_eq!(expected.port, 443);
    assert_eq!(expected.target.to_string(), "203.0.113.10");
}
#[test]
fn unsupported_modes_cannot_be_mistaken_for_zero_io_tcp_probes() {
    for command in [
        "nc 203.0.113.10 443",
        "nc -nv 203.0.113.10 443",
        "nc -vz 203.0.113.10 443",
        "nc -nuvz 203.0.113.10 443",
        "nc -lnvz 203.0.113.10 443",
        "nc -nvz -e sh 203.0.113.10 443",
        "nc --exec sh 203.0.113.10 443",
        "nc -nvz -p 1234 203.0.113.10 443",
        "nc -nvz -w 1 203.0.113.10 443",
        "nc -nvz -s 192.0.2.1 203.0.113.10 443",
        "nc -nnvz 203.0.113.10 443",
        "ncat -nvz 203.0.113.10 443",
        "nc -nvz server.example 443",
        "nc -nvz 2001:db8::1 443",
        "nc -nvz 203.0.113.10 https",
        "nc -nvz 203.0.113.10 22-443",
        "nc -nvz 203.0.113.10 0443",
        "nc -nvz 203.0.113.10 443; id",
        "nc -nvz 203.0.113.10 443 | sh",
        "nc -nvz $(cat target) 443",
        "nc -nvz 203.0.113.10 443 < /etc/passwd",
    ] {
        assert_eq!(
            nc_probe::parse(command).unwrap_err().class,
            Class::Unsupported,
            "{command}"
        );
    }
}
#[test]
fn malformed_operands_and_resource_limits_return_diagnostics() {
    for command in [
        "nc -nvz",
        "nc -nvz 203.0.113.10",
        "nc -nvz 203.0.113.10 443 80",
    ] {
        assert_eq!(
            nc_probe::parse(command).unwrap_err().class,
            Class::Syntax,
            "{command}"
        );
    }
    for command in [
        "nc -nvz 203.0.113.10 0".into(),
        "nc -nvz 203.0.113.10 65536".into(),
        "x".repeat(1025),
        "nc -nvz 203.0.113.10 443\n".into(),
        "nc -nvz 203.0.113.10 443\0".into(),
    ] {
        assert_eq!(
            nc_probe::parse(&command).unwrap_err().class,
            Class::Limit,
            "{command}"
        );
    }
}
#[test]
fn real_virtual_listeners_and_closed_ports_determine_output_and_status() {
    let w = fixture::fixture();
    let before = w.clone();
    for port in [22, 443, 9000] {
        let r = nc_probe::probe(&w, 0, &format!("nc -nvz {} {port}", w.server)).unwrap();
        assert_eq!(r.status, 0);
        assert_eq!(r.observations[0].state, PortState::Open);
        assert!(r.output.contains("succeeded!"));
        assert!(r.output.contains("Simulated nc"));
    }
    for port in [80, 9001] {
        let r = nc_probe::probe(&w, 0, &format!("nc -n -z -v {} {port}", w.server)).unwrap();
        assert_eq!(r.status, 1);
        assert_eq!(r.observations[0].state, PortState::Closed);
        assert!(r.output.contains("Connection refused"));
    }
    assert_eq!(w, before);
}
#[test]
fn observer_specific_input_and_output_drops_are_not_connection_refusals() {
    let mut w = fixture::fixture();
    w.filter
        .command("iptables -A INPUT -s 192.0.2.30 -p tcp --dport 443 -j DROP")
        .unwrap();
    let before = w.clone();
    let command = "nc -nvz 203.0.113.10 443";
    let admin = nc_probe::probe(&w, 0, command).unwrap();
    let public = nc_probe::probe(&w, 1, command).unwrap();
    assert_eq!(admin.status, 0);
    assert_eq!(admin.observations[0].state, PortState::Open);
    assert_eq!(public.status, 1);
    assert_eq!(public.observations[0].state, PortState::Filtered);
    assert!(public.output.contains("modeled drop, not measured time"));
    assert!(!public.output.contains("Connection refused"));
    assert_eq!(w, before);
    w.filter
        .command("iptables -A OUTPUT -d 198.51.100.20 -p tcp --sport 443 -j DROP")
        .unwrap();
    assert_eq!(
        nc_probe::probe(&w, 0, command).unwrap().observations[0].state,
        PortState::Filtered
    );
}
#[test]
fn stopped_listener_can_accept_tcp_but_exited_owner_cannot() {
    let mut w = fixture::fixture();
    let command = "nc -nvz 203.0.113.10 22";
    w.admin_command("kill -STOP 201").unwrap();
    assert_eq!(
        nc_probe::probe(&w, 0, command).unwrap().observations[0].state,
        PortState::Open
    );
    w.admin_command("kill -KILL 201").unwrap();
    assert_eq!(
        nc_probe::probe(&w, 0, command).unwrap().observations[0].state,
        PortState::Closed
    );
}
#[test]
fn targets_and_viewpoints_are_limited_to_the_supplied_world() {
    let w = fixture::fixture();
    let before = w.clone();
    for command in [
        "nc -nvz 127.0.0.1 443",
        "nc -nvz 203.0.113.11 443",
        "nc -nvz 0.0.0.0 443",
    ] {
        assert_eq!(
            nc_probe::probe(&w, 0, command).unwrap_err().class,
            Class::Context
        );
    }
    assert_eq!(
        nc_probe::probe(&w, 99, "nc -nvz 203.0.113.10 443")
            .unwrap_err()
            .class,
        Class::Context
    );
    assert_eq!(w, before);
}
#[test]
fn legacy_nmap_reports_remain_identical_and_do_not_use_nc_formatting() {
    let w = fixture::fixture();
    let r = w
        .scan_command(0, "nmap -n -Pn -sT -p 443,80,22 203.0.113.10")
        .unwrap();
    let text = dlms_learning_lab::service_exposure::report(&w, 0, &r).unwrap();
    assert_eq!(
        text,
        "Simulated TCP connect results · admin (198.51.100.20) → 203.0.113.10\nPORT       STATE\n22/tcp     open\n80/tcp     closed\n443/tcp    open\nSupplied path; numeric TCP subset. No application/version/vulnerability inference.\n"
    );
    assert!(
        w.scan_command(0, "nc -nvz 203.0.113.10 443").is_err(),
        "Legacy nmap-only API stays narrow; native observer routing will opt into the new parser separately"
    );
}
