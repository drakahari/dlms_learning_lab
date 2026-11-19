use dlms_learning_lab::{
    backup, debian_source_cache as cache, debian_sources_workspace as w,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const IDS: [&str; 2] = [
    "lpic1-package-source-selection",
    "lpic1-package-source-recovery",
];
fn all() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/debian-packages-v1.json"),
        include_str!("../scenarios/debian-contents-lessons-v1.json"),
        include_str!("../scenarios/debian-sources-lessons-v1.json"),
    ])
    .unwrap()
}
fn lesson(id: &str) -> pack::Scenario {
    all().into_iter().find(|s| s.id == id).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::DebianSources { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &mut State) -> &mut w::Session {
    let State::DebianSources { simulation } = s else {
        panic!()
    };
    simulation
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for action in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    state
}
#[test]
fn both_authored_tasks_have_real_unfinished_starters_and_separate_worked_solutions() {
    let catalog = all();
    assert_eq!(catalog.len(), 7);
    for id in IDS {
        let s = lesson(id);
        let d = definition(&s);
        w::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut example = w::Session::new(d).unwrap();
        let assignment = example.assignment.clone();
        example.practice.edit(&d.example_worked.sources).unwrap();
        assert_eq!(assignment, example.assignment);
        assert_ne!(example.practice.draft, example.practice.state.saved);
        example.practice.save(&d.example).unwrap();
        example.practice.refresh(&d.example).unwrap();
        assert!(example.practice.check(&d.example).iter().all(|r| r.0));
        assert!(!d.solved(&example));
        solve(&s);
    }
}
#[test]
fn selected_targets_do_not_fetch_and_a_name_from_the_wrong_source_cannot_pass() {
    let s = lesson(IDS[0]);
    let d = definition(&s);
    let mut state = s.initial.clone();
    let work = session(&mut state);
    let before = work.assignment.state.cache.clone();
    work.assignment
        .edit(&d.assignment_reference.sources)
        .unwrap();
    work.assignment.save(&d.initial).unwrap();
    assert_eq!(work.assignment.state.cache, before);
    assert!(
        work.assignment
            .state
            .indexes(&d.initial.model)
            .unwrap()
            .iter()
            .any(|r| r.status == cache::IndexStatus::Missing)
    );
    assert!(!work.assignment.check(&d.initial).iter().all(|r| r.0));
    // All required rows can be current while an accidental extra target still fails.
    let mut wrong = d.assignment_reference.sources.replace(
        "Suites: studio\nComponents: main tools\n",
        "Suites: studio studio-updates\nComponents: main tools\n",
    );
    wrong = wrong.split("\n\n").next().unwrap().to_owned();
    work.assignment.edit(&wrong).unwrap();
    work.assignment.save(&d.initial).unwrap();
    let refreshed = work.assignment.refresh(&d.initial).unwrap();
    assert!(
        refreshed
            .iter()
            .any(|r| r.status == cache::RefreshStatus::Unavailable)
    );
    let rows = work.assignment.check(&d.initial);
    assert!(!rows[0].0 && rows[0].1.contains("extra"));
    assert!(
        rows.iter().skip(1).all(|r| r.0),
        "Keep individual correct outcomes correct while overall selection fails"
    );
    assert!(!d.solved(work));
}
#[test]
fn partial_refresh_keeps_main_evidence_updates_tools_and_excludes_retired_rows() {
    let s = lesson(IDS[1]);
    let d = definition(&s);
    let mut state = s.initial.clone();
    let work = session(&mut state);
    work.assignment
        .edit(&d.assignment_reference.sources)
        .unwrap();
    work.assignment.save(&d.initial).unwrap();
    let result = work.assignment.refresh(&d.initial).unwrap();
    assert_eq!(
        result
            .iter()
            .filter(|r| r.status == cache::RefreshStatus::Updated)
            .count(),
        1
    );
    assert_eq!(
        result
            .iter()
            .filter(|r| r.status == cache::RefreshStatus::Unavailable)
            .count(),
        1
    );
    let indexes = work.assignment.state.indexes(&d.initial.model).unwrap();
    assert_eq!(indexes[0].status, cache::IndexStatus::Retained);
    assert_eq!(indexes[1].status, cache::IndexStatus::Current);
    let rows = work
        .assignment
        .state
        .query(&d.initial.model, "campus-helper")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key.suite, "campus");
    assert_eq!(rows[0].row.version.as_str(), "2.0-1");
    assert!(
        work.assignment
            .state
            .cache
            .iter()
            .any(|c| c.key.suite == "retired"),
        "Old cache bytes remain, but are ineligible"
    );
    work.assignment.check(&d.initial);
    assert!(d.solved(work));
}
#[test]
fn equivalent_configuration_passes_edits_invalidate_and_actual_damage_is_regraded() {
    for id in IDS {
        let s = lesson(id);
        let d = definition(&s);
        let mut state = solve(&s);
        let work = session(&mut state);
        let alternate = d
            .assignment_reference
            .sources
            .replace("Components: main tools", "Components: tools main tools")
            .replace("Types:", "tYpEs:");
        work.assignment.edit(&alternate).unwrap();
        assert!(!d.solved(work));
        work.assignment.save(&d.initial).unwrap();
        work.assignment.check(&d.initial);
        assert!(d.solved(work));
        let retired = d
            .initial
            .starter
            .saved
            .split("\n\n")
            .nth(if id.ends_with("recovery") { 1 } else { 0 })
            .unwrap()
            .trim_end();
        let disabled = format!(
            "{}\n{retired}\nEnabled: no\n",
            d.assignment_reference.sources
        );
        work.assignment.edit(&disabled).unwrap();
        work.assignment.save(&d.initial).unwrap();
        work.assignment.check(&d.initial);
        assert!(
            d.solved(work),
            "Keeping an explicitly disabled retired source is a valid alternative"
        );
        work.assignment
            .state
            .cache
            .retain(|c| c.key.component != "tools");
        assert!(
            !d.solved(work),
            "The checked flag cannot preserve false success"
        );
    }
}
#[test]
fn invalid_input_and_unsupported_commands_preserve_saved_metadata_and_explain_retry() {
    let s = lesson(IDS[0]);
    let d = definition(&s);
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    assert!(
        engine::execute(&mut state, "sh -c anything")
            .unwrap_err()
            .contains("native")
    );
    assert!(engine::execute_for_goal(&mut state, &s.goal, "apt-get update").is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    let work = session(&mut state);
    let saved = work.assignment.state.clone();
    work.assignment
        .edit(&(d.assignment_reference.sources.clone() + "\nTrusted: yes\n"))
        .unwrap();
    assert!(
        work.assignment
            .save(&d.initial)
            .unwrap_err()
            .contains("Only Types")
    );
    assert_eq!(work.assignment.state, saved);
    assert!(!d.solved(work));
}
#[test]
fn actual_progress_safe_write_export_restore_and_selected_reset_keep_separate_raw_drafts() {
    let catalog = all();
    let s = lesson(IDS[0]);
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    a.state = solve(&s);
    let d = definition(&s);
    let work = session(&mut a.state);
    work.practice
        .edit("unfinished field without a colon\n")
        .unwrap();
    work.view = dlms_learning_lab::script_teaching::View::Example;
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    p.attempts
        .insert(catalog[0].id.clone(), Attempt::new(&catalog[0], 2));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut forged: Progress = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    session(&mut forged.attempts.get_mut(&s.id).unwrap().state)
        .assignment
        .state
        .cache[0]
        .snapshot = "invented".into();
    assert!(progress::save_with_catalog(&path, &forged, &catalog).is_err());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "Invalid imported state cannot overwrite saved work"
    );

    let restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[&s.id], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &catalog).unwrap();
    let inspected = backup::inspect(
        &export,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let restored = backup::restore(&path, &inspected, &catalog).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        original
    );
    let other = serde_json::to_value(&p.attempts[&catalog[0].id]).unwrap();
    let practice = session(&mut p.attempts.get_mut(&s.id).unwrap().state)
        .practice
        .clone();
    let mut reset = Attempt::new(&s, 8);
    workspace::prepare(&mut reset);
    p.attempts.insert(s.id.clone(), reset);
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
    assert!(p.favorites.contains(&s.id));
    assert_eq!(
        serde_json::to_value(&p.attempts[&catalog[0].id]).unwrap(),
        other
    );
    assert_ne!(
        session(&mut p.attempts.get_mut(&s.id).unwrap().state).practice,
        practice
    );
    assert_eq!(d.initial.id, s.id);
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &catalog).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
#[test]
fn invalid_pack_unreachable_outcome_and_forged_context_are_rejected() {
    let s = lesson(IDS[0]);
    let mut bad = s.clone();
    if let Goal::DebianSources { definition } = &mut bad.goal {
        definition.assignment_reference.refresh = false;
    }
    assert!(w::source(&bad).is_err());
    let mut bad = s.clone();
    bad.id = "foreign-source-lesson".into();
    assert!(w::source(&bad).is_err());
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    session(&mut a.state).assignment.state.cache[0].snapshot = "invented".into();
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
}

#[test]
fn preserve253_registered_labs72_units_and_append_only_the_two_source_tasks() {
    use dlms_learning_lab::course;
    let old = pack::accepted253().unwrap();
    let new = pack::accepted255().unwrap();
    assert_eq!((old.len(), new.len()), (253, 255));
    for s in &old {
        let now = new.iter().find(|n| n.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(now).unwrap(),
            "Original lab {} changed",
            s.id
        );
    }
    let oldunits = course::accepted253(&old).unwrap();
    let units = course::accepted255(&new).unwrap();
    assert_eq!((oldunits.len(), units.len()), (72, 73));
    for u in oldunits {
        let now = units.iter().find(|n| n.id == u.id).unwrap();
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(now).unwrap()
        );
    }
    let unit = units
        .iter()
        .find(|u| u.id == "debian-source-metadata")
        .unwrap();
    assert_eq!(unit.prerequisites, ["debian-package-file-search"]);
    assert_eq!(unit.labs, IDS);
    let labs: Vec<_> = new.iter().filter(|s| s.unit == unit.id).collect();
    assert_eq!(labs.len(), 2);
    assert_eq!(labs[0].prerequisites, ["lpic1-package-file-provider"]);
    assert_eq!(labs[1].prerequisites, [IDS[0]]);
    for (track, count) in [
        ("CCNA", 72),
        ("LPIC-1", 74),
        ("LPIC-2", 43),
        ("Bash/Shell", 66),
    ] {
        assert_eq!(new.iter().filter(|s| s.track == track).count(), count);
    }
}
