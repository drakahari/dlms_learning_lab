//! Individually authored source tasks; technical fixtures are never catalog labs.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    source_install_workspace as work,
};
const IDS: [&str; 3] = [
    "lpic2-source-project-build",
    "lpic2-source-stage-install",
    "lpic2-source-stale-handover",
];
fn catalog() -> Vec<pack::Scenario> {
    static LABS: std::sync::OnceLock<Vec<pack::Scenario>> = std::sync::OnceLock::new();
    LABS.get_or_init(|| pack::accepted409().unwrap()).clone()
}
fn labs() -> Vec<pack::Scenario> {
    let all = catalog();
    IDS.iter()
        .map(|id| all.iter().find(|s| s.id == *id).unwrap().clone())
        .collect()
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::SourceInstall { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn state(a: &mut Attempt) -> &mut work::Session {
    let State::SourceInstall { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(a: &mut Attempt, s: &pack::Scenario) {
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(progress::current_solved(a, s));
}
#[test]
fn registered_source_unit_preserves_all406_previous_labs_and156_units() {
    let old = pack::accepted406().unwrap();
    let current = catalog();
    assert_eq!(old.len(), 406);
    assert_eq!(current.len(), 409);
    assert_eq!(
        serde_json::to_value(&current[..406]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        current[406..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let old_units = course::accepted406(&old).unwrap();
    let units = course::accepted409(&current).unwrap();
    assert_eq!(old_units.len(), 156);
    assert_eq!(units.len(), 157);
    for old in old_units {
        assert_eq!(
            serde_json::to_value(units.iter().find(|u| u.id == old.id).unwrap()).unwrap(),
            serde_json::to_value(old).unwrap()
        );
    }
    let new = units
        .iter()
        .find(|u| u.id == "linux-source-installation")
        .unwrap();
    assert_eq!(new.labs, IDS);
    for (track, count) in [
        ("CCNA", 93),
        ("LPIC-1", 133),
        ("LPIC-2", 103),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(
            current.iter().filter(|s| s.track == track).count(),
            count,
            "{track}"
        );
    }
}
#[test]
fn all_three_taught_methods_are_reachable_and_worked_examples_never_earn_assignment_credit() {
    for source in labs() {
        let d = def(&source);
        d.validate().unwrap();
        let mut a = Attempt::new(&source, 1);
        assert!(!progress::current_solved(&a, &source));
        for command in &d.example.reference {
            let p = &mut state(&mut a).practice;
            p.input = command.0.clone();
            p.run(&d.example).unwrap().unwrap();
        }
        state(&mut a).practice.check(&d.example).unwrap();
        assert!(!progress::current_solved(&a, &source));
        solve(&mut a, &source);
        let rows = engine::checks(&a.state, &source.goal);
        assert!(rows.iter().all(|r| r.0));
        assert_eq!(
            rows.iter()
                .filter(|r| r.1.starts_with("Program case"))
                .count(),
            4
        );
        assert_ne!(d.initial.required, d.example.required);
        assert_ne!(d.initial.facts, d.example.facts);
    }
}
#[test]
fn legitimate_defaults_other_source_directories_and_dependency_orders_pass() {
    let all = labs();
    let first = &all[0];
    let t = &def(first).initial;
    let mut a = Attempt::new(first, 1);
    for command in [
        "mkdir -p /work".to_string(),
        format!("tar --extract --file={} --directory=/work", t.facts.archive),
        format!("cd /work/{}", t.facts.archive_root),
        format!(
            "./configure --prefix {} --with-greeting '{}'",
            t.required.prefix, t.required.greeting
        ),
        "make all".into(),
        "make check".into(),
        "source-action {\"kind\":\"check\"}".into(),
    ] {
        engine::execute_for_goal(&mut a.state, &first.goal, &command).unwrap();
    }
    assert!(progress::current_solved(&a, first));
    let second = &all[1];
    let t = &def(second).initial;
    let mut a = Attempt::new(second, 1);
    for command in [
        format!("tar -xf {} -C /usr/src", t.facts.archive),
        format!("cd /usr/src/{}", t.facts.archive_root),
        "./configure".into(),
        "make install DESTDIR=/review/install".into(),
        "make check".into(),
        "source-action {\"kind\":\"check\"}".into(),
    ] {
        engine::execute_for_goal(&mut a.state, &second.goal, &command).unwrap();
    }
    assert!(
        progress::current_solved(&a, second),
        "Requested defaults and install's build dependency are valid, not one exact command sequence."
    );
    let third = &all[2];
    let mut a = Attempt::new(third, 1);
    for command in [
        "make DESTDIR=/review/release install",
        "make check",
        "source-action {\"kind\":\"check\"}",
    ] {
        engine::execute_for_goal(&mut a.state, &third.goal, command).unwrap();
    }
    assert!(progress::current_solved(&a, third));
}
#[test]
fn wrong_configuration_and_partial_rebuilds_show_specific_failing_and_passing_results() {
    let all = labs();
    let third = &all[2];
    let mut a = Attempt::new(third, 1);
    engine::execute_for_goal(&mut a.state, &third.goal, "make check").unwrap();
    engine::execute_for_goal(
        &mut a.state,
        &third.goal,
        "source-action {\"kind\":\"check\"}",
    )
    .unwrap();
    let rows = engine::checks(&a.state, &third.goal);
    assert!(
        rows.iter()
            .any(|r| r.0 && r.1.starts_with("Current package self-check"))
    );
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("Program case")));
    assert!(rows.iter().any(|r| r.0 && r.1.contains("Package version")));
    assert!(!progress::current_solved(&a, third));
    let first = &all[0];
    let mut a = Attempt::new(first, 1);
    solve(&mut a, first);
    engine::execute_for_goal(
        &mut a.state,
        &first.goal,
        "./configure --prefix=/opt/wrong --with-greeting='Welcome,'",
    )
    .unwrap();
    assert!(!progress::current_solved(&a, first));
    engine::execute_for_goal(&mut a.state, &first.goal, "make check").unwrap();
    engine::execute_for_goal(
        &mut a.state,
        &first.goal,
        "source-action {\"kind\":\"check\"}",
    )
    .unwrap();
    let rows = engine::checks(&a.state, &first.goal);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.starts_with("Configured project"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("/opt/wrong") && r.1.contains("/opt/site-tools")),
        "Prefix feedback identifies actual and requested values"
    );
    assert!(rows.iter().any(|r| r.0 && r.1.contains("Single name")));
}
#[test]
fn passing_program_cases_cannot_hide_damage_to_existing_live_files() {
    let source = labs().remove(1);
    let mut a = Attempt::new(&source, 1);
    solve(&mut a, &source);
    engine::execute_for_goal(&mut a.state, &source.goal, "make install").unwrap();
    engine::execute_for_goal(
        &mut a.state,
        &source.goal,
        "source-action {\"kind\":\"check\"}",
    )
    .unwrap();
    let rows = engine::checks(&a.state, &source.goal);
    assert!(
        rows.iter()
            .filter(|r| r.1.starts_with("Program case"))
            .all(|r| r.0)
    );
    assert!(rows.iter().any(|r| !r.0 && r.1.starts_with("Preserve")));
    assert!(!engine::solved(&a.state, &source.goal));
    state(&mut a).reset_assignment(def(&source));
    assert!(!progress::current_solved(&a, &source));
    solve(&mut a, &source);
}
#[test]
fn reset_resume_preserves_both_drafts_favorites_and_an_existing_lab_attempt() {
    let current = catalog();
    let source = current.iter().find(|s| s.id == IDS[2]).unwrap();
    let mut a = Attempt::new(source, 1);
    let d = def(source);
    engine::execute_for_goal(&mut a.state, &source.goal, &d.initial.reference[0].0).unwrap();
    state(&mut a).assignment.input = "make --unfinished".into();
    state(&mut a).practice.input = "tar -tf ".into();
    state(&mut a).view = dlms_learning_lab::script_teaching::View::Example;
    let peer = current
        .iter()
        .find(|s| s.id == "lpic1-permissions")
        .unwrap();
    let mut old = Attempt::new(peer, 1);
    dlms_learning_lab::workspace::prepare(&mut old);
    old.workspace.as_mut().unwrap().command = "existing unfinished command".into();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(source.id.clone());
    p.favorites.insert(peer.id.clone());
    p.attempts.insert(source.id.clone(), a.clone());
    p.attempts.insert(peer.id.clone(), old.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        ".build-artifacts/continuation/resume-20261111/next-source-install/authored-progress",
    );
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&path, &p, &current).unwrap();
    let mut restored = progress::load(&path, &current).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[source.id.as_str()]).unwrap(),
        serde_json::to_value(&a).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&restored.attempts[peer.id.as_str()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let practice = state(restored.attempts.get_mut(&source.id).unwrap())
        .practice
        .clone();
    state(restored.attempts.get_mut(&source.id).unwrap()).reset_assignment(d);
    assert_eq!(
        state(restored.attempts.get_mut(&source.id).unwrap()).practice,
        practice
    );
    assert_eq!(
        serde_json::to_value(&restored.attempts[peer.id.as_str()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
}
#[test]
fn invalid_source_pack_or_unreachable_method_is_rejected_without_generic_success() {
    let source = labs().remove(0);
    let mut bad = source.clone();
    let Goal::SourceInstall { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.reference.clear();
    assert!(work::source(&bad).is_err());
    let mut raw = serde_json::to_value(&source).unwrap();
    raw["goal"]["definition"]["initial"]["facts"]["host_command"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<pack::Scenario>(raw).is_err());
    let mut a = Attempt::new(&source, 1);
    let before = a.state.clone();
    for input in [
        "printf 'Welcome, Alex'",
        "source-action {\"kind\":\"success\"}",
        "/bin/bash",
        "make; echo done",
    ] {
        assert!(engine::execute_for_goal(&mut a.state, &source.goal, input).is_err());
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &source));
    }
}
#[test]
fn all_new_teaching_rows_match_full_objectives_and_use_actual_supported_controls() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    for source in labs() {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == source.id)
            .unwrap();
        assert_eq!(
            row["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" "),
            source.objective
        );
        assert!(row["concepts"].as_array().unwrap().len() >= 6);
        assert!(row["worked"].as_array().unwrap().len() >= 7);
        assert_eq!(row["practice"].as_array().unwrap().len(), 3);
        let text = row.to_string();
        assert!(
            text.contains("Run command"),
            "Each guided activity names its actual execution button"
        );
        assert!(!text.contains("source-action"));
        assert!(source.simplifications.contains("not a general C compiler"));
        assert!(source.simplifications.contains("remains incomplete"));
    }
}
