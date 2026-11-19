use dlms_learning_lab::{
    automation_workshop::{
        session::Session,
        source,
        workspace::{Action, Work},
    },
    course, engine, pack, progress,
};

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/host-and-dns-security-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}

fn session(state: &mut engine::State) -> &mut Session {
    let engine::State::AutomationWorkshop { simulation } = state else {
        panic!("Expected the supplied Linux work order");
    };
    simulation
}

#[test]
fn both_original_tracks_keep_their_predecessors_and_teach_before_assessing() {
    let before = pack::accepted536().unwrap();
    let all = pack::accepted538().unwrap();
    assert_eq!((before.len(), all.len()), (536, 538));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&all[..536]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(authored()).unwrap(),
        serde_json::to_value(&all[536..]).unwrap()
    );
    let old = course::accepted536(&before).unwrap();
    let current = course::accepted538(&all).unwrap();
    assert_eq!((old.len(), current.len()), (218, 220));
    let units = [
        "linux-retire-unused-services",
        "linux-authenticated-zone-transfer",
    ];
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            current
                .iter()
                .filter(|unit| !units.contains(&unit.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    for id in units {
        let at = current.iter().position(|unit| unit.id == id).unwrap();
        assert!(current[at].prerequisites.iter().all(|required| {
            current[..at]
                .iter()
                .any(|previous| &previous.id == required)
        }));
        assert_eq!(current[at].labs.len(), 1);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|lab| lab.track == track).count()),
        [125, 158, 175, 80]
    );
}

#[test]
fn authored_solutions_require_actual_actions_and_do_not_credit_guided_practice() {
    for lab in authored() {
        let definition = source::source(&lab).unwrap();
        let mut attempt = lab.initial.clone();
        assert!(!engine::solved(&attempt, &lab.goal));
        for action in &lab.solution[..lab.solution.len() - 1] {
            engine::execute_for_goal(&mut attempt, &lab.goal, action).unwrap();
        }
        assert!(!engine::solved(&attempt, &lab.goal));
        engine::execute_for_goal(&mut attempt, &lab.goal, lab.solution.last().unwrap()).unwrap();
        assert!(engine::solved(&attempt, &lab.goal));
        let session = session(&mut attempt);
        assert_eq!(session.example.work, Work::new(&definition.example));
        if let dlms_learning_lab::automation_workshop::workspace::Task::SocketRetirement {
            supplied,
        } = &definition.assignment
        {
            session
                .assignment
                .work
                .apply(
                    &definition.assignment,
                    Action::RetirementCommand {
                        text: format!("systemctl start {}", supplied.activating_socket),
                    },
                )
                .unwrap();
        } else {
            session
                .assignment
                .work
                .apply(
                    &definition.assignment,
                    Action::ChangeTransferRule {
                        protected: false,
                        text: "allow-transfer { none; };".into(),
                    },
                )
                .unwrap();
        }
        assert!(!engine::solved(&attempt, &lab.goal));
    }
}

#[test]
fn both_draft_types_resume_exactly_and_writes_in_practice_do_not_move_to_assignment() {
    let labs = authored();
    for lab in &labs {
        let definition = source::source(lab).unwrap();
        let mut attempt = progress::Attempt::new(lab, 1);
        let ui = session(&mut attempt.state);
        let edit = if lab.track == "LPIC-1" {
            Action::RetirementDraft {
                text: "systemctl status legacy-import.socket".into(),
            }
        } else {
            Action::ChangeTransferRule {
                protected: false,
                text: "/* unfinished rule */".into(),
            }
        };
        ui.example.work.apply(&definition.example, edit).unwrap();
        assert_eq!(ui.assignment.work, Work::new(&definition.assignment));
        ui.example
            .work
            .apply(&definition.example, Action::Check)
            .unwrap();
        let original = progress::Progress {
            schema_version: 1,
            attempts: [(lab.id.clone(), attempt)].into(),
            ..progress::Progress::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.json");
        progress::save_with_catalog(&path, &original, &labs).unwrap();
        let reopened = progress::load(&path, &labs).unwrap();
        assert_eq!(
            serde_json::to_value(&original).unwrap(),
            serde_json::to_value(&reopened).unwrap()
        );
        assert!(!progress::current_solved(&reopened.attempts[&lab.id], lab));
    }
}
