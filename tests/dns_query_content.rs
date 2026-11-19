use dlms_learning_lab::{
    dns::{self, Data},
    dns_teaching,
    dns_workspace::{Draft, UtilityDraft},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
const IDS: [&str; 3] = [
    "lpic2-dns-dualstack",
    "lpic2-dns-aliases",
    "lpic2-dns-secondary",
];
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/dns-forward-v2.json")).unwrap()
}
fn server(s: &State) -> &dns::Server {
    let State::AuthoritativeDns { server } = s else {
        panic!()
    };
    server
}
fn server_mut(s: &mut State) -> &mut dns::Server {
    let State::AuthoritativeDns { server } = s else {
        panic!()
    };
    server
}
#[test]
fn registered_source_changes_only_explicit_lookup_metadata_and_reference() {
    let old = pack::load(include_str!("../scenarios/dns-forward-v1.json")).unwrap();
    let new = labs();
    assert_eq!((old.len(), new.len()), (6, 6));
    for original in old {
        let updated = new.iter().find(|s| s.id == original.id).unwrap();
        let mut value = serde_json::to_value(updated).unwrap();
        if IDS.contains(&updated.id.as_str()) {
            let cap = value["dns_teaching"]
                .as_object_mut()
                .unwrap()
                .remove("lookup")
                .unwrap();
            let mut endpoints = serde_json::json!([{"address":"192.0.2.53","served":"Primary"}]);
            if updated.id == IDS[2] {
                endpoints
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({"address":"192.0.2.54","served":"Secondary"}));
            }
            assert_eq!(
                cap,
                serde_json::json!({"assignment":{"endpoints":endpoints},"example":{"endpoints":endpoints}})
            );
            assert_eq!(
                value["sources"].as_array_mut().unwrap().pop().unwrap(),
                "https://bind9.readthedocs.io/en/stable/manpages.html"
            );
        }
        assert_eq!(value, serde_json::to_value(original).unwrap());
    }
}
#[test]
fn taught_repairs_pass_observations_cannot_finish_and_examples_remain_separate() {
    for s in labs().into_iter().filter(|s| IDS.contains(&s.id.as_str())) {
        let lesson = s.dns_teaching.as_ref().unwrap();
        let lookup = lesson.lookup.as_ref().unwrap();
        let mut state = s.initial.clone();
        assert!(!engine::solved(&state, &s.goal));
        let before = state.clone();
        let origin = server(&state).primary.origin.clone();
        let input = format!("dig @192.0.2.53 {origin} NS +norecurse +short");
        let observed = lookup.assignment.observe(server(&state), &input).unwrap();
        assert_eq!(observed.status(), 0);
        assert_eq!(
            serde_json::to_value(&state).unwrap(),
            serde_json::to_value(before).unwrap()
        );
        assert!(!engine::solved(&state, &s.goal));
        let mut example = dns_teaching::Learning::new(lesson);
        for action in &lesson.example_solution {
            dlms_learning_lab::dns_workspace::adapter(&mut example.server, action).unwrap();
        }
        example.draft = Draft::new(&example.server);
        example.checked = true;
        assert!(example.checks(lesson).iter().all(|r| r.0));
        assert!(!engine::solved(&state, &s.goal));
        for action in &s.solution {
            engine::execute(&mut state, action).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let mut draft = Draft::new(server(&state));
        draft.utility = Some(UtilityDraft {
            enabled: true,
            input,
        });
        draft
            .lookup_utility(server(&state), &lookup.assignment)
            .unwrap();
        assert!(engine::solved(&state, &s.goal));
        let wrong_server = server(&state)
            .configured
            .records
            .iter()
            .find(|r| matches!(r.data, Data::A { .. }) && !r.owner.starts_with("ns."))
            .unwrap()
            .owner
            .clone();
        let protected = server_mut(&mut state)
            .configured
            .records
            .iter_mut()
            .find(|r| matches!(r.data, Data::Ns { .. }))
            .unwrap();
        protected.data = Data::Ns {
            target: wrong_server,
        };
        server_mut(&mut state).reload_primary().unwrap();
        assert!(
            !engine::solved(&state, &s.goal),
            "Broken required nameserver state must invalidate {}",
            s.id
        );
    }
}
#[test]
fn partial_and_alternate_record_repairs_keep_the_existing_behavioral_grade() {
    let ls = labs();
    let dual = ls.iter().find(|s| s.id == IDS[0]).unwrap();
    let mut state = dual.initial.clone();
    let world = server_mut(&mut state);
    world.configured.records.push(dns::Record {
        owner: "files.workshop.test.".into(),
        ttl: 300,
        data: Data::A {
            address: "192.0.2.40".parse().unwrap(),
        },
    });
    world.reload_primary().unwrap();
    assert!(!engine::solved(&state, &dual.goal));
    let aliases = ls.iter().find(|s| s.id == IDS[1]).unwrap();
    let mut state = aliases.initial.clone();
    let world = server_mut(&mut state);
    world
        .configured
        .records
        .iter_mut()
        .find(|r| r.owner == "status.studio.test.")
        .unwrap()
        .data = Data::Cname {
        target: "app.studio.test.".into(),
    };
    world.reload_primary().unwrap();
    assert!(
        engine::solved(&state, &aliases.goal),
        "One-link repair is a legitimate alternate complete path for both aliases"
    );
    let secondary = ls.iter().find(|s| s.id == IDS[2]).unwrap();
    let mut state = secondary.initial.clone();
    server_mut(&mut state).update_secondary().unwrap();
    assert!(!engine::solved(&state, &secondary.goal));
    for action in &secondary.solution {
        if action != "update-secondary" {
            engine::execute(&mut state, action).unwrap();
        }
    }
    assert!(!engine::solved(&state, &secondary.goal));
    engine::execute(&mut state, "update-secondary").unwrap();
    assert!(engine::solved(&state, &secondary.goal));
}
#[test]
fn actual_legacy_upgrade_keeps_completed_work_favorites_and_both_command_drafts() {
    let old = pack::load(include_str!("../scenarios/dns-forward-v1.json")).unwrap();
    let current = labs();
    let source = old.iter().find(|s| s.id == IDS[0]).unwrap();
    let mut a = Attempt::new(source, 5);
    workspace::prepare(&mut a);
    dns_teaching::prepare(&mut a, source.dns_teaching.as_ref().unwrap(), true);
    for action in &source.solution {
        engine::execute(&mut a.state, action).unwrap();
    }
    a.workspace.as_mut().unwrap().dns_draft = Some(Draft::new(server(&a.state)));
    assert!(progress::current_solved(&a, source));
    let mut p = Progress {
        schema_version: 1,
        clock: 5,
        ..Default::default()
    };
    p.favorites.insert(source.id.clone());
    p.attempts.insert(source.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &old).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut upgraded = progress::load(&path, &current).unwrap();
    assert_eq!(
        serde_json::to_value(&upgraded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let updated = current.iter().find(|s| s.id == source.id).unwrap();
    assert!(progress::current_solved(
        &upgraded.attempts[&source.id],
        updated
    ));
    let w = upgraded
        .attempts
        .get_mut(&source.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap();
    w.dns_draft.as_mut().unwrap().utility = Some(UtilityDraft {
        enabled: true,
        input: "host -r -t AAAA files.workshop.test. 192.0.2.53".into(),
    });
    w.dns_learning.as_mut().unwrap().draft.utility = Some(UtilityDraft {
        enabled: false,
        input: "dig @192.0.2.53 files.dual-practice.test. A +norecurse +short".into(),
    });
    progress::save_with_catalog(&path, &upgraded, &current).unwrap();
    let restored = progress::load(&path, &current).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(upgraded).unwrap()
    );
}
