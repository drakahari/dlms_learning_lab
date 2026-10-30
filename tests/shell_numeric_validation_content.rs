//! Three distinct numeric validation assignments; private normal catalog, not host execution.
//! All scripts execute in the bounded VM. No subprocess or host reference here.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record, Vm},
    workspace,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PACK: &str = include_str!("../scenarios/shell-numeric-validation-v1.json");
const REVIEW: &str = include_str!("fixtures/shell-numeric-validation-review.json");
const IDS: [&str; 3] = [
    "shell-numeric-retry",
    "shell-numeric-intake",
    "shell-numeric-publication",
];
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("A script workspace is required")
    };
    cases
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
fn originals() -> Vec<Scenario> {
    let r = review();
    let mut texts = Vec::new();
    let mut paths = BTreeSet::new();
    for pin in r["original_source_pins"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        assert!(
            name.starts_with("scenarios/") && std::path::Path::new(name).components().count() == 2
        );
        assert!(paths.insert(name));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.len() <= 2 * 1024 * 1024
        );
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            digest::hash(&bytes),
            pin["sha256"].as_str().unwrap(),
            "{name}"
        );
        texts.push(String::from_utf8(bytes).unwrap());
    }
    assert_eq!(texts.len(), 41);
    let old = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(old.len(), 168);
    assert_eq!(r["original_catalog_count"], 168);
    let scripts = old
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 52);
    assert_eq!(scripts.iter().map(|s| cases(s).len()).sum::<usize>(), 384);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| old.iter().filter(|s| s.track == t).count()),
        [48, 36, 32, 52]
    );
    old
}
// Use the application's real bounded multi-pack path, retaining prerequisites.
// This fixture must not flatten a growing catalog into one oversized pack.
fn bounded_catalog(pack: &Pack) -> Result<Vec<Scenario>, String> {
    let texts = pack
        .scenarios
        .chunks(16)
        .map(|rows| {
            serde_json::json!({"schema_version":pack.schema_version,"scenarios":rows}).to_string()
        })
        .collect::<Vec<_>>();
    assert!(texts.iter().all(|text| text.len() <= pack::MAX_PACK_BYTES));
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        assert_eq!(shell::EVALUATOR_REVISION, 14);
        let old = originals();
        let current = pack::bundled().unwrap();
        assert_eq!(one_off_boundary::before_len(&current), 176);
        assert_eq!(value(&current[..168]), value(&old));
        let added: Pack = serde_json::from_str(PACK).unwrap();
        assert_eq!(added.schema_version, 1);
        assert_eq!(
            added
                .scenarios
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert_eq!(value(&current[168..171]), value(&added.scenarios));
        let before = value(&current);
        let loaded = bounded_catalog(&Pack {
            schema_version: 1,
            scenarios: current,
        })
        .unwrap();
        assert_eq!(
            value(&loaded),
            before,
            "All full objects/order survive bounded multi-pack loading"
        );
        loaded
    })
}
fn new_labs() -> Vec<&'static Scenario> {
    IDS.iter()
        .map(|id| catalog().iter().find(|s| s.id == *id).unwrap())
        .collect()
}
fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 11);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    a
}
fn observed_mask(s: &Scenario, source: &str) -> Vec<bool> {
    cases(s)
        .iter()
        .map(|c| c.accepts(&shell::run(source, c)))
        .collect()
}
fn expected_mask(value: &Value) -> Vec<bool> {
    serde_json::from_value(value.clone()).unwrap()
}

#[test]
fn original168_full_specs52_scripts384_cases_and43_raw_unit_inputs_are_preserved() {
    let old = originals();
    assert_eq!(value(&catalog()[..168]), value(&old));
    let r = review();
    let raw = r["original_units43_text"].as_str().unwrap();
    assert_eq!(digest::hash(raw.as_bytes()), r["original_units43_sha256"]);
    let units: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(units["units"].as_array().unwrap().len(), 43);
    let before = course::load(raw, &old).unwrap();
    assert_eq!(before.len(), 43);
    let now = course::bundled(&pack::bundled().unwrap()).unwrap();
    assert_eq!(value(&now[..43]), value(&before));
}
#[test]
fn exact_three_distinct_tasks36_cases_have_closed_prerequisites_and_one_advisory_unit() {
    let labs = new_labs();
    assert_eq!(
        labs.iter().map(|s| cases(s).len()).collect::<Vec<_>>(),
        [12, 12, 12]
    );
    let r = review();
    let unit: course::Unit = serde_json::from_value(r["unit_proposal"].clone()).unwrap();
    assert_eq!(unit.id, "shell-numeric-validation");
    assert_eq!(unit.labs, IDS.map(str::to_string));
    let existing = pack::bundled().unwrap();
    let units = course::bundled(&existing).unwrap();
    assert_eq!(one_off_boundary::before_len(&units), 46);
    assert_eq!(value(&units[43]), value(&unit));
    let index = json!({"version":1,"units":units});
    let validated = course::load(&index.to_string(), catalog()).unwrap();
    assert_eq!(
        value(validated.iter().find(|u| u.id == unit.id).unwrap()),
        value(&unit)
    );
    let all_ids = catalog()
        .iter()
        .map(|s| s.id.as_str())
        .collect::<BTreeSet<_>>();
    for s in labs {
        assert_eq!(s.version, 1);
        assert_eq!(s.track, "Bash/Shell");
        assert_eq!(s.unit, unit.id);
        assert!(s.prerequisites.iter().all(|p| all_ids.contains(p.as_str())));
        assert!(matches!(s.initial, State::Script { .. }));
        assert!(!progress::current_solved(&attempt(s), s));
    }
}
#[test]
fn each_reference_and_two_distinct_supported_solutions_pass_every_complete_behavior_case() {
    for s in new_labs() {
        let r = item(&s.id);
        let source = s.solution.join("\n");
        let alternatives = r["alternatives"].as_array().unwrap();
        assert_eq!(alternatives.len(), 2);
        let distinct = std::iter::once(source.as_str())
            .chain(alternatives.iter().map(|v| v.as_str().unwrap()))
            .collect::<BTreeSet<_>>();
        assert_eq!(distinct.len(), 3);
        assert_eq!(
            observed_mask(s, &source),
            expected_mask(&r["reference_authored_expected_pass"])
        );
        for (n, alternate) in alternatives.iter().enumerate() {
            assert_eq!(
                observed_mask(s, alternate.as_str().unwrap()),
                expected_mask(&r["alternative_authored_expected_pass"][n]),
                "{} alternate{n}",
                s.id
            );
        }
        for c in cases(s) {
            assert!(c.valid());
            let mut vm = Vm::new(&source, c).unwrap();
            for _ in 0..20_000 {
                if vm.done() {
                    break;
                }
                vm.tick(1);
            }
            assert!(vm.done());
            assert!(
                c.accepts(&vm.outcome),
                "{} / {}: {:?}",
                s.id,
                c.name,
                vm.outcome
            );
        }
    }
}
#[test]
fn authored_wrong_partial_and_hardcoded_solutions_fail_the_declared_cases_and_never_complete() {
    let mut total = 0;
    for s in new_labs() {
        let r = item(&s.id);
        for (name, row) in r["common_wrong"].as_object().unwrap() {
            let source = row["source"].as_str().unwrap();
            let mask = observed_mask(s, source);
            assert!(
                mask.iter().any(|pass| !*pass),
                "{} / {name} must never complete",
                s.id
            );
            if let Some(expected) = row.get("authored_expected_pass") {
                assert_eq!(mask, expected_mask(expected), "{} / {name}", s.id);
            }
            for (field, wanted) in [
                ("must_fail_case_names", false),
                ("must_pass_case_names", true),
            ] {
                for named in row[field].as_array().into_iter().flatten() {
                    let wanted_name = named.as_str().unwrap();
                    let i = cases(s).iter().position(|c| c.name == wanted_name).unwrap();
                    assert_eq!(mask[i], wanted, "{} / {name} / {wanted_name}", s.id);
                }
            }
            if ["missing_wait_line", "omits_preview_report"].contains(&name.as_str()) {
                assert!(
                    mask.iter().any(|p| *p) && mask.iter().any(|p| !*p),
                    "Real partial completion is required"
                );
            }
            let mut a = attempt(s);
            a.state = State::Script {
                source: source.into(),
                run: Some(record(s, source)),
            };
            assert!(!progress::current_solved(&a, s));
            total += 1;
        }
    }
    assert_eq!(total, 13);
}
#[test]
fn additional_guards_and_unicode_input_keep_full_state_without_becoming_extra_labs() {
    let mut guards = 0;
    for s in new_labs() {
        let r = item(&s.id);
        for row in r["supplemental_cases"].as_array().into_iter().flatten() {
            let c: Case = serde_json::from_value(row.clone()).unwrap();
            assert!(c.valid());
            let sources = std::iter::once(s.solution.join("\n")).chain(
                r["alternatives"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_owned()),
            );
            for source in sources {
                let actual = shell::run(&source, &c);
                assert!(c.accepts(&actual), "{} / {}: {actual:?}", s.id, c.name);
            }
            guards += 1;
        }
    }
    assert_eq!(guards, 23);
    assert_eq!(IDS.len(), 3);
}
#[test]
fn teaching_uses_plain_prerequisites_and_disjoint_runnable_example_experiment_and_repair() {
    for s in new_labs() {
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!((4..=6).contains(&lesson.concepts.len()));
        for c in &lesson.concepts {
            assert!(!c.title.is_empty() && !c.text.is_empty());
        }
        for jargon in [
            "Read the contract",
            "fresh virtual workspace",
            "local scalar",
        ] {
            assert!(!value(s).to_string().contains(jargon));
        }
        assert_ne!(lesson.example, s.solution.join("\n"));
        for c in cases(s) {
            assert!(
                lesson
                    .fixture
                    .files
                    .keys()
                    .all(|p| !c.files.contains_key(p))
            );
            assert_eq!(
                lesson.fixture.env,
                BTreeMap::from([("LC_ALL".into(), "C".into())])
            );
            assert_eq!(
                c.env, lesson.fixture.env,
                "Only the fixed simulator locale is shared, not working inputs or files"
            );
        }
        let r = item(&s.id);
        let practice = &r["practice"];
        assert_eq!(practice["reference"], lesson.example);
        assert_eq!(practice["repair"], lesson.example);
        let experiment = practice["experiment"]["source"].as_str().unwrap();
        assert_ne!(experiment, lesson.example);
        let fixture: Case = serde_json::from_value(practice["experiment"]["case"].clone()).unwrap();
        assert!(fixture.valid());
        assert_eq!(fixture.args, lesson.fixture.args);
        assert_eq!(fixture.env, lesson.fixture.env);
        assert_eq!(fixture.files, lesson.fixture.files);
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
        assert!(fixture.accepts(&shell::run(experiment, &fixture)));
        assert!(lesson.fixture.accepts(&shell::run(
            practice["repair"].as_str().unwrap(),
            &lesson.fixture
        )));
    }
}
#[test]
fn example_work_and_navigation_never_overwrite_assignment_drafts_or_award_credit() {
    for s in new_labs() {
        let mut a = attempt(s);
        let original = value(&a.state);
        let r = item(&s.id);
        let experiment = r["practice"]["experiment"]["source"].as_str().unwrap();
        let fixture: Case =
            serde_json::from_value(r["practice"]["experiment"]["case"].clone()).unwrap();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        assert_eq!(learning.view, script_teaching::View::Introduction);
        learning.view = script_teaching::View::Example;
        learning.example_source = experiment.into();
        learning.example_result = Some(shell::run(experiment, &fixture));
        learning.example_revision = shell::EVALUATOR_REVISION;
        for view in [
            script_teaching::View::Assignment,
            script_teaching::View::Introduction,
            script_teaching::View::Example,
        ] {
            a.workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap()
                .view = view;
            assert_eq!(value(&a.state), original);
            assert_eq!(
                a.workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap()
                    .example_source,
                experiment
            );
            assert!(!progress::current_solved(&a, s));
        }
    }
}
#[test]
fn success_at_first_walkthrough_position_is_invalidated_by_edits_bad_reruns_and_stop() {
    for s in new_labs() {
        let mut a = attempt(s);
        let source = s.solution.join("\n");
        a.state = State::Script {
            source: source.clone(),
            run: Some(record(s, &source)),
        };
        a.step = 0;
        assert!(progress::current_solved(&a, s));
        let changed = format!("printf '%s' 'unexpected extra output'\n{source}");
        if let State::Script { source, .. } = &mut a.state {
            *source = changed.clone();
        }
        assert!(!progress::current_solved(&a, s));
        a.state = State::Script {
            source: changed.clone(),
            run: Some(record(s, &changed)),
        };
        assert!(!progress::current_solved(&a, s));
        let loop_source = "while true; do :; done";
        let mut vm = Vm::new(loop_source, &cases(s)[0]).unwrap();
        for _ in 0..12 {
            vm.tick(1);
        }
        assert!(!vm.done());
        vm.stop();
        assert!(vm.done());
        assert!(
            vm.outcome
                .error
                .as_deref()
                .is_some_and(|e| e.starts_with("Stopped"))
        );
        a.state = State::Script {
            source: loop_source.into(),
            run: None,
        };
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn learn_and_challenge_have_the_same_full_assignment_and_no_automatic_solution_or_example_credit() {
    for s in new_labs() {
        for challenge in [false, true] {
            let mut a = attempt(s);
            a.challenge = challenge;
            if challenge {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .script_learning
                    .as_mut()
                    .unwrap()
                    .view = script_teaching::View::Assignment;
            }
            assert_eq!(value(&a.state), value(&s.initial));
            assert!(!progress::current_solved(&a, s));
            let source = s.solution.join("\n");
            a.state = State::Script {
                source: source.clone(),
                run: Some(record(s, &source)),
            };
            assert!(progress::current_solved(&a, s));
        }
    }
}
#[test]
fn assignment_and_example_drafts_favorites_and_selected_reset_survive_safe_round_trip() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let unrelated = &all[0];
    p.attempts
        .insert(unrelated.id.clone(), Attempt::new(unrelated, 1));
    p.favorites.insert(unrelated.id.clone());
    for (n, s) in new_labs().into_iter().enumerate() {
        let mut a = attempt(s);
        a.challenge = n == 2;
        let r = item(&s.id);
        let draft = r["common_wrong"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()["source"]
            .as_str()
            .unwrap();
        a.state = State::Script {
            source: draft.into(),
            run: Some(record(s, draft)),
        };
        let practice = &r["practice"];
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.view = script_teaching::View::Example;
        l.example_source = practice["experiment"]["source"].as_str().unwrap().into();
        let c: Case = serde_json::from_value(practice["experiment"]["case"].clone()).unwrap();
        l.example_result = Some(shell::run(&l.example_source, &c));
        l.example_revision = shell::EVALUATOR_REVISION;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    p.clock = 44;
    let before = value(&p);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut loaded = progress::load(&path, all).unwrap();
    assert_eq!(value(&loaded), before);
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![shell::EVALUATOR_REVISION]
    );
    for s in new_labs() {
        let source = s.solution.join("\n");
        loaded.attempts.get_mut(&s.id).unwrap().state = State::Script {
            source: source.clone(),
            run: Some(record(s, &source)),
        };
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    }
    progress::save(&path, &loaded).unwrap();
    let mut resumed = progress::load(&path, all).unwrap();
    assert_eq!(value(&resumed), value(&loaded));
    let selected = new_labs()[1];
    let old = value(&resumed);
    resumed.attempts.insert(
        selected.id.clone(),
        Attempt::new(selected, resumed.clock + 1),
    );
    let now = value(&resumed);
    assert!(!progress::current_solved(
        &resumed.attempts[&selected.id],
        selected
    ));
    for (id, a) in old["attempts"].as_object().unwrap() {
        if id != &selected.id {
            assert_eq!(&now["attempts"][id], a);
        }
    }
    assert_eq!(now["favorites"], old["favorites"]);
    progress::save(&path, &resumed).unwrap();
    assert_eq!(value(&progress::load(&path, all).unwrap()), now);
}
#[test]
fn malformed_unreachable_or_unsafe_content_is_rejected_without_any_host_execution() {
    let all = catalog().clone();
    assert_eq!(
        value(
            &bounded_catalog(&Pack {
                schema_version: 1,
                scenarios: all.clone()
            })
            .unwrap()
        ),
        value(&all),
        "Valid complete context must load before each intended negative"
    );
    fn rejects(p: &Pack, expected: &str) {
        let error = bounded_catalog(p).unwrap_err();
        assert!(
            error.contains(expected),
            "Expected {expected:?}, received {error:?}"
        );
    }
    let new_index = all.iter().position(|s| s.id == IDS[0]).unwrap();
    let mut bad = Pack {
        schema_version: 2,
        scenarios: all.clone(),
    };
    rejects(&bad, "Unsupported or empty pack");
    bad.schema_version = 1;
    bad.scenarios[new_index].hints.clear();
    rejects(&bad, "missing or oversized authoring fields");
    let mut bad = Pack {
        schema_version: 1,
        scenarios: all.clone(),
    };
    if let Goal::Script { cases } = &mut bad.scenarios[new_index].goal {
        cases[0].stdout = "an invented unreachable answer".into();
    }
    rejects(&bad, "solution does not reach objective");
    let mut bad = Pack {
        schema_version: 1,
        scenarios: all.clone(),
    };
    bad.scenarios[new_index].prerequisites = vec![IDS[2].into()];
    rejects(&bad, "cyclic prerequisites");
    let mut bad = Pack {
        schema_version: 1,
        scenarios: all,
    };
    if let Goal::Script { cases } = &mut bad.scenarios[new_index].goal {
        cases[0].files = BTreeMap::from([("/etc/passwd".into(), "only simulated data".into())]);
    }
    rejects(&bad, "invalid script cases");
}

#[test]
fn syntax_unsupported_wrong_output_and_correct_behavior_have_distinct_retry_feedback() {
    let s = new_labs()[0];
    let c = &cases(s)[0];
    for (source, kind) in [
        ("if", script_teaching::Diagnostic::Syntax),
        (
            "printf '%s' \"${TEAM:-$OTHER}\"",
            script_teaching::Diagnostic::Unsupported,
        ),
    ] {
        let result = shell::run(source, c);
        let error = result.error.as_deref().unwrap();
        assert_eq!(script_teaching::diagnostic(error), kind);
        assert!(!script_teaching::diagnostic_title(error).is_empty());
        assert!(!script_teaching::diagnostic_advice(error).is_empty());
        assert_eq!(result.files, c.files);
    }
    // Preserve each case's intended status/stderr/files so this control
    // isolates the stdout diagnostic instead of triggering exit-status first.
    let source = format!("printf '%s' 'wrong output'\n{}", s.solution.join("\n"));
    let result = record(s, &source);
    assert!(result.results.iter().all(|r| r.error.is_none()));
    let checks = shell::checks(&source, &Some(result), cases(s));
    assert!(
        checks
            .iter()
            .all(|(pass, reason)| !*pass && reason.contains("Standard output differs"))
    );
    let source = s.solution.join("\n");
    let checks = shell::checks(&source, &Some(record(s, &source)), cases(s));
    assert!(
        checks
            .iter()
            .all(|(pass, reason)| *pass && reason.starts_with("Pass"))
    );
}

#[test]
fn supported_unreachable_suffixes_remain_valid_behavioral_alternatives() {
    for id in ["shell-numeric-intake", "shell-numeric-publication"] {
        let s = new_labs().into_iter().find(|s| s.id == id).unwrap();
        let source = format!(
            "{}\nexit 0\nprintf '%s' 'unreachable output'\n",
            s.solution.join("\n")
        );
        let result = record(s, &source);
        assert!(
            shell::checks(&source, &Some(result.clone()), cases(s))
                .iter()
                .all(|r| r.0)
        );
        let mut a = attempt(s);
        a.state = State::Script {
            source,
            run: Some(result),
        };
        assert!(
            progress::current_solved(&a, s),
            "Valid source alternatives are graded by executed behavior, not text edits alone"
        );
    }
}
