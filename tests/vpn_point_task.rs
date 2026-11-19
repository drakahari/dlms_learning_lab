//! State-based VPN grading; no host tools or learner subprocesses.
use dlms_learning_lab::vpn_point_task::*;
#[path = "support/vpn_point_fixture.rs"]
mod fixture;
use fixture::peer;
#[path = "support/vpn_point_task_fixture.rs"]
mod task_fixture;
use task_fixture::{repaired, task};
#[test]
fn substantive_peer_repair_has_real_positive_and_preservation_outcomes() {
    let t = task();
    t.validate().unwrap();
    let mut w = t.initial.clone();
    w.apply().unwrap();
    assert!(t.checks(&w).iter().any(|r| !r.passed));
    let good = repaired(&t);
    assert!(t.checks(&good).iter().all(|r| r.passed));
    let mut already = t.clone();
    already.initial.saved[1] = peer(false);
    assert!(already.validate().is_err());
    for case in &t.cases {
        assert_eq!(
            good.probe(case.from, case.source, case.destination)
                .unwrap()
                .outcome,
            case.expected
        );
    }
}
#[test]
fn alternate_equivalent_routes_and_authority_aliases_pass_without_exact_config_strings() {
    let t = task();
    let mut w = repaired(&t);
    for i in 0..2 {
        let base = peer(i == 0).replace("/etc/openvpn/ca.crt", "/etc/openvpn/alias-ca.crt");
        let base = base
            .lines()
            .filter(|l| !l.starts_with("route "))
            .collect::<Vec<_>>()
            .join("\n");
        let n = if i == 0 { "10.40.0" } else { "10.20.0" };
        w.save(i,&format!("{base}\nroute {n}.0 255.255.255.128\nroute {n}.128 255.255.255.128\nroute {n}.64 255.255.255.192\n")).unwrap();
    }
    w.apply().unwrap();
    assert!(t.checks(&w).iter().all(|c| c.passed));
}
#[test]
fn one_sided_routes_explain_missing_outward_and_return_paths() {
    let t = task();
    let mut w = repaired(&t);
    let no_routes = peer(true)
        .lines()
        .filter(|l| !l.starts_with("route "))
        .collect::<Vec<_>>()
        .join("\n");
    w.save(0, &no_routes).unwrap();
    w.apply().unwrap();
    assert_eq!(
        w.probe(
            0,
            "10.20.0.21".parse().unwrap(),
            "10.40.0.41".parse().unwrap()
        )
        .unwrap()
        .outcome,
        Outcome::MissingOutbound
    );
    assert_eq!(
        w.probe(
            1,
            "10.40.0.42".parse().unwrap(),
            "10.20.0.22".parse().unwrap()
        )
        .unwrap()
        .outcome,
        Outcome::MissingReturn
    );
    assert!(t.checks(&w).iter().any(|r| !r.passed));
}
#[test]
fn sample_hosts_broad_defaults_and_unrequested_identity_checks_cannot_pass() {
    let t = task();
    for replacement in [
        "route 10.40.0.41 255.255.255.255",
        "route 0.0.0.0 0.0.0.0",
        "route 10.40.0.0 255.255.255.128",
    ] {
        let mut w = repaired(&t);
        w.save(
            0,
            &peer(true).replace("route 10.40.0.0 255.255.255.0", replacement),
        )
        .unwrap();
        w.apply().unwrap();
        assert!(
            t.checks(&w)
                .iter()
                .any(|r| r.label.contains("full destination policy") && !r.passed),
            "{replacement}"
        );
    }
    let mut w = repaired(&t);
    let text = peer(false)
        .lines()
        .filter(|l| !l.starts_with("remote-cert-tls") && !l.starts_with("verify-x509-name"))
        .collect::<Vec<_>>()
        .join("\n");
    w.save(1, &text).unwrap();
    w.apply().unwrap();
    assert!(
        t.checks(&w)
            .iter()
            .any(|r| r.label.contains("explicit peer verification") && !r.passed)
    );
}
#[test]
fn saving_bad_work_clears_current_success_and_failed_apply_preserves_the_prior_snapshot() {
    let t = task();
    let mut w = repaired(&t);
    let applied = w.applied.clone();
    let broken = peer(false).replace("remote 203.0.113.10 1194", "remote not-an-address 1194");
    w.save(1, &broken).unwrap();
    let before = w.clone();
    assert!(w.apply().is_err());
    assert_eq!(w, before);
    assert_eq!(w.applied, applied);
    assert!(
        t.checks(&w)
            .iter()
            .any(|r| r.label == "Saved settings are applied" && !r.passed)
    );
    w.save(1, &peer(false)).unwrap();
    w.apply().unwrap();
    assert!(t.checks(&w).iter().all(|r| r.passed));
    let before = w.clone();
    assert!(w.save(2, "anything").is_err());
    assert_eq!(w, before);
    assert!(w.save(0, &"x".repeat(8193)).is_err());
    assert_eq!(w, before);
}
#[test]
fn malformed_sources_and_forged_applied_results_are_refused_without_fabricated_credit() {
    let t = task();
    t.validate().unwrap();
    for variant in 0..5 {
        let mut bad = t.clone();
        match variant {
            0 => bad.version = 2,
            1 => bad.cases[0].from = 2,
            2 => bad.cases[0].expected = Outcome::Unavailable,
            3 => bad.cases.truncate(4),
            _ => bad.cases[1].id = bad.cases[0].id.clone(),
        };
        assert!(bad.validate().is_err(), "{variant}");
    }
    let mut w = repaired(&t);
    w.applied.as_mut().unwrap()[0].port = 5555;
    assert!(w.validate().is_err());
    assert!(t.checks(&w).iter().all(|c| !c.passed));
    let mut w = repaired(&t);
    w.supplied.underlay_available = false;
    assert!(t.checks(&w).iter().all(|r| !r.passed));
    let mut value = serde_json::to_value(repaired(&t)).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<World>(value).is_err());
    assert!(
        repaired(&t)
            .probe(
                0,
                "10.99.0.1".parse().unwrap(),
                "10.40.0.41".parse().unwrap()
            )
            .is_err()
    );
}
#[test]
fn roundtrip_retains_saved_versus_applied_state_and_source_reports_are_derived() {
    let t = task();
    let mut w = repaired(&t);
    assert_eq!(w.applied, Some(fixture::configs()));
    let saved = w.saved[0].clone() + "# An unapplied note\n";
    w.save(0, &saved).unwrap();
    let bytes = serde_json::to_vec(&w).unwrap();
    let decoded: World = serde_json::from_slice(&bytes).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, w);
    assert!(t.checks(&decoded).iter().any(|r| !r.passed));
    let mut malformed = serde_json::to_value(&decoded).unwrap();
    malformed["saved"][0] = serde_json::json!("x".repeat(8193));
    assert!(serde_json::from_value::<World>(malformed).is_err());
    let original = t.clone();
    let _ = t.checks(&decoded);
    assert_eq!(t, original);
}
