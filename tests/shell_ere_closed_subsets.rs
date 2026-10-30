//! Capability14 cannot silently widen the two portable scheduling subsets.
use dlms_learning_lab::shell::{self, Case};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "ERE scheduling boundary".into(),
        args: vec![],
        stdin: "WARN\nINFO\n".into(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep".into(), "protected\n".into())]),
        stdout: "WARN\n".into(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::from([("/work/keep".into(), "protected\n".into())]),
    }
}
#[test]
fn actual_full_script_extended_selection_does_not_enter_cron_or_oneoff_execution() {
    assert_eq!(shell::EVALUATOR_REVISION, 15);
    let c = case();
    let source = "grep -E '^WARN$'";
    assert!(c.accepts(&shell::run(source, &c)));
    assert!(shell::cron::prepare(source).is_err());
    assert!(shell::cron::start(source, &c).is_err());
    assert!(shell::one_off::validate(source).is_err());
    assert!(shell::one_off::start(source, &c).is_err());
    assert_eq!(c.files, c.after);
}
#[test]
fn rejected_utility_cannot_truncate_virtual_files_through_a_scheduled_redirection() {
    let c = case();
    for source in [
        "grep -Eq '^WARN$' > keep",
        "printf '%s' \"$(grep -E '^WARN$')\" > keep",
        "if grep -Eq '^WARN$'; then printf bad > keep; fi",
    ] {
        assert!(shell::cron::prepare(source).is_err(), "{source}");
        assert!(shell::cron::start(source, &c).is_err(), "{source}");
        assert!(shell::one_off::validate(source).is_err(), "{source}");
        assert!(shell::one_off::start(source, &c).is_err(), "{source}");
    }
    assert_eq!(c.files, c.after);
}
#[test]
fn quoted_grep_looking_text_is_data_in_both_schedulers_not_a_hidden_utility() {
    let source = "printf '%s\\n' 'grep -E ^WARN$'";
    let mut c = case();
    c.stdout = "grep -E ^WARN$\n".into();
    assert!(c.accepts(&shell::run(source, &c)));
    // Cron processes percent before shell quoting; the original unescaped
    // format is still valid for the full VM/one-off subset, not for cron.
    assert!(shell::cron::prepare(source).is_err());
    assert!(shell::cron::start(source, &c).is_err());
    let cron_source = r"printf '\%s\n' 'grep -E ^WARN$'";
    let mut cron = shell::cron::start(cron_source, &c).unwrap();
    let mut one = shell::one_off::start(source, &c).unwrap();
    for _ in 0..100 {
        if !cron.done() {
            cron.tick(8);
        }
        if !one.done() {
            one.tick(8);
        }
    }
    assert!(cron.done() && one.done());
    assert!(c.accepts(&cron.outcome));
    assert!(c.accepts(&one.outcome));
    assert_eq!(cron.outcome.files, c.files);
    assert_eq!(one.outcome.files, c.files);
}

#[test]
fn literal_trim_is_full_vm_only_and_cannot_enter_either_scheduling_subset() {
    let mut c = case();
    c.stdout = "WARN\n".into();
    c.env.insert("v".into(), "preWARN".into());
    let source = "echo \"${v#pre}\"";
    assert!(c.accepts(&shell::run(source, &c)));
    for text in [source, "echo \"${v#pre}\" > keep"] {
        assert!(shell::cron::prepare(text).is_err());
        assert!(shell::cron::start(text, &c).is_err());
        assert!(shell::one_off::validate(text).is_err());
        assert!(shell::one_off::start(text, &c).is_err());
    }
    let plain = "echo \"$v\"";
    assert!(shell::cron::prepare(plain).is_ok());
    assert!(shell::cron::start(plain, &c).is_ok());
    assert!(shell::one_off::validate(plain).is_ok());
    assert!(shell::one_off::start(plain, &c).is_ok());
    assert_eq!(c.files, c.after);
}
