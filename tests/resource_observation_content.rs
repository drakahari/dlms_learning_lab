//! Admitted resource investigations and the exact402 milestone, with real prerequisites.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    resource_observation_workspace as work, workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
const IDS: [&str; 3] = [
    "lpic2-resource-cpu-intervals",
    "lpic2-resource-swap-activity",
    "lpic2-resource-restart-handover",
];
fn drafts() -> Vec<pack::Scenario> {
    let p: pack::Pack = serde_json::from_str(include_str!(
        "fixtures/resource-observation-lessons-v1.json"
    ))
    .unwrap();
    p.scenarios
}
fn baseline() -> &'static Vec<pack::Scenario> {
    static B: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    B.get_or_init(|| pack::accepted399().unwrap())
}
fn authored() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let mut map: BTreeMap<_, _> = baseline()
            .iter()
            .chain(drafts().iter())
            .map(|s| (s.id.clone(), s.clone()))
            .collect();
        let mut pending: Vec<_> = IDS.iter().map(|s| s.to_string()).collect();
        let mut keep = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if keep.insert(id.clone()) {
                pending.extend(map[&id].prerequisites.clone())
            }
        }
        let texts: Vec<_> = keep
            .iter()
            .map(|id| {
                serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: vec![map.remove(id).unwrap()],
                })
                .unwrap()
            })
            .collect();
        let refs: Vec<_> = texts.iter().map(String::as_str).collect();
        let all = pack::load_catalog(&refs).unwrap();
        IDS.iter()
            .map(|id| all.iter().find(|s| s.id == *id).unwrap().clone())
            .collect()
    })
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::ResourceObservations { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::ResourceObservations { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 2);
    workspace::prepare(&mut a);
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn every_taught_method_reaches_its_own_outcome_and_separate_example_earns_no_credit() {
    assert_eq!(authored().len(), 3);
    for s in authored() {
        let d = def(s);
        d.validate().unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        for op in &d.example_reference {
            session(&mut a).practice.apply(&d.example, op).unwrap();
        }
        assert!(d.checks(session(&mut a), true).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        assert_eq!(session(&mut a).assignment, work::Draft::new(&d.initial));
        assert!(progress::current_solved(&solved(s), s));
    }
}
#[test]
fn alternate_supported_commands_and_tied_measurements_pass_without_exact_string_matching() {
    for s in authored() {
        let d = def(s);
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        let mut actions = d.reference.clone();
        for action in &mut actions {
            if let work::Action::Run { probe } = action
                && probe.command.starts_with("vmstat ")
            {
                probe.command = probe.command.replacen("vmstat ", "vmstat -S K ", 1);
            }
            if s.id == IDS[1]
                && let work::Action::Attach { rule, row } = action
                && rule == "quiet-out"
            {
                *row = 3;
            }
        }
        for op in actions {
            session(&mut a).assignment.apply(&d.initial, &op).unwrap();
        }
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn blank_wrong_partial_and_changed_evidence_invalidate_success_until_rechecked() {
    for s in authored() {
        let d = def(s);
        let mut a = solved(s);
        let id = d.initial.rules[0].id.clone();
        for v in ["", "not-a-number", "999999999"] {
            session(&mut a)
                .assignment
                .apply(
                    &d.initial,
                    &work::Action::Value {
                        rule: id.clone(),
                        value: v.into(),
                    },
                )
                .unwrap();
            assert!(!progress::current_solved(&a, s));
            let rows = session(&mut a).assignment.check(&d.initial).unwrap();
            assert!(!rows[0].0);
            assert!(rows.iter().skip(1).all(|r| r.0));
            assert!(!progress::current_solved(&a, s));
        }
        let mut a = solved(s);
        let first = session(&mut a).assignment.claims[0]
            .evidence
            .clone()
            .unwrap();
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Run { probe: first.probe })
            .unwrap();
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Attach { rule: id, row: 0 })
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        let rows = session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(!rows[0].0 && rows[0].1.contains("since-boot"));
    }
}
#[test]
fn incompatible_boot_requests_never_create_readings_or_mutate_reference_execution() {
    let s = &authored()[2];
    let mut a = Attempt::new(s, 1);
    let before = a.state.clone();
    let bad = work::Action::Run {
        probe: work::Probe {
            start: 0,
            command: "vmstat 1 3".into(),
        },
    };
    let result = engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &format!("resource-action {}", serde_json::to_string(&bad).unwrap()),
    );
    assert!(result.unwrap_err().contains("source"));
    assert_eq!(a.state, before);
    for input in ["cat /etc/passwd", "vmstat 1 999999", "vmstat; id"] {
        assert!(
            def(s)
                .initial
                .probe(&work::Probe {
                    start: 0,
                    command: input.into()
                })
                .is_err()
        );
    }
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn separate_drafts_report_evidence_favorites_and_existing_attempts_survive_resume_and_reset() {
    let mut catalog = baseline().clone();
    catalog.extend(authored().clone());
    let other = &catalog[0];
    let peer = Attempt::new(other, 1);
    let peer_bytes = serde_json::to_value(&peer).unwrap();
    for s in authored() {
        let mut a = solved(s);
        let d = def(s);
        session(&mut a).practice.input = "vmstat 1 3".into();
        session(&mut a)
            .assignment
            .apply(
                &d.initial,
                &work::Action::Value {
                    rule: d.initial.rules[0].id.clone(),
                    value: "123".into(),
                },
            )
            .unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 3,
            ..Default::default()
        };
        p.attempts.insert(other.id.clone(), peer.clone());
        p.attempts.insert(s.id.clone(), a);
        p.favorites.extend([other.id.clone(), s.id.clone()]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, &catalog).unwrap();
        let mut loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
            peer_bytes
        );
        assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
        loaded.clock = 4;
        loaded.attempts.insert(s.id.clone(), Attempt::new(s, 4));
        progress::save_with_catalog(&path, &loaded, &catalog).unwrap();
        let reset = progress::load(&path, &catalog).unwrap();
        assert_eq!(reset.attempts[&s.id].state, s.initial);
        assert!(reset.favorites.contains(&s.id));
        assert_eq!(
            serde_json::to_value(&reset.attempts[&other.id]).unwrap(),
            peer_bytes
        );
    }
}
#[test]
fn authored_pack_rejects_broken_reference_and_unanswered_mismatched_saved_state() {
    for s in drafts() {
        let mut broken = s.clone();
        let Goal::ResourceObservations { definition } = &mut broken.goal else {
            panic!()
        };
        definition.reference.pop();
        assert!(work::source(&broken).is_err());
        let mut a = Attempt::new(&s, 1);
        session(&mut a).assignment.claims[0].rule = "other".into();
        assert!(work::context(&a, &s).is_err());
        assert!(!progress::current_solved(&a, &s));
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}

#[test]
fn unsupported_progress_version_cannot_replace_a_valid_investigation_file() {
    let s = &authored()[0];
    let mut catalog = baseline().clone();
    catalog.extend(authored().clone());
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), solved(s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let before = std::fs::read(&path).unwrap();
    p.schema_version = 99;
    assert!(progress::save_with_catalog(&path, &p, &catalog).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let loaded = progress::load(&path, &catalog).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
}

#[test]
fn registered_resource_extension_preserves_all399_labs_and153_units() {
    let old = baseline();
    let current = pack::accepted402().unwrap();
    assert_eq!((old.len(), current.len()), (399, 402));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&current[..399]).unwrap()
    );
    let before = dlms_learning_lab::course::accepted399(old).unwrap();
    let units = dlms_learning_lab::course::accepted402(&current).unwrap();
    assert_eq!((before.len(), units.len()), (153, 154));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-resource-observations")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(kept).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-resource-observations")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-capacity-handover");
    for id in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [93, 133, 96, 80]
    );
}
