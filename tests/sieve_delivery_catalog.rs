use dlms_learning_lab::{catalog, course, developer_digest, pack};
const IDS: [&str; 3] = [
    "lpic2-sieve-envelope-recipient",
    "lpic2-sieve-redirect-recovery",
    "lpic2-sieve-rejection-priority",
];
#[test]
fn all472_prior_scenarios_and187_unit_objects_stay_exact() {
    let before = pack::accepted472().unwrap();
    let now = pack::accepted475().unwrap();
    assert_eq!((before.len(), now.len()), (472, 475));
    assert_eq!(
        serde_json::to_value(&now[..472]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        now[472..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let old = course::accepted472(&before).unwrap();
    let units = course::accepted475(&now).unwrap();
    assert_eq!((old.len(), units.len()), (187, 188));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-sieve-envelope-delivery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(old).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-sieve-envelope-delivery")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at].prerequisites, ["linux-sieve-address-routing"]);
    assert!(
        units
            .iter()
            .position(|u| u.id == "linux-sieve-address-routing")
            .unwrap()
            < at
    );
    for (track, n) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 128),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
    }
    let filter = catalog::Filter {
        track: "LPIC-2".into(),
        search: "Sieve".into(),
        ..Default::default()
    };
    assert!(
        now[472..]
            .iter()
            .all(|s| catalog::matches_filters(s, &filter, false, false))
    );
}
#[test]
fn selected_pack_is_exact99_plus3_without_raising_source_or_catalog_bounds() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v48.json")).unwrap();
    let leaf: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-delivery-lessons-v1.json")).unwrap();
    let raw = include_str!("../scenarios/boot-entry-lessons-v49.json");
    let now: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = now["scenarios"].as_array().unwrap();
    assert_eq!(rows.len(), 102);
    assert_eq!(&rows[..99], old["scenarios"].as_array().unwrap());
    assert_eq!(&rows[99..], leaf["scenarios"].as_array().unwrap());
    assert!(raw.len() < 2_000_000);
    assert_eq!(pack::MAX_BUNDLED_PACK_FILES, 128);
    assert!(include_str!("../src/pack.rs").len() <= 512 * 1024);
}
#[test]
fn all125_prior_teaching_values_remain_exact_and_new_content_has_individual_requirements() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(rows.len(), 130);
    assert_eq!(
        developer_digest::hash(&serde_json::to_vec(&rows[..125]).unwrap()),
        "b704e1d04cbead5e543fd0b7c1db068df9a00cd950d00b28aa19e6c90f660c5e"
    );
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/sieve-delivery-lessons-v1.json")).unwrap();
    for (r, s) in rows[125..128]
        .iter()
        .zip(source["scenarios"].as_array().unwrap())
    {
        assert_eq!(r["id"], s["id"]);
        for part in ["concepts", "worked", "practice"] {
            assert_eq!(r[part].as_array().unwrap().len(), 3);
        }
        let req = r["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(req, s["objective"].as_str().unwrap());
        assert_ne!(
            s["goal"]["definition"]["assignment"]["reference"],
            s["goal"]["definition"]["example"]["reference"]
        );
    }
}
