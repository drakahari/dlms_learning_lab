//! Diagnostic-only regressions. No host Bash/reference or history generation.
use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;
fn fixture() -> Case {
    Case {
        name: "closed elif diagnostic fixture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([
            ("/work/keep.txt".into(), "original\n".into()),
            ("/work/elif".into(), "ordinary filename\n".into()),
        ]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn static_rejection(source: &str) {
    let case = fixture();
    let error = Vm::new(source, &case)
        .err()
        .expect("bare elif is statically excluded");
    assert!(
        error.contains("Unsupported elif clause"),
        "{source}: {error}"
    );
    assert!(!error.starts_with("Syntax error:"), "{source}: {error}");
    let result = shell::run(source, &case);
    assert_eq!(result.status, 2);
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Unsupported elif clause"))
    );
    assert!(result.stdout.is_empty() && result.stderr.is_empty());
    assert_eq!(
        result.files, case.files,
        "the whole program fails before prior effects: {source}"
    );
}
#[test]
fn excluded_elif_clauses_are_unsupported_before_any_prior_or_branch_effect() {
    for body in [
        "if true; then printf selected; elif true; then printf wrong; fi",
        "if false; then printf wrong; elif true; then printf selected; else printf wrong; fi",
        "if true\nthen\n :\nelif false\nthen\n :\nfi",
        "if true; then if false; then :; elif true; then :; fi; fi",
        "elif true",
        "printf before; elif",
        "helper() { if true; then :; elif false; then :; fi; }; printf after",
        "value=$(if true; then :; elif true; then :; fi); printf after",
    ] {
        static_rejection(&format!("printf changed > keep.txt; {body}"));
    }
}
#[test]
fn pipeline_and_leading_redirects_cannot_disguise_a_bare_elif_head() {
    for body in [
        "printf input | elif true",
        "printf input |\nelif true",
        "> keep.txt elif true",
        "printf input | > keep.txt elif true",
    ] {
        static_rejection(&format!("printf changed > keep.txt; {body}"));
    }
}
#[test]
fn comments_quoted_arguments_filenames_and_keyword_named_variables_keep_data_roles() {
    for (source, expected) in [
        (
            "# elif true; then printf wrong\nprintf kept # elif\n",
            "kept",
        ),
        (
            "printf '%s\\n' elif 'elif' \"elif\" e\\lif",
            "elif\nelif\nelif\nelif\n",
        ),
        ("printf '%s' elif > elif; cat elif", "elif"),
        ("elif=scalar; printf '%s' \"$elif\"", "scalar"),
        ("elif=(one two); printf '%s' \"${elif[0]}\"", "one"),
        ("for tag in elif; do printf '<%s>' \"$tag\"; done", "<elif>"),
        ("case elif in elif) printf matched;; esac", "matched"),
        (
            "if test elif = elif; then printf yes; else printf no; fi",
            "yes",
        ),
        ("if true; then printf '%s' 'elif'; fi", "elif"),
    ] {
        let result = shell::run(source, &fixture());
        assert_eq!(result.error, None, "{source}");
        assert_eq!(result.status, 0, "{source}");
        assert_eq!(result.stdout, expected, "{source}");
    }
}
#[test]
fn quoted_escaped_expanded_and_assignment_first_heads_keep_existing_runtime_timing() {
    for head in ["'elif'", "\"elif\"", "e\\lif", "$head", "\"$head\""] {
        for command in [head.to_owned(), format!("printf input | {head}")] {
            let source = format!("head=elif; printf changed > keep.txt; {command}");
            assert!(Vm::new(&source, &fixture()).is_ok(), "{source}");
            let result = shell::run(&source, &fixture());
            assert_eq!(result.status, 2, "{source}");
            let error = result.error.unwrap();
            assert!(
                error.contains("Unsupported command: elif"),
                "{source}: {error}"
            );
            assert!(
                !error.contains("Unsupported elif clause"),
                "{source}: {error}"
            );
            assert_eq!(result.files["/work/keep.txt"], "changed");
        }
    }
    // Existing assignment-prefix support/error timing owns the later word.
    assert!(Vm::new("tag=value elif", &fixture()).is_ok());
    let result = shell::run(
        "if false; then printf wrong; else if true; then printf nested; else :; fi; fi",
        &fixture(),
    );
    assert_eq!(result.error, None);
    assert_eq!(result.stdout, "nested");
}
