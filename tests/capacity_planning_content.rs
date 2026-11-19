//! Prospective authored planning lessons with their actual prerequisite closure.
use dlms_learning_lab::{
    capacity_planning_workspace as work,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
const IDS: [&str; 2] = [
    "lpic2-capacity-trend-plan",
    "lpic2-capacity-import-handover",
];
fn drafts() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("fixtures/capacity-planning-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn before() -> &'static Vec<pack::Scenario> {
    static V: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    V.get_or_init(|| pack::accepted402().unwrap())
}
fn authored() -> &'static Vec<pack::Scenario> {
    static V: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    V.get_or_init(|| {
        let fresh = drafts();
        let mut all: BTreeMap<_, _> = before()
            .iter()
            .chain(fresh.iter())
            .map(|s| (s.id.clone(), s.clone()))
            .collect();
        let mut needed = BTreeSet::new();
        let mut queue: Vec<_> = IDS.iter().map(|s| s.to_string()).collect();
        while let Some(id) = queue.pop() {
            if needed.insert(id.clone()) {
                queue.extend(all[&id].prerequisites.clone());
            }
        }
        let texts: Vec<_> = needed
            .iter()
            .map(|id| {
                serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: vec![all.remove(id).unwrap()],
                })
                .unwrap()
            })
            .collect();
        let refs: Vec<_> = texts.iter().map(String::as_str).collect();
        let loaded = pack::load_catalog(&refs).unwrap();
        IDS.iter()
            .map(|id| loaded.iter().find(|s| s.id == *id).unwrap().clone())
            .collect()
    })
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::CapacityPlanning { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::CapacityPlanning { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 2);
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn taught_methods_reach_every_case_with_real_prerequisites_and_isolated_worked_data() {
    for s in authored() {
        let d = definition(s);
        d.validate().unwrap();
        let mut a = Attempt::new(s, 1);
        assert!(!progress::current_solved(&a, s));
        let initial = a.state.clone();
        for op in &d.example_reference {
            session(&mut a).practice.apply(&d.example, op).unwrap();
        }
        assert!(d.checks(session(&mut a), true).iter().all(|r| r.0));
        assert_eq!(session(&mut a).assignment, work::Draft::new(&d.initial));
        assert!(!progress::current_solved(&a, s));
        assert_ne!(a.state, initial);
        let mut a = solved(s);
        assert_eq!(
            a.step, 0,
            "Walkthrough position must not gate a complete assignment"
        );
        let report = d.initial.assess(&session(&mut a).assignment);
        assert_eq!(report.cases.len(), 3);
        assert!(report.cases.iter().all(|c| c.first_failure.is_none()));
    }
}
#[test]
fn alternative_quotes_recent_windows_dates_and_larger_reserves_pass_by_outcome() {
    for (n, s) in authored().iter().enumerate() {
        let d = definition(s);
        let mut a = solved(s);
        let (first, last, quote, day, reserve) = if n == 0 {
            (2, 3, "fast", 15, 11 * 1048576)
        } else {
            (3, 4, "urgent-expansion", 31, 13 * 1048576)
        };
        for op in [
            work::Action::Window { first, last },
            work::Action::Offer { id: quote.into() },
            work::Action::OrderDay {
                value: day.to_string(),
            },
            work::Action::Reserve {
                value: reserve.to_string(),
            },
            work::Action::Test,
        ] {
            session(&mut a).assignment.apply(&d.initial, &op).unwrap();
        }
        assert!(
            progress::current_solved(&a, s),
            "{}: {:?}",
            s.id,
            d.initial.assess(&session(&mut a).assignment).checks
        );
    }
}
#[test]
fn copied_worked_values_do_not_complete_the_different_independent_assignment() {
    for s in authored() {
        let d = definition(s);
        let mut a = Attempt::new(s, 1);
        for op in &d.example_reference {
            session(&mut a).assignment.apply(&d.initial, op).unwrap();
        }
        assert!(!progress::current_solved(&a, s));
        let checks = engine::checks(&a.state, &s.goal);
        assert!(checks.iter().any(|r| !r.0 && r.1.starts_with("Reserve:")));
        assert!(checks.iter().any(|r| !r.0 && r.1.starts_with("Order day:")));
    }
}
#[test]
fn oversized_quotes_and_old_windows_cannot_hide_behind_passing_individual_cases() {
    for (n, s) in authored().iter().enumerate() {
        let d = definition(s);
        let mut a = solved(s);
        let quote = if n == 0 { "costly" } else { "over-budget" };
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Offer { id: quote.into() })
            .unwrap();
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Test)
            .unwrap();
        let results = d.initial.assess(&session(&mut a).assignment);
        assert!(results.cases.iter().all(|c| c.first_failure.is_none()));
        assert!(!results.passed());
        assert!(!progress::current_solved(&a, s));
        a = solved(s);
        session(&mut a)
            .assignment
            .apply(
                &d.initial,
                &work::Action::Window {
                    first: 0,
                    last: d.initial.series.samples.len() - 1,
                },
            )
            .unwrap();
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Test)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn partial_repairs_wrong_units_and_edits_invalidate_checked_success() {
    for s in authored() {
        let d = definition(s);
        let mut a = solved(s);
        session(&mut a)
            .assignment
            .apply(
                &d.initial,
                &work::Action::Reserve {
                    value: "12 GiB".into(),
                },
            )
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(!session(&mut a).assignment.checked);
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Test)
            .unwrap();
        let checks = engine::checks(&a.state, &s.goal);
        assert!(checks.iter().any(|r| !r.0 && r.1.contains("unit suffix")));
        let correct = d.initial.policy.minimum_reserve_kib;
        session(&mut a)
            .assignment
            .apply(
                &d.initial,
                &work::Action::Reserve {
                    value: correct.to_string(),
                },
            )
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        session(&mut a)
            .assignment
            .apply(&d.initial, &work::Action::Test)
            .unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn both_drafts_favorites_case_selection_and_peer_attempt_survive_resume_and_selected_reset() {
    let mut catalog = before().clone();
    catalog.extend(authored().clone());
    let peer = &catalog[0];
    let peer_a = Attempt::new(peer, 1);
    let kept = serde_json::to_value(&peer_a).unwrap();
    for s in authored() {
        let mut a = solved(s);
        session(&mut a).practice.inputs.reserve_kib = "draft to revisit".into();
        session(&mut a).assignment.selected_case = 2;
        let mut p = Progress {
            schema_version: 1,
            clock: 3,
            ..Default::default()
        };
        p.attempts.insert(peer.id.clone(), peer_a.clone());
        p.attempts.insert(s.id.clone(), a);
        p.favorites.extend([peer.id.clone(), s.id.clone()]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, &catalog).unwrap();
        let mut loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
        loaded.clock = 4;
        loaded.attempts.insert(s.id.clone(), Attempt::new(s, 4));
        progress::save_with_catalog(&path, &loaded, &catalog).unwrap();
        let reset = progress::load(&path, &catalog).unwrap();
        assert_eq!(reset.attempts[&s.id].state, s.initial);
        assert_eq!(
            serde_json::to_value(&reset.attempts[&peer.id]).unwrap(),
            kept
        );
        assert!(reset.favorites.contains(&s.id));
    }
}
#[test]
fn bad_source_reachability_unknown_actions_and_forged_saved_fields_fail_closed() {
    for s in authored() {
        let mut a = Attempt::new(s, 1);
        for input in [
            "sh -c true",
            "plan-action {}",
            "plan-action {\"kind\":\"delete-data\"}",
        ] {
            let old = serde_json::to_value(&a).unwrap();
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
            assert_eq!(serde_json::to_value(&a).unwrap(), old);
        }
        let mut bad = s.clone();
        let Goal::CapacityPlanning { definition } = &mut bad.goal else {
            panic!()
        };
        definition.reference.pop();
        assert!(work::source(&bad).is_err());
        session(&mut a).assignment.task_id = "other-task".into();
        assert!(work::context(&a, s).is_err());
        assert!(!progress::current_solved(&a, s));
    }
}

#[test]
fn registered_planning_extension_preserves_all402_prior_labs_and154_units() {
    let old = before();
    let current = pack::accepted404().unwrap();
    assert_eq!((old.len(), current.len()), (402, 404));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&current[..402]).unwrap()
    );
    let previous = dlms_learning_lab::course::accepted402(old).unwrap();
    let units = dlms_learning_lab::course::accepted404(&current).unwrap();
    assert_eq!((previous.len(), units.len()), (154, 155));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-capacity-planning")
        .collect();
    assert_eq!(
        serde_json::to_value(previous).unwrap(),
        serde_json::to_value(kept).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-capacity-planning")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-resource-observations");
    for id in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [93, 133, 98, 80]
    );
}
