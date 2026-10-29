//! Authored capture-unit review. Fixed synthetic inputs execute only in the native VM.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell, workspace,
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
    expected: shell::Case,
    repair: String,
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Draft>(include_str!("../scenarios/shell-capture-v1.json"))
        .unwrap()
        .scenarios
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/shell-capture-content-review.json")).unwrap()
}
fn catalog() -> Vec<Scenario> {
    let mut all: Vec<_> = pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit != "shell-command-capture")
        .collect();
    all.extend(authored());
    // The catalog is several independently bounded packs; growing content
    // must not turn this fixture into one oversized artificial pack.
    let before = serde_json::to_value(&all).unwrap();
    let packs = all
        .chunks(16)
        .map(|rows| serde_json::json!({"schema_version":1,"scenarios":rows}).to_string())
        .collect::<Vec<_>>();
    assert!(packs.iter().all(|text| text.len() <= pack::MAX_PACK_BYTES));
    let loaded = pack::load_catalog(&packs.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        before,
        "Chunking retains every typed definition and original fixture order"
    );
    loaded
}
fn cases(s: &Scenario) -> &[shell::Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn run(source: &str, s: &Scenario) -> State {
    State::Script {
        source: source.into(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.into(),
            results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
        }),
    }
}
fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[test]
fn authored_pack_is_reachable_and_its_actual_unit_and_lesson_prerequisites_are_valid() {
    const {
        assert!(
            shell::EVALUATOR_REVISION >= 7,
            "requires the supported command-substitution generation; current records use their actual evaluator"
        );
    };
    let bundled = pack::bundled().unwrap();
    let all = catalog();
    let mut units = course::bundled(&bundled).unwrap();
    units.retain(|u| u.id != "shell-command-capture");
    let proposed: course::Unit =
        serde_json::from_str(include_str!("fixtures/shell-capture-unit-proposal.json")).unwrap();
    let previous: BTreeSet<_> = units.iter().map(|u| u.id.as_str()).collect();
    assert!(
        proposed
            .prerequisites
            .iter()
            .all(|id| previous.contains(id.as_str()))
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
        assert!(!s.objective.contains("canonical"));
        assert!(!s.objective.contains("aliases"));
        assert!(!s.objective.contains("hyphen"));
        if s.id == "shell-capture-handover" {
            assert_eq!(s.objective.matches("\n\n").count(), 2);
        }
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|candidate| &candidate.id == id))
        );
        assert!(engine::solved(&run(&s.solution[0], &s), &s.goal));
        let t = s.teaching.as_ref().unwrap();
        t.validate().unwrap();
        assert!(t.concepts.iter().all(|c| c.text.len() <= 1200));
    }
}

#[test]
fn every_reference_and_two_substantive_alternates_pass_whole_outcomes_including_a_non_capture_capstone()
 {
    for s in authored() {
        let review = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(review.alternates.len(), 2);
        for source in std::iter::once(&s.solution[0]).chain(&review.alternates) {
            assert!(engine::solved(&run(source, &s), &s.goal), "{}", s.id);
            for case in cases(&s) {
                let result = shell::run(source, case);
                assert!(
                    case.accepts(&result),
                    "{} / {}: {:?}",
                    s.id,
                    case.name,
                    result
                );
            }
        }
        if s.id == "shell-capture-handover" {
            assert!(!review.alternates[1].contains("$("));
        }
    }
}

#[test]
fn plausible_wrong_programs_have_specific_failures_and_each_task_has_meaningful_partial_feedback() {
    for s in authored() {
        let review = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        for wrong in &review.wrong {
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
            assert!(!engine::solved(&run(&wrong.source, &s), &s.goal));
            if wrong.name == review.partial {
                assert!(pass.contains(&true), "{} / {pass:?}", s.id);
            }
        }
    }
}

#[test]
fn first_case_hardcodes_and_unrelated_file_damage_cannot_earn_completion() {
    for s in authored() {
        let first = &cases(&s)[0];
        let mut source = format!(
            "printf '%s' {}\nprintf '%s' {} >&2\n",
            quote(&first.stdout),
            quote(&first.stderr)
        );
        for (file, text) in &first.after {
            if first.files.get(file) != Some(text) {
                source.push_str(&format!("printf '%s' {} > {}\n", quote(text), quote(file)));
            }
        }
        source.push_str(&format!("exit {}\n", first.status));
        assert!(first.accepts(&shell::run(&source, first)));
        assert!(!engine::solved(&run(&source, &s), &s.goal));
        let damage = format!("printf damaged > unrelated-damage.txt\n{}", s.solution[0]);
        for case in cases(&s) {
            assert!(!case.accepts(&shell::run(&damage, case)));
        }
        let mut good = run(&s.solution[0], &s);
        assert!(engine::solved(&good, &s.goal));
        let State::Script { source, .. } = &mut good else {
            panic!()
        };
        source.push_str("\n# own unsaved edit");
        assert!(
            !engine::solved(&good, &s.goal),
            "edited source needs its own run"
        );
    }
}

#[test]
fn all_examples_concrete_try_changes_and_repairs_are_exact_and_assignment_independent() {
    for s in authored() {
        let t = s.teaching.as_ref().unwrap();
        let change = reviews()
            .into_iter()
            .find(|r| r.id == s.id)
            .unwrap()
            .try_change;
        assert_ne!(change.source, t.example);
        assert_eq!(change.repair, t.example);
        assert!(t.fixture.accepts(&shell::run(&t.example, &t.fixture)));
        assert!(
            change
                .expected
                .accepts(&shell::run(&change.source, &change.expected))
        );
        assert!(t.fixture.accepts(&shell::run(&change.repair, &t.fixture)));
        assert_eq!(change.expected.files, t.fixture.files);
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, t, true);
        let before = a.state.clone();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        // This is the model boundary used by filling the example: source only, no implicit Run.
        learning.example_source = t.example.clone();
        assert!(learning.example_result.is_none());
        learning.example_source = change.source;
        learning.example_result = Some(shell::run(&learning.example_source, &t.fixture));
        learning.view = View::Example;
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
    }
}

#[test]
fn missing_source_status_two_is_an_explicit_helper_return_with_no_simulator_error() {
    let s = authored()
        .into_iter()
        .find(|s| s.id == "shell-capture-decision")
        .unwrap();
    let missing = cases(&s).iter().find(|c| c.status == 2).unwrap();
    let result = shell::run(&s.solution[0], missing);
    assert_eq!(result.status, 2);
    assert!(result.error.is_none());
    assert!(result.stdout.is_empty());
    assert_eq!(
        result.stderr,
        "Inspecting: absent.txt\nMissing source: absent.txt\n"
    );
    assert_eq!(result.files, missing.files);
}

#[test]
fn existing_attempts_both_new_drafts_results_and_selected_reset_survive_a_temporary_profile_roundtrip()
 {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old: Vec<_> = all
        .iter()
        .filter(|s| s.unit != "shell-command-capture")
        .collect();
    for s in &old {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    p.clock = 317;
    p.favorites.extend(old.iter().take(3).map(|s| s.id.clone()));
    let prior = serde_json::to_value(&p.attempts).unwrap();
    for s in authored() {
        let mut a = Attempt::new(&s, 2);
        a.state = run(&s.solution[0], &s);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        let change = reviews()
            .into_iter()
            .find(|r| r.id == s.id)
            .unwrap()
            .try_change;
        learning.example_source = change.source;
        learning.example_result = Some(shell::run(
            &learning.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        learning.view = View::Assignment;
        assert!(progress::current_solved(&a, &s));
        if s.id == "shell-capture-evidence" {
            let State::Script { source, run } = &mut a.state else {
                panic!()
            };
            source.push_str("\nprintf 'unfinished learner edit");
            *run = None;
            assert!(!progress::current_solved(&a, &s));
        }
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(loaded.clock, p.clock);
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let selected = authored().remove(0);
    let others: Vec<_> = loaded
        .attempts
        .iter()
        .filter(|(id, _)| *id != &selected.id)
        .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
        .collect();
    loaded
        .attempts
        .insert(selected.id.clone(), Attempt::new(&selected, 3));
    assert!(!progress::current_solved(
        &loaded.attempts[&selected.id],
        &selected
    ));
    for (id, bytes) in others {
        assert_eq!(serde_json::to_vec(&loaded.attempts[&id]).unwrap(), bytes);
    }
    for s in authored() {
        loaded.attempts.remove(&s.id);
    }
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), prior);
}
