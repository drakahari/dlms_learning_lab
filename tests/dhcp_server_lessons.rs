use dlms_learning_lab::{
    backup, course, dhcp_server_workspace as w,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/linux-dhcp-lessons-v1.json");
const IDS: [&str; 2] = ["lpic2-dhcp-dynamic-ranges", "lpic2-dhcp-option-scope"];
fn catalog() -> Vec<pack::Scenario> {
    pack::load(RAW).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::DhcpConfig { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::DhcpConfig { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 5);
    workspace::prepare(&mut a);
    a
}
fn reference(s: &pack::Scenario) -> String {
    s.solution[0].strip_prefix(w::PREFIX).unwrap().into()
}
fn apply(s: &pack::Scenario, text: &str) -> State {
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, &format!("{}{text}", w::PREFIX)).unwrap();
    state
}
#[test]
fn additive_catalog_preserves242_labs_66_units_and_separate_dhcp_prerequisites() {
    let before = pack::accepted242().unwrap();
    let now = pack::accepted244().unwrap();
    assert_eq!((before.len(), now.len()), (242, 244));
    assert_eq!(
        serde_json::to_value(before.clone()).unwrap(),
        serde_json::to_value(&now[..242]).unwrap()
    );
    let old = course::accepted242(&before).unwrap();
    let units = course::accepted244(&now).unwrap();
    assert_eq!((old.len(), units.len()), (66, 67));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&units[..66]).unwrap()
    );
    let u = units.last().unwrap();
    assert_eq!(u.id, "linux-dhcp-configuration");
    assert_eq!(u.labs, IDS);
    assert_eq!(u.prerequisites, ["linux-host-networking"]);
    assert_eq!(now[243].prerequisites, [IDS[0]]);
    let totals = ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
        .map(|t| now.iter().filter(|s| s.track == t).count());
    assert_eq!(&totals[..3], &[72, 63, 43]);
    assert_eq!(now.iter().filter(|s| s.teaching.is_some()).count(), 66);
}
#[test]
fn taught_references_and_equivalent_supported_ranges_or_scopes_pass_actual_client_cases() {
    for s in catalog() {
        let d = w::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let text = reference(&s);
        let mut variants = vec![
            text.clone(),
            format!("# Equivalent whitespace and comments\n{text}\n"),
        ];
        if s.id == IDS[0] {
            variants.push(text.replace(
                "range 10.50.0.100 10.50.0.103;",
                "range 10.50.0.102 10.50.0.104;",
            ));
        } else {
            variants.push(
                text.replace("option domain-name-servers 203.0.113.53;", "")
                    .replace(
                        "option routers 10.51.0.1;",
                        "option routers 10.51.0.1; option domain-name-servers 203.0.113.53;",
                    ),
            );
        }
        for v in variants {
            let state = apply(&s, &v);
            assert!(
                engine::solved(&state, &s.goal),
                "{}: {:?}",
                s.id,
                engine::checks(&state, &s.goal)
            );
            let State::DhcpConfig { simulation } = &state else {
                panic!()
            };
            assert!(
                d.initial
                    .run(&simulation.assignment.draft)
                    .unwrap()
                    .iter()
                    .all(w::CaseResult::passed)
            );
        }
        for challenge in [true, false] {
            let mut a = fresh(&s);
            a.challenge = challenge;
            a.state = apply(&s, &text);
            assert!(progress::current_solved(&a, &s));
            session(&mut a)
                .assignment
                .replace(&d.initial.starter)
                .unwrap();
            assert!(!progress::current_solved(&a, &s));
        }
    }
}
#[test]
fn common_mistakes_partial_case_failure_and_retry_have_honest_current_results() {
    for s in catalog() {
        let d = definition(&s);
        let text = reference(&s);
        let mut wrong = vec![
            d.initial.starter.clone(),
            text.replace("default-lease-time 600;", "default-lease-time 900;"),
            text.replace("203.0.113.53", "203.0.113.99"),
            "subnet {".into(),
            "include \"/etc/shadow\";".into(),
            "$(id)".into(),
        ];
        if s.id == IDS[0] {
            wrong.extend([
                text.replace("option routers 10.50.0.1;", "option routers 203.0.113.1;"),
                text.replace("range 10.50.0.100 10.50.0.103;", ""),
                format!("{text}\nsubnet 10.90.0.0 netmask 255.255.255.0 {{ range 10.90.0.100; }}"),
            ]);
        } else {
            wrong.extend([
                text.replace("option domain-name-servers 203.0.113.54;", ""),
                text.replace(
                    "range 10.51.0.10 10.51.0.15;",
                    "range 10.51.0.11 10.51.0.15;",
                ),
                text.replace(
                    "range 10.51.0.10 10.51.0.15;",
                    "range 10.51.0.10 10.51.0.13;",
                ),
            ]);
        }
        for v in wrong {
            let state = apply(&s, &v);
            assert!(
                !engine::solved(&state, &s.goal),
                "false positive {}: {v}",
                s.id
            );
            let checks = engine::checks(&state, &s.goal);
            assert!(checks.iter().any(|r| !r.0));
            let mut repaired = state;
            engine::execute_for_goal(&mut repaired, &s.goal, &s.solution[0]).unwrap();
            assert!(engine::solved(&repaired, &s.goal));
        }
        let mut a = fresh(&s);
        let sim = session(&mut a);
        sim.practice.replace(&d.example_worked).unwrap();
        sim.practice.run(&d.example).unwrap();
        assert!(sim.practice.passed(&d.example));
        assert_eq!(sim.assignment.draft, d.initial.starter);
        assert!(!progress::current_solved(&a, &s));
    }
    let s = &catalog()[0];
    let rows = definition(s)
        .initial
        .run(&definition(s).initial.starter)
        .unwrap();
    assert!(rows[0].passed());
    assert!(!rows[1].passed());
    assert!(rows[1].checks.iter().any(|r| r.0));
}
#[test]
fn drafts_current_credit_backup_restore_and_selected_reset_survive_safe_writes() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = fresh(s);
        a.state = apply(s, &reference(s));
        let d = definition(s);
        session(&mut a).practice.replace(&d.example_worked).unwrap();
        session(&mut a).view = dlms_learning_lab::script_teaching::View::Example;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for s in &all {
        assert!(progress::current_solved(&got.attempts[&s.id], s));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let restore =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let receipt = backup::restore(&path, &restore, &all).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    let kept = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&all[0]));
    assert_eq!(serde_json::to_value(&p.attempts[IDS[1]]).unwrap(), kept);
    assert!(p.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], &all[0]));
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let decoded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
#[test]
fn foreign_progress_unknown_fields_and_invalid_sources_refuse_without_overwriting() {
    let all = catalog();
    let s = &all[0];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = "foreign terminal draft".into();
    assert!(progress::save_with_catalog(&path, &p, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    p.attempts.insert(s.id.clone(), fresh(s));
    session(p.attempts.get_mut(&s.id).unwrap()).origin = "different-task".into();
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let source: serde_json::Value = serde_json::from_str(RAW).unwrap();
    for change in 0..3 {
        let mut v = source.clone();
        match change {
            0 => v["scenarios"][0]["goal"]["definition"]["initial"]["id"] = "wrong".into(),
            1 => {
                v["scenarios"][0]["initial"]["simulation"]["assignment"]["draft"] = "foreign".into()
            }
            _ => v["scenarios"][0]["goal"]["definition"]["example_worked"] = "unsupported;".into(),
        };
        assert!(pack::load(&v.to_string()).is_err());
    }
}
#[test]
fn every_authored_practice_experiment_fails_for_its_taught_reason_then_repairs_without_credit() {
    for source in catalog() {
        let d = definition(&source);
        let mut sim = w::Session::new(d).unwrap();
        let original = sim.assignment.clone();
        assert!(
            !sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let changed = if source.id == IDS[0] {
            d.example_worked
                .replace("range 10.60.0.100 10.60.0.103;", "range 10.60.0.100;")
        } else {
            d.example_worked
                .replace("option domain-name-servers 203.0.113.54;", "")
        };
        assert_ne!(changed, d.example_worked);
        sim.practice.replace(&changed).unwrap();
        assert!(!sim.practice.passed(&d.example));
        let rows = sim.practice.run(&d.example).unwrap();
        assert!(rows.iter().all(|r| !r.passed() && r.diagnostic.is_none()));
        if source.id == IDS[0] {
            assert!(rows.iter().all(|r| {
                r.events
                    .iter()
                    .any(|e| !e.ok && e.message.contains("No available address"))
            }));
        } else {
            for r in rows {
                assert!(r.events.iter().all(|e| e.ok));
                let after = r.after.unwrap();
                assert_eq!(
                    after.leases()["workshop-option-scope-remote"]
                        .parameters
                        .dns,
                    ["203.0.113.53".parse::<std::net::Ipv4Addr>().unwrap()]
                );
                assert!(r.checks.iter().any(|(ok, s)| !ok
                    && s.contains("expected router")
                    && s.contains("203.0.113.54")
                    && s.contains("203.0.113.53")));
            }
        }
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert_eq!(sim.assignment, original);
        assert!(!d.solved(&sim));
    }
}
