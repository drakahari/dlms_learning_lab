use dlms_learning_lab::{
    engine::Node,
    file_backup::{Entry, Tree},
    file_backup_terminal::Store,
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
fn initial() -> Store {
    let mut source = empty();
    source.add("keep.txt", node(false), Some("keep\n")).unwrap();
    source
        .add("change.txt", node(false), Some("before\n"))
        .unwrap();
    source
        .add("remove.txt", node(false), Some("obsolete\n"))
        .unwrap();
    source
        .add(".settings", node(false), Some("private\n"))
        .unwrap();
    Store {
        trees: BTreeMap::from([
            ("/srv/data".into(), source),
            ("/restore/data".into(), empty()),
            ("/verify/data".into(), empty()),
        ]),
        storage: "/backups".into(),
        archives: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        cwd: "/srv".into(),
    }
}
fn payload(tree: &Tree) -> BTreeMap<&str, (&Node, Option<&str>, u64)> {
    tree.entries
        .iter()
        .map(|(p, e)| (p.as_str(), (&e.node, e.text.as_deref(), e.modified)))
        .collect()
}
fn changes(store: &mut Store) {
    let source = store.trees.get_mut("/srv/data").unwrap();
    source.write("change.txt", "updated\n").unwrap();
    source.remove("remove.txt").unwrap();
    source.add("new.txt", node(false), Some("new\n")).unwrap();
}
fn base(store: &mut Store) {
    store
        .execute("tar -cf /backups/base.tar -g /backups/sequence.snar data")
        .unwrap();
}

#[test]
fn long_and_clustered_tar_options_produce_equivalent_owned_virtual_archives() {
    let mut a = initial();
    let mut b = a.clone();
    let output = a
        .execute("tar -cvf/backups/base.tar -C/srv -g/backups/base.snar data")
        .unwrap();
    b.execute("tar --directory=/srv --create --verbose --file /backups/base.tar --listed-incremental=/backups/base.snar data").unwrap();
    assert_eq!(a, b);
    assert!(output.starts_with("data/\n"));
    assert!(output.contains("data/.settings\n"));
    assert_eq!(a.execute("tar -tf /backups/base.tar").unwrap(), output);
    assert_eq!(
        a.archives["/backups/base.tar"].entries["change.txt"]
            .text
            .as_deref(),
        Some("before\n")
    );
    a.trees
        .get_mut("/srv/data")
        .unwrap()
        .write("change.txt", "later\n")
        .unwrap();
    assert_eq!(
        a.archives["/backups/base.tar"].entries["change.txt"]
            .text
            .as_deref(),
        Some("before\n")
    );
}

#[test]
fn actual_terminal_chain_and_a_differential_branch_both_restore_the_requested_data() {
    let mut s = initial();
    base(&mut s);
    s.execute("cp /backups/sequence.snar /backups/branch.snar")
        .unwrap();
    changes(&mut s);
    s.execute("tar -c -f /backups/day1.tar -g /backups/sequence.snar -C /srv data")
        .unwrap();
    s.trees
        .get_mut("/srv/data")
        .unwrap()
        .write(".settings", "after\n")
        .unwrap();
    s.execute("tar --create -f /backups/day2.tar -g /backups/sequence.snar data")
        .unwrap();
    s.execute("tar -cf /backups/differential.tar -g /backups/branch.snar data")
        .unwrap();
    for archive in ["base.tar", "day1.tar", "day2.tar"] {
        s.execute(&format!("tar -xGf /backups/{archive} -C /restore"))
            .unwrap();
    }
    for archive in ["base.tar", "differential.tar"] {
        s.execute(&format!("tar --extract --listed-incremental=/dev/null --file=/backups/{archive} --directory=/verify")).unwrap();
    }
    assert_eq!(
        payload(&s.trees["/restore/data"]),
        payload(&s.trees["/srv/data"])
    );
    assert_eq!(
        payload(&s.trees["/verify/data"]),
        payload(&s.trees["/srv/data"])
    );
    assert!(
        !s.archives["/backups/day2.tar"]
            .entries
            .contains_key("change.txt")
    );
    assert!(
        s.archives["/backups/differential.tar"]
            .entries
            .contains_key("change.txt")
    );
}

#[test]
fn wrong_overlay_and_missing_base_are_honest_wrong_states_not_automatically_successful() {
    let mut s = initial();
    base(&mut s);
    changes(&mut s);
    s.execute("tar -cf /backups/delta.tar -g /backups/sequence.snar data")
        .unwrap();
    s.execute("tar -xf /backups/base.tar -C /restore").unwrap();
    s.execute("tar -xf /backups/delta.tar -C /restore").unwrap();
    assert!(s.trees["/restore/data"].entries.contains_key("remove.txt"));
    s.execute("tar -xGf /backups/delta.tar -C /verify").unwrap();
    assert!(!s.trees["/verify/data"].entries.contains_key("keep.txt"));
    assert_ne!(
        payload(&s.trees["/restore/data"]),
        payload(&s.trees["/srv/data"])
    );
    assert_ne!(
        payload(&s.trees["/verify/data"]),
        payload(&s.trees["/srv/data"])
    );
    // Retrying the correct deletion-aware extraction repairs the same attempt.
    s.execute("tar -xGf /backups/delta.tar -C /restore")
        .unwrap();
    assert_eq!(
        payload(&s.trees["/restore/data"]),
        payload(&s.trees["/srv/data"])
    );
}

#[test]
fn terminal_navigation_hidden_files_permissions_and_artifact_copy_are_real_state_changes() {
    let mut s = initial();
    assert!(s.valid());
    assert_eq!(
        s.execute("ls /").unwrap(),
        "backups\nrestore\nsrv\nverify\n"
    );
    assert_eq!(s.execute("ls /srv").unwrap(), "data\n");
    s.execute("cd /srv/data").unwrap();
    assert_eq!(s.execute("pwd").unwrap(), "/srv/data\n");
    assert!(!s.execute("ls").unwrap().contains(".settings"));
    assert!(s.execute("ls -a").unwrap().contains(".settings"));
    assert_eq!(s.execute("cat .settings").unwrap(), "private\n");
    s.execute("mkdir 'two words'").unwrap();
    s.execute("touch 'two words/new'").unwrap();
    s.execute("chmod 600 'two words/new'").unwrap();
    assert_eq!(
        s.trees["/srv/data"].entries["two words/new"].node.mode,
        0o600
    );
    s.execute("rm 'two words/new'").unwrap();
    assert!(!s.trees["/srv/data"].entries.contains_key("two words/new"));
    s.execute("tar -cf /backups/base.tar -C /srv data").unwrap();
    s.execute("cp /backups/base.tar /backups/copy.tar").unwrap();
    assert_eq!(
        s.archives["/backups/base.tar"],
        s.archives["/backups/copy.tar"]
    );
    s.execute("rm /backups/copy.tar").unwrap();
    assert!(!s.archives.contains_key("/backups/copy.tar"));
    assert_eq!(s.execute("ls /backups").unwrap(), "base.tar\n");
}

#[test]
fn unsupported_flags_operators_and_malformed_input_leave_the_whole_workspace_unchanged() {
    let mut s = initial();
    base(&mut s);
    let before = s.clone();
    for command in [
        "tar -czf /backups/bad.tar data",
        "tar -tvf /backups/base.tar",
        "tar -tf /backups/base.tar data/keep.txt data/keep.txt",
        "tar --checkpoint-action=exec=sh -cf /backups/bad.tar data",
        "tar -cxf /backups/bad.tar data",
        "tar -cf /backups/bad.tar -f /backups/other.tar data",
        "tar -cf /backups/bad.tar -C /srv data keep.txt",
        "tar -cf /backups/bad.tar data -C /restore",
        "tar -xf /backups/base.tar data/missing.txt",
        "tar -cf /backups/bad.tar $(printf data)",
        "tar -cf /backups/bad.tar data; rm /backups/base.tar",
        "tar -cf /backups/bad.tar 'unclosed",
        "tar -cf /backups/bad.tar -C /srv ../data",
        "tar -cf /etc/private data",
        "tar -xf /backups/base.tar -C /home/private",
        "touch /etc/escape",
        "rm -r /srv/data",
        "touch /srv/data",
        "cp /backups/base.tar /backups/base.tar",
    ] {
        assert!(s.execute(command).is_err(), "{command}");
        assert_eq!(s, before, "{command}");
    }
    assert!(s.execute(&"x".repeat(4097)).is_err());
    assert_eq!(s, before);
}

#[test]
fn snapshot_source_identity_prevents_silently_applying_a_different_tree_clock() {
    let mut s = initial();
    s.trees
        .insert("/other/data".into(), s.trees["/srv/data"].clone());
    base(&mut s);
    let before = s.clone();
    let error = s
        .execute("tar -cf /backups/wrong.tar -g /backups/sequence.snar -C /other data")
        .unwrap_err();
    assert!(error.contains("different supplied source tree"));
    assert_eq!(s, before);
    s.execute("cp /backups/sequence.snar /backups/independent.snar")
        .unwrap();
    changes(&mut s);
    s.execute("tar -cf /backups/good.tar -g /backups/independent.snar data")
        .unwrap();
    assert!(s.archives.contains_key("/backups/good.tar"));
    assert_ne!(
        s.snapshots["/backups/independent.snar"],
        s.snapshots["/backups/sequence.snar"]
    );
}

#[test]
fn capacity_and_duplicate_map_refusals_preserve_prior_backups() {
    let mut s = initial();
    for n in 0..8 {
        s.execute(&format!("tar -cf /backups/copy-{n}.tar data"))
            .unwrap();
    }
    let before = s.clone();
    assert!(s.execute("tar -cf /backups/excess.tar data").is_err());
    assert_eq!(s, before);
    assert_eq!(
        Store::from_json(&serde_json::to_string(&s).unwrap()).unwrap(),
        s
    );
    let raw = "{\"trees\":{\"/srv/data\":false,\"/srv/data\":false}}";
    // First prove the duplicate guard with a complete valid first Tree value.
    let t = serde_json::to_string(&s.trees["/srv/data"]).unwrap();
    let duplicate = raw.replacen("false", &t, 1);
    assert!(
        Store::from_json(&duplicate)
            .unwrap_err()
            .contains("duplicate virtual artifact/root")
    );
    let mut value = serde_json::to_value(&s).unwrap();
    value["archives"]["/backups/excess.tar"] = serde_json::json!(false);
    assert!(
        Store::from_json(&value.to_string())
            .unwrap_err()
            .contains("capacity exceeded")
    );
    assert!(Store::from_json(&" ".repeat(2 * 1024 * 1024 + 1)).is_err());
    let mut overlapping = initial();
    overlapping.trees.insert("/srv/data/nested".into(), empty());
    assert!(!overlapping.valid());
}

#[test]
fn quoted_command_characters_and_option_looking_names_stay_virtual_data() {
    let mut s = initial();
    let source = s.trees["/srv/data"].clone();
    s.trees.insert("/srv/my;team".into(), source);
    s.execute("tar -cf /backups/literal.tar -- 'my;team'")
        .unwrap();
    assert_eq!(s.archives["/backups/literal.tar"].root, "my;team");
    s.execute("cd /srv/data").unwrap();
    s.execute("touch -- '- option' ").unwrap();
    assert!(s.trees["/srv/data"].entries.contains_key("- option"));
    s.execute("cat -- '- option'").unwrap();
    s.execute("rm -- '- option'").unwrap();
    assert!(!s.trees["/srv/data"].entries.contains_key("- option"));
}

#[test]
fn literal_partial_restore_repairs_one_file_and_keeps_other_live_changes() {
    let mut s = initial();
    base(&mut s);
    s.trees
        .get_mut("/restore/data")
        .unwrap()
        .add("change.txt", node(false), Some("local change\n"))
        .unwrap();
    s.trees
        .get_mut("/restore/data")
        .unwrap()
        .add(".settings", node(false), Some("damaged\n"))
        .unwrap();
    let old = s.trees["/restore/data"].entries["change.txt"].clone();
    assert_eq!(
        s.execute("tar -tf /backups/base.tar data/.settings")
            .unwrap(),
        "data/.settings\n"
    );
    assert_eq!(
        s.execute("tar -xvf /backups/base.tar -C /restore data/.settings")
            .unwrap(),
        "data/.settings\n"
    );
    assert_eq!(
        s.trees["/restore/data"].entries[".settings"]
            .text
            .as_deref(),
        Some("private\n")
    );
    assert_eq!(s.trees["/restore/data"].entries["change.txt"], old);
    assert!(!s.trees["/restore/data"].entries.contains_key("keep.txt"));
}

#[test]
fn special_stdio_and_remote_operands_are_refused_but_explicit_local_names_work() {
    let mut s = initial();
    s.execute("tar -cf /backups/- data").unwrap();
    s.execute("tar -cf /backups/host:archive data").unwrap();
    s.execute("cd /backups").unwrap();
    let before = s.clone();
    for command in [
        "tar -cf - -C /srv data",
        "tar --create --file=- -C /srv data",
        "tar -tf -",
        "tar -xf - -C /restore",
        "tar -cf host:archive -C /srv data",
        "tar -tf user@host:file",
        "tar --list --file=host:/file",
    ] {
        let error = s.execute(command).unwrap_err();
        assert!(
            error.contains("stream") || error.contains("remote tar"),
            "{command}: {error}"
        );
        assert_eq!(s, before);
    }
    assert!(
        s.execute("tar -tf /backups/-")
            .unwrap()
            .contains("data/keep.txt")
    );
    assert!(
        s.execute("tar -tf /backups/host:archive")
            .unwrap()
            .contains("data/keep.txt")
    );
    s.execute("cd /srv/data").unwrap();
    s.execute("touch -- -").unwrap();
    let before = s.clone();
    let error = s.execute("cat -- -").unwrap_err();
    assert!(error.contains("standard input"));
    assert_eq!(s, before);
    assert_eq!(s.execute("cat /srv/data/-").unwrap(), "");
}

#[test]
fn gnu_numeric_directory_modes_preserve_setid_until_explicit_five_digit_clear() {
    for (directory, mode, expected) in [
        (true, "755", 0o6755),
        (true, "0755", 0o6755),
        (true, "2755", 0o6755),
        (true, "4755", 0o6755),
        (true, "00755", 0o755),
        (true, "000", 0o6000),
        (false, "755", 0o755),
        (false, "4755", 0o4755),
    ] {
        let mut s = initial();
        let target = if directory { "" } else { "keep.txt" };
        s.trees
            .get_mut("/srv/data")
            .unwrap()
            .entries
            .get_mut(target)
            .unwrap()
            .node
            .mode = 0o6750;
        let path = if directory {
            "/srv/data"
        } else {
            "/srv/data/keep.txt"
        };
        s.execute(&format!("chmod {mode} {path}")).unwrap();
        assert_eq!(
            s.trees["/srv/data"].entries[target].node.mode, expected,
            "{directory} {mode}"
        );
    }
    let mut s = initial();
    s.execute("chmod 2755 /srv/data").unwrap();
    s.execute("mkdir /srv/data/team").unwrap();
    s.execute("touch /srv/data/team/new").unwrap();
    let tree = &s.trees["/srv/data"];
    assert_eq!(tree.entries["team"].node.mode, 0o2755);
    assert_eq!(tree.entries["team/new"].node.group, "staff");
    assert_eq!(tree.entries["team/new"].node.owner, "root");
    let before = s.clone();
    assert!(s.execute("chmod 77777 /srv/data").is_err());
    assert_eq!(s, before);
}
