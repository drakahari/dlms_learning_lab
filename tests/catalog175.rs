//! Physical175 component gates composed into private normal176. Not a runtime/curriculum acceptance claim.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    catalog, course,
    engine::{self, Goal},
    pack, physical_link as core, physical_link_workspace as work,
    progress::{self, Attempt, Progress},
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const PIN: &str = "a594ed8c8a4744101869ac2b32e5a01b14afca0eeebc1884f2490b5d5b3c2050";
const IDS: [&str; 4] = [
    "ccna-physical-media-span",
    "ccna-physical-duplex-diagnosis",
    "ccna-physical-counter-baseline",
    "ccna-physical-uplink-handover",
];
fn value<T: serde::Serialize + ?Sized>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn recipe() -> Value {
    let raw = include_bytes!("fixtures/catalog175-registration.json");
    assert_eq!(digest::hash(raw), PIN);
    serde_json::from_slice(raw).unwrap()
}
fn before() -> Vec<pack::Scenario> {
    let r = recipe();
    let mut texts = Vec::new();
    for p in r["before_packs"].as_array().unwrap() {
        let n = p["path"].as_str().unwrap();
        assert!(n.starts_with("scenarios/") && std::path::Path::new(n).components().count() == 2);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(n);
        let meta = std::fs::symlink_metadata(&path).unwrap();
        assert!(
            meta.is_file()
                && !meta.file_type().is_symlink()
                && meta.len() <= pack::MAX_PACK_BYTES as u64
        );
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(bytes.len() as u64, p["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&bytes), p["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(bytes).unwrap());
    }
    assert_eq!(texts.len(), 42);
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn authored() -> Value {
    let raw = include_bytes!("fixtures/physical-link-author-review.json");
    assert_eq!(digest::hash(raw), recipe()["authored_r3_raw_sha256"]);
    serde_json::from_slice(raw).unwrap()
}
fn config(v: &Value) -> core::Configuration {
    serde_json::from_value(v.clone()).unwrap()
}
fn applied(s: &pack::Scenario, c: &core::Configuration) -> Attempt {
    let mut a = Attempt::new(s, 11);
    work::prepare_attempt(&mut a, s, true).unwrap();
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .as_ref()
        .unwrap()
        .clone();
    draft.configuration = c.clone();
    work::set_draft(&mut a, s, work::Scope::Assignment, &draft).unwrap();
    work::apply(&mut a, s, work::Scope::Assignment).unwrap();
    a
}
#[test]
fn complete171_prefix_specs_order_all55_scripts420_cases_and55_examples_remain_exact() {
    let old = before();
    let all = pack::bundled().unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&all)), (171, 176));
    assert_eq!(value(&all[..171]), value(&old));
    assert_eq!(
        value(
            &old.iter()
                .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        recipe()["before_ids"]
    );
    assert_eq!(
        all[171..175]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let scripts = all
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 57);
    assert_eq!(
        scripts
            .iter()
            .map(|s| if let Goal::Script { cases } = &s.goal {
                cases.len()
            } else {
                0
            })
            .sum::<usize>(),
        442
    );
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 57);
    assert_eq!(dlms_learning_lab::shell::EVALUATOR_REVISION, 13);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|t| {
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == t)
                .count()
        }),
        [53, 40, 32, 57]
    );
    assert_eq!(["CCNA","LPIC-1","LPIC-2","Bash/Shell"].map(|t|one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).iter().filter(|s|s.track==t).count()),[53,40,36,57]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all[..176].iter().filter(|s| s.track == t).count()),
        [53, 36, 32, 55]
    );
}
#[test]
fn normal_four_preserve_authored_models_teaching_references_and_exact_negative_outcomes() {
    let all = pack::bundled().unwrap();
    let review = authored();
    let bytes = include_bytes!("../scenarios/physical-links-v1.json");
    assert_eq!(digest::hash(bytes), recipe()["canonical_pack_sha256"]);
    let normal: pack::Pack = serde_json::from_slice(bytes).unwrap();
    assert_eq!(value(&all[171..175]), value(&normal.scenarios));
    for (s, row) in all[171..175]
        .iter()
        .zip(review["tasks"].as_array().unwrap())
    {
        let Goal::PhysicalLink { definition } = &s.goal else {
            panic!("Typed physical Goal")
        };
        assert_eq!(value(definition.as_ref()), row["definition"]);
        assert_eq!(
            value(s.physical_link_teaching.as_ref().unwrap()),
            row["introduction"]
        );
        assert_eq!(s.difficulty, row["difficulty"].as_str().unwrap());
        assert!(engine::checks(&s.initial, &s.goal).iter().any(|r| !r.0));
        for reference in row["references"].as_array().unwrap() {
            let mut a = applied(s, &config(reference));
            assert!(work::checks(&a, s).iter().all(|r| r.0));
            assert!(progress::current_solved(&a, s));
            let before = value(&a);
            work::prepare_attempt(&mut a, s, false).unwrap();
            assert_eq!(value(&a), before);
            work::set_checked(&mut a, s, work::Scope::Assignment, true).unwrap();
            assert!(progress::current_solved(&a, s));
            let mut draft = a
                .workspace
                .as_ref()
                .unwrap()
                .physical_link_draft
                .as_ref()
                .unwrap()
                .clone();
            draft.configuration = config(&row["break_after_success"]);
            work::set_draft(&mut a, s, work::Scope::Assignment, &draft).unwrap();
            assert!(!progress::current_solved(&a, s));
            work::apply(&mut a, s, work::Scope::Assignment).unwrap();
            assert!(!progress::current_solved(&a, s));
        }
        for name in ["wrong", "partial"] {
            let a = applied(s, &config(&row[name]["configuration"]));
            assert!(!progress::current_solved(&a, s), "{}/{name}", s.id);
            assert!(work::checks(&a, s).iter().any(|r| !r.0));
        }
    }
}
#[test]
fn exact45_append_recovers_complete44_raw_objects_and_every_older_closed_inverse() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let r = recipe();
    let old = combined_catalog::before_physical_units(raw).unwrap();
    assert_eq!(digest::hash(&old), r["before_units44_sha256"]);
    assert_eq!(
        value(&serde_json::from_slice::<Value>(&old).unwrap()),
        r["before_units44"]
    );
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let old_units = course::load(std::str::from_utf8(&old).unwrap(), &before()).unwrap();
    assert_eq!(
        (old_units.len(), one_off_boundary::before_len(&units)),
        (44, 46)
    );
    assert_eq!(value(&units[..44]), value(&old_units));
    assert_eq!(value(&units[44]), r["physical_unit"]);
    assert_eq!(units[44].labs, IDS.map(str::to_owned));
    let b43 = combined_catalog::before_numeric_units(&old).unwrap();
    assert_eq!(combined_catalog::before_numeric_units(raw).unwrap(), b43);
    assert_eq!(
        combined_catalog::before_backup_units(raw).unwrap(),
        combined_catalog::before_backup_units(&old).unwrap()
    );
    assert_eq!(
        combined_catalog::old_unit_bytes(raw).unwrap(),
        combined_catalog::old_unit_bytes(&old).unwrap()
    );
    assert_eq!(
        digest::hash(&combined_catalog::old_unit_bytes(raw).unwrap()),
        "e87bd9685cc69cef5a3c4929497323cee15fb44cc1518e7c16fec09c24bcc574"
    );
}
#[test]
fn closed_physical_inverse_rejects_missing_extra_changed_original_and_reordered_units() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let old = combined_catalog::before_physical_units(raw).unwrap();
    assert!(combined_catalog::before_physical_units(&old).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(combined_catalog::before_physical_units(&extra).is_err());
    for change in 0..4 {
        let mut v: Value = serde_json::from_slice(raw).unwrap();
        match change {
            0 => v["units"][0]["summary"] = "changed original".into(),
            1 => {
                let units = v["units"].as_array_mut().unwrap();
                let at = units
                    .iter()
                    .position(|u| u["id"] == "physical-uplink-diagnosis")
                    .unwrap();
                assert_eq!(at, 44);
                units.remove(at);
            }
            2 => v["units"][44]["labs"][0] = "changed new task".into(),
            _ => v["units"].as_array_mut().unwrap().swap(43, 44),
        }
        assert!(combined_catalog::before_physical_units(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
#[test]
fn existing_actual_writer_answers_records_and_receipts_get_no_new_physical_attempts_or_mastery() {
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    assert_eq!(
        digest::hash(raw),
        "88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26"
    );
    let original = progress::decode(raw, &before()).unwrap();
    let current = progress::decode(raw, &pack::bundled().unwrap()).unwrap();
    assert_eq!(value(&original), value(&current));
    assert_eq!(
        value(&original.migration_receipts),
        value(&current.migration_receipts)
    );
    assert_eq!(original.attempts.len(), 116);
    for id in IDS {
        assert!(!current.attempts.contains_key(id));
        assert!(!current.favorites.contains(id));
    }
}
#[test]
fn normal_physical_partial_drafts_resume_and_selected_reset_preserves_global_others_and_real_receipts()
 {
    let all = pack::bundled().unwrap();
    let rows = authored();
    let s = &all[171];
    let other = &all[172];
    let a = applied(s, &config(&rows["tasks"][0]["references"][0]));
    let mut partial = Attempt::new(other, 19);
    work::prepare_attempt(&mut partial, other, true).unwrap();
    let mut d = partial
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .as_ref()
        .unwrap()
        .clone();
    d.configuration = config(&rows["tasks"][1]["partial"]["configuration"]);
    work::set_draft(&mut partial, other, work::Scope::Assignment, &d).unwrap();
    let p = Progress {
        schema_version: 1,
        clock: 20,
        favorites: std::collections::BTreeSet::from([s.id.clone(), other.id.clone()]),
        attempts: std::collections::BTreeMap::from([
            (s.id.clone(), a),
            (other.id.clone(), partial),
        ]),
        migration_receipts: None,
    };
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let mut loaded = progress::decode(&raw, &all).unwrap();
    assert_eq!(value(&p), value(&loaded));
    assert!(loaded.migration_receipts.as_ref().unwrap().workspace);
    let receipts = value(&loaded.migration_receipts);
    let peer = value(&loaded.attempts[&other.id]);
    let clock = loaded.clock;
    let fav = loaded.favorites.clone();
    let visit = loaded.attempts[&s.id].visited;
    work::reset(
        loaded.attempts.get_mut(&s.id).unwrap(),
        s,
        work::Scope::Assignment,
    )
    .unwrap();
    assert_eq!(value(&loaded.attempts[&other.id]), peer);
    assert_eq!(value(&loaded.migration_receipts), receipts);
    assert_eq!(loaded.clock, clock);
    assert_eq!(loaded.favorites, fav);
    assert_eq!(loaded.attempts[&s.id].visited, visit);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn certification_topic_search_favorites_and_prerequisites_find_the_meaningful_normal_unit() {
    let all = pack::bundled().unwrap();
    let mut filter = catalog::Filter {
        track: "CCNA".into(),
        topic: "Physical interfaces and troubleshooting".into(),
        ..Default::default()
    };
    let found = all
        .iter()
        .filter(|s| catalog::matches(s, &filter, false, false, false))
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(found, IDS);
    filter.search = "fresh receive-counter changes".into();
    assert_eq!(
        all.iter()
            .filter(|s| catalog::matches(s, &filter, false, false, false))
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [IDS[2]]
    );
    filter.search.clear();
    filter.favorites = true;
    assert_eq!(
        all.iter()
            .filter(|s| catalog::matches(s, &filter, true, false, s.id == IDS[1]))
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [IDS[1]]
    );
    for (n, s) in all[171..175].iter().enumerate() {
        assert!(
            s.prerequisites
                .iter()
                .all(|p| IDS[..n].contains(&p.as_str()))
        );
    }
}

#[test]
fn private46_inverse_recovers_exact_physical45_then_every_original_closed_unit_recipe() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let bytes = include_bytes!("fixtures/catalog176-registration.json");
    assert_eq!(
        digest::hash(bytes),
        "690e81765c71f591972c921fed7ce417eb21e67afdd272d8049789290757898f"
    );
    let r: Value = serde_json::from_slice(bytes).unwrap();
    let before45 = combined_catalog::before_private_units(raw).unwrap();
    assert_eq!(
        before45.as_slice(),
        include_bytes!("fixtures/private-office-units-before175.json")
    );
    assert_eq!(before45.len() as u64, r["before_units45_bytes"]);
    assert_eq!(digest::hash(&before45), r["before_units45_sha256"]);
    assert_eq!(
        serde_json::from_slice::<Value>(&before45).unwrap(),
        r["before_units45"]
    );
    let all = pack::bundled().unwrap();
    let all175 = all[..175].to_vec();
    assert_eq!(all[175].id, "ccna-private-office-plan");
    let old = course::load(std::str::from_utf8(&before45).unwrap(), &all175).unwrap();
    let current = course::bundled(&all).unwrap();
    assert_eq!(
        (old.len(), one_off_boundary::before_len(&current)),
        (45, 46)
    );
    assert_eq!(value(&current[..45]), value(&old));
    assert_eq!(value(&current[45]), r["private_unit"]);
    assert_eq!(current[45].labs, ["ccna-private-office-plan"]);
    let old44 = combined_catalog::before_physical_units(&before45).unwrap();
    assert_eq!(combined_catalog::before_physical_units(raw).unwrap(), old44);
    assert_eq!(
        combined_catalog::before_numeric_units(raw).unwrap(),
        combined_catalog::before_numeric_units(&before45).unwrap()
    );
    assert_eq!(
        combined_catalog::before_backup_units(raw).unwrap(),
        combined_catalog::before_backup_units(&before45).unwrap()
    );
    assert_eq!(
        combined_catalog::before_cron_units(raw).unwrap(),
        combined_catalog::before_cron_units(&before45).unwrap()
    );
    // These retained older inverses intentionally accept their own exact stage,
    // not arbitrary newer numeric/backup/Cron appends.
    assert!(combined_catalog::before_optional_units(raw).is_err());
    assert!(combined_catalog::before_lacp_units(raw).is_err());
    let before41 = combined_catalog::before_cron_units(raw).unwrap();
    let before40 = combined_catalog::before_optional_units(&before41).unwrap();
    let before39 = combined_catalog::before_lacp_units(&before40).unwrap();
    let combined: Value =
        serde_json::from_str(include_str!("fixtures/catalog160-registration.json")).unwrap();
    assert_eq!(digest::hash(&before41), combined["combined41_sha256"]);
    assert_eq!(digest::hash(&before40), combined["ether40_sha256"]);
    let lacp: Value = serde_json::from_str(include_str!(
        "fixtures/catalog157-original154-boundary.json"
    ))
    .unwrap();
    assert_eq!(digest::hash(&before39), lacp["original_units_sha256"]);
    assert_eq!(
        combined_catalog::old_unit_bytes(&before39).unwrap(),
        combined_catalog::old_unit_bytes(raw).unwrap()
    );
    assert_eq!(
        combined_catalog::old_unit_bytes(raw).unwrap(),
        combined_catalog::old_unit_bytes(&before45).unwrap()
    );
    assert_eq!(
        digest::hash(&combined_catalog::old_unit_bytes(raw).unwrap()),
        "e87bd9685cc69cef5a3c4929497323cee15fb44cc1518e7c16fec09c24bcc574"
    );
}
#[test]
fn private46_inverse_rejects_missing_extra_changed_old_new_reordered_and_duplicate_units() {
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = combined_catalog::before_private_units(raw).unwrap();
    assert!(combined_catalog::before_private_units(&before).is_err());
    let mut trailing = raw.to_vec();
    trailing.push(b' ');
    assert!(combined_catalog::before_private_units(&trailing).is_err());
    for change in 0..6 {
        let mut changed: Value = serde_json::from_slice(raw).unwrap();
        match change {
            0 => changed["units"][0]["title"] = "changed historical unit".into(),
            1 => changed["units"][44]["summary"] = "changed physical component".into(),
            2 => changed["units"][45]["prerequisites"] = serde_json::json!([]),
            3 => {
                let units = changed["units"].as_array_mut().unwrap();
                let at = units
                    .iter()
                    .position(|u| u["id"] == "private-office-planning")
                    .unwrap();
                assert_eq!(at, 45);
                units.remove(at);
            }
            4 => changed["units"].as_array_mut().unwrap().swap(44, 45),
            _ => {
                let duplicate = changed["units"][45].clone();
                changed["units"].as_array_mut().unwrap().push(duplicate);
            }
        }
        let malformed = serde_json::to_vec_pretty(&changed).unwrap();
        assert!(combined_catalog::before_private_units(&malformed).is_err());
        assert!(combined_catalog::before_physical_units(&malformed).is_err());
        assert!(combined_catalog::before_numeric_units(&malformed).is_err());
        assert!(combined_catalog::old_unit_bytes(&malformed).is_err());
    }
}
