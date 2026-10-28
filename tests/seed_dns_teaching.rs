use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    seed_dns_teaching::{self as dns, Action, Learning},
    shell, workspace,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn scenario() -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-dns")
        .unwrap()
}
fn perform(l: &mut Learning, lesson: &dns::Lesson, actions: impl IntoIterator<Item = Action>) {
    for a in actions {
        match a {
            Action::Save { draft } => {
                l.draft = draft;
                l.save(lesson).unwrap();
            }
            Action::Command { command } => {
                l.command = command.command;
                l.submit(lesson).unwrap();
            }
        }
    }
}
fn reference(l: &mut Learning, lesson: &dns::Lesson) {
    perform(
        l,
        lesson,
        lesson.example_actions.iter().map(|a| a.action.clone()),
    );
}
fn assert_feedback_keeps_engine_truth(l: &Learning, lesson: &dns::Lesson) {
    let expected = engine::checks(&l.current.state().unwrap(), &lesson.goal().unwrap());
    let actual = l.checks(lesson);
    assert_eq!(
        actual.iter().map(|c| c.0).collect::<Vec<_>>(),
        expected.iter().map(|c| c.0).collect::<Vec<_>>()
    );
    for ((pass, message), (_, original)) in actual.iter().zip(expected) {
        assert!(!message.is_empty() && message.len() <= 512);
        if *pass {
            assert_eq!(message, &original);
        } else {
            assert_ne!(message, &original);
        }
    }
}
#[test]
fn dns_worked_example_and_saved_versus_serving_experiment_are_real_and_independent() {
    let s = scenario();
    let lesson = s.seed_dns_teaching.as_ref().unwrap();
    assert!(lesson.valid_for(&s.initial, &s.goal));
    let mut l = Learning::new(lesson).unwrap();
    assert!(l.checks(lesson).iter().any(|c| !c.0));
    let initial = l.checks(lesson);
    assert!(initial[0].1.contains("Saved address 198.51.100.4 differs"));
    assert!(
        initial[1]
            .1
            .contains("Serving address 198.51.100.4 differs")
    );
    assert_feedback_keeps_engine_truth(&l, lesson);
    reference(&mut l, lesson);
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!((l.current.serial, l.current.loaded_serial), (42, 42));
    assert_feedback_keeps_engine_truth(&l, lesson);
    let mut absent = l.clone();
    absent.current.served = None;
    assert!(absent.checks(lesson)[1].1.contains("No answer is loaded"));
    assert_feedback_keeps_engine_truth(&absent, lesson);
    let mut wrong_answer = l.clone();
    wrong_answer.current.served = Some("198.51.100.46".parse().unwrap());
    assert!(
        wrong_answer.checks(lesson)[1]
            .1
            .contains("Serving address 198.51.100.46 differs")
    );
    assert_feedback_keeps_engine_truth(&wrong_answer, lesson);
    perform(&mut l, lesson, lesson.try_actions.clone());
    assert_eq!(l.current.saved.to_string(), "198.51.100.45");
    assert_eq!(l.current.served.unwrap().to_string(), "198.51.100.44");
    assert_eq!((l.current.serial, l.current.loaded_serial), (43, 42));
    let checks = l.checks(lesson);
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|c| !c.0));
    assert!(checks[0].1.contains("Saved address 198.51.100.45 differs"));
    assert!(
        checks[1]
            .1
            .contains("Serving serial 42 differs from saved serial 43")
    );
    assert_feedback_keeps_engine_truth(&l, lesson);
    assert!(l.transcript.last().unwrap().contains("198.51.100.44"));
    let before_validation = l.current.clone();
    l.command = "named-checkzone lab.test".into();
    l.submit(lesson).unwrap();
    assert_eq!(
        l.current, before_validation,
        "Validation does not activate the saved record"
    );
    assert_feedback_keeps_engine_truth(&l, lesson);
    perform(&mut l, lesson, lesson.repair_actions.clone());
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!(scenario().initial, s.initial);
}
#[test]
fn dns_insert_only_and_atomic_invalid_save_preserve_raw_drafts_and_allow_current_alternates() {
    let s = scenario();
    let lesson = s.seed_dns_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let index = lesson
        .example_actions
        .iter()
        .position(|a| matches!(a.action, Action::Save { .. }))
        .unwrap();
    l.selected_action = index;
    let before = l.current.clone();
    l.insert(lesson).unwrap();
    assert_eq!(l.current, before);
    assert!(l.transcript.is_empty());
    assert!(!l.draft.matches(&l.current));
    l.draft.serial = "not a serial".into();
    let raw = l.draft.clone();
    assert!(l.save(lesson).is_err());
    assert_eq!(l.current, before);
    assert_eq!(l.draft, raw);
    l.draft.serial = "0".into();
    l.save(lesson).unwrap();
    assert_eq!(l.current.serial, 0);
    let before = l.current.clone();
    l.command = "systemctl reload named".into();
    assert!(l.submit(lesson).is_err());
    assert_eq!(l.current, before);
    l.draft.serial = "99".into();
    l.save(lesson).unwrap();
    l.command = "systemctl reload named".into();
    l.submit(lesson).unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    l.checked = true;
    l.draft.serial = "unsaved".into();
    assert!(l.checks(lesson).iter().all(|c| !c.0));
    assert!(
        l.checks(lesson)[0]
            .1
            .starts_with("Unsaved example settings")
    );
    l.draft = dns::Draft::new(&l.current);
    assert!(l.checks(lesson).iter().all(|c| c.0));
    // Existing goals grade current values, not proof of a new reload event.
    l.draft.address = "198.51.100.45".into();
    l.draft.serial = "100".into();
    l.save(lesson).unwrap();
    l.draft.address = "198.51.100.44".into();
    l.draft.serial = "99".into();
    l.save(lesson).unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
}
#[test]
fn dns_unknown_key_case_limit_and_terminal_draft_refresh_preserve_existing_semantics() {
    let s = scenario();
    let lesson = s.seed_dns_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let before = l.current.clone();
    l.command = "dns set foreign.lab.test 203.0.113.5".into();
    assert!(l.submit(lesson).is_err());
    assert_eq!(l.current, before);
    l.command = "dig Mirror.lab.test".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.transcript.last().unwrap(), "NXDOMAIN");
    assert_eq!(l.current, before);
    l.command = "dns serial 42".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.draft.serial, "42");
    l.draft.serial = "unfinished".into();
    l.command = "dns serial 43".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.draft.serial, "unfinished");
    assert_eq!(l.current.serial, 43);
    assert!(
        lesson
            .concepts
            .iter()
            .any(|c| c.text.contains("ignore ASCII letter case"))
    );
    let mut broken = lesson.clone();
    for a in &mut broken.example_actions {
        if let Action::Command { command } = &mut a.action
            && command.command == "named-checkzone lab.test"
        {
            assert!(command.simulator_only);
            command.simulator_only = false;
        }
    }
    assert!(broken.validate().is_err());
    l.draft = dns::Draft::new(&l.current);
    l.draft.serial = u32::MAX.to_string();
    l.save(lesson).unwrap();
    l.command = "systemctl reload named".into();
    l.submit(lesson).unwrap();
    l.draft.address = "198.51.100.77".into();
    l.save(lesson).unwrap();
    let before = l.current.clone();
    l.command = "systemctl reload named".into();
    assert!(l.submit(lesson).is_err());
    assert_eq!(l.current, before); // no wrap arithmetic
}
#[test]
fn dns_learning_decode_bounds_and_fixture_pins_reject_foreign_context_without_rejecting_coincidences()
 {
    let s = scenario();
    let lesson = s.seed_dns_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    l.current.saved = "192.0.2.80".parse().unwrap();
    assert!(l.valid_for(lesson));
    let mut foreign = l.clone();
    foreign.fixture.serial += 1;
    assert!(foreign.valid());
    assert!(!foreign.valid_for(lesson));
    assert_eq!(
        foreign.checks(lesson),
        [(false, "Invalid or foreign DNS example".into())]
    );
    assert!(foreign.submit(lesson).is_err());
    let mut oversized = lesson.fixture.clone();
    oversized.name = "x".repeat(254);
    assert!(!oversized.valid());
    assert!(oversized.state().is_err());
    let mut json = serde_json::to_value(&l).unwrap();
    json["draft"]["serial"] = Value::String("x".repeat(257));
    assert!(serde_json::from_value::<Learning>(json).is_err());
    let mut json = serde_json::to_value(&l).unwrap();
    json["transcript"] = Value::Array(
        (0..32)
            .map(|_| Value::String(String::new()))
            .chain(std::iter::once(
                serde_json::json!({"must_not_be_typed_as_string":true}),
            ))
            .collect(),
    );
    assert!(
        serde_json::from_value::<Learning>(json)
            .unwrap_err()
            .to_string()
            .contains("exceeds 32")
    );
    let mut metadata = serde_json::to_value(lesson).unwrap();
    metadata["concepts"][0]["text"] = serde_json::json!("x".repeat(1201));
    assert!(serde_json::from_value::<dns::Lesson>(metadata).is_err());
    let mut metadata = serde_json::to_value(lesson).unwrap();
    metadata["example_actions"] = Value::Array(
        std::iter::repeat_n(metadata["example_actions"][0].clone(), 12)
            .chain(std::iter::once(serde_json::json!({"excess_body":true})))
            .collect(),
    );
    assert!(
        serde_json::from_value::<dns::Lesson>(metadata)
            .unwrap_err()
            .to_string()
            .contains("exceeds 12")
    );
    // Maximum live transcript bytes plus worst legal JSON escaping, compact/pretty.
    l.transcript = vec!["\u{0001}".repeat(8192); 16];
    l.command = "\\".repeat(4096);
    l.draft.serial = "\\".repeat(256);
    assert!(l.valid_for(lesson));
    for bytes in [
        serde_json::to_vec(&l).unwrap(),
        serde_json::to_vec_pretty(&l).unwrap(),
    ] {
        assert!(bytes.len() < 1_048_576);
        let decoded: Learning = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, l);
    }
}
#[test]
fn dns_old_first_open_preserves_assignment_and_both_raw_editor_fields() {
    let s = scenario();
    let lesson = s.seed_dns_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    let w = a.workspace.as_mut().unwrap();
    w.command = "dns set 'unfinished".into();
    w.dns_address = Some("invalid address".into());
    w.dns_serial = Some("unfinished serial".into());
    let before = serde_json::to_value(&a).unwrap();
    dns::prepare(&mut a, lesson, false).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .seed_dns_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let mut after = serde_json::to_value(&a).unwrap();
    after["workspace"]
        .as_object_mut()
        .unwrap()
        .remove("seed_dns_learning");
    assert_eq!(after, before);
    assert!(!progress::current_solved(&a, &s));
    let mut fresh = Attempt::new(&s, 8);
    workspace::prepare(&mut fresh);
    dns::prepare(&mut fresh, lesson, true).unwrap();
    assert_eq!(
        fresh.workspace.unwrap().seed_dns_learning.unwrap().view,
        View::Introduction
    );
}
#[test]
fn dns_teaching_pins_only_authored_keys_and_keeps_original_nine_assignment_payloads() {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 154);
    let baseline: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/seed-teaching-original-nine.json")).unwrap();
    assert_eq!(baseline.len(), 9);
    for old in baseline {
        let current = all
            .iter()
            .find(|s| s.id == old["id"].as_str().unwrap())
            .unwrap();
        let current = serde_json::to_value(current).unwrap();
        for key in ["id", "version", "initial", "solution"] {
            assert_eq!(current[key], old[key]);
        }
        // Historical JSON may omit default Goal fields (e.g. Access.exclusive).
        // Compare its decoded meaning; preserve the frozen input bytes.
        let original_goal: Goal = serde_json::from_value(old["goal"].clone()).unwrap();
        assert_eq!(
            current["goal"],
            serde_json::to_value(original_goal).unwrap()
        );
    }
    let s = scenario();
    let mut current = s.initial.clone();
    engine::execute(&mut current, "dns serial 0").unwrap();
    assert!(dns::assignment_context(&current, &s.initial));
    let State::Dns { records, .. } = &mut current else {
        panic!()
    };
    let address = records.remove("portal.lab.test").unwrap();
    records.insert("foreign.lab.test".into(), address);
    assert!(!dns::assignment_context(&current, &s.initial));
    let mut p: Value = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v7-synthetic-progress-original.json"
    ))
    .unwrap();
    p["attempts"]["lpic2-dns"]["state"] = serde_json::to_value(current).unwrap();
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
fn history(p: &Progress) -> BTreeMap<String, Vec<Value>> {
    let mut found = BTreeMap::new();
    for (id, a) in &p.attempts {
        let current = if let State::Script { run, .. } = &a.state {
            run.as_ref()
        } else {
            None
        };
        let w = a.workspace.as_ref();
        let records: Vec<_> = current
            .into_iter()
            .chain(w.and_then(|w| w.script_history.as_ref()))
            .chain(w.into_iter().flat_map(|w| w.script_history_more.iter()))
            .filter(|r| [6, 7].contains(&r.engine_revision))
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        if !records.is_empty() {
            found.insert(id.clone(), records);
        }
    }
    found
}
#[test]
fn both_seed_practices_all_contexts_actual_records_and_raw_drafts_survive_save_restore_and_selected_reset()
 {
    const {
        assert!(shell::EVALUATOR_REVISION >= 8);
    }
    let all = pack::bundled().unwrap();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v7-synthetic-progress-original.json"),
        &all,
    )
    .unwrap();
    let genuine = history(&p);
    assert!(
        genuine
            .values()
            .flatten()
            .any(|r| r["engine_revision"] == 6)
    );
    assert!(
        genuine
            .values()
            .flatten()
            .any(|r| r["engine_revision"] == 7)
    );
    for s in &all {
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
        if let Goal::Script { cases } = &s.goal {
            workspace::prepare(a);
            let source = s.solution[0].clone();
            let results: Vec<_> = cases.iter().map(|c| shell::run(&source, c)).collect();
            assert!(cases.iter().zip(&results).all(|(c, r)| c.accepts(r)));
            a.state = State::Script {
                source: source.clone(),
                run: Some(shell::Record {
                    engine_revision: shell::EVALUATOR_REVISION,
                    source,
                    results,
                }),
            };
        }
    }
    let before_others: BTreeMap<_, _> = p
        .attempts
        .iter()
        .filter(|(id, _)| !matches!(id.as_str(), "lpic2-dns" | "lpic2-storage"))
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect();
    for s in all
        .iter()
        .filter(|s| matches!(s.id.as_str(), "lpic2-dns" | "lpic2-storage"))
    {
        let a = p.attempts.get_mut(&s.id).unwrap();
        workspace::prepare(a);
        a.workspace.as_mut().unwrap().command = "unfinished assignment input".into();
        if let Some(lesson) = &s.seed_dns_teaching {
            dns::prepare(a, lesson, false).unwrap();
            let w = a.workspace.as_mut().unwrap();
            w.dns_address = Some("invalid address".into());
            w.dns_serial = Some("unfinished serial".into());
            let l = w.seed_dns_learning.as_mut().unwrap();
            reference(l, lesson);
            l.draft.serial = "unfinished independent example".into();
            l.command = "dig unfinished".into();
        }
        if let Some(lesson) = &s.seed_mount_teaching {
            dlms_learning_lab::seed_mount_teaching::prepare(a, lesson, false).unwrap();
            let w = a.workspace.as_mut().unwrap();
            w.mount_device = Some("unfinished assignment device".into());
            let l = w.seed_mount_learning.as_mut().unwrap();
            l.draft.device = "unfinished example device".into();
            l.command = "mount unfinished".into();
        }
        p.favorites.insert(s.id.clone());
    }
    let desired = serde_json::to_value(&p).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("profile.json");
    progress::save(&target, &p).unwrap();
    let bytes = std::fs::read(&target).unwrap();
    let loaded = progress::load(&target, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), desired);
    assert_eq!(std::fs::read(&target).unwrap(), bytes);
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let checked =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    // Inspection deliberately refreshes old ungraded example outputs only.
    let mut expected = desired.clone();
    for (id, a) in &loaded.attempts {
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.script_learning.as_ref())
            && l.example_result.is_some()
        {
            let s = all.iter().find(|s| s.id == *id).unwrap();
            expected["attempts"][id]["workspace"]["script_learning"]["example_result"] =
                serde_json::to_value(shell::run(
                    &l.example_source,
                    &s.teaching.as_ref().unwrap().fixture,
                ))
                .unwrap();
            expected["attempts"][id]["workspace"]["script_learning"]["example_revision"] =
                serde_json::json!(shell::EVALUATOR_REVISION);
        }
    }
    assert_eq!(serde_json::to_value(&checked.progress).unwrap(), expected);
    let copy = backup::restore(&target, &checked, &all)
        .unwrap()
        .safety_copy
        .unwrap();
    assert_eq!(std::fs::read(copy).unwrap(), bytes);
    let mut restored = progress::load(&target, &all).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), expected);
    for (id, records) in genuine {
        let kept = history(&restored);
        for record in records {
            assert!(kept[&id].contains(&record));
        }
    }
    for id in before_others.into_keys() {
        assert_eq!(
            serde_json::to_value(&restored.attempts[&id]).unwrap(),
            expected["attempts"][&id]
        );
    }
    let dns_s = all.iter().find(|s| s.id == "lpic2-dns").unwrap();
    let mount_before = serde_json::to_value(&restored.attempts["lpic2-storage"]).unwrap();
    let mut reset = Attempt::new(dns_s, 20);
    workspace::prepare(&mut reset);
    dns::prepare(&mut reset, dns_s.seed_dns_teaching.as_ref().unwrap(), true).unwrap();
    restored.attempts.insert(dns_s.id.clone(), reset);
    assert_eq!(
        serde_json::to_value(&restored.attempts["lpic2-storage"]).unwrap(),
        mount_before
    );
    assert_eq!(restored.favorites, p.favorites);
}
