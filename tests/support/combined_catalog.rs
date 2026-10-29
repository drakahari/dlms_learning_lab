//! Test-only closed composition of reviewed Bash/NFS/LACP unit appends. No runtime loader change.
use super::digest;
use serde_json::Value;
const NFS_INDEX_SHA: &str = "d1781b990368d1e5cecf871c2b11032659756cd67e39f6f5facc51f7998472ef";
const BASH_REVIEW_SHA: &str = "fa59c06a6efcfedb5a04ead90de2df84834d94cec3a5446e82ee5f58a8f9c0eb";
/// Exact private46-to-reviewed45 inverse; both raw and complete typed objects
/// must match. Historical physical45/numeric44/backup43 recipes remain immutable.
pub fn before_private_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let bytes = include_bytes!("../fixtures/catalog176-registration.json");
    if digest::hash(bytes) != "690e81765c71f591972c921fed7ce417eb21e67afdd272d8049789290757898f" {
        return Err("Immutable private-office append recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["after_units46_sha256"]
            .as_str()
            .ok_or("Private46 pin missing")?
        || raw.len() as u64
            != recipe["after_units46_bytes"]
                .as_u64()
                .ok_or("Private46 length missing")?
    {
        return Err("Exact private46 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            recipe["append_suffix_utf8"]
                .as_str()
                .ok_or("Private suffix missing")?,
        )
        .ok_or("Exact final private-office append differs")?;
    let before = format!(
        "{prefix}{}",
        recipe["old_tail_utf8"]
            .as_str()
            .ok_or("Reviewed45 tail missing")?
    )
    .into_bytes();
    if digest::hash(&before)
        != recipe["before_units45_sha256"]
            .as_str()
            .ok_or("Reviewed45 pin missing")?
        || before.len() as u64
            != recipe["before_units45_bytes"]
                .as_u64()
                .ok_or("Reviewed45 length missing")?
    {
        return Err("Full reviewed45 raw bytes differ".into());
    }
    if before.as_slice() != include_bytes!("../fixtures/private-office-units-before175.json") {
        return Err("Immutable reviewed45 raw fixture differs".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != recipe["before_units45"] {
        return Err("Full reviewed45 typed units differ".into());
    }
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Reviewed45 array missing")?;
    if units.len() != 45 {
        return Err("Reviewed45 count differs".into());
    }
    units.push(recipe["private_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact private-office unit append".into());
    }
    Ok(before)
}
fn private_prefix(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    if digest::hash(raw) == "3e616aa9199af235062796880af5735ba3537bf52c0f6a672b1d42e4a7788ae6" {
        Ok(Some(before_private_units(raw)?))
    } else {
        Ok(None)
    }
}
/// Exact private45-to-reviewed44 inverse. All old raw objects/recipes stay fixed.
pub fn before_physical_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_private = private_prefix(raw)?;
    let raw = without_private.as_deref().unwrap_or(raw);
    let bytes = include_bytes!("../fixtures/catalog175-registration.json");
    if digest::hash(bytes) != "a594ed8c8a4744101869ac2b32e5a01b14afca0eeebc1884f2490b5d5b3c2050" {
        return Err("Immutable physical append recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["after_units45_sha256"]
            .as_str()
            .ok_or("Physical45 pin missing")?
        || raw.len() as u64
            != recipe["after_units45_bytes"]
                .as_u64()
                .ok_or("Physical45 length missing")?
    {
        return Err("Exact new45 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            recipe["append_suffix_utf8"]
                .as_str()
                .ok_or("Physical suffix missing")?,
        )
        .ok_or("Exact final physical append differs")?;
    let before = format!(
        "{prefix}{}",
        recipe["old_tail_utf8"]
            .as_str()
            .ok_or("Original44 tail missing")?
    )
    .into_bytes();
    if digest::hash(&before)
        != recipe["before_units44_sha256"]
            .as_str()
            .ok_or("Original44 pin missing")?
        || before.len() as u64
            != recipe["before_units44_bytes"]
                .as_u64()
                .ok_or("Original44 length missing")?
    {
        return Err("Full original44 raw bytes differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != recipe["before_units44"] {
        return Err("Full original44 typed units differ".into());
    }
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Original44 array missing")?;
    if units.len() != 44 {
        return Err("Original44 count differs".into());
    }
    units.push(recipe["physical_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact physical unit append".into());
    }
    Ok(before)
}
fn physical_prefix(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let without_private = private_prefix(raw)?;
    let raw = without_private.as_deref().unwrap_or(raw);
    if digest::hash(raw) == "089c2ffcc7b5901a86960e36fc71bf433bc920e74ef1bf8c8042369c16c01a2a" {
        Ok(Some(before_physical_units(raw)?))
    } else {
        Ok(None)
    }
}
/// Exact new44-to-reviewed43 inverse. All old raw objects and recipes stay fixed.
pub fn before_numeric_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let bytes = include_bytes!("../fixtures/catalog171-registration.json");
    if digest::hash(bytes) != "573d8d85745ef923b515673ebce518107dbdc19e6558906aae13f6b9f3b4519c" {
        return Err("Immutable numeric append recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["after_units44_sha256"]
            .as_str()
            .ok_or("New44 pin missing")?
        || raw.len() as u64
            != recipe["after_units44_bytes"]
                .as_u64()
                .ok_or("New44 length missing")?
    {
        return Err("Exact current44 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            recipe["append_suffix_utf8"]
                .as_str()
                .ok_or("Numeric suffix missing")?,
        )
        .ok_or("Exact final numeric append differs")?;
    let before = format!(
        "{prefix}{}",
        recipe["old_tail_utf8"]
            .as_str()
            .ok_or("Reviewed43 tail missing")?
    )
    .into_bytes();
    if digest::hash(&before)
        != recipe["before_units43_sha256"]
            .as_str()
            .ok_or("Reviewed43 pin missing")?
        || before.len() as u64
            != recipe["before_units43_bytes"]
                .as_u64()
                .ok_or("Reviewed43 length missing")?
    {
        return Err("Full reviewed43 raw bytes differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != recipe["before_units43"] {
        return Err("Full reviewed43 typed units differ".into());
    }
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Reviewed43 units absent")?;
    if units.len() != 43 {
        return Err("Reviewed43 count differs".into());
    }
    units.push(recipe["numeric_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact numeric unit append".into());
    }
    reviewed_before_backup_units(&before)?;
    Ok(before)
}
/// Only normalize the exact accepted numeric suffix before the retained43 body.
pub fn before_backup_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let before = if digest::hash(raw)
        == "4003d17a5eca81584a994dd389063a9efd7abb513093ca802ffff15bf79f6d4b"
    {
        Some(before_numeric_units(raw)?)
    } else {
        None
    };
    reviewed_before_backup_units(before.as_deref().unwrap_or(raw))
}
/// Exact new43-to-reviewed42 inverse. No old unit object or recipe is rewritten.
fn reviewed_before_backup_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let bytes = include_bytes!("../fixtures/catalog168-registration.json");
    if digest::hash(bytes) != "8e8b382e4546f3e48a7441281f4db9448b68bf9333b19c7acf20e882623d4c23" {
        return Err("Immutable backup append recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["after_units43_sha256"]
            .as_str()
            .ok_or("New43 pin missing")?
        || raw.len() as u64
            != recipe["after_units43_bytes"]
                .as_u64()
                .ok_or("New43 length missing")?
    {
        return Err("Exact current43 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            recipe["append_suffix_utf8"]
                .as_str()
                .ok_or("Exact backup suffix missing")?,
        )
        .ok_or("Exact final backup append differs")?;
    let before = format!(
        "{prefix}{}",
        recipe["old_tail_utf8"]
            .as_str()
            .ok_or("Reviewed42 tail missing")?
    )
    .into_bytes();
    if digest::hash(&before)
        != recipe["before_units42_sha256"]
            .as_str()
            .ok_or("Reviewed42 pin missing")?
        || before.len() as u64
            != recipe["before_units42_bytes"]
                .as_u64()
                .ok_or("Reviewed42 length missing")?
    {
        return Err("Full reviewed42 raw bytes differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != recipe["before_units42"] {
        return Err("Full reviewed42 typed units differ".into());
    }
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Reviewed42 unit array absent")?;
    if units.len() != 42 {
        return Err("Reviewed42 count differs".into());
    }
    units.push(recipe["backup_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside ordered backup append".into());
    }
    // Prove every previous inverse still consumes exactly its reviewed source.
    reviewed_before_cron_units(&before)?;
    Ok(before)
}
/// Normalize only the exact new append, then use the retained reviewed42 body.
pub fn before_cron_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let numeric = if digest::hash(raw)
        == "4003d17a5eca81584a994dd389063a9efd7abb513093ca802ffff15bf79f6d4b"
    {
        Some(before_numeric_units(raw)?)
    } else {
        None
    };
    let raw = numeric.as_deref().unwrap_or(raw);
    let before = if digest::hash(raw)
        == "667727c71295957dfe7a0553d938dd2e7c4a4ba82d0af3b33b9e7d24f19dd7a9"
    {
        Some(before_backup_units(raw)?)
    } else {
        None
    };
    reviewed_before_cron_units(before.as_deref().unwrap_or(raw))
}
/// Exact new42→reviewed41 inverse; no historical producer or old recipe changes.
fn reviewed_before_cron_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let bytes = include_bytes!("../fixtures/catalog164-registration.json");
    if digest::hash(bytes) != "980ac05b935f9cbcc76bf3f1dff351de2e11dbad326ccf349f195f70c4f13072" {
        return Err("Immutable Cron combined append recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["after_units42_sha256"]
            .as_str()
            .ok_or("New42 pin missing")?
        || raw.len() as u64
            != recipe["after_units42_bytes"]
                .as_u64()
                .ok_or("New42 length missing")?
    {
        return Err("Exact combined42 raw units differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            recipe["append_suffix_utf8"]
                .as_str()
                .ok_or("Exact Cron suffix missing")?,
        )
        .ok_or("Exact final Cron append differs")?;
    let before = format!(
        "{prefix}{}",
        recipe["old_tail_utf8"]
            .as_str()
            .ok_or("Prior tail missing")?
    )
    .into_bytes();
    if digest::hash(&before)
        != recipe["before_units41_sha256"]
            .as_str()
            .ok_or("Reviewed41 pin missing")?
        || before.len() as u64
            != recipe["before_units41_bytes"]
                .as_u64()
                .ok_or("Reviewed41 length missing")?
    {
        return Err("Full reviewed41 raw bytes differ".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != recipe["before_units41"] {
        return Err("Full reviewed41 typed objects differ".into());
    }
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Reviewed41 unit array absent")?;
    if units.len() != 41 {
        return Err("Reviewed41 count differs".into());
    }
    units.push(recipe["cron_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside ordered Cron append".into());
    }
    before_optional_units(&before)?;
    Ok(before)
}
pub fn before_optional_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let recipe_raw = include_bytes!("../fixtures/catalog160-registration.json");
    if digest::hash(recipe_raw)
        != "e431d3f3e8102fac06272879b91baf20ae00144e410cc3278886f70a37b4e312"
    {
        return Err("Immutable combined160 unit recipe differs".into());
    }
    let recipe: Value = serde_json::from_slice(recipe_raw).map_err(|e| e.to_string())?;
    if digest::hash(raw)
        != recipe["combined41_sha256"]
            .as_str()
            .ok_or("Combined41 pin missing")?
    {
        return Err("Exact combined41 raw unit bytes differ".into());
    }
    let pos = usize::try_from(
        recipe["optional_insert_byte_offset"]
            .as_u64()
            .ok_or("Optional position missing")?,
    )
    .map_err(|e| e.to_string())?;
    let insert = recipe["optional_insert_utf8"]
        .as_str()
        .ok_or("Optional insertion missing")?
        .as_bytes();
    let end = pos
        .checked_add(insert.len())
        .ok_or("Optional position overflow")?;
    if raw.get(pos..end) != Some(insert) {
        return Err("Exact optional unit append differs".into());
    }
    let mut recovered = raw[..pos].to_vec();
    recovered.extend_from_slice(&raw[end..]);
    if digest::hash(&recovered)
        != recipe["ether40_sha256"]
            .as_str()
            .ok_or("Ether40 pin missing")?
    {
        return Err("Exact reviewed Ether40 raw unit bytes differ".into());
    }
    // Reuse the exact peer-reviewed inverse; it independently recovers and
    // validates all original39 bytes and the full authored LACP unit.
    before_lacp_units(&recovered)?;
    let mut expected: Value = serde_json::from_slice(&recovered).map_err(|e| e.to_string())?;
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Ether40 units missing")?;
    if units.len() != 40 {
        return Err("Reviewed Ether40 count differs".into());
    }
    units.push(recipe["optional_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact Ether40 plus optional unit append".into());
    }
    Ok(recovered)
}
pub fn before_lacp_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let recipe_raw = include_str!("../fixtures/catalog157-original154-boundary.json");
    if digest::hash(recipe_raw.as_bytes())
        != "0d2035c4da961607c6b40066a944040f2ee133a1270dceefe5c69598768f9969"
    {
        return Err("Immutable LACP append recipe differs".into());
    }
    let recipe: Value = serde_json::from_str(recipe_raw).map_err(|e| e.to_string())?;
    let original = recipe["original_units_utf8"]
        .as_str()
        .ok_or("Original39 units missing")?
        .as_bytes();
    if digest::hash(original)
        != recipe["original_units_sha256"]
            .as_str()
            .ok_or("Original39 pin missing")?
    {
        return Err("Original39 raw bytes differ".into());
    }
    let pos = usize::try_from(
        recipe["append"]["insertion_offset"]
            .as_u64()
            .ok_or("LACP offset missing")?,
    )
    .map_err(|e| e.to_string())?;
    let insert = recipe["append"]["insert_utf8"]
        .as_str()
        .ok_or("LACP insertion missing")?
        .as_bytes();
    if pos > original.len() {
        return Err("LACP insertion outside original39".into());
    }
    let mut expected = original[..pos].to_vec();
    expected.extend_from_slice(insert);
    expected.extend_from_slice(&original[pos..]);
    if digest::hash(&expected)
        != recipe["append"]["new_units_sha256"]
            .as_str()
            .ok_or("Current40 pin missing")?
        || raw != expected
    {
        return Err("Change outside exact original39 plus LACP append".into());
    }
    let original_value: Value = serde_json::from_slice(original).map_err(|e| e.to_string())?;
    let mut expected_value = original_value;
    let units = expected_value["units"]
        .as_array_mut()
        .ok_or("Original39 array missing")?;
    if units.len() != 39 {
        return Err("Original39 count differs".into());
    }
    units.push(recipe["append"]["new_unit"].clone());
    let actual: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if actual != expected_value {
        return Err("Exact typed original39 and new LACP unit differ".into());
    }
    Ok(original.to_vec())
}
pub fn old_unit_bytes(raw: &[u8]) -> Result<Vec<u8>, String> {
    let without_physical = physical_prefix(raw)?;
    let raw = without_physical.as_deref().unwrap_or(raw);
    let before_cron = before_cron_units(raw).ok();
    let raw = before_cron.as_deref().unwrap_or(raw);
    let before_optional = before_optional_units(raw).ok();
    let raw = before_optional.as_deref().unwrap_or(raw);
    let projected = before_lacp_units(raw).ok();
    let raw = projected.as_deref().unwrap_or(raw);
    let bash_raw = include_str!("../fixtures/shell-command-vectors-review.json");
    let nfs_raw = include_str!("../fixtures/nfs-original147-source-index.json");
    if digest::hash(bash_raw.as_bytes()) != BASH_REVIEW_SHA
        || digest::hash(nfs_raw.as_bytes()) != NFS_INDEX_SHA
    {
        return Err("Immutable append recipe differs".into());
    }
    let bash: Value = serde_json::from_str(bash_raw).map_err(|e| e.to_string())?;
    let nfs: Value = serde_json::from_str(nfs_raw).map_err(|e| e.to_string())?;
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let prefix = text
        .strip_suffix(
            nfs["unit_new_suffix"]
                .as_str()
                .ok_or("NFS suffix missing")?,
        )
        .ok_or("Exact final NFS append differs")?;
    let middle = format!(
        "{prefix}{}",
        nfs["unit_old_tail"].as_str().ok_or("Old tail missing")?
    )
    .into_bytes();
    let pin = &bash["normal_registration"];
    if digest::hash(&middle)
        != pin["new_sha256"]
            .as_str()
            .ok_or("Bash intermediate pin missing")?
    {
        return Err("Exact Bash-only intermediate source differs".into());
    }
    let pos = usize::try_from(
        pin["insert_byte_offset"]
            .as_u64()
            .ok_or("Bash position missing")?,
    )
    .map_err(|e| e.to_string())?;
    let insert = pin["insert_utf8"]
        .as_str()
        .ok_or("Bash insertion missing")?
        .as_bytes();
    let end = pos
        .checked_add(insert.len())
        .ok_or("Bash position overflow")?;
    if middle.get(pos..end) != Some(insert) {
        return Err("Exact Bash append bytes differ".into());
    }
    let mut old = middle[..pos].to_vec();
    old.extend_from_slice(&middle[end..]);
    if digest::hash(&old)
        != pin["old_sha256"]
            .as_str()
            .ok_or("Bash original pin missing")?
        || digest::hash(&old)
            != nfs["unit_original_sha256"]
                .as_str()
                .ok_or("NFS original pin missing")?
        || old.len() as u64
            != nfs["unit_original_bytes"]
                .as_u64()
                .ok_or("NFS original length missing")?
    {
        return Err("Full original37 raw bytes differ".into());
    }
    let original: Value = serde_json::from_slice(&old).map_err(|e| e.to_string())?;
    if original != nfs["units"] {
        return Err("Original typed unit data differs".into());
    }
    let mut expected = original;
    let units = expected["units"]
        .as_array_mut()
        .ok_or("Original units missing")?;
    if units.len() != 37 {
        return Err("Original unit count differs".into());
    }
    units.push(bash["unit_proposal"].clone());
    units.push(nfs["appended_unit"].clone());
    let actual: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if actual != expected {
        return Err("Change outside exact ordered Bash/NFS unit appends".into());
    }
    Ok(old)
}
