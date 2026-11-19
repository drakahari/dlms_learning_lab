use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    ldap_schema_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic2-ldap-schema-extension";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn source() -> &'static pack::Scenario {
    catalog()
        .iter()
        .find(|s| s.id == ID)
        .expect("Schema activity must be registered, never a synthetic fallback")
}
#[test]
fn all370_scenarios_and135_units_remain_exact_and_schema_follows_configuration() {
    let current = catalog();
    let old = pack::accepted370().unwrap();
    assert_eq!(current.len(), 376);
    assert_eq!(old.len(), 370);
    assert_eq!(
        serde_json::to_value(&current[..370]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(current).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted370(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-schema-extension")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-config-deployment");
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(units[at].prerequisites, ["ldap-config-deployment"]);
    assert_eq!(source().prerequisites, ["lpic2-ldap-config-deployment"]);
    let totals = ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
        .map(|track| current.iter().filter(|s| s.track == track).count());
    assert_eq!(totals, [89, 120, 90, 77]);
}
#[test]
fn registered_goal_uses_authored_definition_and_state_changes_with_current_invalidation() {
    let s = source();
    let d = w::source(s).unwrap();
    let expected: w::Definition =
        serde_json::from_str(include_str!("fixtures/ldap_schema_authored.json")).unwrap();
    assert_eq!(d, &expected);
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let State::LdapSchema { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.run(&d.initial).unwrap();
    let output = simulation.assignment.output.as_ref().unwrap();
    assert!(output.accepted && output.passed);
    assert_ne!(output.before, output.after);
    assert_eq!(output.modules.len(), 1);
    simulation
        .assignment
        .edit(d.initial.starter.clone())
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    let State::LdapSchema { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.check(&d.initial).unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(engine::checks(&a.state, &s.goal).iter().all(|r| !r.0));
}
#[test]
fn displayed_folded_workshop_code_reaches_actual_practice_and_cannot_solve_assignment() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == ID)
        .unwrap();
    let worked = row["worked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["label"] == "Worked workshop schema Add")
        .unwrap()["value"]
        .as_str()
        .unwrap();
    assert!(worked.contains("\n  EQUALITY"));
    let Goal::LdapSchema { definition } = &source().goal else {
        panic!()
    };
    assert!(
        definition
            .example
            .reports(worked)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert!(definition.initial.reports(worked).is_err());
    assert!(row["concepts"].as_array().unwrap().iter().any(|r| {
        r["text"]
            .as_str()
            .unwrap()
            .contains("MUST does not itself limit")
    }));
}
