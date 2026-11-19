use dlms_learning_lab::{
    automation_workshop::{
        session::Session,
        source,
        workspace::{Action, Scope, Task, Work},
    },
    course, engine, pack, progress,
};

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/ccna-vlan-gateway-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}

fn session(state: &mut engine::State) -> &mut Session {
    let engine::State::AutomationWorkshop { simulation } = state else {
        panic!("Expected the CCNA work order")
    };
    simulation
}

#[test]
fn appended_lessons_keep_exact_older_catalog_and_follow_prerequisites() {
    let older = pack::accepted538().unwrap();
    let current = pack::accepted540().unwrap();
    assert_eq!((older.len(), current.len()), (538, 540));
    assert_eq!(
        serde_json::to_value(&older).unwrap(),
        serde_json::to_value(&current[..538]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(authored()).unwrap(),
        serde_json::to_value(&current[538..]).unwrap()
    );
    let before = course::accepted538(&older).unwrap();
    let latest = course::accepted540(&current).unwrap();
    let unit = "ccna-vlan-gateway-repair";
    assert_eq!((before.len(), latest.len()), (220, 221));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(latest.iter().filter(|u| u.id != unit).collect::<Vec<_>>()).unwrap()
    );
    let at = latest.iter().position(|u| u.id == unit).unwrap();
    assert!(
        latest[at]
            .prerequisites
            .iter()
            .all(|needed| latest[..at].iter().any(|u| &u.id == needed))
    );
    assert_eq!(
        latest[at].labs,
        [
            "ccna-default-access-membership",
            "ccna-switched-gateway-repair"
        ]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [127, 158, 175, 80]
    );
}

#[test]
fn author_methods_are_reachable_and_editing_after_pass_requires_another_check() {
    for scenario in authored() {
        let definition = source::source(&scenario).unwrap();
        let mut state = scenario.initial.clone();
        assert!(!engine::solved(&state, &scenario.goal));
        for op in &scenario.solution[..scenario.solution.len() - 1] {
            engine::execute_for_goal(&mut state, &scenario.goal, op).unwrap();
        }
        assert!(!engine::solved(&state, &scenario.goal));
        engine::execute_for_goal(
            &mut state,
            &scenario.goal,
            scenario.solution.last().unwrap(),
        )
        .unwrap();
        assert!(engine::solved(&state, &scenario.goal));
        let session = session(&mut state);
        assert_eq!(session.example.work, Work::new(&definition.example));
        let action = match &session.assignment.work.current {
            Scope::AccessDefault { .. } => Action::SetAccessDefault {
                choice: dlms_learning_lab::automation_workshop::access_defaults::Choice {
                    access: None,
                },
            },
            Scope::RoutedVlans { draft } => {
                let mut broken = draft.clone();
                broken.gateways[2].0 = "10.70.1.999".into();
                Action::ChangeSviDraft { draft: broken }
            }
            _ => panic!("Unexpected learner activity"),
        };
        session
            .assignment
            .work
            .apply(&definition.assignment, action)
            .unwrap();
        assert!(!session.assignment.work.checked);
        session
            .assignment
            .work
            .apply(&definition.assignment, Action::Check)
            .unwrap();
        assert!(!engine::solved(&state, &scenario.goal));
    }
}

#[test]
fn practice_stays_separate_and_unfinished_assignment_resumes_after_save() {
    let scenarios = authored();
    for lab in &scenarios {
        let definition = source::source(lab).unwrap();
        let mut attempt = progress::Attempt::new(lab, 1);
        let draft = session(&mut attempt.state);
        let action = match &definition.example {
            Task::AccessDefault { .. } => Action::SetAccessDefault {
                choice: dlms_learning_lab::automation_workshop::access_defaults::Choice {
                    access: Some(30),
                },
            },
            Task::RoutedVlans { supplied } => {
                let mut settings = dlms_learning_lab::automation_workshop::svi_draft::Draft::from(
                    &supplied.initial,
                );
                settings.interfaces[1].0 = "198.51.100.1/".into();
                Action::ChangeSviDraft {
                    draft: Box::new(settings),
                }
            }
            _ => panic!("Unexpected separate practice"),
        };
        draft
            .example
            .work
            .apply(&definition.example, action)
            .unwrap();
        draft
            .example
            .work
            .apply(&definition.example, Action::Check)
            .unwrap();
        assert_eq!(draft.assignment.work, Work::new(&definition.assignment));
        if let Scope::RoutedVlans { draft: work } = &draft.assignment.work.current {
            let mut unfinished = work.clone();
            unfinished.gateways[2].0 = "10.70.1.".into();
            draft
                .assignment
                .work
                .apply(
                    &definition.assignment,
                    Action::ChangeSviDraft { draft: unfinished },
                )
                .unwrap();
        }
        let progress = progress::Progress {
            schema_version: 1,
            attempts: [(lab.id.clone(), attempt)].into(),
            ..Default::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("synthetic.json");
        progress::save_with_catalog(&file, &progress, &scenarios).unwrap();
        let loaded = progress::load(&file, &scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(progress).unwrap(),
            serde_json::to_value(&loaded).unwrap()
        );
        assert!(!progress::current_solved(&loaded.attempts[&lab.id], lab));
    }
}
