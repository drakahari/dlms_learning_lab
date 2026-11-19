//! Authored private-index handover, source authority and saved progress; no host utilities.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    file_index_source as source, file_index_workspace as work, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic1-file-index-handover";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted289().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn definition() -> &'static work::Definition {
    source::source(lab()).unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 1);
    workspace::prepare(&mut a);
    source::context(&a, lab()).unwrap();
    a
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::FileIndex { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn run(a: &mut Attempt, input: &str) -> String {
    engine::execute_for_goal(&mut a.state, &lab().goal, input).unwrap()
}
fn save(a: &mut Attempt, text: &str) {
    let s = session(a);
    s.set_config_draft(definition(), work::Scope::Assignment, text.into())
        .unwrap();
    s.save_config(definition(), work::Scope::Assignment)
        .unwrap();
}
fn solve(a: &mut Attempt) {
    for cmd in &lab().solution {
        run(a, cmd);
    }
}
#[test]
fn retained288_scenarios91_units_are_exact_and_new_unit_follows_prerequisites() {
    let old = pack::accepted288().unwrap();
    assert_eq!((old.len(), catalog().len()), (288, 289));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&catalog()[..288]).unwrap()
    );
    let before = course::accepted288(&old).unwrap();
    let now = course::accepted289(catalog()).unwrap();
    assert_eq!((before.len(), now.len()), (91, 92));
    let retained: Vec<_> = now
        .iter()
        .filter(|u| u.id != "linux-indexed-file-discovery")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = now
        .iter()
        .position(|u| u.id == "linux-indexed-file-discovery")
        .unwrap();
    for id in ["linux-file-locations", "linux-file-search"] {
        assert!(now.iter().position(|u| u.id == id).unwrap() < at)
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| catalog().iter().filter(|s| s.track == t).count()),
        [72, 98, 45, 74]
    );
}
#[test]
fn taught_solution_and_equivalent_configuration_and_copy_paths_pass() {
    definition().validate_reachability().unwrap();
    for (config, copy) in [
        (
            "PRUNEPATHS = \"/srv/intake/archive\"\n",
            "cp /srv/intake/current/runbook.txt /work/handover.txt",
        ),
        (
            "# Keep the archive excluded\nPRUNEPATHS=\"/srv/intake/archive /srv/intake/archive\"\n",
            "cp ../srv/intake/current/runbook.txt handover.txt",
        ),
        (
            "PRUNEPATHS = \"/srv/intake/archive /srv/intake/archive/old\"\n",
            "cp /srv/intake/current/runbook.txt /work/handover.txt",
        ),
    ] {
        let mut a = fresh();
        assert!(!progress::current_solved(&a, lab()));
        save(&mut a, config);
        run(&mut a, &lab().solution[1]);
        run(&mut a, copy);
        assert!(progress::current_solved(&a, lab()), "{config}");
    }
}
#[test]
fn stale_config_only_index_only_wrong_handover_and_excessive_scan_fail_specific_checks() {
    let mut a = fresh();
    let out = run(&mut a, "locate -d /indexes/intake.db runbook");
    assert!(out.contains("runbook-old.txt"));
    assert!(!run(&mut a, "find /srv/intake -name 'runbook*.txt'").contains("runbook-old.txt"));
    run(
        &mut a,
        "cp /srv/intake/current/runbook.txt /work/handover.txt",
    );
    assert!(!progress::current_solved(&a, lab()));
    save(&mut a, "PRUNEPATHS=\"/srv/intake/archive\"\n");
    let rows = engine::checks(&a.state, &lab().goal);
    assert!(rows.iter().any(|(ok, m)| !ok && m.contains("stale")));
    run(&mut a, &lab().solution[1]);
    assert!(progress::current_solved(&a, lab()));
    run(
        &mut a,
        "cp /srv/intake/archive/runbook.txt /work/handover.txt",
    );
    assert!(
        engine::checks(&a.state, &lab().goal)
            .iter()
            .any(|(ok, m)| !ok && m.contains("wrong text"))
    );
    save(&mut a, "PRUNEPATHS=\"\"\n");
    run(&mut a, &lab().solution[1]);
    run(
        &mut a,
        "cp /srv/intake/current/runbook.txt /work/handover.txt",
    );
    assert!(
        engine::checks(&a.state, &lab().goal)
            .iter()
            .any(|(ok, m)| !ok && m.contains("exclusion"))
    );
    let mut b = fresh();
    save(&mut b, "PRUNEPATHS=\"/srv/intake/archive\"\n");
    run(&mut b, &lab().solution[1]);
    assert!(
        engine::checks(&b.state, &lab().goal)
            .iter()
            .any(|(ok, m)| !ok && m.contains("missing"))
    );
}
#[test]
fn pending_invalid_edits_and_broken_copy_clear_success_and_failed_actions_are_atomic() {
    let mut a = fresh();
    solve(&mut a);
    assert!(progress::current_solved(&a, lab()));
    session(&mut a)
        .set_config_draft(
            definition(),
            work::Scope::Assignment,
            "PRUNEPATHS=oops".into(),
        )
        .unwrap();
    assert!(!progress::current_solved(&a, lab()));
    let before = a.state.clone();
    assert!(
        session(&mut a)
            .save_config(definition(), work::Scope::Assignment)
            .is_err()
    );
    assert_eq!(a.state, before);
    for cmd in [
        "sh -c id",
        "locate runbook",
        "updatedb",
        "cat /indexes/intake.db",
        "ls /indexes",
        "rm /work/notes.txt",
        "chmod 0777 /srv/intake/current/runbook.txt",
        "find / -name '*'",
        "locate -d /indexes/intake.db $(id)",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &lab().goal, cmd).is_err(),
            "{cmd}"
        );
        assert_eq!(a.state, before, "{cmd}")
    }
}
#[test]
fn separate_practice_and_stale_name_experiment_never_award_assignment_credit() {
    let mut a = fresh();
    let assignment = a.state.clone();
    let s = session(&mut a);
    let d = definition();
    for step in &d.example.reference {
        match step {
            work::Step::SaveConfig { text } => {
                s.set_config_draft(d, work::Scope::Example, text.clone())
                    .unwrap();
                s.save_config(d, work::Scope::Example).unwrap();
            }
            work::Step::Command { input } => {
                s.set_command(d, work::Scope::Example, input.clone())
                    .unwrap();
                s.run(d, work::Scope::Example).unwrap();
            }
        }
    }
    assert!(s.check(d, work::Scope::Example).unwrap().passed());
    s.set_command(
        d,
        work::Scope::Example,
        "rm /gallery/current/guide.txt".into(),
    )
    .unwrap();
    s.run(d, work::Scope::Example).unwrap();
    assert!(!s.checks(d, work::Scope::Example).unwrap().passed());
    s.set_command(
        d,
        work::Scope::Example,
        "locate -d /indexes/demo.db /gallery/current/guide.txt".into(),
    )
    .unwrap();
    assert!(
        s.run(d, work::Scope::Example)
            .unwrap()
            .stdout
            .contains("/gallery/current/guide.txt")
    );
    s.set_command(
        d,
        work::Scope::Example,
        "locate -e -d /indexes/demo.db /gallery/current/guide.txt".into(),
    )
    .unwrap();
    let r = s.run(d, work::Scope::Example).unwrap();
    assert!(r.stdout.is_empty());
    assert_eq!(r.outcome, work::Outcome::NoMatch);
    let State::FileIndex { simulation: old } = &assignment else {
        panic!()
    };
    assert_eq!(s.assignment, old.assignment);
    assert!(!progress::current_solved(&a, lab()));
}
#[test]
fn strict_source_pairing_wrong_family_and_erased_snapshots_never_grant_credit() {
    let mut a = fresh();
    solve(&mut a);
    let before = a.state.clone();
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic1-permissions")
        .unwrap();
    for input in ["help", "show", "cp /a /b"] {
        assert!(engine::execute_for_goal(&mut a.state, &old.goal, input).is_err());
        assert_eq!(a.state, before);
        let mut other = old.initial.clone();
        let exact = other.clone();
        assert!(engine::execute_for_goal(&mut other, &lab().goal, input).is_err());
        assert_eq!(other, exact);
        assert!(engine::execute(&mut a.state, input).is_err());
        assert_eq!(a.state, before);
    }
    session(&mut a).assignment.index = None;
    assert!(source::context(&a, lab()).is_err());
    assert!(!progress::current_solved(&a, lab()));
    assert!(engine::checks(&a.state, &lab().goal).iter().all(|c| !c.0));
    let mut a = fresh();
    session(&mut a).assignment.context.source_id = "foreign-assignment".into();
    assert!(source::context(&a, lab()).is_err());
    let mut a = fresh();
    a.workspace.as_mut().unwrap().command = "foreign draft".into();
    assert!(source::context(&a, lab()).is_err());
    let mut s = lab().clone();
    let Goal::FileIndex { definition } = &mut s.goal else {
        panic!()
    };
    definition.assignment.initial.context.root = "/gallery".into();
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![s]
            })
            .unwrap()
        )
        .is_err()
    );
}
#[test]
fn actual_save_resume_reset_preserve_both_drafts_and_other_attempts() {
    let mut a = fresh();
    solve(&mut a);
    let s = session(&mut a);
    s.set_config_draft(
        definition(),
        work::Scope::Example,
        "PRUNEPATHS=\"/gallery/archive\"\n# separate draft".into(),
    )
    .unwrap();
    s.set_command(
        definition(),
        work::Scope::Example,
        "find /gallery -type f".into(),
    )
    .unwrap();
    s.set_command(
        definition(),
        work::Scope::Assignment,
        "cat /work/handover.txt".into(),
    )
    .unwrap();
    let old = catalog().iter().find(|s| s.id == "ccna-ipv4").unwrap();
    let other = Attempt::new(old, 2);
    let exact = serde_json::to_value(&other).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(ID.into(), a);
    p.attempts.insert(old.id.clone(), other);
    p.favorites.insert(ID.into());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[ID], lab()));
    assert!(progress::save(&path, &loaded).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let example = session(loaded.attempts.get_mut(ID).unwrap())
        .example
        .clone();
    session(loaded.attempts.get_mut(ID).unwrap())
        .reset_scope(definition(), work::Scope::Assignment)
        .unwrap();
    assert_eq!(
        session(loaded.attempts.get_mut(ID).unwrap()).example,
        example
    );
    assert!(!progress::current_solved(&loaded.attempts[ID], lab()));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        exact
    );
    assert!(loaded.favorites.contains(ID));
    let a = loaded.attempts.get_mut(ID).unwrap();
    session(a).example.index = None;
    assert!(progress::save_with_catalog(&path, &loaded, catalog()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
#[test]
fn teaching_supplies_prerequisites_and_assignment_contains_no_worked_commands() {
    let s = lab();
    let d = definition();
    let text = d
        .teaching
        .concepts
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for term in [
        "find",
        "locate",
        "PRUNEPATHS",
        "updatedb -l 0",
        "-e",
        "Save configuration",
        "cat",
        "cp",
    ] {
        assert!(text.contains(term), "{term}")
    }
    assert!(!s.objective.contains("PRUNEPATHS ="));
    assert!(!s.objective.contains("cp /"));
    assert!(!s.objective.contains("/gallery"));
    assert!(!d.teaching.try_steps.is_empty());
    let wire = serde_json::to_value(&s.initial).unwrap();
    let session = &wire["simulation"];
    for k in ["definition", "expected_tree", "reference", "teaching"] {
        assert!(session.get(k).is_none(), "{k}")
    }
}

#[test]
fn portable_backup_restore_rechecks_private_index_context_and_keeps_a_recovery_copy() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let mut a = fresh();
    solve(&mut a);
    let s = session(&mut a);
    s.set_config_draft(
        definition(),
        work::Scope::Example,
        "unsaved practice investigation".into(),
    )
    .unwrap();
    s.set_command(
        definition(),
        work::Scope::Assignment,
        "cat /work/handover.txt".into(),
    )
    .unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let dir = tempfile::tempdir().unwrap();
    let exported = dir.path().join("exported.json");
    backup::export(&exported, &p, catalog()).unwrap();
    let input = std::fs::read(&exported).unwrap();
    assert!(backup::export(&exported, &p, catalog()).is_err());
    assert_eq!(std::fs::read(&exported).unwrap(), input);
    let cancel = AtomicBool::new(false);
    let checked = AtomicUsize::new(0);
    let admitted = backup::inspect(&exported, catalog(), &cancel, &checked).unwrap();
    assert!(progress::current_solved(
        &admitted.progress.attempts[ID],
        lab()
    ));
    assert_eq!(
        serde_json::to_value(&admitted.progress).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let target = dir.path().join("progress.json");
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic1-permissions")
        .unwrap();
    let mut before = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    before.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    progress::save_with_catalog(&target, &before, catalog()).unwrap();
    let original = std::fs::read(&target).unwrap();
    let restored = backup::restore(&target, &admitted, catalog()).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        original
    );
    let loaded = progress::load(&target, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let correct = std::fs::read(&target).unwrap();
    let mut portable: serde_json::Value = serde_json::from_slice(&input).unwrap();
    portable["progress"]["attempts"][ID]["state"]["simulation"]["assignment"]["index"] =
        serde_json::Value::Null;
    let bad = dir.path().join("erased-index.json");
    std::fs::write(&bad, serde_json::to_vec(&portable).unwrap()).unwrap();
    assert!(backup::inspect(&bad, catalog(), &cancel, &checked).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), correct);
    let mut forged = admitted;
    let attempt = forged.progress.attempts.get_mut(ID).unwrap();
    session(attempt).example.context.source_id = "foreign-example".into();
    let files_before = std::fs::read_dir(dir.path()).unwrap().count();
    assert!(backup::restore(&target, &forged, catalog()).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), correct);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), files_before);
}
