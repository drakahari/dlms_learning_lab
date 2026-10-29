//! Fullscript13 never silently expands the narrower scheduling execution contracts.
use dlms_learning_lab::shell::{self, Case};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "Closed scheduling boundary".into(),
        args: vec![],
        env: BTreeMap::new(),
        stdin: String::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "Untouched\n".into())]),
        stdout: "done\n".into(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::from([("/work/keep.txt".into(), "Untouched\n".into())]),
    }
}
#[test]
fn full_script_ownloops_work_but_cron_and_oneoff_still_reject_their_control_syntax() {
    assert_eq!(shell::EVALUATOR_REVISION, 13);
    let c = case();
    for source in [
        "for item in a b; do break; done; printf 'done\\n'",
        "for item in a b; do continue; done; printf 'done\\n'",
        "while true; do break; done; printf 'done\\n'",
    ] {
        assert!(c.accepts(&shell::run(source, &c)));
        assert!(
            shell::cron::prepare(source)
                .unwrap_err()
                .contains("Unsupported")
        );
        assert!(shell::cron::start(source, &c).is_err());
        assert!(
            shell::one_off::validate(source)
                .unwrap_err()
                .contains("Unsupported")
        );
        assert!(shell::one_off::start(source, &c).is_err());
    }
    assert_eq!(c.files, c.after);
}
#[test]
fn scheduler_gate_cannot_hide_loopcontrol_inside_functions_substitutions_conditions_or_redirects() {
    let c = case();
    for source in [
        "job() { for item in a; do break; done; }; job",
        "printf '%s' \"$(for item in a; do continue; done)\"",
        "if true; then for item in a; do break; done; fi",
        "for item in a; do break; done > keep.txt",
        "break > keep.txt",
        "continue > keep.txt",
    ] {
        assert!(shell::cron::prepare(source).is_err(), "{source}");
        assert!(shell::cron::start(source, &c).is_err(), "{source}");
        assert!(shell::one_off::validate(source).is_err(), "{source}");
        assert!(shell::one_off::start(source, &c).is_err(), "{source}");
    }
    assert_eq!(c.files, c.after);
}
#[test]
fn quoted_words_named_break_continue_remain_literal_data_in_both_schedulers() {
    let source = "printf 'break continue\\n'";
    let mut c = case();
    c.stdout = "break continue\n".into();
    assert!(c.accepts(&shell::run(source, &c)));
    assert!(shell::cron::prepare(source).is_ok());
    let mut cron = shell::cron::start(source, &c).unwrap();
    let mut one = shell::one_off::start(source, &c).unwrap();
    for _ in 0..100 {
        if !cron.done() {
            cron.tick(8)
        }
        if !one.done() {
            one.tick(8)
        }
    }
    assert!(cron.done() && one.done());
    assert!(c.accepts(&cron.outcome));
    assert!(c.accepts(&one.outcome));
}
