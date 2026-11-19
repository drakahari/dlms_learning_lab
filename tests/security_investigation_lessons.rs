use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    security_investigation::{
        session::Session,
        source::{self, PREFIX},
        workspace::{Action, Scope, Task, Work},
    },
};

const UNIT: &str = "ccna-security-investigation";
const IDS: [&str; 2] = [
    "ccna-management-change-evidence",
    "ccna-facility-access-plan",
];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/security-investigation-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
}
fn session(state: &mut State) -> &mut Session {
    let State::SecurityInvestigation { simulation } = state else {
        panic!("Expected investigation state")
    };
    simulation
}
fn act(state: &mut State, lab: &pack::Scenario, action: &Action) {
    let input = format!("{PREFIX}{}", serde_json::to_string(action).unwrap());
    engine::execute_for_goal(state, &lab.goal, &input).unwrap();
}

#[test]
fn investigation_unit_preserves_all_526_labs_and_213_units() {
    let previous = pack::accepted526().unwrap();
    let current = pack::accepted528().unwrap();
    assert_eq!((previous.len(), current.len()), (526, 528));
    assert_eq!(
        serde_json::to_value(&current[..previous.len()]).unwrap(),
        serde_json::to_value(&previous).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&current[previous.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let base = course::accepted526(&previous).unwrap();
    let units = course::accepted528(&current).unwrap();
    assert_eq!((base.len(), units.len()), (213, 214));
    assert_eq!(
        serde_json::to_value(units.iter().filter(|u| u.id != UNIT).collect::<Vec<_>>()).unwrap(),
        serde_json::to_value(&base).unwrap()
    );
    let position = units.iter().position(|u| u.id == UNIT).unwrap();
    assert_eq!(units[position].labs, IDS);
    assert_eq!(
        units[position].prerequisites,
        ["management-policy-foundations", "ccna-syslog-policy"]
    );
    assert!(
        units[position]
            .prerequisites
            .iter()
            .all(|p| units[..position].iter().any(|u| &u.id == p))
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [117, 157, 174, 80]
    );
}

#[test]
fn authored_references_only_pass_after_check_and_do_not_survive_breakage() {
    for lab in authored() {
        let definition = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
        for input in &lab.solution[..lab.solution.len() - 1] {
            engine::execute_for_goal(&mut state, &lab.goal, input).unwrap();
        }
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Check);
        assert!(engine::solved(&state, &lab.goal));
        assert_eq!(
            session(&mut state).example.work,
            Work::new(&definition.example)
        );
        let broken = match &definition.assignment {
            Task::Change { supplied } => Action::Draft {
                contents: supplied.starter.clone(),
            },
            Task::Facility { supplied } => Action::Facility {
                plan: supplied.starter,
            },
            _ => panic!("This historical fixture retains only the two investigation activities"),
        };
        act(&mut state, &lab, &broken);
        assert!(!engine::solved(&state, &lab.goal));
        act(&mut state, &lab, &Action::Check);
        assert!(!engine::solved(&state, &lab.goal));
    }
}

#[test]
fn restored_attempt_retains_separate_evidence_drafts_without_credit() {
    let labs = authored();
    let lab = &labs[0];
    let d = source::source(lab).unwrap();
    let mut attempt = Attempt::new(lab, 1);
    engine::execute_for_goal(&mut attempt.state, &lab.goal, &lab.solution[0]).unwrap();
    act(
        &mut attempt.state,
        lab,
        &Action::Draft {
            contents: "! pending repair".into(),
        },
    );
    let native = session(&mut attempt.state);
    native
        .example
        .work
        .apply(
            &d.example,
            Action::Draft {
                contents: "! different practice draft".into(),
            },
        )
        .unwrap();
    let original_example = native.example.clone();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.attempts.insert(lab.id.clone(), attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let attempt = restored.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(attempt, lab));
    let Scope::Change {
        ref draft,
        ref applied,
        ref report,
    } = session(&mut attempt.state).assignment.work.current
    else {
        panic!()
    };
    assert_ne!(draft, applied);
    assert!(report.approval.is_some());
    act(&mut attempt.state, lab, &Action::Reset);
    assert_eq!(
        session(&mut attempt.state).assignment.work,
        Work::new(&d.assignment)
    );
    assert_eq!(session(&mut attempt.state).example, original_example);
}
