//! Registered mixed-IP bridge: existing parser/reducer, state-based outcomes and retained catalog.
use dlms_learning_lab::{
    engine::{self, State},
    host_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
fn source() -> &'static pack::Scenario {
    static S: OnceLock<pack::Scenario> = OnceLock::new();
    S.get_or_init(|| {
        pack::load(include_str!("fixtures/mixed-ip-preview-v1.json"))
            .unwrap()
            .remove(0)
    })
}
#[test]
fn current437_registers_the_authored_task_and_preserves_every436_scenario_and_169_unit_objects() {
    let before = pack::accepted436().unwrap();
    let current = pack::accepted437().unwrap();
    assert_eq!((before.len(), current.len()), (436, 437));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..436]).unwrap()
    );
    let added = current.iter().find(|s| s.id == source().id).unwrap();
    let mut expected = source().clone();
    expected.prerequisites = vec!["lpic1-host-handover".into()];
    assert_eq!(
        serde_json::to_value(added).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let old = dlms_learning_lab::course::accepted436(&before).unwrap();
    let units = dlms_learning_lab::course::accepted437(&current).unwrap();
    assert_eq!((old.len(), units.len()), (169, 170));
    let new = units.iter().position(|u| u.id == source().unit).unwrap();
    assert!(
        units
            .iter()
            .position(|u| u.id == "linux-host-networking")
            .unwrap()
            < new
    );
    assert!(
        new < units
            .iter()
            .position(|u| u.id == "linux-service-names")
            .unwrap()
    );
    let retained = units
        .iter()
        .filter(|u| u.id != source().unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}
fn def() -> &'static w::Definition {
    let engine::Goal::HostNetwork { definition } = &source().goal else {
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
fn command(a: &mut Attempt, s: &str, practice: bool) {
    sim(a)
        .run(def(), &w::Event::Command { input: s.into() }, practice)
        .unwrap();
}
fn reference(a: &mut Attempt) {
    for action in &source().solution {
        engine::execute_for_goal(&mut a.state, &source().goal, action).unwrap();
    }
}
#[test]
fn full_two_family_task_and_separate_taught_example_are_reachable() {
    let s = source();
    assert_eq!(def().requirements.len(), 13);
    assert!(def().initial.profile.ipv6.is_some());
    let mut a = Attempt::new(s, 1);
    let before = sim(&mut a).assignment.clone();
    for e in &def().example_actions {
        sim(&mut a).run(def(), e, true).unwrap();
    }
    assert!(def().checks(sim(&mut a), true).iter().all(|r| r.0));
    assert_eq!(sim(&mut a).assignment, before);
    assert!(!progress::current_solved(&a, s));
    reference(&mut a);
    assert!(progress::current_solved(&a, s));
}
#[test]
fn each_family_can_fail_independently_and_partial_repair_stays_partial() {
    let s = source();
    let mut a = Attempt::new(s, 1);
    assert!(!progress::current_solved(&a, s));
    for action in &s.solution[..3] {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    let rows = def().checks(sim(&mut a), false);
    assert!(rows.iter().any(|r| r.0 && r.1.contains("203.0.113.80")));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("2001:db8:80::80")));
    assert!(!progress::current_solved(&a, s));
    for action in &s.solution[3..] {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    command(
        &mut a,
        "ip -6 route replace default via fe80::1 dev ens4 metric 100",
        false,
    );
    assert!(!progress::current_solved(&a, s));
    let rows = def().checks(sim(&mut a), false);
    assert!(rows.iter().any(|r| r.0 && r.1.contains("203.0.113.80")));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("2001:db8:80::80")));
}
#[test]
fn alternate_unused_addresses_and_an_additional_lower_metric_default_are_valid() {
    let s = source();
    for (v4, v6) in [
        ("10.24.0.30", "2001:db8:24::30"),
        ("10.24.0.62", "2001:db8:24::abcd"),
    ] {
        let mut a = Attempt::new(s, 1);
        for action in &s.solution {
            let action = action
                .replace("10.24.0.50/26", &format!("{v4}/26"))
                .replace("2001:db8:24::50/64", &format!("{v6}/64"))
                .replace("route replace default", "route add default")
                .replace("metric 100", "metric 10");
            engine::execute_for_goal(&mut a.state, &s.goal, &action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn on_link_peers_network_broadcast_wrong_prefix_and_management_breakage_cannot_pass() {
    let s = source();
    for replacement in [
        "10.24.0.0/26",
        "10.24.0.63/26",
        "10.24.0.20/26",
        "10.24.0.1/26",
        "10.24.0.50/24",
    ] {
        let mut a = Attempt::new(s, 1);
        for action in &s.solution {
            let action = action.replace("10.24.0.50/26", replacement);
            let _ = engine::execute_for_goal(&mut a.state, &s.goal, &action);
        }
        assert!(!progress::current_solved(&a, s), "{replacement}");
    }
    for bad in [
        "ip -4 route del 198.51.100.0/24 via 192.0.2.1 dev ens4 metric 50",
        "ip -6 route del 2001:db8:99::/64 via fe80::1 dev ens4 metric 50",
        "ip link set dev ens4 down",
    ] {
        let mut a = Attempt::new(s, 1);
        reference(&mut a);
        command(&mut a, bad, false);
        assert!(!progress::current_solved(&a, s), "{bad}");
        assert!(def().checks(sim(&mut a), false).iter().any(|r| !r.0));
    }
}
#[test]
fn malformed_commands_fail_atomically_and_resume_keeps_separate_work_and_reset_local() {
    let s = source();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    reference(&mut a);
    for input in [
        "sh /private",
        "ip -4 route replace default via fe80::1 dev ens3",
        "ip -6 address add 2001:db8:24::50/129 dev ens3",
        "ip -6 route show | sh",
    ] {
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            sim(&mut a)
                .run(
                    def(),
                    &w::Event::Command {
                        input: input.into()
                    },
                    false
                )
                .is_err()
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    sim(&mut a).practice.command = "ip -6 route".into();
    sim(&mut a).view = dlms_learning_lab::script_teaching::View::Assignment;
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(s)).unwrap();
    let mut loaded = progress::load(&path, std::slice::from_ref(s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    loaded.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    assert!(loaded.favorites.contains(&s.id));
}

#[test]
fn added_prerequisites_teach_the_actual_example_without_changing_either_original_assignment() {
    let notes: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/presentation/lpic1_foundations.json"
    ))
    .unwrap();
    assert_eq!(notes["version"], 1);
    assert_eq!(notes["lessons"].as_array().unwrap().len(), 2);
    let address = &notes["lessons"][0];
    assert_eq!(address["id"], "lpic1-host-address");
    let text = address["concepts"][0]["text"].as_str().unwrap();
    for term in ["32 bits", "/24", "255.255.255.0", "Other prefixes"] {
        assert!(text.contains(term));
    }
    let before = pack::accepted436().unwrap();
    let current = pack::current().unwrap();
    for id in ["lpic1-host-address", "lpic1-host-routing"] {
        assert_eq!(
            serde_json::to_value(before.iter().find(|s| s.id == id).unwrap()).unwrap(),
            serde_json::to_value(current.iter().find(|s| s.id == id).unwrap()).unwrap()
        );
    }
    let engine::Goal::HostNetwork { definition } = &before
        .iter()
        .find(|s| s.id == "lpic1-host-address")
        .unwrap()
        .goal
    else {
        panic!()
    };
    assert_eq!(
        definition.example.profile.links[0].network.text(),
        "10.71.0.0/24"
    );
    let route = &notes["lessons"][1];
    let text = route["concepts"][0]["text"].as_str().unwrap();
    for term in [
        "10.0.0.0/8",
        "172.16.0.0/12",
        "192.168.0.0/16",
        "not automatically public",
        "192.0.2.0/24",
    ] {
        assert!(text.contains(term));
    }
    assert!(
        route["worked"][0]["explanation"]
            .as_str()
            .unwrap()
            .contains("not a separately graded")
    );
}
