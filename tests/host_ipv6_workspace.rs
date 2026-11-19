//! Unregistered source-binding proof: typed outcomes, isolated examples and real storage.
use dlms_learning_lab::{
    engine::{self, State},
    host_workspace as w,
    ipv6::Prefix,
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde_json::json;
#[path = "support/host_ipv6.rs"]
mod f;
fn command(c: &str) -> w::Event {
    w::Event::Command { input: c.into() }
}
fn definition() -> w::Definition {
    let mut initial = f::fixture();
    initial.configuration.ipv6.as_mut().unwrap().interfaces[0].address = None;
    let mut example = initial.clone();
    example.profile.id = "separate-ipv6-source-proof".into();
    let requirements = vec![
        w::Requirement::Ipv6Address {
            device: "ens3".into(),
            network: Prefix::parse("2001:db8:10::/64").unwrap(),
        },
        w::Requirement::Ipv6Reach {
            target: "2001:db8:10::20".parse().unwrap(),
            device: "ens3".into(),
        },
        w::Requirement::Ipv6Default {
            device: "ens3".into(),
            via: "fe80::1".parse().unwrap(),
        },
        w::Requirement::Ipv6Reach {
            target: "2001:db8:90::80".parse().unwrap(),
            device: "ens3".into(),
        },
        w::Requirement::PreserveIpv4 {},
        w::Requirement::PreserveIpv6Interface {
            device: "ens4".into(),
        },
    ];
    let d = w::Definition {
        direct_dns: None,
        echo: None,
        initial,
        example,
        requirements: requirements.clone(),
        example_requirements: requirements,
        example_actions: vec![
            command("ip -6 address add 2001:db8:10::44/64 dev ens3"),
            command("ip -6 route add default via fe80::1 dev ens3 metric 111"),
        ],
    };
    d.validate().unwrap();
    d
}
fn apply(s: &mut w::Session, d: &w::Definition, c: &str, p: bool) {
    s.run(d, &command(c), p).unwrap();
}
fn passes(s: &w::Session, d: &w::Definition, p: bool) -> bool {
    d.checks(s, p).iter().all(|r| r.0)
}
#[test]
fn new_outcomes_grade_real_state_and_multiple_addresses_metrics_without_exact_commands() {
    let d = definition();
    for (addr, metric) in [
        ("2001:db8:10::41", 1),
        ("2001:db8:10::42", 1024),
        ("2001:db8:10:0:ffff:ffff:ffff:ffff", u32::MAX),
    ] {
        let mut s = w::Session::new(&d);
        assert!(!passes(&s, &d, false));
        let initial = s.assignment.world.clone();
        apply(&mut s, &d, "ip -6 address show", false);
        assert_eq!(s.assignment.world, initial);
        assert!(!passes(&s, &d, false));
        apply(
            &mut s,
            &d,
            &format!("ip -6 address add {addr}/64 dev ens3"),
            false,
        );
        let rows = d.checks(&s, false);
        assert!(rows[1].0 && rows[2].0 && !rows[3].0 && !rows[4].0 && rows[5].0 && rows[6].0);
        apply(
            &mut s,
            &d,
            &format!("ip -6 route add default via fe80::1 dev ens3 metric {metric}"),
            false,
        );
        assert!(passes(&s, &d, false));
        apply(&mut s, &d, "ip link set dev ens3 down", false);
        let rows = d.checks(&s, false);
        assert!(!rows[2].0 && !rows[4].0 && !rows[5].0);
        assert!(
            rows[1].0 && rows[3].0 && rows[6].0,
            "Correct independent checks remain correct"
        );
        apply(&mut s, &d, "ip link set dev ens3 up", false);
        assert!(passes(&s, &d, false));
    }
}
#[test]
fn wrong_prefix_and_gateway_have_specific_feedback_with_legitimate_partial_results() {
    let d = definition();
    let mut s = w::Session::new(&d);
    apply(
        &mut s,
        &d,
        "ip -6 address add 2001:db8:10::41/60 dev ens3",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 route add default via fe80::1 dev ens3 metric 100",
        false,
    );
    let rows = d.checks(&s, false);
    assert!(!rows[1].0);
    assert!(rows[1].1.contains("2001:db8:10::/64") && rows[1].1.contains("2001:db8:10::41/60"));
    assert!(
        rows[2].0 && rows[3].0 && rows[4].0,
        "Wrong mask does not imply all paths fail"
    );
    apply(
        &mut s,
        &d,
        "ip -6 route del default via fe80::1 dev ens3 metric 100",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 address del 2001:db8:10::41/60 dev ens3",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 address add 2001:db8:10::41/64 dev ens3",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 route add default via fe80::1 dev ens4 metric 10",
        false,
    );
    let rows = d.checks(&s, false);
    assert!(!rows[3].0 && rows[3].1.contains("both address and device"));
    assert!(!rows[4].0 && rows[4].1.contains("unavailable"));
    assert!(rows[1].0 && rows[2].0 && rows[5].0 && rows[6].0);
    apply(
        &mut s,
        &d,
        "ip -6 route replace default via fe80::1 dev ens3 metric 10",
        false,
    );
    assert!(passes(&s, &d, false));
}
#[test]
fn example_credit_source_capability_parity_and_poisoned_contexts_are_honest() {
    let d = definition();
    let mut s = w::Session::new(&d);
    let original = s.assignment.clone();
    for c in &d.example_actions {
        s.run(&d, c, true).unwrap();
    }
    assert!(passes(&s, &d, true));
    assert_eq!(s.assignment, original);
    assert!(!passes(&s, &d, false));
    let mut wrong = d.clone();
    wrong.example.profile.ipv6 = None;
    wrong.example.configuration.ipv6 = None;
    assert!(
        wrong
            .validate()
            .unwrap_err()
            .contains("same source-enabled")
    );
    for requirement in [
        json!({"kind":"Ipv6Address","device":"ens3","network":{"address":"2001:db8:ff::","length":64}}),
        json!({"kind":"Ipv6Default","device":"ens3","via":"fe80::99"}),
        json!({"kind":"Ipv6Reach","target":"2001:db8:ff::1","device":"ens3"}),
        json!({"kind":"PreserveIpv6Routes","destination":{"address":"2001:db8:40::","length":64}}),
    ] {
        let mut wrong = d.clone();
        wrong.requirements[0] = serde_json::from_value(requirement).unwrap();
        assert!(wrong.validate().is_err());
    }
    let mut foreign = s.clone();
    foreign
        .assignment
        .world
        .profile
        .ipv6
        .as_mut()
        .unwrap()
        .nodes[0]
        .available = false;
    let before = foreign.clone();
    assert!(
        foreign
            .run(&d, &command("ip -6 address show"), false)
            .is_err()
    );
    assert_eq!(foreign, before);
    let mut forged = w::Session::new(&d);
    forged
        .assignment
        .world
        .configuration
        .ipv6
        .as_mut()
        .unwrap()
        .interfaces[0]
        .address = Some(dlms_learning_lab::linux_host::ipv6_host::Assignment {
        address: "2001:db8:10::41".parse().unwrap(),
        prefix: 64,
    });
    assert!(
        d.bound(&forged).is_err(),
        "A current state without matching applied history cannot be imported as proven work"
    );
    let env = dlms_learning_lab::nm_policy::Environment {
        bridge: None,
        bond: None,
        vlan: None,
        physical: d.initial.profile.clone(),
        default_metrics: std::collections::BTreeMap::from([
            ("ens3".into(), 100),
            ("ens4".into(), 200),
        ]),
        dhcp: Default::default(),
    };
    assert!(env.validate().unwrap_err().message.contains("IPv4 only"));
}
fn reference(c: &str) -> String {
    format!(
        "host-action {}",
        serde_json::to_string(&command(c)).unwrap()
    )
}
fn scenario() -> pack::Scenario {
    let d = definition();
    let session = w::Session::new(&d);
    let mut raw = json!({"schema_version":1,"scenarios":[{"id":"ipv6-source-proof","version":1,"track":"LPIC-2","topic":"Unregistered developer fixture","title":"Synthetic IPv6 proof","summary":"Not authored curriculum or catalog coverage.","objective":"Repair the supplied IPv6 address and route while preserving all IPv4 configuration and ens4 IPv6 state.","difficulty":"Intermediate","minutes":1,"steps":["Understand the task"],"guide":[],"initial":{"kind":"HostNetwork","simulation":session},"goal":{"kind":"HostNetwork","definition":d},"solution":["ip -6 address add 2001:db8:10::41/64 dev ens3","ip -6 route add default via fe80::1 dev ens3 metric 100"],"hints":["Inspect each address family separately."],"explanation":"Developer source and persistence proof only.","sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"simplifications":"Supplied IPv6 facts and static paths, no real network."}]});
    raw["scenarios"][0]["solution"] = json!([
        reference("ip -6 address add 2001:db8:10::41/64 dev ens3"),
        reference("ip -6 route add default via fe80::1 dev ens3 metric 100")
    ]);
    pack::load(&raw.to_string()).unwrap().remove(0)
}
#[test]
fn source_bound_safe_resume_keeps_separate_drafts_favorites_and_current_invalidation() {
    let s = scenario();
    let mut other = s.clone();
    other.id = "ipv6-other-proof".into();
    let catalog = vec![s.clone(), other.clone()];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(other.id.clone(), Attempt::new(&other, 2));
    let other_before = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let mut a = Attempt::new(&s, 1);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let State::HostNetwork { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.command = "ip -6 route ".into();
    simulation.practice.command = "ip -6 address ".into();
    simulation.assignment.probe_ipv6_input = "2001:db8:10:0:ffff:ffff:ffff:ffff".into();
    simulation.view = View::Assignment;
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let mut read = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    assert!(progress::current_solved(&read.attempts[&s.id], &s));
    engine::execute_for_goal(
        &mut read.attempts.get_mut(&s.id).unwrap().state,
        &s.goal,
        &reference("ip link set dev ens3 down"),
    )
    .unwrap();
    assert!(!progress::current_solved(&read.attempts[&s.id], &s));
    read.attempts.insert(s.id.clone(), Attempt::new(&s, 3));
    assert_eq!(read.attempts[&s.id].state, s.initial);
    assert_eq!(
        serde_json::to_value(&read.attempts[&other.id]).unwrap(),
        other_before
    );
    assert!(read.favorites.contains(&s.id));
}
#[test]
fn old_workspace_bytes_and_probe_limits_remain_unchanged_without_new_source_facts() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    for source in raw["scenarios"].as_array().unwrap() {
        let before = source["initial"]["simulation"].clone();
        let mut session: w::Session = serde_json::from_value(before.clone()).unwrap();
        session.validate().unwrap();
        assert_eq!(serde_json::to_value(&session).unwrap(), before);
        session.assignment.probe_ipv6_input = "::1".into();
        assert!(session.validate().is_err());
        session.assignment.probe_ipv6_input.clear();
        session.assignment.probe_input = "x".repeat(33);
        assert!(session.validate().is_err());
    }
    let d = definition();
    let mut session = w::Session::new(&d);
    session.assignment.probe_ipv6_input = "x".repeat(65);
    assert!(session.validate().is_err());
}

#[test]
fn preserved_ipv6_routes_and_management_address_have_independent_current_checks() {
    let mut d = definition();
    let route = dlms_learning_lab::linux_host::ipv6_host::Route {
        destination: Prefix::parse("2001:db8:40::/64").unwrap(),
        via: "fe80::1".parse().unwrap(),
        device: "ens4".into(),
        metric: 50,
    };
    for world in [&mut d.initial, &mut d.example] {
        world
            .configuration
            .ipv6
            .as_mut()
            .unwrap()
            .routes
            .push(route.clone());
    }
    let req = w::Requirement::PreserveIpv6Routes {
        destination: route.destination,
    };
    d.requirements.push(req.clone());
    d.example_requirements.push(req);
    d.validate().unwrap();
    let mut s = w::Session::new(&d);
    apply(
        &mut s,
        &d,
        "ip -6 address add 2001:db8:10::41/64 dev ens3",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 route add default via fe80::1 dev ens3 metric 100",
        false,
    );
    assert!(passes(&s, &d, false));
    apply(
        &mut s,
        &d,
        "ip -6 route del 2001:db8:40::/64 via fe80::1 dev ens4 metric 50",
        false,
    );
    let rows = d.checks(&s, false);
    assert!(rows[..7].iter().all(|r| r.0));
    assert!(!rows[7].0 && rows[7].1.contains("2001:db8:40::/64"));
    apply(
        &mut s,
        &d,
        "ip -6 address del 2001:db8:20::40/64 dev ens4",
        false,
    );
    let rows = d.checks(&s, false);
    assert!(rows[..6].iter().all(|r| r.0));
    assert!(!rows[6].0 && !rows[7].0);
    apply(
        &mut s,
        &d,
        "ip -6 address add 2001:db8:20::40/64 dev ens4",
        false,
    );
    apply(
        &mut s,
        &d,
        "ip -6 route add 2001:db8:40::/64 via fe80::1 dev ens4 metric 50",
        false,
    );
    assert!(passes(&s, &d, false));
}
