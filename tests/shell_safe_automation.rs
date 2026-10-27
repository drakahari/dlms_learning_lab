//! Prospective evaluator10 tests. Bash expectations are exact controlled actual
//! observations; no historical public9 Record/outcome is relabeled as current.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn fixture() -> Case {
    Case {
        name: "safe automation bounded fixture".into(),
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
    let mut vm = Vm::new(source, case).unwrap();
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded VM failed to finish");
}
#[derive(Deserialize)]
struct Input {
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
    input: Input,
    actual_bash: Actual,
    actual_bash_version: String,
    observation_sha256: String,
    excluded: bool,
    excluded_reason: Option<String>,
    unsupported_diagnostic: Option<String>,
}
#[derive(Deserialize)]
struct Oracle {
    schema_version: u32,
    capture_manifest_sha256: String,
    observations: Vec<Observation>,
}

#[test]
fn actual28_bash_observations_have_exact_supported_and_excluded_classification() {
    assert_eq!(shell::EVALUATOR_REVISION, 10);
    let oracle: Oracle = serde_json::from_str(include_str!(
        "fixtures/automation-bash539-observations.json"
    ))
    .unwrap();
    assert_eq!(oracle.schema_version, 1);
    assert_eq!(
        oracle.capture_manifest_sha256,
        "8987f4290cf58bdaa7bd7981f4cbeae9454fb4155bff47f4f254c0d8140aac00"
    );
    assert_eq!(oracle.observations.len(), 28);
    assert_eq!(
        oracle
            .observations
            .iter()
            .filter(|row| row.excluded)
            .count(),
        2
    );
    for row in oracle.observations {
        assert!(
            row.actual_bash_version
                .starts_with("GNU bash, version 5.3.9")
        );
        assert_eq!(row.observation_sha256.len(), 64);
        let case = Case {
            args: row.input.args,
            stdin: row.input.stdin,
            env: row.input.env,
            files: row.input.files,
            ..fixture()
        };
        for slice in [1, 8, 32, 256] {
            let actual = sliced(&row.source, &case, slice);
            if row.excluded {
                assert!(matches!(
                    row.id.as_str(),
                    "read-dynamic-empty-value" | "prefix-other-variable"
                ));
                assert_eq!(actual.status, 2, "{} / {slice}", row.id);
                assert!(
                    row.excluded_reason
                        .as_deref()
                        .is_some_and(|text| !text.is_empty())
                );
                assert_eq!(
                    actual.error.as_deref(),
                    row.unsupported_diagnostic.as_deref(),
                    "{} / {slice}",
                    row.id
                );
                assert_eq!(actual.files, case.files, "{} / {slice}", row.id);
            } else {
                assert!(row.excluded_reason.is_none() && row.unsupported_diagnostic.is_none());
                assert_eq!(actual.error, None, "{} / {slice}", row.id);
                assert_eq!(
                    actual.stdout, row.actual_bash.stdout,
                    "{} / {slice}",
                    row.id
                );
                assert_eq!(
                    actual.stderr, row.actual_bash.stderr,
                    "{} / {slice}",
                    row.id
                );
                assert_eq!(
                    actual.status, row.actual_bash.status,
                    "{} / {slice}",
                    row.id
                );
                assert_eq!(actual.files, row.actual_bash.files, "{} / {slice}", row.id);
            }
        }
    }
}
#[test]
fn empty_ifs_is_only_a_read_policy_and_preserves_partial_eof_utf8_cr_and_backslashes() {
    for (input, expected, status) in [
        ("  a\t \n", "  a\t ", 0),
        ("\n", "", 0),
        ("", "", 1),
        ("  final\t ", "  final\t ", 1),
        (" é\r\\% \n", " é\r\\% ", 0),
    ] {
        let case = Case {
            stdin: input.into(),
            ..fixture()
        };
        for prefix in ["IFS=", "IFS=''", "IFS=\"\"", "IFS=''\"\""] {
            let source = format!(
                "{prefix} 'read' -r row; rc=$?; printf '<%s>|%s|<%s>' \"$row\" \"$rc\" \"$IFS\""
            );
            let actual = sliced(&source, &case, 1);
            assert_eq!(actual.error, None);
            assert_eq!(actual.stdout, format!("<{expected}>|{status}|< \t\n>"));
        }
    }
}
#[test]
fn ordinary_read_trim_and_ifs_splitting_are_unchanged_after_temporary_read() {
    let case = Case {
        stdin: "  first  \n  second\t \n".into(),
        ..fixture()
    };
    let source = "IFS= read -r first; read -r second; pieces='left right'; printf '<%s>' \"$first\" \"$second\" $pieces \"$IFS\"";
    let actual = sliced(source, &case, 1);
    assert_eq!(actual.error, None);
    assert_eq!(actual.stdout, "<  first  ><second><left><right>< \t\n>");
}
#[test]
fn prefix_does_not_expose_empty_ifs_during_argument_or_redirect_captures() {
    let mut case = fixture();
    case.files
        .insert("/work/input.txt".into(), "  from disk  \n".into());
    let source = "IFS= read -r \"$(printf '%s' \"$IFS\" >&2; printf row)\" <\"$(printf '%s' \"$IFS\" >&2; printf input.txt)\"; printf '<%s>' \"$row\"";
    let actual = sliced(source, &case, 1);
    assert_eq!(actual.error, None);
    assert_eq!(actual.stderr, " \t\n \t\n");
    assert_eq!(actual.stdout, "<  from disk  >");
    assert_eq!(actual.files, case.files);
}
#[test]
fn scoped_read_validates_target_before_consumption_and_preserves_redirect_order() {
    for target in ["IFS", "OPTIND", "OPTARG", "UID"] {
        let case = Case {
            stdin: "  retained  \n".into(),
            ..fixture()
        };
        let actual = sliced(&format!("IFS= read -r {target} > created.txt"), &case, 1);
        assert_eq!(actual.status, 2);
        assert!(actual.error.is_some());
        assert_eq!(actual.files["/work/keep.txt"], "retained\n");
        // As for ordinary builtins, accepted output redirects occur first.
        assert_eq!(actual.files["/work/created.txt"], "");
    }
    let case = Case {
        stdin: "  caller  \n".into(),
        ..fixture()
    };
    let actual = sliced(
        "IFS= read -r row < missing.txt > keep.txt; read -r row; printf '<%s>' \"$row\"",
        &case,
        1,
    );
    assert_eq!(actual.error, None);
    assert_eq!(actual.stdout, "<caller>");
    assert_eq!(actual.files, case.files);
}
#[test]
fn unsupported_ifs_prefixes_and_general_assignments_do_not_become_environment_support() {
    for source in [
        "IFS=: read -r row",
        "empty=; IFS=$empty read -r row",
        "IFS=\"$IFS\" read -r row",
        "IFS= printf text",
        "IFS=read read -r row",
        "cmd=read; IFS= \"$cmd\" -r row",
        "IFS= read row",
        "IFS= read -r a b",
        "IFS= read -a row",
        "IFS=",
        "local IFS=",
        "other=child printf text",
    ] {
        let actual = sliced(source, &fixture(), 1);
        assert_eq!(actual.status, 2, "{source}");
        assert!(actual.error.is_some(), "{source}");
        assert_eq!(actual.files, fixture().files);
    }
}
#[test]
fn rightmost_nonzero_uses_pipeline_entry_option_and_ordinary_functions_share_it() {
    for (source, expected) in [
        ("false | true; printf '%s' \"$?\"", "0"),
        (
            "set -o pipefail; exit 3 | exit 7 | true; printf '%s' \"$?\"",
            "7",
        ),
        (
            "set -o pipefail; exit 3 | true | exit 5; printf '%s' \"$?\"",
            "5",
        ),
        ("set -o pipefail; true | true; printf '%s' \"$?\"", "0"),
        (
            "set -o pipefail; false | set +o pipefail; printf '%s' \"$?\"; false | true; printf '%s' \"$?\"",
            "11",
        ),
        (
            "false | set -o pipefail; printf '%s' \"$?\"; false | true; printf '%s' \"$?\"",
            "00",
        ),
        (
            "enable_checks() { set -o pipefail; }; enable_checks; false | true; printf '%s' \"$?\"",
            "1",
        ),
        (
            "set -o pipefail; disable_checks() { set +o pipefail; }; disable_checks; false | true; printf '%s' \"$?\"",
            "0",
        ),
    ] {
        for slice in [1, 8, 32, 256] {
            let actual = sliced(source, &fixture(), slice);
            assert_eq!(actual.error, None, "{source}");
            assert_eq!(actual.stdout, expected, "{source} / {slice}");
        }
    }
}
#[test]
fn capture_options_inherit_but_never_leak_and_pipeline_children_start_from_parent() {
    for (source, expected) in [
        (
            "set -o pipefail; text=$(false | true); printf '<%s>:%s' \"$text\" \"$?\"",
            "<>:1",
        ),
        (
            "set -o pipefail; text=$(set +o pipefail; false | true; printf '%s' \"$?\"); false | true; printf '<%s>:%s' \"$text\" \"$?\"",
            "<0>:1",
        ),
        (
            "text=$(set -o pipefail; false | true; printf '%s' \"$?\"); false | true; printf '<%s>:%s' \"$text\" \"$?\"",
            "<1>:0",
        ),
        (
            "set -o pipefail; set +o pipefail | printf '%s' \"$(false | true; printf '%s' \"$?\")\"",
            "1",
        ),
    ] {
        let actual = sliced(source, &fixture(), 1);
        assert_eq!(actual.error, None, "{source}");
        assert_eq!(actual.stdout, expected, "{source}");
    }
}
#[test]
fn fatal_unsupported_or_resource_errors_never_become_ordinary_handled_statuses() {
    for source in [
        "set -o pipefail; missing | true || printf handled",
        "set +o pipefail; missing | true || printf handled",
        "set -o pipefail; printf '%999s' x | true || printf handled",
    ] {
        let actual = sliced(source, &fixture(), 1);
        assert_eq!(actual.status, 2);
        assert!(actual.error.is_some());
        assert!(!actual.stdout.contains("handled"));
    }
    let actual = sliced(
        "set -o pipefail; false | true && printf wrong || printf handled",
        &fixture(),
        1,
    );
    assert_eq!(actual.error, None);
    assert_eq!(actual.stdout, "handled");
}
#[test]
fn other_set_forms_are_rejected_without_options_or_status_arrays_becoming_variables() {
    for source in [
        "set",
        "set -o",
        "set +o",
        "set -e",
        "set -u",
        "set -o errexit",
        "set -- a",
        "set -o pipefail extra",
        "set pipefail",
        "printf '%s' \"$SHELLOPTS\"",
        "printf '%s' \"${PIPESTATUS[0]}\"",
    ] {
        let actual = sliced(source, &fixture(), 1);
        assert_eq!(actual.status, 2, "{source}");
        assert!(actual.error.is_some(), "{source}");
    }
}
#[test]
fn sixteen_stages_are_supported_and_seventeen_reject_before_effects() {
    let allowed = format!(
        "set -o pipefail; exit 9 | {}; printf '%s' \"$?\"",
        vec!["true"; 15].join(" | ")
    );
    let actual = sliced(&allowed, &fixture(), 1);
    assert_eq!(actual.error, None);
    assert_eq!(actual.stdout, "9");
    let rejected = format!(
        "printf changed > keep.txt; {}",
        vec!["true"; 17].join(" | ")
    );
    assert!(Vm::new(&rejected, &fixture()).is_err());
    assert_eq!(shell::run(&rejected, &fixture()).files, fixture().files);
}
#[test]
fn while_input_frame_and_nested_function_read_modes_preserve_cursors() {
    let mut case = Case {
        stdin: "  parent  \n".into(),
        ..fixture()
    };
    case.files
        .insert("/work/input.txt".into(), "  first  \n\tsecond\t\n".into());
    let source = "consume() { while IFS= read -r row; do printf '<%s>' \"$row\"; done < input.txt; }; consume; read -r row; printf '<%s>' \"$row\"";
    let actual = sliced(source, &case, 1);
    assert_eq!(actual.error, None);
    assert_eq!(actual.stdout, "<  first  ><\tsecond\t><parent>");
    assert_eq!(actual.files, case.files);
}
#[test]
fn stop_is_cooperative_during_read_preparation_capture_and_pipefail_execution() {
    let case = Case {
        stdin: "  input  \n".into(),
        ..fixture()
    };
    for source in [
        "IFS= read -r \"$(while true; do :; done)\" > keep.txt",
        "set -o pipefail; while true; do false | true; done",
    ] {
        let mut vm = Vm::new(source, &case).unwrap();
        vm.tick(1);
        vm.tick(1);
        assert!(!vm.done());
        vm.stop();
        assert!(vm.done());
        assert_eq!(vm.outcome.status, 2);
        assert!(
            vm.outcome
                .error
                .as_deref()
                .is_some_and(|s| s.contains("Stopped"))
        );
        assert_eq!(vm.outcome.files, case.files);
    }
}
#[test]
fn output_instruction_and_memory_limits_remain_shared_across_new_forms() {
    let case = fixture();
    for source in [
        "set -o pipefail; while true; do false | true; done",
        "while true; do IFS= read -r row; printf '%s' 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done",
    ] {
        let actual = sliced(source, &case, 256);
        assert_eq!(actual.status, 2);
        assert!(actual.error.is_some());
        assert!(actual.stdout.len() <= 32_768);
        assert_eq!(actual.files, case.files);
    }
    // Distinct argument copies still share the pre-existing 256KiB memory cap.
    let case = Case {
        env: BTreeMap::from([("large".into(), "x".repeat(100_000))]),
        ..fixture()
    };
    let actual = sliced(
        "set -o pipefail; printf '%s%s%s' \"$large\" \"$large\" \"$large\" | true",
        &case,
        1,
    );
    assert_eq!(actual.status, 2);
    assert!(actual.error.is_some());
    assert_eq!(actual.files, case.files);
}

#[test]
fn all43_existing_scripts282_cases_and43_examples_keep_exact_authored_inputs_and_reachability() {
    use dlms_learning_lab::{engine::Goal, pack};
    let authored: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/automation-before10-authored-inputs.json"
    ))
    .unwrap();
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 147);
    let labs = &current[..133];
    let catalog = labs
        .iter()
        .map(|s| (s.id.clone(), s.version))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(serde_json::to_value(catalog).unwrap(), authored["catalog"]);
    let scripts = authored["scripts"].as_array().unwrap();
    assert_eq!(scripts.len(), 43);
    let mut cases_seen = 0;
    let mut examples_seen = 0;
    for old in scripts {
        let id = old["id"].as_str().unwrap();
        let lab = labs.iter().find(|s| s.id == id).unwrap();
        let Goal::Script { cases } = &lab.goal else {
            panic!("Wrong original script kind")
        };
        assert_eq!(
            serde_json::to_value(cases).unwrap(),
            old["cases"],
            "authored case inputs/expectations changed {id}"
        );
        assert_eq!(lab.version, old["version"].as_u64().unwrap() as u32);
        assert_eq!(
            lab.solution.join("\n"),
            old["source"].as_str().unwrap(),
            "source {id}"
        );
        let lesson = lab.teaching.as_ref().unwrap();
        assert_eq!(
            teaching_amendment::historical_example(&lab.id, lesson).unwrap(),
            old["example_source"].as_str().unwrap()
        );
        assert_eq!(
            serde_json::to_value(&lesson.fixture).unwrap(),
            old["example_case"]
        );
        for (i, case) in cases.iter().enumerate() {
            let actual = sliced(&lab.solution.join("\n"), case, 32);
            assert!(
                case.accepts(&actual),
                "Explicit current evaluator reference failed {id}/{i}: {:?}",
                actual.error
            );
            cases_seen += 1;
        }
        let actual = sliced(&lesson.example, &lesson.fixture, 32);
        assert!(
            lesson.fixture.accepts(&actual),
            "Explicit separate current example failed {id}: {:?}",
            actual.error
        );
        examples_seen += 1;
    }
    assert_eq!(cases_seen, 282);
    assert_eq!(examples_seen, 43);
}
