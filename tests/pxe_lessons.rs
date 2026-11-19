use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    pxe::{source, workspace::Action},
};

const UNIT: &str = "linux-pxe-recovery";
const IDS: [&str; 2] = [
    "lpic2-pxe-firmware-selection",
    "lpic2-pxelinux-client-lookup",
];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/pxe-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn act(state: &mut State, lab: &pack::Scenario, practice: bool, action: &Action) {
    let input = format!(
        "{}{}",
        source::PREFIX,
        serde_json::json!({"practice": practice, "action": action})
    );
    engine::execute_for_goal(state, &lab.goal, &input).unwrap();
}

#[test]
fn network_boot_checkpoint_extends_its_predecessor_by_exactly_two_labs_and_one_unit() {
    let old = pack::accepted515().unwrap();
    let current = pack::accepted517().unwrap();
    assert_eq!((old.len(), current.len()), (515, 517));
    assert_eq!(
        serde_json::to_value(&current[..old.len()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&current[old.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let old_units = course::accepted515(&old).unwrap();
    let current_units = course::accepted517(&current).unwrap();
    assert_eq!((old_units.len(), current_units.len()), (208, 209));
    assert_eq!(
        serde_json::to_value(
            current_units
                .iter()
                .filter(|unit| unit.id != UNIT)
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let position = current_units.iter().position(|u| u.id == UNIT).unwrap();
    let unit = &current_units[position];
    assert_eq!(unit.labs, IDS);
    assert!(
        unit.prerequisites
            .iter()
            .all(|id| current_units[..position].iter().any(|u| &u.id == id))
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|s| s.track == track).count()),
        [115, 154, 168, 80]
    );
}

#[test]
fn checked_practice_has_no_assignment_credit_and_both_authored_repairs_pass() {
    for lab in authored() {
        let definition = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        for action in &definition.example.reference {
            act(&mut state, &lab, true, action);
        }
        assert!(!engine::solved(&state, &lab.goal));
        let (
            State::Pxe { simulation },
            State::Pxe {
                simulation: initial,
            },
        ) = (&state, &lab.initial)
        else {
            panic!()
        };
        assert_eq!(simulation.assignment, initial.assignment);
        assert!(definition.example.solved(&simulation.example.work));
        for input in &lab.solution {
            engine::execute_for_goal(&mut state, &lab.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
    }
}

#[test]
fn compatible_alternate_server_and_distinct_per_client_override_pass() {
    let mut labs = authored();
    let policy = labs.remove(0);
    let mut state = policy.initial.clone();
    let definition = source::source(&policy).unwrap();
    let Action::EditPolicy { mut draft } = definition.assignment.reference[0].clone() else {
        panic!()
    };
    draft.x64_uefi.server = "192.0.2.11".into();
    draft.x64_uefi.filename = "rescue/candidate.efi".into();
    act(&mut state, &policy, false, &Action::EditPolicy { draft });
    act(&mut state, &policy, false, &Action::SavePolicy);
    act(&mut state, &policy, false, &Action::Check);
    assert!(engine::solved(&state, &policy.goal));

    let entry = labs.remove(0);
    let mut state = entry.initial.clone();
    let definition = source::source(&entry).unwrap();
    let Action::EditEntry { mut draft } = definition.assignment.reference[0].clone() else {
        panic!()
    };
    draft.leaf = definition.assignment.cases[0].client.uuid.clone().unwrap();
    act(&mut state, &entry, false, &Action::EditEntry { draft });
    act(&mut state, &entry, false, &Action::SaveEntry);
    act(&mut state, &entry, false, &Action::Check);
    assert!(engine::solved(&state, &entry.goal));
}

#[test]
fn saved_partial_work_restores_exactly_and_reset_affects_only_its_scope() {
    let labs = authored();
    let lab = &labs[1];
    let definition = source::source(lab).unwrap();
    let mut attempt = Attempt::new(lab, 1);
    for (practice, task) in [(false, &definition.assignment), (true, &definition.example)] {
        act(&mut attempt.state, lab, practice, &task.reference[0]);
        act(&mut attempt.state, lab, practice, &Action::Check);
    }
    let mut profile = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    profile.attempts.insert(lab.id.clone(), attempt);
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&profile).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    let attempt = restored.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(attempt, lab));
    let State::Pxe { simulation } = &attempt.state else {
        panic!()
    };
    let example = simulation.example.clone();
    act(&mut attempt.state, lab, false, &Action::Reset);
    let (
        State::Pxe { simulation },
        State::Pxe {
            simulation: initial,
        },
    ) = (&attempt.state, &lab.initial)
    else {
        panic!()
    };
    assert_eq!(simulation.assignment, initial.assignment);
    assert_eq!(simulation.example, example);
    assert!(progress::save(&path, &restored).is_err());
}
