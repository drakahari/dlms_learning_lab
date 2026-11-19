//! Fixed declarative pack materialization, never a learner execution bridge.
//! Emits only the three source-owned modal lessons; accepts no paths or scripts.
use dlms_learning_lab::modal_workspace::{Definition, Event};
fn main() {
    assert_eq!(std::env::args().count(), 1, "No source/path selector");
    let raw = include_str!("../scenarios/authoring/modal-editing-v1.json");
    assert!(raw.len() < 200_000);
    let source: serde_json::Value = serde_json::from_str(raw).unwrap();
    assert_eq!(source["schema_version"], 1);
    let authored = source["lessons"].as_array().unwrap();
    assert_eq!(authored.len(), 3);
    let mut scenarios = vec![];
    for entry in authored {
        let d: Definition = serde_json::from_value(entry["definition"].clone()).unwrap();
        d.validate().unwrap();
        let mut session = d.new_session().unwrap();
        let initial = serde_json::to_value(&session).unwrap();
        let events: Vec<Event> = serde_json::from_value(entry["solution"].clone()).unwrap();
        assert!(events.len() <= 512);
        let mut solution = vec![];
        for event in events {
            session.apply(&d, false, &event).unwrap();
            solution.push(format!(
                "modal-lab {}",
                serde_json::to_string(&event).unwrap()
            ));
        }
        assert!(session.solved(&d), "{} taught assignment solution", d.id);
        let mut scenario = entry["scenario"].clone();
        assert_eq!(scenario["id"], d.id);
        scenario["initial"] = serde_json::json!({"kind":"ModalEditing","simulation":initial});
        scenario["goal"] = serde_json::json!({"kind":"ModalEditing","definition":d});
        scenario["solution"] = serde_json::json!(solution);
        scenarios.push(scenario);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"schema_version":1,"scenarios":scenarios})
        )
        .unwrap()
    );
}
