//! Authored diagnostic handover: actual search and complete metadata outcomes.
use dlms_learning_lab::{
    command_location_workspace as work, course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic1-command-permission-search";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted288().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn fresh() -> Attempt {
    let s = lab();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::prepare(&mut a, s, true).unwrap();
    a
}
fn run(a: &mut Attempt, c: &str) -> String {
    engine::execute_for_goal(&mut a.state, &lab().goal, c).unwrap()
}
#[test]
fn retained287_scenarios90_units_are_exact_and_new_lesson_follows_both_prerequisites() {
    let old = pack::accepted287().unwrap();
    assert_eq!((old.len(), catalog().len()), (287, 288));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&catalog()[..287]).unwrap()
    );
    let before = course::accepted287(&old).unwrap();
    let after = course::accepted288(catalog()).unwrap();
    assert_eq!((before.len(), after.len()), (90, 91));
    let retained: Vec<_> = after
        .iter()
        .filter(|u| u.id != "linux-command-location-diagnosis")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = after
        .iter()
        .position(|u| u.id == "linux-command-location-diagnosis")
        .unwrap();
    for id in ["linux-command-search", "linux-file-access"] {
        assert!(after.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| catalog().iter().filter(|s| s.track == t).count()),
        [72, 97, 45, 74]
    );
    assert_eq!(
        lab().prerequisites,
        ["lpic1-env-tool-path", "lpic1-traversal"]
    );
}
#[test]
fn taught_method_and_legitimate_alternates_select_actual_paths_and_preserve_neighbor() {
    for command in [
        &lab().solution[0],
        "chmod 0740 /tools/preferred/reportctl",
        "chmod u=rwx /tools/preferred/reportctl",
    ] {
        let mut a = fresh();
        assert!(!progress::current_solved(&a, lab()));
        assert_eq!(
            run(&mut a, "which reportctl"),
            "/tools/fallback/reportctl\n"
        );
        run(&mut a, command);
        assert!(progress::current_solved(&a, lab()), "{command}");
        assert_eq!(
            run(&mut a, "which reportctl"),
            "/tools/preferred/reportctl\n"
        );
        assert_eq!(
            run(&mut a, "type -a -p reportctl"),
            "/tools/preferred/reportctl\n/tools/fallback/reportctl\n"
        );
        assert_eq!(run(&mut a, "which auditctl"), "/tools/fallback/auditctl\n");
        run(&mut a, "chmod u-x /tools/preferred/reportctl");
        assert!(!progress::current_solved(&a, lab()));
    }
}
#[test]
fn excessive_permissions_are_not_a_success_even_when_lookup_looks_right_and_feedback_explains_both_mistakes()
 {
    let mut a = fresh();
    let wrong = engine::checks(&a.state, &lab().goal);
    assert!(
        wrong
            .iter()
            .any(|(ok, m)| !ok && m.contains("currently selects /tools/fallback/reportctl"))
    );
    run(&mut a, "chmod 0755 /tools/preferred/reportctl");
    assert_eq!(
        run(&mut a, "which reportctl"),
        "/tools/preferred/reportctl\n"
    );
    let partial = engine::checks(&a.state, &lab().goal);
    assert!(
        partial
            .iter()
            .any(|(ok, m)| !ok && m.contains("Preserve all original bits"))
    );
    assert!(!progress::current_solved(&a, lab()));
    run(&mut a, "chmod 0740 /tools/preferred/reportctl");
    assert!(progress::current_solved(&a, lab()));
    let before = a.state.clone();
    for c in [
        "chmod 0000 /tools/fallback/reportctl",
        "chmod 0777 /tools/preferred",
        "rm /work/notes.txt",
        "reportctl",
        "/tools/preferred/reportctl",
        "whereis reportctl",
        "type reportctl",
        "which reportctl; anything",
    ] {
        assert!(
            engine::execute_for_goal(&mut a.state, &lab().goal, c).is_err(),
            "{c}"
        );
        assert_eq!(a.state, before, "{c}");
    }
}
#[test]
fn worked_example_and_experiment_repair_are_separate_and_source_free_run_preserves_draft() {
    let mut a = fresh();
    let s = lab();
    let l = s.terminal_teaching.as_ref().unwrap();
    let original = a.state.clone();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    for c in &l.example {
        learn.command = c.clone();
        learn.execute_for(l).unwrap();
    }
    assert!(l.outcomes.iter().all(|r| r.check(&learn.tree).0));
    learn.command = "chmod u-x /gallery/preferred/demo-report".into();
    learn.execute_for(l).unwrap();
    assert!(l.outcomes.iter().any(|r| !r.check(&learn.tree).0));
    learn.command = "which demo-report".into();
    learn.execute_for(l).unwrap();
    assert!(
        learn
            .transcript
            .last()
            .unwrap()
            .ends_with("/gallery/fallback/demo-report\n")
    );
    learn.command = "chmod u+x /gallery/preferred/demo-report".into();
    learn.execute_for(l).unwrap();
    assert!(l.outcomes.iter().all(|r| r.check(&learn.tree).0));
    learn.command = "keep this draft".into();
    let before = serde_json::to_value(&*learn).unwrap();
    learn.execute();
    assert_eq!(serde_json::to_value(&*learn).unwrap(), before);
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn saved_assignment_and_practice_drafts_resume_and_reset_retains_other_work() {
    let s = lab();
    let mut a = fresh();
    run(&mut a, &s.solution[0]);
    a.workspace.as_mut().unwrap().command = "ls -l /tools/preferred/reportctl".into();
    a.workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap()
        .command = "which demo-audit".into();
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic1-permissions")
        .unwrap();
    let other = Attempt::new(old, 2);
    let exact = serde_json::to_value(&other).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.attempts.insert(old.id.clone(), other);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut read = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&read).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&read.attempts[ID], s));
    let a = read.attempts.get_mut(ID).unwrap();
    run(a, "chmod u-x /tools/preferred/reportctl");
    assert!(!progress::current_solved(a, s));
    read.attempts.insert(ID.into(), fresh());
    assert_eq!(
        serde_json::to_value(&read.attempts[&old.id]).unwrap(),
        exact
    );
    assert!(read.favorites.contains(ID));
}
#[test]
fn lesson_teaches_every_new_form_and_assignment_hides_the_worked_repair_by_default() {
    let s = lab();
    let l = s.terminal_teaching.as_ref().unwrap();
    l.validate().unwrap();
    let text = l
        .concepts
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for term in [
        "PATH",
        "which NAME",
        "type -a -p NAME",
        "ls -l PATH",
        "chmod u+x PATH",
        "u-x",
        "owner",
        "execute",
    ] {
        assert!(text.contains(term), "{term}");
    }
    assert!(!l.try_change.contains("check the example"));
    assert!(l.try_change.contains("repeat both searches"));
    assert!(!s.objective.contains("chmod"));
    assert!(!s.objective.contains("0740"));
    assert!(!s.objective.contains("/gallery"));
    let State::Filesystem { tree } = &s.initial else {
        panic!()
    };
    assert_eq!(tree.nodes["/tools/preferred/reportctl"].mode, 0o640);
    assert!(work::source(s).unwrap().is_some());
}
