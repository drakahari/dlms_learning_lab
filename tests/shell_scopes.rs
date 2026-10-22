use dlms_learning_lab::shell::{self, Case};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "local scope".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
#[test]
fn locals_start_unset_redeclare_and_restore_earlier_visibility() {
    let r = shell::run(
        "x=outside\nf() { local x; printf '<%s>\\n' \"$x\"; x=inside; local x; printf '%s\\n' \"$x\"; }\nf\nprintf '%s\\n' \"$x\"",
        &case(),
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<>\ninside\noutside\n");
    let r = shell::run(
        "f() { local x=inside; }\nf\nprintf '<%s>\\n' \"$x\"",
        &case(),
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<>\n");
}
#[test]
fn declaration_values_expand_together_without_assignment_field_splitting() {
    let mut c = case();
    c.args = vec!["a * b".into()];
    let r = shell::run(
        "a=outside\nf() { local a=$1 b=\"$a\"; printf '<%s>:%s\\n' \"$a\" \"$b\"; }\nf \"$1\"\nprintf '%s\\n' \"$a\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<a * b>:outside\noutside\n");
}
#[test]
fn callees_share_visible_caller_local_unless_they_shadow_it_themselves() {
    let source = "x=global\nchange() { x=changed; }\nshadow() { local x=child; return 7; }\nparent() { local x=parent; change; printf '%s\\n' \"$x\"; shadow; printf '%s:%s\\n' \"$x\" \"$?\"; }\nparent\nprintf '%s\\n' \"$x\"";
    let r = shell::run(source, &case());
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "changed\nchanged:7\nglobal\n");
}
#[test]
fn unsupported_declaration_contexts_and_options_fail_honestly() {
    for source in [
        "local x=bad",
        "f() { local; }\nf",
        "f() { local $missing; }\nf",
        "f() { local -r x=bad; }\nf",
        "f() { local bad-name=bad; }\nf",
        "f() { local x=bad | printf wrong; }\nf",
        "f() { \"local\" x=bad; }\nf",
        "f() { cmd=local; \"$cmd\" x=bad; }\nf",
        "IFS=,\nprintf wrong",
        "for IFS in comma; do printf wrong; done",
        "read -r IFS\nprintf wrong",
        "f() { local IFS=,; }\nf",
    ] {
        let r = shell::run(source, &case());
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
            "{source}: {r:?}"
        );
        assert!(!r.stdout.contains("wrong"));
    }
}
#[test]
fn shadow_storage_and_variable_names_share_the_bound_and_stop_remains_cooperative() {
    let mut c = case();
    c.env.insert("value".into(), "x".repeat(100_000));
    let r = shell::run("again() { local value=\"$value\"; again; }\nagain", &c);
    assert!(r.error.unwrap().contains("memory"));
    let mut c = case();
    c.env.insert("v".repeat(140_000), String::new());
    assert!(!c.valid());
    let mut vm = shell::Vm::new("again() { local value=inside; again; }\nagain", &case()).unwrap();
    vm.tick(4);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    let mut vm = shell::Vm::new("again() { local value=inside; again; }\nagain", &case()).unwrap();
    for _ in 0..100 {
        vm.tick(64);
        if vm.done() {
            break;
        }
    }
    assert!(vm.outcome.error.unwrap().contains("call depth"));
}

#[test]
fn unmodeled_special_variables_are_rejected_in_every_binding_and_read_context() {
    for variable in ["UID", "_", "RANDOM", "BASHOPTS", "PIPESTATUS"] {
        for source in [
            format!("{variable}=wrong"),
            format!("f() {{ local {variable}=wrong; }}\nf"),
            format!("for {variable} in wrong; do :; done"),
            format!("read -r {variable}"),
            format!("printf '%s' \"${variable}\""),
        ] {
            assert!(shell::run(&source, &case()).error.is_some(), "{source}");
        }
        let mut c = case();
        c.env.insert(variable.into(), "value".into());
        assert!(!c.valid());
    }
    assert_eq!(
        shell::run("printf '<%s>' \"$IFS\"", &case()).stdout,
        "< \t\n>"
    );
}
#[test]
fn unset_local_names_count_as_visible_variable_slots() {
    let declarations = (0..257)
        .map(|n| format!("local name{n}\n"))
        .collect::<String>();
    let r = shell::run(&format!("f() {{\n{declarations}}}\nf"), &case());
    assert!(r.error.unwrap().contains("limit"));
    let declarations = (0..256)
        .map(|n| format!("local name{n}\n"))
        .collect::<String>();
    assert_eq!(
        shell::run(&format!("f() {{\n{declarations}}}\nf"), &case()).error,
        None
    );
}
