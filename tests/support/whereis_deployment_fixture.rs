//! Exact authored support-file exercise, scoped loader fixture (external prereqs omitted).
use dlms_learning_lab::{
    engine::{Goal, State},
    filesystem::whereis_deployment::Definition,
    pack,
};
pub fn value() -> serde_json::Value {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/whereis-lessons-v1.json")).unwrap();
    let mut s = raw["scenarios"][0].clone();
    s["prerequisites"] = serde_json::json!([]);
    s
}
pub fn scenario() -> pack::Scenario {
    pack::load(
        &serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[value()]}))
            .unwrap(),
    )
    .unwrap()
    .remove(0)
}
pub fn definition(s: &pack::Scenario) -> &Definition {
    let Goal::Filesystem { requirements } = &s.goal else {
        panic!()
    };
    let dlms_learning_lab::filesystem::Requirement::WhereisDeployment { definition } =
        &requirements[0]
    else {
        panic!()
    };
    definition
}
pub fn tree(s: &pack::Scenario) -> &dlms_learning_lab::filesystem::Tree {
    let State::Filesystem { tree } = &s.initial else {
        panic!()
    };
    tree
}
