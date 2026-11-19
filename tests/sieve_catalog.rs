use dlms_learning_lab::{catalog, course, developer_digest, pack};
#[test]
fn current_catalog_retains_all467_scenario_values_and_adds_only_the_authored_filter_unit() {
    let before = pack::accepted467().unwrap();
    let current = pack::accepted470().unwrap();
    assert_eq!((before.len(), current.len()), (467, 470));
    assert_eq!(
        serde_json::to_value(&current[..467]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        current[467..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "lpic2-sieve-header-filing",
            "lpic2-sieve-compound-filter",
            "lpic2-sieve-rule-order"
        ]
    );
    for (track, count) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 123),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
    let filter = catalog::Filter {
        track: "LPIC-2".into(),
        search: "Sieve".into(),
        ..Default::default()
    };
    assert!(
        current[467..]
            .iter()
            .all(|s| catalog::matches_filters(s, &filter, false, false))
    );
}
#[test]
fn unit_order_retains_all185_objects_and_places_filtering_after_mail_foundations() {
    let before = pack::accepted467().unwrap();
    let units_before = course::accepted467(&before).unwrap();
    let current = pack::accepted470().unwrap();
    let units = course::accepted470(&current).unwrap();
    assert_eq!((units_before.len(), units.len()), (185, 186));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-sieve-filters")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(units_before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-sieve-filters")
        .unwrap();
    assert_eq!(
        units[at].labs,
        vec![
            "lpic2-sieve-header-filing",
            "lpic2-sieve-compound-filter",
            "lpic2-sieve-rule-order"
        ]
    );
    assert_eq!(units[at].prerequisites.len(), 2);
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| u.id == *p).unwrap() < at);
    }
}
#[test]
fn literal_consolidation_is_the_exact94_predecessors_plus_three_real_lessons_with_unchanged_caps() {
    let before: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v46.json")).unwrap();
    let added: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-filter-lessons-v1.json")).unwrap();
    let raw = include_str!("../scenarios/boot-entry-lessons-v47.json");
    let combined: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = combined["scenarios"].as_array().unwrap();
    assert_eq!(rows.len(), 97);
    assert_eq!(&rows[..94], before["scenarios"].as_array().unwrap());
    assert_eq!(&rows[94..], added["scenarios"].as_array().unwrap());
    assert!(raw.len() < 2_000_000);
    assert_eq!(pack::MAX_BUNDLED_PACK_FILES, 128);
}
#[test]
fn all120_existing_lpic2_teaching_values_match_the_actual_retained_smtp467_input() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(rows.len(), 130);
    assert_eq!(
        developer_digest::hash(&serde_json::to_vec(&rows[..120]).unwrap()),
        "e82212f71ac743fa961182858613a9fc195c51c86776a0de19e13133bd1b39fb"
    );
    for row in &rows[120..123] {
        assert_eq!(row["concepts"].as_array().unwrap().len(), 3);
        assert_eq!(row["practice"].as_array().unwrap().len(), 3);
        assert_eq!(row["requirements"].as_array().unwrap().len(), 3);
        assert_eq!(row["status"], "updated");
    }
}
