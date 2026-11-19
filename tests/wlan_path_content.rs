//! Authored AP-mode/VLAN tasks, real outcomes and retained424data; no host execution.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt},
    wlan_paths::{ApMode, Forwarding},
    wlan_paths_workspace as work,
};
const IDS: [&str; 2] = ["ccna-wlc-campus-forwarding", "ccna-flexconnect-local-vlan"];
fn authored() -> Vec<Scenario> {
    let p: Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v26.json")).unwrap();
    assert_eq!(p.scenarios.len(), 53);
    p.scenarios.into_iter().skip(51).collect()
}
fn def(s: &Scenario) -> &work::Definition {
    let Goal::WlanPaths { definition } = &s.goal else {
        panic!("AP-path objective")
    };
    definition
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::WlanPaths { simulation } = &mut a.state else {
        panic!("AP-path state")
    };
    simulation
}
#[test]
fn both_taught_solutions_and_different_examples_reach_actual_paths_without_example_credit() {
    let labs = authored();
    assert_eq!(labs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    for s in labs {
        let d = work::source(&s).unwrap();
        assert_ne!(
            serde_json::to_value(&d.initial.initial).unwrap(),
            serde_json::to_value(&d.example.initial).unwrap()
        );
        let mut a = Attempt::new(&s, 1);
        session(&mut a).practice.fill(&d.example).unwrap();
        assert!(!session(&mut a).practice.checked());
        session(&mut a).practice.check(&d.example).unwrap();
        assert!(session(&mut a).practice.solved(&d.example));
        assert!(!progress::current_solved(&a, &s));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    }
}
#[test]
fn campus_flexconnect_central_is_a_valid_alternate_without_reference_equality() {
    let s = authored().remove(0);
    let d = def(&s);
    let mut a = Attempt::new(&s, 1);
    let mut alternative = d.initial.reference.clone();
    alternative.mode = ApMode::FlexConnect;
    alternative.wlans[0].forwarding = Forwarding::Central;
    assert_ne!(alternative, d.initial.reference);
    session(&mut a).assignment.fields = alternative;
    session(&mut a).assignment.check(&d.initial).unwrap();
    assert!(progress::current_solved(&a, &s));
    let mut mode_only = Attempt::new(&s, 1);
    session(&mut mode_only).assignment.fields.mode = ApMode::Local;
    session(&mut mode_only)
        .assignment
        .check(&d.initial)
        .unwrap();
    assert!(!progress::current_solved(&mode_only, &s));
    let mut port_only = Attempt::new(&s, 1);
    session(&mut port_only).assignment.fields.ports = d.initial.reference.ports.clone();
    session(&mut port_only)
        .assignment
        .check(&d.initial)
        .unwrap();
    assert!(!progress::current_solved(&port_only, &s));
}
#[test]
fn branch_wrong_native_mapping_allowed_list_and_forwarding_fail_without_hiding_passing_visitors() {
    let s = authored().remove(1);
    let d = def(&s);
    for which in 0..4 {
        let mut a = Attempt::new(&s, 1);
        session(&mut a).assignment.fields = d.initial.reference.clone();
        match which {
            0 => session(&mut a).assignment.fields.native = "41".into(),
            1 => session(&mut a).assignment.fields.wlans[0].local_vlan = "51".into(),
            2 => {
                session(&mut a)
                    .assignment
                    .fields
                    .ports
                    .switches
                    .get_mut("access")
                    .unwrap()
                    .ports
                    .get_mut("ap-port")
                    .unwrap()
                    .allowed = "40,60".into()
            }
            _ => session(&mut a).assignment.fields.mode = ApMode::Local,
        };
        session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(!progress::current_solved(&a, &s), "case{which}");
        let rows = d.initial.checks(&session(&mut a).assignment.fields);
        if which == 0 {
            assert!(rows.iter().any(|r| r.feedback.contains("differs")));
            assert!(
                rows.iter()
                    .filter(|r| r.id != "management")
                    .all(|r| !r.evaluated && !r.correct)
            );
        } else {
            assert!(
                rows.iter()
                    .find(|r| r.id == "visitor-service")
                    .unwrap()
                    .correct
            );
            assert!(
                !rows
                    .iter()
                    .find(|r| r.id == "staff-service")
                    .unwrap()
                    .correct
            );
        }
    }
}
#[test]
fn malformed_fields_edits_and_selected_reset_preserve_examples_without_stale_credit() {
    for s in authored() {
        let d = def(&s);
        let mut a = Attempt::new(&s, 1);
        session(&mut a).practice.fill(&d.example).unwrap();
        session(&mut a).practice.check(&d.example).unwrap();
        let practice = serde_json::to_vec(&session(&mut a).practice).unwrap();
        session(&mut a).assignment.fill(&d.initial).unwrap();
        session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, &s));
        let port = if s.id == IDS[0] {
            "controller-port"
        } else {
            "ap-port"
        };
        session(&mut a)
            .assignment
            .fields
            .ports
            .switches
            .get_mut("access")
            .unwrap()
            .ports
            .get_mut(port)
            .unwrap()
            .allowed = "bad".into();
        assert!(!progress::current_solved(&a, &s));
        session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(!progress::current_solved(&a, &s));
        let restored: Attempt = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        work::context(&restored, &s).unwrap();
        assert!(!progress::current_solved(&restored, &s));
        session(&mut a).assignment.reset(&d.initial);
        assert_eq!(
            serde_json::to_vec(&session(&mut a).practice).unwrap(),
            practice
        );
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn all_424_previous_scenarios_163_units_and_51_tail_values_are_retained_in_the_prospective_graph() {
    let old = pack::accepted424().unwrap();
    assert_eq!(old.len(), 424);
    let old_units = dlms_learning_lab::course::accepted424(&old).unwrap();
    assert_eq!(old_units.len(), 163);
    let before: Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v25.json")).unwrap();
    let after: Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v26.json")).unwrap();
    assert_eq!(before.scenarios.len(), 51);
    assert_eq!(
        serde_json::to_value(&after.scenarios[..51]).unwrap(),
        serde_json::to_value(&before.scenarios).unwrap()
    );
    let mut combined = old.clone();
    combined.extend(authored());
    assert_eq!(combined.len(), 426);
    let mut units = serde_json::to_value(&old_units)
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    let extra: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/wlan-path-units-v1.json")).unwrap();
    assert_eq!(extra["version"], 1);
    units.push(extra["units"][0].clone());
    let index = serde_json::json!({"version":1,"units":units});
    let units = dlms_learning_lab::course::load(&serde_json::to_string(&index).unwrap(), &combined)
        .unwrap();
    assert_eq!(units.len(), 164);
    for s in &old {
        let found = combined.iter().find(|now| now.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(found).unwrap()
        );
    }
    for u in &old_units {
        let found = units.iter().find(|now| now.id == u.id).unwrap();
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(found).unwrap()
        );
    }
    let unit = units
        .iter()
        .find(|u| u.id == "ccna-wireless-forwarding-paths")
        .unwrap();
    assert_eq!(
        unit.prerequisites,
        ["vlan-data-foundations", "ccna-wlan-personal-security"]
    );
    assert_eq!(unit.labs, IDS);
}
#[test]
fn bad_source_scopes_prechecked_initials_and_unreachable_references_refuse() {
    for s in authored() {
        let mut bad = s.clone();
        if let Goal::WlanPaths { definition } = &mut bad.goal {
            definition.initial.id = "other".into();
        }
        assert!(work::source(&bad).is_err());
        let mut bad = s.clone();
        if let State::WlanPaths { simulation } = &mut bad.initial {
            simulation.assignment.checked_fields = Some(simulation.assignment.fields.clone());
        }
        assert!(work::source(&bad).is_err());
        let mut bad = s.clone();
        if let Goal::WlanPaths { definition } = &mut bad.goal {
            definition.initial.reference = definition.initial.initial_fields();
        }
        assert!(work::source(&bad).is_err());
        assert!(
            s.solution
                .iter()
                .all(|c| c.starts_with("@ap-edit ") || c == "@ap-check")
        );
        assert!(s.simplifications.contains("No host I/O"));
        assert!(s.simplifications.contains("LAG"));
    }
}

#[test]
fn normal_catalog_registers_both_real_tasks_after_their_existing_prerequisites() {
    let old = pack::accepted424().unwrap();
    let current = pack::current().unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!((current.len(), units.len()), (426, 164));
    for s in &old {
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(current.iter().find(|n| n.id == s.id).unwrap()).unwrap()
        );
    }
    let previous = units
        .iter()
        .position(|u| u.id == "ccna-wlan-personal-security")
        .unwrap();
    assert_eq!(units[previous + 1].id, "ccna-wireless-forwarding-paths");
    assert_eq!(units[previous + 1].labs, IDS);
    for id in IDS {
        assert!(current.iter().any(|s| s.id == id));
    }
}
