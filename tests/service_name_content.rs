//! Authored tasks and actual normal catalog/persistence, never host lookups.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt},
    service_workspace as w, workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic1-service-client-targets", "lpic1-services-file-repair"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::current().unwrap())
}
fn source(n: usize) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == IDS[n]).unwrap()
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::ServiceClients { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(t: &w::Task, d: &mut w::Draft) {
    if t.mode == w::Mode::Clients {
        for c in &t.required_clients {
            d.configure(t, &c.id, c.protocol, c.service.clone())
                .unwrap();
        }
    } else {
        d.edit(t, t.required_database.clone()).unwrap();
        d.save(t).unwrap();
    }
    assert!(d.check(t).unwrap().passed());
}
#[test]
fn current436_retains_every434_source_and_unit_object_and_places_prerequisites_first() {
    let old = pack::accepted434().unwrap();
    let frozen = pack::accepted436().unwrap();
    assert_eq!((old.len(), frozen.len()), (434, 436));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&catalog()[..434]).unwrap()
    );
    let before = course::accepted434(&old).unwrap();
    let all = course::accepted436(&frozen).unwrap();
    assert_eq!((before.len(), all.len()), (168, 169));
    let new = all
        .iter()
        .position(|u| u.id == "linux-service-names")
        .unwrap();
    assert_eq!(all[new].labs, IDS);
    assert!(
        all.iter()
            .position(|u| u.id == "linux-host-networking")
            .unwrap()
            < new
    );
    let retained = all
        .iter()
        .filter(|u| u.id != "linux-service-names")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let pin: pack::Pack =
        serde_json::from_str(include_str!("fixtures/service-name-preview-v1.json")).unwrap();
    for s in pin.scenarios {
        assert_eq!(
            serde_json::to_value(catalog().iter().find(|a| a.id == s.id).unwrap()).unwrap(),
            serde_json::to_value(s).unwrap()
        );
    }
}
#[test]
fn taught_examples_are_isolated_and_reference_and_equivalent_solutions_pass() {
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        assert_eq!(d.initial.requests.len(), if n == 0 { 6 } else { 4 });
        let mut a = Attempt::new(s, 1);
        let before = session(&mut a).assignment.clone();
        solve(&d.example, &mut session(&mut a).practice);
        assert_eq!(session(&mut a).assignment, before);
        assert!(!progress::current_solved(&a, s));
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let mut b = Attempt::new(s, 1);
        let target = &mut session(&mut b).assignment;
        if n == 0 {
            for c in &d.initial.required_clients {
                let role = &d
                    .initial
                    .requests
                    .iter()
                    .find(|r| r.client == c.id)
                    .unwrap()
                    .required_role;
                let l = d.initial.profile.listener(role).unwrap();
                target
                    .configure(&d.initial, &c.id, l.protocol, l.port.to_string())
                    .unwrap();
            }
        } else {
            target
                .edit(
                    &d.initial,
                    format!(
                        "# equivalent\n{}",
                        d.initial
                            .required_database
                            .lines()
                            .rev()
                            .collect::<Vec<_>>()
                            .join("\n")
                    ),
                )
                .unwrap();
            target.save(&d.initial).unwrap();
        }
        assert!(target.check(&d.initial).unwrap().passed());
        assert!(progress::current_solved(&b, s));
    }
}
#[test]
fn wrong_open_ports_partial_repairs_and_deleting_unrelated_names_cannot_pass() {
    for n in 0..2 {
        let s = source(n);
        let def = w::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        let r = session(&mut a).assignment.check(&def.initial).unwrap();
        assert!(!r.passed());
        assert!(r.cases.iter().any(|r| !r.passed));
        if n == 1 {
            assert!(r.cases.iter().any(|r| {
                r.actual.as_ref().is_some_and(|a| {
                    a.transport.outcome == dlms_learning_lab::transport_core::Outcome::Delivered
                }) && !r.passed
            }));
        }
        let d = &mut session(&mut a).assignment;
        if n == 0 {
            let c = &def.initial.required_clients[0];
            d.configure(&def.initial, &c.id, c.protocol, c.service.clone())
                .unwrap();
            let r = d.check(&def.initial).unwrap();
            assert_eq!(r.cases.iter().filter(|r| r.passed).count(), 2);
        } else {
            d.edit(
                &def.initial,
                def.initial.required_database.replace("https 443/tcp\n", ""),
            )
            .unwrap();
            d.save(&def.initial).unwrap();
            let r = d.check(&def.initial).unwrap();
            assert!(r.cases.iter().all(|r| r.passed));
            assert!(!r.database_errors.is_empty());
        }
        assert!(!progress::current_solved(&a, s));
        solve(&def.initial, &mut session(&mut a).assignment);
        assert!(progress::current_solved(&a, s));
        let d = &mut session(&mut a).assignment;
        if n == 0 {
            d.configure(
                &def.initial,
                "admin",
                dlms_learning_lab::transport_core::Protocol::Tcp,
                "http".into(),
            )
            .unwrap();
        } else {
            d.edit(&def.initial, "ssh".into()).unwrap();
        }
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn source_actions_are_atomic_and_unknown_commands_are_never_executed() {
    for n in 0..2 {
        let s = source(n);
        let mut a = Attempt::new(s, 1);
        let before = serde_json::to_value(&a.state).unwrap();
        for input in [
            "sh /private",
            "getent services ssh",
            "@service-clients {\"action\":\"save\",\"path\":\"/private\"}",
            "@service-clients {\"action\":\"configure\",\"client\":\"invented\",\"protocol\":\"tcp\",\"service\":\"22\"}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
            assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
        }
    }
    let s = source(0);
    let mut damaged = s.clone();
    let engine::Goal::ServiceClients { definition } = &mut damaged.goal else {
        panic!()
    };
    definition.initial.required_clients[0].service = "http".into();
    assert!(w::source(&damaged).is_err());
}
#[test]
fn safe_disk_resume_preserves_all434_prior_attempts_and_both_new_drafts() {
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &catalog()[..434] {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        if dlms_learning_lab::command_location_workspace::has_source(s) {
            dlms_learning_lab::command_location_workspace::prepare(&mut a, s, true).unwrap();
        }
        if dlms_learning_lab::accounts_template_workspace::has_source(s) {
            dlms_learning_lab::accounts_template_workspace::prepare(&mut a, s, true).unwrap();
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let original = serde_json::to_value(&p).unwrap();
    for n in 0..2 {
        let s = source(n);
        let def = w::source(s).unwrap();
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        solve(&def.initial, &mut session(&mut a).assignment);
        session(&mut a).practice.terminal_input = "getent services www/tcp".into();
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for id in original["attempts"].as_object().unwrap().keys() {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            original["attempts"][id]
        );
        assert!(restored.favorites.contains(id));
    }
    for (n, id) in IDS.iter().enumerate() {
        assert!(progress::current_solved(&restored.attempts[*id], source(n)));
    }
    let untouched = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    restored
        .attempts
        .insert(IDS[0].into(), Attempt::new(source(0), 3));
    assert_eq!(
        untouched,
        serde_json::to_value(&restored.attempts[IDS[1]]).unwrap()
    );
    restored.attempts.get_mut(IDS[1]).unwrap().scenario_version += 1;
    assert!(progress::save_with_catalog(&path, &restored, catalog()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
