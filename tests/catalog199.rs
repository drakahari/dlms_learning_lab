//! Exact registration of two reviewed literal projects. No full-course claim.
use dlms_learning_lab::{
    course, developer_digest as digest,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching, shell, workspace,
};
use serde_json::{Value, json};
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::{self as disk,literal_registration as registration};
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn full(p: &Progress) -> Value {
    json!({"portable":p,"receipts":p.migration_receipts})
}
#[test]
fn full197_sources_and477_cases_remain_before_two_normal_projects() {
    let all = pack::bundled().unwrap();
    let old = registration::original197();
    assert_eq!((all.len(), old.len()), (199, 197));
    assert_eq!(value(registration::before_scenarios(&all)), value(&old));
    assert_eq!(value(&all[..197]), value(&old));
    assert_eq!(
        all[197..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        registration::IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 44, 36, 62]
    );
    let count = |xs: &[Scenario]| {
        xs.iter()
            .map(|s| {
                if let Goal::Script { cases } = &s.goal {
                    cases.len()
                } else {
                    0
                }
            })
            .sum::<usize>()
    };
    assert_eq!((count(&old), count(&all)), (477, 501));
    assert_eq!(all.iter().filter(|s| s.teaching.is_some()).count(), 62);
    for s in &all[197..] {
        assert_eq!(s.unit, "shell-literal-normalization");
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert!(matches!(s.initial, State::Script { .. }));
    }
}
#[test]
fn exact53_to52_raw_and_typed_inverse_keeps_all_predecessors_independent() {
    let all = pack::bundled().unwrap();
    let old = registration::before_scenarios(&all);
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert_eq!(
        &before,
        include_bytes!("fixtures/literal-units-before197.json")
    );
    let current = course::bundled(&all).unwrap();
    let previous = course::load(std::str::from_utf8(&before).unwrap(), old).unwrap();
    assert_eq!((current.len(), previous.len()), (53, 52));
    assert_eq!(
        value(registration::before_typed_units(&current)),
        value(&previous)
    );
    assert_eq!(value(&current[..52]), value(&previous));
    assert_eq!(current[52].labs, registration::IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|u| u.track == t).count()),
        [15, 11, 9, 18]
    );
    assert_eq!(
        disk::before_units(raw).unwrap(),
        disk::before_units(&before).unwrap()
    );
    let round = course::load(&json!({"version":1,"units":current}).to_string(), &all).unwrap();
    assert_eq!(value(&round), value(&current));
    assert_eq!(
        digest::hash(&before),
        "3e61db90314703bcd4fcde174c98c1983dfacc809c09ebbdb3093ecbe9c0fb17"
    );
}
#[test]
fn source_inverse_refuses_missing_extra_changed_old_new_and_case_teaching_mutations() {
    let all = pack::bundled().unwrap();
    for mode in 0..9 {
        let mut bad = all.clone();
        match mode {
            0 => {
                bad.pop();
            }
            1 => bad.push(all[197].clone()),
            2 => bad[0].objective.push_str(" changed"),
            3 => bad[197].version += 1,
            4 => bad[198].objective.push_str(" changed"),
            5 => {
                let Goal::Script { cases } = &mut bad[197].goal else {
                    panic!()
                };
                cases[0].stdout.push('!');
            }
            6 => bad[197]
                .teaching
                .as_mut()
                .unwrap()
                .example
                .push_str("\n#changed"),
            7 => bad[197].solution[0].push_str("\n#changed"),
            _ => bad[197].teaching.as_mut().unwrap().concepts[0]
                .text
                .push('!'),
        }
        assert!(std::panic::catch_unwind(|| registration::before_scenarios(&bad)).is_err());
    }
}
#[test]
fn raw_and_full_typed_unit_inverse_refuses_changed_append_and_old_objects() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let whole: Value = serde_json::from_slice(raw).unwrap();
    let mut padded = raw.to_vec();
    padded.push(b' ');
    assert!(registration::before_units(&padded).is_err());
    for mode in 0..8 {
        let mut bad = whole.clone();
        match mode {
            0 => bad["units"][52]["title"] = json!("changed"),
            1 => bad["units"][52]["prerequisites"] = json!([]),
            2 => bad["units"][52]["labs"][0] = json!("shell-greetings"),
            3 => bad["units"][0]["summary"] = json!("changed"),
            4 => {
                bad["units"].as_array_mut().unwrap().pop();
            }
            5 => {
                let x = bad["units"][52].clone();
                bad["units"].as_array_mut().unwrap().push(x);
            }
            6 => bad["units"][52]["track"] = json!("CCNA"),
            _ => bad["units"][52]["summary"] = json!("changed"),
        }
        assert!(registration::before_units(&serde_json::to_vec(&bad).unwrap()).is_err());
        let units: Vec<course::Unit> = serde_json::from_value(bad["units"].clone()).unwrap();
        assert!(std::panic::catch_unwind(|| registration::before_typed_units(&units)).is_err());
        assert!(
            std::panic::catch_unwind(|| registration::before_values(
                bad["units"].as_array().unwrap()
            ))
            .is_err()
        );
    }
}
#[test]
fn current_count_map_validates_every_key_and_new_script_without_awarding_credit() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let before = full(&p);
    assert_eq!(registration::before_attempts_len(&p.attempts), 197);
    assert_eq!(disk::before_attempts_len(&p.attempts), 193);
    assert_eq!(p.attempts.len(), 199);
    assert_eq!(full(&p), before);
    for mode in 0..4 {
        let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
        match mode {
            0 => {
                bad.attempts.remove(registration::IDS[0]);
            }
            1 => {
                bad.attempts
                    .get_mut(registration::IDS[0])
                    .unwrap()
                    .scenario_version += 1
            }
            2 => bad.attempts.get_mut(registration::IDS[1]).unwrap().state = all[0].initial.clone(),
            _ => bad.attempts.get_mut(&all[0].id).unwrap().scenario_version += 1,
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_attempts_len(&bad.attempts)
            }))
            .is_err()
        );
    }
}
#[test]
fn normal_projects_run_reference_and_practice_with_separate_resumable_drafts() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 43,
        ..Default::default()
    };
    for s in &all[197..] {
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        for c in cases {
            let out = shell::run(&s.solution[0], c);
            assert!(out.error.is_none(), "{} {:?}", s.id, out.error);
            assert!(c.accepts(&out), "{} {}", s.id, c.name);
        }
        let l = s.teaching.as_ref().unwrap();
        let out = shell::run(&l.example, &l.fixture);
        assert!(out.error.is_none() && l.fixture.accepts(&out));
        let mut a = Attempt::new(s, 7);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, l, true);
        a.challenge = true;
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        *source = "# assignment draft\n".into();
        a.workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap()
            .example_source = "# separate practice draft\n".into();
        assert!(!progress::current_solved(&a, s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, &all).unwrap();
    assert_eq!(full(&p), before);
    let read = progress::load(&file, &all).unwrap();
    assert_eq!(value(&read), value(&p));
    for id in registration::IDS {
        let a = &read.attempts[id];
        let State::Script { source, .. } = &a.state else {
            panic!()
        };
        assert_eq!(source, "# assignment draft\n");
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_source,
            "# separate practice draft\n"
        );
        assert!(a.challenge);
    }
}
