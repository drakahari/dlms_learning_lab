//! Validate resumed virtual state against its immutable authored fixtures.
//! Importing progress never supplies an endpoint, account, file or message body.
use crate::{
    mailbox_access::{Facts, Failure, Issue, Runtime},
    mailbox_config::{Config, MAX_LINES, MAX_TEXT, Protocol},
    mailbox_protocol::{MAX_COMMAND, MAX_COMMANDS, MAX_OUTPUT, Phase, Session},
};
fn invalid<T>(message: &str) -> Result<T, Failure> {
    Err(Failure {
        issue: Issue::Configuration,
        message: message.into(),
    })
}
impl Runtime {
    pub fn validate(&self, facts: &Facts) -> Result<(), Failure> {
        facts.validate()?;
        if self.saved.len() > MAX_TEXT
            || self.saved.lines().count() > MAX_LINES
            || self.running != self.active.is_some()
            || self.logs.len() > 64
            || self.error.as_ref().is_some_and(|e| e.message.len() > 16384)
        {
            return invalid("Saved mail-access runtime fields exceed their documented bounds.");
        }
        if let Some(text) = &self.active {
            if text.len() > MAX_TEXT || text.lines().count() > MAX_LINES {
                return invalid("Active mail-access settings exceed the file limit.");
            }
            facts.startup(&Config::parse(text)?)?;
        }
        let mut number = 0;
        for log in &self.logs {
            if log.number <= number || log.message.len() > 16384 {
                return invalid("Saved service diagnostics exceed their ordered log bounds.");
            }
            number = log.number;
        }
        if self.store.keys().ne(facts.mailboxes.keys()) {
            return invalid("Restored mail must retain the supplied account namespaces.");
        }
        for (user, boxes) in &self.store {
            let originals = &facts.mailboxes[user];
            if boxes.keys().ne(originals.keys()) {
                return invalid("Restored mail must retain the supplied flat mailbox names.");
            }
            for (name, mailbox) in boxes {
                mailbox.validate()?;
                let original = &originals[name];
                if mailbox.uid_validity != original.uid_validity
                    || mailbox.uid_next != original.uid_next
                {
                    return invalid(
                        "Restored mailbox UIDVALIDITY and UIDNEXT must retain their supplied identities.",
                    );
                }
                for message in &mailbox.messages {
                    let Some(before) = original.messages.iter().find(|m| m.uid == message.uid)
                    else {
                        return invalid("A restored message UID was not supplied by this lesson.");
                    };
                    if message.headers != before.headers || message.body != before.body {
                        return invalid(
                            "This mail-access subset changes flags or deletes messages; it cannot rewrite or invent message content.",
                        );
                    }
                }
            }
        }
        if self.pop_committed.len() > facts.accounts.len() {
            return invalid("Too many recorded POP account outcomes.");
        }
        for (user, uids) in &self.pop_committed {
            let Some(boxes) = facts.mailboxes.get(user) else {
                return invalid("A POP commit observation names an unknown account.");
            };
            if uids.is_empty()
                || uids.len() > 16
                || uids.iter().any(|uid| {
                    !boxes["INBOX"].messages.iter().any(|m| m.uid == *uid)
                        || self.store[user]["INBOX"]
                            .messages
                            .iter()
                            .any(|m| m.uid == *uid)
                })
            {
                return invalid(
                    "A POP commit observation must name supplied messages that are currently absent.",
                );
            }
        }
        Ok(())
    }
}
impl Session {
    pub fn validate(&self, runtime: &Runtime, facts: &Facts) -> Result<(), Failure> {
        self.link.client.validate()?;
        if self.link.protocol != self.link.client.protocol
            || self.link.generation > runtime.generation
            || self.commands > MAX_COMMANDS
            || self.output_bytes > MAX_OUTPUT
            || self.link.client.implicit_tls && !self.link.tls
        {
            return invalid(
                "Saved protocol counters, transport or server generation are inconsistent.",
            );
        }
        match &self.phase {
            Phase::Imap { user, selected } => {
                if self.link.protocol != Protocol::Imap {
                    return invalid("IMAP state cannot belong to a POP connection.");
                }
                if let Some(user) = user {
                    let Some(boxes) = runtime.store.get(user) else {
                        return invalid("Unknown restored IMAP account.");
                    };
                    if selected
                        .as_ref()
                        .is_some_and(|s| !boxes.contains_key(&s.mailbox))
                    {
                        return invalid("Unknown restored IMAP mailbox selection.");
                    }
                } else if selected.is_some() {
                    return invalid("An unauthenticated session cannot select a mailbox.");
                }
            }
            Phase::Pop {
                candidate,
                user,
                uids,
                deleted,
            } => {
                if self.link.protocol != Protocol::Pop3
                    || uids.len() > 16
                    || deleted.len() > 16
                    || candidate.as_ref().is_some_and(|s| {
                        s.is_empty()
                            || s.len() > MAX_COMMAND
                            || !s.bytes().all(|b| b.is_ascii_graphic())
                    })
                {
                    return invalid(
                        "Saved POP state exceeds its bounded authorization/transaction profile.",
                    );
                }
                if let Some(user) = user {
                    let Some(boxes) = facts.mailboxes.get(user) else {
                        return invalid("Unknown restored POP account.");
                    };
                    let original = &boxes["INBOX"];
                    if candidate.is_some()
                        || uids.windows(2).any(|w| w[0] >= w[1])
                        || uids
                            .iter()
                            .any(|u| !original.messages.iter().any(|m| m.uid == *u))
                        || !deleted.iter().all(|u| uids.contains(u))
                    {
                        return invalid(
                            "POP transaction numbers must name a stable supplied UID snapshot.",
                        );
                    }
                    if !self.link.closed && self.link.generation == runtime.generation {
                        let current: Vec<_> = runtime.store[user]["INBOX"]
                            .messages
                            .iter()
                            .map(|m| m.uid)
                            .collect();
                        if current != *uids {
                            return invalid("The open POP snapshot differs from its stored INBOX.");
                        }
                    }
                } else if !uids.is_empty() || !deleted.is_empty() {
                    return invalid(
                        "POP message numbers and deletion marks require authentication.",
                    );
                }
            }
        }
        Ok(())
    }
}
