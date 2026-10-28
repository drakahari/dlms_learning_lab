//! Test-only closed composition of two reviewed unit appends. No runtime loader change.
use super::digest;
use serde_json::Value;
const NFS_INDEX_SHA: &str = "d1781b990368d1e5cecf871c2b11032659756cd67e39f6f5facc51f7998472ef";
const BASH_REVIEW_SHA: &str = "fa59c06a6efcfedb5a04ead90de2df84834d94cec3a5446e82ee5f58a8f9c0eb";
pub fn old_unit_bytes(raw: &[u8]) -> Result<Vec<u8>, String> {
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
