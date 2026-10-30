//! Closed ERE51→50/193→190 predecessor for tests/developer QA only; never grading.
#![allow(dead_code)] // Integration/native binaries import different closed helper subsets.
use dlms_learning_lab::developer_digest as digest;
use dlms_learning_lab::{
    course::Unit,
    engine::{Goal, State},
    pack::{Pack, Scenario},
    script_teaching::Lesson,
};
use serde_json::Value;
use std::sync::OnceLock;
#[path = "ere_original_catalog.rs"]
pub mod original190;
#[path = "partition_fs_registration.rs"]
pub mod partition_fs_registration;
pub const IDS: [&str; 3] = [
    "shell-ere-event-view",
    "shell-ere-inventory-review",
    "shell-ere-incident-report",
];
struct Expected {
    ids: Vec<(String, u32)>,
    old_units: Vec<Unit>,
    old_values: Vec<Value>,
    unit: Unit,
    unit_value: Value,
    new: Vec<Scenario>,
}
fn recipe() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        let raw = include_bytes!("../fixtures/catalog193-registration.json");
        assert_eq!(
            digest::hash(raw),
            "c26dec73f0727ce2ee24d76b991204e3114be028be41cd562397714b7c1ced55"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_disk = partition_fs_registration::prefix_units(raw)?;
    let raw = without_disk.as_deref().unwrap_or(raw);
    let r = recipe();
    if raw.len() as u64 != r["after_units51_bytes"].as_u64().ok_or("51length")?
        || digest::hash(raw) != r["after_units51_sha256"].as_str().ok_or("51pin")?
    {
        return Err("Exact ERE51 raw bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().ok_or("51suffix")?)
        .ok_or("Exact ERE unit append differs")?;
    let before = format!("{head}{}", r["old_tail_utf8"].as_str().ok_or("50tail")?).into_bytes();
    if before.len() as u64 != r["before_units50_bytes"].as_u64().ok_or("50length")?
        || digest::hash(&before) != r["before_units50_sha256"].as_str().ok_or("50pin")?
        || before.as_slice() != include_bytes!("../fixtures/ere-units-before190.json")
    {
        return Err("Complete reviewed190 raw50 differs".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units50"] {
        return Err("Complete50 source objects differ".into());
    }
    let units = expected["units"].as_array_mut().ok_or("50units")?;
    if units.len() != 50 {
        return Err("50count".into());
    }
    units.push(r["ere_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside complete ERE unit append".into());
    }
    Ok(before)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let without_disk = partition_fs_registration::prefix_units(raw)?;
    let raw = without_disk.as_deref().unwrap_or(raw);
    if raw.len() as u64 == recipe()["after_units51_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units51_sha256"].as_str().unwrap()
    {
        before_units(raw).map(Some)
    } else {
        Ok(None)
    }
}
fn expected() -> &'static Expected {
    static E: OnceLock<Expected> = OnceLock::new();
    E.get_or_init(|| {
        let r = recipe();
        let raw = include_bytes!("../../scenarios/shell-ere-evidence-v1.json");
        assert_eq!(
            digest::hash(raw),
            r["canonical_ere_pack_sha256"].as_str().unwrap()
        );
        let pack: Pack = serde_json::from_slice(raw).unwrap();
        assert_eq!(pack.scenarios.len(), 3);
        let ids = r["before_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                (
                    v["id"].as_str().unwrap().into(),
                    v["version"].as_u64().unwrap() as u32,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 190);
        Expected {
            ids,
            old_units: serde_json::from_value(r["before_units50"]["units"].clone()).unwrap(),
            old_values: r["before_units50"]["units"].as_array().unwrap().clone(),
            unit: serde_json::from_value(r["ere_unit"].clone()).unwrap(),
            unit_value: r["ere_unit"].clone(),
            new: pack.scenarios,
        }
    })
}
fn same_lesson(a: &Lesson, b: &Lesson) {
    let Lesson {
        prerequisite_note: _,
        concepts: _,
        example: _,
        inputs: _,
        try_change: _,
        fixture: _,
    } = a;
    assert_eq!(
        (
            &a.prerequisite_note,
            &a.example,
            &a.inputs,
            &a.try_change,
            &a.fixture
        ),
        (
            &b.prerequisite_note,
            &b.example,
            &b.inputs,
            &b.try_change,
            &b.fixture
        )
    );
    assert_eq!(a.concepts.len(), b.concepts.len());
    for (a, b) in a.concepts.iter().zip(&b.concepts) {
        assert_eq!((&a.title, &a.text), (&b.title, &b.text))
    }
}
fn same_scenario(a: &Scenario, b: &Scenario) {
    // Exhaustive current Main190 source schema; reject every foreign family.
    // Schema additions require an explicit new source review.
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
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching: _,
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
    } = a;
    assert!(
        a.physical_link_teaching.is_none()
            && b.physical_link_teaching.is_none()
            && a.one_off_teaching.is_none()
            && b.one_off_teaching.is_none()
            && a.file_backup_teaching.is_none()
            && b.file_backup_teaching.is_none()
            && a.etherchannel_teaching.is_none()
            && b.etherchannel_teaching.is_none()
            && a.cron_teaching.is_none()
            && b.cron_teaching.is_none()
            && a.nfs_teaching.is_none()
            && b.nfs_teaching.is_none()
            && a.nat_teaching.is_none()
            && b.nat_teaching.is_none()
            && a.rpm_packages_teaching.is_none()
            && b.rpm_packages_teaching.is_none()
            && a.apache_static_teaching.is_none()
            && b.apache_static_teaching.is_none()
            && a.spanning_tree_teaching.is_none()
            && b.spanning_tree_teaching.is_none()
            && a.dhcpv4_teaching.is_none()
            && b.dhcpv4_teaching.is_none()
            && a.terminal_teaching.is_none()
            && b.terminal_teaching.is_none()
            && a.process_teaching.is_none()
            && b.process_teaching.is_none()
            && a.accounts_teaching.is_none()
            && b.accounts_teaching.is_none()
            && a.unit_manager_teaching.is_none()
            && b.unit_manager_teaching.is_none()
            && a.ospf_teaching.is_none()
            && b.ospf_teaching.is_none()
            && a.discovery_teaching.is_none()
            && b.discovery_teaching.is_none()
            && a.storage_teaching.is_none()
            && b.storage_teaching.is_none()
            && a.acl_teaching.is_none()
            && b.acl_teaching.is_none()
            && a.ipv6_teaching.is_none()
            && b.ipv6_teaching.is_none()
            && a.ipv4_routing_teaching.is_none()
            && b.ipv4_routing_teaching.is_none()
            && a.address_teaching.is_none()
            && b.address_teaching.is_none()
            && a.service_teaching.is_none()
            && b.service_teaching.is_none()
            && a.client_teaching.is_none()
            && b.client_teaching.is_none()
            && a.seed_dns_teaching.is_none()
            && b.seed_dns_teaching.is_none()
            && a.seed_mount_teaching.is_none()
            && b.seed_mount_teaching.is_none()
            && a.legacy_access_teaching.is_none()
            && b.legacy_access_teaching.is_none()
            && a.legacy_logs_teaching.is_none()
            && b.legacy_logs_teaching.is_none()
            && a.dns_teaching.is_none()
            && b.dns_teaching.is_none()
            && a.switching_teaching.is_none()
            && b.switching_teaching.is_none()
            && a.mac_teaching.is_none()
            && b.mac_teaching.is_none()
            && a.debian_packages_teaching.is_none()
            && b.debian_packages_teaching.is_none()
    );
    macro_rules! same{($($f:ident),+)=>{$(assert_eq!(&a.$f,&b.$f);)+};}
    same!(
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
    assert_eq!(a.guide.len(), b.guide.len());
    for (a, b) in a.guide.iter().zip(&b.guide) {
        assert_eq!((&a.title, &a.why, &a.action), (&b.title, &b.why, &b.action));
        assert_eq!(a.commands.len(), b.commands.len());
        for (a, b) in a.commands.iter().zip(&b.commands) {
            assert_eq!(
                (&a.command, &a.explanation, a.simulator_only),
                (&b.command, &b.explanation, b.simulator_only)
            );
        }
    }
    assert!(matches!(a.initial, State::Script { .. }) && matches!(b.initial, State::Script { .. }));
    assert_eq!(a.initial, b.initial);
    match (&a.goal, &b.goal) {
        (Goal::Script { cases: a }, Goal::Script { cases: b }) => assert_eq!(a, b),
        _ => panic!("Canonical ERE Script goal kind"),
    }
    match (&a.teaching, &b.teaching) {
        (Some(a), Some(b)) => same_lesson(a, b),
        _ => panic!("Canonical ERE Introduction/example teaching required"),
    }
}

pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    let all = partition_fs_registration::prefix_scenarios(all).unwrap_or(all);
    assert_eq!(all.len(), 193);
    let e = expected();
    for (a, (id, v)) in all[..190].iter().zip(&e.ids) {
        assert_eq!((&a.id, a.version), (id, *v))
    }
    assert_eq!(
        all[190..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (a, b) in all[190..].iter().zip(&e.new) {
        same_scenario(a, b)
    }
    &all[..190]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    let all = partition_fs_registration::prefix_scenarios(all).unwrap_or(all);
    if all.len() == 193 {
        Some(before_scenarios(all))
    } else {
        None
    }
}
fn same_unit(a: &Unit, b: &Unit) {
    let Unit {
        id: _,
        track: _,
        title: _,
        summary: _,
        prerequisites: _,
        labs: _,
    } = a;
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
    )
}
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    let all = partition_fs_registration::prefix_typed_units(all).unwrap_or(all);
    assert_eq!(all.len(), 51);
    let e = expected();
    for (a, b) in all[..50].iter().zip(&e.old_units) {
        same_unit(a, b)
    }
    same_unit(&all[50], &e.unit);
    &all[..50]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    let all = partition_fs_registration::prefix_typed_units(all).unwrap_or(all);
    if all.len() == 51 {
        Some(before_typed_units(all))
    } else {
        None
    }
}
pub fn before_values(all: &[Value]) -> &[Value] {
    let all = partition_fs_registration::prefix_values(all).unwrap_or(all);
    assert_eq!(all.len(), 51);
    let e = expected();
    assert_eq!(&all[..50], &e.old_values);
    assert_eq!(&all[50], &e.unit_value);
    &all[..50]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    let all = partition_fs_registration::prefix_values(all).unwrap_or(all);
    if all.len() == 51 {
        Some(before_values(all))
    } else {
        None
    }
}
/// Count only: validate full current key set and bounded new Script attempts;
/// never turn this predecessor count into a completion certificate.
pub fn before_attempts_len(
    all: &std::collections::BTreeMap<String, dlms_learning_lab::progress::Attempt>,
) -> usize {
    let checked193 = if all.len() == 197 {
        partition_fs_registration::before_attempts_len(all)
    } else {
        all.len()
    };
    assert_eq!(checked193, 193);
    let e = expected();
    for (id, version) in &e.ids {
        assert_eq!(all[id].scenario_version, *version)
    }
    for s in &e.new {
        let a = &all[&s.id];
        assert_eq!(a.scenario_version, s.version);
        assert!(matches!(a.state, State::Script { .. }));
        assert!(a.ipv4.is_none());
        assert!(a.step < s.steps.len() && a.hints <= s.hints.len());
        dlms_learning_lab::pack::validate_state(&a.state).unwrap();
        assert!(a.workspace.as_ref().is_none_or(|w| w.valid()));
    }
    190
}
