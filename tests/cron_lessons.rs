//! Four distinct privately registered authored lessons. Variants within each fresh
//! probe set are grading cases, not separately counted authored scenarios.
use dlms_learning_lab::{
    cron_workspace::{self as work, ReferenceAction},
    engine::{self, Goal, State},
    pack,
};
#[path = "support/cron_lessons.rs"]
mod fixture;
fn lessons() -> Vec<pack::Scenario> {
    fixture::lessons()
}
fn review() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/cron-lessons-review-v1.json")).unwrap()
}
fn rows() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/cron-lessons-rows-v1.json")).unwrap()
}
fn actions(value: &serde_json::Value) -> Vec<ReferenceAction> {
    serde_json::from_value(value.clone()).unwrap()
}
fn execute(s: &pack::Scenario, initial: &State, actions: &[ReferenceAction]) -> State {
    let mut state = initial.clone();
    for action in actions {
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &format!("{}{}", work::PREFIX, serde_json::to_string(action).unwrap()),
        )
        .unwrap();
    }
    state
}
fn checked(s: &pack::Scenario, state: &State) -> Vec<bool> {
    engine::checks(state, &s.goal)
        .iter()
        .map(|row| row.0)
        .collect()
}
fn expected(value: &serde_json::Value) -> Vec<bool> {
    serde_json::from_value(value.clone()).unwrap()
}
#[test]
fn four_distinct_registered_lessons_have_actual_prerequisites_and_isolated_examples() {
    let normal = pack::bundled().unwrap();
    let sources = lessons();
    assert_eq!(sources.len(), 4);
    assert_eq!(normal.len(), 164);
    assert_eq!(
        serde_json::to_value(&normal[160..164]).unwrap(),
        serde_json::to_value(&sources).unwrap()
    );
    let ids: std::collections::BTreeSet<_> = normal
        .iter()
        .chain(sources.iter())
        .map(|s| s.id.as_str())
        .collect();
    for s in &sources {
        assert!(!normal[..160].iter().any(|old| old.id == s.id));
        assert_eq!(s.track, "LPIC-1");
        assert_eq!(s.unit, "linux-cron-foundations");
        for id in &s.prerequisites {
            assert!(ids.contains(id.as_str()), "{} prerequisite {id}", s.id);
        }
        let Goal::Cron { definition } = &s.goal else {
            unreachable!()
        };
        assert_eq!(definition.probes.len(), 16);
        let lesson = s.cron_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.separate_from(definition));
        let mut learning = dlms_learning_lab::cron_teaching::Learning::new(lesson).unwrap();
        let original = learning.simulation.clone();
        for n in 0..lesson.example_actions.len() {
            learning.selected_example = n;
            learning.fill(lesson).unwrap();
            assert_eq!(
                learning.simulation, original,
                "Fill cannot execute/install/remove: {} step{n}",
                s.id
            );
        }
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}
#[test]
fn taught_references_two_alternatives_and_literal_initial_vectors_match_actual_behavior() {
    let reviews = review();
    let rows = rows();
    for (i, s) in lessons().iter().enumerate() {
        assert_eq!(reviews[i]["id"], s.id);
        assert_eq!(rows["rows"][i]["id"], s.id);
        assert_eq!(
            checked(s, &s.initial),
            expected(&rows["rows"][i]["initial"]),
            "{} initial",
            s.id
        );
        let reference = execute(s, &s.initial, &actions(&reviews[i]["reference"]));
        assert_eq!(
            checked(s, &reference),
            expected(&rows["rows"][i]["reference"]),
            "{} reference",
            s.id
        );
        assert!(engine::solved(&reference, &s.goal));
        let alternatives = reviews[i]["alternatives"].as_array().unwrap();
        assert_eq!(alternatives.len(), 2);
        for (n, alternative) in alternatives.iter().enumerate() {
            let state = execute(s, &s.initial, &actions(alternative));
            assert_eq!(
                checked(s, &state),
                expected(&rows["rows"][i]["alternatives"][n]),
                "{} alternate{n}",
                s.id
            );
            assert!(engine::solved(&state, &s.goal));
        }
    }
}
#[test]
fn independently_reasoned_wrong_programs_do_not_pass_even_when_current_file_matches() {
    let reviews = review();
    let vectors = rows();
    for (i, s) in lessons().iter().enumerate() {
        let wrong = reviews[i]["wrong"].as_array().unwrap();
        let expected_rows = vectors["rows"][i]["wrong_after_full_actions"]
            .as_array()
            .unwrap();
        assert_eq!(wrong.len(), expected_rows.len());
        for (n, wrong) in wrong.iter().enumerate() {
            let state = execute(s, &s.initial, &actions(&wrong["actions"]));
            let actual = checked(s, &state);
            assert_eq!(
                actual,
                expected(&expected_rows[n]["rows"]),
                "{} wrong{}: {}",
                s.id,
                n,
                wrong["name"]
            );
            assert!(actual.iter().any(|ok| !*ok));
        }
    }
}
#[test]
fn partial_work_remains_partial_until_actual_requested_current_run() {
    let reviews = review();
    let vectors = rows();
    for (i, s) in lessons().iter().enumerate() {
        let reference = actions(&reviews[i]["reference"]);
        let routes: &[(&str, usize)] = match i {
            0 | 1 => &[("installed_not_run", 1)],
            2 => &[
                ("policy_only_no_remove_no_run", 1),
                ("policy_and_remove_not_run", 2),
            ],
            3 => &[("system_only_not_run", 1), ("all_documents_not_run", 3)],
            _ => unreachable!(),
        };
        for (key, count) in routes.iter().copied() {
            let state = execute(s, &s.initial, &reference[..count]);
            let actual = checked(s, &state);
            assert_eq!(
                actual,
                expected(&vectors["rows"][i][key]),
                "{} partial {key}",
                s.id
            );
            assert!(actual.iter().any(|ok| !*ok));
        }
    }
}
#[test]
fn success_is_invalidated_by_real_install_without_clearing_already_correct_output() {
    let reviews = review();
    let vectors = rows();
    for (i, s) in lessons().iter().enumerate() {
        let good = execute(s, &s.initial, &actions(&reviews[i]["reference"]));
        assert!(engine::solved(&good, &s.goal));
        let Goal::Cron { definition } = &s.goal else {
            unreachable!()
        };
        let damage = match i {
            0 => actions(&reviews[i]["wrong"][0]["actions"])[0].clone(),
            1 => {
                let mut action = actions(&reviews[i]["reference"])[0].clone();
                let ReferenceAction::Install { text, .. } = &mut action else {
                    unreachable!()
                };
                *text = text.replace("1-5", "*");
                action
            }
            2 => ReferenceAction::Install {
                actor: "root".into(),
                target: work::Document::User {
                    owner: "sam".into(),
                },
                text: definition.initial.tables["sam"].clone(),
            },
            3 => actions(&reviews[i]["wrong"][2]["actions"])[0].clone(),
            _ => unreachable!(),
        };
        let broken = execute(s, &good, &[damage]);
        assert_eq!(
            checked(s, &broken),
            expected(&vectors["rows"][i]["damage_after_pass"]["rows"]),
            "{} damaged",
            s.id
        );
        assert!(!engine::solved(&broken, &s.goal));
        let (State::Cron { simulation: good }, State::Cron { simulation: broken }) =
            (&good, &broken)
        else {
            unreachable!()
        };
        assert_eq!(good.clock, broken.clock);
        assert_eq!(
            good.files, broken.files,
            "No fabricated run/file mutation while changing the installed table"
        );
    }
}
