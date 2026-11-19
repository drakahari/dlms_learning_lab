//! Authored local-account lessons: actual case outcomes and old-catalog retention.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    local_access_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["ccna-local-account-setup", "ccna-local-account-handover"];
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
fn actual_catalog_adds_two_distinct_labs_and_preserves_all_356_previous_values_and_126_units() {
    let old = pack::accepted356().unwrap();
    assert_eq!(old.len(), 356);
    assert_eq!(catalog().len(), 358);
    assert_eq!(
        serde_json::to_value(&catalog()[..356]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted356(&old).unwrap();
    let now = course::ccna_pilot(catalog()).unwrap();
    assert_eq!((before.len(), now.len()), (126, 127));
    let retained = now
        .iter()
        .filter(|u| u.id != "local-access-foundations")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&retained).unwrap()
    );
    let new = now
        .iter()
        .find(|u| u.id == "local-access-foundations")
        .unwrap();
    assert_eq!(new.labs, IDS);
    assert_eq!(new.prerequisites, ["management-policy-foundations"]);
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
        assert_eq!(positions.len(), 2);
        lines.swap(positions[0], positions[1]);
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
fn wrong_words_roles_surplus_accounts_and_broken_incoming_policy_cannot_pass() {
    for id in IDS {
        let s = lab(id);
        let t = &w::source(s).unwrap().initial;
        let name = &t.cases[0].username;
        let lowered = t.reference.replace(
            &format!("username {name} privilege 15"),
            &format!("username {name} privilege 1"),
        );
        assert_ne!(lowered, t.reference);
        let reports = t.reports(&lowered).unwrap();
        assert_eq!(reports.iter().filter(|r| r.passed()).count(), 12);
        assert!(reports.iter().any(|r| {
            r.first_failure
                .as_deref()
                .is_some_and(|s| s.contains("level 1"))
        }));
        let word = &t.cases[0].training_token;
        let wrong = t
            .reference
            .replace(&format!("secret {word}"), "secret Wrong-Training-Value");
        assert!(t.reports(&wrong).unwrap().iter().any(|r| !r.passed()));
        let blocked = t
            .reference
            .replace("transport input ssh", "transport input none");
        let rows = t.reports(&blocked).unwrap();
        assert!(rows[7..].iter().all(|r| !r.passed()));
    }
    let t = &w::source(lab(IDS[1])).unwrap().initial;
    let old = &t.cases[5].username;
    for name in [old.as_str(), "RenamedSurplus"] {
        let raw = format!(
            "username {name} privilege 15 secret Rotated-Training\n{}",
            t.reference
        );
        let rows = t.reports(&raw).unwrap();
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
        let State::LocalAccess { simulation } = &mut a.state else {
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
        let State::LocalAccess { simulation } = &mut a.state else {
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
        let State::LocalAccess { simulation } = &restored.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.draft, "username unfinished");
        assert!(simulation.assignment.observation.is_none());
        assert!(restored.favorites.contains(id));
    }
}
