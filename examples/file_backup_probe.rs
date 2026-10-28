//! Fixed development fixtures; no arguments/stdin/learner commands are accepted.
use dlms_learning_lab::{
    engine::Node,
    file_backup::{self, Entry, Tree},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[path = "support/automation_digest.rs"]
mod digest;
fn metadata(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
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
                node: metadata(true, 0o750),
                text: None,
                modified: 0,
                changed: 0,
            },
        )]),
    }
}
fn view(tree: &Tree) -> Value {
    let entries = tree
        .entries
        .iter()
        .map(|(name, e)| {
            (
                name.clone(),
                json!({
                    "directory":e.node.directory,"mode":e.node.mode,"text":e.text
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    json!(entries)
}
fn main() {
    assert_eq!(std::env::args_os().len(), 1, "No source/path arguments");
    let mut source = empty();
    source.add("notes", metadata(true, 0o750), None).unwrap();
    for (path, text) in [
        ("notes/keep.txt", "keep\n"),
        ("notes/change.txt", "before\n"),
        ("notes/remove.txt", "obsolete\n"),
        (".settings", "private=true\n"),
    ] {
        source
            .add(path, metadata(false, 0o640), Some(text))
            .unwrap();
    }
    let (base, snap0) = file_backup::create(&source, "data", None, true).unwrap();
    let mut restored = empty();
    base.extract(&mut restored, true).unwrap();
    let base_view = view(&restored);
    let mut partial = restored.clone();
    partial
        .write("notes/change.txt", "independent local change\n")
        .unwrap();
    partial.chmod("notes", 0o700).unwrap();
    partial.write(".settings", "damaged\n").unwrap();
    base.extract_files(&mut partial, &[".settings"]).unwrap();
    let mut partial_only = empty();
    base.extract_files(&mut partial_only, &[".settings"])
        .unwrap();
    source.write("notes/change.txt", "updated\n").unwrap();
    source.remove("notes/remove.txt").unwrap();
    source
        .add("notes/new.txt", metadata(false, 0o640), Some("new\n"))
        .unwrap();
    source.chmod(".settings", 0o600).unwrap();
    let (day1, snap1) = file_backup::create(&source, "data", Some(&snap0), true).unwrap();
    day1.extract(&mut restored, true).unwrap();
    let sequential = view(&restored);
    let mut overlay = empty();
    base.extract(&mut overlay, true).unwrap();
    day1.extract(&mut overlay, false).unwrap();
    let mut alone = empty();
    day1.extract(&mut alone, true).unwrap();
    let mut reversed = empty();
    day1.extract(&mut reversed, true).unwrap();
    base.extract(&mut reversed, true).unwrap();
    source.write(".settings", "private=still\n").unwrap();
    let (day2, _) = file_backup::create(&source, "data", Some(&snap1), true).unwrap();
    let (diff, _) = file_backup::create(&source, "data", Some(&snap0), true).unwrap();
    day2.extract(&mut restored, true).unwrap();
    let mut differential = empty();
    base.extract(&mut differential, true).unwrap();
    diff.extract(&mut differential, true).unwrap();
    let mut missing = empty();
    base.extract(&mut missing, true).unwrap();
    day2.extract(&mut missing, true).unwrap();
    let mut numeric_modes = BTreeMap::new();
    for (directory, mode) in [
        (true, "755"),
        (true, "0755"),
        (true, "2755"),
        (true, "4755"),
        (true, "00755"),
        (true, "000"),
        (false, "755"),
        (false, "4755"),
    ] {
        let mut tree = empty();
        tree.add(
            "target",
            metadata(directory, 0o6750),
            if directory {
                None
            } else {
                Some("fixture only\n")
            },
        )
        .unwrap();
        let mut store = dlms_learning_lab::file_backup_terminal::Store {
            trees: BTreeMap::from([("/fixture/data".into(), tree)]),
            storage: "/backups".into(),
            archives: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            cwd: "/fixture/data".into(),
        };
        store.execute(&format!("chmod {mode} target")).unwrap();
        numeric_modes.insert(
            format!("{}-{mode}", if directory { "directory" } else { "file" }),
            store.trees["/fixture/data"].entries["target"].node.mode,
        );
    }
    println!(
        "{}",
        json!({
            "kind":"fixed-backup-model-comparison", "revision":1,
            "compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),
            "model_sha256":digest::hash(include_bytes!("../src/file_backup.rs")),
            "terminal_sha256":digest::hash(include_bytes!("../src/file_backup_terminal.rs")),
            "probe_sha256":digest::hash(include_bytes!("file_backup_probe.rs")),
            "digest_sha256":digest::hash(include_bytes!("support/automation_digest.rs")),
            "numeric_modes":numeric_modes,"observations":{
                "base":base_view,"sequential":sequential,"overlay":view(&overlay),
                "delta_alone":view(&alone),"reverse":view(&reversed),
                "sequential_day2":view(&restored),"differential":view(&differential),"missing_middle":view(&missing),
                "partial_overwrite":view(&partial),"partial_only":view(&partial_only)
            }
        })
    );
}
