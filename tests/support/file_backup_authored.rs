//! Four distinct authored tasks; not registered until native/progress integration.
use dlms_learning_lab::pack::{self, Pack, Scenario};
use serde_json::Value;
use std::sync::OnceLock;
pub const DATA: &str = include_str!("../../scenarios/file-backup-v1.json");
const REVIEW: &str = include_str!("../fixtures/file-backup-content-review.json");
pub fn review() -> Value {
    serde_json::from_str(REVIEW).unwrap()
}
pub fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut current = pack::bundled().unwrap();
        let added: Pack = serde_json::from_str(DATA).unwrap();
        let present = added
            .scenarios
            .iter()
            .filter(|s| current.iter().any(|c| c.id == s.id))
            .count();
        match present {
            0 => {}
            4 => {
                for s in &added.scenarios {
                    let c = current.iter().find(|c| c.id == s.id).unwrap();
                    assert_eq!(
                        serde_json::to_value(c).unwrap(),
                        serde_json::to_value(s).unwrap()
                    );
                }
                current.retain(|s| !added.scenarios.iter().any(|a| a.id == s.id));
            }
            _ => panic!("All four backup tasks or none must be registered"),
        }
        // Keep each independent authored pack below the existing cap; never
        // serialize the entire growing catalog as one artificial large pack.
        let texts = current
            .chunks(16)
            .map(|batch| {
                serde_json::to_string(&Pack {
                    schema_version: 1,
                    scenarios: batch.to_vec(),
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert!(texts.iter().all(|text| text.len() <= pack::MAX_PACK_BYTES));
        let mut slices = texts.iter().map(String::as_str).collect::<Vec<_>>();
        slices.push(DATA);
        pack::load_catalog(&slices).unwrap()
    })
}
pub fn lab(id: &str) -> &'static Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
