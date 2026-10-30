//! Closed NTP50→49 and190→186 test/developer-QA predecessor, never grading.
//! Canonical data is parsed once; warm source/unit comparisons borrow fields.
#![allow(dead_code)] // Native QA and integration binaries import different helper subsets.
use dlms_learning_lab::developer_digest as digest;
use dlms_learning_lab::{
    course::Unit,
    engine::{Goal, State},
    ntp_workspace::{Definition, Lesson},
    pack::{Pack, Scenario},
};
use serde_json::Value;
use std::sync::OnceLock;
#[path = "ere_registration.rs"]
pub mod ere_registration;
pub const IDS: [&str; 4] = [
    "ccna-ntp-client-chain",
    "ccna-ntp-cycle-repair",
    "ccna-ntp-asymmetric-delivery",
    "ccna-ntp-policy-handover",
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
        let raw = include_bytes!("../fixtures/catalog190-registration.json");
        assert_eq!(
            digest::hash(raw),
            "29d8d6d38b61d6d8a81ab89cf7e4cc7e0454ec605fbc838f73b43a1747331966"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_ere = ere_registration::prefix_units(raw)?;
    let raw = without_ere.as_deref().unwrap_or(raw);
    let r = recipe();
    if raw.len() as u64 != r["after_units50_bytes"].as_u64().ok_or("50length")?
        || digest::hash(raw) != r["after_units50_sha256"].as_str().ok_or("50pin")?
    {
        return Err("Exact NTP50 raw bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().ok_or("NTP50suffix")?)
        .ok_or("Exact NTP unit append differs")?;
    let before = format!("{head}{}", r["old_tail_utf8"].as_str().ok_or("49tail")?).into_bytes();
    if before.len() as u64 != r["before_units49_bytes"].as_u64().ok_or("49length")?
        || digest::hash(&before) != r["before_units49_sha256"].as_str().ok_or("49pin")?
        || before.as_slice() != include_bytes!("../fixtures/ntp-units-before186.json")
    {
        return Err("Complete reviewed186 raw49 differs".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units49"] {
        return Err("Complete reviewed49 typed objects differ".into());
    }
    let units = expected["units"].as_array_mut().ok_or("49units")?;
    if units.len() != 49 {
        return Err("49count".into());
    }
    units.push(r["ntp_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside complete NTP unit append".into());
    }
    Ok(before)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let without_ere = ere_registration::prefix_units(raw)?;
    let raw = without_ere.as_deref().unwrap_or(raw);
    if raw.len() as u64 == recipe()["after_units50_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units50_sha256"].as_str().unwrap()
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
        let raw = include_bytes!("../../scenarios/ntp-client-server-v1.json");
        assert_eq!(
            digest::hash(raw),
            r["canonical_ntp_pack_sha256"].as_str().unwrap()
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
        assert_eq!(ids.len(), 186);
        Expected {
            ids,
            old_units: serde_json::from_value(r["before_units49"]["units"].clone()).unwrap(),
            old_values: r["before_units49"]["units"].as_array().unwrap().clone(),
            unit: serde_json::from_value(r["ntp_unit"].clone()).unwrap(),
            unit_value: r["ntp_unit"].clone(),
            new: p.scenarios,
        }
    })
}
fn same_targets(
    a: &[dlms_learning_lab::ntp_workspace::Target],
    b: &[dlms_learning_lab::ntp_workspace::Target],
) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert_eq!(
            (&a.client, &a.allowed_references, a.must_serve),
            (&b.client, &b.allowed_references, b.must_serve)
        );
    }
}
fn same_command(
    a: &dlms_learning_lab::ntp_workspace::Command,
    b: &dlms_learning_lab::ntp_workspace::Command,
) {
    assert_eq!(
        (a.device, &a.text, &a.explanation),
        (b.device, &b.text, &b.explanation)
    );
}
fn same_lesson(a: &Lesson, b: &Lesson) {
    assert_eq!(
        (
            &a.prerequisite_note,
            &a.description,
            &a.fixture,
            &a.protected,
            &a.try_change
        ),
        (
            &b.prerequisite_note,
            &b.description,
            &b.fixture,
            &b.protected,
            &b.try_change
        )
    );
    same_targets(&a.targets, &b.targets);
    assert_eq!(a.concepts.len(), b.concepts.len());
    for (a, b) in a.concepts.iter().zip(&b.concepts) {
        assert_eq!((&a.title, &a.text), (&b.title, &b.text));
    }
    assert_eq!(a.worked.len(), b.worked.len());
    for (a, b) in a.worked.iter().zip(&b.worked) {
        same_command(a, b);
    }
    same_command(&a.experiment, &b.experiment);
    same_command(&a.repair, &b.repair);
}
fn same_definition(a: &Definition, b: &Definition) {
    assert_eq!((&a.initial, &a.protected), (&b.initial, &b.protected));
    same_targets(&a.targets, &b.targets);
    same_lesson(&a.lesson, &b.lesson);
}
fn same_scenario(a: &Scenario, b: &Scenario) {
    // Exhaustive current Main186 source schema; reject every foreign family.
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
    match (&a.initial, &b.initial) {
        (State::Ntp { simulation: a }, State::Ntp { simulation: b }) => assert_eq!(a, b),
        _ => panic!("Canonical NTP initial kind"),
    }
    match (&a.goal, &b.goal) {
        (Goal::Ntp { definition: a }, Goal::Ntp { definition: b }) => same_definition(a, b),
        _ => panic!("Canonical NTP Goal kind"),
    }
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    let all = ere_registration::prefix_scenarios(all).unwrap_or(all);
    assert_eq!(all.len(), 190);
    let e = expected();
    for (a, (id, v)) in all[..186].iter().zip(&e.ids) {
        assert_eq!((&a.id, a.version), (id, *v));
    }
    assert_eq!(
        all[186..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (a, b) in all[186..].iter().zip(&e.new) {
        same_scenario(a, b);
    }
    &all[..186]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    let all = ere_registration::prefix_scenarios(all).unwrap_or(all);
    if all.len() == 190 {
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
    let all = ere_registration::prefix_typed_units(all).unwrap_or(all);
    assert_eq!(all.len(), 50);
    let e = expected();
    assert_eq!(e.old_units.len(), 49);
    for (a, b) in all[..49].iter().zip(&e.old_units) {
        same_unit(a, b);
    }
    same_unit(&all[49], &e.unit);
    &all[..49]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    let all = ere_registration::prefix_typed_units(all).unwrap_or(all);
    if all.len() == 50 {
        Some(before_typed_units(all))
    } else {
        None
    }
}
pub fn before_values(all: &[Value]) -> &[Value] {
    let all = ere_registration::prefix_values(all).unwrap_or(all);
    assert_eq!(all.len(), 50);
    let e = expected();
    assert_eq!(&all[..49], &e.old_values);
    assert_eq!(&all[49], &e.unit_value);
    &all[..49]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    let all = ere_registration::prefix_values(all).unwrap_or(all);
    if all.len() == 50 {
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
    if all.len() == 186 {
        return 186;
    }
    let normal190 = if matches!(all.len(), 193 | 197) {
        ere_registration::before_attempts_len(all)
    } else {
        all.len()
    };
    assert_eq!(normal190, 190);
    let e = expected();
    for (id, version) in &e.ids {
        assert!(all.contains_key(id));
        assert_eq!(all[id].scenario_version, *version);
    }
    for source in &e.new {
        let a = &all[&source.id];
        dlms_learning_lab::ntp_workspace::context(a, source).unwrap();
    }
    186
}
