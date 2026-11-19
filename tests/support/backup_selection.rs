//! Exact single authored selection task; no generated variants or learner answers.
use dlms_learning_lab::{
    engine::{self, State},
    pack::{self, Scenario},
};
use serde_json::Value;
pub const ID: &str = "lpic2-backup-directory-selection";
pub fn review() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/file-backup-selection-content-review.json"
    ))
    .unwrap()
}
pub fn source() -> Scenario {
    // Validate the genuine prerequisite packs too, rather than removing the
    // prerequisite merely to admit an unregistered one-scenario fixture.
    let all = pack::load_catalog(&[
        include_str!("../../scenarios/starter-v1.json"),
        include_str!("../../scenarios/linux-files-v1.json"),
        include_str!("../../scenarios/file-backup-v1.json"),
        include_str!("../../scenarios/file-backup-selection-v1.json"),
    ])
    .unwrap();
    let canonical: pack::Pack = serde_json::from_str(include_str!(
        "../../scenarios/file-backup-selection-v1.json"
    ))
    .unwrap();
    assert_eq!(canonical.scenarios.len(), 1);
    let s = all.iter().find(|s| s.id == ID).unwrap();
    assert_eq!(
        serde_json::to_value(s).unwrap(),
        serde_json::to_value(&canonical.scenarios[0]).unwrap()
    );
    s.clone()
}

pub fn commands(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_owned())
        .collect()
}
pub fn flags(state: &State, s: &Scenario) -> Vec<bool> {
    engine::checks(state, &s.goal).iter().map(|r| r.0).collect()
}
