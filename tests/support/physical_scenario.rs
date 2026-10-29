//! Technical composition fixture, not a registered curriculum scenario.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack, physical_link as core, physical_link_teaching as teaching,
};
#[path = "physical_assignment_fixture.rs"]
mod assignment;
pub fn scenario() -> pack::Scenario {
    let f = assignment::fixture("assignment");
    let example = assignment::fixture("example");
    let lesson=teaching::Lesson{version:1,introduction:"Technical native composition fixture. Inspect both simulated endpoints, edit the full configuration, Apply, then use bounded samples. The separate example never replaces assignment work.".into(),concepts:vec![teaching::Concept{title:"Inspect and retry".into(),text:"Compare operating mode and fresh receive changes. Old counters and reported notes never grant current success.".into()}],practice:example.definition,reference:example.reference,try_change:example.partial,repair:example.alternate,experiment_description:"Choose the other supplied cable, compare a new sample and repair both endpoints.".into()};
    // Retain a real supported alternative and independent negative fixture in
    // this helper; no generic context-free solution strings or dummy State.
    f.definition.validate_reference(&f.alternate).unwrap();
    assert!(!f.definition.solved(
        &core::State::new(f.definition.initial.topology().clone(), f.neighbor_broken).unwrap()
    ));
    let action = core::Action::Configure {
        configuration: f.reference,
    };
    let raw = serde_json::json!({"schema_version":1,"scenarios":[{"id":"test-physical-composition","version":1,"track":"CCNA","topic":"Physical technical fixture","unit":"","title":"Technical physical composition","summary":"Unregistered composition fixture","objective":"Repair the target while preserving operations; examples remain separate.","difficulty":"Beginner","minutes":12,"steps":["Use the native topology, draft and bounded samples"],"hints":["Compare both ends"],"sources":["https://www.cisco.com/c/en/us/support/docs/lan-switching/ethernet/10561-3.html"],"simplifications":"Supplied point-to-point profiles only; no host or IOS execution.","initial":State::PhysicalLink{simulation:f.definition.initial.clone()},"goal":Goal::PhysicalLink{definition:Box::new(f.definition)},"physical_link_teaching":lesson,"solution":[format!("{}{}",dlms_learning_lab::physical_link_workspace::PREFIX,serde_json::to_string(&action).unwrap())]}]});
    pack::load(&serde_json::to_string(&raw).unwrap())
        .unwrap()
        .remove(0)
}
