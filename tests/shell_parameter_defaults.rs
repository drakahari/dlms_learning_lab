//! Capability vectors only, not authored labs or captured Bash observations.
//! No subprocess, host file lookup or network can execute learner source here.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use serde::Deserialize;
#[path = "support/parameter_arithmetic_amendment.rs"]
mod arithmetic_amendment;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixed {
    name: String,
    source: String,
    case: Case,
}
fn case() -> Case {
    Case {
        name: "Parameter capability control".into(),
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
    let mut vm = Vm::new(source, case).expect("Supported fixed source compiles");
    for _ in 0..20_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Cooperative source exceeded finite test tick bound");
}
#[test]
fn frozen35_proposals_have34_supported_results_and_one_explicit_real_syntax_failure() {
    assert_eq!(shell::EVALUATOR_REVISION, 13);
    let fixed: Vec<Fixed> = serde_json::from_str(include_str!(
        "fixtures/bash-parameter-defaults-reference.json"
    ))
    .unwrap();
    assert_eq!(fixed.len(), 35);
    assert_eq!(
        fixed.iter().map(|f| &f.name).collect::<BTreeSet<_>>().len(),
        35
    );
    let mut supported = 0;
    let mut invalid = 0;
    for (index, f) in fixed.into_iter().enumerate() {
        assert!(f.case.valid(), "{}", f.name);
        if arithmetic_amendment::invalid_proposal(&f.name, &f.source, &f.case) {
            assert_eq!(index, 32);
            arithmetic_amendment::assert_syntax_before_effects(&f.source, &f.case);
            invalid += 1;
        } else {
            for slice in [1, 7, 32, 64] {
                let result = sliced(&f.source, &f.case, slice);
                assert!(f.case.accepts(&result), "{} / {slice}: {result:?}", f.name);
            }
            supported += 1;
        }
    }
    assert_eq!((supported, invalid), (34, 1));
}
#[test]
fn genuinely_different_conditionals_agree_and_common_wrong_defaults_do_not() {
    let direct = r#"printf '<%s>|<%s>|<%s>|<%s>\n' "${value-Default}" "${value:-Default}" "${value+Marker}" "${value:+Marker}""#;
    let alternative = r#"
if test -n "${value+set}"; then first=$value; alt=Marker; else first=Default; alt=; fi
if test -n "$value"; then second=$value; nonempty=Marker; else second=Default; nonempty=; fi
printf '<%s>|<%s>|<%s>|<%s>\n' "$first" "$second" "$alt" "$nonempty"
"#;
    let wrong = [
        direct.replace("${value-Default}", "${value:-Default}"),
        direct.replace("${value:-Default}", "${value-Default}"),
        direct.replace("${value+Marker}", "${value:+Marker}"),
        direct.replace("${value:+Marker}", "${value+Marker}"),
        "printf '<Default>|<Default>|<>|<>\\n'".into(),
    ];
    let mut rejected = vec![false; wrong.len()];
    for value in [None, Some(""), Some("two words % *")] {
        let mut c = case();
        if let Some(value) = value {
            c.env.insert("value".into(), value.into());
        }
        let expected = shell::run(direct, &c);
        assert_eq!(expected.error, None);
        assert_eq!(shell::run(alternative, &c), expected);
        for (n, source) in wrong.iter().enumerate() {
            rejected[n] |= shell::run(source, &c) != expected;
        }
    }
    assert!(
        rejected.into_iter().all(|r| r),
        "Every plausible misconception and first-case hardcode is rejected"
    );
}
#[test]
fn unsupported_word_forms_are_diagnosed_statically_even_in_an_unchosen_branch() {
    for word in [
        "$other",
        "${other}",
        "$(printf bad)",
        "$((1+2))",
        "'inner'",
        "\"inner\"",
        "a\\ b",
        "~",
        "~user",
        "[abc]",
        "a{b",
        "`printf bad`",
    ] {
        let source = format!(
            "present=kept; printf changed > keep.txt; printf '%s' \"${{present:-{word}}}\""
        );
        let c = case();
        let error = Vm::new(&source, &c)
            .err()
            .expect("Excluded WORD fails before execution");
        assert!(error.contains("Unsupported"), "{word}: {error}");
        let out = shell::run(&source, &c);
        assert!(
            out.error
                .as_deref()
                .is_some_and(|e| e.contains("Unsupported"))
        );
        assert_eq!(out.status, 2);
        assert!(out.stdout.is_empty() && out.stderr.is_empty());
        assert_eq!(out.files, c.files, "No earlier write executes: {word}");
    }
}
#[test]
fn arrays_special_targets_and_other_parameter_operators_remain_honest_failures() {
    for source in [
        "printf '%s' \"${0-default}\" > keep.txt",
        "printf '%s' \"${12-default}\" > keep.txt",
        "printf '%s' \"${UID-default}\" > keep.txt",
        "printf '%s' \"${IFS-default}\" > keep.txt",
        "value=(); printf '%s' \"${value:-default}\" > keep.txt",
        "value=(one); printf '%s' \"${value+other}\" > keep.txt",
        "printf '%s' \"${value[0]:-default}\" > keep.txt",
        "printf '%s' \"${value:=assigned}\" > keep.txt",
        "printf '%s' \"${value?required}\" > keep.txt",
        "printf '%s' \"${value:?required}\" > keep.txt",
        "printf '%s' \"${#value}\" > keep.txt",
        "printf '%s' \"${value%tail}\" > keep.txt",
        "printf '%s' \"${value:1}\" > keep.txt",
    ] {
        let c = case();
        let result = shell::run(source, &c);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Unsupported")),
            "{source}: {result:?}"
        );
        assert_eq!(result.status, 2);
        assert_eq!(
            result.files, c.files,
            "Command preparation rejects before its redirect"
        );
    }
    for source in [
        "printf '%s' \"${value:-missing\"",
        "printf '%s' ${value:-missing",
    ] {
        let c = case();
        assert!(Vm::new(source, &c).is_err());
        assert_eq!(shell::run(source, &c).files, c.files);
    }
}
#[test]
fn unchosen_alternate_does_not_copy_a_large_existing_value() {
    let mut c = case();
    c.env.insert("large".into(), "v".repeat(16_384));
    let words = "item ".repeat(100);
    let source = format!("for n in {words}; do printf '%s' \"${{large:+x}}\"; done");
    let out = shell::run(&source, &c);
    assert_eq!(
        out.error, None,
        "Presence check must not expand/copy100large values"
    );
    assert_eq!(out.stdout, "x".repeat(100));
    assert_eq!(out.files, c.files);
}
#[test]
fn selected_fallbacks_reuse_existing_argument_memory_work_and_output_limits() {
    let c = case();
    let source = format!("printf '%s' ${{missing:-{}}} > keep.txt", "x ".repeat(200));
    let out = shell::run(&source, &c);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("argument")),
        "{out:?}"
    );
    assert_eq!(out.files, c.files);
    let mut c = case();
    c.env.insert("large".into(), "v".repeat(90_000));
    let out = shell::run(
        "printf '%s' \"${large-default}\" \"${large-default}\" \"${large-default}\" > keep.txt",
        &c,
    );
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("memory")),
        "{out:?}"
    );
    assert_eq!(out.files, c.files);
    let source = format!(
        "while true; do printf '%s' \"${{missing:-{}}}\"; done",
        "x".repeat(256)
    );
    let out = shell::run(&source, &case());
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("output")),
        "{out:?}"
    );
    assert!(out.stdout.len() <= 32_768);
    let words = "item ".repeat(90);
    let body = "x".repeat(8000);
    let skipped = format!(
        "value=kept; for n in {words}; do result=\"${{value:-{body}}}\"; done; printf '%s' \"$result\""
    );
    let out = shell::run(&skipped, &case());
    assert_eq!(
        out.error, None,
        "Unchosen literal incurs scan work, not selected-value copies: {out:?}"
    );
    assert_eq!(out.stdout, "kept");
    let selected = skipped.replacen("value=kept; ", "", 1);
    let out = shell::run(&selected, &case());
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("work limit")),
        "{out:?}"
    );
}
#[test]
fn repeat_execution_is_bounded_and_stop_stays_ungraded() {
    let source = "while true; do : \"${missing:-x}\"; done";
    let c = case();
    let out = shell::run(source, &c);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("instruction")),
        "{out:?}"
    );
    assert_eq!(out.files, c.files);
    let mut vm = Vm::new(source, &c).unwrap();
    for _ in 0..12 {
        vm.tick(1);
    }
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(
        vm.outcome
            .error
            .as_deref()
            .is_some_and(|e| e.starts_with("Stopped"))
    );
    let before = vm.outcome.clone();
    vm.tick(64);
    assert_eq!(vm.outcome, before);
}
