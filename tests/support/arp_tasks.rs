// Synthetic behavior fixtures, not registered authored lessons.
use dlms_learning_lab::{
    arp_inspection as core,
    arp_inspection_workspace::*,
    dhcp_snooping as dhcp,
    dhcp_snooping_workspace::Name,
    mac_learning::{Mac, State},
    switching::Jack,
};
#[path = "dhcp_snooping_fixture.rs"]
mod access;
use access::mac;
pub fn task(kind: u8, example: bool) -> Task {
    let prefix = if example { "sample" } else { "assigned" };
    let host = |n: &str| Name(format!("{prefix}-{n}"));
    let original = access::fixture();
    let mut f = original.fabric().clone();
    let sw = f.switches.remove("edge").unwrap();
    let switch = format!("{prefix}-edge");
    f.switches.insert(switch.clone(), sw);
    f.hosts = std::mem::take(&mut f.hosts)
        .into_iter()
        .map(|(n, h)| (format!("{prefix}-{n}"), h))
        .collect();
    for c in &mut f.cables {
        c.id = format!("{prefix}-{}", c.id);
        for jack in [&mut c.a, &mut c.b] {
            match jack {
                Jack::Host { host: h } => *h = format!("{prefix}-{h}"),
                Jack::Switch { switch: s, .. } => *s = switch.clone(),
            }
        }
    }
    let addresses = original
        .fabric()
        .hosts
        .keys()
        .map(|n| (format!("{prefix}-{n}"), original.address(n).unwrap()))
        .collect();
    let mut d = dhcp::World::new(State::new(f, addresses, 300).unwrap(), true).unwrap();
    let mut c = d.config().clone();
    c.enabled = true;
    c.vlans = vec![10];
    c.ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    d.configure(c).unwrap();
    assert_eq!(
        d.config()
            .ports
            .iter()
            .map(|p| p.port.clone())
            .collect::<std::collections::BTreeSet<_>>(),
        access::names(&["client", "server", "peer", "other"]),
        "The synthetic task must retain the supplied access-port roster"
    );
    let mut initial = core::World::new(d).unwrap();
    let mut reference = initial.config().clone();
    reference.vlans = vec![10];
    reference
        .ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    let client_ip = if example { "172.18.40.8" } else { "10.1.0.10" }
        .parse()
        .unwrap();
    let server_ip = if example { "172.18.40.1" } else { "10.1.0.1" }
        .parse()
        .unwrap();
    let peer_ip = if example { "172.18.40.99" } else { "10.1.0.99" }
        .parse()
        .unwrap();
    let request = dhcp::Packet {
        source: mac(1),
        destination: Mac::BROADCAST,
        transaction: 7,
        client: mac(1),
        relay: "0.0.0.0".parse().unwrap(),
        option82: false,
        message: dhcp::Message::Request,
    };
    let mut ack = request.clone();
    ack.source = mac(2);
    ack.destination = mac(1);
    ack.message = dhcp::Message::Ack {
        address: client_ip,
        lease: 120,
    };
    let setup = [
        Step::Dhcp {
            host: host("client"),
            packet: request,
            recipients: vec![host("server")],
            learned: false,
            removed: false,
        },
        Step::Dhcp {
            host: host("server"),
            packet: ack,
            recipients: vec![host("client")],
            learned: true,
            removed: false,
        },
    ];
    let request = core::Packet {
        operation: core::Operation::Request,
        ethernet_source: mac(1),
        ethernet_destination: Mac::BROADCAST,
        sender_mac: mac(1),
        sender_ip: client_ip,
        target_mac: Mac([0; 6]),
        target_ip: server_ip,
    };
    let mut reply = request.clone();
    reply.operation = core::Operation::Reply;
    reply.ethernet_destination = mac(2);
    reply.target_mac = mac(2);
    let good = |p: core::Packet, recipients: Vec<Name>| Step::Arp {
        host: host("client"),
        packet: p,
        expected: Expected::Inspected { recipients },
    };
    let bad = |host: Name, p: core::Packet, cause| Step::Arp {
        host,
        packet: p,
        expected: Expected::Rejected { cause },
    };
    let case = |label: &str, extra: Vec<Step>, leases_after| Case {
        label: label.into(),
        steps: setup.iter().cloned().chain(extra).collect(),
        leases_after,
    };
    let mut forged = request.clone();
    forged.ethernet_source = mac(3);
    forged.sender_mac = mac(3);
    let mut expired = vec![Step::Advance { seconds: 120 }];
    expired.push(bad(host("client"), request.clone(), None));
    let cases = match kind {
        1 => {
            let mut invalid_client = request.clone();
            invalid_client.sender_ip = peer_ip;
            let mut server = reply.clone();
            server.ethernet_source = mac(2);
            server.ethernet_destination = mac(1);
            server.sender_mac = mac(2);
            server.sender_ip = server_ip;
            server.target_mac = mac(1);
            server.target_ip = client_ip;
            vec![
                case(
                    "Valid client request",
                    vec![good(request.clone(), vec![host("peer"), host("server")])],
                    1,
                ),
                case(
                    "Peer claims the client address",
                    vec![bad(host("peer"), forged, None)],
                    1,
                ),
                case(
                    "Client claims another address",
                    vec![bad(host("client"), invalid_client, None)],
                    1,
                ),
                case(
                    "Approved server reply",
                    vec![Step::Arp {
                        host: host("server"),
                        packet: server,
                        expected: Expected::Delivered {
                            recipients: vec![host("client")],
                        },
                    }],
                    1,
                ),
                case("Expired client record", expired, 0),
            ]
        }
        2 => {
            let mut peer = request.clone();
            peer.ethernet_source = mac(3);
            peer.sender_mac = mac(3);
            peer.sender_ip = peer_ip;
            reference.filters = vec![core::Filter {
                vlan: 10,
                static_only: false,
                rules: vec![core::Rule {
                    action: core::Action::Permit,
                    sender_ip: peer_ip,
                    sender_mac: mac(3),
                }],
            }];
            let mut wrong = reference.clone();
            wrong.filters[0].static_only = true;
            wrong.filters[0].rules[0].sender_mac = mac(9);
            initial.configure(wrong).unwrap();
            let mut bad_peer = peer.clone();
            bad_peer.ethernet_source = mac(9);
            bad_peer.sender_mac = mac(9);
            vec![
                case(
                    "Static peer has no DHCP record",
                    vec![Step::Arp {
                        host: host("peer"),
                        packet: peer,
                        expected: Expected::Inspected {
                            recipients: vec![host("client"), host("server")],
                        },
                    }],
                    1,
                ),
                case(
                    "Unexpected static sender",
                    vec![bad(host("peer"), bad_peer, None)],
                    1,
                ),
                case(
                    "DHCP client still uses its record",
                    vec![good(request.clone(), vec![host("peer"), host("server")])],
                    1,
                ),
                case("Expired client record", expired, 0),
                case(
                    "Peer claims a different address",
                    vec![bad(host("peer"), forged, None)],
                    1,
                ),
            ]
        }
        3 => {
            initial.configure(reference.clone()).unwrap();
            reference.checks.source_mac = true;
            reference.checks.destination_mac = true;
            let mut source_request = request.clone();
            source_request.ethernet_source = mac(9);
            let mut source_reply = reply.clone();
            source_reply.ethernet_source = mac(9);
            let mut target_reply = reply.clone();
            target_reply.target_mac = mac(9);
            vec![
                case(
                    "Valid request keeps an unknown target MAC",
                    vec![good(request.clone(), vec![host("peer"), host("server")])],
                    1,
                ),
                case(
                    "Valid reply",
                    vec![good(reply.clone(), vec![host("server")])],
                    1,
                ),
                case(
                    "Request header disagrees",
                    vec![bad(
                        host("client"),
                        source_request,
                        Some(core::Rejection::SourceMacMismatch),
                    )],
                    1,
                ),
                case(
                    "Reply source disagrees",
                    vec![bad(
                        host("client"),
                        source_reply,
                        Some(core::Rejection::SourceMacMismatch),
                    )],
                    1,
                ),
                case(
                    "Reply target disagrees",
                    vec![bad(
                        host("client"),
                        target_reply,
                        Some(core::Rejection::DestinationMacMismatch),
                    )],
                    1,
                ),
                case(
                    "Unknown sender remains rejected",
                    vec![bad(host("peer"), forged, None)],
                    1,
                ),
            ]
        }
        _ => panic!("Fixed fixture kind"),
    };
    Task{id:Name(format!("arp-{kind}-{prefix}")),instructions:"Repair the supplied ARP policy and verify all traffic requirements while retaining the supplied DHCP policy.".into(),initial,reference,cases}
}
pub fn definition(kind: u8) -> Definition {
    Definition {
        initial: task(kind, false),
        example: task(kind, true),
    }
}
