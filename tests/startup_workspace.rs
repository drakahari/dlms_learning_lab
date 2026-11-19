#[path = "support/startup_workspace.rs"]
mod fixture;
use dlms_learning_lab::{
    shell::startup::File,
    startup_workspace::{CaseResult, Definition, MAX_TICKS, Run, Scope, Session},
};
#[test]
fn actual_startup_then_new_program_grade_all_four_inputs_and_supported_alternatives() {
    let d = fixture::definition();
    d.validate().unwrap();
    let initial = d.initial.run(&d.initial.starter).unwrap();
    assert_eq!(initial.iter().filter(|r| r.passed()).count(), 2);
    let good = fixture::solved(&d.initial);
    let rows = d.initial.run(&good).unwrap();
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(CaseResult::passed), "{rows:?}");
    for (c, r) in d.initial.cases.iter().zip(&rows) {
        let parent = r.parent.as_ref().unwrap();
        assert!(parent.command_finished);
        assert_eq!(parent.output.files, c.input.files);
        assert_eq!(r.child.as_ref().unwrap().files, c.child.files);
        assert_eq!(
            parent.variables.child["APP_NAME"],
            c.parent["APP_NAME"].value.as_ref().unwrap().as_str()
        );
        assert!(!parent.variables.child.contains_key("LOCAL_NOTE"));
    }
    let mut alt = good.clone();
    alt.0.get_mut("~/.bash_profile").unwrap().text =
        "LOCAL_NOTE=only-here\nexport APP_NAME=$PROJECT-app\n".into();
    assert!(d.initial.run(&alt).unwrap().iter().all(CaseResult::passed));
}
#[test]
fn plausible_wrong_file_no_export_empty_profile_hardcoded_and_print_exit_fail_usefully() {
    let d = fixture::definition();
    let good = fixture::solved(&d.initial);
    for code in [
        "APP_NAME=\"$PROJECT-app\"\nLOCAL_NOTE=only-here",
        "export APP_NAME=harbor-app\nLOCAL_NOTE=only-here",
        "",
        "APP_NAME=\"$PROJECT-app\"; export APP_NAME; LOCAL_NOTE=only-here; printf '%s|%s\\n' \"$APP_NAME\" \"$BASE\"; exit 0",
    ] {
        let mut wrong = good.clone();
        wrong.0.get_mut("~/.bash_profile").unwrap().text = code.into();
        let rows = d.initial.run(&wrong).unwrap();
        assert!(rows.iter().any(|r| !r.passed()), "{code}");
        assert!(
            rows.iter().any(CaseResult::passed),
            "Fallbacks should retain their own correct results"
        );
        assert!(
            rows.iter().any(|r| r.checks.iter().any(|c| !c.0)),
            "Explain each failure"
        );
    }
    let mut wrong = d.initial.starter.clone();
    wrong.0.get_mut("~/.profile").unwrap().text = "export APP_NAME=changed-wrong-file".into();
    let rows = d.initial.run(&wrong).unwrap();
    assert!(!rows[0].passed() && !rows[3].passed());
    let mut invalid = good.clone();
    invalid.0.get_mut("~/.bash_profile").unwrap().text = "while true; do :; done".into();
    let rows = d.initial.run(&invalid).unwrap();
    assert!(
        rows[..2]
            .iter()
            .all(|r| r.error.as_ref().is_some_and(|e| e.contains("Unsupported")))
    );
}
#[test]
fn examples_drafts_proofs_stale_results_stop_and_resume_are_separate_and_honest() {
    let d = fixture::definition();
    let mut session = Session::new(&d);
    session.validate(&d).unwrap();
    let assignment = session.assignment.clone();
    session
        .practice
        .replace(&d.example, &d.example_worked)
        .unwrap();
    assert!(
        session
            .practice
            .run(&d.example)
            .unwrap()
            .iter()
            .all(CaseResult::passed)
    );
    assert_eq!(session.assignment, assignment);
    assert!(!d.solved(&session));
    session
        .assignment
        .replace(&d.initial, &fixture::solved(&d.initial))
        .unwrap();
    session.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&session));
    let raw = serde_json::to_string(&session).unwrap();
    let mut loaded: Session = serde_json::from_str(&raw).unwrap();
    loaded.validate(&d).unwrap();
    assert!(loaded.assignment.pending());
    assert!(!d.solved(&loaded));
    loaded.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&loaded));
    let run = loaded.assignment.begin(&d.initial).unwrap();
    let mut stopped = run;
    stopped.tick();
    stopped.stop();
    for _ in 0..10 {
        assert!(!stopped.tick());
    }
    assert!(loaded.assignment.finish(&d.initial, &stopped).is_err());
    assert!(!d.solved(&loaded));
    loaded
        .assignment
        .edit(
            &d.initial,
            "~/.bash_profile",
            "# unfinished learner draft\nAPP_NAME='",
        )
        .unwrap();
    assert!(!d.solved(&loaded));
    let data = serde_json::to_string(&loaded).unwrap();
    let mut restored: Session = serde_json::from_str(&data).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored.assignment.files, loaded.assignment.files);
    assert!(
        restored
            .assignment
            .run(&d.initial)
            .unwrap()
            .iter()
            .any(|r| !r.passed())
    );
}
#[test]
fn foreign_or_unfinished_jobs_and_forged_saved_stamps_cannot_grant_success() {
    let d = fixture::definition();
    let good = fixture::solved(&d.initial);
    let mut scope = Scope::new(&d.initial);
    scope.replace(&d.initial, &good).unwrap();
    let run = scope.begin(&d.initial).unwrap();
    assert!(scope.finish(&d.initial, &run).is_err());
    let mut foreign = Run::new(&d.example, &d.example_worked).unwrap();
    for _ in 0..MAX_TICKS {
        if foreign.tick() {
            break;
        }
    }
    assert!(foreign.done());
    assert!(scope.finish(&d.initial, &foreign).is_err());
    let mut raw = serde_json::to_value(&scope).unwrap();
    raw["tested"] = serde_json::to_value(&scope.files).unwrap();
    let forged: Scope = serde_json::from_value(raw).unwrap();
    forged.validate(&d.initial).unwrap();
    assert!(forged.pending());
    assert!(!forged.passed(&d.initial));
}
#[test]
fn invalid_definitions_and_boundaries_reject_unknown_files_cases_and_unreachable_work() {
    let d = fixture::definition();
    let mut bad = d.clone();
    bad.initial.cases[0].present.push("~/.bash_profile".into());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example_worked = bad.example.starter.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial
        .starter
        .0
        .insert("/etc/profile".into(), File { text: ":".into() });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0]
        .child
        .env
        .insert("APP_NAME".into(), "injected".into());
    assert!(bad.validate().is_err());
    let raw = serde_json::to_string(&d).unwrap();
    assert_eq!(serde_json::from_str::<Definition>(&raw).unwrap(), d);
    assert!(
        serde_json::from_str::<Definition>(&raw.replacen(
            "\"initial\":{",
            "\"initial\":{\"host\":true,",
            1
        ))
        .is_err()
    );
    let mut s = Session::new(&d);
    s.origin = "foreign".into();
    assert!(s.validate(&d).is_err());
}

fn scenario() -> dlms_learning_lab::pack::Scenario {
    use dlms_learning_lab::{
        engine::{Goal, State},
        pack,
        startup_workspace::Session,
    };
    let d = fixture::definition();
    let solved = fixture::solved(&d.initial);
    let raw = serde_json::json!({"schema_version":1,"scenarios":[{"id":d.initial.id,"version":1,"track":"LPIC-1","topic":"Bash startup settings","unit":"startup-fixture","title":"Source-bound startup fixture","summary":"Developer content fixture; not a registered lab.","objective":d.initial.instructions,"difficulty":"Intermediate","minutes":25,"steps":["Edit the provided files and verify the supplied sessions."],"guide":[],"explanation":"First present personal profile wins; exported settings reach the simulated program.","hints":["Inspect actual file selection."],"sources":["https://www.gnu.org/software/bash/manual/html_node/Bash-Startup-Files.html"],"simplifications":"Bounded in-process settings; no real files or Bash.","initial":State::StartupFiles{simulation:Box::new(Session::new(&d))},"goal":Goal::StartupFiles{definition:Box::new(d.clone())},"solution":[format!("{}{}",dlms_learning_lab::startup_workspace::PREFIX,serde_json::to_string(&solved).unwrap())]}]});
    pack::load(&raw.to_string()).unwrap().remove(0)
}
#[test]
fn source_codec_safe_storage_favorites_backup_and_reverification_preserve_both_drafts() {
    use dlms_learning_lab::{
        backup,
        engine::{self, State},
        progress::{self, Attempt, Progress},
        startup_workspace as w, workspace,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let s = scenario();
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    w::context(&a, &s).unwrap();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, &s));
    if let State::StartupFiles { simulation } = &mut a.state {
        simulation
            .practice
            .edit(
                &fixture::definition().example,
                "~/.profile",
                "# unfinished practice\nVALUE='",
            )
            .unwrap();
    }
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let mut loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    let a = loaded.attempts.get_mut(&s.id).unwrap();
    if let State::StartupFiles { simulation } = &mut a.state {
        let d = fixture::definition();
        assert!(simulation.assignment.pending());
        simulation.assignment.run(&d.initial).unwrap();
        assert!(
            simulation.practice.files.0["~/.profile"]
                .text
                .contains("unfinished practice")
        );
    }
    assert!(progress::current_solved(a, &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, std::slice::from_ref(&s)).unwrap();
    let preview = backup::inspect(
        &export,
        std::slice::from_ref(&s),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    backup::restore(&path, &preview, std::slice::from_ref(&s)).unwrap();
    assert!(
        progress::load(&path, std::slice::from_ref(&s))
            .unwrap()
            .favorites
            .contains(&s.id)
    );
    let before = std::fs::read(&path).unwrap();
    let mut bad: Progress = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    if let State::StartupFiles { simulation } = &mut bad.attempts.get_mut(&s.id).unwrap().state {
        simulation.origin = "foreign".into();
    }
    assert!(progress::save_with_catalog(&path, &bad, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(progress::save(&dir.path().join("unbound.json"), &p).is_err());
}
#[test]
fn source_binding_refuses_foreign_goal_version_and_workspace_metadata() {
    use dlms_learning_lab::{
        engine::{Goal, State},
        progress::Attempt,
        startup_workspace as w, workspace,
    };
    let s = scenario();
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    w::context(&a, &s).unwrap();
    a.scenario_version += 1;
    assert!(w::context(&a, &s).is_err());
    a.scenario_version = s.version;
    if let State::StartupFiles { simulation } = &mut a.state {
        simulation.assignment.case = 99;
    }
    assert!(w::context(&a, &s).is_err());
    let mut bad = s.clone();
    if let Goal::StartupFiles { definition } = &mut bad.goal {
        definition.initial.instructions = "Changed requirements".into();
    }
    assert!(w::source(&bad).is_err());
}
