use dlms_learning_lab::{catalog, course, developer_digest, pack};
const IDS: [&str; 2] = [
    "lpic2-sieve-sender-domain",
    "lpic2-sieve-recipient-recovery",
];
#[test]
fn normal_catalog_preserves_all470_values_and_adds_only_two_distinct_address_tasks() {
    let before = pack::accepted470().unwrap();
    let current = pack::accepted472().unwrap();
    assert_eq!((before.len(), current.len()), (470, 472));
    assert_eq!(
        serde_json::to_value(&current[..470]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        current[470..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    for (track, n) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 125),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), n);
    }
    let filter = catalog::Filter {
        track: "LPIC-2".into(),
        search: "Sieve".into(),
        ..Default::default()
    };
    assert!(
        current[470..]
            .iter()
            .all(|s| catalog::matches_filters(s, &filter, false, false))
    );
}
#[test]
fn normal_units_keep_all186_predecessors_and_place_addressing_after_filter_fundamentals() {
    let old = pack::accepted470().unwrap();
    let before = course::accepted470(&old).unwrap();
    let current = pack::accepted472().unwrap();
    let units = course::accepted472(&current).unwrap();
    assert_eq!((before.len(), units.len()), (186, 187));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-sieve-address-routing")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-sieve-address-routing")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at].prerequisites, ["linux-sieve-filters"]);
    assert!(
        units
            .iter()
            .position(|u| u.id == "linux-sieve-filters")
            .unwrap()
            < at
    );
}
#[test]
fn selected_pack_is_exact97_predecessors_and_two_additions_with_original_caps() {
    let before: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v47.json")).unwrap();
    let added: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-address-lessons-v1.json")).unwrap();
    let raw = include_str!("../scenarios/boot-entry-lessons-v48.json");
    let selected: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = selected["scenarios"].as_array().unwrap();
    assert_eq!(rows.len(), 99);
    assert_eq!(&rows[..97], before["scenarios"].as_array().unwrap());
    assert_eq!(&rows[97..], added["scenarios"].as_array().unwrap());
    assert!(raw.len() < 2_000_000);
    assert_eq!(pack::MAX_BUNDLED_PACK_FILES, 128);
}
#[test]
fn all123_teaching_records_match_the_actual_retained470_input_and_new_views_are_individual() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(rows.len(), 130);
    assert_eq!(
        developer_digest::hash(&serde_json::to_vec(&rows[..123]).unwrap()),
        "468215f850896a7a4a7b98ff527d57ba0955aeee151460e00f9d80a675a9920a"
    );
    let authored: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-address-lessons-v1.json")).unwrap();
    for (row, source) in rows[123..]
        .iter()
        .zip(authored["scenarios"].as_array().unwrap())
    {
        assert_eq!(row["id"], source["id"]);
        assert_eq!(row["status"], "updated");
        assert_eq!(
            row["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" "),
            source["objective"]
        );
        for key in ["concepts", "practice", "worked"] {
            assert_eq!(row[key].as_array().unwrap().len(), 3);
        }
        assert!(
            source["goal"]["definition"]["assignment"]["address_tests"]
                .as_bool()
                .unwrap()
        );
    }
}
