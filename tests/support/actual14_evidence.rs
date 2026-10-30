//! Fixed genuine MAIN14 data captured/attested before15. No source/path CLI,
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
const CAPTURE: &str = ".build-artifacts/reviews/bash-literal-trim/captures/public14-193-477-r1";
pub const MANIFEST_SHA: Option<&str> =
    Some("6046f5fb00aed71e341cf16c5a09f76d29f74a54b5f590f2740da7f24fa8ab57"); // fill ONLY after genuine14 independent sealing
const ATTESTATION_SHA: Option<&str> =
    Some("9089741a7451c4557c3988d20d27e72cd124e8dca2b92dfc984f0f9099825a3a"); // fill ONLY after genuine14 independent sealing
const SOURCE_SHA: Option<&str> =
    Some("f461f2760b9474a8b667605a34562756934eb0fc32a4d49f40cf4380ff1ca6fc"); // fill ONLY after genuine14 independent sealing
const CAP: usize = 16_000_000;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join(".git").is_dir() && p.join("Cargo.toml").is_file())
        .expect("Fixed private14 evidence requires this project's MAIN ancestor")
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
        "actual14-producer",
        producer_size,
        producer_sha,
        512_000_000,
    );
    let mut producer = ordinary_file(folder, "actual14-producer", producer_size, 512_000_000);
    let mut magic = [0u8; 4];
    producer.read_exact(&mut magic).unwrap();
    assert_eq!(magic, *b"\x7fELF");
}
fn verified() -> &'static serde_json::Value {
    static EXACT: OnceLock<serde_json::Value> = OnceLock::new();
    EXACT.get_or_init(|| {
        let seal = MANIFEST_SHA.expect("Actual14 manifest is not independently sealed");
        let attested = ATTESTATION_SHA.expect("Actual14 attestation is not independently sealed");
        let source_pin = SOURCE_SHA.expect("Actual14 compiled source is not independently sealed");
        let folder = root().join(CAPTURE);
        let raw = bounded(&folder.join("MANIFEST.json"), None);
        assert_eq!(sha(&raw), seal);
        assert_eq!(
            bounded(&folder.join("COMPLETE"), Some(65)),
            format!("{seal}\n").as_bytes()
        );
        let m: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (key, count) in [
            ("actual_evaluator_revision", 14),
            ("catalog_contexts", 193),
            ("script_records", 60),
            ("authored_cases", 477),
            ("examples", 60),
        ] {
            assert_eq!(m[key], count);
        }
        assert_eq!(
            m["prior_actual13_manifest_sha256"],
            "8ea1df74fdc71db3f7290e2908467c6ec4162eff7231d91440ad4131bc983441"
        );
        let raw = bounded(&folder.join("ATTESTATION.json"), None);
        assert_eq!(sha(&raw), attested);
        let a: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(a["actual14_manifest_sha256"], seal);
        assert_eq!(a["records_regenerated"], false);
        let raw = bounded(&folder.join("SOURCE.json"), None);
        assert_eq!(sha(&raw), source_pin);
        let source: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(source["actual_evaluator_revision"], 14);
        assert_eq!(
            source["source_index_sha256"],
            "e06c569f6a16c7d5997d2deedb04ade3cfeab05a2b882f7771b8ad204d3f0f86"
        );
        let inputs = source["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 737);
        let outputs = m["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 11);
        let names = outputs
            .iter()
            .map(|v| v["file"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            names,
            [
                "SOURCE.json",
                "VM-OBSERVATIONS.jsonl",
                "actual14-authored.jsonl",
                "draft-profile/progress-v1.before-shell-v14.json",
                "draft-profile/progress-v1.json",
                "input13-drafts.json",
                "input13-original.json",
                "original-profile/progress-v1.before-shell-v14.json",
                "original-profile/progress-v1.json",
                "writer14-drafts.json",
                "writer14-original.json"
            ]
            .into_iter()
            .collect()
        );
        verify_inventory(
            &folder,
            outputs,
            inputs,
            a["producer"]["bytes"].as_u64().unwrap(),
            a["producer"]["sha256"].as_str().unwrap(),
        );
        m
    })
}
pub fn observed(name: &str) -> Vec<u8> {
    assert!(
        [
            "SOURCE.json",
            "VM-OBSERVATIONS.jsonl",
            "actual14-authored.jsonl",
            "writer14-original.json",
            "writer14-drafts.json",
            "input13-original.json",
            "input13-drafts.json"
        ]
        .contains(&name),
        "Only fixed actual14 evidence names"
    );
    let m = verified();
    let row = m["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["file"] == name)
        .unwrap();
    let raw = bounded(
        &root().join(CAPTURE).join(name),
        Some(row["bytes"].as_u64().unwrap() as usize),
    );
    assert_eq!(sha(&raw), row["sha256"].as_str().unwrap());
    raw
}
