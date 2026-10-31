//! Prospective private authored gates; no observed outcomes or curriculum credit.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::OnceLock};
#[path = "support/ntp_registration.rs"]
mod registration;
use disk::literal_registration as literal;
use registration::ere_registration::partition_fs_registration as disk;
const DATA: &str = include_str!("../scenarios/shell-snapshot-rollout-draft.json");
const REVIEW: &str = include_str!("fixtures/shell-snapshot-rollout-review.json");
const LITERAL: &str = include_str!("../scenarios/shell-literal-normalization-draft.json");
const LITERAL_REVIEW: &str = include_str!("fixtures/shell-literal-normalization-review.json");
const IDS: [&str; 2] = ["shell-snapshot-audit", "shell-conditional-rollout"];
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn labs() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(DATA).unwrap().scenarios
}
fn item(id: &str) -> Value {
    review()["labs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == id)
        .unwrap()
        .clone()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn texts(all: &[Scenario]) -> Vec<String> {
    all.chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect()
}
fn admit(all: &[Scenario]) -> Result<Vec<Scenario>, String> {
    let chunks = texts(all);
    assert!(chunks.iter().all(|s| s.len() <= pack::MAX_PACK_BYTES));
    pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>())
}
fn before() -> Vec<Scenario> {
    assert_eq!(shell::EVALUATOR_REVISION, 15, "No older evaluator fallback");
    let current = normal199_predecessor();
    assert_eq!(current.len(), 199);
    let old197 = literal::before_scenarios(&current);
    assert_eq!(old197.len(), 197);
    assert_eq!(disk::before_scenarios(old197).len(), 193);
    let expected_literal: Pack = serde_json::from_str(LITERAL).unwrap();
    assert_eq!(value(&current[197..]), value(&expected_literal.scenarios));
    assert_eq!(
        current[197..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        literal::IDS
    );
    // Admit the actual normal199 exactly once; do not append those two again.
    let all = admit(&current).unwrap();
    assert_eq!(all.len(), 199);
    assert_eq!(value(&all), value(&current));
    assert_eq!(value(&all[..197]), value(old197));
    all
}

fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut all = before();
        assert!(labs().iter().all(|s| !all.iter().any(|old| old.id == s.id)));
        all.extend(labs());
        admit(&all).unwrap()
    })
}
fn run(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            assert!(
                vm.outcome.error.is_none(),
                "Behavioral source must use supported code: {:?}; case {:?}; source SHA {}",
                vm.outcome.error,
                c.name,
                dlms_learning_lab::developer_digest::hash(source.as_bytes())
            );
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Bounded authored script did not complete");
}
// Developer-only exact preflight amendment for retained raw authored sources.
// Never substitute inside ordinary run/record or label old-source success current.
fn budget_amended(source: &str) -> String {
    let r = review();
    let rule = &r["budget_preflight_r4"];
    let pairs = rule["replacements"].as_array().unwrap();
    assert_eq!(pairs.len(), 3);
    if !source.contains(pairs[0]["before"].as_str().unwrap()) {
        assert!(!source.contains(pairs[1]["before"].as_str().unwrap()));
        assert!(!source.contains(pairs[2]["before"].as_str().unwrap()));
        return source.into();
    }
    let hash = dlms_learning_lab::developer_digest::hash(source.as_bytes());
    let pin = rule["programs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["before_sha256"].as_str() == Some(hash.as_str()))
        .unwrap();
    let mut derived = source.to_owned();
    let counts = pairs
        .iter()
        .map(|p| source.matches(p["before"].as_str().unwrap()).count())
        .collect::<Vec<_>>();
    assert_eq!(value(&counts), pin["replacement_counts"]);
    for pair in pairs {
        derived = derived.replace(
            pair["before"].as_str().unwrap(),
            pair["after"].as_str().unwrap(),
        );
    }
    assert_eq!(
        dlms_learning_lab::developer_digest::hash(derived.as_bytes()),
        pin["after_sha256"].as_str().unwrap()
    );
    let mut inverse = derived.clone();
    for pair in pairs.iter().rev() {
        inverse = inverse.replace(
            pair["after"].as_str().unwrap(),
            pair["before"].as_str().unwrap(),
        );
    }
    assert_eq!(
        inverse, source,
        "Only fixed membership preflight may change"
    );
    derived
}
fn raw_budget_boundary(source: &str, c: &Case, slice: usize) -> Outcome {
    let r = review();
    let rule = &r["budget_preflight_r4"];
    assert_eq!(
        dlms_learning_lab::developer_digest::hash(source.as_bytes()),
        "f3b40149ab28d9eff829401ca215bde48514b84abcb1d1546a37eac55369da4d"
    );
    assert_eq!(source, rule["raw_reference"].as_str().unwrap());
    assert_eq!(value(c), rule["raw_case"]);
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Fixed raw budget witness did not terminate");
}

fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| run(source, c, 32)).collect(),
    }
}
fn programs(s: &Scenario) -> Vec<String> {
    std::iter::once(s.solution.join("\n"))
        .chain(
            item(&s.id)["alternatives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned()),
        )
        .collect()
}
fn full(p: &Progress) -> Value {
    json!({"portable":p,"receipts":p.migration_receipts})
}

#[test]
fn two_distinct_decisions_preserve_full197_literal_predecessor_and_all_raw_units_privately() {
    let current = normal199_predecessor();
    assert_eq!(current.len(), 199);
    let old197 = literal::before_scenarios(&current);
    let old = before();
    let all = catalog();
    assert_eq!(all.len(), 201, "Private snapshot fixture, not registration");
    assert_eq!(value(&all[..199]), value(&old));
    assert_eq!(value(&all[..199]), value(&current));
    assert_eq!(value(&all[..197]), value(old197));
    assert_eq!(value(&all[199..]), value(&labs()));
    assert_eq!(
        all[199..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        all.iter().map(|s| &s.id).collect::<BTreeSet<_>>().len(),
        all.len()
    );
    let script = current
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(script.len(), 62);
    assert_eq!(script.iter().map(|s| cases(s).len()).sum::<usize>(), 501);
    assert_eq!(script.iter().filter(|s| s.teaching.is_some()).count(), 62);
    let original = old197
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(original.len(), 60);
    assert_eq!(original.iter().map(|s| cases(s).len()).sum::<usize>(), 477);
    assert_eq!(original.iter().filter(|s| s.teaching.is_some()).count(), 60);
    let previous53 =
        literal::snapshot_registration::before_units(include_bytes!("../scenarios/units-v1.json"))
            .unwrap();
    let raw = std::str::from_utf8(&previous53).unwrap();
    let units53 = course::load(raw, &current).unwrap();
    assert_eq!(units53.len(), 53);
    let raw52 = literal::before_units(raw.as_bytes()).unwrap();
    let units52 = course::load(std::str::from_utf8(&raw52).unwrap(), old197).unwrap();
    assert_eq!(units52.len(), 52);
    assert_eq!(
        value(literal::before_typed_units(&units53)),
        value(&units52)
    );
    let raw51 = disk::before_units(&raw52).unwrap();
    let old193 = disk::before_scenarios(old197);
    assert_eq!(
        value(disk::before_typed_units(&units52)),
        value(&course::load(std::str::from_utf8(&raw51).unwrap(), old193).unwrap())
    );
    let mut private: Value = serde_json::from_str(raw).unwrap();
    let literal_review: Value = serde_json::from_str(LITERAL_REVIEW).unwrap();
    assert_eq!(private["units"][52], literal_review["unit_proposal"]);
    private["units"]
        .as_array_mut()
        .unwrap()
        .push(review()["unit_proposal"].clone());
    let units = course::load(&private.to_string(), all).unwrap();
    assert_eq!(units.len(), 54, "Only snapshot unit appended privately");
    assert_eq!(value(&units[..53]), value(&units53));
    assert_eq!(value(&units[..52]), value(&units52));
    assert_eq!(
        units[53].labs,
        IDS.iter().map(|id| id.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(value(&normal199_predecessor()), value(&current));
}

#[test]
fn references_and_two_meaningful_alternatives_match_every_complete_case_and_guard_at_four_slices() {
    for s in labs() {
        let row = item(&s.id);
        assert_eq!(s.solution.join("\n"), row["reference"].as_str().unwrap());
        assert_eq!(row["alternatives"].as_array().unwrap().len(), 2);
        let guards: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
        assert_eq!(cases(&s).len(), 12);
        for c in cases(&s).iter().chain(&guards) {
            assert!(c.valid());
            for source in programs(&s) {
                assert!(source.len() <= shell::MAX_SOURCE);
                for slice in [1, 7, 32, 64] {
                    let actual = run(&source, c, slice);
                    assert!(
                        c.accepts(&actual),
                        "{}/{} expected {:?} actual {:?}; source {:?}",
                        s.id,
                        c.name,
                        c,
                        actual,
                        source
                    );
                }
            }
        }
    }
    catalog();
}

#[test]
fn wrong_partial_and_hardcoded_sources_have_exact_nonvacuous_graded_and_guard_masks() {
    catalog();
    for s in labs() {
        let row = item(&s.id);
        let guards: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
        for wrong in row["wrong"].as_array().unwrap() {
            assert!(wrong["actual"].is_null());
            let source = wrong["source"].as_str().unwrap();
            let expected: Vec<bool> =
                serde_json::from_value(wrong["expected_mask"].clone()).unwrap();
            let guard: Vec<bool> = serde_json::from_value(wrong["guard_mask"].clone()).unwrap();
            assert_eq!(expected.len(), cases(&s).len());
            assert_eq!(guard.len(), guards.len());
            let actual = cases(&s)
                .iter()
                .map(|c| c.accepts(&run(source, c, 7)))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{} {}", s.id, wrong["label"]);
            assert_eq!(
                guards
                    .iter()
                    .map(|c| c.accepts(&run(source, c, 7)))
                    .collect::<Vec<_>>(),
                guard
            );
            assert!(actual.iter().any(|pass| !pass));
            assert!(actual.iter().chain(&guard).any(|pass| *pass));
            assert!(!engine::solved(
                &State::Script {
                    source: source.into(),
                    run: Some(record(&s, source))
                },
                &s.goal
            ));
        }
    }
}

#[test]
fn sentinel_prerequisite_is_taught_and_separate_example_change_repair_never_grants_assignment_credit()
 {
    catalog();
    for s in labs() {
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert_eq!(lesson.concepts.len(), 6);
        assert!(
            lesson.concepts[1]
                .text
                .contains("Command capture removes all final LF")
        );
        let assignment = cases(&s)
            .iter()
            .flat_map(|c| c.files.keys())
            .collect::<BTreeSet<_>>();
        assert!(lesson.fixture.files.keys().all(|p| !assignment.contains(p)));
        assert!(
            lesson
                .fixture
                .accepts(&run(&lesson.example, &lesson.fixture, 7))
        );
        let row = item(&s.id);
        for key in ["try_change", "repair"] {
            let c: Case = serde_json::from_value(row[key]["case"].clone()).unwrap();
            assert!(c.accepts(&run(row[key]["source"].as_str().unwrap(), &c, 7)));
        }
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, lesson, true);
        let assignment = value(&a.state);
        for challenge in [false, true] {
            a.challenge = challenge;
            for view in [View::Introduction, View::Example, View::Assignment] {
                let l = a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .script_learning
                    .as_mut()
                    .unwrap();
                l.view = view;
                l.example_source = lesson.example.clone();
                l.example_revision = shell::EVALUATOR_REVISION;
                l.example_result = Some(run(&l.example_source, &lesson.fixture, 7));
                assert_eq!(value(&a.state), assignment);
                assert!(!progress::current_solved(&a, &s));
            }
        }
    }
}

#[test]
fn current_result_damage_and_source_edits_invalidate_success_without_history_authority() {
    catalog();
    for s in labs() {
        let source = s.solution.join("\n");
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        a.state = State::Script {
            source: source.clone(),
            run: Some(record(&s, &source)),
        };
        assert!(progress::current_solved(&a, &s));
        let correct = a.clone();
        if let State::Script { run: Some(r), .. } = &mut a.state {
            r.results[0]
                .files
                .insert("/work/keep.txt".into(), "Changed protected input".into());
        }
        assert!(!progress::current_solved(&a, &s));
        a = correct.clone();
        let before_history = value(&a);
        workspace::preserve_script_history(&mut a);
        assert_eq!(
            value(&a),
            before_history,
            "A current result is not legacy history"
        );
        if let State::Script { source, run } = &mut a.state {
            source.push_str("# Pending real editor change\n");
            *run = None;
        }
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(
            value(&a.workspace.as_ref().unwrap().script_history),
            value(&correct.workspace.as_ref().unwrap().script_history)
        );
        a = correct;
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn save_resume_and_selected_reset_preserve_the_full_sibling_profile_and_absent_receipts() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        clock: 40,
        ..Default::default()
    };
    let sibling = all.iter().find(|s| s.id == "shell-trim-options").unwrap();
    let mut a = Attempt::new(sibling, 2);
    workspace::prepare(&mut a);
    if let State::Script { source, .. } = &mut a.state {
        source.push_str("# Synthetic unfinished sibling development draft\n");
    }
    p.attempts.insert(sibling.id.clone(), a);
    p.favorites.insert(sibling.id.clone());
    for s in labs() {
        let mut a = Attempt::new(&s, 3);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let source = s.solution.join("\n");
        a.state = State::Script {
            source: source.clone(),
            run: Some(record(&s, &source)),
        };
        assert!(progress::current_solved(&a, &s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id);
    }
    assert!(
        p.migration_receipts.is_none(),
        "Only absent-caller-metadata coverage"
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let caller = full(&p);
    progress::save_with_catalog(&path, &p, all).unwrap();
    assert_eq!(full(&p), caller, "Saving does not stamp caller metadata");
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        saved["migration_receipts"],
        json!({"workspace":true,"shell":[15]})
    );
    let mut warm = progress::load(&path, all).unwrap();
    assert_eq!(
        full(&warm),
        json!({"portable":p,"receipts":saved["migration_receipts"]})
    );
    let before = full(&warm);
    let first = all.iter().find(|s| s.id == IDS[0]).unwrap();
    let visited = warm.attempts[&first.id].visited;
    let mut reset = Attempt::new(first, visited);
    workspace::prepare(&mut reset);
    script_teaching::prepare(&mut reset, first.teaching.as_ref().unwrap(), true);
    warm.attempts.insert(first.id.clone(), reset);
    let after = full(&warm);
    for (id, old) in before["portable"]["attempts"].as_object().unwrap() {
        if id != &first.id {
            assert_eq!(&after["portable"]["attempts"][id], old);
        }
    }
    for key in ["favorites", "clock"] {
        assert_eq!(after["portable"][key], before["portable"][key]);
    }
    assert_eq!(after["receipts"], before["receipts"]);
    assert!(!progress::current_solved(&warm.attempts[&first.id], first));
    progress::save_with_catalog(&path, &warm, all).unwrap();
    assert_eq!(full(&progress::load(&path, all).unwrap()), full(&warm));
}

#[test]
fn source_guards_and_invalid_pack_mutations_reject_without_vacuous_prefix_or_duplicate_failures() {
    let all = catalog();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let s = all.iter().find(|s| s.id == IDS[1]).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    p.attempts.insert(s.id.clone(), a);
    progress::save_with_catalog(&path, &p, all).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let caller = full(&p);
    let mut bad = progress::load(&path, all).unwrap();
    assert_eq!(full(&p), caller);
    let stored: Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(
        stored["migration_receipts"],
        json!({"workspace":true,"shell":[15]})
    );
    assert_eq!(
        full(&bad),
        json!({"portable":p,"receipts":stored["migration_receipts"]})
    );
    bad.attempts.get_mut(&s.id).unwrap().state = all[0].initial.clone();
    assert!(progress::save_with_catalog(&path, &bad, all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    let missing = all
        .iter()
        .filter(|old| old.id != s.id)
        .cloned()
        .collect::<Vec<_>>();
    assert!(progress::save_with_catalog(&path, &p, &missing).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    let old = before();
    for mode in ["prerequisite", "teaching", "solution", "expected"] {
        let mut data: Value = serde_json::from_str(DATA).unwrap();
        let s = &mut data["scenarios"][0];
        match mode {
            "prerequisite" => s["prerequisites"] = json!(["missing-snapshot-prerequisite"]),
            "teaching" => s["teaching"]["example"] = json!("unimplemented-tool"),
            "solution" => s["solution"] = json!(["printf wrong"]),
            "expected" => s["goal"]["cases"][0]["stdout"] = json!("unreachable\n"),
            _ => unreachable!(),
        }
        let mut proposed = old.clone();
        proposed.extend(serde_json::from_value::<Pack>(data).unwrap().scenarios);
        assert_eq!(proposed.len(), 201);
        assert_eq!(
            proposed
                .iter()
                .map(|s| &s.id)
                .collect::<BTreeSet<_>>()
                .len(),
            201
        );
        let error = admit(&proposed).unwrap_err();
        assert!(
            error.starts_with(&format!("{}:", IDS[0])),
            "{mode}: {error}"
        );
    }
}

#[test]
fn authored_sources_remain_within_existing_caps_and_every_prediction_stays_unobserved() {
    catalog();
    let r = review();
    assert!(r["actual"].is_null());
    assert_eq!(r["required_evaluator"], 15);
    assert!(DATA.len() < pack::MAX_PACK_BYTES);
    assert_eq!(pack::MAX_PACK_BYTES, 2_000_000);
    for s in labs() {
        assert_eq!(s.steps.len(), s.guide.len());
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        assert!(pack::DIFFICULTIES.contains(&s.difficulty.as_str()));
        for source in programs(&s) {
            assert!(source.len() <= shell::MAX_SOURCE);
            assert!(
                !source.contains("wc -l") && !source.contains("cmp ") && !source.contains("eval ")
            );
        }
        let row = item(&s.id);
        assert!(row["actual"].is_null());
        for wrong in row["wrong"].as_array().unwrap() {
            assert!(wrong["actual"].is_null());
        }
    }
}

#[test]
fn r1_whole_cases_programs_masks_replay_and_exact_new_counters_fail_grading_without_vm_errors() {
    catalog();
    let r = review();
    let legacy = &r["legacy_r1"];
    assert!(legacy["actual"].is_null());
    let old: Pack = serde_json::from_str(legacy["pack_utf8"].as_str().unwrap()).unwrap();
    let old_review: Value = serde_json::from_str(legacy["review_utf8"].as_str().unwrap()).unwrap();
    assert_eq!(old.scenarios.len(), 2);
    for (index, s) in old.scenarios.iter().enumerate() {
        let row = &old_review["labs"][index];
        let gs: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
        assert_eq!(cases(s).len(), 12);
        assert_eq!(gs.len(), [8, 9][index]);
        assert_eq!(row["wrong"].as_array().unwrap().len(), 6);
        let current = labs().remove(index);
        assert_eq!(
            budget_amended(&s.solution.join("\n")),
            current.solution.join("\n")
        );
        assert_eq!(
            value(
                &row["alternatives"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| budget_amended(v.as_str().unwrap()))
                    .collect::<Vec<_>>()
            ),
            item(&s.id)["alternatives"]
        );
        let mut supported = vec![s.solution.join("\n")];
        supported.extend(
            row["alternatives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned()),
        );
        for source in supported {
            let source = budget_amended(&source);
            for c in cases(s).iter().chain(&gs) {
                assert!(c.accepts(&run(&source, c, 7)));
            }
        }
        for w in row["wrong"].as_array().unwrap() {
            let optimized_source = budget_amended(w["source"].as_str().unwrap());
            let source = optimized_source.as_str();
            let actual = cases(s)
                .iter()
                .map(|c| c.accepts(&run(source, c, 7)))
                .collect::<Vec<_>>();
            let guard = gs
                .iter()
                .map(|c| c.accepts(&run(source, c, 7)))
                .collect::<Vec<_>>();
            assert_eq!(value(&actual), w["expected_mask"]);
            assert_eq!(value(&guard), w["guard_mask"]);
        }
    }
    let witnesses = r["remedy_r2"]["counterexample_programs"]
        .as_array()
        .unwrap();
    assert_eq!(witnesses.len(), 4);
    for w in witnesses {
        let optimized_source = budget_amended(w["source"].as_str().unwrap());
        let source = optimized_source.as_str();
        assert!(w["actual"].is_null());
        let index = if w["lab"] == IDS[0] { 0 } else { 1 };
        let current = labs().remove(index);
        let current_guards: Vec<Case> =
            serde_json::from_value(item(&current.id)["guards"].clone()).unwrap();
        let old = &old.scenarios[index];
        let old_guards: Vec<Case> =
            serde_json::from_value(old_review["labs"][index]["guards"].clone()).unwrap();
        for (cs, predicted, mask) in [
            (cases(old), "original_graded_predictions", "old_graded_mask"),
            (
                &old_guards[..],
                "original_guard_predictions",
                "old_supplemental_guard_mask",
            ),
            (
                cases(&current),
                "current_graded_predictions",
                "current_graded_mask",
            ),
            (
                &current_guards[..],
                "current_guard_predictions",
                "current_guard_mask",
            ),
        ] {
            let expected: Vec<Outcome> = serde_json::from_value(w[predicted].clone()).unwrap();
            assert_eq!(cs.len(), expected.len());
            let mut accepts = Vec::new();
            for (c, expected) in cs.iter().zip(expected) {
                let actual = run(source, c, 7);
                assert_eq!(actual, expected, "{} {}", w["kind"], c.name);
                accepts.push(c.accepts(&actual));
            }
            assert_eq!(value(&accepts), w[mask]);
        }
        assert!(
            w["current_graded_mask"]
                .as_array()
                .unwrap()
                .iter()
                .any(|pass| pass.as_bool() == Some(false))
        );
        for proposed in w["new_counterexamples"].as_array().unwrap() {
            let c: Case = serde_json::from_value(proposed["case"].clone()).unwrap();
            assert!(
                cases(&current).contains(&c) || current_guards.contains(&c),
                "Exact independent Case is preserved; current graded mask independently rejects source"
            );
            let actual = run(source, &c, 7);
            let expected: Outcome = serde_json::from_value(
                json!({"error":null, "stdout":proposed["predicted_wrong"]["stdout"],
                    "stderr":proposed["predicted_wrong"]["stderr"],
                    "status":proposed["predicted_wrong"]["status"],
                    "files":proposed["predicted_wrong"]["files"]}),
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert!(!c.accepts(&actual));
        }
    }
}

#[test]
fn r2_replay_and_complete_input_fourth_field_counter_preserve_independent_grammar_refusal() {
    catalog();
    let r = review();
    let legacy = &r["legacy_r2"];
    assert!(legacy["actual"].is_null());
    let old: Pack = serde_json::from_str(legacy["pack_utf8"].as_str().unwrap()).unwrap();
    let old_review: Value = serde_json::from_str(legacy["review_utf8"].as_str().unwrap()).unwrap();
    for (index, s) in old.scenarios.iter().enumerate() {
        let row = &old_review["labs"][index];
        let guards: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
        assert_eq!(cases(s).len(), 12);
        assert_eq!(guards.len(), [11, 13][index]);
        assert_eq!(row["wrong"].as_array().unwrap().len(), 8);
        for source in std::iter::once(s.solution.join("\n")).chain(
            row["alternatives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned()),
        ) {
            let source = budget_amended(&source);
            for c in cases(s).iter().chain(&guards) {
                assert!(c.accepts(&run(&source, c, 7)));
            }
        }
        for w in row["wrong"].as_array().unwrap() {
            let optimized_source = budget_amended(w["source"].as_str().unwrap());
            let source = optimized_source.as_str();
            assert_eq!(
                value(
                    &cases(s)
                        .iter()
                        .map(|c| c.accepts(&run(source, c, 7)))
                        .collect::<Vec<_>>()
                ),
                w["expected_mask"]
            );
            assert_eq!(
                value(
                    &guards
                        .iter()
                        .map(|c| c.accepts(&run(source, c, 7)))
                        .collect::<Vec<_>>()
                ),
                w["guard_mask"]
            );
        }
    }
    let w = &r["remedy_r3"];
    assert!(w["actual"].is_null());
    let optimized_source = budget_amended(w["source"].as_str().unwrap());
    let source = optimized_source.as_str();
    let current = labs().remove(1);
    let guards: Vec<Case> = serde_json::from_value(item(&current.id)["guards"].clone()).unwrap();
    let old_guards: Vec<Case> =
        serde_json::from_value(old_review["labs"][1]["guards"].clone()).unwrap();
    for (cs, key) in [
        (cases(&old.scenarios[1]), "original_R2_graded_predictions"),
        (&old_guards[..], "original_R2_guard_predictions"),
        (cases(&current), "current_graded_predictions"),
        (&guards[..], "current_guard_predictions"),
    ] {
        let expected: Vec<Outcome> = serde_json::from_value(w[key].clone()).unwrap();
        assert_eq!(cs.len(), expected.len());
        for (c, predicted) in cs.iter().zip(expected) {
            assert_eq!(run(source, c, 7), predicted, "{key}/{}", c.name);
        }
    }
    assert!(
        cases(&old.scenarios[1])
            .iter()
            .all(|c| c.accepts(&run(source, c, 7)))
    );
    assert!(
        cases(&current)
            .iter()
            .any(|c| !c.accepts(&run(source, c, 7)))
    );
    let c: Case = serde_json::from_value(w["proposed_graded_case"].clone()).unwrap();
    assert!(cases(&current).contains(&c));
    let actual = run(source, &c, 7);
    let expected: Outcome = serde_json::from_value(json!({"error":null,
        "stdout":w["predicted_wrong"]["stdout"],"stderr":w["predicted_wrong"]["stderr"],
        "status":w["predicted_wrong"]["status"],"files":w["predicted_wrong"]["files"]}))
    .unwrap();
    assert_eq!(actual, expected);
    assert!(!c.accepts(&actual));
    assert!(c.files.contains_key("/work/want") && c.files.contains_key("/work/live"));
    assert_eq!(
        c.files, c.after,
        "Predictable refusal must preserve full prior world"
    );
}

#[test]
fn raw2000_boundary_is_retained_and_preflight_amendment_uses_existing_quote_safe_case() {
    let r = review();
    let rule = &r["budget_preflight_r4"];
    assert!(rule["actual"].is_null());
    let raw = rule["raw_reference"].as_str().unwrap();
    let c: Case = serde_json::from_value(rule["raw_case"].clone()).unwrap();
    let current = labs().remove(1);
    assert_eq!(value(&c), value(&cases(&current)[0]));
    assert_eq!(budget_amended(raw), current.solution.join("\n"));
    for slice in [1, 7, 32, 64] {
        let boundary = raw_budget_boundary(raw, &c, slice);
        assert_eq!(boundary.error.as_deref(), rule["raw_error"].as_str());
        assert_eq!(boundary.status, 2);
        assert!(!c.accepts(&boundary));
        assert!(c.accepts(&run(&budget_amended(raw), &c, slice)));
    }
    let old = &rule["raw_current_rollout"];
    assert_eq!(
        budget_amended(old["reference"].as_str().unwrap()),
        item(&current.id)["reference"].as_str().unwrap()
    );
    for (before, after) in old["alternatives"]
        .as_array()
        .unwrap()
        .iter()
        .zip(item(&current.id)["alternatives"].as_array().unwrap())
    {
        assert_eq!(
            budget_amended(before.as_str().unwrap()),
            after.as_str().unwrap()
        );
    }
    for (before, after) in old["wrong"]
        .as_array()
        .unwrap()
        .iter()
        .zip(item(&current.id)["wrong"].as_array().unwrap())
    {
        let mut expected = before.clone();
        expected["source"] = budget_amended(before["source"].as_str().unwrap()).into();
        assert_eq!(
            &expected, after,
            "Every mask/label/outcome premise stays exact"
        );
    }
}

fn normal199_predecessor() -> Vec<Scenario> {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 201);
    literal::snapshot_registration::before_scenarios(&all).to_vec()
}
