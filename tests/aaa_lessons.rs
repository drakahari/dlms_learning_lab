//! Authored local-account lessons: actual case outcomes and old-catalog retention.
use dlms_learning_lab::{
    aaa_workspace as w, course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "ccna-aaa-method-recovery",
    "ccna-aaa-authorization-recovery",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn actual_catalog_adds_two_distinct_labs_and_preserves_all_358_previous_values_and_127_units() {
    let old = pack::accepted358().unwrap();
    assert_eq!(old.len(), 358);
    let accepted = pack::accepted360().unwrap();
    assert_eq!(accepted.len(), 360);
    assert_eq!(
        serde_json::to_value(&catalog()[..360]).unwrap(),
        serde_json::to_value(&accepted).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&catalog()[..358]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted358(&old).unwrap();
    let now = course::accepted360(&accepted).unwrap();
    let current = course::ccna_pilot(catalog()).unwrap();
    let previous=pack::accepted362().unwrap();
    assert_eq!(serde_json::to_value(&catalog()[..362]).unwrap(),serde_json::to_value(&previous).unwrap());
    let retained362=current.iter().filter(|u|u.id!="ldap-directory-changes" && u.id!="ldap-utility-work").collect::<Vec<_>>();
    assert_eq!(serde_json::to_value(retained362).unwrap(),serde_json::to_value(course::accepted362(&previous).unwrap()).unwrap());
    let retained360 = current
        .iter()
        .filter(|u| u.id != "ldap-client-queries" && u.id != "ldap-directory-changes" && u.id != "ldap-utility-work")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&retained360).unwrap(),
        serde_json::to_value(&now).unwrap()
    );
    assert_eq!((before.len(), now.len()), (127, 128));
    let retained = now
        .iter()
        .filter(|u| u.id != "aaa-decisions")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&retained).unwrap()
    );
    let new = now.iter().find(|u| u.id == "aaa-decisions").unwrap();
    assert_eq!(new.labs, IDS);
    assert_eq!(new.prerequisites, ["local-access-foundations"]);
}
#[test]
fn taught_documents_and_equivalent_grouping_rule_and_record_order_pass_behaviorally() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        d.validate().unwrap();
        let mut a = attempt(s);
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let alternate=d.initial.reference.replace("line vty 0 15\n transport input ssh\n access-class 10 in\nexit","line vty 0 7\n transport input ssh\n access-class 10 in\n login authentication default\nexit\nline vty 8 15\n transport input ssh\n access-class 10 in\nexit").replace("permit host 192.0.2.10\n permit host 192.0.2.11","permit 192.0.2.10 0.0.0.1");
        assert_ne!(alternate, d.initial.reference);
        assert!(
            d.initial
                .reports(&alternate)
                .unwrap()
                .iter()
                .all(w::Report::passed)
        );
        let mut lines = alternate.lines().map(str::to_string).collect::<Vec<_>>();
        let positions = lines
            .iter()
            .enumerate()
            .filter(|(_, s)| s.starts_with("username "))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(positions.len(), 3);
        lines.swap(positions[0], positions[2]);
        assert!(
            d.initial
                .reports(&lines.join("\n"))
                .unwrap()
                .iter()
                .all(w::Report::passed)
        );
    }
}
#[test]
fn wrong_words_roles_surplus_profiles_bypasses_and_broken_incoming_policy_cannot_pass() {
    for id in IDS {
        let t = &w::source(lab(id)).unwrap().initial;
        let name = &t.cases[0].username;
        let lowered = t.reference.replace(
            &format!("username {name} privilege 15"),
            &format!("username {name} privilege 1"),
        );
        assert_ne!(lowered, t.reference);
        let rows = t.reports(&lowered).unwrap();
        assert_eq!(rows.iter().filter(|r| r.passed()).count(), 14);
        assert!(rows.iter().any(|r| {
            r.first_failure
                .as_ref()
                .is_some_and(|s| s.contains("level 1"))
        }));
        let recovery = &t.cases[2];
        let wrong = t.reference.replace(
            &format!("secret {}", recovery.training_token),
            "secret Wrong-Recovery",
        );
        assert_ne!(wrong, t.reference);
        assert!(!t.reports(&wrong).unwrap()[9].passed());
        let bypass = t
            .reference
            .replace("group OfficeIdentity local-case", "local-case");
        assert_ne!(bypass, t.reference);
        assert!(t.reports(&bypass).unwrap()[7..].iter().any(|r| !r.passed()));
        let blocked = t
            .reference
            .replace("transport input ssh", "transport input none");
        assert!(
            t.reports(&blocked).unwrap()[7..]
                .iter()
                .all(|r| !r.passed())
        );
        let extra = format!(
            "username UnwantedProfile privilege 15 secret Surplus-Training\n{}",
            t.reference
        );
        let rows = t.reports(&extra).unwrap();
        assert!(!rows[0].passed());
        assert!(rows[1..].iter().all(w::Report::passed));
    }
}
#[test]
fn worked_examples_are_ungraded_and_invalid_drafts_resume_without_stale_credit() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = attempt(s);
        let State::AaaAccess { simulation } = &mut a.state else {
            panic!()
        };
        simulation.fill_example(d).unwrap();
        assert!(
            simulation
                .practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(w::Report::passed)
        );
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::AaaAccess { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.inspect(&d.initial).unwrap();
        simulation
            .assignment
            .edit("username unfinished".into())
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        p.favorites.insert(id.into());
        p.attempts.insert(id.into(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("local-progress.json");
    progress::save_with_catalog(&file, &p, catalog()).unwrap();
    let restored = progress::load(&file, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for id in IDS {
        let s = lab(id);
        assert!(!progress::current_solved(&restored.attempts[id], s));
        let State::AaaAccess { simulation } = &restored.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.draft, "username unfinished");
        assert!(simulation.assignment.observation.is_none());
        assert!(restored.favorites.contains(id));
    }
}
