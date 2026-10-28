//! One explicit invalid proposal, not a skipped positive or VM dispatch exception.
use super::amendment_sha as digest;
use dlms_learning_lab::{
    script_teaching::{self, Diagnostic},
    shell::{self, Case, Vm},
};
pub fn invalid_proposal(name: &str, source: &str, case: &Case) -> bool {
    if name != "arithmetic ignores parentheses in unselected literal" {
        return false;
    }
    let m: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/bash-parameter-arithmetic-amendment.json"
    ))
    .unwrap();
    assert_eq!(m["schema_version"], 1);
    assert_eq!(m["original_index"], 32);
    assert_eq!(m["name"], name);
    assert_eq!(
        m["fixed_input_sha256"],
        digest::hash(include_bytes!(
            "../fixtures/bash-parameter-defaults-reference.json"
        ))
    );
    assert_eq!(m["source_sha256"], digest::hash(source.as_bytes()));
    assert_eq!(
        m["case_canonical_sha256"],
        digest::hash(&serde_json::to_vec(&serde_json::to_value(case).unwrap()).unwrap())
    );
    assert_eq!(m["original_expected_stdout"], case.stdout);
    assert_eq!(case.stdout, "8\n");
    assert_eq!(m["actual_bash"]["status"], 1);
    assert_eq!(m["actual_bash"]["stdout"], "");
    assert_eq!(
        m["actual_bash"]["observation_sha256"],
        "a88b399953a9bb660a1492cd565c8e34e009ebe85de3486379dea775833169ab"
    );
    assert_eq!(
        m["actual_bash"]["capture_manifest_sha256"],
        "86b789a47debdccbbbcab130fe40cacd16ac44ca049d593b9bf8bf6a6de8bbad"
    );
    true
}
pub fn assert_syntax_before_effects(source: &str, case: &Case) {
    let error = Vm::new(source, case)
        .err()
        .expect("Malformed arithmetic/parameter span must fail while compiling");
    assert_eq!(
        script_teaching::diagnostic(&error),
        Diagnostic::Syntax,
        "{error}"
    );
    let out = shell::run(source, case);
    assert_eq!(out.error.as_deref(), Some(error.as_str()));
    assert_eq!(out.status, 2);
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
    assert_eq!(out.files, case.files);
}
