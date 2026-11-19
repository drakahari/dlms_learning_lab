use dlms_learning_lab::{
    mailbox_access::{Account, Certificate, Client, Facts, Key, Mailbox, Message},
    mailbox_config::Protocol,
};
use std::collections::{BTreeMap, BTreeSet};
pub const CONFIG: &str = "dovecot_config_version = 2.4.0\ndovecot_storage_version = 2.4.0\nprotocols = imap pop3\nssl = required\nauth_allow_cleartext = no\nauth_mechanisms = plain login\nssl_server_cert_file = /etc/dovecot/mail.pem\nssl_server_key_file = /etc/dovecot/mail.key\nmail_driver = maildir\nmail_path = /srv/mail/%{user}/Maildir\n";
pub fn message(uid: u32) -> Message {
    Message {
        uid,
        headers: vec![("Subject".into(), format!("Sample{uid}"))],
        body: "Hello\n.one line\n".into(),
        flags: BTreeSet::new(),
    }
}
pub fn facts() -> Facts {
    let accounts = ["alex", "sam"]
        .into_iter()
        .map(|u| {
            (
                u.into(),
                Account {
                    sample_password: format!("sample-{u}"),
                    enabled: true,
                    home: format!("/srv/mail/{u}"),
                    maildir_path: format!("/srv/mail/{u}/Maildir"),
                },
            )
        })
        .collect();
    let mailboxes = ["alex", "sam"]
        .into_iter()
        .map(|u| {
            (
                u.into(),
                BTreeMap::from([(
                    "INBOX".into(),
                    Mailbox {
                        uid_validity: 77,
                        uid_next: 110,
                        messages: vec![message(101), message(107)],
                    },
                )]),
            )
        })
        .collect();
    Facts {
        server_name: "mail.example.test".into(),
        day: 100,
        accounts,
        mailboxes,
        certificates: BTreeMap::from([(
            "/etc/dovecot/mail.pem".into(),
            Certificate {
                key_id: "sample-key".into(),
                issuer_id: "training-ca".into(),
                dns_names: vec!["mail.example.test".into()],
                valid_from_day: 90,
                valid_through_day: 110,
            },
        )]),
        keys: BTreeMap::from([(
            "/etc/dovecot/mail.key".into(),
            Key {
                key_id: "sample-key".into(),
                owner: "root".into(),
                mode: 0o600,
            },
        )]),
    }
}
pub fn client(tls: bool) -> Client {
    Client {
        protocol: Protocol::Imap,
        port: if tls { 993 } else { 143 },
        implicit_tls: tls,
        server_name: "mail.example.test".into(),
        trusted_issuers: ["training-ca".into()].into(),
    }
}
