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
        "../scenarios/network-evidence-review-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}

fn session(state: &mut engine::State) -> &mut Session {
    let engine::State::AutomationWorkshop { simulation } = state else {
        panic!("Expected network change work");
    };
    simulation
}

#[test]
fn evidence_review_preserves_all_532_labs_and_216_units() {
    let prior = pack::accepted532().unwrap();
    let next = pack::accepted534().unwrap();
    assert_eq!((prior.len(), next.len()), (532, 534));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&next[..532]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(authored()).unwrap(),
        serde_json::to_value(&next[532..]).unwrap()
    );
    let old = course::accepted532(&prior).unwrap();
    let current = course::accepted534(&next).unwrap();
    let unit = "ccna-review-network-change-evidence";
    assert_eq!((old.len(), current.len()), (216, 217));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(current.iter().filter(|u| u.id != unit).collect::<Vec<_>>()).unwrap()
    );
    let at = current.iter().position(|u| u.id == unit).unwrap();
    assert_eq!(
        current[at].labs,
        [
            "ccna-review-ai-operations-claim",
            "ccna-review-declarative-change"
        ]
    );
    assert!(
        current[at]
            .prerequisites
            .iter()
            .all(|p| current[..at].iter().any(|u| &u.id == p))
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| next.iter().filter(|s| s.track == track).count()),
        [123, 157, 174, 80]
    );
}

#[test]
fn both_actual_author_solutions_pass_only_after_check_and_leave_practice_free() {
    for lesson in authored() {
        let d = source::source(&lesson).unwrap();
        let mut state = lesson.initial.clone();
        assert!(!engine::solved(&state, &lesson.goal));
        for action in lesson.solution.iter().take(lesson.solution.len() - 1) {
            engine::execute_for_goal(&mut state, &lesson.goal, action).unwrap();
        }
        assert!(!engine::solved(&state, &lesson.goal));
        engine::execute_for_goal(&mut state, &lesson.goal, lesson.solution.last().unwrap())
            .unwrap();
        assert!(engine::solved(&state, &lesson.goal));
        let s = session(&mut state);
        assert_eq!(s.example.work, Work::new(&d.example));
        s.assignment
            .work
            .apply(&d.assignment, Action::Restore)
            .unwrap();
        assert!(!engine::solved(&state, &lesson.goal));
    }
}

#[test]
fn fixed_observations_survive_reload_and_keep_separate_credit() {
    let labs = authored();
    let current = &labs[0];
    let d = source::source(current).unwrap();
    let mut attempt = progress::Attempt::new(current, 1);
    let app = session(&mut attempt.state);
    app.assignment
        .work
        .apply(&d.assignment, Action::ReadForecastSample)
        .unwrap();
    app.example.work.apply(&d.example, Action::Check).unwrap();
    let store = progress::Progress {
        schema_version: 1,
        attempts: [(current.id.clone(), attempt)].into(),
        ..progress::Progress::default()
    };
    let sandbox = tempfile::tempdir().unwrap();
    let path = sandbox.path().join("progress.json");
    progress::save_with_catalog(&path, &store, &labs).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&store).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(!progress::current_solved(
        &loaded.attempts[&current.id],
        current
    ));
}
