use dlms_learning_lab::{
    engine::{self, Goal, State},
    ldap_schema_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
#[path = "support/ldap_schema_task.rs"]
mod fixture;
fn source() -> pack::Scenario {
    let mut s = pack::ccna_pilot()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-ldap-config-deployment")
        .unwrap();
    let mut d = fixture::fixture();
    d.initial.id = "schema-binding-test".into();
    s.id = d.initial.id.clone();
    s.objective = d.initial.instructions.clone();
    s.initial = State::LdapSchema {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    s.solution = vec![format!(
        "{}{}",
        w::PREFIX,
        serde_json::json!({"schema":d.initial.reference})
    )];
    s.goal = Goal::LdapSchema {
        definition: Box::new(d),
    };
    s
}
#[test]
fn engine_progress_and_source_binding_regrade_actual_assignment_only() {
    let s = source();
    let d = w::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    w::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, &s));
    assert_eq!(engine::checks(&a.state, &s.goal).len(), 3);
    let State::LdapSchema { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.starter.clone())
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
    let mut restored: Attempt = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
    assert!(!progress::current_solved(&restored, &s));
    assert!(engine::execute_for_goal(&mut restored.state, &s.goal, "sudo sh -c anything").is_err());
    assert!(!progress::current_solved(&restored, &s));
}
#[test]
fn mismatched_state_goal_context_or_version_cannot_borrow_schema_credit() {
    let s = source();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    a.scenario_version += 1;
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut s = source();
    s.objective = "different requirement".into();
    assert!(w::source(&s).is_err());
    let mut s = source();
    s.id = "different-id".into();
    assert!(w::source(&s).is_err());
}
