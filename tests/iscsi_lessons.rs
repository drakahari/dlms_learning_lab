use dlms_learning_lab::{
    course,
    engine::{self, State},
    iscsi::{source, workspace::Attempt as Work},
    pack,
    progress::{self, Attempt, Progress},
};

const UNIT: &str = "linux-iscsi-initiator";
const IDS: [&str; 2] = ["lpic2-iscsi-target-identity", "lpic2-iscsi-startup-policy"];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/iscsi-lessons-v1.json"))
        .unwrap()
        .scenarios
}

#[test]
fn iscsi_checkpoint_retains_exact_network_boot_and_prior_work() {
    let prior = pack::accepted517().unwrap();
    let current = pack::accepted519().unwrap();
    assert_eq!((prior.len(), current.len()), (517, 519));
    assert_eq!(
        serde_json::to_value(&current[..prior.len()]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&current[prior.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let old_units = course::accepted517(&prior).unwrap();
    let current_units = course::accepted519(&current).unwrap();
    assert_eq!((old_units.len(), current_units.len()), (209, 210));
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
        current_units[..position]
            .iter()
            .any(|u| u.id == "persistent-mounts")
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|s| s.track == track).count()),
        [115, 154, 170, 80]
    );
}

#[test]
fn authored_references_pass_but_read_only_inspection_or_a_script_do_not() {
    for lab in authored() {
        let definition = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        let _ = engine::execute_for_goal(&mut state, &lab.goal, "iscsiadm -m node");
        assert!(engine::execute_for_goal(&mut state, &lab.goal, "lsblk --sysroot /").is_err());
        engine::execute_for_goal(&mut state, &lab.goal, "iscsi-workspace check").unwrap();
        assert!(!engine::solved(&state, &lab.goal));
        for command in &lab.solution {
            engine::execute_for_goal(&mut state, &lab.goal, command).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
        let State::Iscsi { simulation } = &state else {
            panic!()
        };
        assert_eq!(simulation.example.work, Work::new(&definition.example));
        assert!(engine::execute(&mut state, "iscsiadm -m session").is_err());
    }
}

#[test]
fn restores_discovery_and_drafts_without_a_connection_then_reset_preserves_practice() {
    let labs = authored();
    let lab = &labs[0];
    let mut attempt = Attempt::new(lab, 1);
    engine::execute_for_goal(&mut attempt.state, &lab.goal, &lab.solution[0]).unwrap();
    engine::execute_for_goal(&mut attempt.state, &lab.goal, "iscsi-workspace check").unwrap();
    let State::Iscsi { simulation } = &mut attempt.state else {
        panic!()
    };
    let task = source::source(lab).unwrap();
    simulation.assignment.work.input = "iscsiadm -m node -T".into();
    simulation.example.work.input = "iscsiadm -m session -P".into();
    let mut profile = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    profile.attempts.insert(lab.id.clone(), attempt);
    let store = tempfile::tempdir().unwrap();
    let path = store.path().join("progress.json");
    progress::save_with_catalog(&path, &profile, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&profile).unwrap()
    );
    let saved = restored.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(saved, lab));
    let State::Iscsi { simulation } = &mut saved.state else {
        panic!()
    };
    assert!(simulation.assignment.work.initiator.active.is_empty());
    assert_eq!(simulation.assignment.work.initiator.nodes.len(), 2);
    let example = simulation.example.clone();
    simulation.assignment.work = Work::new(&task.assignment);
    assert_eq!(simulation.example, example);
    assert!(simulation.assignment.work.initiator.nodes.is_empty());
    simulation.reset_example(task).unwrap();
    assert!(simulation.example.work.input.is_empty());
    assert!(progress::save(&path, &restored).is_err());
}
