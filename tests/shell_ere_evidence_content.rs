//! Registered original ERE assignments. All execution is the bounded in-process VM.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use ntp_registration::ere_registration::partition_fs_registration as disk_registration;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "support/ntp_registration.rs"]
mod ntp_registration;
const PACK: &str = include_str!("../scenarios/shell-ere-evidence-v1.json");
const REVIEW: &str = include_str!("fixtures/shell-ere-evidence-review.json");
const IDS: [&str; 3] = [
    "shell-ere-event-view",
    "shell-ere-inventory-review",
    "shell-ere-incident-report",
];
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn full(p: &Progress) -> Value {
    let mut v = value(p);
    v["migration_receipts"] = value(&p.migration_receipts);
    v
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(PACK).unwrap().scenarios
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
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn texts(all: &[Scenario]) -> Vec<String> {
    all.chunks(16)
        .map(|r| json!({"schema_version":1,"scenarios":r}).to_string())
        .collect()
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        assert_eq!(shell::EVALUATOR_REVISION, 14);
        let all = pack::bundled().unwrap();
        assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
        assert_eq!(all.len(), 197);
        let t = texts(&all);
        assert!(t.iter().all(|s| s.len() <= pack::MAX_PACK_BYTES));
        pack::load_catalog(&t.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
    })
}
fn run(p: &str, c: &Case, n: usize) -> Outcome {
    let mut vm = Vm::new(p, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(n)
    }
    panic!("Bounded authored run did not finish")
}
fn record(s: &Scenario, p: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: p.into(),
        results: cases(s).iter().map(|c| shell::run(p, c)).collect(),
    }
}
fn programs(s: &Scenario) -> Vec<String> {
    let mut p = vec![s.solution.join("\n")];
    p.extend(
        item(&s.id)["alternatives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string()),
    );
    assert_eq!(p.iter().collect::<BTreeSet<_>>().len(), 3);
    p
}
#[test]
fn original57_scripts442_cases57_examples_and_unit_bytes_stay_exact_with_new_dependency_closure() {
    let r = review();
    let current = pack::bundled().unwrap();
    let old = ntp_registration::ere_registration::before_scenarios(&current);
    let scripts = old
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 57);
    assert_eq!(scripts.iter().map(|s| cases(s).len()).sum::<usize>(), 442);
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 57);
    let mut pinned = BTreeMap::new();
    for row in r["original_pack_pins"].as_array().unwrap() {
        let name = row["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        let raw =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        assert_eq!(digest::hash(&raw), row["sha256"].as_str().unwrap());
        let v: Pack = serde_json::from_slice(&raw).unwrap();
        for s in v.scenarios {
            if matches!(s.initial, State::Script { .. }) {
                pinned.insert(s.id.clone(), s);
            }
        }
    }
    assert_eq!(pinned.len(), 57);
    for s in scripts {
        assert_eq!(value(s), value(&pinned[&s.id]));
    }
    let units = include_str!("../scenarios/units-v1.json");
    // Exact current193/51→190/50 admission must recover reviewed186/49
    // bytes. The original historical hash below remains unchanged authority.
    let checked50 = ntp_registration::ere_registration::before_units(units.as_bytes()).unwrap();
    let predecessor = ntp_registration::before_units(&checked50).unwrap();
    assert_eq!(
        digest::hash(&predecessor),
        r["original_units_sha256"].as_str().unwrap()
    );
    let full_units = course::bundled(&current).unwrap();
    assert_eq!(disk_registration::before_typed_units(&full_units).len(), 51);
    assert_eq!(full_units.len(), 52);
    let typed_current = ntp_registration::ere_registration::before_typed_units(&full_units);
    let typed_predecessor = course::load(
        std::str::from_utf8(&predecessor).unwrap(),
        ntp_registration::before_scenarios(old),
    )
    .unwrap();
    assert_eq!(
        value(&typed_predecessor),
        value(ntp_registration::before_typed_units(typed_current))
    );
    let all = catalog();
    assert_eq!(value(&all[..old.len()]), value(old));
    assert_eq!(value(&all[old.len()..193]), value(&authored()));
    assert_eq!(
        authored().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let current52 = course::bundled(all).unwrap();
    let typed = disk_registration::before_typed_units(&current52);
    assert_eq!(value(typed.last().unwrap()), r["unit_proposal"]);
}
#[test]
fn three_references_and_two_distinct_alternates_match_complete_outcomes_at_four_work_slices() {
    for s in authored() {
        assert!((6..=shell::MAX_CASES).contains(&cases(&s).len()));
        for p in programs(&s) {
            for c in cases(&s) {
                assert!(c.valid());
                for n in [1, 7, 32, 64] {
                    let outcome = run(&p, c, n);
                    assert!(
                        c.accepts(&outcome),
                        "{}/{}/{}: actual {:?}; expected stdout {:?}, stderr {:?}, status {}, files {:?}; program {:?}",
                        s.id,
                        c.name,
                        n,
                        outcome,
                        c.stdout,
                        c.stderr,
                        c.status,
                        c.after,
                        p
                    )
                }
            }
        }
    }
}
#[test]
fn common_mistakes_partial_work_and_hardcoded_first_case_fail_behavior_not_string_identity() {
    for s in authored() {
        let r = item(&s.id);
        let mut probes = r["wrong"].as_array().unwrap().iter().collect::<Vec<_>>();
        probes.extend([&r["partial"], &r["hardcoded_first_case"]]);
        for p in probes {
            let source = p["source"].as_str().unwrap();
            let rec = record(&s, source);
            for (key, expected) in [
                ("must_pass_case_names", true),
                ("must_fail_case_names", false),
            ] {
                if let Some(names) = p.get(key) {
                    for name in names.as_array().unwrap() {
                        let mut candidate = cases(&s).to_vec();
                        candidate.extend(
                            serde_json::from_value::<Vec<Case>>(r["guards"].clone()).unwrap(),
                        );
                        let c = candidate
                            .iter()
                            .find(|c| c.name == name.as_str().unwrap())
                            .unwrap();
                        assert_eq!(
                            c.accepts(&shell::run(source, c)),
                            expected,
                            "{}/{}",
                            s.id,
                            name
                        );
                    }
                }
            }
            assert!(!engine::solved(
                &State::Script {
                    source: source.into(),
                    run: Some(rec)
                },
                &s.goal
            ));
        }
    }
}
#[test]
fn retained_supplemental_records_test_duplicate_evidence_and_cr_without_inflating_authored_count() {
    for s in authored() {
        for c in item(&s.id)["guards"].as_array().unwrap() {
            let c: Case = serde_json::from_value(c.clone()).unwrap();
            assert!(c.valid());
            for p in programs(&s) {
                assert!(c.accepts(&run(&p, &c, 7)), "{}/{}", s.id, c.name)
            }
        }
    }
}
#[test]
fn examples_change_repair_and_optional_views_preserve_assignment_and_never_award_credit() {
    for s in authored() {
        let l = s.teaching.as_ref().unwrap();
        assert!((4..=6).contains(&l.concepts.len()));
        l.validate().unwrap();
        let r = item(&s.id);
        let changed: Case = serde_json::from_value(r["try_change"]["expected"].clone()).unwrap();
        assert!(changed.accepts(&run(
            r["try_change"]["source"].as_str().unwrap(),
            &changed,
            7
        )));
        assert_eq!(r["try_change"]["repair"], l.example);
        assert!(l.fixture.accepts(&run(&l.example, &l.fixture, 1)));
        let assignment = cases(&s)
            .iter()
            .flat_map(|c| c.files.keys())
            .collect::<BTreeSet<_>>();
        assert!(l.fixture.files.keys().all(|n| !assignment.contains(n)));
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, l, true);
        let initial = value(&a.state);
        for view in [View::Introduction, View::Example, View::Assignment] {
            let ex = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            ex.view = view;
            ex.example_source = r["try_change"]["source"].as_str().unwrap().into();
            ex.example_result = Some(shell::run(&ex.example_source, &l.fixture));
            ex.example_revision = shell::EVALUATOR_REVISION;
            assert_eq!(value(&a.state), initial);
            assert!(!progress::current_solved(&a, &s));
        }
        a.challenge = true;
        assert_eq!(value(&a.state), initial);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn actual_run_edit_invalidation_retry_save_resume_and_selected_reset_preserve_other_work() {
    let all = catalog();
    let new = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 30,
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
        source.push_str("# edited draft\n");
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
            .contains(&14)
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
fn invalid_pack_missing_teaching_prerequisite_solution_and_unreachable_expectations_fail() {
    let current = catalog();
    assert_eq!(current.len(), 197);
    let original193 = disk_registration::before_scenarios(current);
    assert_eq!(original193.len(), 193);
    assert_eq!(
        current[193..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        disk_registration::IDS
    );
    for s in &current[193..] {
        assert!(matches!(
            (&s.initial, &s.goal),
            (State::PartitionFs { .. }, Goal::PartitionFs { .. })
        ));
    }
    let mut original190 = original193.to_vec();
    original190.retain(|s| !IDS.contains(&s.id.as_str()));
    assert_eq!(original190.len(), 190);
    let mut current194 = current.clone();
    current194.retain(|s| !IDS.contains(&s.id.as_str()));
    assert_eq!(current194.len(), 194);
    assert_eq!(value(&current194[..190]), value(&original190));
    assert_eq!(value(&current194[190..]), value(&current[193..]));

    for (base, expected_count) in [(&original190, 193), (&current194, 197)] {
        // A valid control reaches the real validator with both the complete
        // historical base and the current base including all four native labs.
        let mut valid = base.clone();
        valid.extend(authored());
        assert_eq!(valid.len(), expected_count);
        assert_eq!(
            valid.iter().map(|s| &s.id).collect::<BTreeSet<_>>().len(),
            expected_count
        );
        let t = texts(&valid);
        let checked =
            pack::load_catalog(&t.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
        assert_eq!(value(&checked), value(&valid));
        if expected_count == 193 {
            assert_eq!(value(&checked), value(original193));
        } else {
            // Removing/reappending ERE changes only order, never a definition.
            let by_id = |all: &[Scenario]| {
                all.iter()
                    .map(|s| (s.id.clone(), value(s)))
                    .collect::<BTreeMap<_, _>>()
            };
            assert_eq!(by_id(&checked), by_id(current));
        }
        for failure in ["prerequisite", "teaching", "solution", "expected"] {
            let mut p: Value = serde_json::from_str(PACK).unwrap();
            let s = &mut p["scenarios"][0];
            match failure {
                "prerequisite" => s["prerequisites"] = json!(["missing-ere-prerequisite"]),
                "teaching" => s["teaching"]["example"] = json!("unimplemented-tool"),
                "solution" => s["solution"] = json!(["printf wrong"]),
                "expected" => {
                    s["goal"]["cases"][0]["stdout"] = json!("unreachable expected line\n")
                }
                _ => unreachable!(),
            }
            let mut all = base.clone();
            all.extend(serde_json::from_value::<Pack>(p).unwrap().scenarios);
            assert_eq!(all.len(), expected_count);
            assert_eq!(
                all.iter().map(|s| &s.id).collect::<BTreeSet<_>>().len(),
                expected_count
            );
            let t = texts(&all);
            let error =
                pack::load_catalog(&t.iter().map(String::as_str).collect::<Vec<_>>()).unwrap_err();
            let expected = match failure {
                "prerequisite" => "invalid prerequisites",
                "teaching" => "Teaching example does not produce its declared behavior",
                "solution" | "expected" => "solution does not reach objective",
                _ => unreachable!(),
            };
            assert_eq!(
                error,
                format!("{}: {expected}", IDS[0]),
                "{failure} on {expected_count}"
            );
        }
    }
}

#[test]
fn complete_r1_cases_sources_and_negative_premises_survive_the_graded_boundary_additions() {
    let r = review();
    for s in authored() {
        let lab = item(&s.id);
        let old = &lab["retained_R1"];
        let original: Scenario = serde_json::from_value(old["scenario"].clone()).unwrap();
        let mut projected = value(&s);
        projected["goal"] = value(&original.goal);
        assert_eq!(projected, value(&original));
        let current = cases(&s);
        let guards: Vec<Case> = serde_json::from_value(lab["guards"].clone()).unwrap();
        for c in cases(&original) {
            assert!(current.iter().chain(&guards).any(|x| value(x) == value(c)));
        }
        for p in programs(&s) {
            for c in cases(&original).iter().chain(guards.iter()) {
                assert!(c.accepts(&run(&p, c, 7)), "{}/{}", s.id, c.name);
            }
        }
        let oldlab = &old["review_lab"];
        for key in [
            "reference",
            "alternatives",
            "try_change",
            "partial",
            "hardcoded_first_case",
        ] {
            let mut expected = oldlab[key].clone();
            if key == "alternatives" && s.id == IDS[2] {
                let old = expected[1].as_str().unwrap();
                assert_eq!(
                    digest::hash(old.as_bytes()),
                    "9f8a8d074146334f1c1bcce34786bffdffb20d9941f26db05ddab4b82b132a90"
                );
                let old_block = "publish() {\n  printf 'Incident evidence\\n'\n  for input in \"$1\" \"$2\"; do\n  printf 'Source: %s\\n' \"$input\"\n  grep -En -- '^(FAIL|DENY):' \"$input\"\n  done\n}\npublish \"$1\" \"$2\" > \"$3\"\n";
                let new_block = "publish() {\n  printf 'Incident evidence\\n' > \"$3\"\n  for input in \"$1\" \"$2\"; do\n    printf 'Source: %s\\n' \"$input\" >> \"$3\"\n    grep -En -- '^(FAIL|DENY):' \"$input\" >> \"$3\"\n  done\n}\npublish \"$1\" \"$2\" \"$3\"\n";
                assert_eq!(old.matches(old_block).count(), 1);
                expected[1] = json!(old.replacen(old_block, new_block, 1));
            }
            assert_eq!(lab[key], expected);
        }
        let oldwrong = oldlab["wrong"].as_array().unwrap();
        assert_eq!(
            &lab["wrong"].as_array().unwrap()[..oldwrong.len()],
            oldwrong
        );
        let mut negatives = oldwrong.iter().collect::<Vec<_>>();
        negatives.extend([&oldlab["partial"], &oldlab["hardcoded_first_case"]]);
        for p in negatives {
            let src = p["source"].as_str().unwrap();
            for (key, pass) in [
                ("must_pass_case_names", true),
                ("must_fail_case_names", false),
            ] {
                if let Some(names) = p.get(key) {
                    for name in names.as_array().unwrap() {
                        let c = cases(&original)
                            .iter()
                            .find(|c| c.name == name.as_str().unwrap())
                            .unwrap();
                        assert_eq!(c.accepts(&run(src, c, 7)), pass);
                    }
                }
            }
        }
        for c in lab["new_peer_counterexamples"].as_array().unwrap() {
            let src = c["exact_wrong_source"].as_str().unwrap();
            assert_eq!(
                digest::hash(src.as_bytes()),
                c["wrong_source_pin"]["sha256"].as_str().unwrap()
            );
            let oldmask = cases(&original)
                .iter()
                .map(|case| case.accepts(&run(src, case, 7)))
                .collect::<Vec<_>>();
            assert_eq!(value(&oldmask), c["predicted_original_pass_mask"]);
            let newcase: Case = serde_json::from_value(c["proposed_graded_case"].clone()).unwrap();
            assert!(
                current
                    .iter()
                    .chain(&guards)
                    .any(|x| value(x) == value(&newcase))
            );
            let graded: Case = serde_json::from_value(
                c.get("current_graded_case")
                    .unwrap_or(&c["proposed_graded_case"])
                    .clone(),
            )
            .unwrap();
            assert!(current.iter().any(|x| value(x) == value(&graded)));
            let graded_result = run(src, &graded, 7);
            assert!(!graded.accepts(&graded_result));
            let predicted = c
                .get("predicted_wrong_current_outcome")
                .unwrap_or(&c["predicted_wrong_outcome_on_proposed_case"]);
            assert!(graded_result.error.is_none());
            assert_eq!(graded_result.stdout, predicted["stdout"].as_str().unwrap());
            assert_eq!(graded_result.stderr, predicted["stderr"].as_str().unwrap());
            assert_eq!(
                graded_result.status,
                predicted["status"].as_i64().unwrap() as i32
            );
            assert_eq!(value(&graded_result.files), predicted["after"]);
            let result = run(src, &newcase, 7);
            assert!(!newcase.accepts(&result));
            let expected = &c["predicted_wrong_outcome_on_proposed_case"];
            assert!(result.error.is_none());
            assert_eq!(result.stdout, expected["stdout"].as_str().unwrap());
            assert_eq!(result.stderr, expected["stderr"].as_str().unwrap());
            assert_eq!(result.status, expected["status"].as_i64().unwrap() as i32);
            assert_eq!(value(&result.files), expected["after"]);
        }
    }
    assert_eq!(r["required_evaluator"], 14);
    assert_eq!(
        authored()
            .iter()
            .map(|s| cases(s).len())
            .collect::<Vec<_>>(),
        [12, 11, 12]
    );
}

#[test]
fn both_sources_and_first_error_priority_remain_graded_after_info_refinement() {
    let scenario = authored().pop().unwrap();
    let lab = item(&scenario.id);
    let peer = &lab["second_only_counterexample"];
    let old: Vec<Case> = serde_json::from_value(lab["retained_R2_cases"].clone()).unwrap();
    let guards: Vec<Case> = serde_json::from_value(lab["guards"].clone()).unwrap();
    assert_eq!(old.len(), 12);
    assert_eq!(cases(&scenario).len(), 12);
    for c in &old {
        assert!(
            cases(&scenario)
                .iter()
                .chain(&guards)
                .any(|x| value(x) == value(c))
        );
    }
    for p in programs(&scenario) {
        for c in old.iter().chain(&guards) {
            assert!(c.accepts(&run(&p, c, 7)), "{}", c.name);
        }
    }
    let src = peer["exact_wrong_source"].as_str().unwrap();
    assert_eq!(
        digest::hash(src.as_bytes()),
        peer["source_pin"]["sha256"].as_str().unwrap()
    );
    let oldmask = old
        .iter()
        .map(|c| c.accepts(&run(src, c, 7)))
        .collect::<Vec<_>>();
    assert_eq!(value(&oldmask), peer["predicted_current_R2_pass_mask"]);
    for (case_key, expected_key, graded) in [
        (
            "old_R1_graded_witness_now_only_supplemental",
            "wrong_prediction_on_old_witness",
            false,
        ),
        (
            "suggested_replacement_graded_INFO_case",
            "wrong_prediction_on_replacement",
            true,
        ),
    ] {
        let c: Case = serde_json::from_value(peer[case_key].clone()).unwrap();
        if graded {
            assert!(cases(&scenario).iter().any(|x| value(x) == value(&c)));
        } else {
            assert!(guards.iter().any(|x| value(x) == value(&c)));
        }
        let result = run(src, &c, 7);
        assert!(!c.accepts(&result));
        assert!(result.error.is_none());
        let expected = &peer[expected_key];
        assert_eq!(result.stdout, expected["stdout"].as_str().unwrap());
        assert_eq!(result.stderr, expected["stderr"].as_str().unwrap());
        assert_eq!(result.status, expected["status"].as_i64().unwrap() as i32);
        assert_eq!(value(&result.files), expected["after"]);
    }
}

#[test]
fn both_info_sources_are_independently_graded_without_losing_successful_evidence_boundaries() {
    let s = authored().pop().unwrap();
    let lab = item(&s.id);
    let peer = &lab["strict_first_lax_second_counterexample"];
    let old: Vec<Case> = serde_json::from_value(lab["retained_R3_cases"].clone()).unwrap();
    let guards: Vec<Case> = serde_json::from_value(lab["guards"].clone()).unwrap();
    for c in &old {
        assert!(
            cases(&s)
                .iter()
                .chain(&guards)
                .any(|x| value(x) == value(c))
        );
    }
    for p in programs(&s) {
        for c in old.iter().chain(&guards) {
            assert!(c.accepts(&run(&p, c, 7)), "{}", c.name);
        }
    }
    let src = peer["exact_wrong_source"].as_str().unwrap();
    assert_eq!(
        digest::hash(src.as_bytes()),
        peer["source_pin"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        value(
            &old.iter()
                .map(|c| c.accepts(&run(src, c, 7)))
                .collect::<Vec<_>>()
        ),
        peer["predicted_R3_current_pass_mask"]
    );
    let second: Case =
        serde_json::from_value(peer["lost_R2_graded_second_INFO_case_now_supplemental"].clone())
            .unwrap();
    assert!(cases(&s).iter().any(|c| value(c) == value(&second)));
    let result = run(src, &second, 7);
    assert!(!second.accepts(&result));
    assert!(result.error.is_none());
    let expected = &peer["wrong_outcome_on_second_INFO"];
    assert_eq!(result.stdout, expected["stdout"].as_str().unwrap());
    assert_eq!(result.stderr, expected["stderr"].as_str().unwrap());
    assert_eq!(result.status, expected["status"].as_i64().unwrap() as i32);
    assert_eq!(value(&result.files), expected["after"]);
    let positive: Case =
        serde_json::from_value(peer["bounded_consolidation_proposal"].clone()).unwrap();
    assert!(
        cases(&s)
            .iter()
            .chain(&guards)
            .any(|c| value(c) == value(&positive))
    );
    assert_eq!(value(&cases(&s)[0]), lab["current_consolidated_case"]);
    assert!(positive.stdin.is_empty());
    assert!(positive.files["/work/abcdefghijkl.log"].ends_with("FAIL:a:000"));
    assert!(positive.stdout.contains("abcdefghijkl.report"));
    assert!(positive.after["/work/abcdefghijkl.report"].contains("3:FAIL:a:000\n4:FAIL:a:000\n"));
    assert!(
        cases(&s)
            .iter()
            .any(|c| c.name == "Malformed first INFO source wins before malformed second")
    );
    assert!(
        cases(&s)
            .iter()
            .any(|c| c.name == "Malformed second source preserves old report")
    );
    assert_eq!(cases(&s).len(), 12);
}

#[test]
fn valid_digit_hyphen_maximum_names_remain_graded_after_positive_consolidation() {
    let s = authored().pop().unwrap();
    let lab = item(&s.id);
    let peer = &lab["filename_class_counterexample"];
    let old: Vec<Case> = serde_json::from_value(lab["retained_R4_cases"].clone()).unwrap();
    let guards: Vec<Case> = serde_json::from_value(lab["guards"].clone()).unwrap();
    for c in &old {
        assert!(
            cases(&s)
                .iter()
                .chain(&guards)
                .any(|v| value(v) == value(c))
        );
    }
    for p in programs(&s) {
        for c in old.iter().chain(&guards) {
            assert!(c.accepts(&run(&p, c, 7)), "{}", c.name);
        }
    }
    let src = peer["exact_wrong_source"].as_str().unwrap();
    assert_eq!(
        digest::hash(src.as_bytes()),
        peer["source_pin"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        value(
            &old.iter()
                .map(|c| c.accepts(&run(src, c, 7)))
                .collect::<Vec<_>>()
        ),
        peer["predicted_R4_current_pass_mask"]
    );
    let c: Case =
        serde_json::from_value(peer["proposed_name_only_corrected_graded_case"].clone()).unwrap();
    assert_eq!(value(&cases(&s)[0]), value(&c));
    for a in &c.args {
        let stem = a.split('.').next().unwrap();
        assert_eq!(stem.len(), 12);
        assert!(stem.contains('-'));
        assert!(stem.bytes().any(|b| b.is_ascii_digit()));
    }
    let result = run(src, &c, 7);
    assert!(!c.accepts(&result));
    assert!(result.error.is_none());
    let expected = &peer["wrong_prediction_on_corrected_case"];
    assert_eq!(result.stdout, expected["stdout"].as_str().unwrap());
    assert_eq!(result.stderr, expected["stderr"].as_str().unwrap());
    assert_eq!(result.status, expected["status"].as_i64().unwrap() as i32);
    assert_eq!(value(&result.files), expected["after"]);
}

#[test]
fn retained_unsupported_function_redirection_stays_diagnosed_without_publishing() {
    let scenario = authored().pop().unwrap();
    let lab = item(&scenario.id);
    let original = lab["retained_R1"]["review_lab"]["alternatives"][1]
        .as_str()
        .unwrap();
    assert_eq!(
        digest::hash(original.as_bytes()),
        "9f8a8d074146334f1c1bcce34786bffdffb20d9941f26db05ddab4b82b132a90"
    );
    assert_ne!(lab["alternatives"][1].as_str().unwrap(), original);
    let mut observed = 0;
    for c in cases(&scenario).iter().filter(|c| c.status == 0) {
        let o = run(original, c, 7);
        assert!(
            o.error
                .as_ref()
                .is_some_and(|e| e.contains("Unsupported redirection on a function call")),
            "{}: {:?}",
            c.name,
            o
        );
        assert_eq!(o.stdout, "");
        assert_eq!(o.stderr, "");
        assert_eq!(o.status, 2);
        assert_eq!(
            o.files, c.files,
            "Unsupported publication must preserve the existing report and both sources"
        );
        observed += 1;
    }
    assert!(observed >= 2);
}
