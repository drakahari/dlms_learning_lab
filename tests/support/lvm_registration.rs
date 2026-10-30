//! Closed LVM49→48 and186→182 test/developer-QA predecessor, never grading.
//! Canonical data is parsed once; warm source/unit comparisons borrow fields.
#![allow(dead_code)] // Native QA and integration binaries import different helper subsets.
use dlms_learning_lab::developer_digest as digest;
#[path = "ntp_registration.rs"]
pub mod ntp_registration;
use dlms_learning_lab::{
    course::Unit,
    engine::{Goal, State},
    lvm_workspace::{Definition, Lesson},
    pack::{Pack, Scenario},
};
use serde_json::Value;
use std::sync::OnceLock;
pub const IDS: [&str; 4] = [
    "lpic2-lvm-free-space",
    "lpic2-lvm-add-pv",
    "lpic2-lvm-filesystem-gap",
    "lpic2-lvm-handover",
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
        let raw = include_bytes!("../fixtures/catalog186-registration.json");
        assert_eq!(
            digest::hash(raw),
            "72befcc13bf6fb0f8d78b301f699de98b105430d5b4240babcc3b7b409cab861"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_ntp = ntp_registration::prefix_units(raw)?;
    let raw = without_ntp.as_deref().unwrap_or(raw);
    let r = recipe();
    if raw.len() as u64 != r["after_units49_bytes"].as_u64().ok_or("49length")?
        || digest::hash(raw) != r["after_units49_sha256"].as_str().ok_or("49pin")?
    {
        return Err("Exact LVM49 raw bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().ok_or("LVM49suffix")?)
        .ok_or("Exact LVM unit append differs")?;
    let before = format!("{head}{}", r["old_tail_utf8"].as_str().ok_or("48tail")?).into_bytes();
    if before.len() as u64 != r["before_units48_bytes"].as_u64().ok_or("48length")?
        || digest::hash(&before) != r["before_units48_sha256"].as_str().ok_or("48pin")?
        || before.as_slice() != include_bytes!("../fixtures/lvm-units-before182.json")
    {
        return Err("Complete reviewed182 raw48 differs".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units48"] {
        return Err("Complete reviewed48 typed objects differ".into());
    }
    let units = expected["units"].as_array_mut().ok_or("48units")?;
    if units.len() != 48 {
        return Err("48count".into());
    }
    units.push(r["lvm_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside complete LVM unit append".into());
    }
    Ok(before)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let without_ntp = ntp_registration::prefix_units(raw)?;
    let raw = without_ntp.as_deref().unwrap_or(raw);
    if raw.len() as u64 == recipe()["after_units49_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units49_sha256"].as_str().unwrap()
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
        let raw = include_bytes!("../../scenarios/lvm-growth-v1.json");
        assert_eq!(
            digest::hash(raw),
            r["canonical_lvm_pack_sha256"].as_str().unwrap()
        );
        let p: Pack = serde_json::from_slice(raw).unwrap();
        assert_eq!(p.scenarios.len(), 4);
        let ids = r["before_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                (
                    v["id"].as_str().unwrap().to_owned(),
                    v["version"].as_u64().unwrap() as u32,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 182);
        Expected {
            ids,
            old_units: serde_json::from_value(r["before_units48"]["units"].clone()).unwrap(),
            old_values: r["before_units48"]["units"].as_array().unwrap().clone(),
            unit: serde_json::from_value(r["lvm_unit"].clone()).unwrap(),
            unit_value: r["lvm_unit"].clone(),
            new: p.scenarios,
        }
    })
}
fn same_lesson(a: &Lesson, b: &Lesson) {
    assert_eq!(
        (
            &a.prerequisite_note,
            &a.description,
            &a.fixture,
            &a.targets,
            &a.try_change,
            &a.experiment,
            &a.repair
        ),
        (
            &b.prerequisite_note,
            &b.description,
            &b.fixture,
            &b.targets,
            &b.try_change,
            &b.experiment,
            &b.repair
        )
    );
    assert_eq!(a.concepts.len(), b.concepts.len());
    for (a, b) in a.concepts.iter().zip(&b.concepts) {
        assert_eq!((&a.title, &a.text), (&b.title, &b.text));
    }
    assert_eq!(a.worked.len(), b.worked.len());
    for (a, b) in a.worked.iter().zip(&b.worked) {
        assert_eq!((&a.text, &a.explanation), (&b.text, &b.explanation));
    }
}
fn same_definition(a: &Definition, b: &Definition) {
    assert_eq!(&a.initial, &b.initial);
    assert_eq!(&a.targets, &b.targets);
    same_lesson(&a.lesson, &b.lesson);
}
fn same_scenario(a: &Scenario, b: &Scenario) {
    // Exhaustive current Main182 source schema; reject every foreign family.
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
            && a.teaching.is_none()
            && b.teaching.is_none()
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
    assert!(a.guide.is_empty() && b.guide.is_empty());
    match (&a.initial, &b.initial) {
        (State::Lvm { simulation: a }, State::Lvm { simulation: b }) => assert_eq!(a, b),
        _ => panic!("Canonical LVM initial kind"),
    }
    match (&a.goal, &b.goal) {
        (Goal::Lvm { definition: a }, Goal::Lvm { definition: b }) => same_definition(a, b),
        _ => panic!("Canonical LVM Goal kind"),
    }
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    let all = ntp_registration::prefix_scenarios(all).unwrap_or(all);
    assert_eq!(all.len(), 186);
    let e = expected();
    for (a, (id, v)) in all[..182].iter().zip(&e.ids) {
        assert_eq!((&a.id, a.version), (id, *v));
    }
    assert_eq!(
        all[182..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (a, b) in all[182..].iter().zip(&e.new) {
        same_scenario(a, b);
    }
    &all[..182]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    let all = ntp_registration::prefix_scenarios(all).unwrap_or(all);
    if all.len() == 186 {
        Some(before_scenarios(all))
    } else {
        None
    }
}
fn same_unit(a: &Unit, b: &Unit) {
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
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    let all = ntp_registration::prefix_typed_units(all).unwrap_or(all);
    assert_eq!(all.len(), 49);
    let e = expected();
    assert_eq!(e.old_units.len(), 48);
    for (a, b) in all[..48].iter().zip(&e.old_units) {
        same_unit(a, b);
    }
    same_unit(&all[48], &e.unit);
    &all[..48]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    let all = ntp_registration::prefix_typed_units(all).unwrap_or(all);
    if all.len() == 49 {
        Some(before_typed_units(all))
    } else {
        None
    }
}
pub fn before_values(all: &[Value]) -> &[Value] {
    let all = ntp_registration::prefix_values(all).unwrap_or(all);
    assert_eq!(all.len(), 49);
    let e = expected();
    assert_eq!(&all[..48], &e.old_values);
    assert_eq!(&all[48], &e.unit_value);
    &all[..48]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    let all = ntp_registration::prefix_values(all).unwrap_or(all);
    if all.len() == 49 {
        Some(before_values(all))
    } else {
        None
    }
}

/// Counting predecessor only, never completion. Validate every new saved context
/// against its canonical authored Goal while keeping the original map intact.
pub fn before_attempts_len(
    all: &std::collections::BTreeMap<String, dlms_learning_lab::progress::Attempt>,
) -> usize {
    let normal186 = if matches!(all.len(), 190 | 193 | 197) {
        ntp_registration::before_attempts_len(all)
    } else {
        all.len()
    };
    if normal186 == 182 {
        return 182;
    }
    assert_eq!(normal186, 186);
    let e = expected();
    for (id, version) in &e.ids {
        assert!(all.contains_key(id));
        assert_eq!(all[id].scenario_version, *version);
    }
    for source in &e.new {
        let a = &all[&source.id];
        dlms_learning_lab::lvm_workspace::context(a, source).unwrap();
    }
    182
}
