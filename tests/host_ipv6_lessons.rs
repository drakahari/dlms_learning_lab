//! Three authored and registered IPv6 activities: real state and separate practice.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    host_workspace as work, pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde_json::Value;
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-host-ipv6-addition",
    "lpic2-host-ipv6-gateway-scope",
    "lpic2-host-dual-stack-handover",
];
fn labs() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        let old = pack::accepted307().unwrap();
        let mut texts = old
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        texts.push(include_str!("../scenarios/host-ipv6-lessons-v1.json").into());
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::HostNetwork { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &Attempt) -> &work::Session {
    let State::HostNetwork { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn session_mut(a: &mut Attempt) -> &mut work::Session {
    let State::HostNetwork { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn command(session: &mut work::Session, d: &work::Definition, c: &str, practice: bool) {
    session
        .run(d, &work::Event::Command { input: c.into() }, practice)
        .unwrap_or_else(|e| panic!("{c}: {e}"));
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn authored_references_and_examples_repair_real_state_while_both_ipv4_contexts_work() {
    assert_eq!(labs().len(), 3);
    for s in labs() {
        let d = definition(s);
        d.validate().unwrap();
        assert!(d.initial.profile.ipv6.is_some() && d.example.profile.ipv6.is_some());
        assert!(!engine::solved(&s.initial, &s.goal));
        for (world, targets) in [
            (&d.initial, ["192.0.2.20", "203.0.113.80", "10.40.0.20"]),
            (&d.example, ["10.31.0.20", "10.39.0.80", "10.34.0.20"]),
        ] {
            for ip in targets {
                let p = world.probe(ip.parse().unwrap());
                assert!(p.pass, "{} {ip}: {p:?}", s.id);
            }
        }
        let good = solved(s);
        let w = &session(&good).assignment.world;
        assert!(
            w.probe_ipv6("2001:db8:140::20".parse().unwrap()).pass,
            "Protected management is actually usable"
        );
        let mut ex = Attempt::new(s, 2);
        let before = session(&ex).assignment.clone();
        for e in &d.example_actions {
            session_mut(&mut ex).run(d, e, true).unwrap();
        }
        assert!(d.checks(session(&ex), true).iter().all(|r| r.0));
        assert_eq!(session(&ex).assignment, before);
        assert!(!progress::current_solved(&ex, s));
        let assignment_lans = &d.initial.profile.ipv6.as_ref().unwrap().lans;
        let example_lans = &d.example.profile.ipv6.as_ref().unwrap().lans;
        assert!(
            assignment_lans
                .iter()
                .all(|a| example_lans.iter().all(|b| a.network != b.network)),
            "Different worked prefixes, not assignment answers"
        );
    }
}
#[test]
fn taught_adverse_changes_keep_correct_results_and_retry_without_resetting_assignment() {
    let f: Value =
        serde_json::from_str(include_str!("fixtures/host-ipv6-experiments-v1.json")).unwrap();
    assert_eq!(f["version"], 1);
    for (s, recipe) in labs().iter().zip(f["experiments"].as_array().unwrap()) {
        assert_eq!(s.id, recipe["id"].as_str().unwrap());
        let d = definition(s);
        let mut a = Attempt::new(s, 1);
        let before = a.state.clone();
        for e in &d.example_actions {
            session_mut(&mut a).run(d, e, true).unwrap();
        }
        for c in recipe["commands"].as_array().unwrap() {
            command(session_mut(&mut a), d, c.as_str().unwrap(), true);
        }
        let rows = d.checks(session(&a), true);
        let expected: Vec<bool> = serde_json::from_value(recipe["expected"].clone()).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            expected,
            "{} {rows:?}",
            s.id
        );
        assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
        for c in recipe["repair"].as_array().unwrap() {
            command(session_mut(&mut a), d, c.as_str().unwrap(), true);
        }
        assert!(d.checks(session(&a), true).iter().all(|r| r.0));
        assert_eq!(
            a.state,
            match before {
                State::HostNetwork { mut simulation } => {
                    simulation.practice = session(&a).practice.clone();
                    State::HostNetwork { simulation }
                }
                _ => panic!(),
            },
            "Only separate practice changes"
        );
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn read_only_fake_success_and_partial_repairs_do_not_pass_and_feedback_identifies_the_mistake() {
    for s in labs() {
        let d = definition(s);
        let mut a = Attempt::new(s, 1);
        for c in [
            "ip -4 address show",
            "ip -6 address show",
            "ip -6 route show",
        ] {
            command(session_mut(&mut a), d, c, false);
        }
        session_mut(&mut a).assignment.checked = true;
        session_mut(&mut a).assignment.probe_result = "Path available · fake text".into();
        assert!(!progress::current_solved(&a, s));
        if s.id == IDS[0] {
            command(
                session_mut(&mut a),
                d,
                "ip -6 address add 2001:db8:110::41/60 dev ens3",
                false,
            );
            let rows = d.checks(session(&a), false);
            assert!(!rows[1].0 && rows[1].1.contains("/60"));
            assert!(
                rows[2].0,
                "Nearby traffic can work with the wrong required mask"
            );
        } else {
            command(
                session_mut(&mut a),
                d,
                "ip -6 route replace default via fe80::1 dev ens3 metric 100",
                false,
            );
            let rows = d.checks(session(&a), false);
            assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
            assert!(
                rows.iter().any(|r| !r.0 && r.1.contains("/128")),
                "{} {rows:?}",
                s.id
            );
        }
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn alternate_addresses_metrics_and_host_route_repairs_pass_then_breaking_state_invalidates_current_success()
 {
    for (s, commands) in labs().iter().zip([
        vec!["ip -6 address add 2001:0db8:0110:0000:ffff:ffff:ffff:ffff/64 dev ens3"],
        vec![
            "ip -6 route add default via fe80::1 dev ens3 metric 9",
            "ip -6 route replace 2001:db8:190::80/128 via fe80::1 dev ens3 metric 10",
        ],
        vec![
            "ip -6 route del 2001:db8:190::90/128 via fe80::2 dev ens3 metric 5",
            "ip -6 address del 2001:db8:110::44/60 dev ens3",
            "ip -6 address add 2001:db8:110::44/64 dev ens3",
            "ip -6 route add default via fe80::1 dev ens3 metric 55",
        ],
    ]) {
        let d = definition(s);
        let mut a = Attempt::new(s, 1);
        for c in commands {
            command(session_mut(&mut a), d, c, false);
        }
        assert!(
            progress::current_solved(&a, s),
            "{} {:?}",
            s.id,
            d.checks(session(&a), false)
        );
        command(
            session_mut(&mut a),
            d,
            "ip -6 route del 2001:db8:140::/64 via fe80::1 dev ens4 metric 50",
            false,
        );
        let rows = d.checks(session(&a), false);
        assert!(!progress::current_solved(&a, s));
        assert!(rows[..rows.len() - 1].iter().all(|r| r.0));
        assert!(!rows.last().unwrap().0 && rows.last().unwrap().1.contains("2001:db8:140::/64"));
        command(
            session_mut(&mut a),
            d,
            "ip -6 route add 2001:db8:140::/64 via fe80::1 dev ens4 metric 50",
            false,
        );
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn actual_safe_save_resume_keeps_long_ipv6_and_separate_drafts_favorites_and_selected_reset() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = solved(s);
        let x = session_mut(&mut a);
        x.assignment.command = "ip -6 route ".into();
        x.practice.command = "ip -6 address ".into();
        x.assignment.probe_ipv6_input = "2001:db8:110:0:ffff:ffff:ffff:ffff".into();
        x.view = View::Assignment;
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, labs()).unwrap();
    let mut q = progress::load(&path, labs()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    for s in labs() {
        assert!(progress::current_solved(&q.attempts[&s.id], s));
    }
    let other = serde_json::to_value(&q.attempts[IDS[1]]).unwrap();
    q.attempts
        .insert(IDS[0].into(), Attempt::new(&labs()[0], 2));
    assert_eq!(q.attempts[IDS[0]].state, labs()[0].initial);
    assert_eq!(serde_json::to_value(&q.attempts[IDS[1]]).unwrap(), other);
    assert!(q.favorites.contains(IDS[0]));
}

#[test]
fn registered_dual_stack_unit_preserves307_labs103_units_and_prerequisite_order() {
    let old = pack::accepted307().unwrap();
    let all = pack::accepted310().unwrap();
    assert_eq!((old.len(), all.len()), (307, 310));
    assert_eq!(
        serde_json::to_value(&all[..307]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 109, 50, 77]
    );
    for s in labs() {
        let at = all.iter().position(|r| r.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(&all[at]).unwrap(),
            serde_json::to_value(s).unwrap()
        );
        for p in &s.prerequisites {
            assert!(all.iter().position(|r| &r.id == p).unwrap() < at);
        }
    }
    let before = dlms_learning_lab::course::accepted307(&old).unwrap();
    let units = dlms_learning_lab::course::accepted310(&all).unwrap();
    assert_eq!((before.len(), units.len()), (103, 104));
    let kept = units
        .iter()
        .filter(|u| u.id != "linux-host-ipv6")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-host-ipv6")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 42, 16, 26]
    );
}
