//! App-owned file-loading boundary. All paths/data are private disposable fixtures.
use dlms_learning_lab::pack;
use std::{fs, process::Command};
#[test]
fn ordinary_pack_files_are_bounded_before_decoding_and_original_bytes_stay_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pack.json");
    let mut bytes = include_bytes!("../scenarios/starter-v1.json").to_vec();
    bytes.resize(pack::MAX_PACK_BYTES, b' ');
    fs::write(&path, &bytes).unwrap();
    assert_eq!(pack::load_file(&path).unwrap().len(), 9);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    bytes.push(b' ');
    fs::write(&path, &bytes).unwrap();
    assert!(pack::load_file(&path).unwrap_err().contains("exceeds"));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::write(&path, [0xff, 0xfe]).unwrap();
    assert!(pack::load_file(&path).unwrap_err().contains("UTF-8"));
    assert_eq!(fs::read(&path).unwrap(), [0xff, 0xfe]);
    fs::write(&path, r#"{"schema_version":999,"scenarios":[]}"#).unwrap();
    assert!(pack::load_file(&path).is_err());
    assert!(
        pack::load_file(dir.path())
            .unwrap_err()
            .contains("ordinary file")
    );
}
#[test]
fn cli_uses_the_bounded_loader_and_reports_invalid_data_without_opening_a_gui() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pack.json");
    fs::write(&path, include_bytes!("../scenarios/starter-v1.json")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_dlms-learning-lab"))
        .arg("--validate-pack")
        .arg(&path)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap().trim(),
        "Validated 9 scenarios"
    );
    let oversized = vec![b' '; pack::MAX_PACK_BYTES + 1];
    fs::write(&path, &oversized).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_dlms-learning-lab"))
        .arg("--validate-pack")
        .arg(&path)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("exceeds")
    );
    assert_eq!(fs::read(&path).unwrap(), oversized);
}
#[cfg(unix)]
#[test]
fn linked_pack_files_are_rejected_before_their_targets_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.json");
    let link = dir.path().join("linked.json");
    let data = include_bytes!("../scenarios/starter-v1.json");
    fs::write(&target, data).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(
        pack::load_file(&link)
            .unwrap_err()
            .contains("ordinary file")
    );
    assert_eq!(fs::read(target).unwrap(), data);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn unsupported_reader_ceiling_returns_a_diagnostic_without_overflow_or_data_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.json");
    fs::write(&path, b"retained").unwrap();
    assert!(
        dlms_learning_lab::progress::read_bounded(&path, usize::MAX)
            .unwrap_err()
            .contains("limit is too large")
    );
    assert_eq!(fs::read(path).unwrap(), b"retained");
}

#[test]
fn advisory_difficulty_labels_are_shared_with_filters_and_unknown_tags_still_fail() {
    use dlms_learning_lab::catalog::{self, Filter};
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    for difficulty in pack::DIFFICULTIES {
        value["scenarios"][0]["difficulty"] = difficulty.into();
        let labs = pack::load(&value.to_string()).unwrap();
        let filter = Filter {
            difficulty: difficulty.into(),
            ..Default::default()
        };
        assert!(catalog::matches(&labs[0], &filter, false, false, false));
        let mismatch = Filter {
            difficulty: "Unknown".into(),
            ..Default::default()
        };
        assert!(!catalog::matches(&labs[0], &mismatch, false, false, false));
    }
    value["scenarios"][0]["difficulty"] = "Expert".into();
    assert!(pack::load(&value.to_string()).is_err());
}
