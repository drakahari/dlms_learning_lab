use dlms_learning_lab::{
    mailbox_access::{self as m, Flag, Issue, Runtime},
    mailbox_config::Config,
};
#[path = "support/mailbox_fixtures.rs"]
mod fixture;
use fixture::*;
#[test]
fn save_active_restart_and_enabled_state_are_distinct_and_mail_survives_failures() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), true).unwrap();
    let before = r.store.clone();
    let link = r.connect(&f, &client(true)).unwrap();
    r.save("broken configuration").unwrap();
    assert!(r.running);
    assert!(r.connect(&f, &client(true)).is_ok());
    assert_eq!(r.restart(&f).unwrap_err().issue, Issue::Syntax);
    assert!(!r.running);
    assert!(r.enabled);
    assert!(r.active.is_none());
    assert_eq!(r.store, before);
    assert!(link.current(&r).is_err());
    r.save(CONFIG).unwrap();
    r.restart(&f).unwrap();
    assert!(r.running);
    assert!(r.error.is_none());
    assert_eq!(r.store, before);
    assert_eq!(r.logs.len(), 2);
    assert!(r.logs[0].message.contains("Failed"));
    assert!(r.logs[1].message.contains("Started"));
}
#[test]
fn listener_modes_and_remote_cleartext_authentication_have_real_distinct_failures() {
    let f = facts();
    let r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(false);
    let plain = r.connect(&f, &c).unwrap();
    assert_eq!(
        r.authenticate(&f, &plain, "alex", "sample-alex")
            .unwrap_err()
            .issue,
        Issue::Authentication
    );
    c.port = 993;
    assert_eq!(r.connect(&f, &c).unwrap_err().issue, Issue::Tls);
    c = client(true);
    c.port = 143;
    assert_eq!(r.connect(&f, &c).unwrap_err().issue, Issue::Tls);
    c.port = 9999;
    assert_eq!(r.connect(&f, &c).unwrap_err().issue, Issue::Connection);
    let mut tls = plain;
    tls.start_tls(&r, &f).unwrap();
    r.authenticate(&f, &tls, "alex", "sample-alex").unwrap();
    assert_eq!(tls.start_tls(&r, &f).unwrap_err().issue, Issue::Tls);
}
#[test]
fn authentication_uses_the_supplied_identity_not_header_labels_or_canned_success() {
    let f = facts();
    let r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let l = r.connect(&f, &client(true)).unwrap();
    r.authenticate(&f, &l, "alex", "sample-alex").unwrap();
    for (u, p) in [
        ("alex", "wrong"),
        ("sam", "sample-alex"),
        ("../../alex", "sample-alex"),
    ] {
        assert_eq!(
            r.authenticate(&f, &l, u, p).unwrap_err().issue,
            Issue::Authentication
        );
    }
    let mut disabled = f.clone();
    disabled.accounts.get_mut("alex").unwrap().enabled = false;
    assert_eq!(
        r.authenticate(&disabled, &l, "alex", "sample-alex")
            .unwrap_err()
            .issue,
        Issue::Authentication
    );
}
#[test]
fn certificate_identity_trust_name_and_validity_are_derived_without_host_keys() {
    let f = facts();
    let r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let mut c = client(true);
    c.trusted_issuers.clear();
    assert_eq!(r.connect(&f, &c).unwrap_err().issue, Issue::Tls);
    c = client(true);
    c.server_name = "different.example.test".into();
    assert_eq!(r.connect(&f, &c).unwrap_err().issue, Issue::Tls);
    for day in [90, 110] {
        let mut f = f.clone();
        f.day = day;
        assert!(r.connect(&f, &client(true)).is_ok());
    }
    for day in [89, 111] {
        let mut f = f.clone();
        f.day = day;
        assert_eq!(r.connect(&f, &client(true)).unwrap_err().issue, Issue::Tls);
        assert!(r.running);
    }
    let mut link = r.connect(&f, &client(false)).unwrap();
    let mut bad = client(false);
    bad.trusted_issuers.clear();
    link.client = bad;
    assert!(link.start_tls(&r, &f).is_err());
    assert!(link.closed);
}
#[test]
fn key_mismatch_is_a_startup_problem_but_expiration_is_a_client_verification_problem() {
    let mut f = facts();
    f.keys.get_mut("/etc/dovecot/mail.key").unwrap().key_id = "other-key".into();
    let mut r = Runtime::initial(&f, CONFIG, None, true).unwrap();
    assert_eq!(r.restart(&f).unwrap_err().issue, Issue::Service);
    assert!(!r.running);
    assert!(r.enabled);
    f.keys.get_mut("/etc/dovecot/mail.key").unwrap().key_id = "sample-key".into();
    f.day = 111;
    r.restart(&f).unwrap();
    assert!(r.running);
    assert_eq!(r.connect(&f, &client(true)).unwrap_err().issue, Issue::Tls);
    let missing = CONFIG.replace("/etc/dovecot/mail.pem", "/etc/shadow");
    assert!(Config::parse(&missing).is_ok());
    r.save(&missing).unwrap();
    assert_eq!(r.restart(&f).unwrap_err().issue, Issue::Unsupported);
    assert!(!r.running);
}
#[test]
fn duplicate_active_ports_fail_at_service_start_not_at_configuration_syntax() {
    let f = facts();
    let bad =
        format!("{CONFIG}service imap-login {{\n inet_listener imap {{\n port = 110\n }}\n}}\n");
    assert!(Config::parse(&bad).is_ok());
    let mut r = Runtime::initial(&f, &bad, None, false).unwrap();
    assert_eq!(r.restart(&f).unwrap_err().issue, Issue::Service);
    assert!(!r.running);
    let disabled = CONFIG.replace("protocols = imap pop3", "protocols = pop3");
    let r = Runtime::initial(&f, &disabled, Some(&disabled), false).unwrap();
    assert_eq!(
        r.connect(&f, &client(true)).unwrap_err().issue,
        Issue::Connection
    );
}
#[test]
fn virtual_mail_location_resolves_for_each_user_and_accepts_equivalent_paths() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    assert_eq!(r.mailbox(&f, "alex", "inbox").unwrap().messages.len(), 2);
    let equivalent = CONFIG.replace("/srv/mail/%{user}/Maildir", "~/Maildir");
    r.save(&equivalent).unwrap();
    r.restart(&f).unwrap();
    for user in ["alex", "sam"] {
        assert!(r.mailbox(&f, user, "INBOX").is_ok());
    }
    let hardcoded = CONFIG.replace("/srv/mail/%{user}/Maildir", "/srv/mail/alex/Maildir");
    r.save(&hardcoded).unwrap();
    r.restart(&f).unwrap();
    assert!(r.mailbox(&f, "alex", "INBOX").is_ok());
    let e = r.mailbox(&f, "sam", "INBOX").unwrap_err();
    assert_eq!(e.issue, Issue::Mailbox);
    assert!(e.message.contains("mail_path"));
}
#[test]
fn expunge_renumbers_sequences_but_never_reuses_uidnext_and_restart_keeps_flags() {
    let f = facts();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), true).unwrap();
    let b = r.mailbox_mut(&f, "alex", "INBOX").unwrap();
    for m in &mut b.messages {
        m.flags.insert(Flag::Deleted);
    }
    assert_eq!(b.expunge(), [1, 1]);
    assert!(b.messages.is_empty());
    assert_eq!((b.uid_validity, b.uid_next), (77, 110));
    r.restart(&f).unwrap();
    assert_eq!(r.mailbox(&f, "alex", "INBOX").unwrap().uid_next, 110);
    assert!(
        r.mailbox(&f, "sam", "INBOX")
            .unwrap()
            .messages
            .iter()
            .all(|m| m.flags.is_empty())
    );
}
#[test]
fn message_wire_size_uses_canonical_crlf_and_rejects_control_or_uid_corruption() {
    let m = message(101);
    m.validate().unwrap();
    assert_eq!(m.raw(), "Subject: Sample101\r\n\r\nHello\r\n.one line\r\n");
    let mut same = m.clone();
    same.body = "Hello\r\n.one line\r\n".into();
    assert_eq!(same.raw(), m.raw());
    let mut bad = facts();
    bad.mailboxes
        .get_mut("alex")
        .unwrap()
        .get_mut("INBOX")
        .unwrap()
        .uid_next = 107;
    assert!(bad.validate().is_err());
    let mut bad = m.clone();
    bad.body = "bad\0body".into();
    assert_eq!(bad.validate().unwrap_err().issue, Issue::Unsupported);
    bad.body = "a".repeat(2049);
    assert_eq!(bad.validate().unwrap_err().issue, Issue::Limit);
}
#[test]
fn malformed_facts_and_oversized_saved_files_fail_without_losing_a_previous_configuration() {
    let f = facts();
    f.validate().unwrap();
    let mut r = Runtime::initial(&f, CONFIG, Some(CONFIG), false).unwrap();
    let saved = r.saved.clone();
    assert_eq!(r.save(&"x".repeat(8193)).unwrap_err().issue, Issue::Limit);
    assert_eq!(r.saved, saved);
    let mut bad = f.clone();
    bad.accounts.get_mut("alex").unwrap().home = "/srv/../secret".into();
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.certificates
        .get_mut("/etc/dovecot/mail.pem")
        .unwrap()
        .dns_names = vec!["*.example.test".into()];
    assert!(bad.validate().is_err());
    assert!(!m::dns("192.0.2.10"));
    assert!(!m::mailbox_name("../other"));
}
