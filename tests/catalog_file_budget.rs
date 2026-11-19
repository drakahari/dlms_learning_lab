//! Exercise the actual small production guards without linking all library unit tests.
use dlms_learning_lab::pack;
#[path = "../src/pack/collection_budget.rs"]
mod budget;
#[test]
fn external_import_stays100_and_static_bundle_stops_at128_files() {
    let invalid = "not JSON";
    assert_eq!(pack::MAX_BUNDLED_PACK_FILES, 128);
    assert_eq!(
        pack::load_catalog(&[invalid; 101]).unwrap_err(),
        "Invalid pack collection size"
    );
    assert_eq!(
        budget::collection(&[invalid; 101], pack::MAX_BUNDLED_PACK_FILES),
        Ok(())
    );
    assert_eq!(
        budget::collection(&[invalid; 128], pack::MAX_BUNDLED_PACK_FILES),
        Ok(())
    );
    assert_eq!(
        budget::collection(&[invalid; 129], pack::MAX_BUNDLED_PACK_FILES).unwrap_err(),
        "Invalid pack collection size"
    );
    assert_eq!(
        budget::collection(&[], 128).unwrap_err(),
        "Invalid pack collection size"
    );
}
#[test]
fn both_paths_keep_the_same_total_and_per_file_byte_guards() {
    let one = "x".repeat(2_000_001);
    assert_eq!(budget::pack(&one).unwrap_err(), "Pack exceeds 2 MB");
    assert_eq!(
        pack::load_catalog(&[&one]).unwrap_err(),
        "Pack exceeds 2 MB"
    );
    let body = "x".repeat(2_000_000);
    assert_eq!(budget::pack(&body), Ok(()));
    for limit in [100, 128] {
        assert_eq!(budget::collection(&[body.as_str(); 16], limit), Ok(()));
        // Repeated references, not seventeen large allocations or a stress test.
        assert_eq!(
            budget::collection(&[body.as_str(); 17], limit).unwrap_err(),
            "Invalid pack collection size"
        );
    }
    assert_eq!(
        pack::load_catalog(&[body.as_str(); 17]).unwrap_err(),
        "Invalid pack collection size"
    );
}
#[test]
fn both_paths_keep_the_combined_thousand_scenario_guard_before_grading() {
    assert_eq!(budget::scenarios(800, 200), Ok(()));
    assert_eq!(
        budget::scenarios(800, 201).unwrap_err(),
        "Catalog exceeds 1,000 authored labs"
    );
    assert_eq!(
        budget::scenarios(usize::MAX, 1).unwrap_err(),
        "Catalog exceeds 1,000 authored labs"
    );
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    let text=serde_json::json!({"schema_version":1,"scenarios":vec![source["scenarios"][0].clone();201]}).to_string();
    assert!(text.len() < 2_000_000);
    assert_eq!(
        pack::load_catalog(&[&text, &text, &text, &text, &text]).unwrap_err(),
        "Catalog exceeds 1,000 authored labs"
    );
}
