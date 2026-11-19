//! Three actual physical cases and isolated drafts; no registered bridge lab yet.
#[path = "support/bridge_workspace.rs"]
mod fixture;
use dlms_learning_lab::{nm_workspace as w, script_teaching::View};
#[test]
fn real_starter_partial_and_complete_outcomes_teach_ip_and_bridge_membership() {
    let d = fixture::definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let rows = t.run(&t.starter).unwrap();
        assert_eq!(rows.iter().filter(|r| r.passed()).count(), 0);
        let rows = t.run(&fixture::partial(t)).unwrap();
        assert_eq!(
            rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
            [false, false, true]
        );
        assert!(rows[0].checks.iter().any(|(ok, _)| *ok));
        assert!(
            rows[0]
                .checks
                .iter()
                .any(|(ok, text)| !*ok && text.contains("blocked"))
        );
        let rows = t.run(&fixture::solved(t)).unwrap();
        assert!(rows.iter().all(w::CaseResult::passed));
    }
}
#[test]
fn equivalent_host_address_stp_and_forced_port_flags_are_valid_independent_solutions() {
    let t = fixture::task("bridge-alternates", 180);
    let mut good = fixture::solved(&t);
    good.get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=false\n[ipv4]\naddress1=10.180.0.21/24\n");
    for n in ["left.nmconnection", "right.nmconnection"] {
        good.get_mut(n)
            .unwrap()
            .push_str("\n[connection]\nautoconnect=true\n");
    }
    assert!(t.run(&good).unwrap().iter().all(w::CaseResult::passed));
}
#[test]
fn common_wrong_repairs_and_incorrect_mask_never_keep_success() {
    let d = fixture::definition();
    let mut session = w::Session::new(&d).unwrap();
    let good = fixture::solved(&d.initial);
    session.assignment.replace(&good).unwrap();
    session.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&session));
    for (file, tail) in [
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.180.0.21/25\n"),
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.180.0.60/24\n"),
        (
            "bridge.nmconnection",
            "\n[connection]\nautoconnect-ports=0\n",
        ),
        (
            "audit.nmconnection",
            "\n[ipv4]\nnever-default=false\ngateway=10.181.0.1\n",
        ),
    ] {
        let mut wrong = good.clone();
        wrong.get_mut(file).unwrap().push_str(tail);
        session.assignment.replace(&wrong).unwrap();
        assert!(session.assignment.tested.is_none());
        assert!(!d.solved(&session));
        session.assignment.run(&d.initial).unwrap();
        assert!(!d.solved(&session));
        assert!(d.checks(&session).iter().any(|(ok, _)| !*ok));
    }
    session.assignment.replace(&good).unwrap();
    assert!(!d.solved(&session));
    session.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&session));
}
#[test]
fn practice_resume_and_reset_preserve_independent_assignment_and_bounded_state() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    assert_eq!(s.assignment.selected, "bridge.nmconnection");
    let original = s.assignment.clone();
    s.view = View::Example;
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(!d.solved(&s));
    assert_eq!(s.assignment, original);
    s.assignment.replace(&fixture::partial(&d.initial)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    let loaded = w::Session::from_json(&serde_json::to_string(&s).unwrap(), &d).unwrap();
    assert_eq!(loaded, s);
    assert!(!d.solved(&loaded));
    let reset = w::Session::new(&d).unwrap();
    assert_eq!(reset.assignment.draft, d.initial.starter);
    assert!(!d.solved(&reset));
}
#[test]
fn explicit_current_source_and_practice_identities_cannot_be_invented_or_mixed() {
    let d = fixture::definition();
    d.validate().unwrap();
    let mut wrong = d.clone();
    wrong.initial.cases[0].current_bridge = None;
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[0].initially_selected.remove(0);
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[1].environment.bridge = None;
    assert!(wrong.validate().is_err());
    let mut wrong = d.clone();
    wrong.initial.cases[0]
        .requirements
        .push(w::Requirement::FutureBridgeDelivery {
            interface: "mgmt0".into(),
            from: "10.180.0.50".parse().unwrap(),
            to: "10.180.0.60".parse().unwrap(),
            delivered: true,
        });
    assert!(wrong.validate().is_err());
    let mut wrong = d;
    wrong.example = wrong.initial.clone();
    wrong.example.id = "separate-name-only".into();
    wrong.example_worked = fixture::solved(&wrong.example);
    assert!(wrong.validate().is_err());
}
#[test]
fn errors_do_not_satisfy_an_expected_blocked_path_and_stop_cannot_publish_credit() {
    let t = fixture::task("bridge-errors", 180);
    let mut wrong = fixture::solved(&t);
    wrong
        .get_mut("right.nmconnection")
        .unwrap()
        .push_str("\n[bridge-port]\nhairpin-mode=true\n");
    let rows = t.run(&wrong).unwrap();
    assert!(rows.iter().all(|r| !r.passed() && r.diagnostic.is_some()));
    let mut scope = w::Session::new(&fixture::definition()).unwrap().assignment;
    let mut run = scope.begin(&t).unwrap();
    run.tick();
    run.cancel();
    assert!(scope.finish(&t, &run).is_err());
    assert!(scope.tested.is_none());
}
