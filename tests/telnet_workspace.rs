//! Replay/default compatibility before teaching or native Telnet integration.
use dlms_learning_lab::{
    service_exposure_workspace::{Action, Event, Scope, Session},
    telnet_client::Mode,
};
#[path = "support/service_exposure_fixture.rs"]
mod server_fixture;
#[path = "support/service_exposure_task_fixture.rs"]
mod task_fixture;
#[path = "support/service_exposure_workspace_fixture.rs"]
mod workspace_fixture;
use workspace_fixture::{action, definition, repair};
fn command(input: &str) -> Action {
    action(Event::ObserverV3 {
        input: input.into(),
    })
}

#[test]
fn absent_legacy_phase_and_rejected_telnet_history_stay_exact() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    assert!(serde_json::to_value(&s).unwrap().get("telnet").is_none());
    let input = "telnet -c 203.0.113.10 443";
    s.apply(
        &d.assignment,
        &action(Event::ProbeV2 {
            input: input.into(),
        }),
    )
    .unwrap();
    assert_eq!(s.scan_status, Some(1));
    assert!(s.last_scan.is_none());
    assert_eq!(
        s.scan_output,
        "observer $ telnet -c 203.0.113.10 443\nUnsupported command or syntax: Observer subset: nmap -n -Pn -sT -p PORT[,PORT...] SUPPLIED_IP or nc -n -v -z SUPPLIED_IP PORT. No real command, data session, listener or host target runs.\n"
    );
    let old = s.scan_output.clone();
    let bytes = serde_json::to_vec(&s).unwrap();
    let restored: Scope = serde_json::from_slice(&bytes).unwrap();
    restored.bound(&d.assignment).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    s.apply(&d.assignment, &command(input)).unwrap();
    assert!(s.scan_output.starts_with(&old));
    assert_eq!(s.telnet, Some(Mode::Connected { port: 443 }));
    assert_eq!(s.scan_status, None, "A live client has not exited0");
    assert!(!s.solved(&d.assignment));
    s.bound(&d.assignment).unwrap();
}

#[test]
fn live_local_and_exited_client_phases_replay_with_separate_pending_input() {
    let d = definition();
    let mut s = Scope::new(&d.assignment).unwrap();
    let original = s.world.clone();
    s.apply(&d.assignment, &command("telnet -c 203.0.113.10 443"))
        .unwrap();
    for event in [
        Event::Scan {
            input: "nmap -n -Pn -sT -p 443 203.0.113.10".into(),
        },
        Event::ProbeV2 {
            input: "nc -nvz 203.0.113.10 443".into(),
        },
        Event::Select { viewpoint: 1 },
    ] {
        let before = s.clone();
        assert!(s.apply(&d.assignment, &action(event)).is_err());
        assert_eq!(s, before);
    }
    for text in ["GET / HTTP/1.0", "quit", "^]"] {
        s.apply(&d.assignment, &command(text)).unwrap();
        assert_eq!(s.telnet, Some(Mode::Connected { port: 443 }));
        assert_eq!(s.scan_status, None);
        assert!(s.scan_output.contains("Unsupported"));
    }
    s.apply(&d.assignment, &action(Event::TelnetEscapeV3))
        .unwrap();
    assert_eq!(s.telnet, Some(Mode::Command { port: 443 }));
    s.apply(&d.assignment, &Action::ScanInput { text: "qui".into() })
        .unwrap();
    let restored: Scope = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    restored.bound(&d.assignment).unwrap();
    assert_eq!(restored, s);
    s.apply(&d.assignment, &command("quit")).unwrap();
    assert!(s.telnet.is_none());
    assert!(s.scan_input.is_empty());
    assert_eq!(s.scan_status, Some(0));
    assert!(s.scan_output.contains("client exited"));
    assert!(s.scan_output.contains("\ntelnet> quit\n"));
    assert!(
        !s.scan_output.contains("telnet> telnet>"),
        "Submitting a local command must consume its pending prompt, not echo it twice"
    );
    assert_eq!(s.world, original);
    assert!(!s.solved(&d.assignment));
    s.bound(&d.assignment).unwrap();
}

#[test]
fn examples_do_not_award_credit_negative_probes_preserve_correct_grades_and_changes_invalidate() {
    let d = definition();
    let mut session = Session::new(&d).unwrap();
    let original = session.assignment.clone();
    for a in [
        command("telnet -c 203.0.113.42 443"),
        action(Event::TelnetEscapeV3),
        command("quit"),
    ] {
        session.action(&d, &a, true).unwrap();
    }
    assert_eq!(session.assignment, original);
    assert!(!session.assignment.solved(&d.assignment));
    for a in repair() {
        session.action(&d, &a, false).unwrap();
    }
    assert!(session.assignment.solved(&d.assignment));
    assert_eq!(session.assignment.world, task_fixture::solve(&d.assignment));
    session
        .action(&d, &command("telnet -c 203.0.113.10 80"), false)
        .unwrap();
    assert_eq!(session.assignment.scan_status, Some(1));
    assert!(session.assignment.last_scan.is_some());
    assert!(session.assignment.solved(&d.assignment));
    session
        .action(&d, &command("telnet -c 203.0.113.10 443"), false)
        .unwrap();
    assert!(session.assignment.solved(&d.assignment));
    session
        .action(
            &d,
            &action(Event::Admin {
                input: "iptables -F INPUT".into(),
            }),
            false,
        )
        .unwrap();
    assert!(session.assignment.telnet.is_none());
    assert!(session.assignment.last_scan.is_none());
    assert!(!session.assignment.checked && !session.assignment.solved(&d.assignment));
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&session).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, session);
    let example = session.example.clone();
    session.reset(&d, false).unwrap();
    assert_eq!(session.example, example);
    assert_eq!(session.assignment, Scope::new(&d.assignment).unwrap());
}

#[test]
fn forged_or_missing_client_phase_cannot_replace_actual_history() {
    let d = definition();
    let pristine = Scope::new(&d.assignment).unwrap();
    for mode in [
        Mode::Connected { port: 443 },
        Mode::Command { port: 443 },
        Mode::Idle,
        Mode::Connected { port: 0 },
        Mode::Command { port: 23 },
    ] {
        let mut forged = pristine.clone();
        forged.telnet = Some(mode);
        assert!(forged.bound(&d.assignment).is_err());
    }
    let mut s = pristine;
    s.apply(&d.assignment, &command("telnet -c 203.0.113.10 443"))
        .unwrap();
    let mut forged = s.clone();
    forged.telnet = Some(Mode::Command { port: 443 });
    assert!(forged.bound(&d.assignment).is_err());
    let mut forged = s.clone();
    forged.scan_status = Some(0);
    assert!(forged.bound(&d.assignment).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value.as_object_mut().unwrap().remove("telnet");
    let missing: Scope = serde_json::from_value(value).unwrap();
    assert!(
        missing.bound(&d.assignment).is_err(),
        "Do not invent missing new state for new history"
    );
    let before = s.clone();
    assert!(s.apply(&d.assignment, &command(&"x".repeat(1025))).is_err());
    assert_eq!(s, before);
}

#[test]
fn version_three_nc_and_nmap_keep_the_actual_version_two_reports() {
    let d = definition();
    for input in [
        "nc -n -v -z 203.0.113.10 80",
        "nc -nvz 203.0.113.10 443",
        "nmap -n -Pn -sT -p 22,80,443 203.0.113.10",
    ] {
        let mut old = Scope::new(&d.assignment).unwrap();
        let mut new = old.clone();
        old.apply(
            &d.assignment,
            &action(Event::ProbeV2 {
                input: input.into(),
            }),
        )
        .unwrap();
        new.apply(&d.assignment, &command(input)).unwrap();
        assert_eq!(old.scan_output, new.scan_output);
        assert_eq!(old.scan_status, new.scan_status);
        assert_eq!(old.last_scan, new.last_scan);
        assert!(new.telnet.is_none());
        assert_eq!(old.world, new.world);
        new.bound(&d.assignment).unwrap();
    }
}
