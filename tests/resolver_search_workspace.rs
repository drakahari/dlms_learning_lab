use dlms_learning_lab::{client_names as n, client_names_workspace as w, engine, pack};
use std::collections::BTreeMap;
fn files(text: &str) -> w::Files {
    BTreeMap::from([("resolv.conf".into(), text.into())])
}
fn definition() -> w::Definition {
    fn task(id: &str, query: &str, other: &str) -> w::Task {
        let mut cases = vec![];
        for (i, address) in ["192.0.2.80", "192.0.2.82"].into_iter().enumerate() {
            cases.push(w::Case {
                id: format!("snapshot-{i}"),
                label: format!("DNS sample {i}"),
                hostname: Some(format!("client.old{i}.lab.test")),
                context: n::Context {
                    files_available: true,
                    dns: n::Dns {
                        status: n::Status::Success,
                        records: BTreeMap::from([
                            (format!("{query}.north.lab.test"), address.parse().unwrap()),
                            (
                                format!("{query}.south.lab.test"),
                                "192.0.2.99".parse().unwrap(),
                            ),
                            (
                                format!("{other}.south.lab.test"),
                                "192.0.2.81".parse().unwrap(),
                            ),
                        ]),
                    },
                },
                probes: vec![
                    w::Probe {
                        query: query.into(),
                        address: Some(address.parse().unwrap()),
                        status: Some(n::Status::Success),
                        via: Some(n::Service::Dns),
                    },
                    w::Probe {
                        query: other.into(),
                        address: Some("192.0.2.81".parse().unwrap()),
                        status: Some(n::Status::Success),
                        via: Some(n::Service::Dns),
                    },
                ],
            });
        }
        w::Task {
            mode: w::Mode::Search,
            resolver:None,
            id: id.into(),
            instructions: "Prefer the north site and retain the supplied second-site fallback."
                .into(),
            starter: files("search south.lab.test north.lab.test\n"),
            cases,
        }
    }
    w::Definition {
        initial: task("assignment-search", "report", "asset"),
        example: task("example-search", "preview", "image"),
        example_worked: files("search north.lab.test south.lab.test\noptions ndots:1\n"),
    }
}
#[test]
fn source_defaults_serialize_exact_original_goal_and_initial_data_without_new_fields() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/client-name-lessons-v1.json")).unwrap();
    let labs = pack::load_catalog(&[
        include_str!("../scenarios/linux-host-lessons-v1.json"),
        include_str!("../scenarios/client-name-lessons-v1.json"),
    ])
    .unwrap();
    for original in raw["scenarios"].as_array().unwrap() {
        let s = labs
            .iter()
            .find(|s| s.id == original["id"].as_str().unwrap())
            .unwrap();
        assert_eq!(serde_json::to_value(&s.goal).unwrap(), original["goal"]);
        assert_eq!(
            serde_json::to_value(&s.initial).unwrap(),
            original["initial"]
        );
        let engine::Goal::ClientNames { definition } = &s.goal else {
            panic!()
        };
        assert_eq!(definition.initial.mode, w::Mode::Sources);
        assert!(
            definition
                .initial
                .cases
                .iter()
                .all(|c| c.hostname.is_none())
        );
    }
}
#[test]
fn single_excerpt_mode_grades_actual_candidate_plans_and_equivalent_supported_settings() {
    let d = definition();
    d.validate().unwrap();
    let rows = d.initial.run(&d.example_worked).unwrap();
    assert!(rows.iter().all(w::CaseResult::passed));
    assert_eq!(rows[0].searches[0].observed.attempts.len(), 1);
    assert_eq!(rows[0].searches[1].observed.attempts.len(), 2);
    for text in [
        "search NORTH.LAB.TEST. south.lab.test\n",
        "# supplied excerpt\nsearch north.lab.test south.lab.test\noptions ndots:2\n",
    ] {
        assert!(
            d.initial
                .run(&files(text))
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
    }
    let rows = d.initial.run(&d.initial.starter).unwrap();
    assert!(rows.iter().all(|r| !r.passed()));
    assert!(rows.iter().all(|r| r.checks.iter().any(|c| c.0)));
    assert!(rows[0].checks[0].1.contains("suffix order"));
}
#[test]
fn wrong_or_foreign_file_sets_unknown_directives_and_missing_context_cannot_panic_or_pass() {
    let d = definition();
    for bundle in [
        BTreeMap::from([
            ("hosts".into(), String::new()),
            ("nsswitch.conf".into(), "hosts: files dns".into()),
        ]),
        BTreeMap::from([("../resolv.conf".into(), String::new())]),
        BTreeMap::new(),
    ] {
        assert!(d.initial.run(&bundle).is_err());
    }
    for text in [
        "nameserver 192.0.2.53",
        "options ndots:-1",
        "search ../private",
    ] {
        let rows = d.initial.run(&files(text)).unwrap();
        assert!(
            rows.iter()
                .all(|r| !r.passed() && r.diagnostic.is_some() && r.searches.is_empty())
        );
    }
    let mut bad = d.clone();
    bad.initial.cases[0].hostname = None;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0].context.dns.status = n::Status::Unavailable;
    bad.initial.cases[0].context.dns.records.clear();
    assert!(bad.validate().is_err());
}
#[test]
fn search_practice_is_separate_and_resume_regrades_current_source_after_changes() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(!d.solved(&s));
    assert_eq!(s.assignment.draft, d.initial.starter);
    s.assignment.replace(&d.example_worked).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let encoded = serde_json::to_string(&s).unwrap();
    let mut got = w::Session::from_json(&encoded, &d).unwrap();
    assert_eq!(s, got);
    assert!(d.solved(&got));
    got.assignment
        .edit("resolv.conf", "search south.lab.test north.lab.test")
        .unwrap();
    assert!(got.assignment.tested.is_none());
    assert!(!d.solved(&got));
    got.assignment.run(&d.initial).unwrap();
    assert!(!d.solved(&got));
    got.assignment.replace(&d.example_worked).unwrap();
    got.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&got));
}
#[test]
fn cooperative_stop_and_source_mode_binding_reject_unfinished_or_substituted_work() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&d.example_worked).unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    run.cancel();
    assert!(!run.tick());
    assert!(run.results().is_none());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut foreign = d.clone();
    foreign.initial.mode = w::Mode::Sources;
    assert!(foreign.bound(&s).is_err());
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.draft = BTreeMap::from([
        ("hosts".into(), String::new()),
        ("nsswitch.conf".into(), "hosts: files dns".into()),
    ]);
    assert!(d.bound(&s).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn absolute_queries_are_allowed_only_in_search_mode_and_preserve_no_suffix_behavior() {
    let mut d = definition();
    for c in &mut d.initial.cases {
        c.probes.push(w::Probe {
            query: "missing.north.lab.test.".into(),
            address: None,
            status: Some(n::Status::NotFound),
            via: Some(n::Service::Dns),
        });
        c.context.dns.records.insert(
            "missing.north.lab.test.north.lab.test".into(),
            "192.0.2.99".parse().unwrap(),
        );
    }
    d.validate().unwrap();
    let rows = d.initial.run(&d.example_worked).unwrap();
    assert!(rows.iter().all(w::CaseResult::passed));
    assert_eq!(rows[0].searches.last().unwrap().observed.attempts.len(), 1);
    let mut ordinary: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/client-name-lessons-v1.json")).unwrap();
    ordinary["scenarios"][0]["goal"]["definition"]["initial"]["cases"][0]["probes"][0]["query"] =
        "dispatch.office.lab.test.".into();
    let error =
        w::Definition::from_json(&ordinary["scenarios"][0]["goal"]["definition"].to_string())
            .unwrap_err();
    assert!(error.contains("queries"), "{error}");
}
