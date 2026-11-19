//! Registered schema-extension engine/progress checks; native acceptance recorded separately.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    ldap_schema_query_workspace as w,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<Scenario> {
    static C: OnceLock<Vec<Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        let c = pack::ccna_pilot().unwrap();
        assert_eq!(c.len(), 376);
        let actual = c
            .iter()
            .find(|s| s.id == "lpic2-ldap-schema-consumer")
            .expect("New activity must be registered, never replaced by a synthetic fallback");
        let Goal::LdapSchemaQuery { definition } = &actual.goal else {
            panic!()
        };
        let expected: w::Definition =
            serde_json::from_str(include_str!("fixtures/ldap_schema_query_authored.json")).unwrap();
        assert_eq!(**definition, expected);
        c
    })
}
fn source() -> &'static Scenario {
    catalog()
        .iter()
        .find(|s| s.id == "lpic2-ldap-schema-consumer")
        .unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(source(), 1);
    workspace::prepare(&mut a);
    w::context(&a, source()).unwrap();
    a
}
#[test]
fn registered_engine_goal_grades_actual_entry_outcomes_and_clears_success_when_broken() {
    let s = source();
    let mut a = fresh();
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    assert_eq!(engine::checks(&a.state, &s.goal).len(), 4);
    let State::LdapSchemaQuery { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .assignment
        .edit_schema("version: 1".into())
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "sudo sh -c anything").is_err());
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn forged_metadata_foreign_context_and_unrelated_workspace_content_are_rejected() {
    let s = source();
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let mut wrong = a.clone();
    wrong.scenario_version += 1;
    assert!(w::context(&wrong, s).is_err());
    let mut wrong = a.clone();
    let State::LdapSchemaQuery { simulation } = &mut wrong.state else {
        panic!()
    };
    simulation.context = "foreign-context".into();
    assert!(w::context(&wrong, s).is_err());
    let mut wrong = a.clone();
    wrong.ipv4 = Some(dlms_learning_lab::ipv4::Lesson::from_legacy(
        "192.168.10.77".parse().unwrap(),
        27,
        0,
    ));
    assert!(w::context(&wrong, s).is_err());
    let mut wrong = a.clone();
    wrong.workspace.as_mut().unwrap().command = "unrelated workspace draft".into();
    assert!(w::context(&wrong, s).is_err());
    let mut wrong = a;
    let State::LdapSchemaQuery { simulation } = &mut wrong.state else {
        panic!()
    };
    simulation.assignment.schema = "version: 1".into();
    simulation.assignment.checked_schema = simulation.assignment.schema.clone();
    assert!(!progress::current_solved(&wrong, s));
}
#[test]
fn actual_private_save_backup_restore_preserves_pending_drafts_and_prior_configuration_work() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let s = source();
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let Goal::LdapSchemaQuery { definition } = &s.goal else {
        panic!()
    };
    let State::LdapSchemaQuery { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .practice
        .edit_schema(definition.example.reference_schema.clone())
        .unwrap();
    simulation
        .practice
        .edit_filter(definition.example.reference_filter.clone())
        .unwrap();
    simulation.practice.run(&definition.example).unwrap();
    simulation
        .assignment
        .select(&definition.initial, 2)
        .unwrap();
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic2-ldap-config-deployment")
        .unwrap();
    let mut prior = Attempt::new(old, 1);
    workspace::prepare(&mut prior);
    let original = serde_json::to_value(&prior).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), prior);
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        original
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    let raw = serde_json::to_string(&loaded).unwrap();
    assert!(
        !raw.contains("\"modules\":")
            && !raw.contains("\"registry\":")
            && !raw.contains("\"outcomes\":")
    );
    let archive = dir.path().join("backup.json");
    backup::export(&archive, &p, catalog()).unwrap();
    let inspected = backup::inspect(
        &archive,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let restored = backup::restore(&path, &inspected, catalog()).unwrap();
    assert!(restored.safety_copy.is_some());
    assert_eq!(
        serde_json::to_value(progress::load(&path, catalog()).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::save(&dir.path().join("no-catalog.json"), &loaded).is_err());
}
