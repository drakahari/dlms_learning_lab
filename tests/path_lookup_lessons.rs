//! Actual authored inherited search; no selected artifact is executed.
use dlms_learning_lab::{
    course,
    engine::State,
    environment_workspace as e, pack,
    progress::{self, Attempt, Progress},
};
use std::sync::OnceLock;
const ID: &str = "lpic1-env-tool-path";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted286().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn session() -> e::Session {
    let State::Environment { simulation } = &lab().initial else {
        panic!()
    };
    (**simulation).clone()
}
fn run(d: &mut e::Draft, t: &e::Task, s: &str) -> Vec<(bool, String)> {
    d.edit(s.into()).unwrap();
    assert!(d.store(e::run(t, s).unwrap(), t));
    e::checks(t, d)
}
#[test]
fn path_lesson_preserves_all285_scenarios88_units_and_follows_its_prerequisites() {
    let old = pack::accepted285().unwrap();
    assert_eq!((old.len(), catalog().len()), (285, 286));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&catalog()[..285]).unwrap()
    );
    let before = course::accepted285(&old).unwrap();
    let units = course::accepted286(catalog()).unwrap();
    assert_eq!((before.len(), units.len()), (88, 89));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-command-search")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-command-search")
        .unwrap();
    for id in ["linux-shell-environment", "linux-file-locations"] {
        assert!(units.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| catalog().iter().filter(|s| s.track == t).count()),
        [72, 95, 45, 74]
    );
}
#[test]
fn taught_example_and_assignment_alternatives_pass_real_lookups_not_exact_path_strings() {
    let mut x = session();
    assert!(!x.solved());
    let original = x.assignment.clone();
    let worked = x.example.source.clone();
    assert!(
        run(&mut x.example, &x.definition.example, &worked)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(x.assignment, original);
    assert!(!x.solved());
    let t = x.definition.assignment.clone();
    assert_eq!(t.cases.len(), 6);
    assert_eq!(x.definition.example.cases.len(), 2);
    for source in [
        lab().solution[0].clone(),
        "PATH=\"$BASE_PATH:$TOOL_DIR\"\nexport PATH\n".into(),
        "scratch=$TOOL_DIR\nPATH=\"$scratch:$BASE_PATH:$scratch:$LEGACY_DIR\"\nexport PATH\n"
            .into(),
    ] {
        let checks = run(&mut x.assignment, &t, &source);
        assert!(checks.iter().all(|r| r.0), "{source}: {checks:?}");
        assert!(x.solved());
        for (c, r) in t
            .cases
            .iter()
            .zip(&x.assignment.result.as_ref().unwrap().results)
        {
            let e::path_lookup::Run::Observed { observation } = r.path_lookup.as_ref().unwrap()
            else {
                panic!()
            };
            assert_eq!(observation.path, r.bindings.as_ref().unwrap().child["PATH"]);
            assert!(c.child.accepts(r.child.as_ref().unwrap()));
        }
    }
}
#[test]
fn every_practice_experiment_explains_real_wrong_provider_or_permission_skip() {
    let mut x = session();
    let t = x.definition.example.clone();
    let worked = x.example.source.clone();
    let wrong = worked.replace("$TOOL_DIR", "$LEGACY_DIR");
    let checks = run(&mut x.example, &t, &wrong);
    assert!(
        checks
            .iter()
            .all(|r| !r.0 && r.1.contains("not an approved provider")),
        "{checks:?}"
    );
    let alternate = worked.replace("$TOOL_DIR:$BASE_PATH", "$BASE_PATH:$TOOL_DIR");
    assert!(run(&mut x.example, &t, &alternate).iter().all(|r| r.0));
    for r in &x.example.result.as_ref().unwrap().results {
        let e::path_lookup::Run::Observed { observation } = r.path_lookup.as_ref().unwrap() else {
            panic!()
        };
        assert_eq!(
            observation.reports[0].candidates[0].state,
            dlms_learning_lab::filesystem::path_lookup::CandidateState::NotExecutable
        );
    }
    assert!(!x.solved());
}
#[test]
fn wrong_partial_hardcoded_unsafe_and_other_setting_damage_do_not_pass() {
    let mut x = session();
    let t = x.definition.assignment.clone();
    let starter = x.assignment.source.clone();
    assert!(
        run(&mut x.assignment, &t, &starter)
            .iter()
            .all(|r| !r.0 && r.1.contains("not an approved provider"))
    );
    let hardcoded = "export PATH=\"/opt/reports/current/bin:$BASE_PATH\"\n";
    let r = run(&mut x.assignment, &t, hardcoded);
    assert_eq!(r.iter().filter(|r| r.0).count(), 1);
    let private = "PATH=\"$TOOL_DIR:$BASE_PATH\"\n";
    let r = run(&mut x.assignment, &t, private);
    assert_eq!(
        r.iter().filter(|r| r.0).count(),
        3,
        "incoming exported values retain their export attribute, new ones do not: {r:?}"
    );
    for s in [
        format!("{}export TOKEN=changed\n", lab().solution[0]),
        format!("{}export EXTRA=unrequested\n", lab().solution[0]),
        format!("{}printf changed > keep.txt\n", lab().solution[0]),
        format!(
            "{}printf 'Worker token: %s\\n' \"$TOKEN\"\n",
            lab().solution[0]
        ),
        "export PATH=\"$TOOL_DIR:$BASE_PATH:/home/alex/bin\"\n".into(),
        "export PATH=\":$TOOL_DIR:$BASE_PATH\"\n".into(),
    ] {
        let r = run(&mut x.assignment, &t, &s);
        assert!(r.iter().any(|r| !r.0), "false pass {s}");
    }
    for (s, message) in [
        ("if true; then\n", "Setup could not run"),
        ("export -q PATH\n", "Unsupported"),
        ("export PATH=\":$BASE_PATH\"\n", "Unsupported"),
        ("export PATH=\"$BASE_PATH\"\n", "execute permission"),
    ] {
        let r = run(&mut x.assignment, &t, s);
        assert!(
            r.iter().all(|r| !r.0 && r.1.contains(message)),
            "{s}: {r:?}"
        );
    }
}
#[test]
fn completion_invalidates_on_edit_and_disk_resume_reverifies_without_changing_other_labs() {
    let s = lab();
    let other = catalog()
        .iter()
        .find(|s| s.id == "lpic1-env-parent-note")
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(other.id.clone(), Attempt::new(other, 1));
    p.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    let untouched = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let a = p.attempts.get_mut(ID).unwrap();
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    x.example
        .edit("# Keep my independent gallery draft\nexport PATH\n".into())
        .unwrap();
    assert!(
        run(&mut x.assignment, &x.definition.assignment, &s.solution[0])
            .iter()
            .all(|r| r.0)
    );
    assert!(progress::current_solved(a, s));
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut loaded = progress::decode(&std::fs::read(&path).unwrap(), catalog()).unwrap();
    assert!(!progress::current_solved(&loaded.attempts[ID], s));
    let a = loaded.attempts.get_mut(ID).unwrap();
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    assert!(x.example.source.contains("independent gallery"));
    assert!(x.assignment.needs_reverification(&x.definition.assignment));
    let source = x.assignment.source.clone();
    assert!(
        run(&mut x.assignment, &x.definition.assignment, &source)
            .iter()
            .all(|r| r.0)
    );
    assert!(progress::current_solved(a, s));
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    x.assignment
        .edit("export PATH=$BASE_PATH\n".into())
        .unwrap();
    assert!(x.assignment.result.is_none());
    assert!(!progress::current_solved(a, s));
    loaded.attempts.insert(ID.into(), Attempt::new(s, 4));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
        untouched
    );
    assert_eq!(loaded.favorites, p.favorites);
}
#[test]
fn source_erasure_forged_observations_and_swapped_practice_are_rejected_before_save() {
    let s = lab();
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    run(&mut x.assignment, &x.definition.assignment, &s.solution[0]);
    p.attempts.insert(ID.into(), a);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let old = std::fs::read(&path).unwrap();
    for n in 0..4 {
        let mut bad: Progress = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        let a = bad.attempts.get_mut(ID).unwrap();
        let State::Environment { simulation: x } = &mut a.state else {
            panic!()
        };
        match n {
            0 => x.definition.assignment.path_lookup = None,
            1 => x.assignment.result.as_mut().unwrap().results[0].path_lookup = None,
            2 => x.definition.example.path_lookup = x.definition.assignment.path_lookup.clone(),
            _ => {
                x.definition
                    .assignment
                    .path_lookup
                    .as_mut()
                    .unwrap()
                    .profile
                    .tree
                    .nodes
                    .get_mut("/usr/bin/reportgen")
                    .unwrap()
                    .mode = 0o555
            }
        };
        assert!(e::context(a, s).is_err());
        assert!(progress::save_with_catalog(&path, &bad, catalog()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), old);
    }
}
#[test]
fn independent_starters_hide_solutions_and_all_three_screens_have_reviewed_content() {
    let s = lab();
    let x = session();
    assert_ne!(
        x.definition.assignment.path_lookup,
        x.definition.example.path_lookup
    );
    assert!(!x.assignment.source.contains("$TOOL_DIR"));
    assert!(!s.objective.contains("export PATH=\""));
    assert!(s.teaching.as_ref().unwrap().concepts.len() >= 5);
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic1.json")).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == ID)
        .unwrap();
    assert_eq!(row["requirements"].as_array().unwrap().len(), 4);
    assert_eq!(row["practice"].as_array().unwrap().len(), 3);
    assert_eq!(row["concepts"].as_array().unwrap().len(), 5);
    let mut job = e::Job::new(&x.definition.assignment, "while true; do :; done\n").unwrap();
    for _ in 0..20 {
        job.advance(16);
    }
    assert!(!job.done());
    job.stop();
    assert!(job.record().is_none());
}
