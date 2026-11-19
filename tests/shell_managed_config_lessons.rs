//! Authored managed-configuration project; real bounded VM, no host execution.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Record},
    workspace,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted284().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog()
        .iter()
        .find(|s| s.id == "shell-managed-config")
        .unwrap()
}
fn record(source: &str) -> Record {
    let Goal::Script { cases } = &lab().goal else {
        panic!()
    };
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases.iter().map(|c| shell::run(source, c)).collect(),
    }
}
#[test]
fn managed_lesson_keeps_all283_scenarios_and86_units_exact_and_follows_prerequisites() {
    let old = pack::accepted283().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (283, 284));
    assert_eq!(
        serde_json::to_value(&now[..283]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted283(&old).unwrap();
    let units = course::accepted284(now).unwrap();
    assert_eq!((before.len(), units.len()), (86, 87));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "shell-managed-configuration")
        .collect();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-managed-configuration")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < at)
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 93, 45, 74]
    );
    for p in &lab().prerequisites {
        assert!(now.iter().position(|s| &s.id == p).unwrap() < 283)
    }
}
#[test]
fn managed_lesson_copies_reviewed_cases_primary_alternate_and_separate_example_without_credit() {
    let s = lab();
    let f: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-managed-config-reference.json")).unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    assert_eq!(serde_json::to_value(cases).unwrap(), f["cases"]);
    assert_eq!(s.solution[0], f["source"].as_str().unwrap());
    let l = s.teaching.as_ref().unwrap();
    l.validate().unwrap();
    assert_eq!(l.example, f["example_source"].as_str().unwrap());
    assert_eq!(serde_json::to_value(&l.fixture).unwrap(), f["example_case"]);
    for source in [s.solution[0].as_str(), f["alternate"].as_str().unwrap()] {
        let state = State::Script {
            source: source.into(),
            run: Some(record(source)),
        };
        assert!(engine::solved(&state, &s.goal));
    }
    let State::Script { source, .. } = &s.initial else {
        panic!()
    };
    let starter = record(source);
    assert!(starter.results.iter().all(|r| r.error.is_none()));
    assert!(!engine::solved(
        &State::Script {
            source: source.clone(),
            run: Some(starter)
        },
        &s.goal
    ));
    assert!(!engine::solved(&s.initial, &s.goal));
}
#[test]
fn managed_lesson_each_taught_experiment_changes_the_promised_behavior() {
    let l = lab().teaching.as_ref().unwrap();
    let mut changed = l.example.clone();
    let at = changed.rfind("configure_report compact columns").unwrap();
    changed.replace_range(
        at..at + "configure_report compact columns".len(),
        "configure_report expanded columns",
    );
    let r = shell::run(&changed, &l.fixture);
    assert!(r.error.is_none());
    assert_eq!(r.stdout, "Updated\nUpdated\n");
    assert!(r.files["/work/report.conf"].ends_with("style=expanded\nlayout=columns\n"));
    assert!(!l.fixture.accepts(&r));
    let r = shell::run(
        &l.example.replace("while IFS= read -r", "while read -r"),
        &l.fixture,
    );
    assert!(r.error.is_none());
    assert_eq!(r.stdout, "Updated\nUnchanged\n");
    assert!(!r.files["/work/report.conf"].contains("  retain this spacing  "));
    assert!(!l.fixture.accepts(&r));
}
#[test]
fn managed_lesson_practice_and_assignment_drafts_survive_save_resume_reset_is_selected_only() {
    let s = lab();
    let l = s.teaching.as_ref().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    let old = &catalog()[0];
    let mut other = Attempt::new(old, 2);
    workspace::prepare(&mut other);
    p.attempts.insert(old.id.clone(), other);
    p.favorites.insert(old.id.clone());
    let preserved = serde_json::to_value(&p).unwrap();
    let mut a = Attempt::new(s, 3);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, l, true);
    let initial = a.state.clone();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learn
        .example_source
        .push_str("# my separate practice note\n");
    learn.example_result = Some(shell::run(&learn.example_source, &l.fixture));
    assert!(l.fixture.accepts(learn.example_result.as_ref().unwrap()));
    assert_eq!(a.state, initial);
    assert!(!progress::current_solved(&a, s));
    a.state = State::Script {
        source: s.solution[0].clone(),
        run: Some(record(&s.solution[0])),
    };
    assert!(progress::current_solved(&a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("printf extra\n");
    assert!(!progress::current_solved(&a, s));
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&q.attempts[&old.id]).unwrap(),
        preserved["attempts"][&old.id]
    );
    assert!(q.favorites.contains(&old.id));
    let a = q.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, s));
    *a = Attempt::new(s, 4);
    workspace::prepare(a);
    script_teaching::prepare(a, l, true);
    assert_eq!(a.state, s.initial);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        l.example
    );
    assert_eq!(q.attempts[&old.id].state, p.attempts[&old.id].state);
}
