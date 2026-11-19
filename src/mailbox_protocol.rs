//! Bounded in-process IMAP/POP dialogue. Every response follows current typed
//! server/session/mailbox state; no network, subprocess or external interpreter.
use crate::mailbox_access::{Client, Facts, Failure, Issue, Link, Runtime};
use crate::mailbox_config::{Protocol, Ssl};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_COMMAND: usize = 2048;
pub const MAX_COMMANDS: u16 = 128;
pub const MAX_OUTPUT: usize = 64 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Code {
    Ok,
    No,
    Bad,
    Unsupported,
    Limit,
    Closed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Reply {
    pub text: String,
    pub code: Code,
    pub issue: Option<Issue>,
    pub client_note: Option<String>,
}
impl Reply {
    fn pop(ok: bool, text: impl AsRef<str>) -> Self {
        Self {
            text: format!("{} {}\r\n", if ok { "+OK" } else { "-ERR" }, text.as_ref()),
            code: if ok { Code::Ok } else { Code::No },
            issue: None,
            client_note: None,
        }
    }
    fn imap(tag: &str, code: Code, text: impl AsRef<str>) -> Self {
        Self {
            text: format!(
                "{tag} {} {}\r\n",
                if code == Code::Ok {
                    "OK"
                } else if code == Code::No {
                    "NO"
                } else {
                    "BAD"
                },
                text.as_ref()
            ),
            code,
            issue: None,
            client_note: None,
        }
    }
    fn failure(protocol: Protocol, tag: &str, e: Failure) -> Self {
        let code = match e.issue {
            Issue::Unsupported => Code::Unsupported,
            Issue::Syntax => Code::Bad,
            _ => Code::No,
        };
        let mut r = match protocol {
            Protocol::Pop3 => Self::pop(false, &e.message),
            Protocol::Imap => Self::imap(tag, code, &e.message),
        };
        r.code = code;
        r.issue = Some(e.issue);
        r
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub mailbox: String,
    pub read_only: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Phase {
    Imap {
        user: Option<String>,
        selected: Option<Selection>,
    },
    Pop {
        candidate: Option<String>,
        user: Option<String>,
        uids: Vec<u32>,
        deleted: BTreeSet<u32>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub link: Link,
    pub phase: Phase,
    pub commands: u16,
    pub output_bytes: usize,
}
#[path = "mailbox_protocol/imap.rs"]
mod imap;
impl Session {
    pub fn open(r: &Runtime, f: &Facts, client: &Client) -> Result<(Self, Reply), Failure> {
        let link = r.connect(f, client)?;
        let phase = match client.protocol {
            Protocol::Imap => Phase::Imap {
                user: None,
                selected: None,
            },
            Protocol::Pop3 => Phase::Pop {
                candidate: None,
                user: None,
                uids: vec![],
                deleted: BTreeSet::new(),
            },
        };
        let greeting = match client.protocol {
            Protocol::Imap => Reply {
                text: "* OK Simulated Dovecot ready\r\n".into(),
                code: Code::Ok,
                issue: None,
                client_note: None,
            },
            Protocol::Pop3 => Reply::pop(true, "Simulated Dovecot ready"),
        };
        Ok((
            Self {
                link,
                phase,
                commands: 0,
                output_bytes: greeting.text.len(),
            },
            greeting,
        ))
    }
    pub fn disconnect(&mut self) {
        self.link.closed = true;
        if let Phase::Pop { deleted, .. } = &mut self.phase {
            deleted.clear();
        }
    }
    pub fn command(&mut self, r: &mut Runtime, f: &Facts, line: &str) -> Reply {
        if let Err(e) = self.link.current(r) {
            self.disconnect();
            return Reply {
                text: format!("[Simulator] {e}\r\n"),
                code: Code::Closed,
                issue: Some(e.issue),
                client_note: None,
            };
        }
        if self.commands >= MAX_COMMANDS || line.len() > MAX_COMMAND {
            self.disconnect();
            return Reply{text:"[Simulator] Command/source limit reached; this command was not applied. Pending POP deletions were not committed.\r\n".into(),code:Code::Limit,issue:Some(Issue::Limit),client_note:None};
        }
        self.commands += 1;
        let mut next = self.clone();
        let mut server = r.clone();
        let reply = if line.trim().is_empty()
            || !line
                .bytes()
                .all(|b| b.is_ascii_graphic() || b == b' ' || b == b'\t')
        {
            Reply{text:"[Simulator] Enter one nonblank ASCII protocol command, without embedded line breaks or control bytes.\r\n".into(),code:Code::Bad,issue:Some(Issue::Syntax),client_note:None}
        } else {
            match self.link.protocol {
                Protocol::Pop3 => next.pop(&mut server, f, line),
                Protocol::Imap => next.imap(&mut server, f, line),
            }
        };
        let bytes = reply.text.len() + reply.client_note.as_ref().map_or(0, String::len);
        if bytes > MAX_OUTPUT.saturating_sub(self.output_bytes) {
            self.disconnect();
            return Reply{text:"[Simulator] Output limit reached; no changes from this command were applied. Pending POP deletions were not committed.\r\n".into(),code:Code::Limit,issue:Some(Issue::Limit),client_note:None};
        }
        next.output_bytes += bytes;
        *self = next;
        *r = server;
        reply
    }
    fn pop(&mut self, r: &mut Runtime, f: &Facts, line: &str) -> Reply {
        let words: Vec<_> = line.split_ascii_whitespace().collect();
        let command = words[0].to_ascii_uppercase();
        let args = &words[1..];
        let Phase::Pop {
            candidate,
            user,
            uids,
            deleted,
        } = &mut self.phase
        else {
            unreachable!()
        };
        let arity = match command.as_str() {
            "USER" | "PASS" | "RETR" | "DELE" => Some(1),
            "TOP" => Some(2),
            "QUIT" | "NOOP" | "RSET" | "STAT" | "STLS" | "CAPA" => Some(0),
            "LIST" | "UIDL" if args.len() <= 1 => None,
            "LIST" | "UIDL" => Some(1),
            _ => None,
        };
        if let Some(n) = arity
            && args.len() != n
        {
            let mut out = Reply::pop(
                false,
                format!("{command} needs {n} argument(s) in this subset."),
            );
            out.code = Code::Bad;
            out.issue = Some(Issue::Syntax);
            return out;
        }
        if command == "QUIT" {
            if let Some(user) = user {
                match r.mailbox_mut(f, user, "INBOX") {
                    Ok(b) => b.messages.retain(|m| !deleted.contains(&m.uid)),
                    Err(e) => return Reply::failure(Protocol::Pop3, "", e),
                }
                if !deleted.is_empty() {
                    r.pop_committed
                        .entry(user.clone())
                        .or_default()
                        .extend(deleted.iter().copied());
                }
            }
            deleted.clear();
            self.link.closed = true;
            return Reply::pop(
                true,
                "Logging out; committed pending transaction deletions, if any.",
            );
        }
        if command == "CAPA" {
            let cfg = match r.active_config() {
                Ok(c) => c,
                Err(e) => return Reply::failure(Protocol::Pop3, "", e),
            };
            let mut text = "+OK Capability list follows\r\nUIDL\r\nTOP\r\n".to_owned();
            if cfg.permits_password(self.link.tls, false) && cfg.auth_mechanisms.contains("plain") {
                text.push_str("USER\r\n");
            }
            if !self.link.tls && user.is_none() && cfg.ssl != Ssl::No {
                text.push_str("STLS\r\n");
            }
            text.push_str(".\r\n");
            return Reply {
                text,
                code: Code::Ok,
                issue: None,
                client_note: None,
            };
        }
        if command == "STLS" {
            if user.is_some() {
                return Reply::pop(false, "STLS is only available before authentication.");
            }
            return match self.link.start_tls(r, f) {
                Ok(()) => {
                    let mut out = Reply::pop(true, "Begin TLS negotiation.");
                    out.client_note=Some("Virtual TLS certificate checks passed; subsequent commands use this logical protected connection.".into());
                    out
                }
                Err(e) if self.link.closed => {
                    *candidate = None;
                    let mut out = Reply::pop(true, "Begin TLS negotiation.");
                    out.code = Code::Closed;
                    out.issue = Some(e.issue);
                    out.client_note = Some(e.message);
                    out
                }
                Err(e) => Reply::failure(Protocol::Pop3, "", e),
            };
        }
        if command == "USER" {
            if user.is_some() {
                return Reply::pop(
                    false,
                    "Already in the transaction state; open a new session to change account.",
                );
            }
            let cfg = match r.active_config() {
                Ok(c) => c,
                Err(e) => return Reply::failure(Protocol::Pop3, "", e),
            };
            if !cfg.permits_password(self.link.tls, false) {
                *candidate = None;
                return Reply::failure(Protocol::Pop3,"",Failure{issue:Issue::Authentication,message:"Plaintext authentication is disabled for this remote connection. Establish verified TLS before USER and PASS.".into()});
            }
            *candidate = Some(args[0].into());
            return Reply::pop(true, "Send the sample password with PASS.");
        }
        if command == "PASS" {
            if user.is_some() {
                return Reply::pop(false, "Already authenticated.");
            }
            let Some(name) = candidate.take() else {
                return Reply::pop(false, "Send USER before PASS.");
            };
            if let Err(e) = r.authenticate(f, &self.link, &name, args[0]) {
                return Reply::failure(Protocol::Pop3, "", e);
            }
            let mailbox = match r.mailbox(f, &name, "INBOX") {
                Ok(b) => b,
                Err(e) => return Reply::failure(Protocol::Pop3, "", e),
            };
            *uids = mailbox.messages.iter().map(|m| m.uid).collect();
            deleted.clear();
            *user = Some(name);
            return Reply::pop(true, format!("Logged in; {} messages.", uids.len()));
        }
        if !matches!(
            command.as_str(),
            "STAT" | "LIST" | "UIDL" | "RETR" | "TOP" | "DELE" | "RSET" | "NOOP"
        ) {
            return Reply::failure(
                Protocol::Pop3,
                "",
                Failure {
                    issue: Issue::Unsupported,
                    message: format!(
                        "{command} is outside this simulator's documented POP subset; it was not executed."
                    ),
                },
            );
        }
        let Some(user) = user.as_deref() else {
            return Reply::pop(
                false,
                "Authenticate with USER and PASS before a transaction command.",
            );
        };
        let mailbox = match r.mailbox(f, user, "INBOX") {
            Ok(b) => b,
            Err(e) => return Reply::failure(Protocol::Pop3, "", e),
        };
        if command == "RSET" {
            deleted.clear();
            return Reply::pop(
                true,
                "Pending deletion marks cleared; stored messages are unchanged.",
            );
        }
        if command == "NOOP" {
            return Reply::pop(true, "No operation.");
        }
        let live = || {
            uids.iter()
                .enumerate()
                .filter(|(_, uid)| !deleted.contains(*uid))
                .filter_map(|(at, uid)| {
                    mailbox
                        .messages
                        .iter()
                        .find(|m| m.uid == *uid)
                        .map(|m| (at + 1, m))
                })
        };
        if command == "STAT" {
            let messages: Vec<_> = live().collect();
            return Reply::pop(
                true,
                format!(
                    "{} {}",
                    messages.len(),
                    messages.iter().map(|(_, m)| m.raw().len()).sum::<usize>()
                ),
            );
        }
        if args.is_empty() && matches!(command.as_str(), "LIST" | "UIDL") {
            let mut text = "+OK Listing follows\r\n".to_owned();
            for (n, m) in live() {
                let value = if command == "LIST" {
                    m.raw().len().to_string()
                } else {
                    format!("sim-{}-{}", mailbox.uid_validity, m.uid)
                };
                text.push_str(&format!("{n} {value}\r\n"));
            }
            text.push_str(".\r\n");
            return Reply {
                text,
                code: Code::Ok,
                issue: None,
                client_note: None,
            };
        }
        let number = match args
            .first()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n > 0 && *n <= uids.len())
        {
            Some(n) => n,
            None => return Reply::pop(false, "No such message number in this transaction."),
        };
        let uid = uids[number - 1];
        if deleted.contains(&uid) {
            return Reply::pop(
                false,
                "This message is already marked for deletion in this transaction.",
            );
        }
        let Some(message) = mailbox.messages.iter().find(|m| m.uid == uid) else {
            return Reply::pop(false, "The selected sample message is no longer available.");
        };
        match command.as_str() {
            "LIST" => Reply::pop(true, format!("{number} {}", message.raw().len())),
            "UIDL" => Reply::pop(true, format!("{number} sim-{}-{uid}", mailbox.uid_validity)),
            "DELE" => {
                deleted.insert(uid);
                Reply::pop(
                    true,
                    format!(
                        "Message {number} marked; deletion commits only on QUIT from this transaction."
                    ),
                )
            }
            "RETR" => {
                let raw = message.raw();
                let cfg = match r.active_config() {
                    Ok(c) => c,
                    Err(e) => return Reply::failure(Protocol::Pop3, "", e),
                };
                if !cfg.pop3_no_flag_updates {
                    match r.mailbox_mut(f, user, "INBOX") {
                        Ok(b) => {
                            if let Some(m) = b.messages.iter_mut().find(|m| m.uid == uid) {
                                m.flags.insert(crate::mailbox_access::Flag::Seen);
                            }
                        }
                        Err(e) => return Reply::failure(Protocol::Pop3, "", e),
                    }
                }
                Reply {
                    text: format!("+OK {} octets\r\n{}", raw.len(), dot_stuff(&raw)),
                    code: Code::Ok,
                    issue: None,
                    client_note: None,
                }
            }
            "TOP" => {
                let Some(lines) = args[1].parse::<usize>().ok().filter(|n| *n <= 2048) else {
                    return Reply::pop(false, "TOP needs a nonnegative, bounded body-line count.");
                };
                let raw = message.raw();
                let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
                let mut part = format!("{head}\r\n\r\n");
                part.push_str(&body.split_inclusive("\r\n").take(lines).collect::<String>());
                Reply {
                    text: format!("+OK Top of message follows\r\n{}", dot_stuff(&part)),
                    code: Code::Ok,
                    issue: None,
                    client_note: None,
                }
            }
            _ => unreachable!(),
        }
    }
}
pub(crate) fn dot_stuff(text: &str) -> String {
    let mut result = String::new();
    for line in text.split_inclusive("\r\n") {
        if line.starts_with('.') {
            result.push('.');
        }
        result.push_str(line);
    }
    if !result.ends_with("\r\n") {
        result.push_str("\r\n");
    }
    result.push_str(".\r\n");
    result
}
