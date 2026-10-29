//! Original143 registration scope retained under current147, with full original140 preservation.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    course,
    engine::{self, State},
    nat_teaching, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const IDS: [&str; 3] = ["ccna-nat-static", "ccna-nat-pool", "ccna-nat-handover"];
const ORIGINAL_PACKS: &[(&str, &str)] = &[
    (
        "scenarios/starter-v1.json",
        include_str!("../scenarios/starter-v1.json"),
    ),
    (
        "scenarios/shell-unit-v1.json",
        include_str!("../scenarios/shell-unit-v1.json"),
    ),
    (
        "scenarios/shell-composition-v1.json",
        include_str!("../scenarios/shell-composition-v1.json"),
    ),
    (
        "scenarios/shell-reusable-v1.json",
        include_str!("../scenarios/shell-reusable-v1.json"),
    ),
    (
        "scenarios/linux-files-v1.json",
        include_str!("../scenarios/linux-files-v1.json"),
    ),
    (
        "scenarios/ipv6-static-v1.json",
        include_str!("../scenarios/ipv6-static-v1.json"),
    ),
    (
        "scenarios/shell-scope-v1.json",
        include_str!("../scenarios/shell-scope-v1.json"),
    ),
    (
        "scenarios/shell-text-v1.json",
        include_str!("../scenarios/shell-text-v1.json"),
    ),
    (
        "scenarios/shell-counts-v1.json",
        include_str!("../scenarios/shell-counts-v1.json"),
    ),
    (
        "scenarios/linux-access-v1.json",
        include_str!("../scenarios/linux-access-v1.json"),
    ),
    (
        "scenarios/vlan-data-v1.json",
        include_str!("../scenarios/vlan-data-v1.json"),
    ),
    (
        "scenarios/shell-file-batches-v1.json",
        include_str!("../scenarios/shell-file-batches-v1.json"),
    ),
    (
        "scenarios/ipv4-static-v1.json",
        include_str!("../scenarios/ipv4-static-v1.json"),
    ),
    (
        "scenarios/dns-forward-v1.json",
        include_str!("../scenarios/dns-forward-v1.json"),
    ),
    (
        "scenarios/shell-arrays-v1.json",
        include_str!("../scenarios/shell-arrays-v1.json"),
    ),
    (
        "scenarios/linux-processes-v1.json",
        include_str!("../scenarios/linux-processes-v1.json"),
    ),
    (
        "scenarios/discovery-v1.json",
        include_str!("../scenarios/discovery-v1.json"),
    ),
    (
        "scenarios/storage-v1.json",
        include_str!("../scenarios/storage-v1.json"),
    ),
    (
        "scenarios/shell-capture-v1.json",
        include_str!("../scenarios/shell-capture-v1.json"),
    ),
    (
        "scenarios/acl-policy-v1.json",
        include_str!("../scenarios/acl-policy-v1.json"),
    ),
    (
        "scenarios/accounts-lifecycle-v1.json",
        include_str!("../scenarios/accounts-lifecycle-v1.json"),
    ),
    (
        "scenarios/ospf-p2p-v1.json",
        include_str!("../scenarios/ospf-p2p-v1.json"),
    ),
    (
        "scenarios/unit-manager-v1.json",
        include_str!("../scenarios/unit-manager-v1.json"),
    ),
    (
        "scenarios/shell-case-v1.json",
        include_str!("../scenarios/shell-case-v1.json"),
    ),
    (
        "scenarios/mac-learning-v1.json",
        include_str!("../scenarios/mac-learning-v1.json"),
    ),
    (
        "scenarios/linux-find-v1.json",
        include_str!("../scenarios/linux-find-v1.json"),
    ),
    (
        "scenarios/dns-reverse-v1.json",
        include_str!("../scenarios/dns-reverse-v1.json"),
    ),
    (
        "scenarios/shell-getopts-v1.json",
        include_str!("../scenarios/shell-getopts-v1.json"),
    ),
    (
        "scenarios/dhcpv4-client-v1.json",
        include_str!("../scenarios/dhcpv4-client-v1.json"),
    ),
    (
        "scenarios/debian-packages-v1.json",
        include_str!("../scenarios/debian-packages-v1.json"),
    ),
    (
        "scenarios/spanning-tree-v1.json",
        include_str!("../scenarios/spanning-tree-v1.json"),
    ),
    (
        "scenarios/shell-safe-automation-v1.json",
        include_str!("../scenarios/shell-safe-automation-v1.json"),
    ),
    (
        "scenarios/apache-static-v1.json",
        include_str!("../scenarios/apache-static-v1.json"),
    ),
];

#[track_caller]
fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(a: &A, b: &B) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete specification/value differs; no field exclusions"
    );
}
#[test]
fn exact_140_typed_scenarios_and35units_are_unchanged_ordered_prefixes_of147() {
    let index: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nat-original140-source-index.json")).unwrap();
    let pins = index["packs"].as_array().unwrap();
    assert_eq!(pins.len(), ORIGINAL_PACKS.len());
    for ((path, text), pin) in ORIGINAL_PACKS.iter().zip(pins) {
        assert_eq!(*path, pin["path"].as_str().unwrap());
        assert_eq!(text.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(text.as_bytes()),
            pin["sha256"].as_str().unwrap(),
            "Original140 authored input changed: {path}"
        );
    }
    let original = pack::load_catalog(
        &ORIGINAL_PACKS
            .iter()
            .map(|(_, text)| *text)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert_eq!(original.len(), 140);
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    exact(&all[..140], &original);
    assert_eq!(
        all[140..143]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|s| s.track == track).count()),
        [53, 40, 32, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all[..176].iter().filter(|s| s.track == track).count()),
        [53, 36, 32, 55]
    );
    let versions = original
        .iter()
        .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
        .collect::<Vec<_>>();
    exact(&versions, &index["catalog"]);
    assert!(original.iter().all(|s| s.nat_teaching.is_none()));
    let units = course::bundled(&all).unwrap();
    assert_eq!(one_off_boundary::before_len(&units), 46);
    let old_units: Vec<course::Unit> =
        serde_json::from_value(index["units"]["units"].clone()).unwrap();
    assert_eq!(index["units"]["version"], 1);
    assert_eq!(old_units.len(), 35);
    exact(&units[..35], &old_units);
    assert_eq!(units[35].id, "basic-nat");
    assert_eq!(units[35].labs, IDS.map(String::from));
    assert_eq!(units[35].prerequisites, ["ipv4-foundations"]);
    for s in &all[140..143] {
        assert!(s.nat_teaching.is_some());
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}
#[test]
fn genuine_old_profiles_keep_exact_attempts_and_receive_no_nat_answers_on_open() {
    let all = pack::bundled().unwrap();
    for (raw, n) in [
        (
            include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json").as_slice(),
            102,
        ),
        (
            include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
            116,
        ),
    ] {
        let parsed: Progress = serde_json::from_slice(raw).unwrap();
        let mut loaded = progress::decode(raw, &all).unwrap();
        assert_eq!(loaded.attempts.len(), n);
        exact(&loaded.attempts, &parsed.attempts);
        exact(&loaded.migration_receipts, &parsed.migration_receipts);
        for id in IDS {
            assert!(!loaded.attempts.contains_key(id));
            assert!(!loaded.favorites.contains(id));
        }
        let before = serde_json::to_value(&loaded.attempts).unwrap();
        for s in &all {
            loaded
                .attempts
                .entry(s.id.clone())
                .or_insert_with(|| Attempt::new(s, 1));
        }
        assert_eq!(one_off_boundary::before_len(&loaded.attempts), 176);
        for (id, value) in before.as_object().unwrap() {
            exact(&loaded.attempts[id], value);
        }
        for s in &all[140..143] {
            let a = &loaded.attempts[&s.id];
            assert_eq!(a.state, s.initial);
            assert!(a.workspace.is_none());
            assert!(!progress::current_solved(a, s));
        }
    }
}
#[test]
fn three_new_reference_and_real_goal_breakage_use_current_state_and_preserve_favorites() {
    let all = pack::bundled().unwrap();
    let review: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nat-basic-review.json")).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (s, row) in all[140..143].iter().zip(review.as_array().unwrap()) {
        assert_eq!(s.id, row["id"].as_str().unwrap());
        let mut a = Attempt::new(s, 2);
        for line in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
        }
        workspace::prepare(&mut a);
        nat_teaching::prepare(&mut a, s.nat_teaching.as_ref().unwrap(), true).unwrap();
        assert!(progress::current_solved(&a, s));
        for action in row["broken"]["actions"].as_array().unwrap() {
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!("{}{}", dlms_learning_lab::nat_workspace::PREFIX, action),
            )
            .unwrap();
        }
        assert!(!engine::solved(&a.state, &s.goal));
        let State::BasicNat { simulation } = &a.state else {
            panic!()
        };
        a.workspace.as_mut().unwrap().nat_draft =
            Some(dlms_learning_lab::nat_workspace::Draft::new(simulation).unwrap());
        assert!(!progress::current_solved(&a, s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    exact(&loaded.attempts, &p.attempts);
    assert_eq!(loaded.favorites, p.favorites);
}
