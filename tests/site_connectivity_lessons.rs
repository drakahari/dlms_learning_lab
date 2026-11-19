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
        "../scenarios/site-connectivity-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}

fn session(state: &mut engine::State) -> &mut Session {
    let engine::State::AutomationWorkshop { simulation } = state else {
        panic!("Expected circuit planning work");
    };
    simulation
}

#[test]
fn both_retained_values_and_prerequisite_order_remain_exact() {
    let prior = pack::accepted534().unwrap();
    let current = pack::accepted536().unwrap();
    assert_eq!((prior.len(), current.len()), (534, 536));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&current[..534]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(authored()).unwrap(),
        serde_json::to_value(&current[534..]).unwrap()
    );
    let old_units = course::accepted534(&prior).unwrap();
    let current_units = course::accepted536(&current).unwrap();
    let id = "ccna-site-connectivity-planning";
    assert_eq!((old_units.len(), current_units.len()), (217, 218));
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(
            current_units
                .iter()
                .filter(|unit| unit.id != id)
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let index = current_units.iter().position(|unit| unit.id == id).unwrap();
    assert!(
        current_units[index]
            .prerequisites
            .iter()
            .all(|needed| current_units[..index].iter().any(|unit| &unit.id == needed))
    );
    assert_eq!(
        current_units[index].labs,
        [
            "ccna-diverse-branch-paths",
            "ccna-local-and-hosted-connectivity"
        ]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|lab| lab.track == track).count()),
        [125, 157, 174, 80],
    );
}

#[test]
fn author_methods_pass_and_breaking_a_real_circuit_removes_success() {
    for lesson in authored() {
        let definition = source::source(&lesson).unwrap();
        let mut state = lesson.initial.clone();
        assert!(!engine::solved(&state, &lesson.goal));
        for op in &lesson.solution {
            engine::execute_for_goal(&mut state, &lesson.goal, op).unwrap();
        }
        assert!(engine::solved(&state, &lesson.goal));
        let work = session(&mut state);
        assert_eq!(work.example.work, Work::new(&definition.example));
        work.assignment
            .work
            .apply(
                &definition.assignment,
                Action::SiteLink {
                    index: 8,
                    connected: true,
                },
            )
            .unwrap();
        assert!(!work.assignment.work.checked);
        work.assignment
            .work
            .apply(&definition.assignment, Action::Check)
            .unwrap();
        assert!(!engine::solved(&state, &lesson.goal));
    }
}

#[test]
fn saved_circuit_choices_resume_without_crossing_the_example_boundary() {
    let lessons = authored();
    let lab = &lessons[0];
    let definition = source::source(lab).unwrap();
    let mut attempt = progress::Attempt::new(lab, 1);
    let sample = session(&mut attempt.state);
    for (index, connected) in [(5, false), (6, true)] {
        sample
            .example
            .work
            .apply(&definition.example, Action::SiteLink { index, connected })
            .unwrap();
    }
    sample
        .example
        .work
        .apply(&definition.example, Action::Check)
        .unwrap();
    assert!(
        sample
            .example
            .work
            .outcomes(&definition.example)
            .iter()
            .all(|row| row.passed)
    );
    sample
        .assignment
        .work
        .apply(
            &definition.assignment,
            Action::SiteLink {
                index: 8,
                connected: false,
            },
        )
        .unwrap();
    let before = progress::Progress {
        schema_version: 1,
        attempts: [(lab.id.clone(), attempt)].into(),
        ..progress::Progress::default()
    };
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("sample.json");
    progress::save_with_catalog(&data, &before, &lessons).unwrap();
    let after = progress::load(&data, &lessons).unwrap();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after).unwrap()
    );
    assert!(!progress::current_solved(&after.attempts[&lab.id], lab));
}
