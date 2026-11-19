use dlms_learning_lab::{acl, management_path as path, management_policy as policy};
#[path = "support/management_fixture.rs"]
mod fixture;
use fixture::{binding, device, ip, network, port, route, settings};

#[test]
fn management_target_is_the_actual_routers_local_interface_and_reply_path() {
    let n = network();
    let before = n.clone();
    let p = settings();
    let original = p.clone();
    assert!(n.topology.devices["R2"].forwarding);
    let path::Observation::Policy { path, conditions } =
        n.inspect(&p, "admin", 0, policy::Protocol::Ssh2).unwrap()
    else {
        panic!("Expected actual IP path")
    };
    assert_eq!(path.source, ip("192.0.2.10"));
    assert_eq!(path.forward.destination_device, "R2");
    assert_eq!(path.reply.destination_device, "Client");
    assert_eq!(path.forward.hops.len(), 2);
    assert!(conditions.allowed_to_authenticate());
    let guest = n.inspect(&p, "guest", 0, policy::Protocol::Ssh2).unwrap();
    let path::Observation::Policy { conditions, .. } = guest else {
        panic!("Guest also has an IP path")
    };
    assert!(!conditions.allowed_to_authenticate());
    assert_eq!(
        conditions.source_decision.unwrap().cause,
        acl::Cause::ImplicitDeny
    );
    assert_eq!(n, before);
    assert_eq!(p, original);
}

#[test]
fn a_failed_reply_route_never_counts_as_policy_rejection_and_a_default_can_restore_it() {
    let mut n = network();
    let p = settings();
    n.topology.devices.get_mut("R2").unwrap().routes.clear();
    let result = n.inspect(&p, "guest", 0, policy::Protocol::Ssh2).unwrap();
    assert!(!result.allowed_to_authenticate());
    assert!(matches!(result,path::Observation::PathUnavailable{reason}if reason.contains("reply")));
    n.topology.devices.get_mut("R2").unwrap().routes.push(route(
        "0.0.0.0/0",
        "transit",
        "10.0.0.1",
    ));
    assert!(
        n.inspect(&p, "admin", 0, policy::Protocol::Ssh2)
            .unwrap()
            .allowed_to_authenticate()
    );
}

#[test]
fn down_client_transit_or_management_interfaces_are_path_failures_not_correct_denials() {
    for (device, port) in [("Client", "lan"), ("R1", "transit"), ("R2", "transit")] {
        let mut n = network();
        n.topology
            .devices
            .get_mut(device)
            .unwrap()
            .interfaces
            .get_mut(port)
            .unwrap()
            .up = false;
        assert!(matches!(
            n.inspect(&settings(), "admin", 0, policy::Protocol::Ssh2)
                .unwrap(),
            path::Observation::PathUnavailable { .. }
        ));
    }
    let mut n = network();
    n.topology.devices.get_mut("R1").unwrap().forwarding = false;
    assert!(
        matches!(n.inspect(&settings(),"admin",0,policy::Protocol::Ssh2).unwrap(),path::Observation::PathUnavailable{reason}if reason.contains("forwarding"))
    );
}

#[test]
fn a_reachable_path_does_not_bypass_line_version_or_identity_conditions() {
    let n = network();
    let mut p = settings();
    assert!(
        !n.inspect(&p, "admin", 0, policy::Protocol::Ssh1)
            .unwrap()
            .allowed_to_authenticate()
    );
    assert!(
        !n.inspect(&p, "admin", 0, policy::Protocol::Telnet)
            .unwrap()
            .allowed_to_authenticate()
    );
    p.host_identity = None;
    let path::Observation::Policy { conditions, .. } =
        n.inspect(&p, "admin", 0, policy::Protocol::Ssh2).unwrap()
    else {
        panic!("Path remains available")
    };
    assert_eq!(conditions.identity_available, Some(false));
    assert!(!conditions.allowed_to_authenticate());
}

#[test]
fn duplicate_bindings_addresses_unknown_equipment_and_bad_queries_are_errors() {
    let p = settings();
    let mut n = network();
    n.clients[1].binding = n.clients[0].binding.clone();
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Invalid);
    let mut n = network();
    n.topology
        .devices
        .get_mut("Guest")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = ip("192.0.2.10");
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Invalid);
    let mut n = network();
    n.management.device = "Missing".into();
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Invalid);
    let mut n = network();
    n.clients[1].id = n.clients[0].id.clone();
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Invalid);
    let n = network();
    for bad in ["missing", "../../admin", "$(anything)", ""] {
        assert_eq!(
            n.inspect(&p, bad, 0, policy::Protocol::Ssh2)
                .unwrap_err()
                .kind,
            policy::ErrorKind::Invalid
        );
    }
    assert_eq!(
        n.inspect(&p, "admin", 1, policy::Protocol::Ssh2)
            .unwrap_err()
            .kind,
        policy::ErrorKind::Invalid
    );
}

#[test]
fn alternate_source_selection_is_not_misattributed_to_the_bound_client_address() {
    let mut n = network();
    let c = n.topology.devices.get_mut("Client").unwrap();
    c.interfaces
        .insert("other".into(), port("other-link", "10.1.0.2", 30));
    c.routes.push(route("10.0.0.2/32", "other", "10.1.0.1"));
    n.topology
        .devices
        .get_mut("R1")
        .unwrap()
        .interfaces
        .insert("other".into(), port("other-link", "10.1.0.1", 30));
    n.topology.devices.get_mut("R2").unwrap().routes.push(route(
        "10.1.0.0/30",
        "transit",
        "10.0.0.1",
    ));
    n.validate().unwrap();
    assert_eq!(
        n.inspect(&settings(), "admin", 0, policy::Protocol::Ssh2)
            .unwrap_err()
            .kind,
        policy::ErrorKind::Unsupported
    );
}

#[test]
fn raw_decoding_is_bounded_strict_and_valid_paths_roundtrip_without_result_authority() {
    let n = network();
    let bytes = serde_json::to_vec(&n).unwrap();
    assert_eq!(path::Network::decode(&bytes).unwrap(), n);
    let mut value = serde_json::to_value(&n).unwrap();
    value["verified"] = true.into();
    assert_eq!(
        path::Network::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .kind,
        policy::ErrorKind::Invalid
    );
    value.as_object_mut().unwrap().remove("verified");
    value["version"] = 2.into();
    assert_eq!(
        path::Network::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .kind,
        policy::ErrorKind::Unsupported
    );
    assert_eq!(
        path::Network::decode(&vec![b' '; path::MAX_JSON_BYTES + 1])
            .unwrap_err()
            .kind,
        policy::ErrorKind::Limit
    );
    for raw in [b"not JSON".as_slice(), br#"{"version":1,"version":1}"#] {
        assert_eq!(
            path::Network::decode(raw).unwrap_err().kind,
            policy::ErrorKind::Invalid
        );
    }
}

#[test]
fn bounded_client_roster_rejects_excess_and_same_device_self_checks() {
    let mut n = network();
    n.clients.clear();
    for number in 0..6 {
        let name = format!("admin{number}");
        n.topology.devices.insert(
            name.clone(),
            device(
                false,
                vec![(
                    "lan",
                    port("office", &format!("192.0.2.{}", 20 + number), 24),
                )],
                vec![route("0.0.0.0/0", "lan", "192.0.2.1")],
            ),
        );
        n.clients.push(path::Client {
            id: name.clone(),
            binding: binding(&name, "lan"),
        });
    }
    n.validate().unwrap();
    assert_eq!(
        path::Network::decode(&serde_json::to_vec(&n).unwrap()).unwrap(),
        n
    );
    n.clients.push(path::Client {
        id: "extra".into(),
        binding: binding("Client", "lan"),
    });
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Limit);
    let mut n = network();
    n.clients[0].binding = n.management.clone();
    assert_eq!(n.validate().unwrap_err().kind, policy::ErrorKind::Invalid);
}
