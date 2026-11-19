//! Outcome/semantic grading over real production listeners and filter decisions.
use dlms_learning_lab::{
    processes::{ExpectedPhase, Requirement},
    service_exposure::PortState,
};
#[path = "support/service_exposure_fixture.rs"]
mod server_fixture;
#[path = "support/service_exposure_task_fixture.rs"]
mod task_fixture;
use task_fixture::{fixture, solve};
#[test]
fn repaired_policy_passes_and_legitimate_rule_order_and_prefix_partitions_pass() {
    let t = fixture();
    assert!(t.checks(&t.initial).iter().any(|c| !c.passed));
    let mut w = t.initial.clone();
    for net in ["192.0.2.0/25", "192.0.2.128/25"] {
        for port in [9000, 22] {
            w.admin_command(&format!(
                "iptables -I INPUT -s {net} -p tcp --dport {port} -j DROP"
            ))
            .unwrap();
        }
    }
    assert!(t.checks(&w).iter().all(|c| c.passed));
    solve(&t);
}
#[test]
fn blanket_denial_partial_repair_and_killing_a_required_owner_fail_usefully() {
    let t = fixture();
    let mut w = t.initial.clone();
    w.admin_command("iptables -P INPUT DROP").unwrap();
    let c = t.checks(&w);
    assert!(
        c.iter()
            .any(|c| !c.passed && c.detail.contains("actual filtered"))
    );
    let mut w = t.initial.clone();
    w.admin_command("iptables -A INPUT -s 192.0.2.0/24 -p tcp --dport 22 -j DROP")
        .unwrap();
    assert!(t.checks(&w).iter().any(|c| !c.passed));
    let mut w = solve(&t);
    w.admin_command("kill -TERM 201").unwrap();
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.detail.contains("actual closed"))
    );
}
#[test]
fn displayed_addresses_and_fixed_probe_source_port_cannot_stand_in_for_policy_coverage() {
    let t = fixture();
    let mut w = t.initial.clone();
    for port in [22, 9000] {
        w.admin_command(&format!(
            "iptables -A INPUT -s 192.0.2.30 -p tcp --dport {port} -j DROP"
        ))
        .unwrap();
    }
    assert!(t.cases.iter().all(|c| {
        c.accepted
            .contains(&w.scan(c.viewpoint, &[c.port]).unwrap()[0].state)
    }));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.label == "INPUT access policy")
    );
    let mut w = solve(&t);
    w.admin_command("iptables -P OUTPUT DROP").unwrap();
    w.admin_command("iptables -A OUTPUT -p tcp --dport 40000 -j ACCEPT")
        .unwrap();
    assert!(t.cases.iter().all(|c| {
        c.accepted
            .contains(&w.scan(c.viewpoint, &[c.port]).unwrap()[0].state)
    }));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.label == "OUTPUT access policy")
    );
}
#[test]
fn breaking_previously_correct_state_cannot_remain_successful() {
    let t = fixture();
    let mut w = solve(&t);
    w.admin_command("iptables -F INPUT").unwrap();
    assert!(t.checks(&w).iter().any(|c| !c.passed));
    let mut w = solve(&t);
    w.admin_command("iptables -P OUTPUT DROP").unwrap();
    assert!(t.checks(&w).iter().any(|c| !c.passed));
}
#[test]
fn malformed_impossible_or_already_solved_task_contracts_are_refused() {
    let t = fixture();
    let mut bad = t.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    assert!(bad.checks(&bad.initial).iter().all(|c| !c.passed));
    let mut bad = t.clone();
    bad.cases[0].accepted.clear();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].accepted.push(PortState::Open);
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].viewpoint = 8;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.policies.push(t.policies[0].clone());
    assert!(bad.validate().is_err());
    let mut done = t.clone();
    done.initial = solve(&t);
    assert!(done.validate().is_err());
}
#[test]
fn owner_retirement_allows_closed_or_filtered_while_preserving_required_services() {
    let mut t = fixture();
    t.requirements
        .retain(|r| !matches!(r, Requirement::Unchanged { pid: 101, .. }));
    t.requirements.push(Requirement::Phase {
        pid: 101,
        expected: ExpectedPhase::Exited,
    });
    t.policies[0]
        .domains
        .retain(|r| r.destination_port != Some(9000));
    t.policies[0]
        .allowed
        .retain(|r| r.destination_port != Some(9000));
    for c in &mut t.cases {
        if c.port == 9000 {
            c.accepted = vec![PortState::Closed, PortState::Filtered];
        }
    }
    t.validate().unwrap();
    for signal in ["TERM", "INT", "KILL"] {
        let mut w = solve(&fixture());
        w.admin_command(&format!("kill -{signal} 101")).unwrap();
        assert!(t.checks(&w).iter().all(|c| c.passed), "{signal}");
    }
    let mut w = solve(&fixture());
    w.admin_command("kill -STOP 101").unwrap();
    assert!(t.checks(&w).iter().any(|c| !c.passed));
}
