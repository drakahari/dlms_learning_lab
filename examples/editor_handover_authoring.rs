//! Fixed declarative authoring input; never reads learner text or a host path.
use dlms_learning_lab::{editor_handover as handover, editor_handover_source::Event, engine, pack};
fn main() {
    assert_eq!(
        std::env::args().len(),
        1,
        "This fixed authoring check accepts no arguments"
    );
    let author: serde_json::Value = serde_json::from_str(include_str!(
        "../scenarios/authoring/editor-handover-v1.json"
    ))
    .unwrap();
    assert_eq!(author["schema_version"], 1);
    let mut scenarios = vec![];
    for lesson in author["lessons"].as_array().unwrap() {
        let d: handover::Definition = serde_json::from_value(lesson["definition"].clone()).unwrap();
        d.validate_example().unwrap();
        let events: Vec<Event> =
            serde_json::from_value(lesson["assignment_events"].clone()).unwrap();
        let mut value = lesson["scenario"].clone();
        value["initial"] = serde_json::to_value(engine::State::EditorHandover {
            simulation: Box::new(d.new_session().unwrap()),
        })
        .unwrap();
        value["goal"] = serde_json::to_value(engine::Goal::EditorHandover {
            definition: Box::new(d),
        })
        .unwrap();
        value["solution"] = events
            .iter()
            .map(|e| format!("editor-handover {}", serde_json::to_string(e).unwrap()))
            .collect::<Vec<_>>()
            .into();
        scenarios.push(value);
    }
    let result = serde_json::json!({"schema_version":1,"scenarios":scenarios});
    let raw = serde_json::to_string_pretty(&result).unwrap();
    let loaded = pack::load(&raw).unwrap();
    assert_eq!(loaded.len(), 1);
    for s in loaded {
        let mut state = s.initial.clone();
        for event in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, event).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{:?}",
            engine::checks(&state, &s.goal)
        );
    }
    println!("{raw}");
}
