//! Closed loop48→47 and182→180 predecessor checks for tests/developer QA only.
#![allow(dead_code)] // Each integration binary imports a different helper subset.
#[path = "lvm_registration.rs"]
pub mod lvm_registration;
use dlms_learning_lab::{
    course::Unit,
    engine::{Goal, State},
    pack::{Pack, Scenario},
};
use serde_json::Value;
use std::sync::OnceLock;
pub const IDS: [&str; 2] = ["shell-loop-intake", "shell-loop-fallback"];
fn recipe() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!("../fixtures/catalog182-registration.json")).unwrap()
    })
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_lvm = lvm_registration::prefix_units(raw)?;
    let raw = without_lvm.as_deref().unwrap_or(raw);
    let r = recipe();
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    if raw.len() as u64 != r["after_units48_bytes"].as_u64().ok_or("48bytes")? {
        return Err("Exact loop48 length differs".into());
    }
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().ok_or("48suffix")?)
        .ok_or("Exact loop48 suffix differs")?;
    let before = format!("{head}{}", r["old_tail_utf8"].as_str().ok_or("47tail")?).into_bytes();
    if before.as_slice() != include_bytes!("../fixtures/loop-units-before180.json") {
        return Err("Original47 raw bytes differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units47"] {
        return Err("Original47 typed units differ".into());
    }
    let old = expected["units"].as_array_mut().ok_or("47units")?;
    if old.len() != 47 {
        return Err("47count".into());
    }
    old.push(r["loop_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact loop append".into());
    }
    Ok(before)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let without_lvm = lvm_registration::prefix_units(raw)?;
    let raw = without_lvm.as_deref().unwrap_or(raw);
    if raw == include_bytes!("../fixtures/lvm-units-before182.json") {
        before_units(raw).map(Some)
    } else {
        Ok(None)
    }
}
fn canonical() -> &'static Vec<Scenario> {
    static S: OnceLock<Vec<Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str::<Pack>(include_str!("../../scenarios/shell-loop-recovery-v1.json"))
            .unwrap()
            .scenarios
    })
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    let all = lvm_registration::prefix_scenarios(all).unwrap_or(all);
    assert_eq!(all.len(), 182);
    let old = recipe()["before_ids"].as_array().unwrap();
    assert_eq!(old.len(), 180);
    for (s, v) in all[..180].iter().zip(old) {
        assert_eq!(s.id, v["id"].as_str().unwrap());
        assert_eq!(s.version as u64, v["version"].as_u64().unwrap());
    }
    assert_eq!(
        all[180..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (a, b) in all[180..].iter().zip(canonical()) {
        same_scenario(a, b);
        assert!(matches!(a.initial, State::Script { .. }) && matches!(a.goal, Goal::Script { .. }));
    }
    &all[..180]
}
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    let all = lvm_registration::prefix_typed_units(all).unwrap_or(all);
    static UNITS: OnceLock<Vec<Unit>> = OnceLock::new();
    let expected = UNITS.get_or_init(|| {
        let r = recipe();
        let mut units: Vec<Unit> =
            serde_json::from_value(r["before_units47"]["units"].clone()).unwrap();
        assert_eq!(units.len(), 47);
        units.push(serde_json::from_value(r["loop_unit"].clone()).unwrap());
        units
    });
    assert_eq!(all.len(), 48);
    for (a, b) in all.iter().zip(expected) {
        assert_eq!(
            (
                &a.id,
                &a.track,
                &a.title,
                &a.summary,
                &a.prerequisites,
                &a.labs
            ),
            (
                &b.id,
                &b.track,
                &b.title,
                &b.summary,
                &b.prerequisites,
                &b.labs
            )
        );
    }
    &all[..47]
}
pub fn before_values(all: &[Value]) -> &[Value] {
    let all = lvm_registration::prefix_values(all).unwrap_or(all);
    assert_eq!(all.len(), 48);
    let r = recipe();
    assert_eq!(&all[..47], r["before_units47"]["units"].as_array().unwrap());
    assert_eq!(all[47], r["loop_unit"]);
    &all[..47]
}

fn same_scenario(a: &Scenario, b: &Scenario) {
    // Exhaustive source shape keeps future fields visible; no serialization/clones on warm QA calls.
    let Scenario {
        physical_link_teaching: _,
        one_off_teaching: _,
        file_backup_teaching: _,
        etherchannel_teaching: _,
        cron_teaching: _,
        nfs_teaching: _,
        nat_teaching: _,
        rpm_packages_teaching: _,
        apache_static_teaching: _,
        spanning_tree_teaching: _,
        dhcpv4_teaching: _,
        terminal_teaching: _,
        process_teaching: _,
        accounts_teaching: _,
        unit_manager_teaching: _,
        ospf_teaching: _,
        discovery_teaching: _,
        storage_teaching: _,
        acl_teaching: _,
        ipv6_teaching: _,
        ipv4_routing_teaching: _,
        address_teaching: _,
        service_teaching: _,
        client_teaching: _,
        seed_dns_teaching: _,
        seed_mount_teaching: _,
        legacy_access_teaching: _,
        legacy_logs_teaching: _,
        dns_teaching: _,
        switching_teaching: _,
        mac_teaching: _,
        debian_packages_teaching: _,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        solution: _,
        prerequisites: _,
        initial: _,
        goal: _,
        guide: _,
        teaching: _,
    } = a;
    macro_rules! eq_fields{($($f:ident),+)=>{$(assert_eq!(&a.$f,&b.$f);)+};}
    eq_fields!(
        id,
        version,
        track,
        topic,
        unit,
        title,
        summary,
        objective,
        retry_guidance,
        difficulty,
        minutes,
        steps,
        explanation,
        hints,
        sources,
        simplifications,
        solution,
        prerequisites
    );
    macro_rules! no_foreign{($($f:ident),+)=>{$(assert!(a.$f.is_none()&&b.$f.is_none());)+};}
    no_foreign!(
        physical_link_teaching,
        one_off_teaching,
        file_backup_teaching,
        etherchannel_teaching,
        cron_teaching,
        nfs_teaching,
        nat_teaching,
        rpm_packages_teaching,
        apache_static_teaching,
        spanning_tree_teaching,
        dhcpv4_teaching,
        terminal_teaching,
        process_teaching,
        accounts_teaching,
        unit_manager_teaching,
        ospf_teaching,
        discovery_teaching,
        storage_teaching,
        acl_teaching,
        ipv6_teaching,
        ipv4_routing_teaching,
        address_teaching,
        service_teaching,
        client_teaching,
        seed_dns_teaching,
        seed_mount_teaching,
        legacy_access_teaching,
        legacy_logs_teaching,
        dns_teaching,
        switching_teaching,
        mac_teaching,
        debian_packages_teaching
    );
    assert_eq!(
        serde_json::to_value(&a.guide).unwrap(),
        serde_json::to_value(&b.guide).unwrap(),
        "Preserve every authored guide step, including nonempty teaching"
    );
    assert_eq!(&a.initial, &b.initial);
    match (&a.goal, &b.goal) {
        (Goal::Script { cases: x }, Goal::Script { cases: y }) => assert_eq!(x, y),
        _ => panic!("Canonical loop script goal"),
    }
    let (x, y) = (a.teaching.as_ref().unwrap(), b.teaching.as_ref().unwrap());
    assert_eq!(
        (
            &x.prerequisite_note,
            &x.example,
            &x.inputs,
            &x.try_change,
            &x.fixture
        ),
        (
            &y.prerequisite_note,
            &y.example,
            &y.inputs,
            &y.try_change,
            &y.fixture
        )
    );
    assert_eq!(x.concepts.len(), y.concepts.len());
    for (x, y) in x.concepts.iter().zip(&y.concepts) {
        assert_eq!((&x.title, &x.text), (&y.title, &y.text));
    }
}
