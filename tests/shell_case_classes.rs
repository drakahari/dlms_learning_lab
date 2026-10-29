//! Bounded case-class capability tests. Authored expectations, no fabricated
//! reference observations; real Bash comparison is a separate controlled gate.
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn fixture() -> Case {
    Case {
        name: "Case-class control".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "untouched\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn sliced(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap_or_else(|e| panic!("{source}: {e}"));
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Bounded VM did not finish");
}
#[derive(Deserialize)]
struct Fixed {
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    id: String,
    source: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    proposed_stdout: String,
}

#[test]
fn fixed48_authored_class_controls_are_slice_independent_and_preserve_files() {
    assert_eq!(shell::EVALUATOR_REVISION, 12);
    let fixed: Fixed =
        serde_json::from_str(include_str!("fixtures/shell-case-class-inputs-v12.json")).unwrap();
    assert_eq!(fixed.rows.len(), 48);
    for row in fixed.rows {
        let mut case = fixture();
        case.args = row.args;
        case.env = row.env;
        for slice in [1, 7, 32, 64] {
            let result = sliced(&row.source, &case, slice);
            assert_eq!(result.error, None, "{} / {slice}: {result:?}", row.id);
            assert_eq!(result.stdout, row.proposed_stdout, "{} / {slice}", row.id);
            assert_eq!(result.stderr, "");
            assert_eq!(result.status, 0);
            assert_eq!(result.files, case.files);
        }
    }
}

#[test]
fn exact_former_class_negative_premises_now_have_explicit_whole_outcome_controls() {
    for source in [
        "printf changed > keep.txt; case x in [ab]) :;; esac",
        "printf changed > keep.txt; case good in good) :;; [ab]) :;; esac",
    ] {
        let case = fixture();
        let out = shell::run(source, &case);
        assert_eq!(out.error, None);
        assert_eq!(
            (out.stdout.as_str(), out.stderr.as_str(), out.status),
            ("", "", 0)
        );
        assert_eq!(
            out.files,
            BTreeMap::from([("/work/keep.txt".into(), "changed".into())])
        );
    }
    let mut case = fixture();
    case.env.insert("p".into(), "[ab]".into());
    let out = shell::run(
        "printf before > before.txt; case x in $p) printf no;; esac",
        &case,
    );
    assert_eq!(out.error, None);
    assert_eq!(
        (out.stdout.as_str(), out.stderr.as_str(), out.status),
        ("", "", 0)
    );
    let mut expected = case.files.clone();
    expected.insert("/work/before.txt".into(), "before".into());
    assert_eq!(out.files, expected);
    case.env.insert("subject".into(), "[ab]".into());
    let out = shell::run("case $subject in \"$p\") printf yes;; esac", &case);
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "yes");
    assert_eq!(out.files, case.files);
}

const GUARDED: &str = r#"
bad() { printf 'Invalid request\n' >&2; exit 2; }
canonical() {
    case "$1" in ''|*[!0-9]*|0?*|???????????????????*) return 2 ;; esac
    return 0
}
if test "$#" -ne 2; then bad; fi
if canonical "$1" && canonical "$2"; then :; else bad; fi
if test "$1" -lt 1 || test "$1" -gt 5000 || test "$2" -gt 10000; then bad; fi
printf 'Total: %s\n' "$(printf '%s' "$(( $1 * $2 ))")"
"#;
const POSITIVE_LAYOUT: &str = r#"
bad() { printf 'Invalid request\n' >&2; exit 2; }
bounded_digits() {
    case "$1" in 0|[1-9]|[1-9][0-9]|[1-9][0-9][0-9]|[1-9][0-9][0-9][0-9]|10000) return 0 ;; *) return 2 ;; esac
}
if test "$#" -ne 2; then bad; fi
if bounded_digits "$1"; then :; else bad; fi
if bounded_digits "$2"; then :; else bad; fi
if test "$1" -ge 1 && test "$1" -le 5000 && test "$2" -le 10000; then
    total=$(( $1 * $2 )); printf 'Total: %s\n' "$total"
else bad; fi
"#;

#[test]
fn two_different_numeric_guards_agree_and_wrong_partial_or_hardcoded_programs_fail() {
    let mut cases = Vec::new();
    for (a, b, total) in [
        ("6", "7", 42),
        ("1", "0", 0),
        ("5000", "10000", 50_000_000),
        ("19", "23", 437),
    ] {
        let mut c = fixture();
        c.args = vec![a.into(), b.into()];
        c.stdout = format!("Total: {total}\n");
        c.after = c.files.clone();
        cases.push(c);
    }
    for args in [
        vec![],
        vec!["3"],
        vec!["2", "3", "extra"],
        vec!["0", "3"],
        vec!["5001", "2"],
        vec!["1", "10001"],
        vec!["08", "2"],
        vec!["01", "2"],
        vec!["2", "00"],
        vec!["", "4"],
        vec!["1x", "4"],
        vec!["3", "+4"],
        vec!["-1", "4"],
        vec!["1 2", "4"],
        vec!["1\n2", "4"],
        vec!["é", "4"],
        vec!["999999999999999999", "2"],
        vec!["9223372036854775808", "1"],
        vec!["18446744073709551615", "2"],
    ] {
        let mut c = fixture();
        c.args = args.into_iter().map(String::from).collect();
        c.stderr = "Invalid request\n".into();
        c.status = 2;
        c.after = c.files.clone();
        cases.push(c);
    }
    let wrong = [GUARDED.replace("|0?*", ""), GUARDED.replace("*[!0-9]*|", ""),
        GUARDED.replace("test \"$1\" -gt 5000", "false"),
        GUARDED.replace("printf 'Total: %s\\n'", "printf 'Extra\\n'; printf 'Total: %s\\n'"),
        format!("printf changed > keep.txt; {GUARDED}"), "printf 'Total: 42\\n'".into(),
        "case \"$1\" in [0-9]*) printf 'Total: %s\\n' \"$(( $1 * $2 ))\" ;; *) printf 'Invalid request\\n' >&2; exit 2 ;; esac".into()];
    let mut rejected = vec![false; wrong.len()];
    for case in &cases {
        for source in [GUARDED, POSITIVE_LAYOUT] {
            for slice in [1, 7, 32, 64] {
                let out = sliced(source, case, slice);
                assert!(case.accepts(&out), "{:?}: {out:?}", case.args);
            }
        }
        for (n, source) in wrong.iter().enumerate() {
            rejected[n] |= !case.accepts(&shell::run(source, case));
        }
    }
    assert!(
        rejected.into_iter().all(|r| r),
        "Every incorrect policy/hardcode/write-before-validation is detected"
    );
}

#[test]
fn classes_follow_c_bytes_not_unicode_characters_or_filename_rules() {
    for (pattern, expected) in [
        ("?", false),
        ("??", true),
        ("[!0-9]", false),
        ("[!0-9][!0-9]", true),
        ("*[!0-9]*", true),
        ("'é'", true),
        ("[a-z]", false),
    ] {
        let source = format!("case \"$1\" in {pattern}) printf match;; *) printf miss;; esac");
        let mut case = fixture();
        case.args = vec!["é".into()];
        let out = shell::run(&source, &case);
        assert_eq!(out.error, None);
        assert_eq!(out.stdout, if expected { "match" } else { "miss" });
    }
    let mut case = fixture();
    case.env.insert("p".into(), "[0-9]".into());
    case.args = vec!["7".into()];
    assert_eq!(
        shell::run("case \"$1\" in $p) printf yes;; esac", &case).stdout,
        "yes"
    );
    case.args = vec!["[0-9]".into()];
    assert_eq!(
        shell::run("case \"$1\" in \"$p\") printf literal;; esac", &case).stdout,
        "literal"
    );
    assert!(
        shell::run("case é in [é]) printf wrong;; esac", &case)
            .error
            .unwrap()
            .contains("Unsupported non-ASCII")
    );
    case.args = vec![".hidden/a\nb".into()];
    assert_eq!(
        shell::run("case \"$1\" in [!0-9]*) printf yes;; esac", &case).stdout,
        "yes"
    );
}

#[test]
fn first_last_hyphens_leading_close_and_quoted_metacharacters_have_literal_members() {
    for (pattern, yes, no) in [
        ("[-az]", "-", "b"),
        ("[az-]", "-", "b"),
        ("[]a]", "]", "b"),
        ("[]-]", "-", "a"),
        ("[!]]", "a", "]"),
        ("[!-]", "a", "-"),
        ("[!^]", "a", "^"),
        ("[!!]", "a", "!"),
        ("['^']", "^", "a"),
        ("[a'-'z]", "-", "m"),
        ("['[']", "[", "a"),
        ("[0-9A-F]", "C", "G"),
    ] {
        let source = format!("case \"$1\" in {pattern}) printf yes;; *) printf no;; esac");
        for (value, expected) in [(yes, "yes"), (no, "no")] {
            let mut case = fixture();
            case.args = vec![value.into()];
            let out = shell::run(&source, &case);
            assert_eq!(out.error, None, "{pattern}: {out:?}");
            assert_eq!(out.stdout, expected, "{pattern}/{value}");
        }
    }
}

#[test]
fn excluded_static_patterns_fail_before_effects_and_dynamic_ones_remain_lazy() {
    for p in [
        "[",
        "]",
        "[]",
        "[!]",
        "[z-a]",
        "[9-0]",
        "[--a]",
        "[a--]",
        "[[:digit:]]",
        "[[.a.]]",
        "[[=a=]]",
        "[^a]",
        "[é]",
    ] {
        for source in [
            format!("printf changed > keep.txt; case a in {p}) printf bad;; esac"),
            format!("printf changed > keep.txt; case good in good) :;; {p}) printf bad;; esac"),
        ] {
            assert!(Vm::new(&source, &fixture()).is_err(), "{p}");
            let out = shell::run(&source, &fixture());
            assert_eq!(out.status, 2);
            assert!(out.error.unwrap().contains("Unsupported"));
            assert_eq!(out.files, fixture().files);
            assert!(out.stdout.is_empty());
        }
        let mut case = fixture();
        case.env.insert("p".into(), p.into());
        let out = shell::run(
            "case good in good) printf yes;; $p) printf bad;; esac",
            &case,
        );
        assert_eq!(out.error, None);
        assert_eq!(out.stdout, "yes");
        let out = shell::run(
            "printf before > marker.txt; case a in $p) printf changed > keep.txt;; esac",
            &case,
        );
        assert_eq!(out.status, 2);
        assert!(out.error.unwrap().contains("Unsupported"));
        assert_eq!(out.files["/work/keep.txt"], "untouched\n");
        assert_eq!(out.files["/work/marker.txt"], "before");
    }
}

#[test]
fn source_looking_data_never_gains_command_or_network_semantics() {
    for value in [
        "$(touch /outside)",
        "`command`",
        "../../personal",
        "https://host/",
        "; printf hacked",
        "1*",
        "1?",
        "[0-9]",
    ] {
        let mut case = fixture();
        case.args = vec![value.into()];
        let out = shell::run(
            "case \"$1\" in ''|*[!0-9]*) printf rejected;; *) printf number;; esac",
            &case,
        );
        assert_eq!(out.error, None);
        assert_eq!(out.stdout, "rejected");
        assert_eq!(out.files, case.files);
    }
}

#[test]
fn class_support_does_not_leak_into_filename_parameter_or_array_namespaces() {
    for source in [
        "printf '%s' [0-9]*",
        "printf '%s' \"${value:-[0-9]}\"",
        "items=(one); printf '%s' \"${items[x]}\"",
    ] {
        let out = shell::run(source, &fixture());
        assert_eq!(out.status, 2);
        assert!(out.error.unwrap().contains("Unsupported") || Vm::new(source, &fixture()).is_err());
        assert_eq!(out.files, fixture().files);
    }
    let out = shell::run(
        "items=(7); case ${items[0]} in [0-9]) printf '%s' \"${#items[@]}\";; esac",
        &fixture(),
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "1");
}

#[test]
fn capture_quote_masks_status_order_and_virtual_effects_remain_owned_by_case() {
    let out = shell::run(
        "false; case 7 in [0-9]) printf '%s' \"$?\";; esac",
        &fixture(),
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "1");
    let out = shell::run(
        "false; case \"$(printf 7; false)\" in $(printf '[0-9]'; true)) printf '%s' \"$?\";; esac",
        &fixture(),
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "0");
    let out = shell::run(
        "case 7 in \"$(printf '[0-9]')\") printf wrong;; *) printf miss;; esac",
        &fixture(),
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "miss");
    let out = shell::run(
        "case 7 in $(printf prepared > marker.txt; printf '[0-9]')) printf selected;; esac",
        &fixture(),
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "selected");
    assert_eq!(out.files["/work/marker.txt"], "prepared");
    assert_eq!(out.files["/work/keep.txt"], "untouched\n");
}

#[test]
fn locale_guard_and_scalar_context_keep_their_existing_timing() {
    let source = "printf before > marker.txt; case 7 in [0-9]) printf changed > keep.txt;; esac";
    for locale in ["en_US.UTF-8", "C.UTF-8"] {
        let mut case = fixture();
        case.env.insert("LC_ALL".into(), locale.into());
        let out = shell::run(source, &case);
        assert!(out.error.unwrap().contains("locale"));
        assert_eq!(out.files["/work/marker.txt"], "before");
        assert_eq!(out.files["/work/keep.txt"], "untouched\n");
    }
    let mut case = fixture();
    case.env.insert("LC_ALL".into(), "C".into());
    case.env.insert("LANG".into(), "en_US.UTF-8".into());
    assert_eq!(
        shell::run("case 7 in [0-9]) printf yes;; esac", &case).stdout,
        "yes"
    );
    case.env.insert("LC_ALL".into(), "en_US.UTF-8".into());
    assert_eq!(
        shell::run("case 7 in esac; printf empty", &case).stdout,
        "empty"
    );
}

#[test]
fn class_backtracking_yields_stop_drops_pending_and_later_ticks_cannot_run_body() {
    let mut case = fixture();
    case.env.insert("subject".into(), "a".repeat(14_000));
    case.env
        .insert("pattern".into(), format!("*{}b", "[a]".repeat(80)));
    let source = "case $subject in $pattern) printf changed > keep.txt;; *) printf wrong;; esac";
    let mut vm = Vm::new(source, &case).unwrap();
    for _ in 0..20 {
        vm.tick(1);
    }
    assert!(!vm.done());
    vm.stop();
    let stopped = vm.outcome.clone();
    for _ in 0..20 {
        vm.tick(64);
    }
    assert_eq!(vm.outcome, stopped);
    assert!(stopped.error.is_some());
    assert_eq!(stopped.files, case.files);
    assert!(stopped.stdout.is_empty());
}

#[test]
fn class_pattern_length_and_expansion_work_fail_honestly_without_fallback_success() {
    let mut case = fixture();
    case.env.insert("subject".into(), "a".repeat(14_000));
    case.env
        .insert("pattern".into(), format!("*{}b", "[a]".repeat(80)));
    let source = "case $subject in $pattern) printf wrong;; *) printf fallback;; esac";
    for slice in [1, 16, 64] {
        let out = sliced(source, &case, slice);
        assert!(out.error.unwrap().contains("Expansion work limit"));
        assert_eq!(out.status, 2);
        assert!(out.stdout.is_empty());
        assert_eq!(out.files, case.files);
    }
    case.env.insert("pattern".into(), "[a]".repeat(86));
    let out = shell::run(source, &case);
    assert!(out.error.unwrap().contains("256 bytes"));
    assert_eq!(out.files, case.files);
    assert!(out.stdout.is_empty());
}
