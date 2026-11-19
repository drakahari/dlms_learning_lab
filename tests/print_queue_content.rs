//! Actual authored proposals; normal registration/native review remain separate.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    print_queue_workspace::{self as work, Command, Session},
    progress::{self, Attempt},
};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-print-paused-queue",
    "lpic1-print-held-request",
    "lpic1-print-destination-handover",
];
fn proposals() -> Vec<pack::Scenario> {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v21.json")).unwrap();
        let mut all = pack::accepted412().unwrap();
        assert_eq!(all.len(), 412);
        all.extend(
            raw.scenarios
                .into_iter()
                .filter(|s| IDS.contains(&s.id.as_str())),
        );
        let strings = all
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let refs = strings.iter().map(String::as_str).collect::<Vec<_>>();
        pack::load_catalog(&refs).unwrap()
    })
    .iter()
    .filter(|s| IDS.contains(&s.id.as_str()))
    .cloned()
    .collect()
}
fn session(a: &mut Attempt) -> &mut Session {
    let State::PrintQueues { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn all_three_teaching_methods_and_separate_examples_reach_actual_outcomes() {
    let labs = proposals();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let Goal::PrintQueues { definition } = &s.goal else {
            panic!()
        };
        definition.validate().unwrap();
        let mut a = Attempt::new(&s, 1);
        for command in &definition.example.reference {
            let draft = &mut session(&mut a).practice;
            draft.input = command.clone();
            draft.run(&definition.example).unwrap().unwrap();
        }
        assert!(
            session(&mut a)
                .practice
                .check(&definition.example)
                .unwrap()
                .iter()
                .all(|r| r.correct)
        );
        assert!(!progress::current_solved(&a, &s));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        assert!(progress::current_solved(&a, &s), "{}", s.id);
        assert_ne!(definition.initial.facts, definition.example.facts);
        assert_eq!(definition.initial.cases.len(), 4);
    }
}
#[test]
fn legitimate_equivalent_interfaces_and_nonritual_investigation_orders_pass() {
    for s in proposals() {
        let Goal::PrintQueues { definition } = &s.goal else {
            panic!()
        };
        let mut a = Attempt::new(&s, 1);
        for command in &definition.initial.reference {
            if command.0.starts_with("lpstat ") || command.0.starts_with("lpq ") {
                continue;
            }
            let words = command.0.split_whitespace().collect::<Vec<_>>();
            let raw = if words.first() == Some(&"lprm") {
                format!("cancel {}", words[1])
            } else if words.first() == Some(&"lp") && words.get(1) == Some(&"-i") {
                format!("lp -H resume -i {}", words[2])
            } else if words.first() == Some(&"lp") && words.get(1) == Some(&"-d") {
                format!("lpr -P {} -# {} {}", words[2], words[4], words[5])
            } else if words.first() == Some(&"lpr") {
                format!("lp -n 2 {}", words.last().unwrap())
            } else {
                command.0.clone()
            };
            engine::execute_for_goal(&mut a.state, &s.goal, &raw).unwrap();
        }
        engine::execute_for_goal(&mut a.state, &s.goal, "@print-check").unwrap();
        assert!(progress::current_solved(&a, &s), "{}", s.id);
    }
}
#[test]
fn partial_and_rebroken_repairs_fail_with_current_specific_results() {
    for s in proposals() {
        let Goal::PrintQueues { definition } = &s.goal else {
            panic!()
        };
        let target = &definition.initial.required.printers[0].printer.0;
        let mut a = Attempt::new(&s, 1);
        engine::execute_for_goal(&mut a.state, &s.goal, &format!("cupsaccept {target}")).unwrap();
        engine::execute_for_goal(&mut a.state, &s.goal, "@print-check").unwrap();
        assert!(!progress::current_solved(&a, &s));
        let rows = session(&mut a)
            .assignment
            .checks(&definition.initial)
            .unwrap();
        assert!(rows.iter().any(|r| !r.correct));
        assert!(rows.iter().any(|r| r.correct));
        a = Attempt::new(&s, 1);
        for raw in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        engine::execute_for_goal(&mut a.state, &s.goal, &format!("cupsdisable {target}")).unwrap();
        assert!(!progress::current_solved(&a, &s));
        assert!(!session(&mut a).assignment.checked);
        assert!(!session(&mut a).assignment.attempted);
        engine::execute_for_goal(&mut a.state, &s.goal, "@print-check").unwrap();
        assert!(
            session(&mut a)
                .assignment
                .checks(&definition.initial)
                .unwrap()
                .iter()
                .filter(|r| !r.correct)
                .all(|r| !r.feedback.is_empty())
        );
    }
}
#[test]
fn every_previous_412_value_and_frozen_415_catalog_remain_exact_in_the_current_catalog() {
    let old = pack::accepted412().unwrap();
    let admitted = pack::accepted415().unwrap();
    let normal = pack::current().unwrap();
    assert_eq!(old.len(), 412);
    assert_eq!(admitted.len(), 415);
    assert_eq!(normal.len(), 424);
    assert_eq!(
        serde_json::to_value(&normal[..415]).unwrap(),
        serde_json::to_value(&admitted).unwrap()
    );
    let current_units = dlms_learning_lab::course::current(&normal).unwrap();
    assert_eq!(current_units.len(), 163);
    let admitted_units = dlms_learning_lab::course::accepted415(&admitted).unwrap();
    assert_eq!(admitted_units.len(), 159);
    for prior in &admitted_units {
        let actual = current_units.iter().find(|u| u.id == prior.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(prior).unwrap()
        );
    }
    let retained = admitted
        .iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&retained).unwrap()
    );
    assert_eq!(
        dlms_learning_lab::course::accepted412(&old).unwrap().len(),
        158
    );
    let prior: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v20.json")).unwrap();
    let next: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v21.json")).unwrap();
    assert_eq!(prior["scenarios"].as_array().unwrap().len(), 39);
    assert_eq!(
        &next["scenarios"].as_array().unwrap()[..39],
        prior["scenarios"].as_array().unwrap().as_slice()
    );
}
#[test]
fn malformed_unreachable_or_invented_initial_source_refuses_without_a_success_string_shortcut() {
    for s in proposals() {
        let mut bad = s.clone();
        let Goal::PrintQueues { definition } = &mut bad.goal else {
            panic!()
        };
        definition.initial.reference = vec![Command("lpstat -t".into())];
        assert!(work::source(&bad).is_err());
        let mut bad = s.clone();
        let State::PrintQueues { simulation } = &mut bad.initial else {
            panic!()
        };
        simulation.assignment.checked = true;
        simulation.assignment.attempted = true;
        assert!(work::source(&bad).is_err());
        let mut a = Attempt::new(&s, 1);
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, "echo 'all requirements pass'")
                .is_err()
        );
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
    }
}
