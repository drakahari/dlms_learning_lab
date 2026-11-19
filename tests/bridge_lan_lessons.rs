//! Authored bridge commissioning: exact earlier catalog and concrete progress.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    nm_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const ID: &str = "lpic2-bridge-lan-commission";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted342().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn definition() -> &'static w::Definition {
    let Goal::NmProfiles { definition } = &lab().goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::NmProfiles { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solution() -> w::Files {
    serde_json::from_str(lab().solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 9);
    workspace::prepare(&mut a);
    w::context(&a, lab()).unwrap();
    a
}
#[test]
fn all341_prior_scenarios_and119_units_remain_exact_with_one_substantive_bridge_task() {
    let old = pack::accepted341().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (341, 342));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(now.iter().filter(|s| s.id != ID).collect::<Vec<_>>()).unwrap()
    );
    let before = course::accepted341(&old).unwrap();
    let units = course::accepted342(now).unwrap();
    assert_eq!((before.len(), units.len()), (119, 120));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-bridged-lan")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let i = units
        .iter()
        .position(|u| u.id == "linux-bridged-lan")
        .unwrap();
    assert_eq!(units[i - 1].id, "linux-bonded-vlan");
    assert_eq!(units[i].labs, [ID]);
    assert_eq!(lab().prerequisites, ["lpic2-bonded-vlan-handover"]);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 77),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn real_case_feedback_reports_zero_partial_and_complete_without_bond_fallback() {
    let d = definition();
    let rows = d.initial.run(&d.initial.starter).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed()).count(), 0);
    let mut partial = d.initial.starter.clone();
    partial.insert(
        "bridge.nmconnection".into(),
        solution()["bridge.nmconnection"].clone(),
    );
    let rows = d.initial.run(&partial).unwrap();
    assert_eq!(
        rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
        [false, false, true]
    );
    assert!(
        rows[0]
            .checks
            .iter()
            .any(|(ok, s)| !*ok && s.contains("blocked"))
    );
    let rows = d.initial.run(&solution()).unwrap();
    assert!(rows.iter().all(w::CaseResult::passed));
    for row in rows {
        assert_eq!(
            row.current
                .unwrap()
                .bridge_controls()
                .unwrap()
                .current
                .members
                .len(),
            2
        );
        let future = row.future.unwrap();
        assert_eq!(future.bridge_controls().unwrap().current.members.len(), 2);
    }
}
#[test]
fn taught_solution_and_legitimate_address_stp_port_and_spelling_alternatives_pass() {
    let d = definition();
    assert!(
        d.example
            .run(&d.example_worked)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    let mut files = solution();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=false\n[ipv4]\naddress1=10.240.0.21/24\n");
    for name in ["port-enp1s0.nmconnection", "port-enp2s0.nmconnection"] {
        let text = files.get_mut(name).unwrap();
        *text = text.replace("type=ethernet", "type=802-3-ethernet");
        text.push_str("\n[connection]\nautoconnect=true\n[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\naddr-gen-mode=default\n# equivalent allowed port\n");
    }
    assert!(
        d.initial
            .run(&files)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn plausible_incorrect_ip_mask_policy_and_audit_repairs_fail_meaningfully() {
    let d = definition();
    for (name, tail) in [
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.242.0.21/24\n"),
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.240.0.21/25\n"),
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.240.0.50/24\n"),
        (
            "bridge.nmconnection",
            "\n[connection]\nautoconnect-ports=0\n",
        ),
        ("audit.nmconnection", "\n[ipv4]\naddress1=10.241.0.21/24\n"),
    ] {
        let mut files = solution();
        files.get_mut(name).unwrap().push_str(tail);
        let rows = d.initial.run(&files).unwrap();
        assert!(rows.iter().any(|r| !r.passed()));
        assert!(rows.iter().filter(|r| !r.passed()).all(
            |r| r.diagnostic.is_some() || r.checks.iter().any(|(ok, s)| !*ok && !s.is_empty())
        ));
    }
}
#[test]
fn unsupported_settings_and_errors_never_satisfy_a_required_isolated_path() {
    let d = definition();
    for (name, tail) in [
        ("bridge.nmconnection", "\n[bridge]\nvlan-filtering=true\n"),
        (
            "port-enp2s0.nmconnection",
            "\n[bridge-port]\nhairpin-mode=true\n",
        ),
        ("bridge.nmconnection", "\n[ipv6]\nmethod=auto\n"),
    ] {
        let mut files = solution();
        files.get_mut(name).unwrap().push_str(tail);
        assert!(
            d.initial
                .run(&files)
                .unwrap()
                .iter()
                .all(|r| !r.passed() && r.diagnostic.is_some())
        );
    }
}
#[test]
fn practice_is_ungraded_and_damaged_or_unchecked_restored_drafts_clear_current_success() {
    let d = definition();
    let mut a = fresh();
    let original = sim(&mut a).assignment.clone();
    sim(&mut a).practice.replace(&d.example_worked).unwrap();
    sim(&mut a).practice.run(&d.example).unwrap();
    assert_eq!(sim(&mut a).assignment, original);
    assert!(!progress::current_solved(&a, lab()));
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    assert!(progress::current_solved(&a, lab()));
    sim(&mut a).assignment.replace(&d.initial.starter).unwrap();
    assert!(sim(&mut a).assignment.tested.is_none());
    assert!(!progress::current_solved(&a, lab()));
    sim(&mut a).assignment.replace(&solution()).unwrap();
    assert!(!progress::current_solved(&a, lab()));
    sim(&mut a).assignment.run(&d.initial).unwrap();
    assert!(progress::current_solved(&a, lab()));
}
#[test]
fn progress_backup_and_selected_reset_preserve_the_actual_old_bond_draft() {
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    sim(&mut a)
        .practice
        .select("port-enp2s0.nmconnection")
        .unwrap();
    sim(&mut a)
        .practice
        .edit("port-enp2s0.nmconnection", "# unfinished separate practice")
        .unwrap();
    let old_source = catalog()
        .iter()
        .find(|s| s.id == "lpic2-bonded-vlan-handover")
        .unwrap();
    let mut old = Attempt::new(old_source, 4);
    workspace::prepare(&mut old);
    sim(&mut old)
        .assignment
        .edit("bond.nmconnection", "# preserved unfinished bond")
        .unwrap();
    assert!(!progress::current_solved(&old, old_source));
    let old_bytes = serde_json::to_value(&old).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(ID.into(), a);
    p.attempts.insert(old_source.id.clone(), old);
    p.favorites.extend([ID.into(), old_source.id.clone()]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[ID], lab()));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old_source.id]).unwrap(),
        old_bytes
    );
    let exported = dir.path().join("backup.json");
    backup::export(&exported, &p, catalog()).unwrap();
    let checked = backup::inspect(
        &exported,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &checked, catalog()).unwrap();
    assert_eq!(std::fs::read(receipt.safety_copy.unwrap()).unwrap(), bytes);
    p.attempts.insert(ID.into(), fresh());
    assert!(!progress::current_solved(&p.attempts[ID], lab()));
    assert_eq!(
        serde_json::to_value(&p.attempts[&old_source.id]).unwrap(),
        old_bytes
    );
    assert!(p.favorites.contains(ID) && p.favorites.contains(&old_source.id));
}
