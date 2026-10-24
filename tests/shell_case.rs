//! Original bounded-case regression cases, plus retained actual fixed Bash5.3.9
//! observations. No test invokes a host shell, account tool, process or network.
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
    assert_eq!(shell::EVALUATOR_REVISION, 8); // isolated runtime; migration remains a separate gate
    for observation in oracle.observations {
        assert_eq!(observation.observation_sha256.len(), 64);
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
        "case x in [ab]) :;; esac",
        "case x in x) :;; esac > keep.txt",
        "case x in x) :;; esac | cat",
        "printf x | case x in x) :;; esac",
        "case x in x) :;; esac;; printf bad",
        "if true; then case x in esac; else fi",
        "esac",
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
    for pattern in ["[ab]", "\\*", "@(a|b)"] {
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
