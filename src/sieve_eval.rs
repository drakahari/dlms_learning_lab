//! Bounded in-process virtual mail filtering; no host I/O or delivery.
use super::sieve_lex::{Fault, Kind, fail};
use super::sieve_parse::{AddressPart, Comparison, Match, Program, Statement, Test};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
pub const MAX_MESSAGE: usize = 64 * 1024;
pub const MAX_HEADERS: usize = 32;
pub const MAX_ACTIONS: usize = 32;
pub const MAX_WORK: usize = 2 * 1024 * 1024;
pub const MAX_ELAPSED_MILLIS: u64 = 250;
struct Clock {
    started: Instant,
    allowance: Duration,
}
impl Clock {
    fn check(&self) -> Result<(), Fault> {
        if self.started.elapsed() >= self.allowance {
            return fail(
                Kind::Limit,
                0,
                "The filter reached its elapsed-time limit. No filing actions were applied; the sample was kept in INBOX. Simplify the filter and retry.",
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    headers: Vec<(String, String)>,
    octets: u64,
}
impl Message {
    pub fn new(headers: Vec<(String, String)>, body: &str) -> Result<Self, Fault> {
        if headers.len() > MAX_HEADERS {
            return fail(Kind::Limit, 0, "A sample may contain at most32 headers.");
        }
        let mut octets = 2usize;
        for (name, value) in &headers {
            if !valid_header(name) {
                return fail(
                    Kind::Syntax,
                    0,
                    "A represented header needs a valid name without a colon.",
                );
            }
            if value.len() > 1024 {
                return fail(
                    Kind::Limit,
                    0,
                    "A represented header value exceeds1024 bytes.",
                );
            }
            if !value.is_ascii()
                || value.contains("=?")
                || value.bytes().any(|b| b < 32 && b != b'\t' || b == 127)
            {
                return fail(
                    Kind::Unsupported,
                    0,
                    "Samples support plain, already-unfolded ASCII headers. Encoded or multiline header values are not decoded by this simulator.",
                );
            }
            octets = octets
                .checked_add(name.len() + 2 + value.len() + 2)
                .ok_or_else(|| Fault {
                    kind: Kind::Limit,
                    offset: 0,
                    message: "Message size overflow.".into(),
                })?;
        }
        if body.len() > MAX_MESSAGE || body.contains('\0') {
            return fail(
                Kind::Limit,
                0,
                "The represented message body is too large or contains NUL.",
            );
        }
        // Canonical represented message uses CRLF; an editor's LF counts as CRLF.
        let bytes = body.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            match bytes[at] {
                b'\r' => {
                    if bytes.get(at + 1) != Some(&b'\n') {
                        return fail(
                            Kind::Syntax,
                            0,
                            "Use LF or CRLF body lines, not a lone carriage return.",
                        );
                    }
                    at += 2;
                    octets += 2;
                }
                b'\n' => {
                    at += 1;
                    octets += 2;
                }
                _ => {
                    at += 1;
                    octets += 1;
                }
            }
        }
        if octets > MAX_MESSAGE {
            return fail(
                Kind::Limit,
                0,
                "The canonical sample message exceeds64 KiB.",
            );
        }
        Ok(Self {
            headers,
            octets: octets as u64,
        })
    }
    pub(crate) fn header_fields(&self) -> &[(String, String)] {
        &self.headers
    }
    pub fn octets(&self) -> u64 {
        self.octets
    }
}
fn valid_header(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| (33..=126).contains(&b) && b != b':')
}
pub fn valid_mailbox(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.contains("..")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub vacation: Option<crate::sieve_vacation_policy::RunResult>,
    pub redirects: BTreeSet<String>,
    pub rejection: Option<String>,
    pub mailboxes: BTreeSet<String>,
    pub implicit_keep: bool,
    pub discarded: bool,
    pub fault: Option<Fault>,
    pub actions: usize,
    pub work: usize,
}
#[derive(Clone, Copy)]
struct DeliveryContext<'a> {
    envelope: &'a crate::sieve_envelope::Envelope,
    policy: &'a crate::sieve_delivery_policy::Policy,
}
/// One simulated arrival. The caller supplies a per-user/per-run history.
#[derive(Clone, Copy)]
pub struct VacationContext<'a> {
    pub envelope: &'a crate::sieve_envelope::Envelope,
    pub forwarding: &'a crate::sieve_delivery_policy::Policy,
    pub replies: &'a crate::sieve_vacation_policy::Policy,
    pub at_seconds: u64,
}
struct Vacation<'a> {
    policy: &'a crate::sieve_vacation_policy::Policy,
    history: &'a mut crate::sieve_vacation_history::History,
    at: u64,
    executed: bool,
}
struct Evaluation<'a> {
    vacation: Option<Vacation<'a>>,
    delivery: Option<DeliveryContext<'a>>,
    redirect_actions: usize,
    message: &'a Message,
    allowed: &'a BTreeSet<String>,
    out: Outcome,
    cancelled: bool,
    clock: Clock,
}
impl Program {
    pub fn evaluate(&self, message: &Message, mailboxes: &BTreeSet<String>) -> Outcome {
        self.evaluate_timed(
            message,
            mailboxes,
            Duration::from_millis(MAX_ELAPSED_MILLIS),
        )
    }
    pub(crate) fn evaluate_timed(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        allowance: Duration,
    ) -> Outcome {
        self.evaluate_context(message, mailboxes, allowance, None, None)
    }
    pub fn evaluate_delivery(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        envelope: &crate::sieve_envelope::Envelope,
        policy: &crate::sieve_delivery_policy::Policy,
    ) -> Outcome {
        self.evaluate_delivery_timed(
            message,
            mailboxes,
            envelope,
            policy,
            Duration::from_millis(MAX_ELAPSED_MILLIS),
        )
    }
    pub(crate) fn evaluate_delivery_timed(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        envelope: &crate::sieve_envelope::Envelope,
        policy: &crate::sieve_delivery_policy::Policy,
        allowance: Duration,
    ) -> Outcome {
        self.evaluate_context(
            message,
            mailboxes,
            allowance,
            Some(DeliveryContext { envelope, policy }),
            None,
        )
    }
    pub fn evaluate_vacation(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        context: VacationContext<'_>,
        history: &mut crate::sieve_vacation_history::History,
    ) -> Outcome {
        self.evaluate_vacation_timed(
            message,
            mailboxes,
            context,
            history,
            Duration::from_millis(MAX_ELAPSED_MILLIS),
        )
    }
    pub(crate) fn evaluate_vacation_timed(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        context: VacationContext<'_>,
        history: &mut crate::sieve_vacation_history::History,
        allowance: Duration,
    ) -> Outcome {
        let before = history.clone();
        let out = self.evaluate_context(
            message,
            mailboxes,
            allowance,
            Some(DeliveryContext {
                envelope: context.envelope,
                policy: context.forwarding,
            }),
            Some(Vacation {
                policy: context.replies,
                history,
                at: context.at_seconds,
                executed: false,
            }),
        );
        if out.fault.is_some() {
            *history = before;
        }
        out
    }
    fn evaluate_context(
        &self,
        message: &Message,
        mailboxes: &BTreeSet<String>,
        allowance: Duration,
        delivery: Option<DeliveryContext<'_>>,
        vacation: Option<Vacation<'_>>,
    ) -> Outcome {
        let mut state = Evaluation {
            vacation,
            delivery,
            redirect_actions: 0,
            message,
            allowed: mailboxes,
            out: Outcome {
                vacation: None,
                redirects: BTreeSet::new(),
                rejection: None,
                mailboxes: BTreeSet::new(),
                implicit_keep: false,
                discarded: false,
                fault: None,
                actions: 0,
                work: 0,
            },
            cancelled: false,
            clock: Clock {
                started: Instant::now(),
                allowance,
            },
        };
        let result = if !mailboxes.contains("INBOX")
            || mailboxes.len() > 32
            || mailboxes.iter().any(|m| !valid_mailbox(m))
        {
            fail(
                Kind::Runtime,
                0,
                "The virtual mailbox directory is invalid; INBOX and at most32 named destinations are required.",
            )
        } else {
            state
                .delivery
                .map_or(Ok(()), |context| context.policy.validate())
                .and_then(|()| {
                    state.vacation.as_mut().map_or(Ok(()), |ctx| {
                        ctx.policy.validate()?;
                        ctx.history.observe(ctx.at)
                    })
                })
                .and_then(|()| state.sequence(&self.statements))
                .and_then(|stopped| {
                    state.clock.check()?;
                    Ok(stopped)
                })
        };
        if let Err(fault) = result {
            state.out.vacation = None;
            state.out.redirects.clear();
            state.out.rejection = None;
            state.out.mailboxes.clear();
            state.out.mailboxes.insert("INBOX".into());
            state.out.implicit_keep = true;
            state.out.fault = Some(fault);
        } else if !state.cancelled {
            state.out.mailboxes.insert("INBOX".into());
            state.out.implicit_keep = true;
        } else {
            state.out.discarded = state.out.mailboxes.is_empty()
                && state.out.redirects.is_empty()
                && state.out.rejection.is_none();
        }
        state.out
    }
}
impl Evaluation<'_> {
    fn delivery_compatible(&self) -> Result<(), Fault> {
        if self.out.rejection.is_some() {
            return fail(
                Kind::Runtime,
                0,
                "This simulated site cannot both refuse and deliver the same message. Stop or choose a different branch; the original sample was kept in INBOX.",
            );
        }
        Ok(())
    }
    fn charge(&mut self, n: usize) -> Result<(), Fault> {
        self.clock.check()?;
        if n > MAX_WORK.saturating_sub(self.out.work) {
            return fail(
                Kind::Limit,
                0,
                "The filter exceeded the comparison-work limit. No filing actions were applied; the sample was kept in INBOX.",
            );
        }
        self.out.work += n;
        Ok(())
    }
    fn count_action(&mut self) -> Result<(), Fault> {
        if self.out.actions == MAX_ACTIONS {
            return fail(
                Kind::Limit,
                0,
                "The filter exceeded32 actions for one sample. No filing actions were applied; the sample was kept in INBOX.",
            );
        }
        self.out.actions += 1;
        Ok(())
    }
    fn action(&mut self) -> Result<(), Fault> {
        self.count_action()?;
        self.cancelled = true;
        Ok(())
    }
    fn vacation_action(
        &mut self,
        request: &crate::sieve_vacation_policy::Request,
    ) -> Result<(), Fault> {
        self.charge(1)?;
        if self.out.rejection.is_some() {
            return fail(
                Kind::Runtime,
                0,
                "Vacation and rejection cannot be combined for one message. No reply or refusal was applied; the original is kept in INBOX.",
            );
        }
        self.count_action()?;
        let delivery = self.delivery.ok_or_else(|| Fault {
            kind: Kind::Unsupported,
            offset: 0,
            message: "Vacation needs this lesson's simulated delivery envelope and reply history."
                .into(),
        })?;
        let ctx=self.vacation.as_mut().ok_or_else(||Fault{kind:Kind::Unsupported,offset:0,message:"Vacation needs a simulated timeline context; a fresh independent-message run is not a reply-history test.".into()})?;
        if ctx.executed {
            return fail(
                Kind::Runtime,
                0,
                "Only one vacation action may execute for a message, even when a reply is suppressed. Use mutually exclusive branches or stop.",
            );
        }
        ctx.executed = true;
        use crate::sieve_vacation_history::Decision;
        use crate::sieve_vacation_policy::{Preparation, RunResult};
        let result = match ctx
            .policy
            .prepare(self.message, delivery.envelope, request)?
        {
            Preparation::Suppressed(reason) => RunResult::Suppressed { reason },
            Preparation::Ready { reply, key } => {
                let sender = delivery.envelope.from.as_ref().ok_or_else(|| Fault {
                    kind: Kind::Runtime,
                    offset: 0,
                    message: "A null return address cannot receive a vacation response.".into(),
                })?;
                match ctx.history.consider(sender, &key, ctx.at, request.days)? {
                    Decision::Send { effective_days } => RunResult::Sent {
                        reply,
                        effective_days,
                    },
                    Decision::Suppressed {
                        last_sent,
                        remaining_seconds,
                    } => RunResult::Recent {
                        last_sent,
                        remaining_seconds,
                    },
                }
            }
        };
        self.out.vacation = Some(result);
        Ok(())
    }
    fn sequence(&mut self, statements: &[Statement]) -> Result<bool, Fault> {
        for statement in statements {
            self.charge(1)?;
            match statement {
                Statement::Stop => return Ok(true),
                Statement::Keep => {
                    self.delivery_compatible()?;
                    self.action()?;
                    self.out.mailboxes.insert("INBOX".into());
                }
                Statement::Discard => {
                    self.action()?;
                }
                Statement::FileInto(name) => {
                    self.delivery_compatible()?;
                    self.action()?;
                    let name = if name.eq_ignore_ascii_case("INBOX") {
                        String::from("INBOX")
                    } else {
                        name.clone()
                    };
                    if !valid_mailbox(&name) || !self.allowed.contains(&name) {
                        return fail(
                            Kind::Runtime,
                            0,
                            "This fileinto destination is not an available virtual mailbox. Check the displayed mailbox names; no filing actions were applied and the sample was kept in INBOX.",
                        );
                    }
                    self.out.mailboxes.insert(name);
                }
                Statement::Redirect(target) => {
                    self.delivery_compatible()?;
                    let context=self.delivery.ok_or_else(||Fault{kind:Kind::Unsupported,offset:0,message:"This filter needs the lesson's simulated envelope and forwarding policy; no actual mail is sent.".into()})?;
                    let target = context.policy.redirect(target, context.envelope)?;
                    if self.redirect_actions >= crate::sieve_delivery_policy::MAX_REDIRECTS {
                        return fail(
                            Kind::Limit,
                            0,
                            "The filter exceeds four forwarding actions for one sample. No deliveries were applied; the sample was kept in INBOX.",
                        );
                    }
                    self.action()?;
                    self.redirect_actions += 1;
                    self.out.redirects.insert(target);
                }
                Statement::Vacation(request) => self.vacation_action(request)?,
                Statement::Reject(reason) => {
                    if self.vacation.as_ref().is_some_and(|ctx| ctx.executed) {
                        return fail(
                            Kind::Runtime,
                            0,
                            "Rejection cannot follow an executed vacation action, even if no reply was sent. Separate the branches.",
                        );
                    }
                    if self.delivery.is_none() {
                        return fail(
                            Kind::Unsupported,
                            0,
                            "Rejection needs this lesson's simulated delivery profile; no actual refusal or bounce is sent.",
                        );
                    }
                    if self.out.rejection.is_some() {
                        return fail(
                            Kind::Runtime,
                            0,
                            "A filter cannot execute reject twice for one message. Use one matching branch or stop after the refusal.",
                        );
                    }
                    if !self.out.mailboxes.is_empty() || !self.out.redirects.is_empty() {
                        return fail(
                            Kind::Runtime,
                            0,
                            "This simulated site cannot both refuse and deliver the same message. Separate the matching branches; the original sample was kept in INBOX.",
                        );
                    }
                    self.action()?;
                    self.out.rejection = Some(reason.clone());
                }
                Statement::If {
                    branches,
                    otherwise,
                } => {
                    let mut selected = otherwise.as_slice();
                    for (test, body) in branches {
                        if self.test(test)? {
                            selected = body;
                            break;
                        }
                    }
                    if self.sequence(selected)? {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    fn test(&mut self, test: &Test) -> Result<bool, Fault> {
        self.charge(1)?;
        match test {
            Test::Constant(v) => Ok(*v),
            Test::Not(t) => self.test(t).map(|v| !v),
            Test::All(tests) => {
                for t in tests {
                    if !self.test(t)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Test::Any(tests) => {
                for t in tests {
                    if self.test(t)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Test::Size { over, bytes } => Ok(if *over {
                self.message.octets > *bytes
            } else {
                self.message.octets < *bytes
            }),
            Test::Exists(names) => {
                for name in names {
                    self.charge(self.message.headers.len())?;
                    if !valid_header(name)
                        || !self
                            .message
                            .headers
                            .iter()
                            .any(|(n, _)| n.eq_ignore_ascii_case(name))
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Test::Envelope {
                comparison,
                mode,
                part,
                names,
                keys,
            } => {
                let context=self.delivery.ok_or_else(||Fault{kind:Kind::Unsupported,offset:0,message:"This filter requires simulated envelope data. Visible From/To headers are not a substitute.".into()})?;
                for name in names {
                    let value = context.envelope.part(name, *part)?;
                    for key in keys {
                        self.charge((value.len() + 1).saturating_mul(key.len() + 1))?;
                        if matches_value(
                            value.as_bytes(),
                            key.as_bytes(),
                            *mode,
                            *comparison,
                            &self.clock,
                        )? {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            Test::Address {
                comparison,
                mode,
                part,
                names,
                keys,
            } => {
                for name in names {
                    if !crate::sieve_address::address_field(name) {
                        continue;
                    }
                    for i in 0..self.message.headers.len() {
                        self.charge(1)?;
                        let (header, value) = &self.message.headers[i];
                        if !header.eq_ignore_ascii_case(name) {
                            continue;
                        }
                        let length = value.len();
                        let addresses = crate::sieve_address::mailboxes(value)?;
                        self.charge(length + 1)?;
                        for address in addresses {
                            let value = match part {
                                AddressPart::All => address.all(),
                                AddressPart::Local => address.local,
                                AddressPart::Domain => address.domain,
                            };
                            for key in keys {
                                self.charge((value.len() + 1).saturating_mul(key.len() + 1))?;
                                if matches_value(
                                    value.as_bytes(),
                                    key.as_bytes(),
                                    *mode,
                                    *comparison,
                                    &self.clock,
                                )? {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
                Ok(false)
            }
            Test::Header {
                comparison,
                mode,
                names,
                keys,
            } => {
                for name in names {
                    if !valid_header(name) {
                        continue;
                    }
                    for i in 0..self.message.headers.len() {
                        self.charge(1)?;
                        let (header, value) = &self.message.headers[i];
                        if !header.eq_ignore_ascii_case(name) {
                            continue;
                        }
                        let value = value.trim_matches([' ', '\t']).as_bytes().to_vec();
                        for key in keys {
                            self.charge((value.len() + 1).saturating_mul(key.len() + 1))?;
                            if matches_value(
                                &value,
                                key.as_bytes(),
                                *mode,
                                *comparison,
                                &self.clock,
                            )? {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
        }
    }
}

fn equal(a: u8, b: u8, comparison: Comparison) -> bool {
    match comparison {
        Comparison::Octet => a == b,
        Comparison::AsciiCasemap => a.eq_ignore_ascii_case(&b),
    }
}
fn matches_value(
    value: &[u8],
    key: &[u8],
    mode: Match,
    comparison: Comparison,
    clock: &Clock,
) -> Result<bool, Fault> {
    match mode {
        Match::Is => Ok(value.len() == key.len()
            && value
                .iter()
                .zip(key)
                .all(|(&a, &b)| equal(a, b, comparison))),
        Match::Contains => Ok(key.is_empty()
            || (key.len() <= value.len()
                && value
                    .windows(key.len())
                    .any(|v| v.iter().zip(key).all(|(&a, &b)| equal(a, b, comparison))))),
        Match::Matches => {
            let mut prior = vec![false; value.len() + 1];
            prior[0] = true;
            let mut at = 0;
            while at < key.len() {
                clock.check()?;
                let mut next = vec![false; value.len() + 1];
                let current = key[at];
                at += 1;
                match current {
                    b'*' => {
                        next[0] = prior[0];
                        for j in 1..=value.len() {
                            next[j] = prior[j] || next[j - 1];
                        }
                    }
                    b'?' => {
                        next[1..].copy_from_slice(&prior[..value.len()]);
                    }
                    _ => {
                        let literal = if current == b'\\' {
                            if at == key.len() {
                                return fail(
                                    Kind::Syntax,
                                    0,
                                    "The wildcard pattern ends with an unpaired backslash.",
                                );
                            }
                            let literal = key[at];
                            at += 1;
                            literal
                        } else {
                            current
                        };
                        for j in 1..=value.len() {
                            next[j] = prior[j - 1] && equal(value[j - 1], literal, comparison);
                        }
                    }
                }
                prior = next;
            }
            Ok(prior[value.len()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sieve_elapsed_limit_is_fail_closed_without_waiting_or_stressing_the_machine() {
        let p = crate::sieve_parse::parse("discard;").unwrap();
        let m = Message::new(Vec::new(), "sample").unwrap();
        let boxes = BTreeSet::from([String::from("INBOX")]);
        let result = p.evaluate_timed(&m, &boxes, Duration::ZERO);
        assert_eq!(result.fault.as_ref().unwrap().kind, Kind::Limit);
        assert!(
            result
                .fault
                .as_ref()
                .unwrap()
                .message
                .contains("elapsed-time")
        );
        assert_eq!(result.mailboxes, boxes);
        assert!(result.implicit_keep);
        assert!(!result.discarded);
        assert_eq!(result.actions, 0);
        assert_eq!(result.work, 0);
    }
}
