//! Actual typed environment state, not recognition of expected command strings.
use dlms_learning_lab::shell::{self, Case, EnvironmentSnapshot, EnvironmentVariable, Vm};
use std::collections::BTreeMap;

fn fixture() -> Case {
    Case {
        name: "Environment contract".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep".into(), "retained\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn completed(source: &str, case: &Case, slice: usize) -> Vm {
    let mut vm = Vm::new(source, case).expect("Fixed contract source compiles");
    for _ in 0..20_000 {
        if vm.done() {
            return vm;
        }
        vm.tick(slice);
    }
    panic!("Cooperative contract execution exceeded its finite bound");
}
fn snapshot(source: &str, case: &Case, slice: usize) -> EnvironmentSnapshot {
    let vm = completed(source, case, slice);
    assert!(
        vm.outcome.error.is_none(),
        "{source}: {:?}",
        vm.outcome.error
    );
    assert_eq!(vm.outcome.files, case.files);
    vm.environment_snapshot()
        .expect("Supported completed state is observable")
}

#[test]
fn inherited_values_begin_exported_but_script_only_values_do_not() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let mut c = fixture();
    c.env.insert("INHERITED".into(), "original".into());
    for slice in [1, 7, 64] {
        let s = snapshot("INHERITED=updated\nLOCAL='West Court'\n", &c, slice);
        assert_eq!(
            s.parent["INHERITED"],
            EnvironmentVariable {
                value: Some("updated".into()),
                exported: true
            }
        );
        assert_eq!(
            s.parent["LOCAL"],
            EnvironmentVariable {
                value: Some("West Court".into()),
                exported: false
            }
        );
        assert_eq!(
            s.child,
            BTreeMap::from([("INHERITED".into(), "updated".into())])
        );
        assert!(!s.parent.contains_key("OPTIND"));
    }
}
#[test]
fn supported_export_orders_are_equivalent_and_not_exact_command_matches() {
    let mut c = fixture();
    c.args.push("West * Court\\Wing".into());
    let mut prior = None;
    for source in [
        "ROOM=$1\nexport ROOM\n",
        "export ROOM\nROOM=$1\n",
        "export ROOM=$1\n",
        "export ROOM=\"$1\"\n",
    ] {
        for slice in [1, 7, 64] {
            let s = snapshot(source, &c, slice);
            assert_eq!(s.child["ROOM"], c.args[0]);
            if let Some(prior) = &prior {
                assert_eq!(&s, prior);
            } else {
                prior = Some(s);
            }
        }
    }
}
#[test]
fn unset_empty_and_unexported_states_are_distinct() {
    let c = fixture();
    let s = snapshot("export EMPTY='' UNSET\nLOCAL=''\n", &c, 1);
    assert_eq!(s.parent["EMPTY"].value.as_deref(), Some(""));
    assert_eq!(
        s.parent["UNSET"],
        EnvironmentVariable {
            value: None,
            exported: true
        }
    );
    assert_eq!(
        s.parent["LOCAL"],
        EnvironmentVariable {
            value: Some("".into()),
            exported: false
        }
    );
    assert_eq!(s.child, BTreeMap::from([("EMPTY".into(), "".into())]));
    let s = snapshot("export UNSET\nexport -n UNSET\nexport -n MISSING\n", &c, 64);
    assert_eq!(
        s.parent.get("UNSET").cloned().unwrap_or_default(),
        EnvironmentVariable::default()
    );
    assert!(!s.child.contains_key("UNSET"));
}
#[test]
fn declaration_rhs_preserves_fields_and_globs_and_uses_original_values() {
    let mut c = fixture();
    c.env
        .insert("SEED".into(), "*.txt West\tCourt\nUpper".into());
    c.files
        .insert("/work/ROOM=a.txt".into(), "not a declaration value".into());
    for slice in [1, 7, 64] {
        let s = snapshot("A=old\nexport A=new B=$A ROOM=$SEED\n", &c, slice);
        assert_eq!(s.child["A"], "new");
        assert_eq!(s.child["B"], "old");
        assert_eq!(s.child["ROOM"], c.env["SEED"]);
    }
    c.env.insert("SEED".into(), "*.txt".into());
    assert_eq!(
        snapshot("export ROOM=$SEED\n", &c, 1).child["ROOM"],
        "*.txt"
    );
}
#[test]
fn unexport_removes_only_attribute_and_assignment_read_preserve_it() {
    let mut c = fixture();
    c.env.insert("ROOM".into(), "East".into());
    c.stdin = "West Court\n".into();
    for source in [
        "export -n ROOM\nROOM='West Court'\n",
        "export -n ROOM\nread -r ROOM\n",
    ] {
        let s = snapshot(source, &c, 7);
        assert_eq!(s.parent["ROOM"].value.as_deref(), Some("West Court"));
        assert!(!s.parent["ROOM"].exported);
        assert!(!s.child.contains_key("ROOM"));
    }
    assert_eq!(
        snapshot("read -r ROOM\n", &c, 1).child["ROOM"],
        "West Court"
    );
    c.env.clear();
    assert!(!snapshot("read -r ROOM\n", &c, 1).parent["ROOM"].exported);
    assert_eq!(
        snapshot("export ROOM\nread -r ROOM\n", &c, 1).child["ROOM"],
        "West Court"
    );
}
#[test]
fn decisions_apply_only_reached_declarations() {
    let c = fixture();
    let s = snapshot(
        "if false; then export WRONG=bad; else export RIGHT=good; fi\nfalse && export SKIP=no\ntrue || export ALSO_SKIP=no\nfalse || export FALLBACK=yes\n",
        &c,
        1,
    );
    assert_eq!(
        s.child,
        BTreeMap::from([
            ("RIGHT".into(), "good".into()),
            ("FALLBACK".into(), "yes".into())
        ])
    );
}
#[test]
fn unsupported_scope_is_refused_for_operations_and_observations_without_changing_ordinary_support()
{
    let c = fixture();
    for body in [
        "printf a | cat",
        "for x in a; do :; done",
        "while false; do :; done",
        "case a in a) : ;; esac",
        "f() { :; }; f",
        "a=(a)",
        "a=$(printf a)",
        "IFS= read -r name",
    ] {
        let ordinary = completed(body, &c, 1);
        assert!(
            ordinary.outcome.error.is_none(),
            "{body}: {:?}",
            ordinary.outcome.error
        );
        assert!(
            ordinary
                .environment_snapshot()
                .unwrap_err()
                .contains("Unsupported environment program shape")
        );
        let new = completed(&format!("export ROOM\n{body}"), &c, 1);
        assert!(
            new.outcome
                .error
                .as_ref()
                .is_some_and(|e| e.contains("Unsupported environment program shape"))
        );
        assert_eq!(new.outcome.files, c.files);
    }
}
#[test]
fn computed_heads_operands_special_names_and_unimplemented_flags_fail_honestly() {
    let c = fixture();
    for source in [
        "export",
        "export -p",
        "export -f NAME",
        "export -n",
        "export -n NAME=value",
        "export 9BAD",
        "export IFS",
        "export OPTIND",
        "export UID",
        "export BASH_ENV",
        "name=ROOM\nexport $name",
        "name=ROOM\nexport ${name}=value",
        "cmd=export\n\"$cmd\" ROOM",
        "\"export\" ROOM",
    ] {
        let vm = completed(source, &c, 1);
        let r = &vm.outcome;
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
            "{source}: {r:?}"
        );
        assert_eq!(r.files, c.files);
        assert!(vm.environment_snapshot().is_err(), "{source}");
    }

    // The existing lexer rejects unquoted brackets before export admission.
    // Preserve that specific diagnostic rather than requiring a new parser path.
    assert!(Vm::new("export names[0]", &c).is_err());
    let bracket_operand = shell::run("export names[0]", &c);
    assert_eq!(
        bracket_operand.error.as_deref(),
        Some("Bracket globbing is unsupported; quote literal brackets")
    );
    assert_eq!(bracket_operand.files, c.files);
    assert_eq!(bracket_operand.status, 2);

    // Prefix assignment rejection belongs to the pre-existing command path.
    let command_prefix = completed("ROOM=value printf x", &c, 1);
    assert_eq!(
        command_prefix.outcome.error.as_deref(),
        Some("Assignments before a command are unsupported; put the assignment on its own line")
    );
    assert_eq!(command_prefix.outcome.files, c.files);
    assert!(command_prefix.environment_snapshot().is_err());
}
#[test]
fn export_and_xtrace_are_honestly_incompatible_in_both_orders() {
    let c = fixture();
    for source in [
        "set -x\nexport ROOM",
        "set -x\nset +x\nexport ROOM",
        "export ROOM\nset -x",
        "export ROOM\nset -o xtrace",
    ] {
        let vm = completed(source, &c, 1);
        assert!(
            vm.outcome
                .error
                .as_ref()
                .is_some_and(|e| e.contains("Unsupported"))
        );
        assert!(vm.environment_snapshot().is_err());
    }
    let vm = completed("set -x\nprintf hi\nset +x", &c, 1);
    assert!(vm.outcome.error.is_none());
    assert!(vm.environment_snapshot().is_err());
}
#[test]
fn environment_activity_refuses_locale_controls_without_changing_ordinary_scripts() {
    let c = fixture();
    for key in ["LANG", "LANGUAGE", "LC_ALL", "LC_CTYPE", "LC_FUTURE"] {
        for source in [
            format!("export {key}=C\n"),
            format!("export -n {key}\n"),
            format!("{key}=C\nexport ROOM=west\n"),
        ] {
            let vm = completed(&source, &c, 1);
            assert!(
                vm.outcome.error.as_ref().is_some_and(|e| e.contains("C/POSIX")),
                "{source}: {:?}",
                vm.outcome.error
            );
            assert_eq!(vm.outcome.files, c.files);
            assert!(vm.environment_snapshot().is_err());
        }
        let ordinary = completed(&format!("{key}=C\nprintf '%s' \"${key}\"\n"), &c, 1);
        assert!(ordinary.outcome.error.is_none());
        assert_eq!(ordinary.outcome.stdout, "C");
        assert!(ordinary.environment_snapshot().unwrap_err().contains("C/POSIX"));

        let mut imported = c.clone();
        imported.env.insert(key.into(), "C".into());
        let ordinary = completed("printf ordinary\n", &imported, 1);
        assert!(ordinary.outcome.error.is_none());
        assert_eq!(ordinary.outcome.stdout, "ordinary");
        assert!(ordinary.environment_snapshot().unwrap_err().contains("C/POSIX"));
        let exporting = completed("export ROOM=west\n", &imported, 1);
        assert!(exporting.outcome.error.as_ref().is_some_and(|e| e.contains("C/POSIX")));
        assert_eq!(exporting.outcome.files, imported.files);
        assert!(exporting.environment_snapshot().is_err());
    }
    let changed_later = completed("export ROOM=west\nLC_ALL=C\n", &c, 1);
    assert!(changed_later.outcome.error.is_none());
    assert!(changed_later.environment_snapshot().unwrap_err().contains("C/POSIX"));
}
#[test]
fn name_limit_counts_unset_attributes_and_snapshot_is_not_available_after_stop_or_failure() {
    let c = fixture();
    let declarations = (0..256)
        .map(|n| format!("export V{n}\n"))
        .collect::<String>();
    let s = snapshot(&declarations, &c, 1);
    assert_eq!(s.parent.len(), 256);
    assert!(s.child.is_empty());
    let no_op = snapshot(&(declarations.clone() + "export -n MISSING\n"), &c, 1);
    assert_eq!(
        no_op, s,
        "Unexporting an absent name must not require a free slot"
    );
    let replaced = snapshot(
        &(declarations.clone() + "export -n V0\nexport NEW\n"),
        &c,
        1,
    );
    assert_eq!(replaced.parent.len(), 256);
    assert!(!replaced.parent.contains_key("V0"));
    assert!(replaced.parent["NEW"].exported);
    let vm = completed(&(declarations + "export EXCESS\n"), &c, 1);
    assert!(
        vm.outcome
            .error
            .as_ref()
            .is_some_and(|e| e.contains("limit"))
    );
    assert!(vm.environment_snapshot().is_err());
    let mut vm = Vm::new("export ROOM=hello\n", &c).unwrap();
    assert!(vm.environment_snapshot().is_err());
    vm.tick(1);
    vm.stop();
    assert!(vm.done());
    assert!(vm.environment_snapshot().is_err());
}
#[test]
fn snapshots_do_not_change_old_outcome_wire_shape_or_mutate_vm() {
    let c = fixture();
    let vm = completed("export ROOM='West Court'\n", &c, 64);
    let before = serde_json::to_value(&vm.outcome).unwrap();
    let mut s = vm.environment_snapshot().unwrap();
    s.child.clear();
    assert_eq!(
        vm.environment_snapshot().unwrap().child["ROOM"],
        "West Court"
    );
    assert_eq!(before, serde_json::to_value(&vm.outcome).unwrap());
    assert_eq!(
        before
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["error", "files", "status", "stderr", "stdout"]
    );
    assert!(
        serde_json::from_str::<EnvironmentVariable>(
            "{\"value\":null,\"exported\":false,\"unknown\":true}"
        )
        .is_err()
    );
}
