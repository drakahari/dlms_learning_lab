//! Plain-message vacation eligibility and logical reply metadata, no transport.
use crate::{
    sieve_address::{Mailbox, mailboxes},
    sieve_envelope::{Envelope, bare, same_mailbox},
    sieve_eval::Message,
    sieve_lex::{Fault, Kind, MAX_STRING, fail},
    sieve_vacation_history::ResponseKey,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_ALIASES: usize = 16;
pub const MAX_ADDRESSES: usize = 32;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub primary: String,
    pub aliases: BTreeSet<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Request {
    pub days: Option<u64>,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub addresses: Vec<String>,
    pub handle: Option<String>,
    pub reason: String,
}
impl Request {
    pub fn response_key(&self) -> ResponseKey {
        if let Some(handle) = &self.handle {
            ResponseKey::Handle(handle.clone())
        } else {
            ResponseKey::Plain {
                subject: self.subject.clone(),
                from: self.from.clone(),
                reason: self.reason.clone(),
            }
        }
    }
    fn validate(&self) -> Result<(), Fault> {
        if self.addresses.len() > MAX_ADDRESSES
            || self.reason.len() > MAX_STRING
            || [&self.subject, &self.from, &self.handle]
                .iter()
                .any(|s| s.as_ref().is_some_and(|s| s.len() > MAX_STRING))
        {
            return fail(
                Kind::Limit,
                0,
                "Vacation arguments exceed the supported string/address limits.",
            );
        }
        if self
            .subject
            .as_ref()
            .is_some_and(|s| s.chars().any(char::is_control))
        {
            return fail(
                Kind::Syntax,
                0,
                "A vacation subject must be one logical line, without control characters.",
            );
        }
        if let Some(from) = &self.from {
            bare(from)?;
        }
        for address in &self.addresses {
            bare(address)?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suppression {
    NullSender,
    DaemonSender,
    MailingList,
    Automated,
    NotPersonal,
    SelfReply,
}
impl Suppression {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::NullSender => "No reply: the delivery has a null return address.",
            Self::DaemonSender => {
                "No reply: this sender is on the simulated site's unattended-address list."
            }
            Self::MailingList => "No reply: mailing-list metadata is present.",
            Self::Automated => "No reply: Auto-Submitted identifies an automatic message.",
            Self::NotPersonal => {
                "No reply: no supported visible recipient field names this user or a supplied alias."
            }
            Self::SelfReply => "No reply: the sender is one of this recipient's own addresses.",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoSubmitted {
    #[serde(rename = "auto-replied")]
    AutoReplied,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub auto_submitted: AutoSubmitted,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Preparation {
    Suppressed(Suppression),
    Ready { reply: Box<Reply>, key: ResponseKey },
}
impl Policy {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.aliases.len() > MAX_ALIASES {
            return fail(
                Kind::Limit,
                0,
                "This simulated user has too many configured aliases.",
            );
        }
        let mut addresses = vec![bare(&self.primary)?];
        for alias in &self.aliases {
            let alias = bare(alias)?;
            if addresses.iter().any(|a| same_mailbox(a, &alias)) {
                return fail(
                    Kind::Syntax,
                    0,
                    "Use distinct primary and alias mailboxes under the site's address comparison policy.",
                );
            }
            addresses.push(alias);
        }
        Ok(())
    }
    pub fn prepare(
        &self,
        message: &Message,
        envelope: &Envelope,
        request: &Request,
    ) -> Result<Preparation, Fault> {
        self.validate()?;
        request.validate()?;
        let Some(sender) = &envelope.from else {
            return Ok(Preparation::Suppressed(Suppression::NullSender));
        };
        let local = sender.local.to_ascii_lowercase();
        if ["mailer-daemon", "listserv", "majordomo"].contains(&local.as_str())
            || local.starts_with("owner-")
            || local.ends_with("-request")
        {
            return Ok(Preparation::Suppressed(Suppression::DaemonSender));
        }
        let fields = message.header_fields();
        if fields.iter().any(|(name, _)| {
            [
                "list-id",
                "list-help",
                "list-subscribe",
                "list-unsubscribe",
                "list-post",
                "list-owner",
                "list-archive",
            ]
            .iter()
            .any(|n| name.eq_ignore_ascii_case(n))
        }) {
            return Ok(Preparation::Suppressed(Suppression::MailingList));
        }
        for (name, value) in fields {
            if name.eq_ignore_ascii_case("auto-submitted") {
                let token = value.trim();
                if token.is_empty()
                    || !token
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                {
                    return fail(
                        Kind::Unsupported,
                        0,
                        "This plain-message profile supports bare Auto-Submitted keywords only; comments and parameters need a fuller header parser.",
                    );
                }
                if !token.eq_ignore_ascii_case("no") {
                    return Ok(Preparation::Suppressed(Suppression::Automated));
                }
            }
            // Optional conservative site heuristic, not a universal RFC rule.
            if name.eq_ignore_ascii_case("precedence")
                && ["bulk", "list", "junk"]
                    .iter()
                    .any(|v| value.trim().eq_ignore_ascii_case(v))
            {
                return Ok(Preparation::Suppressed(Suppression::MailingList));
            }
        }
        let mut owned = vec![bare(&self.primary)?, envelope.to.clone()];
        for value in self.aliases.iter().chain(&request.addresses) {
            owned.push(bare(value)?);
        }
        if owned.iter().any(|a| same_mailbox(a, sender)) {
            return Ok(Preparation::Suppressed(Suppression::SelfReply));
        }
        let mut personal = false;
        for (name, value) in fields {
            if ["to", "cc", "bcc", "resent-to", "resent-cc", "resent-bcc"]
                .iter()
                .any(|n| name.eq_ignore_ascii_case(n))
            {
                for recipient in mailboxes(value)? {
                    if owned.iter().any(|a| same_mailbox(a, &recipient)) {
                        personal = true;
                    }
                }
            }
        }
        if !personal {
            return Ok(Preparation::Suppressed(Suppression::NotPersonal));
        }
        let subject = if let Some(subject) = &request.subject {
            subject.clone()
        } else {
            let values = values(fields, "subject");
            if values.len() > 1 {
                return fail(
                    Kind::Unsupported,
                    0,
                    "This reply profile requires a single original Subject when deriving a default reply subject.",
                );
            }
            match values.first() {
                None => String::from("Automated reply"),
                Some(value) => {
                    if value.contains("=?") {
                        return fail(
                            Kind::Unsupported,
                            0,
                            "MIME-encoded original subjects are outside this plain-message reply profile; no decoding is pretended.",
                        );
                    }
                    format!("Auto: {value}")
                }
            }
        };
        let ids = values(fields, "message-id");
        let refs = values(fields, "references");
        if ids.len() > 1 || refs.len() > 1 {
            return fail(
                Kind::Unsupported,
                0,
                "This reply profile requires at most one Message-ID and one References field.",
            );
        }
        let in_reply_to = ids.first().map(|id| message_id(id)).transpose()?;
        let mut references = Vec::new();
        if let Some(id) = &in_reply_to {
            if let Some(value) = refs.first() {
                for token in value.split_whitespace() {
                    references.push(message_id(token)?);
                }
            }
            if references.len() >= 32 {
                return fail(
                    Kind::Limit,
                    0,
                    "The represented reply thread exceeds32message identifiers.",
                );
            }
            references.push(id.clone());
        }
        Ok(Preparation::Ready {
            reply: Box::new(Reply {
                from: request.from.clone().unwrap_or_else(|| self.primary.clone()),
                to: sender.all(),
                subject,
                body: request.reason.clone(),
                auto_submitted: AutoSubmitted::AutoReplied,
                in_reply_to,
                references,
            }),
            key: request.response_key(),
        })
    }
}
fn values<'a>(fields: &'a [(String, String)], name: &str) -> Vec<&'a str> {
    fields
        .iter()
        .filter_map(|(n, v)| n.eq_ignore_ascii_case(name).then_some(v.as_str()))
        .collect()
}
fn message_id(value: &str) -> Result<String, Fault> {
    let value = value.trim();
    let Some(inner) = value.strip_prefix('<').and_then(|s| s.strip_suffix('>')) else {
        return fail(
            Kind::Unsupported,
            0,
            "Reply threading supports plain angle-bracket dot-atom message identifiers only.",
        );
    };
    let _: Mailbox = bare(inner)?;
    Ok(value.into())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunResult {
    Sent {
        reply: Box<Reply>,
        effective_days: u64,
    },
    Suppressed {
        reason: Suppression,
    },
    Recent {
        last_sent: u64,
        remaining_seconds: u64,
    },
}
impl RunResult {
    /// Context-free restore bounds; lesson replay verifies the actual effects.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Sent {
                reply,
                effective_days,
            } => {
                if !(crate::sieve_vacation_history::MIN_DAYS
                    ..=crate::sieve_vacation_history::MAX_DAYS)
                    .contains(effective_days)
                    || reply.body.len() > MAX_STRING
                    || reply.subject.len() > MAX_STRING + 6
                    || reply.subject.chars().any(|c| c.is_control() && c != '\t')
                    || reply.references.len() > 32
                {
                    return Err(
                        "Saved vacation reply exceeds the message or interval limits.".into(),
                    );
                }
                bare(&reply.from).map_err(|e| e.to_string())?;
                bare(&reply.to).map_err(|e| e.to_string())?;
                for value in reply.references.iter().chain(reply.in_reply_to.iter()) {
                    message_id(value).map_err(|e| e.to_string())?;
                }
            }
            Self::Recent {
                remaining_seconds, ..
            } => {
                if *remaining_seconds == 0
                    || *remaining_seconds
                        > crate::sieve_vacation_history::MAX_DAYS
                            * crate::sieve_vacation_history::SECONDS_PER_DAY
                {
                    return Err("Saved repeat suppression exceeds the supported interval.".into());
                }
            }
            Self::Suppressed { .. } => {}
        }
        Ok(())
    }
    pub fn description(&self) -> String {
        match self {
            Self::Sent {
                reply,
                effective_days,
            } => format!(
                "Vacation reply prepared for {} · interval {effective_days} day(s)",
                reply.to
            ),
            Self::Suppressed { reason } => reason.explanation().into(),
            Self::Recent {
                last_sent,
                remaining_seconds,
            } => format!(
                "No repeat reply · last prepared at {last_sent}s · {remaining_seconds}s until eligible"
            ),
        }
    }
}
