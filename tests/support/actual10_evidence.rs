//! Fixed, genuine MAIN10 observations captured before evaluator11 integration.
//! Private bytes stay ignored; no source/path options, subprocess or historical producer.
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
#[path = "../../examples/support/automation_digest.rs"]
mod digest;
const CAPTURE: &str =
    ".build-artifacts/reviews/bash-parameter-defaults/captures/public10-154-350-r1";
const MANIFEST_SHA: &str = "220cebac329a62adcc59e8338df80b94a07a383807d5832dd6905bbf332776a1";
const CAP: usize = 16_000_000;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join(".git").is_dir() && p.join("Cargo.toml").is_file())
        .expect("Fixed private evidence requires a checkout inside this project's MAIN root")
        .to_path_buf()
}
fn bounded(path: &Path, size: Option<usize>) -> Vec<u8> {
    let root = root();
    assert!(path.starts_with(&root));
    for part in path
        .strip_prefix(&root)
        .unwrap()
        .components()
        .scan(root, |at, p| {
            at.push(p);
            Some(at.clone())
        })
    {
        assert!(
            !std::fs::symlink_metadata(part)
                .expect("Missing fixed actual10 evidence")
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
    assert!(raw.len() <= CAP);
    if let Some(size) = size {
        assert_eq!(raw.len(), size);
    }
    raw
}
pub fn sha(raw: &[u8]) -> String {
    digest::hash(raw)
}
pub fn observed(name: &str) -> Vec<u8> {
    // The caller chooses only this closed enum-like set, never a learner path.
    let (size, expected) = match name {
        "writer10-original.json" => (
            633995,
            "bce977c138f5afc1aee4095d5c0e01d290a8cb50a129a7099e09770c720bfd06",
        ),
        "writer10-drafts.json" => (
            626389,
            "254c9267091462fef82b10b27c9ab1d76ec176af368871c9a18ef03a25cccd7c",
        ),
        "actual10-authored.jsonl" => (
            281604,
            "47891c79f71d29bdae297ce3aec9f4cffc657c1f2c09c64123765ec3600e57d8",
        ),
        "actual10-parameters.json" => (
            18948,
            "cd472489902a108f135bf7853fc23b5d95f0b5863818f41d1da6468cdb1a1814",
        ),
        _ => panic!("Only fixed reviewed actual10 evidence names are accepted"),
    };
    let folder = root().join(CAPTURE);
    let raw = bounded(&folder.join("MANIFEST.json"), None);
    assert_eq!(sha(&raw), MANIFEST_SHA);
    assert_eq!(
        bounded(&folder.join("COMPLETE"), Some(65)),
        format!("{MANIFEST_SHA}\n").as_bytes()
    );
    let manifest: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(manifest["actual_evaluator_revision"], 10);
    assert_eq!(manifest["catalog_contexts"], 154);
    assert_eq!(manifest["script_records"], 49);
    assert_eq!(manifest["authored_cases"], 350);
    assert_eq!(manifest["examples"], 49);
    assert_eq!(manifest["parameter_proposals"], 35);
    assert_eq!(manifest["profile_contexts"], 139);
    let pin = manifest["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["file"] == name)
        .unwrap();
    assert_eq!(pin["sha256"], expected);
    assert_eq!(pin["bytes"], size);
    let source = bounded(&folder.join("SOURCE.json"), Some(31028));
    assert_eq!(
        sha(&source),
        "0dd83f123a383f86b7f6e83db15356594c4d26beab48eeb916b204e814450e01"
    );
    let source: serde_json::Value = serde_json::from_slice(&source).unwrap();
    assert_eq!(source["actual_evaluator_revision"], 10);
    assert_eq!(
        source["recipe_sha256"],
        "84241e2ecf944ae7976bdaabab1ccdcc9e4ddab209d1febb446b48f1ff30cb7e"
    );
    let raw = bounded(&folder.join(name), Some(size));
    assert_eq!(sha(&raw), expected);
    raw
}
