use dlms_learning_lab::{
    ata_smart::{
        source::{self, Action},
        workspace::Work,
    },
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};

const TRACKS: [&str; 4] = ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"];
const UNIT: &str = "linux-ata-smart-maintenance";
const IDS: [&str; 2] = [
    "lpic2-smart-short-diagnosis",
    "lpic2-smart-monitor-identity",
];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/ata-smart-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn act(state: &mut State, lab: &pack::Scenario, action: &Action) {
    let input = format!(
        "{}{}",
        source::PREFIX,
        serde_json::to_string(action).unwrap()
    );
    engine::execute_for_goal(state, &lab.goal, &input).unwrap();
}

#[test]
fn the_two_different_smart_tasks_extend_the_exact_519_checkpoint() {
    let before = pack::accepted519().unwrap();
    let current = pack::accepted521().unwrap();
    assert_eq!((before.len(), current.len()), (519, 521));
    assert_eq!(
        serde_json::to_value(&current[..before.len()]).unwrap(),
        serde_json::to_value(&before).unwrap(),
    );
    assert_eq!(
        serde_json::to_value(&current[before.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap(),
    );
    let baseline = course::accepted519(&before).unwrap();
    let units = course::accepted521(&current).unwrap();
    assert_eq!((baseline.len(), units.len()), (210, 211));
    assert_eq!(
        serde_json::to_value(units.iter().filter(|u| u.id != UNIT).collect::<Vec<_>>()).unwrap(),
        serde_json::to_value(&baseline).unwrap(),
    );
    let position = units.iter().position(|u| u.id == UNIT).unwrap();
    assert_eq!(units[position].labs, IDS);
    assert!(
        units[position]
            .prerequisites
            .iter()
            .all(|p| units[..position].iter().any(|u| &u.id == p))
    );
    assert_eq!(
        TRACKS.map(|track| current.iter().filter(|s| s.track == track).count()),
        [115, 154, 172, 80]
    );
}

#[test]
fn shipped_references_pass_and_results_cannot_survive_actual_breakage() {
    for (n, lab) in authored().iter().enumerate() {
        let mut state = lab.initial.clone();
        let d = source::source(lab).unwrap();
        let unsupported = format!(
            "{}{{\"op\":\"Run\",\"input\":\"printf success\"}}",
            source::PREFIX
        );
        let _ = engine::execute_for_goal(&mut state, &lab.goal, &unsupported);
        act(&mut state, lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
        for input in &lab.solution {
            engine::execute_for_goal(&mut state, &lab.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
        let State::AtaSmart { simulation } = &state else {
            panic!()
        };
        assert_eq!(simulation.example.work, Work::new(&d.example));
        if n == 0 {
            act(
                &mut state,
                lab,
                &Action::Record {
                    ticket: dlms_learning_lab::ata_smart::workspace::Ticket::default(),
                },
            );
        } else {
            act(
                &mut state,
                lab,
                &Action::Draft {
                    contents: "DEVICESCAN -H".into(),
                },
            );
            let save = format!("{}{{\"op\":\"Save\"}}", source::PREFIX);
            assert!(engine::execute_for_goal(&mut state, &lab.goal, &save).is_err());
        }
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
    }
}

#[test]
fn real_profile_store_restores_pending_test_and_separate_drafts_and_reset() {
    let labs = authored();
    let lab = &labs[0];
    let d = source::source(lab).unwrap();
    let mut attempt = Attempt::new(lab, 1);
    let first = &d.assignment.inventory.disks[1].current_name;
    act(
        &mut attempt.state,
        lab,
        &Action::Run {
            input: format!("smartctl -t short {first}"),
        },
    );
    act(&mut attempt.state, lab, &Action::Advance);
    act(&mut attempt.state, lab, &Action::Check);
    let State::AtaSmart { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.assignment.work.command = "smartctl -l selftest".into();
    simulation.example.work.command = "smartctl -c".into();
    let mut progress = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    progress.attempts.insert(lab.id.clone(), attempt);
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("progress.json");
    progress::save_with_catalog(&path, &progress, &labs).unwrap();
    let mut recovered = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&recovered).unwrap(),
        serde_json::to_value(&progress).unwrap()
    );
    let attempt = recovered.attempts.get_mut(&lab.id).unwrap();
    let State::AtaSmart { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(
        simulation.assignment.work.state.disks[1].remaining_minutes,
        Some(2)
    );
    let example = simulation.example.clone();
    act(&mut attempt.state, lab, &Action::Reset);
    let State::AtaSmart { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(simulation.assignment.work, Work::new(&d.assignment));
    assert_eq!(simulation.example, example);
    assert!(progress::save(&path, &recovered).is_err());
}
