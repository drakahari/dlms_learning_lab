use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    ldap_access_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn source() -> &'static pack::Scenario {
    catalog()
        .iter()
        .find(|s| s.id == "lpic2-ldap-contact-delegation")
        .unwrap()
}
#[test]
fn all368_prior_scenarios_and133_units_remain_exact_with_delegation_after_protected_identities() {
    let current = catalog();
    let old = pack::accepted368().unwrap();
    assert_eq!(current.len(), 376);
    assert_eq!(old.len(), 368);
    assert_eq!(
        serde_json::to_value(&current[..368]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(current).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-access-policy" && u.id != "ldap-config-deployment" && u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted368(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-access-policy")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-protected-passwords");
    assert_eq!(units[at].labs, ["lpic2-ldap-contact-delegation"]);
    assert_eq!(units[at].prerequisites, ["ldap-protected-passwords"]);
    assert_eq!(source().prerequisites, ["lpic2-ldap-protected-password"]);
}
#[test]
fn registered_reference_alternates_partial_and_broken_policy_use_the_actual_authored_goal() {
    let s = source();
    let d = w::source(s).unwrap();
    let fixture: w::Definition =
        serde_json::from_str(include_str!("fixtures/ldap_access_authored.json")).unwrap();
    assert_eq!(d, &fixture);
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let partial = d.initial.reference.replace("dn.subtree=", "dn.one=");
    let State::LdapAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.edit(partial).unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    let rows = engine::checks(&a.state, &s.goal);
    assert!(rows[0].0 && rows[1].0 && !rows[2].0);
    assert!(!progress::current_solved(&a, s));
    let State::LdapAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(d.initial.reference.replace("by * none", "by * read"))
        .unwrap();
    simulation.assignment.check(&d.initial).unwrap();
    assert!(progress::current_solved(&a, s));
    let State::LdapAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("access to * by users write by * none".into())
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    let Goal::LdapAccess { definition } = &s.goal else {
        panic!()
    };
    assert_ne!(definition.initial.reference, definition.example.reference);
}
