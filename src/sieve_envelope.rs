//! Plain simulated SMTP envelope data, distinct from visible message headers.
//! No socket, DNS, filesystem, clock or delivery operations.
use crate::{
    sieve_address::{Mailbox, mailboxes},
    sieve_lex::{Fault, Kind, fail},
    sieve_parse::AddressPart,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Empty means SMTP's null reverse path, not a missing or visible From field.
    pub from: String,
    /// Exactly the one delivery recipient relevant to this simulated user.
    pub to: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub(crate) from: Option<Mailbox>,
    pub(crate) to: Mailbox,
}
impl Input {
    pub fn validate(&self) -> Result<Envelope, Fault> {
        let from = if self.from.is_empty() {
            None
        } else {
            Some(bare(&self.from)?)
        };
        let to = bare(&self.to)?;
        Ok(Envelope { from, to })
    }
}
/// This disclosed envelope/redirect subset uses a single bare dot-atom mailbox.
/// Header names, display phrases, comments and named groups are not envelope data.
pub fn bare(value: &str) -> Result<Mailbox, Fault> {
    if value.len() > 254 {
        return fail(
            Kind::Limit,
            0,
            "A represented envelope mailbox exceeds 254 bytes.",
        );
    }
    let mut parsed = mailboxes(value)?;
    if parsed.len() != 1 {
        return fail(
            Kind::Runtime,
            0,
            "Supply one bare mailbox address in the simulated envelope or redirect target.",
        );
    }
    let mailbox = parsed.remove(0);
    if mailbox.all() != value {
        return fail(
            Kind::Unsupported,
            0,
            "This envelope/redirect subset supports bare dot-atom mailboxes only, without display names, brackets or comments. No host delivery is attempted.",
        );
    }
    Ok(mailbox)
}
impl Envelope {
    pub fn part(&self, field: &str, part: AddressPart) -> Result<String, Fault> {
        let address = if field.eq_ignore_ascii_case("from") {
            // RFC5228: the null reverse path matches an empty string for every part.
            let Some(from) = &self.from else {
                return Ok(String::new());
            };
            from
        } else if field.eq_ignore_ascii_case("to") {
            &self.to
        } else {
            return fail(
                Kind::Unsupported,
                0,
                "Supported envelope parts are from and to. Visible From/To headers are separate inputs.",
            );
        };
        Ok(match part {
            AddressPart::All => address.all(),
            AddressPart::Local => address.local.clone(),
            AddressPart::Domain => address.domain.clone(),
        })
    }
}
pub fn same_mailbox(left: &Mailbox, right: &Mailbox) -> bool {
    left.local == right.local && left.domain.eq_ignore_ascii_case(&right.domain)
}
