//! Explicit fixture-only forwarding policy. No real recipients or delivery I/O.
use crate::{
    sieve_envelope::{Envelope, bare, same_mailbox},
    sieve_lex::{Fault, Kind, fail},
};
use std::collections::BTreeSet;
pub const MAX_REDIRECTS: usize = 4;
pub const MAX_TARGETS: usize = 16;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    pub targets: BTreeSet<String>,
}
impl Policy {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.targets.len() > MAX_TARGETS {
            return fail(
                Kind::Limit,
                0,
                "This lesson has too many simulated forwarding targets.",
            );
        }
        let mut parsed = Vec::new();
        for target in &self.targets {
            let address = bare(target)?;
            if parsed.iter().any(|earlier| same_mailbox(earlier, &address)) {
                return fail(
                    Kind::Syntax,
                    0,
                    "Simulated forwarding targets must name distinct mailboxes, including domain case differences.",
                );
            }
            parsed.push(address);
        }
        Ok(())
    }
    /// Resolve only an explicitly supplied simulated mailbox. Domain comparison
    /// ignores ASCII case; local-part comparison is exact under this site policy.
    pub fn redirect(&self, value: &str, envelope: &Envelope) -> Result<String, Fault> {
        let destination = bare(value)?;
        if same_mailbox(&destination, &envelope.to) {
            return fail(
                Kind::Runtime,
                0,
                "Forwarding would loop to this delivery recipient. Inspect the envelope and forwarding target; the original sample is kept in INBOX.",
            );
        }
        for target in &self.targets {
            let permitted = bare(target)?;
            if same_mailbox(&destination, &permitted) {
                return Ok(target.clone());
            }
        }
        fail(
            Kind::Runtime,
            0,
            "That forwarding target is outside this lesson's simulated targets. The original sample is kept in INBOX.",
        )
    }
}
