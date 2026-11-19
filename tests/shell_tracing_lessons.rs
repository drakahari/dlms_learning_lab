//! Authored independent repair tasks; all code runs only in the bounded VM.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::collections::BTreeMap;
fn labs() -> Vec<Scenario> {
    // The standalone activity pack declares the real foundation prerequisite;
    // include it rather than weakening the loader's prerequisite validation.
    pack::load_catalog(&[
        include_str!("../scenarios/shell-unit-v1.json"),
        include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "shell-tracing-diagnosis")
    .collect()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn all_cases(s: &Scenario) -> Vec<Case> {
    let extra: BTreeMap<String, Vec<Case>> = serde_json::from_str(include_str!(
        "fixtures/shell-tracing-lesson-boundaries.json"
    ))
    .unwrap();
    let mut out = cases(s).to_vec();
    out.extend(extra[&s.id].clone());
    out
}
fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn flags(s: &Scenario, source: &str) -> Vec<bool> {
    cases(s)
        .iter()
        .zip(record(s, source).results)
        .map(|(c, r)| c.accepts(&r))
        .collect()
}
fn alternate(s: &Scenario) -> String {
    let source = &s.solution[0];
    if s.id.ends_with("delivery") {
        let at = source.find("if test \"$mode\" = send;").unwrap();
        format!(
            "{}{}",
            &source[..at],
            r#"case "$mode" in
 send) printf 'Ready: %s\n' "$place"; exit 0 ;;
 hold) printf 'Queued: %s\n' "$place"; exit 3 ;;
 *) printf 'Unsupported mode: %s\n' "$mode" >&2; exit 2 ;;
esac
"#
        )
    } else {
        let at = source.find("if test -f \"$first\";").unwrap();
        format!(
            "{}{}",
            &source[..at],
            r#"chosen=$first
if test -f "$first"; then :; else
 if test "$mode" = strict; then printf 'No usable snapshot\n' >&2; exit 2; fi
 chosen=$spare
 if test -f "$chosen"; then :; else printf 'No usable snapshot\n' >&2; exit 2; fi
fi
printf 'Source: %s\n' "$chosen"
cat < "$chosen"
"#
        )
    }
}
#[test]
fn both_references_supported_alternatives_and_separate_examples_match_all31_cases() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let mut total = 0;
    for s in labs() {
        assert_eq!(cases(&s).len(), 12);
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        let initial = flags(&s, source);
        assert!(initial.iter().any(|p| !*p));
        assert!(
            initial.iter().any(|p| *p),
            "Intentional starter retains useful working guards"
        );
        for text in [&s.solution[0], &alternate(&s)] {
            for c in all_cases(&s) {
                let r = shell::run(text, &c);
                assert!(
                    c.accepts(&r),
                    "{} / {}\nactual{r:?}\nexpected{c:?}",
                    s.id,
                    c.name
                );
            }
        }
        total += all_cases(&s).len();
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
        assert!(
            !s.solution[0].contains("elif")
                && !s.solution[0].contains("&&")
                && !s.solution[0].contains("||")
        );
    }
    assert_eq!(total, 31);
}
#[test]
fn unquoted_swapped_status_and_hardcoded_delivery_repairs_cannot_pass() {
    let s = labs()
        .into_iter()
        .find(|s| s.id.ends_with("delivery"))
        .unwrap();
    for wrong in [
        s.solution[0].replace("\"$place\"\n", "$place\n"),
        s.solution[0].replace("exit 3", "exit 0"),
        "printf 'Ready: North Annex\\n'\nexit 0".into(),
    ] {
        assert_ne!(wrong, s.solution[0]);
        let result = flags(&s, &wrong);
        assert!(result.iter().any(|p| !*p));
        assert!(result.iter().any(|p| *p));
        let state = State::Script {
            source: wrong.clone(),
            run: Some(record(&s, &wrong)),
        };
        assert!(!dlms_learning_lab::engine::solved(&state, &s.goal));
    }
}
#[test]
fn snapshot_policy_empty_files_and_literal_dash_have_distinct_negative_checks() {
    let s = labs()
        .into_iter()
        .find(|s| s.id.ends_with("fallback"))
        .unwrap();
    let operand = s.solution[0].replace("cat < \"$chosen\"", "cat -- \"$chosen\"");
    let wrong = flags(&s, &operand);
    assert_eq!(wrong.iter().filter(|p| !**p).count(), 1);
    assert!(!wrong[8]);
    let permissive = s.solution[0].replace(
        "  if test \"$mode\" = strict; then\n    printf 'No usable snapshot\\n' >&2\n    exit 2\n  fi\n",
        "",
    );
    assert_ne!(permissive, s.solution[0]);
    let wrong = flags(&s, &permissive);
    assert!(!wrong[2]);
    assert!(wrong.iter().any(|p| *p));
    let empty = s.solution[0].replace(
        "if test -f \"$first\"; then",
        "if test -f \"$first\" && test -n \"$(cat < \"$first\")\"; then",
    );
    assert_ne!(empty, s.solution[0]);
    assert!(!flags(&s, &empty)[4]);
}
#[test]
fn active_traces_are_real_diagnostics_and_never_independent_assignment_credit() {
    for s in labs() {
        let source = format!("set -x\n{}set +x\n", s.solution[0]);
        let r = record(&s, &source);
        assert!(r.results.iter().all(|r| r.error.is_none()));
        assert!(r.results.iter().any(|r| r.stderr.starts_with("+ ")));
        assert!(
            shell::checks(&source, &Some(r), cases(&s))
                .iter()
                .all(|c| !c.0)
        );
        let lesson = s.teaching.as_ref().unwrap();
        let changed = if s.id.ends_with("delivery") {
            lesson.example.replace("\"$place\"", "$place")
        } else {
            lesson
                .example
                .replace("first='gallery.log'", "first='live.log'")
        };
        assert_ne!(changed, lesson.example);
        assert!(
            !lesson
                .fixture
                .accepts(&shell::run(&changed, &lesson.fixture))
        );
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
    }
}
#[test]
fn failure_categories_remain_syntax_unsupported_and_wrong_behavior_not_canned_success() {
    let s = labs().remove(0);
    let c = &cases(&s)[0];
    let syntax = shell::run("if true; then printf broken", c);
    let e = syntax.error.unwrap();
    assert_eq!(
        script_teaching::diagnostic(&e),
        script_teaching::Diagnostic::Syntax
    );
    let unsupported = shell::run("set -x\nfor x in a; do printf '%s' \"$x\"; done", c);
    let e = unsupported.error.unwrap();
    assert_eq!(
        script_teaching::diagnostic(&e),
        script_teaching::Diagnostic::Unsupported
    );
    let wrong = shell::run("printf wrong", c);
    assert!(wrong.error.is_none());
    assert!(!c.accepts(&wrong));
    let escaped = shell::run("printf changed > /home/escape", c);
    assert!(escaped.error.is_some());
    assert_eq!(escaped.files, c.files);
}
#[test]
fn examples_drafts_current_success_selected_reset_and_safe_resume_stay_separate() {
    let catalog = labs();
    let s = &catalog[0];
    let other = &catalog[1];
    let mut p = Progress {
        schema_version: 1,
        clock: 4,
        ..Default::default()
    };
    for (n, s) in catalog.iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert(other.id.clone());
    let untouched = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let assignment = a.state.clone();
    let lesson = s.teaching.as_ref().unwrap();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learning.example_source.push_str("# Practice draft only\n");
    learning.example_result = Some(shell::run(&learning.example_source, &lesson.fixture));
    learning.example_revision = 19;
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(a, s));
    let source = s.solution[0].clone();
    a.state = State::Script {
        source: source.clone(),
        run: Some(record(s, &source)),
    };
    assert!(progress::current_solved(a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("\nprintf broken\n");
    assert!(!progress::current_solved(a, s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let mut resumed = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&resumed.attempts[&other.id]).unwrap(),
        untouched
    );
    let visited = resumed.attempts[&s.id].visited;
    let mut reset = Attempt::new(s, visited);
    workspace::prepare(&mut reset);
    script_teaching::prepare(&mut reset, lesson, true);
    resumed.attempts.insert(s.id.clone(), reset);
    assert!(!progress::current_solved(&resumed.attempts[&s.id], s));
    assert_eq!(
        serde_json::to_value(&resumed.attempts[&other.id]).unwrap(),
        untouched
    );
    assert_eq!(resumed.favorites, p.favorites);
}
#[test]
fn preserve255_labs73_units_and_place_the_two_trace_tasks_after_foundations() {
    let old = pack::accepted255().unwrap();
    let new = pack::accepted257().unwrap();
    assert_eq!((old.len(), new.len()), (255, 257));
    for (before, after) in old.iter().zip(&new) {
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(after).unwrap(),
            "{} changed",
            before.id
        );
    }
    let prior = course::accepted255(&old).unwrap();
    let units = course::accepted257(&new).unwrap();
    assert_eq!((prior.len(), units.len()), (73, 74));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "shell-tracing-diagnosis")
        .collect();
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-foundations")
        .unwrap();
    assert_eq!(units[at + 1].id, "shell-tracing-diagnosis");
    assert_eq!(
        units[at + 1].labs,
        vec!["shell-trace-delivery", "shell-trace-fallback"]
    );
    assert_eq!(new.iter().filter(|s| s.track == "Bash/Shell").count(), 68);
}
