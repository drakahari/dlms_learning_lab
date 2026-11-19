//! Fixed pure simulation fixtures, never a host dig/network comparison.
use dlms_learning_lab::{engine::Goal, host_direct_dns as dns, host_workspace as work, pack};

fn fixture() -> (work::Definition, pack::Scenario) {
    let scenario = pack::load(include_str!("../scenarios/linux-host-lessons-v1.json"))
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic1-host-resolver")
        .unwrap();
    let Goal::HostNetwork { definition } = &scenario.goal else {
        panic!()
    };
    let mut definition = (**definition).clone();
    definition.direct_dns = Some(dns::Lesson {
        initial: dns::Plan {
            servers: definition
                .initial
                .profile
                .resolvers
                .iter()
                .map(|r| r.address)
                .collect(),
        },
        example: dns::Plan {
            servers: definition
                .example
                .profile
                .resolvers
                .iter()
                .map(|r| r.address)
                .collect(),
        },
    });
    definition.validate().unwrap();
    (definition, scenario)
}
fn command(input: &str) -> work::Event {
    work::Event::Command {
        input: input.into(),
    }
}
fn passes(d: &work::Definition, s: &work::Session) -> bool {
    d.checks(s, false).iter().all(|r| r.0)
}

#[test]
fn direct_queries_bypass_bad_config_compare_both_actual_snapshots_and_do_not_repair() {
    let (d, _) = fixture();
    let mut s = work::Session::new(&d);
    let before = s.clone();
    for (server, name, expected) in [
        ("203.0.113.53", "service.lab.test.", "203.0.113.80"),
        ("203.0.113.54", "service.lab.test.", "203.0.113.90"),
        ("203.0.113.53", "inventory.lab.test.", "203.0.113.90"),
        ("203.0.113.54", "inventory.lab.test.", "203.0.113.80"),
    ] {
        let out = s
            .run(
                &d,
                &command(&format!("dig @{server} {name} A +short")),
                false,
            )
            .unwrap();
        assert!(out.starts_with(&format!("{expected}\n")), "{out}");
        assert!(out.contains("command exit status 0"));
        assert!(out.contains(server));
        assert_eq!(s, before);
        assert!(!passes(&d, &s));
    }
    assert!(s.assignment.world.resolve("service.lab.test").is_err());
}

#[test]
fn supported_equivalents_case_and_whitespace_derive_the_same_a_value() {
    let (d, _) = fixture();
    let world = &d.initial;
    let plan = &d.direct_dns.as_ref().unwrap().initial;
    for input in [
        "dig @203.0.113.53 service.lab.test. A +short",
        "dig +short @203.0.113.53 service.lab.test. A",
        "dig @203.0.113.53 service.lab.test. -t A +short",
        " dig\t+short @203.0.113.53 SERVICE.LAB.TEST. -t a ",
    ] {
        let o = plan.observe(world, input).unwrap();
        assert_eq!(o.address.unwrap().to_string(), "203.0.113.80");
        assert_eq!(o.name, "service.lab.test");
    }
    let mut changed = world.clone();
    changed.profile.resolvers[0].records[0].address = "203.0.113.90".parse().unwrap();
    assert_eq!(
        plan.observe(&changed, "dig @203.0.113.53 service.lab.test. A +short")
            .unwrap()
            .address
            .unwrap()
            .to_string(),
        "203.0.113.90"
    );
}

#[test]
fn unavailable_path_is_not_an_invented_negative_dns_response_or_timed_out_process() {
    let (d, _) = fixture();
    let mut s = work::Session::new(&d);
    s.run(&d, &command("ip link set dev ens3 down"), false)
        .unwrap();
    let o = d
        .direct_dns
        .as_ref()
        .unwrap()
        .initial
        .observe(
            &s.assignment.world,
            "dig @203.0.113.53 service.lab.test. A +short",
        )
        .unwrap();
    assert!(o.address.is_none());
    let output = o.output();
    assert!(output.contains("no reply evaluated"));
    assert!(output.contains("no utility exit status is inferred"));
    assert!(!output.contains("NXDOMAIN"));
    assert!(!output.contains("command exit status 0"));
    assert!(!passes(&d, &s));
}

#[test]
fn unsupported_forms_missing_facts_and_escape_attempts_refuse_without_mutation() {
    let (d, _) = fixture();
    let mut s = work::Session::new(&d);
    let before = s.clone();
    for input in [
        "dig",
        "dig @203.0.113.53 service.lab.test. A",
        "dig @203.0.113.53 service.lab.test A +short",
        "dig @203.0.113.53 missing.lab.test. A +short",
        "dig @203.0.113.53 service.lab.test. AAAA +short",
        "dig @203.0.113.99 service.lab.test. A +short",
        "dig @1.1.1.1 service.lab.test. A +short",
        "dig @resolver.lab.test service.lab.test. A +short",
        "dig @203.0.113.53 service.lab.test. A +short +trace",
        "dig @203.0.113.53 service.lab.test. A +noall +answer",
        "dig @203.0.113.53 service.lab.test. A +short;id",
        "dig @203.0.113.53 $(id).lab.test. A +short",
        "dig @203.0.113.53 ../../a.lab.test. A +short",
        "dig @203.0.113.53 service.lab.test. A +short\nid",
        "dig @203.0.113.53 service.lab.test.. A +short",
    ] {
        assert!(s.run(&d, &command(input), false).is_err(), "{input}");
        assert_eq!(s, before, "{input}");
    }
    assert!(dns::Query::parse(&"x".repeat(513)).is_err());
    assert!(dns::Query::parse("dig @203.0.113.53 café.lab.test. A +short").is_err());
    let mut bad = d.clone();
    bad.direct_dns
        .as_mut()
        .unwrap()
        .initial
        .servers
        .push("203.0.113.53".parse().unwrap());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.direct_dns.as_mut().unwrap().initial.servers.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.direct_dns.as_mut().unwrap().initial.servers[0] = "203.0.113.99".parse().unwrap();
    assert!(bad.validate().is_err());
}

#[test]
fn repairs_alternatives_partial_work_practice_reset_and_resume_keep_existing_outcomes() {
    let (d, old) = fixture();
    let mut s = work::Session::new(&d);
    let original_assignment = s.assignment.clone();
    let ex = s
        .run(
            &d,
            &command("dig @10.73.0.54 service.lab.test. A +short"),
            true,
        )
        .unwrap();
    assert!(ex.starts_with("10.73.0.90\n"));
    assert_eq!(s.assignment, original_assignment);
    s.run(
        &d,
        &work::Event::Resolver {
            text: "nameserver 10.73.0.53\n".into(),
        },
        true,
    )
    .unwrap();
    assert!(d.checks(&s, true).iter().all(|r| r.0));
    assert!(!passes(&d, &s));
    s.run(
        &d,
        &work::Event::Resolver {
            text: "nameserver 203.0.113.54\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!passes(&d, &s));
    assert!(d.checks(&s, false).iter().any(|r| r.0));
    for text in [
        "nameserver 203.0.113.53\n",
        "# selected after comparing both answers\nnameserver 203.0.113.53\nnameserver 203.0.113.54\n",
    ] {
        s.run(&d, &work::Event::Resolver { text: text.into() }, false)
            .unwrap();
        assert!(passes(&d, &s));
        s.assignment.checked = true;
        let before = s.clone();
        s.run(
            &d,
            &command("dig @203.0.113.54 service.lab.test. A +short"),
            false,
        )
        .unwrap();
        assert_eq!(s, before);
        assert!(passes(&d, &s));
        let serialized = serde_json::to_vec(&s).unwrap();
        let restored: work::Session = serde_json::from_slice(&serialized).unwrap();
        d.bound(&restored).unwrap();
        assert_eq!(restored, s);
    }
    s.assignment.resolver_draft = "nameserver 203.0.113.54\n".into();
    assert!(!passes(&d, &s));
    s.run(
        &d,
        &work::Event::Resolver {
            text: "nameserver 203.0.113.54\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!s.assignment.checked);
    assert!(!passes(&d, &s));
    let Goal::HostNetwork { definition: legacy } = old.goal else {
        panic!()
    };
    let old_session = work::Session::new(&legacy);
    d.bound(&old_session).unwrap();
    assert_eq!(
        serde_json::to_value(&old_session).unwrap(),
        serde_json::to_value(work::Session::new(&d)).unwrap()
    );
    assert!(
        !serde_json::to_value(&legacy)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("direct_dns")
    );
    assert_eq!(work::Session::new(&d).assignment, original_assignment);
}

#[test]
fn actual_legacy_progress_load_keeps_credit_favorite_and_unfinished_separate_drafts() {
    use dlms_learning_lab::{
        engine::State,
        progress::{self, Attempt, Progress},
    };
    let old = pack::load(include_str!("../scenarios/linux-host-lessons-v1.json")).unwrap();
    let current = pack::load(include_str!("../scenarios/linux-host-lessons-v2.json")).unwrap();
    let source = old.iter().find(|s| s.id == "lpic1-host-resolver").unwrap();
    let expanded = current.iter().find(|s| s.id == source.id).unwrap();
    let Goal::HostNetwork { definition } = &source.goal else {
        panic!()
    };
    let mut attempt = Attempt::new(source, 8);
    let State::HostNetwork { simulation } = &mut attempt.state else {
        panic!()
    };
    for action in &source.solution {
        simulation.execute_reference(definition, action).unwrap();
    }
    simulation.assignment.checked = true;
    simulation.practice.command = "getent ahostsv4 inventory.lab.test".into();
    simulation
        .practice
        .resolver_draft
        .push_str("# unfinished example\n");
    assert!(progress::current_solved(&attempt, source));
    let mut saved = Progress {
        schema_version: 1,
        clock: 8,
        ..Default::default()
    };
    saved.favorites.insert(source.id.clone());
    saved.attempts.insert(source.id.clone(), attempt);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &saved, &old).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &current).unwrap();
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(progress::current_solved(
        &loaded.attempts[&source.id],
        expanded
    ));
    let Goal::HostNetwork { definition } = &expanded.goal else {
        panic!()
    };
    let mut unfinished = loaded;
    let attempt = unfinished.attempts.get_mut(&source.id).unwrap();
    let State::HostNetwork { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation
        .assignment
        .resolver_draft
        .push_str("# unfinished assignment\n");
    simulation.assignment.command = "dig @203.0.113.54 service.lab.test. A +short".into();
    assert!(!definition.checks(simulation, false).iter().all(|r| r.0));
    progress::save_with_catalog(&path, &unfinished, &current).unwrap();
    let resumed = progress::load(&path, &current).unwrap();
    assert_eq!(
        serde_json::to_value(&unfinished).unwrap(),
        serde_json::to_value(&resumed).unwrap()
    );
    assert!(!progress::current_solved(
        &resumed.attempts[&source.id],
        expanded
    ));
}
