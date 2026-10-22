use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    switching::{self, Requirement},
    switching_teaching::{self, Learning},
    switching_workspace::{self, Draft},
    workspace,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/switching-preview.json"))
        .unwrap()
        .remove(0)
}
fn solved(lab: &pack::Scenario) -> State {
    let mut state = lab.initial.clone();
    for action in &lab.solution {
        engine::execute(&mut state, action).unwrap();
    }
    state
}
#[test]
fn editor_applies_atomically_with_clear_invalid_and_unsupported_feedback() {
    let lab = scenario();
    let State::Switching { mut fabric } = lab.initial else {
        panic!()
    };
    let original = fabric.clone();
    let mut draft = Draft::new(&fabric);
    draft
        .switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .access = "10".into();
    draft.apply(&mut fabric).unwrap();
    assert!(draft.matches(&fabric));
    let repaired = fabric.clone();
    for bad in ["", "-1", "0", "4095", "ten"] {
        draft
            .switches
            .get_mut("right")
            .unwrap()
            .ports
            .get_mut("staff")
            .unwrap()
            .access = bad.into();
        assert!(
            draft
                .apply(&mut fabric)
                .unwrap_err()
                .contains("right/staff access")
        );
        assert_eq!(fabric, repaired);
        assert!(!draft.matches(&fabric));
    }
    for bad in ["10,10", "10,,20", "1-10", "10 20", "4095", "0"] {
        let mut d = Draft::new(&fabric);
        d.switches.get_mut("left").unwrap().vlans = bad.into();
        assert!(d.apply(&mut fabric).is_err());
        assert_eq!(fabric, repaired);
    }
    let mut d = Draft::new(&fabric);
    d.switches.get_mut("left").unwrap().ports.remove("guest");
    assert!(d.apply(&mut fabric).is_err());
    assert_eq!(fabric, repaired);
    for input in [
        "show vlan",
        "ping staff-right",
        "access missing staff 10",
        "trunk left missing 99 10,20",
        "vlan left 4095 present",
        "port left staff maybe",
        &"x".repeat(4097),
    ] {
        assert!(switching_workspace::adapter(&mut fabric, input).is_err());
        assert_eq!(fabric, repaired);
    }
    assert_ne!(fabric, original);
}
#[test]
fn goals_reject_partial_disconnected_disabled_missing_active_and_one_way_isolation() {
    let lab = scenario();
    assert!(!engine::solved(&lab.initial, &lab.goal));
    let good = solved(&lab);
    assert!(engine::solved(&good, &lab.goal));
    let State::Switching { fabric } = good else {
        panic!()
    };
    let Goal::Switching { requirements } = &lab.goal else {
        panic!()
    };
    let isolation = Requirement::Visibility {
        label: "No leakage".into(),
        from: "staff-left".into(),
        to: "guest-right".into(),
        visible: false,
    };
    assert!(isolation.check(&fabric).0);
    for change in 0..6 {
        let mut f = fabric.clone();
        match change {
            0 => f.hosts.get_mut("guest-right").unwrap().up = false,
            1 => f.cables.retain(|c| c.id != "guest-right"),
            2 => f.cables[0].up = false,
            3 => {
                f.switches
                    .get_mut("right")
                    .unwrap()
                    .ports
                    .get_mut("guest")
                    .unwrap()
                    .up = false
            }
            4 => {
                f.switches.get_mut("right").unwrap().vlans.remove(&20);
            }
            _ => {
                f.switches
                    .get_mut("left")
                    .unwrap()
                    .ports
                    .get_mut("uplink")
                    .unwrap()
                    .up = false;
            }
        }
        assert!(!isolation.check(&f).0, "change {change}");
    }
    let mut f = fabric.clone();
    switching_workspace::adapter(&mut f, "trunk left uplink 10 10,20,99").unwrap();
    switching_workspace::adapter(&mut f, "trunk right uplink 20 10,20,99").unwrap();
    assert!(!isolation.check(&f).0);
    assert!(switching::valid_requirements(requirements, &fabric));
    assert!(!switching::valid_requirements(
        std::slice::from_ref(&isolation),
        &fabric
    ));
    let only_staff = vec![requirements[0].clone(), isolation];
    assert!(!switching::valid_requirements(&only_staff, &fabric));
    for action in [
        "access right staff 20",
        "port left uplink down",
        "vlan right 10 absent",
        "trunk right uplink 99 20,99",
    ] {
        let mut f = fabric.clone();
        switching_workspace::adapter(&mut f, action).unwrap();
        assert!(!engine::solved(&State::Switching { fabric: f }, &lab.goal));
    }
    let mut alt = fabric.clone();
    for side in ["left", "right"] {
        switching_workspace::adapter(&mut alt, &format!("trunk {side} uplink 50 10,20")).unwrap();
    }
    assert!(engine::solved(&State::Switching { fabric: alt }, &lab.goal));
}
#[test]
fn saved_drafts_invalid_input_practice_and_credit_remain_separate_and_reset_is_selected() {
    let lab = scenario();
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    switching_teaching::prepare(&mut a, lab.switching_teaching.as_ref().unwrap(), true);
    let original = a.state.clone();
    let assignment = a.workspace.as_ref().unwrap().switching_draft.clone();
    let lesson = lab.switching_teaching.as_ref().unwrap();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .switching_learning
        .as_mut()
        .unwrap();
    l.fill_example(lesson).unwrap();
    assert!(!l.draft.matches(&l.fabric));
    l.apply().unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!(a.state, original);
    assert_eq!(a.workspace.as_ref().unwrap().switching_draft, assignment);
    assert!(!progress::current_solved(&a, &lab));
    a.state = solved(&lab);
    let State::Switching { fabric } = &a.state else {
        panic!()
    };
    a.workspace.as_mut().unwrap().switching_draft = Some(Draft::new(fabric));
    assert!(progress::current_solved(&a, &lab));
    a.workspace
        .as_mut()
        .unwrap()
        .switching_draft
        .as_mut()
        .unwrap()
        .switches
        .get_mut("right")
        .unwrap()
        .ports
        .get_mut("staff")
        .unwrap()
        .access = "oops".into();
    assert!(!progress::current_solved(&a, &lab));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, std::slice::from_ref(&lab)).unwrap();
    let a = &q.attempts[&lab.id];
    assert!(!progress::current_solved(a, &lab));
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .switching_draft
            .as_ref()
            .unwrap()
            .switches["right"]
            .ports["staff"]
            .access,
        "oops"
    );
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .switching_learning
            .as_ref()
            .unwrap()
            .checks(lesson)
            .iter()
            .all(|c| c.0)
    );
    let reset = Attempt::new(&lab, 2);
    assert_eq!(reset.state, lab.initial);
    assert!(reset.workspace.is_none());
    assert_ne!(p.attempts[&lab.id].state, reset.state);
}
#[test]
fn malformed_packs_and_tampered_profile_shape_are_rejected_before_credit() {
    let lab = scenario();
    let mut raw: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/switching-preview.json")).unwrap();
    raw["scenarios"][0]["goal"]["requirements"] = serde_json::json!([{"kind":"visibility","label":"No service","from":"staff-left","to":"guest-right","visible":false}]);
    assert!(
        pack::load(&raw.to_string())
            .unwrap_err()
            .contains("positive service")
    );
    assert!(pack::load(&" ".repeat(2_000_001)).is_err());
    assert!(progress::decode(&vec![b' '; 16_000_001], std::slice::from_ref(&lab)).is_err());
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    switching_teaching::prepare(&mut a, lab.switching_teaching.as_ref().unwrap(), true);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    let original = serde_json::to_value(&p).unwrap();
    for change in 0..5 {
        let mut v = original.clone();
        let a = &mut v["attempts"][&lab.id];
        match change {
            0 => a["state"]["fabric"]["cables"][0]["up"] = false.into(),
            1 => a["state"]["fabric"]["hosts"]["staff-left"]["up"] = false.into(),
            2 => {
                a["state"]["fabric"]["switches"]["left"]["ports"]["new"] =
                    serde_json::json!({"up":true,"mode":{"kind":"access","vlan":10}})
            }
            3 => {
                a["workspace"]["switching_draft"]["switches"]["left"]["ports"]
                    .as_object_mut()
                    .unwrap()
                    .remove("staff");
            }
            _ => a["workspace"]["switching_learning"]["fabric"]["cables"][0]["up"] = false.into(),
        };
        assert!(
            progress::decode(&serde_json::to_vec(&v).unwrap(), std::slice::from_ref(&lab)).is_err(),
            "change {change}"
        );
    }
    let mut bad_lesson = lab.switching_teaching.clone().unwrap();
    bad_lesson.example_solution = vec!["access right staff 4095".into()];
    assert!(bad_lesson.validate().is_err());
    let l = Learning::new(lab.switching_teaching.as_ref().unwrap());
    assert!(l.valid());
}
