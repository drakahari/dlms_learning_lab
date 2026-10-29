//! Original bounded-case regression cases, plus retained actual fixed Bash5.3.9
//! observations. No test invokes a host shell, account tool, process or network.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
use dlms_learning_lab::shell::{self, Case, Outcome, Record, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn fixture() -> Case {
    Case {
        name: "Case construct fixture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "keep\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn sliced(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap_or_else(|error| panic!("{source}: {error}"));
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded evaluator did not finish: {source}");
}
#[derive(Deserialize)]
struct Oracle {
    manifest_sha256: String,
    manual_sha256: String,
    locale: String,
    observations: Vec<Observation>,
}
#[derive(Deserialize)]
struct Observation {
    name: String,
    source: String,
    case: Case,
    actual: Outcome,
    excluded: bool,
    observation_sha256: String,
}
#[test]
fn retained_actual_bash_observations_match_at_all_tick_slices_with_explicit_exclusions() {
    let oracle: Oracle =
        serde_json::from_str(include_str!("fixtures/shell-case-bash539.json")).unwrap();
    assert_eq!(
        oracle.manifest_sha256,
        "7af1b2e86a9cf9364c5ea170bcdfba4142a5c8cff0c5046efd0be57c55af02bd"
    );
    assert_eq!(
        oracle.manual_sha256,
        "b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4"
    );
    assert_eq!(oracle.locale, "C");
    assert_eq!(oracle.observations.len(), 37);
    const {
        assert!(shell::EVALUATOR_REVISION >= 8);
    } // Retained case capability; migration/current acceptance remains a separate gate
    let mut amended = 0;
    for observation in oracle.observations {
        assert_eq!(observation.observation_sha256.len(), 64);
        if observation.name == "dynamic-bracket-boundary" {
            let amendment: serde_json::Value =
                serde_json::from_str(include_str!("fixtures/shell-case-class-amendment-v12.json"))
                    .unwrap();
            assert_eq!(
                amendment["original_fixture_sha256"],
                amendment_sha::hash(include_bytes!("fixtures/shell-case-bash539.json"))
            );
            assert!(observation.excluded && amendment["old_excluded"] == true);
            assert_eq!(amendment["amended_name"], observation.name);
            assert_eq!(
                amendment["original_observation_sha256"],
                observation.observation_sha256
            );
            assert_eq!(amendment["source"], observation.source);
            assert_eq!(
                amendment["case"],
                serde_json::to_value(&observation.case).unwrap()
            );
            for slice in [1, 16, 32, 64] {
                assert_eq!(
                    sliced(&observation.source, &observation.case, slice),
                    observation.actual
                );
            }
            amended += 1;
            continue;
        }
        if observation.excluded {
            let result = shell::run(&observation.source, &observation.case);
            assert_eq!(result.status, 2, "{}", observation.name);
            assert!(
                result
                    .error
                    .as_deref()
                    .is_some_and(|e| e.contains("Unsupported")),
                "{}: {result:?}",
                observation.name
            );
            continue;
        }
        for slice in [1, 16, 32, 64] {
            assert_eq!(
                sliced(&observation.source, &observation.case, slice),
                observation.actual,
                "{} / {slice}",
                observation.name
            );
        }
    }
    assert_eq!(
        amended, 1,
        "Only the source-keyed class feature changes retained exclusions"
    );
}
#[test]
fn malformed_or_excluded_static_grammar_fails_before_any_effect() {
    for source in [
        "case x in x printf bad;; esac",
        "case x x) :;; esac",
        "case x in |x) :;; esac",
        "case x in x|) :;; esac",
        "case x in x) :;;",
        "case x in x) printf bad esac",
        "case x in x) :;& *) :;; esac",
        "case x in x) :;;& *) :;; esac",
        "case x in x) :;; esac > keep.txt",
        "case x in x) :;; esac | cat",
        "printf x | case x in x) :;; esac",
        "case x in x) :;; esac;; printf bad",
        "if true; then case x in esac; else fi",
        "esac",
        "printf x | case x",
        "printf x | esac",
        "printf x | > keep.txt case x",
        "> keep.txt esac",
        "case x in x) :;; esac )",
    ] {
        let source = format!("printf changed > keep.txt; {source}");
        assert!(Vm::new(&source, &fixture()).is_err(), "{source}");
        let result = shell::run(&source, &fixture());
        assert_eq!(result.files, fixture().files, "{source}");
        assert!(
            result.stdout.is_empty() && result.stderr.is_empty(),
            "{source}"
        );
        assert_eq!(result.status, 2);
    }
}
#[test]
fn pattern_alternative_arm_and_nesting_limits_are_independent() {
    let arms = |count| {
        format!(
            "case x in {} esac",
            (0..count).map(|i| format!("p{i}) :;;")).collect::<String>()
        )
    };
    assert_eq!(shell::run(&arms(32), &fixture()).error, None);
    assert!(
        Vm::new(&arms(33), &fixture())
            .err()
            .unwrap()
            .contains("arm limit")
    );
    let alternatives = |count| format!("case x in {}) :;; esac", vec!["p"; count].join("|"));
    assert_eq!(shell::run(&alternatives(8), &fixture()).error, None);
    assert!(
        Vm::new(&alternatives(9), &fixture())
            .err()
            .unwrap()
            .contains("alternative limit")
    );
    let total = |count| {
        format!(
            "case x in {} esac",
            (0..count)
                .map(|_| "a|b|c|d|e|f|g|h) :;;")
                .collect::<String>()
        )
    };
    assert_eq!(shell::run(&total(16), &fixture()).error, None);
    assert!(
        Vm::new(&total(17), &fixture())
            .err()
            .unwrap()
            .contains("alternative limit")
    );
    let nested = |count| {
        format!(
            "{} :; {}",
            "case x in x) ".repeat(count),
            ";; esac".repeat(count)
        )
    };
    assert_eq!(shell::run(&nested(20), &fixture()).error, None);
    assert!(
        Vm::new(&nested(25), &fixture())
            .err()
            .unwrap()
            .contains("nesting")
    );
}
#[test]
fn expanded_pattern_bound_applies_to_literal_and_quoted_text_without_truncation() {
    for quoted in [false, true] {
        let mut case = fixture();
        let source = if quoted {
            "case x in \"$p\") printf hit;; *) printf other;; esac"
        } else {
            "case x in $p) printf hit;; *) printf other;; esac"
        };
        case.env.insert("p".into(), "x".repeat(256));
        assert_eq!(shell::run(source, &case).stdout, "other");
        case.env.insert("p".into(), "x".repeat(257));
        let result = shell::run(source, &case);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("256 bytes"))
        );
        assert!(result.stdout.is_empty());
        assert_eq!(result.files, case.files);
    }
}
#[test]
fn dynamic_unsupported_patterns_are_lazy_and_quoted_data_stays_literal() {
    for pattern in ["\\*", "@(a|b)"] {
        let mut case = fixture();
        case.env.insert("p".into(), pattern.into());
        assert_eq!(
            shell::run(
                "case good in good) printf yes;; $p) printf no;; esac",
                &case
            )
            .stdout,
            "yes"
        );
        let result = shell::run(
            "printf before > before.txt; case x in $p) printf no;; esac",
            &case,
        );
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Unsupported"))
        );
        assert_eq!(
            result.files.get("/work/before.txt").map(String::as_str),
            Some("before")
        );
        assert!(result.stdout.is_empty());
        case.env.insert("subject".into(), pattern.into());
        assert_eq!(
            shell::run("case $subject in \"$p\") printf yes;; esac", &case).stdout,
            "yes"
        );
    }
}
#[test]
fn selector_is_scalar_data_including_brackets_while_list_parameters_remain_excluded() {
    assert_eq!(
        shell::run("case [ab] in '[ab]') printf yes;; esac", &fixture()).stdout,
        "yes"
    );
    for word in ["$@", "\"$@\"", "${items[@]}", "\"${items[@]}\""] {
        for source in [
            format!("items=(one two); case {word} in *) :;; esac"),
            format!("items=(one two); case x in {word}) :;; esac"),
        ] {
            let result = shell::run(&source, &fixture());
            assert_eq!(result.status, 2, "{source}");
            assert!(result.error.is_some());
        }
    }
    assert_eq!(
        shell::run(
            "items=(one two); case ${items[1]} in two) printf '%s' \"${#items[@]}\";; esac",
            &fixture()
        )
        .stdout,
        "2"
    );
}
#[test]
fn compound_contexts_capture_delimiters_and_function_targets_do_not_escape_their_arena() {
    for source in [
        "result=$(case x in x) printf one;; esac); printf '%s' \"$result\"",
        "f() { case x in x) printf one;; esac; }; result=$(f); printf '%s' \"$result\"",
        "result=$(f() { case x in x) printf one;; esac; }; f); printf '%s' \"$result\"",
        "result=$(if true; then case x in x) printf one;; esac; fi); printf '%s' \"$result\"",
        "result=$(for x in x; do case $x in x) printf one;; esac; done); printf '%s' \"$result\"",
        "result=$(values=(x); case ${values[0]} in x) printf one;; esac); printf '%s' \"$result\"",
        "result=$(case x in x) case y in y) printf one;; esac;; esac); printf '%s' \"$result\"",
        "result=$(ca\\\nse x in (x) printf one;; esac); printf '%s' \"$result\"",
        "false || case x in x) printf one;; esac",
        "while true; do case x in x) printf one; exit 0;; esac; done",
    ] {
        for slice in [1, 16, 32, 64] {
            let result = sliced(source, &fixture(), slice);
            assert_eq!(result.error, None, "{source}");
            assert_eq!(result.stdout, "one", "{source}");
        }
    }
    let result = sliced(
        "result=$(case x in y) :;; x) > esac;; z) :;; esac); printf '[%s]' \"$result\"",
        &fixture(),
        1,
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "[]");
    assert_eq!(result.files.get("/work/esac").map(String::as_str), Some(""));
    // Literal arguments and quoted/mixed keyword-looking words never open a case scanner frame.
    for source in [
        "value=$(printf '%s' case); printf '%s' \"$value\"",
        "value=$(printf '%s' 'case'); printf '%s' \"$value\"",
        "value=$(printf '%s' c\"a\"se); printf '%s' \"$value\"",
    ] {
        assert_eq!(sliced(source, &fixture(), 1).stdout, "case");
    }
}
#[test]
fn reached_capture_effects_stderr_and_variable_scope_are_once_only() {
    let source = "value=parent; case \"$(printf s >> trace.txt; printf x)\" in \"$(value=child; printf p >> trace.txt; printf err >&2; printf x)\") value=body; printf '%s' \"$value\";; $(printf later >> trace.txt; printf '*')) printf no;; esac; printf ':%s' \"$value\"";
    for slice in [1, 16, 32, 64] {
        let result = sliced(source, &fixture(), slice);
        assert_eq!(result.error, None);
        assert_eq!(result.stdout, "body:body");
        assert_eq!(result.stderr, "err");
        assert_eq!(
            result.files.get("/work/trace.txt").map(String::as_str),
            Some("sp")
        );
    }
}
#[test]
fn stop_during_pattern_child_drops_its_continuation_and_never_runs_the_body() {
    let source = "case x in \"$(printf entered > trace.txt; while true; do :; done)\") printf wrong > keep.txt;; esac";
    let mut vm = Vm::new(source, &fixture()).unwrap();
    for _ in 0..1000 {
        vm.tick(1);
        if vm.outcome.files.contains_key("/work/trace.txt") {
            break;
        }
    }
    assert_eq!(
        vm.outcome.files.get("/work/trace.txt").map(String::as_str),
        Some("entered")
    );
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    let stopped = vm.outcome.clone();
    for _ in 0..100 {
        vm.tick(64);
    }
    assert_eq!(vm.outcome, stopped);
    assert_eq!(
        vm.outcome.files.get("/work/keep.txt"),
        fixture().files.get("/work/keep.txt")
    );
}
#[test]
fn matching_work_is_shared_bounded_and_cannot_turn_failure_into_no_match() {
    let mut case = fixture();
    case.env.insert("subject".into(), "a".repeat(12_000));
    case.env
        .insert("pattern".into(), format!("*{}b", "a".repeat(120)));
    for slice in [1, 16, 32, 64] {
        let result = sliced(
            "case $subject in $pattern) printf wrong;; *) printf fallback;; esac",
            &case,
            slice,
        );
        assert_eq!(result.status, 2);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Expansion work limit"))
        );
        assert!(result.stdout.is_empty());
    }
}
#[test]
fn case_live_subject_pattern_and_suspended_child_share_the_existing_memory_ceiling() {
    let mut case = fixture();
    case.env.insert("pad".into(), "p".repeat(60_000));
    case.env.insert("subject".into(), "x".repeat(60_000));
    let result = shell::run("copy=$pad; case $subject in *) printf wrong;; esac", &case);
    assert_eq!(result.status, 2);
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("memory") || e.contains("limit"))
    );
    assert!(result.stdout.is_empty());
    let mut case = fixture();
    case.env.insert("pad".into(), "p".repeat(80_000));
    case.env.insert("subject".into(), "x".repeat(35_000));
    let result = shell::run(
        "case $subject in \"$(printf x)\") printf wrong;; esac",
        &case,
    );
    assert_eq!(result.status, 2);
    assert!(result.error.is_some());
    assert!(result.stdout.is_empty());
}
#[test]
fn literal_terminal_and_historical_records_do_not_gain_execution_or_current_credit() {
    for source in [
        "case x in x) :;; esac",
        "show $(case x in x) printf x;; esac)",
    ] {
        assert!(shell::literal_command(source).is_err());
    }
    assert_eq!(
        shell::literal_command("show 'case' ';;' 'esac'").unwrap(),
        ["show", "case", ";;", "esac"]
    );
    // Structural legacy shape only: no revision8 execution is relabeled as7.
    // Genuine actual6/7 outcomes are retained separately for migration tests.
    let old = Record {
        engine_revision: 7,
        source: "printf old".into(),
        results: vec![],
    };
    assert!(old.valid());
    assert!(!old.current());
    let checks = shell::checks(&old.source, &Some(old.clone()), &[fixture()]);
    assert!(checks.iter().all(|(pass, _)| !pass));
    assert_eq!(
        serde_json::from_str::<Record>(&serde_json::to_string(&old).unwrap()).unwrap(),
        old
    );
}

#[test]
fn captures_in_case_preserve_shared_finite_input_and_active_source_write_guards() {
    let mut case = fixture();
    case.stdin = "first\nsecond\n".into();
    let source = "case \"$(read -r line; printf '%s' \"$line\")\" in first) read -r tail; printf '%s' \"$tail\";; esac";
    for slice in [1, 16, 32, 64] {
        let result = sliced(source, &case, slice);
        assert_eq!(result.error, None);
        assert_eq!(result.stdout, "second");
    }
    case.files
        .insert("/work/source.txt".into(), "first\nsecond\n".into());
    let source = "while read -r line; do case \"$line\" in first) value=$(printf wrong > source.txt);; esac; done < source.txt";
    let result = shell::run(source, &case);
    assert_eq!(result.status, 2);
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|error| error.contains("Unsupported write"))
    );
    assert_eq!(
        result.files.get("/work/source.txt"),
        case.files.get("/work/source.txt")
    );
}

#[test]
fn capture_delimiter_keeps_array_elements_as_data_and_reenters_real_commands_afterwards() {
    for (source, expected) in [
        (
            r#"value=$(items=(case x in); printf '%s' "${items[0]}"); printf '%s' "$value""#,
            "case",
        ),
        (
            r#"value=$(items=(); printf '%s' "${#items[@]}"); printf '%s' "$value""#,
            "0",
        ),
        (
            r#"value=$(items=('case' "x" 'in' 'esac' ')'); printf '%s:%s' "${items[0]}" "${items[4]}"); printf '%s' "$value""#,
            "case:)",
        ),
        (
            r#"value=$(items=("$(case x in x) printf inner;; esac)" case in esac); printf '%s:%s' "${items[0]}" "${items[1]}"); printf '%s' "$value""#,
            "inner:case",
        ),
        (
            r#"value=$(case x in x) items=(case x in esac); printf '%s' "${items[0]}";; esac); printf '%s' "$value""#,
            "case",
        ),
        (
            r#"value=$(items=(case x in); case "${items[1]}" in x) printf real;; esac); printf '%s' "$value""#,
            "real",
        ),
        (
            "value=$(items=(# case esac ) ignored comment\ncase x in\n); case x in x) printf '%s' \"${items[0]}\";; esac); printf '%s' \"$value\"",
            "case",
        ),
        (
            r#"value=$(f() { items=(case x in); case x in x) printf '%s' "${items[0]}";; esac; }; f); printf '%s' "$value""#,
            "case",
        ),
    ] {
        for slice in [1, 16, 32, 64] {
            let result = sliced(source, &fixture(), slice);
            assert_eq!(result.error, None, "{source}");
            assert_eq!(result.stdout, expected, "{source}");
        }
    }
}

#[test]
fn static_case_brackets_fail_before_effects_without_changing_ordinary_test_words() {
    for pattern in ["[", "]"] {
        for source in [
            format!("printf changed > keep.txt; case x in {pattern}) :;; esac"),
            format!("printf changed > keep.txt; case good in good) :;; {pattern}) :;; esac"),
        ] {
            assert!(Vm::new(&source, &fixture()).is_err());
            let result = shell::run(&source, &fixture());
            assert_eq!(result.status, 2);
            assert!(
                result
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("Unsupported bracket case pattern"))
            );
            assert_eq!(result.files, fixture().files);
            assert!(result.stdout.is_empty());
        }
    }
    let ordinary = shell::run("if [ x = x ]; then printf yes; fi", &fixture());
    assert_eq!(ordinary.error, None);
    assert_eq!(ordinary.stdout, "yes");
    for source in [
        "case '[' in '[') printf literal;; esac",
        "case ']' in ']') printf literal;; esac",
    ] {
        let result = shell::run(source, &fixture());
        assert_eq!(result.error, None);
        assert_eq!(result.stdout, "literal");
    }
    for pattern in ["[", "]"] {
        let mut case = fixture();
        case.env.insert("p".into(), pattern.into());
        let result = shell::run(
            "case good in good) printf selected;; $p) printf wrong;; esac",
            &case,
        );
        assert_eq!(result.error, None);
        assert_eq!(result.stdout, "selected");
        let result = shell::run(
            "printf before > marker.txt; case x in $p) printf wrong > keep.txt;; esac",
            &case,
        );
        assert_eq!(result.status, 2);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("Unsupported"))
        );
        assert_eq!(
            result.files.get("/work/keep.txt"),
            case.files.get("/work/keep.txt")
        );
        assert_eq!(
            result.files.get("/work/marker.txt").map(String::as_str),
            Some("before")
        );
    }
}
#[test]
fn case_matching_rejects_non_c_locale_and_obeys_lc_all_precedence() {
    let source = "case é in ?) printf character;; *) printf bytes;; esac";
    for category in ["LC_ALL", "LC_COLLATE", "LC_CTYPE", "LANG"] {
        let mut case = fixture();
        case.env.insert(category.into(), "C.UTF-8".into());
        let result = shell::run(source, &case);
        assert_eq!(result.status, 2);
        assert_eq!(
            result.error.as_deref(),
            Some(
                "Unsupported case-matching locale; this simulator supports C/POSIX byte matching only"
            )
        );
        assert!(result.stdout.is_empty());
        assert_eq!(result.files, case.files);
    }
    for locale in ["C", "POSIX"] {
        let mut case = fixture();
        for category in ["LC_COLLATE", "LC_CTYPE", "LANG"] {
            case.env.insert(category.into(), "C.UTF-8".into());
        }
        case.env.insert("LC_ALL".into(), locale.into());
        let result = shell::run(source, &case);
        assert_eq!(result.error, None);
        assert_eq!(result.stdout, "bytes");
    }
    let mut case = fixture();
    case.env.insert("LC_ALL".into(), String::new());
    case.env.insert("LANG".into(), "C.UTF-8".into());
    assert!(
        shell::run(source, &case)
            .error
            .as_deref()
            .is_some_and(|error| error.contains("Unsupported case-matching locale"))
    );
    // Neither skipped cases nor an empty case perform a pattern match.
    assert_eq!(
        shell::run("false && case x in x) :;; esac", &case).error,
        None
    );
    assert_eq!(shell::run("case x in esac", &case).error, None);
}
#[test]
fn locale_guard_keeps_prior_selector_and_pattern_capture_effects_but_never_enters_a_body() {
    let mut case = fixture();
    case.env.insert("LC_ALL".into(), "C.UTF-8".into());
    let source = r#"case "$(printf selector > selector.txt; printf x)" in "$(printf pattern > pattern.txt; printf x)") printf wrong > keep.txt;; esac"#;
    for slice in [1, 16, 32, 64] {
        let result = sliced(source, &case, slice);
        assert_eq!(result.status, 2);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("Unsupported case-matching locale"))
        );
        assert_eq!(
            result.files.get("/work/keep.txt"),
            case.files.get("/work/keep.txt")
        );
        assert_eq!(
            result.files.get("/work/selector.txt").map(String::as_str),
            Some("selector")
        );
        assert_eq!(
            result.files.get("/work/pattern.txt").map(String::as_str),
            Some("pattern")
        );
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn reserved_pipeline_heads_are_static_only_when_first_bare_literal_words() {
    let result = shell::run("printf case; printf '%s' esac", &fixture());
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "caseesac");
    for keyword in ["case", "esac"] {
        for head in [
            format!("\"{keyword}\""),
            format!("\\{keyword}"),
            "\"$head\"".into(),
        ] {
            let source = format!("head={keyword}; printf changed > keep.txt; printf x | {head} x");
            assert!(
                Vm::new(&source, &fixture()).is_ok(),
                "quoted/escaped/expanded head is runtime data: {source}"
            );
            let result = shell::run(&source, &fixture());
            assert_eq!(result.status, 2);
            assert!(result.error.is_some());
            assert_eq!(
                result.files.get("/work/keep.txt").map(String::as_str),
                Some("changed"),
                "prior command must execute: {source}"
            );
        }
    }
    // A leading assignment-looking word is not the bare reserved head. Existing
    // command preparation owns its support/error timing; this patch does not
    // reinterpret the second word as compound syntax.
    assert!(Vm::new("tag=value case x", &fixture()).is_ok());
    assert!(Vm::new("tag=value esac", &fixture()).is_ok());
}
