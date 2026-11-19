//! Unregistered authored-source gates. Every group is prospective, not runtime evidence.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    syslog as core, syslog_workspace as work,
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicUsize};
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk;
const DATA: &str = include_str!("../scenarios/syslog-policy-v1.json");
const REVIEW: &str = include_str!("fixtures/syslog-policy-review.json");
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
fn labs() -> Vec<Scenario> {
    serde_json::from_str::<pack::Pack>(DATA).unwrap().scenarios
}
fn admit(raw: &str) -> Result<Vec<Scenario>, String> {
    let current = predecessor197();
    let mut texts = current
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    texts.push(raw.into());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::Syslog { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &Attempt) -> &work::Session {
    let State::Syslog { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn session_mut(a: &mut Attempt) -> &mut work::Session {
    let State::Syslog { simulation } = &mut a.state else {
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
fn actions(v: &Value) -> Vec<work::Action> {
    serde_json::from_value(v.clone()).unwrap()
}
fn run(a: &mut Attempt, s: &Scenario, list: &Value) {
    for action in actions(list) {
        engine::execute_for_goal(
            &mut a.state,
            &s.goal,
            &serde_json::to_string(&action).unwrap(),
        )
        .unwrap();
        work::context(a, s).unwrap();
    }
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
fn four_distinct_native_tasks_append_privately_without_changing_full197_sources_or52_units() {
    let current = predecessor197();
    assert_eq!(current.len(), 197);
    assert_eq!(disk::before_scenarios(&current).len(), 193);
    let original = value(&current);
    let prior53 = disk::literal_registration::snapshot_registration::before_units(include_bytes!(
        "../scenarios/units-v1.json"
    ))
    .unwrap();
    let prior52 = disk::literal_registration::before_units(&prior53).unwrap();
    let original_units = course::load(std::str::from_utf8(&prior52).unwrap(), &current).unwrap();
    assert_eq!(original_units.len(), 52);
    let extra = labs();
    assert_eq!(extra.len(), 4);
    let all = admit(DATA).unwrap();
    assert_eq!(all.len(), 201);
    assert_eq!(value(&all[..197]), original);
    assert_eq!(value(&all[197..]), value(&extra));
    let round: Vec<Scenario> = serde_json::from_value(value(&all)).unwrap();
    assert_eq!(value(&round), value(&all));
    let mut index: Value = serde_json::from_slice(&prior52).unwrap();
    index["units"]
        .as_array_mut()
        .unwrap()
        .push(review()["unit_proposal"].clone());
    let units = course::load(&index.to_string(), &all).unwrap();
    assert_eq!(units.len(), 53);
    assert_eq!(value(&units[..52]), value(&original_units));
    assert_eq!(
        units[52].labs,
        extra.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let expected_rows = [11, 15, 15, 52];
    let expected_sources = [1, 1, 2, 3];
    for (i, s) in extra.iter().enumerate() {
        let d = definition(s);
        d.validate().unwrap();
        work::source(s).unwrap();
        assert_eq!(
            rows(&fresh(s), s),
            bits(&review()["labs"][i]["initial_rows"])
        );
        assert_eq!(rows(&fresh(s), s).len(), expected_rows[i]);
        assert_eq!(
            d.initial.environment().unwrap().devices.len(),
            expected_sources[i]
        );
        assert_eq!(d.lesson.concepts.len(), 6);
        assert_eq!(s.steps.len(), s.guide.len());
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        assert!(pack::DIFFICULTIES.contains(&s.difficulty.as_str()));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|s| &s.id == id))
        );
        let e = d.initial.environment().unwrap();
        let p = d.lesson.fixture.environment().unwrap();
        assert!(
            e.devices
                .keys()
                .chain(e.collectors.keys())
                .all(|ip| !p.devices.contains_key(ip) && !p.collectors.contains_key(ip))
        );
        assert!(e.events.keys().all(|id| !p.events.contains_key(id)));
        assert!(
            s.hints.len() == 3 && extra.iter().filter(|other| other.hints == s.hints).count() == 1
        );
    }
    assert_eq!(value(&predecessor197()), original);
    // A separate full-current positive retains the exact199/53 catalog while
    // adding these same four sources privately, without normal registration.
    let actual201 = pack::bundled().unwrap();
    let actual199 =
        disk::literal_registration::snapshot_registration::before_scenarios(&actual201).to_vec();
    assert_eq!((actual201.len(), actual199.len()), (201, 199));
    assert_eq!(actual199.len(), 199);
    assert_eq!(
        value(disk::literal_registration::before_scenarios(&actual199)),
        original
    );
    let units54_current = course::bundled(&actual201).unwrap();
    assert_eq!(units54_current.len(), 54);
    let units53 =
        disk::literal_registration::snapshot_registration::before_typed_units(&units54_current)
            .to_vec();
    assert_eq!(units53.len(), 53);
    assert_eq!(
        value(disk::literal_registration::before_typed_units(&units53)),
        value(&original_units)
    );
    let mut with203 = actual199.clone();
    with203.extend(labs());
    let chunks = with203
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let with203 =
        pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(with203.len(), 203);
    assert_eq!(value(&with203[..199]), value(&actual199));
    assert_eq!(value(&with203[199..]), value(&extra));
    let raw53 = disk::literal_registration::snapshot_registration::before_units(include_bytes!(
        "../scenarios/units-v1.json"
    ))
    .unwrap();
    let mut index53: Value = serde_json::from_slice(&raw53).unwrap();
    index53["units"]
        .as_array_mut()
        .unwrap()
        .push(review()["unit_proposal"].clone());
    let units54 = course::load(&index53.to_string(), &with203).unwrap();
    assert_eq!(units54.len(), 54);
    assert_eq!(value(&units54[..53]), value(&units53));
    assert_eq!(
        units54[53].labs,
        extra.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let old_profile = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &with203[199..] {
        assert!(
            matches!(source.initial, State::Syslog { .. })
                && matches!(source.goal, Goal::Syslog { .. })
        );
        work::source(source).unwrap();
        assert!(
            !old_profile.attempts.contains_key(&source.id)
                && !old_profile.favorites.contains(&source.id)
        );
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
    // Separate full-current private205 world: its four new sources are Syslog,
    // never the unrelated conditional normal HSRP205 registration.
    let mut with205 = actual201.clone();
    with205.extend(labs());
    let chunks = with205
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let with205 =
        pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(with205.len(), 205);
    assert_eq!(value(&with205[..201]), value(&actual201));
    assert_eq!(value(&with205[201..]), value(&extra));
    let mut index54: Value =
        serde_json::from_slice(include_bytes!("../scenarios/units-v1.json")).unwrap();
    index54["units"]
        .as_array_mut()
        .unwrap()
        .push(review()["unit_proposal"].clone());
    let units55 = course::load(&index54.to_string(), &with205).unwrap();
    assert_eq!(units55.len(), 55);
    assert_eq!(value(&units55[..54]), value(&units54_current));
    assert_eq!(
        units55[54].labs,
        extra.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    let round: Vec<Scenario> = serde_json::from_value(value(&with205)).unwrap();
    assert_eq!(value(&round), value(&with205));
    for source in &with205[201..] {
        assert!(
            matches!(source.initial, State::Syslog { .. })
                && matches!(source.goal, Goal::Syslog { .. })
        );
        work::source(source).unwrap();
        assert!(
            !old_profile.attempts.contains_key(&source.id)
                && !old_profile.favorites.contains(&source.id)
        );
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
}

#[test]
fn full_reference_and_two_behavioral_alternatives_pass_every_positive_negative_and_protected_row() {
    let r = review();
    for (i, s) in labs().iter().enumerate() {
        let v = &r["labs"][i];
        assert_eq!(
            s.solution,
            actions(&v["reference"]["actions"])
                .iter()
                .map(|a| serde_json::to_string(a).unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(v["alternatives"].as_array().unwrap().len(), 2);
        for candidate in
            std::iter::once(&v["reference"]).chain(v["alternatives"].as_array().unwrap())
        {
            assert!(candidate["actual"].is_null());
            let mut a = fresh(s);
            assert!(!progress::current_solved(&a, s));
            run(&mut a, s, &candidate["actions"]);
            assert_eq!(rows(&a, s), bits(&candidate["rows"]));
            assert!(progress::current_solved(&a, s));
            assert!(rows(&a, s).len() > 1 && rows(&a, s).iter().all(|b| *b));
        }
    }
}

#[test]
fn twenty_four_plausible_wrong_or_partial_policies_have_exact_failure_masks_and_readable_reasons() {
    let r = review();
    let mut count = 0;
    for (i, s) in labs().iter().enumerate() {
        let cases = r["labs"][i]["wrong_partial"].as_array().unwrap();
        assert_eq!(cases.len(), 6);
        for case in cases {
            let mut a = fresh(s);
            run(&mut a, s, &case["actions"]);
            assert_eq!(
                rows(&a, s),
                bits(&case["rows"]),
                "{} {}",
                s.id,
                case["name"]
            );
            assert!(!progress::current_solved(&a, s));
            let checks = definition(s).checks(session(&a));
            assert!(checks.iter().any(|r| !r.0));
            assert!(
                checks
                    .iter()
                    .filter(|r| !r.0)
                    .all(|r| !r.1.trim().is_empty())
            );
            assert!(!case["explanation"].as_str().unwrap().is_empty());
            assert!(case["actual"].is_null());
            count += 1;
        }
    }
    assert_eq!(count, 24);
}

#[test]
fn historical_receipts_and_checked_flags_never_preserve_success_after_current_policy_damage() {
    let r = review();
    for (i, s) in labs().iter().enumerate() {
        let d = definition(s);
        let v = &r["labs"][i];
        let mut a = fresh(s);
        run(&mut a, s, &v["reference"]["actions"]);
        let t = &d.targets[0];
        session_mut(&mut a)
            .action(
                d,
                false,
                &work::Action::Generate {
                    device: t.device,
                    event: t.event.clone(),
                },
            )
            .unwrap();
        assert!(
            !session(&a)
                .current
                .local_history(t.device)
                .unwrap()
                .is_empty()
                || d.initial
                    .device_policy(t.device)
                    .unwrap()
                    .buffer_threshold
                    .is_none()
        );
        let receipts = value(&session(&a).current)["remote"].clone();
        assert!(
            receipts
                .as_object()
                .unwrap()
                .values()
                .any(|v| !v.as_array().unwrap().is_empty())
        );
        let probe_before = value(&session(&a).current);
        for target in &d.targets {
            session(&a)
                .current
                .current_probe(target.device, &target.event)
                .unwrap();
        }
        assert_eq!(value(&session(&a).current), probe_before);
        run(&mut a, s, &v["damage"]["actions"]);
        session_mut(&mut a).checked = true;
        assert_eq!(rows(&a, s), bits(&v["damage"]["rows"]));
        assert!(!progress::current_solved(&a, s));
        assert_eq!(value(&session(&a).current)["remote"], receipts);
        for ip in d.initial.environment().unwrap().collectors.keys() {
            session_mut(&mut a)
                .action(d, false, &work::Action::ClearCollector { collector: *ip })
                .unwrap();
        }
        assert_eq!(rows(&a, s), bits(&v["damage"]["rows"]));
        run(&mut a, s, &v["damage"]["repair"]);
        assert_eq!(rows(&a, s), bits(&v["damage"]["repaired_rows"]));
        assert!(progress::current_solved(&a, s));
    }
}

#[test]
fn separate_worked_fill_is_not_apply_and_experiment_repair_reset_leave_assignment_and_drafts_exact()
{
    let r = review();
    for (i, s) in labs().iter().enumerate() {
        let d = definition(s);
        let mut a = fresh(s);
        let assignment = value(&session(&a).current);
        let draft = value(&session(&a).editor);
        assert!(!session(&a).example_checked);
        for n in 0..d.lesson.worked.len() {
            let before = value(&session(&a).example);
            session_mut(&mut a).fill_worked(d, n).unwrap();
            assert_eq!(value(&session(&a).example), before);
            assert!(session(&a).has_unapplied(d, true).unwrap());
            assert!(!d.practice_checks(session(&a))[0].0);
            session_mut(&mut a).apply_selected(d, true).unwrap();
        }
        assert_eq!(
            d.practice_checks(session(&a))
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            bits(&r["labs"][i]["practice"]["worked_rows"])
        );
        let t = &d.lesson.targets[0];
        session_mut(&mut a)
            .action(
                d,
                true,
                &work::Action::Generate {
                    device: t.device,
                    event: t.event.clone(),
                },
            )
            .unwrap();
        session_mut(&mut a)
            .action(d, true, &d.lesson.experiment)
            .unwrap();
        assert_eq!(
            d.practice_checks(session(&a))
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            bits(&r["labs"][i]["practice"]["experiment_rows"])
        );
        session_mut(&mut a)
            .action(d, true, &d.lesson.repair)
            .unwrap();
        assert_eq!(
            d.practice_checks(session(&a))
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            bits(&r["labs"][i]["practice"]["repaired_rows"])
        );
        assert_eq!(value(&session(&a).current), assignment);
        assert_eq!(value(&session(&a).editor), draft);
        assert!(!progress::current_solved(&a, s));
        session_mut(&mut a).reset_example(d).unwrap();
        let expected = work::Session::new(d).unwrap();
        assert_eq!(session(&a).example, expected.example);
        assert_eq!(session(&a).example_editor, expected.example_editor);
        assert!(!session(&a).example_checked);
        assert_eq!(session(&a).selected_example, 0);
        assert_eq!(value(&session(&a).current), assignment);
    }
}

#[test]
fn pending_invalid_host_and_unfinished_selector_survive_codec_but_apply_is_atomic_and_not_success()
{
    let r = review();
    for (i, s) in labs().iter().enumerate() {
        let d = definition(s);
        let mut a = fresh(s);
        run(&mut a, s, &r["labs"][i]["reference"]["actions"]);
        let ip = *d
            .initial
            .environment()
            .unwrap()
            .devices
            .keys()
            .next()
            .unwrap();
        session_mut(&mut a)
            .select(d, false, work::Component::Source(ip))
            .unwrap();
        let mut draft = session(&a).editor.sources[&ip].clone();
        draft.hosts = vec!["10.214.".into()];
        session_mut(&mut a)
            .set_source_draft(d, false, ip, draft.clone())
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        let before = value(&a);
        assert!(session_mut(&mut a).apply_selected(d, false).is_err());
        assert_eq!(value(&a), before);
        let collector = *d
            .initial
            .environment()
            .unwrap()
            .collectors
            .keys()
            .next()
            .unwrap();
        session_mut(&mut a)
            .select(d, false, work::Component::Collector(collector))
            .unwrap();
        let unfinished = work::CollectorDraft {
            rows: vec![work::SelectorDraft {
                facility: None,
                threshold: core::Severity::Warning,
            }],
        };
        session_mut(&mut a)
            .set_collector_draft(d, false, collector, unfinished.clone())
            .unwrap();
        let before = value(&a);
        assert!(session_mut(&mut a).apply_selected(d, false).is_err());
        assert_eq!(value(&a), before);
        let raw = serde_json::to_vec(session(&a)).unwrap();
        let reopened = work::Session::decode(&raw).unwrap();
        assert_eq!(reopened, *session(&a));
        assert_eq!(reopened.editor.sources[&ip], draft);
        assert_eq!(reopened.editor.collectors[&collector], unfinished);
        assert!(!rows(&a, s)[0]);
    }
}

#[test]
fn hints_are_persisted_one_at_a_time_without_running_an_action_or_changing_other_scopes() {
    let extra = labs();
    for s in &extra {
        let mut a = fresh(s);
        let original = value(session(&a));
        for expected in 1..=3 {
            work::binding::request_hint(&mut a, s).unwrap();
            assert_eq!(a.hints, expected);
            assert_eq!(value(session(&a)), original);
        }
        let before = value(&a);
        assert!(work::binding::request_hint(&mut a, s).is_err());
        assert_eq!(value(&a), before);
        work::binding::set_challenge(&mut a, s, true).unwrap();
        assert!(a.challenge);
        assert_eq!(a.hints, 3);
        assert_eq!(value(session(&a)), original);
        assert!(!progress::current_solved(&a, s));
    }
}

#[test]
fn source_aware_roundtrip_restore_preserves_four_drafts_histories_hints_old_progress_and_default_none_caller_receipts()
 {
    let all = admit(DATA).unwrap();
    let extra = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    let old = &all[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 0));
    let old_bytes = value(&p.attempts[&old.id]);
    for s in &extra {
        let mut a = fresh(s);
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        let d = definition(s);
        session_mut(&mut a).fill_worked(d, 0).unwrap();
        let t = &d.targets[0];
        session_mut(&mut a)
            .action(
                d,
                false,
                &work::Action::Generate {
                    device: t.device,
                    event: t.event.clone(),
                },
            )
            .unwrap();
        work::binding::request_hint(&mut a, s).unwrap();
        if s.id == extra[3].id {
            let mut draft = session(&a).editor.sources[&t.device].clone();
            draft.hosts = vec!["10.217.".into()];
            session_mut(&mut a)
                .set_source_draft(d, false, t.device, draft)
                .unwrap();
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let portable = value(&p);
    let receipts = value(&p.migration_receipts);
    // This is only an absent-caller-metadata control, not genuine nonempty provenance.
    assert!(receipts.is_null());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(value(&progress::load(&path, &all).unwrap()), portable);
    assert_eq!(value(&p.migration_receipts), receipts);
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let raw_export = std::fs::read(&export).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(value(&candidate.progress), portable);
    let restored = dir.path().join("restored.json");
    backup::restore(&restored, &candidate, &all).unwrap();
    let warm = progress::load(&restored, &all).unwrap();
    assert_eq!(value(&warm), portable);
    assert_eq!(value(&warm.attempts[&old.id]), old_bytes);
    assert_eq!(std::fs::read(export).unwrap(), raw_export);
    for (i, s) in extra.iter().enumerate() {
        assert_eq!(progress::current_solved(&warm.attempts[&s.id], s), i != 3);
        let mut reset = warm.attempts[&s.id].clone();
        reset.state = s.initial.clone();
        reset.hints = 0;
        work::context(&reset, s).unwrap();
        assert!(!progress::current_solved(&reset, s));
        assert_eq!(value(&warm), portable);
    }
}

#[test]
fn inactive_forged_environment_and_foreign_metadata_refuse_save_without_replacing_any_original_bytes()
 {
    let all = admit(DATA).unwrap();
    let extra = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    for s in &extra {
        p.attempts.insert(s.id.clone(), fresh(s));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let original = files(dir.path());
    let caller = value(&p);
    let s = &extra[3]; // Inactive relative to the first task: all Attempts must be checked.
    for mode in 0..5 {
        let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
        let a = bad.attempts.get_mut(&s.id).unwrap();
        match mode {
            0 => a.scenario_version += 1,
            1 => a.workspace.as_mut().unwrap().command = "old terminal draft".into(),
            2 => {
                let mut v = value(&*a);
                v["state"]["simulation"]["current"]["environment"]["deliveries"][0]["available"] =
                    false.into();
                *a = serde_json::from_value(v).unwrap();
            }
            3 => {
                let mut v = value(&*a);
                let id = definition(s).targets[0].event.as_str();
                v["state"]["simulation"]["current"]["environment"]["events"][id]["text"] =
                    "Forged event".into();
                *a = serde_json::from_value(v).unwrap();
            }
            _ => {
                let current = session(a).current.clone();
                session_mut(a).example = current;
            }
        }
        assert!(work::context(a, s).is_err());
        assert!(!progress::current_solved(a, s));
        let bad_before = value(&bad);
        assert!(progress::save_with_catalog(&path, &bad, &all).is_err());
        assert_eq!(files(dir.path()), original);
        assert_eq!(value(&bad), bad_before);
    }
    let missing = all
        .iter()
        .filter(|b| b.id != s.id)
        .cloned()
        .collect::<Vec<_>>();
    assert!(progress::save_with_catalog(&path, &p, &missing).is_err());
    assert_eq!(files(dir.path()), original);
    let mut wrong = all.clone();
    let held = wrong.iter_mut().find(|b| b.id == s.id).unwrap();
    held.goal = all[0].goal.clone();
    assert!(progress::save_with_catalog(&path, &p, &wrong).is_err());
    assert_eq!(files(dir.path()), original);
    // A self-consistent forged profile/Goal pair is not authority over the held catalog.
    let mut fake_source = s.clone();
    let mut dv = value(&fake_source);
    dv["goal"]["definition"]["initial"]["environment"]["deliveries"][0]["available"] = false.into();
    dv["initial"]["simulation"]["current"]["environment"]["deliveries"][0]["available"] =
        false.into();
    fake_source = serde_json::from_value(dv).unwrap();
    work::source(&fake_source).unwrap();
    let forged = fresh(&fake_source);
    assert!(work::context(&forged, s).is_err());
    assert_eq!(value(&p), caller);
}

#[test]
fn invalid_packs_unreachable_recipes_and_unsupported_terminal_text_are_refused_without_partial_state()
 {
    for s in labs() {
        let mut a = fresh(&s);
        let before = value(&a);
        for text in [
            "logging trap warnings",
            "logger hello",
            "$(touch /private)",
            "{\"action\":\"apply_collector\",\"collector\":\"10.0.0.99\",\"policy\":{\"selectors\":{}}}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, text).is_err());
            assert_eq!(value(&a), before);
        }
        assert!(engine::execute(&mut a.state, &s.solution[0]).is_err());
        assert_eq!(value(&a), before);
    }
    let raw: Value = serde_json::from_str(DATA).unwrap();
    for mode in 0..5 {
        let mut v = raw.clone();
        let s = &mut v["scenarios"][0];
        match mode {
            0 => s["goal"]["definition"]["version"] = 2.into(),
            1 => {
                s["goal"]["definition"]["initial"]["devices"]["10.214.0.10"]["export_facility"] =
                    24.into()
            }
            2 => {
                s["goal"]["definition"]["lesson"]["fixture"] =
                    s["goal"]["definition"]["initial"].clone()
            }
            3 => s["solution"] = json!(["{\"action\":\"clear_local\",\"device\":\"10.214.0.10\"}"]),
            _ => s["goal"]["definition"]["targets"][0]["unknown"] = true.into(),
        }
        assert!(admit(&v.to_string()).is_err());
    }
}

#[test]
fn authored_payloads_keep_existing_caps_and_all_fixture_predictions_explicitly_unexecuted() {
    assert!(DATA.len() < pack::MAX_PACK_BYTES);
    assert_eq!(pack::MAX_PACK_BYTES, 2_000_000);
    let r = review();
    assert!(r["actual"].is_null());
    for (i, s) in labs().iter().enumerate() {
        let a = fresh(s);
        assert!(serde_json::to_vec(definition(s)).unwrap().len() < work::MAX_DEFINITION);
        assert!(serde_json::to_vec(session(&a)).unwrap().len() < work::MAX_SESSION);
        assert!(serde_json::to_vec(&(s, &a)).unwrap().len() < work::MAX_CONTEXT);
        assert!(
            s.solution.len() <= 50
                && s.solution
                    .iter()
                    .all(|x| x.len() <= work::MAX_ACTION && x.len() <= 4000)
        );
        for v in r["labs"][i]["wrong_partial"].as_array().unwrap() {
            assert!(v["actual"].is_null());
        }
        assert!(r["labs"][i]["practice"]["actual"].is_null());
        assert!(r["labs"][i]["damage"]["actual"].is_null());
    }
}

fn predecessor197() -> Vec<Scenario> {
    let current201 = pack::bundled().unwrap();
    let current199 =
        disk::literal_registration::snapshot_registration::before_scenarios(&current201);
    assert_eq!((current201.len(), current199.len()), (201, 199));
    assert_eq!(current199.len(), 199);
    disk::literal_registration::before_scenarios(current199).to_vec()
}
