use dlms_learning_lab::{
    mailbox_access::{Facts, Flag, Issue, Runtime},
    mailbox_protocol::{Code, MAX_COMMANDS, MAX_OUTPUT, Phase, Session},
};
#[path = "support/mailbox_fixtures.rs"]
mod fixture;
use fixture::*;

fn session() -> (Facts, Runtime, Session) {
    let f = facts();
    let r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let (s, greeting) = Session::open(&r, &f, &client(true)).unwrap();
    assert_eq!(greeting.text, "* OK Simulated Dovecot ready\r\n");
    (f, r, s)
}
fn send(s: &mut Session, r: &mut Runtime, f: &Facts, command: &str) -> String {
    let reply = s.command(r, f, command);
    assert_eq!(reply.code, Code::Ok, "{command}: {reply:?}");
    reply.text
}
fn login(s: &mut Session, r: &mut Runtime, f: &Facts) {
    send(s, r, f, "a1 LOGIN alex sample-alex");
}
fn select(s: &mut Session, r: &mut Runtime, f: &Facts) {
    login(s, r, f);
    send(s, r, f, "a2 SELECT INBOX");
}

#[test]
fn login_retry_and_access_states_are_distinct() {
    let (f, mut r, mut s) = session();
    assert_eq!(
        s.command(&mut r, &f, "a1 SELECT INBOX").issue,
        Some(Issue::Authentication)
    );
    assert_eq!(s.command(&mut r, &f, "a2 LOGIN alex wrong").code, Code::No);
    assert!(matches!(s.phase, Phase::Imap { user: None, .. }));
    login(&mut s, &mut r, &f);
    assert_eq!(
        s.command(&mut r, &f, "a3 FETCH 1 FLAGS").issue,
        Some(Issue::Mailbox)
    );
    assert_eq!(
        s.command(&mut r, &f, "a4 LOGIN sam sample-sam").code,
        Code::Bad
    );
    send(&mut s, &mut r, &f, "a5 SELECT inbox");
    assert!(send(&mut s, &mut r, &f, "a6 FETCH 1 UID").contains("* 1 FETCH (UID 101)"));
}

#[test]
fn quoted_credentials_are_parsed_instead_of_split_on_spaces() {
    let mut f = facts();
    f.accounts.get_mut("alex").unwrap().sample_password = "sample\"alex\\key".into();
    f.validate().unwrap();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let (mut s, _) = Session::open(&r, &f, &client(true)).unwrap();
    send(&mut s, &mut r, &f, r#"a1 LOGIN "alex" "sample\"alex\\key""#);
    assert!(matches!(s.phase, Phase::Imap { user: Some(_), .. }));
}

#[test]
fn capability_and_starttls_reflect_policy_without_claiming_full_conformance() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let (mut s, _) = Session::open(&r, &f, &client(false)).unwrap();
    let caps = send(&mut s, &mut r, &f, "a1 CAPABILITY");
    assert!(caps.contains("X-SIMULATOR STARTTLS LOGINDISABLED"));
    assert!(!caps.contains("IMAP4rev1"));
    assert_eq!(
        s.command(&mut r, &f, "a2 LOGIN alex sample-alex").issue,
        Some(Issue::Authentication)
    );
    let tls = s.command(&mut r, &f, "a3 STARTTLS");
    assert_eq!(tls.code, Code::Ok);
    assert_eq!(tls.text, "a3 OK Begin TLS negotiation now\r\n");
    assert!(
        tls.client_note
            .unwrap()
            .contains("certificate checks passed")
    );
    let caps = send(&mut s, &mut r, &f, "a4 CAPABILITY");
    assert!(!caps.contains("STARTTLS"));
    assert!(!caps.contains("LOGINDISABLED"));
    login(&mut s, &mut r, &f);
    assert_eq!(s.command(&mut r, &f, "a5 STARTTLS").code, Code::Bad);
}

#[test]
fn failed_client_tls_verification_is_separate_from_server_starttls_acceptance() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(false);
    c.trusted_issuers.clear();
    let (mut s, _) = Session::open(&r, &f, &c).unwrap();
    let reply = s.command(&mut r, &f, "a1 STARTTLS");
    assert_eq!(reply.text, "a1 OK Begin TLS negotiation now\r\n");
    assert_eq!(reply.code, Code::Closed);
    assert!(reply.client_note.unwrap().contains("does not trust"));
    assert_eq!(
        s.command(&mut r, &f, "a2 LOGIN alex sample-alex").code,
        Code::Closed
    );
}

#[test]
fn select_metadata_and_flat_list_derive_from_the_authenticated_store() {
    let (mut f, _, _) = session();
    let mut archive = f.mailboxes["alex"]["INBOX"].clone();
    archive.messages.clear();
    f.mailboxes
        .get_mut("alex")
        .unwrap()
        .insert("Work archive".into(), archive);
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let (mut s, _) = Session::open(&r, &f, &client(true)).unwrap();
    login(&mut s, &mut r, &f);
    let list = send(&mut s, &mut r, &f, "a2 LIST \"\" \"*\"");
    assert!(list.contains("* LIST () NIL \"Work archive\"\r\n"));
    let selected = send(&mut s, &mut r, &f, "a3 SELECT INBOX");
    for part in [
        "* 2 EXISTS",
        "* 0 RECENT",
        "[UIDVALIDITY 77]",
        "[UIDNEXT 110]",
        "[UNSEEN 1]",
        "[READ-WRITE]",
    ] {
        assert!(selected.contains(part), "{part}");
    }
    send(&mut s, &mut r, &f, "a4 SELECT \"Work archive\"");
    assert!(
        send(&mut s, &mut r, &f, "a5 UID FETCH 1:* FLAGS").ends_with("a5 OK UID completed\r\n")
    );
    assert_eq!(s.command(&mut r, &f, "a6 FETCH 1 FLAGS").code, Code::Bad);
}

#[test]
fn failed_select_deselects_the_old_mailbox_without_expunging_it() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a3 STORE 1 +FLAGS (\Deleted)");
    assert_eq!(s.command(&mut r, &f, "a4 SELECT Missing").code, Code::No);
    assert!(matches!(
        s.phase,
        Phase::Imap {
            user: Some(_),
            selected: None
        }
    ));
    assert_eq!(s.command(&mut r, &f, "a5 EXPUNGE").code, Code::No);
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
}

#[test]
fn peek_and_read_fetch_differ_and_imap_literals_are_not_pop_dot_stuffed() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    let raw = r.mailbox(&f, "alex", "INBOX").unwrap().messages[0].raw();
    let peek = send(
        &mut s,
        &mut r,
        &f,
        "a3 FETCH 1 (UID BODY.PEEK[] RFC822.SIZE)",
    );
    assert!(peek.contains(&format!("BODY[] {{{}}}\r\n{raw}", raw.len())));
    assert!(peek.contains("\r\n.one line\r\n"));
    assert!(!peek.contains("\r\n..one line\r\n"));
    assert!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .is_empty()
    );
    let read = send(&mut s, &mut r, &f, "a4 FETCH 1 BODY[]");
    assert!(read.contains(r"FLAGS (\Seen)"));
    assert!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .contains(&Flag::Seen)
    );
    assert!(
        r.mailbox(&f, "sam", "INBOX").unwrap().messages[0]
            .flags
            .is_empty()
    );
}

#[test]
fn examine_never_changes_seen_or_deletion_and_close_does_not_expunge() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a3 STORE 1 +FLAGS (\Deleted)");
    assert!(send(&mut s, &mut r, &f, "a4 EXAMINE INBOX").contains("[READ-ONLY]"));
    let before = r.store.clone();
    send(&mut s, &mut r, &f, "a5 FETCH 1 BODY[]");
    assert_eq!(
        s.command(&mut r, &f, r"a6 STORE 1 +FLAGS (\Seen)").code,
        Code::No
    );
    assert_eq!(s.command(&mut r, &f, "a7 EXPUNGE").code, Code::No);
    send(&mut s, &mut r, &f, "a8 CLOSE");
    assert_eq!(r.store, before);
    assert!(matches!(s.phase, Phase::Imap { selected: None, .. }));
}

#[test]
fn store_alternatives_replace_add_remove_and_silent_have_actual_effects() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    let response = send(&mut s, &mut r, &f, r"a3 STORE 1 +FLAGS (\Seen \Deleted)");
    assert!(response.contains(r"FLAGS (\Seen \Deleted)"));
    let silent = send(
        &mut s,
        &mut r,
        &f,
        r"a4 UID STORE 101 -FLAGS.SILENT (\Deleted)",
    );
    assert_eq!(silent, "a4 OK UID completed\r\n");
    assert_eq!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [Flag::Seen]
    );
    send(&mut s, &mut r, &f, r"a5 STORE 1 FLAGS (\Deleted)");
    assert_eq!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [Flag::Deleted]
    );
    send(&mut s, &mut r, &f, "a6 STORE 1 FLAGS ()");
    assert!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .is_empty()
    );
}

#[test]
fn expunge_renumbers_sequence_but_uid_and_uidnext_remain_stable() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a3 UID STORE 101 +FLAGS \Deleted");
    assert!(send(&mut s, &mut r, &f, "a4 EXPUNGE").contains("* 1 EXPUNGE\r\n"));
    assert!(
        send(&mut s, &mut r, &f, "a5 UID FETCH 107 FLAGS").contains("* 1 FETCH (FLAGS () UID 107)")
    );
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().uid_next, 110);
    assert_eq!(
        send(&mut s, &mut r, &f, "a6 UID FETCH 101 UID"),
        "a6 OK UID completed\r\n"
    );
    send(&mut s, &mut r, &f, r"a7 STORE 1 +FLAGS \Deleted");
    let response = send(&mut s, &mut r, &f, "a8 CLOSE");
    assert!(!response.contains("EXPUNGE"));
    assert!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.is_empty());
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().uid_next, 110);
}

#[test]
fn logout_select_and_disconnect_do_not_implicitly_expunge() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a3 STORE 1 +FLAGS \Deleted");
    send(&mut s, &mut r, &f, "a4 SELECT INBOX");
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
    let out = send(&mut s, &mut r, &f, "a5 LOGOUT");
    assert!(out.starts_with("* BYE"));
    assert!(s.link.closed);
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
    let (mut s, _) = Session::open(&r, &f, &client(true)).unwrap();
    select(&mut s, &mut r, &f);
    s.disconnect();
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().messages.len(), 2);
    assert!(
        r.mailbox(&f, "alex", "INBOX").unwrap().messages[0]
            .flags
            .contains(&Flag::Deleted)
    );
}

#[test]
fn sets_are_bounded_deduplicated_and_uid_ranges_do_not_expand_integers() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    let out = send(&mut s, &mut r, &f, "a3 FETCH 2:1,1 UID");
    assert_eq!(out.matches(" FETCH (").count(), 2);
    let out = send(&mut s, &mut r, &f, "a4 UID FETCH 4294967295:1 UID");
    assert_eq!(out.matches(" FETCH (").count(), 2);
    for set in ["0", "3", "1:2:3", "1,,2", "4294967296", "-1"] {
        assert_eq!(
            s.command(&mut r, &f, &format!("a5 FETCH {set} UID")).code,
            Code::Bad,
            "{set}"
        );
    }
}

#[test]
fn syntax_and_unsupported_features_are_honest_and_never_mutate_mail() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    let before = r.store.clone();
    for line in [
        "a3 FETCH 1 BODY[HEADER]",
        r"a4 STORE 1 +FLAGS (\Answered)",
        "a5 APPEND INBOX",
        "a6 UID SEARCH ALL",
        "a7 bash -c id",
        "a8 LOGIN {4}",
        "a9 FETCH 1 ((FLAGS))",
    ] {
        let out = s.command(&mut r, &f, line);
        assert_eq!(out.code, Code::Unsupported, "{line}: {out:?}");
    }
    for line in [
        "a3 FETCH 1 (FLAGS",
        "a4 STORE 1 +FLAGS (\\Seen))",
        "a5 LOGIN \"unterminated",
        "a6 FETCH 1 \"FLAGS\"",
        "a7 SELECT \"INBOX\"extra",
        "* NOOP",
        "+ NOOP",
        "a8 NOOP\nLOGOUT",
    ] {
        let out = s.command(&mut r, &f, line);
        assert_eq!(out.code, Code::Bad, "{line}: {out:?}");
    }
    assert_eq!(r.store, before);
}

#[test]
fn output_limit_is_atomic_even_when_fetch_would_set_seen_or_close_would_expunge() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    let before = r.store.clone();
    s.output_bytes = MAX_OUTPUT - 1;
    assert_eq!(s.command(&mut r, &f, "a3 FETCH 1 BODY[]").code, Code::Limit);
    assert_eq!(r.store, before);
    assert!(s.link.closed);
    let (mut s, _) = Session::open(&r, &f, &client(true)).unwrap();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a4 STORE 1 +FLAGS \Deleted");
    let before = r.store.clone();
    s.output_bytes = MAX_OUTPUT - 1;
    assert_eq!(s.command(&mut r, &f, "a5 CLOSE").code, Code::Limit);
    assert_eq!(r.store, before);
}

#[test]
fn restart_and_command_limits_close_sessions_without_expunge() {
    let (f, mut r, mut s) = session();
    select(&mut s, &mut r, &f);
    send(&mut s, &mut r, &f, r"a3 STORE 1 +FLAGS \Deleted");
    let before = r.store.clone();
    r.restart(&f).unwrap();
    assert_eq!(s.command(&mut r, &f, "a4 EXPUNGE").code, Code::Closed);
    assert_eq!(r.store, before);
    let (mut s, _) = Session::open(&r, &f, &client(true)).unwrap();
    select(&mut s, &mut r, &f);
    s.commands = MAX_COMMANDS;
    assert_eq!(s.command(&mut r, &f, "a5 EXPUNGE").code, Code::Limit);
    assert_eq!(r.store, before);
}
