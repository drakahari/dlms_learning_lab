use dlms_learning_lab::{
    dns::{self, Data, QueryType},
    dns_teaching::{self, Learning},
    dns_workspace::{self as editor, Draft, Requirement, Snapshot},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn lab() -> pack::Scenario {
    pack::load(include_str!("fixtures/dns-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn server(state: &State) -> &dns::Server {
    let State::AuthoritativeDns { server } = state else {
        panic!()
    };
    server
}
fn solved(lab: &pack::Scenario) -> State {
    let mut state = lab.initial.clone();
    for action in &lab.solution {
        engine::execute(&mut state, action).unwrap();
    }
    assert!(engine::solved(&state, &lab.goal));
    state
}
fn attempt(lab: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(lab, 1);
    workspace::prepare(&mut a);
    dns_teaching::prepare(&mut a, lab.dns_teaching.as_ref().unwrap(), true);
    a
}
#[test]
fn editor_parses_real_record_fields_roundtrips_and_keeps_save_separate_from_publication() {
    let lab = lab();
    let mut s = server(&lab.initial).clone();
    let original = s.clone();
    let mut d = Draft::new(&s);
    assert_eq!(d.parse(&s.configured.origin).unwrap(), s.configured);
    assert!(d.matches(&s));
    d.text = d.text.replace("192.0.2.8", "192.0.2.99");
    d.text.push_str("\n; harmless comment\nmx.preview.test. 60 IN AAAA 2001:db8::25\npreview.test. 60 IN MX 10 mx.preview.test.\n");
    d.save(&mut s).unwrap();
    assert!(d.matches(&s));
    assert_eq!(s.primary, original.primary);
    assert_eq!(s.secondary, original.secondary);
    assert_ne!(s.configured, original.configured);
    // The alias loop is query-diagnosable, not silently successful.
    assert_eq!(
        s.primary
            .query("portal.preview.test", QueryType::A)
            .unwrap()
            .code,
        dns::Code::Servfail
    );
    s.reload_primary().unwrap();
    assert_eq!(s.primary, s.configured);
    assert_eq!(s.update_secondary().unwrap(), dns::Transfer::UnchangedEqual);
    assert_ne!(s.primary, s.secondary);
    d.serial = "11".into();
    d.save(&mut s).unwrap();
    s.reload_primary().unwrap();
    assert_eq!(s.update_secondary().unwrap(), dns::Transfer::Updated);
    assert_eq!(s.primary, s.secondary);
}
#[test]
fn malformed_and_unsupported_editor_input_fails_atomically_with_line_feedback() {
    let lab = lab();
    let original = server(&lab.initial).clone();
    for line in [
        "bad",
        "host.preview.test 300 IN A 192.0.2.1",
        "host.preview.test. x IN A 192.0.2.1",
        "host.preview.test. 1 CH A 192.0.2.1",
        "host.preview.test. 1 IN A 999.1.1.1",
        "host.preview.test. 1 IN AAAA bad",
        "host.preview.test. 1 IN MX 65536 mx.preview.test.",
        "host.preview.test. 1 IN MX 10",
        "host.preview.test. 1 IN TXT hello",
        "$INCLUDE /etc/passwd",
        "host.preview.test. 1 IN A 192.0.2.1 extra",
        "host.preview.test. 1 IN CNAME relative",
    ] {
        let mut s = original.clone();
        let mut d = Draft::new(&s);
        d.text.push('\n');
        d.text.push_str(line);
        let e = d.save(&mut s).unwrap_err();
        assert!(e.starts_with("Line "), "{line}: {e}");
        assert_eq!(s, original);
        assert!(d.valid());
    }
    for serial in ["", "-1", "+1", "4294967296", "1.2"] {
        let mut d = Draft::new(&original);
        d.serial = serial.into();
        assert!(d.parse(&original.configured.origin).is_err());
    }
    let mut d = Draft::new(&original);
    d.text = "x".repeat(editor::MAX_TEXT + 1);
    assert!(!d.valid());
    assert!(d.parse(&original.configured.origin).is_err());
    let mut d = Draft::new(&original);
    d.text = (0..65)
        .map(|n| format!("n{n}.preview.test. 1 IN A 192.0.2.1"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        d.parse(&original.configured.origin)
            .unwrap_err()
            .contains("64")
    );
}
#[test]
fn records_aliases_publication_and_preservation_are_current_behavior_not_exact_editor_strings() {
    let lab = lab();
    let state = solved(&lab);
    let mut alternate = server(&state).clone();
    alternate.configured.records.reverse();
    alternate
        .configured
        .records
        .push(alternate.configured.records[0].clone());
    for r in &mut alternate.configured.records {
        r.owner = r.owner.to_ascii_uppercase();
        r.ttl = 60;
        if r.owner == "PORTAL.PREVIEW.TEST." {
            r.data = Data::Cname {
                target: "via.preview.test".into(),
            };
        }
    }
    alternate.configured.records.push(dns::Record {
        owner: "via.preview.test.".into(),
        ttl: 60,
        data: Data::Cname {
            target: "APP.PREVIEW.TEST".into(),
        },
    });
    alternate.configured.serial = 12;
    alternate.reload_primary().unwrap();
    alternate.update_secondary().unwrap();
    assert!(engine::solved(
        &State::AuthoritativeDns {
            server: alternate.clone()
        },
        &lab.goal
    ));
    // A portal address cannot mask a wrong canonical app or replace the requested alias.
    let mut bypass = alternate.clone();
    for r in &mut bypass.configured.records {
        if r.owner.eq_ignore_ascii_case("APP.PREVIEW.TEST.") && matches!(r.data, Data::A { .. }) {
            r.data = Data::A {
                address: "192.0.2.8".parse().unwrap(),
            };
        }
        if r.owner.eq_ignore_ascii_case("PORTAL.PREVIEW.TEST.") {
            r.data = Data::A {
                address: "192.0.2.42".parse().unwrap(),
            };
        }
    }
    bypass.configured.serial = 13;
    bypass.reload_primary().unwrap();
    bypass.update_secondary().unwrap();
    assert!(!engine::solved(
        &State::AuthoritativeDns { server: bypass },
        &lab.goal
    ));
    let engine::Goal::AuthoritativeDns { requirements } = &lab.goal else {
        panic!()
    };
    for replacement in [
        Data::A {
            address: "192.0.2.42".parse().unwrap(),
        },
        Data::Cname {
            target: "shadow.preview.test.".into(),
        },
    ] {
        let mut wrong_alias = alternate.clone();
        for r in &mut wrong_alias.configured.records {
            if r.owner.eq_ignore_ascii_case("PORTAL.PREVIEW.TEST.") {
                r.data = replacement.clone();
            }
        }
        wrong_alias.configured.records.push(dns::Record {
            owner: "shadow.preview.test.".into(),
            ttl: 60,
            data: Data::A {
                address: "192.0.2.42".parse().unwrap(),
            },
        });
        wrong_alias.configured.serial = 13;
        wrong_alias.reload_primary().unwrap();
        wrong_alias.update_secondary().unwrap();
        let failed = requirements
            .iter()
            .filter(|r| !r.check(&wrong_alias).0)
            .collect::<Vec<_>>();
        assert_eq!(failed.len(), 1);
        assert!(matches!(failed[0], Requirement::AliasTo { .. }));
    }
    // Single wrong peer, missing mail address and stale serving copy all invalidate completion.
    for name in [
        "APP.PREVIEW.TEST.",
        "mail.preview.test.",
        "ns.preview.test.",
    ] {
        let mut broken = alternate.clone();
        broken
            .configured
            .records
            .retain(|r| !r.owner.eq_ignore_ascii_case(name));
        let _ = broken.reload_primary();
        let _ = broken.update_secondary();
        assert!(!engine::solved(
            &State::AuthoritativeDns { server: broken },
            &lab.goal
        ));
    }
    let mut broken = alternate.clone();
    broken.configured.records.push(dns::Record {
        owner: "mail.preview.test.".into(),
        ttl: 300,
        data: Data::A {
            address: "192.0.2.26".parse().unwrap(),
        },
    });
    assert!(!engine::solved(
        &State::AuthoritativeDns {
            server: broken.clone()
        },
        &lab.goal
    ));
    broken.reload_primary().unwrap();
    broken.update_secondary().unwrap();
    assert!(!engine::solved(
        &State::AuthoritativeDns { server: broken },
        &lab.goal
    ));
}
#[test]
fn partial_work_failure_flags_and_unrelated_ready_targets_do_not_pass() {
    let lab = lab();
    assert!(!engine::solved(&lab.initial, &lab.goal));
    let engine::Goal::AuthoritativeDns { requirements } = &lab.goal else {
        panic!()
    };
    let mut state = lab.initial.clone();
    engine::execute(&mut state, &lab.solution[0]).unwrap();
    assert!(!engine::solved(&state, &lab.goal));
    engine::execute(&mut state, "reload-primary").unwrap();
    assert!(!engine::solved(&state, &lab.goal));
    engine::execute(&mut state, "update-secondary").unwrap();
    assert!(engine::solved(&state, &lab.goal));
    assert!(
        requirements
            .iter()
            .any(|r| matches!(r, Requirement::TargetAddress { .. }))
    );
    let wrong = Requirement::TargetAddress {
        copy: Snapshot::Primary,
        owner: "other.preview.test.".into(),
        record_type: QueryType::Mx,
        target: "mail.preview.test.".into(),
    };
    assert!(!wrong.check(server(&state)).0);
    let mut bad = server(&lab.initial).clone();
    bad.configured.records.push(dns::Record {
        owner: "outside.test.".into(),
        ttl: 1,
        data: Data::A {
            address: "192.0.2.1".parse().unwrap(),
        },
    });
    let before = bad.clone();
    assert!(bad.reload_primary().is_err());
    assert_eq!(bad, before);
}
#[test]
fn isolated_example_fill_never_checks_applies_or_mutates_assignment_drafts() {
    let lab = lab();
    let a = attempt(&lab);
    let assignment = serde_json::to_vec(&a.state).unwrap();
    let lesson = lab.dns_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson);
    let initial = l.server.clone();
    l.fill_example(lesson).unwrap();
    assert!(!l.checked);
    assert_eq!(l.server, initial);
    assert!(l.checks(lesson).is_empty());
    l.draft.save(&mut l.server).unwrap();
    l.server.reload_primary().unwrap();
    l.checked = true;
    assert!(l.checks(lesson).iter().any(|c| !c.0));
    l.server.update_secondary().unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    l.draft.text.push_str("\nunsaved malformed text");
    assert!(l.checks(lesson).iter().any(|c| !c.0));
    assert_eq!(serde_json::to_vec(&a.state).unwrap(), assignment);
    assert!(!progress::current_solved(&a, &lab));
}
#[test]
fn lookup_is_in_memory_bounded_and_names_or_copy_changes_do_not_mutate_zone_state() {
    let lab = lab();
    let s = server(&solved(&lab)).clone();
    let before = s.clone();
    let mut d = Draft::new(&s);
    d.query_name = "portal.preview.test".into();
    d.lookup(&s).unwrap();
    assert!(d.output.contains("Noerror"));
    assert!(d.output.contains("192.0.2.42"));
    d.query_name = "outside.test".into();
    d.lookup(&s).unwrap();
    assert!(d.output.contains("Refused"));
    assert!(d.output.contains("no recursive"));
    d.query_name = "/etc/passwd".into();
    assert!(d.lookup(&s).is_err());
    assert_eq!(s, before);
    // Long valid owner/RDATA rows cannot overflow a saved output field.
    let suffix = format!("{}.{}.{}.", "x".repeat(63), "y".repeat(63), "z".repeat(60));
    let mut zone = dns::Zone {
        origin: suffix.clone(),
        serial: 1,
        records: vec![],
    };
    zone.records.push(dns::Record {
        owner: suffix.clone(),
        ttl: 1,
        data: Data::Ns {
            target: format!("ns.{suffix}"),
        },
    });
    for n in 0..63 {
        zone.records.push(dns::Record {
            owner: format!("www.{suffix}"),
            ttl: 1,
            data: Data::Aaaa {
                address: format!("2001:db8::{:x}", n + 1).parse().unwrap(),
            },
        });
    }
    let s = dns::Server::new(zone).unwrap();
    let mut d = Draft::new(&s);
    d.query_name = format!("www.{suffix}");
    d.query_type = QueryType::Aaaa;
    d.lookup(&s).unwrap();
    assert!(d.valid());
    assert!(d.output.len() > 12_000);
}
#[test]
fn progress_roundtrip_retains_broken_drafts_examples_and_legacy_attempts_with_honest_completion() {
    let fixture = lab();
    let mut labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let legacy = serde_json::to_value(&p).unwrap();
    labs.push(fixture.clone());
    let mut a = attempt(&fixture);
    a.state = solved(&fixture);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().dns_draft = Some(Draft::new(server(&a.state)));
    assert!(progress::current_solved(&a, &fixture));
    a.workspace
        .as_mut()
        .unwrap()
        .dns_draft
        .as_mut()
        .unwrap()
        .text
        .push_str("\nunfinished line");
    a.workspace
        .as_mut()
        .unwrap()
        .dns_learning
        .as_mut()
        .unwrap()
        .draft
        .text
        .push_str("\nmy separate practice draft");
    assert!(!progress::current_solved(&a, &fixture));
    p.attempts.insert(fixture.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    let reread = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&reread).unwrap(), bytes);
    assert!(!progress::current_solved(
        &reread.attempts[&fixture.id],
        &fixture
    ));
    for id in legacy["attempts"].as_object().unwrap().keys() {
        assert_eq!(
            serde_json::to_value(&reread).unwrap()["attempts"][id],
            legacy["attempts"][id]
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let stored: Progress = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(serde_json::to_vec(&stored).unwrap(), bytes);
    let receipts = stored.migration_receipts.as_ref().unwrap();
    assert!(receipts.workspace);
    assert!(
        receipts
            .shell
            .contains(&dlms_learning_lab::shell::EVALUATOR_REVISION)
    );
    // Malformed Unicode is editable work, not a reason to lose unrelated progress.
    let mut malformed = progress::decode(&bytes, &labs).unwrap();
    malformed
        .attempts
        .get_mut(&fixture.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_draft
        .as_mut()
        .unwrap()
        .serial = "é".repeat(10);
    let draft = malformed.attempts[&fixture.id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_draft
        .as_ref()
        .unwrap();
    assert!(draft.valid());
    assert!(
        draft
            .parse(&server(&fixture.initial).configured.origin)
            .unwrap_err()
            .contains("decimal number")
    );
    let malformed_bytes = serde_json::to_vec(&malformed).unwrap();
    let restored = progress::decode(&malformed_bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), malformed_bytes);
    progress::save_with_catalog(&dir.path().join("malformed.json"), &restored, &labs).unwrap();
    let mut reset = progress::decode(&bytes, &labs).unwrap();
    reset.attempts.insert(fixture.id.clone(), attempt(&fixture));
    assert_eq!(reset.attempts.len(), p.attempts.len());
    assert_eq!(
        serde_json::to_value(&reset).unwrap()["attempts"][&labs[0].id],
        serde_json::to_value(&p).unwrap()["attempts"][&labs[0].id]
    );
}
#[test]
fn packs_and_imports_reject_foreign_identity_metadata_vacuous_goals_and_unknown_syntax() {
    let lab = lab();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), attempt(&lab));
    let original = serde_json::to_value(&p).unwrap();
    for field in ["configured", "primary", "secondary"] {
        let mut v = original.clone();
        v["attempts"][&lab.id]["state"]["server"][field]["origin"] = "foreign.test.".into();
        assert!(
            progress::decode(&serde_json::to_vec(&v).unwrap(), std::slice::from_ref(&lab)).is_err()
        );
    }
    let mut v = original.clone();
    let l = &mut v["attempts"][&lab.id]["workspace"]["dns_learning"]["server"];
    for field in ["configured", "primary", "secondary"] {
        l[field]["origin"] = "foreign.test.".into();
        for r in l[field]["records"].as_array_mut().unwrap() {
            let owner = r["owner"]
                .as_str()
                .unwrap()
                .replace("practice.test", "foreign.test");
            r["owner"] = owner.into();
            if let Some(t) = r["data"].get_mut("target") {
                *t = t
                    .as_str()
                    .unwrap()
                    .replace("practice.test", "foreign.test")
                    .into();
            }
        }
    }
    assert!(
        progress::decode(&serde_json::to_vec(&v).unwrap(), std::slice::from_ref(&lab)).is_err()
    );
    let text = include_str!("fixtures/dns-workspace-preview.json");
    let original: serde_json::Value = serde_json::from_str(text).unwrap();
    for goals in [
        serde_json::json!([]),
        serde_json::json!([{"kind":"Published","secondary":false}]),
        serde_json::json!([{"kind":"Resolves","copy":"Configured","name":"app.preview.test.","record_type":"A","data":[]}]),
    ] {
        let mut v = original.clone();
        v["scenarios"][0]["goal"]["requirements"] = goals;
        assert!(pack::load(&v.to_string()).is_err());
    }
    let mut v = original.clone();
    v["scenarios"][0]["dns_teaching"]["concepts"] = serde_json::json!([]);
    assert!(pack::load(&v.to_string()).is_err());
    let mut v = original;
    v["scenarios"][0]["initial"]["server"]["configured"]["records"][0]["data"]["type"] =
        "TXT".into();
    assert!(pack::load(&v.to_string()).is_err());
}
