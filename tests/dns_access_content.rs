//! Three authored proposals. Registration/native review are separate acceptance gates.
use dlms_learning_lab::{
    course,
    dns_access_workspace::{Action, Config, Session},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-dns-recursion-boundary",
    "lpic2-dns-secondary-transfer-policy",
    "lpic2-dns-access-handover",
];
fn proposed() -> Vec<Scenario> {
    static DATA: OnceLock<Vec<Scenario>> = OnceLock::new();
    DATA.get_or_init(|| {
        // Validate the complete graph; retained fragment prerequisites live in other packs.
        let raw: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v20.json")).unwrap();
        let mut all = pack::accepted409().unwrap();
        all.extend(
            raw.scenarios
                .into_iter()
                .filter(|s| IDS.contains(&s.id.as_str())),
        );
        let chunks = all
            .chunks(8)
            .map(|labs| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":labs}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let refs = chunks.iter().map(String::as_str).collect::<Vec<_>>();
        pack::load_catalog(&refs).unwrap()
    })
    .iter()
    .filter(|s| IDS.contains(&s.id.as_str()))
    .cloned()
    .collect()
}
fn session(a: &mut Attempt) -> &mut Session {
    let State::DnsAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn action(a: &mut Attempt, s: &Scenario, value: &Action) {
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &format!("dns-policy {}", serde_json::to_string(value).unwrap()),
    )
    .unwrap();
}
#[test]
fn all_taught_methods_and_distinct_examples_reach_their_real_behavior_without_assignment_credit() {
    let labs = proposed();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let Goal::DnsAccess { definition } = &s.goal else {
            panic!()
        };
        definition.validate().unwrap();
        let mut a = Attempt::new(&s, 1);
        assert!(!progress::current_solved(&a, &s));
        for step in &definition.example.reference {
            if let Action::Save { config } = step {
                session(&mut a)
                    .practice
                    .set_editor(config.0.clone())
                    .unwrap();
            }
            session(&mut a)
                .practice
                .run(&definition.example, step.clone())
                .unwrap()
                .unwrap();
        }
        assert!(
            session(&mut a)
                .practice
                .check(&definition.example)
                .unwrap()
                .iter()
                .all(|r| r.correct)
        );
        assert!(!progress::current_solved(&a, &s));
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        assert!(progress::current_solved(&a, &s), "{}", s.id);
        assert_ne!(definition.initial.facts, definition.example.facts);
        assert!(definition.initial.cases.len() >= 6);
        assert!(definition.initial.cases.iter().any(|c| matches!(
            c.cache,
            dlms_learning_lab::dns_access_workspace::Cache::SuppliedWarm
        )));
    }
}
#[test]
fn split_client_ranges_broader_recursive_gate_behind_restricted_cache_and_host_slash32_pass() {
    for s in proposed() {
        let Goal::DnsAccess { definition } = &s.goal else {
            panic!()
        };
        let mut a = Attempt::new(&s, 1);
        let mut steps = definition.initial.reference.clone();
        let Action::Save { config } = &mut steps[0] else {
            panic!()
        };
        if let dlms_learning_lab::dns_access::acl::Acl::Networks { networks } =
            &definition.initial.required.cached_answers
        {
            let network = &networks[0];
            assert_eq!(network.prefix, 24);
            let base = u32::from(network.address);
            let staff = format!("{}/24", network.address);
            config.0 = config
                .0
                .replace(
                    &format!("allow-recursion {{ {staff}; }};"),
                    "allow-recursion { any; };",
                )
                .replace(
                    &format!("{staff};"),
                    &format!(
                        "{}/25; {}/25;",
                        std::net::Ipv4Addr::from(base),
                        std::net::Ipv4Addr::from(base + 128)
                    ),
                );
        }
        for required in &definition.initial.required.transfers {
            if let dlms_learning_lab::dns_access::acl::Acl::Networks { networks } =
                &required.clients
            {
                let host = networks[0].address;
                config.0 = config
                    .0
                    .replace(&format!("{{ {host}; }}"), &format!("{{ {host}/32; }}"));
            }
        }
        for step in steps {
            action(&mut a, &s, &step);
        }
        engine::execute_for_goal(&mut a.state, &s.goal, "dns-check").unwrap();
        assert!(progress::current_solved(&a, &s), "{}", s.id);
    }
}
#[test]
fn partial_saved_rules_and_rebroken_state_fail_with_specific_scope_feedback() {
    for s in proposed() {
        let Goal::DnsAccess { definition } = &s.goal else {
            panic!()
        };
        let mut a = Attempt::new(&s, 1);
        action(&mut a, &s, &definition.initial.reference[0]);
        engine::execute_for_goal(&mut a.state, &s.goal, "dns-check").unwrap();
        assert!(!progress::current_solved(&a, &s));
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        action(
            &mut a,
            &s,
            &Action::Save {
                config: definition.initial.initial_config.clone(),
            },
        );
        action(&mut a, &s, &Action::Reload {});
        assert!(!progress::current_solved(&a, &s));
        engine::execute_for_goal(&mut a.state, &s.goal, "dns-check").unwrap();
        let checks = engine::checks(&a.state, &s.goal);
        assert!(
            checks.iter().any(|r| !r.0
                && (r.1.starts_with("Cached-data access") || r.1.starts_with("Transfer access"))),
            "{}: {checks:?}",
            s.id
        );
        assert!(checks.iter().any(|r| r.0 && r.1.starts_with("Query case")));
    }
}
#[test]
fn registered_unit_preserves_all409_values_units_and_reachable_prerequisites() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v19.json")).unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v20.json")).unwrap();
    assert_eq!(old["scenarios"].as_array().unwrap().len(), 36);
    assert_eq!(new["scenarios"].as_array().unwrap().len(), 39);
    assert_eq!(
        &new["scenarios"].as_array().unwrap()[..36],
        old["scenarios"].as_array().unwrap()
    );
    let old = pack::accepted409().unwrap();
    assert_eq!(old.len(), 409);
    assert_eq!(course::accepted409(&old).unwrap().len(), 157);
    assert!(old.iter().all(|s| !IDS.contains(&s.id.as_str())));
    let mut combined = old;
    combined.extend(proposed());
    let chunks = combined
        .chunks(8)
        .map(|labs| {
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":labs}))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let refs = chunks.iter().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(pack::load_catalog(&refs).unwrap().len(), 412);
    assert_eq!(combined.len(), 412);
    let current = pack::current().unwrap();
    assert_eq!(current.len(), 424);
    let units = course::current(&current).unwrap();
    assert_eq!(units.len(), 163);
    for id in IDS {
        assert!(current.iter().any(|s| s.id == id));
    }
    let previous = pack::accepted409().unwrap();
    let previous_units = course::accepted409(&previous).unwrap();
    for prior in &previous_units {
        let actual = units.iter().find(|u| u.id == prior.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(prior).unwrap()
        );
    }
    for prior in previous {
        let actual = current.iter().find(|s| s.id == prior.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(&prior).unwrap()
        );
    }

    assert!(combined.iter().any(|s| s.id == "lpic2-dns-handover"));
}
#[test]
fn resume_preserves_two_drafts_favorites_and_existing_lab_and_reset_changes_only_selected_attempt()
{
    let mut catalog = pack::accepted409().unwrap();
    let peer = catalog[0].clone();
    let labs = proposed();
    catalog.extend(labs.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(peer.id.clone(), Attempt::new(&peer, 1));
    let peer_before = serde_json::to_value(&p.attempts[&peer.id]).unwrap();
    for s in &labs {
        let mut a = Attempt::new(s, 1);
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        session(&mut a)
            .practice
            .set_editor("# retain this unfinished practice\noptions {".into())
            .unwrap();
        session(&mut a).assignment.query_name.0 = "unfinished.name".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let restored = progress::decode(&serde_json::to_vec(&p).unwrap(), &catalog).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&peer.id]).unwrap(),
        peer_before
    );
    for s in &labs {
        let mut a = restored.attempts[&s.id].clone();
        assert!(progress::current_solved(&a, s));
        assert_eq!(
            session(&mut a).practice.editor.0,
            "# retain this unfinished practice\noptions {"
        );
        let practice = session(&mut a).practice.clone();
        let Goal::DnsAccess { definition } = &s.goal else {
            panic!()
        };
        session(&mut a).reset_assignment(definition).unwrap();
        assert_eq!(session(&mut a).practice, practice);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn malformed_unknown_foreign_and_falsely_solved_source_data_refuse() {
    for s in proposed() {
        let mut bad = s.clone();
        let State::DnsAccess { simulation } = &mut bad.initial else {
            panic!()
        };
        simulation.assignment.editor = Config("invented starter".into());
        assert!(dlms_learning_lab::dns_access_workspace::source(&bad).is_err());
        let mut bad = s.clone();
        let Goal::DnsAccess { definition } = &mut bad.goal else {
            panic!()
        };
        definition
            .initial
            .reference
            .retain(|a| !matches!(a, Action::Reload {}));
        assert!(dlms_learning_lab::dns_access_workspace::source(&bad).is_err());
        let mut equivalent = s.clone();
        let Goal::DnsAccess { definition } = &mut equivalent.goal else {
            panic!()
        };
        definition
            .initial
            .reference
            .retain(|a| !matches!(a, Action::Query { .. }));
        assert!(
            dlms_learning_lab::dns_access_workspace::source(&equivalent).is_ok(),
            "Observation queries are useful teaching, not a forced ritual for reaching correct state"
        );
        let mut a = Attempt::new(&s, 1);
        let before = serde_json::to_value(&a).unwrap();
        for command in [
            "dig @real-host example.org",
            "sh script.sh",
            "dns-policy {\"kind\":\"reload\",\"shell\":true}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, command).is_err());
            assert_eq!(serde_json::to_value(&a).unwrap(), before);
        }
    }
}
