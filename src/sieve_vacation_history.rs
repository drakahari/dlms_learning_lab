//! Bounded virtual vacation reply history, not a message sender or execution engine.
//! RFC5230 response identity and interval handling; no I/O or real scheduling.
use crate::{
    sieve_address::Mailbox,
    sieve_envelope::bare,
    sieve_lex::{Fault, Kind, MAX_STRING, fail},
};
use std::collections::BTreeMap;

pub const CAPACITY: usize = 1000;
pub const DEFAULT_DAYS: u64 = 7;
pub const MIN_DAYS: u64 = 1;
pub const MAX_DAYS: u64 = 365;
pub const SECONDS_PER_DAY: u64 = 86_400;

/// Explicit handles replace the content-based identity. Otherwise raw optional
/// command arguments stay distinct; a default generated subject is not a key.
/// MIME is unsupported in the planned plain-message profile, not approximated.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResponseKey {
    Handle(String),
    Plain {
        subject: Option<String>,
        from: Option<String>,
        reason: String,
    },
}
impl ResponseKey {
    fn validate(&self) -> Result<(), Fault> {
        let too_long = match self {
            Self::Handle(value) => value.len() > MAX_STRING,
            Self::Plain {
                subject,
                from,
                reason,
            } => {
                subject.as_ref().is_some_and(|s| s.len() > MAX_STRING)
                    || from.as_ref().is_some_and(|s| s.len() > MAX_STRING)
                    || reason.len() > MAX_STRING
            }
        };
        if too_long {
            return fail(
                Kind::Limit,
                0,
                "A vacation response identifier exceeds the supported string limit.",
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EntryKey {
    sender: String,
    response: ResponseKey,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Send {
        effective_days: u64,
    },
    Suppressed {
        last_sent: u64,
        remaining_seconds: u64,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    sent: BTreeMap<EntryKey, u64>,
    last_observed: Option<u64>,
}
pub fn effective_days(requested: Option<u64>) -> u64 {
    requested.unwrap_or(DEFAULT_DAYS).clamp(MIN_DAYS, MAX_DAYS)
}
impl History {
    pub fn len(&self) -> usize {
        self.sent.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sent.is_empty()
    }
    /// Observe ordered simulated arrival seconds; never wait on the host clock.
    pub fn observe(&mut self, now: u64) -> Result<(), Fault> {
        if self.last_observed.is_some_and(|previous| now < previous) {
            return fail(
                Kind::Runtime,
                0,
                "The simulated message timeline moves backward. Retain the draft and inspect the supplied arrival times.",
            );
        }
        self.last_observed = Some(now);
        Ok(())
    }
    /// Call only after eligibility checks, on the evaluation's transactional copy.
    /// A later script error must discard that copy together with proposed effects.
    /// Histories belong to one user/scope/run; assignment and practice never share.
    pub fn consider(
        &mut self,
        sender: &Mailbox,
        response: &ResponseKey,
        now: u64,
        requested_days: Option<u64>,
    ) -> Result<Decision, Fault> {
        response.validate()?;
        // Defend this public API even if a caller manually builds a Mailbox.
        let checked = bare(&sender.all())?;
        let key = EntryKey {
            sender: format!("{}@{}", checked.local, checked.domain.to_ascii_lowercase()),
            response: response.clone(),
        };
        self.observe(now)?;
        let days = effective_days(requested_days);
        let interval = days * SECONDS_PER_DAY;
        if let Some(last) = self.sent.get(&key) {
            // observe established monotonicity, so subtraction cannot underflow.
            let elapsed = now - *last;
            if elapsed < interval {
                return Ok(Decision::Suppressed {
                    last_sent: *last,
                    remaining_seconds: interval - elapsed,
                });
            }
        }
        if !self.sent.contains_key(&key) && self.sent.len() == CAPACITY {
            // RFC5230 permits bounded history (minimum1000) and recommends oldest
            // eviction. Ordered keys give deterministic ties, not caller hashes.
            let oldest = self
                .sent
                .iter()
                .min_by_key(|(_, at)| *at)
                .map(|(key, _)| key.clone())
                .expect("A full history is nonempty");
            self.sent.remove(&oldest);
        }
        self.sent.insert(key, now);
        Ok(Decision::Send {
            effective_days: days,
        })
    }
}
