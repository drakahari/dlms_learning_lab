use dlms_learning_lab::{
    course, engine, pack, progress,
    security_investigation::{
        session::Session,
        source::{self, PREFIX},
        workspace::{Action, Scope, Task, Work},
    },
};

const UNIT: &str = "ccna-credentials-and-vpn";
const IDS: [&str; 2] = ["ccna-credential-policy-repair", "ccna-ipsec-design-review"];

fn authored() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/credentials-vpn-lessons-v1.json"))
        .unwrap()
        .scenarios
}

fn session(state: &mut engine::State) -> &mut Session {
    let engine::State::SecurityInvestigation { simulation } = state else {
        panic!("Expected bound security state")
    };
    simulation
}

fn execute(
    state: &mut engine::State,
    lab: &pack::Scenario,
    action: Action,
) -> Result<String, String> {
    engine::execute_for_goal(
        state,
        &lab.goal,
        &format!("{PREFIX}{}", serde_json::to_string(&action).unwrap()),
    )
}

#[test]
fn two_policy_tasks_extend_the_exact_preceding_catalog_in_order() {
    let prior = pack::accepted528().unwrap();
    let current = pack::accepted530().unwrap();
    assert_eq!((prior.len(), current.len()), (528, 530));
    assert_eq!(
        serde_json::to_value(&current[..prior.len()]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&current[prior.len()..]).unwrap(),
        serde_json::to_value(authored()).unwrap()
    );
    let base = course::accepted528(&prior).unwrap();
    let units = course::accepted530(&current).unwrap();
    assert_eq!((base.len(), units.len()), (214, 215));
    assert_eq!(
        serde_json::to_value(units.iter().filter(|u| u.id != UNIT).collect::<Vec<_>>()).unwrap(),
        serde_json::to_value(&base).unwrap()
    );
    let pos = units.iter().position(|u| u.id == UNIT).unwrap();
    assert_eq!(units[pos].labs, IDS);
    assert_eq!(
        units[pos].prerequisites,
        ["aaa-decisions", "ccna-security-investigation"]
    );
    assert!(
        units[pos]
            .prerequisites
            .iter()
            .all(|name| units[..pos].iter().any(|u| &u.id == name))
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|lab| lab.track == track).count()),
        [119, 157, 174, 80]
    );
}

#[test]
fn canonical_actions_pass_only_when_checked_and_never_award_example_credit() {
    for lab in authored() {
        let d = source::source(&lab).unwrap();
        let mut state = lab.initial.clone();
        assert!(!engine::solved(&state, &lab.goal));
        execute(&mut state, &lab, Action::Check).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
        for action in &lab.solution[..lab.solution.len() - 1] {
            engine::execute_for_goal(&mut state, &lab.goal, action).unwrap();
        }
        assert!(!engine::solved(&state, &lab.goal));
        execute(&mut state, &lab, Action::Check).unwrap();
        assert!(engine::solved(&state, &lab.goal));
        assert_eq!(session(&mut state).example.work, Work::new(&d.example));
        let broken = match &d.assignment {
            Task::Credentials { supplied } => Action::Credentials {
                policy: supplied.starter,
            },
            Task::VpnDesign { supplied } => Action::VpnDrafts {
                drafts: supplied.starter.map(
                    dlms_learning_lab::security_investigation::vpn_design_task::Draft::from_profile,
                ),
            },
            _ => panic!("Unexpected registered exercise"),
        };
        execute(&mut state, &lab, broken).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
        execute(&mut state, &lab, Action::Check).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
    }
}

#[test]
fn malformed_vpn_draft_persists_without_changing_applied_or_example() {
    let labs = authored();
    let lab = &labs[1];
    let d = source::source(lab).unwrap();
    let mut attempt = progress::Attempt::new(lab, 1);
    let native = session(&mut attempt.state);
    let Scope::VpnDesign { drafts, applied } = &native.assignment.work.current else {
        panic!();
    };
    let applied = *applied;
    let mut drafts = drafts.clone();
    drafts[1].right_traffic = "pending input".into();
    native
        .assignment
        .work
        .apply(
            &d.assignment,
            Action::VpnDrafts {
                drafts: drafts.clone(),
            },
        )
        .unwrap();
    let unchanged = native.assignment.clone();
    assert!(
        native
            .assignment
            .work
            .apply(&d.assignment, Action::ApplyVpn)
            .is_err()
    );
    assert_eq!(native.assignment, unchanged);
    native
        .example
        .work
        .apply(&d.example, Action::Check)
        .unwrap();
    let p = progress::Progress {
        schema_version: 1,
        attempts: [(lab.id.clone(), attempt)].into(),
        ..progress::Progress::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let a = restored.attempts.get_mut(&lab.id).unwrap();
    assert!(!progress::current_solved(a, lab));
    let saved = session(&mut a.state);
    assert_eq!(
        saved.assignment.work.current,
        Scope::VpnDesign { drafts, applied }
    );
    assert!(!saved.assignment.work.checked);
    assert!(saved.example.work.checked);
}
