//! Independent complete190 reconstruction; cold preservation tests only.
use dlms_learning_lab::{
    developer_digest as digest,
    pack::{Pack, Scenario},
};
use serde_json::Value;
#[path = "ntp_original_catalog.rs"]
pub mod before_ntp;
pub fn load() -> Vec<Scenario> {
    let raw = include_bytes!("../fixtures/catalog193-registration.json");
    assert_eq!(
        digest::hash(raw),
        "c26dec73f0727ce2ee24d76b991204e3114be028be41cd562397714b7c1ced55"
    );
    let recipe: Value = serde_json::from_slice(raw).unwrap();
    let mut all = before_ntp::load();
    assert_eq!(all.len(), 186);
    let bytes = include_bytes!("../../scenarios/ntp-client-server-v1.json");
    let pin = recipe["before_packs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "scenarios/ntp-client-server-v1.json")
        .unwrap();
    assert_eq!(bytes.len() as u64, pin["bytes"].as_u64().unwrap());
    assert_eq!(digest::hash(bytes), pin["sha256"].as_str().unwrap());
    let pack: Pack = serde_json::from_slice(bytes).unwrap();
    assert_eq!(pack.scenarios.len(), 4);
    all.extend(pack.scenarios);
    assert_eq!(all.len(), 190);
    assert_eq!(
        all.iter()
            .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
            .collect::<Vec<_>>(),
        *recipe["before_ids"].as_array().unwrap()
    );
    all
}
