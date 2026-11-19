use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    ldap_access_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic2-ldap-config-deployment";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn source() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
#[test]
fn all369_scenarios_and134_units_remain_exact_with_config_deployment_after_static_policy() {
    let current = catalog();
    let old = pack::accepted369().unwrap();
    assert_eq!(current.len(), 376);
    assert_eq!(old.len(), 369);
    assert_eq!(
        serde_json::to_value(&current[..369]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(current).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-config-deployment" && u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted369(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-config-deployment")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-access-policy");
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(units[at].prerequisites, ["ldap-access-policy"]);
    assert_eq!(source().prerequisites, ["lpic2-ldap-contact-delegation"]);
}
#[test]
fn main_catalog_uses_the_real_authored_config_goal_with_partial_cases_and_current_invalidation() {
    let source = source();
    let d = w::source(source).unwrap();
    let fixture: w::Definition =
        serde_json::from_str(include_str!("fixtures/ldap_configuration_authored.json")).unwrap();
    assert_eq!(d, &fixture);
    let mut attempt = Attempt::new(source, 1);
    workspace::prepare(&mut attempt);
    engine::execute_for_goal(&mut attempt.state, &source.goal, &source.solution[0]).unwrap();
    assert!(progress::current_solved(&attempt, source));
    let Goal::LdapAccess { definition } = &source.goal else {
        panic!()
    };
    assert!(definition.initial.configuration.is_some());
    let State::LdapAccess { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.reference.replace("dn.subtree=", "dn.one="))
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    let rows = engine::checks(&attempt.state, &source.goal);
    assert!(rows[0].0 && rows[1].0 && !rows[2].0);
    assert!(!progress::current_solved(&attempt, source));
    let State::LdapAccess { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.reference.clone())
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    assert!(progress::current_solved(&attempt, source));
    let State::LdapAccess { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.starter.clone())
        .unwrap();
    assert!(!progress::current_solved(&attempt, source));
}
