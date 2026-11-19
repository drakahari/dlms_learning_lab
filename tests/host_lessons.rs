use dlms_learning_lab::{
    engine::{Goal, State},
    host_workspace as w, pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap()
}
fn parts(s: &pack::Scenario) -> (&w::Definition, &w::Session) {
    let (Goal::HostNetwork { definition }, State::HostNetwork { simulation }) =
        (&s.goal, &s.initial)
    else {
        panic!()
    };
    (definition, simulation)
}
fn command(x: &str) -> w::Event {
    w::Event::Command { input: x.into() }
}
fn run(x: &mut w::Session, d: &w::Definition, text: &str) {
    x.run(d, &command(text), false).unwrap();
}
fn done(s: &pack::Scenario) -> w::Session {
    let (d, initial) = parts(s);
    let mut x = initial.clone();
    for c in &s.solution {
        x.execute_reference(d, c).unwrap();
    }
    x
}
fn passes(d: &w::Definition, x: &w::Session) -> bool {
    d.checks(x, false).iter().all(|r| r.0)
}
#[test]
fn host_four_working_solutions_separate_examples_and_reset() {
    for s in labs() {
        let (d, init) = parts(&s);
        assert!(!passes(d, init));
        let x = done(&s);
        assert!(passes(d, &x), "{} {:?}", s.id, d.checks(&x, false));
        let mut ex = init.clone();
        for e in &d.example_actions {
            ex.run(d, e, true).unwrap();
        }
        assert!(d.checks(&ex, true).iter().all(|r| r.0));
        assert_eq!(ex.assignment, init.assignment);
        assert!(!passes(d, &ex));
        assert_eq!(w::Session::new(d), *init);
    }
}
#[test]
fn host_missing_link_address_route_and_dns_are_distinct_failures() {
    let ls = labs();
    let s = &ls[3];
    let (d, init) = parts(s);
    let x = done(s);
    for cmd in [
        "ip link set dev ens3 down",
        "ip -4 route del default via 192.0.2.1 dev ens3 metric 5",
    ] {
        let mut y = x.clone();
        run(&mut y, d, cmd);
        let rows = d.checks(&y, false);
        assert!(rows.iter().any(|r| !r.0), "{cmd}");
        assert!(rows.iter().any(|r| r.0), "partial successes retained");
    }
    let mut y = init.clone();
    y.run(
        d,
        &w::Event::Resolver {
            text: "nameserver 203.0.113.53\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!passes(d, &y));
    assert!(y.assignment.world.resolve("service.lab.test").is_err());
    let mut y = x;
    y.run(
        d,
        &w::Event::Resolver {
            text: "nameserver 203.0.113.54\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(
        y.assignment
            .world
            .probe("203.0.113.54".parse().unwrap())
            .pass
    );
    assert!(!passes(d, &y));
    assert!(
        d.checks(&y, false)
            .iter()
            .any(|r| !r.0 && r.1.contains("Actual: 203.0.113.90"))
    );
}
#[test]
fn host_alternate_unused_addresses_and_operation_orders_pass() {
    for i in [0, 3] {
        let s = labs().remove(i);
        let (d, init) = parts(&s);
        for last in [2, 47, 126, 254] {
            let mut x = init.clone();
            run(&mut x, d, "ip link set dev ens3 up");
            run(&mut x, d, "ip -4 address flush dev ens3");
            run(
                &mut x,
                d,
                &format!("ip -4 address add 192.0.2.{last}/24 dev ens3"),
            );
            if i == 3 {
                run(
                    &mut x,
                    d,
                    "ip -4 route add default via 192.0.2.1 dev ens3 metric 1",
                );
                x.run(
                    d,
                    &w::Event::Resolver {
                        text: "# client resolver\n nameserver 203.0.113.53 # supplied\n".into(),
                    },
                    false,
                )
                .unwrap();
            }
            assert!(passes(d, &x), "{} {last} {:?}", s.id, d.checks(&x, false));
        }
    }
}
#[test]
fn host_network_broadcast_peer_gateway_and_wrong_prefix_cannot_pass() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    for addr in [
        "192.0.2.0/24",
        "192.0.2.255/24",
        "192.0.2.1/24",
        "192.0.2.20/24",
        "192.0.2.200/24",
        "192.0.2.10/25",
        "10.20.0.44/24",
    ] {
        let mut x = init.clone();
        run(&mut x, d, "ip -4 address flush dev ens3");
        let e = command(&format!("ip -4 address add {addr} dev ens3"));
        let _ = x.run(d, &e, false);
        run(&mut x, d, "ip link set dev ens3 up");
        assert!(!passes(d, &x), "{addr}");
    }
}
#[test]
fn host_route_metrics_and_replacements_accept_multiple_valid_repairs() {
    let s = labs().remove(1);
    let (d, init) = parts(&s);
    for high in [101, 1000, u32::MAX] {
        let mut x = init.clone();
        run(
            &mut x,
            d,
            "ip -4 route del default via 192.0.2.1 dev ens3 metric 10",
        );
        run(
            &mut x,
            d,
            &format!("ip -4 route add default via 192.0.2.1 dev ens3 metric {high}"),
        );
        run(
            &mut x,
            d,
            "ip -4 route replace 203.0.113.80/32 via 198.51.100.1 dev ens4 metric 999",
        );
        assert!(passes(d, &x), "{:?}", d.checks(&x, false));
    }
    let mut x = init.clone();
    run(
        &mut x,
        d,
        "ip -4 route add default via 198.51.100.1 dev ens4 metric 5",
    );
    assert!(!passes(d, &x), "longer host route still fails");
    run(
        &mut x,
        d,
        "ip -4 route del 203.0.113.80/32 via 192.0.2.1 dev ens3 metric 999",
    );
    assert!(passes(d, &x));
}
#[test]
fn host_management_changes_fail_even_when_application_works() {
    let s = labs().remove(3);
    let (d, _) = parts(&s);
    let mut x = done(&s);
    run(&mut x, d, "ip link set dev ens4 down");
    assert!(
        x.assignment
            .world
            .probe("203.0.113.80".parse().unwrap())
            .pass
    );
    assert!(!passes(d, &x));
    assert!(
        d.checks(&x, false)
            .iter()
            .any(|r| !r.0 && r.1.contains("Retain the complete supplied ens4"))
    );
    let mut x = done(&s);
    run(
        &mut x,
        d,
        "ip -4 route del 10.40.0.0/24 via 198.51.100.1 dev ens4 metric 50",
    );
    assert!(!passes(d, &x));
}
#[test]
fn host_dirty_resolver_clears_success_without_a_forgeable_boolean() {
    for s in labs() {
        let (d, _) = parts(&s);
        let mut x = done(&s);
        assert!(passes(d, &x));
        x.assignment.resolver_draft.push_str("# unsaved\n");
        assert!(!passes(d, &x));
        assert!(d.bound(&x).is_ok());
        x.assignment.resolver_draft = x.assignment.resolver_applied.clone();
        assert!(passes(d, &x));
    }
}
#[test]
fn host_resume_preserves_unfinished_commands_raw_resolver_comments_and_separate_practice() {
    let ls = labs();
    let s = &ls[2];
    let (d, init) = parts(s);
    let mut x = init.clone();
    x.view = View::Example;
    x.assignment.command = "ip -4 route show".into();
    let raw = "# saved original formatting\n nameserver 203.0.113.53 # retained\n";
    x.run(d, &w::Event::Resolver { text: raw.into() }, false)
        .unwrap();
    x.assignment.resolver_draft.push_str("# unfinished\n");
    x.practice.command = "getent ahostsv4 service.lab.test".into();
    let mut a = Attempt::new(s, 1);
    a.state = State::HostNetwork {
        simulation: Box::new(x),
    };
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &ls).unwrap();
    let got = progress::load(&path, &ls).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&got).unwrap()
    );
    assert!(!progress::current_solved(&got.attempts[&s.id], s));
    assert!(progress::save(&path, &p).is_err());
    let bytes = std::fs::read(&path).unwrap();
    p.attempts.get_mut(&s.id).unwrap().workspace =
        Some(dlms_learning_lab::workspace::WorkspaceData::new());
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = "foreign".into();
    assert!(progress::save_with_catalog(&path, &p, &ls).is_err());
    assert_eq!(bytes, std::fs::read(path).unwrap());
}
#[test]
fn host_forged_state_or_applied_text_cannot_replace_real_history() {
    let ls = labs();
    let s = &ls[0];
    let (d, init) = parts(s);
    for n in 0..3 {
        let mut x = init.clone();
        match n {
            0 => x.assignment.world = done(s).assignment.world,
            1 => x.assignment.resolver_applied = "# forged saved text".into(),
            _ => x.assignment.world.profile.gateways[0].available = false,
        };
        assert!(d.bound(&x).is_err());
        let mut a = Attempt::new(s, 1);
        a.state = State::HostNetwork {
            simulation: Box::new(x),
        };
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &ls).is_err());
    }
}
#[test]
fn host_unsupported_commands_and_invalid_resolver_keep_world_exact() {
    let s = labs().remove(2);
    let (d, init) = parts(&s);
    for input in [
        "sudo ip link show",
        "ip link show; id",
        "$(id)",
        "cat /etc/passwd",
        "ip route get 8.8.8.8",
        "ip -4 address replace 192.0.2.10/24 dev ens3",
    ] {
        let mut x = init.clone();
        assert!(x.run(d, &command(input), false).is_err());
        assert_eq!(x, *init);
    }
    for text in [
        "nameserver 8.8.8.8\n",
        "search example.com\n",
        "nameserver ../../personal\n",
        "nameserver 203.0.113.53\noptions rotate\n",
    ] {
        let mut x = init.clone();
        assert!(
            x.run(d, &w::Event::Resolver { text: text.into() }, false)
                .is_err()
        );
        assert_eq!(x, *init);
    }
}
#[test]
fn host_invalid_pack_and_unreachable_solution_refused() {
    let raw = include_str!("../scenarios/linux-host-lessons-v1.json");
    for n in 0..5 {
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let s = &mut v["scenarios"][0];
        match n {
            0 => s["goal"]["definition"]["requirements"] = serde_json::json!([]),
            1 => {
                s["initial"]["simulation"]["assignment"]["world"]["profile"]["id"] =
                    serde_json::json!("foreign")
            }
            2 => {
                s["goal"]["definition"]["example_actions"] =
                    serde_json::json!([{"kind":"Command","input":"ip link show"}])
            }
            3 => {
                s["solution"] = serde_json::json!([
                    "host-action {\"kind\":\"Command\",\"input\":\"ip link show\"}"
                ])
            }
            _ => s["goal"]["definition"]["example_actions"][0]["extra"] = serde_json::json!(true),
        };
        assert!(pack::load(&v.to_string()).is_err(), "{n}");
    }
}
#[test]
fn host_history_bound_is_atomic_and_transcript_cannot_award_credit() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    for i in 0..64 {
        run(
            &mut x,
            d,
            if i % 2 == 0 {
                "ip link set dev ens3 up"
            } else {
                "ip link set dev ens3 down"
            },
        );
    }
    let before = x.clone();
    assert!(
        x.run(d, &command("ip link set dev ens3 up"), false)
            .unwrap_err()
            .contains("64")
    );
    assert_eq!(x, before);
    x.assignment.transcript = vec!["All checks passed".into()];
    x.assignment.checked = true;
    assert!(!passes(d, &x));
}

#[test]
fn host_catalog_keeps_all225_objects_and60_units() {
    let prior = pack::accepted225().unwrap();
    let now = pack::accepted229().unwrap();
    assert_eq!(prior.len(), 225);
    assert_eq!(now.len(), 229);
    assert_eq!(
        serde_json::to_value(&now[..225]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    let old = dlms_learning_lab::course::accepted225(&prior).unwrap();
    let units = dlms_learning_lab::course::accepted229(&now).unwrap();
    assert_eq!(old.len(), 60);
    assert_eq!(units.len(), 61);
    assert_eq!(
        serde_json::to_value(&units[..60]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["linux-file-foundations"]
    );
    assert_eq!(
        units.last().unwrap().labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
}
