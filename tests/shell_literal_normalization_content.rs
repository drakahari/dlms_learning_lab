//! Conditional unregistered teaching tests. Require actual evaluator15 first.
//! Every result is computed by the bounded in-process VM; no host shell route.
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
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk_registration;
use std::{collections::BTreeSet, sync::OnceLock};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PACK: &str = include_str!("../scenarios/shell-literal-normalization-draft.json");
const REVIEW: &str = include_str!("fixtures/shell-literal-normalization-review.json");
const IDS: [&str; 2] = ["shell-trim-options", "shell-trim-manifest"];
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
        .find(|x| x["id"] == id)
        .unwrap()
        .clone()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("Authored script goal required")
    };
    cases
}
fn programs(s: &Scenario) -> Vec<String> {
    let mut v = vec![s.solution.join("\n")];
    v.extend(
        item(&s.id)["alternatives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string()),
    );
    assert_eq!(v.iter().collect::<BTreeSet<_>>().len(), 3);
    v
}
fn texts(v: &[Scenario]) -> Vec<String> {
    v.chunks(16)
        .map(|x| json!({"schema_version":1,"scenarios":x}).to_string())
        .collect()
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        assert_eq!(
            shell::EVALUATOR_REVISION,
            15,
            "No fallback to an older evaluator"
        );
        let mut all = pack::bundled().unwrap();
        assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
        assert_eq!(all.len(), 197, "Unregistered development fixture base only");
        assert!(!all.iter().any(|x| IDS.contains(&x.id.as_str())));
        all.extend(authored());
        let t = texts(&all);
        assert!(t.iter().all(|v| v.len() <= pack::MAX_PACK_BYTES));
        pack::load_catalog(&t.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
    })
}
fn run(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Authored finite program failed its bounded completion premise")
}
fn supported_run(source: &str, c: &Case, slice: usize) -> Outcome {
    let out = run(source, c, slice);
    assert!(
        out.error.is_none(),
        "Behavioral authored source must execute supported code: {:?}; source {:?}",
        out.error,
        source
    );
    out
}
// Developer-only amendment of the exact archived authored sources. Original
// unsupported bytes remain provenance, never falsely accepted observations.
fn guard_amended(source: &str) -> String {
    let r = review();
    let pairs = r["guard_replacements_r4"].as_array().unwrap();
    assert_eq!(pairs.len(), 5);
    let mut result = source.to_string();
    let mut counts = Vec::new();
    for pair in pairs {
        let before = pair["before"].as_str().unwrap();
        let after = pair["after"].as_str().unwrap();
        counts.push(result.matches(before).count());
        result = result.replace(before, after);
    }
    if source.contains("wc -l") {
        let sha = digest::hash(source.as_bytes());
        let pin = r["guard_programs_r4"]
            .as_array()
            .unwrap()
            .iter()
            .find(|pin| pin["before_sha256"].as_str() == Some(sha.as_str()))
            .unwrap();
        assert_eq!(pin["replacement_counts"], value(&counts));
        assert_eq!(
            pin["after_sha256"].as_str().unwrap(),
            digest::hash(result.as_bytes())
        );
        let mut inverse = result.clone();
        for pair in pairs.iter().rev() {
            inverse = inverse.replace(
                pair["after"].as_str().unwrap(),
                pair["before"].as_str().unwrap(),
            );
        }
        assert_eq!(inverse, source, "Only exact fixed LF guards may change");
    } else {
        assert_eq!(result, source);
    }
    assert!(!result.contains("wc -l"));
    result
}
// Only one exact archived wrong source and four complete Cases own this
// intentional unsupported boundary. No general error allowance or label bypass.
fn boundary_wrong_run(source: &str, c: &Case, slice: usize) -> Outcome {
    let r = review();
    let rule = &r["active_input_boundary_r6"];
    assert_eq!(
        rule["source_sha256"],
        "5bfb95df1c794b944295cab0f48cfff78737330add658d0abf0fa4fd25c64d03"
    );
    if digest::hash(source.as_bytes()) == rule["source_sha256"].as_str().unwrap() {
        assert_eq!(source, rule["source"].as_str().unwrap());
        if let Some(pair) = rule["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|pair| pair["case"] == value(c))
        {
            let out = run(source, c, slice);
            let expected: Outcome = serde_json::from_value(pair["predicted"].clone()).unwrap();
            assert_eq!(out, expected, "Exact active-input boundary/{}", c.name);
            assert_eq!(out.error.as_deref(), rule["diagnostic"].as_str());
            return out;
        }
    }
    supported_run(source, c, slice)
}
fn wrong_record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s)
            .iter()
            .map(|c| boundary_wrong_run(source, c, 32))
            .collect(),
    }
}

fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s)
            .iter()
            .map(|c| supported_run(source, c, 32))
            .collect(),
    }
}

#[test]
fn private_fixture_preserves_all193_specs60_scripts477_cases60_examples_and51_unit_objects() {
    let all = catalog();
    assert_eq!(all.len(), 199, "Unregistered two-task fixture only");
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 197);
    let old = disk_registration::before_scenarios(&current).to_vec();
    assert_eq!(value(&all[..193]), value(&old));
    assert_eq!(value(&all[..197]), value(&current));
    assert_eq!(value(&all[197..]), value(&authored()));
    let scripts = old
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 60);
    assert_eq!(scripts.iter().map(|s| cases(s).len()).sum::<usize>(), 477);
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 60);
    let r = review();
    for pin in r["original_pack_pins"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        let raw =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        assert_eq!(digest::hash(&raw), pin["sha256"].as_str().unwrap());
    }
    let current_raw = include_str!("../scenarios/units-v1.json");
    let old_raw = disk_registration::before_units(current_raw.as_bytes()).unwrap();
    let raw = std::str::from_utf8(&old_raw).unwrap();
    assert_eq!(
        digest::hash(raw.as_bytes()),
        r["original_units_sha256"].as_str().unwrap()
    );
    let current_units = course::bundled(&current).unwrap();
    assert_eq!(current_units.len(), 52);
    let old_units = course::load(raw, &old).unwrap();
    assert_eq!(
        value(disk_registration::before_typed_units(&current_units)),
        value(&old_units)
    );
    assert_eq!(old_units.len(), 51);
    let mut private: Value = serde_json::from_str(current_raw).unwrap();
    private["units"]
        .as_array_mut()
        .unwrap()
        .push(r["unit_proposal"].clone());
    let private = course::load(&private.to_string(), all).unwrap();
    assert_eq!(value(&private[..51]), value(&old_units));
    assert_eq!(value(&private[..52]), value(&current_units));
    assert_eq!(value(private.last().unwrap()), r["unit_proposal"]);
    assert_eq!(
        private.len(),
        53,
        "A private test union, not normal registration"
    );
}

#[test]
fn reference_and_two_supported_alternatives_match_complete_cases_and_guards_at_four_slices() {
    catalog();
    for s in authored() {
        assert_eq!(cases(&s).len(), 12);
        let guards: Vec<Case> = serde_json::from_value(item(&s.id)["guards"].clone()).unwrap();
        let alternatives = programs(&s);
        assert!(
            !alternatives[1].contains("${1#")
                && !alternatives[1].contains("${line#")
                && !alternatives[1].contains("${item%")
                && !alternatives[1].contains("${stem#")
        );
        for c in cases(&s).iter().chain(&guards) {
            assert!(c.valid());
            for source in &alternatives {
                for slice in [1, 7, 32, 64] {
                    let out = supported_run(source, c, slice);
                    assert!(
                        c.accepts(&out),
                        "{}/{}/{} expected {:?} actual {:?}; source {:?}",
                        s.id,
                        c.name,
                        slice,
                        c,
                        out,
                        source
                    );
                }
            }
        }
    }
}

#[test]
fn plausible_wrong_partial_and_hardcoded_programs_fail_actual_graded_cases() {
    catalog();
    for s in authored() {
        let guards: Vec<Case> = serde_json::from_value(item(&s.id)["guards"].clone()).unwrap();
        for wrong in item(&s.id)["wrong"].as_array().unwrap() {
            let source = wrong["source"].as_str().unwrap();
            for (key, pass) in [
                ("must_pass_case_names", true),
                ("must_fail_case_names", false),
            ] {
                for name in wrong[key].as_array().unwrap() {
                    let c = cases(&s)
                        .iter()
                        .chain(&guards)
                        .find(|c| c.name == name.as_str().unwrap())
                        .unwrap();
                    assert_eq!(
                        c.accepts(&boundary_wrong_run(source, c, 7)),
                        pass,
                        "{}/{}/{}",
                        s.id,
                        wrong["label"],
                        c.name
                    );
                }
            }
            assert!(!engine::solved(
                &State::Script {
                    source: source.into(),
                    run: Some(wrong_record(&s, source))
                },
                &s.goal
            ));
        }
    }
}

#[test]
fn separate_practice_change_repair_and_challenge_views_never_award_assignment_credit() {
    catalog();
    for s in authored() {
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let assignment = cases(&s)
            .iter()
            .flat_map(|c| c.files.keys())
            .collect::<BTreeSet<_>>();
        assert!(lesson.fixture.files.keys().all(|p| !assignment.contains(p)));
        let row = item(&s.id);
        for key in ["try_change", "repair"] {
            let c: Case = serde_json::from_value(row[key]["case"].clone()).unwrap();
            assert!(c.accepts(&supported_run(row[key]["source"].as_str().unwrap(), &c, 7)));
        }
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, lesson, true);
        let before = value(&a.state);
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
                l.example_source = row["try_change"]["source"].as_str().unwrap().into();
                l.example_result = Some(shell::run(&l.example_source, &lesson.fixture));
                l.example_revision = shell::EVALUATOR_REVISION;
                assert_eq!(value(&a.state), before);
                assert!(!progress::current_solved(&a, &s));
            }
        }
    }
}

#[test]
fn actual_current_records_edit_invalidation_save_resume_and_selected_reset_preserve_other_work_and_receipts()
 {
    let all = catalog();
    let new = authored();
    let old = all.iter().find(|s| s.id == "shell-validation").unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 30,
        ..Default::default()
    };
    let mut sibling = Attempt::new(old, 3);
    workspace::prepare(&mut sibling);
    script_teaching::prepare(&mut sibling, old.teaching.as_ref().unwrap(), true);
    if let State::Script { source, .. } = &mut sibling.state {
        source.push_str("# Synthetic preexisting unfinished sibling draft\n");
    }
    p.attempts.insert(old.id.clone(), sibling);
    p.favorites.insert(old.id.clone());
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
    let s = &new[0];
    let a = p.attempts.get_mut(&s.id).unwrap();
    if let State::Script { source, .. } = &mut a.state {
        source.push_str("# Edited after a real run\n");
    }
    assert!(!progress::current_solved(a, s));
    let source = s.solution.join("\n");
    a.state = State::Script {
        source: source.clone(),
        run: Some(record(s, &source)),
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let caller = full(&p);
    progress::save_with_catalog(&path, &p, all).unwrap();
    assert_eq!(full(&p), caller);
    let mut resumed = progress::load(&path, all).unwrap();
    assert_eq!(value(&resumed), value(&p));
    assert!(
        resumed
            .migration_receipts
            .as_ref()
            .unwrap()
            .shell
            .contains(&15)
    );
    let before = full(&resumed);
    let mut fresh = Attempt::new(s, resumed.attempts[&s.id].visited);
    workspace::prepare(&mut fresh);
    script_teaching::prepare(&mut fresh, s.teaching.as_ref().unwrap(), true);
    let expected_fresh = value(&fresh);
    resumed.attempts.insert(s.id.clone(), fresh);
    let after = full(&resumed);
    assert_eq!(after["attempts"][&s.id], expected_fresh);
    for (id, previous) in before["attempts"].as_object().unwrap() {
        if id != &s.id {
            assert_eq!(&after["attempts"][id], previous);
        }
    }
    for key in ["clock", "favorites", "migration_receipts"] {
        assert_eq!(before[key], after[key]);
    }
    assert!(!progress::current_solved(&resumed.attempts[&s.id], s));
    assert!(progress::current_solved(
        &resumed.attempts[&new[1].id],
        &new[1]
    ));
    progress::save_with_catalog(&path, &resumed, all).unwrap();
    assert_eq!(full(&progress::load(&path, all).unwrap()), full(&resumed));
}

#[test]
fn missing_prerequisite_wrong_expected_behavior_or_unsupported_example_cannot_load() {
    catalog();
    for key in ["prerequisite", "expected", "example"] {
        let mut bad: Value = serde_json::from_str(PACK).unwrap();
        match key {
            "prerequisite" => {
                bad["scenarios"][0]["prerequisites"] = json!(["missing-literal-prerequisite"])
            }
            "expected" => {
                bad["scenarios"][0]["goal"]["cases"][0]["stdout"] =
                    json!("Not the authored behavior\n")
            }
            "example" => bad["scenarios"][0]["teaching"]["example"] = json!("unimplemented-tool\n"),
            _ => unreachable!(),
        }
        let mut all = pack::bundled().unwrap();
        all.extend(serde_json::from_value::<Pack>(bad).unwrap().scenarios);
        let t = texts(&all);
        assert!(
            pack::load_catalog(&t.iter().map(String::as_str).collect::<Vec<_>>()).is_err(),
            "{key}"
        );
    }
}

#[test]
fn retained_original_24_and_17_outcomes_and_both_concrete_counterexamples_remain_explicit() {
    catalog();
    let r = review();
    for row in r["labs"].as_array().unwrap() {
        let s = authored()
            .into_iter()
            .find(|s| s.id == row["id"].as_str().unwrap())
            .unwrap();
        let old = &row["retained_r1"];
        let mut retained: Vec<Case> = serde_json::from_value(old["cases"].clone()).unwrap();
        let old_guards: Vec<Case> = serde_json::from_value(old["guards"].clone()).unwrap();
        assert_eq!(retained.len(), 12);
        assert_eq!(old_guards.len(), if s.id == IDS[0] { 8 } else { 9 });
        retained.extend(old_guards);
        let positive = &retained[0];
        for original in old["programs"].as_array().unwrap() {
            let original = original.as_str().unwrap();
            assert!(original.contains("wc -l"));
            let unsupported = run(original, positive, 7);
            assert!(
                unsupported
                    .error
                    .as_deref()
                    .is_some_and(|e| e.contains("Unsupported command: wc"))
            );
            assert!(!positive.accepts(&unsupported));
        }
        let current_guards: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
        for c in &retained {
            assert!(
                cases(&s)
                    .iter()
                    .chain(&current_guards)
                    .any(|now| value(now) == value(c))
            );
            for source in old["programs"].as_array().unwrap() {
                let derived = guard_amended(source.as_str().unwrap());
                assert!(
                    c.accepts(&supported_run(&derived, c, 7)),
                    "retained {}/{}",
                    s.id,
                    c.name
                );
            }
        }
        for old_wrong in old["wrong"].as_array().unwrap() {
            let derived = guard_amended(old_wrong["source"].as_str().unwrap());
            let mut amended_wrong = old_wrong.clone();
            amended_wrong["source"] = derived.clone().into();
            assert!(row["wrong"].as_array().unwrap().contains(&amended_wrong));
            for (key, pass) in [
                ("must_pass_case_names", true),
                ("must_fail_case_names", false),
            ] {
                for name in old_wrong[key].as_array().unwrap() {
                    let c = retained
                        .iter()
                        .find(|c| c.name == name.as_str().unwrap())
                        .unwrap();
                    assert_eq!(c.accepts(&supported_run(&derived, c, 7)), pass);
                }
            }
        }
        if s.id == IDS[0] {
            let old_cut = guard_amended(old["programs"][1].as_str().unwrap());
            for name in [
                "Inline output trailing LF cannot be removed before validation",
                "Inline source trailing LF cannot become a different valid name",
            ] {
                let c = cases(&s).iter().find(|c| c.name == name).unwrap();
                assert!(!c.accepts(&supported_run(&old_cut, c, 7)));
                for source in programs(&s) {
                    assert!(c.accepts(&supported_run(&source, c, 7)));
                }
            }
        } else {
            let c = cases(&s)
                .iter()
                .find(|c| {
                    c.name == "A late manifest destination collision preserves every earlier output"
                })
                .unwrap();
            assert_eq!(c.files, c.after);
            assert!(c.files.contains_key("/work/b.copy"));
            for source in old["programs"].as_array().unwrap() {
                let derived = guard_amended(source.as_str().unwrap());
                assert!(!c.accepts(&supported_run(&derived, c, 7)));
            }
            for source in programs(&s) {
                assert!(c.accepts(&supported_run(&source, c, 7)));
            }
        }
    }
}

#[test]
fn active_input_boundary_is_exact_and_after_loop_omission_still_fails_by_supported_behavior() {
    catalog();
    let r = review();
    let rule = &r["active_input_boundary_r6"];
    assert!(rule["actual"].is_null());
    let source = rule["source"].as_str().unwrap();
    let s = authored().remove(1);
    let row = item(&s.id);
    assert_eq!(source, row["wrong"][2]["source"].as_str().unwrap());
    assert_eq!(rule["pairs"].as_array().unwrap().len(), 4);
    let guards: Vec<Case> = serde_json::from_value(row["guards"].clone()).unwrap();
    for slice in [1, 7, 32, 64] {
        let mut errors = 0;
        for c in cases(&s).iter().chain(&guards) {
            let actual = boundary_wrong_run(source, c, slice);
            if actual.error.is_some() {
                errors += 1;
            }
        }
        assert_eq!(
            errors, 4,
            "Only four complete source/Case pins may admit the boundary"
        );
        let omission = rule["supported_omission_source"].as_str().unwrap();
        assert_eq!(omission, row["wrong"][7]["source"].as_str().unwrap());
        assert_eq!(
            digest::hash(omission.as_bytes()),
            rule["supported_omission_source_sha256"].as_str().unwrap()
        );
        assert_eq!(
            rule["supported_omission_cases"].as_array().unwrap().len(),
            4
        );
        for pair in rule["supported_omission_cases"].as_array().unwrap() {
            assert!(pair["actual"].is_null());
            let c: Case = serde_json::from_value(pair["case"].clone()).unwrap();
            assert!(cases(&s).iter().chain(&guards).any(|old| old == &c));
            let actual = supported_run(omission, &c, slice);
            let expected: Outcome = serde_json::from_value(pair["predicted"].clone()).unwrap();
            assert_eq!(actual, expected);
            assert!(
                !c.accepts(&actual),
                "Omitted collision must fail by real outcome"
            );
        }
    }
    let correct = s.solution.join("\n");
    for pair in rule["pairs"].as_array().unwrap() {
        assert!(pair["actual"].is_null());
        let c: Case = serde_json::from_value(pair["case"].clone()).unwrap();
        assert!(c.accepts(&supported_run(&correct, &c, 7)));
    }
}
