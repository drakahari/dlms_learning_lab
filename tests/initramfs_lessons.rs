use dlms_learning_lab::{
    course,
    engine::{self, State},
    initramfs_source as binding,
    initramfs_workspace::Action,
    pack,
    progress::{self, Attempt, Progress},
};

const UNIT: &str = "linux-initramfs-recovery";
const IDS: [&str; 2] = [
    "lpic2-initramfs-root-drivers",
    "lpic2-initramfs-kernel-handover",
];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/initramfs-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn act(state: &mut State, lesson: &pack::Scenario, practice: bool, action: &Action) {
    let routed = serde_json::json!({"practice": practice, "action": action});
    engine::execute_for_goal(state, &lesson.goal, &format!("{}{routed}", binding::PREFIX)).unwrap();
}

#[test]
fn current_catalog_adds_only_the_two_authored_lessons_and_their_unit() {
    let before = pack::accepted513().unwrap();
    let current = pack::accepted515().unwrap();
    assert_eq!((before.len(), current.len()), (513, 515));
    assert_eq!(
        serde_json::to_value(&current[..before.len()]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&current[before.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let old_units = course::accepted513(&before).unwrap();
    let units = course::accepted515(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (207, 208));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|unit| unit.id != UNIT)
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let position = units.iter().position(|unit| unit.id == UNIT).unwrap();
    assert_eq!(units[position].labs, IDS);
    for prerequisite in &units[position].prerequisites {
        assert!(
            units[..position]
                .iter()
                .any(|unit| &unit.id == prerequisite)
        );
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|track| current
            .iter()
            .filter(|lesson| lesson.track == track)
            .count()),
        [115, 154, 166, 80]
    );
}

#[test]
fn separate_example_and_authored_solution_evaluate_real_selected_boot_state() {
    for lesson in authored() {
        let definition = binding::source(&lesson).unwrap();
        let mut state = lesson.initial.clone();
        for action in &definition.example.reference {
            act(&mut state, &lesson, true, action);
        }
        let (
            State::Initramfs { simulation },
            State::Initramfs {
                simulation: initial,
            },
        ) = (&state, &lesson.initial)
        else {
            panic!("Expected early-userspace workspaces");
        };
        assert_eq!(simulation.assignment, initial.assignment);
        assert!(definition.example.solved(&simulation.example.work));
        assert!(!engine::solved(&state, &lesson.goal));
        for input in &lesson.solution {
            engine::execute_for_goal(&mut state, &lesson.goal, input).unwrap();
        }
        assert!(
            engine::solved(&state, &lesson.goal),
            "{:?}",
            engine::checks(&state, &lesson.goal)
        );
        act(
            &mut state,
            &lesson,
            false,
            &Action::ConfigDraft {
                text: "omit_drivers=\"nvme\"".into(),
            },
        );
        assert!(!engine::solved(&state, &lesson.goal));
    }
}

#[test]
fn authored_facts_and_internal_routes_reject_incompatible_input() {
    for mut lesson in authored() {
        let mut state = lesson.initial.clone();
        assert!(engine::execute(&mut state, "dracut --force /boot/test.img 6.6.61").is_err());
        assert!(
            engine::execute_for_goal(
                &mut state,
                &lesson.goal,
                &format!(
                    "{}{}",
                    binding::PREFIX,
                    serde_json::json!({
                        "practice":false,"action":{"op":"Check"},"unexpected":true,
                    })
                )
            )
            .is_err()
        );
        let engine::Goal::Initramfs { definition } = &mut lesson.goal else {
            panic!("Expected an early-userspace definition");
        };
        definition.assignment.reference = vec![Action::Check];
        assert!(binding::source(&lesson).is_err());
    }
}

#[test]
fn checked_partial_states_and_unfinished_commands_survive_save_load_and_selected_reset() {
    let lessons = authored();
    let lesson = &lessons[1];
    let definition = binding::source(lesson).unwrap();
    let mut attempt = Attempt::new(lesson, 1);
    for (practice, task) in [(true, &definition.example), (false, &definition.assignment)] {
        // Generate a matching image and draft an entry without saving the selection.
        for action in &task.reference[..2] {
            act(&mut attempt.state, lesson, practice, action);
        }
        act(
            &mut attempt.state,
            lesson,
            practice,
            &Action::CommandDraft {
                text: format!("dracut --kver {}", task.wanted_release),
            },
        );
        act(&mut attempt.state, lesson, practice, &Action::Check);
    }
    let mut progress = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    progress.attempts.insert(lesson.id.clone(), attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &progress, &lessons).unwrap();
    let mut loaded = progress::load(&path, &lessons).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&progress).unwrap()
    );
    let attempt = loaded.attempts.get_mut(&lesson.id).unwrap();
    assert!(!progress::current_solved(attempt, lesson));
    let State::Initramfs { simulation } = &attempt.state else {
        panic!()
    };
    let example = simulation.example.clone();
    act(&mut attempt.state, lesson, false, &Action::Reset);
    let (
        State::Initramfs { simulation },
        State::Initramfs {
            simulation: initial,
        },
    ) = (&attempt.state, &lesson.initial)
    else {
        panic!()
    };
    assert_eq!(simulation.assignment, initial.assignment);
    assert_eq!(simulation.example, example);
}
