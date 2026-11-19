//! Technical model fixture, not registered curriculum or presentation acceptance.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    source_install_workspace as work,
};
fn scenario() -> pack::Scenario {
    let def: work::Definition =
        serde_json::from_str(include_str!("fixtures/source-install-definition-v1.json")).unwrap();
    let mut solution = def
        .initial
        .reference
        .iter()
        .map(|c| c.0.clone())
        .collect::<Vec<_>>();
    solution.push("source-action {\"kind\":\"check\"}".into());
    let value = serde_json::json!({"schema_version":1,"scenarios":[{"id":"source-install-capability-fixture","version":1,"track":"LPIC-2","topic":"Supplied source installation","unit":"source-install-model-fixture","title":"Verify a staged supplied project","summary":"Technical integration fixture, not authored curriculum.","objective":"Repair a supplied project's configuration and staged behavior while preserving the existing virtual package.","difficulty":"Intermediate","minutes":25,"steps":["Inspect the supplied package and request.","Repair, rebuild, stage and verify every outcome."],"hints":["A rebuilt program does not update an already staged copy."],"sources":["https://www.lpi.org/our-certifications/exam-201-202-objectives/"],"explanation":"Technical fixture only; teaching and native content integration remain pending.","simplifications":"Finite supplied project, typed artifacts, fixed dependency graph and virtual files; no compiler or host execution.","initial":{"kind":"SourceInstall","simulation":work::Session::new(&def)},"goal":{"kind":"SourceInstall","definition":def},"solution":solution,"prerequisites":[]}]});
    pack::load(&serde_json::to_string(&value).unwrap())
        .unwrap()
        .remove(0)
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::SourceInstall { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn engine_grades_resulting_state_and_distinguishes_attempted_partial_check_from_success() {
    let s = scenario();
    let Goal::SourceInstall { definition } = &s.goal else {
        panic!()
    };
    let mut a = Attempt::new(&s, 1);
    assert!(!progress::current_solved(&a, &s));
    for command in &definition.example.reference {
        let d = &mut session(&mut a).practice;
        d.input = command.0.clone();
        d.run(&definition.example).unwrap().unwrap();
    }
    session(&mut a).practice.check(&definition.example).unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "source-action {\"kind\":\"check\"}").unwrap();
    assert!(session(&mut a).assignment.attempted);
    assert!(!session(&mut a).assignment.checked);
    assert!(
        engine::checks(&a.state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("settings"))
    );
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "./configure --prefix=/opt/wrong --with-greeting=Wrong",
    )
    .unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "source-action {\"kind\":\"check\"}").unwrap();
    let rows = engine::checks(&a.state, &s.goal);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.starts_with("Configured project"))
    );
    assert!(rows.iter().any(|r| r.0 && r.1.starts_with("Program case")));
    assert!(!engine::solved(&a.state, &s.goal));
}
#[test]
fn invalid_pairs_unsafe_helpers_and_broken_source_methods_fail_closed_atomically() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    for input in [
        "sh configure",
        "source-action {}",
        "source-action {\"kind\":\"host_shell\"}",
        "cat /etc/passwd",
        "make; pwd",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, input).is_err(),
            "{input}"
        );
        assert_eq!(a.state, before);
    }
    let before = a.state.clone();
    assert!(engine::execute(&mut a.state, "make install").is_err());
    assert_eq!(a.state, before);
    let mut invalid = s.clone();
    let Goal::SourceInstall { definition } = &mut invalid.goal else {
        panic!()
    };
    definition.initial.reference.pop();
    assert!(work::source(&invalid).is_err());
    let mut invalid = s.clone();
    let State::SourceInstall { simulation } = &mut invalid.initial else {
        panic!()
    };
    simulation.assignment.input = "invented starter".into();
    assert!(work::source(&invalid).is_err());
    let mut invalid = s.clone();
    invalid.goal = Goal::Subnet;
    assert!(work::source(&invalid).is_err());
}
#[test]
fn catalog_bound_save_resume_preserves_independent_drafts_favorites_and_existing_file_on_refusal() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    let Goal::SourceInstall { definition } = &s.goal else {
        panic!()
    };
    engine::execute_for_goal(&mut a.state, &s.goal, &definition.initial.reference[0].0).unwrap();
    session(&mut a).assignment.input = "make --unfinished".into();
    session(&mut a).practice.input = "tar -tf ".into();
    let mut progress = Progress {
        schema_version: 1,
        ..Default::default()
    };
    progress.favorites.insert(s.id.clone());
    progress.attempts.insert(s.id.clone(), a.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261111/next-source-install/progress");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, b"existing file must remain").unwrap();
    let old = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &progress).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old);
    assert!(progress::save_with_catalog(&path, &progress, std::slice::from_ref(&s)).is_err());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        old,
        "An unreadable existing profile must be preserved by both APIs"
    );
    let path = path.with_file_name(format!(
        "{}-valid.json",
        path.file_stem().unwrap().to_string_lossy()
    ));
    assert!(!path.exists());
    progress::save_with_catalog(&path, &progress, std::slice::from_ref(&s)).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(loaded.favorites, progress.favorites);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    let old = std::fs::read(&path).unwrap();
    let mut bad = loaded;
    session(bad.attempts.get_mut(&s.id).unwrap())
        .assignment
        .task = "foreign-task".into();
    assert!(progress::save_with_catalog(&path, &bad, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old);
}
#[test]
fn scenario_version_source_and_foreign_workspace_metadata_cannot_claim_completion() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let mut bad = a.clone();
    bad.scenario_version += 1;
    assert!(work::context(&bad, &s).is_err());
    assert!(!progress::current_solved(&bad, &s));
    let mut bad = a.clone();
    bad.workspace = Some(dlms_learning_lab::workspace::WorkspaceData::new());
    bad.workspace.as_mut().unwrap().command = "foreign terminal draft".into();
    assert!(work::context(&bad, &s).is_err());
    let mut bad = a.clone();
    session(&mut bad).assignment.history.clear();
    assert!(work::context(&bad, &s).is_err());
    assert!(!progress::current_solved(&bad, &s));
}
