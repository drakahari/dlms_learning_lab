//! Exact registration of four reviewed native disk assignments; no exam/platform claim.
use dlms_learning_lab::{
    backup, course, developer_digest as digest,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    partition_fs_workspace as work,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde_json::{Value, json};
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::{
    self as ere, partition_fs_registration as registration,
};
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn full(p: &Progress) -> Value {
    json!({"portable":p,"receipts":p.migration_receipts})
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    work::binding::prepare(&mut a, s).unwrap();
    a
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::PartitionFs { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn complete193_sources_and_all60_script_cases_stay_before_four_normal_native_assignments() {
    let all = pack::bundled().unwrap();
    let old = registration::original193();
    assert_eq!((all.len(), old.len()), (197, 193));
    assert_eq!(value(registration::before_scenarios(&all)), value(&old));
    assert_eq!(value(&all[..193]), value(&old));
    let authored: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/disk-preparation-v1.json")).unwrap();
    assert_eq!(value(&all[193..]), value(&authored.scenarios));
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 44, 36, 60]
    );
    let scripts = all
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 60);
    assert_eq!(
        scripts
            .iter()
            .map(|s| {
                let Goal::Script { cases } = &s.goal else {
                    unreachable!()
                };
                cases.len()
            })
            .sum::<usize>(),
        477
    );
    for s in &all[193..] {
        assert_eq!(s.track, "LPIC-1");
        assert_eq!(s.unit, "linux-disk-preparation");
        assert!(
            matches!(s.initial, State::PartitionFs { .. })
                && matches!(s.goal, Goal::PartitionFs { .. })
        );
        assert!(!progress::current_solved(&fresh(s), s));
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
    }
}
#[test]
fn closed52_to51_raw_and_full_typed_inverse_retains_every_old_unit_and_all_prior_boundaries() {
    let all = pack::bundled().unwrap();
    let old = registration::original193();
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert_eq!(before, include_bytes!("fixtures/disk-units-before193.json"));
    let recipe: Value =
        serde_json::from_str(include_str!("fixtures/catalog197-registration.json")).unwrap();
    assert_eq!(digest::hash(raw), recipe["after_units52_sha256"]);
    assert_eq!(digest::hash(&before), recipe["before_units51_sha256"]);
    let units = course::bundled(&all).unwrap();
    let previous = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    assert_eq!((units.len(), previous.len()), (52, 51));
    assert_eq!(
        value(registration::before_typed_units(&units)),
        value(&previous)
    );
    assert_eq!(value(&units[..51]), value(&previous));
    assert_eq!(value(&units[51]), recipe["disk_unit"]);
    assert_eq!(units[51].labs, registration::IDS);
    assert_eq!(units[51].prerequisites, ["linux-file-foundations"]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [15, 11, 9, 17]
    );
    let decoded = course::load(&json!({"version":1,"units":units}).to_string(), &all).unwrap();
    assert_eq!(value(&decoded), value(&units));
    assert_eq!(
        ere::before_units(raw).unwrap(),
        ere::before_units(&before).unwrap()
    );
    assert_eq!(
        value(ere::before_typed_units(&units)),
        value(ere::before_typed_units(&previous))
    );
    assert!(registration::before_units(&before).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(registration::before_units(&extra).is_err());
    for mode in 0..7 {
        let mut bad: Value = serde_json::from_slice(raw).unwrap();
        match mode {
            0 => bad["units"][51]["title"] = json!("changed"),
            1 => bad["units"][51]["summary"] = json!("changed"),
            2 => bad["units"][51]["prerequisites"] = json!([]),
            3 => bad["units"][51]["labs"][0] = json!("ccna-ipv4"),
            4 => bad["units"][0]["title"] = json!("changed old source"),
            5 => {
                bad["units"].as_array_mut().unwrap().pop().unwrap();
            }
            _ => {
                let duplicate = bad["units"][51].clone();
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
fn exact_source_inverse_rejects_partial_extra_changed_old_or_new_source_and_foreign_family() {
    let all = pack::bundled().unwrap();
    for mode in 0..9 {
        let mut bad = all.clone();
        match mode {
            0 => {
                bad.pop().unwrap();
            }
            1 => bad.push(all[193].clone()),
            2 => bad[193].objective.push_str(" changed"),
            3 => bad[193].hints[0].push_str(" changed"),
            4 => {
                let Goal::PartitionFs { definition } = &mut bad[193].goal else {
                    panic!()
                };
                definition.reference.push(definition.reference[0].clone());
            }
            5 => bad[193].initial = all[0].initial.clone(),
            6 => bad[0].version += 1,
            7 => bad[0].objective.push_str(" changed old whole source"),
            _ => {
                bad[193].terminal_teaching = all.iter().find_map(|s| s.terminal_teaching.clone());
                assert!(bad[193].terminal_teaching.is_some());
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
fn count_only_predecessor_preserves_full_map_validates_each_new_context_and_awards_no_credit() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let before = full(&p);
    assert_eq!(registration::before_attempts_len(&p.attempts), 193);
    assert_eq!(ere::before_attempts_len(&p.attempts), 190);
    assert_eq!(p.attempts.len(), 197);
    assert_eq!(full(&p), before);
    for mode in 0..4 {
        let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
        match mode {
            0 => {
                let a = bad.attempts.remove(&all[193].id).unwrap();
                bad.attempts.insert("foreign-id".into(), a);
            }
            1 => bad.attempts.get_mut(&all[193].id).unwrap().scenario_version += 1,
            2 => bad.attempts.get_mut(&all[193].id).unwrap().state = all[0].initial.clone(),
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
    for s in &all[193..] {
        assert!(!progress::current_solved(&p.attempts[&s.id], s));
    }
}
#[test]
fn retained_profile_does_not_invent_new_disk_attempts_answers_favorites_or_completion() {
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
fn all_four_normal_assignment_and_example_drafts_resume_separately_with_validated_backup() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 41,
        ..Default::default()
    };
    for s in &all[193..] {
        let mut a = fresh(s);
        let State::PartitionFs { simulation } = &mut a.state else {
            panic!()
        };
        simulation.view = View::Example;
        let disk = simulation
            .current
            .disks
            .iter()
            .find(|(_, d)| d.writable && !d.in_use)
            .unwrap()
            .0
            .clone();
        simulation.assignment_editor.selected = disk.clone();
        simulation
            .assignment_editor
            .drafts
            .get_mut(&disk)
            .unwrap()
            .name = "assignment-draft".into();
        let disk = simulation
            .example
            .disks
            .iter()
            .find(|(_, d)| d.writable && !d.in_use)
            .unwrap()
            .0
            .clone();
        simulation.example_editor.selected = disk.clone();
        simulation
            .example_editor
            .drafts
            .get_mut(&disk)
            .unwrap()
            .name = "example-draft".into();
        work::binding::set_challenge(&mut a, s, true).unwrap();
        assert!(!progress::current_solved(&a, s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, &all).unwrap();
    assert_eq!(full(&p), before);
    let resumed = progress::load(&file, &all).unwrap();
    assert_eq!(value(&resumed), value(&p));
    for s in &all[193..] {
        let a = &resumed.attempts[&s.id];
        assert!(a.challenge);
        assert!(!progress::current_solved(a, s));
        work::context(a, s).unwrap();
    }
    let portable = dir.path().join("portable.json");
    backup::export(&portable, &resumed, &all).unwrap();
    let decoded = backup::inspect(
        &portable,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(value(&decoded.progress), value(&resumed));
    assert!(
        all[..193]
            .iter()
            .all(|s| !resumed.attempts.contains_key(&s.id))
    );
}
#[test]
fn actual_normal_sources_accept_reference_and_two_alternatives_then_invalidate_after_later_damage()
{
    let all = pack::bundled().unwrap();
    let review: Value =
        serde_json::from_str(include_str!("fixtures/disk-preparation-review.json")).unwrap();
    for s in &all[193..] {
        let row = review["labs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == s.id)
            .unwrap();
        assert_eq!(value(&definition(s).reference), row["reference"]);
        for (n, ops) in std::iter::once(&row["reference"])
            .chain(row["alternatives"].as_array().unwrap())
            .enumerate()
        {
            let mut a = fresh(s);
            let operations: Vec<work::Operation> = serde_json::from_value(ops.clone()).unwrap();
            for op in operations {
                work::binding::apply(&mut a, s, &op, false).unwrap();
            }
            assert!(progress::current_solved(&a, s));
            if n == 0 {
                let damage: work::Operation =
                    serde_json::from_value(row["damage_after_reference"]["operation"].clone())
                        .unwrap();
                work::binding::apply(&mut a, s, &damage, false).unwrap();
                assert!(!progress::current_solved(&a, s));
            }
            let before = value(&a.state);
            assert!(engine::execute(&mut a.state, "mkfs.ext4 /dev/vda2").is_err());
            assert_eq!(value(&a.state), before);
        }
    }
}

#[test]
fn borrowed_complete_serialized_equality_matches_all197_and_scalar_container_corpus_without_masking_fields()
 {
    use serde::{
        Serialize, Serializer,
        ser::{SerializeMap, SerializeSeq},
    };
    use std::{collections::BTreeMap, fmt};
    fn parity<T: ?Sized + Serialize>(actual: &T) {
        let expected = serde_json::to_value(actual).unwrap();
        assert!(
            registration::serialized_equal(actual, &expected),
            "{expected:?}"
        );
        let wrong = match &expected {
            Value::Null => json!(false),
            Value::Bool(v) => json!(!v),
            Value::Number(_) => json!("not a number"),
            Value::String(v) => json!(format!("{v}!")),
            Value::Array(v) => {
                let mut v = v.clone();
                v.push(Value::Null);
                Value::Array(v)
            }
            Value::Object(v) => {
                let mut v = v.clone();
                v.insert("outside-key".into(), Value::Null);
                Value::Object(v)
            }
        };
        assert!(!registration::serialized_equal(actual, &wrong));
    }
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 197);
    let expected = all.iter().map(value).collect::<Vec<_>>();
    let start = std::time::Instant::now();
    for (actual, expected) in all.iter().zip(&expected) {
        assert_eq!(serde_json::to_value(actual).unwrap(), *expected);
    }
    let allocated_elapsed = start.elapsed();
    let start = std::time::Instant::now();
    for (actual, expected) in all.iter().zip(&expected) {
        assert!(registration::serialized_equal(actual, expected));
    }
    let borrowed_elapsed = start.elapsed();
    eprintln!(
        "One ordinary all197 source corpus: to_value comparison {allocated_elapsed:?}; borrowed exact comparison {borrowed_elapsed:?}"
    );
    for (actual, expected) in all.iter().zip(&expected) {
        for field in [
            "id",
            "version",
            "objective",
            "goal",
            "initial",
            "guide",
            "sources",
            "hints",
        ] {
            let mut changed = expected.clone();
            changed[field] = json!({"outside-field": "mutation"});
            assert!(
                !registration::serialized_equal(actual, &changed),
                "{}/{field}",
                actual.id
            );
        }
        let mut missing = expected.clone();
        missing.as_object_mut().unwrap().remove("goal");
        assert!(!registration::serialized_equal(actual, &missing));
        let mut extra = expected.clone();
        extra["outside-field"] = Value::Null;
        assert!(!registration::serialized_equal(actual, &extra));
    }
    parity(&());
    parity(&None::<u8>);
    parity(&Some("kept"));
    parity(&true);
    parity(&'é');
    parity(&"escaped \"quote\"\n\\ and UTF8 é");
    parity(&i64::MIN);
    parity(&u64::MAX);
    parity(&i128::from(i64::MIN));
    parity(&u128::from(u64::MAX));
    parity(&5.0f64);
    parity(&-0.0f32);
    parity(&1.25f32);
    parity(&f64::NAN);
    parity(&f64::INFINITY);
    assert!(!registration::serialized_equal(&5u64, &json!(5.0)));
    assert!(!registration::serialized_equal(&5.0f64, &json!(5)));
    assert!(!registration::serialized_equal(&i128::MIN, &Value::Null));
    assert!(!registration::serialized_equal(&u128::MAX, &Value::Null));
    parity(&("tuple", 3, Some(false)));
    assert!(!registration::serialized_equal(&vec![1, 2], &json!([1])));
    assert!(!registration::serialized_equal(&vec![1], &json!([1, 2])));
    parity(&vec![vec![1u8, 2], vec![]]);
    parity(&BTreeMap::from([(2u64, "two"), (10u64, "ten")]));
    parity(&BTreeMap::from([(false, "no"), (true, "yes")]));
    // Exercise larger-map seen-key spill as well as ordinary small stack maps.
    parity(
        &(0..100)
            .map(|i| (format!("key-{i}"), i))
            .collect::<BTreeMap<_, _>>(),
    );
    #[derive(Serialize)]
    struct Named<'a> {
        first: &'a str,
        second: Option<u8>,
    }
    #[derive(Serialize)]
    struct Newtype(u16);
    #[derive(Serialize)]
    struct Tuple(u8, &'static str);
    #[derive(Serialize)]
    enum Tagged {
        Unit,
        Newtype(u8),
        Tuple(&'static str, bool),
        Struct { value: &'static str },
    }
    #[derive(Serialize)]
    #[serde(tag = "kind", content = "body")]
    enum Adjacent {
        Plain,
        Named { value: u8 },
    }
    #[derive(Serialize)]
    struct Flatten {
        present: u8,
        #[serde(flatten)]
        extra: BTreeMap<&'static str, u8>,
    }
    parity(&Named {
        first: "same",
        second: None,
    });
    parity(&Newtype(7));
    parity(&Tuple(3, "three"));
    for v in [
        Tagged::Unit,
        Tagged::Newtype(7),
        Tagged::Tuple("a", false),
        Tagged::Struct { value: "a" },
    ] {
        parity(&v);
    }
    assert!(!registration::serialized_equal(
        &Tagged::Newtype(7),
        &json!({"Other":7})
    ));
    assert!(!registration::serialized_equal(
        &Tagged::Newtype(7),
        &json!({"Newtype":7,"extra":0})
    ));
    parity(&Adjacent::Plain);
    parity(&Adjacent::Named { value: 3 });
    parity(&Flatten {
        present: 1,
        extra: BTreeMap::from([("extra", 2)]),
    });
    struct Bytes;
    impl Serialize for Bytes {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.serialize_bytes(&[0, 127, 255])
        }
    }
    parity(&Bytes);
    struct Displayed;
    impl fmt::Display for Displayed {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "piece:é")
        }
    }
    impl Serialize for Displayed {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.collect_str(self)
        }
    }
    parity(&Displayed);
    struct FloatKey(f32);
    impl Serialize for FloatKey {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(Some(1))?;
            m.serialize_entry(&self.0, &"float")?;
            m.end()
        }
    }
    parity(&FloatKey(1.0));
    parity(&FloatKey(-0.0));
    parity(&FloatKey(1.25));
    assert!(!registration::serialized_equal(
        &FloatKey(f32::NAN),
        &json!({"null":"float"})
    ));
    struct OrderedMap;
    impl Serialize for OrderedMap {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(None)?;
            m.serialize_entry("z", &1)?;
            m.serialize_entry("a", &2)?;
            m.end()
        }
    }
    parity(&OrderedMap);
    struct Duplicate;
    impl Serialize for Duplicate {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(None)?;
            m.serialize_entry("same", &7)?;
            let _ = m.serialize_entry("same", &7);
            m.end()
        }
    }
    assert!(
        !registration::serialized_equal(&Duplicate, &json!({"same":7})),
        "Duplicate keys fail even if an erroneous custom serializer ignores the error"
    );
    struct IgnoredSequenceError;
    impl Serialize for IgnoredSequenceError {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_seq(None)?;
            let _ = seq.serialize_element(&999);
            let _ = seq.serialize_element(&7);
            seq.end()
        }
    }
    assert!(!registration::serialized_equal(
        &IgnoredSequenceError,
        &json!([7])
    ));
    struct IgnoredExtraKey;
    impl Serialize for IgnoredExtraKey {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(None)?;
            let _ = m.serialize_entry("outside", &1);
            let _ = m.serialize_entry("kept", &7);
            m.end()
        }
    }
    assert!(!registration::serialized_equal(
        &IgnoredExtraKey,
        &json!({"kept":7})
    ));
    struct IgnoredDisplayError;
    impl fmt::Display for IgnoredDisplayError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let _ = f.write_str("wrong");
            f.write_str("kept")
        }
    }
    impl Serialize for IgnoredDisplayError {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.collect_str(self)
        }
    }
    assert!(!registration::serialized_equal(
        &IgnoredDisplayError,
        &json!("kept")
    ));
    struct MissingMapValue;
    impl Serialize for MissingMapValue {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(None)?;
            m.serialize_key("key")?;
            m.end()
        }
    }
    assert!(!registration::serialized_equal(
        &MissingMapValue,
        &json!({"key":1})
    ));
}
