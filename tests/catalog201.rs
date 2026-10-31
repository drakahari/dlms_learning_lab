//! Exact normal snapshot registration, preserving all199 predecessors. No acceptance claim.
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
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::{self as disk,literal_registration as literal};
use literal::snapshot_registration as registration;
use one_off_boundary::loop_registration::lvm_registration as lvm;
use lvm::ntp_registration as ntp;
use ntp::ere_registration as ere;
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn full(p: &Progress) -> Value {
    json!({"portable":p,"receipts":p.migration_receipts})
}
#[test]
fn full199_sources_and501_cases_remain_before_two_normal_snapshot_projects() {
    let all = pack::bundled().unwrap();
    let old = registration::original199();
    assert_eq!((all.len(), old.len()), (201, 199));
    assert_eq!(value(registration::before_scenarios(&all)), value(&old));
    assert_eq!(value(&all[..199]), value(&old));
    assert_eq!(
        all[199..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        registration::IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 44, 36, 64]
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
    assert_eq!((count(&old), count(&all)), (501, 525));
    assert_eq!(all.iter().filter(|s| s.teaching.is_some()).count(), 64);
    for s in &all[199..] {
        assert_eq!(s.unit, "shell-snapshot-rollout");
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert!(matches!(s.initial, State::Script { .. }));
    }
}
#[test]
fn exact54_to53_raw_and_typed_inverse_keeps_all_predecessors_independent() {
    let all = pack::bundled().unwrap();
    let old = registration::before_scenarios(&all);
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert_eq!(
        &before,
        include_bytes!("fixtures/snapshot-units-before199.json")
    );
    let current = course::bundled(&all).unwrap();
    let previous = course::load(std::str::from_utf8(&before).unwrap(), old).unwrap();
    assert_eq!((current.len(), previous.len()), (54, 53));
    assert_eq!(
        value(registration::before_typed_units(&current)),
        value(&previous)
    );
    assert_eq!(value(&current[..53]), value(&previous));
    assert_eq!(current[53].labs, registration::IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|u| u.track == t).count()),
        [15, 11, 9, 19]
    );
    assert_eq!(
        disk::before_units(raw).unwrap(),
        disk::before_units(&before).unwrap()
    );
    let round = course::load(&json!({"version":1,"units":current}).to_string(), &all).unwrap();
    assert_eq!(value(&round), value(&current));
    assert_eq!(
        digest::hash(&before),
        "ccbed0ebbaf7228ed147c9af236f4fa6c7d3e1f32ee05fa1fe46c93d6f068fdf"
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
            1 => bad.push(all[199].clone()),
            2 => bad[0].objective.push_str(" changed"),
            3 => bad[199].version += 1,
            4 => bad[200].objective.push_str(" changed"),
            5 => {
                let Goal::Script { cases } = &mut bad[199].goal else {
                    panic!()
                };
                cases[0].stdout.push('!');
            }
            6 => bad[199]
                .teaching
                .as_mut()
                .unwrap()
                .example
                .push_str("\n#changed"),
            7 => bad[199].solution[0].push_str("\n#changed"),
            _ => bad[199].teaching.as_mut().unwrap().concepts[0]
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
            0 => bad["units"][53]["title"] = json!("changed"),
            1 => bad["units"][53]["prerequisites"] = json!([]),
            2 => bad["units"][53]["labs"][0] = json!("shell-greetings"),
            3 => bad["units"][0]["summary"] = json!("changed"),
            4 => {
                bad["units"].as_array_mut().unwrap().pop();
            }
            5 => {
                let x = bad["units"][53].clone();
                bad["units"].as_array_mut().unwrap().push(x);
            }
            6 => bad["units"][53]["track"] = json!("CCNA"),
            _ => bad["units"][53]["summary"] = json!("changed"),
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
    assert_eq!(registration::before_attempts_len(&p.attempts), 199);
    assert_eq!(disk::before_attempts_len(&p.attempts), 193);
    type Parent = fn(&std::collections::BTreeMap<String, Attempt>) -> usize;
    let parents: [(&str, Parent, usize); 5] = [
        ("literal", literal::before_attempts_len, 197),
        ("disk", disk::before_attempts_len, 193),
        ("ERE", ere::before_attempts_len, 190),
        ("NTP", ntp::before_attempts_len, 186),
        ("LVM", lvm::before_attempts_len, 182),
    ];
    for (name, check, expected) in &parents {
        assert_eq!(check(&p.attempts), *expected, "Full201 parent {name}");
        assert_eq!(full(&p), before);
    }
    assert_eq!(p.attempts.len(), 201);
    assert_eq!(full(&p), before);
    for mode in 0..6 {
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
            3 => bad.attempts.get_mut(&all[0].id).unwrap().scenario_version += 1,
            4 => {
                let a = bad.attempts.remove(registration::IDS[0]).unwrap();
                bad.attempts.insert("unreviewed-key".into(), a);
            }
            _ => {
                bad.attempts.get_mut(registration::IDS[0]).unwrap().hints = all[199].hints.len() + 1
            }
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_attempts_len(&bad.attempts)
            }))
            .is_err()
        );
        let untouched = full(&bad);
        for (name, check, _) in &parents {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&bad.attempts)))
                    .is_err(),
                "Full201 refusal through {name}, mutation {mode}"
            );
            assert_eq!(full(&bad), untouched);
        }
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
    for s in &all[199..] {
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

#[test]
fn streaming_typed201_and54_guards_match_complete_original_value_authority() {
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let mut expected = registration::original199();
    let new: pack::Pack = serde_json::from_str(include_str!(
        "../scenarios/shell-snapshot-rollout-draft.json"
    ))
    .unwrap();
    expected.extend(new.scenarios);
    assert_eq!(value(&all), value(&expected));
    let raw = registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let mut expected_units =
        course::load(std::str::from_utf8(&raw).unwrap(), &expected[..199]).unwrap();
    let recipe: Value =
        serde_json::from_slice(include_bytes!("fixtures/catalog201-registration.json")).unwrap();
    expected_units.push(serde_json::from_value(recipe["snapshot_unit"].clone()).unwrap());
    assert_eq!((all.len(), units.len()), (201, 54));
    for (actual, independent) in all.iter().zip(&expected) {
        assert!(disk::serialized_equal(actual, &value(independent)));
        assert!(registration::canonical_serialized_equal(
            actual,
            &serde_json::to_vec(independent).unwrap(),
        ));
    }
    for (actual, independent) in units.iter().zip(&expected_units) {
        assert!(disk::serialized_equal(actual, &value(independent)));
        assert!(registration::canonical_serialized_equal(
            actual,
            &serde_json::to_vec(independent).unwrap(),
        ));
    }
    // Warm only immutable expectations, then report costs without a flaky
    // timing threshold or changing the native route's15-second deadline.
    registration::before_scenarios(&all);
    registration::before_typed_units(&units);
    let values = expected.iter().map(value).collect::<Vec<_>>();
    let start = std::time::Instant::now();
    for (a, e) in all.iter().zip(&values) {
        assert!(disk::serialized_equal(a, e));
    }
    let previous = start.elapsed();
    let start = std::time::Instant::now();
    registration::before_scenarios(&all);
    registration::before_typed_units(&units);
    eprintln!(
        "Complete201/54 streaming guard {:?}; prior201 Value guard {:?}",
        start.elapsed(),
        previous
    );
    // Compare complete fixed201/54 inputs using the default Vec writer. Every
    // measured invocation serializes all actual data afresh; only independent
    // expected bytes/Values are prepared outside the measured region.
    let expected_source_bytes = expected
        .iter()
        .map(|s| serde_json::to_vec(s).unwrap())
        .collect::<Vec<_>>();
    let expected_unit_bytes = expected_units
        .iter()
        .map(|u| serde_json::to_vec(u).unwrap())
        .collect::<Vec<_>>();
    let expected_all_bytes = serde_json::to_vec(&expected).unwrap();
    let expected_units_bytes = serde_json::to_vec(&expected_units).unwrap();
    let unit_values = expected_units.iter().map(value).collect::<Vec<_>>();
    let per_scenario_vec = || {
        for (actual, bytes) in all.iter().zip(&expected_source_bytes) {
            assert_eq!(
                serde_json::to_vec(actual).unwrap().as_slice(),
                bytes.as_slice()
            );
        }
        for (actual, bytes) in units.iter().zip(&expected_unit_bytes) {
            assert_eq!(
                serde_json::to_vec(actual).unwrap().as_slice(),
                bytes.as_slice()
            );
        }
    };
    let whole_catalog_vec = || {
        assert_eq!(serde_json::to_vec(&all).unwrap(), expected_all_bytes);
        assert_eq!(serde_json::to_vec(&units).unwrap(), expected_units_bytes);
    };
    let borrowed_full = || {
        for (actual, expected) in all.iter().zip(&values) {
            assert!(disk::serialized_equal(actual, expected));
        }
        for (actual, expected) in units.iter().zip(&unit_values) {
            assert!(disk::serialized_equal(actual, expected));
        }
    };
    per_scenario_vec();
    whole_catalog_vec();
    borrowed_full();
    for round in 0..3 {
        let started = std::time::Instant::now();
        per_scenario_vec();
        let per_scenario = started.elapsed();
        let started = std::time::Instant::now();
        whole_catalog_vec();
        let whole_catalog = started.elapsed();
        let started = std::time::Instant::now();
        borrowed_full();
        eprintln!(
            "Warmed complete201/54 round{round}: per-object to_vec {per_scenario:?}; whole-catalog to_vec {whole_catalog:?}; borrowed Value {:?}; expected bytes {}/{}",
            started.elapsed(),
            expected_all_bytes.len(),
            expected_units_bytes.len()
        );
    }
    let source_tapes = expected
        .iter()
        .map(registration::expected_source_tape)
        .collect::<Vec<_>>();
    let unit_tapes = expected_units
        .iter()
        .map(registration::expected_source_tape)
        .collect::<Vec<_>>();
    let complete_events = || {
        for (actual, tape) in all.iter().zip(&source_tapes) {
            assert!(registration::event_serialized_equal(actual, tape));
        }
        for (actual, tape) in units.iter().zip(&unit_tapes) {
            assert!(registration::event_serialized_equal(actual, tape));
        }
    };
    complete_events();
    registration::before_scenarios(&all);
    registration::before_typed_units(&units);
    for round in 0..3 {
        let started = std::time::Instant::now();
        complete_events();
        let event_comparison = started.elapsed();
        let started = std::time::Instant::now();
        registration::before_scenarios(&all);
        registration::before_typed_units(&units);
        eprintln!(
            "Warmed complete201/54 event round{round}: independent tape {event_comparison:?}; actual admission {:?}",
            started.elapsed()
        );
    }

}

#[test]
fn streaming_source_guard_rejects_every_top_level_field_and_later_same_pointer_edits() {
    let all = pack::bundled().unwrap();
    for s in &all {
        let original = value(s);
        let bytes = serde_json::to_vec(&original).unwrap();
        assert!(registration::canonical_serialized_equal(&original, &bytes));
        for key in original.as_object().unwrap().keys() {
            let mut changed = original.clone();
            changed[key] = json!({"outside-source-field": true});
            assert!(!disk::serialized_equal(&changed, &original));
            assert!(
                !registration::canonical_serialized_equal(&changed, &bytes),
                "{}/{}",
                s.id,
                key
            );
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(!registration::canonical_serialized_equal(&missing, &bytes));
        }
        let mut extra = original.clone();
        extra["outside-source-field"] = json!(true);
        assert!(!registration::canonical_serialized_equal(&extra, &bytes));
    }
    for unit in course::bundled(&all).unwrap() {
        let original = value(&unit);
        let bytes = serde_json::to_vec(&original).unwrap();
        for key in original.as_object().unwrap().keys() {
            let mut changed = original.clone();
            changed[key] = json!({"outside-unit-field": true});
            assert!(!disk::serialized_equal(&changed, &original));
            assert!(!registration::canonical_serialized_equal(&changed, &bytes));
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(!registration::canonical_serialized_equal(&missing, &bytes));
        }
        let mut extra = original.clone();
        extra["outside-unit-field"] = json!(true);
        assert!(!registration::canonical_serialized_equal(&extra, &bytes));
    }
    let mut current = all.clone();
    let pointer = current.as_ptr();
    registration::before_scenarios(&current);
    for i in [0, 198, 199, 200] {
        current[i].objective.push('!');
        assert_eq!(current.as_ptr(), pointer);
        assert!(std::panic::catch_unwind(|| registration::before_scenarios(&current)).is_err());
        current[i].objective.pop();
        registration::before_scenarios(&current);
    }
    if let Goal::Script { cases } = &mut current[200].goal {
        cases[0].stdout.push('!');
    } else {
        panic!()
    }
    assert!(std::panic::catch_unwind(|| registration::before_scenarios(&current)).is_err());
    let mut units = course::bundled(&all).unwrap();
    let pointer = units.as_ptr();
    registration::before_typed_units(&units);
    for i in [0, 52, 53] {
        units[i].summary.push('!');
        assert_eq!(units.as_ptr(), pointer);
        assert!(std::panic::catch_unwind(|| registration::before_typed_units(&units)).is_err());
        units[i].summary.pop();
        registration::before_typed_units(&units);
    }
}

#[test]
fn streaming_byte_comparator_checks_escaping_lengths_scalars_and_sticky_writer_errors() {
    use serde::{Serialize, Serializer, ser::SerializeSeq};
    #[derive(Serialize)]
    enum Variant {
        Unit,
        New(String),
        Tuple(u8, bool),
        Struct { text: String },
    }
    let variants = [
        Variant::Unit,
        Variant::New("é\n\"\\\r\t".into()),
        Variant::Tuple(5, true),
        Variant::Struct {
            text: "終\u{0}line".into(),
        },
    ];
    for v in variants {
        let bytes = serde_json::to_vec(&v).unwrap();
        assert!(registration::canonical_serialized_equal(&v, &bytes));
        assert!(!registration::canonical_serialized_equal(
            &v,
            &bytes[..bytes.len() - 1]
        ));
        let mut extra = bytes.clone();
        extra.push(b' ');
        assert!(!registration::canonical_serialized_equal(&v, &extra));
        for i in 0..bytes.len() {
            let mut changed = bytes.clone();
            changed[i] ^= 1;
            assert!(!registration::canonical_serialized_equal(&v, &changed));
        }
    }
    assert!(registration::canonical_serialized_equal(&5u64, b"5"));
    assert!(!registration::canonical_serialized_equal(&5u64, b"5.0"));
    assert!(registration::canonical_serialized_equal(&5.0f64, b"5.0"));
    assert!(!registration::canonical_serialized_equal(&5.0f64, b"5"));
    assert!(registration::canonical_serialized_equal(
        &Option::<u8>::None,
        b"null"
    ));
    assert!(registration::canonical_serialized_equal(
        &Some("é"),
        "\"é\"".as_bytes()
    ));
    assert!(registration::canonical_serialized_equal(
        &("x", vec![1u8, 2]),
        b"[\"x\",[1,2]]"
    ));
    struct Swallow;
    impl Serialize for Swallow {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_seq(Some(2))?;
            let _ = seq.serialize_element(&999u64);
            let _ = seq.serialize_element(&7u64);
            seq.end()
        }
    }
    assert!(!registration::canonical_serialized_equal(
        &Swallow, b"[7,7]"
    ));
    struct Broken;
    impl Serialize for Broken {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(
                "deliberate source serialization error",
            ))
        }
    }
    assert!(!registration::canonical_serialized_equal(&Broken, b""));
}

#[test]
fn source_events_detect_every_field_missing_extra_and_nested_same_pointer_changes() {
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    for original in all.iter().map(value).chain(units.iter().map(value)) {
        let tape = registration::expected_source_tape(&original);
        assert!(registration::event_serialized_equal(&original, &tape));
        for key in original.as_object().unwrap().keys() {
            let mut changed = original.clone();
            changed[key] = json!({"outside-event-field":true});
            assert!(!disk::serialized_equal(&changed, &original));
            assert!(!registration::event_serialized_equal(&changed, &tape));
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(!registration::event_serialized_equal(&missing, &tape));
        }
        let mut extra = original.clone();
        extra["outside-event-field"] = json!(true);
        assert!(!registration::event_serialized_equal(&extra, &tape));
    }
    let mut actual = all.clone();
    let pointer = actual.as_ptr();
    registration::before_scenarios(&actual);
    actual[199].teaching.as_mut().unwrap().concepts[0]
        .text
        .push('!');
    assert_eq!(actual.as_ptr(), pointer);
    assert!(std::panic::catch_unwind(|| registration::before_scenarios(&actual)).is_err());
    actual[199].teaching.as_mut().unwrap().concepts[0]
        .text
        .pop();
    registration::before_scenarios(&actual);
    if let Goal::Script { cases } = &mut actual[200].goal {
        cases[0]
            .files
            .insert("/work/outside-event".into(), "é\n".into());
    } else {
        panic!()
    }
    assert_eq!(actual.as_ptr(), pointer);
    assert!(std::panic::catch_unwind(|| registration::before_scenarios(&actual)).is_err());
    // The original same-pointer old199/new201/unit54 mutation tests above are
    // retained and now exercise this same event-backed complete admission.
}

#[test]
fn source_events_cover_metadata_scalars_containers_display_and_sticky_errors() {
    use serde::{
        Serialize, Serializer,
        ser::{SerializeMap, SerializeSeq},
    };
    fn same<T: Serialize>(v: &T) {
        let tape = registration::expected_source_tape(v);
        assert!(registration::event_serialized_equal(v, &tape));
    }
    #[derive(Serialize)]
    struct Unit;
    #[derive(Serialize)]
    struct New(String);
    #[derive(Serialize)]
    struct Pair(u16, bool);
    #[derive(Serialize)]
    enum Variant {
        Unit,
        New(String),
        Tuple(u8, bool),
        Struct { text: String },
    }
    #[derive(Serialize)]
    struct Fields {
        text: String,
        flag: bool,
        absent: Option<u8>,
        present: Option<u8>,
    }
    same(&());
    same(&Unit);
    same(&New("é\n\"\\\u{0}".into()));
    same(&Pair(9, true));
    same(&Variant::Unit);
    same(&Variant::New("終".into()));
    same(&Variant::Tuple(3, false));
    same(&Variant::Struct {
        text: "text".into(),
    });
    same(&Fields {
        text: "é\n".into(),
        flag: true,
        absent: None,
        present: Some(8),
    });
    same(&(vec![1u8, 2], (4u32, 'é'), [true, false]));
    same(&std::collections::BTreeMap::from([("a", 3u64), ("é", 4)]));
    same(&i128::MIN);
    same(&u128::MAX);
    same(&1.25f32);
    same(&2.5f64);
    let unsigned = registration::expected_source_tape(&5u64);
    assert!(!registration::event_serialized_equal(&5.0f64, &unsigned));
    let text = registration::expected_source_tape(&"é\n");
    assert!(!registration::event_serialized_equal(&"é", &text));
    assert!(!registration::event_serialized_equal(&"é\n!", &text));
    let seq = registration::expected_source_tape(&vec![7u64, 7]);
    assert!(!registration::event_serialized_equal(&vec![7u64], &seq));
    assert!(!registration::event_serialized_equal(
        &vec![7u64, 7, 7],
        &seq
    ));
    struct BadLength;
    impl Serialize for BadLength {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_seq(Some(3))?;
            seq.serialize_element(&7u64)?;
            seq.serialize_element(&7u64)?;
            seq.end()
        }
    }
    assert!(!registration::event_serialized_equal(&BadLength, &seq));
    struct Broken;
    impl Serialize for Broken {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("deliberate child failure"))
        }
    }
    struct Swallow;
    impl Serialize for Swallow {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_seq(Some(2))?;
            let _ = seq.serialize_element(&Broken);
            let _ = seq.serialize_element(&7u64);
            seq.end()
        }
    }
    assert!(!registration::event_serialized_equal(&Swallow, &seq));
    assert!(!registration::event_serialized_equal(&Broken, &seq));
    struct Duplicate;
    impl Serialize for Duplicate {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut map = s.serialize_map(Some(1))?;
            map.serialize_entry("a", &7u64)?;
            let _ = map.serialize_entry("a", &7u64);
            map.end()
        }
    }
    let map = registration::expected_source_tape(&std::collections::BTreeMap::from([("a", 7u64)]));
    assert!(!registration::event_serialized_equal(&Duplicate, &map));
    struct Bytes;
    impl Serialize for Bytes {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.serialize_bytes(&[0, 128, 255])
        }
    }
    same(&Bytes);
    struct DisplayText;
    impl std::fmt::Display for DisplayText {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("é")?;
            f.write_str("\ntext")
        }
    }
    struct DisplaySource;
    impl Serialize for DisplaySource {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.collect_str(&DisplayText)
        }
    }
    same(&DisplaySource);
    let display = registration::expected_source_tape(&DisplaySource);
    assert!(!registration::event_serialized_equal(&"é\ntex", &display));
    #[derive(Serialize)]
    struct OtherName {
        text: String,
        flag: bool,
        absent: Option<u8>,
        present: Option<u8>,
    }
    let fields = registration::expected_source_tape(&Fields {
        text: "x".into(),
        flag: true,
        absent: None,
        present: Some(8),
    });
    assert!(!registration::event_serialized_equal(
        &OtherName {
            text: "x".into(),
            flag: true,
            absent: None,
            present: Some(8)
        },
        &fields
    ));
}
