//! Authored repair projects use the existing bounded evaluator; never host execution.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Record, Vm},
    workspace,
};
fn labs() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/shell-debug-recovery-v1.json")).unwrap()
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn all_cases(s: &Scenario) -> Vec<Case> {
    let v: std::collections::BTreeMap<String, Vec<Case>> =
        serde_json::from_str(include_str!("fixtures/shell-debug-recovery-cases.json")).unwrap();
    v[&s.id].clone()
}

fn record(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn passes(s: &Scenario, source: &str) -> Vec<bool> {
    cases(s)
        .iter()
        .zip(record(s, source).results)
        .map(|(c, r)| c.accepts(&r))
        .collect()
}
fn alternate(s: &Scenario) -> String {
    let text = &s.solution[0];
    if s.id.ends_with("debug-status") {
        text.replace(
            "if test \"$mode\" = trace; then printf 'Trace: reading %s\\n' \"$file\" >&2; fi",
            "test \"$mode\" != trace || printf 'Trace: reading %s\\n' \"$file\" >&2",
        )
        .replace(
            "if test \"$mode\" = trace; then printf 'Trace: result %s\\n' \"$result\" >&2; fi",
            "test \"$mode\" != trace || printf 'Trace: result %s\\n' \"$result\" >&2",
        )
    } else {
        text.replace("pending=()","pending=0").replace("pending=( \"${pending[@]}\" \"$id\" )","pending=$((pending+1))").replace("for id in \"${pending[@]}\"; do printf 'Completed: %s\\n' \"$id\" > \"$id.done\"; done","for id in \"${jobs[@]}\"; do\n case \"$completed\" in *\"|$id|\"*) : ;; *) printf 'Completed: %s\\n' \"$id\" > \"$id.done\" ;; esac\ndone").replace("${#pending[@]}","$pending")
    }
}
#[test]
fn debug_recovery_both_solutions_alternates_examples_and_starter_failures() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    for s in labs() {
        let c = cases(&s);
        assert_eq!(c.len(), 12);
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        assert!(passes(&s, source).iter().any(|r| !r));
        for source in [&s.solution[0], &alternate(&s)] {
            for case in &all_cases(&s) {
                let out = shell::run(source, case);
                assert!(
                    case.accepts(&out),
                    "{} / {}\nactual {:?}\nexpected {:?}",
                    s.id,
                    case.name,
                    out,
                    case
                );
            }
        }
    }
}
#[test]
fn debug_recovery_cooperative_execution_matches_small_slices_and_stop_is_bounded() {
    for s in labs() {
        let source = &s.solution[0];
        for case in &all_cases(&s) {
            let mut vm = Vm::new(source, case).unwrap();
            for _ in 0..5000 {
                if vm.done() {
                    break;
                }
                vm.tick(1);
            }
            assert!(vm.done());
            assert!(case.accepts(&vm.outcome), "{} {}", s.id, case.name);
        }
        let case = &cases(&s)[0];
        let mut vm = Vm::new("while true; do printf 'loop\\n'; done", case).unwrap();
        vm.tick(2);
        assert!(!vm.done());
        vm.stop();
        assert!(vm.done());
        assert!(vm.outcome.error.is_some());
        assert!(!case.accepts(&vm.outcome));
    }
}
#[test]
fn debug_recovery_examples_and_intentional_changes_are_isolated_and_ungraded() {
    for s in labs() {
        let lesson = s.teaching.as_ref().unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, lesson, true);
        let original = a.state.clone();
        for view in [View::Introduction, View::Example, View::Assignment] {
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            l.view = view;
            l.example_source = if s.id.ends_with("debug-status") {
                lesson
                    .example
                    .replace("result=$?\nprintf", "printf 'Oops\\n'\nresult=$?\nprintf")
            } else {
                lesson.example.replace("fern moss", "elm moss")
            };
            l.example_result = Some(shell::run(&l.example_source, &lesson.fixture));
            assert!(!lesson.fixture.accepts(l.example_result.as_ref().unwrap()));
            assert_eq!(a.state, original);
            assert!(!progress::current_solved(&a, &s));
        }
    }
}
#[test]
fn debug_recovery_records_invalidate_edits_preserve_drafts_and_reset_only_one_attempt() {
    let all = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let source = s.solution[0].clone();
        a.state = State::Script {
            source: source.clone(),
            run: Some(record(s, &source)),
        };
        assert!(progress::current_solved(&a, s));
        a.workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap()
            .example_source
            .push_str("# unfinished sample\n");
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let id = &all[0].id;
    let before = p.attempts[&all[1].id].clone();
    if let State::Script { source, .. } = &mut p.attempts.get_mut(id).unwrap().state {
        source.push_str("# edit invalidates old result\n");
    }
    assert!(!progress::current_solved(&p.attempts[id], &all[0]));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let q = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    p.attempts.insert(id.clone(), Attempt::new(&all[0], 2));
    assert_eq!(
        serde_json::to_value(&p.attempts[&all[1].id]).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert!(p.favorites.contains(id));
}
#[test]
fn debug_trace_quiet_pairs_match_business_output_and_status_but_not_diagnostics() {
    let s = labs().remove(0);
    for pair in 0..5 {
        let full = all_cases(&s);
        let a = &full[pair];
        let b = &full[pair + 5];
        assert_eq!(
            (a.stdout.clone(), a.status, a.after.clone()),
            (b.stdout.clone(), b.status, b.after.clone())
        );
        assert!(!a.stderr.contains("Trace:"));
        assert!(b.stderr.starts_with("Trace: reading events.log\n"));
        assert!(b.stderr.ends_with(&format!("Trace: result {}\n", b.status)));
    }
}
#[test]
fn debug_recovery_rejects_hardcoded_outputs_early_writes_substring_checks_and_late_status_capture()
{
    for s in labs() {
        let mut wrong = vec![
            "true".into(),
            format!(
                "printf '%s' '{}'\nexit 0",
                cases(&s)[0].stdout.replace('\'', "")
            ),
        ];
        if s.id.ends_with("debug-status") {
            wrong.push(s.solution[0].replace("result=$?", "printf 'Debug\\n' >&2\n  result=$?"));
            wrong.push(s.solution[0].replace("exit \"$result\"", "exit 0"));
            wrong.push(s.solution[0].replace("\"$matches\"", "$matches"));
            wrong.push(s.solution[0].replace(" >&2", ""));
        } else {
            wrong.push(s.solution[0].replace("if test \"$mode\" = apply; then", "if true; then"));
            wrong.push(
                s.solution[0].replace("if test \"$actual\" = \"$expected\"; then", "if true; then"),
            );
            wrong.push(s.solution[0].replace("if test -f \"$id.done\"; then printf 'Untracked output: %s\\n' \"$id\" >&2; exit 2; fi", ":"));
        }
        for source in wrong {
            let rows = passes(&s, &source);
            assert!(rows.iter().any(|r| !r), "{} bad solution passed", s.id);
        }
    }
}
#[test]
fn recovery_all_errors_are_read_only_and_second_apply_is_idempotent() {
    let s = labs().remove(1);
    for case in &all_cases(&s) {
        if case.status == 2 {
            assert_eq!(case.files, case.after);
            assert!(case.stdout.is_empty());
        } else if case.args == ["apply"] {
            let first = shell::run(&s.solution[0], case);
            assert!(case.accepts(&first));
            let mut again = case.clone();
            again.files = first.files.clone();
            let result = shell::run(&s.solution[0], &again);
            assert_eq!(result.status, 0);
            assert!(result.error.is_none());
            assert_eq!(result.files, first.files);
            assert!(result.stdout.ends_with("Pending count: 0\n"));
        }
    }
}

#[test]
fn debug_recovery_preserves_all229_prior_labs_and61_units() {
    let old = pack::accepted229().unwrap();
    let now = pack::accepted231().unwrap();
    assert_eq!(old.len(), 229);
    assert_eq!(now.len(), 231);
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..229]).unwrap()
    );
    let prior = dlms_learning_lab::course::accepted229(&old).unwrap();
    let units = dlms_learning_lab::course::accepted231(&now).unwrap();
    assert_eq!(prior.len(), 61);
    assert_eq!(units.len(), 62);
    assert_eq!(
        serde_json::to_value(prior).unwrap(),
        serde_json::to_value(&units[..61]).unwrap()
    );
    assert_eq!(
        units.last().unwrap().labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    assert_eq!(now.iter().filter(|s| s.track == "Bash/Shell").count(), 66);
}
