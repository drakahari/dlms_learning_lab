use dlms_learning_lab::pxe::session::{Definition, Session};

#[test]
fn original_network_boot_tasks_use_independent_runtime_workspaces() {
    let definitions: Vec<Definition> =
        serde_json::from_str(include_str!("fixtures/pxe_tasks_v1.json")).unwrap();
    assert_eq!(definitions.len(), 2);
    let mut models = vec![];
    for definition in definitions {
        let session = Session::new(&definition).unwrap();
        session.validate_shape().unwrap();
        let mut trial = session.clone();
        for action in &definition.example.reference {
            trial.apply(&definition, true, action).unwrap();
        }
        assert!(definition.example.solved(&trial.example.work));
        assert_eq!(trial.assignment, session.assignment);
        assert!(!definition.assignment.solved(&trial.assignment.work));
        models.push(serde_json::json!({"definition": definition, "simulation": session}));
    }
    println!(
        "PXE_AUTHORED_MODELS={}",
        serde_json::to_string(&models).unwrap()
    );
}
