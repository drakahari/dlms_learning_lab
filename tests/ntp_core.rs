use dlms_learning_lab::ntp::*;
use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;
fn ip(n: u8) -> Ipv4Addr {
    Ipv4Addr::new(192, 0, 2, n)
}
fn device(n: u8, up: Option<u8>) -> Node {
    Node {
        label: format!("router{n}"),
        online: true,
        role: Role::Device {
            upstream: up.map(ip),
        },
    }
}
fn reference(n: u8, st: u8) -> Node {
    Node {
        label: format!("reference{n}"),
        online: true,
        role: Role::Reference {
            stratum: st,
            healthy: true,
        },
    }
}
fn delivery(c: u8, s: u8) -> Delivery {
    Delivery {
        client: ip(c),
        server: ip(s),
        request_delivered: true,
        reply_delivered: true,
        udp123_allowed: true,
    }
}
fn fixture() -> State {
    State {
        version: VERSION,
        nodes: BTreeMap::from([
            (ip(1), reference(1, 1)),
            (ip(2), device(2, Some(1))),
            (ip(3), device(3, Some(2))),
            (ip(4), reference(4, 2)),
        ]),
        deliveries: vec![delivery(2, 1), delivery(3, 2), delivery(2, 4)],
    }
}
fn goal() -> Vec<Requirement> {
    vec![
        Requirement {
            client: ip(2),
            allowed_references: BTreeSet::from([ip(1), ip(4)]),
            must_serve: true,
        },
        Requirement {
            client: ip(3),
            allowed_references: BTreeSet::from([ip(2)]),
            must_serve: false,
        },
    ]
}
fn reason(s: &State, n: u8) -> FailureReason {
    s.status(ip(n)).unwrap().failure.unwrap().reason
}
#[test]
fn configured_chain_strata_and_immediate_reference_are_derived() {
    let s = fixture();
    s.validate().unwrap();
    let a = s.status(ip(3)).unwrap();
    assert!(a.synchronized && a.serving);
    assert_eq!(a.stratum, 3);
    assert_eq!(a.reference, Some(ip(2)));
    assert_eq!(a.chain, vec![ip(3), ip(2), ip(1)]);
    assert!(check(&s, &goal()).unwrap().iter().all(|c| c.passed));
    assert_eq!(s.status(ip(1)).unwrap().reference, None);
}
#[test]
fn legitimate_alternative_repair_and_current_success_invalidation() {
    let mut s = fixture();
    s.configure(ip(2), Some(ip(4))).unwrap();
    assert_eq!(s.status(ip(3)).unwrap().stratum, 4);
    assert!(check(&s, &goal()).unwrap().iter().all(|c| c.passed));
    s.configure(ip(3), None).unwrap();
    assert_eq!(
        check(&s, &goal())
            .unwrap()
            .iter()
            .map(|c| c.passed)
            .collect::<Vec<_>>(),
        vec![true, false]
    );
    s.configure(ip(3), Some(ip(2))).unwrap();
    assert!(check(&s, &goal()).unwrap().iter().all(|c| c.passed));
    s.nodes.get_mut(&ip(4)).unwrap().online = false;
    assert!(check(&s, &goal()).unwrap().iter().all(|c| !c.passed));
}
#[test]
fn single_upstream_is_not_a_shortest_reachable_source_search() {
    let mut s = fixture();
    s.deliveries.push(delivery(3, 1));
    s.deliveries.push(delivery(3, 4));
    s.nodes.get_mut(&ip(2)).unwrap().online = false;
    assert_eq!(reason(&s, 3), FailureReason::Offline);
    assert_eq!(s.status(ip(3)).unwrap().chain, vec![ip(3), ip(2)]);
    assert!(!s.status(ip(3)).unwrap().synchronized);
}
#[test]
fn request_reply_and_udp_policy_failures_are_distinct_and_do_not_mutate() {
    for (which, expected) in [
        (0, FailureReason::RequestUnreachable),
        (1, FailureReason::ReplyUnreachable),
        (2, FailureReason::Udp123Blocked),
    ] {
        let mut s = fixture();
        match which {
            0 => s.deliveries[0].request_delivered = false,
            1 => s.deliveries[0].reply_delivered = false,
            _ => s.deliveries[0].udp123_allowed = false,
        };
        let before = s.clone();
        assert_eq!(reason(&s, 3), expected);
        assert_eq!(s.status(ip(3)).unwrap().failure.unwrap().at, ip(1));
        assert_eq!(s, before);
    }
    let mut s = fixture();
    s.deliveries.remove(0);
    assert_eq!(reason(&s, 2), FailureReason::RequestUnreachable);
}
#[test]
fn unknown_address_is_admitted_to_config_but_cannot_synchronize() {
    let mut s = fixture();
    s.execute(ip(2), "no ntp server 192.0.2.1").unwrap();
    s.execute(ip(2), "ntp server 192.0.2.99").unwrap();
    assert_eq!(reason(&s, 2), FailureReason::UnknownServer);
    assert_eq!(s.status(ip(2)).unwrap().chain, vec![ip(2), ip(99)]);
    assert!(
        s.execute(ip(2), "show ntp status")
            .unwrap()
            .contains("not present in this virtual topology")
    );
    assert!(!check(&s, &goal()).unwrap()[0].passed);
}
#[test]
fn unsynchronized_source_does_not_become_server_through_client() {
    let mut s = fixture();
    if let Role::Reference { healthy, .. } = &mut s.nodes.get_mut(&ip(1)).unwrap().role {
        *healthy = false;
    } else {
        panic!()
    };
    for n in [1, 2, 3] {
        let a = s.status(ip(n)).unwrap();
        assert!(!a.synchronized && !a.serving);
        assert_eq!(a.stratum, 16);
        assert_eq!(
            a.failure.unwrap().reason,
            FailureReason::SourceUnsynchronized
        );
    }
    assert_eq!(s.status(ip(3)).unwrap().chain, vec![ip(3), ip(2), ip(1)]);
}
#[test]
fn self_and_multinode_cycles_fail_with_closed_chain_no_infinite_loop() {
    let mut s = fixture();
    s.configure(ip(2), Some(ip(2))).unwrap();
    assert_eq!(reason(&s, 3), FailureReason::Cycle);
    assert_eq!(s.status(ip(3)).unwrap().chain, vec![ip(3), ip(2), ip(2)]);
    s.configure(ip(2), Some(ip(3))).unwrap();
    s.deliveries.push(delivery(2, 3));
    assert_eq!(s.status(ip(2)).unwrap().chain, vec![ip(2), ip(3), ip(2)]);
    assert_eq!(reason(&s, 2), FailureReason::Cycle);
    assert_eq!(reason(&s, 3), FailureReason::Cycle);
}
#[test]
fn fifteenth_stratum_server_does_not_make_valid_sixteenth_stratum_client() {
    let mut s = fixture();
    if let Role::Reference { stratum, .. } = &mut s.nodes.get_mut(&ip(1)).unwrap().role {
        *stratum = 14;
    }
    assert_eq!(s.status(ip(2)).unwrap().stratum, 15);
    assert!(s.status(ip(2)).unwrap().synchronized);
    assert_eq!(s.status(ip(3)).unwrap().stratum, 16);
    assert_eq!(reason(&s, 3), FailureReason::StratumExhausted);
    assert!(!s.status(ip(3)).unwrap().serving);
}
#[test]
fn command_subset_preserves_fixture_facts_and_rejects_multiplexer_and_shell() {
    let mut s = fixture();
    let before = s.clone();
    for cmd in [
        "ntp server 192.0.2.4 prefer",
        "ntp master 3",
        "show ntp associations detail",
        "sh -c id",
        "show ntp status; id",
        "ntp server $(id)",
        "ntp server 192.0.2.1\nshow ntp status",
    ] {
        assert!(s.execute(ip(2), cmd).is_err(), "{cmd}");
        assert_eq!(s, before);
    }
    assert_eq!(
        s.execute(ip(2), "ntp server 192.0.2.4").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(s, before);
    s.execute(ip(2), "no ntp server 192.0.2.4").unwrap();
    assert_eq!(s, before);
    s.execute(ip(2), "ntp server 192.0.2.1").unwrap();
    assert_eq!(s, before);
    assert_eq!(s.deliveries, before.deliveries);
    assert_eq!(s.nodes[&ip(1)], before.nodes[&ip(1)]);
}
#[test]
fn invalid_config_commands_and_read_only_reference_are_atomic() {
    let mut s = fixture();
    for bad in [
        "0.0.0.0",
        "127.0.0.1",
        "224.0.0.1",
        "255.255.255.255",
        "bad",
        "1.2.3",
    ] {
        let before = s.clone();
        assert!(s.execute(ip(2), &format!("ntp server {bad}")).is_err());
        assert_eq!(s, before);
    }
    let before = s.clone();
    assert!(s.configure(ip(1), Some(ip(2))).is_err());
    assert!(s.execute(ip(1), "no ntp server 192.0.2.2").is_err());
    assert_eq!(s, before);
}
#[test]
fn status_and_associations_are_current_read_only_reports_not_fixed_success() {
    let mut s = fixture();
    let before = s.clone();
    let status = s.execute(ip(3), "show ntp status").unwrap();
    assert!(status.contains("stratum 3") && status.contains("reference is 192.0.2.2"));
    let a = s.execute(ip(3), "show ntp associations").unwrap();
    assert!(a.contains("192.0.2.2") && a.contains("* configured, synchronized source"));
    assert_eq!(s, before);
    s.configure(ip(3), None).unwrap();
    assert!(
        s.execute(ip(3), "show ntp associations")
            .unwrap()
            .contains("no configured")
    );
    assert!(
        s.execute(ip(3), "show ntp status")
            .unwrap()
            .contains("No time source is configured")
    );
}
#[test]
fn roundtrip_resume_regrade_and_reset_are_honest() {
    let initial = fixture();
    let mut s = initial.clone();
    s.configure(ip(2), Some(ip(99))).unwrap();
    let bytes = s.to_json().unwrap();
    let mut resumed = State::from_json(&bytes).unwrap();
    assert_eq!(s, resumed);
    assert!(!check(&resumed, &goal()).unwrap()[0].passed);
    resumed.configure(ip(2), Some(ip(4))).unwrap();
    assert!(check(&resumed, &goal()).unwrap().iter().all(|c| c.passed));
    resumed = initial.clone();
    assert_eq!(resumed, initial);
    assert!(check(&resumed, &goal()).unwrap().iter().all(|c| c.passed));
    assert_eq!(s, State::from_json(&bytes).unwrap());
}
#[test]
fn snapshot_validation_rejects_foreign_duplicates_unknown_fields_and_bad_versions() {
    let s = fixture();
    let mut x = s.clone();
    x.version = 99;
    assert!(x.status(ip(2)).is_err());
    x = s.clone();
    x.deliveries.push(delivery(2, 99));
    assert!(x.validate().is_err());
    x = s.clone();
    x.deliveries.push(delivery(2, 1));
    assert!(x.validate().is_err());
    x = s.clone();
    x.nodes.get_mut(&ip(2)).unwrap().label = "../x".into();
    assert!(x.validate().is_err());
    x = s.clone();
    x.nodes.insert(ip(5), reference(1, 1));
    assert!(x.validate().is_err());
    x = s.clone();
    x.nodes.get_mut(&ip(1)).unwrap().role = Role::Reference {
        stratum: 16,
        healthy: true,
    };
    assert!(x.validate().is_err());
    let text = String::from_utf8(s.to_json().unwrap()).unwrap();
    assert!(
        State::from_json(
            text.replacen("\"version\":1", "\"version\":1,\"extra\":0", 1)
                .as_bytes()
        )
        .is_err()
    );
    let node = serde_json::to_string(&s.nodes[&ip(1)]).unwrap();
    let duplicate = format!(
        "{{\"version\":1,\"nodes\":{{\"192.0.2.1\":{node},\"192.0.2.1\":{node}}},\"deliveries\":[]}}"
    );
    let failure = State::from_json(duplicate.as_bytes()).unwrap_err();
    assert!(failure.message.contains("duplicate address"));
}
#[test]
fn bound_full_sixteen_node_chain_and_all_members_derive_without_overflow() {
    let mut s = State {
        version: VERSION,
        nodes: BTreeMap::new(),
        deliveries: vec![],
    };
    s.nodes.insert(ip(1), reference(1, 1));
    for n in 2..=16 {
        s.nodes.insert(ip(n), device(n, Some(n - 1)));
        s.deliveries.push(delivery(n, n - 1));
    }
    assert!(s.validate().is_ok());
    let all = s.statuses().unwrap();
    assert_eq!(all.len(), 16);
    assert_eq!(all[&ip(15)].stratum, 15);
    assert_eq!(all[&ip(16)].stratum, 16);
    assert_eq!(all[&ip(16)].chain.len(), 16);
    s.nodes.insert(ip(17), device(17, None));
    assert_eq!(s.statuses().unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(
        State::from_json(&vec![b' '; MAX_STATE_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
#[test]
fn malformed_goals_cannot_pass_by_empty_duplicate_or_foreign_identity() {
    let s = fixture();
    assert!(check(&s, &[]).is_err());
    let mut g = goal();
    g.push(g[0].clone());
    assert!(check(&s, &g).is_err());
    let mut g = goal();
    g[0].allowed_references.clear();
    assert!(check(&s, &g).is_err());
    g = goal();
    g[0].allowed_references.insert(ip(99));
    assert!(check(&s, &g).is_err());
    g = goal();
    g[0].client = ip(99);
    assert!(check(&s, &g).is_err());
}

#[test]
fn synchronized_but_wrong_configured_source_fails_goal_and_transport_hides_unknown_stratum() {
    let mut s = fixture();
    s.deliveries.push(delivery(3, 1));
    s.configure(ip(3), Some(ip(1))).unwrap();
    assert!(s.status(ip(3)).unwrap().synchronized);
    assert!(!check(&s, &goal()).unwrap()[1].passed);
    s.configure(ip(3), Some(ip(2))).unwrap();
    s.deliveries
        .iter_mut()
        .find(|d| d.client == ip(3) && d.server == ip(2))
        .unwrap()
        .reply_delivered = false;
    let output = s.execute(ip(3), "show ntp associations").unwrap();
    assert!(output.contains("--") && output.contains("not selected"));
}
#[test]
fn oversized_node_label_collections_and_commands_fail_before_unbounded_work() {
    let mut s = fixture();
    let before = s.clone();
    assert_eq!(
        s.execute(ip(2), &"x".repeat(513)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["nodes"]["192.0.2.1"]["label"] = "x".repeat(33).into();
    assert!(
        State::from_json(&serde_json::to_vec(&raw).unwrap())
            .unwrap_err()
            .message
            .contains("bounded NTP node label")
    );
    let node = serde_json::to_string(&device(1, None)).unwrap();
    let entries = (1..=17)
        .map(|n| format!("\"192.0.2.{n}\":{node}"))
        .collect::<Vec<_>>()
        .join(",");
    let raw = format!("{{\"version\":1,\"nodes\":{{{entries}}},\"deliveries\":[]}}");
    assert!(
        State::from_json(raw.as_bytes())
            .unwrap_err()
            .message
            .contains("node bound")
    );
    let d = serde_json::to_string(&delivery(2, 1)).unwrap();
    let over = std::iter::repeat_n(d, MAX_DELIVERIES + 1)
        .collect::<Vec<_>>()
        .join(",");
    let raw = format!("{{\"version\":1,\"nodes\":{{}},\"deliveries\":[{over}]}}");
    // Raw admission is stricter than a maximum collection for this encoding.
    assert!(State::from_json(raw.as_bytes()).is_err());
}

#[test]
fn restored_attempt_cannot_rewrite_supplied_environment_to_award_success() {
    let initial = fixture();
    let mut s = initial.clone();
    s.configure(ip(2), Some(ip(4))).unwrap();
    assert!(environment_matches(&s, &initial).unwrap());
    assert!(
        check_with_environment(&s, &initial, &goal())
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    for n in 0..5 {
        let mut changed = s.clone();
        match n {
            0 => changed.deliveries[0].reply_delivered = false,
            1 => {
                changed.nodes.get_mut(&ip(1)).unwrap().role = Role::Reference {
                    stratum: 2,
                    healthy: true,
                }
            }
            2 => changed.nodes.get_mut(&ip(1)).unwrap().online = false,
            3 => changed.nodes.get_mut(&ip(2)).unwrap().label = "changed-router".into(),
            _ => {
                changed.nodes.get_mut(&ip(2)).unwrap().role = Role::Reference {
                    stratum: 1,
                    healthy: true,
                }
            }
        }
        assert!(!environment_matches(&changed, &initial).unwrap());
        assert!(check_with_environment(&changed, &initial, &goal()).is_err());
    }
    assert!(
        environment_matches(&State::from_json(&s.to_json().unwrap()).unwrap(), &initial).unwrap()
    );
}
