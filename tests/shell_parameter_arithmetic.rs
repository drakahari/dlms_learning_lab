//! Structural arithmetic/parameter boundaries, including the real Bash case032 correction.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
use dlms_learning_lab::{
    script_teaching::{self, Diagnostic},
    shell::{self, Case, Outcome, Vm},
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[path = "support/parameter_arithmetic_amendment.rs"]
mod amendment;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixed {
    name: String,
    source: String,
    case: Case,
}
fn case() -> Case {
    Case {
        name: "Arithmetic boundary negative".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "retained\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn sliced(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).expect("Balanced fixed source compiles");
    for _ in 0..20_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Finite test tick bound exceeded");
}
#[test]
fn separate_balanced_controls_are_supported_at_four_slices_without_editing_frozen35() {
    let rows: Vec<Fixed> = serde_json::from_str(include_str!(
        "fixtures/bash-parameter-arithmetic-reference.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 5);
    for row in rows {
        assert!(row.case.valid());
        for slice in [1, 7, 32, 64] {
            let out = sliced(&row.source, &row.case, slice);
            assert!(row.case.accepts(&out), "{} / {slice}: {out:?}", row.name);
        }
    }
}
#[test]
fn exact_original_invalid_case032_is_a_real_syntax_failure_not_a_skipped_positive() {
    let rows: Vec<Fixed> = serde_json::from_str(include_str!(
        "fixtures/bash-parameter-defaults-reference.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 35);
    let row = &rows[32];
    assert!(amendment::invalid_proposal(
        &row.name,
        &row.source,
        &row.case
    ));
    amendment::assert_syntax_before_effects(&row.source, &row.case);
}
#[test]
fn premature_arithmetic_closure_at_any_nesting_cannot_cross_the_parameter_span_or_run_prior_writes()
{
    let c = case();
    for expression in [
        "$(( ${number:-))))} + 1 ))",
        "$(( $(( ${number:-))} )) + 1 ))",
        "$(( 1 + $(( ${number:-))))} + 2 )) ))",
    ] {
        let source = format!("printf changed > keep.txt; number=7; printf '%s' \"{expression}\"");
        amendment::assert_syntax_before_effects(&source, &c);
    }
    let source = "printf changed > keep.txt; number=7; result=$(printf '%s' \"$(( $(( ${number:-))} )) + 1 ))\")";
    amendment::assert_syntax_before_effects(source, &c);
}
#[test]
fn nested_source_depth_and_excluded_word_syntax_remain_bounded_before_effects() {
    let c = case();
    let source = format!(
        "printf changed > keep.txt; printf '%s' \"{}1{}\"",
        "$(( ".repeat(25),
        " ))".repeat(25)
    );
    let error = Vm::new(&source, &c)
        .err()
        .expect("Source nesting is bounded even before selection");
    assert_eq!(
        script_teaching::diagnostic(&error),
        Diagnostic::Limit,
        "{error}"
    );
    assert_eq!(shell::run(&source, &c).files, c.files);
    for body in [
        "${number:-$other}",
        "${number:-$(printf bad)}",
        "${number:-'inner'}",
    ] {
        let source =
            format!("printf changed > keep.txt; number=7; printf '%s' \"$(( {body} + 1 ))\"");
        let error = Vm::new(&source, &c)
            .err()
            .expect("Unselected unsupported arithmetic fallback is still checked");
        assert_eq!(
            script_teaching::diagnostic(&error),
            Diagnostic::Unsupported,
            "{error}"
        );
        assert_eq!(shell::run(&source, &c).files, c.files);
    }
}

#[test]
fn valid_selected_arithmetic_value_errors_are_runtime_failures_not_syntax_preflight() {
    let c = case();
    let source = "printf changed > keep.txt; printf '%s' \"$(( ${number:-1/0} ))\"";
    assert!(Vm::new(source, &c).is_ok());
    let out = shell::run(source, &c);
    let error = out
        .error
        .as_deref()
        .expect("Selected division by zero is still an arithmetic failure");
    assert_eq!(script_teaching::diagnostic(error), Diagnostic::Execution);
    assert_eq!(out.status, 2);
    assert_eq!(out.files["/work/keep.txt"], "changed");
    assert!(out.stdout.is_empty());
}

#[test]
fn ordinary_arithmetic_lexical_messages_are_not_blanket_reclassified_by_the_parameter_extension() {
    let c = case();
    for source in ["printf '%s' $((1 + 2", "printf '%s' \"$((1 + 2)\""] {
        let error = Vm::new(source, &c)
            .err()
            .expect("Original malformed arithmetic remains rejected");
        assert_eq!(
            error,
            "Syntax error: Arithmetic error: close expansion with ))"
        );
        assert_eq!(shell::run(source, &c).files, c.files);
    }
}
