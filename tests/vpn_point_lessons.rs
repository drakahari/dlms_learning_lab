//! Two distinct authored drafts; do not count them until catalog/native integration.
use dlms_learning_lab::{
    engine::{self, State},
    pack, vpn_point_source as binding,
    vpn_point_workspace::Action,
};
fn lessons() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/vpn-point-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn act(state: &mut State, s: &pack::Scenario, a: &Action, practice: bool) {
    let raw = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    );
    engine::execute_for_goal(state, &s.goal, &raw).unwrap();
}
fn actions(s: &pack::Scenario) -> Vec<Action> {
    s.solution
        .iter()
        .map(|r| {
            let v: serde_json::Value =
                serde_json::from_str(r.strip_prefix(binding::PREFIX).unwrap()).unwrap();
            serde_json::from_value(v["action"].clone()).unwrap()
        })
        .collect()
}
fn solved(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for r in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, r).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn both_authored_tasks_and_separate_worked_examples_load_with_real_prerequisites() {
    let new = lessons();
    assert_eq!(new.len(), 2);
    let old = pack::current().unwrap();
    let mut closure = new.clone();
    let mut at = 0;
    while at < closure.len() {
        for id in closure[at].prerequisites.clone() {
            if !closure.iter().any(|s| s.id == id) {
                closure.push(
                    old.iter()
                        .find(|s| s.id == id)
                        .expect("Real prerequisite must exist")
                        .clone(),
                );
            }
        }
        at += 1;
    }
    let owned = closure
        .into_iter()
        .map(|s| {
            serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![s],
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    pack::load_catalog(&owned.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    for s in new {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::VpnPoint { simulation } = &state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        for a in &d.worked {
            act(&mut state, &s, a, true);
        }
        let State::VpnPoint { simulation } = &state else {
            panic!()
        };
        assert!(simulation.example.solved(&d.example));
        assert_eq!(simulation.assignment, original);
        assert!(!engine::solved(&state, &s.goal));
        solved(&s);
    }
}
#[test]
fn supported_ca_alias_udp4_comments_and_equivalent_split_routes_are_valid_solutions() {
    for s in lessons() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        for mut a in actions(&s) {
            if let Action::Edit { peer, text } = &mut a {
                let authority = &d.assignment.requirements[*peer].authority;
                *text = text.replace("proto udp\n", "proto udp4\n").replace(
                    &format!("/{authority}.crt"),
                    &format!("/{authority}-alias.crt"),
                );
                let n = d.assignment.requirements[*peer].destinations[0];
                let prefix = n.prefix + 1;
                let mask = std::net::Ipv4Addr::from(u32::MAX << (32 - prefix));
                let full_mask = std::net::Ipv4Addr::from(u32::MAX << (32 - n.prefix));
                let second =
                    std::net::Ipv4Addr::from(u32::from(n.address) + (1u32 << (32 - prefix)));
                *text = text.replace(
                    &format!("route {} {full_mask}", n.address),
                    &format!("route {} {mask}\nroute {second} {mask}", n.address),
                );
                text.push_str("# Equivalent supported configuration\n");
            }
            act(&mut state, &s, &a, false);
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn partial_work_overbroad_routes_and_disabled_peer_verification_cannot_pass() {
    for s in lessons() {
        let mut partial = s.initial.clone();
        act(&mut partial, &s, &Action::Apply, false);
        act(&mut partial, &s, &Action::Check, false);
        assert!(!engine::solved(&partial, &s.goal));
        let good = solved(&s);
        let d = binding::source(&s).unwrap();
        for broad in [false, true] {
            let mut state = good.clone();
            let State::VpnPoint { simulation } = &state else {
                panic!()
            };
            let original = simulation.assignment.world.saved[0].clone();
            let text = if broad {
                original
                    .lines()
                    .filter(|l| !l.starts_with("route "))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\nroute 0.0.0.0 0.0.0.0\n"
            } else {
                original
                    .lines()
                    .filter(|l| !l.starts_with("verify-x509-name "))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n"
            };
            for a in [
                Action::Edit { peer: 0, text },
                Action::Save { peer: 0 },
                Action::Apply,
                Action::Check,
            ] {
                act(&mut state, &s, &a, false);
            }
            assert!(!engine::solved(&state, &s.goal), "{} broad={broad}", s.id);
        }
        assert_ne!(d.assignment.initial.saved, d.example.initial.saved);
    }
}
#[test]
fn full_twenty_bit_lans_need_more_than_the_first_twenty_four_bit_block() {
    let s = lessons().remove(1);
    let d = binding::source(&s).unwrap();
    let mut state = solved(&s);
    let State::VpnPoint { simulation } = &state else {
        panic!()
    };
    let text = simulation.assignment.world.saved[1].replace(
        "route 172.20.16.0 255.255.240.0",
        "route 172.20.16.0 255.255.255.0",
    );
    for a in [
        Action::Edit { peer: 1, text },
        Action::Save { peer: 1 },
        Action::Apply,
        Action::Check,
    ] {
        act(&mut state, &s, &a, false);
    }
    let State::VpnPoint { simulation } = &state else {
        panic!()
    };
    let rows = d.assignment.checks(&simulation.assignment.world);
    assert!(rows.iter().any(|r| r.passed));
    assert!(
        rows.iter()
            .any(|r| r.label.contains("destination policy") && !r.passed)
    );
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn edits_and_forged_reports_clear_success_and_bad_authored_work_is_rejected() {
    for mut s in lessons() {
        let mut state = solved(&s);
        act(
            &mut state,
            &s,
            &Action::Edit {
                peer: 0,
                text: "up /etc/passwd\n".into(),
            },
            false,
        );
        assert!(!engine::solved(&state, &s.goal));
        for a in [Action::Save { peer: 0 }, Action::Apply, Action::Check] {
            act(&mut state, &s, &a, false);
        }
        assert!(!engine::solved(&state, &s.goal));
        let State::VpnPoint { simulation } = &mut state else {
            panic!()
        };
        simulation.assignment.source_task = "{}".into();
        assert!(!engine::solved(&state, &s.goal));
        let engine::Goal::VpnPoint { definition } = &mut s.goal else {
            panic!()
        };
        definition.worked = vec![Action::Check];
        assert!(binding::source(&s).is_err());
    }
}
#[test]
fn registration_preserves_all499_labs200_units_and_teaching_values() {
    let before = pack::accepted499().unwrap();
    let now = pack::current().unwrap();
    assert_eq!((before.len(), now.len()), (499, 505));
    for old in &before {
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(now.iter().find(|s| s.id == old.id).unwrap()).unwrap()
        );
    }
    let old = dlms_learning_lab::course::accepted499(&before).unwrap();
    let units = dlms_learning_lab::course::current(&now).unwrap();
    assert_eq!((old.len(), units.len()), (200, 203));
    for u in old {
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(units.iter().find(|v| v.id == u.id).unwrap()).unwrap()
        );
    }
    let unit = units.iter().find(|u| u.id == "linux-vpn-point").unwrap();
    assert_eq!(
        unit.labs,
        ["lpic2-vpn-peer-profile", "lpic2-vpn-lan-return"]
    );
    for prerequisite in &unit.prerequisites {
        assert!(
            units.iter().position(|u| u.id == *prerequisite).unwrap()
                < units.iter().position(|u| u.id == unit.id).unwrap()
        );
    }
    let presentation: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(presentation.len(), 158);
    for s in lessons() {
        let e = presentation.iter().find(|e| e["id"] == s.id).unwrap();
        assert_eq!(e["practice"].as_array().unwrap().len(), 3);
        assert!(e["concepts"].as_array().unwrap().len() >= 3);
        assert!(e["worked"].as_array().unwrap().len() >= 3);
        assert!(e["requirements"].as_array().unwrap().len() >= 4);
    }
    let tracks = now
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut m, s| {
            *m.entry(s.track.as_str()).or_insert(0) += 1;
            m
        });
    assert_eq!(
        tracks,
        std::collections::BTreeMap::from([
            ("CCNA", 113),
            ("LPIC-1", 154),
            ("LPIC-2", 158),
            ("Bash/Shell", 80)
        ])
    );
}

#[test]
fn changing_both_ports_or_both_tunnel_pairs_is_not_preserving_the_assignment() {
    for s in lessons() {
        let good = solved(&s);
        for change_ports in [true, false] {
            let mut state = good.clone();
            let State::VpnPoint { simulation } = &state else {
                panic!()
            };
            let originals = simulation.assignment.world.saved.clone();
            for (peer, original) in originals.iter().enumerate() {
                let text = if change_ports {
                    original.replace("1194", "1196")
                } else {
                    original
                        .lines()
                        .map(|line| {
                            if line.starts_with("ifconfig ") {
                                if peer == 0 {
                                    "ifconfig 10.99.0.1 10.99.0.2"
                                } else {
                                    "ifconfig 10.99.0.2 10.99.0.1"
                                }
                            } else {
                                line
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        + "\n"
                };
                act(&mut state, &s, &Action::Edit { peer, text }, false);
                act(&mut state, &s, &Action::Save { peer }, false);
            }
            act(&mut state, &s, &Action::Apply, false);
            act(&mut state, &s, &Action::Check, false);
            let State::VpnPoint { simulation } = &state else {
                panic!()
            };
            // Logical connectivity can pass while the supplied configuration is violated.
            let task = &binding::source(&s).unwrap().assignment;
            for case in &task.cases {
                assert_eq!(
                    simulation
                        .assignment
                        .world
                        .probe(case.from, case.source, case.destination)
                        .unwrap()
                        .outcome,
                    case.expected
                );
            }
            assert!(!engine::solved(&state, &s.goal));
            assert!(
                engine::checks(&state, &s.goal)
                    .iter()
                    .any(|(ok, detail)| !*ok && detail.contains("supplied local port"))
            );
        }
    }
}
