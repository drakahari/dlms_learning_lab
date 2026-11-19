//! Catalog integration and teaching; variants are not extra authored scenarios.
use dlms_learning_lab::{
    command_location_workspace as work, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic1-whereis-support-handover";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted290().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 1);
    workspace::prepare(&mut a);
    work::prepare(&mut a, lab(), true).unwrap();
    a
}
#[test]
fn all289_predecessor_scenarios92_units_are_exact_and_prerequisites_precede_new_unit() {
    let old = pack::accepted289().unwrap();
    assert_eq!((old.len(), catalog().len()), (289, 290));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&catalog()[..289]).unwrap()
    );
    let before = course::accepted289(&old).unwrap();
    let after = course::accepted290(catalog()).unwrap();
    assert_eq!((before.len(), after.len()), (92, 93));
    let retained: Vec<_> = after
        .iter()
        .filter(|u| u.id != "linux-application-support-discovery")
        .collect();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = after
        .iter()
        .position(|u| u.id == "linux-application-support-discovery")
        .unwrap();
    for id in ["linux-command-location-diagnosis", "linux-file-locations"] {
        assert!(after.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| catalog().iter().filter(|s| s.track == t).count()),
        [72, 99, 45, 74]
    );
    assert_eq!(
        lab().prerequisites,
        [
            "lpic1-command-permission-search",
            "lpic1-fhs-locate-active-files"
        ]
    );
}
#[test]
fn actual_registered_taught_solution_and_guided_wrong_release_exploration_work() {
    let s = lab();
    let l = s.terminal_teaching.as_ref().unwrap();
    l.validate().unwrap();
    let mut a = fresh();
    let original = a.state.clone();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    for c in &l.example {
        learn.command = c.clone();
        learn.execute_for(l).unwrap();
    }
    assert!(l.outcomes.iter().all(|r| r.check(&learn.tree).0));
    learn.command = "cp /gallery/archive/democtl.c /gallery/source/democtl.c".into();
    learn.execute_for(l).unwrap();
    assert!(l.outcomes.iter().any(|r| !r.check(&learn.tree).0));
    learn.command = "cp /gallery/stage/democtl.c /gallery/source/democtl.c".into();
    learn.execute_for(l).unwrap();
    assert!(l.outcomes.iter().all(|r| r.check(&learn.tree).0));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, s));
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "cp /apps/archive/reportctl.c /apps/source/reportctl.c",
    )
    .unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        workspace::issues(&a.state, &s.goal)
            .iter()
            .any(|m| m.contains("release contents"))
    );
}
#[test]
fn new_source_admission_rejects_erased_duplicate_crossed_and_reused_references() {
    let s = lab();
    for kind in 0..6 {
        let mut bad = s.clone();
        let Goal::Filesystem { requirements } = &mut bad.goal else {
            panic!()
        };
        match kind {
            0 => {
                requirements.remove(0);
            }
            1 => requirements.push(requirements[0].clone()),
            2 => {
                let State::Filesystem { tree } = &mut bad.initial else {
                    panic!()
                };
                tree.command_location = None;
            }
            3 => {
                bad.terminal_teaching.as_mut().unwrap().fixture = match &s.initial {
                    State::Filesystem { tree } => tree.clone(),
                    _ => panic!(),
                }
            }
            4 => bad.terminal_teaching = None,
            _ => {
                let l = bad.terminal_teaching.as_mut().unwrap();
                l.outcomes.clear();
            }
        }
        assert!(work::source(&bad).is_err(), "kind {kind}");
    }
}
#[test]
fn essential_teaching_supplies_flags_result_meaning_release_check_and_a_separate_worked_task() {
    let s = lab();
    let l = s.terminal_teaching.as_ref().unwrap();
    assert_eq!(l.concepts.len(), 4);
    assert_eq!(l.example.len(), 8);
    let text = l
        .concepts
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for term in [
        "which",
        "-m",
        "-M",
        "-f",
        "-b/-B",
        "-s/-S",
        "bare democtl:",
        "cat",
        "cp",
        "rm",
        "contents",
    ] {
        assert!(text.contains(term), "missing teaching: {term}");
    }
    assert!(s.retry_guidance.contains("release"));
    assert!(!s.retry_guidance.contains("add only the owner execute"));
    assert!(
        l.example
            .iter()
            .all(|c| !c.contains("/apps/") && !c.contains("reportctl"))
    );
    assert!(!s.objective.contains("cp /apps/"));
    assert!(s.objective.contains("Preserve"));
}
