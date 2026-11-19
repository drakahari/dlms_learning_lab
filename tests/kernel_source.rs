#[path = "support/kernel_build_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, Goal, State},
    kernel_source as binding,
    kernel_workspace::{Action, Definition, Session},
    pack,
    progress::{self, Attempt, Progress},
};
fn scenario(boot: bool) -> pack::Scenario {
    let mut s =
        serde_json::from_str::<pack::Pack>(include_str!("../scenarios/poe-lessons-v1.json"))
            .unwrap()
            .scenarios
            .remove(0);
    let task = if boot {
        fixture::boot()
    } else {
        fixture::image()
    };
    let mut example = task.clone();
    example.id.push_str("-example");
    example.sources.last_mut().unwrap().release = "6.6.51".into();
    // The image task's reference has no release-specific paths.
    if boot {
        for action in &mut example.reference {
            if let Action::Initramfs { image_path } = action {
                *image_path = image_path.replace("6.6.50", "6.6.51");
            }
            if let Action::EntryDraft { text, .. } = action {
                *text = text.replace("6.6.50", "6.6.51");
            }
        }
    }
    let d = Definition {
        version: 1,
        source_id: task.id.clone(),
        assignment: task,
        example,
    };
    s.id = d.source_id.clone();
    s.track = "LPIC-2".into();
    s.objective = d.assignment.instructions.clone();
    s.initial = State::KernelBuild {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.solution = d
        .assignment
        .reference
        .iter()
        .map(|a| routed(a, false))
        .collect();
    s.goal = Goal::KernelBuild {
        definition: Box::new(d),
    };
    s
}
fn routed(a: &Action, practice: bool) -> String {
    format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    )
}
#[test]
fn native_source_executes_real_task_outcomes_and_keeps_practice_ungraded() {
    for boot in [false, true] {
        let s = scenario(boot);
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::KernelBuild { simulation } = &state else {
            panic!()
        };
        let untouched = simulation.assignment.clone();
        for a in &d.example.reference {
            engine::execute_for_goal(&mut state, &s.goal, &routed(a, true)).unwrap();
        }
        let State::KernelBuild { simulation } = &state else {
            panic!()
        };
        assert!(d.example.solved(&simulation.example.work));
        assert_eq!(simulation.assignment, untouched);
        assert!(!engine::solved(&state, &s.goal));
        for input in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{:?}",
            engine::checks(&state, &s.goal)
        );
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &routed(
                &Action::Config {
                    text: "# unfinished draft\n".into(),
                },
                false,
            ),
        )
        .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        assert!(
            engine::checks(&state, &s.goal)
                .iter()
                .any(|c| !c.0 && c.1.contains("Not checked"))
        );
    }
}
#[test]
fn mismatched_goals_malformed_routes_and_unbound_execution_do_not_change_work() {
    let s = scenario(false);
    let mut state = s.initial.clone();
    let original = serde_json::to_value(&state).unwrap();
    assert!(engine::execute(&mut state, "make all").is_err());
    for input in [
        "make all".to_owned(),
        format!("{}{{}}", binding::PREFIX),
        format!("{}{}", binding::PREFIX, " ".repeat(16385)),
        routed(&Action::Source { index: 99 }, false),
    ] {
        assert!(engine::execute_for_goal(&mut state, &s.goal, &input).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), original);
    }
    let other = scenario(true);
    assert!(
        engine::execute_for_goal(&mut state, &other.goal, &routed(&Action::Check, false)).is_err()
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), original);
    let mut bad = s.clone();
    bad.track = "CCNA".into();
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    bad.objective.push_str(" altered");
    assert!(binding::source(&bad).is_err());
    let mut bad = s;
    bad.initial = other.initial;
    assert!(binding::source(&bad).is_err());
}
#[test]
fn unfinished_and_successful_progress_restore_with_exact_source_binding() {
    let all = [scenario(false), scenario(true)];
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        a.challenge = true;
        p.favorites.insert(s.id.clone());
        for input in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, input).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        engine::execute_for_goal(
            &mut a.state,
            &s.goal,
            &routed(
                &Action::CommandDraft {
                    text: "make mod".into(),
                },
                true,
            ),
        )
        .unwrap();
        p.attempts.insert(s.id.clone(), a);
    }
    let raw = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&raw, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for s in &all {
        assert!(progress::current_solved(&restored.attempts[&s.id], s));
    }
    let mut changed = all.clone();
    changed[0].version = 2;
    assert!(progress::decode(&raw, &changed).is_err());
    let mut bad = p;
    let State::KernelBuild { simulation } = &mut bad.attempts.get_mut(&all[0].id).unwrap().state
    else {
        panic!()
    };
    simulation.example.source_task = "forged".into();
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
}
