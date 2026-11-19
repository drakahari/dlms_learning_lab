use dlms_learning_lab::{
    course,
    desktop::{
        session::Session,
        source::{self, PREFIX},
        workspace::{Action, Task, Work},
    },
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};

const UNIT: &str = "linux-graphical-workstations";
const IDS: [&str; 3] = [
    "lpic1-xorg-keyboard-scope",
    "lpic1-remote-desktop-arrangement",
    "lpic1-accessible-input-profile",
];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/desktop-profile-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn session(state: &mut State) -> &mut Session {
    let State::Desktop { simulation } = state else {
        panic!("Expected virtual desktop state")
    };
    simulation
}

fn act(state: &mut State, lab: &pack::Scenario, op: &Action) {
    let command = format!("{PREFIX}{}", serde_json::to_string(op).unwrap());
    engine::execute_for_goal(state, &lab.goal, &command).unwrap();
}

#[test]
fn three_desktop_tasks_extend_exact_prior_catalog_and_units() {
    let before = pack::accepted523().unwrap();
    let now = pack::accepted526().unwrap();
    assert_eq!((before.len(), now.len()), (523, 526));
    assert_eq!(
        serde_json::to_value(&now[..before.len()]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&now[before.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let base_units = course::accepted523(&before).unwrap();
    let units = course::accepted526(&now).unwrap();
    assert_eq!((base_units.len(), units.len()), (212, 213));
    assert_eq!(
        serde_json::to_value(units.iter().filter(|u| u.id != UNIT).collect::<Vec<_>>()).unwrap(),
        serde_json::to_value(&base_units).unwrap()
    );
    let position = units.iter().position(|u| u.id == UNIT).unwrap();
    assert_eq!(units[position].labs, IDS);
    assert_eq!(units[position].prerequisites, ["linux-file-foundations"]);
    assert!(
        units[..position]
            .iter()
            .any(|u| u.id == "linux-file-foundations")
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|name| now.iter().filter(|s| s.track == name).count()),
        [115, 157, 174, 80]
    );
}

#[test]
fn provided_references_need_correct_results_and_edits_clear_success() {
    for lab in authored() {
        let definition = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal), "{:?}", lab.id);
        for line in &lab.solution[..lab.solution.len() - 1] {
            engine::execute_for_goal(&mut state, &lab.goal, line).unwrap();
        }
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Check);
        assert!(engine::solved(&state, &lab.goal));
        assert_eq!(
            session(&mut state).example.work,
            Work::new(&definition.example)
        );
        let edit = match &definition.assignment {
            Task::Keyboard { starter, .. } => Action::EditKeyboard {
                contents: starter.clone(),
            },
            Task::Remote { starter, .. } => Action::SetRemote { draft: *starter },
            Task::Typing { starter, .. } => Action::SetTyping { current: *starter },
        };
        act(&mut state, &lab, &edit);
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Reset);
        assert!(!engine::solved(&state, &lab.goal));
        assert_eq!(
            session(&mut state).assignment.work,
            Work::new(&definition.assignment)
        );
    }
}

#[test]
fn progress_restores_a_pending_keyboard_edit_without_applying_it() {
    let labs = authored();
    let lab = &labs[0];
    let d = source::source(lab).unwrap();
    let mut attempt = Attempt::new(lab, 1);
    let mut expected = lab.initial.clone();
    let pending = Action::EditKeyboard {
        contents: "# unfinished".into(),
    };
    act(&mut attempt.state, lab, &pending);
    let original = Work::new(&d.assignment);
    assert_ne!(session(&mut attempt.state).assignment.work, original);
    let isolated = Action::EditKeyboard {
        contents: "# separate sample edit".into(),
    };
    session(&mut attempt.state)
        .example
        .work
        .apply(&d.example, isolated.clone())
        .unwrap();
    session(&mut expected)
        .example
        .work
        .apply(&d.example, isolated)
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.attempts.insert(lab.id.clone(), attempt);
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let mut loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let a = loaded.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(a, lab));
    act(&mut a.state, lab, &Action::Reset);
    assert_eq!(
        serde_json::to_value(&a.state).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
}
