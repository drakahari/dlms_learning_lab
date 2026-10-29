use dlms_learning_lab::{physical_link as core, physical_link_assignment as work};
#[path = "support/physical_assignment_fixture.rs"]
mod fixture;
#[test]
fn actual_reference_alternate_partial_and_neighbor_outcomes_are_not_strings_or_counters() {
    let f = fixture::fixture("assignment");
    f.definition.validate().unwrap();
    assert!(!f.definition.solved(&f.definition.initial));
    for c in [&f.reference, &f.alternate] {
        let mut state = f.definition.initial.clone();
        state
            .perform(&core::Action::Configure {
                configuration: c.clone(),
            })
            .unwrap();
        assert!(f.definition.solved(&state));
    }
    for c in [&f.partial, &f.neighbor_broken] {
        let mut state = f.definition.initial.clone();
        state
            .perform(&core::Action::Configure {
                configuration: c.clone(),
            })
            .unwrap();
        assert!(!f.definition.solved(&state));
    }
    f.definition.validate_reference(&f.reference).unwrap();
    f.definition.validate_reference(&f.alternate).unwrap();
}
#[test]
fn pending_wrong_draft_invalidates_success_without_mutating_applied_work() {
    let f = fixture::fixture("assignment");
    let mut state = f.definition.initial.clone();
    state
        .perform(&core::Action::Configure {
            configuration: f.reference.clone(),
        })
        .unwrap();
    let before = state.clone();
    let mut draft = work::Draft::from_state(&state).unwrap();
    assert!(
        work::current_checks(&f.definition, &state, &draft)
            .iter()
            .all(|r| r.0)
    );
    draft.configuration = f.partial.clone();
    let rows = work::current_checks(&f.definition, &state, &draft);
    assert!(!rows[0].0);
    assert_eq!(state, before);
    work::apply(&f.definition, &mut state, &draft).unwrap();
    assert!(!f.definition.solved(&state));
}
#[test]
fn fresh_samples_ignore_saved_counter_values_phase_and_reported_raw_readings() {
    let mut f = fixture::fixture("assignment");
    let mut state = f.definition.initial.clone();
    state
        .perform(&core::Action::Configure {
            configuration: f.partial.clone(),
        })
        .unwrap();
    for name in ["assignment-left", "assignment-right"] {
        state
            .perform(&core::Action::ClearCounters { port: name.into() })
            .unwrap();
    }
    assert!(!f.definition.solved(&state));
    let raw = f.definition.observations.clone();
    f.definition.observations[0].reading.counters.crc_errors = 0;
    assert!(!f.definition.solved(&state));
    state
        .perform(&core::Action::Configure {
            configuration: f.reference.clone(),
        })
        .unwrap();
    assert!(f.definition.solved(&state));
    let before = state.clone();
    let rows = f.definition.checks(&state);
    assert!(rows.iter().all(|r| r.0));
    assert_eq!(state, before);
    assert_eq!(raw[0].reading.counters.crc_errors, 37);
}
#[test]
fn full_topology_span_environment_pin_and_success_invalidation_hold_after_resume() {
    let f = fixture::fixture("assignment");
    let mut state = f.definition.initial.clone();
    state
        .perform(&core::Action::Configure {
            configuration: f.reference.clone(),
        })
        .unwrap();
    let bytes = serde_json::to_vec(&state).unwrap();
    let mut resumed = core::State::from_json(&bytes).unwrap();
    assert!(f.definition.solved(&resumed));
    let mut broken = f.reference.clone();
    broken.ports.get_mut("assignment-left").unwrap().admin_up = false;
    resumed
        .perform(&core::Action::Configure {
            configuration: broken,
        })
        .unwrap();
    assert!(!f.definition.solved(&resumed));
    let mut source = serde_json::to_value(&state).unwrap();
    source["topology"]["links"]["assignment-uplink"]["required_span_m"] = 1.into();
    let altered = core::State::from_json(&serde_json::to_vec(&source).unwrap()).unwrap();
    assert!(!f.definition.solved(&altered));
    assert!(
        f.definition
            .context(&altered)
            .unwrap_err()
            .contains("route spans")
    );
}
#[test]
fn malformed_unreachable_short_cases_duplicate_targets_and_foreign_drafts_refuse() {
    let f = fixture::fixture("assignment");
    let raw = serde_json::to_vec(&f.definition).unwrap();
    assert_eq!(work::Definition::from_json(&raw).unwrap(), f.definition);
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_DEFINITION + 1]).is_err());
    for mode in 0..4 {
        let mut d = f.definition.clone();
        match mode {
            0 => d.requirements[0].probes[0].a_frames = 1,
            1 => d.requirements.push(d.requirements[0].clone()),
            2 => assert!(d.protected_ports.insert("foreign".into())),
            _ => d.version = 0,
        };
        assert!(d.validate().is_err());
    }
    assert!(f.definition.validate_reference(&f.partial).is_err());
    let other = fixture::fixture("other");
    let mut state = f.definition.initial.clone();
    let before = state.clone();
    let draft = work::Draft::from_state(&other.definition.initial).unwrap();
    assert!(work::apply(&f.definition, &mut state, &draft).is_err());
    assert_eq!(state, before);
}
#[test]
fn handover_requires_old_interfaces_retired_new_service_healthy_and_operations_preserved() {
    let f = fixture::fixture("handover");
    let mut d = f.definition;
    let mut topology = d.initial.topology().clone();
    let mut initial = d.initial.configuration().clone();
    let old_left = "handover-old-left";
    let old_right = "handover-old-right";
    for name in [old_left, old_right] {
        let mut spec = topology.ports["handover-left"].clone();
        spec.device = format!("{name}-device");
        spec.label = name.into();
        topology.ports.insert(name.into(), spec);
        initial
            .ports
            .insert(name.into(), initial.ports["handover-ops-left"].clone());
    }
    topology.cables.insert(
        "handover-old-wire".into(),
        core::Cable {
            label: "Technical old path".into(),
            kind: core::CableKind::Cat5e,
            length_m: 30,
            condition: core::Condition::Clean,
        },
    );
    topology.links.insert(
        "handover-old-link".into(),
        core::LinkSpec {
            a: old_left.into(),
            b: old_right.into(),
            required_span_m: 5,
            pattern: core::ProbePattern::Serialized,
        },
    );
    initial
        .cables
        .insert("handover-old-link".into(), "handover-old-wire".into());
    d.initial = core::State::new(topology, initial.clone()).unwrap();
    d.retired_ports = [old_left.into(), old_right.into()].into_iter().collect();
    let mut replacement = initial;
    for (port, config) in f.reference.ports {
        replacement.ports.insert(port, config);
    }
    for (link, cable) in f.reference.cables {
        replacement.cables.insert(link, cable);
    }
    let mut state = d.initial.clone();
    state
        .perform(&core::Action::Configure {
            configuration: replacement.clone(),
        })
        .unwrap();
    assert!(
        !d.solved(&state),
        "New service alone does not retire the old path"
    );
    for port in [old_left, old_right] {
        replacement.ports.get_mut(port).unwrap().admin_up = false;
    }
    d.validate_reference(&replacement).unwrap();
    state
        .perform(&core::Action::Configure {
            configuration: replacement.clone(),
        })
        .unwrap();
    assert!(d.solved(&state));
    replacement
        .ports
        .get_mut("handover-ops-left")
        .unwrap()
        .admin_up = false;
    state
        .perform(&core::Action::Configure {
            configuration: replacement,
        })
        .unwrap();
    assert!(
        !d.solved(&state),
        "Retiring an unrelated operations interface is not success"
    );
}
#[test]
fn undeclared_neighbor_scope_cannot_silently_drop_preservation_requirements() {
    let f = fixture::fixture("assignment");
    let mut d = f.definition.clone();
    d.protected_ports.remove("assignment-ops-left");
    assert!(d.validate().is_err());
    let mut d = f.definition;
    d.protected_links.clear();
    assert!(d.validate().is_err());
}
#[test]
fn oversize_goal_collections_refuse_the_next_body_and_sparse_faults_cannot_evade_samples() {
    let f = fixture::fixture("assignment");
    let mut value = serde_json::to_value(&f.definition).unwrap();
    let item = value["requirements"][0].clone();
    let mut rows = vec![item; 8];
    rows.push(serde_json::Value::Null);
    value["requirements"] = rows.into();
    let e = work::Definition::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err();
    assert!(e.contains("before body"), "{e}");
    let mut topology = f.definition.initial.topology().clone();
    topology.cables.get_mut("assignment-bad").unwrap().condition =
        core::Condition::ReceiveImpairment { every: 99 };
    let current = core::State::new(topology, f.definition.initial.configuration().clone()).unwrap();
    let mut d = f.definition;
    d.initial = current;
    assert!(d.validate().unwrap_err().contains("too short"));
}
