//! Authored capstone source, outcome, persistence and isolation checks.
use dlms_learning_lab::{
    engine::{self, State},
    linux_v6_startup as core, linux_v6_startup_source as binding,
    linux_v6_startup_workspace::{Action, Session},
    pack,
    progress::{self, Attempt, Progress},
};
fn source() -> pack::Scenario {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/linux-v6-startup-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0)
}
fn session(s: &mut State) -> &mut Session {
    let State::LinuxV6Startup { simulation } = s else {
        panic!()
    };
    simulation
}
fn act(state: &mut State, s: &pack::Scenario, a: Action) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!("{}{}", binding::PREFIX, serde_json::to_string(&a).unwrap()),
    )
    .unwrap();
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn authored_worked_and_independent_methods_reach_outcomes_without_cross_credit() {
    let s = source();
    // Validate the real dependency closure, retaining the isolated-pack rejection.
    assert!(
        pack::load(include_str!(
            "../scenarios/linux-v6-startup-lessons-v1.json"
        ))
        .unwrap_err()
        .contains("invalid prerequisites")
    );
    let before = pack::current().unwrap();
    let mut closure = vec![s.clone()];
    let mut at = 0;
    while at < closure.len() {
        for id in closure[at].prerequisites.clone() {
            if !closure.iter().any(|x| x.id == id) {
                closure.push(
                    before
                        .iter()
                        .find(|x| x.id == id)
                        .expect("real prerequisite")
                        .clone(),
                );
            }
        }
        at += 1;
    }
    let json = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: closure,
    })
    .unwrap();
    pack::load(&json).unwrap();
    let d = binding::source(&s).unwrap();
    let mut state = s.initial.clone();
    let initial = session(&mut state).assignment.clone();
    for a in &d.worked {
        session(&mut state).action(d, a, true).unwrap();
    }
    assert_eq!(session(&mut state).assignment, initial);
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        session(&mut state)
            .practice
            .checks(&d.example)
            .iter()
            .all(|c| c.passed)
    );
    solve(&s);
}
#[test]
fn initial_partial_live_only_disabled_unsaved_and_broken_saved_policy_cannot_pass() {
    let s = source();
    let mut state = s.initial.clone();
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    for c in s.solution.iter().take(5) {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    let mut state = solve(&s);
    act(
        &mut state,
        &s,
        Action::Command {
            input: format!("systemctl disable {}", core::UNIT),
        },
    );
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    let mut state = solve(&s);
    act(
        &mut state,
        &s,
        Action::EditFile {
            text: "# unfinished draft\n".into(),
        },
    );
    assert!(!engine::solved(&state, &s.goal));
    let mut state = solve(&s);
    let bad = session(&mut state)
        .assignment
        .editor
        .replace(":FORWARD DROP", ":FORWARD ACCEPT");
    act(&mut state, &s, Action::EditFile { text: bad });
    act(&mut state, &s, Action::SaveFile);
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    let rows = engine::checks(&state, &s.goal);
    assert!(
        rows.iter()
            .any(|(p, d)| !p && d.contains("Startup · FORWARD"))
    );
    assert!(
        rows.iter()
            .filter(|(_, d)| d.starts_with("Current · FORWARD"))
            .all(|(p, _)| *p)
    );
}
#[test]
fn malformed_pack_unreachable_worked_and_fake_history_are_refused() {
    let s = source();
    let mut v = serde_json::to_value(&s).unwrap();
    v["id"] = "wrong-source".into();
    let bad: pack::Scenario = serde_json::from_value(v).unwrap();
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    let engine::Goal::LinuxV6Startup { definition } = &mut bad.goal else {
        panic!()
    };
    definition.worked[0] = Action::Command {
        input: "bash -c true".into(),
    };
    assert!(binding::source(&bad).is_err());
    let mut state = solve(&s);
    session(&mut state).assignment.history.clear();
    assert!(!engine::solved(&state, &s.goal));
    assert!(engine::execute(&mut s.initial.clone(), "ip6tables -F").is_err());
}
#[test]
fn actual_saved_drafts_favorites_and_current_completion_resume_and_reset_safely() {
    let s = source();
    let all = vec![s.clone()];
    let d = binding::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    a.state = solve(&s);
    let sim = session(&mut a.state);
    sim.action(
        d,
        &Action::EditFile {
            text: "# assignment unsaved\n*filter\n".into(),
        },
        false,
    )
    .unwrap();
    sim.action(
        d,
        &Action::EditFile {
            text: "# example unsaved\n*filter\n".into(),
        },
        true,
    )
    .unwrap();
    sim.action(
        d,
        &Action::Input {
            text: format!("ip6tables-restore --test {}", core::FILE),
        },
        false,
    )
    .unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    binding::context(&loaded.attempts[&s.id], &s).unwrap();
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    let sim = session(&mut loaded.attempts.get_mut(&s.id).unwrap().state);
    let practice = sim.practice.clone();
    sim.reset(d, false).unwrap();
    assert_eq!(sim.practice, practice);
    assert_eq!(sim.assignment.world, d.assignment.initial);
}
#[test]
fn registered_catalog_preserves_all_493_scenarios_and_197_units() {
    use dlms_learning_lab::course;
    let old = pack::accepted493().unwrap();
    let now = pack::accepted494().unwrap();
    assert_eq!((old.len(), now.len()), (493, 494));
    for s in &old {
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(now.iter().find(|n| n.id == s.id).unwrap()).unwrap(),
            "{}",
            s.id
        );
    }
    let before = course::accepted493(&old).unwrap();
    let after = course::accepted494(&now).unwrap();
    assert_eq!((before.len(), after.len()), (197, 198));
    for u in before {
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(after.iter().find(|n| n.id == u.id).unwrap()).unwrap()
        );
    }
    let unit = after
        .iter()
        .find(|u| u.id == "linux-ipv6-firewall-startup")
        .unwrap();
    assert_eq!(unit.labs, [source().id]);
    for id in &unit.prerequisites {
        assert!(
            after.iter().position(|u| &u.id == id).unwrap()
                < after.iter().position(|u| u.id == unit.id).unwrap()
        );
    }
    for (track, n) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 147),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
    }
}
