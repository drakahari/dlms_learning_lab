//! Opt-in known template text leaves only. No real account or filesystem I/O.
use super::*;
pub const MAX_EDITABLE: usize = 4;
/// Kind is implicitly ordinary text. Source ownership remains fixed even when
/// an explicitly removable leaf is absent; no action accepts a replacement UID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonLeaf {
    pub uid: u32,
    pub gid: u32,
    pub allow_remove: bool,
}
pub(super) fn leaves<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, SkeletonLeaf>, D::Error> {
    map::<D, SkeletonLeaf, 4>(d)
}
fn matches(leaf: &SkeletonLeaf, entry: &Entry) -> bool {
    entry.uid == leaf.uid && entry.gid == leaf.gid && matches!(entry.data, Kind::Text { .. })
}
fn value_valid(c: &Configuration, path: &str, leaf: &SkeletonLeaf) -> bool {
    c.entries
        .get(path)
        .map_or(leaf.allow_remove, |entry| matches(leaf, entry))
}
pub(super) fn declarations(p: &Policy, c: &Configuration) -> bool {
    p.editable_skeleton_leaves.len() <= MAX_EDITABLE
        && p.editable_skeleton_leaves.iter().all(|(key, leaf)| {
            path(key)
                && leaf.uid <= 60000
                && leaf.gid <= 60000
                && !p.protected_entries.contains(key)
                && p.skeletons
                    .iter()
                    .any(|root| key != root && within(key, root))
                && !p.skeletons.iter().any(|root| within(root, key))
                && c.entries.get(parent(key)).is_some_and(Entry::directory)
                && value_valid(c, key, leaf)
        })
}
pub(super) fn initial(state: &State) -> bool {
    state
        .policy
        .editable_skeleton_leaves
        .iter()
        .all(|(key, leaf)| {
            state
                .configuration
                .entries
                .get(key)
                .is_some_and(|entry| matches(leaf, entry))
        })
}
pub(super) fn protected_entries_same(p: &Policy, c: &Configuration, b: &Configuration) -> bool {
    b.entries
        .iter()
        .filter(|(key, _)| {
            p.protected_entries.contains(*key) || p.skeletons.iter().any(|r| within(key, r))
        })
        .all(|(key, entry)| {
            if p.protected_entries.contains(key) {
                return c.entries.get(key) == Some(entry);
            }
            if let Some(leaf) = p.editable_skeleton_leaves.get(key) {
                value_valid(c, key, leaf)
            } else {
                c.entries.get(key) == Some(entry)
            }
        })
        && c.entries
            .keys()
            .filter(|key| p.skeletons.iter().any(|r| within(key, r)))
            .all(|key| b.entries.contains_key(key) || p.editable_skeleton_leaves.contains_key(key))
        && p.editable_skeleton_leaves
            .iter()
            .all(|(key, leaf)| value_valid(c, key, leaf))
}
pub(super) fn save(state: &mut State, key: &str, contents: &str, mode: u16) -> Result<()> {
    if !path(key) || mode > 0o777 || contents.len() > MAX_FILE_BYTES || contents.contains('\0') {
        return Err(err(
            ErrorKind::Invalid,
            "Template text, path or mode is outside the supported bounds",
        ));
    }
    let leaf = state
        .policy
        .editable_skeleton_leaves
        .get(key)
        .ok_or_else(|| {
            err(
                ErrorKind::Protected,
                "Only a declared template text leaf can be edited",
            )
        })?;
    if !value_valid(&state.configuration, key, leaf) {
        return Err(err(
            ErrorKind::Protected,
            "Template leaf identity/kind changed; restore the authored context",
        ));
    }
    state.configuration.entries.insert(
        key.into(),
        Entry {
            uid: leaf.uid,
            gid: leaf.gid,
            mode,
            data: Kind::Text {
                contents: contents.into(),
            },
        },
    );
    Ok(())
}
pub(super) fn remove(state: &mut State, key: &str) -> Result<()> {
    if !path(key) {
        return Err(invalid());
    }
    let leaf = state
        .policy
        .editable_skeleton_leaves
        .get(key)
        .ok_or_else(|| {
            err(
                ErrorKind::Protected,
                "Only a declared template text leaf can be removed",
            )
        })?;
    if !leaf.allow_remove {
        return Err(err(
            ErrorKind::Protected,
            "This template leaf may be updated but not removed",
        ));
    }
    let entry = state
        .configuration
        .entries
        .get(key)
        .ok_or_else(|| err(ErrorKind::Missing, "Template leaf is already absent"))?;
    if !matches(leaf, entry) {
        return Err(err(
            ErrorKind::Protected,
            "Template leaf identity/kind changed; restore the authored context",
        ));
    }
    state.configuration.entries.remove(key);
    Ok(())
}
pub(super) fn expected_entry(p: &Policy, key: &str, entry: &Entry) -> bool {
    p.editable_skeleton_leaves
        .get(key)
        .is_some_and(|leaf| matches(leaf, entry))
}
pub(super) fn expected_absence(p: &Policy, key: &str) -> bool {
    p.editable_skeleton_leaves
        .get(key)
        .is_some_and(|leaf| leaf.allow_remove)
}
