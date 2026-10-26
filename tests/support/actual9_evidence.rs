//! Test-only fixed synthetic evidence. No environment/path arguments, no host
//! command, no generated historical records, and no private bytes in Git.
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
#[path = "../../examples/support/automation_digest.rs"]
mod digest;
const CAPTURE: &str = ".build-artifacts/reviews/bash-safe-automation/captures/public9-133-282-r3";
const MANIFEST_SHA: &str = "8987f4290cf58bdaa7bd7981f4cbeae9454fb4155bff47f4f254c0d8140aac00";
const RAW9_SHA: &str = "0056319207aa4e196d8521bc64e6809daf5dd40049b1aa88aced65462dfbb492";
const MAX_BYTES: usize = 16_000_000;
fn main_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|root| root.join(".git").is_dir() && root.join("Cargo.toml").is_file())
        .expect(
            "Fixed private actual9 replay requires a checkout under this same project's MAIN root",
        )
        .to_path_buf()
}
fn bounded(path: &Path, expected: Option<usize>) -> Vec<u8> {
    let root = main_root();
    assert!(path.starts_with(&root));
    for parent in path
        .strip_prefix(&root)
        .unwrap()
        .components()
        .scan(root, |at, part| {
            at.push(part);
            Some(at.clone())
        })
    {
        assert!(
            !std::fs::symlink_metadata(&parent)
                .expect("Missing fixed private actual9 evidence")
                .file_type()
                .is_symlink(),
            "Symlink in fixed evidence path"
        );
    }
    let file = File::open(path).unwrap();
    let size = file.metadata().unwrap();
    assert!(size.is_file() && size.len() <= MAX_BYTES as u64);
    if let Some(expected) = expected {
        assert_eq!(size.len(), expected as u64);
    }
    let mut raw = Vec::new();
    file.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut raw)
        .unwrap();
    assert!(raw.len() <= MAX_BYTES);
    if let Some(expected) = expected {
        assert_eq!(raw.len(), expected);
    }
    raw
}
pub fn original9() -> Vec<u8> {
    let root = main_root().join(CAPTURE);
    let manifest = bounded(&root.join("MANIFEST.json"), None);
    assert_eq!(digest::hash(&manifest), MANIFEST_SHA);
    assert_eq!(
        bounded(&root.join("COMPLETE"), Some(65)),
        format!("{MANIFEST_SHA}\n").as_bytes()
    );
    let manifest: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
    assert_eq!(manifest["actual_evaluator_revision"], 9);
    assert_eq!(manifest["catalog_contexts"], 133);
    let pin = manifest["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pin| pin["file"] == "vm-profile/writer9-original.json")
        .unwrap();
    assert_eq!(pin["sha256"], RAW9_SHA);
    assert_eq!(pin["bytes"], 529317);
    let raw = bounded(&root.join("vm-profile/writer9-original.json"), Some(529317));
    assert_eq!(digest::hash(&raw), RAW9_SHA);
    let profile: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let attempts = profile["attempts"].as_object().unwrap();
    assert_eq!(attempts.len(), 133);
    assert_eq!(
        profile["migration_receipts"]["shell"],
        serde_json::json!([7, 8, 9])
    );
    let records = attempts
        .values()
        .filter_map(|a| a["state"].get("run").filter(|r| !r.is_null()))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 43);
    assert!(records.iter().all(|r| r["engine_revision"] == 9));
    raw
}
pub fn difference(a: &serde_json::Value, b: &serde_json::Value) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            if a.len() != b.len() {
                return Some("object key count differs".into());
            }
            for (k, v) in a {
                let Some(other) = b.get(k) else {
                    return Some(format!("missing key {:.80}", k));
                };
                if let Some(detail) = difference(v, other) {
                    return Some(format!("{:.80}.{:.160}", k, detail));
                }
            }
            Some("object differs".into())
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
            if a.len() != b.len() {
                return Some("array length differs".into());
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (a, b))| difference(a, b).map(|d| format!("[{i}].{d:.160}")))
        }
        _ => Some("scalar/type differs (values withheld)".into()),
    }
}
pub fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    assert!(
        difference(&a, &b).is_none(),
        "Full-value mismatch: {}",
        difference(&a, &b).unwrap_or_default()
    );
}
