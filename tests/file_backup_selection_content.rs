//! Actual source behavior: independent alternatives, adverse cases, repair and ungraded practice.
#[path = "support/backup_selection.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, State},
    file_backup_teaching::Learning,
    pack::Scenario,
};
use fixture::*;
fn run(s: &Scenario, cmds: &[String]) -> State {
    let mut state = s.initial.clone();
    for c in cmds {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap_or_else(|e| panic!("{c}: {e}"));
    }
    state
}

fn expected(v: &serde_json::Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
#[test]
fn exact_initial_and_reference_plus_two_equivalent_supported_solutions() {
    let s = source();
    let r = review();
    assert_eq!(flags(&s.initial, &s), expected(&r["initial_expected"]));
    assert_eq!(s.solution, commands(&r["reference"]["commands"]));
    assert_eq!(r["alternates"].as_array().unwrap().len(), 2);
    for row in std::iter::once(&r["reference"]).chain(r["alternates"].as_array().unwrap()) {
        let state = run(&s, &commands(&row["commands"]));
        assert_eq!(flags(&state, &s), expected(&row["expected"]));
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn every_literal_wrong_partial_damaged_and_repair_vector_checks_all_thirteen_rows() {
    let s = source();
    let r = review();
    assert_eq!(r["negatives"].as_array().unwrap().len(), 8);
    for row in r["negatives"].as_array().unwrap() {
        let mut state = s.initial.clone();
        let expected_error = row["expect_command_error_index"]
            .as_u64()
            .map(|n| n as usize);
        for (index, command) in commands(&row["commands"]).iter().enumerate() {
            let before = state.clone();
            let result = engine::execute_for_goal(&mut state, &s.goal, command);
            if expected_error == Some(index) {
                assert!(
                    result.is_err(),
                    "{} must reject missing archive",
                    row["name"]
                );
                assert_eq!(state, before);
            } else {
                result.unwrap_or_else(|e| panic!("{}: {command}: {e}", row["name"]));
            }
        }
        let wanted = expected(if row["expected"].is_null() {
            &row["expected_after_successful_prefix"]
        } else {
            &row["expected"]
        });
        assert_eq!(wanted.len(), 13);
        assert!(wanted.contains(&false));
        assert_eq!(flags(&state, &s), wanted, "{}", row["name"]);
        assert!(!engine::solved(&state, &s.goal));
        for c in commands(if row["repair"].is_null() {
            &r["reference"]["commands"]
        } else {
            &row["repair"]
        }) {
            engine::execute_for_goal(&mut state, &s.goal, &c).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "repair {}: {:?}",
            row["name"],
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn equal_top_folder_and_member_names_cannot_stand_in_for_selected_source_contents() {
    let s = source();
    let State::FileBackup { simulation } = &s.initial else {
        panic!()
    };
    assert_eq!(simulation.trees.len(), 6);
    let roots = [
        "/etc/orchard",
        "/var/lib/orchard",
        "/var/cache/orchard",
        "/run/orchard",
    ];
    for x in roots {
        assert!(simulation.trees.contains_key(x));
    }
    let left = &simulation.trees[roots[0]];
    let right = &simulation.trees[roots[1]];
    assert_eq!(
        left.entries.keys().collect::<Vec<_>>(),
        right.entries.keys().collect::<Vec<_>>()
    );
    assert_ne!(left, right);
    let wrong = run(
        &s,
        &[
            "tar -cf /backups/orchard-config.tar -C /var/cache orchard".into(),
            "tar -cf /backups/orchard-data.tar -C /run orchard".into(),
            "tar -xf /backups/orchard-config.tar -C /recovery/etc".into(),
            "tar -xf /backups/orchard-data.tar -C /recovery/lib".into(),
        ],
    );
    let rows = flags(&wrong, &s);
    assert!(
        rows[4] && rows[11] && rows[12],
        "Filename/root checks intentionally pass while content rejects"
    );
    assert!(!rows[5] && !rows[6]);
    assert!(rows[7..11].iter().all(|x| !*x));
    assert!(!engine::solved(&wrong, &s.goal));
}
#[test]
fn willow_practice_is_separate_runnable_changeable_repairable_and_ungraded() {
    let s = source();
    let lesson = s.file_backup_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    let State::FileBackup { simulation } = &s.initial else {
        panic!()
    };
    assert!(lesson.separate_from(simulation));
    assert!(!lesson.concepts[0].text.contains("/etc/orchard"));
    assert!(!s.objective.contains("FROM /etc/orchard"));
    let r = review();
    let mut l = Learning::new(lesson).unwrap();
    assert_eq!(
        lesson
            .definition
            .checks(&l.state)
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        expected(&r["practice"]["initial_expected"])
    );
    for c in &lesson.example_commands {
        l.command = c.command.clone();
        l.execute(lesson).unwrap();
    }
    assert_eq!(
        lesson
            .definition
            .checks(&l.state)
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        expected(&r["practice"]["after_worked_expected"])
    );
    for c in &lesson.try_change.commands {
        l.command = c.command.clone();
        l.execute(lesson).unwrap();
    }
    assert_eq!(
        lesson
            .definition
            .checks(&l.state)
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        expected(&r["practice"]["after_experiment_expected"])
    );
    for c in &lesson.repair {
        l.command = c.command.clone();
        l.execute(lesson).unwrap();
    }
    assert_eq!(
        lesson
            .definition
            .checks(&l.state)
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        expected(&r["practice"]["after_repair_expected"])
    );
    assert!(!engine::solved(&s.initial, &s.goal));
}
