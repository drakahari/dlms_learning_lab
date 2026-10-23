use dlms_learning_lab::shell::{self, Case, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;

fn case() -> Case {
    Case {
        name: "Substitution runtime fixture".into(),
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
fn excluded_or_malformed_nested_syntax_fails_before_any_parent_mutation() {
    assert_eq!(shell::EVALUATOR_REVISION, 7);
    for source in [
        "printf changed > keep.txt; value=$(printf unfinished",
        "printf changed > keep.txt; value=$((1 + $(printf 2)))",
        "printf changed > keep.txt; value=$(< keep.txt)",
        "printf changed > keep.txt; value=`printf text`",
    ] {
        assert!(Vm::new(source, &case()).is_err());
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
fn all_selected_reference_cases_execute_through_public_revision_seven() {
    let cases: Vec<Reference> =
        serde_json::from_str(include_str!("fixtures/shell-substitution-reference.json")).unwrap();
    assert_eq!(cases.len(), 62);
    for fixture in cases {
        for slice in [1, 16, 32, 64] {
            let result = sliced(&fixture.source, &fixture.case, slice);
            assert!(
                fixture.case.accepts(&result),
                "{} / {slice}: {result:?}",
                fixture.name
            );
        }
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

fn sliced(source: &str, fixture: &Case, slice: usize) -> shell::Outcome {
    let mut vm = Vm::new(source, fixture).unwrap();
    for _ in 0..20_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded fixture failed to finish with slice {slice}");
}
#[test]
fn phased_consumers_preserve_results_across_tick_slices() {
    let mut fixture = case();
    fixture.stdin = "parent\nremaining\n".into();
    fixture
        .files
        .insert("/work/source.txt".into(), "alpha\nbeta\n".into());
    for source in [
        "first=one second=$first; printf '%s:%s' \"$first\" \"$second\"",
        "label=outer; f() { local label=inner prior=$label; printf '%s:%s' \"$label\" \"$prior\"; }; head=f; \"$head\"",
        "parts=('one two' '*.txt'); for label in \"${parts[@]}\"; do printf '<%s>' \"$label\"; done",
        "parts=(*.txt); printf '<%s>' \"${parts[@]}\"",
        "while read -r line; do printf '<%s>' \"$line\"; done < source.txt; read -r line; printf '<%s>' \"$line\"",
        "read -r line < source.txt < keep.txt; printf '<%s>' \"$line\"; read -r line; printf '<%s>' \"$line\"",
        "read -r first | uniq; read -r second; printf '<%s>' \"$second\"",
        "printf first 2> errors.txt > out.txt; cat out.txt | uniq",
    ] {
        let expected = shell::run(source, &fixture);
        assert_eq!(expected.error, None, "{source}");
        for slice in [1, 2, 16, 64] {
            assert_eq!(
                sliced(source, &fixture, slice),
                expected,
                "{source}, slice {slice}"
            );
        }
    }
}
#[test]
fn missing_input_stops_later_redirect_effects_and_restores_caller_input() {
    let mut fixture = case();
    fixture.stdin = "parent\n".into();
    fixture
        .files
        .insert("/work/source.txt".into(), "source\n".into());
    let source = "read -r value < source.txt < missing.txt > keep.txt; read -r value; printf '%s' \"$value\"";
    let outcome = sliced(source, &fixture, 1);
    assert_eq!(outcome.error, None);
    assert_eq!(outcome.stdout, "parent");
    assert_eq!(outcome.stderr, "shell: missing.txt: No such file\n");
    assert_eq!(outcome.files, fixture.files);
}
#[test]
fn stop_discards_a_large_pending_preparation_without_applying_its_redirect() {
    let source = format!("printf '%s' {} > keep.txt", "'value' ".repeat(100));
    let fixture = case();
    let mut vm = Vm::new(&source, &fixture).unwrap();
    vm.tick(1); // Schedule the command, without executing its body.
    vm.tick(1); // Prepare only a bounded prefix of the argument list.
    assert!(!vm.done());
    assert_eq!(vm.outcome.files, fixture.files);
    assert!(vm.outcome.stdout.is_empty());
    vm.stop();
    let stopped = vm.outcome.clone();
    vm.tick(64);
    assert_eq!(vm.outcome, stopped);
    assert_eq!(vm.outcome.status, 2);
    assert!(vm.outcome.error.as_deref().unwrap().contains("Stopped"));
}
#[test]
fn pipeline_context_checks_do_not_speculatively_expand_a_later_stage() {
    let result = shell::run(
        "render() { :; }; printf retained > stage.txt | render | printf '%s' \"$((1 + $(printf 0)))\"",
        &case(),
    );
    // Excluded arithmetic positions are detected in whole-source compilation.
    assert!(result.error.as_deref().unwrap().contains("arithmetic"));
    assert_eq!(result.files, case().files);
    let result = shell::run(
        "render() { :; }; printf retained > stage.txt | render | printf '%s' \"$RANDOM\"",
        &case(),
    );
    assert!(
        result
            .error
            .as_deref()
            .unwrap()
            .contains("Unsupported function call")
    );
    assert_eq!(result.files.get("/work/stage.txt").unwrap(), "retained");
}

#[test]
fn pending_words_share_memory_with_current_bindings_and_fail_before_side_effects() {
    let mut fixture = case();
    fixture.env.insert("pad".into(), "x".repeat(100_000));
    fixture.env.insert("large".into(), "y".repeat(30_000));
    assert!(fixture.valid());
    for source in [
        "copy=$pad; printf '' \"$large\" \"$large\" \"$large\" > keep.txt",
        "copy=$pad; f() { local a=$large b=$large c=$large; }; f",
    ] {
        for slice in [1, 16, 32, 64] {
            let result = sliced(source, &fixture, slice);
            assert!(
                result.error.as_deref().unwrap().contains("memory"),
                "{result:?}"
            );
            assert_eq!(result.files, fixture.files);
            assert!(result.stdout.is_empty());
        }
    }
    let mut fixture = case();
    fixture.env.insert("pad".into(), "x".repeat(80_000));
    fixture.env.insert("large".into(), "y".repeat(20_000));
    for (count, passes) in [(3, true), (6, false)] {
        let source = format!(
            "copy=$pad; printf '' {} > keep.txt",
            "\"$large\" ".repeat(count)
        );
        let result = sliced(&source, &fixture, 1);
        assert_eq!(result.error.is_none(), passes, "{count}: {result:?}");
        if !passes {
            assert_eq!(result.files, fixture.files);
        }
    }
}

#[test]
fn prepared_function_arguments_move_without_an_extra_live_copy() {
    let mut fixture = case();
    fixture.args = vec!["x".repeat(2000); 64];
    assert!(fixture.valid());
    for slice in [1, 16, 32, 64] {
        let result = sliced("f() { :; }; f \"$@\"", &fixture, slice);
        assert_eq!(result.error, None, "{slice}: {result:?}");
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn saved_command_input_and_pipeline_buffers_share_the_live_text_ceiling() {
    for (copies, passes) in [(6, true), (7, false)] {
        let mut fixture = case();
        fixture.stdin = "p".repeat(40_000);
        fixture
            .files
            .insert("/work/source.txt".into(), "s".repeat(40_000));
        fixture.env.insert("pad".into(), "x".repeat(20_000));
        assert!(fixture.valid());
        let source = (0..copies)
            .map(|i| format!("v{i}=$pad; "))
            .collect::<String>()
            + "printf '' < source.txt";
        for slice in [1, 16, 32, 64] {
            let result = sliced(&source, &fixture, slice);
            assert_eq!(
                result.error.is_none(),
                passes,
                "{copies}/{slice}: {result:?}"
            );
            assert_eq!(result.files, fixture.files);
        }
    }
    let mut fixture = case();
    fixture.env.insert("pad".into(), "x".repeat(60_000));
    fixture.env.insert("line".into(), "y".repeat(30_000));
    let result = sliced(
        "printf '%s' \"$line\" | printf '' \"$line\" \"$line\" > keep.txt",
        &fixture,
        1,
    );
    assert!(result.error.as_deref().unwrap().contains("memory"));
    assert_eq!(result.files, fixture.files);
}

#[test]
fn writing_an_active_ordinary_input_is_explicitly_unsupported_before_truncation() {
    let mut fixture = case();
    fixture
        .files
        .insert("/work/source.txt".into(), "alpha\n".into());
    fixture
        .files
        .insert("/work/other.txt".into(), "beta\n".into());
    for source in [
        "read -r line < source.txt > source.txt; printf wrong > keep.txt",
        "printf replacement < source.txt >> /work/source.txt",
        "read -r line < source.txt 2> source.txt",
    ] {
        for slice in [1, 16, 32, 64] {
            let result = sliced(source, &fixture, slice);
            assert!(
                result
                    .error
                    .as_deref()
                    .unwrap()
                    .contains("active input source")
            );
            assert_eq!(result.files, fixture.files);
            assert!(result.stdout.is_empty());
        }
    }
    // The superseded input binding is no longer active and is not over-guarded.
    let result = sliced(
        "read -r line < source.txt < other.txt > source.txt; printf '%s' \"$line\"",
        &fixture,
        1,
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "beta");
    assert_eq!(result.files["/work/source.txt"], "");
    assert_eq!(result.files["/work/other.txt"], "beta\n");
}

#[derive(Deserialize)]
struct OldResourceEvidence {
    actual_v6_probe_sha256: String,
    actual_v6_shell_source_sha256: String,
    original_test_file_sha256: String,
    fixtures: Vec<OldResourceFixture>,
}
#[derive(Deserialize)]
struct OldResourceFixture {
    name: String,
    source: String,
    case: Case,
    actual_v6_outcome: shell::Outcome,
}
#[test]
fn actual_v6_resource_and_input_results_remain_explicit_historical_evidence() {
    let evidence: OldResourceEvidence = serde_json::from_str(include_str!(
        "fixtures/shell-v6-resource-before-substitution.json"
    ))
    .unwrap();
    assert_eq!(
        evidence.actual_v6_probe_sha256,
        "1a19f870fc03f226a7bbbf215e402b3dbe54aac57609d16ba6ca3cd71a0f7b8c"
    );
    assert_eq!(
        evidence.actual_v6_shell_source_sha256,
        "3187404491dc64b1b2836f795a575b762f37fd88bddbefb51ad92d4ad7705f26"
    );
    assert_eq!(
        evidence.original_test_file_sha256,
        "903ed163ce5bfbc4c4213bff92ae05b6074bb8030aef731841a1d51f90fe26ad"
    );
    assert_eq!(evidence.fixtures.len(), 6);
    for fixture in evidence.fixtures {
        assert!(fixture.case.valid());
        let old = shell::Record {
            engine_revision: 6,
            source: fixture.source.clone(),
            results: vec![fixture.actual_v6_outcome.clone()],
        };
        assert!(old.valid());
        let result = sliced(&fixture.source, &fixture.case, 1);
        match fixture.name.as_str() {
            "quoted-multipart-29" => assert_eq!(result, fixture.actual_v6_outcome),
            "quoted-multipart-30" | "quoted-multipart-31" => {
                assert_eq!(fixture.actual_v6_outcome.error, None);
                assert_eq!(fixture.actual_v6_outcome.status, 0);
                assert_eq!(
                    result.error.as_deref(),
                    Some("Virtual memory/file limit exceeded")
                );
                assert_eq!(result.files, fixture.case.files);
            }
            "quoted-four-65536" | "quoted-multipart-1000" => {
                assert_eq!(
                    fixture.actual_v6_outcome.error.as_deref(),
                    Some("Expansion memory limit reached (256 KiB of expanded text)")
                );
                assert_eq!(
                    result.error.as_deref(),
                    Some("Virtual memory/file limit exceeded")
                );
                assert_eq!(result.files, fixture.case.files);
            }
            "same-file-finite-input-v6" => {
                assert_eq!(fixture.actual_v6_outcome.error, None);
                assert_eq!(fixture.actual_v6_outcome.stdout, "<alpha>");
                assert!(
                    result
                        .error
                        .as_deref()
                        .unwrap()
                        .contains("active input source")
                );
                assert_eq!(result.files, fixture.case.files);
                assert!(result.stdout.is_empty());
            }
            name => panic!("Unexpected historical fixture: {name}"),
        }
    }
}
