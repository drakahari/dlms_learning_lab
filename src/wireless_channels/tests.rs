use super::*;
fn radio(id: &str, managed: bool, channel: u8) -> Radio {
    Radio {
        id: id.into(),
        label: format!("Access point {id}"),
        managed,
        initial_channel: channel,
    }
}
fn pair(a: &str, b: &str) -> Pair {
    Pair {
        a: a.into(),
        b: b.into(),
    }
}
fn triangle() -> Network {
    Network {
        radios: vec![
            radio("a", true, 1),
            radio("b", true, 1),
            radio("c", true, 6),
        ],
        surveys: vec![Survey {
            id: "busy-floor".into(),
            label: "Busy-floor survey".into(),
            neighbours: vec![pair("a", "b"), pair("b", "c"), pair("a", "c")],
        }],
    }
}
fn passed(n: &Network, p: &BTreeMap<String, u8>) -> bool {
    n.checks(p).unwrap().iter().all(|r| r.correct)
}
#[test]
fn same_different_and_separated_channels_are_distinguished_without_throughput_claims() {
    assert_eq!(relationship(1, 1).unwrap(), Relationship::SharedChannel);
    assert_eq!(relationship(1, 5).unwrap(), Relationship::AdjacentOverlap);
    assert_eq!(relationship(6, 10).unwrap(), Relationship::AdjacentOverlap);
    assert_eq!(relationship(1, 6).unwrap(), Relationship::Separated);
    assert_eq!(relationship(6, 11).unwrap(), Relationship::Separated);
    for (a, b) in [(0, 1), (1, 12), (255, 6)] {
        assert!(relationship(a, b).is_err());
    }
}
#[test]
fn all_six_equivalent_site_plans_pass_and_conflicting_permutations_fail() {
    let n = triangle();
    n.validate().unwrap();
    assert!(!passed(&n, &n.initial_plan()));
    let mut accepted = 0;
    for a in SITE_CHANNELS {
        for b in SITE_CHANNELS {
            for c in SITE_CHANNELS {
                let p = BTreeMap::from([("a".into(), a), ("b".into(), b), ("c".into(), c)]);
                let ok = passed(&n, &p);
                assert_eq!(ok, a != b && b != c && a != c);
                accepted += usize::from(ok);
            }
        }
    }
    assert_eq!(accepted, 6);
}
#[test]
fn second_supplied_survey_can_reject_a_plan_that_passes_the_first() {
    let mut n = triangle();
    n.surveys[0].neighbours = vec![pair("a", "b"), pair("b", "c")];
    let p = BTreeMap::from([("a".into(), 1), ("b".into(), 6), ("c".into(), 1)]);
    // Reuse between cells that do not hear each other is not blanket failure.
    assert!(passed(&n, &p));
    n.surveys.push(Survey {
        id: "open-partition".into(),
        label: "Partition-open survey".into(),
        neighbours: vec![pair("a", "c")],
    });
    let checks = n.checks(&p).unwrap();
    assert_eq!(checks.iter().filter(|r| !r.correct).count(), 1);
    assert!(
        checks
            .iter()
            .find(|r| !r.correct)
            .unwrap()
            .label
            .contains("Partition-open")
    );
    assert!(
        checks
            .iter()
            .find(|r| !r.correct)
            .unwrap()
            .feedback
            .contains("share airtime")
    );
}
#[test]
fn protected_neighbours_and_site_policy_cannot_be_bypassed_by_new_numbers() {
    let mut n = triangle();
    n.radios[0].managed = false;
    let mut p = BTreeMap::from([("a".into(), 1), ("b".into(), 6), ("c".into(), 11)]);
    assert!(passed(&n, &p));
    p.insert("a".into(), 11);
    p.insert("c".into(), 1);
    assert!(!passed(&n, &p));
    assert!(
        n.checks(&p)
            .unwrap()
            .iter()
            .any(|r| !r.correct && r.feedback.contains("change authority"))
    );
    p.insert("a".into(), 1);
    p.insert("b".into(), 7);
    p.insert("c".into(), 11);
    assert!(!passed(&n, &p));
    assert!(
        n.checks(&p)
            .unwrap()
            .iter()
            .any(|r| !r.correct && r.feedback.contains("site's 1 / 6 / 11"))
    );
}
#[test]
fn incomplete_foreign_and_out_of_range_plans_are_not_silently_graded_as_success() {
    let n = triangle();
    let mut p = n.initial_plan();
    p.remove("b");
    assert!(n.checks(&p).is_err());
    p.insert("other".into(), 6);
    assert!(n.checks(&p).is_err());
    p.remove("other");
    p.insert("b".into(), 0);
    assert!(n.checks(&p).is_err());
    p.insert("b".into(), 12);
    assert!(n.checks(&p).is_err());
}
#[test]
fn malformed_duplicate_or_unbounded_surveys_and_useless_managed_radios_refuse() {
    let mut n = triangle();
    n.surveys[0].neighbours.push(pair("b", "a"));
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.surveys[0].neighbours[0] = pair("a", "a");
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.surveys[0].neighbours[0] = pair("a", "unknown");
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.radios[0].id = "../../home".into();
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.radios.push(radio("unused", true, 1));
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.surveys = vec![n.surveys[0].clone(); MAX_SURVEYS + 1];
    assert!(n.validate().is_err());
    let mut n = triangle();
    n.radios[0].managed = false;
    n.radios[1].managed = false;
    assert!(n.validate().is_err());
}

#[test]
fn bounded_deserialization_rejects_oversized_or_unknown_source_fields() {
    let source = triangle();
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["radios"] = serde_json::json!(vec![source.radios[0].clone(); MAX_RADIOS + 1]);
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["radios"][0]["label"] = serde_json::json!("x".repeat(121));
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["surveys"][0]["neighbours"] = serde_json::json!(vec![pair("a", "b"); 29]);
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["host_scan"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Network>(raw).is_err());
    for bad in ["", "-", "A", "a/../../home", "a b"] {
        let mut n = source.clone();
        n.radios[0].id = bad.into();
        assert!(n.validate().is_err());
    }
}
#[test]
fn initially_nonstandard_but_supported_channel_is_a_diagnosable_state_not_a_fake_success() {
    let mut n = triangle();
    n.radios[0].initial_channel = 2;
    n.validate().unwrap();
    let rows = n.checks(&n.initial_plan()).unwrap();
    assert!(
        rows.iter()
            .any(|r| !r.correct && r.feedback.contains("possible 2.4 GHz channel"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.correct && r.feedback.contains("Different channel numbers"))
    );
    let ids = rows.iter().map(|r| &r.id).collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), rows.len());
}
