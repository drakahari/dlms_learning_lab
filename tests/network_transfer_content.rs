use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    network_transfer as core, network_transfer_source as binding,
    network_transfer_workspace as work, pack,
};
const IDS: [&str; 3] = [
    "ccna-tftp-recovery-copy",
    "ccna-ftp-archive-channel",
    "ccna-transfer-file-handover",
];
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!(
        "../scenarios/network-transfers-lessons-v1.json"
    ))
    .unwrap()
}
fn session(s: &mut State) -> &mut work::Session {
    let State::NetworkTransfer { simulation } = s else {
        panic!()
    };
    simulation
}
fn recipes(s: &pack::Scenario) -> Vec<work::Draft> {
    s.solution
        .iter()
        .filter_map(|line| {
            match serde_json::from_str::<binding::Event>(
                line.strip_prefix(binding::PREFIX).unwrap(),
            )
            .unwrap()
            {
                binding::Event::Edit { draft } => Some(draft),
                _ => None,
            }
        })
        .collect()
}
fn apply(a: &mut work::Attempt, t: &work::Task, recipe: work::Draft) {
    a.edit(t, recipe).unwrap();
    let r = a.transfer(t).unwrap();
    assert!(r.success, "{}", r.detail);
}
#[test]
fn registered_catalog_preserves_every449_scenario_and178_unit_and_adds_only_the_authored_unit() {
    let old = pack::accepted449().unwrap();
    let current = pack::accepted452().unwrap();
    assert_eq!(old.len(), 449);
    assert_eq!(current.len(), 452);
    for prior in &old {
        assert_eq!(
            serde_json::to_value(prior).unwrap(),
            serde_json::to_value(current.iter().find(|s| s.id == prior.id).unwrap()).unwrap(),
            "{}",
            prior.id
        );
    }
    let new: Vec<_> = current
        .iter()
        .filter(|s| !old.iter().any(|o| o.id == s.id))
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(new, IDS);
    for (track, count) in [
        ("CCNA", 107),
        ("LPIC-1", 151),
        ("LPIC-2", 114),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
    let previous = course::accepted449(&old).unwrap();
    let units = course::accepted452(&current).unwrap();
    assert_eq!((previous.len(), units.len()), (178, 179));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| previous.iter().any(|p| p.id == u.id))
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(&previous).unwrap()
    );
    let n = units
        .iter()
        .position(|u| u.id == "ccna-file-transfers")
        .unwrap();
    assert_eq!(units[n - 1].id, "ccna-transport-conversations");
    assert_eq!(units[n].labs, IDS);
    assert_eq!(units[n].prerequisites, vec!["ccna-transport-conversations"]);
}
#[test]
fn every_curated_worked_practice_and_assignment_reaches_all_actual_file_and_policy_goals() {
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let a = session(&mut state);
        let unchanged = a.assignment.clone();
        for r in &d.worked_example {
            apply(&mut a.practice, &d.example, r.clone());
        }
        a.practice.check(&d.example).unwrap();
        assert!(a.checks(d, true).iter().all(|r| r.0));
        assert_eq!(a.assignment, unchanged);
        assert!(!engine::solved(&state, &s.goal));
        for event in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, event).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn a_successful_wrong_file_copy_is_not_completion_and_ftp_control_only_is_not_a_copy() {
    let list = labs();
    let s = &list[0];
    let d = binding::source(s).unwrap();
    let mut a = work::Attempt::new(&d.assignment).unwrap();
    assert!(a.transfer(&d.assignment).unwrap().success);
    a.check(&d.assignment).unwrap();
    assert!(!a.checks(&d.assignment).iter().all(|r| r.0));
    assert!(
        d.assignment
            .case_checks(0, &a)
            .iter()
            .any(|r| !r.0 && r.1.contains("different file"))
    );
    let s = &list[1];
    let d = binding::source(s).unwrap();
    let mut a = work::Attempt::new(&d.assignment).unwrap();
    let r = a.transfer(&d.assignment).unwrap();
    assert!(!r.success);
    assert_eq!(r.stage, core::Stage::DataConnection);
    assert!(
        r.events
            .iter()
            .any(|e| e.stage == core::Stage::Login && e.delivered)
    );
    a.check(&d.assignment).unwrap();
    assert!(!a.checks(&d.assignment).iter().all(|r| r.0));
}
#[test]
fn ftp_accepts_either_supplied_writer_and_preserves_denial_instead_of_exact_rule_strings() {
    let list = labs();
    let s = &list[1];
    let d = binding::source(s).unwrap();
    let mut r = recipes(s).remove(0);
    r.request.login = Some("backup-operator".into());
    let doc = r.documents.routers.get_mut("R1").unwrap();
    *doc = doc
        .replace("FORWARD", "ARCHIVE-OUT")
        .replace("RETURN", "ARCHIVE-IN");
    let mut a = work::Attempt::new(&d.assignment).unwrap();
    apply(&mut a, &d.assignment, r);
    a.check(&d.assignment).unwrap();
    assert!(
        a.checks(&d.assignment).iter().all(|r| r.0),
        "{:?}",
        a.checks(&d.assignment)
    );
}
#[test]
fn capstone_partial_wrong_original_and_reversed_order_have_honest_outcomes() {
    let list = labs();
    let s = &list[2];
    let d = binding::source(s).unwrap();
    let r = recipes(s);
    assert_eq!(r.len(), 2);
    let mut a = work::Attempt::new(&d.assignment).unwrap();
    apply(&mut a, &d.assignment, r[0].clone());
    a.check(&d.assignment).unwrap();
    assert!(!a.checks(&d.assignment).iter().all(|r| r.0));
    assert!(
        !d.assignment.replay(0, &a.history).unwrap().files["Device"].contains_key("staged.cfg")
    );
    apply(&mut a, &d.assignment, r[1].clone());
    a.check(&d.assignment).unwrap();
    assert!(a.checks(&d.assignment).iter().all(|r| r.0));
    let mut bad = r[0].clone();
    bad.request.local_path = "staged.cfg".into();
    apply(&mut a, &d.assignment, bad);
    a.check(&d.assignment).unwrap();
    assert!(!a.checks(&d.assignment).iter().all(|r| r.0));
    assert!(
        d.assignment
            .case_checks(0, &a)
            .iter()
            .any(|r| !r.0 && r.1.contains("Archive/archive/original.cfg"))
    );
    let mut alternative = work::Attempt::new(&d.assignment).unwrap();
    for recipe in r.into_iter().rev() {
        apply(&mut alternative, &d.assignment, recipe);
    }
    alternative.check(&d.assignment).unwrap();
    assert!(alternative.checks(&d.assignment).iter().all(|r| r.0));
}
#[test]
fn introductions_and_guidance_are_curated_for_all_three_controls_and_not_assignment_solutions() {
    let author: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/authoring/network-transfers-v1.json"
    ))
    .unwrap();
    let presentation: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/ccna.json")).unwrap();
    for lesson in author["lessons"].as_array().unwrap() {
        let id = lesson["scenario"]["id"].as_str().unwrap();
        let note = &lesson["presentation"];
        assert_eq!(
            presentation
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["id"] == id)
                .unwrap(),
            note
        );
        assert!(note["concepts"].as_array().unwrap().len() >= 3);
        assert_eq!(note["practice"].as_array().unwrap().len(), 3);
        assert_eq!(note["worked"].as_array().unwrap().len(), 4);
        let steps = note["practice"].to_string();
        for action in ["Transfer file", "Fill worked transfer", "Check example"] {
            assert!(steps.contains(action), "{id}: {action}");
        }
        assert!(
            lesson["scenario"]["simplifications"]
                .as_str()
                .unwrap()
                .contains("2026-11-13")
        );
        assert_eq!(
            lesson["scenario"]["objective"].as_str().unwrap(),
            note["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect::<Vec<_>>()
                .join("\n\n")
        );
    }
    let text = author["lessons"][0]["presentation"]["concepts"].to_string();
    assert!(text.contains("no directory-listing command"));
    for s in labs() {
        let Goal::NetworkTransfer { definition } = &s.goal else {
            panic!()
        };
        assert!(definition.assignment.cases.len() > 1);
        if !definition.assignment.editable_routers.is_empty() {
            assert!(
                definition
                    .assignment
                    .new_draft()
                    .unwrap()
                    .documents
                    .routers
                    .values()
                    .all(|d| !d.contains("range 52010 52012")),
                "The independent router fault stays in the starter"
            );
        }
    }
}
