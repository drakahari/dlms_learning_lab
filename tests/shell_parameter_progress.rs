//! Current13 compatibility of retained capability11. Historical8 bytes are genuine and hash-pinned.
//! This is not evidence of evaluator10; its separate capture is required before integration.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching, shell, workspace,
};
use serde_json::Value;
#[path = "../examples/support/getopts_digest.rs"]
mod digest;
const ACTUAL8: &[u8] = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
fn json(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).unwrap()
}
#[test]
fn actual8_drafts_examples_results_and_raw_backup_survive_without_invented_current_credit() {
    assert_eq!(shell::EVALUATOR_REVISION, 13);
    assert_eq!(
        digest::hash(ACTUAL8),
        "88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26"
    );
    let all = pack::bundled().unwrap();
    let original: Progress = serde_json::from_slice(ACTUAL8).unwrap();
    let mut loaded = progress::decode(ACTUAL8, &all).unwrap();
    assert_eq!(json(&loaded), json(&original));
    assert_eq!(loaded.attempts.len(), 116);
    let mut scripts = 0;
    for (id, attempt) in &mut loaded.attempts {
        let old = &original.attempts[id];
        let scenario = all.iter().find(|s| &s.id == id).unwrap();
        let State::Script {
            source: old_source,
            run: Some(old_record),
        } = &old.state
        else {
            assert_eq!(json(attempt), json(old));
            continue;
        };
        scripts += 1;
        assert_eq!(old_record.engine_revision, 8);
        assert!(!progress::current_solved(attempt, scenario));
        workspace::prepare(attempt);
        let State::Script { source, run } = &attempt.state else {
            panic!()
        };
        assert_eq!(source, old_source);
        assert!(run.is_none());
        let now = attempt.workspace.as_ref().unwrap();
        let old_workspace = old.workspace.as_ref().unwrap();
        assert!(
            now.script_history
                .as_ref()
                .into_iter()
                .chain(&now.script_history_more)
                .any(|r| r == old_record)
        );
        assert_eq!(
            json(&now.script_learning),
            json(&old_workspace.script_learning)
        );
        assert_eq!(attempt.visited, old.visited);
        assert_eq!(attempt.transcript, old.transcript);
        assert_eq!(attempt.challenge, old.challenge);
        assert_eq!(attempt.scenario_version, old.scenario_version);
        assert!(!progress::current_solved(attempt, scenario));
    }
    assert_eq!(scripts, 40);
    assert_eq!(loaded.favorites, original.favorites);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    std::fs::write(&path, ACTUAL8).unwrap();
    progress::save(&path, &loaded).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v13.json")).unwrap(),
        ACTUAL8
    );
    assert_eq!(json(&progress::load(&path, &all).unwrap()), json(&loaded));
    assert!(
        loaded
            .migration_receipts
            .as_ref()
            .unwrap()
            .shell
            .iter()
            .all(|r| *r < 12),
        "Saving cannot stamp its caller"
    );
    assert!(
        !loaded
            .migration_receipts
            .as_ref()
            .unwrap()
            .shell
            .contains(&13),
        "Saving cannot add current13 to its caller"
    );
    assert_eq!(
        progress::load(&path, &all)
            .unwrap()
            .migration_receipts
            .unwrap()
            .shell
            .last(),
        Some(&13)
    );
}
#[test]
fn explicit12_run_edit_invalidation_separate_example_and_selected_reset_round_trip() {
    let all = pack::bundled().unwrap();
    let scenario = all.iter().find(|s| s.id == "shell-handover").unwrap();
    let Goal::Script { cases } = &scenario.goal else {
        panic!()
    };
    let lesson = scenario.teaching.as_ref().unwrap();
    let mut p = progress::decode(ACTUAL8, &all).unwrap();
    let original = json(&p);
    let attempt = p.attempts.get_mut(&scenario.id).unwrap();
    workspace::prepare(attempt);
    let learning = attempt
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learning.view = script_teaching::View::Example;
    let example = lesson.example.replacen("Mina", "Rowan", 1);
    learning.example_source = example.clone();
    learning.example_revision = shell::EVALUATOR_REVISION;
    learning.example_result = Some(shell::run(&example, &lesson.fixture));
    assert!(
        !progress::current_solved(attempt, scenario),
        "Practice cannot award assignment credit"
    );
    let assignment = "printf 'Site: %s\\nTask: %s\\n' \"${SITE-unused}\" \"${TASK-unused}\"\n";
    let results: Vec<_> = cases.iter().map(|c| shell::run(assignment, c)).collect();
    assert!(cases.iter().zip(&results).all(|(c, r)| c.accepts(r)));
    attempt.state = State::Script {
        source: assignment.into(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: assignment.into(),
            results,
        }),
    };
    assert!(progress::current_solved(attempt, scenario));
    let State::Script { source, .. } = &mut attempt.state else {
        panic!()
    };
    source.push_str("printf 'incorrect extra output'\n");
    assert!(
        !progress::current_solved(attempt, scenario),
        "A stale passing record cannot cover an edit"
    );
    let State::Script { source, run } = &mut attempt.state else {
        panic!()
    };
    *run = Some(shell::Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.clone(),
        results: cases.iter().map(|c| shell::run(source, c)).collect(),
    });
    assert!(
        !progress::current_solved(attempt, scenario),
        "Incorrect edited behavior fails after retry too"
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(json(&restored), json(&p));
    let learning = restored.attempts[&scenario.id]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(learning.example_source, example);
    assert_eq!(learning.example_revision, 13);
    assert!(learning.example_result.is_some());
    assert!(!progress::current_solved(
        &restored.attempts[&scenario.id],
        scenario
    ));
    restored.attempts.insert(
        scenario.id.clone(),
        Attempt::new(scenario, restored.clock + 1),
    );
    let reset = json(&restored);
    for (id, before) in original["attempts"].as_object().unwrap() {
        if id != &scenario.id {
            assert_eq!(
                &reset["attempts"][id], before,
                "Only selected attempt resets: {id}"
            );
        }
    }
    assert_eq!(reset["favorites"], original["favorites"]);
    assert!(!progress::current_solved(
        &restored.attempts[&scenario.id],
        scenario
    ));
    progress::save(&path, &restored).unwrap();
    assert_eq!(json(&progress::load(&path, &all).unwrap()), reset);
}
