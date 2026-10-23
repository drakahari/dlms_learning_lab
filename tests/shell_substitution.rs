use dlms_learning_lab::shell::{self, Case, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn case() -> Case {
    Case {
        name: "Parser foundation".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "retained".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
#[test]
fn public_execution_is_still_gated_before_any_parent_or_child_mutation() {
    assert_eq!(shell::EVALUATOR_REVISION, 6);
    for source in [
        "printf changed > keep.txt; value=$(printf captured)",
        "printf changed > keep.txt; value=$(while true; do :; done)",
        "outer() { local label=visible; value=$(printf '%s' \"$label\"); }; outer",
        ": > \"$(printf changed > side.txt; printf destination)\"",
    ] {
        assert!(
            Vm::new(source, &case())
                .err()
                .unwrap()
                .contains("Command substitution is unsupported")
        );
        let result = shell::run(source, &case());
        assert_eq!(result.files, case().files);
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
        assert_eq!(result.status, 2);
        assert!(result.error.is_some());
    }
}
#[derive(Deserialize)]
struct Reference {
    name: String,
    source: String,
    case: Case,
}
#[test]
fn selected_future_reference_cases_cannot_run_under_the_parser_only_milestone() {
    let cases: Vec<Reference> =
        serde_json::from_str(include_str!("fixtures/shell-substitution-reference.json")).unwrap();
    assert_eq!(cases.len(), 62);
    for fixture in cases {
        let result = shell::run(&fixture.source, &fixture.case);
        assert_eq!(
            result.error.as_deref(),
            Some("Command substitution is unsupported"),
            "{} must parse successfully and remain gated",
            fixture.name
        );
        assert_eq!(result.status, 2, "{}", fixture.name);
        assert_eq!(result.files, fixture.case.files, "{}", fixture.name);
        assert!(result.stdout.is_empty(), "{}", fixture.name);
    }
}
#[test]
fn literal_syntax_and_existing_functions_arrays_pipelines_and_arithmetic_are_unchanged() {
    for (source, stdout) in [
        (
            r#"printf '<%s>' '$(false)' "\$(false)""#,
            "<$(false)><$(false)>",
        ),
        (
            "parts=(one two); render() { local label=inner; printf '%s:%s' \"${parts[1]}\" \"$((2+3))\"; }; render",
            "two:5",
        ),
        (
            "printf input | read -r label; printf '<%s>' \"$label\"",
            "<>",
        ),
    ] {
        let result = shell::run(source, &case());
        assert_eq!(result.error, None, "{source}");
        assert_eq!(result.stdout, stdout);
    }
    for source in [
        "show $(printf x)",
        "show \"$(printf x)\"",
        "show pre$(printf x)post",
    ] {
        assert!(shell::literal_command(source).is_err());
    }
    assert_eq!(
        shell::literal_command("show '$(printf x)'").unwrap(),
        ["show", "$(printf x)"]
    );
    assert_eq!(
        shell::literal_command(r#"show "\$(printf x)""#).unwrap(),
        ["show", "$(printf x)"]
    );
}
