//! Closed test-only projection of five reviewed teaching amendments.
//! Never used by application loading, grading, persistence or historical producers.
#![allow(dead_code)] // Integration test binaries use different subsets of this helper.
use crate::amendment_sha;
use dlms_learning_lab::{pack::Scenario, script_teaching::Lesson};
use serde_json::Value;
use std::sync::OnceLock;

pub const MANIFEST_SHA256: &str =
    "4f852c80674081bf83cfc15dbdb23458625f684cb34cab4ad355f55b27f04ef2";
pub const MANIFEST_BYTES: &[u8] = include_bytes!("../fixtures/shell-teaching-amendment-v1.json");
pub const ALLOWED: [(&str, &str, usize); 5] = [
    ("scenarios/shell-text-v1.json", "shell-text-capstone", 3),
    ("scenarios/shell-reusable-v1.json", "shell-bundle", 3),
    (
        "scenarios/shell-file-batches-v1.json",
        "shell-file-batch-capstone",
        2,
    ),
    ("scenarios/shell-case-v1.json", "shell-case-interface", 0),
    ("scenarios/shell-arrays-v1.json", "shell-array-priority", 1),
];
pub const CURRENT_PACKS: [(&str, &[u8]); 5] = [
    (
        ALLOWED[0].0,
        include_bytes!("../../scenarios/shell-text-v1.json"),
    ),
    (
        ALLOWED[1].0,
        include_bytes!("../../scenarios/shell-reusable-v1.json"),
    ),
    (
        ALLOWED[2].0,
        include_bytes!("../../scenarios/shell-file-batches-v1.json"),
    ),
    (
        ALLOWED[3].0,
        include_bytes!("../../scenarios/shell-case-v1.json"),
    ),
    (
        ALLOWED[4].0,
        include_bytes!("../../scenarios/shell-arrays-v1.json"),
    ),
];
fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("Missing string {key}"))
}
fn number(v: &Value, key: &str) -> Result<usize, String> {
    v[key]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("Missing integer {key}"))
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    v[key]
        .as_array()
        .ok_or_else(|| format!("Missing array {key}"))
}
fn pinned(bytes: &[u8], pin: &Value) -> Result<(), String> {
    require(
        bytes.len() == number(pin, "bytes")?,
        "Source byte length differs",
    )?;
    require(
        amendment_sha::hash(bytes) == text(pin, "sha256")?,
        "Source SHA256 differs",
    )
}
fn typed_lesson(v: &Value) -> Result<Value, String> {
    let lesson: Lesson = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    serde_json::to_value(lesson).map_err(|e| e.to_string())
}
/// The digest pins every byte, including the exact before/after strings and offsets.
/// Structural checks also document the narrow review boundary independently.
pub fn validate_manifest(bytes: &[u8]) -> Result<Value, String> {
    require(
        amendment_sha::hash(bytes) == MANIFEST_SHA256,
        "Closed teaching manifest SHA256 differs",
    )?;
    let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    require(
        v["schema_version"] == 1 && v["kind"] == "closed-test-only-teaching-amendment",
        "Unknown teaching amendment schema",
    )?;
    let files = array(&v, "files")?;
    require(
        files.len() == ALLOWED.len(),
        "Expected exactly five amended packs",
    )?;
    let pins: Value =
        serde_json::from_str(include_str!("../fixtures/catalog133-original-index.json"))
            .map_err(|e| e.to_string())?;
    let mut operations = 0;
    let mut ranges = 0;
    for (file, (path, id, index)) in files.iter().zip(ALLOWED) {
        require(
            text(file, "path")? == path
                && text(file, "scenario_id")? == id
                && number(file, "scenario_index")? == index,
            "Unexpected teaching amendment identity/order",
        )?;
        let old_pin = array(&pins, "packs")?
            .iter()
            .find(|p| p["path"] == path)
            .ok_or("Original pack pin missing")?;
        require(
            file["before"]["bytes"] == old_pin["bytes"]
                && file["before"]["sha256"] == old_pin["sha256"],
            "Original immutable pack pin differs",
        )?;
        let old = typed_lesson(&file["original_teaching"])?;
        let new = typed_lesson(&file["amended_teaching"])?;
        require(
            old["fixture"] == new["fixture"],
            "Teaching fixture amendment forbidden",
        )?;
        let mut rebuilt = file["original_teaching"].clone();
        let prefix = format!("/scenarios/{index}/teaching/");
        for delta in array(file, "leaf_delta")? {
            let pointer = text(delta, "pointer")?;
            let key = pointer
                .strip_prefix(&prefix)
                .ok_or("Leaf outside named teaching object")?;
            match text(delta, "op")? {
                "replace" => {
                    require(
                        matches!(
                            key,
                            "prerequisite_note" | "example" | "inputs" | "try_change"
                        ),
                        "Unapproved teaching leaf",
                    )?;
                    require(
                        delta["before"].is_string()
                            && delta["after"].is_string()
                            && rebuilt.get(key) == Some(&delta["before"]),
                        "Teaching before leaf differs",
                    )?;
                    rebuilt[key] = delta["after"].clone();
                }
                "append_concept" => {
                    require(key == "concepts", "Concept outside approved leaf")?;
                    let at = number(delta, "index")?;
                    let list = rebuilt["concepts"]
                        .as_array_mut()
                        .ok_or("Missing concepts")?;
                    require(
                        at == list.len() && number(delta, "before_length")? == list.len(),
                        "Concept position differs",
                    )?;
                    let _: dlms_learning_lab::script_teaching::Concept =
                        serde_json::from_value(delta["after"].clone())
                            .map_err(|e| e.to_string())?;
                    list.push(delta["after"].clone());
                    require(list.len() <= 6, "Too many concepts")?;
                }
                _ => return Err("Unapproved amendment operation".into()),
            }
            operations += 1;
        }
        require(
            rebuilt == file["amended_teaching"],
            "Closed leaf reconstruction differs",
        )?;
        let edits = array(file, "byte_edits")?;
        require(!edits.is_empty(), "Missing exact byte edits")?;
        ranges += edits.len();
    }
    require(
        operations == 13 && ranges == 9,
        "Closed operation/range counts differ",
    )?;
    Ok(v)
}
fn reverse_pack(file: &Value, current: &[u8]) -> Result<Vec<u8>, String> {
    pinned(current, &file["after"])?;
    let edits = array(file, "byte_edits")?;
    let mut original = current.to_vec();
    for edit in edits.iter().rev() {
        let start = number(edit, "after_offset")?;
        let after = text(edit, "after_utf8")?.as_bytes();
        let end = start
            .checked_add(after.len())
            .ok_or("Byte range overflow")?;
        require(
            original.get(start..end) == Some(after),
            "Amended byte range differs",
        )?;
        drop(original.splice(start..end, text(edit, "before_utf8")?.bytes()));
    }
    pinned(&original, &file["before"])?;
    // Check before offsets too, then require exact forward reconstruction.
    let mut forward = original.clone();
    for edit in edits.iter().rev() {
        let start = number(edit, "before_offset")?;
        let before = text(edit, "before_utf8")?.as_bytes();
        let end = start
            .checked_add(before.len())
            .ok_or("Byte range overflow")?;
        require(
            forward.get(start..end) == Some(before),
            "Original byte range differs",
        )?;
        drop(forward.splice(start..end, text(edit, "after_utf8")?.bytes()));
    }
    require(forward == current, "Forward byte reconstruction differs")?;
    Ok(original)
}
pub fn manifest() -> &'static Value {
    static VERIFIED: OnceLock<Value> = OnceLock::new();
    VERIFIED.get_or_init(|| {
        let value = validate_manifest(MANIFEST_BYTES).expect("Exact closed teaching amendment");
        for ((path, bytes), file) in CURRENT_PACKS.iter().zip(value["files"].as_array().unwrap()) {
            assert_eq!(file["path"], *path);
            reverse_pack(file, bytes).expect("Exact amended source and original pin");
        }
        value
    })
}
/// Unamended packs still face the caller's original raw length/SHA assertions.
pub fn original_pack(path: &str, current: &[u8]) -> Result<Vec<u8>, String> {
    match manifest()["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == path)
    {
        Some(file) => reverse_pack(file, current),
        None => Ok(current.to_vec()),
    }
}
fn amendment(id: &str) -> Option<&'static Value> {
    manifest()["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["scenario_id"] == id)
}
fn verified_old_lesson(id: &str, current: &Lesson) -> Result<Option<Value>, String> {
    let Some(file) = amendment(id) else {
        return Ok(None);
    };
    let typed = serde_json::to_value(current).map_err(|e| e.to_string())?;
    require(
        typed == typed_lesson(&file["amended_teaching"])?,
        "Whole amended typed Lesson differs",
    )?;
    Ok(Some(typed_lesson(&file["original_teaching"])?))
}
/// Replace only the known typed Lesson; every other Scenario field is retained
/// for the caller's existing complete historical equality assertion.
pub fn original_scenario(current: &Scenario) -> Result<Value, String> {
    let mut value = serde_json::to_value(current).map_err(|e| e.to_string())?;
    if amendment(&current.id).is_some() {
        let lesson = current
            .teaching
            .as_ref()
            .ok_or("Amended teaching missing")?;
        value["teaching"] = verified_old_lesson(&current.id, lesson)?.ok_or("Amendment missing")?;
    }
    Ok(value)
}
pub fn original_catalog(current: &[Scenario]) -> Result<Vec<Value>, String> {
    current.iter().map(original_scenario).collect()
}
/// Historical input comparisons keep their original source. Actual saved
/// records and historical vector construction never pass through this helper.
pub fn historical_example(id: &str, current: &Lesson) -> Result<String, String> {
    match verified_old_lesson(id, current)? {
        Some(old) => Ok(text(&old, "example")?.into()),
        None => Ok(current.example.clone()),
    }
}
