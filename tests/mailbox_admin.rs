// Utility capability tests; not authored curriculum or external Dovecot acceptance.
use dlms_learning_lab::{
    mailbox_access::{Flag, Issue, Runtime},
    mailbox_admin as admin,
};
#[path = "support/mailbox_fixtures.rs"]
mod fixture;
use fixture::*;
fn runtime() -> Runtime {
    Runtime::initial(&facts(), CONFIG, Some(CONFIG), true).unwrap()
}
#[test]
fn named_configuration_queries_show_saved_values_without_changing_the_running_server() {
    let f = facts();
    let mut r = runtime();
    let original = r.active.clone();
    let link = r.connect(&f, &client(true)).unwrap();
    r.save(&CONFIG.replace("ssl = required", "ssl = yes"))
        .unwrap();
    let out = admin::run(&mut r, &f, "doveconf ssl").unwrap();
    assert_eq!(out.text, "ssl = yes\n");
    assert!(!out.changed);
    assert_eq!(r.active, original);
    assert!(link.current(&r).is_ok());
    assert_eq!(
        admin::run(&mut r, &f, "doveconf -h ssl").unwrap().text,
        "yes\n"
    );
    assert_eq!(
        admin::run(&mut r, &f, "doveconf 'ssl' \"mail_driver\"")
            .unwrap()
            .text,
        "ssl = yes\nmail_driver = maildir\n"
    );
}
#[test]
fn unsupported_dumps_shell_operators_and_bad_quotes_are_honest_and_atomic() {
    let f = facts();
    let mut r = runtime();
    let before = r.clone();
    for text in [
        "doveconf -n",
        "doveconf -a",
        "doveconf -c /etc/passwd ssl",
        "doveconf -h UNKNOWN",
        "doveadm -S socket search -u alex all",
        "doveconf ssl; whoami",
        "doveconf $(id)",
        "doveconf ssl | cat",
        "doveadm search -u alex mailbox INBOX OR seen unseen",
    ] {
        assert_eq!(
            admin::run(&mut r, &f, text).unwrap_err().issue,
            Issue::Unsupported,
            "{text}"
        );
        assert_eq!(r, before);
    }
    assert_eq!(
        admin::run(&mut r, &f, "doveconf 'ssl").unwrap_err().issue,
        Issue::Syntax
    );
    assert_eq!(
        admin::run(&mut r, &f, &"x".repeat(2049)).unwrap_err().issue,
        Issue::Limit
    );
    assert_eq!(r, before);
}
#[test]
fn mailbox_listing_and_queries_are_calculated_from_current_users_and_flags() {
    let f = facts();
    let mut r = runtime();
    let initial = r.store["alex"]["INBOX"].messages.clone();
    assert_eq!(initial.len(), 2);
    let out = admin::run(&mut r, &f, "doveadm mailbox list -u alex").unwrap();
    assert!(out.text.lines().any(|s| s == "INBOX"));
    let id = initial[0].uid;
    let search = format!("doveadm search -u alex mailbox INBOX uid {id} unseen");
    let out = admin::run(&mut r, &f, &search).unwrap();
    assert_eq!(out.text.lines().count(), 1);
    assert!(out.text.ends_with(&format!(" {id}\n")));
    r.store
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .messages[0]
        .flags
        .insert(Flag::Seen);
    assert!(admin::run(&mut r, &f, &search).unwrap().text.is_empty());
    assert_eq!(
        admin::run(&mut r, &f, &search.replace("unseen", "seen"))
            .unwrap()
            .text
            .lines()
            .count(),
        1
    );
}
#[test]
fn expunge_commits_only_query_matches_and_never_fabricates_a_pop_commit() {
    let f = facts();
    let mut r = runtime();
    let before = r.store.clone();
    let id = before["alex"]["INBOX"].messages[0].uid;
    let line = format!("doveadm expunge -u alex mailbox INBOX uid {id}");
    let out = admin::run(&mut r, &f, &line).unwrap();
    assert!(out.changed);
    assert!(out.text.is_empty());
    assert!(r.pop_committed.is_empty());
    assert_eq!(
        r.store["alex"]["INBOX"].messages,
        before["alex"]["INBOX"].messages[1..]
    );
    assert_eq!(r.store["sam"], before["sam"]);
    assert!(!admin::run(&mut r, &f, &line).unwrap().changed);
}
#[test]
fn reversed_and_giant_uid_ranges_use_bounded_message_matching() {
    let f = facts();
    let mut r = runtime();
    let a = admin::run(
        &mut r,
        &f,
        "doveadm search -u alex mailbox INBOX uid 1:4294967295",
    )
    .unwrap();
    let b = admin::run(
        &mut r,
        &f,
        "doveadm search -u alex mailbox INBOX uid 4294967295:1",
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.text.lines().count(), 2);
    for text in [
        "doveadm search -u alex uid 0",
        "doveadm search -u alex uid 2::3",
    ] {
        assert_eq!(
            admin::run(&mut r, &f, text).unwrap_err().issue,
            Issue::Syntax
        );
    }
}
#[test]
fn fetch_peeks_without_marking_seen_and_paths_are_only_unknown_virtual_names() {
    let f = facts();
    let mut r = runtime();
    let before = r.clone();
    let id = r.store["alex"]["INBOX"].messages[0].uid;
    let line = format!("doveadm fetch -u alex 'uid flags hdr.subject body' mailbox INBOX uid {id}");
    let out = admin::run(&mut r, &f, &line).unwrap();
    assert!(out.text.contains(&format!("uid: {id}\n")));
    assert!(out.text.contains("hdr.subject:"));
    assert!(!out.changed);
    assert_eq!(r, before);
    assert_eq!(
        admin::run(
            &mut r,
            &f,
            "doveadm search -u alex mailbox ../../etc/passwd all"
        )
        .unwrap_err()
        .issue,
        Issue::Mailbox
    );
    assert_eq!(r, before);
}

#[test]
fn valid_but_unmodeled_uid_star_and_quoted_control_bytes_are_diagnosed_without_mutation() {
    let f = facts();
    let mut r = runtime();
    let before = r.clone();
    for line in [
        "doveadm search -u alex uid *",
        "doveadm search -u alex mailbox IN* all",
        "doveconf \"bad\u{1b}key\"",
    ] {
        assert_eq!(
            admin::run(&mut r, &f, line).unwrap_err().issue,
            Issue::Unsupported
        );
        assert_eq!(r, before);
    }
}

#[test]
fn large_fetch_output_is_bounded_and_retains_the_entire_previous_runtime() {
    let mut f = facts();
    let inbox = f
        .mailboxes
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap();
    inbox.uid_next = 100;
    inbox.messages = (1..=16)
        .map(|uid| {
            let mut m = message(uid);
            m.body = "x".repeat(2048);
            m
        })
        .collect();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), true).unwrap();
    let before = r.clone();
    assert_eq!(
        admin::run(
            &mut r,
            &f,
            "doveadm fetch -u alex 'body body body body body body body body' mailbox INBOX all"
        )
        .unwrap_err()
        .issue,
        Issue::Limit
    );
    assert_eq!(r, before);
}

#[test]
fn signed_uids_are_syntax_errors_and_empty_subject_is_explicitly_unsupported() {
    let f = facts();
    let mut r = runtime();
    let before = r.clone();
    assert_eq!(
        admin::run(&mut r, &f, "doveadm search -u alex uid +1")
            .unwrap_err()
            .issue,
        Issue::Syntax
    );
    assert_eq!(
        admin::run(&mut r, &f, "doveadm search -u alex subject ''")
            .unwrap_err()
            .issue,
        Issue::Unsupported
    );
    assert_eq!(
        admin::run(&mut r, &f, "doveadm expunge -u unknown mailbox INBOX all")
            .unwrap_err()
            .issue,
        Issue::Authentication
    );
    assert_eq!(r, before);
}
#[test]
fn header_fetch_retains_every_matching_header_without_changing_mail() {
    let mut f = facts();
    let message = &mut f
        .mailboxes
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .messages[0];
    let uid = message.uid;
    let first = message.headers[0].1.clone();
    message
        .headers
        .push(("subject".into(), "Second supplied field".into()));
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), true).unwrap();
    let before = r.clone();
    let out = admin::run(
        &mut r,
        &f,
        &format!("doveadm fetch -u alex hdr.subject mailbox INBOX uid {uid}"),
    )
    .unwrap();
    assert_eq!(
        out.text,
        format!("hdr.subject: {first}\nSecond supplied field\n")
    );
    assert_eq!(r, before);
}
