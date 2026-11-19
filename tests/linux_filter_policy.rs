//! Exact typed-predicate binding; sampled enumeration below is an independent test oracle only.
use dlms_learning_lab::{
    linux_filter::{Chain, Cidr, Class, Packet, Protocol, Rule, Table, Verdict},
    linux_filter_policy::compare_filter,
    linux_router_path::{Facts, Link},
};
fn facts() -> Facts {
    Facts {
        links: vec![
            Link {
                name: "lan0".into(),
                address: "10.20.30.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
            Link {
                name: "wan0".into(),
                address: "198.51.100.1".parse().unwrap(),
                prefix: 24,
                up: true,
            },
        ],
        routes: vec![],
    }
}
fn clause(text: &str) -> Rule {
    let mut t = Table::new(Verdict::Drop);
    t.command(&format!("iptables -A FORWARD {text} -j ACCEPT"))
        .unwrap();
    t.forward.rules.remove(0)
}
#[test]
fn exact_allow_and_equivalent_splits_pass_and_nonexistent_interfaces_do_not_fabricate_matches() {
    let f = facts();
    let domain = [clause("")];
    let allowed = [clause(
        "-i lan0 -o wan0 -s 10.20.30.128/25 -d 198.51.100.9/32 -p tcp --dport 443",
    )];
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -i ghost0 -j DROP").unwrap();
    for net in ["10.20.30.128/26", "10.20.30.192/26"] {
        t.command(&format!("iptables -A FORWARD -i lan0 -o wan0 -s {net} -d 198.51.100.9/32 -p tcp --dport 443 -j ACCEPT")).unwrap();
    }
    assert_eq!(
        compare_filter(&t, Chain::Forward, &f, &domain, &allowed).unwrap(),
        None
    );
    let out = t.command("iptables -S FORWARD").unwrap();
    assert!(!out.changed);
    assert_eq!(
        compare_filter(&t, Chain::Forward, &f, &domain, &allowed).unwrap(),
        None
    );
}
#[test]
fn a_broad_allow_and_a_shadowed_allow_report_real_typed_counterexamples() {
    let f = facts();
    let domain = [clause("")];
    let allowed = [clause(
        "-i lan0 -o wan0 -s 10.20.30.128/25 -d 198.51.100.9/32 -p tcp --dport 443",
    )];
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -s 10.20.30.128/25 -p tcp --dport 443 -j ACCEPT")
        .unwrap();
    let m = compare_filter(&t, Chain::Forward, &f, &domain, &allowed)
        .unwrap()
        .unwrap();
    assert_eq!(
        (m.actual, m.expected, m.rule),
        (Verdict::Accept, Verdict::Drop, Some(1))
    );
    assert_eq!(
        t.evaluate(Chain::Forward, &m.packet).unwrap().verdict,
        m.actual
    );
    assert!(!allowed[0].matches(&m.packet));
    t.command("iptables -I FORWARD 1 -j DROP").unwrap();
    let m = compare_filter(&t, Chain::Forward, &f, &domain, &allowed)
        .unwrap()
        .unwrap();
    assert_eq!(
        (m.actual, m.expected, m.rule),
        (Verdict::Drop, Verdict::Accept, Some(1))
    );
    assert!(allowed[0].matches(&m.packet));
}
#[test]
fn icmp_has_no_tcp_udp_ports_and_zero_ports_are_valid_numeric_ports() {
    let f = facts();
    let domain = [clause("")];
    let allowed = [clause("-p icmp")];
    let mut t = Table::new(Verdict::Drop);
    t.command("iptables -A FORWARD -p icmp -j ACCEPT").unwrap();
    assert!(
        compare_filter(&t, Chain::Forward, &f, &domain, &allowed)
            .unwrap()
            .is_none()
    );
    t.command("iptables -F").unwrap();
    let m = compare_filter(&t, Chain::Forward, &f, &domain, &allowed)
        .unwrap()
        .unwrap();
    assert_eq!(m.packet.protocol, Protocol::Icmp);
    assert_eq!(
        (m.packet.source_port, m.packet.destination_port),
        (None, None)
    );
    let allowed = [clause("-p tcp --dport 0")];
    t.command("iptables -A FORWARD -p tcp --dport 0 -j ACCEPT")
        .unwrap();
    assert!(
        compare_filter(&t, Chain::Forward, &f, &domain, &allowed)
            .unwrap()
            .is_none()
    );
}
#[test]
fn malformed_or_impossible_requirements_are_context_errors_not_proof_success() {
    let f = facts();
    let domain = [clause("")];
    let t = Table::new(Verdict::Drop);
    for mut bad in [clause("-i ghost0"), clause("")] {
        if bad.ingress.is_none() {
            bad.source = Some(Cidr {
                network: "10.0.0.1".parse().unwrap(),
                prefix: 24,
            });
        }
        assert_eq!(
            compare_filter(&t, Chain::Forward, &f, &domain, &[bad])
                .unwrap_err()
                .class,
            Class::Context
        );
    }
    assert_eq!(
        compare_filter(&t, Chain::Forward, &f, &[], &[])
            .unwrap_err()
            .class,
        Class::Context
    );
    let mut bad = clause("");
    bad.target = Verdict::Drop;
    assert_eq!(
        compare_filter(&t, Chain::Forward, &f, &domain, &[bad])
            .unwrap_err()
            .class,
        Class::Context
    );
}
#[test]
fn all_seven_real_match_dimensions_agree_with_exhaustive_small_header_domains() {
    let f = facts();
    let mut domains = vec![];
    for proto in [Protocol::Tcp, Protocol::Udp] {
        for sp in [0, 1] {
            for dp in [0, 1] {
                let name = if proto == Protocol::Tcp { "tcp" } else { "udp" };
                domains.push(clause(&format!(
                    "-s 10.20.30.0/31 -d 198.51.100.0/31 -p {name} --sport {sp} --dport {dp}"
                )));
            }
        }
    }
    domains.push(clause("-s 10.20.30.0/31 -d 198.51.100.0/31 -p icmp"));
    let allowed = vec![
        clause("-s 10.20.30.1/32 -d 198.51.100.1/32 -i lan0 -o wan0 -p tcp --dport 1"),
        clause("-s 10.20.30.0/32 -d 198.51.100.0/32 -i wan0 -o lan0 -p udp --sport 0 --dport 0"),
    ];
    let mut packets = vec![];
    for a in [0, 1] {
        for b in [0, 1] {
            for i in ["lan0", "wan0"] {
                for o in ["lan0", "wan0"] {
                    for proto in [Protocol::Tcp, Protocol::Udp, Protocol::Icmp] {
                        for sp in [0, 1] {
                            for dp in [0, 1] {
                                if proto == Protocol::Icmp && (sp != 0 || dp != 0) {
                                    continue;
                                }
                                packets.push(Packet {
                                    source: format!("10.20.30.{a}").parse().unwrap(),
                                    destination: format!("198.51.100.{b}").parse().unwrap(),
                                    ingress: Some(i.into()),
                                    egress: Some(o.into()),
                                    protocol: proto,
                                    source_port: (proto != Protocol::Icmp).then_some(sp),
                                    destination_port: (proto != Protocol::Icmp).then_some(dp),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(packets.len(), 144);
    let mut seed = 0x37a9_1e21u32;
    let mut n = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        seed
    };
    for case in 0..32 {
        let mut t = Table::new(if case % 4 < 2 || n() % 2 == 0 {
            Verdict::Drop
        } else {
            Verdict::Accept
        });
        if case % 4 < 2 {
            if case % 4 == 1 {
                t.command("iptables -A FORWARD -i ghost0 -j DROP").unwrap();
            }
            t.forward.rules.extend(allowed.clone());
        } else {
            for _ in 0..8 {
                let mut r = clause("");
                r.source = Some(
                    Cidr::parse(if n() % 2 == 0 {
                        "10.20.30.1/32"
                    } else {
                        "10.20.30.0/31"
                    })
                    .unwrap(),
                );
                r.destination = Some(
                    Cidr::parse(if n() % 2 == 0 {
                        "198.51.100.1/32"
                    } else {
                        "198.51.100.0/31"
                    })
                    .unwrap(),
                );
                r.protocol = match n() % 3 {
                    0 => Some(Protocol::Tcp),
                    1 => Some(Protocol::Udp),
                    _ => None,
                };
                if r.protocol.is_some() {
                    r.source_port = Some((n() % 2) as u16);
                    r.destination_port = Some((n() % 2) as u16);
                }
                r.ingress = if n() % 3 == 0 {
                    None
                } else {
                    Some(if n() % 2 == 0 { "lan0" } else { "wan0" }.into())
                };
                r.egress = if n() % 3 == 0 {
                    None
                } else {
                    Some(if n() % 2 == 0 { "lan0" } else { "wan0" }.into())
                };
                r.target = if n() % 2 == 0 {
                    Verdict::Accept
                } else {
                    Verdict::Drop
                };
                t.forward.rules.push(r);
            }
        }
        let wrong = packets.iter().any(|p| {
            (t.evaluate(Chain::Forward, p).unwrap().verdict == Verdict::Accept)
                != allowed.iter().any(|r| r.matches(p))
        });
        let m = compare_filter(&t, Chain::Forward, &f, &domains, &allowed).unwrap();
        assert_eq!(m.is_some(), wrong, "case{case}");
        if let Some(m) = m {
            assert!(domains.iter().any(|d| d.matches(&m.packet)));
            assert_eq!(
                t.evaluate(Chain::Forward, &m.packet).unwrap().verdict,
                m.actual
            );
            assert_eq!(
                allowed.iter().any(|a| a.matches(&m.packet)),
                m.expected == Verdict::Accept
            );
        }
    }
}
