use dlms_learning_lab::{course, developer_digest, pack};
#[test]
fn retained_smtp_catalog_retains_every_prior_scenario_value_and_relative_order() {
    let before = pack::accepted464().unwrap();
    let current = pack::accepted467().unwrap();
    assert_eq!((before.len(), current.len()), (464, 467));
    assert_eq!(
        serde_json::to_value(&current[..464]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        current[464..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "lpic2-smtp-destination-role",
            "lpic2-smtp-relay-boundary",
            "lpic2-smtp-queue-recovery"
        ]
    );
    for (track, count) in [
        ("CCNA", 113),
        ("LPIC-1", 154),
        ("LPIC-2", 120),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
    let filter = dlms_learning_lab::catalog::Filter {
        search: "SMTP".into(),
        track: "LPIC-2".into(),
        ..Default::default()
    };
    assert!(
        current[464..]
            .iter()
            .all(|s| dlms_learning_lab::catalog::matches_filters(s, &filter, false, false))
    );
}
#[test]
fn prior_unit_objects_are_exact_and_smtp_follows_its_real_prerequisites() {
    let old = pack::accepted464().unwrap();
    let old_units = course::accepted464(&old).unwrap();
    let labs = pack::accepted467().unwrap();
    let units = course::accepted467(&labs).unwrap();
    assert_eq!((old_units.len(), units.len()), (184, 185));
    let remaining = units
        .iter()
        .filter(|u| u.id != "linux-smtp-routing")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(remaining).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-smtp-routing")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| u.id == *p).unwrap() < at);
    }
}
#[test]
fn selected_pack_is_exact_predecessor_plus_authored_mail_scenarios_under_the_existing_limit() {
    let before: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v45.json")).unwrap();
    let added: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/smtp-routing-lessons-v1.json")).unwrap();
    let raw = include_str!("../scenarios/boot-entry-lessons-v46.json");
    let selected: serde_json::Value = serde_json::from_str(raw).unwrap();
    assert!(raw.len() < 2_000_000);
    let rows = selected["scenarios"].as_array().unwrap();
    assert_eq!(rows.len(), 94);
    assert_eq!(&rows[..91], before["scenarios"].as_array().unwrap());
    assert_eq!(&rows[91..], added["scenarios"].as_array().unwrap());
}
#[test]
fn existing117_lpic2_presentation_values_match_the_retained464_baseline() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(rows.len(), 130);
    // Canonical value hash read from actual retained464 selected inputs, not later main.
    assert_eq!(
        developer_digest::hash(&serde_json::to_vec(&rows[..117]).unwrap()),
        "4ee752dde716e41099b81a56e38896bf9260b81d33eca568be2d97fd488f2afe"
    );
    for e in &rows[117..120] {
        assert_eq!(e["concepts"].as_array().unwrap().len(), 3);
        assert_eq!(e["practice"].as_array().unwrap().len(), 3);
        assert_eq!(e["worked"].as_array().unwrap().len(), 3);
        assert_eq!(e["requirements"].as_array().unwrap().len(), 4);
    }
}
