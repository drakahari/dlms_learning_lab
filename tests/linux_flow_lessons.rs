//! Registered authored lessons, real progress files and exact prior-catalog preservation.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    linux_flow_source as binding,
    linux_flow_workspace::{Action, Session},
    pack,
    progress::{self, Attempt, Progress},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/linux-flow-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn session(state: &mut State) -> &mut Session {
    let State::LinuxFlow { simulation } = state else {
        panic!("Actual registered flow workspace required")
    };
    simulation
}
#[test]
fn actual_registered_examples_are_ungraded_and_assignments_require_real_current_checks() {
    for source in sources() {
        let d = binding::source(&source).unwrap();
        let mut state = source.initial.clone();
        let before = session(&mut state).assignment.clone();
        for a in &d.worked {
            session(&mut state).action(d, a, true).unwrap();
        }
        assert_eq!(session(&mut state).assignment, before);
        assert!(!engine::solved(&state, &source.goal));
        for command in &source.solution {
            engine::execute_for_goal(&mut state, &source.goal, command).unwrap();
        }
        assert!(
            engine::solved(&state, &source.goal),
            "{}: {:?}",
            source.id,
            engine::checks(&state, &source.goal)
        );
        let mut attempt = Attempt::new(&source, 1);
        attempt.state = state;
        assert!(progress::current_solved(&attempt, &source));
        let action = format!(
            "{}{}",
            binding::PREFIX,
            serde_json::to_string(&Action::Command {
                line: "iptables -P FORWARD ACCEPT".into()
            })
            .unwrap()
        );
        engine::execute_for_goal(&mut attempt.state, &source.goal, &action).unwrap();
        assert!(!progress::current_solved(&attempt, &source));
        let action = format!(
            "{}{}",
            binding::PREFIX,
            serde_json::to_string(&Action::Check).unwrap()
        );
        engine::execute_for_goal(&mut attempt.state, &source.goal, &action).unwrap();
        assert!(!progress::current_solved(&attempt, &source));
        assert!(engine::execute(&mut attempt.state, "iptables -S").is_err());
    }
}
#[test]
fn malformed_source_and_unsupported_author_recipe_never_mutate_or_award_credit() {
    let source = sources().remove(0);
    for mutation in ["version", "track", "state", "guide"] {
        let mut bad = source.clone();
        match mutation {
            "version" => bad.version = 2,
            "track" => bad.track = "CCNA".into(),
            "state" => {
                let State::LinuxFlow { simulation } = &mut bad.initial else {
                    panic!()
                };
                simulation.source_id = "other-source".into();
            }
            "guide" => bad.guide[0].commands[0].command = "sh -c 'touch private'".into(),
            _ => unreachable!(),
        }
        assert!(binding::source(&bad).is_err(), "{mutation}");
    }
    let mut state = source.initial.clone();
    let before = state.clone();
    let line = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&Action::Command {
            line: "iptables -A FORWARD -m conntrack --ctstate RELATED -j ACCEPT".into()
        })
        .unwrap()
    );
    assert!(engine::execute_for_goal(&mut state, &source.goal, &line).is_err());
    assert_eq!(state, before);
    assert!(!engine::solved(&state, &source.goal));
}
#[test]
fn distinct_drafts_and_actual_trial_results_resume_from_disk_without_credit_invention() {
    for source in sources() {
        let d = binding::source(&source).unwrap();
        let mut state = source.initial.clone();
        for (practice, text) in [(false, "iptables -t nat -S"), (true, "iptables -S")] {
            for action in [Action::Input { text: text.into() }, Action::RunTraffic] {
                session(&mut state).action(d, &action, practice).unwrap();
            }
        }
        let expected = state.clone();
        let mut attempt = Attempt::new(&source, 1);
        attempt.state = state;
        let mut stored = Progress {
            schema_version: 1,
            clock: 2,
            ..Default::default()
        };
        stored.attempts.insert(source.id.clone(), attempt);
        stored.favorites.insert(source.id.clone());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress-v1.json");
        assert!(progress::save(&path, &stored).is_err());
        progress::save_with_catalog(&path, &stored, std::slice::from_ref(&source)).unwrap();
        let restored = progress::load(&path, std::slice::from_ref(&source)).unwrap();
        assert_eq!(restored.attempts[&source.id].state, expected);
        assert!(restored.favorites.contains(&source.id));
        binding::context(&restored.attempts[&source.id], &source).unwrap();
        assert!(!progress::current_solved(
            &restored.attempts[&source.id],
            &source
        ));
        let mut state = restored.attempts[&source.id].state.clone();
        let before = session(&mut state).practice.clone();
        session(&mut state).reset(d, false).unwrap();
        assert_eq!(session(&mut state).practice, before);
        assert_eq!(
            session(&mut state).assignment,
            Session::new(d).unwrap().assignment
        );
    }
}
#[test]
fn normal_catalog_preserves_all_485_labs_and_193_units_with_two_prerequisite_ordered_units() {
    let before = pack::accepted485().unwrap();
    let now = pack::current().unwrap();
    assert_eq!((before.len(), now.len()), (485, 490));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&now[..485]).unwrap()
    );
    let old_units = course::accepted485(&before).unwrap();
    let units = course::current(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (193, 196));
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-connection-nat"
                    && u.id != "linux-ipv6-router-policy"
                    && u.id != "linux-router-service-names")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let unit = units
        .iter()
        .position(|u| u.id == "linux-connection-nat")
        .unwrap();
    assert!(
        units
            .iter()
            .position(|u| u.id == "linux-router-policies")
            .unwrap()
            < unit
    );
    assert_eq!(
        units[unit].labs,
        sources().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    for s in &now[485..487] {
        binding::source(s).unwrap();
    }
    let counts = now
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut counts, s| {
            *counts.entry(s.track.clone()).or_insert(0usize) += 1;
            counts
        });
    assert_eq!(counts.get("CCNA"), Some(&113));
    assert_eq!(counts.get("LPIC-1"), Some(&154));
    assert_eq!(counts.get("LPIC-2"), Some(&143));
    assert_eq!(counts.get("Bash/Shell"), Some(&80));
}
#[test]
fn guide_values_are_separate_supported_and_first_choice_is_inspection_not_execution() {
    for source in sources() {
        let d = binding::source(&source).unwrap();
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .collect::<Vec<_>>();
        assert_eq!(commands[0].command, "iptables -S");
        assert!(
            commands
                .iter()
                .all(|c| !c.simulator_only && !c.explanation.trim().is_empty())
        );
        for c in commands {
            let mut state = d.example.initial.clone();
            d.example
                .command(&mut state, &c.command)
                .unwrap_or_else(|e| panic!("{}: {}", c.command, e.message));
        }
        let mut state = source.initial.clone();
        let before = session(&mut state).clone();
        session(&mut state)
            .action(
                d,
                &Action::Input {
                    text: "iptables -S".into(),
                },
                true,
            )
            .unwrap();
        assert_eq!(session(&mut state).practice.state, before.practice.state);
        assert_eq!(session(&mut state).practice.output, before.practice.output);
        assert_eq!(session(&mut state).assignment, before.assignment);
        assert!(!engine::solved(&state, &source.goal));
    }
}
