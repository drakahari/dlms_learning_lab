//! Bounded GNU-style cat operand selection; all effects stay in the existing VM.
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use serde::Deserialize;
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
        name: "Cat boundary".into(),
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
fn sliced(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..20_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Finite cooperative test bound exceeded");
}
#[test]
fn fixed_cat_operand_stdin_redirect_pipeline_and_capture_cases_at_four_slices() {
    let rows: Vec<Fixed> =
        serde_json::from_str(include_str!("fixtures/bash-cat-reference.json")).unwrap();
    assert_eq!(rows.len(), 13);
    assert_eq!(
        rows.iter().map(|r| &r.name).collect::<BTreeSet<_>>().len(),
        13
    );
    for r in rows {
        assert!(r.case.valid(), "{}", r.name);
        for slice in [1, 7, 32, 64] {
            let out = sliced(&r.source, &r.case, slice);
            assert!(r.case.accepts(&out), "{} / {slice}: {out:?}", r.name);
        }
    }
}
#[test]
fn independent_read_loop_matches_exact_finite_stdin_and_virtual_prefix_is_a_valid_alternative() {
    let read = "while IFS= read -r line; do printf '%s\\n' \"$line\"; done; printf '%s' \"$line\"";
    for text in [
        "",
        "\n",
        "one\n",
        "one",
        "a\n\n  b  \\c\nlast",
        "éclair\n旗",
    ] {
        let mut c = case();
        c.stdin = text.into();
        let out = shell::run("cat -- -", &c);
        assert_eq!(out.error, None);
        assert_eq!(out.stdout, text);
        assert_eq!(shell::run(read, &c), out);
    }
    let mut c = case();
    c.stdin = "caller\n".into();
    for file in ["-notes", "--", "two words.txt"] {
        c.files
            .insert(format!("/work/{file}"), "input % * data\n".into());
        c.env.insert("FILE".into(), file.into());
        let normal = shell::run(
            "cat -- \"$FILE\"; read -r next; printf '<%s>\\n' \"$next\"",
            &c,
        );
        assert_eq!(normal.error, None);
        assert_eq!(
            shell::run(
                "cat \"/work/$FILE\"; read -r next; printf '<%s>\\n' \"$next\"",
                &c
            ),
            normal
        );
    }
}
#[test]
fn missing_file_is_normal_status_but_other_flags_multiple_operands_and_escapes_fail_honestly() {
    let c = case();
    let out = shell::run("cat -- missing.txt", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 1);
    assert!(out.stderr.contains("missing.txt"));
    assert_eq!(out.files, c.files);
    let out = shell::run("cat -- missing.txt || printf 'retry\\n'", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 0);
    assert_eq!(out.stdout, "retry\n");
    for source in [
        "cat -n",
        "cat -n keep.txt",
        "cat -- keep.txt extra.txt",
        "cat -- -- keep.txt",
        "cat -- /etc/passwd",
        "cat -- ../keep.txt",
        "cat -- ''",
    ] {
        let out = shell::run(source, &c);
        assert!(out.error.is_some(), "{source}: {out:?}");
        assert_eq!(out.status, 2);
        assert_eq!(out.files, c.files);
    }
}
#[test]
fn cat_preparation_failure_precedes_own_redirect_and_normal_missing_file_does_not_undo_it() {
    let mut c = case();
    c.stdin = "stream\n".into();
    for source in [
        "cat -- \"$((1/0))\" > keep.txt",
        "cat -- \"${value:-$other}\" > keep.txt",
        "cat -- > ../escape",
    ] {
        let out = shell::run(source, &c);
        assert!(out.error.is_some(), "{source}: {out:?}");
        assert_eq!(out.files, c.files);
    }
    let out = shell::run("cat -- missing.txt > keep.txt", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 1);
    assert_eq!(out.files["/work/keep.txt"], "");
    let mut c = case();
    c.env.insert("many".into(), "x ".repeat(200));
    let out = shell::run("cat $many > keep.txt", &c);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("argument"))
    );
    assert_eq!(out.files, c.files);
}
#[test]
fn cat_uses_existing_output_instruction_and_stop_bounds_without_host_io() {
    let mut c = case();
    c.stdin = "x".repeat(40_000);
    let out = shell::run("cat --", &c);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("output")),
        "{out:?}"
    );
    assert!(out.stdout.len() <= 32_768);
    assert_eq!(out.files, c.files);
    let mut memory = case();
    memory.stdin = "x".repeat(125_000);
    assert!(memory.valid());
    let source = format!(
        "padding() {{ printf '%s' '{}'; }}; cat --",
        "x".repeat(15_000)
    );
    let out = shell::run(&source, &memory);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("memory")),
        "{out:?}"
    );
    assert_eq!(out.files, memory.files);
    let c = case();
    let source = "while true; do cat --; done";
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
    let old = vm.outcome.clone();
    vm.tick(64);
    assert_eq!(vm.outcome, old);
}
#[test]
fn ordinary_copy_helper_and_existing_builtin_name_and_pipeline_restrictions_are_preserved() {
    let mut c = case();
    c.stdin = "caller\n".into();
    c.files
        .insert("/work/-notes".into(), "from virtual file\n".into());
    let out = shell::run(
        "copy() { cat -- \"$1\"; }; copy -notes; read -r next; printf '<%s>\\n' \"$next\"",
        &c,
    );
    assert_eq!(out.error, None);
    assert_eq!(out.status, 0);
    assert_eq!(out.stdout, "from virtual file\n<caller>\n");
    assert_eq!(out.files, c.files);
    let reserved = shell::run("cat() { printf replacement; }; cat -- -notes", &c);
    assert!(
        reserved
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Unsupported function name")),
        "{reserved:?}"
    );
    assert_eq!(reserved.files, c.files);
    let excluded = shell::run("copy() { cat --; }; printf pipe | copy", &c);
    assert!(
        excluded
            .error
            .as_deref()
            .is_some_and(|e| e.contains("function call")),
        "{excluded:?}"
    );
    assert_eq!(excluded.files, c.files);
}
