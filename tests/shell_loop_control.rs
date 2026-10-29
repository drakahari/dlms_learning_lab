//! Prospective in-process tests; no shell, subprocess, network or personal files.
use dlms_learning_lab::shell::{Case, Outcome, Vm};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize)]
struct Fixed {
    supported_cases: usize,
    honest_exclusions: usize,
    cases: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    id: String,
    partition: String,
    source: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    stdin: String,
    files: BTreeMap<String, String>,
    expected: Option<Expected>,
    vm_observation: Option<serde_json::Value>,
    bash_observation: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct Expected {
    stdout: String,
    stderr: String,
    status: i32,
    after: BTreeMap<String, String>,
}
fn fixture() -> Case {
    Case {
        name: "Loop control".into(),
        args: vec![],
        env: BTreeMap::new(),
        stdin: String::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn sliced(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).unwrap();
    for _ in 0..100_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Cooperative loop VM did not finish within bounded polling")
}
fn expect(source: &str, c: &Case, stdout: &str, files: &BTreeMap<String, String>) {
    for slice in [1, 7, 32, 64] {
        let o = sliced(source, c, slice);
        assert_eq!(o.error, None, "{source} slice{slice}");
        assert_eq!(o.stdout, stdout, "slice{slice}");
        assert_eq!(o.stderr, "");
        assert_eq!(o.status, 0);
        assert_eq!(&o.files, files);
    }
}
#[test]
fn fixed16_supported_sources_and8_explicit_exclusions_at_four_slices() {
    let f: Fixed =
        serde_json::from_str(include_str!("fixtures/shell-loop-control-fixed.json")).unwrap();
    assert_eq!(
        (f.supported_cases, f.honest_exclusions, f.cases.len()),
        (16, 8, 24)
    );
    let (mut positives, mut negatives) = (0, 0);
    for r in f.cases {
        assert!(r.vm_observation.is_none() && r.bash_observation.is_none());
        let c = Case {
            name: r.id.clone(),
            args: r.args,
            env: r.env,
            stdin: r.stdin,
            files: r.files,
            ..fixture()
        };
        for slice in [1, 7, 32, 64] {
            let o = sliced(&r.source, &c, slice);
            if r.partition == "supported" {
                let e = r.expected.as_ref().unwrap();
                assert_eq!(o.error, None, "{} slice{slice}", r.id);
                assert_eq!(o.stdout, e.stdout, "{}", r.id);
                assert_eq!(o.stderr, e.stderr);
                assert_eq!(o.status, e.status);
                assert_eq!(o.files, e.after);
            } else {
                assert_eq!(r.partition, "honest_exclusion");
                assert!(r.expected.is_none());
                assert!(
                    o.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
                    "{} {o:?}",
                    r.id
                );
                assert_eq!(o.files, c.files);
                assert_ne!(o.status, 0);
            }
        }
        if r.partition == "supported" {
            positives += 1
        } else {
            negatives += 1
        }
    }
    assert_eq!((positives, negatives), (16, 8));
}
#[test]
fn expanded_counts_heads_and_quoted_counts_keep_normal_semantics() {
    let s = "action=continue; depth=2; for a in A B; do for b in 1 2; do printf '%s%s\\n' \"$a\" \"$b\"; \"$action\" \"$depth\"; printf 'wrong'; done; done; printf '%s\\n' \"$?\"";
    expect(s, &fixture(), "A1\nB1\n0\n", &BTreeMap::new());
    expect(
        "for x in a b; do 'break' '24'; done; printf '%s\\n' \"$?\"",
        &fixture(),
        "0\n",
        &BTreeMap::new(),
    );
}
#[test]
fn mixed_three_level_unwind_restores_outer_input_and_snapshot() {
    let mut c = fixture();
    c.stdin = "outer\n".into();
    c.files
        .insert("/work/in.txt".into(), "first\nsecond\n".into());
    expect(
        "for a in A B; do while IFS= read -r b; do for z in x y; do printf '%s:%s:%s\\n' \"$a\" \"$b\" \"$z\"; continue 3; done; done < in.txt; done; IFS= read -r tail; printf '%s\\n' \"$tail\"",
        &c,
        "A:first:x\nB:first:x\nouter\n",
        &c.files,
    );
}
#[test]
fn continue_while_keeps_live_cursor_and_updates_last_body_status() {
    let mut c = fixture();
    c.files.insert("/work/in.txt".into(), "skip\nkeep\n".into());
    expect(
        "while IFS= read -r line; do if test \"$line\" = skip; then false; continue; fi; printf '%s\\n' \"$line\"; done < in.txt; printf 'status=%s\\n' \"$?\"",
        &c,
        "keep\nstatus=0\n",
        &c.files,
    );
}
#[test]
fn break_in_while_condition_and_for_values_snapshot_are_real_control_flow() {
    expect(
        "while break; do printf 'wrong'; done; printf 'done\\n'",
        &fixture(),
        "done\n",
        &BTreeMap::new(),
    );
    expect(
        "items='a b'; for x in $items; do items=changed; if test \"$x\" = a; then continue; fi; printf '%s\\n' \"$x\"; done",
        &fixture(),
        "b\n",
        &BTreeMap::new(),
    );
}
#[test]
fn temporary_command_input_is_restored_before_redirected_loop_unwind() {
    let mut c = fixture();
    c.stdin = "outside\n".into();
    c.files = BTreeMap::from([
        ("/work/loop.txt".into(), "one\ntwo\n".into()),
        ("/work/command.txt".into(), "temporary\n".into()),
    ]);
    expect(
        "while IFS= read -r line; do break < command.txt; done < loop.txt; IFS= read -r last; printf '%s\\n' \"$last\"",
        &c,
        "outside\n",
        &c.files,
    );
    expect(
        "for x in a b; do while IFS= read -r line; do printf '%s\\n' \"$x\"; continue 2 < command.txt; done < loop.txt; done; IFS= read -r last; printf '%s\\n' \"$last\"",
        &c,
        "a\nb\noutside\n",
        &c.files,
    );
}
#[test]
fn missing_input_redirect_fails_command_without_transfer_or_truncate() {
    let s = "for x in a b; do break < missing.txt > untouched.txt; printf '%s\\n' \"$x\"; done";
    for slice in [1, 7, 64] {
        let o = sliced(s, &fixture(), slice);
        assert_eq!(o.error, None);
        assert_eq!(o.stdout, "a\nb\n");
        assert!(o.stderr.contains("No such file"));
        assert!(o.files.is_empty());
    }
}
#[test]
fn ordered_redirect_and_capture_expansion_effects_happen_once_before_transfer() {
    let c = fixture();
    let files = BTreeMap::from([
        ("/work/count.txt".into(), "once\n".into()),
        ("/work/out.txt".into(), String::new()),
    ]);
    expect(
        "for x in a b; do break \"$(printf 'once\\n' > count.txt; printf '1')\" > out.txt; done; printf 'done\\n'",
        &c,
        "done\n",
        &files,
    );
}
#[test]
fn own_function_loops_preserve_caller_state_and_function_return_cleanup() {
    expect(
        "f() { for x in x y; do while true; do printf '%s\\n' \"$x\"; continue 2; done; done; }; for a in A B; do f; printf '%s\\n' \"$a\"; done",
        &fixture(),
        "x\ny\nA\nx\ny\nB\n",
        &BTreeMap::new(),
    );
}
#[test]
fn nested_capture_own_loop_does_not_change_parent_loop_or_control_head() {
    expect(
        "for a in A B; do v=$(for b in x y; do printf '%s' \"$(for c in i j; do printf '%s' \"$c\"; break; done)\"; continue; done); printf '%s:%s\\n' \"$a\" \"$v\"; done",
        &fixture(),
        "A:ii\nB:ii\n",
        &BTreeMap::new(),
    );
}
#[test]
fn all_noncanonical_counts_and_pipeline_heads_fail_honestly_without_file_changes() {
    for bad in [
        "0",
        "00",
        "01",
        "+1",
        "-1",
        "25",
        "999999999999999999999",
        "１２",
        "1 2",
        "''",
    ] {
        let s = format!("for x in a; do break {bad}; done");
        let o = sliced(&s, &fixture(), 1);
        assert!(
            o.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
            "{bad}: {o:?}"
        );
        assert!(o.files.is_empty());
    }
    for s in [
        "control=continue; for x in a; do printf a | \"$control\"; done",
        "for x in a; do 'break' | cat; done",
    ] {
        let o = sliced(s, &fixture(), 1);
        assert!(o.error.unwrap().contains("pipeline"));
    }
}
#[test]
fn caller_loops_are_never_escaped_by_function_or_capture() {
    for s in [
        "f() { break; }; for x in a b; do f; printf 'wrong'; done",
        "for x in a; do v=$(continue); printf 'wrong'; done",
    ] {
        let o = sliced(s, &fixture(), 1);
        assert!(o.error.unwrap().contains("caller loops"));
        assert!(!o.stdout.contains("wrong"));
    }
}
#[test]
fn instruction_limit_and_stop_remain_cooperative_for_continue_forever() {
    let s = "while true; do continue; done";
    let o = sliced(s, &fixture(), 1);
    assert!(o.error.unwrap().contains("Execution limit"));
    let mut vm = Vm::new(s, &fixture()).unwrap();
    vm.tick(7);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    let old = vm.outcome.clone();
    vm.tick(64);
    assert_eq!(vm.outcome, old);
    assert!(old.error.unwrap().contains("Stopped"));
}
#[test]
fn equivalent_skip_and_early_stop_programs_produce_the_same_behavior() {
    for values in [
        vec!["skip", "keep", "END", "tail"],
        vec!["first", "second"],
        vec!["END"],
        vec!["skip", "skip"],
    ] {
        let mut c = fixture();
        c.args = values.iter().map(|s| s.to_string()).collect();
        let control = "for row in \"$@\"; do if test \"$row\" = END; then break; fi; if test \"$row\" = skip; then continue; fi; printf '%s\\n' \"$row\"; done";
        let alternative = "active=yes; for row in \"$@\"; do if test \"$row\" = END; then active=no; fi; if test \"$active\" = yes; then if test \"$row\" != skip; then printf '%s\\n' \"$row\"; fi; fi; done";
        for n in [1, 7, 64] {
            let a = sliced(control, &c, n);
            let b = sliced(alternative, &c, n);
            assert_eq!(a.error, None);
            assert_eq!(b.error, None);
            assert_eq!(a, b);
        }
    }
}

#[test]
fn inherited_function_keeps_own_loop_scope_inside_nested_captures() {
    expect(
        "f() { for x in a b; do printf '%s' \"$x\"; break; done; }; for outer in A B; do value=$(printf '%s' \"$(f)\"); printf '%s:%s\\n' \"$outer\" \"$value\"; done",
        &fixture(),
        "A:a\nB:a\n",
        &BTreeMap::new(),
    );
}
