// Synthetic task data for the bounded observer; no host files or packets.
use dlms_learning_lab::{
    dhcp_snooping::{self as core, Message, Packet},
    dhcp_snooping_workspace::*,
    mac_learning::{Mac, State},
    switching::Jack,
};
#[path = "dhcp_snooping_fixture.rs"]
mod fabric;
use std::{collections::BTreeMap, net::Ipv4Addr};
pub fn task(kind: u8, example: bool) -> Task {
    let prefix = if example { "sample" } else { "assigned" };
    let offset = if example { 20 } else { 0 };
    let old = fabric::fixture();
    let mut f = old.fabric().clone();
    let old_switch = f.switches.remove("edge").unwrap();
    let switch = format!("{prefix}-edge");
    f.switches.insert(switch.clone(), old_switch);
    f.hosts = std::mem::take(&mut f.hosts)
        .into_iter()
        .map(|(k, v)| (format!("{prefix}-{k}"), v))
        .collect();
    for c in &mut f.cables {
        c.id = format!("{prefix}-{}", c.id);
        for jack in [&mut c.a, &mut c.b] {
            match jack {
                Jack::Host { host } => *host = format!("{prefix}-{host}"),
                Jack::Switch { switch: s, .. } => *s = switch.clone(),
            }
        }
    }
    let addresses = ["client", "server", "peer", "other"]
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            (
                format!("{prefix}-{n}"),
                Mac([2, 0, 0, 0, 0, offset + i as u8 + 1]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut initial = core::World::new(State::new(f, addresses, 300).unwrap(), true).unwrap();
    let mut reference = initial.config().clone();
    assert_eq!(
        reference
            .ports
            .iter()
            .map(|p| p.port.clone())
            .collect::<std::collections::BTreeSet<_>>(),
        fabric::names(&["client", "server", "peer", "other"])
    );
    reference.enabled = true;
    reference.vlans = vec![10];
    reference
        .ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = true;
    if kind == 2 {
        let mut c = reference.clone();
        c.verify_source = false;
        initial.configure(c).unwrap();
    } else {
        assert_eq!(kind, 1);
    }
    let client = Mac([2, 0, 0, 0, 0, offset + 1]);
    let server = Mac([2, 0, 0, 0, 0, offset + 2]);
    let peer = Mac([2, 0, 0, 0, 0, offset + 3]);
    let addr = if example {
        Ipv4Addr::new(172, 18, 20, 50)
    } else {
        Ipv4Addr::new(10, 25, 0, 90)
    };
    let p = |message, source, claimed| Packet {
        source,
        destination: Mac::BROADCAST,
        transaction: 71 + u32::from(offset),
        client: claimed,
        relay: Ipv4Addr::UNSPECIFIED,
        option82: false,
        message,
    };
    let delivered = |host: &str, packet, recipient: &str, learned| Step::Message {
        host: Name(format!("{prefix}-{host}")),
        packet,
        expected: Expected::Forwarded {
            accepted_by: vec![Name(format!("{prefix}-{recipient}"))],
            learned,
            removed: false,
        },
    };
    let approved = vec![
        delivered(
            "client",
            p(Message::Discover, client, client),
            "server",
            false,
        ),
        delivered(
            "server",
            p(Message::Offer { address: addr }, server, client),
            "client",
            false,
        ),
        delivered(
            "client",
            p(Message::Request, client, client),
            "server",
            false,
        ),
        delivered(
            "server",
            p(
                Message::Ack {
                    address: addr,
                    lease: 120,
                },
                server,
                client,
            ),
            "client",
            true,
        ),
    ];
    let mut cases = vec![
        Case {
            label: "Approved initial exchange".into(),
            steps: approved.clone(),
            leases_after: 1,
        },
        Case {
            label: "Unexpected server reply".into(),
            steps: vec![
                delivered(
                    "client",
                    p(Message::Request, client, client),
                    "server",
                    false,
                ),
                Step::Message {
                    host: Name(format!("{prefix}-peer")),
                    packet: p(
                        Message::Offer {
                            address: Ipv4Addr::new(10, 99, 0, 7),
                        },
                        peer,
                        client,
                    ),
                    expected: Expected::Rejected {
                        cause: core::Rejection::UntrustedServer,
                    },
                },
                delivered(
                    "server",
                    p(
                        Message::Ack {
                            address: addr,
                            lease: 120,
                        },
                        server,
                        client,
                    ),
                    "client",
                    true,
                ),
            ],
            leases_after: 1,
        },
        Case {
            label: "Client identifiers disagree".into(),
            steps: vec![Step::Message {
                host: Name(format!("{prefix}-client")),
                packet: p(Message::Request, client, peer),
                expected: Expected::Rejected {
                    cause: core::Rejection::SourceMismatch,
                },
            }],
            leases_after: 0,
        },
        Case {
            label: "Recorded lease expires".into(),
            steps: approved
                .into_iter()
                .chain([Step::Advance { seconds: 120 }])
                .collect(),
            leases_after: 0,
        },
    ];
    // Different objectives receive different observations, not renamed copies.
    if kind == 1 {
        cases[2] = Case {
            label: "An Offer does not record a lease".into(),
            steps: vec![
                delivered(
                    "client",
                    p(Message::Discover, client, client),
                    "server",
                    false,
                ),
                delivered(
                    "server",
                    p(Message::Offer { address: addr }, server, client),
                    "client",
                    false,
                ),
            ],
            leases_after: 0,
        };
    } else {
        cases[3] = Case {
            label: "Matching fields are not authentication".into(),
            steps: vec![delivered(
                "client",
                p(Message::Request, peer, peer),
                "server",
                false,
            )],
            leases_after: 0,
        };
    }
    Task {
        id: Name(format!("{prefix}-snoop-{kind}")),
        instructions: if kind == 1 {
            "Protect the supplied client VLAN from server replies on user ports while the approved server's exchange still reaches the client and produces an observed lease.".into()
        } else {
            "Reject requests whose Ethernet source and DHCP client MAC address disagree while valid client exchanges and protection from unexpected server replies continue working.".into()
        },
        initial,
        reference,
        cases,
    }
}
pub fn definition(kind: u8) -> Definition {
    Definition {
        initial: task(kind, false),
        example: task(kind, true),
    }
}
