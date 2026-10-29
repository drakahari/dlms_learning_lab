//! Fixed genuine MAIN11 data captured/attested before12. No source/path CLI,
//! producer invocation, personal progress or constructed historical outcomes.
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::OnceLock,
};
#[path = "../../examples/support/automation_digest.rs"]
mod digest;
const CAPTURE: &str =
    ".build-artifacts/reviews/bash-numeric-validation/captures/public11-164-384-r1";
pub const MANIFEST_SHA: &str = "e303bdecb9f57a89debd77c8300d27b523c94630446bc918e4c5e4c171373168";
const ATTESTATION_SHA: &str = "254d696e4ca90ec508b6034f015ae5710bf2184a30e068d9839a9f93d6523e48";
const SOURCE_SHA: &str = "4e1122d7edf7c85ce15b7bdde738f2a86fd271cbdb14c863cee6e127f52bfb4b";
const CAP: usize = 16_000_000;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join(".git").is_dir() && p.join("Cargo.toml").is_file())
        .expect("Fixed private11 evidence requires this project's MAIN ancestor")
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
                .expect("Missing fixed11 evidence")
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
            ("actual_evaluator_revision", 11),
            ("catalog_contexts", 164),
            ("script_records", 52),
            ("authored_cases", 384),
            ("examples", 52),
        ] {
            assert_eq!(manifest[name], count);
        }
        assert_eq!(
            manifest["prior_actual10_manifest_sha256"],
            "220cebac329a62adcc59e8338df80b94a07a383807d5832dd6905bbf332776a1"
        );
        let attestation = bounded(&folder.join("ATTESTATION.json"), None);
        assert_eq!(sha(&attestation), ATTESTATION_SHA);
        let a: serde_json::Value = serde_json::from_slice(&attestation).unwrap();
        assert_eq!(a["actual11_manifest_sha256"], MANIFEST_SHA);
        assert_eq!(a["records_regenerated"], false);
        assert_eq!(
            a["producer"]["sha256"],
            "0d3e3bf4033cec16ac5cc9580732f23145272904729fd80160f7ddc65e78e608"
        );
        let raw = bounded(&folder.join("SOURCE.json"), Some(36258));
        assert_eq!(sha(&raw), SOURCE_SHA);
        let source: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(source["actual_evaluator_revision"], 11);
        let inputs = source["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 283);
        for row in inputs {
            let name = row["file"].as_str().unwrap();
            let raw = bounded(
                &contained(&folder.join("compiled-source"), name),
                Some(row["bytes"].as_u64().unwrap() as usize),
            );
            assert_eq!(
                sha(&raw),
                row["sha256"].as_str().unwrap(),
                "Frozen actual11 source changed: {name}"
            );
        }
    });
}
pub fn observed(name: &str) -> Vec<u8> {
    let (size, expected) = match name {
        "actual11-authored.jsonl" => (
            315776,
            "bc4db55327e66174754dfc1445c58cb0469b4539f8bfc024f14823cf13758375",
        ),
        "writer11-original.json" => (
            742264,
            "36520522e257712241f56f6e76fda27569c7f3b824977ada2fcf7856aaca1bb2",
        ),
        "writer11-drafts.json" => (
            729330,
            "7cfa23dec5c451b5994876a10ff677908e2cfe87f7c99e080dedb839183cd217",
        ),
        _ => panic!("Only fixed reviewed actual11 observations are accepted"),
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
