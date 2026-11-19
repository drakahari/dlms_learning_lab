//! Actual registered lessons, progress files and source gates; no host networking.
use dlms_learning_lab::{
    engine::{self, State},
    linux_router_source as binding,
    linux_router_workspace::{Action, Session},
    pack,
    progress::{self, Attempt, Progress},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/linux-router-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn session(s: &mut State) -> &mut Session {
    let State::LinuxRouter { simulation } = s else {
        panic!()
    };
    simulation
}
#[test]
fn registered_worked_examples_are_ungraded_and_independent_recipes_require_current_checks() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let mut st = s.initial.clone();
        for a in &d.worked {
            session(&mut st).action(d, a, true).unwrap();
        }
        assert!(!engine::solved(&st, &s.goal));
        assert_eq!(
            session(&mut st).assignment,
            Session::new(d).unwrap().assignment
        );
        for line in &s.solution {
            engine::execute_for_goal(&mut st, &s.goal, line).unwrap();
        }
        assert!(
            engine::solved(&st, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&st, &s.goal)
        );
        let mut a = Attempt::new(&s, 1);
        a.state = st;
        assert!(progress::current_solved(&a, &s));
        let line = format!(
            "{}{}",
            binding::PREFIX,
            serde_json::to_string(&Action::Input {
                text: "iptables -S".into()
            })
            .unwrap()
        );
        engine::execute_for_goal(&mut a.state, &s.goal, &line).unwrap();
        assert!(!progress::current_solved(&a, &s));
        assert!(engine::execute(&mut a.state, "iptables -S").is_err());
    }
}
#[test]
fn source_mismatches_and_unsupported_author_recipes_cannot_mutate_or_award_success() {
    let s = sources().remove(0);
    let mut wrong = s.clone();
    wrong.version = 2;
    assert!(binding::source(&wrong).is_err());
    wrong = s.clone();
    wrong.track = "CCNA".into();
    assert!(binding::source(&wrong).is_err());
    let mut a = Attempt::new(&s, 1);
    a.scenario_version = 2;
    assert!(binding::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut st = s.initial.clone();
    let before = st.clone();
    let line = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&Action::Command {
            line: "iptables -L".into()
        })
        .unwrap()
    );
    assert!(engine::execute_for_goal(&mut st, &s.goal, &line).is_err());
    assert_eq!(st, before);
    let d = binding::source(&s).unwrap();
    session(&mut st)
        .assignment
        .state
        .host
        .configuration
        .interfaces[0]
        .address
        .as_mut()
        .unwrap()
        .address = "10.20.30.2".parse().unwrap();
    assert!(d.bound(session(&mut st)).is_err());
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn dirty_file_and_command_drafts_resume_without_assignment_credit_or_cross_scope_reset() {
    let s = sources().remove(2);
    let d = binding::source(&s).unwrap();
    let mut st = s.initial.clone();
    for (practice, text) in [
        (false, "# assignment unsaved\nnet.ipv4.ip_forward=0\n"),
        (true, "# practice unsaved\nnet.ipv4.ip_forward=1\n"),
    ] {
        for action in [
            Action::File {
                file: dlms_learning_lab::linux_router_workspace::File::Forwarding,
            },
            Action::Edit { text: text.into() },
            Action::Input {
                text: if practice {
                    "sysctl -n net.ipv4.ip_forward"
                } else {
                    "iptables-restore --test /etc/lab-filter.rules"
                }
                .into(),
            },
            Action::SelectCase { index: 3 },
        ] {
            session(&mut st).action(d, &action, practice).unwrap();
        }
    }
    let expected = st.clone();
    let mut a = Attempt::new(&s, 1);
    a.state = st;
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let read = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(read.attempts[&s.id].state, expected);
    assert!(read.favorites.contains(&s.id));
    assert!(!progress::current_solved(&read.attempts[&s.id], &s));
    binding::context(&read.attempts[&s.id], &s).unwrap();
    let mut st = read.attempts[&s.id].state.clone();
    let before = session(&mut st).practice.clone();
    session(&mut st).reset(d, false).unwrap();
    assert_eq!(session(&mut st).practice, before);
    assert_eq!(
        session(&mut st).assignment,
        Session::new(d).unwrap().assignment
    );
}
#[test]
fn normal_catalog_preserves_every_preceding_lab_and_unit_and_orders_new_prerequisites() {
    let old = pack::accepted482().unwrap();
    let now = pack::current().unwrap();
    assert_eq!((old.len(), now.len()), (482, 490));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..482]).unwrap()
    );
    let old_units = dlms_learning_lab::course::accepted482(&old).unwrap();
    let units = dlms_learning_lab::course::current(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (192, 196));
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-router-policies"
                    && u.id != "linux-connection-nat"
                    && u.id != "linux-ipv6-router-policy"
                    && u.id != "linux-router-service-names")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let unit = units
        .iter()
        .position(|u| u.id == "linux-router-policies")
        .unwrap();
    for prerequisite in ["linux-host-networking", "ssh-administration"] {
        assert!(units.iter().position(|u| u.id == prerequisite).unwrap() < unit);
    }
    assert_eq!(
        units[unit].labs,
        sources().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    for s in &now[482..485] {
        binding::source(s).unwrap();
    }
}

#[test]
fn every_guide_command_runs_in_its_own_example_and_initial_choice_is_inspection() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let commands = s.guide.iter().flat_map(|g| &g.commands).collect::<Vec<_>>();
        assert!(!commands.is_empty());
        for c in &commands {
            assert!(!c.simulator_only);
            assert!(!c.explanation.trim().is_empty());
            let mut state = d.example.initial.clone();
            state
                .command(&c.command)
                .unwrap_or_else(|e| panic!("{} example {}: {}", s.id, c.command, e.message));
        }
        let mut first = d.example.initial.clone();
        let before = first.clone();
        assert!(!first.command(&commands[0].command).unwrap().changed);
        assert_eq!(
            first, before,
            "Begin guided practice by inspecting, without silently changing the example."
        );
    }
}
