//! Exact test-only inverse of the one reviewed Cron unit append.
use super::digest;
use serde_json::Value;
const RECIPE_SHA: &str = "6999a0b2499d96bb25973898ea981c94e2ae38c32e8bcc2744715942eafaadec";
const INDEX_SHA: &str = "f19f5bf1794b2eaa79e023c0355ed926372740b17a6ae27c23dbac881681948c";
pub fn recipe() -> Value {
    let raw = include_bytes!("../fixtures/cron-registration-v1.json");
    assert_eq!(digest::hash(raw), RECIPE_SHA);
    serde_json::from_slice(raw).unwrap()
}
pub fn index() -> Value {
    let raw = include_bytes!("../fixtures/cron-original154-source-index.json");
    assert_eq!(digest::hash(raw), INDEX_SHA);
    serde_json::from_slice(raw).unwrap()
}
pub fn previous_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let r = recipe();
    let i = index();
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(r["new_suffix_utf8"].as_str().ok_or("Missing Cron suffix")?)
        .ok_or("Exact last Cron append differs")?;
    let old = format!(
        "{prefix}{}",
        r["old_tail_utf8"].as_str().ok_or("Missing prior tail")?
    )
    .into_bytes();
    if old.len() as u64
        != r["unit_before_bytes"]
            .as_u64()
            .ok_or("Prior length missing")?
        || digest::hash(&old)
            != r["unit_before_sha256"]
                .as_str()
                .ok_or("Prior pin missing")?
    {
        return Err("Full original39 raw unit bytes differ".into());
    }
    let old_typed: Value = serde_json::from_slice(&old).map_err(|e| e.to_string())?;
    if old_typed != i["units"] {
        return Err("Original39 typed objects differ".into());
    }
    let mut expected = old_typed;
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Prior units absent")?;
    if units.len() != 39 {
        return Err("Original39 count differs".into());
    }
    units.push(r["unit"].clone());
    let actual: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if actual != expected
        || digest::hash(raw)
            != r["unit_after_sha256"]
                .as_str()
                .ok_or("Current pin missing")?
    {
        return Err("Change outside exact ordered Cron append".into());
    }
    Ok(old)
}

/// Reconstruct the independently pinned standalone private Cron40 source from
/// current42 through the reviewed Root41→Ether40→original39 inverses.
pub fn private_cron_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let root41 = super::combined_catalog::before_cron_units(raw)?;
    let ether40 = super::combined_catalog::before_optional_units(&root41)?;
    let old39 = super::combined_catalog::before_lacp_units(&ether40)?;
    let r = recipe();
    let text = std::str::from_utf8(&old39).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(r["old_tail_utf8"].as_str().ok_or("Old tail absent")?)
        .ok_or("Original39 tail differs")?;
    let private = format!(
        "{prefix}{}",
        r["new_suffix_utf8"]
            .as_str()
            .ok_or("Private Cron suffix absent")?
    )
    .into_bytes();
    previous_units(&private)?;
    Ok(private)
}
