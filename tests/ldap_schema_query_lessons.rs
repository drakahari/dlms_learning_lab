use dlms_learning_lab::{
    course,
    engine::{self, State},
    ldap_schema_query_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
const ID: &str = "lpic2-ldap-schema-consumer";
#[test]
fn all371_prior_labs_and136_units_remain_exact_with_consumer_after_schema() {
    let current = catalog();
    let old = pack::accepted371().unwrap();
    assert_eq!(current.len(), 376);
    assert_eq!(old.len(), 371);
    assert_eq!(
        serde_json::to_value(&current[..371]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(current).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted371(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-schema-consumers")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-schema-extension");
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(units[at].prerequisites, ["ldap-schema-extension"]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [89, 120, 90, 77]
    );
}
#[test]
fn registered_actual_behavior_and_every_displayed_worked_value_are_current() {
    let s = catalog().iter().find(|s| s.id == ID).unwrap();
    let d = w::source(s).unwrap();
    let fixture: w::Definition =
        serde_json::from_str(include_str!("fixtures/ldap_schema_query_authored.json")).unwrap();
    assert_eq!(d, &fixture);
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let State::LdapSchemaQuery { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit_filter("(accessState=*)".into())
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == ID)
        .unwrap();
    let worked = row["worked"].as_array().unwrap();
    assert_eq!(worked[1]["value"], d.example.reference_schema);
    assert_eq!(worked[2]["value"], d.example.reference_filter);
    assert!(
        d.example
            .reports(
                worked[1]["value"].as_str().unwrap(),
                worked[2]["value"].as_str().unwrap()
            )
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert_ne!(d.initial.starter_schema, d.initial.reference_schema);
    assert_ne!(d.initial.starter_filter, d.initial.reference_filter);
    assert_eq!(
        row["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" "),
        s.objective
    );
}
