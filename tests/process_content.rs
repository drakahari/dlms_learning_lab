use dlms_learning_lab::{
    engine::{self, State},
    pack, process_teaching as teaching,
    processes::{Phase, Requirement},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Review {
    id: String,
    alternate: Vec<String>,
    incorrect: Vec<String>,
    misconception: String,
}
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "linux-processes")
        .collect()
}
fn run(s: &pack::Scenario, commands: &[String]) -> State {
    let mut state = s.initial.clone();
    for command in commands {
        engine::execute(&mut state, command).unwrap();
    }
    state
}
#[test]
fn four_distinct_tasks_have_working_taught_alternate_and_wrong_outcomes() {
    let labs = labs();
    assert_eq!(labs.len(), 4);
    let rows: Vec<Review> =
        serde_json::from_str(include_str!("fixtures/process-content-review.json")).unwrap();
    assert_eq!(rows.len(), labs.len());
    for row in rows {
        let s = labs.iter().find(|s| s.id == row.id).unwrap();
        assert!(row.misconception.len() > 30);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(
            engine::solved(&run(s, &s.solution), &s.goal),
            "{} taught",
            s.id
        );
        assert!(
            engine::solved(&run(s, &row.alternate), &s.goal),
            "{} alternate",
            s.id
        );
        let wrong = run(s, &row.incorrect);
        assert!(!engine::solved(&wrong, &s.goal), "{} incorrect", s.id);
        let checks = engine::checks(&wrong, &s.goal);
        assert!(
            checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0),
            "{} partial",
            s.id
        );
        let mut read_only = s.initial.clone();
        engine::execute(&mut read_only, "ps -eo pid,ppid,user,stat,ni,comm").unwrap();
        assert!(!engine::solved(&read_only, &s.goal));
        let t = s.process_teaching.as_ref().unwrap();
        t.validate().unwrap();
        let State::Processes { table } = &s.initial else {
            panic!()
        };
        assert!(
            table
                .processes
                .keys()
                .filter(|&&id| id != 1)
                .all(|id| !t.fixture.processes.contains_key(id)),
            "{} uses different example PIDs",
            s.id
        );
    }
}
#[test]
fn each_suggested_experiment_runs_and_changes_only_the_example() {
    for s in labs() {
        let t = s.process_teaching.as_ref().unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, t, true);
        let initial = a.state.clone();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .process_learning
            .as_mut()
            .unwrap();
        let commands: Vec<&str> = match s.id.as_str() {
            "lpic1-process-pause" => vec!["kill -STOP 6010", "kill -CONT 6010"],
            "lpic1-process-pending" => vec!["kill -STOP 7010", "kill -KILL 7010"],
            "lpic1-process-priority" => vec!["renice --priority 12 -p 8010"],
            _ => vec!["kill -TERM 9011", "kill -TERM 9010"],
        };
        for command in commands {
            l.command = command.into();
            l.execute();
        }
        match s.id.as_str() {
            "lpic1-process-pause" => assert!(matches!(
                l.table.processes[&6010].phase,
                Phase::Active { .. }
            )),
            "lpic1-process-pending" => assert_eq!(l.table.processes[&7010].phase, Phase::Exited),
            "lpic1-process-priority" => assert_eq!(l.table.processes[&8010].nice, 12),
            _ => assert!(t.outcomes.iter().all(|r| r.check(&l.table).0)),
        }
        assert_eq!(a.state, initial);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn source_state_changes_invalidate_success_and_nonrequested_settings_are_graded() {
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        a.state = run(&s, &s.solution);
        assert!(progress::current_solved(&a, &s));
        let broken = match s.id.as_str() {
            "lpic1-process-pause" => "kill -CONT 1510",
            "lpic1-process-pending" => "renice 5 -p 1600",
            "lpic1-process-priority" => "kill -STOP 1710",
            _ => "kill -STOP 1911",
        };
        engine::execute(&mut a.state, broken).unwrap();
        assert!(!progress::current_solved(&a, &s), "{} invalidation", s.id);
    }
    for (id, wrong) in [
        ("lpic1-process-pause", "renice 5 -p 1510"),
        ("lpic1-process-handover", "renice 5 -p 1920"),
    ] {
        let s = labs().into_iter().find(|s| s.id == id).unwrap();
        let mut state = run(&s, &s.solution);
        engine::execute(&mut state, wrong).unwrap();
        assert!(
            !engine::solved(&state, &s.goal),
            "{id} target setting must be preserved"
        );
    }
}
#[test]
fn feedback_explains_one_root_mistake_per_process_without_hiding_other_processes() {
    let s = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-process-pause")
        .unwrap();
    let mut state = s.initial.clone();
    engine::execute(&mut state, "kill -TERM 1510").unwrap();
    let messages = workspace::issues(&state, &s.goal);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("ended") && messages[0].contains("Reset"));
    // A distinct harmed process still needs its own diagnosis.
    engine::execute(&mut state, "kill -STOP 1500").unwrap();
    assert_eq!(workspace::issues(&state, &s.goal).len(), 2);
    // Pending termination is also distinct from a successful safe pause.
    let mut state = run(&s, &s.solution);
    engine::execute(&mut state, "kill -TERM 1510").unwrap();
    let messages = workspace::issues(&state, &s.goal);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("ending request waiting"));
}
#[test]
fn family_handover_requires_the_child_and_preserves_healthy_families_and_reported_parents() {
    let s = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-process-handover")
        .unwrap();
    let mut state = s.initial.clone();
    engine::execute(&mut state, "kill 1810").unwrap();
    let State::Processes { table } = &state else {
        panic!()
    };
    assert_eq!(table.observed_process(1811).unwrap().parent, 1);
    assert!(matches!(table.processes[&1811].phase, Phase::Active { .. }));
    assert!(!engine::solved(&state, &s.goal));
    let mut wrong = run(&s, &s.solution);
    engine::execute(&mut wrong, "kill 1911").unwrap();
    assert!(!engine::solved(&wrong, &s.goal));
}
#[test]
fn all_prior_attempts_and_separate_new_drafts_survive_resume_and_selected_reset() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in all.iter().take(74) {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let previous = serde_json::to_vec(&p).unwrap();
    for s in labs() {
        let mut a = Attempt::new(&s, 2);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.process_teaching.as_ref().unwrap(), true);
        a.state = run(&s, &s.solution);
        let w = a.workspace.as_mut().unwrap();
        w.command = "ps -eo pid,comm".into();
        let l = w.process_learning.as_mut().unwrap();
        l.command = "ps -eo pid,stat".into();
        l.view = View::Example;
        p.attempts.insert(s.id.clone(), a);
    }
    let encoded = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&encoded, &all).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), encoded);
    for s in labs() {
        assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    }
    let selected = labs().remove(1);
    loaded
        .attempts
        .insert(selected.id.clone(), Attempt::new(&selected, 3));
    assert!(!progress::current_solved(
        &loaded.attempts[&selected.id],
        &selected
    ));
    let other = &labs()[0];
    assert_eq!(
        serde_json::to_vec(&loaded.attempts[&other.id]).unwrap(),
        serde_json::to_vec(&p.attempts[&other.id]).unwrap()
    );
    loaded
        .attempts
        .retain(|id, _| !id.starts_with("lpic1-process-"));
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), previous);
}
#[test]
fn pending_termination_and_priority_failures_explain_the_actual_mistake() {
    let s = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-process-pending")
        .unwrap();
    let wrong = run(&s, &["kill -TERM 1610".into()]);
    let message = &engine::checks(&wrong, &s.goal)[0].1;
    assert!(message.contains("still paused") && message.contains("pending"));
    let s = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-process-priority")
        .unwrap();
    let mut wrong = s.initial.clone();
    let denied = engine::execute(&mut wrong, "renice --priority -10 -p 1710").unwrap_err();
    assert!(denied.contains("Permission denied"));
    assert!(!engine::solved(&wrong, &s.goal));
    engine::execute(&mut wrong, "renice 19 -p 1710").unwrap();
    let message = &engine::checks(&wrong, &s.goal)[0].1;
    assert!(message.contains("19") && message.contains("10") && message.contains("Reset"));
}
#[test]
fn unknown_and_unsupported_targets_cannot_fake_objective_success() {
    for s in labs() {
        let mut state = s.initial.clone();
        assert!(engine::execute(&mut state, "kill -STOP 2147483647").is_err());
        assert_eq!(state, s.initial);
        for command in [
            "echo complete",
            "kill -s SIG19 1510",
            "sudo kill -STOP 1510",
            "kill -STOP $(pgrep -x worker)",
        ] {
            assert!(engine::execute(&mut state, command).is_err());
            assert_eq!(state, s.initial);
        }
        assert!(!engine::solved(&state, &s.goal));
        let engine::Goal::Processes { requirements } = &s.goal else {
            panic!()
        };
        assert!(
            requirements
                .iter()
                .any(|r| matches!(r, Requirement::Unchanged { .. }))
        );
    }
}

#[test]
fn a_correct_pause_cannot_stay_successful_after_a_termination_request_is_queued() {
    let s = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-process-pause")
        .unwrap();
    for signal in ["TERM", "INT"] {
        let mut a = Attempt::new(&s, 1);
        a.state = run(&s, &s.solution);
        assert!(progress::current_solved(&a, &s));
        engine::execute(&mut a.state, &format!("kill -{signal} 1510")).unwrap();
        assert!(!progress::current_solved(&a, &s));
        assert!(
            engine::checks(&a.state, &s.goal)
                .iter()
                .any(|(pass, text)| !pass
                    && text.contains("ending request waiting")
                    && text.contains("Reset"))
        );
        let State::Processes { table } = &a.state else {
            panic!()
        };
        assert!(matches!(
            table.processes[&1510].phase,
            Phase::Stopped {
                pending_termination: true,
                ..
            }
        ));
    }
}
