//! One authored listener investigation, using only the supplied process model.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack, process_teaching,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted285().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog()
        .iter()
        .find(|s| s.id == "lpic1-listener-audit")
        .unwrap()
}
fn execute(state: &mut State, commands: &[&str]) {
    for c in commands {
        engine::execute(state, c).unwrap_or_else(|e| panic!("{c}: {e}"));
    }
}
fn solved() -> State {
    let s = lab();
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
#[test]
fn listener_catalog_retains_every284_scenario87_unit_and_prerequisite_order() {
    let old = pack::accepted284().unwrap();
    assert_eq!((old.len(), catalog().len()), (284, 285));
    assert_eq!(
        serde_json::to_value(old.as_slice()).unwrap(),
        serde_json::to_value(&catalog()[..284]).unwrap()
    );
    let before = course::accepted284(&old).unwrap();
    let now = course::accepted285(catalog()).unwrap();
    assert_eq!((before.len(), now.len()), (87, 88));
    let retained: Vec<_> = now
        .iter()
        .filter(|u| u.id != "linux-listener-audit")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = now
        .iter()
        .position(|u| u.id == "linux-listener-audit")
        .unwrap();
    for p in &now[at].prerequisites {
        assert!(now.iter().position(|u| &u.id == p).unwrap() < at)
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| catalog().iter().filter(|s| s.track == t).count()),
        [72, 94, 45, 74]
    );
}
#[test]
fn listener_taught_solution_and_legitimate_signal_spellings_grade_current_outcomes() {
    let s = lab();
    assert!(!engine::solved(&s.initial, &s.goal));
    solved();
    for cmds in [
        vec!["kill -INT 410"],
        vec!["kill -9 410"],
        vec!["kill -s SIGTERM -- 410"],
        vec!["kill -STOP 410", "kill -TERM 410", "kill -CONT 410"],
    ] {
        let mut state = s.initial.clone();
        execute(&mut state, &cmds);
        assert!(engine::solved(&state, &s.goal), "{cmds:?}");
    }
}
#[test]
fn listener_stop_and_pending_term_remain_partial_and_explain_why() {
    let s = lab();
    let mut state = s.initial.clone();
    execute(&mut state, &["kill -STOP 410"]);
    assert!(!engine::solved(&state, &s.goal));
    let issues = workspace::issues(&state, &s.goal).join("\n");
    assert!(
        issues.contains("paused") && issues.contains("still bound"),
        "{issues}"
    );
    execute(&mut state, &["kill -TERM 410"]);
    let issues = workspace::issues(&state, &s.goal).join("\n");
    assert!(issues.contains("termination request waiting"), "{issues}");
    assert!(
        engine::execute(&mut state, "ss -lntp")
            .unwrap()
            .contains("0.0.0.0:8081")
    );
    let checks = engine::checks(&state, &s.goal);
    assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
    execute(&mut state, &["kill -CONT 410"]);
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn listener_killing_every_matching_name_or_pausing_a_protected_owner_cannot_pass() {
    let s = lab();
    let mut state = solved();
    execute(&mut state, &["kill -TERM 411"]);
    assert!(!engine::solved(&state, &s.goal));
    let mut state = solved();
    execute(&mut state, &["kill -STOP 411"]);
    assert!(!engine::solved(&state, &s.goal));
    assert!(workspace::issues(&state, &s.goal).join("\n").contains("owner is paused"));
    assert!(
        engine::execute(&mut state, "ss -lntp")
            .unwrap()
            .contains("127.0.0.1:8082")
    );
    execute(&mut state, &["kill -CONT 411"]);
    assert!(engine::solved(&state, &s.goal));
    let mut state = s.initial.clone();
    execute(&mut state, &["kill -TERM 410 411"]);
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn listener_unsupported_host_composition_and_foreign_owner_do_not_mutate_state() {
    let s = lab();
    for c in [
        "ss -lntup | sh",
        "ss -lntup > /etc/passwd",
        "ss -K",
        "netstat -lntup",
        "nmap 127.0.0.1",
    ] {
        let mut state = s.initial.clone();
        assert!(engine::execute(&mut state, c).is_err());
        assert_eq!(state, s.initial);
    }
    let State::Processes { table } = &s.initial else {
        panic!()
    };
    let foreign = table
        .listeners
        .iter()
        .find(|l| table.processes[&l.pid].owner.uid != table.user.uid)
        .unwrap()
        .pid;
    let mut state = s.initial.clone();
    assert!(engine::execute(&mut state, &format!("kill -KILL {foreign}")).is_err());
    assert_eq!(state, s.initial);
}
#[test]
fn listener_separate_eight_command_example_teaches_a_reversible_protected_pause() {
    let s = lab();
    let l = s.process_teaching.as_ref().unwrap();
    l.validate().unwrap();
    assert_eq!(l.example.len(), 8);
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    process_teaching::prepare(&mut a, l, true);
    let original = a.state.clone();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .process_learning
        .as_mut()
        .unwrap();
    for c in &l.example {
        learn.command = c.clone();
        learn.execute();
    }
    assert!(l.outcomes.iter().all(|r| r.check(&learn.table).0));
    learn.command = "kill -STOP 6020".into();
    learn.execute();
    assert!(l.outcomes.iter().any(|r| !r.check(&learn.table).0));
    assert!(
        learn
            .table
            .execute("ss -lntp")
            .unwrap()
            .stdout
            .contains("127.0.0.1:9091")
    );
    learn.command = "kill -CONT 6020".into();
    learn.execute();
    assert!(l.outcomes.iter().all(|r| r.check(&learn.table).0));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn listener_both_drafts_current_invalidation_reset_and_resume_preserve_unrelated_work() {
    let s = lab();
    let l = s.process_teaching.as_ref().unwrap();
    let other = &catalog()[0];
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(other.id.clone(), Attempt::new(other, 1));
    p.favorites.insert(other.id.clone());
    let keep = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let mut a = Attempt::new(s, 2);
    workspace::prepare(&mut a);
    process_teaching::prepare(&mut a, l, true);
    a.state = solved();
    a.workspace.as_mut().unwrap().command = "ps -p unfinished".into();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .process_learning
        .as_mut()
        .unwrap();
    learn.command = "ss -ln unfinished".into();
    learn.view = dlms_learning_lab::script_teaching::View::Example;
    assert!(progress::current_solved(&a, s));
    p.attempts.insert(s.id.clone(), a);
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    let a = q.attempts.get_mut(&s.id).unwrap();
    execute(&mut a.state, &["kill -TERM 411"]);
    assert!(!progress::current_solved(a, s));
    progress::save_with_catalog(&path, &q, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert!(!progress::current_solved(&q.attempts[&s.id], s));
    let a = q.attempts.get_mut(&s.id).unwrap();
    *a = Attempt::new(s, 2);
    workspace::prepare(a);
    process_teaching::prepare(a, l, true);
    assert_eq!(a.state, s.initial);
    assert_eq!(serde_json::to_value(&q.attempts[&other.id]).unwrap(), keep);
    assert!(q.favorites.contains(&other.id));
}
#[test]
fn listener_no_assignment_pid_or_solution_is_taught_as_default_task() {
    let s = lab();
    assert!(!s.objective.contains("410"));
    assert!(!s.objective.contains("kill -"));
    assert!(s.objective.contains("0.0.0.0:8081"));
    assert_ne!(
        s.initial,
        s.process_teaching
            .as_ref()
            .map(|l| State::Processes {
                table: l.fixture.clone()
            })
            .unwrap()
    );
    assert!(s.simplifications.contains("supporting only"));
    assert!(s.simplifications.contains("nmap/netstat"));
}
