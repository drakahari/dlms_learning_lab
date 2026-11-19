//! Fixed independently sealed genuine17; never invokes a producer or host shell.
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::OnceLock,
};
mod digest {
    include!("../../examples/support/automation_digest.rs");
    pub(super) fn stream(mut file: std::fs::File, expected: u64) -> String {
        use std::io::Read;
        let mut state = Sha256::new();
        let mut buffer = [0; 65536];
        let mut n = 0;
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            n += count as u64;
            assert!(n <= expected);
            state.update(&buffer[..count]);
        }
        assert_eq!(n, expected);
        state.finish()
    }
}
const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const FOLDER: &str = ".build-artifacts/reviews/bash-file-lifecycle/captures/public17-269-573-r2";
const ATTESTATION: &str =
    ".build-artifacts/reviews/bash-file-lifecycle/before18-r2-attestation.json";
const ATTESTATION_SHA: &str = "3c51aea64b39fab23995b12b3197b62164c1e22e510cbc7ed91ea573086cbf19";
const MANIFEST_SHA: &str = "54e0d097de1711e34eb1f698cd66a76f23ffc5ee62cef540f6def6a29675ce1a";
const CAP: usize = 16_000_000;
fn path(name: &str) -> PathBuf {
    assert!(
        !name.is_empty()
            && name.split('/').all(|x| !matches!(x, "" | "." | ".."))
            && !name.contains('\\')
    );
    let mut p = PathBuf::from(ROOT);
    for part in name.split('/') {
        p.push(part);
        assert!(
            !std::fs::symlink_metadata(&p)
                .unwrap()
                .file_type()
                .is_symlink(),
            "Evidence alias"
        );
    }
    p
}
fn raw(name: &str) -> Vec<u8> {
    let file = File::open(path(name)).unwrap();
    let n = file.metadata().unwrap().len();
    assert!(n <= CAP as u64);
    let mut raw = Vec::new();
    file.take((CAP + 1) as u64).read_to_end(&mut raw).unwrap();
    assert!(raw.len() <= CAP);
    assert_eq!(raw.len() as u64, n);
    raw
}
fn hash_file(path: &Path, expected: &serde_json::Value) {
    let file = File::open(path).unwrap();
    let info = file.metadata().unwrap();
    assert!(info.is_file() && info.len() <= 1024 * 1024 * 1024);
    assert_eq!(info.len(), expected["size"].as_u64().unwrap());
    assert_eq!(
        digest::stream(file, info.len()),
        expected["sha256"].as_str().unwrap()
    );
}
fn verified() -> &'static serde_json::Value {
    static VALUE: OnceLock<serde_json::Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        let a = raw(ATTESTATION);
        assert_eq!(digest::hash(&a), ATTESTATION_SHA);
        let a: serde_json::Value = serde_json::from_slice(&a).unwrap();
        let outputs = a["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 18);
        for pin in outputs
            .iter()
            .chain(a["retained_inputs"].as_array().unwrap())
            .chain(std::iter::once(&a["producer"]))
        {
            hash_file(&path(pin["path"].as_str().unwrap()), pin);
        }
        let manifest = raw(&format!("{FOLDER}/MANIFEST.json"));
        assert_eq!(digest::hash(&manifest), MANIFEST_SHA);
        assert_eq!(
            raw(&format!("{FOLDER}/COMPLETE")),
            format!("{MANIFEST_SHA}\n").as_bytes()
        );
        a
    })
}
pub fn observed(name: &str) -> Vec<u8> {
    assert!(
        [
            "actual17-authored.jsonl",
            "writer17-original.json",
            "writer17-drafts.json",
            "TYPED-OBSERVATIONS.json",
            "writer17-typed-passed.json",
            "writer17-typed-drafts.json"
        ]
        .contains(&name)
    );
    let a = verified();
    let n = format!("{FOLDER}/{name}");
    let pin = a["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == n)
        .unwrap();
    let out = raw(&n);
    assert_eq!(out.len() as u64, pin["size"].as_u64().unwrap());
    assert_eq!(digest::hash(&out), pin["sha256"].as_str().unwrap());
    out
}
