//! Authored dependency/search tasks, strict binding and actual progress boundaries.
//! Learner activity uses only typed metadata; no host loader commands run.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shared_libraries_workspace as work, workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        pack::accepted281().expect("authored library catalog including reached solutions")
    })
}
fn labs() -> Vec<&'static pack::Scenario> {
    catalog()
        .iter()
        .filter(|s| s.unit == "linux-shared-library-resolution")
        .collect()
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::SharedLibraries { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &mut State) -> &mut work::Session {
    let State::SharedLibraries { simulation } = s else {
        panic!()
    };
    simulation
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for line in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
    }
    state
}
fn apply(s: &mut work::Session, d: &work::Definition, p: &str, value: &str) {
    s.edit_launch(d, work::Scope::Assignment, p, value.into())
        .unwrap();
    s.save_launch(d, work::Scope::Assignment, p).unwrap();
}
#[test]
fn current_catalog_preserves_every_old_lab_unit_and_relative_order() {
    let old = pack::accepted279().unwrap();
    let new = catalog();
    assert_eq!(new.len(), 281);
    assert_eq!(labs().len(), 2);
    assert_eq!(
        new.iter()
            .take(279)
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>(),
        old.iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>()
    );
    let old_units = course::accepted279(&old).unwrap();
    let units = course::accepted281(new).unwrap();
    assert_eq!(units.len(), 85);
    let preserved = units
        .iter()
        .filter(|u| u.id != "linux-shared-library-resolution")
        .map(|u| serde_json::to_value(u).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        preserved,
        old_units
            .iter()
            .map(|u| serde_json::to_value(u).unwrap())
            .collect::<Vec<_>>()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-shared-library-resolution")
        .unwrap();
    for id in ["linux-link-handover", "linux-shell-environment"] {
        assert!(units.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| new.iter().filter(|s| s.track == t).count()),
        [72, 91, 45, 73]
    );
    assert!(course::accepted281(&old).is_err());
    assert!(course::accepted281(&[]).is_err());
    let mut changed = new.clone();
    changed.last_mut().unwrap().unit = "wrong-unit".into();
    assert!(course::accepted281(&changed).is_err());
}
#[test]
fn actual_rust_constructor_matches_source_wire_and_both_taught_paths_work() {
    for s in labs() {
        let d = definition(s);
        d.validate().unwrap();
        d.verify_worked().unwrap();
        work::source(s).unwrap();
        assert_eq!(
            serde_json::to_value(&s.initial).unwrap()["simulation"],
            serde_json::to_value(work::Session::new(d).unwrap()).unwrap()
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(engine::solved(&solve(s), &s.goal));
        let mut fresh = work::Session::new(d).unwrap();
        let original = fresh.assignment.clone();
        for step in &d.lesson.worked {
            match &step.action {
                work::WorkedAction::Configuration { text } => {
                    fresh
                        .edit_configuration(d, work::Scope::Example, text.clone())
                        .unwrap();
                    fresh.save_configuration(d, work::Scope::Example).unwrap();
                }
                work::WorkedAction::LaunchRecipe {
                    program,
                    environment,
                } => {
                    fresh
                        .edit_launch(
                            d,
                            work::Scope::Example,
                            program,
                            environment.clone().unwrap_or_default(),
                        )
                        .unwrap();
                    fresh.save_launch(d, work::Scope::Example, program).unwrap();
                }
                work::WorkedAction::RebuildCache => {
                    fresh.rebuild_cache(d, work::Scope::Example).unwrap()
                }
                work::WorkedAction::Inspect { command } => {
                    assert_eq!(
                        fresh.run(d, work::Scope::Example, command).unwrap().outcome,
                        work::Outcome::Supported
                    );
                }
            }
        }
        assert!(fresh.check(d, work::Scope::Example).unwrap().passed());
        assert_eq!(fresh.assignment, original);
        assert!(!fresh.current_success(d));
    }
}
#[test]
fn supported_directory_order_comments_and_redundant_paths_are_valid_alternatives() {
    let l = labs();
    let s = l[0];
    let d = definition(s);
    for paths in [
        "/opt/site/lib:/opt/common/lib",
        "/opt/common/lib:/opt/site/lib",
        "/opt/site/lib:/opt/common/lib:/opt/site/lib",
    ] {
        let mut state = s.initial.clone();
        let w = session(&mut state);
        apply(w, d, "/usr/bin/report", paths);
        apply(w, d, "/usr/bin/audit", "/opt/common/lib");
        assert!(w.check(d, work::Scope::Assignment).unwrap().passed());
        assert!(engine::solved(&state, &s.goal));
    }
    let s = l[1];
    let d = definition(s);
    for text in [
        "/opt/site/lib\n/opt/common/lib\n/opt/keep/lib\n",
        "# retain viewer\n/opt/keep/lib\n\n/opt/common/lib\n/opt/site/lib\n/opt/site/lib\n",
    ] {
        let mut state = s.initial.clone();
        let w = session(&mut state);
        w.edit_configuration(d, work::Scope::Assignment, text.into())
            .unwrap();
        w.save_configuration(d, work::Scope::Assignment).unwrap();
        w.rebuild_cache(d, work::Scope::Assignment).unwrap();
        for p in ["/usr/bin/report", "/usr/bin/audit"] {
            apply(w, d, p, "");
        }
        assert!(w.check(d, work::Scope::Assignment).unwrap().passed());
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn one_off_success_missing_transitive_and_partial_repairs_get_specific_feedback() {
    let s = labs()[0];
    let d = definition(s);
    let mut w = work::Session::new(d).unwrap();
    let row = w
        .run(
            d,
            work::Scope::Assignment,
            "env LD_LIBRARY_PATH=/opt/site/lib:/opt/common/lib ldd /usr/bin/report",
        )
        .unwrap();
    assert_eq!(row.outcome, work::Outcome::Supported);
    assert!(!row.output.contains("not found"));
    assert!(!w.check(d, work::Scope::Assignment).unwrap().passed());
    apply(&mut w, d, "/usr/bin/report", "/opt/site/lib");
    let r = w.check(d, work::Scope::Assignment).unwrap();
    assert!(!r.passed());
    assert!(
        r.checks
            .iter()
            .any(|c| !c.passed && c.feedback.contains("libformat.so.1"))
    );
    apply(
        &mut w,
        d,
        "/usr/bin/report",
        "/opt/site/lib:/opt/common/lib",
    );
    let r = w.check(d, work::Scope::Assignment).unwrap();
    assert!(
        r.checks
            .iter()
            .any(|c| c.label.contains("report") && c.passed)
    );
    assert!(
        r.checks
            .iter()
            .any(|c| c.label.contains("audit") && !c.passed)
    );
    apply(&mut w, d, "/usr/bin/audit", "/opt/common/lib");
    assert!(w.check(d, work::Scope::Assignment).unwrap().passed());
}
#[test]
fn editing_or_breaking_a_pass_clears_current_credit_and_requires_fresh_check() {
    for s in labs() {
        let d = definition(s);
        let mut state = solve(s);
        let w = session(&mut state);
        assert!(w.current_success(d));
        w.edit_launch(
            d,
            work::Scope::Assignment,
            "/usr/bin/report",
            "/opt/old/lib".into(),
        )
        .unwrap();
        assert!(!w.current_success(d));
        assert!(w.assignment.pending());
        w.save_launch(d, work::Scope::Assignment, "/usr/bin/report")
            .unwrap();
        assert!(!w.check(d, work::Scope::Assignment).unwrap().passed());
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn cache_handover_requires_save_rebuild_clean_settings_and_protected_viewer() {
    let s = labs()[1];
    let d = definition(s);
    let mut w = work::Session::new(d).unwrap();
    assert!(
        w.assignment.recipes.iter().any(|r| r.environment.is_some()),
        "A real temporary workaround must need removal"
    );
    w.edit_configuration(
        d,
        work::Scope::Assignment,
        "/opt/site/lib\n/opt/common/lib\n".into(),
    )
    .unwrap();
    w.save_configuration(d, work::Scope::Assignment).unwrap();
    assert!(!w.check(d, work::Scope::Assignment).unwrap().passed());
    w.rebuild_cache(d, work::Scope::Assignment).unwrap();
    for p in ["/usr/bin/report", "/usr/bin/audit"] {
        apply(&mut w, d, p, "");
    }
    let report = w.check(d, work::Scope::Assignment).unwrap();
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.label.contains("viewer") && !c.passed)
    );
    assert!(!report.passed());
    w.edit_configuration(
        d,
        work::Scope::Assignment,
        "/opt/keep/lib\n/opt/site/lib\n/opt/common/lib\n".into(),
    )
    .unwrap();
    w.save_configuration(d, work::Scope::Assignment).unwrap();
    w.rebuild_cache(d, work::Scope::Assignment).unwrap();
    assert!(w.check(d, work::Scope::Assignment).unwrap().passed());
}
#[test]
fn malformed_unsupported_or_escaping_commands_never_change_the_virtual_or_host_environment() {
    let s = labs()[0];
    let d = definition(s);
    let mut w = work::Session::new(d).unwrap();
    let original = w.assignment.state.clone();
    for cmd in [
        "sh -c id",
        "ldd /../../etc/passwd",
        "ldconfig; id",
        "env LD_LIBRARY_PATH=$(id) ldd /usr/bin/report",
        "ldconfig -X",
        "ldd /usr/bin/report > /home/personal",
    ] {
        let row = w.run(d, work::Scope::Assignment, cmd).unwrap();
        assert_ne!(row.outcome, work::Outcome::Supported, "{cmd}");
        assert_eq!(w.assignment.state, original);
        assert!(!w.current_success(d));
    }
    assert!(
        w.run(
            d,
            work::Scope::Assignment,
            &"x".repeat(work::MAX_COMMAND + 1)
        )
        .is_err()
    );
}
#[test]
fn source_plans_initial_settings_and_foreign_metadata_cannot_be_rebound_or_forged() {
    let s = labs()[0];
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::context(&a, s).unwrap();
    assert!(engine::execute(&mut a.state, "ldconfig").is_err());
    let mut other = s.clone();
    let Goal::SharedLibraries { definition } = &mut other.goal else {
        unreachable!()
    };
    definition.plan.preserve_configuration_and_cache = false;
    assert!(!progress::current_solved(&a, &other));
    assert!(work::context(&a, &other).is_err());
    session(&mut a.state)
        .assignment_plan
        .preserve_configuration_and_cache = false;
    assert!(work::context(&a, s).is_err());
    a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().command = "foreign injected command".into();
    assert!(work::context(&a, s).is_err());
    let mut raw = serde_json::to_value(s).unwrap();
    raw["initial"]["simulation"]["assignment"]["checked"] = serde_json::json!(true);
    let changed: pack::Scenario = serde_json::from_value(raw).unwrap();
    assert!(work::source(&changed).is_err());
}
#[test]
fn partial_drafts_applied_work_and_favorites_resume_without_touching_other_attempts() {
    let c = catalog();
    let l = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 12,
        ..Default::default()
    };
    let old = &c[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 2));
    let untouched = serde_json::to_value(&p.attempts[&old.id]).unwrap();
    let s = l[0];
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    let d = definition(s);
    let w = session(&mut a.state);
    apply(w, d, "/usr/bin/report", "/opt/site/lib:/opt/common/lib");
    w.edit_launch(
        d,
        work::Scope::Assignment,
        "/usr/bin/audit",
        "/opt/common/lib".into(),
    )
    .unwrap();
    w.edit_configuration(
        d,
        work::Scope::Example,
        "# unfinished separate example\n".into(),
    )
    .unwrap();
    w.set_view(d, work::View::Example).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let expected = serde_json::to_value(&p).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, c).unwrap();
    let mut restored = progress::load(&path, c).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), expected);
    assert!(!progress::current_solved(&restored.attempts[&s.id], s));
    let a = restored.attempts.get_mut(&s.id).unwrap();
    let w = session(&mut a.state);
    w.save_launch(d, work::Scope::Assignment, "/usr/bin/audit")
        .unwrap();
    assert!(w.check(d, work::Scope::Assignment).unwrap().passed());
    let example = w.example.clone();
    w.reset(d, work::Scope::Assignment).unwrap();
    assert_eq!(w.example, example);
    assert!(!w.current_success(d));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old.id]).unwrap(),
        untouched
    );
    assert!(restored.favorites.contains(&s.id));
    progress::save_with_catalog(&path, &restored, c).unwrap();
    assert!(progress::load(&path, c).is_ok());
}

#[test]
fn invalid_source_packs_refuse_solved_starters_wrong_worked_actions_and_unknown_fields() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/shared-library-lessons-v1.json")).unwrap();
    let mut changed = raw.clone();
    changed["scenarios"][0]["initial"]["simulation"]["assignment"]["checked"] =
        serde_json::json!(true);
    assert!(
        pack::load(&serde_json::to_string(&changed).unwrap())
            .unwrap_err()
            .contains("fresh task")
    );
    let mut changed = raw.clone();
    changed["scenarios"][0]["goal"]["definition"]["lesson"]["worked"][0]["action"]["command"] =
        serde_json::json!("sh -c id");
    assert!(pack::load(&serde_json::to_string(&changed).unwrap()).is_err());
    let mut changed = raw;
    changed["scenarios"][0]["initial"]["simulation"]["assignment"]["invented_result"] =
        serde_json::json!("passed");
    assert!(
        pack::load(&serde_json::to_string(&changed).unwrap())
            .unwrap_err()
            .contains("unknown field")
    );
}
#[test]
fn portable_backup_recomputes_current_credit_and_rejects_forgery_before_replacing_a_profile() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let all = catalog();
    let s = labs()[1];
    let mut p = Progress {
        schema_version: 1,
        clock: 8,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 8);
    a.state = solve(s);
    workspace::prepare(&mut a);
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let tmp = tempfile::tempdir().unwrap();
    let export = tmp.path().join("learning-backup.json");
    backup::export(&export, &p, all).unwrap();
    let mut candidate =
        backup::inspect(&export, all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert!(progress::current_solved(
        &candidate.progress.attempts[&s.id],
        s
    ));
    let dest = tmp.path().join("progress.json");
    let mut original = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    original
        .attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 2));
    progress::save_with_catalog(&dest, &original, all).unwrap();
    let before = std::fs::read(&dest).unwrap();
    let a = candidate.progress.attempts.get_mut(&s.id).unwrap();
    session(&mut a.state).assignment_plan.require_clean_recipes = false;
    assert!(backup::restore(&dest, &candidate, all).is_err());
    assert_eq!(std::fs::read(&dest).unwrap(), before);
    let candidate =
        backup::inspect(&export, all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let result = backup::restore(&dest, &candidate, all).unwrap();
    assert_eq!(std::fs::read(result.safety_copy.unwrap()).unwrap(), before);
    let loaded = progress::load(&dest, all).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    assert!(loaded.favorites.contains(&s.id));
}
