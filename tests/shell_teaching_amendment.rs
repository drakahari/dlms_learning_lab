//! Teaching corrections change new practice defaults, never assignment identity
//! or retained work. All executions here are explicit current in-process VM tests.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, Lesson, View},
    shell, workspace,
};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize};

fn value<T: serde::Serialize>(input: &T) -> Value {
    serde_json::to_value(input).unwrap()
}
fn old_lesson(id: &str) -> Lesson {
    let file = teaching_amendment::manifest()["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["scenario_id"] == id)
        .unwrap();
    serde_json::from_value(file["original_teaching"].clone()).unwrap()
}
fn cases(s: &pack::Scenario) -> &[shell::Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("Script assignment required")
    };
    cases
}

#[test]
fn exact_closed_manifest_raw_bytes_and_typed_lessons_reject_unlisted_changes() {
    teaching_amendment::validate_manifest(teaching_amendment::MANIFEST_BYTES).unwrap();
    let mut changed = teaching_amendment::MANIFEST_BYTES.to_vec();
    changed.push(b' '); // Even equivalent JSON cannot amend a frozen manifest.
    assert!(teaching_amendment::validate_manifest(&changed).is_err());
    let all = pack::bundled().unwrap();
    for (path, current) in teaching_amendment::CURRENT_PACKS {
        let original = teaching_amendment::original_pack(path, current).unwrap();
        assert_ne!(original, current);
        let mut changed = current.to_vec();
        changed.push(b' ');
        assert!(teaching_amendment::original_pack(path, &changed).is_err());
        // An original pack cannot masquerade as the reviewed amended source.
        assert!(teaching_amendment::original_pack(path, &original).is_err());
    }
    for (_, id, _) in teaching_amendment::ALLOWED {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let original = teaching_amendment::original_scenario(s).unwrap();
        assert_eq!(original["teaching"], value(&old_lesson(id)));
        let mut altered = s.clone();
        altered.teaching.as_mut().unwrap().concepts[0]
            .text
            .push_str(" unreviewed");
        assert!(teaching_amendment::original_scenario(&altered).is_err());
        assert!(
            teaching_amendment::historical_example(id, altered.teaching.as_ref().unwrap()).is_err()
        );
        let mut altered = s.clone();
        altered
            .teaching
            .as_mut()
            .unwrap()
            .fixture
            .args
            .push("unreviewed".into());
        assert!(teaching_amendment::original_scenario(&altered).is_err());
        let mut altered = s.clone();
        altered.objective.push_str(" unreviewed assignment change");
        let projected = teaching_amendment::original_scenario(&altered).unwrap();
        assert_ne!(
            projected, original,
            "Non-teaching changes must remain visible"
        );
        assert_eq!(projected["objective"], altered.objective);
        assert_eq!(
            s.version, 1,
            "Teaching-only corrections do not bump assignment identity"
        );
    }
}

#[test]
fn all_current_fresh_examples_and_two_original_sources_match_the_same_declared_fixtures() {
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all).iter().filter(|s|matches!(s.goal,Goal::Script{..})).count(),57);
    let mut count = 0;
    let mut changed_defaults = Vec::new();
    for s in all.iter().filter(|s| matches!(s.goal, Goal::Script { .. })) {
        let lesson = s.teaching.as_ref().unwrap();
        let actual = shell::run(&lesson.example, &lesson.fixture);
        assert!(
            lesson.fixture.accepts(&actual),
            "{} fresh default: {:?}",
            s.id,
            actual.error
        );
        let historical = teaching_amendment::historical_example(&s.id, lesson).unwrap();
        if historical != lesson.example {
            changed_defaults.push(s.id.as_str());
            let old = old_lesson(&s.id);
            assert_eq!(value(&lesson.fixture), value(&old.fixture));
            assert!(
                old.fixture.accepts(&shell::run(&historical, &old.fixture)),
                "{} original source",
                s.id
            );
            assert!(
                old.fixture.accepts(&actual),
                "{} amended source has original outcome",
                s.id
            );
        }
        count += 1;
    }
    assert_eq!(count, 60);
    assert_eq!(changed_defaults, ["shell-bundle", "shell-text-capstone"]);
}

#[test]
fn documented_count_collision_and_threshold_experiments_reject_then_repair_without_file_changes() {
    let all = pack::bundled().unwrap();
    for (id, from, to, stderr) in [
        (
            "shell-text-capstone",
            "count=2",
            "count=1",
            "Choose two distinct example names\n",
        ),
        (
            "shell-text-capstone",
            "output=example-audit.txt",
            "output=labels.txt",
            "Choose two distinct example names\n",
        ),
        (
            "shell-bundle",
            "-lt 2",
            "-lt 3",
            "Need at least two example files\n",
        ),
    ] {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(lesson.example.matches(from).count(), 1);
        let draft = lesson.example.replacen(from, to, 1);
        let rejected = shell::run(&draft, &lesson.fixture);
        assert!(rejected.error.is_none(), "{id}: {:?}", rejected.error);
        assert_eq!(rejected.status, 2);
        assert_eq!(rejected.stdout, "");
        assert_eq!(rejected.stderr, stderr);
        assert_eq!(rejected.files, lesson.fixture.files);
        let repaired = draft.replacen(to, from, 1);
        assert_eq!(repaired, lesson.example);
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&repaired, &lesson.fixture))
        );
        let mut attempt = Attempt::new(s, 1);
        workspace::prepare(&mut attempt);
        teaching::prepare(&mut attempt, lesson, true);
        let assignment = value(&attempt.state);
        let learning = attempt
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning.example_source = draft;
        learning.example_result = Some(rejected);
        assert_eq!(value(&attempt.state), assignment);
        assert!(!progress::current_solved(&attempt, s));
    }
}

fn retained_current_profile(all: &[pack::Scenario]) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 19,
        ..Default::default()
    };
    for (_, id, _) in teaching_amendment::ALLOWED {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let mut a = Attempt::new(s, 11);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, &old_lesson(id), true);
        let source = format!("{}\n# retained assignment draft\n", s.solution[0]);
        a.state = State::Script {
            source: source.clone(),
            run: Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases(s).iter().map(|c| shell::run(&source, c)).collect(),
            }),
        };
        a.hints = 1;
        a.transcript.push("retained simulated work".into());
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source
            .push_str("\n# retained separate practice draft\n");
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        l.view = View::Example;
        assert!(progress::current_solved(&a, s));
        p.favorites.insert(id.into());
        p.attempts.insert(id.into(), a);
    }
    p
}

#[test]
fn saved_old_default_drafts_results_credits_and_favorites_survive_prepare_resume_backup_and_selected_reset()
 {
    let all = pack::bundled().unwrap();
    let mut p = retained_current_profile(&all);
    let before = value(&p);
    for s in &all {
        if let Some(a) = p.attempts.get_mut(&s.id) {
            workspace::prepare(a);
            teaching::prepare(a, s.teaching.as_ref().unwrap(), false);
        }
    }
    assert_eq!(
        value(&p),
        before,
        "Preparing an existing draft cannot replace its source or result"
    );
    let directory = tempfile::tempdir().unwrap();
    let profile = directory.path().join("profile.json");
    let export = directory.path().join("export.json");
    progress::save(&profile, &p).unwrap();
    let saved = std::fs::read(&profile).unwrap();
    let mut resumed = progress::load(&profile, &all).unwrap();
    assert_eq!(value(&resumed), before);
    assert_eq!(std::fs::read(&profile).unwrap(), saved);
    backup::export(&export, &resumed, &all).unwrap();
    let exported = std::fs::read(&export).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(
        candidate.rechecked, 10,
        "Five explicit assignment and five practice checks"
    );
    assert_eq!(
        value(&candidate.progress),
        before,
        "Rechecking retains actual saved sources"
    );
    let selected = all.iter().find(|s| s.id == "shell-text-capstone").unwrap();
    let mut reset = Attempt::new(selected, 20);
    workspace::prepare(&mut reset);
    teaching::prepare(&mut reset, selected.teaching.as_ref().unwrap(), true);
    assert!(!progress::current_solved(&reset, selected));
    assert_eq!(
        reset
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        selected.teaching.as_ref().unwrap().example
    );
    resumed.attempts.insert(selected.id.clone(), reset);
    assert_eq!(resumed.favorites, p.favorites);
    for (_, id, _) in teaching_amendment::ALLOWED {
        if id != selected.id {
            assert_eq!(value(&resumed.attempts[id]), before["attempts"][id]);
            assert!(progress::current_solved(
                &resumed.attempts[id],
                all.iter().find(|s| s.id == id).unwrap()
            ));
        }
    }
    progress::save(&profile, &resumed).unwrap();
    let reset_saved = std::fs::read(&profile).unwrap();
    let restored = backup::restore(&profile, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        reset_saved
    );
    assert_eq!(value(&progress::load(&profile, &all).unwrap()), before);
    assert_eq!(std::fs::read(&export).unwrap(), exported);
}

#[test]
fn genuine_writer8_records_and_separate_drafts_are_not_replaced_by_new_teaching_defaults() {
    let bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let old: Progress = serde_json::from_slice(bytes).unwrap();
    let all = pack::bundled().unwrap();
    let mut resumed = progress::decode(bytes, &all).unwrap();
    assert_eq!(
        value(&resumed),
        value(&old),
        "Read-only decode retains every actual saved field"
    );
    for (_, id, _) in teaching_amendment::ALLOWED {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let before = resumed.attempts[id].clone();
        let a = resumed.attempts.get_mut(id).unwrap();
        teaching::prepare(a, s.teaching.as_ref().unwrap(), false);
        teaching::prepare(a, s.teaching.as_ref().unwrap(), true);
        assert_eq!(
            value(a),
            value(&before),
            "Existing learning cannot be reset by prepare: {id}"
        );
        workspace::prepare(a); // Existing old main record moves to history by normal policy.
        let (
            State::Script { source, .. },
            State::Script {
                source: old_source,
                run: old_run,
            },
        ) = (&a.state, &before.state)
        else {
            panic!("Saved script required")
        };
        assert_eq!(source, old_source);
        let w = a.workspace.as_ref().unwrap();
        if let Some(record) = old_run {
            assert!(
                w.script_history
                    .as_ref()
                    .into_iter()
                    .chain(&w.script_history_more)
                    .any(|retained| retained == record),
                "Exact actual8 record retained: {id}"
            );
        }
        assert_eq!(
            value(&w.script_learning),
            value(&before.workspace.as_ref().unwrap().script_learning)
        );
        assert_eq!(a.scenario_version, before.scenario_version);
        assert_eq!(a.transcript, before.transcript);
        assert_eq!(a.visited, before.visited);
        assert_eq!(a.challenge, before.challenge);
        assert_eq!(a.hints, before.hints);
        assert!(
            !progress::current_solved(a, s),
            "Old records cannot acquire current credit"
        );
    }
    assert_eq!(resumed.favorites, old.favorites);
}
