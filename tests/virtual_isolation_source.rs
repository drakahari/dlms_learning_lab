use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    virtual_isolation_source as binding, virtual_isolation_workspace as work,
};
#[path = "support/virtual_isolation_tasks.rs"]
mod setup;
fn scenario(routing: bool) -> pack::Scenario {
    let mut s = pack::load(include_str!("../scenarios/leaf-spine-lessons-v1.json"))
        .unwrap()
        .remove(0);
    let d = setup::definition(routing);
    s.id = d.id.clone();
    s.version = d.version;
    s.initial = State::VirtualIsolation {
        simulation: Box::new(d.new_session().unwrap()),
    };
    s.goal = Goal::VirtualIsolation {
        definition: Box::new(d.clone()),
    };
    s.solution = vec![
        encode(binding::Event::Edit {
            configuration: d.assignment.reference(),
        }),
        encode(binding::Event::Check {}),
    ];
    s
}
fn encode(e: binding::Event) -> String {
    format!("{}{}", binding::PREFIX, serde_json::to_string(&e).unwrap())
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::VirtualIsolation { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn authored_typed_actions_grade_real_state_and_edits_clear_success() {
    for routing in [false, true] {
        let s = scenario(routing);
        let d = binding::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        assert!(!progress::current_solved(&a, &s));
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        session(&mut a)
            .assignment
            .edit(&d.assignment, d.assignment.initial())
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        session(&mut a).assignment.checked = true;
        assert!(!progress::current_solved(&a, &s));
        let raw = serde_json::json!({"schema_version":1,"scenarios":[s]});
        assert_eq!(pack::load(&raw.to_string()).unwrap().len(), 1);
    }
}
#[test]
fn separate_examples_and_real_disk_restore_do_not_invent_credit() {
    for routing in [false, true] {
        let s = scenario(routing);
        let d = binding::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        let sim = session(&mut a);
        sim.practice
            .edit(&d.example, d.example.reference())
            .unwrap();
        sim.practice.check(&d.example).unwrap();
        sim.practice.run(&d.example).unwrap();
        let expected = sim.assignment.configuration.clone();
        assert!(!progress::current_solved(&a, &s));
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("progress-v1.json");
        progress::save_with_catalog(&file, &p, std::slice::from_ref(&s)).unwrap();
        let mut restored = progress::load(&file, std::slice::from_ref(&s)).unwrap();
        assert!(restored.favorites.contains(&s.id));
        let a = restored.attempts.get_mut(&s.id).unwrap();
        binding::context(a, &s).unwrap();
        let sim = session(a);
        assert_eq!(sim.assignment.configuration, expected);
        assert!(sim.practice.solved(&d.example));
        assert!(sim.practice.actual.is_none());
        assert!(!progress::current_solved(a, &s));
        let before = std::fs::read(&file).unwrap();
        assert!(progress::save(&file, &restored).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), before);
    }
}
#[test]
fn malformed_and_host_escape_inputs_are_rejected_without_mutation() {
    for routing in [false, true] {
        let s = scenario(routing);
        let mut a = Attempt::new(&s, 1);
        let before = a.state.clone();
        for input in [
            "docker run arbitrary",
            "virsh start guest",
            "ip netns exec tenant sh",
            "virtual-net {\"kind\":\"Spawn\"}",
            "virtual-net {\"kind\":\"Check\",\"passed\":true}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
            assert_eq!(a.state, before);
        }
        assert!(engine::execute(&mut a.state, "virtual-net {\"kind\":\"Check\"}").is_err());
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, &"x".repeat(binding::MAX_ACTION + 1))
                .is_err()
        );
        assert_eq!(a.state, before);
        let mut wrong = s.clone();
        wrong.track = "LPIC-2".into();
        assert!(binding::source(&wrong).is_err());
        assert!(!progress::current_solved(&a, &wrong));
        let mut wrong = s.clone();
        wrong.version += 1;
        assert!(binding::source(&wrong).is_err());
        session(&mut a).id = "foreign".into();
        assert!(binding::context(&a, &s).is_err());
    }
}
