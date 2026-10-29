use dlms_learning_lab::{
    acl_editor as editor, acl_routing as routing, acl_teaching, acl_workspace as aw, backup,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell, workspace,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/acl-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn network(state: &State) -> &routing::Network {
    let State::Ipv4Acl { network } = state else {
        panic!()
    };
    network
}
fn network_mut(state: &mut State) -> &mut routing::Network {
    let State::Ipv4Acl { network } = state else {
        panic!()
    };
    network
}
fn definition(s: &pack::Scenario) -> &editor::Definition {
    let Goal::Ipv4Acl { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn reference(s: &pack::Scenario) -> editor::Drafts {
    let data = s.solution[0].strip_prefix("acl-author ").unwrap();
    let aw::Action::ApplyDocuments { documents } = serde_json::from_str(data).unwrap();
    documents
}
fn prepared(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    acl_teaching::prepare(&mut a, s.acl_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = prepared(s);
    a.workspace
        .as_mut()
        .unwrap()
        .acl_draft
        .as_mut()
        .unwrap()
        .documents = reference(s);
    let w = a.workspace.as_mut().unwrap();
    w.acl_draft
        .as_mut()
        .unwrap()
        .apply(network_mut(&mut a.state))
        .unwrap();
    w.checked = true;
    assert!(progress::current_solved(&a, s));
    a
}
fn profile(s: &pack::Scenario, a: Attempt) -> Progress {
    Progress {
        schema_version: 1,
        attempts: BTreeMap::from([(s.id.clone(), a)]),
        ..Default::default()
    }
}
fn encoded(p: &Progress) -> Vec<u8> {
    serde_json::to_vec_pretty(p).unwrap()
}

#[test]
fn typed_state_pack_and_reference_are_reachable_without_bundling_a_new_lab() {
    let s = scenario();
    assert_eq!(pack::bundled().unwrap().len(), 168);
    assert!(!pack::bundled().unwrap().iter().any(|lab| lab.id == s.id));
    let mut current = s.initial.clone();
    assert!(!engine::solved(&current, &s.goal));
    for action in &s.solution {
        engine::execute(&mut current, action).unwrap();
    }
    assert!(engine::solved(&current, &s.goal));
    assert!(engine::help(&current).contains("not a full IOS"));
    assert!(engine::describe(&current).contains("Virtual ACL"));
    let bytes = serde_json::to_vec(&current).unwrap();
    assert!(engine::execute(&mut current, "ip access-list extended unsafe").is_err());
    assert_eq!(serde_json::to_vec(&current).unwrap(), bytes);
    assert_eq!(network(&s.initial), &definition(&s).initial);
    assert!(
        s.acl_teaching
            .as_ref()
            .unwrap()
            .distinct_from(network(&s.initial))
    );
}

#[test]
fn alternate_policies_partial_and_wrong_work_are_graded_by_fresh_outcomes() {
    let s = scenario();
    let explicit = "ip access-list extended alternative\n deny tcp any any eq 22\n permit ip any any\nexit\ninterface office\n ip access-group alternative in";
    let alternatives = [
        reference(&s),
        editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([("Gateway".into(), explicit.into())]),
        },
    ];
    for documents in alternatives {
        let candidate = aw::apply_documents(network(&s.initial), &documents).unwrap();
        assert!(
            editor::grade(definition(&s), &candidate, &editor::Drafts::default())
                .unwrap()
                .complete
        );
    }
    for text in [
        "ip access-list extended x\n deny ip any any\nexit\ninterface office\n ip access-group x in",
        "ip access-list extended x\n permit ip any any\nexit\ninterface office\n ip access-group x in",
        "ip access-list extended x\n permit tcp any any eq 443\nexit\ninterface office\n ip access-group x in",
        "ip access-list extended x\n permit tcp any any eq 443\n permit udp any any eq 53\nexit\ninterface office\n ip access-group x out",
    ] {
        let documents = editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([("Gateway".into(), text.into())]),
        };
        let candidate = aw::apply_documents(network(&s.initial), &documents).unwrap();
        let checks = aw::checks(&candidate, definition(&s), &editor::Drafts::default());
        assert!(checks.iter().any(|c| !c.0));
        if !text.contains("deny ip") && text.contains("permit tcp") && !text.contains(" out") {
            assert!(checks.iter().any(|c| c.0));
        }
    }
}

#[test]
fn selected_and_global_raw_changes_cannot_retain_cached_success() {
    let s = scenario();
    let mut a = solved(&s);
    let saved = serde_json::to_vec(&a.state).unwrap();
    let w = a.workspace.as_mut().unwrap();
    w.checked = true;
    let text = w
        .acl_draft
        .as_mut()
        .unwrap()
        .documents
        .routers
        .get_mut("Gateway")
        .unwrap();
    let good = text.clone();
    *text = good.replace("eq 443", "eq 444");
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(serde_json::to_vec(&a.state).unwrap(), saved);
    a.workspace
        .as_mut()
        .unwrap()
        .acl_draft
        .as_mut()
        .unwrap()
        .documents
        .routers
        .insert("Gateway".into(), good);
    assert!(progress::current_solved(&a, &s));
    let w = a.workspace.as_mut().unwrap();
    w.acl_draft
        .as_mut()
        .unwrap()
        .documents
        .routers
        .get_mut("Gateway")
        .unwrap()
        .clear();
    w.acl_draft
        .as_mut()
        .unwrap()
        .apply(network_mut(&mut a.state))
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
    assert!(engine::checks(&a.state, &s.goal).iter().any(|c| !c.0));
}

fn add_router(n: &mut routing::Network, id: &str) {
    n.topology.devices.insert(
        id.into(),
        dlms_learning_lab::ipv4_routing::Device {
            forwarding: true,
            interfaces: BTreeMap::from([(
                "spare".into(),
                dlms_learning_lab::ipv4_routing::Interface {
                    link: format!("wire-{id}"),
                    up: false,
                    address: None,
                },
            )]),
            routes: vec![],
        },
    );
}
#[test]
fn global_apply_moves_full_capacity_policies_without_intermediate_order_rejection() {
    let s = scenario();
    let mut n = network(&s.initial).clone();
    add_router(&mut n, "ARouter");
    add_router(&mut n, "ZRouter");
    let eight = (0..8)
        .map(|i| {
            format!(
                "ip access-list standard list{i}\n{}exit\n",
                " permit any\n".repeat(64)
            )
        })
        .collect::<String>();
    n = aw::apply_documents(
        &n,
        &editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([("ZRouter".into(), eight.clone())]),
        },
    )
    .unwrap();
    let mut fixed = definition(&s).clone();
    fixed.initial = n.clone();
    fixed.validate().unwrap();
    assert!(editor::apply_router(&fixed, &n, "ARouter", &eight).is_err());
    let documents = editor::Drafts {
        schema_version: 1,
        routers: BTreeMap::from([("ARouter".into(), eight), ("ZRouter".into(), String::new())]),
    };
    let candidate = aw::apply_documents(&n, &documents).unwrap();
    assert!(!candidate.devices.contains_key("ZRouter"));
    assert_eq!(candidate.devices["ARouter"].policy.lists.len(), 8);
    assert_eq!(
        candidate.devices["ARouter"]
            .policy
            .lists
            .values()
            .map(|l| l.rules.len())
            .sum::<usize>(),
        512
    );
    assert_eq!(candidate.topology, n.topology);
    let before = serde_json::to_vec(&n).unwrap();
    let mut malformed = documents.clone();
    malformed.routers.insert(
        "ZRouter".into(),
        "interface spare\n unsupported syntax".into(),
    );
    assert!(aw::apply_documents(&n, &malformed).is_err());
    assert_eq!(serde_json::to_vec(&n).unwrap(), before);
}

#[test]
fn teaching_fill_only_apply_try_and_resume_do_not_touch_assignment_or_earn_its_credit() {
    let s = scenario();
    let lesson = s.acl_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    let mut a = prepared(&s);
    let main = serde_json::to_value(&a.state).unwrap();
    let main_draft =
        serde_json::to_value(a.workspace.as_ref().unwrap().acl_draft.as_ref().unwrap()).unwrap();
    let learning = a.workspace.as_mut().unwrap().acl_learning.as_mut().unwrap();
    let initial = learning.network.clone();
    learning.view = View::Example;
    learning.fill_example(lesson).unwrap();
    assert_eq!(learning.network, initial);
    assert!(!learning.checked);
    assert!(learning.checks(lesson).is_empty());
    assert!(!learning.draft.matches(&learning.network));
    assert!(learning.apply_check(lesson).unwrap().iter().all(|c| c.0));
    let worked = learning.network.clone();
    let changed = aw::apply_documents(&worked, &lesson.try_change.documents).unwrap();
    assert_ne!(changed, worked);
    let report = editor::grade(&lesson.definition, &changed, &editor::Drafts::default()).unwrap();
    for c in report.cases {
        assert_eq!(lesson.try_change.expected[&c.id], c.passed);
    }
    learning
        .draft
        .documents
        .routers
        .get_mut("DemoGateway")
        .unwrap()
        .push_str("not finished\n");
    assert!(learning.checks(lesson).iter().any(|c| !c.0));
    assert_eq!(serde_json::to_value(&a.state).unwrap(), main);
    assert_eq!(
        serde_json::to_value(a.workspace.as_ref().unwrap().acl_draft.as_ref().unwrap()).unwrap(),
        main_draft
    );
    assert!(!progress::current_solved(&a, &s));
    let p = profile(&s, a);
    let bytes = encoded(&p);
    let loaded = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
}

#[test]
fn invalid_packs_reject_unpinned_wrong_type_missing_and_false_teaching_metadata() {
    let base: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/acl-workspace-preview.json")).unwrap();
    for change in 0..5 {
        let mut bad = base.clone();
        let s = &mut bad["scenarios"][0];
        match change {
            0 => {
                s.as_object_mut().unwrap().remove("acl_teaching");
            }
            1 => {
                s["goal"]["definition"]["initial"]["topology"]["devices"]["OfficePC"]["interfaces"]
                    ["lan"]["address"]["address"] = serde_json::json!("192.0.2.11")
            }
            2 => {
                s["acl_teaching"]["try_change"]["expected"]["demo-blocked"] =
                    serde_json::json!(true)
            }
            3 => {
                s["acl_teaching"]["try_change"]["documents"] =
                    s["acl_teaching"]["example_documents"].clone()
            }
            4 => s["goal"] = serde_json::json!({"kind":"Network","address":"192.0.2.10"}),
            _ => unreachable!(),
        }
        assert!(pack::load(&bad.to_string()).is_err(), "change {change}");
    }
    let mut bad = base.clone();
    let s = &mut bad["scenarios"][0];
    let reference = reference(&scenario());
    s["acl_teaching"]["fixture"] = s["initial"]["network"].clone();
    s["acl_teaching"]["definition"] = s["goal"]["definition"].clone();
    s["acl_teaching"]["example_documents"] = serde_json::to_value(reference).unwrap();
    s["acl_teaching"]["try_change"]["documents"] =
        serde_json::json!({"schema_version":1,"routers":{"Gateway":""}});
    s["acl_teaching"]["try_change"]["expected"] =
        serde_json::json!({"main-web":true,"main-dns":true,"main-blocked":false});
    let lesson: acl_teaching::Lesson = serde_json::from_value(s["acl_teaching"].clone()).unwrap();
    lesson.validate().unwrap();
    assert!(
        pack::load(&bad.to_string()).is_err(),
        "otherwise valid but copied example must not reuse assignment"
    );
}

#[test]
fn imported_assignment_example_and_foreign_workspace_attacks_fail_without_file_changes() {
    let s = scenario();
    let p = profile(&s, prepared(&s));
    let good = encoded(&p);
    progress::decode(&good, std::slice::from_ref(&s)).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &good).unwrap();
    for change in 0..5 {
        let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
        let a = &mut bad["attempts"][&s.id];
        match change {
            0 => {
                a["state"]["network"]["topology"]["devices"]["OfficePC"]["interfaces"]["lan"]["address"]
                    ["address"] = serde_json::json!("192.0.2.11")
            }
            1 => {
                a["workspace"]["acl_draft"]["documents"]["routers"]["Unknown"] =
                    serde_json::json!("")
            }
            2 => {
                a["workspace"]["acl_draft"]["documents"]["routers"]
                    .as_object_mut()
                    .unwrap()
                    .remove("Gateway");
            }
            3 => {
                a["workspace"]["acl_learning"]["network"]["topology"]["devices"]["DemoPC"]["interfaces"]
                    ["lan"]["address"]["address"] = serde_json::json!("203.0.113.11")
            }
            4 => a["workspace"]["acl_learning"]["version"] = serde_json::json!(2),
            _ => unreachable!(),
        }
        assert!(
            progress::decode(&serde_json::to_vec(&bad).unwrap(), std::slice::from_ref(&s)).is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), good);
    }
    let old = pack::bundled().unwrap().remove(0);
    let mut foreign = Attempt::new(&old, 1);
    workspace::prepare(&mut foreign);
    foreign.workspace.as_mut().unwrap().acl_draft = p.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .acl_draft
        .clone();
    let foreign = profile(&old, foreign);
    assert!(progress::save(&path, &foreign).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), good);
}

#[test]
fn all_bundled_attempts_current_runs_real6_history_and_malformed_acl_drafts_survive() {
    let s = scenario();
    let mut labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), 168);
    let mut p = Progress {
        schema_version: 1,
        clock: 99,
        ..Default::default()
    };
    for lab in &labs {
        p.attempts.insert(lab.id.clone(), Attempt::new(lab, 1));
        p.favorites.insert(lab.id.clone());
    }
    let actual: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap();
    let sample = &actual[0];
    let id = sample["id"].as_str().unwrap();
    let historic: shell::Record = serde_json::from_value(sample["record"].clone()).unwrap();
    assert_eq!(historic.engine_revision, 6);
    let lab = labs.iter().find(|s| s.id == id).unwrap();
    let Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let a = p.attempts.get_mut(id).unwrap();
    a.state = State::Script {
        source: historic.source.clone(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: historic.source.clone(),
            results: cases
                .iter()
                .map(|c| shell::run(&historic.source, c))
                .collect(),
        }),
    };
    workspace::prepare(a);
    a.workspace.as_mut().unwrap().script_history = Some(historic.clone());
    let expected = serde_json::to_value(&p.attempts).unwrap();
    let mut acl = prepared(&s);
    acl.workspace
        .as_mut()
        .unwrap()
        .acl_draft
        .as_mut()
        .unwrap()
        .documents
        .routers
        .insert("Gateway".into(), "unfinished ü configuration\n".into());
    acl.workspace
        .as_mut()
        .unwrap()
        .acl_learning
        .as_mut()
        .unwrap()
        .draft
        .documents
        .routers
        .insert("DemoGateway".into(), "separate example draft\n".into());
    p.attempts.insert(s.id.clone(), acl);
    labs.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let raw = encoded(&p);
    std::fs::write(&path, &raw).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(
            path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION))
        )
        .unwrap(),
        raw
    );
    let loaded = progress::load(&path, &labs).unwrap();
    assert!(
        loaded
            .migration_receipts
            .as_ref()
            .unwrap()
            .shell
            .contains(&shell::EVALUATOR_REVISION)
    );
    for (id, want) in expected.as_object().unwrap() {
        assert_eq!(serde_json::to_value(&loaded.attempts[id]).unwrap(), *want);
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &labs).unwrap();
    let checked = backup::inspect(
        &export,
        &labs,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let target = dir.path().join("restored.json");
    backup::restore(&target, &checked, &labs).unwrap();
    let restored = progress::load(&target, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
    assert_eq!(
        restored.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .script_history
            .as_ref()
            .unwrap(),
        &historic
    );
    let mut reset = loaded;
    reset.attempts.insert(s.id.clone(), prepared(&s));
    for (id, want) in expected.as_object().unwrap() {
        assert_eq!(serde_json::to_value(&reset.attempts[id]).unwrap(), *want);
    }
    assert!(!progress::current_solved(&reset.attempts[&s.id], &s));
    assert!(
        !reset.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .acl_learning
            .as_ref()
            .unwrap()
            .checked
    );
}

#[test]
fn feedback_names_visible_rule_positions_sequences_and_implicit_denial() {
    let s = scenario();
    for (rules, expected) in [
        (
            "permit tcp any any eq 443\ndeny tcp any any eq 22\npermit udp any any eq 53\n",
            "via rule 2",
        ),
        (
            "20 permit udp any any eq 53\n10 deny tcp any any eq 22\n5 permit tcp any any eq 443\n",
            "via sequence 10 (rule 2)",
        ),
        (
            "permit tcp any any eq 443\npermit udp any any eq 53\n",
            "implicit deny (no rule matched)",
        ),
    ] {
        let documents = editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([(
                "Gateway".into(),
                format!(
                    "ip access-list extended services\n{rules}exit\ninterface office\n ip access-group services in\nexit\n"
                ),
            )]),
        };
        let applied = aw::apply_documents(network(&s.initial), &documents).unwrap();
        let checks = aw::checks(&applied, definition(&s), &documents);
        assert!(checks.iter().all(|c| c.0), "{checks:?}");
        let feedback = &checks
            .iter()
            .find(|c| c.1.contains("denied as required"))
            .unwrap()
            .1;
        assert!(feedback.contains(expected), "{feedback}");
        assert!(!feedback.contains("row-"), "{feedback}");
        assert!(!feedback.contains("seq-"), "{feedback}");
        assert!(feedback.contains("Gateway/office in"), "{feedback}");
    }
}

#[test]
fn feedback_distinguishes_blocked_required_service_from_wrong_boundary_and_guarded_denial() {
    let s = scenario();
    for (text, needles) in [
        (
            "ip access-list extended x\n permit tcp any any eq 443\nexit\ninterface office\n ip access-group x in",
            vec![
                "This flow must be allowed",
                "Required allowed traffic also fails",
            ],
        ),
        (
            "ip access-list extended x\n permit tcp any any eq 443\n permit udp any any eq 53\nexit\ninterface service\n ip access-group x out",
            vec!["the task requires Gateway/office in"],
        ),
    ] {
        let documents = editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([("Gateway".into(), text.into())]),
        };
        let applied = aw::apply_documents(network(&s.initial), &documents).unwrap();
        let checks = aw::checks(&applied, definition(&s), &documents);
        assert!(!checks.iter().all(|c| c.0));
        for needle in needles {
            assert!(
                checks.iter().any(|c| !c.0 && c.1.contains(needle)),
                "{checks:?}"
            );
        }
    }
}
