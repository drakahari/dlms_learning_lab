//! Colon is an ordinary no-output success builtin, after full preparation.
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
        name: "Colon failure control".into(),
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
fn fixed_colon_arguments_capture_status_redirects_and_input_restore_match_at_every_slice() {
    let rows: Vec<Fixed> =
        serde_json::from_str(include_str!("fixtures/bash-colon-reference.json")).unwrap();
    assert_eq!(rows.len(), 6);
    assert_eq!(
        rows.iter().map(|r| &r.name).collect::<BTreeSet<_>>().len(),
        6
    );
    for r in rows {
        assert!(r.case.valid());
        for slice in [1, 7, 32, 64] {
            let out = sliced(&r.source, &r.case, slice);
            assert!(r.case.accepts(&out), "{} / {slice}: {out:?}", r.name);
        }
    }
}
#[test]
fn colon_does_not_skip_failed_expansion_ambiguous_redirect_or_virtual_path_guards() {
    let c = case();
    for source in [
        ": \"$((1/0))\" > keep.txt",
        ": \"${value:-$other}\" > keep.txt",
        ": anything > ../keep.txt",
        ": anything > /etc/passwd",
    ] {
        let out = shell::run(source, &c);
        assert!(out.error.is_some(), "{source}: {out:?}");
        assert_eq!(out.status, 2);
        assert_eq!(out.files, c.files);
        assert!(out.stdout.is_empty());
    }
    let mut ambiguous = case();
    ambiguous
        .env
        .insert("destination".into(), "two words".into());
    let out = shell::run(": anything > $destination", &ambiguous);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.contains("Ambiguous")),
        "{out:?}"
    );
    assert_eq!(out.files, ambiguous.files);
    let mut many = case();
    many.env.insert("many".into(), "word ".repeat(200));
    let out = shell::run(": $many > keep.txt", &many);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("argument")),
        "{out:?}"
    );
    assert_eq!(out.files, many.files);
    let mut large = case();
    large.env.insert("large".into(), "x".repeat(90_000));
    let out = shell::run(": \"$large\" \"$large\" \"$large\" > keep.txt", &large);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.to_lowercase().contains("memory")),
        "{out:?}"
    );
    assert_eq!(out.files, large.files);
}
#[test]
fn colon_failed_input_redirect_retains_ordered_effects_and_is_a_recoverable_normal_status() {
    let c = case();
    let out = shell::run(": < missing.txt > keep.txt", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 1);
    assert!(!out.stderr.is_empty());
    assert_eq!(out.files, c.files);
    let out = shell::run(": > touched.txt < missing.txt", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 1);
    assert!(!out.stderr.is_empty());
    let mut expected = c.files.clone();
    expected.insert("/work/touched.txt".into(), String::new());
    assert_eq!(
        out.files, expected,
        "A failed later redirect does not roll back an earlier truncate"
    );
    let out = shell::run(": < missing.txt || printf 'retry\\n'", &c);
    assert_eq!(out.error, None);
    assert_eq!(out.status, 0);
    assert_eq!(out.stdout, "retry\n");
    assert!(!out.stderr.is_empty());
    assert_eq!(out.files, c.files);
}
