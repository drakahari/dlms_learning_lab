//! Source-bound task fixtures for the next unit; not registered curriculum yet.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    ftp_identity_source as binding,
    ftp_identity_workspace::{Action, Definition, Session},
    pack,
    progress::{self, Attempt},
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Fixture {
    definition: Definition,
    assignment_solution: Vec<Action>,
}
fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("fixtures/ftp-identity-tasks-v1.json")).unwrap()
}
fn scenario(f: &Fixture) -> pack::Scenario {
    serde_json::from_value(serde_json::json!({"id":f.definition.source_id,"version":1,"track":"LPIC-2","unit":"ftp-fixture-unit","topic":"FTP identity fixture","title":"FTP source binding fixture","summary":"Unregistered deterministic source fixture.","objective":"Exercise the supplied identity, confinement and transfer requirements.","difficulty":"Intermediate","minutes":20,"steps":["Inspect and repair the separate practice fixture."],"guide":[{"title":"Inspect","why":"Read saved configuration before editing.","action":"Run cat /etc/vsftpd.conf in the administrative terminal.","commands":[{"command":"cat /etc/vsftpd.conf","explanation":"Read the saved virtual file.","simulator_only":false}]}],"hints":["Login, confinement and file access are separate gates."],"explanation":"Use the supported outcomes, rather than a particular configuration string.","sources":["https://security.appspot.com/vsftpd/vsftpd_conf.html"],"simplifications":"In-process regular files and supplied credential proofs only; no daemon or packets.","initial":State::FtpIdentity{simulation:Box::new(Session::new(&f.definition).unwrap())},"goal":Goal::FtpIdentity{definition:Box::new(f.definition.clone())},"solution":f.assignment_solution.iter().map(|a|format!("{}{}",binding::PREFIX,serde_json::to_string(a).unwrap())).collect::<Vec<_>>()})).unwrap()
}
fn session(s: &mut State) -> &mut Session {
    let State::FtpIdentity { simulation } = s else {
        panic!("FTP fixture")
    };
    simulation
}
fn solve(f: &Fixture) -> (pack::Scenario, State) {
    let s = scenario(f);
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    (s, state)
}
#[test]
fn three_distinct_tasks_and_their_worked_examples_and_assignments_are_reachable() {
    for f in fixtures() {
        f.definition.validate().unwrap();
        let s = scenario(&f);
        binding::source(&s).unwrap();
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![s.clone()],
            })
            .unwrap(),
        )
        .unwrap();
        let mut state = s.initial.clone();
        let before = session(&mut state).assignment.clone();
        for a in &f.definition.worked {
            session(&mut state).action(&f.definition, a, true).unwrap();
        }
        assert_eq!(session(&mut state).assignment, before);
        assert!(!engine::solved(&state, &s.goal));
        solve(&f);
    }
}
#[test]
fn source_mismatch_malformed_guide_and_unsupported_reference_fail_honestly() {
    let f = fixtures().remove(0);
    let mut s = scenario(&f);
    s.id = "wrong-source".into();
    assert!(binding::source(&s).is_err());
    let mut s = scenario(&f);
    s.guide[0].commands[0].command = "real-host-program --do-work".into();
    assert!(binding::source(&s).is_err());
    let s = scenario(&f);
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    assert!(engine::execute(&mut state, "systemctl restart vsftpd").is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    assert!(engine::execute_for_goal(
        &mut state,
        &s.goal,
        "ftp-identity-workspace {\"kind\":\"host_exec\"}"
    )
    .is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}
#[test]
fn progress_source_binding_preserves_unfinished_work_and_clears_broken_success() {
    for f in fixtures() {
        let (s, mut state) = solve(&f);
        let mut a = Attempt::new(&s, 0);
        a.state = state.clone();
        binding::context(&a, &s).unwrap();
        assert!(progress::current_solved(&a, &s));
        session(&mut state)
            .action(
                &f.definition,
                &Action::Edit {
                    text: "# unfinished independent draft\n".into(),
                },
                false,
            )
            .unwrap();
        a.state = state;
        assert!(!progress::current_solved(&a, &s));
        let raw = serde_json::to_string(&a).unwrap();
        let restored: Attempt = serde_json::from_str(&raw).unwrap();
        binding::context(&restored, &s).unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        let mut wrong = restored.clone();
        session(&mut wrong.state).source_id = "another-source".into();
        assert!(binding::context(&wrong, &s).is_err());
    }
}
#[test]
fn every_authored_task_rejects_a_writable_jail_root() {
    for f in fixtures() {
        let (s, mut state) = solve(&f);
        let home = f.definition.assignment.initial.accounts[0].home.clone();
        session(&mut state)
            .action(
                &f.definition,
                &Action::Event {
                    event: dlms_learning_lab::ftp_identity_workspace::Event::Admin {
                        input: format!("chmod 777 {home}"),
                    },
                },
                false,
            )
            .unwrap();
        session(&mut state)
            .action(&f.definition, &Action::Check, false)
            .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        assert!(engine::checks(&state, &s.goal)
            .iter()
            .any(|(p, m)| !p && m.contains("writable")));
    }
}
