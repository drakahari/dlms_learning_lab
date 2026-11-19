//! Current registration and exact historical source preservation, not new mastery.
use dlms_learning_lab::{course, engine::State, pack};
#[test]
fn current_syslog_unit_appends_after_every_accepted213_source_and_unit() {
    let old = pack::accepted213().unwrap();
    let current = pack::accepted221().unwrap();
    assert_eq!(old.len(), 213);
    assert_eq!(current.len(), 221);
    assert_eq!(
        serde_json::to_value(&current[..213]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let old_units = course::accepted213(&old).unwrap();
    let units = course::accepted221(&current).unwrap();
    assert_eq!(units.len(), 59);
    assert_eq!(old_units.len(), 57);
    assert_eq!(
        serde_json::to_value(&units[..57]).unwrap(),
        serde_json::to_value(&old_units).unwrap()
    );
    let authored = serde_json::from_str::<pack::Pack>(include_str!("../scenarios/syslog-policy-v1.json")).unwrap().scenarios;
    assert_eq!(
        serde_json::to_value(&current[213..217]).unwrap(),
        serde_json::to_value(&authored).unwrap()
    );
    assert!(
        current[213..217]
            .iter()
            .all(|s| matches!(s.initial, State::Syslog { .. }))
    );
    assert_eq!(
        units.get(57).unwrap().prerequisites,
        vec!["ipv4-foundations"]
    );
    assert_eq!(
        units.get(57).unwrap().labs,
        current[213..217]
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn syslog_nested_selector_keys_roundtrip_without_admitting_text_scalar_facilities() {
    let s = pack::accepted221().unwrap().into_iter().find(|s|s.id=="ccna-syslog-local-remote").unwrap();
    let value = serde_json::to_value(&s.initial).unwrap();
    let restored: State = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), value);
    assert!(serde_json::from_str::<dlms_learning_lab::syslog::Facility>("\"23\"").is_err());
    assert!(
        serde_json::from_str::<dlms_learning_lab::syslog::CollectorPolicy>(
            r#"{"selectors":{"23":"warning","023":"error"}}"#
        )
        .is_err()
    );
}
