//! Real catalog integration and preserved historical source/progress contracts.
use dlms_learning_lab::{
    course, directory_policy_source,
    engine::{self, State},
    pack,
    progress::{self, Attempt},
};
const IDS: [&str; 2] = [
    "lpic1-sticky-collaboration",
    "lpic1-sticky-collision-handover",
];
#[test]
fn catalog_retains_all290_sources93_units_and_orders_real_prerequisites() {
    let old = pack::accepted290().unwrap();
    let all = pack::accepted292().unwrap();
    assert_eq!((old.len(), all.len()), (290, 292));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&all[..290]).unwrap()
    );
    let before = course::accepted290(&old).unwrap();
    let after = course::accepted292(&all).unwrap();
    assert_eq!((before.len(), after.len()), (93, 94));
    let retained: Vec<_> = after
        .iter()
        .filter(|u| u.id != "linux-sticky-directory-policy")
        .collect();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = after
        .iter()
        .position(|u| u.id == "linux-sticky-directory-policy")
        .unwrap();
    assert!(
        after
            .iter()
            .position(|u| u.id == "linux-access-policy")
            .unwrap()
            < at
    );
    assert_eq!(after[at].labs, IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [72, 101, 45, 74]
    );
    let fixtures: pack::Pack =
        serde_json::from_str(include_str!("fixtures/directory-policy-lessons-v1.json")).unwrap();
    for (fixture, id) in fixtures.scenarios.iter().zip(IDS) {
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(fixture).unwrap()
        );
        directory_policy_source::source(s).unwrap();
        for parent in &s.prerequisites {
            assert!(all.iter().any(|s| &s.id == parent));
        }
    }
}
#[test]
fn registered_examples_and_assignments_are_separate_reachable_and_damage_invalidates() {
    let all = pack::accepted292().unwrap();
    for id in IDS {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let d = directory_policy_source::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        let State::DirectoryPolicy { simulation } = &mut a.state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        for command in &d.example_commands {
            simulation.run(d, command, true).unwrap();
        }
        assert!(d.checks(simulation, true).iter().all(|r| r.0));
        assert_eq!(simulation.assignment, original);
        assert!(!progress::current_solved(&a, s));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let damaging = if id == IDS[0] {
            "chmod 0700 /work/shared"
        } else {
            "cp /work/handover/bea-release.txt /work/handover/release.txt"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, damaging).unwrap();
        assert!(!progress::current_solved(&a, s));
        let repair = if id == IDS[0] {
            "chmod 1770 /work/shared"
        } else {
            "cp /work/retained-release.txt /work/handover/release.txt"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, repair).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}

#[test]
fn mistaken_repairs_explain_the_specific_collaboration_and_data_misconceptions() {
    let all = pack::accepted292().unwrap();
    for id in IDS {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let d = directory_policy_source::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        if id == IDS[1] {
            for c in &s.solution {
                engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
            }
        }
        let bad = if id == IDS[0] {
            "chmod 1700 /work/shared"
        } else {
            "cp /work/handover/bea-release.txt /work/handover/release.txt"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, bad).unwrap();
        let State::DirectoryPolicy { simulation } = &a.state else {
            panic!()
        };
        let failures: Vec<_> = d
            .checks(simulation, false)
            .into_iter()
            .filter(|r| !r.0)
            .map(|r| r.1)
            .collect();
        assert!(!failures.is_empty());
        assert!(if id == IDS[0] {
            failures.iter().any(|t| {
                t.contains("ordinary permissions changed") && t.contains("locks teammates out")
            })
        } else {
            failures
                .iter()
                .any(|t| t.contains("different or missing contents") && t.contains("writable file"))
        });
        assert!(!progress::current_solved(&a, s));
    }
}
