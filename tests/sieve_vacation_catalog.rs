use dlms_learning_lab::{catalog, course, developer_digest, pack};
const IDS: [&str; 2] = [
    "lpic2-sieve-vacation-safety",
    "lpic2-sieve-vacation-routing-recovery",
];
#[test]
fn all475_scenarios_and188_units_are_retained_before_the_new_two_tasks() {
    let before = pack::accepted475().unwrap();
    let now = pack::accepted477().unwrap();
    assert_eq!((before.len(), now.len()), (475, 477));
    assert_eq!(
        serde_json::to_value(&now[..475]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        now[475..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let old = course::accepted475(&before).unwrap();
    let units = course::accepted477(&now).unwrap();
    assert_eq!((old.len(), units.len()), (188, 189));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-sieve-vacation")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(old).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-sieve-vacation")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at].prerequisites, ["linux-sieve-envelope-delivery"]);
    assert!(
        units
            .iter()
            .position(|u| u.id == "linux-sieve-envelope-delivery")
            .unwrap()
            < at
    );
    for (track, n) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 130),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
    }
    let filter = catalog::Filter {
        track: "LPIC-2".into(),
        search: "vacation".into(),
        ..Default::default()
    };
    assert!(
        now[475..]
            .iter()
            .all(|s| catalog::matches_filters(s, &filter, false, false))
    );
}
#[test]
fn registry_extraction_keeps_both_retained_factories_verbatim_and_public() {
    let source = include_str!("../src/pack_retained.rs");
    let block=source.strip_prefix("//! Retained470/472 catalog factories. Bodies and source paths are unchanged.\nuse super::{Scenario, load_bundled_catalog};\n").unwrap();
    assert_eq!(block.len(), 15708);
    assert_eq!(
        developer_digest::hash(block.as_bytes()),
        "8cbe085d34266a499031c747747a6d260a87f033175b0cc5628a6c29e5755512"
    );
    let before = pack::accepted470().unwrap();
    let after = pack::accepted472().unwrap();
    assert_eq!((before.len(), after.len()), (470, 472));
    assert_eq!(
        serde_json::to_value(&after[..470]).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}
#[test]
fn current_consolidation_is_exact102_plus_two_without_raised_source_or_pack_limits() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v49.json")).unwrap();
    let leaf: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-vacation-lessons-v1.json")).unwrap();
    let raw = include_str!("../scenarios/boot-entry-lessons-v50.json");
    let new: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = new["scenarios"].as_array().unwrap();
    assert_eq!(rows.len(), 104);
    assert_eq!(&rows[..102], old["scenarios"].as_array().unwrap());
    assert_eq!(&rows[102..], leaf["scenarios"].as_array().unwrap());
    assert!(raw.len() < 2_000_000);
    assert_eq!(pack::MAX_BUNDLED_PACK_FILES, 128);
    assert!(include_str!("../src/pack.rs").len() <= 512000);
}

#[test]
fn all128_prior_teaching_records_stay_exact_before_the_two_individual_lessons() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(rows.len(), 131);
    assert_eq!(
        developer_digest::hash(&serde_json::to_vec(&rows[..128]).unwrap()),
        "f773a60e9911fbffa38ae87a392c937579759bc7f50610e4b53d0595d0a247bd"
    );
    let leaf: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-vacation-lessons-v1.json")).unwrap();
    for (row, lab) in rows[128..130]
        .iter()
        .zip(leaf["scenarios"].as_array().unwrap())
    {
        assert_eq!(row["id"], lab["id"]);
        assert_eq!(row["concepts"].as_array().unwrap().len(), 4);
        assert_eq!(row["practice"].as_array().unwrap().len(), 3);
        assert_eq!(row["worked"].as_array().unwrap().len(), 3);
        assert_eq!(
            row["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" "),
            lab["objective"].as_str().unwrap()
        );
    }
}
