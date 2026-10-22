use dlms_learning_lab::{
    engine::{self, Goal, State},
    ipv4_routing::{self as model, InterfaceCondition, Requirement},
    ipv4_routing_teaching::{self as teaching, Learning},
    ipv4_routing_workspace::{self as editor, Draft, RouteDraft},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/ipv4-routing-preview.json"))
        .unwrap()
        .remove(0)
}
fn topology(s: &State) -> &model::Topology {
    let State::Ipv4Routing { topology } = s else {
        panic!()
    };
    topology
}
fn edit_address(d: &mut Draft, address: &str) {
    d.devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = address.into();
}
fn solved(lab: &pack::Scenario) -> State {
    let mut s = lab.initial.clone();
    for action in &lab.solution {
        engine::execute(&mut s, action).unwrap();
    }
    s
}
fn attempt(lab: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(lab, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, lab.ipv4_routing_teaching.as_ref().unwrap(), true);
    a
}
#[test]
fn native_drafts_are_atomic_bounded_and_keep_invalid_text_editable() {
    let lab = scenario();
    let original = topology(&lab.initial);
    let mut t = original.clone();
    let mut d = Draft::new(&t);
    edit_address(&mut d, "192.0.2.99/24");
    d.apply(&mut t).unwrap();
    assert!(d.matches(&t));
    let good = t.clone();
    for text in [
        "",
        "broken",
        "192.0.2.5/31",
        "192.0.2.5/32",
        "192.0.2.5/0",
        "192.0.2.5/+24",
        "192.0.2.5/-1",
        "1.1.1.1/24",
    ] {
        edit_address(&mut d, text);
        assert!(d.valid());
        if text.is_empty() {
            d.apply(&mut t).unwrap();
            assert!(
                !Requirement::AddressIn {
                    device: "Client".into(),
                    interface: "lan".into(),
                    network: model::Prefix::parse("192.0.2.0/24").unwrap(),
                    exclude: vec![]
                }
                .check(&t)
                .0
            );
            t = good.clone();
        } else {
            assert!(d.apply(&mut t).is_err(), "{text}");
            assert_eq!(t, good);
        }
        assert!(!d.matches(&good));
    }
    edit_address(&mut d, "192.0.2.99/24");
    for bad in ["192.0.2.5/24", "bad", "192.0.2.0/33"] {
        d.devices.get_mut("Client").unwrap().routes[0].destination = bad.into();
        assert!(d.apply(&mut t).is_err());
        assert_eq!(t, good);
    }
    d = Draft::new(&good);
    d.devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .remove("lan");
    assert!(d.apply(&mut t).is_err());
    assert_eq!(t, good);
    d = Draft::new(&good);
    edit_address(&mut d, &"x".repeat(33));
    assert!(!d.valid());
    d = Draft::new(&good);
    d.devices.get_mut("Client").unwrap().routes[0]
        .next_hop
        .clear();
    assert!(d.apply(&mut t).unwrap_err().contains("immediate"));
    assert_eq!(t, good);
}
#[test]
fn grading_accepts_alternate_addresses_routes_and_diagnoses_partial_reserved_or_broken_work() {
    let lab = scenario();
    assert!(!engine::solved(&lab.initial, &lab.goal));
    for host in [2, 10, 99, 254] {
        let mut state = lab.initial.clone();
        engine::execute(&mut state, &format!("address Client lan 192.0.2.{host}/24")).unwrap();
        assert!(engine::solved(&state, &lab.goal));
        engine::execute(&mut state, "remove-route Client 0").unwrap();
        engine::execute(&mut state, "route Client 198.51.100.0/24 lan 192.0.2.1 17").unwrap();
        assert!(engine::solved(&state, &lab.goal));
        engine::execute(&mut state, "forward R1 off").unwrap();
        assert!(!engine::solved(&state, &lab.goal));
    }
    for (address, issue) in [
        ("192.0.2.0/24", "network and broadcast"),
        ("192.0.2.255/24", "network and broadcast"),
        ("192.0.2.1/24", "already assigned"),
        ("192.0.2.10/16", "prefix"),
        ("198.51.100.30/24", "outside"),
    ] {
        let mut state = solved(&lab);
        engine::execute(&mut state, &format!("address Client lan {address}")).unwrap();
        assert!(!engine::solved(&state, &lab.goal));
        assert!(
            engine::checks(&state, &lab.goal)[0].1.contains(issue),
            "{address}"
        );
    }
    let mut state = solved(&lab);
    engine::execute(&mut state, "remove-route Server 0").unwrap();
    assert!(!engine::solved(&state, &lab.goal));
    assert!(
        engine::checks(&state, &lab.goal)
            .last()
            .unwrap()
            .1
            .contains("reply fails")
    );
}
#[test]
fn endpoint_binding_cannot_be_replaced_by_ownership_inferred_from_moved_addresses() {
    let lab = scenario();
    let Goal::Ipv4Routing { requirements } = &lab.goal else {
        panic!()
    };
    let r = requirements.last().unwrap();
    let mut t = topology(&solved(&lab)).clone();
    // Router owns the old server address, server is now elsewhere: raw address probe is not endpoint grading.
    t.devices
        .get_mut("Server")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = "198.51.100.77".parse().unwrap();
    t.devices
        .get_mut("R1")
        .unwrap()
        .interfaces
        .get_mut("branch")
        .unwrap()
        .address
        .as_mut()
        .unwrap()
        .address = "198.51.100.20".parse().unwrap();
    t.devices.get_mut("Server").unwrap().routes.clear();
    assert!(t.probe("Client", "198.51.100.20".parse().unwrap()).is_ok());
    assert!(!r.check(&t).0);
    t.devices
        .get_mut("Server")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .up = false;
    assert!(!r.check(&t).0);
}
#[test]
fn required_static_prefix_must_actually_serve_the_named_endpoint_and_valid_paths_stay_flexible() {
    let lab = scenario();
    let mut t = topology(&solved(&lab)).clone();
    let network = model::Prefix::parse("198.51.100.0/24").unwrap();
    let r = Requirement::StaticRouteFor {
        device: "Client".into(),
        network,
        target_device: "Server".into(),
        target_interface: "lan".into(),
    };
    assert!(!r.check(&t).0); // Default reaches server, but this specific assignment asks for a /24 static entry.
    editor::adapter(&mut t, "route Client 198.51.100.0/24 lan 192.0.2.1 70").unwrap();
    assert!(r.check(&t).0);
    editor::adapter(&mut t, "route Client 198.51.100.20/32 lan 192.0.2.1 2").unwrap();
    assert!(!r.check(&t).0); // Dummy unused /24 not enough.
    editor::adapter(&mut t, "remove-route Client 2").unwrap();
    editor::adapter(&mut t, "remove-route Server 0").unwrap();
    assert!(!r.check(&t).0);
}
#[test]
fn controlled_failure_checks_clone_state_and_invalid_pack_references_fail_closed() {
    let lab = scenario();
    let mut t = topology(&solved(&lab)).clone();
    // Two parallel local links. Normal first-hop selection and controlled withdrawal both derive from state.
    let mut client = t.devices["Client"].interfaces["lan"].clone();
    client.link = "backup".into();
    client.address.as_mut().unwrap().address = "10.20.30.10".parse().unwrap();
    let mut router = t.devices["R1"].interfaces["office"].clone();
    router.link = "backup".into();
    router.address.as_mut().unwrap().address = "10.20.30.1".parse().unwrap();
    t.devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .insert("backup".into(), client);
    t.devices
        .get_mut("R1")
        .unwrap()
        .interfaces
        .insert("backup".into(), router);
    let r = Requirement::Reachable {
        label: "Primary client link withdrawn".into(),
        from: "Client".into(),
        target_device: "Server".into(),
        target_interface: "lan".into(),
        first_hop: Some("backup".into()),
        when: vec![InterfaceCondition {
            device: "Client".into(),
            interface: "lan".into(),
            up: false,
        }],
    };
    let before = t.clone();
    assert!(!r.check(&t).0);
    assert_eq!(t, before);
    editor::adapter(&mut t, "route Client 0.0.0.0/0 backup 10.20.30.1 10").unwrap();
    let before = t.clone();
    assert!(r.check(&t).0);
    assert_eq!(t, before);
    let mut value = serde_json::to_value(pack::Pack {
        schema_version: 1,
        scenarios: vec![lab],
    })
    .unwrap();
    for (path, bad) in [
        (
            vec!["goal", "requirements", "5", "from"],
            serde_json::json!("missing"),
        ),
        (
            vec!["goal", "requirements", "5", "target_device"],
            serde_json::json!("missing"),
        ),
    ] {
        let mut invalid = value.clone();
        let field = path.last().unwrap();
        invalid["scenarios"][0]["goal"]["requirements"][5][field] = bad;
        assert!(pack::load(&invalid.to_string()).is_err());
    }
    value["scenarios"][0]["goal"]["requirements"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(pack::load(&value.to_string()).is_err());
    for input in [
        "route Client 0.0.0.0/0 unknown 192.0.2.1 1",
        "route Client 0.0.0.0/0 lan 192.0.2.1 0",
        "link Client lan maybe",
        "bash -c true",
        "address Client lan $(id)",
        &"x".repeat(4097),
    ] {
        let before = t.clone();
        assert!(editor::adapter(&mut t, input).is_err());
        assert_eq!(t, before);
    }
}
#[test]
fn isolated_examples_preserve_assignment_drafts_across_views_and_resets() {
    let lab = scenario();
    let lesson = lab.ipv4_routing_teaching.as_ref().unwrap();
    let mut a = attempt(&lab);
    let original = serde_json::to_value(&a).unwrap();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .routing_v4_learning
        .as_mut()
        .unwrap();
    l.fill_example(lesson).unwrap();
    assert!(l.checks(lesson).is_empty());
    l.apply().unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    edit_address(&mut l.draft, "10.10.4.99/24");
    l.apply().unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    edit_address(&mut l.draft, "10.10.4.99/16");
    assert!(l.checks(lesson).is_empty());
    l.apply().unwrap();
    assert!(!l.checks(lesson).iter().all(|c| c.0));
    l.view = dlms_learning_lab::script_teaching::View::Assignment;
    assert_eq!(
        serde_json::to_value(&a).unwrap()["state"],
        original["state"]
    );
    assert_eq!(
        serde_json::to_value(&a).unwrap()["workspace"]["routing_v4_draft"],
        original["workspace"]["routing_v4_draft"]
    );
    assert!(!progress::current_solved(&a, &lab));
    let mut bad = lesson.clone();
    bad.example_solution = vec!["address Client lan 10.10.4.10/16".into()];
    assert!(bad.validate().is_err());
    let reset = Learning::new(lesson);
    assert_eq!(reset.topology, lesson.fixture);
    assert!(!reset.checked);
}
#[test]
fn save_resume_preserves_old_attempts_and_unfinished_drafts_without_stale_success() {
    let lab = scenario();
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut a = attempt(&lab);
    a.state = solved(&lab);
    a.workspace.as_mut().unwrap().routing_v4_draft = Some(Draft::new(topology(&a.state)));
    assert!(progress::current_solved(&a, &lab));
    edit_address(
        a.workspace
            .as_mut()
            .unwrap()
            .routing_v4_draft
            .as_mut()
            .unwrap(),
        "unfinished/",
    );
    assert!(!progress::current_solved(&a, &lab));
    assert!(engine::solved(&a.state, &lab.goal));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (i, s) in labs.iter().enumerate().take(labs.len() - 1) {
        p.attempts.insert(s.id.clone(), Attempt::new(s, i as u64));
    }
    p.attempts.insert(lab.id.clone(), a);
    let before = serde_json::to_value(&p).unwrap();
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), before);
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], &lab));
    loaded.attempts.insert(lab.id.clone(), attempt(&lab));
    progress::save(&path, &loaded).unwrap();
    let after = serde_json::to_value(progress::load(&path, &labs).unwrap()).unwrap();
    for s in labs.iter().take(labs.len() - 1) {
        assert_eq!(after["attempts"][&s.id], before["attempts"][&s.id]);
    }
}
#[test]
fn imports_reject_foreign_changed_topology_and_practice_shape_and_save_is_atomic() {
    let lab = scenario();
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), attempt(&lab));
    let original = serde_json::to_value(&p).unwrap();
    for section in ["state", "practice", "draft"] {
        let mut value = original.clone();
        let a = &mut value["attempts"][&lab.id];
        match section {
            "state" => {
                a["state"]["topology"]["devices"]["Client"]["interfaces"]["lan"]["link"] =
                    "foreign".into()
            }
            "practice" => {
                a["workspace"]["routing_v4_learning"]["topology"]["devices"]["Client"]["interfaces"]
                    ["lan"]["link"] = "foreign".into()
            }
            _ => {
                a["workspace"]["routing_v4_draft"]["devices"]
                    .as_object_mut()
                    .unwrap()
                    .remove("Server");
            }
        }
        assert!(
            progress::decode(&serde_json::to_vec(&value).unwrap(), &labs).is_err(),
            "{section}"
        );
    }
    let mut foreign = Attempt::new(&labs[0], 1);
    foreign.workspace = Some(p.attempts[&lab.id].workspace.clone().unwrap());
    p.attempts.insert(labs[0].id.clone(), foreign);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_err());
    p.attempts.remove(&labs[0].id);
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let d = p
        .attempts
        .get_mut(&lab.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .routing_v4_draft
        .as_mut()
        .unwrap();
    d.devices.remove("Server");
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    let mut d = Draft::new(topology(&lab.initial));
    let row = RouteDraft {
        destination: "0.0.0.0/0".into(),
        interface: "lan".into(),
        next_hop: "192.0.2.1".into(),
        distance: "1".into(),
    };
    d.devices.get_mut("Client").unwrap().routes = vec![row; 33];
    assert!(!d.valid());
}
