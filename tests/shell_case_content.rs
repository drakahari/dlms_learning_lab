//! Authored case unit: whole outcomes, alternate methods and preserved prior work.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use std::collections::BTreeSet;
#[derive(Deserialize)]
struct Draft {
    scenarios: Vec<Scenario>,
}
#[derive(Deserialize)]
struct Review {
    id: String,
    alternates: Vec<String>,
    wrong: Vec<Wrong>,
    partial: String,
    try_change: Change,
    first_case_hardcode: String,
    file_damage: String,
    record_input: Option<RecordInput>,
}
#[derive(Deserialize)]
struct RecordInput {
    argument: usize,
    example_file: String,
    canonical_record_edges: bool,
    output_files: Vec<String>,
    example_output_files: Vec<String>,
}
#[derive(Deserialize)]
struct Wrong {
    name: String,
    source: String,
    misconception: String,
}
#[derive(Deserialize)]
struct Change {
    source: String,
    expected: Case,
    repair: String,
}
#[derive(Deserialize)]
struct Old {
    id: String,
    version: u32,
    unit: String,
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Draft>(include_str!("../scenarios/shell-case-v1.json"))
        .unwrap()
        .scenarios
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/shell-case-content-review.json")).unwrap()
}
fn old_catalog() -> Vec<Scenario> {
    let baseline: Vec<Old> =
        serde_json::from_str(include_str!("fixtures/shell-case-original-catalog.json")).unwrap();
    let ids: BTreeSet<_> = baseline.iter().map(|old| old.id.as_str()).collect();
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| ids.contains(s.id.as_str()))
        .collect()
}
fn catalog() -> Vec<Scenario> {
    pack::bundled().unwrap()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("script kind")
    };
    cases
}
fn outcome(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap_or_else(|e| panic!("{}: {e}", case.name));
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded script failed to finish");
}
fn run(source: &str, s: &Scenario) -> State {
    State::Script {
        source: source.into(),
        run: Some(Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.into(),
            results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
        }),
    }
}
#[test]
fn normal_pack_and_unit_prerequisites_are_real_and_all_three_assignments_are_registered() {
    const {
        assert!(shell::EVALUATOR_REVISION >= 8);
    }
    let all = catalog();
    assert_eq!(all.len(), 140);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|s| s.track == track).count()),
        [42, 28, 24, 46]
    );
    let original = pack::bundled().unwrap();
    let normal_units = course::bundled(&original).unwrap();
    assert_eq!(normal_units.len(), 35);
    let normal_unit = normal_units
        .iter()
        .find(|u| u.id == "shell-case-dispatch")
        .unwrap();
    assert_eq!(
        normal_unit.labs,
        authored().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let mut units = course::bundled(&original).unwrap();
    units.retain(|u| u.id != "shell-case-dispatch");
    let proposed: course::Unit =
        serde_json::from_str(include_str!("fixtures/shell-case-unit-proposal.json")).unwrap();
    assert_eq!(proposed.labs.len(), 3);
    assert!(
        proposed
            .prerequisites
            .iter()
            .all(|id| units.iter().any(|u| &u.id == id))
    );
    units.push(proposed);
    course::load(
        &serde_json::json!({"version":1,"units":units}).to_string(),
        &all,
    )
    .unwrap();
    for s in authored() {
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(s.objective.ends_with("Use the supplied filenames exactly."));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|candidate| &candidate.id == id))
        );
        assert_eq!(s.solution.len(), 1);
        s.teaching.as_ref().unwrap().validate().unwrap();
    }
}
// This guard validates author data only; it does not execute the shell.
fn canonical_records(text: &str) -> bool {
    text.split_terminator('\n')
        .all(|line| line == line.trim_matches([' ', '\t']))
}
#[test]
fn supplied_record_inputs_follow_the_declared_default_ifs_format() {
    assert!(canonical_records(""));
    assert!(canonical_records("\nready:100% ready\nship:path\\name"));
    for invalid in [
        " leading\n",
        "trailing ",
        "\tleading",
        "trailing\t\n",
        " ",
        "\t",
    ] {
        assert!(
            !canonical_records(invalid),
            "edge whitespace escaped the guard"
        );
    }
    let metadata = reviews();
    assert_eq!(
        metadata.iter().filter(|r| r.record_input.is_some()).count(),
        2
    );
    for s in authored() {
        let r = metadata.iter().find(|r| r.id == s.id).unwrap();
        let Some(format) = &r.record_input else {
            assert_eq!(s.id, "shell-case-interface"); // argv/env/raw cat are unrestricted here.
            continue;
        };
        assert!(format.canonical_record_edges);
        assert_eq!(
            format.argument,
            if s.id == "shell-case-routing" { 0 } else { 1 }
        );
        assert_eq!(
            format.output_files,
            if s.id == "shell-case-routing" {
                vec!["/work/priority.txt", "/work/regular.txt"]
            } else {
                vec!["/work/published.txt", "/work/ledger.txt"]
            }
        );
        assert_eq!(
            format.example_output_files,
            if s.id == "shell-case-routing" {
                vec!["/work/fast lane.txt", "/work/normal lane.txt"]
            } else {
                vec!["/work/docket.txt", "/work/loan ledger.txt"]
            }
        );
        let mut supplied = 0;
        for case in cases(&s) {
            if let Some(name) = case
                .args
                .get(format.argument)
                .filter(|name| !name.is_empty())
            {
                let source = format!("/work/{name}");
                assert!(
                    !format.output_files.contains(&source),
                    "{} / {}: source overlaps an output",
                    s.id,
                    case.name
                );
                if let Some(text) = case.files.get(&source) {
                    assert!(
                        canonical_records(text),
                        "{} / {}: noncanonical input",
                        s.id,
                        case.name
                    );
                    supplied += 1;
                }
            }
        }
        assert_eq!(supplied, if s.id == "shell-case-routing" { 8 } else { 9 });
        assert!(
            !format.example_output_files.contains(&format.example_file),
            "example source overlaps an output"
        );
        let lesson = s.teaching.as_ref().unwrap();
        for case in [&lesson.fixture, &r.try_change.expected] {
            let text = case
                .files
                .get(&format.example_file)
                .expect("separate example record input");
            assert!(
                canonical_records(text),
                "{} / {}: noncanonical example",
                s.id,
                case.name
            );
        }
        assert_eq!(
            lesson.fixture.files[&format.example_file],
            r.try_change.expected.files[&format.example_file]
        );
    }
}
#[test]
fn all_references_and_two_distinct_alternates_match_full_outcomes_at_every_slice() {
    for s in authored() {
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(r.alternates.len(), 2);
        for source in std::iter::once(&s.solution[0]).chain(&r.alternates) {
            for c in cases(&s) {
                for slice in [1, 16, 32, 64] {
                    let result = outcome(source, c, slice);
                    assert!(
                        c.accepts(&result),
                        "{} / {} / {slice}: {result:?}",
                        s.id,
                        c.name
                    );
                }
            }
            assert!(engine::solved(&run(source, &s), &s.goal));
        }
        assert!(
            r.alternates.iter().any(|source| !source.contains("case ")),
            "equivalent conditional solution missing"
        );
        if s.id == "shell-case-publication" {
            assert!(r.alternates.iter().any(|source| !source.contains("$(")));
        }
    }
}
#[test]
fn plausible_wrong_partial_hardcodes_and_file_damage_fail_with_useful_feedback() {
    for s in authored() {
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        for wrong in &r.wrong {
            assert!(wrong.misconception.len() > 20);
            let results: Vec<_> = cases(&s)
                .iter()
                .map(|c| shell::run(&wrong.source, c))
                .collect();
            let pass: Vec<_> = cases(&s)
                .iter()
                .zip(&results)
                .map(|(c, r)| c.accepts(r))
                .collect();
            assert!(pass.contains(&false), "{} / {}", s.id, wrong.name);
            assert!(
                results.iter().any(|r| r.error.is_none()),
                "negative must not merely be unsupported syntax"
            );
            if wrong.name == r.partial {
                assert!(pass.contains(&true), "partial must have a real passed case");
            }
            let record = Some(Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: wrong.source.clone(),
                results,
            });
            let feedback = shell::checks(&wrong.source, &record, cases(&s));
            assert!(
                feedback
                    .iter()
                    .any(|(pass, text)| !pass && !text.is_empty())
            );
        }
        let first = &cases(&s)[0];
        assert!(first.accepts(&shell::run(&r.first_case_hardcode, first)));
        assert!(!engine::solved(&run(&r.first_case_hardcode, &s), &s.goal));
        for c in cases(&s) {
            assert!(!c.accepts(&shell::run(&r.file_damage, c)));
        }
    }
}
#[test]
fn separate_working_examples_experiments_and_repairs_have_independent_expected_files() {
    for s in authored() {
        let lesson = s.teaching.as_ref().unwrap();
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        assert_ne!(lesson.example, s.solution[0]);
        assert_eq!(r.try_change.repair, lesson.example);
        assert_ne!(lesson.fixture.files, cases(&s)[0].files);
        for slice in [1, 16, 32, 64] {
            assert!(
                lesson
                    .fixture
                    .accepts(&outcome(&lesson.example, &lesson.fixture, slice))
            );
            let changed = outcome(&r.try_change.source, &r.try_change.expected, slice);
            assert!(r.try_change.expected.accepts(&changed));
            assert!(!lesson.fixture.accepts(&changed));
            assert!(
                lesson
                    .fixture
                    .accepts(&outcome(&r.try_change.repair, &lesson.fixture, slice))
            );
        }
    }
}
#[test]
fn current_source_freshness_invalidates_and_repair_can_restore_completion() {
    for s in authored() {
        let mut state = run(&s.solution[0], &s);
        assert!(engine::solved(&state, &s.goal));
        let State::Script { source, .. } = &mut state else {
            panic!()
        };
        source.push_str("\n# pending edit\n");
        assert!(!engine::solved(&state, &s.goal));
        let State::Script { source, .. } = &state else {
            panic!()
        };
        state = run(source, &s);
        assert!(engine::solved(&state, &s.goal));
        state = run("printf wrong", &s);
        assert!(!engine::solved(&state, &s.goal));
        assert!(engine::solved(&run(&s.solution[0], &s), &s.goal));
    }
}
#[test]
fn practice_preparation_is_source_only_and_both_drafts_resume_then_selected_reset_is_local() {
    let all = catalog();
    let authored = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 50,
        ..Default::default()
    };
    for (i, s) in authored.iter().enumerate() {
        let mut a = Attempt::new(s, i as u64 + 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let applied = a.state.clone();
        let lesson = s.teaching.as_ref().unwrap();
        let learning = a
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap();
        assert_eq!(learning.example_source, lesson.example);
        assert!(learning.example_result.is_none());
        assert_eq!(a.state, applied);
        let w = a.workspace.as_mut().unwrap();
        let learning = w.script_learning.as_mut().unwrap();
        learning
            .example_source
            .push_str("\nprintf 'unfinished example");
        learning.view = View::Example;
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\nprintf 'unfinished assignment");
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec_pretty(&p).unwrap();
    let restored = progress::decode(&bytes, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&p.attempts).unwrap(),
        serde_json::to_value(&restored.attempts).unwrap()
    );
    assert_eq!(p.favorites, restored.favorites);
    let keep = restored.attempts[&authored[1].id].clone();
    let mut reset = restored;
    reset
        .attempts
        .insert(authored[0].id.clone(), Attempt::new(&authored[0], 100));
    assert_eq!(
        serde_json::to_value(&keep).unwrap(),
        serde_json::to_value(&reset.attempts[&authored[1].id]).unwrap()
    );
    assert_eq!(reset.favorites, p.favorites);
}
#[test]
fn original_catalog_ids_versions_and_real_current_states_survive_source_only_unit_operations() {
    let baseline: Vec<Old> =
        serde_json::from_str(include_str!("fixtures/shell-case-original-catalog.json")).unwrap();
    assert_eq!(baseline.len(), 106);
    let old = old_catalog();
    let all = catalog();
    for id in baseline {
        assert!(
            old.iter()
                .any(|s| s.id == id.id && s.version == id.version && s.unit == id.unit)
        );
    }
    let mut p = Progress {
        schema_version: 1,
        clock: 100,
        ..Default::default()
    };
    let mut script_records = 0;
    for (i, s) in old.iter().enumerate() {
        let mut a = Attempt::new(s, i as u64 + 1);
        if matches!(&s.goal, Goal::Script { .. }) {
            a.state = run(&s.solution[0], s);
            script_records += 1;
        } else {
            for command in &s.solution {
                engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
            }
        }
        assert!(engine::solved(&a.state, &s.goal));
        workspace::prepare(&mut a);
        if let Some(lesson) = &s.teaching {
            script_teaching::prepare(&mut a, lesson, true);
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            l.example_result = Some(shell::run(&lesson.example, &lesson.fixture));
            l.example_revision = 8;
            if s.id == "shell-arguments" {
                l.example_source.push_str("\nprintf 'unfinished example");
            }
        }
        if s.id == "shell-handover" {
            let State::Script { source, .. } = &mut a.state else {
                panic!()
            };
            source.push_str("\nprintf 'unfinished assignment");
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    assert_eq!(script_records, 37);
    let before = serde_json::to_value(&p).unwrap();
    for s in authored() {
        assert!(engine::solved(&run(&s.solution[0], &s), &s.goal));
        let l = s.teaching.unwrap();
        assert!(l.fixture.accepts(&shell::run(&l.example, &l.fixture)));
    }
    let loaded = progress::decode(&serde_json::to_vec(&p).unwrap(), &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), before);
    let unique: BTreeSet<_> = loaded.attempts.keys().collect();
    assert_eq!(unique.len(), old.len());
    // These are genuine newly run current8 records, never relabeled as6/7.
    // Actual6+7 migration/history is the separate runtime acceptance suite.
}
