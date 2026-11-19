//! Integrated attribution project: behavior, source preservation and real progress storage.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "shell-log-owner-triage";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted305().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn cases() -> &'static [Case] {
    let Goal::Script { cases } = &lab().goal else {
        panic!()
    };
    cases
}
fn record(source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases().iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn state(source: &str) -> State {
    State::Script {
        source: source.into(),
        run: Some(record(source)),
    }
}
#[test]
fn log_project_preserves_all304_scenarios_and101_units_and_follows_real_prerequisites() {
    let old = pack::accepted304().unwrap();
    let all = catalog();
    assert_eq!((old.len(), all.len()), (304, 305));
    assert_eq!(
        serde_json::to_value(&all[..304]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = course::accepted304(&old).unwrap();
    let units = course::accepted305(all).unwrap();
    assert_eq!((before.len(), units.len()), (101, 102));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "shell-log-correlation")
        .collect();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-log-correlation")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < at)
    }
    for p in &lab().prerequisites {
        assert!(all.iter().position(|s| &s.id == p).unwrap() < 304)
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 107, 47, 77]
    );
}
#[test]
fn registered_roster_taught_example_and_two_legitimate_implementations_produce_current_outcomes() {
    let s = lab();
    let f: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-log-triage-recipes-v1.json")).unwrap();
    assert_eq!(cases().len(), 12);
    assert_eq!(serde_json::to_value(cases()).unwrap(), f["cases"]);
    assert_eq!(s.solution[0], f["source"].as_str().unwrap());
    let l = s.teaching.as_ref().unwrap();
    l.validate().unwrap();
    assert_eq!(l.example, f["example_source"].as_str().unwrap());
    assert_eq!(serde_json::to_value(&l.fixture).unwrap(), f["example_case"]);
    for source in [&s.solution[0], f["alternate"].as_str().unwrap()] {
        assert!(engine::solved(&state(source), &s.goal));
    }
    let State::Script { source, .. } = &s.initial else {
        panic!()
    };
    let r = record(source);
    assert!(r.results.iter().all(|r| r.error.is_none()));
    assert!(r.results.iter().zip(cases()).any(|(r, c)| c.accepts(r)));
    assert!(!engine::solved(&state(source), &s.goal));
    assert!(s.objective.contains("at most 8 rows"));
    assert!(!s.objective.contains("at most 64 rows"));
    assert!(l.concepts.iter().any(|c| c.text.contains("Bash-specific")));
    assert!(!engine::solved(&s.initial, &s.goal));
}
#[test]
fn guided_experiments_really_change_attribution_and_final_record_counts_then_retry() {
    let l = lab().teaching.as_ref().unwrap();
    let changed = l
        .example
        .replace("owner=\"${owners[$device]}\"", "owner='Unassigned'");
    assert_ne!(changed, l.example);
    let r = shell::run(&changed, &l.fixture);
    assert!(r.error.is_none());
    assert_eq!(
        r.stdout,
        "WARN|shelf|Unassigned|stock low\nERROR|reader|Unassigned|paper jam\nReported: 2\nSkipped: 0\n"
    );
    assert!(!l.fixture.accepts(&r));
    let changed = l.example.replace(" || test -n \"$event\"", "");
    assert_ne!(changed, l.example);
    let r = shell::run(&changed, &l.fixture);
    assert!(r.error.is_none());
    assert_eq!(
        r.stdout,
        "WARN|shelf|Archive desk|stock low\nReported: 1\nSkipped: 0\n"
    );
    assert!(!l.fixture.accepts(&r));
    assert!(l.fixture.accepts(&shell::run(&l.example, &l.fixture)));
}
#[test]
fn overall_failure_retains_correct_cases_and_edited_success_is_never_stale_credit() {
    let s = lab();
    let wrong = s.solution[0].replace("if test \"$skipped\" -gt 0; then exit 1; fi", "");
    let run = record(&wrong);
    let pass: Vec<_> = run
        .results
        .iter()
        .zip(cases())
        .map(|(r, c)| c.accepts(r))
        .collect();
    assert!(pass.iter().any(|p| *p) && pass.iter().any(|p| !*p));
    assert_eq!(run.results[3].stdout, cases()[3].stdout);
    assert_eq!(run.results[3].stderr, cases()[3].stderr);
    assert_ne!(run.results[3].status, cases()[3].status);
    assert!(!engine::solved(
        &State::Script {
            source: wrong,
            run: Some(run)
        },
        &s.goal
    ));
    let mut good = state(&s.solution[0]);
    assert!(engine::solved(&good, &s.goal));
    let State::Script { source, .. } = &mut good else {
        panic!()
    };
    source.push_str("printf 'extra output\\n'\n");
    assert!(!engine::solved(&good, &s.goal));
}
#[test]
fn isolated_example_and_assignment_drafts_survive_safe_resume_and_selected_reset_only() {
    let s = lab();
    let l = s.teaching.as_ref().unwrap();
    let old = &catalog()[0];
    let mut other = Attempt::new(old, 1);
    workspace::prepare(&mut other);
    let preserved = serde_json::to_value(&other).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), other);
    p.favorites.insert(old.id.clone());
    p.favorites.insert(s.id.clone());
    let mut a = Attempt::new(s, 2);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, l, true);
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learning.example_source.push_str("# my practice draft\n");
    learning.example_result = Some(shell::run(&learning.example_source, &l.fixture));
    assert!(l.fixture.accepts(learning.example_result.as_ref().unwrap()));
    assert_eq!(a.state, s.initial);
    assert!(!progress::current_solved(&a, s));
    a.state = state(&s.solution[0]);
    assert!(progress::current_solved(&a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("# assignment draft\n");
    assert!(!progress::current_solved(&a, s));
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut read = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&read).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&read.attempts[&old.id]).unwrap(),
        preserved
    );
    assert!(read.favorites.contains(&s.id));
    let a = read.attempts.get_mut(&s.id).unwrap();
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source
            .ends_with("# my practice draft\n")
    );
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
    assert_eq!(
        serde_json::to_value(&read.attempts[&old.id]).unwrap(),
        preserved
    );
}
