//! Authored IPv6 configuration tasks, not placeholder or command-string grades.
use dlms_learning_lab::{
    engine::{self, State},
    linux_v6_admin_source as binding,
    linux_v6_admin_workspace::{Action, Session},
    pack,
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/linux-v6-admin-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn session(state: &mut State) -> &mut Session {
    let State::LinuxV6Administration { simulation } = state else {
        panic!("new native administration state required")
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
fn command(state: &mut State, s: &pack::Scenario, text: &str) {
    act(state, s, Action::Command { input: text.into() });
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn authored_sources_validate_and_each_worked_example_is_separate() {
    let all = sources();
    pack::load(include_str!("../scenarios/linux-v6-admin-lessons-v1.json")).unwrap();
    assert_eq!(all.len(), 3);
    for s in all {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let original = session(&mut state).assignment.clone();
        for a in &d.worked {
            session(&mut state).apply(d, a, true).unwrap();
        }
        assert_eq!(session(&mut state).assignment, original);
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
    }
}
#[test]
fn all_assignments_reject_initial_partial_and_broken_current_state() {
    for s in sources() {
        let mut state = s.initial.clone();
        act(&mut state, &s, Action::Check);
        assert!(!engine::solved(&state, &s.goal));
        let first = &s.solution[0];
        engine::execute_for_goal(&mut state, &s.goal, first).unwrap();
        act(&mut state, &s, Action::Check);
        assert!(!engine::solved(&state, &s.goal));
        let mut state = solve(&s);
        command(&mut state, &s, "sysctl -w net.ipv6.conf.all.forwarding=0");
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, &s, Action::Check);
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn alternate_unused_address_split_prefixes_and_metrics_pass_but_sample_only_route_fails() {
    let s = sources().remove(0);
    let mut state = s.initial.clone();
    for c in s.solution.iter().take(2) {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    command(
        &mut state,
        &s,
        "ip -6 address add fd62:70:80::abcd/64 dev lan0",
    );
    for p in ["2001:db8:99::/65", "2001:db8:99:0:8000::/65"] {
        command(
            &mut state,
            &s,
            &format!("ip -6 route add {p} via fe80::1 dev wan0 metric 20"),
        );
    }
    act(&mut state, &s, Action::Check);
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    let mut state = solve(&s);
    command(
        &mut state,
        &s,
        "ip -6 route del 2001:db8:99::/64 via fe80::1 dev wan0 metric 10",
    );
    for ip in ["80", "81"] {
        command(
            &mut state,
            &s,
            &format!("ip -6 route add 2001:db8:99::{ip}/128 via fe80::1 dev wan0 metric 10"),
        );
    }
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        session(&mut state)
            .assignment
            .checks(&binding::source(&s).unwrap().assignment)
            .iter()
            .any(|c| c.label == "Complete route plan" && !c.passed)
    );
}
#[test]
fn settings_alone_do_not_fabricate_an_ra_and_withdrawal_requires_new_observation() {
    let s = sources().remove(1);
    let mut state = s.initial.clone();
    for c in s.solution.iter().take(3) {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    act(&mut state, &s, Action::Observe { message: 0 });
    act(&mut state, &s, Action::Check);
    assert!(engine::solved(&state, &s.goal));
    act(&mut state, &s, Action::Observe { message: 2 });
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    act(&mut state, &s, Action::Observe { message: 0 });
    act(&mut state, &s, Action::Check);
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn saved_file_is_behavior_graded_and_live_only_unsaved_and_wrong_final_duplicate_fail() {
    let s = sources().remove(2);
    let mut state = solve(&s);
    let good = "# alternate formatting\nnet.ipv6.conf.all.forwarding=0\nnet.ipv6.conf.all.forwarding = 1\n; explicit interfaces\nnet.ipv6.conf.lan0.accept_ra=0\nnet.ipv6.conf.wan0.accept_ra=2\n";
    act(&mut state, &s, Action::EditFile { text: good.into() });
    act(&mut state, &s, Action::Check);
    assert!(!engine::solved(&state, &s.goal));
    act(&mut state, &s, Action::SaveFile);
    act(&mut state, &s, Action::RunRestart);
    act(&mut state, &s, Action::Check);
    assert!(engine::solved(&state, &s.goal));
    act(
        &mut state,
        &s,
        Action::EditFile {
            text: format!("{good}net.ipv6.conf.all.forwarding=0\n"),
        },
    );
    act(&mut state, &s, Action::SaveFile);
    act(&mut state, &s, Action::Check);
    assert!(
        !engine::solved(&state, &s.goal),
        "current success cannot replace restart outcome"
    );
}
#[test]
fn reset_resume_and_unsupported_input_preserve_isolation() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let mut state = solve(&s);
        act(
            &mut state,
            &s,
            Action::Input {
                text: "ip -6 route show".into(),
            },
        );
        let saved = serde_json::to_vec(&state).unwrap();
        let mut restored: State = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored, state);
        d.bound(session(&mut restored)).unwrap();
        let world = session(&mut restored).assignment.world.clone();
        session(&mut restored)
            .apply(
                d,
                &Action::Command {
                    input: "bash -c 'touch /tmp/learner'".into(),
                },
                false,
            )
            .unwrap();
        assert_eq!(session(&mut restored).assignment.world, world);
        assert!(
            session(&mut restored)
                .assignment
                .output
                .contains("[Not applied]")
        );
        let practice = session(&mut restored).practice.clone();
        session(&mut restored).reset(d, false).unwrap();
        assert_eq!(session(&mut restored).practice, practice);
        assert!(!d.solved(session(&mut restored)));
    }
}
#[test]
fn current_catalog_preserves_all_490_prior_scenarios_and_196_units() {
    use dlms_learning_lab::course;
    let old = pack::accepted490().unwrap();
    let now = pack::accepted493().unwrap();
    assert_eq!((old.len(), now.len()), (490, 493));
    for s in &old {
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(now.iter().find(|n| n.id == s.id).unwrap()).unwrap(),
            "{}",
            s.id
        );
    }
    let before = course::accepted490(&old).unwrap();
    let after = course::accepted493(&now).unwrap();
    assert_eq!((before.len(), after.len()), (196, 197));
    for u in before {
        assert_eq!(
            serde_json::to_value(&u).unwrap(),
            serde_json::to_value(after.iter().find(|n| n.id == u.id).unwrap()).unwrap()
        );
    }
    let unit = after
        .iter()
        .find(|u| u.id == "linux-ipv6-router-configuration")
        .unwrap();
    assert_eq!(
        unit.labs,
        sources().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    for prereq in &unit.prerequisites {
        assert!(
            after.iter().position(|u| &u.id == prereq).unwrap()
                < after.iter().position(|u| u.id == unit.id).unwrap()
        );
    }
}
#[test]
fn actual_authored_assignment_example_drafts_and_favorites_resume_from_safe_disk_storage() {
    use dlms_learning_lab::progress::{self, Attempt, Progress};
    let all = sources();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        let d = binding::source(s).unwrap();
        let sim = session(&mut a.state);
        sim.apply(
            d,
            &Action::Input {
                text: "ip -6 route show".into(),
            },
            false,
        )
        .unwrap();
        sim.apply(
            d,
            &Action::Input {
                text: "sysctl net.ipv6.conf.all.forwarding".into(),
            },
            true,
        )
        .unwrap();
        if d.assignment.file.is_some() {
            sim.apply(
                d,
                &Action::EditFile {
                    text: "# Unfinished independent draft\n".into(),
                },
                false,
            )
            .unwrap();
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("progress-v1.json");
    assert!(progress::save(&file, &p).is_err());
    progress::save_with_catalog(&file, &p, &all).unwrap();
    let loaded = progress::load(&file, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for s in &all {
        binding::context(&loaded.attempts[&s.id], s).unwrap();
        assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    }
}
#[test]
fn taught_file_and_proc_read_actions_are_available_in_the_actual_example_picker() {
    let all = sources();
    let file = &all[2];
    assert!(
        file.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command == "cat /etc/sysctl.d/70-learning-lab-ipv6.conf")
    );
    let ra = &all[1];
    assert!(
        ra.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command == "cat /proc/sys/net/ipv6/conf/uplink0/accept_ra")
    );
    for s in all {
        binding::source(&s).unwrap();
        for c in s.guide.iter().flat_map(|g| &g.commands) {
            assert!(
                !c.explanation
                    .starts_with("Apply this one example configuration change")
            );
        }
    }
}
