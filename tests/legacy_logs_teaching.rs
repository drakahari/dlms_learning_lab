//! Literal current-query teaching with fixed virtual text; no host log access.
use dlms_learning_lab::{
    backup,
    engine::State,
    legacy_access_teaching as access,
    legacy_logs_teaching::{self as logs, Learning, Snapshot},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize};
const EXPECTED: &str = "11:01 FAIL task18 rejected\n11:03 FAIL task27 expired";
fn lab() -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic1-logs")
        .unwrap()
}
fn submit(p: &mut Learning, l: &logs::Lesson, c: &str) {
    p.command = c.into();
    p.submit(l).unwrap();
}
#[test]
fn reference_pipeline_alternative_and_experiment_repair_use_exact_independent_text() {
    let s = lab();
    let l = s.legacy_logs_teaching.as_ref().unwrap();
    assert!(l.valid_for(&s.initial, &s.goal));
    let mut p = Learning::new(l).unwrap();
    for c in &l.reference {
        submit(&mut p, l, c);
    }
    assert_eq!(p.current.result, EXPECTED);
    assert!(p.checks(l).iter().all(|r| r.0));
    let mut alternate = Learning::new(l).unwrap();
    submit(&mut alternate, l, "cat /var/log/worker.log | grep FAIL");
    assert_eq!(alternate.current.result, EXPECTED);
    assert!(alternate.checks(l)[0].0);
    for c in &l.try_commands {
        submit(&mut p, l, c);
    }
    assert!(!p.checks(l)[0].0);
    assert!(p.current.result.contains("INFO"));
    assert!(p.checks(l)[0].1.contains("includes lines outside"));
    assert!(p.checks(l)[0].1.len() <= 512);
    for c in &l.repair_commands {
        submit(&mut p, l, c);
    }
    assert_eq!(p.current.result, EXPECTED);
    assert_ne!(l.task.path, "/var/log/app.log");
    assert_eq!(l.task.needle, "FAIL");
}
#[test]
fn exact_find_inspection_queries_case_sensitive_substrings_and_tail_boundaries_remain_legacy() {
    let s = lab();
    let l = s.legacy_logs_teaching.as_ref().unwrap();
    let mut p = Learning::new(l).unwrap();
    submit(&mut p, l, "grep FAIL /var/log/worker.log");
    for c in ["find /var/log -name worker.log", "show", "help"] {
        let out = p.current.execute(c).unwrap();
        assert!(!out.is_empty());
        assert_eq!(p.current.result, EXPECTED);
    }
    assert_eq!(p.current.execute("find /var/log -name *.log").unwrap(), "");
    assert_eq!(p.current.result, EXPECTED);
    assert_eq!(
        p.current.execute("grep fail /var/log/worker.log").unwrap(),
        "(no matching lines)"
    );
    assert!(p.current.result.is_empty());
    assert!(!p.checks(l)[0].0);
    assert!(p.checks(l)[0].1.contains("No text is selected"));
    // Quotes are not removed by this whitespace-token model.
    assert_eq!(
        p.current
            .execute("grep 'FAIL' /var/log/worker.log")
            .unwrap(),
        "(no matching lines)"
    );
    assert!(p.current.result.is_empty());
    assert_eq!(
        p.current.execute("tail -n 0 /var/log/worker.log").unwrap(),
        "(no matching lines)"
    );
    p.current
        .execute("tail -n 10000 /var/log/worker.log")
        .unwrap();
    assert!(p.current.result.contains("INFO"));
    let before = p.current.clone();
    assert!(
        p.current
            .execute("tail -n 10001 /var/log/worker.log")
            .is_err()
    );
    assert_eq!(p.current, before);
    submit(&mut p, l, "grep FAIL /var/log/audit-practice.log");
    assert_ne!(p.current.result, EXPECTED);
    assert!(!p.checks(l)[0].0);
}
#[test]
fn unsupported_or_bad_pipeline_is_atomic_but_successful_overbroad_text_invalidates() {
    let s = lab();
    let l = s.legacy_logs_teaching.as_ref().unwrap();
    let mut p = Learning::new(l).unwrap();
    submit(&mut p, l, "grep FAIL /var/log/worker.log");
    let good = p.current.clone();
    for c in [
        "cat /var/log/worker.log | sed FAIL",
        "cat /var/log/worker.log|grep FAIL",
        "find /var/log -type f",
        "cat /host/log",
        "sh -c cat",
    ] {
        assert!(p.current.execute(c).is_err());
        assert_eq!(p.current, good);
    }
    submit(&mut p, l, "cat /var/log/worker.log");
    assert!(!p.checks(l)[0].0);
    assert_eq!(p.current.files, good.files);
    submit(&mut p, l, "cat /var/log/worker.log | grep FAIL");
    assert!(p.checks(l)[0].0);
    let before = p.current.clone();
    p.selected_action = 0;
    p.insert(l).unwrap();
    assert_eq!(p.command, l.commands[0].command);
    assert_eq!(p.current, before);
}
#[test]
fn typed_text_map_count_duplicate_and_aggregate_limits_are_explicit() {
    let s = lab();
    let l = s.legacy_logs_teaching.as_ref().unwrap();
    let mut raw = serde_json::to_value(&l.fixture).unwrap();
    raw["files"] = serde_json::json!({});
    for i in 0..8 {
        raw["files"][format!("/var/log/a{i}")] = "fixed".into();
    }
    raw["files"]["/var/log/zz-invalid"] = serde_json::json!({"not_text":true});
    assert!(
        serde_json::from_value::<Snapshot>(raw)
            .unwrap_err()
            .to_string()
            .contains("Practice file count limit")
    );
    let duplicate = r#"{"files":{"/var/log/a":"one","/var/log/a":{"not_text":true}},"result":""}"#;
    assert!(
        serde_json::from_str::<Snapshot>(duplicate)
            .unwrap_err()
            .to_string()
            .contains("Duplicate practice file")
    );
    let mut oversized = serde_json::to_value(&l.fixture).unwrap();
    oversized["files"]["/var/log/worker.log"] = "x".repeat(16385).into();
    assert!(serde_json::from_value::<Snapshot>(oversized).is_err());
    let files: std::collections::BTreeMap<_, _> = (0..5)
        .map(|i| (format!("/var/log/large{i}"), "x".repeat(16384)))
        .collect();
    let big = serde_json::json!({"files":files,"result":""});
    assert!(
        serde_json::from_value::<Snapshot>(big)
            .unwrap_err()
            .to_string()
            .contains("aggregate limit")
    );
    let mut direct = l.fixture.clone();
    for i in 0..9 {
        direct
            .files
            .insert(format!("/var/log/extra{i}"), "small".into());
    }
    let before = serde_json::to_value(&direct).unwrap();
    assert!(!direct.valid());
    assert!(direct.execute("show").is_err());
    assert_eq!(serde_json::to_value(direct).unwrap(), before);
    let mut changed = Learning::new(l).unwrap();
    changed
        .current
        .files
        .get_mut("/var/log/worker.log")
        .unwrap()
        .push_str("injected");
    assert!(changed.valid());
    assert!(!changed.valid_for(l));
    assert!(changed.submit(l).is_err());
}
#[test]
fn new_practice_bounds_do_not_reject_old_assignment_result_or_catalog_imports() {
    let s = lab();
    let l = s.legacy_logs_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().command = "grep 'unfinished".into();
    logs::prepare(&mut a, l, false).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .legacy_logs_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Assignment
    );
    let State::Logs { files, result } = &mut a.state else {
        panic!()
    };
    *result = "x".repeat(99999);
    for i in 0..12 {
        files.insert(format!("/var/log/old{i}"), "old synthetic text".into());
    }
    assert!(pack::validate_state(&a.state).is_ok());
    assert!(Snapshot::from_state(&a.state).is_none());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    let loaded = progress::decode(&bytes, &pack::bundled().unwrap()).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
}
#[test]
fn new_learning_drafts_context_and_backup_restore_preserve_every_old_context_and_history() {
    let all = pack::bundled().unwrap();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v7-synthetic-progress-original.json"),
        &all,
    )
    .unwrap();
    for s in &all {
        p.attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
    }
    let old = serde_json::to_value(&p).unwrap();
    assert_eq!(p.attempts.len(), 122);
    assert!(
        old["attempts"]
            .as_object()
            .unwrap()
            .values()
            .any(|a| a["workspace"]["script_history"]["engine_revision"] == 6)
    );
    assert!(old["attempts"].as_object().unwrap().values().any(|a| {
        a["state"]["run"]["engine_revision"] == 7
            || a["workspace"]["script_history_more"]
                .as_array()
                .is_some_and(|v| v.iter().any(|r| r["engine_revision"] == 7))
    }));
    for id in ["lpic1-permissions", "lpic1-traversal", "lpic1-logs"] {
        let s = all.iter().find(|s| s.id == id).unwrap();
        let a = p.attempts.get_mut(id).unwrap();
        workspace::prepare(a);
        if let Some(l) = &s.legacy_access_teaching {
            access::prepare(a, l, false).unwrap();
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .legacy_access_learning
                .as_mut()
                .unwrap();
            l.command = "chmod 'unfinished access".into();
        }
        if let Some(l) = &s.legacy_logs_teaching {
            logs::prepare(a, l, false).unwrap();
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .legacy_logs_learning
                .as_mut()
                .unwrap();
            l.command = "grep 'unfinished log".into();
        }
        a.workspace.as_mut().unwrap().command = "unfinished assignment".into();
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&bytes, &all).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    for (id, before) in old["attempts"].as_object().unwrap() {
        if !["lpic1-permissions", "lpic1-traversal", "lpic1-logs"].contains(&id.as_str()) {
            assert_eq!(
                serde_json::to_value(&restored.attempts[id]).unwrap(),
                *before
            );
        }
    }
    let unchanged = serde_json::to_value(&restored.attempts["lpic1-traversal"]).unwrap();
    let s = all.iter().find(|s| s.id == "lpic1-permissions").unwrap();
    restored.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert_eq!(
        serde_json::to_value(&restored.attempts["lpic1-traversal"]).unwrap(),
        unchanged
    );
    // Backup inspection intentionally rechecks saved scripts; this new-only slice
    // contains no script result, so exact comparison requires no dropped fields.
    let mut local = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in ["lpic1-traversal", "lpic1-logs"] {
        local
            .attempts
            .insert(id.into(), restored.attempts.remove(id).unwrap());
    }
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("backup.json");
    backup::export(&export, &local, &all).unwrap();
    let c = backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(c.rechecked, 0);
    assert_eq!(
        serde_json::to_value(&c.progress).unwrap(),
        serde_json::to_value(&local).unwrap()
    );
    let target = dir.path().join("progress.json");
    backup::restore(&target, &c, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&target, &all).unwrap()).unwrap(),
        serde_json::to_value(&local).unwrap()
    );
    let mut bad = serde_json::to_value(&local).unwrap();
    bad["attempts"]["lpic1-logs"]["workspace"]["legacy_access_learning"] =
        bad["attempts"]["lpic1-traversal"]["workspace"]["legacy_access_learning"].clone();
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
    let before_target = std::fs::read(&target).unwrap();
    let bad_progress: Progress = serde_json::from_value(bad).unwrap();
    assert!(progress::save(&target, &bad_progress).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), before_target);
    let mut foreign = serde_json::to_value(&local).unwrap();
    foreign["attempts"]["lpic1-traversal"]["workspace"]["legacy_access_learning"]["current"]["users"]
        ["foreign"] = serde_json::json!(["root"]);
    assert!(progress::decode(&serde_json::to_vec(&foreign).unwrap(), &all).is_err());
}
#[test]
fn max_legal_fixed_text_roundtrip_and_command_transcript_bounds_remain_finite() {
    let s = lab();
    let mut l = s.legacy_logs_teaching.unwrap();
    let text = "FAIL first\nFAIL second\nINFO other\n".to_owned() + &"x".repeat(16000);
    l.fixture.files.clear();
    for i in 0..4 {
        l.fixture
            .files
            .insert(format!("/var/log/large{i}"), text.clone());
    }
    l.task.path = "/var/log/large0".into();
    l.commands = vec![pack::CommandExample {
        command: "grep FAIL /var/log/large0".into(),
        explanation: "Select the two fixed failures.".into(),
        simulator_only: false,
    }];
    l.reference = vec!["grep FAIL /var/log/large0".into()];
    l.try_commands = vec!["cat /var/log/large0".into()];
    l.repair_commands = l.reference.clone();
    assert!(l.fixture.valid());
    let pretty = serde_json::to_vec_pretty(&l.fixture).unwrap();
    assert_eq!(
        serde_json::from_slice::<Snapshot>(&pretty).unwrap(),
        l.fixture
    );
    assert_eq!(Snapshot::decode(&pretty).unwrap(), l.fixture);
    assert!(Snapshot::decode(&vec![b' '; 1024 * 1024 + 1]).is_err());
    // Independent serialized-size boundary: each legal control byte becomes
    // six JSON bytes.64KiB source +32KiB result exceeds the old proposed512KiB.
    let escaped = Snapshot {
        files: (0..4)
            .map(|i| (format!("/var/log/escaped{i}"), "\u{1}".repeat(16384)))
            .collect(),
        result: "\u{1}".repeat(32768),
    };
    assert!(escaped.valid());
    for encoded in [
        serde_json::to_vec(&escaped).unwrap(),
        serde_json::to_vec_pretty(&escaped).unwrap(),
    ] {
        assert!(encoded.len() > 512 * 1024 && encoded.len() < 1024 * 1024);
        assert_eq!(Snapshot::decode(&encoded).unwrap(), escaped);
    }
    let mut p = Learning::new(&l).unwrap();
    for _ in 0..20 {
        submit(&mut p, &l, "cat /var/log/large0");
    }
    assert!(p.valid_for(&l));
    assert!(p.transcript.iter().map(String::len).sum::<usize>() <= 131072);
    assert!(
        p.transcript
            .iter()
            .any(|s| s.starts_with("[Earlier practice output omitted]"))
    );
    let mut bad: Value = serde_json::to_value(&p).unwrap();
    bad["transcript"] = serde_json::to_value(vec!["x".repeat(40000); 4]).unwrap();
    assert!(serde_json::from_value::<Learning>(bad).is_err());
    let mut count = serde_json::to_value(&p).unwrap();
    let mut rows = vec![Value::String(String::new()); 32];
    rows.push(serde_json::json!({"not_text":true}));
    count["transcript"] = Value::Array(rows);
    assert!(
        serde_json::from_value::<Learning>(count)
            .unwrap_err()
            .to_string()
            .contains("Practice transcript count limit")
    );
}
