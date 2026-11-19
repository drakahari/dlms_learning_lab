#[path = "support/smtp_tasks.rs"]
mod setup;
use dlms_learning_lab::{
    smtp_config::{Config, Document},
    smtp_console as console,
    smtp_workspace::{Action, Session},
};
fn run(s: &mut Session, d: &dlms_learning_lab::smtp_workspace::Definition, input: &str) {
    s.action(
        d,
        &Action::Run {
            input: input.into(),
        },
        false,
    )
    .unwrap();
}
#[test]
fn explicit_values_can_be_inspected_before_semantic_repair_and_saved_is_not_running() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    let old = s.assignment.runtime.active.clone();
    let bad = s.assignment.runtime.files["relay"].replace("[gate.lab.test]", "not-bracketed.test");
    s.action(
        &d,
        &Action::Edit {
            node: "relay".into(),
            text: bad.clone(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Save {
            node: "relay".into(),
        },
        false,
    )
    .unwrap();
    assert!(Config::parse(&bad).is_err());
    assert_eq!(
        Document::parse(&bad).unwrap().parameters["relayhost"],
        "not-bracketed.test"
    );
    run(&mut s, &d, "postconf -n");
    assert!(
        s.assignment
            .output
            .contains("relayhost = not-bracketed.test")
    );
    let r = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "relay",
        "postfix reload",
    );
    assert_eq!(r.status, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.contains("subset"));
    assert_eq!(s.assignment.runtime.active, old);
}
#[test]
fn queue_request_exit_success_is_distinct_from_delivery_and_repair_retries_real_sample_state() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment.selected = "gate".into();
    let listed = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "gate",
        "postqueue -p",
    );
    assert_eq!(listed.status, 0);
    assert!(listed.stdout.contains("nora@staff.lab.test"));
    assert!(listed.stdout.contains("MAILER-DAEMON"));
    assert!(listed.stdout.contains("Sample bytes"));
    let first = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "gate",
        "postqueue -f",
    );
    assert_eq!(first.status, 0);
    assert!(first.stdout.is_empty());
    assert!(first.observations.iter().any(|x| x.contains("cycles")));
    assert!(!d.current(&s));
    let listed = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "gate",
        "postqueue -p",
    );
    assert!(listed.stdout.contains("model relay path repeats"));
    for a in &d.reference[..2] {
        s.action(&d, a, false).unwrap();
    }
    s.assignment.selected = "relay".into();
    run(&mut s, &d, "postfix reload");
    s.assignment.selected = "gate".into();
    run(&mut s, &d, "postqueue -f");
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.current(&s));
    run(&mut s, &d, "postqueue -p");
    assert!(s.assignment.output.contains("Mail queue is empty"));
    assert!(d.current(&s));
    run(&mut s, &d, "postconf -n");
    assert!(d.current(&s));
}
#[test]
fn unsupported_shell_and_host_paths_never_mutate_state_or_claim_command_success() {
    let d = setup::definition();
    let s = Session::new(&d).unwrap();
    for input in [
        "sh -c whoami",
        "postqueue -f; reboot",
        "postconf -n | cat",
        "postconf -c /etc/postfix -n",
        "postfix check",
        "sudo postfix reload",
        "postqueue -f\npostqueue -p",
        "postqueue -i A0000000",
        "$(touch /tmp/not-allowed)",
        "postconf -n > /home/user/file",
    ] {
        let mut state = s.assignment.runtime.clone();
        let r = console::run(&d.assignment.facts, &mut state, "relay", input);
        assert_ne!(r.status, 0, "{input}");
        assert!(!r.changed);
        assert_eq!(state, s.assignment.runtime);
        assert!(!r.stderr.is_empty());
    }
    let mut state = s.assignment.runtime.clone();
    assert_eq!(
        console::run(
            &d.assignment.facts,
            &mut state,
            "../../host",
            "postqueue -f"
        )
        .status,
        2
    );
    assert_eq!(
        console::run(&d.assignment.facts, &mut state, "relay", &"x".repeat(4097)).status,
        2
    );
}
#[test]
fn command_output_history_remains_bounded_and_utf8_safe_and_scopes_stay_separate() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    let example = s.practice.clone();
    s.assignment.output = "é".repeat(16380);
    run(&mut s, &d, "postconf -n");
    assert!(s.assignment.output.len() <= 32768);
    assert!(s.assignment.output.ends_with("[command exit status: 0]\n"));
    assert_eq!(s.practice, example);
    d.bound(&s).unwrap();
}
#[test]
fn literal_document_preserves_continuations_duplicate_warnings_and_empty_values_without_inventing_defaults()
 {
    let d=Document::parse("relayhost = [first.test]\nrelayhost = [second.test]\nmydestination = one.test,\n two.test\nrelay_domains =\n").unwrap();
    assert_eq!(d.parameters["relayhost"], "[second.test]");
    assert_eq!(d.parameters["mydestination"], "one.test, two.test");
    assert_eq!(d.parameters["relay_domains"], "");
    assert_eq!(d.warnings.len(), 1);
    assert!(!d.parameters.contains_key("mynetworks"));
    assert!(Document::parse("unexpected = yes").is_err());
    assert!(Document::parse("relayhost = $myhostname").is_err());
}
#[test]
fn retrying_an_empty_or_different_owners_queue_never_claims_a_delivery_attempt() {
    let d = setup::definition();
    let mut s = Session::new(&d).unwrap();
    let before = s.assignment.runtime.clone();
    let result = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "relay",
        "postqueue -f",
    );
    assert_eq!(result.status, 0);
    assert!(!result.changed);
    assert_eq!(s.assignment.runtime, before);
    assert!(result.stdout.is_empty());
    assert!(
        result.observations[0].contains("No pending sample mail on relay; nothing was retried")
    );
    let result = console::run(
        &d.assignment.facts,
        &mut s.assignment.runtime,
        "gate",
        "postqueue -f",
    );
    assert!(result.changed);
    assert!(result.observations[0].contains("handled 2 pending sample(s) on gate"));
}
