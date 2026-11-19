use dlms_learning_lab::{client_names as model, client_names_workspace as w};
use std::collections::BTreeMap;
fn files(hosts: &str, nss: &str) -> w::Files {
    BTreeMap::from([
        ("hosts".into(), hosts.into()),
        ("nsswitch.conf".into(), nss.into()),
    ])
}
fn definition() -> (w::Definition, w::Files) {
    fn task(prefix: &str) -> (w::Task, w::Files) {
        let name = |n: &str| format!("{n}.{prefix}.lab.test");
        let hosts = format!("192.0.2.40 {} {}\n", name("rescue"), name("alias"));
        let worked = files(&hosts, "hosts: files dns\n");
        let mut cases = vec![];
        for (i, live) in ["192.0.2.80", "192.0.2.81"].into_iter().enumerate() {
            cases.push(w::Case {
                hostname: None,
                id: format!("available-{i}"),
                label: format!("Supplied DNS snapshot {i}"),
                context: model::Context {
                    files_available: true,
                    dns: model::Dns {
                        status: model::Status::Success,
                        records: BTreeMap::from([
                            (name("rescue"), "192.0.2.99".parse().unwrap()),
                            (name("live"), live.parse().unwrap()),
                        ]),
                    },
                },
                probes: vec![
                    w::Probe {
                        query: name("rescue"),
                        address: Some("192.0.2.40".parse().unwrap()),
                        status: Some(model::Status::Success),
                        via: Some(model::Service::Files),
                    },
                    w::Probe {
                        query: name("alias"),
                        address: Some("192.0.2.40".parse().unwrap()),
                        status: Some(model::Status::Success),
                        via: Some(model::Service::Files),
                    },
                    w::Probe {
                        query: name("live"),
                        address: Some(live.parse().unwrap()),
                        status: Some(model::Status::Success),
                        via: Some(model::Service::Dns),
                    },
                    w::Probe {
                        query: name("absent"),
                        address: None,
                        status: Some(model::Status::NotFound),
                        via: Some(model::Service::Dns),
                    },
                ],
            });
        }
        cases.push(w::Case {
            hostname: None,
            id: "dns-unavailable".into(),
            label: "DNS unavailable".into(),
            context: model::Context {
                files_available: true,
                dns: model::Dns {
                    status: model::Status::Unavailable,
                    records: BTreeMap::new(),
                },
            },
            probes: vec![
                w::Probe {
                    query: name("rescue"),
                    address: Some("192.0.2.40".parse().unwrap()),
                    status: Some(model::Status::Success),
                    via: Some(model::Service::Files),
                },
                w::Probe {
                    query: name("absent"),
                    address: None,
                    status: Some(model::Status::Unavailable),
                    via: Some(model::Service::Dns),
                },
            ],
        });
        (w::Task{mode:w::Mode::Sources,resolver:None,id:format!("sample-{prefix}"),instructions:"Keep rescue names local; resolve changing live names through supplied DNS and report unknown names honestly.".into(),starter:files(&hosts,"hosts: dns [NOTFOUND=return] files\n"),cases},worked)
    }
    let (initial, reference) = task("assignment");
    let (example, example_worked) = task("practice");
    (
        w::Definition {
            initial,
            example,
            example_worked,
        },
        reference,
    )
}
#[test]
fn taught_and_equivalent_configuration_derive_actual_source_traces() {
    let (d, reference) = definition();
    d.validate().unwrap();
    let rows = d.initial.run(&reference).unwrap();
    assert!(rows.iter().all(w::CaseResult::passed));
    assert!(rows[0].lookups.iter().any(|l| l.steps.len() == 2));
    let mut alt = reference.clone();
    alt.insert("nsswitch.conf".into(),"# defaults are explicit\nhosts: files [NOTFOUND=continue UNAVAIL=continue TRYAGAIN=continue SUCCESS=return] dns\n".into());
    assert!(
        d.initial
            .run(&alt)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn wrong_order_stopping_misses_and_hardcoded_live_hosts_fail_without_erasing_valid_results() {
    let (d, reference) = definition();
    for nss in [
        "hosts: dns files",
        "hosts: files [NOTFOUND=return] dns",
        "hosts: files",
        "hosts: dns [UNAVAIL=return] files",
    ] {
        let mut bad = reference.clone();
        bad.insert("nsswitch.conf".into(), nss.into());
        let rows = d.initial.run(&bad).unwrap();
        assert!(rows.iter().any(|r| !r.passed()), "{nss}");
        assert!(
            rows.iter().flat_map(|r| &r.checks).any(|c| c.0),
            "Retain correct per-query results: {nss}"
        );
    }
    let mut hard = reference.clone();
    hard.get_mut("hosts")
        .unwrap()
        .push_str("192.0.2.80 live.assignment.lab.test\n");
    let r = d.initial.run(&hard).unwrap();
    assert!(!r[1].passed());
    assert!(r[2].passed());
}
#[test]
fn syntax_or_unsupported_service_is_not_the_expected_unresolved_answer() {
    let (d, mut files) = definition();
    for text in [
        "hosts:",
        "hosts: files mdns4_minimal dns",
        "hosts: files [SUCCESS=merge] dns",
    ] {
        files.insert("nsswitch.conf".into(), text.into());
        let r = d.initial.run(&files).unwrap();
        assert!(
            r.iter()
                .all(|r| !r.passed() && r.diagnostic.is_some() && r.lookups.is_empty())
        );
    }
}
#[test]
fn separate_practice_and_assignment_drafts_resume_and_edits_clear_current_credit() {
    let (d, reference) = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(!d.solved(&s));
    assert_eq!(s.assignment.draft, d.initial.starter);
    s.assignment.replace(&reference).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    s.assignment.select("nsswitch.conf").unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let mut restored = w::Session::from_json(&raw, &d).unwrap();
    assert_eq!(s, restored);
    assert!(d.solved(&restored));
    restored
        .assignment
        .edit("nsswitch.conf", "hosts: dns files")
        .unwrap();
    assert!(restored.assignment.tested.is_none());
    assert!(!d.solved(&restored));
    restored.assignment.run(&d.initial).unwrap();
    assert!(!d.solved(&restored));
    restored.assignment.replace(&reference).unwrap();
    restored.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&restored));
    assert_eq!(restored.practice.draft, d.example_worked);
}
#[test]
fn unfinished_stopped_edited_and_foreign_runs_cannot_publish_credit() {
    let (d, reference) = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&reference).unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    run.cancel();
    assert!(run.results().is_none());
    assert!(!run.tick());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut run = s.assignment.begin(&d.initial).unwrap();
    for _ in 0..w::MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    assert!(run.done());
    assert!(s.assignment.finish(&d.example, &run).is_err());
    s.assignment.edit("hosts", "").unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn source_validation_bounds_unknown_paths_duplicate_data_and_unreachable_practice_refuse() {
    let (d, reference) = definition();
    let mut s = w::Session::new(&d).unwrap();
    assert!(s.assignment.edit("../../etc/hosts", "bad").is_err());
    assert!(
        s.assignment
            .edit("hosts", &"x".repeat(model::MAX_CONFIG + 1))
            .is_err()
    );
    assert_eq!(s.assignment.draft, d.initial.starter);
    let mut bad = reference.clone();
    bad.insert("../hosts".into(), "x".into());
    assert!(d.initial.run(&bad).is_err());
    let mut d2 = d.clone();
    d2.example_worked = d2.example.starter.clone();
    assert!(d2.validate().is_err());
    let mut d2 = d.clone();
    d2.example = d2.initial.clone();
    assert!(d2.validate().is_err());
    let mut d2 = d.clone();
    d2.initial.cases[0].probes[0].query = "$(hostcmd)".into();
    assert!(d2.validate().is_err());
    s.origin = "foreign-source".into();
    assert!(w::Session::from_json(&serde_json::to_string(&s).unwrap(), &d).is_err());
    assert!(w::Definition::from_json(&" ".repeat(w::MAX_DEFINITION + 1)).is_err());
    let mut raw = serde_json::to_string(&w::Session::new(&d).unwrap()).unwrap();
    raw = raw.replacen("\"draft\":{", "\"draft\":{\"hosts\":\"duplicate\",", 1);
    assert!(w::Session::from_json(&raw, &d).is_err());
}

#[test]
fn all_valid_size_drafts_can_resume_even_when_json_escaping_doubles_their_bytes() {
    let (d, _) = definition();
    let mut s = w::Session::new(&d).unwrap();
    let text = format!("#{}", "\\".repeat(model::MAX_CONFIG - 1));
    for scope in [&mut s.assignment, &mut s.practice] {
        for file in ["hosts", "nsswitch.conf"] {
            scope.edit(file, &text).unwrap();
        }
        scope.tested = Some(scope.draft.clone());
    }
    let raw = serde_json::to_string(&s).unwrap();
    assert!(raw.len() > 150_000);
    assert!(raw.len() < w::MAX_SESSION);
    assert_eq!(w::Session::from_json(&raw, &d).unwrap(), s);
    assert!(!d.solved(&s));
}
