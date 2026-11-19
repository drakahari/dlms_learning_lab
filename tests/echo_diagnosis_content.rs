//! Authored preview: do not infer catalog admission from fixture success.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    host_workspace as w, pack,
    progress::{self, Attempt},
};
use std::sync::OnceLock;
fn source() -> &'static pack::Scenario {
    static S: OnceLock<pack::Scenario> = OnceLock::new();
    S.get_or_init(|| {
        pack::load(include_str!("fixtures/echo-diagnosis-preview-v1.json"))
            .unwrap()
            .remove(0)
    })
}
fn def() -> &'static w::Definition {
    let Goal::HostNetwork { definition } = &source().goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::HostNetwork { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn run(a: &mut Attempt, c: &str) {
    sim(a)
        .run(def(), &w::Event::Command { input: c.into() }, false)
        .unwrap();
}
fn solve(a: &mut Attempt) {
    for action in &source().solution {
        engine::execute_for_goal(&mut a.state, &source().goal, action).unwrap();
    }
}
fn checks(a: &mut Attempt) -> Vec<(bool, String)> {
    def().checks(sim(a), false)
}
#[test]
fn taught_example_is_reachable_and_cannot_change_or_complete_the_independent_task() {
    let d = def();
    d.validate().unwrap();
    let mut a = Attempt::new(source(), 1);
    let initial = sim(&mut a).assignment.clone();
    assert_eq!(checks(&mut a).len(), 13);
    assert!(!progress::current_solved(&a, source()));
    assert_ne!(d.initial.profile, d.example.profile);
    for event in &d.example_actions {
        sim(&mut a).run(d, event, true).unwrap();
    }
    assert!(d.checks(sim(&mut a), true).iter().all(|r| r.0));
    assert_eq!(sim(&mut a).assignment, initial);
    assert!(!progress::current_solved(&a, source()));
    solve(&mut a);
    assert!(checks(&mut a).iter().all(|r| r.0));
    assert!(progress::current_solved(&a, source()));
}
#[test]
fn nearby_reply_is_not_complete_success_and_correct_default_does_not_hide_a_host_route() {
    let mut a = Attempt::new(source(), 1);
    let p = &def().echo.as_ref().unwrap().initial.plan;
    assert!(
        p.observe(
            &sim(&mut a).assignment.world,
            "10.96.19.20".parse().unwrap()
        )
        .unwrap()
        .report
        .unwrap()
        .round_trip()
    );
    assert!(!progress::current_solved(&a, source()));
    for action in &source().solution[..3] {
        engine::execute_for_goal(&mut a.state, &source().goal, action).unwrap();
    }
    assert!(
        p.observe(
            &sim(&mut a).assignment.world,
            "10.96.30.20".parse().unwrap()
        )
        .unwrap()
        .report
        .unwrap()
        .round_trip()
    );
    assert!(
        !p.observe(
            &sim(&mut a).assignment.world,
            "172.22.40.20".parse().unwrap()
        )
        .unwrap()
        .report
        .unwrap()
        .round_trip()
    );
    let rows = checks(&mut a);
    assert!(rows.iter().any(|r| r.0));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("172.22.40.20")));
    assert!(!progress::current_solved(&a, source()));
}
#[test]
fn alternate_address_and_repaired_specific_route_pass_without_exact_solution_strings() {
    let mut a = Attempt::new(source(), 1);
    run(&mut a, "ip -4 address del 10.96.19.10/24 dev ens3");
    // .255 is an ordinary host here: it is not the end of the complete /20.
    run(&mut a, "ip -4 address add 10.96.17.255/20 dev ens3");
    run(
        &mut a,
        "ip -4 route replace default via 10.96.16.1 dev ens3 metric 5",
    );
    run(
        &mut a,
        "ip -4 route replace 172.22.40.20/32 via 10.96.16.1 dev ens3 metric 1",
    );
    assert!(checks(&mut a).iter().all(|r| r.0));
    assert!(progress::current_solved(&a, source()));
    assert!(
        sim(&mut a)
            .assignment
            .world
            .configuration
            .routes
            .iter()
            .any(|r| r.destination.length == 32)
    );
}
#[test]
fn network_broadcast_peer_conflict_wrong_mask_and_copied_example_do_not_pass() {
    for addr in [
        "10.96.16.0/20",
        "10.96.31.255/20",
        "10.96.30.20/20",
        "10.96.16.1/20",
        "10.96.19.10/24",
        "10.110.67.10/20",
    ] {
        let mut a = Attempt::new(source(), 1);
        solve(&mut a);
        run(
            &mut a,
            "ip -4 route del default via 10.96.16.1 dev ens3 metric 5",
        );
        run(&mut a, "ip -4 address del 10.96.19.10/20 dev ens3");
        let result = sim(&mut a).run(
            def(),
            &w::Event::Command {
                input: format!("ip -4 address add {addr} dev ens3"),
            },
            false,
        );
        if result.is_ok() {
            let rows = checks(&mut a);
            assert!(
                rows.iter()
                    .any(|r| !r.0 && r.1.contains("unused conventional host address")),
                "The address/prefix requirement itself must diagnose {addr}: {rows:?}"
            );
            assert!(
                rows.iter().any(|r| r.0),
                "Unrelated preserved management outcomes still pass"
            );
        }
        assert!(!progress::current_solved(&a, source()), "{addr}");
    }
}
#[test]
fn silent_control_is_not_an_error_requirement_or_a_negative_grading_shortcut() {
    let mut a = Attempt::new(source(), 1);
    solve(&mut a);
    let exercise = &def().echo.as_ref().unwrap().initial;
    let control = "172.22.40.99".parse().unwrap();
    assert!(!exercise.required_replies.contains(&control));
    let o = exercise
        .plan
        .observe(&sim(&mut a).assignment.world, control)
        .unwrap();
    let r = o.report.unwrap();
    assert!(!r.round_trip());
    assert_eq!(r.events.len(), 1);
    assert!(progress::current_solved(&a, source()));
    run(&mut a, "ip link set dev ens3 down");
    assert!(!progress::current_solved(&a, source()));
    assert!(checks(&mut a).iter().any(|r| r.0));
    assert!(checks(&mut a).iter().any(|r| !r.0));
}
#[test]
fn invalid_commands_resume_reset_and_preservation_do_not_fabricate_mastery() {
    let s = source();
    let mut a = Attempt::new(s, 1);
    solve(&mut a);
    sim(&mut a).practice.command = "ping -c 1 10.111.8.99".into();
    for cmd in [
        "ping -c 99999 172.22.40.20",
        "ping -c 1 example.com",
        "ping -c 1 172.22.40.20; sh",
        "cat /etc/passwd",
    ] {
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            sim(&mut a)
                .run(def(), &w::Event::Command { input: cmd.into() }, false)
                .is_err()
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, std::slice::from_ref(s)).unwrap();
    let mut restored = progress::load(&file, std::slice::from_ref(s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[&s.id], s));
    let a = restored.attempts.get_mut(&s.id).unwrap();
    run(a, "ip link set dev ens4 down");
    assert!(!progress::current_solved(a, s));
    restored.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert!(!progress::current_solved(&restored.attempts[&s.id], s));
    assert!(restored.favorites.contains(&s.id));
}

#[test]
fn registered439_preserves_every438_scenario_and171_units_and_resolves_all_prerequisites() {
    let before = pack::accepted438().unwrap();
    let current = pack::current().unwrap();
    assert_eq!((before.len(), current.len()), (438, 439));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..438]).unwrap()
    );
    let added = current.iter().find(|s| s.id == source().id).unwrap();
    let mut expected = source().clone();
    expected.prerequisites = vec![
        "lpic1-host-mixed-ip".into(),
        "lpic1-protocol-round-trips".into(),
        "lpic1-listener-audit".into(),
    ];
    assert_eq!(
        serde_json::to_value(added).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let old_units = dlms_learning_lab::course::accepted438(&before).unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (171, 172));
    let retained = units
        .iter()
        .filter(|u| u.id != source().unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let pos = units.iter().position(|u| u.id == source().unit).unwrap();
    for name in ["linux-protocol-evidence", "linux-listener-audit"] {
        assert!(units.iter().position(|u| u.id == name).unwrap() < pos);
    }
}
