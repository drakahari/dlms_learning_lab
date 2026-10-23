use dlms_learning_lab::{
    address_teaching::{self, Learning, Lesson},
    engine::{self, Allocation, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{Concept, View},
    workspace,
};
fn subnet() -> Lesson {
    Lesson{prerequisite_note:"No prior binary knowledge is required.".into(),concepts:vec![Concept{title:"Addresses form groups".into(),text:"The prefix describes the group size. First and last addresses identify network and broadcast in this exercise.".into()}],example_description:"A different /26 example has64 addresses per group.".into(),fixture:State::Subnet{ip:"192.168.5.139".parse().unwrap(),prefix:26,answers:Default::default(),second:Some(String::new())},example_solution:["answer network 192.168.5.128","answer broadcast 192.168.5.191","answer first 192.168.5.129","answer last 192.168.5.190","answer second 192.168.5.140"].into_iter().map(Into::into).collect(),try_change:"Choose another unused usable second address, then try the broadcast address.".into()}
}
fn allocation() -> Lesson {
    Lesson{prerequisite_note:"Use network and usable-host ideas from the first assignment.".into(),concepts:vec![Concept{title:"Make room".into(),text:"A /27 has32 total and30 usable addresses. Keep each group inside the pool, aligned and separate.".into()}],example_description:"Plan different team requirements in a separate address pool.".into(),fixture:State::Vlsm{base:"192.0.2.0".parse().unwrap(),prefix:24,allocations:vec![Allocation{name:"staff".into(),hosts:30,cidr:String::new()},Allocation{name:"visitors".into(),hosts:6,cidr:String::new()}]},example_solution:vec!["allocate staff 192.0.2.0/27".into(),"allocate visitors 192.0.2.32/29".into()],try_change:"Move the visitors’ /29 to another unused aligned block.".into()}
}
#[test]
fn both_native_math_examples_accept_alternatives_and_keep_checks_current() {
    for lesson in [subnet(), allocation()] {
        lesson.validate().unwrap();
        let mut l = Learning::new(&lesson);
        assert!(l.valid());
        l.fill_example(&lesson).unwrap();
        assert!(l.checks().is_empty());
        assert!(!l.solved());
        l.check();
        assert!(l.solved());
        match &mut l.state {
            State::Subnet { second, .. } => *second = Some("192.168.5.150".into()),
            State::Vlsm { allocations, .. } => allocations[1].cidr = "192.0.2.64/28".into(),
            _ => panic!(),
        }
        assert!(!l.solved());
        l.check();
        assert!(l.solved());
        match &mut l.state {
            State::Subnet { answers, .. } => answers[0] = "192.168.5.139".into(),
            State::Vlsm { allocations, .. } => allocations[1].cidr = "192.0.2.64/30".into(),
            _ => panic!(),
        }
        l.check();
        assert!(!l.solved());
        assert!(l.checks().iter().any(|r| r.0));
        assert!(l.checks().iter().any(|r| !r.0));
        assert!(l.valid());
    }
}
#[test]
fn changing_octet_visual_is_correct_across_prefixes_and_not_only_last_octet() {
    for prefix in 0..=30 {
        for ip in [
            "0.0.0.0",
            "255.255.255.255",
            "172.20.77.139",
            "192.168.5.139",
        ] {
            let ip = ip.parse().unwrap();
            let d = address_teaching::block_diagram(ip, prefix).unwrap();
            assert_eq!(d.octet, prefix as usize / 8);
            assert_eq!(d.later_octets, 3 - d.octet);
            assert!(d.start <= u16::from(d.value) && u16::from(d.value) <= d.end);
            assert_eq!(d.start % d.block_size, 0);
            assert_eq!(d.end - d.start + 1, d.block_size);
            assert!(d.visible.len() <= 4);
            assert!(d.visible.contains(&(d.start, d.end)));
            let expected = engine::bounds(ip, prefix).unwrap();
            assert_eq!(
                std::net::Ipv4Addr::from(expected.0).octets()[d.octet] as u16,
                d.start
            );
            assert_eq!(
                std::net::Ipv4Addr::from(expected.1).octets()[d.octet] as u16,
                d.end
            );
        }
    }
    for p in [31, 32, 33, 255] {
        assert!(address_teaching::block_diagram("192.0.2.1".parse().unwrap(), p).is_none());
    }
}
#[test]
fn old_assignment_and_numerical_practice_are_unchanged_by_example_fill_check_and_round_trip() {
    let mut labs = pack::bundled().unwrap();
    for (i, lesson) in [(0, subnet()), (1, allocation())] {
        labs[i].address_teaching = Some(lesson.clone());
        let mut a = Attempt::new(&labs[i], 1);
        workspace::prepare(&mut a);
        if i == 0 {
            workspace::start_practice(&mut a);
        }
        let original = a.state.clone();
        let numerical = a.ipv4.clone();
        address_teaching::prepare(&mut a, &lesson, true);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .address_learning
            .as_mut()
            .unwrap();
        assert_eq!(l.view, View::Introduction);
        l.fill_example(&lesson).unwrap();
        l.check();
        assert!(l.solved());
        assert_eq!(a.state, original);
        assert_eq!(a.ipv4, numerical);
        assert!(!progress::current_solved(&a, &labs[i]));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(labs[i].id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save(&path, &p).unwrap();
        let loaded = progress::load(&path, &labs).unwrap();
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            serde_json::to_value(&loaded).unwrap()
        );
    }
}
#[test]
fn malformed_teaching_imports_and_changed_fixed_problems_are_rejected() {
    let mut labs = pack::bundled().unwrap();
    let lesson = subnet();
    labs[0].address_teaching = Some(lesson.clone());
    let mut a = Attempt::new(&labs[0], 1);
    workspace::prepare(&mut a);
    address_teaching::prepare(&mut a, &lesson, true);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(labs[0].id.clone(), a);
    let original = serde_json::to_value(&p).unwrap();
    for field in ["ip", "prefix"] {
        let mut j = original.clone();
        j["attempts"][&labs[0].id]["workspace"]["address_learning"]["state"][field] =
            if field == "ip" {
                "10.0.0.1".into()
            } else {
                24.into()
            };
        assert!(progress::decode(&serde_json::to_vec(&j).unwrap(), &labs).is_err());
    }
    let mut foreign = original;
    let metadata = foreign["attempts"][&labs[0].id]["workspace"]["address_learning"].take();
    let mut a = Attempt::new(&labs[2], 1);
    workspace::prepare(&mut a);
    let mut v = serde_json::to_value(a).unwrap();
    v["workspace"]["address_learning"] = metadata;
    foreign["attempts"].as_object_mut().unwrap().clear();
    foreign["attempts"][&labs[2].id] = v;
    assert!(progress::decode(&serde_json::to_vec(&foreign).unwrap(), &labs).is_err());
    let mut bad = lesson;
    bad.example_solution = vec!["answer network 192.168.5.139".into()];
    assert!(bad.validate().is_err());
    let mut bad = Learning::new(&allocation());
    if let State::Vlsm { allocations, .. } = &mut bad.state {
        allocations[0].hosts = 1;
    }
    assert!(!address_teaching::same_problem(
        &bad.state,
        &allocation().fixture
    ));
}
#[test]
fn authored_methods_experiments_and_partial_feedback_are_reachable() {
    let labs = pack::bundled().unwrap();
    for i in [0, 1] {
        let lesson = labs[i].address_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(address_teaching::distinct_example(
            &labs[i].initial,
            &lesson.fixture
        ));
        let mut l = Learning::new(lesson);
        l.check();
        assert!(!l.solved());
        assert!(l.checks().iter().all(|r| !r.0));
        l.fill_example(lesson).unwrap();
        l.check();
        assert!(l.solved());
        if let State::Subnet { second, .. } = &mut l.state {
            *second = Some("192.168.5.150".into());
        } else if let State::Vlsm { allocations, .. } = &mut l.state {
            allocations[1].cidr = "192.0.2.64/29".into();
        }
        l.check();
        assert!(l.solved());
        if let State::Subnet { second, .. } = &mut l.state {
            *second = Some("192.168.5.191".into());
        } else if let State::Vlsm { allocations, .. } = &mut l.state {
            allocations[1].cidr = "192.0.2.8/29".into();
        }
        assert!(!l.solved());
        l.check();
        assert!(!l.solved());
    }
}
#[test]
fn fixed_original_inputs_cannot_be_lowered_to_earn_credit_and_legacy_omission_survives() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for i in [0, 1] {
        let mut a = Attempt::new(&labs[i], 1);
        workspace::prepare(&mut a);
        for action in &labs[i].solution {
            engine::execute(&mut a.state, action).unwrap();
        }
        assert!(progress::current_solved(&a, &labs[i]));
        p.attempts.insert(labs[i].id.clone(), a);
    }
    let original = serde_json::to_value(&p).unwrap();
    for field in ["ip", "prefix", "hosts", "name", "base", "row_order"] {
        let mut j = original.clone();
        let id = &labs[usize::from(!matches!(field, "ip" | "prefix"))].id;
        let state = &mut j["attempts"][id]["state"];
        match field {
            "ip" => state["ip"] = "10.0.0.5".into(),
            "prefix" => state["prefix"] = 30.into(),
            "hosts" => state["allocations"][0]["hosts"] = 1.into(),
            "name" => state["allocations"][0]["name"] = "someone-else".into(),
            "base" => state["base"] = "10.0.1.0".into(),
            "row_order" => state["allocations"].as_array_mut().unwrap().swap(0, 1),
            _ => unreachable!(),
        }
        let bad: Progress = serde_json::from_value(j.clone()).unwrap();
        assert!(!progress::current_solved(
            &bad.attempts[id],
            &labs[usize::from(!matches!(field, "ip" | "prefix"))]
        ));
        assert!(
            progress::decode(&serde_json::to_vec(&j).unwrap(), &labs).is_err(),
            "{field}"
        );
    }
    // Actual pre-workstation-2 progress: four saved answers, no new metadata, no invented answer.
    let mut old = original;
    old["attempts"][&labs[0].id]["state"]
        .as_object_mut()
        .unwrap()
        .remove("second");
    old["attempts"][&labs[0].id]
        .as_object_mut()
        .unwrap()
        .remove("workspace");
    let bytes = serde_json::to_vec(&old).unwrap();
    let loaded = progress::decode(&bytes, &labs).unwrap();
    let a = &loaded.attempts[&labs[0].id];
    assert!(progress::legacy_credit(a));
    assert!(!progress::current_solved(a, &labs[0]));
    assert!(matches!(a.state, State::Subnet { second: None, .. }));
    assert!(a.workspace.is_none());
    assert_eq!(
        serde_json::to_value(a).unwrap()["state"]["answers"],
        old["attempts"][&labs[0].id]["state"]["answers"]
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.json");
    std::fs::write(&path, &bytes).unwrap();
    let _ = progress::load(&path, &labs).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn metadata_changes_are_not_distinct_examples_and_team_names_are_bounded() {
    let labs = pack::bundled().unwrap();
    let mut same = labs[0].initial.clone();
    if let State::Subnet { second, .. } = &mut same {
        *second = None;
    }
    assert!(!address_teaching::distinct_example(&same, &labs[0].initial));
    let mut same = labs[1].initial.clone();
    if let State::Vlsm { allocations, .. } = &mut same {
        allocations.swap(0, 1);
    }
    assert!(!address_teaching::distinct_example(&same, &labs[1].initial));
    if let State::Vlsm { allocations, .. } = &mut same {
        for (n, a) in allocations.iter_mut().enumerate() {
            a.name = format!("other-{n}");
        }
    }
    assert!(!address_teaching::distinct_example(&same, &labs[1].initial));
    let mut bad = allocation();
    if let State::Vlsm { allocations, .. } = &mut bad.fixture {
        allocations[0].name = "a".repeat(81);
    }
    assert!(bad.validate().is_err());
    assert!(pack::validate_state(&bad.fixture).is_err());
    let mut a = Attempt::new(&labs[1], 1);
    a.state = bad.fixture;
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(labs[1].id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_err());
}

#[test]
fn explicit_worked_summaries_explain_math_without_grading_or_mutation() {
    for lesson in [subnet(), allocation()] {
        let mut learning = Learning::new(&lesson);
        assert!(address_teaching::worked_summary(&learning.state).is_none());
        learning.fill_example(&lesson).unwrap();
        let before = learning.clone();
        let explanation = address_teaching::worked_summary(&learning.state).unwrap();
        assert!(explanation.contains("ungraded"));
        assert_eq!(learning, before);
        assert!(!learning.solved());
        if matches!(learning.state, State::Subnet { .. }) {
            for expected in [
                "6 host bits",
                "64 addresses",
                "192.168.5.128",
                "192.168.5.191",
                "192.168.5.129",
                "192.168.5.190",
                "192.168.5.140",
            ] {
                assert!(explanation.contains(expected), "{explanation}");
            }
            if let State::Subnet { second, .. } = &mut learning.state {
                *second = Some("192.168.5.191".into());
            }
        } else {
            for expected in ["30 usable places", "6 usable places", "avoids overlap"] {
                assert!(explanation.contains(expected), "{explanation}");
            }
            if let State::Vlsm { allocations, .. } = &mut learning.state {
                allocations[1].cidr = "192.0.2.8/29".into();
            }
        }
        assert!(address_teaching::worked_summary(&learning.state).is_none());
    }
    let e = dlms_learning_lab::ipv4::Exercise {
        ip: "192.168.20.77".parse().unwrap(),
        prefix: 20,
        answers: Default::default(),
        second: Some(String::new()),
    };
    assert_eq!(e.capacity(), Some(4096));
    assert_eq!(
        e.expected().unwrap().map(|a| a.to_string()),
        [
            "192.168.16.0",
            "192.168.31.255",
            "192.168.16.1",
            "192.168.31.254"
        ]
    );
}
