//! Focused finite-transport state, boundary and isolation tests.
use dlms_learning_lab::{acl, transport_core::*};
use std::net::Ipv4Addr;

fn endpoint(host: &str, port: u16) -> Endpoint {
    Endpoint {
        host: host.into(),
        port,
    }
}
fn config(protocol: Protocol) -> Config {
    Config {
        version: VERSION,
        hosts: vec![
            Host {
                id: "client".into(),
                address: Ipv4Addr::new(192, 0, 2, 10),
            },
            Host {
                id: "server".into(),
                address: Ipv4Addr::new(192, 0, 2, 20),
            },
        ],
        listeners: vec![Listener {
            protocol,
            endpoint: endpoint("server", 8443),
        }],
        exchanges: vec![Exchange {
            id: "report".into(),
            protocol,
            client: endpoint("client", 50123),
            server: endpoint("server", 8443),
            payload: "report: café\n".into(),
        }],
    }
}
fn allow(_: &Packet) -> Result<Decision, Error> {
    Ok(Decision::delivered("Supplied path permits this packet."))
}

#[test]
fn tcp_open_has_three_real_peer_transitions_then_one_write() {
    let c = config(Protocol::Tcp);
    let mut m = Machine::new(&c, "report").unwrap();
    assert_eq!(m.report().outcome, Outcome::Unrun);
    assert!(m.report().trace.is_empty());
    assert!(m.report().received.is_none());
    assert_eq!(
        (m.report().client_state, m.report().server_state),
        (Some(TcpState::Closed), Some(TcpState::Listen))
    );
    m.apply(Action::Start, &mut allow).unwrap();
    assert_eq!(
        (m.report().client_state, m.report().server_state),
        (Some(TcpState::SynSent), Some(TcpState::SynReceived))
    );
    m.apply(Action::Advance, &mut allow).unwrap();
    assert_eq!(
        (m.report().client_state, m.report().server_state),
        (Some(TcpState::Established), Some(TcpState::SynReceived))
    );
    m.apply(Action::Advance, &mut allow).unwrap();
    assert_eq!(m.report().outcome, Outcome::HandshakeComplete);
    assert_eq!(
        (m.report().client_state, m.report().server_state),
        (Some(TcpState::Established), Some(TcpState::Established))
    );
    assert!(m.report().received.is_none());
    m.apply(Action::Write, &mut allow).unwrap();
    assert_eq!(m.report().outcome, Outcome::Delivered);
    assert_eq!(
        m.report().received.as_deref(),
        Some(c.exchanges[0].payload.as_str())
    );
    let packets: Vec<_> = m.report().trace.iter().map(|e| e.packet).collect();
    assert_eq!(
        packets.iter().map(|p| p.kind).collect::<Vec<_>>(),
        vec![
            PacketKind::Syn,
            PacketKind::SynAck,
            PacketKind::Ack,
            PacketKind::TcpData
        ]
    );
    assert_eq!(
        packets.iter().map(|p| p.payload_bytes).collect::<Vec<_>>(),
        vec![0, 0, 0, c.exchanges[0].payload.len()]
    );
    for i in [0, 2, 3] {
        assert_eq!(
            (packets[i].source, packets[i].destination),
            (m.report().client, m.report().server)
        );
    }
    assert_eq!(
        (packets[1].source, packets[1].destination),
        (m.report().server, m.report().client)
    );
}

#[test]
fn each_dropped_handshake_leg_keeps_the_actual_distinct_peer_states() {
    let c = config(Protocol::Tcp);
    for (kind, client, server, count) in [
        (PacketKind::Syn, TcpState::SynSent, TcpState::Listen, 1),
        (
            PacketKind::SynAck,
            TcpState::SynSent,
            TcpState::SynReceived,
            2,
        ),
        (
            PacketKind::Ack,
            TcpState::Established,
            TcpState::SynReceived,
            3,
        ),
    ] {
        // A pure fixed decision fixture can distinguish final ACK. This does
        // not pretend an existing static five-tuple ACL can see TCP flags.
        let r = run(&c, "report", &mut |p| {
            Ok(if p.kind == kind {
                Decision::dropped("Fixed loss fixture.")
            } else {
                Decision::delivered("Delivered.")
            })
        })
        .unwrap();
        assert_eq!(r.outcome, Outcome::PacketDropped { kind });
        assert_eq!(
            (r.client_state, r.server_state),
            (Some(client), Some(server))
        );
        assert_eq!(r.trace.len(), count);
        assert!(r.received.is_none());
    }
}

#[test]
fn sent_tcp_data_does_not_imply_receipt_or_invent_retransmission() {
    let r = run(&config(Protocol::Tcp), "report", &mut |p| {
        Ok(if p.kind == PacketKind::TcpData {
            Decision::dropped("Drop data.")
        } else {
            Decision::delivered("Pass.")
        })
    })
    .unwrap();
    assert_eq!(
        r.outcome,
        Outcome::PacketDropped {
            kind: PacketKind::TcpData
        }
    );
    assert_eq!(
        (r.client_state, r.server_state),
        (Some(TcpState::Established), Some(TcpState::Established))
    );
    assert_eq!(r.trace.len(), 4);
    assert!(r.received.is_none());
}

#[test]
fn udp_has_one_datagram_no_tcp_state_and_drop_never_earns_receipt() {
    let c = config(Protocol::Udp);
    let r = run(&c, "report", &mut allow).unwrap();
    assert_eq!(r.trace.len(), 1);
    assert_eq!(r.trace[0].packet.kind, PacketKind::Datagram);
    assert_eq!((r.client_state, r.server_state), (None, None));
    assert_eq!(r.received, Some(c.exchanges[0].payload.clone()));
    let dropped = run(&c, "report", &mut |_| {
        Ok(Decision::dropped("UDP filtered."))
    })
    .unwrap();
    assert_eq!(
        dropped.outcome,
        Outcome::PacketDropped {
            kind: PacketKind::Datagram
        }
    );
    assert!(dropped.received.is_none());
}

#[test]
fn wrong_protocol_port_or_host_is_no_listener_without_fabricated_rst_or_icmp() {
    for proto in [Protocol::Tcp, Protocol::Udp] {
        for change in 0..3 {
            let mut c = config(proto);
            match change {
                0 => {
                    c.listeners[0].protocol = if proto == Protocol::Tcp {
                        Protocol::Udp
                    } else {
                        Protocol::Tcp
                    }
                }
                1 => c.listeners[0].endpoint.port += 1,
                _ => c.listeners[0].endpoint.host = "client".into(),
            }
            let r = run(&c, "report", &mut allow).unwrap();
            assert_eq!(r.outcome, Outcome::NoListener);
            assert_eq!(r.trace.len(), 1);
            assert!(r.received.is_none());
            if proto == Protocol::Tcp {
                assert_eq!(r.server_state, Some(TcpState::Closed));
            }
        }
    }
}

#[test]
fn same_port_tcp_and_udp_are_distinct_namespaces() {
    let mut c = config(Protocol::Tcp);
    c.listeners.push(Listener {
        protocol: Protocol::Udp,
        endpoint: endpoint("server", 8443),
    });
    let mut udp = c.exchanges[0].clone();
    udp.id = "datagram".into();
    udp.protocol = Protocol::Udp;
    c.exchanges.push(udp);
    let r = run_all(&c, &mut allow).unwrap();
    assert_eq!(r.len(), 2);
    assert!(r.iter().all(|r| r.outcome == Outcome::Delivered));
    assert_eq!(r[0].trace.len(), 4);
    assert_eq!(r[1].trace.len(), 1);
    c.listeners.push(c.listeners[0].clone());
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Invalid);
}

#[test]
fn two_clients_are_not_collapsed_by_service_port() {
    let mut c = config(Protocol::Tcp);
    c.hosts.push(Host {
        id: "other".into(),
        address: Ipv4Addr::new(192, 0, 2, 11),
    });
    let mut other = c.exchanges[0].clone();
    other.id = "other-report".into();
    other.client.host = "other".into();
    other.payload = "different".into();
    c.exchanges.push(other);
    let r = run_all(&c, &mut |p| {
        Ok(if p.source.address == Ipv4Addr::new(192, 0, 2, 11) {
            Decision::dropped("Second client denied.")
        } else {
            Decision::delivered("First client permitted.")
        })
    })
    .unwrap();
    assert_eq!(r[0].outcome, Outcome::Delivered);
    assert_eq!(
        r[1].outcome,
        Outcome::PacketDropped {
            kind: PacketKind::Syn
        }
    );
    assert_ne!(r[0].client, r[1].client);
    assert!(r[1].received.is_none());
}

#[test]
fn reversed_port_order_is_visible_to_existing_stateless_acl_projection() {
    let r = run(&config(Protocol::Tcp), "report", &mut |p| {
        assert_eq!(
            packet_protocol(p),
            if p.kind == PacketKind::SynAck {
                acl::PacketProtocol::Tcp {
                    source_port: 8443,
                    destination_port: 50123,
                }
            } else {
                acl::PacketProtocol::Tcp {
                    source_port: 50123,
                    destination_port: 8443,
                }
            }
        );
        Ok(Decision::delivered("Classified exact direction."))
    })
    .unwrap();
    assert_eq!(r.outcome, Outcome::Delivered);
    let udp = run(&config(Protocol::Udp), "report", &mut |p| {
        assert_eq!(
            packet_protocol(p),
            acl::PacketProtocol::Udp {
                source_port: 50123,
                destination_port: 8443
            }
        );
        allow(p)
    })
    .unwrap();
    assert_eq!(udp.outcome, Outcome::Delivered);
}

#[test]
fn bad_actions_and_classifier_errors_are_atomic_and_do_not_call_early() {
    let c = config(Protocol::Tcp);
    let mut m = Machine::new(&c, "report").unwrap();
    let initial = m.report().clone();
    for a in [Action::Advance, Action::Write] {
        assert_eq!(
            m.apply(a, &mut |_| panic!("invalid action called classifier"))
                .unwrap_err()
                .kind,
            ErrorKind::Transition
        );
        assert_eq!(m.report(), &initial);
    }
    assert_eq!(
        m.apply(Action::Start, &mut |_| Err(Error::new(
            ErrorKind::Classifier,
            "Unsupported path inspection."
        )))
        .unwrap_err()
        .kind,
        ErrorKind::Classifier
    );
    assert_eq!(m.report(), &initial);
    assert_eq!(
        m.apply(Action::Start, &mut |_| Ok(Decision::delivered(
            &"x".repeat(MAX_DETAIL_BYTES + 1)
        )))
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    assert_eq!(m.report(), &initial);
    assert_eq!(
        m.apply(Action::Start, &mut |_| Err(Error::new(
            ErrorKind::Classifier,
            "x".repeat(MAX_DETAIL_BYTES + 1)
        )))
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    assert_eq!(m.report(), &initial);
    m.apply(Action::Start, &mut allow).unwrap();
    let started = m.report().clone();
    assert_eq!(
        m.apply(Action::Start, &mut allow).unwrap_err().kind,
        ErrorKind::Transition
    );
    assert_eq!(m.report(), &started);
}

#[test]
fn write_requires_both_established_and_one_write_only() {
    let c = config(Protocol::Tcp);
    let mut m = Machine::new(&c, "report").unwrap();
    m.apply(Action::Start, &mut allow).unwrap();
    m.apply(Action::Advance, &mut allow).unwrap();
    let half = m.report().clone();
    assert_eq!(
        m.apply(Action::Write, &mut allow).unwrap_err().kind,
        ErrorKind::Transition
    );
    assert_eq!(m.report(), &half);
    m.apply(Action::Advance, &mut allow).unwrap();
    m.apply(Action::Write, &mut allow).unwrap();
    let done = m.report().clone();
    for a in [Action::Start, Action::Advance, Action::Write] {
        assert_eq!(
            m.apply(a, &mut allow).unwrap_err().kind,
            ErrorKind::Transition
        );
        assert_eq!(m.report(), &done);
    }
}

#[test]
fn invalid_source_never_initializes_or_calls_the_classifier() {
    let base = config(Protocol::Tcp);
    let mut cases = Vec::new();
    let mut c = base.clone();
    c.hosts[1].id = c.hosts[0].id.clone();
    cases.push(c);
    let mut c = base.clone();
    c.hosts[1].address = c.hosts[0].address;
    cases.push(c);
    let mut c = base.clone();
    c.exchanges[0].client.host = "unknown".into();
    cases.push(c);
    let mut c = base.clone();
    c.exchanges[0].client.port = 0;
    cases.push(c);
    let mut c = base.clone();
    c.exchanges[0].server = c.exchanges[0].client.clone();
    cases.push(c);
    let mut c = base.clone();
    c.exchanges[0].payload = "bad\0payload".into();
    cases.push(c);
    let mut c = base.clone();
    c.exchanges.push(c.exchanges[0].clone());
    cases.push(c);
    let mut c = base.clone();
    let mut alias = c.exchanges[0].clone();
    alias.id = "different-id".into();
    c.exchanges.push(alias);
    cases.push(c);
    let mut c = base.clone();
    c.exchanges[0].id = "é".into();
    cases.push(c);
    for c in cases {
        assert!(run_all(&c, &mut |_| panic!("invalid source executed")).is_err());
    }
    assert_eq!(
        Machine::new(&base, "foreign").unwrap_err().kind,
        ErrorKind::Invalid
    );
}

#[test]
fn source_variation_changes_ports_payload_and_protocol_without_answer_strings() {
    for (proto, source, service, text) in [
        (Protocol::Tcp, 1, 65535, "first"),
        (Protocol::Udp, 65535, 1, "second % []"),
        (Protocol::Tcp, 40001, 8080, "third\nline"),
    ] {
        let mut c = config(proto);
        c.exchanges[0].client.port = source;
        c.exchanges[0].server.port = service;
        c.listeners[0].endpoint.port = service;
        c.exchanges[0].payload = text.into();
        let r = run(&c, "report", &mut allow).unwrap();
        assert_eq!(r.client.port, source);
        assert_eq!(r.server.port, service);
        assert_eq!(r.received.as_deref(), Some(text));
    }
}

#[test]
fn payload_byte_bounds_and_collection_limits_precede_execution() {
    let mut c = config(Protocol::Tcp);
    c.exchanges[0].payload = "é".repeat(MAX_PAYLOAD_BYTES / 2);
    assert_eq!(
        run(&c, "report", &mut allow)
            .unwrap()
            .received
            .unwrap()
            .len(),
        MAX_PAYLOAD_BYTES
    );
    c.exchanges[0].payload.push('é');
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Limit);
    c.exchanges[0].payload.clear();
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut c = config(Protocol::Tcp);
    c.hosts.resize(MAX_HOSTS + 1, c.hosts[0].clone());
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut c = config(Protocol::Tcp);
    c.listeners
        .resize(MAX_LISTENERS + 1, c.listeners[0].clone());
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Limit);
    let mut c = config(Protocol::Tcp);
    c.exchanges
        .resize(MAX_EXCHANGES + 1, c.exchanges[0].clone());
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Limit);
}

#[test]
fn capped_raw_decode_rejects_overflow_unknown_protocol_duplicate_fields_and_versions() {
    let c = config(Protocol::Tcp);
    let raw = serde_json::to_vec(&c).unwrap();
    assert_eq!(Config::decode(&raw).unwrap(), c);
    assert_eq!(
        Config::decode(&vec![b' '; MAX_CONFIG_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let text = String::from_utf8(raw).unwrap();
    for bad in [
        text.replace("50123", "65536"),
        text.replace("50123", "-1"),
        text.replace("\"tcp\"", "\"sctp\""),
        text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1),
        text.replacen("\"version\":1", "\"version\":18446744073709551616", 1),
    ] {
        assert_eq!(
            Config::decode(bad.as_bytes()).unwrap_err().kind,
            ErrorKind::Invalid
        );
    }
    let mut c = config(Protocol::Tcp);
    c.version += 1;
    assert_eq!(c.validate().unwrap_err().kind, ErrorKind::Unsupported);
}

#[test]
fn largest_supported_batch_is_finite_and_data_driven() {
    let mut c = config(Protocol::Tcp);
    let first = c.exchanges[0].clone();
    c.exchanges = (0..MAX_EXCHANGES)
        .map(|i| {
            let mut e = first.clone();
            e.id = format!("report-{i}");
            e.client.port += i as u16;
            e.payload = char::from(b'A' + i as u8)
                .to_string()
                .repeat(MAX_PAYLOAD_BYTES);
            e
        })
        .collect();
    let reports = run_all(&c, &mut |_| {
        Ok(Decision::delivered(&"x".repeat(MAX_DETAIL_BYTES)))
    })
    .unwrap();
    assert_eq!(reports.len(), MAX_EXCHANGES);
    assert_eq!(reports.iter().map(|r| r.trace.len()).sum::<usize>(), 32);
    for (i, r) in reports.iter().enumerate() {
        assert_eq!(r.received.as_deref(), Some(c.exchanges[i].payload.as_str()));
    }
    assert!(reports.iter().map(|r| r.trace.len()).sum::<usize>() <= MAX_TRACE_EVENTS);
}
