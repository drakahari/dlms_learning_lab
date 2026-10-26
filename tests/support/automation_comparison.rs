//! Fixed accepted-public9 evidence reader/classifier. Test only; no producer,
//! external child, arbitrary path/source option, or historical record rewrite.
use serde_json::Value;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
#[path = "../../examples/support/automation_digest.rs"]
mod digest;
pub const MANIFEST_SHA: &str = "8987f4290cf58bdaa7bd7981f4cbeae9454fb4155bff47f4f254c0d8140aac00";
const CAPTURE: &str = ".build-artifacts/reviews/bash-safe-automation/captures/public9-133-282-r3";
const CAP: usize = 4 * 1024 * 1024;
pub fn sha(bytes: &[u8]) -> String {
    digest::hash(bytes)
}
pub fn main_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join(".git").is_dir() && p.join("Cargo.toml").is_file())
        .expect("Fixed same-project private baseline required")
        .into()
}
fn ordinary(path: &Path, cap: usize) -> Vec<u8> {
    let root = main_root();
    let relative = path.strip_prefix(&root).unwrap();
    let mut at = root.clone();
    for c in relative.components() {
        at.push(c);
        let info =
            std::fs::symlink_metadata(&at).expect("Missing immutable private public9 capture");
        assert!(!info.file_type().is_symlink());
        if at != path {
            assert!(info.is_dir());
        } else {
            assert!(info.is_file());
        }
    }
    let before = std::fs::symlink_metadata(path).unwrap();
    assert!(before.len() <= cap as u64);
    let file = File::open(path).unwrap();
    let opened = file.metadata().unwrap();
    assert!(opened.is_file());
    assert_eq!(opened.len(), before.len());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!((opened.dev(), opened.ino()), (before.dev(), before.ino()));
    }
    let mut raw = Vec::new();
    file.take(cap as u64 + 1).read_to_end(&mut raw).unwrap();
    assert_eq!(raw.len() as u64, opened.len());
    assert!(raw.len() <= cap);
    raw
}
pub struct Baseline {
    root: PathBuf,
    pub manifest: Value,
    pub rows: Vec<Value>,
}
impl Baseline {
    pub fn open() -> Self {
        let root = main_root().join(CAPTURE);
        let raw = ordinary(&root.join("MANIFEST.json"), CAP);
        assert_eq!(sha(&raw), MANIFEST_SHA);
        assert_eq!(
            ordinary(&root.join("COMPLETE"), 65),
            format!("{MANIFEST_SHA}\n").as_bytes()
        );
        let manifest: Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(manifest["actual_evaluator_revision"], 9);
        assert_eq!(manifest["catalog_contexts"], 133);
        assert_eq!(manifest["vm_rows"], 1570);
        assert_eq!(manifest["observed_input_vectors"], 380);
        let mut result = Self {
            root,
            manifest,
            rows: vec![],
        };
        let raw = result.read("vm/stdout.raw");
        assert!(result.read("vm/stderr.raw").is_empty());
        result.rows = raw
            .split(|c| *c == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| {
                assert!(line.len() <= 256 * 1024);
                serde_json::from_slice(line).unwrap()
            })
            .collect();
        assert_eq!(result.rows.len(), 1570);
        result
    }
    pub fn read(&self, name: &str) -> Vec<u8> {
        assert!(
            matches!(
                name,
                "vm/stdout.raw" | "vm/stderr.raw" | "vm-profile/writer9-original.json"
            ) || (0..28).any(|i| name == format!("fixed-{i:03}/OBSERVATION.json")),
            "Closed evidence path only"
        );
        let rows = self.manifest["outputs"].as_array().unwrap();
        let matching = rows
            .iter()
            .filter(|p| p["file"] == name)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let pin = matching[0];
        assert!(pin["bytes"].as_u64().unwrap() <= CAP as u64);
        let raw = ordinary(&self.root.join(name), CAP);
        assert_eq!(raw.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(sha(&raw), pin["sha256"].as_str().unwrap());
        raw
    }
    pub fn committed_input(&self, name: &str, bytes: &[u8]) {
        let suffix = format!("/{name}");
        let pins = self.manifest["pins"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["path"].as_str().is_some_and(|p| p.ends_with(&suffix)))
            .collect::<Vec<_>>();
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0]["bytes"], bytes.len());
        assert_eq!(pins[0]["sha256"], sha(bytes));
    }
    pub fn unchanged(&self) {
        let raw = ordinary(&self.root.join("MANIFEST.json"), CAP);
        assert_eq!(sha(&raw), MANIFEST_SHA);
        assert_eq!(
            ordinary(&self.root.join("COMPLETE"), 65),
            format!("{MANIFEST_SHA}\n").as_bytes()
        );
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Named,
    Record,
    Stop,
    Identity,
    Profile,
    Finished,
}
pub fn kind(row: &Value) -> Result<Kind, &'static str> {
    let name = row
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("Missing row kind")?;
    match name {
        "authored-case"
        | "worked-example"
        | "unfinished-assignment"
        | "unfinished-example"
        | "imported-example"
        | "fixed-automation-public9"
        | "vm-boundary-public9" => {
            if row
                .get("id")
                .and_then(Value::as_str)
                .is_none_or(|s| s.is_empty() || s.len() > 160)
            {
                return Err("Named row requires bounded ID");
            }
            if !row
                .get("slice")
                .and_then(Value::as_u64)
                .is_some_and(|s| [1, 16, 32, 64].contains(&s))
            {
                return Err("Named row requires known slice");
            }
            if row.get("actual").is_none_or(|a| !a.is_object()) {
                return Err("Named row requires Outcome");
            }
            Ok(Kind::Named)
        }
        "authored-record" => Ok(Kind::Record),
        "stopped-ungraded" => Ok(Kind::Stop),
        "identity" => Ok(Kind::Identity),
        "profile-evidence" => Ok(Kind::Profile),
        "finished" => Ok(Kind::Finished),
        _ => Err("Unknown public9 row kind"),
    }
}
fn difference(a: &Value, b: &Value) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some("key count".into());
            }
            for (k, v) in a {
                let Some(other) = b.get(k) else {
                    return Some(format!("missing {:.64}", k));
                };
                if let Some(d) = difference(v, other) {
                    return Some(format!("{:.64}.{:.128}", k, d));
                }
            }
            Some("object".into())
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some("array length".into());
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (a, b))| difference(a, b).map(|d| format!("[{i}].{d:.128}")))
        }
        _ => Some("scalar/type (values withheld)".into()),
    }
}
pub fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize, label: &str) {
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    let d = difference(&a, &b);
    assert!(d.is_none(), "{label:.100}: {}", d.unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_without_id_is_distinct_from_named_outcomes() {
        let stop = serde_json::json!({"kind":"stopped-ungraded","slice":1,"actual":{},"record_created":false});
        assert_eq!(kind(&stop), Ok(Kind::Stop));
        for n in [
            "authored-case",
            "worked-example",
            "unfinished-assignment",
            "unfinished-example",
            "imported-example",
            "fixed-automation-public9",
            "vm-boundary-public9",
        ] {
            let mut row = stop.clone();
            row["kind"] = n.into();
            assert!(kind(&row).is_err());
            row["id"] = "fixed".into();
            assert_eq!(kind(&row), Ok(Kind::Named));
            row["slice"] = 2.into();
            assert!(kind(&row).is_err());
        }
    }
    #[test]
    fn unknown_metadata_or_named_kinds_never_disappear() {
        for name in [
            "future-kind",
            "stopped-ungraded-extra",
            "authored-case-extra",
        ] {
            assert!(
                kind(&serde_json::json!({"kind":name,"id":"x","slice":1,"actual":{}})).is_err()
            );
        }
    }
    #[test]
    fn full_outcome_diagnostics_detect_field_changes_without_dumping_bodies() {
        let a = serde_json::json!({"actual":{"files":{"/work/keep.txt":"x".repeat(1000)},"error":null,"status":0,"stdout":"","stderr":""}});
        let mut b = a.clone();
        b["actual"]["files"]["/work/keep.txt"] = "different".into();
        let d = difference(&a, &b).unwrap();
        assert!(d.len() < 256);
        assert!(d.contains("files"));
        assert!(!d.contains("different"));
        assert!(difference(&a, &a).is_none());
    }
}
