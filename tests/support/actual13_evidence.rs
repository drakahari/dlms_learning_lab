//! Fixed genuine MAIN13 data captured/attested before14. No source/path CLI,
//! producer invocation, personal progress or constructed historical outcomes.
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::OnceLock,
};
mod digest {
    // Extend the original immutable development digest locally; do not change
    // original producers, their source pins, or allocate the148MB ELF at once.
    include!("../../examples/support/automation_digest.rs");
    pub(super) fn stream(mut file: std::fs::File, size: u64) -> String {
        use std::io::Read;
        let mut state = Sha256::new();
        let mut chunk = [0u8; 65536];
        let mut count = 0u64;
        loop {
            let n = file
                .read(&mut chunk)
                .expect("Read retained ordinary evidence");
            if n == 0 {
                break;
            }
            count = count
                .checked_add(n as u64)
                .expect("Evidence length overflow");
            assert!(count <= size, "Retained evidence grew while hashing");
            state.update(&chunk[..n]);
        }
        assert_eq!(count, size, "Retained evidence shrank while hashing");
        state.finish()
    }
}
const CAPTURE: &str = ".build-artifacts/reviews/bash-ere-integration/captures/public13-182-442-r1";
pub const MANIFEST_SHA: &str = "8ea1df74fdc71db3f7290e2908467c6ec4162eff7231d91440ad4131bc983441";
const ATTESTATION_SHA: &str = "0855fd6df6c1ad0260fddc8a5c6d1f04c6e065e16d0ce8a176d44581a53b49d1";
const SOURCE_SHA: &str = "a1c391c474ff2d2f44d93852ed1a0369f2d75c71bdad0934d5735705fe7a6f5d";
const CAP: usize = 16_000_000;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join(".git").is_dir() && p.join("Cargo.toml").is_file())
        .expect("Fixed private13 evidence requires this project's MAIN ancestor")
        .to_path_buf()
}
fn bounded(path: &Path, size: Option<usize>) -> Vec<u8> {
    let root = root();
    assert!(path.starts_with(&root));
    let mut at = root.clone();
    for part in path.strip_prefix(&root).unwrap().components() {
        at.push(part);
        assert!(
            !std::fs::symlink_metadata(&at)
                .expect("Missing fixed13 evidence")
                .file_type()
                .is_symlink()
        );
    }
    let file = File::open(path).unwrap();
    let info = file.metadata().unwrap();
    assert!(info.is_file() && info.len() <= CAP as u64);
    if let Some(size) = size {
        assert_eq!(info.len(), size as u64);
    }
    let mut raw = Vec::new();
    file.take(CAP as u64 + 1).read_to_end(&mut raw).unwrap();
    assert_eq!(raw.len() as u64, info.len());
    assert!(raw.len() <= CAP);
    raw
}
fn contained(root: &Path, name: &str) -> PathBuf {
    let p = Path::new(name);
    assert!(
        !p.is_absolute()
            && !p.as_os_str().is_empty()
            && !p.components().any(|c| matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            ))
    );
    root.join(p)
}
pub fn sha(raw: &[u8]) -> String {
    digest::hash(raw)
}

fn ordinary_file(anchor: &Path, relative: &str, size: u64, cap: u64) -> File {
    assert!(size <= cap, "Retained evidence exceeds independent cap");
    assert!(
        !relative.contains('\\')
            && relative
                .split('/')
                .all(|p| !p.is_empty() && p != "." && p != ".."),
        "Noncanonical retained path"
    );
    let path = contained(anchor, relative);
    // Check every ancestor too: a missing/aliased whole tree is never an
    // optional historical-data fallback, even if its last file has good bytes.
    for ancestor in anchor.ancestors() {
        let m = std::fs::symlink_metadata(ancestor).expect("Missing retained evidence root");
        assert!(
            m.is_dir() && !m.file_type().is_symlink(),
            "Retained evidence root alias"
        );
    }
    let mut at = anchor.to_path_buf();
    for component in Path::new(relative).components() {
        at.push(component);
        let m = std::fs::symlink_metadata(&at).expect("Missing retained evidence input");
        assert!(!m.file_type().is_symlink(), "Retained evidence alias");
        if at != path {
            assert!(m.is_dir(), "Retained evidence parent is not a directory");
        }
    }
    let before = std::fs::symlink_metadata(&path).unwrap();
    assert!(
        before.is_file() && before.len() == size,
        "Retained evidence size/type differs"
    );
    let file = File::open(&path).unwrap();
    let opened = file.metadata().unwrap();
    assert!(opened.is_file() && opened.len() == size);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(
            (before.dev(), before.ino()),
            (opened.dev(), opened.ino()),
            "Retained evidence changed before open"
        );
    }
    file
}
fn verify_file(anchor: &Path, relative: &str, size: u64, expected: &str, cap: u64) {
    assert_eq!(expected.len(), 64);
    assert!(
        expected
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    let file = ordinary_file(anchor, relative, size, cap);
    let opened = file.metadata().unwrap();
    assert_eq!(
        digest::stream(file, size),
        expected,
        "Retained evidence hash differs: {relative}"
    );
    let after = std::fs::symlink_metadata(contained(anchor, relative)).unwrap();
    assert!(after.is_file() && !after.file_type().is_symlink() && after.len() == size);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(
            (opened.dev(), opened.ino()),
            (after.dev(), after.ino()),
            "Retained evidence changed while reading"
        );
    }
    #[cfg(not(unix))]
    let _ = opened;
}
fn verify_inventory(
    folder: &Path,
    outputs: &[serde_json::Value],
    inputs: &[serde_json::Value],
    producer_size: u64,
    producer_sha: &str,
) {
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for row in outputs {
        let name = row["file"].as_str().unwrap();
        assert!(names.insert(name), "Duplicate manifested output");
        let size = row["bytes"].as_u64().unwrap();
        total = total.checked_add(size).unwrap();
        assert!(total <= 64_000_000, "Manifest output aggregate exceeds64MB");
        verify_file(
            folder,
            name,
            size,
            row["sha256"].as_str().unwrap(),
            CAP as u64,
        );
    }
    let source_root = folder.join("compiled-source");
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for row in inputs {
        let name = row["file"].as_str().unwrap();
        assert!(names.insert(name));
        let size = row["bytes"].as_u64().unwrap();
        total = total.checked_add(size).unwrap();
        assert!(
            total <= 256_000_000,
            "Retained source aggregate exceeds256MB"
        );
        verify_file(
            &source_root,
            name,
            size,
            row["sha256"].as_str().unwrap(),
            CAP as u64,
        );
    }
    verify_file(
        folder,
        "actual13-producer",
        producer_size,
        producer_sha,
        512_000_000,
    );
    let mut producer = ordinary_file(folder, "actual13-producer", producer_size, 512_000_000);
    let mut magic = [0u8; 4];
    producer.read_exact(&mut magic).unwrap();
    assert_eq!(magic, *b"\x7fELF");
}
fn verified() {
    static EXACT: OnceLock<()> = OnceLock::new();
    EXACT.get_or_init(|| {
        let folder = root().join(CAPTURE);
        assert_eq!(
            sha(&bounded(&folder.join("MANIFEST.json"), None)),
            MANIFEST_SHA
        );
        assert_eq!(
            bounded(&folder.join("COMPLETE"), Some(65)),
            format!("{MANIFEST_SHA}\n").as_bytes()
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&bounded(&folder.join("MANIFEST.json"), None)).unwrap();
        for (name, count) in [
            ("actual_evaluator_revision", 13),
            ("catalog_contexts", 182),
            ("script_records", 57),
            ("authored_cases", 442),
            ("examples", 57),
        ] {
            assert_eq!(manifest[name], count);
        }
        assert_eq!(
            manifest["prior_actual12_manifest_sha256"],
            "109d1c747c8bb9cb8f497abcc5a3e13be5de53de6447b37fccc728ec15aeb448"
        );
        let attestation = bounded(&folder.join("ATTESTATION.json"), None);
        assert_eq!(sha(&attestation), ATTESTATION_SHA);
        let a: serde_json::Value = serde_json::from_slice(&attestation).unwrap();
        assert_eq!(a["actual13_manifest_sha256"], MANIFEST_SHA);
        assert_eq!(a["records_regenerated"], false);
        assert_eq!(
            a["producer"]["sha256"],
            "338e4bccc584e656b168cb055360815c32edc9022f805ff1c3d1ee35895624d3"
        );
        assert_eq!(a["producer"]["bytes"], 148202504u64);
        let raw = bounded(&folder.join("SOURCE.json"), Some(93909));
        assert_eq!(sha(&raw), SOURCE_SHA);
        let source: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(source["actual_evaluator_revision"], 13);
        let inputs = source["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 708);
        let outputs = manifest["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 11);
        let actual_names = outputs
            .iter()
            .map(|row| row["file"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let expected_names = [
            "SOURCE.json",
            "VM-OBSERVATIONS.jsonl",
            "actual13-authored.jsonl",
            "draft-profile/progress-v1.before-shell-v13.json",
            "draft-profile/progress-v1.json",
            "input12-drafts.json",
            "input12-original.json",
            "original-profile/progress-v1.before-shell-v13.json",
            "original-profile/progress-v1.json",
            "writer13-drafts.json",
            "writer13-original.json",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(actual_names, expected_names);

        verify_inventory(
            &folder,
            outputs,
            inputs,
            a["producer"]["bytes"].as_u64().unwrap(),
            a["producer"]["sha256"].as_str().unwrap(),
        );

        assert_eq!(
            source["source_index_sha256"],
            "e44861c14f96ccb573273ea893ccee23fab9a7f96ee1af33022612f2871fc497"
        );
        assert!(folder.join("compiled-source").is_dir());
        let mut names = std::collections::BTreeSet::new();
        for row in inputs {
            let name = row["file"].as_str().unwrap();
            assert!(names.insert(name), "Duplicate retained source input");
            let raw = bounded(
                &contained(&folder.join("compiled-source"), name),
                Some(row["bytes"].as_u64().unwrap() as usize),
            );
            assert_eq!(
                sha(&raw),
                row["sha256"].as_str().unwrap(),
                "Frozen actual13 source changed: {name}"
            );
        }
    });
}
pub fn observed(name: &str) -> Vec<u8> {
    let (size, expected) = match name {
        "actual13-authored.jsonl" => (
            369099,
            "4e7276d4859ea8d6b11430793e297c5cdacf07664d3ba88029ec463fcf0f4c3e",
        ),
        "writer13-original.json" => (
            987003,
            "46bb51b8bf57cd96e89f8e924cfed901771c931b9d9dfede2bfa6b42856941ee",
        ),
        "writer13-drafts.json" => (
            963423,
            "1555ffa94ba2460f0cb1410a68b889e4fd5e606c40f3d6b62d2d981017e1e721",
        ),
        "input12-original.json" => (
            862090,
            "f172e2d9ac4aacb55fafedd7b4c1f39118191c1857276991e46c9ab2264f2cec",
        ),
        "input12-drafts.json" => (
            843821,
            "44ba6ffcdbb141f60313e8192a4ee2349516eb076ba15bb9ac8d2d133564750d",
        ),
        _ => panic!("Only fixed reviewed actual13 observations are accepted"),
    };
    verified();
    let folder = root().join(CAPTURE);
    let manifest: serde_json::Value =
        serde_json::from_slice(&bounded(&folder.join("MANIFEST.json"), None)).unwrap();
    let pin = manifest["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["file"] == name)
        .unwrap();
    assert_eq!(pin["bytes"], size);
    assert_eq!(pin["sha256"], expected);
    let raw = bounded(&folder.join(name), Some(size));
    assert_eq!(sha(&raw), expected);
    raw
}

#[cfg(test)]
mod retention_controls {
    // These are small artificial file inventories only. They never construct a
    // historical Outcome, fake actual13 manifest/seal, or accepted learner run.
    use super::*;
    use serde_json::{Value, json};
    struct Fixture {
        _dir: tempfile::TempDir,
        folder: PathBuf,
        outputs: Vec<Value>,
        inputs: Vec<Value>,
        producer: Vec<u8>,
    }
    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let folder = dir.path().join("inventory");
            std::fs::create_dir(&folder).unwrap();
            std::fs::create_dir(folder.join("profile")).unwrap();
            std::fs::create_dir(folder.join("compiled-source")).unwrap();
            let outputs = vec![
                pin("selected.txt", b"selected"),
                pin("profile/unselected.txt", b"unselected"),
            ];
            let inputs = vec![
                pin("source.rs", b"synthetic source"),
                pin("second.rs", b"second source"),
            ];
            for (root, rows) in [
                (&folder, &outputs),
                (&folder.join("compiled-source"), &inputs),
            ] {
                for row in rows {
                    let data = match row["file"].as_str().unwrap() {
                        "selected.txt" => b"selected".as_slice(),
                        "profile/unselected.txt" => b"unselected".as_slice(),
                        "source.rs" => b"synthetic source".as_slice(),
                        _ => b"second source".as_slice(),
                    };
                    std::fs::write(contained(root, row["file"].as_str().unwrap()), data).unwrap();
                }
            }
            let producer = b"\x7fELFsynthetic non-executable retention fixture".to_vec();
            std::fs::write(folder.join("actual13-producer"), &producer).unwrap();
            Self {
                _dir: dir,
                folder,
                outputs,
                inputs,
                producer,
            }
        }
        fn verify(&self) {
            verify_inventory(
                &self.folder,
                &self.outputs,
                &self.inputs,
                self.producer.len() as u64,
                &sha(&self.producer),
            );
        }
    }
    fn pin(name: &str, data: &[u8]) -> Value {
        json!({"file":name,"bytes":data.len(),"sha256":sha(data)})
    }
    fn rejects<T>(f: impl FnOnce() -> T) {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err());
    }
    #[test]
    fn ordinary_complete_inventory_and_multi_chunk_streaming_positive() {
        let f = Fixture::new();
        f.verify();
        let bytes = vec![b'x'; 131_073];
        std::fs::write(f.folder.join("many.txt"), &bytes).unwrap();
        verify_file(
            &f.folder,
            "many.txt",
            bytes.len() as u64,
            &sha(&bytes),
            CAP as u64,
        );
    }
    #[test]
    fn every_manifested_output_and_producer_is_required_not_just_selected_records() {
        for name in [
            "selected.txt",
            "profile/unselected.txt",
            "actual13-producer",
        ] {
            let f = Fixture::new();
            std::fs::remove_file(f.folder.join(name)).unwrap();
            rejects(|| f.verify());
        }
    }
    #[test]
    fn same_size_tampering_of_output_source_and_elf_fails_streaming_hash() {
        for name in [
            "profile/unselected.txt",
            "compiled-source/source.rs",
            "actual13-producer",
        ] {
            let f = Fixture::new();
            let path = f.folder.join(name);
            let mut bytes = std::fs::read(&path).unwrap();
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
            std::fs::write(path, bytes).unwrap();
            rejects(|| f.verify());
        }
    }
    #[test]
    fn entire_missing_source_and_output_directories_and_capture_root_fail() {
        for name in ["compiled-source", "profile"] {
            let f = Fixture::new();
            std::fs::rename(f.folder.join(name), f._dir.path().join("retained-original")).unwrap();
            rejects(|| f.verify());
        }
        let f = Fixture::new();
        std::fs::rename(&f.folder, f._dir.path().join("moved-inventory")).unwrap();
        rejects(|| f.verify());
    }
    #[cfg(unix)]
    #[test]
    fn aliases_at_leaf_and_parent_are_refused_even_with_identical_bytes() {
        use std::os::unix::fs::symlink;
        for name in [
            "profile/unselected.txt",
            "compiled-source/source.rs",
            "actual13-producer",
            "compiled-source",
            "profile",
        ] {
            let f = Fixture::new();
            let path = f.folder.join(name);
            let saved = f._dir.path().join("original");
            std::fs::rename(&path, &saved).unwrap();
            symlink(&saved, &path).unwrap();
            rejects(|| f.verify());
        }
    }
    #[test]
    fn duplicate_traversal_caps_and_non_elf_magic_are_rejected_without_outcomes() {
        let mut f = Fixture::new();
        f.outputs.push(f.outputs[0].clone());
        rejects(|| f.verify());
        let f = Fixture::new();
        rejects(|| ordinary_file(&f.folder, "profile/../selected.txt", 8, CAP as u64));
        rejects(|| ordinary_file(&f.folder, "selected.txt", 8, 7));
        let mut f = Fixture::new();
        f.producer[0] = b'X';
        std::fs::write(f.folder.join("actual13-producer"), &f.producer).unwrap();
        rejects(|| f.verify());
    }
}
