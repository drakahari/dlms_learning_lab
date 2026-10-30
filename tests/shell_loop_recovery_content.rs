//! Unregistered two-task authoring proposal; actual in-process tests, never host execution.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "support/lvm_registration.rs"]
mod lvm_registration;
const PACK: &str = include_str!("../scenarios/shell-loop-recovery-v1.json");
const REVIEW: &str = include_str!("fixtures/shell-loop-recovery-review.json");
const IDS: [&str; 2] = ["shell-loop-intake", "shell-loop-fallback"];
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn item(id: &str) -> Value {
    review()["labs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap()
        .clone()
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(PACK).unwrap().scenarios
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn bounded(all: Vec<Scenario>) -> Vec<Scenario> {
    let texts = all
        .chunks(16)
        .map(|rows| json!({"schema_version":1,"scenarios":rows}).to_string())
        .collect::<Vec<_>>();
    assert!(texts.iter().all(|s| s.len() <= pack::MAX_PACK_BYTES));
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        assert_eq!(shell::EVALUATOR_REVISION, 13);
        let all = pack::bundled().unwrap();
        assert_eq!(
            lvm_registration::ntp_registration::before_scenarios(&all).len(),
            186
        );
        assert_eq!(all.len(), 190);
        assert_eq!(value(&all[180..182]), value(&authored()));
        bounded(all)
    })
}
fn run(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice)
    }
    panic!("Finite authored script exceeded bounded polling")
}
fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn rows(s: &Scenario, source: &str) -> Vec<bool> {
    cases(s)
        .iter()
        .zip(record(s, source).results)
        .map(|(c, r)| c.accepts(&r))
        .collect()
}
fn expected(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_bool().unwrap())
        .collect()
}
fn full(p: &Progress) -> Value {
    let mut v = value(p);
    v["migration_receipts"] = value(&p.migration_receipts);
    v
}

#[test]
fn two_distinct_unregistered_tasks_preserve_all55_original_script_objects_and420_cases() {
    let current = pack::bundled().unwrap();
    let prior182 = lvm_registration::before_scenarios(&current);
    let old = prior182[..180].to_vec();
    let r = review();
    assert_eq!(
        (
            r["existing_script_count"].as_u64(),
            r["existing_case_count"].as_u64(),
            r["existing_example_count"].as_u64()
        ),
        (Some(55), Some(420), Some(55))
    );
    let mut pinned = BTreeMap::new();
    for pin in r["original_pack_pins"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let info = std::fs::symlink_metadata(&path).unwrap();
        assert!(
            info.is_file()
                && !info.file_type().is_symlink()
                && info.len() <= pack::MAX_PACK_BYTES as u64
        );
        let raw = std::fs::read(path).unwrap();
        assert_eq!(digest::hash(&raw), pin["sha256"].as_str().unwrap());
        let v: Value = serde_json::from_slice(&raw).unwrap();
        for s in v["scenarios"].as_array().unwrap() {
            pinned.insert(s["id"].as_str().unwrap().to_string(), s.clone());
        }
    }
    let mut ids = BTreeSet::new();
    for pin in r["existing_scripts"].as_array().unwrap() {
        let id = pin["id"].as_str().unwrap();
        assert!(ids.insert(id));
        let raw = &pinned[id];
        assert_eq!(
            digest::hash(&serde_json::to_vec(raw).unwrap()),
            pin["whole_scenario_sha256"].as_str().unwrap()
        );
        let expected: Scenario = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(
            value(old.iter().find(|s| s.id == id).unwrap()),
            value(&expected)
        );
    }
    assert_eq!(ids.len(), 55);
    assert_eq!(
        old.iter()
            .filter(|s| matches!(s.initial, State::Script { .. }))
            .count(),
        55
    );
    let proposed = authored();
    assert_eq!(
        proposed.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        proposed.iter().map(|s| cases(s).len()).collect::<Vec<_>>(),
        [10, 12]
    );
    let normal = catalog();
    assert_eq!(
        value(normal),
        value(&current),
        "Full current catalog JSON roundtrip"
    );
    let current_units = course::bundled(normal).unwrap();
    let current_roundtrip = course::load(
        &json!({"version":1,"units":current_units}).to_string(),
        normal,
    )
    .unwrap();
    assert_eq!(value(&current_roundtrip), value(&current_units));
    // Project only after validating every new LVM source/unit against the
    // complete canonical append; retain the original Loop182 assertions below.
    let all = lvm_registration::before_scenarios(normal);
    assert_eq!(value(&all[..old.len()]), value(&old));
    assert_eq!(value(&all[old.len()..]), value(&proposed));
    let raw48 =
        lvm_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let units = course::load(std::str::from_utf8(&raw48).unwrap(), all).unwrap();
    assert_eq!(
        value(&units),
        value(lvm_registration::before_typed_units(&current_units))
    );
    assert_eq!(value(units.last().unwrap()), r["unit_proposal"]);
    let units = course::load(&json!({"version":1,"units":units}).to_string(), all).unwrap();
    assert_eq!(units.last().unwrap().labs, IDS);
}
#[test]
fn references_and_two_distinct_supported_alternates_pass22_complete_behavior_cases_at_four_slices()
{
    for s in authored() {
        let r = item(&s.id);
        let source = s.solution.join("\n");
        let alts = r["alternates"].as_array().unwrap();
        assert_eq!(alts.len(), 2);
        let programs = std::iter::once(source.as_str())
            .chain(alts.iter().map(|s| s.as_str().unwrap()))
            .collect::<BTreeSet<_>>();
        assert_eq!(programs.len(), 3);
        for p in programs {
            for c in cases(&s) {
                assert!(c.valid());
                for slice in [1, 7, 32, 64] {
                    let result = run(p, c, slice);
                    assert!(
                        c.accepts(&result),
                        "{}/{} slice{slice}: {result:?}",
                        s.id,
                        c.name
                    );
                }
            }
        }
    }
}
#[test]
fn wrong_partial_hardcoded_and_file_damage_programs_have_exact_authored_masks_and_never_complete() {
    for s in authored() {
        let r = item(&s.id);
        let mut probes = r["wrong"].as_array().unwrap().iter().collect::<Vec<_>>();
        probes.extend([&r["partial"], &r["first_case_hardcode"], &r["file_damage"]]);
        for p in probes {
            let source = p["source"].as_str().unwrap();
            let actual = rows(&s, source);
            assert_eq!(
                actual,
                expected(&p["expected_rows"]),
                "{} {:?}",
                s.id,
                p.get("name")
            );
            assert!(actual.iter().any(|p| !*p));
            let state = State::Script {
                source: source.into(),
                run: Some(record(&s, source)),
            };
            assert!(!dlms_learning_lab::engine::solved(&state, &s.goal));
        }
    }
}
#[test]
fn overflow_missing_arguments_path_and_non_lf_supplemental_guards_use_full_outcome_checks() {
    for s in authored() {
        let r = item(&s.id);
        let mut programs = vec![s.solution.join("\n")];
        programs.extend(
            r["alternates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p.as_str().unwrap().into()),
        );
        for guard in r["guard_cases"].as_array().unwrap() {
            let c: Case = serde_json::from_value(guard.clone()).unwrap();
            assert!(c.valid());
            for p in &programs {
                for n in [1, 64] {
                    assert!(c.accepts(&run(p, &c, n)), "{}/{}", s.id, c.name);
                }
            }
        }
    }
}
#[test]
fn introduction_example_change_and_repair_are_separate_from_assignment_and_never_award_credit() {
    for s in authored() {
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(lesson.concepts.len(), 6);
        lesson.validate().unwrap();
        let r = item(&s.id);
        let change = &r["try_change"];
        let changed: Case = serde_json::from_value(change["expected"].clone()).unwrap();
        assert!(changed.valid());
        assert!(changed.accepts(&shell::run(change["source"].as_str().unwrap(), &changed)));
        assert_eq!(change["repair"], lesson.example);
        assert!(lesson.fixture.accepts(&shell::run(
            change["repair"].as_str().unwrap(),
            &lesson.fixture
        )));
        let assignment_files = cases(&s)
            .iter()
            .flat_map(|c| c.files.keys().cloned())
            .collect::<BTreeSet<_>>();
        assert!(
            lesson
                .fixture
                .files
                .keys()
                .all(|n| !assignment_files.contains(n))
        );
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, lesson, true);
        let assignment = value(&a.state);
        for view in [View::Introduction, View::Example, View::Assignment] {
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            l.view = view;
            l.example_source = change["source"].as_str().unwrap().into();
            l.example_result = Some(shell::run(&l.example_source, &lesson.fixture));
            l.example_revision = shell::EVALUATOR_REVISION;
            assert_eq!(value(&a.state), assignment);
            assert!(!progress::current_solved(&a, &s));
        }
        a.challenge = true;
        assert_eq!(value(&a.state), assignment);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn actual_runs_edit_invalidation_retry_save_resume_and_selected_reset_keep_other_work() {
    let all = catalog();
    let new = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 22,
        ..Default::default()
    };
    for s in &new {
        let mut a = Attempt::new(s, 10);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let source = s.solution.join("\n");
        a.state = State::Script {
            source: source.clone(),
            run: Some(record(s, &source)),
        };
        assert!(progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let first = &new[0];
    let a = p.attempts.get_mut(&first.id).unwrap();
    if let State::Script { source, .. } = &mut a.state {
        source.push_str("# new editor text\n");
    }
    assert!(!progress::current_solved(a, first));
    let wrong = item(&first.id)["wrong"][0]["source"]
        .as_str()
        .unwrap()
        .to_string();
    a.state = State::Script {
        source: wrong.clone(),
        run: Some(record(first, &wrong)),
    };
    assert!(!progress::current_solved(a, first));
    let source = first.solution.join("\n");
    a.state = State::Script {
        source: source.clone(),
        run: Some(record(first, &source)),
    };
    assert!(progress::current_solved(a, first));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let caller = full(&p);
    progress::save(&path, &p).unwrap();
    assert_eq!(full(&p), caller);
    let mut resumed = progress::load(&path, all).unwrap();
    assert_eq!(value(&resumed), value(&p));
    assert!(
        resumed
            .migration_receipts
            .as_ref()
            .unwrap()
            .shell
            .contains(&13)
    );
    let before = full(&resumed);
    let visited = resumed.attempts[&first.id].visited;
    let mut reset = Attempt::new(first, visited);
    workspace::prepare(&mut reset);
    script_teaching::prepare(&mut reset, first.teaching.as_ref().unwrap(), true);
    resumed.attempts.insert(first.id.clone(), reset);
    let after = full(&resumed);
    for (id, old) in before["attempts"].as_object().unwrap() {
        if id != &first.id {
            assert_eq!(&after["attempts"][id], old);
        }
    }
    for key in ["favorites", "clock", "migration_receipts"] {
        assert_eq!(before[key], after[key]);
    }
    assert!(!progress::current_solved(
        &resumed.attempts[&first.id],
        first
    ));
    assert!(progress::current_solved(
        &resumed.attempts[&new[1].id],
        &new[1]
    ));
    progress::save(&path, &resumed).unwrap();
    assert_eq!(full(&progress::load(&path, all).unwrap()), full(&resumed));
}
#[test]
fn active_loop_stop_is_bounded_and_does_not_create_an_assignment_record() {
    let s = &authored()[0];
    let c = &cases(s)[0];
    let mut vm = Vm::new("while true; do continue; done", c).unwrap();
    vm.tick(7);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    assert!(matches!(a.state, State::Script { run: None, .. }));
    assert!(!progress::current_solved(&a, s));
}

#[test]
fn invalid_authoring_prerequisites_teaching_and_unreachable_outcomes_fail_after_valid_control() {
    assert_eq!(catalog().len(), pack::bundled().unwrap().len());
    for failure in [
        "title",
        "hints",
        "prerequisite",
        "solution",
        "expected",
        "example",
    ] {
        let mut new = value(&serde_json::from_str::<Pack>(PACK).unwrap());
        let first = &mut new["scenarios"][0];
        match failure {
            "title" => first["title"] = json!(""),
            "hints" => first["hints"] = json!([]),
            "prerequisite" => first["prerequisites"] = json!(["missing-real-prerequisite"]),
            "solution" => first["solution"] = json!(["printf wrong"]),
            "expected" => {
                first["goal"]["cases"][0]["stdout"] = json!("incorrect declared outcome\n")
            }
            "example" => first["teaching"]["example"] = json!("unimplemented-tool"),
            _ => unreachable!(),
        }
        let broken: Pack = serde_json::from_value(new).unwrap();
        let mut all = pack::bundled().unwrap()[..180].to_vec();
        all.extend(broken.scenarios);
        let texts = all
            .chunks(16)
            .map(|rows| json!({"schema_version":1,"scenarios":rows}).to_string())
            .collect::<Vec<_>>();
        assert!(texts.iter().all(|s| s.len() <= pack::MAX_PACK_BYTES));
        assert!(
            pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).is_err(),
            "{failure}"
        );
    }
}

#[test]
fn r1_complete_cases_negative_masks_and_counterexample_premises_survive_the_two_graded_replacements()
 {
    let s = &authored()[1];
    let r = item(&s.id);
    let retained = &r["retained_R1_coverage"];
    let old: Vec<Case> = serde_json::from_value(retained["cases"].clone()).unwrap();
    let guards: Vec<Case> = serde_json::from_value(retained["guard_cases"].clone()).unwrap();
    assert_eq!((old.len(), guards.len(), cases(s).len()), (12, 6, 12));
    for (index, c) in old.iter().enumerate() {
        if index != 7 && index != 8 {
            assert_eq!(value(c), value(&cases(s)[index]));
        }
    }
    let all_guards: Vec<Case> = serde_json::from_value(r["guard_cases"].clone()).unwrap();
    assert_eq!(all_guards.len(), 8);
    assert_eq!(value(&all_guards[..6]), value(&guards));
    assert_eq!(
        value(&all_guards[6..]),
        value(&[old[7].clone(), old[8].clone()])
    );
    for p in retained["programs"].as_array().unwrap() {
        let source = p["source"].as_str().unwrap();
        let mask = expected(&p["expected_rows"]);
        assert_eq!(mask.len(), 12);
        for n in [1, 64] {
            let observed = old
                .iter()
                .map(|c| c.accepts(&run(source, c, n)))
                .collect::<Vec<_>>();
            assert_eq!(observed, mask, "Retained {:?} slice{n}", p["name"]);
        }
    }
    for p in retained["counterexamples_original_guard_checks"]
        .as_array()
        .unwrap()
    {
        let source = p["source"].as_str().unwrap();
        let mask = expected(&p["expected_rows"]);
        for n in [1, 64] {
            assert_eq!(
                guards
                    .iter()
                    .map(|c| c.accepts(&run(source, c, n)))
                    .collect::<Vec<_>>(),
                mask
            );
        }
    }
    assert!(!cases(s)[7].files.contains_key("/work/missing.result"));
    assert!(!cases(s)[7].after.contains_key("/work/missing.result"));
    assert_eq!(cases(s)[8].args, ["2", "a", "a-1", "abcd1234"]);
    for (id, sha, index) in [
        (
            "UNAVAILABLE-CREATES-ABSENT-RESULT",
            "d712629d95b3ee3875fab8fb428dd262a4b92804da94f76205d1d7e8ffc9882e",
            7,
        ),
        (
            "VALID-ID-DOMAIN-REJECTED",
            "3a557b180abad99ca464346940ebb699647808a7612d30feef45f9ce692d3584",
            8,
        ),
    ] {
        let bad = r["wrong"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == id)
            .unwrap();
        let source = bad["source"].as_str().unwrap();
        assert_eq!(digest::hash(source.as_bytes()), sha);
        let mask = expected(&bad["expected_rows"]);
        assert_eq!(mask.iter().filter(|p| !**p).count(), 1);
        assert!(!mask[index]);
        for n in [1, 64] {
            let actual = run(source, &cases(s)[index], n);
            assert!(!cases(s)[index].accepts(&actual));
        }
    }
}
