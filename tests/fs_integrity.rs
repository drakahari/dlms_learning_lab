//! Original graph vectors; no host device, fsck, mount or disk image is used.
use dlms_learning_lab::fs_integrity::{
    self as f, Correction, ErrorKind, Finding, Inode, Mode, Volume,
};
use std::collections::BTreeMap;
fn fixture() -> Volume {
    Volume {
        version: 1,
        block_count: 32,
        inode_count: 32,
        files: BTreeMap::from([
            (
                12,
                Inode {
                    content: "project report\n".into(),
                    blocks: vec![8],
                    links: 1,
                },
            ),
            (
                13,
                Inode {
                    content: "protected notes\n".into(),
                    blocks: vec![9],
                    links: 1,
                },
            ),
        ]),
        entries: BTreeMap::from([("report.txt".into(), 12), ("notes.txt".into(), 13)]),
        allocated_blocks: (0..10).collect(),
        allocated_inodes: (1..=13).collect(),
        free_blocks: 22,
        free_inodes: 19,
        mounted: false,
    }
}
#[test]
fn healthy_graph_is_derived_readonly_and_idempotent() {
    let mut v = fixture();
    v.validate().unwrap();
    let original = v.clone();
    assert!(v.findings().unwrap().is_empty());
    for mode in [Mode::ReadOnly, Mode::Preen, Mode::Yes] {
        let r = v.check(mode).unwrap();
        assert_eq!(r.status, 0);
        assert!(r.before.is_empty() && r.after.is_empty() && r.corrections.is_empty());
        assert_eq!(v, original);
    }
    assert_eq!(Volume::decode(&serde_json::to_vec(&v).unwrap()).unwrap(), v);
}
#[test]
fn bitmap_and_counter_findings_are_independent_and_repairs_preserve_bytes_and_names() {
    let mut v = fixture();
    v.allocated_blocks.remove(&8);
    v.allocated_blocks.insert(25);
    v.allocated_inodes.remove(&12);
    v.allocated_inodes.insert(24);
    v.free_blocks = 30;
    v.free_inodes = u32::MAX;
    let original = v.clone();
    let r = v.check(Mode::ReadOnly).unwrap();
    assert_eq!(r.status, 4);
    assert_eq!(v, original);
    assert!(r.before.contains(&Finding::BlockBitmap {
        missing: vec![8],
        extra: vec![25]
    }));
    assert!(r.before.contains(&Finding::InodeBitmap {
        missing: vec![12],
        extra: vec![24]
    }));
    assert!(r.before.contains(&Finding::FreeBlocks {
        recorded: 30,
        actual: 22
    }));
    assert!(r.before.contains(&Finding::FreeInodes {
        recorded: u32::MAX,
        actual: 19
    }));
    let fixed = v.check(Mode::Preen).unwrap();
    assert_eq!(fixed.status, 1);
    assert!(fixed.after.is_empty());
    assert_eq!(
        fixed.corrections,
        vec![
            Correction::BlockBitmap,
            Correction::InodeBitmap,
            Correction::FreeBlocks,
            Correction::FreeInodes
        ]
    );
    assert_eq!(v.files, original.files);
    assert_eq!(v.entries, original.entries);
    assert_eq!(v, fixture());
    assert_eq!(v.check(Mode::Preen).unwrap().status, 0);
}
#[test]
fn hard_link_counts_follow_actual_directory_references_and_new_damage_is_detected() {
    let mut v = fixture();
    v.entries.insert("report-alias.txt".into(), 12);
    assert!(v.findings().unwrap().contains(&Finding::LinkCount {
        inode: 12,
        recorded: 1,
        actual: 2
    }));
    let content = v.files[&12].content.clone();
    let r = v.check(Mode::Preen).unwrap();
    assert_eq!(r.status, 1);
    assert_eq!(v.files[&12].links, 2);
    assert_eq!(v.files[&12].content, content);
    assert_eq!(v.entries["report.txt"], v.entries["report-alias.txt"]);
    v.entries.remove("report-alias.txt");
    assert!(!v.findings().unwrap().is_empty());
    assert_eq!(v.check(Mode::ReadOnly).unwrap().status, 4);
    v.check(Mode::Preen).unwrap();
    assert!(v.findings().unwrap().is_empty());
}
#[test]
fn preen_stops_for_nonempty_unattached_data_and_explicit_recovery_preserves_it() {
    let mut v = fixture();
    v.files.get_mut(&12).unwrap().links = 9;
    v.files.insert(
        14,
        Inode {
            content: "unreferenced retained bytes\n".into(),
            blocks: vec![10],
            links: 1,
        },
    );
    let content = v.files[&14].content.clone();
    let bitmap = v.allocated_blocks.clone();
    let r = v.check(Mode::Preen).unwrap();
    assert_eq!(r.status, 5); // A prior count was corrected, an orphan remains.
    assert!(r.blocked.as_ref().unwrap().contains("recovery decision"));
    assert_eq!(v.files[&12].links, 1);
    assert_eq!(v.files[&14].content, content);
    assert_eq!(v.allocated_blocks, bitmap);
    assert!(!v.entries.values().any(|i| *i == 14));
    let r = v.check(Mode::Yes).unwrap();
    assert_eq!(r.status, 1);
    assert!(r.after.is_empty());
    assert_eq!(v.entries["lost+found/#14"], 14);
    assert_eq!(v.files[&14].content, content);
    assert_eq!(v.files[&14].blocks, vec![10]);
    assert_eq!(v.files[&14].links, 1);
    assert_eq!(v.files[&13], fixture().files[&13]);
    assert_eq!(v.check(Mode::ReadOnly).unwrap().status, 0);
}
#[test]
fn recovery_names_never_overwrite_existing_entries_and_empty_orphans_need_no_bytes_guessed() {
    let mut v = fixture();
    v.entries.insert("lost+found/#14".into(), 13);
    v.files.get_mut(&13).unwrap().links = 2;
    v.files.insert(
        14,
        Inode {
            content: "recovered\n".into(),
            blocks: vec![10],
            links: 0,
        },
    );
    v.check(Mode::Yes).unwrap();
    assert_eq!(v.entries["lost+found/#14"], 13);
    assert_eq!(v.entries["lost+found/#14.1"], 14);
    let mut v = fixture();
    v.files.insert(
        14,
        Inode {
            content: String::new(),
            blocks: vec![],
            links: 1,
        },
    );
    v.allocated_inodes.insert(14);
    v.free_inodes = 18;
    let r = v.check(Mode::Preen).unwrap();
    assert_eq!(r.status, 1);
    assert!(
        r.corrections
            .contains(&Correction::ClearedEmpty { inode: 14 })
    );
    assert_eq!(v, fixture());
}
#[test]
fn shared_blocks_and_missing_inodes_are_reported_but_never_guessed_away() {
    for shared in [true, false] {
        let mut v = fixture();
        if shared {
            v.files.get_mut(&13).unwrap().blocks = vec![8];
        } else {
            v.entries.insert("missing.txt".into(), 14);
        }
        let original = v.clone();
        for mode in [Mode::ReadOnly, Mode::Preen, Mode::Yes] {
            let r = v.check(mode).unwrap();
            assert_eq!(r.status, 4);
            assert!(!r.after.is_empty());
            assert!(r.corrections.is_empty());
            assert_eq!(v, original);
            if mode != Mode::ReadOnly {
                assert!(r.blocked.as_ref().unwrap().contains("unsupported"));
            }
        }
    }
    let mut v = fixture();
    v.files.get_mut(&12).unwrap().blocks = vec![32];
    let original = v.clone();
    assert_eq!(v.check(Mode::Yes).unwrap_err().kind, ErrorKind::Unsupported);
    assert_eq!(v, original);
}
#[test]
fn mounted_check_policy_refuses_even_readonly_without_mutation() {
    let mut v = fixture();
    v.mounted = true;
    v.free_blocks = 0;
    let original = v.clone();
    for mode in [Mode::ReadOnly, Mode::Preen, Mode::Yes] {
        let e = v.check(mode).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Mounted);
        assert!(e.message.contains("unreliable"));
        assert_eq!(v, original);
    }
    v.mounted = false;
    assert_eq!(v.check(Mode::Preen).unwrap().status, 1);
    assert!(v.findings().unwrap().is_empty());
}
#[test]
fn resource_failure_keeps_the_original_graph_and_never_loses_file_content() {
    let mut v = fixture();
    v.entries = (0..64).map(|n| (format!("name-{n}"), 12)).collect();
    // Inode13 is nonempty and unattached, and its recovery needs another name.
    let original = v.clone();
    let error = v.check(Mode::Yes).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Limit);
    assert_eq!(v, original);
}
#[test]
fn every_single_bitmap_flip_is_detected_and_repaired_from_references() {
    for b in 0..32 {
        let mut v = fixture();
        if !v.allocated_blocks.remove(&b) {
            v.allocated_blocks.insert(b);
        }
        assert!(!v.findings().unwrap().is_empty());
        assert_eq!(v.check(Mode::Preen).unwrap().status, 1);
        assert_eq!(v, fixture());
    }
    for ino in 1..=32 {
        let mut v = fixture();
        if !v.allocated_inodes.remove(&ino) {
            v.allocated_inodes.insert(ino);
        }
        assert!(!v.findings().unwrap().is_empty());
        v.check(Mode::Yes).unwrap();
        assert_eq!(v, fixture());
    }
    let mut v = fixture();
    v.block_count = 16;
    v.inode_count = 16;
    v.free_blocks = 6;
    v.free_inodes = 3;
    assert!(v.findings().unwrap().is_empty());
}
#[test]
fn malformed_unknown_traversal_duplicate_and_oversized_inputs_fail_before_execution() {
    let v = fixture();
    let raw = serde_json::to_string(&v).unwrap();
    for bad in [
        raw.replace("\"version\":1", "\"version\":2"),
        raw.replace(
            "\"mounted\":false",
            "\"mounted\":false,\"run_shell\":\"touch secret\"",
        ),
        raw.replace("\"report.txt\":12", "\"report.txt\":12,\"report.txt\":13"),
        raw.replace("\"report.txt\":12", "\"../../secret\":12"),
        raw.replace("\"blocks\":[8]", "\"blocks\":[8,9,10,11,12]"),
        raw.replace(
            "\"allocated_blocks\":[0,1,2,3,4,5,6,7,8,9]",
            "\"allocated_blocks\":[0,0]",
        ),
    ] {
        assert_ne!(bad, raw);
        assert!(Volume::decode(bad.as_bytes()).is_err(), "{bad}");
    }
    let mut invalid = v.clone();
    invalid.files.get_mut(&12).unwrap().content = "x".repeat(f::MAX_CONTENT + 1);
    assert_eq!(invalid.validate().unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(
        Volume::decode(&vec![b' '; f::MAX_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut value = serde_json::to_value(&v).unwrap();
    value["files"] = serde_json::Value::Object(
        (12..45)
            .map(|n| (n.to_string(), serde_json::to_value(&v.files[&12]).unwrap()))
            .collect(),
    );
    assert!(Volume::decode(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn numeric_inode_keys_roundtrip_through_tagged_content_and_reject_ambiguous_spellings() {
    let volume = fixture();
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "kind")]
    enum Wrapper {
        Graph {
            volume: dlms_learning_lab::fs_integrity::Volume,
        },
    }
    let raw = serde_json::to_string(&Wrapper::Graph {
        volume: volume.clone(),
    })
    .unwrap();
    let Wrapper::Graph { volume: decoded } = serde_json::from_str(&raw).unwrap();
    assert_eq!(decoded, volume);
    let Wrapper::Graph { volume: decoded } =
        serde_json::from_value(serde_json::from_str(&raw).unwrap()).unwrap();
    assert_eq!(decoded, volume);
    for key in ["012", "+12", "12.0", "-12", "129"] {
        assert!(
            serde_json::from_str::<Wrapper>(&raw.replace("\"12\":", &format!("\"{key}\":")))
                .is_err(),
            "{key}"
        );
    }
}
