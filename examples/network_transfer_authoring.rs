//! Fixed bundled declarative authoring data. No learner/path/program selector.
use dlms_learning_lab::{
    engine, network_transfer_source as source, network_transfer_workspace as work, pack,
};
fn main() {
    assert_eq!(
        std::env::args().len(),
        1,
        "Fixed authoring check accepts no arguments"
    );
    let author: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/authoring/network-transfers-v1.json"
    ))
    .unwrap();
    assert_eq!(author["schema_version"], 1);
    let mut scenarios = vec![];
    for lesson in author["lessons"].as_array().unwrap() {
        let d: work::Definition = serde_json::from_value(lesson["definition"].clone()).unwrap();
        d.validate().unwrap();
        let events: Vec<source::Event> =
            serde_json::from_value(lesson["assignment_events"].clone()).unwrap();
        let mut value = lesson["scenario"].clone();
        value["initial"] = serde_json::to_value(engine::State::NetworkTransfer {
            simulation: Box::new(d.new_session().unwrap()),
        })
        .unwrap();
        value["goal"] = serde_json::to_value(engine::Goal::NetworkTransfer {
            definition: Box::new(d),
        })
        .unwrap();
        value["solution"] = events
            .iter()
            .map(|e| format!("{}{}", source::PREFIX, serde_json::to_string(e).unwrap()))
            .collect::<Vec<_>>()
            .into();
        scenarios.push(value);
    }
    let result = serde_json::json!({"schema_version":1,"scenarios":scenarios});
    let raw = serde_json::to_string_pretty(&result).unwrap();
    let labs = pack::load(&raw).unwrap();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let mut state = s.initial.clone();
        for event in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, event).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
    }
    println!("{raw}");
}
