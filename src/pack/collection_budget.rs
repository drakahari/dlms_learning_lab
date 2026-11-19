//! Small admission guards shared by external and source-owned catalog loading.
//! No parsing, I/O, learner commands or allocation of an oversized collection.
pub(super) const PACK_BYTES: usize = 2_000_000;
pub(super) fn collection(packs: &[&str], maximum_files: usize) -> Result<(), String> {
    let bytes = packs.iter().try_fold(0usize, |n, p| n.checked_add(p.len()));
    if packs.is_empty() || packs.len() > maximum_files || bytes.is_none_or(|n| n > 32_000_000) {
        return Err("Invalid pack collection size".into());
    }
    Ok(())
}
pub(super) fn pack(text: &str) -> Result<(), String> {
    if text.len() > PACK_BYTES {
        return Err("Pack exceeds 2 MB".into());
    }
    Ok(())
}
pub(super) fn scenarios(current: usize, added: usize) -> Result<(), String> {
    if current.checked_add(added).is_none_or(|n| n > 1000) {
        return Err("Catalog exceeds 1,000 authored labs".into());
    }
    Ok(())
}
