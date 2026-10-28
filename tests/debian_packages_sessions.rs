//! Source-only prospective app tests; real commands run only in the pure model.
use dlms_learning_lab::{
    backup,
    debian_packages::{self as core, Action},
    debian_packages_teaching::{self as teaching, Learning},
    debian_packages_terminal::{
        self as terminal, Session,
        session::{self, Entry},
    },
    debian_packages_workspace as work,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn first_difference(a: &serde_json::Value, b: &serde_json::Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        for key in a.keys().chain(b.keys()) {
            match (a.get(key), b.get(key)) {
                (Some(a), Some(b)) => {
                    if let Some(p) = first_difference(a, b, &format!("{path}/{key}")) {
                        return Some(p);
                    }
                }
                _ => return Some(format!("{path}/{key} (presence)")),
            }
        }
    }
    Some(path.to_owned())
}
fn scenario() -> Scenario {
    pack::load(include_str!(
        "fixtures/debian-packages-workspace-preview.json"
    ))
    .unwrap()
    .remove(0)
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(s: &State) -> &core::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn sim_mut(s: &mut State) -> &mut core::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.debian_packages_teaching.as_ref().unwrap(), true).unwrap();
    a
}
const REPAIR: &str = "apt-get -y --reinstall install museum-report=1.0-1";
fn execute(t: &mut Session, d: &work::Definition, s: &mut core::State, command: &str) -> bool {
    t.command = command.into();
    t.submit(d, s).unwrap()
}
#[test]
fn real_query_repair_and_rejected_retry_have_fresh_state_and_actual_statuses() {
    let s = scenario();
    let d = definition(&s);
    let mut state = d.initial.clone();
    let mut terminal = Session::default();
    let initial = state.clone();
    assert!(!execute(
        &mut terminal,
        d,
        &mut state,
        "dpkg-query -W museum-report"
    ));
    assert_eq!(state, initial);
    assert_eq!(terminal.entries.last().unwrap().status, 0);
    assert!(!d.checks(&state).iter().all(|r| r.0));
    assert!(execute(&mut terminal, d, &mut state, REPAIR));
    assert!(d.checks(&state).iter().all(|r| r.0));
    assert!(terminal.command.is_empty());
    let good = state.clone();
    assert!(!execute(
        &mut terminal,
        d,
        &mut state,
        "apt-get --unknown install museum-report"
    ));
    assert_eq!(state, good);
    let e = terminal.entries.last().unwrap();
    assert_eq!(e.status, 100);
    assert!(e.diagnostic.is_some());
    assert!(!terminal.command.is_empty());
    assert!(!execute(
        &mut terminal,
        d,
        &mut state,
        "dpkg --compare-versions 1.0 lt 0.9"
    ));
    assert_eq!(terminal.entries.last().unwrap().status, 1);
    assert_eq!(state, good);
    assert!(execute(
        &mut terminal,
        d,
        &mut state,
        "dpkg -r museum-report"
    ));
    assert!(!d.checks(&state).iter().all(|r| r.0));
    assert!(execute(
        &mut terminal,
        d,
        &mut state,
        "apt-get -y install museum-report=1.0-1"
    ));
    assert!(d.checks(&state).iter().all(|r| r.0));
}
#[test]
fn separate_practice_insert_is_not_execution_and_queries_preserve_checked_success() {
    let s = scenario();
    let mut a = attempt(&s);
    let original = a.state.clone();
    let l = s.debian_packages_teaching.as_ref().unwrap();
    let mut practice = Learning::new(l).unwrap();
    assert!(practice.terminal.is_none());
    practice.initialize_terminal(l).unwrap();
    let command = l.example_actions[0].command.as_deref().unwrap();
    let before = practice.simulation.clone();
    practice.insert_command(l, command).unwrap();
    assert_eq!(practice.simulation, before);
    assert!(practice.terminal.as_ref().unwrap().entries.is_empty());
    assert!(practice.submit_command(l).unwrap());
    assert!(practice.checks(l).iter().all(|r| r.0));
    practice.checked = true;
    practice
        .insert_command(l, "dpkg-query -W gallery-report")
        .unwrap();
    assert!(!practice.submit_command(l).unwrap());
    assert!(practice.checked);
    practice
        .insert_command(l, "dpkg -r gallery-report")
        .unwrap();
    assert!(practice.submit_command(l).unwrap());
    assert!(!practice.checked);
    assert!(!practice.checks(l).iter().all(|r| r.0));
    a.workspace.as_mut().unwrap().debian_packages_learning = Some(practice);
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &s));
    a.workspace.as_mut().unwrap().debian_packages_terminal = Some(Session {
        command: "assignment unfinished '".into(),
        ..Default::default()
    });
    assert_ne!(
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_terminal
            .as_ref()
            .unwrap()
            .command,
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
            .command
    );
}
#[test]
fn annotation_checks_semantic_identity_not_noop_coincidence_and_reached_apt_guards() {
    let s = scenario();
    let original = s.debian_packages_teaching.as_ref().unwrap();
    assert!(original.validate().is_ok());
    let mut bad = original.clone();
    bad.example_actions[0].command = Some("dpkg-query -W gallery-report".into());
    assert!(bad.validate().is_err());
    // With already repaired initial payload the declared reinstall is also a
    // no-op; a read-only query STILL cannot be annotated as that operation.
    let mut ready = original.clone();
    let mut state = ready.definition.initial.clone();
    work::apply(
        &ready.definition,
        &mut state,
        &Action::Reinstall {
            name: "gallery-report".into(),
        },
    )
    .unwrap();
    ready.definition.initial = state.clone();
    let mut query = state.clone();
    terminal::execute(
        &ready.definition,
        &mut query,
        "dpkg-query -W gallery-report",
    )
    .unwrap();
    assert_eq!(query, state);
    ready.example_actions[0].command = Some("dpkg-query -W gallery-report".into());
    assert!(ready.validate().is_err());
    let mut bad = original.clone();
    let mut runtime = bad.definition.initial.runtime().clone();
    runtime.cached = Some(bad.definition.initial.catalog().published.clone());
    bad.definition.initial =
        core::State::new(bad.definition.initial.catalog().clone(), runtime).unwrap();
    // The typed exact-artifact operation remains valid, but APT's closed cached
    // candidate contract is not the same operation at this reached state.
    let mut expected = bad.definition.initial.clone();
    assert!(
        work::apply(
            &bad.definition,
            &mut expected,
            &bad.example_actions[0].action
        )
        .is_ok()
    );
    assert!(bad.validate().is_err());
    let mut legacy = original.clone();
    legacy.example_actions[0].command = None;
    assert!(legacy.validate().is_ok());
    assert!(
        serde_json::to_value(&legacy.example_actions[0])
            .unwrap()
            .get("command")
            .is_none()
    );
}
#[test]
fn inspector_reads_current_presence_and_does_not_forge_recorded_ownership_or_expected_payload() {
    let s = scenario();
    let d = definition(&s);
    let mut state = d.initial.clone();
    let mut t = Session::default();
    let known = session::known_paths(&state).unwrap();
    assert!(known.iter().any(|p| p == "/usr/bin/museum-report"));
    assert_eq!(
        session::current_file(&state, "/usr/bin/museum-report").unwrap(),
        Some("damaged\n")
    );
    // Arrange a legitimate missing ordinary payload in the same immutable
    // catalog; this is an operational finding, not a fabricated observation.
    let mut runtime = state.runtime().clone();
    runtime.files.remove("/usr/bin/museum-report");
    state = core::State::new(state.catalog().clone(), runtime).unwrap();
    assert_eq!(
        session::current_file(&state, "/usr/bin/museum-report").unwrap(),
        None
    );
    let before = state.clone();
    let r = terminal::execute(d, &mut state, "dpkg-query -S /usr/bin/museum-report").unwrap();
    assert_eq!(r.status, 0);
    assert!(r.stdout.contains("museum-report"));
    assert_eq!(state, before);
    assert!(session::current_file(&state, "/etc/passwd").is_err());
    assert_eq!(state, before);
    execute(&mut t, d, &mut state, REPAIR);
    assert_eq!(
        session::current_file(&state, "/usr/bin/museum-report").unwrap(),
        Some("report one\n")
    );
    t.selected_path = Some("/usr/bin/museum-report".into());
    assert!(t.valid_for(&state));
    t.selected_path = Some("/etc/passwd".into());
    assert!(!t.valid_for(&state));
}
#[test]
fn raw_command_caps_are_atomic_and_malformed_quotes_control_bytes_roundtrip() {
    let s = scenario();
    let d = definition(&s);
    let mut state = d.initial.clone();
    let mut t = Session {
        command: "apt-get -y install 'unfinished\n\0".into(),
        ..Default::default()
    };
    assert!(t.valid());
    let raw = serde_json::to_vec(&t).unwrap();
    assert_eq!(Session::decode(&raw, &state).unwrap(), t);
    let before = state.clone();
    assert!(!t.submit(d, &mut state).unwrap());
    assert_eq!(state, before);
    assert!(t.command.contains('\0'));
    assert!(t.entries.last().unwrap().diagnostic.is_some());
    t.command = "é".repeat(terminal::MAX_COMMAND_BYTES / 2);
    assert_eq!(t.command.len(), terminal::MAX_COMMAND_BYTES);
    assert!(t.valid());
    t.command.push('x');
    let exact = t.clone();
    assert!(t.submit(d, &mut state).is_err());
    assert_eq!(t, exact);
    assert_eq!(state, before);
    assert!(Session::decode(&vec![b' '; session::MAX_SESSION_JSON + 1], &state).is_err());
}
#[test]
fn transcript_count_aggregate_status_and_duplicate_fields_reject_without_truncation() {
    let s = scenario();
    let state = sim(&s.initial);
    let entry = Entry {
        command: "dpkg-query -W".into(),
        stdout: "record\n".into(),
        stderr: String::new(),
        status: 0,
        diagnostic: None,
    };
    let max = Session {
        version: 1,
        command: String::new(),
        entries: vec![entry.clone(); session::MAX_ENTRIES],
        selected_path: None,
    };
    assert!(max.valid());
    assert_eq!(
        Session::decode(&serde_json::to_vec(&max).unwrap(), state).unwrap(),
        max
    );
    let mut many = serde_json::to_value(&max).unwrap();
    many["entries"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!(42));
    let e = serde_json::from_value::<Session>(many)
        .unwrap_err()
        .to_string();
    assert!(e.contains("count"), "{e}");
    let mut large = max.clone();
    large.entries[0].stdout = "x".repeat(terminal::MAX_OUTPUT_BYTES);
    large.entries[1].stdout = "x".repeat(terminal::MAX_OUTPUT_BYTES);
    assert!(!large.valid());
    assert!(Session::decode(&serde_json::to_vec(&large).unwrap(), state).is_err());
    let mut bad = entry.clone();
    bad.status = 99;
    let invalid = Session {
        entries: vec![bad],
        ..Default::default()
    };
    assert!(!invalid.valid());
    let raw = serde_json::to_string(&Session::default()).unwrap();
    let dup = raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(Session::decode(dup.as_bytes(), state).is_err());
    let mut unknown = serde_json::to_value(Session::default()).unwrap();
    unknown["cached_success"] = true.into();
    assert!(Session::decode(&serde_json::to_vec(&unknown).unwrap(), state).is_err());
}
#[test]
fn runtime_history_eviction_keeps_latest_whole_observations_and_reserves_next_draft_budget() {
    let s = scenario();
    let d = definition(&s);
    let mut state = d.initial.clone();
    let mut t = Session::default();
    for n in 0..40 {
        execute(
            &mut t,
            d,
            &mut state,
            &format!("dpkg --compare-versions {n} eq {n}"),
        );
    }
    assert_eq!(t.entries.len(), session::MAX_ENTRIES);
    assert!(t.entries.first().unwrap().command.contains(" 8 eq 8"));
    assert!(t.entries.last().unwrap().command.contains(" 39 eq 39"));
    t.command = "\\".repeat(terminal::MAX_COMMAND_BYTES);
    assert!(t.valid());
    for raw in [
        serde_json::to_vec(&t).unwrap(),
        serde_json::to_vec_pretty(&t).unwrap(),
    ] {
        assert!(raw.len() < session::MAX_SESSION_JSON);
        assert_eq!(Session::decode(&raw, &state).unwrap(), t);
    }
}
#[test]
fn absent_sessions_preserve_old125_wire_and_wrong_kind_metadata_never_grants_credit() {
    let s = scenario();
    let mut a = attempt(&s);
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_terminal
            .is_none()
    );
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .is_none()
    );
    let mut legacy = a
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap()
        .clone();
    legacy.view = dlms_learning_lab::script_teaching::View::Assignment;
    assert!(
        serde_json::to_value(&legacy)
            .unwrap()
            .get("terminal")
            .is_none()
    );
    assert!(legacy.valid_for(lesson));
    for old in pack::bundled().unwrap() {
        let mut olda = Attempt::new(&old, 1);
        workspace::prepare(&mut olda);
        assert!(
            serde_json::to_value(olda.workspace)
                .unwrap()
                .get("debian_packages_terminal")
                .is_none()
        );
    }
    let other = pack::bundled().unwrap().remove(0);
    a.state = other.initial.clone();
    a.workspace.as_mut().unwrap().debian_packages_terminal = Some(Session::default());
    assert!(!progress::current_solved(&a, &other));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(other.id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &[other]).is_err());
}
#[test]
fn current_credit_ignores_raw_command_and_history_but_rejects_foreign_inspector_context() {
    let s = scenario();
    let mut a = attempt(&s);
    let mut t = Session::default();
    execute(&mut t, definition(&s), sim_mut(&mut a.state), REPAIR);
    a.workspace.as_mut().unwrap().debian_packages_terminal = Some(t);
    assert!(progress::current_solved(&a, &s));
    let t = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_terminal
        .as_mut()
        .unwrap();
    t.command = "malformed proposal '".into();
    t.entries.clear();
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .debian_packages_terminal
        .as_mut()
        .unwrap()
        .selected_path = Some("/outside/fake".into());
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn both_sessions_and_full_attempts_survive_actual_backup_restore_before_selected_reset() {
    let s = scenario();
    let mut all = pack::bundled().unwrap();
    assert_eq!(all.len(), 164);
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    for old in &all {
        p.attempts
            .entry(old.id.clone())
            .or_insert_with(|| Attempt::new(old, 1));
    }
    let old = serde_json::to_value(&p.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    // Explicit backup inspection legitimately rechecks old scripting results.
    // Compare that existing behavior with an independent old-only control,
    // rather than expecting evaluator-8 current results to stay current forever.
    let control_export = dir.path().join("old-only-control.json");
    backup::export(&control_export, &p, &all).unwrap();
    let control = backup::inspect(
        &control_export,
        &all,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let after_inspection = serde_json::to_value(&control.progress.attempts).unwrap();
    let mut a = attempt(&s);
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    let mut t = Session::default();
    execute(
        &mut t,
        definition(&s),
        sim_mut(&mut a.state),
        "dpkg-query -W museum-report",
    );
    t.command = "unfinished assignment '".into();
    a.workspace.as_mut().unwrap().debian_packages_terminal = Some(t);
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_learning
        .as_mut()
        .unwrap();
    l.insert_command(
        lesson,
        lesson.example_actions[0].command.as_deref().unwrap(),
    )
    .unwrap();
    l.submit_command(lesson).unwrap();
    l.insert_command(lesson, "unfinished practice '").unwrap();
    l.view = dlms_learning_lab::script_teaching::View::Example;
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    all.push(s.clone());
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        selected
    );
    let mut ordinary_resume = serde_json::to_value(&loaded.attempts).unwrap();
    ordinary_resume.as_object_mut().unwrap().remove(&s.id);
    assert!(
        ordinary_resume == old,
        "Ordinary resume changed existing work at {:?}",
        first_difference(&old, &ordinary_resume, "")
    );
    let export = dir.path().join("explicit.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts[&s.id]).unwrap(),
        selected
    );
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        selected
    );
    let mut observed = serde_json::to_value(&restored.attempts).unwrap();
    observed.as_object_mut().unwrap().remove(&s.id);
    assert!(
        observed == after_inspection,
        "Existing attempts changed at {:?}",
        first_difference(&after_inspection, &observed, "")
    );
    assert_eq!(restored.favorites, loaded.favorites);
    restored.attempts.insert(s.id.clone(), attempt(&s));
    let mut observed = serde_json::to_value(&restored.attempts).unwrap();
    observed.as_object_mut().unwrap().remove(&s.id);
    assert!(
        observed == after_inspection,
        "Existing attempts changed at {:?}",
        first_difference(&after_inspection, &observed, "")
    );
    assert!(
        restored.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .debian_packages_terminal
            .is_none()
    );
}
