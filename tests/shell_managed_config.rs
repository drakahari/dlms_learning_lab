//! Fixed authoring capability checks before admitting the managed-config lesson.
//! These scripts execute only in the bounded in-process interpreter.
use dlms_learning_lab::shell::{self, Case};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    source: String,
    alternate: String,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
}
fn fixture() -> Fixture {
    let f: Fixture =
        serde_json::from_str(include_str!("fixtures/shell-managed-config-reference.json")).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.cases.len(), 12);
    assert!(f.cases.iter().all(Case::valid));
    assert!(f.example_case.valid());
    f
}
#[test]
fn managed_config_both_supported_implementations_meet_all_twelve_behavioral_predictions() {
    let f = fixture();
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    for source in [&f.source, &f.alternate] {
        for c in &f.cases {
            let outcome = shell::run(source, c);
            assert!(c.accepts(&outcome), "{}: {outcome:?}", c.name);
        }
    }
}
#[test]
fn managed_config_separate_worked_example_really_applies_twice_in_one_run() {
    let f = fixture();
    let outcome = shell::run(&f.example_source, &f.example_case);
    assert!(f.example_case.accepts(&outcome), "{outcome:?}");
    assert_eq!(outcome.stdout, "Updated\nUnchanged\n");
    assert!(outcome.files.contains_key("/work/report.conf"));
    assert!(!outcome.files.contains_key("/work/service.conf"));
}
#[test]
fn managed_config_an_actual_first_result_is_the_unchanged_input_of_a_second_evaluation() {
    let f = fixture();
    let first_case = &f.cases[0];
    for source in [&f.source, &f.alternate] {
        let first = shell::run(source, first_case);
        assert!(first_case.accepts(&first));
        let mut next = first_case.clone();
        next.files = first.files;
        next.after = next.files.clone();
        next.stdout = "Unchanged\n".into();
        let second = shell::run(source, &next);
        assert!(next.accepts(&second), "{second:?}");
    }
}

#[test]
fn managed_config_default_read_preview_mutation_and_unconditional_reports_fail_usefully() {
    let f = fixture();
    let wrong = [
        f.source.replace("while IFS= read -r", "while read -r"),
        f.source.replace(
            "if test \"$mode\" = preview; then",
            "if test \"$mode\" = never; then",
        ),
        f.source.replace(" || test -n \"$line\"", ""),
        format!("{}\nrm keep.txt\n", f.source),
        format!("{}\nprintf 'extra\\n' > accidental.tmp\n", f.source),
    ];
    for source in &wrong {
        let outcomes = f
            .cases
            .iter()
            .map(|c| shell::run(source, c))
            .collect::<Vec<_>>();
        assert!(
            outcomes.iter().all(|r| r.error.is_none()),
            "Plausible wrong scripts must execute: {outcomes:?}"
        );
        let flags = f
            .cases
            .iter()
            .zip(&outcomes)
            .map(|(c, r)| c.accepts(r))
            .collect::<Vec<_>>();
        assert!(
            flags.iter().any(|p| *p) && flags.iter().any(|p| !*p),
            "Wrong solution must remain partial: {flags:?}"
        );
    }
    let no_op = "printf 'Unchanged\\n'\n";
    let flags = f
        .cases
        .iter()
        .map(|c| c.accepts(&shell::run(no_op, c)))
        .collect::<Vec<_>>();
    assert!(flags.iter().any(|p| *p) && flags.iter().any(|p| !*p));
}

#[test]
fn managed_config_hardcoding_a_successful_case_does_not_complete_the_project() {
    let f = fixture();
    let text = &f.cases[0].after["/work/service.conf"];
    assert!(!text.contains('\''));
    let source = format!("printf '%s' '{text}' > service.conf\nprintf 'Updated\\n'\n");
    let flags = f
        .cases
        .iter()
        .map(|c| c.accepts(&shell::run(&source, c)))
        .collect::<Vec<_>>();
    assert!(
        flags[0],
        "The negative must actually hardcode the first case correctly"
    );
    assert!(
        flags.iter().any(|p| !*p),
        "Other deterministic inputs must defeat hardcoding"
    );
}

#[test]
fn managed_config_syntax_unsupported_and_resource_errors_never_become_success() {
    let f = fixture();
    for source in [
        "if true; then\n",
        "sed -i 's/error/info/' service.conf\n",
        "while true; do :; done\n",
    ] {
        let result = shell::run(source, &f.cases[0]);
        assert!(
            result.error.is_some(),
            "Expected honest interpreter diagnostic: {source}"
        );
        assert!(!f.cases[0].accepts(&result));
    }
    let unsupported = shell::run("sed -i 's/error/info/' service.conf\n", &f.cases[0]);
    assert!(
        unsupported
            .error
            .unwrap()
            .to_ascii_lowercase()
            .contains("unsupported")
    );
}

#[test]
fn managed_config_correct_record_is_invalid_after_source_changes() {
    use dlms_learning_lab::engine::{self, Goal, State};
    let f = fixture();
    let mut state = State::Script {
        source: f.source.clone(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: f.source.clone(),
            results: f.cases.iter().map(|c| shell::run(&f.source, c)).collect(),
        }),
    };
    let goal = Goal::Script { cases: f.cases };
    assert!(engine::solved(&state, &goal));
    let State::Script { source, .. } = &mut state else {
        unreachable!()
    };
    source.push_str("\nprintf 'extra output\\n'\n");
    assert!(!engine::solved(&state, &goal));
}
