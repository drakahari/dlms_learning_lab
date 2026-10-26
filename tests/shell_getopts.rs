//! Current pure-VM comparisons with retained actual55 Bash reference outputs;
//! no historical evaluator result is relabeled as current.
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn case() -> Case {
    Case {
        name: "getopts controlled fixture".into(),
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
fn sliced(source: &str, fixture: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, fixture).unwrap();
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded VM did not finish");
}
#[derive(Deserialize)]
struct Inputs {
    args: Vec<String>,
    stdin: String,
    env: BTreeMap<String, String>,
    files: BTreeMap<String, String>,
}
#[derive(Deserialize)]
struct Actual {
    stdout: String,
    stderr: String,
    status: i32,
    files: BTreeMap<String, String>,
}
#[derive(Deserialize)]
struct Observation {
    id: String,
    source: String,
    input: Inputs,
    actual_bash: Actual,
    observation_sha256: String,
    explicitly_excluded: bool,
    diagnostic_wording_variance: bool,
}
#[derive(Deserialize)]
struct Oracle {
    manifest_sha256: Vec<String>,
    observations: Vec<Observation>,
}
#[test]
fn all55_actual_bash_observations_have_explicit_supported_excluded_and_diagnostic_classification() {
    const {
        assert!(shell::EVALUATOR_REVISION >= 9);
    }
    let oracle: Oracle =
        serde_json::from_str(include_str!("fixtures/getopts-bash539-observations.json")).unwrap();
    assert_eq!(
        oracle.manifest_sha256,
        [
            "d6b5777260f45c9551f1df013c531cbd0547b3f234edfa369e913c8b49cca827",
            "f15f29582e1302622f444e2ae5c7df72fa2e6bc9dbcdaef6a55dd92276dea19a"
        ]
    );
    assert_eq!(oracle.observations.len(), 55);
    assert_eq!(
        oracle
            .observations
            .iter()
            .filter(|r| r.explicitly_excluded)
            .count(),
        1
    );
    assert_eq!(
        oracle
            .observations
            .iter()
            .filter(|r| r.diagnostic_wording_variance)
            .count(),
        2
    );
    for row in oracle.observations {
        assert_eq!(row.observation_sha256.len(), 64);
        let fixture = Case {
            args: row.input.args,
            stdin: row.input.stdin,
            env: row.input.env,
            files: row.input.files,
            ..case()
        };
        for slice in [1, 8, 32, 256] {
            let result = sliced(&row.source, &fixture, slice);
            if row.explicitly_excluded {
                assert_eq!(row.id, "function-local-uninitialized");
                assert!(
                    result
                        .error
                        .as_deref()
                        .is_some_and(|s| s.contains("Unsupported local OPTIND"))
                );
                assert_eq!(result.status, 2);
                assert_eq!(result.files, fixture.files);
                continue;
            }
            assert_eq!(result.error, None, "{} / {slice}: {result:?}", row.id);
            assert_eq!(
                result.stdout, row.actual_bash.stdout,
                "{} / {slice}",
                row.id
            );
            assert_eq!(
                result.status, row.actual_bash.status,
                "{} / {slice}",
                row.id
            );
            assert_eq!(result.files, row.actual_bash.files, "{} / {slice}", row.id);
            if row.diagnostic_wording_variance {
                assert!(!row.actual_bash.stderr.is_empty());
                assert!(result.stderr.starts_with("getopts: "));
                assert_eq!(result.stderr.lines().count(), 1);
                assert!(result.stderr.len() < 128);
            } else {
                assert_eq!(
                    result.stderr, row.actual_bash.stderr,
                    "{} / {slice}",
                    row.id
                );
            }
        }
    }
}
#[test]
fn resets_exact_contents_and_scalar_destination_tail_work_without_consuming_stdin() {
    let mut fixture = case();
    fixture.stdin = "finite input\n".into();
    let result = shell::run(
        "opt=(old tail); args=(-ab); getopts ab opt \"${args[@]}\"; args=(-ab); getopts ab opt \"${args[@]}\"; printf '%s:%s:%s\\n' \"$opt\" \"${opt[1]}\" \"$OPTIND\"; OPTIND=1; getopts c: opt -c ''; printf '%s:%s:%s\\n' \"$opt\" \"${#OPTARG[@]}\" \"$OPTARG\"; read -r line; printf '%s' \"$line\"",
        &fixture,
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "b:tail:2\nc:1:\nfinite input");
    assert_eq!(result.files, fixture.files);
}
#[test]
fn malformed_declarations_aliases_seeks_and_changed_identity_fail_honestly() {
    for source in [
        "getopts a",
        "getopts aa opt -a",
        "getopts a:: opt -a",
        "getopts :? opt -a",
        "getopts é opt -a",
        "getopts a OPTIND -a",
        "getopts a OPTARG -a",
        "getopts a OPTERR -a",
        "getopts a IFS -a",
        "getopts a BASH_ENV -a",
        "getopts a 'opt[0]' -a",
        "OPTIND=0",
        "OPTIND=2",
        "OPTIND=01",
        "OPTIND=''",
        "OPTARG=x",
        "OPTIND=(1)",
        "OPTARG=(x)",
        "for OPTIND in 1; do :; done",
        "read -r OPTIND",
        "read -r OPTARG",
        "f() { local OPTIND; }; f",
        "f() { local OPTARG=1; }; f",
        "getopts ab opt -ab; getopts ab opt -ac",
        "getopts ab opt -ab; getopts a opt -ab",
        "OPTERR=text; getopts a opt -a",
        "OPTERR=(0 tail); getopts a opt -a",
        "getopts a opt -é",
    ] {
        let result = shell::run(source, &case());
        assert_eq!(result.status, 2, "{source}");
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Unsupported")),
            "{source}: {result:?}"
        );
        assert_eq!(result.files, case().files);
    }
    for key in ["OPTIND", "OPTARG"] {
        let mut fixture = case();
        fixture.env.insert(key.into(), "1".into());
        assert!(!fixture.valid());
    }
}
#[test]
fn ordinary_opterr_operations_survive_while_reached_getopts_has_a_narrow_admission_boundary() {
    let result = shell::run(
        "OPTERR=(left right); f() { local OPTERR=child; printf '%s' \"$OPTERR\"; }; f; printf ':%s:%s' \"${OPTERR[0]}\" \"${OPTERR[1]}\"; OPTERR=0; getopts a opt -z; printf ':%s' \"$opt\"",
        &case(),
    );
    // Scalar assignment preserves the ordinary array tail; parsing that array
    // is excluded even when its first element happens to be0.
    assert_eq!(result.stdout, "child:left:right");
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Unsupported OPTERR"))
    );
    let result = shell::run(
        "OPTERR=text; value=$(OPTERR=child; printf '%s' \"$OPTERR\"); printf '%s:%s' \"$value\" \"$OPTERR\"",
        &case(),
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "child:text");
}
#[test]
fn builtin_admission_does_not_replay_arguments_or_undo_prior_redirect_effects() {
    let result = shell::run(
        "head=getopts; \"$head\" a OPTIND \"$(printf x >> prep.txt; printf '%s' -a)\" > keep.txt; printf unreachable",
        &case(),
    );
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Unsupported"))
    );
    assert_eq!(result.files["/work/prep.txt"], "x");
    assert_eq!(result.files["/work/keep.txt"], "");
    assert!(result.stdout.is_empty());
}
#[test]
fn retained_argument_identity_and_optarg_copies_share_live_memory_budget() {
    let mut fixture = case();
    fixture.env.insert("pad".into(), "p".repeat(75_000));
    fixture.env.insert("value".into(), "v".repeat(50_000));
    assert!(fixture.valid());
    let result = shell::run(
        "getopts o: option -o \"$value\"; printf bad > keep.txt",
        &fixture,
    );
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("memory")),
        "{result:?}"
    );
    assert_eq!(result.files, fixture.files);
    fixture.env.insert("pad".into(), "p".repeat(10_000));
    fixture.env.insert("value".into(), "v".repeat(10_000));
    let result = shell::run(
        "getopts o: option -o \"$value\"; printf '%s' \"${#OPTARG[@]}\"",
        &fixture,
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "1");
}
#[test]
fn child_and_pipeline_duplicate_scanner_bytes_are_reserved_before_child_effects() {
    let mut fixture = case();
    fixture.env.insert("pad".into(), "p".repeat(30_000));
    fixture.env.insert("value".into(), "v".repeat(35_000));
    for suffix in [
        "child=$(printf bad > keep.txt)",
        "printf '' | printf bad > keep.txt",
    ] {
        let source = format!("getopts o: option -o \"$value\"; {suffix}");
        let result = shell::run(&source, &fixture);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("memory")),
            "{result:?}"
        );
        assert_eq!(result.files, fixture.files);
    }
    fixture.env.insert("pad".into(), "p".repeat(5_000));
    let result = shell::run(
        "getopts o: option -o \"$value\"; child=$(printf child); printf '%s' \"$child\"",
        &fixture,
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "child");
}
#[test]
fn resets_never_refresh_work_and_stop_discards_current_child_scanners() {
    let mut fixture = case();
    fixture.env.insert("value".into(), "x".repeat(25_000));
    let result = shell::run(
        "while true; do OPTIND=1; getopts a option -a \"$value\"; done",
        &fixture,
    );
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("work limit")),
        "{result:?}"
    );
    for slice in [1, 8, 32] {
        let mut vm = Vm::new(
            "pending=$(while true; do OPTIND=1; getopts ab opt -ab; done); printf bad > keep.txt",
            &case(),
        )
        .unwrap();
        vm.tick(slice);
        vm.stop();
        let stopped = vm.outcome.clone();
        vm.tick(1000);
        assert_eq!(vm.outcome, stopped);
        assert!(
            vm.outcome
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Stopped"))
        );
        assert_eq!(vm.outcome.files, case().files);
    }
}
#[test]
fn hidden_parent_unset_local_counts_when_capture_adds_a_getopts_output() {
    let prefix: String = (0..255).map(|i| format!("v{i}=; ")).collect();
    let source = format!(
        "{prefix} getopts ao: choice -o value -a; f() {{ local v0; child=$(getopts ao: replacement -o value -a; printf bad > keep.txt); }}; f"
    );
    for slice in [1, 32] {
        let result = sliced(&source, &case(), slice);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|e| e.contains("Variable limit")),
            "{result:?}"
        );
        assert_eq!(result.files, case().files);
    }
}

#[test]
fn ordinary_128_and_original_256_pass_without_getopts_while_257_still_fails() {
    for count in [128, 256] {
        let prefix: String = (0..count).map(|i| format!("v{i}=; ")).collect();
        let result = shell::run(&format!("{prefix} printf ordinary-boundary"), &case());
        assert_eq!(result.error, None, "{count}: {result:?}");
        assert_eq!(result.stdout, "ordinary-boundary");
    }
    let prefix: String = (0..257).map(|i| format!("v{i}=; ")).collect();
    let result = shell::run(&format!("{prefix} printf bad > keep.txt"), &case());
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|s| s.contains("Variable limit"))
    );
    assert_eq!(result.files, case().files);
}
