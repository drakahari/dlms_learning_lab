use dlms_learning_lab::{
    engine::{self, Goal, State},
    ldap_schema_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
#[path = "support/ldap_schema_task.rs"]
mod fixture;
fn definition() -> w::Definition {
    serde_json::from_str(include_str!("fixtures/ldap_schema_authored.json")).unwrap()
}
#[test]
fn authored_workshop_and_assignment_are_reachable_without_hidden_solution_matching() {
    let d = definition();
    d.validate().unwrap();
    let mut f = fixture::fixture();
    f.initial.id = d.initial.id.clone();
    f.initial.instructions = d.initial.instructions.clone();
    f.example.id = d.example.id.clone();
    f.example.instructions = d.example.instructions.clone();
    assert_eq!(d, f);
    for t in [&d.initial, &d.example] {
        assert_eq!(t.cases.len(), 3);
        assert!(t.reports(&t.reference).unwrap().iter().all(|r| r.passed));
        assert!(t.reports(&t.starter).unwrap().iter().any(|r| !r.passed));
        let alternate = t
            .reference
            .replace("SUP person", "SUP 2.5.6.6")
            .replace("caseExactMatch", "2.5.13.5")
            .replace("integerMatch", "2.5.13.14");
        assert!(t.reports(&alternate).unwrap().iter().all(|r| r.passed));
    }
}
#[test]
fn authored_assignment_and_example_keep_independent_data_and_contexts() {
    let d = definition();
    assert_ne!(d.initial.schema_name, d.example.schema_name);
    assert_ne!(d.initial.oid_branch, d.example.oid_branch);
    assert_ne!(d.initial.cases, d.example.cases);
    let mut session = w::Session::new(&d).unwrap();
    session.practice.edit(d.example.reference.clone()).unwrap();
    session.practice.check(&d.example).unwrap();
    assert!(session.practice.checks(&d.example).iter().all(|r| r.0));
    assert!(d.checks(&session).iter().all(|r| !r.0));
}
#[test]
fn authored_source_identity_and_engine_solution_are_actual_not_a_success_string() {
    let d = definition();
    let mut source = pack::ccna_pilot()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-ldap-config-deployment")
        .unwrap();
    source.id = d.initial.id.clone();
    source.objective = d.initial.instructions.clone();
    source.initial = State::LdapSchema {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    source.goal = Goal::LdapSchema {
        definition: Box::new(d.clone()),
    };
    w::source(&source).unwrap();
    let mut a = Attempt::new(&source, 1);
    workspace::prepare(&mut a);
    let raw = format!(
        "{}{}",
        w::PREFIX,
        serde_json::json!({"schema":d.initial.reference})
    );
    engine::execute_for_goal(&mut a.state, &source.goal, &raw).unwrap();
    assert!(progress::current_solved(&a, &source));
    let State::LdapSchema { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.starter.clone())
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    assert!(!progress::current_solved(&a, &source));
}
