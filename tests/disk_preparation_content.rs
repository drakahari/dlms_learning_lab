//! Prospective authored state/behavior/source gates. No author engine or host disk run.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    partition_fs_workspace as work,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde_json::{Value, json};
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as registration;
const DATA: &str = include_str!("../scenarios/disk-preparation-v1.json");
const REVIEW: &str = include_str!("fixtures/disk-preparation-review.json");
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn labs() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(DATA).unwrap().scenarios
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn admit(raw: &str) -> Result<Vec<Scenario>, String> {
    let current = pack::bundled().unwrap();
    let before = registration::before_scenarios(&current);
    let mut texts = before
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    texts.push(raw.into());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::PartitionFs { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &Attempt) -> &work::Session {
    let State::PartitionFs { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    work::binding::prepare(&mut a, s).unwrap();
    a
}
fn rows(a: &Attempt, s: &Scenario) -> Vec<bool> {
    engine::checks(&a.state, &s.goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
fn bits(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn operations(v: &Value) -> Vec<work::Operation> {
    serde_json::from_value(v.clone()).unwrap()
}
fn run(a: &mut Attempt, s: &Scenario, ops: &Value) {
    for o in operations(ops) {
        work::binding::apply(a, s, &o, false).unwrap();
    }
}
fn profile(all: &[Scenario]) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    for s in all {
        p.attempts.insert(s.id.clone(), fresh(s));
    }
    p
}
fn files(dir: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect()
}
#[test]
fn independent_four_task_pack_and_appended_unit_preserve_full_current_catalog_and_prerequisites() {
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 197);
    let before = registration::before_scenarios(&current).to_vec();
    assert_eq!(before.len(), 193);
    let old = value(&before);
    let original_units =
        registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let raw = std::str::from_utf8(&original_units).unwrap();
    let old_units = course::load(raw, &before).unwrap();
    let old_unit_values = value(&old_units);
    let mut combined = before.clone();
    let extra = labs();
    assert_eq!(extra.len(), 4);
    assert!(extra.iter().all(|s| !before.iter().any(|b| b.id == s.id)));
    combined.extend(extra.clone());
    let texts = combined
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let all = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(value(&all[..before.len()]), old);
    assert_eq!(value(&all[before.len()..]), value(&extra));
    assert_eq!(value(&all), value(&current));
    let round: Vec<Scenario> = serde_json::from_value(value(&all)).unwrap();
    assert_eq!(value(&round), value(&all));
    let mut index: Value = serde_json::from_str(raw).unwrap();
    index["units"]
        .as_array_mut()
        .unwrap()
        .push(review()["unit_proposal"].clone());
    let units = course::load(&index.to_string(), &all).unwrap();
    assert_eq!(
        value(&units),
        value(&course::load(include_str!("../scenarios/units-v1.json"), &current).unwrap())
    );
    assert_eq!(value(&units[..old_units.len()]), old_unit_values);
    assert_eq!(
        units.last().unwrap().labs,
        extra.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let expected_worked = [2, 2, 5, 4];
    let expected_rows = [2, 3, 3, 3];
    for (i, s) in extra.iter().enumerate() {
        let d = definition(s);
        d.validate().unwrap();
        work::binding::source(s).unwrap();
        assert_eq!(d.lesson.concepts.len(), 6);
        assert_eq!(d.lesson.worked.len(), expected_worked[i]);
        assert_eq!(rows(&fresh(s), s).len(), expected_rows[i]);
        assert!(rows(&fresh(s), s).contains(&false));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|s| &s.id == id))
        );
        assert!(pack::DIFFICULTIES.contains(&s.difficulty.as_str()));
        assert!(
            d.initial
                .disks
                .keys()
                .all(|p| !d.lesson.fixture.disks.contains_key(p))
        );
        assert!(
            d.lesson
                .concepts
                .iter()
                .any(|c| c.text.contains("type hint"))
        );
        assert!(s.simplifications.contains("No real device access"));
        assert!(
            s.hints.len() == 3 && extra.iter().filter(|other| other.hints == s.hints).count() == 1
        );
    }
    // Distinct work: existing-GPT gap; formatting-only; eligibility/new table; mixed media.
    assert!(
        definition(&extra[0])
            .reference
            .iter()
            .all(|o| !matches!(o, work::Operation::CreateGpt { .. }))
    );
    assert!(
        definition(&extra[1])
            .reference
            .iter()
            .all(|o| matches!(o, work::Operation::Format { .. }))
    );
    assert!(definition(&extra[2]).initial.disks["/dev/vda"].occupied_without_table);
    assert_eq!(
        definition(&extra[3]).plan.targets[1].eligible_disks.len(),
        2
    );
}
#[test]
fn each_reference_and_two_distinct_allowed_alternatives_satisfy_current_state_not_a_script_string()
{
    let r = review();
    for (s, v) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        assert_eq!(value(&definition(s).reference), v["reference"]);
        assert_eq!(v["alternatives"].as_array().unwrap().len(), 2);
        let mut finals = vec![];
        for ops in std::iter::once(&v["reference"]).chain(v["alternatives"].as_array().unwrap()) {
            let mut a = fresh(s);
            run(&mut a, s, ops);
            assert_eq!(rows(&a, s), bits(&v["positive_expected"]));
            assert!(progress::current_solved(&a, s));
            assert!(!session(&a).checked); // State truth is not a cached Check badge.
            work::binding::check(&mut a, s, false).unwrap();
            assert!(session(&a).checked);
            finals.push(value(&session(&a).current));
        }
        assert_ne!(finals[0], finals[1]);
        assert_ne!(finals[0], finals[2]);
        assert_ne!(finals[1], finals[2]);
        assert!(v["actual"].is_null());
    }
}
#[test]
fn all_twenty_eight_wrong_partial_and_late_damage_vectors_are_full_row_predictions() {
    let r = review();
    let mut total = 0;
    for (s, v) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        for wrong in v["wrong_and_partial"].as_array().unwrap() {
            let mut a = fresh(s);
            run(&mut a, s, &wrong["operations"]);
            work::binding::check(&mut a, s, false).unwrap();
            let expected = bits(&wrong["expected_rows"]);
            assert!(expected.contains(&false));
            assert_eq!(rows(&a, s), expected, "{}: {}", s.id, wrong["name"]);
            assert!(!progress::current_solved(&a, s));
            assert!(wrong["actual"].is_null());
            total += 1;
        }
    }
    assert_eq!(total, 28);
}
#[test]
fn plausible_unsafe_overlap_overwrite_and_protected_media_actions_refuse_whole_attempt_then_valid_work_runs()
 {
    let r = review();
    for (s, v) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        for op in operations(&v["refusals_on_initial"]) {
            let mut a = fresh(s);
            let before = value(&a);
            assert!(work::binding::apply(&mut a, s, &op, false).is_err());
            assert_eq!(value(&a), before);
        }
        let mut good = fresh(s);
        run(&mut good, s, &v["reference"]);
        assert!(progress::current_solved(&good, s));
    }
}
#[test]
fn copied_answers_from_other_assignments_and_canned_first_fixture_operations_do_not_gain_success() {
    let all = labs();
    let r = review();
    for (i, s) in all.iter().enumerate() {
        for (j, other) in r["labs"].as_array().unwrap().iter().enumerate() {
            if i == j {
                continue;
            }
            let mut a = fresh(s);
            for op in operations(&other["reference"]) {
                let before = value(&a);
                if work::binding::apply(&mut a, s, &op, false).is_err() {
                    assert_eq!(value(&a), before);
                    break;
                }
            }
            assert!(
                !progress::current_solved(&a, s),
                "{} cannot use {} as a canned answer",
                s.id,
                all[j].id
            );
        }
    }
}
#[test]
fn valid_extra_allocation_invalidates_success_and_reset_manual_replay_is_the_honest_repair() {
    let r = review();
    for (s, v) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let mut a = fresh(s);
        run(&mut a, s, &v["reference"]);
        assert!(progress::current_solved(&a, s));
        let damage: work::Operation =
            serde_json::from_value(v["damage_after_reference"]["operation"].clone()).unwrap();
        work::binding::apply(&mut a, s, &damage, false).unwrap();
        assert_eq!(
            rows(&a, s),
            bits(&v["damage_after_reference"]["expected_rows"])
        );
        assert!(!progress::current_solved(&a, s));
        let wrong = value(&a);
        let mut reset = fresh(s);
        assert!(rows(&reset, s).contains(&false));
        assert_eq!(value(&a), wrong);
        run(&mut reset, s, &v["reference"]);
        assert!(progress::current_solved(&reset, s));
        assert_ne!(value(&reset), wrong);
    }
}
#[test]
fn all_four_independent_practices_insert_without_execution_then_experiment_reset_and_replay() {
    for s in labs() {
        let d = definition(&s);
        let mut a = fresh(&s);
        let assignment = value(&session(&a).current);
        assert_eq!(session(&a).view, View::Introduction);
        assert!(!session(&a).example_checked);
        work::binding::set_view(&mut a, &s, View::Example).unwrap();
        assert!(d.practice_checks(session(&a)).iter().any(|r| !r.0));
        for (n, step) in d.lesson.worked.iter().enumerate() {
            let before = value(&session(&a).example);
            let transcript = value(&session(&a).example_transcript);
            work::binding::select_example(&mut a, &s, n).unwrap();
            work::binding::insert(&mut a, &s, n).unwrap();
            assert_eq!(value(&session(&a).example), before);
            assert_eq!(value(&session(&a).example_transcript), transcript);
            assert!(!session(&a).example_checked);
            work::binding::apply(&mut a, &s, &step.operation, true).unwrap();
        }
        work::binding::check(&mut a, &s, true).unwrap();
        assert!(session(&a).example_checked);
        assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
        work::binding::apply(&mut a, &s, &d.lesson.experiment, true).unwrap();
        assert!(d.practice_checks(session(&a)).iter().any(|r| !r.0));
        assert!(!session(&a).example_checked);
        assert_eq!(value(&session(&a).current), assignment);
        assert!(!progress::current_solved(&a, &s));
        work::binding::reset_example(&mut a, &s).unwrap();
        assert_eq!(session(&a).example, d.lesson.fixture);
        assert!(session(&a).example_transcript.is_empty());
        for step in &d.lesson.worked {
            work::binding::apply(&mut a, &s, &step.operation, true).unwrap();
        }
        assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
        assert_eq!(value(&session(&a).current), assignment);
    }
}
#[test]
fn unfinished_drafts_are_persisted_separately_from_valid_applied_configuration_and_modes() {
    let r = review();
    for (s, v) in labs().iter().zip(r["labs"].as_array().unwrap()) {
        let mut a = fresh(s);
        run(&mut a, s, &v["reference"]);
        work::binding::check(&mut a, s, false).unwrap();
        let model = value(&session(&a).current);
        let disk = session(&a).assignment_editor.selected.clone();
        let mut draft = session(&a).assignment_editor.drafts[&disk].clone();
        draft.start = "unfinished".into();
        draft.end = "".into();
        work::binding::set_draft(&mut a, s, &disk, draft, false).unwrap();
        assert_eq!(value(&session(&a).current), model);
        assert!(progress::current_solved(&a, s));
        work::binding::set_challenge(&mut a, s, true).unwrap();
        assert!(a.challenge);
        assert!(progress::current_solved(&a, s));
        work::binding::set_challenge(&mut a, s, false).unwrap();
        assert!(!a.challenge);
        assert_eq!(
            session(&a).assignment_editor.drafts[&disk].start,
            "unfinished"
        );
    }
}
#[test]
fn warm_cold_export_restore_and_reset_retain_all_four_contexts_practice_drafts_and_real_receipts() {
    let all = labs();
    let r = review();
    let mut p = profile(&all);
    for (s, v) in all.iter().zip(r["labs"].as_array().unwrap()) {
        let a = p.attempts.get_mut(&s.id).unwrap();
        run(a, s, &v["alternatives"][0]);
        work::binding::insert(a, s, 0).unwrap();
        work::binding::request_hint(a, s).unwrap();
        work::binding::check(a, s, false).unwrap();
    }
    let before = value(&p);
    let receipt = value(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(value(&p), before);
    assert_eq!(value(&p.migration_receipts), receipt);
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(value(&loaded.attempts), value(&p.attempts));
    assert!(loaded.migration_receipts.is_some());
    let exported = dir.path().join("portable.json");
    backup::export(&exported, &p, &all).unwrap();
    let candidate = backup::inspect(
        &exported,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(value(&candidate.progress.attempts), value(&p.attempts));
    let restored = dir.path().join("restored.json");
    backup::restore(&restored, &candidate, &all).unwrap();
    let cold = progress::load(&restored, &all).unwrap();
    assert_eq!(value(&cold.attempts), value(&p.attempts));
    let reset = fresh(&all[0]);
    assert!(!progress::current_solved(&reset, &all[0]));
    assert_eq!(value(&cold.attempts), value(&p.attempts));
    assert!(
        all.iter()
            .all(|s| progress::current_solved(&cold.attempts[&s.id], s))
    );
}
#[test]
fn every_nonselected_source_is_authoritative_and_context_free_or_joint_forged_save_is_atomic() {
    let all = labs();
    let mut p = profile(&all);
    let r = review();
    for (s, v) in all.iter().zip(r["labs"].as_array().unwrap()) {
        run(p.attempts.get_mut(&s.id).unwrap(), s, &v["reference"]);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let old = files(dir.path());
    let caller = value(&p);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(files(dir.path()), old);
    assert_eq!(value(&p), caller);
    for s in &all {
        let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
        let a = bad.attempts.get_mut(&s.id).unwrap();
        let State::PartitionFs { simulation } = &mut a.state else {
            panic!()
        };
        simulation.origin.plan.targets[0].minimum_bytes = 512; // Saved witnesses do not replace the held source, even if state/Goal are jointly carried elsewhere.
        let forged_goal = Goal::PartitionFs {
            definition: {
                let mut d = definition(s).clone();
                d.plan.targets[0].minimum_bytes = 512;
                Box::new(d)
            },
        };
        assert_eq!(
            value(&forged_goal)["definition"]["plan"]["targets"][0]["minimum_bytes"],
            json!(512)
        );
        assert!(
            engine::solved(&a.state, &forged_goal),
            "A matching self-carried Goal is not authority"
        );
        assert!(
            !progress::current_solved(a, s),
            "Held source must reject the joint forgery"
        );
        let before = value(&bad);
        assert!(progress::save_with_catalog(&path, &bad, &all).is_err());
        assert_eq!(files(dir.path()), old);
        assert_eq!(value(&bad), before);
        let held = all
            .iter()
            .filter(|x| x.id != s.id)
            .cloned()
            .collect::<Vec<_>>();
        assert!(progress::save_with_catalog(&path, &p, &held).is_err());
        assert_eq!(files(dir.path()), old);
    }
}
#[test]
fn foreign_historical_metadata_and_bad_source_packs_never_replace_progress_or_create_credit() {
    let all = labs();
    assert!(admit(DATA).is_ok());
    for s in &all {
        let mut a = fresh(s);
        let before = value(&a);
        a.workspace.as_mut().unwrap().checked = true;
        assert!(work::binding::context(&a, s).is_err());
        assert!(!progress::current_solved(&a, s));
        assert_ne!(value(&a), before);
    }
    let mut malformed: Value = serde_json::from_str(DATA).unwrap();
    malformed["scenarios"][0]["goal"]["definition"]["plan"]["targets"][0]["minimum_bytes"] =
        0.into();
    assert!(admit(&malformed.to_string()).is_err());
    let mut impossible: Value = serde_json::from_str(DATA).unwrap();
    impossible["scenarios"][1]["goal"]["definition"]["reference"][0]["label"] = "incorrect".into();
    assert!(admit(&impossible.to_string()).is_err());
    let mut shared: Value = serde_json::from_str(DATA).unwrap();
    shared["scenarios"][2]["goal"]["definition"]["lesson"]["fixture"] =
        shared["scenarios"][2]["goal"]["definition"]["initial"].clone();
    assert!(admit(&shared.to_string()).is_err());
    assert_eq!(review()["actual"], Value::Null);
    assert!(!review()["normal_registration"].as_bool().unwrap());
}
#[test]
fn native_configuration_is_not_a_shell_alias_and_limits_remain_honest_on_all_four_tasks() {
    for s in labs() {
        let mut a = fresh(&s);
        let before = value(&a);
        for command in [
            "parted -s /dev/vda mklabel gpt",
            "mkfs.ext4 /dev/vda1",
            "swapon /dev/vda1",
            "mount /dev/vda1 /mnt",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, command).is_err());
            assert_eq!(value(&a), before);
        }
        let help = engine::execute_for_goal(&mut a.state, &s.goal, "help").unwrap();
        assert!(help.contains("No delete/resize/overwrite"));
        assert_eq!(value(&a), before);
        assert!(serde_json::to_vec(&(s.clone(), &a)).unwrap().len() < work::MAX_CONTEXT);
        assert!(serde_json::to_vec(&definition(&s).initial).unwrap().len() < work::MAX_MODEL);
    }
}
