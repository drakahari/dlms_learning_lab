use dlms_learning_lab::shell::{self, Case};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "function bounds".into(),
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
fn function_return_restores_arguments_and_discards_only_callee_loop_frames() {
    let r = shell::run(
        "inside() { for x in inner; do while true; do return 7; done; done; }\nfor x in a b; do inside child; printf '%s:%s:%s\\n' \"$x\" \"$?\" \"$#\"; done",
        &case(),
    );
    assert_eq!(r.error, None);
    // Scalars are global: inside's x is shared; the caller's next iteration resets it.
    assert_eq!(r.stdout, "inner:7:0\ninner:7:0\n");
}
#[test]
fn recursion_is_cooperative_and_bounded_and_stop_discards_success() {
    let source = "again() { again; }\nagain";
    let mut vm = shell::Vm::new(source, &case()).unwrap();
    vm.tick(4);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    let r = shell::run(source, &case());
    assert!(r.error.unwrap().contains("call depth limit"));
    let r = shell::run("spin() { while true; do :; done; }\nspin", &case());
    assert!(r.error.unwrap().contains("Execution limit"));
}
#[test]
fn unsupported_function_contexts_fail_instead_of_approximating() {
    for source in [
        "f() { :; }\nf | cat source",
        "f() { :; }\nf > output",
        "f() { :; }\nf < source",
        "f() { return | printf wrong; }\nf",
        "return 0",
        "f() ( printf wrong )\nf",
        "printf() { :; }",
        "f() { return 256; }\nf",
        "shift -1",
        "echo {a,b}",
        "echo $(( $(printf 1) ))",
        "{ printf wrong; }",
    ] {
        let r = shell::run(source, &case());
        assert!(r.error.is_some(), "{source}: {r:?}");
    }
}
#[test]
fn shift_status_and_function_memory_are_bounded() {
    let r = shell::run("shift\nprintf '%s:%s\\n' \"$?\" \"$#\"", &case());
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "1:0\n");
    let mut c = case();
    c.args = vec!["x".repeat(100_000)];
    let r = shell::run("again() { again \"$1\"; }\nagain \"$1\"", &c);
    assert!(r.error.unwrap().contains("memory"));
}

#[test]
fn reserved_names_cannot_be_smuggled_in_as_definitions() {
    for name in [
        "elif", "in", "esac", "time", "coproc", "function", "until", "case", "select", "local",
        "set", "eval", "printf",
    ] {
        let r = shell::run(&format!("{name}() {{ :; }}"), &case());
        assert!(r.error.is_some(), "{name}: {r:?}");
    }
}
#[test]
fn expanded_and_quoted_return_keep_real_redirection_behavior() {
    for source in [
        r#"f() { r=return; "$r" > output; printf wrong; }; f"#,
        r#"f() { "return" > output; printf wrong; }; f"#,
    ] {
        let r = shell::run(source, &case());
        assert_eq!(r.error, None);
        assert_eq!(r.stdout, "");
        assert_eq!(r.files.get("/work/output").map(String::as_str), Some(""));
        assert_eq!(r.status, 0);
    }
}

#[test]
fn captures_call_inherited_and_child_defined_functions_without_leaking_bindings() {
    let result = shell::run(
        "label=parent; inherited() { local label=inside; printf '%s' \"$label\"; }; value=$(inherited; child() { printf child; }; child); printf '<%s>:%s' \"$value\" \"$label\"",
        &case(),
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "<insidechild>:parent");
    let result = shell::run("value=$(child() { :; }; child); child", &case());
    assert!(result.error.unwrap().contains("Unsupported"));
}
