use dlms_learning_lab::{
    mailbox_access::{Issue, Runtime},
    mailbox_config::Protocol,
    mailbox_protocol::{Code, MAX_COMMANDS, MAX_OUTPUT, Phase, Session},
};
#[path = "support/mailbox_fixtures.rs"]
mod fixture;
use fixture::*;
fn session() -> (dlms_learning_lab::mailbox_access::Facts, Runtime, Session) {
    let f = facts();
    let r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (s, g) = Session::open(&r, &f, &c).unwrap();
    assert!(g.text.starts_with("+OK"));
    (f, r, s)
}
fn login(s: &mut Session, r: &mut Runtime, f: &dlms_learning_lab::mailbox_access::Facts) {
    assert_eq!(s.command(r, f, "USER alex").code, Code::Ok);
    assert_eq!(s.command(r, f, "PASS sample-alex").code, Code::Ok);
}
#[test]
fn transaction_commands_require_authentication_and_retry_does_not_invent_a_user() {
    let (f, mut r, mut s) = session();
    assert_eq!(s.command(&mut r, &f, "STAT").code, Code::No);
    assert_eq!(s.command(&mut r, &f, "PASS sample-alex").code, Code::No);
    s.command(&mut r, &f, "USER alex");
    assert_eq!(s.command(&mut r, &f, "PASS wrong").code, Code::No);
    assert_eq!(s.command(&mut r, &f, "PASS sample-alex").code, Code::No);
    login(&mut s, &mut r, &f);
    assert!(s.command(&mut r, &f, "STAT").text.starts_with("+OK 2 "));
}
#[test]
fn dele_is_pending_rset_recovers_and_only_transaction_quit_commits() {
    let (f, mut r, mut s) = session();
    let initial = r.store.clone();
    login(&mut s, &mut r, &f);
    assert_eq!(s.command(&mut r, &f, "DELE 1").code, Code::Ok);
    assert_eq!(r.store, initial);
    assert!(s.command(&mut r, &f, "STAT").text.starts_with("+OK 1 "));
    assert_eq!(s.command(&mut r, &f, "RETR 1").code, Code::No);
    s.command(&mut r, &f, "RSET");
    assert!(s.command(&mut r, &f, "STAT").text.starts_with("+OK 2 "));
    s.command(&mut r, &f, "DELE 1");
    s.command(&mut r, &f, "QUIT");
    assert!(s.link.closed);
    let b = r.mailbox(&f, "alex", "INBOX").unwrap();
    assert_eq!(b.messages.iter().map(|m| m.uid).collect::<Vec<_>>(), [107]);
    assert_eq!(b.uid_next, 110);
    assert_eq!(r.mailbox(&f, "sam", "INBOX").unwrap().messages.len(), 2);
}
#[test]
fn only_successful_pop_update_records_transaction_credit_and_output_failure_is_atomic() {
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    assert!(r.pop_committed.is_empty());
    s.command(&mut r, &f, "RSET");
    s.command(&mut r, &f, "QUIT");
    assert!(r.pop_committed.is_empty());
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    s.output_bytes = MAX_OUTPUT - 1;
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Limit);
    assert!(r.pop_committed.is_empty());
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Ok);
    assert_eq!(
        r.pop_committed["alex"].iter().copied().collect::<Vec<_>>(),
        [101]
    );
    r.validate(&f).unwrap();
}
#[test]
fn disconnect_and_server_restart_never_commit_pending_pop_deletion() {
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    s.disconnect();
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Closed);
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    r.restart(&f).unwrap();
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Closed);
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
}
#[test]
fn retr_and_top_dot_stuff_but_list_sizes_count_only_canonical_message_octets() {
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    let bytes = r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
        .raw()
        .len();
    assert_eq!(
        s.command(&mut r, &f, "LIST 1").text,
        format!("+OK 1 {bytes}\r\n")
    );
    let retr = s.command(&mut r, &f, "RETR 1");
    assert!(retr.text.starts_with(&format!("+OK {bytes} octets\r\n")));
    assert!(retr.text.contains("\r\n..one line\r\n"));
    assert!(retr.text.ends_with(".\r\n"));
    let top = s.command(&mut r, &f, "TOP 1 0");
    assert!(top.text.contains("Subject: Sample101\r\n\r\n.\r\n"));
    assert!(!top.text.contains("Hello"));
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
}
#[test]
fn uidl_stays_stable_while_sequence_numbers_change_after_a_committed_deletion() {
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    let before = s.command(&mut r, &f, "UIDL 2");
    assert!(before.text.contains("2 sim-77-107"));
    s.command(&mut r, &f, "DELE 1");
    assert!(s.command(&mut r, &f, "UIDL").text.contains("2 sim-77-107"));
    s.command(&mut r, &f, "QUIT");
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    assert!(
        s.command(&mut r, &f, "UIDL 1")
            .text
            .contains("1 sim-77-107")
    );
}
#[test]
fn required_tls_rejects_user_before_stls_and_separates_server_reply_from_client_verification() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(false);
    c.protocol = Protocol::Pop3;
    c.port = 110;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    assert_eq!(s.command(&mut r, &f, "USER alex").code, Code::No);
    let tls = s.command(&mut r, &f, "STLS");
    assert_eq!(tls.code, Code::Ok);
    assert_eq!(tls.text, "+OK Begin TLS negotiation.\r\n");
    assert!(
        tls.client_note
            .unwrap()
            .contains("certificate checks passed")
    );
    assert_eq!(s.command(&mut r, &f, "PASS sample-alex").code, Code::No);
    login(&mut s, &mut r, &f);
    c.trusted_issuers.clear();
    let (mut bad, _) = Session::open(&r, &f, &c).unwrap();
    let out = bad.command(&mut r, &f, "STLS");
    assert_eq!(out.code, Code::Closed);
    assert!(out.text.starts_with("+OK Begin"));
    assert!(out.client_note.unwrap().contains("does not trust"));
    assert!(bad.link.closed);
}
#[test]
fn rejected_stls_does_not_discard_an_existing_user_candidate() {
    let (f, mut r, mut s) = session();
    assert_eq!(s.command(&mut r, &f, "USER alex").code, Code::Ok);
    assert_eq!(s.command(&mut r, &f, "STLS").code, Code::No);
    assert_eq!(s.command(&mut r, &f, "PASS sample-alex").code, Code::Ok);
}
#[test]
fn capa_and_pre_tls_user_follow_the_active_remote_policy() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(false);
    c.protocol = Protocol::Pop3;
    c.port = 110;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    let capa = s.command(&mut r, &f, "CAPA");
    assert!(capa.text.contains("\r\nSTLS\r\n"));
    assert!(!capa.text.contains("\r\nUSER\r\n"));
    let allowed = CONFIG
        .replace("ssl = required", "ssl = yes")
        .replace("auth_allow_cleartext = no", "auth_allow_cleartext = yes");
    r.save(&allowed).unwrap();
    r.restart(&f).unwrap();
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    assert!(s.command(&mut r, &f, "CAPA").text.contains("\r\nUSER\r\n"));
    assert_eq!(s.command(&mut r, &f, "USER alex").code, Code::Ok);
    assert_eq!(s.command(&mut r, &f, "STLS").code, Code::Ok);
    assert_eq!(s.command(&mut r, &f, "PASS sample-alex").code, Code::Ok);
}
#[test]
fn retr_updates_seen_by_default_while_top_and_the_explicit_no_updates_setting_do_not() {
    use dlms_learning_lab::mailbox_access::Flag;
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "TOP 1 1");
    assert!(
        !r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .contains(&Flag::Seen)
    );
    assert_eq!(s.command(&mut r, &f, "RETR 1").code, Code::Ok);
    assert!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .contains(&Flag::Seen)
    );
    assert!(
        !r.mailbox(&f, "sam", "INBOX").unwrap().messages[0]
            .flags
            .contains(&Flag::Seen)
    );
    let config = format!("{CONFIG}pop3_no_flag_updates = yes\n");
    r.save(&config).unwrap();
    r.restart(&f).unwrap();
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    assert_eq!(s.command(&mut r, &f, "RETR 2").code, Code::Ok);
    assert!(
        !r.mailbox(&f, "alex", "INBOX").unwrap().messages[1]
            .flags
            .contains(&Flag::Seen)
    );
}
#[test]
fn unsupported_and_malformed_commands_do_not_run_host_actions_or_change_the_store() {
    let (f, mut r, mut s) = session();
    let before = r.store.clone();
    for line in ["bash -c id", "APOP anything", "cat /etc/shadow"] {
        let out = s.command(&mut r, &f, line);
        assert_eq!(out.code, Code::Unsupported);
        assert_eq!(out.issue, Some(Issue::Unsupported));
    }
    for line in ["USER", "STAT extra", "USER alex\nPASS sample-alex"] {
        assert_eq!(s.command(&mut r, &f, line).code, Code::Bad);
    }
    assert_eq!(r.store, before);
}
#[test]
fn output_and_command_limits_abort_without_committing_a_pending_or_current_effect() {
    let (f, mut r, mut s) = session();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    s.output_bytes = MAX_OUTPUT - 1;
    let before = r.store.clone();
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Limit);
    assert_eq!(r.store, before);
    assert!(s.link.closed);
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    login(&mut s, &mut r, &f);
    s.command(&mut r, &f, "DELE 1");
    s.commands = MAX_COMMANDS;
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Limit);
    assert_eq!(r.store, before);
    if let Phase::Pop { deleted, .. } = s.phase {
        assert!(deleted.is_empty());
    } else {
        panic!()
    }
}
#[test]
fn even_syntax_error_output_is_bounded_and_authorization_quit_deletes_nothing() {
    let (f, mut r, mut s) = session();
    s.output_bytes = MAX_OUTPUT - 1;
    assert_eq!(s.command(&mut r, &f, "\n").code, Code::Limit);
    assert!(s.link.closed);
    let mut c = client(true);
    c.protocol = Protocol::Pop3;
    c.port = 995;
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    let before = r.store.clone();
    assert_eq!(s.command(&mut r, &f, "QUIT").code, Code::Ok);
    assert_eq!(r.store, before);
}
