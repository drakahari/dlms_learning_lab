//! Exact normal registration of three reviewed ERE projects. Source/test evidence,
//! never a claim of full curriculum, native, platform or beginner acceptance.
use dlms_learning_lab::{
    course, developer_digest as digest,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching, workspace,
};
use serde_json::{Value, json};
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use ntp_registration::ere_registration as registration;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration;
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    a
}
#[test]
fn complete190_sources_and57_case_objects_remain_before_three_real_normal_projects() {
    let all = pack::bundled().unwrap();
    let old = registration::original190::load();
    assert_eq!((all.len(), old.len()), (193, 190));
    assert_eq!(value(registration::before_scenarios(&all)), value(&old));
    assert_eq!(value(&all[..190]), value(&old));
    let authored: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/shell-ere-evidence-v1.json")).unwrap();
    assert_eq!(value(&all[190..]), value(&authored.scenarios));
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 40, 36, 60]
    );
    let scripts = all
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 60);
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 60);
    assert_eq!(
        scripts
            .iter()
            .map(|s| if let Goal::Script { cases } = &s.goal {
                cases.len()
            } else {
                0
            })
            .sum::<usize>(),
        477
    );
    assert_eq!(
        all[190..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        registration::IDS
    );
    for s in &all[190..] {
        assert_eq!(s.unit, "shell-ere-evidence");
        assert_eq!(s.track, "Bash/Shell");
        assert!(s.teaching.is_some() && matches!(s.goal, Goal::Script { .. }));
        assert!(!progress::current_solved(&fresh(s), s));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
    }
}
#[test]
fn closed51_to50_raw_and_full_typed_inverse_keeps_all_predecessors_and_rejects_mutations() {
    let all = pack::bundled().unwrap();
    let old = registration::original190::load();
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert_eq!(before, include_bytes!("fixtures/ere-units-before190.json"));
    let recipe: Value =
        serde_json::from_str(include_str!("fixtures/catalog193-registration.json")).unwrap();
    assert_eq!(digest::hash(raw), recipe["after_units51_sha256"]);
    assert_eq!(digest::hash(&before), recipe["before_units50_sha256"]);
    let units = course::bundled(&all).unwrap();
    let previous = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    assert_eq!((units.len(), previous.len()), (51, 50));
    assert_eq!(
        value(registration::before_typed_units(&units)),
        value(&previous)
    );
    assert_eq!(value(&units[..50]), value(&previous));
    assert_eq!(value(&units[50]), recipe["ere_unit"]);
    assert_eq!(units[50].labs, registration::IDS);
    assert_eq!(
        units[50].prerequisites,
        [
            "shell-text",
            "shell-numeric-validation",
            "shell-loop-recovery"
        ]
    );
    let decoded = course::load(&json!({"version":1,"units":units}).to_string(), &all).unwrap();
    assert_eq!(value(&decoded), value(&units));
    assert_eq!(
        ntp_registration::before_units(raw).unwrap(),
        ntp_registration::before_units(&before).unwrap()
    );
    assert_eq!(
        value(ntp_registration::before_typed_units(&units)),
        value(ntp_registration::before_typed_units(&previous))
    );
    assert!(registration::before_units(&before).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(registration::before_units(&extra).is_err());
    for mode in 0..7 {
        let mut bad: Value = serde_json::from_slice(raw).unwrap();
        match mode {
            0 => bad["units"][50]["title"] = json!("changed"),
            1 => bad["units"][50]["summary"] = json!("changed"),
            2 => bad["units"][50]["prerequisites"] = json!([]),
            3 => bad["units"][50]["labs"][0] = json!("shell-greetings"),
            4 => bad["units"][0]["title"] = json!("changed old source"),
            5 => {
                bad["units"].as_array_mut().unwrap().pop().unwrap();
            }
            _ => {
                let duplicate = bad["units"][50].clone();
                bad["units"].as_array_mut().unwrap().push(duplicate);
            }
        }
        assert!(registration::before_units(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert!(
            std::panic::catch_unwind(|| registration::before_values(
                bad["units"].as_array().unwrap()
            ))
            .is_err()
        );
        let typed: Vec<course::Unit> = serde_json::from_value(bad["units"].clone()).unwrap();
        assert!(std::panic::catch_unwind(|| registration::before_typed_units(&typed)).is_err());
    }
}
#[test]
fn exact_source_inverse_refuses_partial_extra_old_version_and_changed_teaching_or_case_objects() {
    let all = pack::bundled().unwrap();
    for mode in 0..8 {
        let mut bad = all.clone();
        match mode {
            0 => {
                bad.pop().unwrap();
            }
            1 => bad.push(all[190].clone()),
            2 => bad[190].objective.push_str(" changed"),
            3 => {
                let Goal::Script { cases } = &mut bad[190].goal else {
                    panic!()
                };
                cases[0].stdout.push_str(" changed");
            }
            4 => bad[190].teaching.as_mut().unwrap().concepts[0]
                .text
                .push_str(" changed"),
            5 => bad[190]
                .teaching
                .as_mut()
                .unwrap()
                .fixture
                .args
                .push("changed".into()),
            6 => bad[0].version += 1,
            _ => {
                bad[190].terminal_teaching = all.iter().find_map(|s| s.terminal_teaching.clone());
                assert!(bad[190].terminal_teaching.is_some());
            }
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_scenarios(&bad)
            }))
            .is_err()
        );
    }
}
#[test]
fn count_only_predecessor_validates_every_current_key_new_state_and_version_without_credit() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    assert_eq!(registration::before_attempts_len(&p.attempts), 190);
    assert_eq!(ntp_registration::before_attempts_len(&p.attempts), 186);
    for mode in 0..4 {
        let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
        match mode {
            0 => {
                let a = bad.attempts.remove(&all[190].id).unwrap();
                bad.attempts.insert("foreign-id".into(), a);
            }
            1 => bad.attempts.get_mut(&all[190].id).unwrap().scenario_version += 1,
            2 => bad.attempts.get_mut(&all[190].id).unwrap().state = all[0].initial.clone(),
            _ => bad.attempts.get_mut(&all[0].id).unwrap().scenario_version += 1,
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_attempts_len(&bad.attempts)
            }))
            .is_err()
        );
    }
    assert!(
        p.attempts
            .values()
            .all(|a| !matches!(a.state, State::Script { run: Some(_), .. }))
    );
}
#[test]
fn retained_profile_never_invents_new_project_answers_favorites_or_completion() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let old: Progress = serde_json::from_slice(raw).unwrap();
    let current = progress::decode(raw, &all).unwrap();
    assert_eq!(value(&current.attempts), value(&old.attempts));
    assert_eq!(current.favorites, old.favorites);
    for id in registration::IDS {
        assert!(!current.attempts.contains_key(id) && !current.favorites.contains(id));
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(!progress::current_solved(&fresh(s), s));
    }
}
#[test]
fn all_three_normal_introduction_example_and_assignment_drafts_roundtrip_separately() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 41,
        ..Default::default()
    };
    for s in &all[190..] {
        let mut a = fresh(s);
        let before = value(&a.state);
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning
            .example_source
            .push_str("\n# Separate unfinished example\n");
        learning.view = script_teaching::View::Example;
        assert_eq!(value(&a.state), before);
        assert!(!progress::current_solved(&a, s));
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\n# Separate unfinished assignment\n");
        a.challenge = true;
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let before = value(&p);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, &all).unwrap();
    assert_eq!(value(&p), before);
    let resumed = progress::load(&file, &all).unwrap();
    assert_eq!(value(&resumed), before);
    for s in &all[190..] {
        let a = &resumed.attempts[&s.id];
        assert!(a.challenge);
        assert!(!progress::current_solved(a, s));
    }
    let portable = dir.path().join("portable.json");
    dlms_learning_lab::backup::export(&portable, &resumed, &all).unwrap();
    let decoded = dlms_learning_lab::backup::inspect(
        &portable,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(value(&decoded.progress), value(&resumed));
}
