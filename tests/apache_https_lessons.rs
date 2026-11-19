use dlms_learning_lab::{
    apache_https::{
        source::{self, PREFIX},
        workspace::{Action, Work},
    },
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};

const UNIT: &str = "linux-apache-https-identity";

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/apache-https-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn act(state: &mut State, scenario: &pack::Scenario, action: &Action) {
    let input = format!("{PREFIX}{}", serde_json::to_string(action).unwrap());
    engine::execute_for_goal(state, &scenario.goal, &input).unwrap();
}

fn workspace(state: &mut State) -> &mut dlms_learning_lab::apache_https::session::Session {
    let State::ApacheHttps { simulation } = state else {
        panic!("Expected HTTPS work");
    };
    simulation
}

#[test]
fn exact_preservation_for_521_labs_and_211_units() {
    let previous = pack::accepted521().unwrap();
    let current = pack::accepted523().unwrap();
    assert_eq!((previous.len(), current.len()), (521, 523));
    assert_eq!(
        serde_json::to_value(&current[..previous.len()]).unwrap(),
        serde_json::to_value(&previous).unwrap(),
    );
    assert_eq!(
        serde_json::to_value(&current[previous.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap(),
    );
    let baseline = course::accepted521(&previous).unwrap();
    let current_units = course::accepted523(&current).unwrap();
    assert_eq!((baseline.len(), current_units.len()), (211, 212));
    assert_eq!(
        serde_json::to_value(
            current_units
                .iter()
                .filter(|u| u.id != UNIT)
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&baseline).unwrap(),
    );
    let position = current_units.iter().position(|u| u.id == UNIT).unwrap();
    let unit = &current_units[position];
    assert_eq!(
        unit.labs,
        ["lpic2-apache-https-chain", "lpic2-apache-https-sni"]
    );
    assert_eq!(unit.prerequisites, ["static-web-services"]);
    assert!(
        current_units[..position]
            .iter()
            .any(|u| u.id == unit.prerequisites[0])
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|s| s.track == track).count()),
        [115, 154, 174, 80],
    );
}

#[test]
fn shipped_solutions_need_save_apply_and_check_and_breakage_clears_success() {
    for lab in authored() {
        let d = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
        let example = workspace(&mut state).example.clone();
        for input in &lab.solution[..lab.solution.len() - 1] {
            engine::execute_for_goal(&mut state, &lab.goal, input).unwrap();
        }
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Check);
        assert!(engine::solved(&state, &lab.goal));
        assert_eq!(workspace(&mut state).example, example);
        act(
            &mut state,
            &lab,
            &Action::Draft {
                contents: d.assignment.starter.clone(),
            },
        );
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Save);
        act(&mut state, &lab, &Action::Apply);
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
    }
}

#[test]
fn progress_restores_edited_saved_applied_and_example_work_separately() {
    let labs = authored();
    let lab = &labs[0];
    let definition = source::source(lab).unwrap();
    let mut attempt = Attempt::new(lab, 1);
    let examples: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/apache_https_tasks_v1.json")).unwrap();
    let example_answer = examples[&lab.id]["answers"]["example"].as_str().unwrap();
    let instance = workspace(&mut attempt.state);
    for action in [
        Action::Draft {
            contents: example_answer.into(),
        },
        Action::Save,
        Action::Apply,
        Action::Check,
    ] {
        instance
            .example
            .work
            .apply(&definition.example, action)
            .unwrap();
    }
    assert!(
        instance
            .example
            .work
            .checks(&definition.example)
            .iter()
            .all(|c| c.passed)
    );
    let practice = instance.example.clone();
    assert!(!engine::solved(&attempt.state, &lab.goal));
    for input in &lab.solution[..2] {
        engine::execute_for_goal(&mut attempt.state, &lab.goal, input).unwrap();
    }
    act(
        &mut attempt.state,
        lab,
        &Action::Draft {
            contents: "# unfinished".into(),
        },
    );
    let staged = &workspace(&mut attempt.state).assignment.work;
    assert_ne!(staged.draft, staged.saved);
    assert_ne!(staged.saved, staged.applied);
    assert_ne!(staged.draft, staged.applied);
    let mut progress = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    progress.attempts.insert(lab.id.clone(), attempt);
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("progress.json");
    progress::save_with_catalog(&path, &progress, &labs).unwrap();
    let mut loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&progress).unwrap()
    );
    let reopened = loaded.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(reopened, lab));
    act(&mut reopened.state, lab, &Action::Reset);
    let session = workspace(&mut reopened.state);
    assert_eq!(session.assignment.work, Work::new(&definition.assignment));
    assert_eq!(session.example, practice);
    assert!(progress::save(&path, &loaded).is_err());
}
