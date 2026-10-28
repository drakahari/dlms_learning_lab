//! Authored command-construction unit in the private combined catalog. In-process simulation only.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;

#[derive(Deserialize)]
struct ReviewFile {
    schema_version: u32,
    original_count: usize,
    original_script_count: usize,
    original_ids: Vec<String>,
    original_source_pins: Vec<Pin>,
    unit_proposal: course::Unit,
    reviews: Vec<Review>,
    normal_registration: Append,
}
#[derive(Deserialize)]
struct Append {
    old_sha256: String,
    new_sha256: String,
    insert_byte_offset: usize,
    insert_utf8: String,
}
fn original_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let old = combined_catalog::old_unit_bytes(raw)?;
    let pin = review().normal_registration;
    if digest::hash(&old) != pin.old_sha256 {
        return Err("Original Bash recipe source differs".into());
    }
    let at = pin.insert_byte_offset;
    if at > old.len() {
        return Err("Bash recipe offset exceeds original source".into());
    }
    let mut middle = old[..at].to_vec();
    middle.extend_from_slice(pin.insert_utf8.as_bytes());
    middle.extend_from_slice(&old[at..]);
    if digest::hash(&middle) != pin.new_sha256 {
        return Err("Original Bash recipe intermediate differs".into());
    }
    Ok(old)
}

#[derive(Deserialize)]
struct Pin {
    path: String,
    sha256: String,
}
#[derive(Deserialize)]
struct Review {
    id: String,
    alternates: Vec<String>,
    wrong: Vec<Wrong>,
    partial: String,
    first_case_hardcode: Program,
    file_damage: Program,
    try_change: Change,
    guard_cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Program {
    source: String,
    expected_rows: Vec<bool>,
}
#[derive(Deserialize)]
struct Wrong {
    name: String,
    source: String,
    expected_rows: Vec<bool>,
    misconception: String,
}
#[derive(Deserialize)]
struct Change {
    source: String,
    expected: Case,
    repair: String,
}
fn review() -> ReviewFile {
    serde_json::from_str(include_str!("fixtures/shell-command-vectors-review.json")).unwrap()
}
fn authored() -> Vec<Scenario> {
    let p: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/shell-command-vectors-v1.json")).unwrap();
    p.scenarios
}
fn catalog() -> Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let all = pack::bundled().unwrap();
        assert_eq!(all.len(), 154);
        for new in authored() {
            let actual = all.iter().find(|s| s.id == new.id).unwrap();
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(new).unwrap()
            );
        }
        all
    })
    .clone()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("Script goal required")
    };
    cases
}
fn run(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap_or_else(|e| panic!("{}: {e}", case.name));
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded authored source did not finish");
}
fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s)
            .iter()
            .map(|case| shell::run(source, case))
            .collect(),
    }
}
fn state(s: &Scenario, source: &str) -> State {
    State::Script {
        source: source.into(),
        run: Some(record(s, source)),
    }
}
fn rows(s: &Scenario, source: &str) -> Vec<bool> {
    let record = record(s, source);
    cases(s)
        .iter()
        .zip(&record.results)
        .map(|(c, o)| c.accepts(o))
        .collect()
}
fn full(p: &Progress) -> serde_json::Value {
    let mut value = serde_json::to_value(p).unwrap();
    value["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    value
}

#[test]
fn three_distinct_registered_tasks_preserve_original147_and_exact37_unit_prefix() {
    let r = review();
    assert_eq!(r.schema_version, 1);
    let all = catalog();
    let existing = &all[..147];
    assert_eq!(existing.len(), r.original_count);
    assert_eq!(r.original_count, 147);
    assert_eq!(
        existing
            .iter()
            .filter(|s| matches!(s.initial, State::Script { .. }))
            .count(),
        r.original_script_count
    );
    assert_eq!(r.original_script_count, 46);
    assert_eq!(
        existing.iter().map(|s| &s.id).collect::<Vec<_>>(),
        r.original_ids.iter().collect::<Vec<_>>()
    );
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 39);
    let old_raw = original_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let original = course::load(std::str::from_utf8(&old_raw).unwrap(), existing).unwrap();
    assert_eq!(original.len(), 37);
    assert_eq!(
        serde_json::to_value(&units[..37]).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    let proposed = authored();
    assert_eq!(proposed.len(), 3);
    assert_eq!(
        all[147..150].iter().map(|s| &s.id).collect::<Vec<_>>(),
        proposed.iter().map(|s| &s.id).collect::<Vec<_>>()
    );
    assert_eq!(
        serde_json::to_value(&units[37]).unwrap(),
        serde_json::to_value(&r.unit_proposal).unwrap()
    );
    for s in &proposed {
        assert_eq!(s.unit, r.unit_proposal.id);
        assert_eq!(s.track, "Bash/Shell");
        assert_eq!(s.version, 1);
        assert_eq!(cases(s).len(), 12);
        assert!(cases(s).iter().all(Case::valid));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        assert_eq!(s.guide.len(), 3);
        assert_eq!(s.guide[0].title, "Understand the task");
        assert!(s.guide[0].why.contains("sample inputs") && s.guide[0].why.contains("reset"));
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        let lesson = s.teaching.as_ref().unwrap();
        assert!((4..=6).contains(&lesson.concepts.len()));
        assert!(!lesson.prerequisite_note.is_empty());
        lesson.validate().unwrap();
    }
    let mut changed = include_bytes!("../scenarios/units-v1.json").to_vec();
    changed.push(b' ');
    assert!(original_units(&changed).is_err());
    let mut changed: serde_json::Value =
        serde_json::from_slice(include_bytes!("../scenarios/units-v1.json")).unwrap();
    changed["units"][0]["title"] = "Unreviewed change".into();
    assert!(original_units(&serde_json::to_vec(&changed).unwrap()).is_err());
}

#[test]
fn references_and_two_genuinely_supported_alternates_pass_all36_behavior_cases() {
    for s in authored() {
        let r = review().reviews.into_iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(r.alternates.len(), 2);
        assert_ne!(r.alternates[0], r.alternates[1]);
        assert!(
            r.alternates
                .iter()
                .any(|source| !source.contains("${command[@]}")),
            "a direct-branch equivalent must pass"
        );
        for source in std::iter::once(&s.solution[0]).chain(&r.alternates) {
            assert_ne!(
                source,
                match &s.initial {
                    State::Script { source, .. } => source,
                    _ => panic!(),
                }
            );
            for c in cases(&s).iter().chain(&r.guard_cases) {
                for slice in [1, 16, 64] {
                    let got = run(source, c, slice);
                    assert!(c.accepts(&got), "{} / {} / {slice}: {got:?}", s.id, c.name);
                }
            }
            assert!(engine::solved(&state(&s, source), &s.goal));
        }
    }
}

#[test]
fn exact_independent_negative_vectors_reject_partial_stale_hardcoded_and_early_write_sources() {
    for s in authored() {
        let r = review().reviews.into_iter().find(|r| r.id == s.id).unwrap();
        for bad in &r.wrong {
            assert!(bad.misconception.len() > 30);
            let actual = rows(&s, &bad.source);
            assert_eq!(actual, bad.expected_rows, "{} / {}", s.id, bad.name);
            assert!(actual.contains(&false));
            if bad.name == r.partial {
                assert!(actual.contains(&true));
            }
            assert!(!engine::solved(&state(&s, &bad.source), &s.goal));
            assert!(
                cases(&s)
                    .iter()
                    .any(|c| shell::run(&bad.source, c).error.is_none()),
                "not an unsupported-only negative"
            );
        }
        for bad in [&r.first_case_hardcode, &r.file_damage] {
            assert_eq!(
                rows(&s, &bad.source),
                bad.expected_rows,
                "{} hardcode/damage",
                s.id
            );
            assert!(!engine::solved(&state(&s, &bad.source), &s.goal));
        }
        assert!(r.first_case_hardcode.expected_rows[0]);
        assert!(
            r.first_case_hardcode
                .expected_rows
                .iter()
                .skip(1)
                .all(|x| !x)
        );
        assert!(r.file_damage.expected_rows.iter().all(|x| !x));
    }
}

#[test]
fn every_case_compares_status_both_outputs_and_the_complete_file_state() {
    for s in authored() {
        for c in cases(&s) {
            let good = shell::run(&s.solution[0], c);
            assert!(c.accepts(&good));
            let mut bad = good.clone();
            bad.status = (bad.status + 1) % 256;
            assert!(!c.accepts(&bad));
            let mut bad = good.clone();
            bad.stdout.push('!');
            assert!(!c.accepts(&bad));
            let mut bad = good.clone();
            bad.stderr.push('!');
            assert!(!c.accepts(&bad));
            let mut bad = good.clone();
            bad.files.insert("/work/keep.txt".into(), "damaged".into());
            assert!(!c.accepts(&bad));
            let mut bad = good;
            bad.files
                .insert("/work/unrequested.txt".into(), "extra".into());
            assert!(!c.accepts(&bad));
        }
    }
}

#[test]
fn all_three_separate_examples_experiments_and_repairs_are_real_and_ungraded() {
    for s in authored() {
        let r = review().reviews.into_iter().find(|r| r.id == s.id).unwrap();
        let l = s.teaching.as_ref().unwrap();
        assert_ne!(l.example, s.solution[0]);
        assert_eq!(r.try_change.repair, l.example);
        assert_ne!(r.try_change.source, l.example);
        assert!(
            l.fixture
                .files
                .keys()
                .all(|p| cases(&s).iter().all(|c| !c.files.contains_key(p)))
        );
        assert_eq!(r.try_change.expected.files, l.fixture.files);
        for slice in [1, 16, 64] {
            assert!(l.fixture.accepts(&run(&l.example, &l.fixture, slice)));
            let changed = run(&r.try_change.source, &r.try_change.expected, slice);
            assert!(
                r.try_change.expected.accepts(&changed),
                "{}: {changed:?}",
                s.id
            );
            assert!(!l.fixture.accepts(&changed));
            assert!(
                l.fixture
                    .accepts(&run(&r.try_change.repair, &l.fixture, slice))
            );
        }
        assert!(rows(&s, &l.example).contains(&false));
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, l, true);
        let before = a.state.clone();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning.example_result = Some(shell::run(&learning.example_source, &l.fixture));
        learning.example_revision = shell::EVALUATOR_REVISION;
        learning.view = View::Example;
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
    }
}

#[test]
fn syntax_unsupported_wrong_behavior_and_correct_results_are_distinct() {
    for s in authored() {
        let c = &cases(&s)[0];
        let syntax = shell::run("printf '", c);
        let unsupported = shell::run("eval 'printf unsafe'", c);
        let wrong = shell::run("printf wrong", c);
        assert_eq!(
            teaching::diagnostic(syntax.error.as_deref().unwrap()),
            teaching::Diagnostic::Syntax
        );
        assert_eq!(
            teaching::diagnostic(unsupported.error.as_deref().unwrap()),
            teaching::Diagnostic::Unsupported
        );
        assert!(wrong.error.is_none() && !c.accepts(&wrong));
        assert!(c.accepts(&shell::run(&s.solution[0], c)));
    }
}

#[test]
fn source_edits_and_broken_runs_invalidate_current_success_then_retry_repairs_it() {
    for s in authored() {
        let mut a = Attempt::new(&s, 3);
        workspace::prepare(&mut a);
        a.state = state(&s, &s.solution[0]);
        assert!(progress::current_solved(&a, &s));
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\n# unfinished edit\n");
        assert!(!progress::current_solved(&a, &s));
        let State::Script { source, .. } = &a.state else {
            panic!()
        };
        a.state = state(&s, source);
        assert!(progress::current_solved(&a, &s));
        a.state = state(&s, "printf wrong");
        assert!(!progress::current_solved(&a, &s));
        a.state = state(&s, &s.solution[0]);
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn both_drafts_favorites_whole_progress_resume_and_selected_reset_preserve_other_work() {
    let all = catalog();
    let projects = authored();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    let old = full(&p);
    for (i, s) in projects.iter().enumerate() {
        let mut a = Attempt::new(s, 300 + i as u64);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source.push_str("\nprintf 'unfinished example");
        l.view = View::Example;
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\nprintf 'unfinished assignment");
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("profile.json");
    let original = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    std::fs::write(&profile, original).unwrap();
    progress::save(&profile, &p).unwrap();
    assert_eq!(full(&p), before, "save does not stamp in-memory provenance");
    let bytes = std::fs::read(&profile).unwrap();
    let mut loaded = progress::decode(&bytes, &all).unwrap();
    let mut expected = before.clone();
    expected["migration_receipts"]["shell"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!(shell::EVALUATOR_REVISION));
    assert_eq!(
        full(&loaded),
        expected,
        "only the explicit committed current-version receipt is added"
    );
    let saved_original =
        profile.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    assert_eq!(std::fs::read(&saved_original).unwrap(), original);
    progress::save(&profile, &loaded).unwrap();
    assert_eq!(std::fs::read(&profile).unwrap(), bytes);
    assert_eq!(std::fs::read(&saved_original).unwrap(), original);
    for (id, attempt) in old["attempts"].as_object().unwrap() {
        assert_eq!(full(&loaded)["attempts"][id], *attempt);
    }
    let other = loaded.attempts[&projects[1].id].clone();
    loaded
        .attempts
        .insert(projects[0].id.clone(), Attempt::new(&projects[0], 999));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&projects[1].id]).unwrap(),
        serde_json::to_value(other).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    for (id, attempt) in old["attempts"].as_object().unwrap() {
        assert_eq!(full(&loaded)["attempts"][id], *attempt);
    }
}

#[test]
fn validated_export_inspection_restore_and_cancellation_keep_separate_sources() {
    let all = catalog();
    let projects = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 500,
        ..Default::default()
    };
    for (i, s) in projects.iter().enumerate() {
        let mut a = Attempt::new(s, 10 + i as u64);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        a.state = state(s, &s.solution[0]);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        l.example_revision = shell::EVALUATOR_REVISION;
        l.view = View::Assignment;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("export.json");
    backup::export(&export, &p, &all).unwrap();
    let original = std::fs::read(&export).unwrap();
    let cancel = AtomicBool::new(false);
    let checked = AtomicUsize::new(0);
    let candidate = backup::inspect(&export, &all, &cancel, &checked).unwrap();
    assert_eq!(candidate.rechecked, 6);
    for s in &projects {
        assert!(progress::current_solved(
            &candidate.progress.attempts[&s.id],
            s
        ));
    }
    let profile = dir.path().join("profile.json");
    progress::save(&profile, &p).unwrap();
    let prior = std::fs::read(&profile).unwrap();
    let restored = backup::restore(&profile, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), prior);
    let mut expected = full(&candidate.progress);
    let prior_json: serde_json::Value = serde_json::from_slice(&prior).unwrap();
    expected["migration_receipts"] = prior_json["migration_receipts"].clone();
    assert_eq!(
        expected["migration_receipts"],
        serde_json::json!({"workspace":true,"shell":[shell::EVALUATOR_REVISION]})
    );
    assert_eq!(
        full(&progress::decode(&std::fs::read(profile).unwrap(), &all).unwrap()),
        expected
    );
    assert!(
        candidate.progress.migration_receipts.is_none(),
        "portable backup does not claim main-file transition provenance"
    );
    assert_eq!(std::fs::read(&export).unwrap(), original);
    let cancelled = AtomicBool::new(true);
    let checked = AtomicUsize::new(0);
    assert!(backup::inspect(&export, &all, &cancelled, &checked).is_err());
    assert_eq!(std::fs::read(export).unwrap(), original);
}

#[test]
fn current_old_pack_bytes_and_real_legacy_records_are_not_reauthored_or_relabelled() {
    let r = review();
    for pin in r.original_source_pins {
        let path = std::path::Path::new(&pin.path);
        assert!(
            pin.path.starts_with("scenarios/")
                && path
                    .components()
                    .all(|x| matches!(x, std::path::Component::Normal(_)))
        );
        let bytes =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let original = if pin.path == "scenarios/units-v1.json" {
            original_units(&bytes).unwrap()
        } else {
            bytes
        };
        assert_eq!(digest::hash(&original), pin.sha256, "{}", pin.path);
    }
    let bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let expected: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let loaded = progress::decode(bytes, &catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        expected["attempts"]
    );
    assert!(
        authored()
            .iter()
            .all(|s| !loaded.attempts.contains_key(&s.id))
    );
    for a in loaded.attempts.values() {
        if let State::Script { run: Some(r), .. } = &a.state {
            assert_eq!(r.engine_revision, 8);
        }
    }
}
