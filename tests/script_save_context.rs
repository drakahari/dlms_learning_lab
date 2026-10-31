//! Source-aware writes must retain the original when a script's identity is wrong.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching, workspace,
};
use serde_json::{Value, json};
use std::{path::Path, sync::OnceLock};

fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| pack::bundled().unwrap())
}
fn full(p: &Progress) -> Value {
    json!({"portable":p,"receipts":p.migration_receipts})
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    a
}
fn initial(path: &Path) -> (Progress, String, Vec<u8>) {
    let s = catalog().iter().find(|s| s.id == "shell-handover").unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(s));
    p.favorites.insert(s.id.clone());
    progress::save_with_catalog(path, &p, catalog()).unwrap();
    let raw = std::fs::read(path).unwrap();
    (progress::load(path, catalog()).unwrap(), s.id.clone(), raw)
}
fn refused(path: &Path, p: &Progress, sources: &[Scenario], raw: &[u8]) {
    let caller = full(p);
    let error = progress::save_with_catalog(path, p, sources).unwrap_err();
    assert!(error.contains("original retained"), "{error}");
    assert_eq!(full(p), caller);
    assert_eq!(std::fs::read(path).unwrap(), raw);
}

#[test]
fn every_current_script_draft_saves_without_executing_or_parsing_editor_text() {
    let mut p = Progress {
        schema_version: 1,
        clock: 1000,
        ..Default::default()
    };
    for s in catalog()
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
    {
        let mut a = fresh(s);
        // Invalid/unfinished source is legitimate editor work, never an executable save hook.
        a.state = State::Script {
            source: "if unfinished syntax\nwhile true; do :; done\n".into(),
            run: None,
        };
        p.attempts.insert(s.id.clone(), a);
    }
    assert!(p.attempts.len() >= 62);
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    assert_eq!(full(&p), before);
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(
        restored
            .attempts
            .values()
            .all(|a| matches!(a.state, State::Script { run: None, .. }))
    );
}

#[test]
fn foreign_state_under_a_known_script_cannot_replace_the_saved_draft() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let (mut p, id, raw) = initial(&path);
    let mut checked = 0;
    for s in catalog()
        .iter()
        .filter(|s| !matches!(s.initial, State::Script { .. }))
    {
        p.attempts.get_mut(&id).unwrap().state = s.initial.clone();
        refused(&path, &p, catalog(), &raw);
        checked += 1;
    }
    assert!(checked > 100);
}

#[test]
fn missing_or_changed_authored_script_identity_is_rejected_before_file_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let (p, id, raw) = initial(&path);
    let s = catalog().iter().find(|s| s.id == id).unwrap();
    refused(&path, &p, &[], &raw);
    let mut changed = s.clone();
    changed.version += 1;
    refused(&path, &p, &[changed], &raw);
    let other = catalog()
        .iter()
        .find(|s| !matches!(s.initial, State::Script { .. }))
        .unwrap();
    let mut changed = s.clone();
    changed.initial = other.initial.clone();
    refused(&path, &p, &[changed], &raw);
    let mut changed = s.clone();
    changed.goal = other.goal.clone();
    assert!(!matches!(changed.goal, Goal::Script { .. }));
    refused(&path, &p, &[changed], &raw);
}
