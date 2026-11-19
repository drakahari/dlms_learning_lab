use dlms_learning_lab::{client_names_workspace as w, engine, pack};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/client-diagnostic-preview-v1.json"))
        .unwrap()
        .remove(0)
}
fn solved(s: &pack::Scenario) -> w::Files {
    serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
#[test]
fn capstone_and_separate_example_work_with_equivalent_actions_and_row_order() {
    let s = scenario();
    let d = w::source(&s).unwrap();
    let answer = solved(&s);
    assert!(
        d.initial
            .run(&answer)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    assert!(
        d.example
            .run(&d.example_worked)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    let mut other = answer.clone();
    other.insert("nsswitch.conf".into(),"# explicit defaults\nhosts : files [NOTFOUND=continue SUCCESS=return UNAVAIL=continue TRYAGAIN=continue] dns\n".into());
    let mut rows = answer["hosts"].lines().collect::<Vec<_>>();
    rows.reverse();
    other.insert("hosts".into(), rows.join("\n"));
    assert!(
        d.initial
            .run(&other)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn investigating_current_drafts_is_readonly_and_does_not_award_credit() {
    let s = scenario();
    let d = w::source(&s).unwrap();
    let mut session = w::Session::new(d).unwrap();
    let out = d
        .initial
        .diagnose(&session.assignment.draft, 0, "getent ahostsv4 report")
        .unwrap();
    assert!(out.stdout.contains("192.0.2.55"));
    assert_eq!(
        out.resolution.unwrap().answered_by,
        Some(dlms_learning_lab::client_names::Service::Files)
    );
    session.assignment.replace(&solved(&s)).unwrap();
    let before = session.clone();
    assert!(
        d.initial
            .diagnose(&session.assignment.draft, 1, "getent ahostsv4 report")
            .unwrap()
            .stdout
            .contains("192.0.2.82")
    );
    assert_eq!(session, before);
    assert!(!d.solved(&session));
    assert!(
        d.initial
            .diagnose(&session.assignment.draft, 8, "getent ahostsv4 report")
            .is_err()
    );
}
#[test]
fn partial_fixes_hardcoding_wrong_source_and_deleted_protected_names_fail() {
    let s = scenario();
    let d = w::source(&s).unwrap();
    let answer = solved(&s);
    let mut only_policy = d.initial.starter.clone();
    only_policy.insert("nsswitch.conf".into(), answer["nsswitch.conf"].clone());
    let rows = d.initial.run(&only_policy).unwrap();
    assert!(rows.iter().any(|r| !r.passed()));
    assert!(rows.iter().flat_map(|r| &r.checks).any(|c| c.0));
    for (file, value) in [
        ("nsswitch.conf", "hosts: dns files\n".to_string()),
        (
            "nsswitch.conf",
            "hosts: files [NOTFOUND=return] dns\n".to_string(),
        ),
        ("hosts", format!("{}192.0.2.80 report\n", answer["hosts"])),
        (
            "hosts",
            answer["hosts"]
                .lines()
                .filter(|l| !l.contains("rescue"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (
            "hosts",
            format!("{}192.0.2.55 old-rescue\n", answer["hosts"]),
        ),
    ] {
        let mut bad = answer.clone();
        bad.insert(file.into(), value);
        let rows = d.initial.run(&bad).unwrap();
        assert!(rows.iter().any(|r| !r.passed()), "{file}");
    }
    let mut malformed = answer.clone();
    malformed.insert("nsswitch.conf".into(), "hosts: resolve files".into());
    assert!(
        d.initial
            .run(&malformed)
            .unwrap()
            .iter()
            .all(|r| !r.passed() && r.diagnostic.is_some())
    );
}
#[test]
fn resume_keeps_both_drafts_and_query_selection_without_trusting_old_credit() {
    let s = scenario();
    let d = w::source(&s).unwrap();
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    let engine::State::ClientNames { simulation } = &mut state else {
        panic!()
    };
    simulation.assignment.diagnostic.as_mut().unwrap().input = "getent ahostsv4 report".into();
    simulation.assignment.diagnostic.as_mut().unwrap().sample = 1;
    simulation.practice.replace(&d.example_worked).unwrap();
    simulation.practice.run(&d.example).unwrap();
    let raw = serde_json::to_string(&simulation).unwrap();
    let mut restored = w::Session::from_json(&raw, d).unwrap();
    assert_eq!(**simulation, restored);
    assert!(d.solved(&restored));
    restored
        .assignment
        .edit("nsswitch.conf", "hosts: files [NOTFOUND=return] dns")
        .unwrap();
    assert!(!d.solved(&restored));
    restored.assignment.run(&d.initial).unwrap();
    assert!(!d.solved(&restored));
    restored.assignment.replace(&solved(&s)).unwrap();
    restored.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&restored));
    let before = restored.practice.clone();
    restored.assignment = w::Session::new(d).unwrap().assignment;
    assert_eq!(restored.practice, before);
    assert!(!d.solved(&restored));
}
#[test]
fn old_source_and_search_sessions_keep_exact_serialization_and_new_data_is_bounded() {
    for raw in [
        include_str!("../scenarios/client-name-lessons-v1.json"),
        include_str!("../scenarios/resolver-search-lessons-v1.json"),
    ] {
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();
        for original in value["scenarios"].as_array().unwrap() {
            let s: pack::Scenario = serde_json::from_value(original.clone()).unwrap();
            assert_eq!(serde_json::to_value(&s.goal).unwrap(), original["goal"]);
            assert_eq!(
                serde_json::to_value(&s.initial).unwrap(),
                original["initial"]
            );
            w::source(&s).unwrap();
        }
    }
    let s = scenario();
    let d = w::source(&s).unwrap();
    let session = w::Session::new(d).unwrap();
    let mut raw = serde_json::to_value(&session).unwrap();
    raw["assignment"]["diagnostic"]["input"] = serde_json::json!("x".repeat(513));
    assert!(w::Session::from_json(&raw.to_string(), d).is_err());
    let mut raw = serde_json::to_value(&session).unwrap();
    raw["assignment"]["diagnostic"]["sample"] = serde_json::json!(3);
    assert!(w::Session::from_json(&raw.to_string(), d).is_err());
    let mut bad = d.clone();
    bad.initial.resolver = Some("search north.test\noptions timeout:4".into());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.mode = w::Mode::Sources;
    assert!(bad.validate().is_err());
}
#[test]
fn cooperative_stop_and_edited_run_cannot_publish_current_credit() {
    let s = scenario();
    let d = w::source(&s).unwrap();
    let mut session = w::Session::new(d).unwrap();
    session.assignment.replace(&solved(&s)).unwrap();
    let mut run = session.assignment.begin(&d.initial).unwrap();
    run.tick();
    run.cancel();
    assert!(session.assignment.finish(&d.initial, &run).is_err());
    let mut run = session.assignment.begin(&d.initial).unwrap();
    for _ in 0..w::MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    session
        .assignment
        .edit("hosts", "127.0.0.1 localhost")
        .unwrap();
    assert!(session.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&session));
}

#[test]
fn current_catalog_integrates_capstone_after_its_prerequisites_and_preserves_all441_labs() {
    let old = pack::accepted441().unwrap();
    let current = pack::accepted442().unwrap();
    assert_eq!((old.len(), current.len()), (441, 442));
    for source in &old {
        let retained = current.iter().find(|s| s.id == source.id).unwrap();
        let mut expanded = serde_json::to_value(retained).unwrap();
        if source.id == "lpic1-host-resolver" {
            let capability = expanded["goal"]["definition"]
                .as_object_mut()
                .unwrap()
                .remove("direct_dns");
            assert_eq!(
                capability.unwrap(),
                serde_json::json!({
                    "initial":{"servers":["203.0.113.53","203.0.113.54"]},
                    "example":{"servers":["10.73.0.53","10.73.0.54"]}
                })
            );
        }
        if matches!(
            source.id.as_str(),
            "lpic2-dns-dualstack" | "lpic2-dns-aliases" | "lpic2-dns-secondary"
        ) {
            let mut endpoints = serde_json::json!([{"address":"192.0.2.53","served":"Primary"}]);
            if source.id == "lpic2-dns-secondary" {
                endpoints
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({"address":"192.0.2.54","served":"Secondary"}));
            }
            assert_eq!(
                expanded["dns_teaching"]
                    .as_object_mut()
                    .unwrap()
                    .remove("lookup")
                    .unwrap(),
                serde_json::json!({"assignment":{"endpoints":endpoints},"example":{"endpoints":endpoints}})
            );
            assert_eq!(
                expanded["sources"].as_array_mut().unwrap().pop().unwrap(),
                "https://bind9.readthedocs.io/en/stable/manpages.html"
            );
        }
        assert_eq!(
            serde_json::to_value(source).unwrap(),
            expanded,
            "{}",
            source.id
        );
    }
    let old_units = dlms_learning_lab::course::accepted441(&old).unwrap();
    let units = dlms_learning_lab::course::accepted442(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (173, 174));
    for unit in &old_units {
        let retained = units.iter().find(|u| u.id == unit.id).unwrap();
        assert_eq!(
            serde_json::to_value(unit).unwrap(),
            serde_json::to_value(retained).unwrap()
        );
    }
    let prerequisite = units
        .iter()
        .position(|u| u.id == "linux-resolver-search")
        .unwrap();
    assert_eq!(
        units[prerequisite + 1].id,
        "linux-client-name-investigation"
    );
    let lab = current
        .iter()
        .find(|s| s.id == "lpic1-client-name-investigation")
        .unwrap();
    assert_eq!(lab.prerequisites, vec!["lpic1-dns-absolute-precedence"]);
    w::source(lab).unwrap();
    for (track, count) in [
        ("CCNA", 104),
        ("LPIC-1", 146),
        ("LPIC-2", 112),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
}
