//! Prospective closed-command checks; no host RPM invocation or generated observations.
use dlms_learning_lab::{
    engine, pack, rpm_packages as core, rpm_packages_terminal as terminal,
    rpm_packages_workspace as work,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/rpm-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn context(s: &pack::Scenario) -> &work::Definition {
    let engine::Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn state(s: &pack::Scenario) -> core::State {
    let engine::State::RpmPackages { simulation } = &s.initial else {
        panic!()
    };
    simulation.clone()
}
fn out(d: &work::Definition, s: &mut core::State, c: &str) -> String {
    match terminal::execute(d, s, c) {
        terminal::Outcome::Utility {
            stdout,
            stderr,
            status,
            ..
        } => {
            assert_eq!(status, 0);
            assert!(stderr.is_empty());
            stdout
        }
        e => panic!("expected supported command: {c}: {e:?}"),
    }
}
#[test]
fn metadata_queries_are_actual_installed_or_local_sources_and_never_install() {
    let s = scenario();
    let d = context(&s);
    let mut x = state(&s);
    let before = x.clone();
    assert_eq!(
        out(d, &mut x, "rpm -q desk-client"),
        "desk-client-1.0-1.noarch\n"
    );
    assert_eq!(
        out(d, &mut x, "rpm -qp --provides /pkgs/provider-a.rpm"),
        "desk-api = 1.0-1\nink-bridge = 1.0-1\n"
    );
    assert_eq!(
        out(d, &mut x, "rpm -q --requires desk-client"),
        "desk-api >= 1.0-1\n"
    );
    assert_eq!(
        out(d, &mut x, "rpm -qf /usr/share/desk/report"),
        "desk-client-1.0-1.noarch\n"
    );
    assert_eq!(
        out(d, &mut x, "rpm -qpl /pkgs/client-a.rpm"),
        "/etc/desk.conf\n/usr/share/desk/report\n"
    );
    assert!(x == before);
}
#[test]
fn explicit_epoch_format_is_fixed_and_default_does_not_invent_epoch() {
    let s = scenario();
    let d = context(&s);
    let mut x = state(&s);
    let c = format!("rpm -q --queryformat '{}' desk-client", terminal::FORMAT);
    assert_eq!(out(d, &mut x, &c), "desk-client\t0:1.0-1.noarch\n");
    for f in ["%{NAME}", "%{NAME:expand}", "%{EPOCH}", "[%{FILENAMES}\\n]"] {
        let b = x.clone();
        assert!(matches!(
            terminal::execute(
                d,
                &mut x,
                &format!("rpm -q --queryformat '{f}' desk-client")
            ),
            terminal::Outcome::SimulatorUnsupported { .. }
        ));
        assert!(x == b);
    }
}
#[test]
fn every_unmodeled_form_is_atomic_and_has_no_fake_numeric_rpm_status() {
    let s = scenario();
    let d = context(&s);
    let mut x = state(&s);
    for c in [
        "rpm -V desk-client",
        "rpm -Uvh /pkgs/client-a.rpm",
        "rpm --root /tmp -q desk-client",
        "rpm --eval '%{NAME}'",
        "rpm -i https://example.invalid/a.rpm",
        "rpm -q desk-client; echo bad",
        "rpm -q $(id)",
        "rpm -q *",
        "rpm -q desk-client | cat",
        "rpm -e --nodeps desk-client",
        "rpm -q missing",
        "rpm -q --whatprovides missing",
        "rpm -i /pkgs/decoy-a.rpm",
    ] {
        let before = x.clone();
        let result = terminal::execute(d, &mut x, c);
        assert!(
            matches!(result, terminal::Outcome::SimulatorUnsupported { .. }),
            "{c}"
        );
        let v = serde_json::to_value(result).unwrap();
        assert!(v.get("status").is_none());
        assert!(x == before, "Rejected command changed State");
    }
    assert!(terminal::parse(&format!("{}rpm -q desk-client", " ".repeat(2049))).is_err());
}
#[test]
fn exact_reinstall_and_erase_install_set_alternatives_reach_real_outcomes() {
    let s = scenario();
    let d = context(&s);
    for commands in [
        vec![
            "rpm -i /pkgs/provider-a.rpm",
            "rpm --reinstall /pkgs/client-a.rpm",
        ],
        vec![
            "rpm -i /pkgs/provider-b.rpm",
            "rpm --reinstall /pkgs/client-a.rpm",
        ],
        vec![
            "rpm -e desk-client",
            "rpm -i /pkgs/client-a.rpm /pkgs/provider-a.rpm",
        ],
    ] {
        let mut x = state(&s);
        for c in commands {
            out(d, &mut x, c);
        }
        assert!(d.checks(&x).iter().all(|r| r.0));
        let before = x.clone();
        assert!(matches!(
            terminal::execute(d, &mut x, "rpm -e ink-bridge"),
            terminal::Outcome::SimulatorUnsupported { .. }
        ));
        assert!(x == before);
        out(d, &mut x, "rpm -e desk-client");
        assert!(!d.checks(&x).iter().all(|r| r.0));
    }
}
#[test]
fn pending_command_and_past_output_are_not_credit_and_history_is_bounded() {
    let s = scenario();
    let d = context(&s);
    let mut x = state(&s);
    let mut t = terminal::Session::default();
    t.insert("rpm -i /pkgs/provider-a.rpm").unwrap();
    let before = x.clone();
    assert_eq!(x, before);
    assert!(!d.checks(&x).iter().all(|r| r.0));
    t.run(d, &mut x).unwrap();
    t.insert("rpm --reinstall /pkgs/client-a.rpm").unwrap();
    t.run(d, &mut x).unwrap();
    assert!(d.checks(&x).iter().all(|r| r.0));
    t.insert("unrun malformed ;").unwrap();
    assert!(d.checks(&x).iter().all(|r| r.0));
    for _ in 0..40 {
        t.insert("rpm -ql desk-client").unwrap();
        t.run(d, &mut x).unwrap();
    }
    assert!(t.entries.len() <= 16);
    assert!(serde_json::to_vec(&t).unwrap().len() <= terminal::MAX_SESSION);
    let old = t.clone();
    assert!(t.insert(&"x".repeat(2049)).is_err());
    assert_eq!(t, old);
}
#[test]
fn missing_recorded_path_and_empty_manifests_are_not_fabricated_success() {
    let s = scenario();
    let mut d = context(&s).clone();
    let old = state(&s);
    let mut r = old.runtime().clone();
    r.files.remove("/usr/share/desk/report");
    let mut x = core::State::new(old.catalog().clone(), r).unwrap();
    assert!(matches!(
        terminal::execute(&d, &mut x, "rpm -qf /usr/share/desk/report"),
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
    assert_eq!(
        out(&d, &mut x, "rpm -q --path /usr/share/desk/report"),
        "desk-client-1.0-1.noarch\n"
    );
    let mut c = old.catalog().clone();
    c.archives.get_mut("decoy-a").unwrap().files.clear();
    d.initial = core::State::new(c, old.runtime().clone()).unwrap();
    let mut x = d.initial.clone();
    assert!(matches!(
        terminal::execute(&d, &mut x, "rpm -qpl /pkgs/decoy-a.rpm"),
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
}
#[test]
fn direct_session_import_rejects_overflow_unknown_duplicate_and_status_fabrication() {
    let base = terminal::Session::default();
    let mut v = serde_json::to_value(&base).unwrap();
    v["command"] = serde_json::json!("x".repeat(2049));
    assert!(serde_json::from_value::<terminal::Session>(v).is_err());
    assert!(
        serde_json::from_str::<terminal::Session>(
            r#"{"version":1,"version":1,"command":"","entries":[],"selected_path":null}"#
        )
        .is_err()
    );
    let mut v = serde_json::to_value(base).unwrap();
    v["entries"] = serde_json::json!([{"command":"rpm -q x","output":{"kind":"Utility","stdout":"ok","stderr":"","status":1,"summary":null}}]);
    assert!(serde_json::from_value::<terminal::Session>(v).is_err());
}

#[test]
fn malformed_nul_command_cannot_run_or_publish_unreloadable_history() {
    let s = scenario();
    let d = context(&s);
    let mut x = state(&s);
    let before = x.clone();
    let text = "rpm -i /pkgs/provider-a.rpm\0";
    let mut t = terminal::Session::default();
    assert!(t.insert(text).is_err());
    t.command = text.into();
    let saved = serde_json::to_value(&t).unwrap();
    let error = t.run(d, &mut x).unwrap_err();
    assert!(error.contains("SimulatorUnsupported") && error.contains("NUL"));
    assert!(x == before);
    assert!(t.entries.is_empty());
    assert!(serde_json::to_value(&t).unwrap() == saved);
    let result = terminal::execute(d, &mut x, text);
    assert!(matches!(
        result,
        terminal::Outcome::SimulatorUnsupported { .. }
    ));
    assert!(result.valid());
    assert!(
        serde_json::to_value(&result)
            .unwrap()
            .get("status")
            .is_none()
    );
    assert!(x == before);
}
