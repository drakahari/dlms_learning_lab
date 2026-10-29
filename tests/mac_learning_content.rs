//! Authored MAC data acceptance proposals. No host network, packet or shell engine.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    mac_learning as mac, mac_learning_teaching as teaching, mac_learning_workspace as work,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize)]
struct Review {
    id: String,
    alternates: Vec<Vec<String>>,
    partial: Vec<String>,
    wrong: Vec<Wrong>,
    experiment_actions: Vec<work::Action>,
    experiment_outcomes: Vec<work::Requirement>,
    repair_actions: Vec<work::Action>,
    retry_repair: Vec<String>,
}
#[derive(Deserialize)]
struct Wrong {
    name: String,
    actions: Vec<String>,
    why: String,
}
fn authored() -> Vec<Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/vlan-data-v1.json"),
        include_str!("../scenarios/mac-learning-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "ethernet-learning")
    .collect()
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/mac-content-review.json")).unwrap()
}
fn catalog() -> Vec<Scenario> {
    let mut all = pack::bundled().unwrap();
    for s in authored() {
        if !all.iter().any(|old| old.id == s.id) {
            all.push(s)
        }
    }
    all
}
fn model(s: &State) -> &mac::State {
    let State::MacLearning { simulation } = s else {
        panic!()
    };
    simulation
}
fn goal(s: &Scenario) -> (&mac::State, &[work::Requirement]) {
    let Goal::MacLearning {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!()
    };
    (initial, requirements)
}
fn solve(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.mac_teaching.as_ref().unwrap(), true);
    a
}
#[test]
fn authored_references_two_distinct_alternatives_and_independent_probes_reach_the_whole_task() {
    let labs = authored();
    let rows = reviews();
    assert_eq!(labs.len(), 4);
    for s in labs {
        assert!(!engine::solved(&s.initial, &s.goal));
        let row = rows.iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(row.alternates.len(), 2);
        let solved = solve(&s);
        let before = serde_json::to_value(&solved).unwrap();
        let checks = engine::checks(&solved, &s.goal);
        assert!(checks.iter().all(|c| c.0));
        assert_eq!(serde_json::to_value(&solved).unwrap(), before);
        for inputs in &row.alternates {
            let mut state = s.initial.clone();
            for input in inputs {
                engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
            }
            assert!(
                engine::solved(&state, &s.goal),
                "{}: {:?}",
                s.id,
                engine::checks(&state, &s.goal)
            );
        }
    }
}
#[test]
fn declared_misconceptions_and_partial_results_fail_without_exact_command_grading() {
    let rows = reviews();
    for s in authored() {
        let row = rows.iter().find(|r| r.id == s.id).unwrap();
        assert!(row.wrong.len() >= 2);
        let mut partial = s.initial.clone();
        for input in &row.partial {
            engine::execute_for_goal(&mut partial, &s.goal, input).unwrap();
        }
        let c = engine::checks(&partial, &s.goal);
        assert!(
            c.iter().any(|c| c.0) && c.iter().any(|c| !c.0),
            "{} partial {c:?}",
            s.id
        );
        for w in &row.wrong {
            assert!(!w.why.is_empty());
            let mut bad = s.initial.clone();
            for input in &w.actions {
                let before = bad.clone();
                if engine::execute_for_goal(&mut bad, &s.goal, input).is_err() {
                    assert_eq!(bad, before);
                    break;
                }
            }
            assert!(
                !engine::solved(&bad, &s.goal),
                "{} misconception{} passed",
                s.id,
                w.name
            );
        }
    }
}
fn break_requirement(state: &mac::State, r: &work::Requirement) -> mac::State {
    let mut v = serde_json::to_value(state).unwrap();
    match r {
        work::Requirement::Outcome {
            outcome:
                mac::Requirement::Learned {
                    switch,
                    vlan,
                    address,
                    ..
                },
        } => v["entries"].as_array_mut().unwrap().retain(|e| {
            !(e["switch"] == *switch && e["vlan"] == *vlan && e["address"] == address.text())
        }),
        work::Requirement::Outcome {
            outcome: mac::Requirement::Aging { .. },
        } => v["aging_seconds"] = 0.into(),
        work::Requirement::Outcome {
            outcome: mac::Requirement::Frame { from, .. },
        } => {
            let (host, port) = state
                .fabric()
                .cables
                .iter()
                .find_map(|c| {
                    use dlms_learning_lab::switching::Jack;
                    match (&c.a, &c.b) {
                        (Jack::Host { host }, Jack::Switch { switch, port })
                        | (Jack::Switch { switch, port }, Jack::Host { host })
                            if host == from =>
                        {
                            Some((switch, port))
                        }
                        _ => None,
                    }
                })
                .unwrap();
            v["fabric"]["switches"][host]["ports"][port]["up"] = false.into();
        }
        work::Requirement::Port { switch, port, .. } => {
            v["fabric"]["switches"][switch]["ports"][port]["up"] = false.into()
        }
        work::Requirement::ActiveVlans { switch, vlans } => {
            let removed = *vlans.iter().next().unwrap();
            v["fabric"]["switches"][switch]["vlans"]
                .as_array_mut()
                .unwrap()
                .retain(|n| n != &serde_json::json!(removed));
            v["entries"]
                .as_array_mut()
                .unwrap()
                .retain(|e| !(e["switch"] == *switch && e["vlan"] == removed));
        }
    }
    serde_json::from_value(v).unwrap()
}
#[test]
fn each_declared_requirement_has_a_valid_negative_state_and_does_not_reuse_a_passed_trace() {
    for s in authored() {
        let solved = solve(&s);
        for (index, r) in goal(&s).1.iter().enumerate() {
            let broken = break_requirement(model(&solved), r);
            broken.validate().unwrap();
            let state = State::MacLearning { simulation: broken };
            let checks = engine::checks(&state, &s.goal);
            assert_eq!(checks.len(), goal(&s).1.len());
            assert!(!checks[index].0, "{} objective{index} {:?}", s.id, checks);
        }
    }
}
#[test]
fn every_isolated_example_insert_and_fill_are_not_execution_and_exact_experiments_repair() {
    let rows = reviews();
    for s in authored() {
        let l = s.mac_teaching.as_ref().unwrap();
        let row = rows.iter().find(|r| r.id == s.id).unwrap();
        l.validate().unwrap();
        let mut learning = teaching::Learning::new(l).unwrap();
        let before = learning.simulation.clone();
        learning.fill_example(l).unwrap();
        assert_eq!(learning.simulation, before);
        assert!(!learning.checked);
        for action in &l.example_solution {
            let before = learning.simulation.clone();
            action
                .insert(&learning.simulation, &mut learning.draft)
                .unwrap();
            assert_eq!(learning.simulation, before);
            action.apply(&mut learning.simulation).unwrap();
        }
        learning.checked = true;
        assert!(learning.checks(l).iter().all(|c| c.0));
        let before = learning.simulation.clone();
        assert_eq!(row.experiment_actions, l.try_actions);
        for action in &row.experiment_actions {
            action.apply(&mut learning.simulation).unwrap();
        }
        assert_ne!(learning.simulation, before);
        assert!(
            work::checks(&l.fixture, &learning.simulation, &row.experiment_outcomes)
                .iter()
                .all(|c| c.0)
        );
        assert!(
            learning.checks(l).iter().any(|c| !c.0),
            "{} experiment did not disturb the promised learning",
            s.id
        );
        for action in &row.repair_actions {
            action.apply(&mut learning.simulation).unwrap();
        }
        assert!(
            work::checks(&l.fixture, &learning.simulation, &l.outcomes)
                .iter()
                .all(|c| c.0),
            "{} repair {:?}",
            s.id,
            learning.checks(l)
        );
        assert_eq!(
            s.initial,
            State::MacLearning {
                simulation: goal(&s).0.clone()
            }
        );
    }
}
#[test]
fn cleared_and_pending_work_loses_current_credit_retry_reestablishes_truth_and_raw_drafts_resume() {
    let rows = reviews();
    let labs = catalog();
    for s in authored() {
        let row = rows.iter().find(|r| r.id == s.id).unwrap();
        let mut a = prepared(&s);
        a.state = solve(&s);
        a.workspace.as_mut().unwrap().mac_draft = Some(work::Draft::new(model(&a.state)).unwrap());
        assert!(progress::current_solved(&a, &s));
        let (first_switch, first_vlan) = goal(&s)
            .1
            .iter()
            .find_map(|r| {
                if let work::Requirement::Outcome {
                    outcome: mac::Requirement::Learned { switch, vlan, .. },
                } = r
                {
                    Some((switch, *vlan))
                } else {
                    None
                }
            })
            .unwrap();
        let State::MacLearning { simulation } = &mut a.state else {
            panic!()
        };
        simulation
            .clear_dynamic(first_switch, Some(first_vlan))
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        for input in &row.retry_repair {
            // Native controls first retain the entered setting/send inputs; the
            // goal-aware typed action then performs Apply/Send/Clear. A valid
            // different aging value must not leave the previous raw draft
            // divergent from the newly applied state.
            let action: work::Action =
                serde_json::from_str(input.strip_prefix("@mac ").unwrap()).unwrap();
            action
                .insert(
                    model(&a.state),
                    a.workspace.as_mut().unwrap().mac_draft.as_mut().unwrap(),
                )
                .unwrap();
            engine::execute_for_goal(&mut a.state, &s.goal, input).unwrap();
        }
        assert!(
            progress::current_solved(&a, &s),
            "{} {:?}",
            s.id,
            engine::checks(&a.state, &s.goal)
        );
        let w = a.workspace.as_mut().unwrap();
        w.mac_draft.as_mut().unwrap().aging = "bad".into();
        w.mac_draft.as_mut().unwrap().destination = "not-a-mac".into();
        assert!(!progress::current_solved(&a, &s));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let bytes = serde_json::to_vec(&p).unwrap();
        let loaded = progress::decode(&bytes, &labs).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
            serde_json::to_value(&p.attempts[&s.id]).unwrap()
        );
        assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    }
}
#[derive(Deserialize)]
struct Old {
    id: String,
    version: u32,
    unit: String,
}
#[test]
fn original109_catalog_and_advisory_unit_prerequisites_are_preserved_without_registering_variants()
{
    let old: Vec<Old> =
        serde_json::from_str(include_str!("fixtures/mac-original-catalog.json")).unwrap();
    assert_eq!(old.len(), 109);
    let labs = catalog();
    for row in &old {
        let s = labs.iter().find(|s| s.id == row.id).unwrap();
        assert_eq!(s.version, row.version);
        assert_eq!(s.unit, row.unit);
    }
    let mut units = course::bundled(&pack::bundled().unwrap()).unwrap();
    let proposed: course::Unit =
        serde_json::from_str(include_str!("fixtures/mac-unit-proposal.json")).unwrap();
    if !units.iter().any(|u| u.id == proposed.id) {
        units.push(proposed)
    }
    let index = serde_json::json!({"version":1,"units":&units});
    course::load(&index.to_string(), &labs).unwrap();
    let authored_ids: Vec<_> = authored().iter().map(|s| s.id.clone()).collect();
    let index = units.iter().find(|u| u.id == "ethernet-learning").unwrap();
    assert_eq!(index.labs, authored_ids);
    let position = |id: &str| units.iter().position(|u| u.id == id).unwrap();
    assert!(position("vlan-data-foundations") < position("ethernet-learning"));
    assert!(position("ethernet-learning") < position("ipv4-static-foundations"));
    for s in authored() {
        for id in &s.prerequisites {
            assert!(labs.iter().any(|lab| &lab.id == id));
        }
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
    }
}
#[test]
fn new_data_never_awards_assignment_credit_from_practice_and_each_fixture_is_exactly_pinned() {
    let labs = catalog();
    for s in authored() {
        let mut a = prepared(&s);
        let before = a.state.clone();
        let l = s.mac_teaching.as_ref().unwrap();
        let mut learning = teaching::Learning::new(l).unwrap();
        for action in &l.example_solution {
            action.apply(&mut learning.simulation).unwrap();
        }
        learning.checked = true;
        a.workspace.as_mut().unwrap().mac_learning = Some(learning);
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let good = serde_json::to_value(&p).unwrap();
        assert!(progress::decode(&serde_json::to_vec(&good).unwrap(), &labs).is_ok());
        let mut foreign = good.clone();
        foreign["attempts"][&s.id]["workspace"]["mac_learning"]["fixture"]["aging_seconds"] =
            200.into();
        assert!(progress::decode(&serde_json::to_vec(&foreign).unwrap(), &labs).is_err());
    }
}
#[test]
fn export_inspect_restore_retains_actual6_7_history_current_runs_and_selected_reset_only() {
    let labs = catalog();
    let original: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    let mut p: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-writer-progress.json"
    ))
    .unwrap();
    for s in &labs {
        p.attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
    }
    // One current record is actually evaluated now, never relabeled as6/7.
    let script = labs.iter().find(|s| s.id == "shell-handover").unwrap();
    let Goal::Script { cases } = &script.goal else {
        panic!()
    };
    let source = script.solution[0].clone();
    let run = shell::Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.clone(),
        results: cases.iter().map(|c| shell::run(&source, c)).collect(),
    };
    let a = p.attempts.get_mut(&script.id).unwrap();
    workspace::preserve_script_history(a);
    a.state = State::Script {
        source,
        run: Some(run),
    };
    for s in authored() {
        let mut a = prepared(&s);
        a.state = solve(&s);
        a.workspace.as_mut().unwrap().mac_draft = Some(work::Draft::new(model(&a.state)).unwrap());
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let selected = authored().remove(1);
    let other = p
        .attempts
        .iter()
        .filter(|(id, _)| *id != &selected.id)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<BTreeMap<_, _>>();
    p.attempts.insert(selected.id.clone(), prepared(&selected));
    for (id, expected) in &other {
        assert_eq!(&serde_json::to_value(&p.attempts[id]).unwrap(), expected)
    }
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("export.json");
    dlms_learning_lab::backup::export(&backup, &p, &labs).unwrap();
    let candidate = dlms_learning_lab::backup::inspect(
        &backup,
        &labs,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let mut six = 0;
    let mut seven = 0;
    for (id, old) in &original.attempts {
        let State::Script {
            run: Some(old7), ..
        } = &old.state
        else {
            continue;
        };
        let w = candidate.progress.attempts[id].workspace.as_ref().unwrap();
        let histories = w
            .script_history
            .iter()
            .chain(&w.script_history_more)
            .collect::<Vec<_>>();
        assert!(histories.contains(&old7));
        seven += 1;
        if let Some(old6) = old
            .workspace
            .as_ref()
            .and_then(|w| w.script_history.as_ref())
        {
            assert_eq!(w.script_history.as_ref(), Some(old6));
            six += 1;
        }
    }
    assert_eq!((six, seven), (33, 37));
    let target = dir.path().join("profile.json");
    progress::save_with_catalog(&target, &p, &labs).unwrap();
    let before = std::fs::read(&target).unwrap();
    let saved = dlms_learning_lab::backup::restore(&target, &candidate, &labs).unwrap();
    assert_eq!(std::fs::read(saved.safety_copy.unwrap()).unwrap(), before);
    let loaded = progress::load(&target, &labs).unwrap();
    assert_eq!(loaded.favorites, p.favorites);
    assert!(!progress::current_solved(
        &loaded.attempts[&selected.id],
        &selected
    ));
    for s in authored().into_iter().filter(|s| s.id != selected.id) {
        assert_eq!(loaded.attempts[&s.id].state, p.attempts[&s.id].state);
        assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    }
}
#[test]
fn source_refresh_is_not_destination_use_and_future_checks_are_independent() {
    let s = authored()
        .into_iter()
        .find(|s| s.id == "ccna-mac-source-refresh")
        .unwrap();
    let mut state = s.initial.clone();
    for action in [
        "@mac {\"kind\":\"Aging\",\"seconds\":120}",
        "@mac {\"kind\":\"Send\",\"source\":\"collector\",\"destination\":\"02:00:00:03:00:01\"}",
    ] {
        engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
    }
    let current = engine::checks(&state, &s.goal);
    assert!(current.iter().any(|c| !c.0));
    let before = serde_json::to_value(&state).unwrap();
    for _ in 0..2 {
        let checks = engine::checks(&state, &s.goal);
        assert_eq!(checks, current);
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
}
