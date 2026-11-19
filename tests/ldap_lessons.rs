use dlms_learning_lab::{
    backup, course,
    engine::{self, State},
    ldap_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
const IDS: [&str; 2] = ["lpic2-ldap-client-base", "lpic2-ldap-contact-query"];
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn actual_catalog_preserves_every_previous360_lab_and128_units_with_two_distinct_tasks() {
    let c = catalog();
    assert_eq!(c.len(), 366);
    let previous=pack::accepted362().unwrap();
    assert_eq!(serde_json::to_value(&c[..362]).unwrap(),serde_json::to_value(&previous).unwrap());
    let old = pack::accepted360().unwrap();
    assert_eq!(
        serde_json::to_value(&c[..360]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 131);
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "ldap-client-queries" && u.id != "ldap-directory-changes" && u.id != "ldap-utility-work" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(course::accepted360(&old).unwrap()).unwrap()
    );
    for id in IDS {
        let s = c.iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        assert_ne!(d.initial.id, d.example.id);
        assert_ne!(d.initial.reference_config, d.example.reference_config);
        assert!(d.initial.cases.len() >= 4);
        assert!(
            d.initial
                .cases
                .iter()
                .filter(|c| c.probe.is_empty())
                .count()
                >= 2
        );
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}
#[test]
fn authored_worked_methods_and_equivalent_queries_pass_without_exact_string_matching() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::LdapClient { simulation } = &mut a.state else {
            panic!()
        };
        let query = if id == IDS[0] {
            "ldapsearch -LLL -s sub -x '(&(objectClass=person)(uid=*))' cn UID"
        } else {
            "ldapsearch -LLL -x '(&(mail=*)(description=on-call)(objectClass=person))' mail uid"
        };
        simulation.assignment.edit_command(query.into()).unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn partial_wrong_scope_extra_fields_unknown_filters_and_hardcoded_names_do_not_pass() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        let start = d
            .initial
            .reports(&d.initial.starter_config, &d.initial.starter_query)
            .unwrap();
        assert!(start.iter().any(|r| !r.passed));
        for query in [
            "ldapsearch -x -LLL '(uid=alex)' uid cn",
            "ldapsearch -x -LLL -s one '(objectClass=person)' uid cn",
            "ldapsearch -x -LLL '(objectClass=person)' '*'",
            "ldapsearch -x -LLL '(!(unknownAttribute=*))' uid mail",
        ] {
            assert!(
                d.initial
                    .reports(&d.initial.reference_config, query)
                    .unwrap()
                    .iter()
                    .any(|r| !r.passed),
                "{id}: {query}"
            );
        }
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let State::LdapClient { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "unfinished query".into();
        simulation.assignment.checked = true;
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn private_catalog_save_backup_restore_keep_new_drafts_and_old_aaa_work() {
    let c = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = c.iter().find(|s| s.id == id).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let d = w::source(s).unwrap();
        let State::LdapClient { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.run(&d.initial).unwrap();
        simulation
            .practice
            .edit_config("URI unfinished\nBASE".into())
            .unwrap();
        p.favorites.insert(id.into());
        p.attempts.insert(id.into(), a);
    }
    let old = c
        .iter()
        .find(|s| s.id == "ccna-aaa-method-recovery")
        .unwrap();
    let mut prior = Attempt::new(old, 2);
    workspace::prepare(&mut prior);
    let State::AaaAccess { simulation } = &mut prior.state else {
        panic!()
    };
    simulation.assignment.draft = "unfinished AAA draft".into();
    let retained = serde_json::to_value(&prior).unwrap();
    p.attempts.insert(old.id.clone(), prior);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, c).unwrap();
    let bytes = std::fs::read(&file).unwrap();
    let loaded = progress::load(&file, c).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        retained
    );
    for id in IDS {
        let s = c.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&loaded.attempts[id], s));
        let State::LdapClient { simulation } = &loaded.attempts[id].state else {
            panic!()
        };
        assert!(simulation.assignment.output.is_none());
        assert_eq!(simulation.practice.config, "URI unfinished\nBASE");
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, c).unwrap();
    let inspected =
        backup::inspect(&export, c, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let result = backup::restore(&file, &inspected, c).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), bytes);
    let selected = c.iter().find(|s| s.id == IDS[0]).unwrap();
    p.attempts.insert(selected.id.clone(), fresh(selected));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], selected));
    let other = c.iter().find(|s| s.id == IDS[1]).unwrap();
    assert!(progress::current_solved(&p.attempts[IDS[1]], other));
}
#[test]
fn incompatible_source_goal_context_and_unsupported_helpers_fail_without_changing_old_data() {
    let s = catalog().iter().find(|s| s.id == IDS[0]).unwrap();
    let mut a = fresh(s);
    let before = a.state.clone();
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "ldapsearch localhost").is_err());
    assert_eq!(a.state, before);
    let old = catalog()
        .iter()
        .find(|s| s.id == "ccna-aaa-method-recovery")
        .unwrap();
    assert!(engine::execute_for_goal(&mut a.state, &old.goal, &s.solution[0]).is_err());
    assert_eq!(a.state, before);
    a.scenario_version += 1;
    assert!(w::context(&a, s).is_err());
    let mut bad = s.clone();
    bad.objective.push_str(" changed");
    assert!(w::source(&bad).is_err());
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![bad]
            })
            .unwrap()
        )
        .is_err()
    );
}
