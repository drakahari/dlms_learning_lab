use dlms_learning_lab::{
    dns_access::world::World,
    dns_access_workspace::{self as work, Action, Cache, Config, Name},
};
#[path = "support/dns_access_task.rs"]
mod fixture;
#[test]
fn taught_reference_and_alternate_policy_placements_pass_by_current_behavior() {
    let def = fixture::definition();
    def.validate().unwrap();
    let task = fixture::task();
    assert!(
        task.grade(&task.initial().unwrap())
            .iter()
            .any(|r| !r.correct)
    );
    let reached = fixture::reached(&task);
    assert!(task.grade(&reached).iter().all(|r| r.correct));
    let mut alternate = task.initial().unwrap();
    for action in [
        Action::Save {
            config: Config(fixture::alternate_config()),
        },
        Action::Reload {},
        task.reference[2].clone(),
    ] {
        work::apply(&mut alternate, &action).unwrap();
    }
    assert!(task.grade(&alternate).iter().all(|r| r.correct));
}
#[test]
fn public_deny_all_recursion_only_restrictions_and_save_without_reload_are_not_solutions() {
    let task = fixture::task();
    let correct = match &task.reference[0] {
        Action::Save { config } => config.0.clone(),
        _ => unreachable!(),
    };
    for config in [
        correct.replace("allow-query { any; };", "allow-query { none; };"),
        correct.replace(
            "allow-query-cache { 192.0.2.0/24; };",
            "allow-query-cache { any; };",
        ),
        correct.replace(
            "allow-recursion { 192.0.2.0/24; };",
            "allow-recursion { 192.0.2.7; };",
        ),
        correct.replace("allow-transfer { none; };", "allow-transfer { any; };"),
    ] {
        let mut world = task.initial().unwrap();
        for action in [
            Action::Save {
                config: Config(config),
            },
            Action::Reload {},
            task.reference[2].clone(),
        ] {
            work::apply(&mut world, &action).unwrap();
        }
        assert!(task.grade(&world).iter().any(|r| !r.correct));
    }
    let mut partial = task.initial().unwrap();
    work::apply(&mut partial, &task.reference[0]).unwrap();
    assert!(task.grade(&partial).iter().any(|r| !r.correct));
    work::apply(&mut partial, &Action::Reload {}).unwrap();
    assert!(
        task.grade(&partial)
            .iter()
            .any(|r| !r.correct && r.label.starts_with("Secondary copy"))
    );
}
#[test]
fn full_scope_checks_reject_policy_holes_between_sampled_clients() {
    let task = fixture::task();
    let mut world = fixture::reached(&task);
    let config = match &task.reference[0] {
        Action::Save { config } => config.0.replace("192.0.2.0/24;", "192.0.2.7; 192.0.2.8;"),
        _ => unreachable!(),
    };
    work::apply(
        &mut world,
        &Action::Save {
            config: Config(config),
        },
    )
    .unwrap();
    work::apply(&mut world, &Action::Reload {}).unwrap();
    let rows = task.grade(&world);
    assert!(
        rows.iter()
            .filter(|r| r.label.starts_with("Query case"))
            .all(|r| r.correct)
    );
    assert!(
        rows.iter()
            .any(|r| !r.correct && r.label == "New recursive lookups")
    );
    assert!(
        rows.iter()
            .any(|r| !r.correct && r.label == "Cached-data access")
    );
}
#[test]
fn validation_refuses_fabricated_fixture_cases_unreachable_methods_and_solved_starters() {
    let task = fixture::task();
    task.validate().unwrap();
    let mut bad = task.clone();
    bad.cases[0].name = Name("unprovided.invalid.".into());
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.cases[1].cache = Cache::SuppliedWarm;
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.reference.pop();
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.starter = bad.reference.clone();
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.cases[0].label = bad.cases[1].label.clone();
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.required.transfers.pop();
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.cases.clear();
    assert!(bad.validate().is_err());
    for raw in [
        r#"{"kind":"reload","host":true}"#,
        r#"{"kind":"clear_cache","command":"id"}"#,
    ] {
        assert!(serde_json::from_str::<Action>(raw).is_err());
    }
}
#[test]
fn foreign_world_and_later_policy_breakage_cannot_reuse_a_successful_result() {
    let task = fixture::task();
    let mut world = fixture::reached(&task);
    assert!(task.grade(&world).iter().all(|r| r.correct));
    let mut foreign = task.facts.clone();
    foreign.zones[1].server.primary.serial += 1;
    let other = World::new(&foreign, &task.initial_config.0).unwrap();
    let result = task.grade(&other);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label, "Supplied world");
    assert!(!result[0].correct);
    work::apply(
        &mut world,
        &Action::Save {
            config: task.initial_config.clone(),
        },
    )
    .unwrap();
    work::apply(&mut world, &Action::Reload {}).unwrap();
    assert!(task.grade(&world).iter().any(|r| !r.correct));
}
