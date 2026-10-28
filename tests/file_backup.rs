use dlms_learning_lab::{
    engine::Node,
    file_backup::{self, Archive, Entry, Tree},
};
use std::collections::BTreeMap;

fn node(directory: bool) -> Node {
    Node {
        directory,
        mode: if directory { 0o750 } else { 0o640 },
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn empty() -> Tree {
    Tree {
        clock: 0,
        entries: BTreeMap::from([(
            "".into(),
            Entry {
                node: node(true),
                text: None,
                modified: 0,
                changed: 0,
            },
        )]),
    }
}
fn sample() -> Tree {
    let mut tree = empty();
    tree.add("notes", node(true), None).unwrap();
    tree.add("notes/keep.txt", node(false), Some("keep\n"))
        .unwrap();
    tree.add("notes/change.txt", node(false), Some("before\n"))
        .unwrap();
    tree.add("notes/remove.txt", node(false), Some("obsolete\n"))
        .unwrap();
    tree.add(".settings", node(false), Some("private=true\n"))
        .unwrap();
    tree
}

// Restoring preserves file mtime and requested metadata, while ctime must change.
fn payload(tree: &Tree) -> BTreeMap<&str, (&Node, Option<&str>, u64)> {
    tree.entries
        .iter()
        .map(|(p, e)| (p.as_str(), (&e.node, e.text.as_deref(), e.modified)))
        .collect()
}

#[test]
fn full_backup_is_an_owned_snapshot_and_restores_metadata_and_hidden_files() {
    let mut source = sample();
    let (full, _) = file_backup::create(&source, "project", None, false).unwrap();
    source.write("notes/change.txt", "after\n").unwrap();
    let mut restored = empty();
    full.extract(&mut restored, false).unwrap();
    assert_eq!(
        restored.entries["notes/change.txt"].text.as_deref(),
        Some("before\n")
    );
    assert_eq!(
        restored.entries[".settings"].text.as_deref(),
        Some("private=true\n")
    );
    assert_eq!(restored.entries["notes"].node.mode, 0o750);
    assert_eq!(restored.entries["notes/change.txt"].node.owner, "alex");
    assert_eq!(full.entries.len(), 6);
    let wire = serde_json::to_string(&full).unwrap();
    assert_eq!(Archive::from_json(&wire).unwrap(), full);
}

#[test]
fn incremental_selection_tracks_changes_and_deletions_without_unchanged_payloads() {
    let mut source = sample();
    let (base, snapshot) = file_backup::create(&source, "project", None, true).unwrap();
    source.write("notes/change.txt", "updated\n").unwrap();
    source.remove("notes/remove.txt").unwrap();
    source
        .add("notes/new.txt", node(false), Some("new\n"))
        .unwrap();
    source.chmod(".settings", 0o600).unwrap();
    let (delta, _) = file_backup::create(&source, "project", Some(&snapshot), true).unwrap();
    assert!(!delta.entries.contains_key("notes/keep.txt"));
    assert!(!delta.entries.contains_key("notes/remove.txt"));
    assert_eq!(
        delta.entries["notes/change.txt"].text.as_deref(),
        Some("updated\n")
    );
    assert!(delta.entries.contains_key(".settings"));
    let mut restored = empty();
    base.extract(&mut restored, true).unwrap();
    delta.extract(&mut restored, true).unwrap();
    assert_eq!(payload(&restored), payload(&source));
    assert!(!restored.entries.contains_key("notes/remove.txt"));
}

#[test]
fn overlay_extraction_keeps_deleted_names_and_latest_delta_alone_is_incomplete() {
    let mut source = sample();
    let (base, snapshot) = file_backup::create(&source, "project", None, true).unwrap();
    source.remove("notes/remove.txt").unwrap();
    source.write("notes/change.txt", "newest\n").unwrap();
    let (delta, _) = file_backup::create(&source, "project", Some(&snapshot), true).unwrap();
    let mut overlay = empty();
    base.extract(&mut overlay, true).unwrap();
    delta.extract(&mut overlay, false).unwrap();
    assert!(overlay.entries.contains_key("notes/remove.txt"));
    let mut incomplete = empty();
    delta.extract(&mut incomplete, true).unwrap();
    assert!(!incomplete.entries.contains_key("notes/keep.txt"));
    assert_ne!(payload(&incomplete), payload(&source));
    // Out-of-order levels honestly apply: no fictitious chain gate masks stale data.
    let mut reversed = empty();
    delta.extract(&mut reversed, true).unwrap();
    base.extract(&mut reversed, true).unwrap();
    assert_eq!(
        reversed.entries["notes/change.txt"].text.as_deref(),
        Some("before\n")
    );
    assert_ne!(payload(&reversed), payload(&source));
}

#[test]
fn independent_snapshot_branches_are_alternate_valid_full_restore_chains() {
    let mut source = sample();
    let (base, snap0) = file_backup::create(&source, "project", None, true).unwrap();
    source.write("notes/change.txt", "day1\n").unwrap();
    let (day1, snap1) = file_backup::create(&source, "project", Some(&snap0), true).unwrap();
    source.write(".settings", "private=still\n").unwrap();
    let (day2, _) = file_backup::create(&source, "project", Some(&snap1), true).unwrap();
    let (since_base, _) = file_backup::create(&source, "project", Some(&snap0), true).unwrap();
    let mut sequential = empty();
    for archive in [&base, &day1, &day2] {
        archive.extract(&mut sequential, true).unwrap();
    }
    let mut differential = empty();
    for archive in [&base, &since_base] {
        archive.extract(&mut differential, true).unwrap();
    }
    assert_eq!(payload(&sequential), payload(&source));
    assert_eq!(payload(&differential), payload(&source));
    let mut missing_middle = empty();
    base.extract(&mut missing_middle, true).unwrap();
    day2.extract(&mut missing_middle, true).unwrap();
    assert_ne!(payload(&missing_middle), payload(&source));
}

#[test]
fn directory_deletion_removes_descendants_but_never_similarly_prefixed_siblings() {
    let mut source = sample();
    source.add("notes-other", node(true), None).unwrap();
    source
        .add("notes-other/keep", node(false), Some("sibling\n"))
        .unwrap();
    let (base, snap) = file_backup::create(&source, "project", None, true).unwrap();
    source.remove("notes").unwrap();
    let (delta, _) = file_backup::create(&source, "project", Some(&snap), true).unwrap();
    let mut restored = empty();
    base.extract(&mut restored, true).unwrap();
    delta.extract(&mut restored, true).unwrap();
    assert_eq!(payload(&restored), payload(&source));
    assert!(restored.entries.contains_key("notes-other/keep"));
}

#[test]
fn stale_wrong_root_and_future_snapshots_are_rejected_without_mutation() {
    let source = sample();
    let (_, mut snapshot) = file_backup::create(&source, "project", None, true).unwrap();
    assert!(file_backup::create(&source, "other", Some(&snapshot), true).is_err());
    assert!(file_backup::create(&source, "project", Some(&snapshot), false).is_err());
    snapshot.cutoff = source.clock + 1;
    assert!(file_backup::create(&source, "project", Some(&snapshot), true).is_err());
    assert_eq!(source, sample());
}

#[test]
fn malicious_archive_paths_and_inventories_cannot_escape_or_purge_other_roots() {
    let source = sample();
    let (archive, _) = file_backup::create(&source, "project", None, true).unwrap();
    for bad in [
        "../outside",
        "/absolute",
        "a/../../private",
        "a//file",
        "a/./file",
        "bad\nname",
    ] {
        let mut value = archive.clone();
        let e = value.entries["notes/keep.txt"].clone();
        value.entries.insert(bad.into(), e);
        assert!(!value.valid(), "{bad}");
        assert!(Archive::from_json(&serde_json::to_string(&value).unwrap()).is_err());
        let mut target = source.clone();
        assert!(value.extract(&mut target, true).is_err());
        assert_eq!(target, source);
    }
    let mut value = archive.clone();
    value
        .directories
        .get_mut("notes")
        .unwrap()
        .push("..".into());
    assert!(!value.valid());
    let mut value = archive.clone();
    value
        .directories
        .get_mut("notes")
        .unwrap()
        .push("keep.txt".into());
    assert!(!value.valid());
    let mut value = archive.clone();
    value.directories.remove("notes");
    assert!(!value.valid());
    let mut value = archive.clone();
    value.incremental = false;
    value.entries.remove("notes/keep.txt");
    assert!(!value.valid());
}

#[test]
fn capacity_errors_type_conflicts_and_invalid_edits_are_atomic() {
    let mut source = sample();
    let before = source.clone();
    assert!(source.write("notes", "wrong").is_err());
    assert_eq!(source, before);
    assert!(
        source
            .write("notes/keep.txt", &"x".repeat(file_backup::MAX_TEXT))
            .is_err()
    );
    assert_eq!(source, before);
    assert!(source.add("missing/file", node(false), Some("x")).is_err());
    assert_eq!(source, before);
    assert!(source.remove("").is_err());
    assert_eq!(source, before);
    let (full, _) = file_backup::create(&source, "project", None, false).unwrap();
    let mut target = empty();
    target.add("notes", node(false), Some("collision")).unwrap();
    let prior = target.clone();
    assert!(full.extract(&mut target, false).is_err());
    assert_eq!(target, prior);
    let mut fulltarget = empty();
    fulltarget
        .add(
            "large",
            node(false),
            Some(&"x".repeat(file_backup::MAX_TEXT)),
        )
        .unwrap();
    let prior = fulltarget.clone();
    assert!(full.extract(&mut fulltarget, false).is_err());
    assert_eq!(fulltarget, prior);
    assert!(Tree::from_json(&" ".repeat(file_backup::MAX_WIRE + 1)).is_err());
    assert!(Archive::from_json(&" ".repeat(file_backup::MAX_WIRE + 1)).is_err());
}

#[test]
fn unchanged_incremental_contains_directories_and_can_remove_unrequested_restore_files() {
    let source = sample();
    let (base, snap) = file_backup::create(&source, "project", None, true).unwrap();
    let (delta, _) = file_backup::create(&source, "project", Some(&snap), true).unwrap();
    assert_eq!(delta.entries.len(), 2);
    assert!(delta.entries.values().all(|e| e.node.directory));
    let mut restored = empty();
    base.extract(&mut restored, true).unwrap();
    restored
        .add("notes/stray", node(false), Some("not in source"))
        .unwrap();
    delta.extract(&mut restored, true).unwrap();
    assert!(!restored.entries.contains_key("notes/stray"));
    assert_eq!(payload(&restored), payload(&source));
}

#[test]
fn decoders_reject_duplicate_excessive_keys_and_names_before_their_values() {
    let source = empty();
    let entry = serde_json::to_string(&source.entries[""]).unwrap();
    // The second value is deliberately malformed for Entry: the duplicate-key
    // guard must fail before attempting to decode it, rather than overwriting.
    let raw = format!("{{\"clock\":0,\"entries\":{{\"\":{entry},\"\":false}}}}");
    assert!(
        Tree::from_json(&raw)
            .unwrap_err()
            .contains("duplicate virtual path")
    );
    let mut entries = vec![format!("\"\":{entry}")];
    let file = serde_json::to_string(&Entry {
        node: node(false),
        text: Some(String::new()),
        modified: 0,
        changed: 0,
    })
    .unwrap();
    for n in 0..255 {
        entries.push(format!("\"file-{n}\":{file}"));
    }
    entries.push("\"excess\":false".into());
    let raw = format!("{{\"clock\":0,\"entries\":{{{}}}}}", entries.join(","));
    assert!(
        Tree::from_json(&raw)
            .unwrap_err()
            .contains("count exceeds256")
    );
    let (archive, _) = file_backup::create(&source, "project", None, true).unwrap();
    let wire = serde_json::to_string(&archive).unwrap();
    let duplicate = wire.replace(
        "\"directories\":{\"\":[]}",
        "\"directories\":{\"\":[\"same\",\"same\"]}",
    );
    assert_ne!(duplicate, wire);
    assert!(
        Archive::from_json(&duplicate)
            .unwrap_err()
            .contains("duplicate directory entry")
    );
    let mut value = serde_json::to_value(&archive).unwrap();
    value["directories"][""] = serde_json::json!(["x".repeat(256)]);
    assert!(
        Archive::from_json(&value.to_string())
            .unwrap_err()
            .contains("directory entry")
    );
    let mut value = serde_json::to_value(&archive).unwrap();
    value["entries"][""]["text"] = serde_json::json!("x".repeat(file_backup::MAX_TEXT + 1));
    assert!(
        Archive::from_json(&value.to_string())
            .unwrap_err()
            .contains("file text bound")
    );
}

#[test]
fn restore_preserves_mtime_but_changes_ctime_and_enters_the_next_incremental() {
    let original = sample();
    let (old_archive, _) = file_backup::create(&original, "project", None, false).unwrap();
    let mut current = original.clone();
    current
        .write("notes/change.txt", "later contents\n")
        .unwrap();
    let (_, cutoff) = file_backup::create(&current, "project", None, true).unwrap();
    old_archive.extract(&mut current, false).unwrap();
    assert_eq!(
        current.entries["notes/change.txt"].modified,
        original.entries["notes/change.txt"].modified
    );
    assert!(current.entries["notes/change.txt"].changed > cutoff.cutoff);
    let (delta, _) = file_backup::create(&current, "project", Some(&cutoff), true).unwrap();
    assert_eq!(
        delta.entries["notes/change.txt"].text.as_deref(),
        Some("before\n")
    );
    assert_eq!(delta.entries["notes/change.txt"].changed, current.clock);
    current.clock = 1_000_000;
    let unchanged = current.clone();
    assert!(old_archive.extract(&mut current, false).is_err());
    assert_eq!(current, unchanged);
}

#[test]
fn unchanged_omitted_files_cannot_be_replaced_by_restore_target_directories() {
    let source = sample();
    let (_, snapshot) = file_backup::create(&source, "project", None, true).unwrap();
    let (delta, _) = file_backup::create(&source, "project", Some(&snapshot), true).unwrap();
    assert!(!delta.entries.contains_key("notes/keep.txt"));
    let mut target = empty();
    target.add("notes", node(true), None).unwrap();
    target.add("notes/keep.txt", node(true), None).unwrap();
    target
        .add(
            "notes/keep.txt/nested",
            node(false),
            Some("must not masquerade as keep.txt"),
        )
        .unwrap();
    let before = target.clone();
    assert!(
        delta
            .extract(&mut target, true)
            .unwrap_err()
            .contains("replacement")
    );
    assert_eq!(target, before);
    assert!(delta.extract(&mut target, false).is_err());
    assert_eq!(target, before);
}

#[test]
fn metadata_scalar_bounds_reject_before_unrelated_later_fields_are_decoded() {
    let source = sample();
    let (archive, _) = file_backup::create(&source, "project", None, true).unwrap();
    let wire = serde_json::to_string(&archive).unwrap();
    let root = wire.replace(
        "\"root\":\"project\"",
        &format!("\"root\":\"{}\"", "r".repeat(256)),
    );
    assert!(Archive::from_json(&root).unwrap_err().contains("root name"));
    for key in ["owner", "group"] {
        let bad = wire.replace(
            &format!(
                "\"{key}\":\"{}\"",
                if key == "owner" { "alex" } else { "staff" }
            ),
            &format!("\"{key}\":\"{}\"", "n".repeat(33)),
        );
        assert_ne!(bad, wire);
        assert!(
            Archive::from_json(&bad)
                .unwrap_err()
                .contains("account/group")
        );
    }
}

#[test]
fn selective_file_restore_preserves_unselected_changes_and_directory_metadata() {
    let original = sample();
    let (archive, _) = file_backup::create(&original, "project", None, true).unwrap();
    let mut target = original.clone();
    target.write(".settings", "damaged\n").unwrap();
    target
        .write("notes/change.txt", "independent local update\n")
        .unwrap();
    target.chmod("notes", 0o700).unwrap();
    let before = target.clone();
    archive.extract_files(&mut target, &[".settings"]).unwrap();
    assert_eq!(
        target.entries[".settings"].text,
        original.entries[".settings"].text
    );
    assert!(target.entries[".settings"].changed > before.clock);
    assert_eq!(
        target.entries["notes/change.txt"],
        before.entries["notes/change.txt"]
    );
    assert_eq!(target.entries["notes"], before.entries["notes"]);
}

#[test]
fn selective_missing_members_parents_and_directories_fail_without_partial_effects() {
    let original = sample();
    let (archive, snapshot) = file_backup::create(&original, "project", None, true).unwrap();
    let (delta, _) = file_backup::create(&original, "project", Some(&snapshot), true).unwrap();
    for files in [
        vec![".settings", "missing"],
        vec!["notes"],
        vec![".settings", ".settings"],
        vec!["../outside"],
        vec![],
    ] {
        let mut target = original.clone();
        target.write(".settings", "retain on error").unwrap();
        let before = target.clone();
        assert!(archive.extract_files(&mut target, &files).is_err());
        assert_eq!(target, before);
    }
    let mut target = empty();
    let before = target.clone();
    assert!(
        archive
            .extract_files(&mut target, &["notes/keep.txt"])
            .is_err()
    );
    assert_eq!(target, before);
    assert!(delta.extract_files(&mut target, &[".settings"]).is_err());
    assert_eq!(target, before);
}

#[test]
fn directory_entry_changes_update_only_the_immediate_parent_clock() {
    let mut tree = empty();
    tree.add("notes", node(true), None).unwrap();
    assert_eq!(tree.entries[""].modified, tree.clock);
    let root = tree.entries[""].clone();
    tree.add("notes/file", node(false), Some("first")).unwrap();
    assert_eq!(tree.entries["notes"].modified, tree.clock);
    assert_eq!(tree.entries["notes"].changed, tree.clock);
    assert_eq!(tree.entries[""], root);
    let folder = tree.entries["notes"].clone();
    tree.write("notes/file", "second").unwrap();
    assert_eq!(tree.entries["notes"], folder);
    assert_eq!(tree.entries[""], root);
    tree.remove("notes/file").unwrap();
    assert_eq!(tree.entries["notes"].modified, tree.clock);
    assert_eq!(tree.entries[""], root);
    tree.remove("notes").unwrap();
    assert_eq!(tree.entries[""].modified, tree.clock);
    assert_eq!(tree.entries[""].changed, tree.clock);
}
