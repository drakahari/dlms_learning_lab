//! Reader bounds only: reuse real revision6 evidence; never execute/regenerate history.
use dlms_learning_lab::{
    engine::State,
    pack,
    progress::{self, Attempt, MigrationReceipts, Progress},
    shell,
    workspace::WorkspaceData,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Original {
    id: String,
    record: shell::Record,
}
fn originals() -> Vec<Original> {
    serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap()
}
fn old_record() -> shell::Record {
    let record = originals().remove(0).record;
    assert_eq!(record.engine_revision, 6);
    assert!(record.valid());
    record
}
fn empty() -> Progress {
    Progress {
        schema_version: 1,
        ..Default::default()
    }
}
fn profile(w: WorkspaceData) -> (Vec<pack::Scenario>, Progress, String) {
    let labs = pack::bundled().unwrap();
    let original = originals().remove(0);
    let lab = labs.iter().find(|s| s.id == original.id).unwrap();
    let mut a = Attempt::new(lab, 1);
    a.workspace = Some(w);
    let id = lab.id.clone();
    let mut p = empty();
    p.attempts.insert(id.clone(), a);
    (labs, p, id)
}

#[test]
fn missing_empty_lists_and_portable_receipts_keep_exact_serialization() {
    let minimal = br#"{"schema_version":1,"clock":0,"favorites":[],"attempts":{}}"#;
    let labs = pack::bundled().unwrap();
    let decoded = progress::decode(minimal, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), minimal);
    let default_receipts: MigrationReceipts = serde_json::from_str("{}").unwrap();
    assert_eq!(
        serde_json::to_string(&default_receipts).unwrap(),
        r#"{"workspace":false,"shell":[]}"#
    );
    let old_workspace = r#"{"version":1,"command":"","service_port":null,"dns_address":null,"dns_serial":null,"mount_device":null,"checked":false,"ipv4_check":null,"practice_check":null,"migrated_host":false,"legacy_host_reviewed":false,"legacy_practice_reviewed":false,"migrated_practice_host":false}"#;
    let w: WorkspaceData = serde_json::from_str(old_workspace).unwrap();
    assert!(w.valid());
    assert_eq!(serde_json::to_string(&w).unwrap(), old_workspace);
    let explicit_empty: WorkspaceData =
        serde_json::from_str(r#"{"version":1,"script_history_more":[]}"#).unwrap();
    assert_eq!(
        serde_json::to_value(&explicit_empty).unwrap(),
        serde_json::to_value(&w).unwrap()
    );
    let mut with_receipts: serde_json::Value = serde_json::from_slice(minimal).unwrap();
    with_receipts["migration_receipts"] = serde_json::json!({"workspace":true,"shell":[1,6,7]});
    let p = progress::decode(&serde_json::to_vec(&with_receipts).unwrap(), &labs).unwrap();
    assert_eq!(p.migration_receipts.as_ref().unwrap().shell, vec![1, 6, 7]);
    assert_eq!(
        serde_json::to_vec(&p).unwrap(),
        minimal,
        "portable data still omits provenance"
    );
}

#[test]
fn maximum_history_field_and_combined_history_contract_remain_distinct() {
    let record = old_record();
    for (first, count, valid) in [(false, 32, true), (true, 31, true), (true, 32, false)] {
        let w = WorkspaceData {
            script_history: first.then(|| record.clone()),
            script_history_more: vec![record.clone(); count],
            ..WorkspaceData::new()
        };
        let bytes = serde_json::to_vec(&w).unwrap();
        let read: WorkspaceData = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(read.valid(), valid);
        assert_eq!(serde_json::to_vec(&read).unwrap(), bytes);
        let (labs, p, _) = profile(read);
        assert_eq!(
            progress::decode(&serde_json::to_vec(&p).unwrap(), &labs).is_ok(),
            valid
        );
    }
    let mut value = serde_json::to_value(WorkspaceData::new()).unwrap();
    value["script_history_more"] = serde_json::json!(vec![record; 33]);
    assert!(
        serde_json::from_value::<WorkspaceData>(value)
            .unwrap_err()
            .to_string()
            .contains("exceeds 32")
    );
}

#[test]
fn excess_history_element_is_rejected_before_its_record_is_decoded() {
    let records = serde_json::to_string(&vec![old_record(); 32]).unwrap();
    let text = format!(
        r#"{{"version":1,"script_history_more":[{},{{"source":false}}]}}"#,
        &records[1..records.len() - 1]
    );
    let error = serde_json::from_str::<WorkspaceData>(&text)
        .unwrap_err()
        .to_string();
    assert!(error.contains("exceeds 32"), "{error}");
    for bad in ["null", "{}", "[null]", "[true]", "[{}]"] {
        let text = format!(r#"{{"version":1,"script_history_more":{bad}}}"#);
        assert!(
            serde_json::from_str::<WorkspaceData>(&text).is_err(),
            "{bad}"
        );
    }
    let huge = format!(
        r#"{{"version":1,"script_history_more":[{}]}}"#,
        std::iter::repeat_n(r#"{"source":"","results":[]}"#, 4096)
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(
        serde_json::from_str::<WorkspaceData>(&huge)
            .unwrap_err()
            .to_string()
            .contains("exceeds 32")
    );
}

#[test]
fn receipt_reader_caps_elements_without_changing_order_future_or_numeric_validation() {
    let maximum = serde_json::json!({"shell":(1..=32).collect::<Vec<u32>>()});
    let r: MigrationReceipts = serde_json::from_value(maximum).unwrap();
    assert_eq!(
        r.shell.len(),
        32,
        "field count cap independent of current revision policy"
    );
    let excessive = format!(
        r#"{{"shell":[{},{{"not":"a revision"}}]}}"#,
        std::iter::repeat_n("1", 32).collect::<Vec<_>>().join(",")
    );
    // MigrationReceipts has no Debug: inspect Err without imposing a new model trait.
    let error = serde_json::from_str::<MigrationReceipts>(&excessive)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("exceeds 32"), "{error}");
    for bad in [
        "null",
        "{}",
        "[null]",
        "[true]",
        "[-1]",
        "[4294967296]",
        "[\"7\"]",
    ] {
        let value = format!(r#"{{"shell":{bad}}}"#);
        assert!(
            serde_json::from_str::<MigrationReceipts>(&value).is_err(),
            "{bad}"
        );
    }
    let labs = pack::bundled().unwrap();
    for shell in [
        vec![0],
        vec![7, 6],
        vec![6, 6],
        vec![shell::EVALUATOR_REVISION + 1],
        (1..=32).collect(),
    ] {
        let mut value = serde_json::to_value(empty()).unwrap();
        value["migration_receipts"] = serde_json::json!({"shell":shell});
        assert!(progress::decode(&serde_json::to_vec(&value).unwrap(), &labs).is_err());
    }
    let excessive = format!(
        r#"{{"shell":[{}]}}"#,
        std::iter::repeat_n("1", 4096).collect::<Vec<_>>().join(",")
    );
    assert!(
        serde_json::from_str::<MigrationReceipts>(&excessive)
            .err()
            .unwrap()
            .to_string()
            .contains("exceeds 32")
    );
}

#[test]
fn all_original_revision6_records_and_current7_empty_record_shape_remain_exact() {
    assert_eq!(
        shell::EVALUATOR_REVISION,
        7,
        "this reader-only change must not activate a new evaluator"
    );
    let labs = pack::bundled().unwrap();
    let mut p = empty();
    let original = originals();
    assert_eq!(original.len(), 33);
    for old in original {
        let lab = labs.iter().find(|s| s.id == old.id).unwrap();
        let mut a = Attempt::new(lab, 1);
        a.state = State::Script {
            source: old.record.source.clone(),
            run: None,
        };
        let mut w = WorkspaceData::new();
        w.script_history_more.push(old.record);
        a.workspace = Some(w);
        p.attempts.insert(old.id, a);
    }
    let before = serde_json::to_vec(&p).unwrap();
    let read = progress::decode(&before, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&read).unwrap(), before);
    for (id, a) in &read.attempts {
        assert!(progress::legacy_script_credit(
            a,
            labs.iter().find(|s| &s.id == id).unwrap()
        ));
    }
    // This is an empty current record shape, NOT a claimed observed solution/history.
    let current = shell::Record {
        engine_revision: 7,
        source: String::new(),
        results: vec![],
    };
    assert!(current.valid());
    let (labs, mut p, id) = profile(WorkspaceData::new());
    p.attempts.get_mut(&id).unwrap().state = State::Script {
        source: String::new(),
        run: Some(current),
    };
    let bytes = serde_json::to_vec(&p).unwrap();
    let read = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&read).unwrap(), bytes);
    assert!(!progress::current_solved(
        &read.attempts[&id],
        labs.iter().find(|s| s.id == id).unwrap()
    ));
    let w = p.attempts.get_mut(&id).unwrap().workspace.as_mut().unwrap();
    w.script_history_more.push(shell::Record {
        engine_revision: 7,
        source: String::new(),
        results: vec![],
    });
    assert!(
        !w.valid(),
        "current records still cannot be relabeled earlier history"
    );
}

#[test]
fn rejected_list_input_and_invalid_direct_values_retain_exact_on_disk_bytes() {
    let (labs, mut p, id) = profile(WorkspaceData::new());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let exact = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&exact).unwrap();
    value["attempts"][&id]["workspace"]["script_history_more"] =
        serde_json::json!(vec![old_record(); 33]);
    let history_overflow = serde_json::to_vec(&value).unwrap();
    assert!(progress::decode(&history_overflow, &labs).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), exact);
    p.attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .script_history_more = vec![old_record(); 33];
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), exact);
    for receipt in [
        serde_json::json!({"shell":vec![1;33]}),
        serde_json::json!({"shell":[8]}),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(&exact).unwrap();
        value["migration_receipts"] = receipt;
        let input = serde_json::to_vec_pretty(&value).unwrap();
        std::fs::write(&path, &input).unwrap();
        assert!(progress::load(&path, &labs).is_err());
        assert!(progress::save(&path, &empty()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), input);
    }
}
