//! Separate source-bound guided and independent mail-filter drafts and runs.
use crate::sieve_lex::Kind;
use crate::{
    script_teaching::View,
    sieve_eval::{Message, Outcome, valid_mailbox},
    sieve_parse::{self, Program},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
const RUN_BUDGET: Duration = Duration::from_millis(crate::sieve_eval::MAX_ELAPSED_MILLIS);
pub const MAX_CASES: usize = 12;
/// Reply history is derived from the ordered samples, never trusted saved input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VacationTask {
    pub user: crate::sieve_vacation_policy::Policy,
    pub required_reply_text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub label: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub expected: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<crate::sieve_envelope::Input>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub expected_redirects: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub expected_rejection: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_reply: Option<bool>,
}
impl Case {
    fn matches(&self, r: &CaseResult) -> bool {
        r.actual == self.expected
            && r.redirects == self.expected_redirects
            && r.rejection.is_some() == self.expected_rejection
            && r.rejection
                .as_ref()
                .is_none_or(|reason| !reason.trim().is_empty())
            && self.expected_reply.is_none_or(|expected| {
                expected
                    == matches!(
                        r.vacation,
                        Some(crate::sieve_vacation_policy::RunResult::Sent { .. })
                    )
            })
    }
    pub fn expected_description(&self) -> String {
        let mut description = describe_effects(
            &self.expected,
            &self.expected_redirects,
            self.expected_rejection.then_some("a nonblank reason"),
        );
        if let Some(reply) = self.expected_reply {
            description.push_str(if reply {
                "; prepare one vacation reply"
            } else {
                "; no vacation reply"
            });
        }
        description
    }
    pub fn message(&self) -> Result<Message, String> {
        Message::new(self.headers.clone(), &self.body).map_err(|e| e.to_string())
    }
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Inert legacy context for migration exercises, never parsed or executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_recipe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vacation: Option<VacationTask>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub address_tests: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub delivery_actions: bool,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub redirect_targets: BTreeSet<String>,
    pub starter: String,
    pub mailboxes: BTreeSet<String>,
    pub cases: Vec<Case>,
    pub reference: String,
}
fn is_false(value: &bool) -> bool {
    !*value
}
impl Task {
    pub fn parse(&self, source: &str) -> Result<Program, crate::sieve_lex::Fault> {
        if self.vacation.is_some() {
            sieve_parse::parse_vacation(source)
        } else if self.delivery_actions {
            sieve_parse::parse_delivery(source)
        } else if self.address_tests {
            sieve_parse::parse_address(source)
        } else {
            sieve_parse::parse(source)
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.legacy_recipe.as_ref().is_some_and(|text| {
            text.trim().is_empty()
                || text.len() > 4096
                || !text.bytes().all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\n' | b'\t'))
        }) {
            return Err("Legacy recipe context must be nonblank plain text up to4 KiB; it is read only and never executed.".into());
        }
        if let Some(v) = &self.vacation {
            if !self.delivery_actions
                || v.required_reply_text.trim().is_empty()
                || v.required_reply_text.len() > crate::sieve_lex::MAX_STRING
            {
                return Err("Vacation tasks need the disclosed delivery profile and a short required reply message or contact detail.".into());
            }
            v.user.validate().map_err(|e| e.to_string())?;
        }
        if self.delivery_actions && !self.address_tests
            || !self.delivery_actions && !self.redirect_targets.is_empty()
        {
            return Err("Delivery actions need the disclosed address profile and bounded simulated forwarding targets.".into());
        }
        let policy = crate::sieve_delivery_policy::Policy {
            targets: self.redirect_targets.clone(),
        };
        policy.validate().map_err(|e| e.to_string())?;
        if self.starter.len() > crate::sieve_lex::MAX_SOURCE
            || self.reference.len() > crate::sieve_lex::MAX_SOURCE
            || !self.mailboxes.contains("INBOX")
            || self.mailboxes.len() > 16
            || self.mailboxes.iter().any(|s| !valid_mailbox(s))
            || self.cases.is_empty()
            || self.cases.len() > MAX_CASES
        {
            return Err("Mail filters need bounded source, INBOX and named virtual mailboxes, and1to12 sample cases.".into());
        }
        let mut ids = BTreeSet::new();
        let mut total = 0usize;
        let mut last_arrival = None;
        for c in &self.cases {
            if self.vacation.is_some() {
                let at = c.arrival_seconds.ok_or("Every vacation sample needs its arrival time in seconds on the shared virtual timeline.")?;
                if last_arrival.is_some_and(|last| at < last)
                    || c.expected_reply.is_none()
                    || c.expected_rejection && c.expected_reply == Some(true)
                {
                    return Err("Vacation samples need ordered arrival times and compatible explicit reply expectations.".into());
                }
                last_arrival = Some(at);
            } else if c.arrival_seconds.is_some() || c.expected_reply.is_some() {
                return Err("Only a disclosed vacation timeline can contain arrival times or reply expectations.".into());
            }
            if !id(&c.id)
                || !ids.insert(&c.id)
                || c.label.is_empty()
                || c.label.len() > 160
                || !c.expected.is_subset(&self.mailboxes)
            {
                return Err("Filter cases need unique stable identifiers, clear labels and supplied mailbox outcomes.".into());
            }
            if self.delivery_actions {
                c.envelope
                    .as_ref()
                    .ok_or("Delivery cases need explicit simulated envelope data.")?
                    .validate()
                    .map_err(|e| e.to_string())?;
                if !c.expected_redirects.is_subset(&self.redirect_targets)
                    || c.expected_rejection
                        && (!c.expected.is_empty() || !c.expected_redirects.is_empty())
                {
                    return Err("Expected delivery effects must follow this site's named-target/refusal policy.".into());
                }
            } else if c.envelope.is_some()
                || !c.expected_redirects.is_empty()
                || c.expected_rejection
            {
                return Err("Basic/address cases cannot claim undisclosed delivery effects or envelope data.".into());
            }
            let m = c.message()?;
            total += m.octets() as usize;
        }
        if total > 256 * 1024 {
            return Err("All sample messages together exceed256 KiB.".into());
        }
        let p = self
            .parse(&self.reference)
            .map_err(|e| format!("The authored filter method is invalid: {e}"))?;
        let started = Instant::now();
        if self.cases.iter().any(|c| {
            !self
                .evaluate_timed(c, &p, RUN_BUDGET.saturating_sub(started.elapsed()))
                .passed
        }) {
            return Err(
                "The authored filter method does not meet every sample's required outcome.".into(),
            );
        }
        Ok(())
    }
    pub fn evaluate(&self, c: &Case, p: &Program) -> CaseResult {
        self.evaluate_timed(c, p, RUN_BUDGET)
    }
    fn evaluate_timed(&self, c: &Case, p: &Program, remaining: Duration) -> CaseResult {
        let mut history = crate::sieve_vacation_history::History::default();
        if self.vacation.is_none() {
            return self.evaluate_once(c, p, remaining, &mut history);
        }
        // Bounded prefix replay makes frame-by-frame execution, grading and
        // restored partial results agree without serializing a forgeable cache.
        let Some(index) = self.cases.iter().position(|sample| sample == c) else {
            return CaseResult::failed(
                c,
                Kind::Runtime,
                "Choose a supplied sample from this vacation timeline.".into(),
            );
        };
        if index >= MAX_CASES {
            return CaseResult::failed(
                c,
                Kind::Limit,
                "This timeline exceeds the12sample limit.".into(),
            );
        }
        let started = Instant::now();
        for sample in &self.cases[..=index] {
            let r = self.evaluate_once(
                sample,
                p,
                remaining.saturating_sub(started.elapsed()),
                &mut history,
            );
            if sample == c {
                return r;
            }
            if r.fault_kind == Some(Kind::Limit) {
                return CaseResult::failed(c, Kind::Limit, "The shared timeline reached its execution limit. Shorten the filter and retry.".into());
            }
        }
        unreachable!("The selected sample belongs to the bounded prefix")
    }
    fn evaluate_once(
        &self,
        c: &Case,
        p: &Program,
        remaining: Duration,
        history: &mut crate::sieve_vacation_history::History,
    ) -> CaseResult {
        let evaluated = c.message().and_then(|m| {
            if self.delivery_actions {
                let input = c
                    .envelope
                    .as_ref()
                    .ok_or("This sample has no simulated envelope; headers cannot replace it.")?;
                let envelope = input.validate().map_err(|e| e.to_string())?;
                let policy = crate::sieve_delivery_policy::Policy {
                    targets: self.redirect_targets.clone(),
                };
                if let Some(v) = &self.vacation {
                    let at_seconds = c
                        .arrival_seconds
                        .ok_or("This vacation sample has no virtual arrival time.")?;
                    Ok(p.evaluate_vacation_timed(
                        &m,
                        &self.mailboxes,
                        crate::sieve_eval::VacationContext {
                            envelope: &envelope,
                            forwarding: &policy,
                            replies: &v.user,
                            at_seconds,
                        },
                        history,
                        remaining,
                    ))
                } else {
                    Ok(p.evaluate_delivery_timed(
                        &m,
                        &self.mailboxes,
                        &envelope,
                        &policy,
                        remaining,
                    ))
                }
            } else {
                Ok(p.evaluate_timed(&m, &self.mailboxes, remaining))
            }
        });
        match evaluated {
            Err(e) => CaseResult::failed(c, Kind::Runtime, e),
            Ok(Outcome {
                mailboxes,
                redirects,
                rejection,
                vacation,
                implicit_keep,
                discarded,
                fault,
                ..
            }) => {
                let mut r = CaseResult {
                    id: c.id.clone(),
                    passed: false,
                    actual: mailboxes,
                    redirects,
                    rejection,
                    vacation,
                    implicit_keep,
                    discarded,
                    fault_kind: fault.as_ref().map(|e| e.kind),
                    error: fault.map(|e| format!("{}: {}", e.kind.label(), e.message)),
                };
                r.passed = r.error.is_none()
                    && c.matches(&r)
                    && self.vacation.as_ref().is_none_or(|v| match &r.vacation {
                        Some(crate::sieve_vacation_policy::RunResult::Sent { reply, .. }) => {
                            reply.body.contains(&v.required_reply_text)
                        }
                        _ => true,
                    });
                r
            }
        }
    }
    /// Opt-in profile explanations; older basic/address lesson wording stays intact.
    pub fn delivery_hint(&self, c: &Case, r: &CaseResult) -> Option<&'static str> {
        if self.legacy_recipe.is_some() {
            return Some(if c.expected.contains("INBOX") && !r.actual.contains("INBOX") {
                "The copied message still needs its original in INBOX. A Sieve fileinto cancels implicit keep; check the explicit default-delivery path."
            } else if !c.expected.contains("INBOX") && r.actual == BTreeSet::from([String::from("INBOX")]) {
                "No required filing happened. Compare the named header and complete value, then check that the matching delivery rules are reachable."
            } else if !c.expected.contains("INBOX") && r.actual.contains("INBOX") {
                "This message needs final delivery without an INBOX copy. Check whether processing reaches a default keep after the final rule."
            } else if c.expected.len() > r.actual.len() {
                "One required destination is missing. The copy rule must let the original continue to a later matching delivery rule; check any early stop."
            } else {
                "Compare the complete header values and mailbox sets. A body mention or substring lookalike must not match these whole-value rules."
            });
        }
        if self.vacation.is_some() && c.expected_reply.is_some() {
            return Some(
                if c.expected_reply == Some(true)
                    && !matches!(
                        r.vacation,
                        Some(crate::sieve_vacation_policy::RunResult::Sent { .. })
                    )
                {
                    "This personal message needs a vacation reply. Compare the virtual arrival with the last reply to this sender, the selected interval and any earlier stop."
                } else if c.expected_reply == Some(false)
                    && matches!(
                        r.vacation,
                        Some(crate::sieve_vacation_policy::RunResult::Sent { .. })
                    )
                {
                    "This arrival must not receive another reply. Check the shared sender history and interval; list, automatic and null-return messages need no reply."
                } else if !r.passed
                    && r.actual == c.expected
                    && r.redirects == c.expected_redirects
                    && r.rejection.is_some() == c.expected_rejection
                {
                    "The reply was prepared, but its message is missing the required text or contact detail. Use your own wording around that required detail."
                } else {
                    "A vacation reply does not replace delivery. Compare the required mailbox or forward with later filing, keep and stop actions."
                },
            );
        }
        if !self.delivery_actions {
            return None;
        }
        Some(if c.expected_rejection && r.rejection.is_none() {
            "This sample must be refused with a reason. Discard silently drops a message; it is a different outcome. Check the actual envelope and the refusal branch."
        } else if c.expected_rejection && r.rejection.as_ref().is_some_and(|s| s.trim().is_empty())
        {
            "The refusal needs a nonblank explanation so the sender can understand the decision. Choose your own clear wording."
        } else if r.rejection.is_some() && !c.expected_rejection {
            "This sample should be delivered, not refused. Compare the actual envelope values and narrow the refusal condition."
        } else if r.redirects != c.expected_redirects {
            "The virtual forwarding targets differ. Test the actual envelope recipient, not just visible To/Cc text, and check that unrelated samples stay local."
        } else if !r.redirects.is_empty() && r.actual != c.expected {
            "Forwarding cancels the default INBOX copy, but an explicit keep adds it back. Check later rules and where processing should stop."
        } else {
            "Compare the actual delivery envelope with the named mailbox condition. Visible From/To/Cc headers can differ; filing and forwarding do not stop later rules."
        })
    }
    pub fn complete(&self, text: &str) -> bool {
        let started = Instant::now();
        self.parse(text).is_ok_and(|p| {
            self.cases.iter().all(|c| {
                self.evaluate_timed(c, &p, RUN_BUDGET.saturating_sub(started.elapsed()))
                    .passed
            })
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseResult {
    pub id: String,
    pub passed: bool,
    pub actual: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub redirects: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vacation: Option<crate::sieve_vacation_policy::RunResult>,
    pub implicit_keep: bool,
    pub discarded: bool,
    pub error: Option<String>,
    pub fault_kind: Option<Kind>,
}
impl CaseResult {
    fn failed(c: &Case, kind: Kind, error: String) -> Self {
        Self {
            id: c.id.clone(),
            passed: false,
            actual: BTreeSet::new(),
            redirects: BTreeSet::new(),
            rejection: None,
            vacation: None,
            implicit_keep: false,
            discarded: false,
            error: Some(error),
            fault_kind: Some(kind),
        }
    }
    pub fn description(&self) -> String {
        let mut description =
            describe_effects(&self.actual, &self.redirects, self.rejection.as_deref());
        if let Some(reply) = &self.vacation {
            description.push_str("; ");
            description.push_str(&reply.description());
        }
        description
    }
}
fn describe_effects(
    mailboxes: &BTreeSet<String>,
    redirects: &BTreeSet<String>,
    rejection: Option<&str>,
) -> String {
    let mut effects = Vec::new();
    if !mailboxes.is_empty() {
        effects.push(format!(
            "Mailboxes: {}",
            mailboxes.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if !redirects.is_empty() {
        effects.push(format!(
            "Virtual forward: {}",
            redirects.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if let Some(reason) = rejection {
        effects.push(format!("Refused: {reason}"));
    }
    if effects.is_empty() {
        String::from("Discard, no mailbox copy or forward")
    } else {
        effects.join("; ")
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub draft: String,
    pub selected: usize,
    pub results: Vec<CaseResult>,
    pub error: Option<String>,
    pub checked: bool,
    pub running: bool,
    pub stopped: bool,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            selected: 0,
            results: Vec::new(),
            error: None,
            checked: false,
            running: false,
            stopped: false,
        }
    }
    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if text.len() > crate::sieve_lex::MAX_SOURCE {
            return Err(
                "The script exceeds the16 KiB source limit; shorten it before running.".into(),
            );
        }
        if self.draft != text {
            self.draft = text;
            self.results.clear();
            self.error = None;
            self.checked = false;
            self.running = false;
            self.stopped = false;
        }
        Ok(())
    }
    pub fn start(&mut self) {
        let parsed = sieve_parse::parse(&self.draft);
        self.begin(parsed);
    }
    pub fn start_for(&mut self, task: &Task) {
        let parsed = task.parse(&self.draft);
        self.begin(parsed);
    }
    fn begin(&mut self, parsed: Result<Program, crate::sieve_lex::Fault>) {
        self.results.clear();
        self.error = None;
        self.checked = false;
        self.running = false;
        self.stopped = false;
        match parsed {
            Ok(_) => self.running = true,
            Err(e) => self.error = Some(e.explain(&self.draft)),
        }
    }
    /// At most one bounded message per frame. No worker or host process exists.
    pub fn tick(&mut self, t: &Task) {
        if !self.running {
            return;
        }
        let Some(c) = t.cases.get(self.results.len()) else {
            self.running = false;
            self.checked = true;
            return;
        };
        match t.parse(&self.draft) {
            Err(e) => {
                self.running = false;
                self.checked = false;
                self.error = Some(e.explain(&self.draft));
            }
            Ok(p) => {
                let result = t.evaluate(c, &p);
                if result.fault_kind == Some(Kind::Limit) {
                    self.running = false;
                    self.checked = false;
                    self.error = result.error;
                    self.results.clear();
                    return;
                }
                self.results.push(result);
                if self.results.len() == t.cases.len() {
                    self.running = false;
                    self.checked = true;
                }
            }
        }
    }
    pub fn stop(&mut self) {
        if self.running {
            self.running = false;
            self.checked = false;
            self.stopped = true;
        }
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checked
            && !self.running
            && self.error.is_none()
            && self.results.len() == t.cases.len()
            && self.results.iter().all(|r| r.passed)
            && t.complete(&self.draft)
    }
    fn structural(&self, t: &Task) -> Result<(), String> {
        if self.draft.len() > crate::sieve_lex::MAX_SOURCE
            || self.selected >= t.cases.len()
            || self.results.len() > t.cases.len()
            || self.error.as_ref().is_some_and(|e| e.len() > 2048)
            || self.checked && self.running
            || self.stopped && self.running
            || self.checked && (self.error.is_some() || self.stopped)
            || self.running && self.results.len() == t.cases.len()
        {
            return Err(
                "Saved filter work exceeds this lesson's bounds; retain the original for recovery."
                    .into(),
            );
        }
        if self.checked && self.results.len() != t.cases.len() {
            return Err("A completed filter run needs every sample result.".into());
        }
        for (c, r) in t.cases.iter().zip(&self.results) {
            if r.id != c.id
                || !r.actual.is_subset(&t.mailboxes)
                || r.error.as_ref().is_some_and(|e| e.len() > 2048)
                || r.error.is_some() != r.fault_kind.is_some()
                || !r.redirects.is_subset(&t.redirect_targets)
                || r.vacation.as_ref().is_some_and(|v| v.validate().is_err())
                || t.vacation.is_none() && r.vacation.is_some()
                || (r.error.is_some() || r.rejection.is_some()) && r.vacation.is_some()
                || !t.delivery_actions && (!r.redirects.is_empty() || r.rejection.is_some())
                || r.rejection
                    .as_ref()
                    .is_some_and(|reason| reason.len() > crate::sieve_lex::MAX_STRING)
                || r.rejection.is_some() && (!r.actual.is_empty() || !r.redirects.is_empty())
                || r.passed && (!c.matches(r) || r.error.is_some())
                || r.discarded
                    && (!r.actual.is_empty()
                        || !r.redirects.is_empty()
                        || r.rejection.is_some()
                        || r.error.is_some())
                || r.implicit_keep
                    && (r.actual != BTreeSet::from([String::from("INBOX")])
                        || !r.redirects.is_empty()
                        || r.rejection.is_some())
            {
                return Err("Saved filter result fields do not match this sample's bounds.".into());
            }
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        self.structural(t)?;
        if !self.results.is_empty() {
            let started = Instant::now();
            let p = t
                .parse(&self.draft)
                .map_err(|_| "Saved filter results do not belong to a parsable current draft.")?;
            for (c, r) in t.cases.iter().zip(&self.results) {
                let observed =
                    t.evaluate_timed(c, &p, RUN_BUDGET.saturating_sub(started.elapsed()));
                if observed.fault_kind == Some(Kind::Limit) {
                    return Err("The saved draft is intact, but its earlier results could not be verified within the current resource limit. Retain the original and retry when resources are available.".into());
                }
                if observed != *r {
                    return Err("Saved filter results differ from current sample behavior; retain the original.".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema: u32,
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema: u32,
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit { script: String },
    Run,
    Tick,
    Stop,
    Select { index: usize },
    Reset,
}
impl Definition {
    pub fn checks(&self, s: &Session) -> Vec<Check> {
        let failed = |detail: String| {
            vec![Check {
                passed: false,
                label: "Run and verify the assignment filter".into(),
                detail,
            }]
        };
        if let Err(e) = self.structural(s) {
            return failed(e);
        }
        let scope = &s.assignment;
        if let Some(e) = &scope.error {
            return failed(e.clone());
        }
        if scope.running {
            return failed(format!(
                "Run in progress: {}/{} samples evaluated. Stop retains the draft; Run retries from the first sample.",
                scope.results.len(),
                self.assignment.cases.len()
            ));
        }
        if !scope.checked {
            return failed("Unchecked: run the assignment filter against every sample. Separate worked practice cannot earn assignment credit.".into());
        }
        let p = match self.assignment.parse(&scope.draft) {
            Ok(p) => p,
            Err(e) => return failed(e.explain(&scope.draft)),
        };
        let started = Instant::now();
        self.assignment.cases.iter().map(|c| {
            let r = self.assignment.evaluate_timed(c,&p,RUN_BUDGET.saturating_sub(started.elapsed()));
            let describe = |set: &BTreeSet<String>| if set.is_empty() { String::from("discard with no mailbox copy") } else { set.iter().cloned().collect::<Vec<_>>().join(", ") };
            let expected = if self.assignment.delivery_actions {c.expected_description()}else{describe(&c.expected)};
            let actual = if self.assignment.delivery_actions {r.description()}else{describe(&r.actual)};
            let detail = if let Some(e) = &r.error { e.clone() }
                else if r.passed { format!("Expected: {expected}. Actual: {actual}.") }
                else {
                    let why = if let Some(hint)=self.assignment.delivery_hint(c,&r) {hint} else if r.actual.len() > 1 { "More than one copy remains. Filing does not stop later rules, and an explicit keep adds an INBOX copy. Check rule order and where processing ends." }
                        else if self.assignment.address_tests { "Compare the parsed mailbox and the selected address part. Display names are ignored, exact domains exclude similar endings and subdomains, and local parts retain plus suffixes. Check From/To/Cc and the chosen comparator." }
                        else if r.implicit_keep { "No filing action matched, so the default INBOX copy was kept. Check the named header, whole value and combined condition." }
                        else if c.expected.contains("INBOX") { "This sample should remain in INBOX. The filter matched too broadly; compare exact values, missing fields and the strict size boundary." }
                        else { "A rule chose the wrong destination or discarded the sample. Compare the priority and full condition before changing the filing action." };
                    format!("Expected: {expected}. Actual: {actual}. {why}")
                };
            Check { passed:r.passed,label:c.label.clone(),detail }
        }).collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1 || !id(&self.source_id) {
            return Err("Use filter schema1 and a stable source identifier.".into());
        }
        if self.assignment.address_tests != self.example.address_tests
            || self.assignment.delivery_actions != self.example.delivery_actions
            || self.assignment.vacation.is_some() != self.example.vacation.is_some()
        {
            return Err(
                "Practice and assignment must disclose the same supported filter syntax.".into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.complete(&self.assignment.starter) {
            return Err("The independent filter starter must need meaningful work.".into());
        }
        if self.assignment.cases == self.example.cases {
            return Err(
                "Worked practice must use separate sample inputs from the assignment.".into(),
            );
        }
        if self.assignment.cases.len() < 4 || self.example.cases.len() < 3 {
            return Err(
                "Use at least four independent cases and three separate worked samples.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.structural(s)?;
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    fn structural(&self, s: &Session) -> Result<(), String> {
        if s.schema != 1 || s.source_id != self.source_id {
            return Err("Saved filter work belongs to another activity/version.".into());
        }
        s.assignment.structural(&self.assignment)?;
        s.practice.structural(&self.example)
    }
    pub fn solved(&self, s: &Session) -> bool {
        s.assignment.checked
            && self.structural(s).is_ok()
            && s.assignment.bound(&self.assignment).is_ok()
            && s.assignment.solved(&self.assignment)
    }
}
impl Session {
    /// Context-free decoding bounds. Source-bound checks additionally verify every
    /// saved result against this lesson's actual samples and current script.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1 || !id(&self.source_id) {
            return Err("Unsupported filter state identifier/version.".into());
        }
        for s in [&self.assignment, &self.practice] {
            if s.draft.len() > crate::sieve_lex::MAX_SOURCE
                || s.selected >= MAX_CASES
                || s.results.len() > MAX_CASES
                || s.error.as_ref().is_some_and(|e| e.len() > 2048)
                || s.checked && s.running
                || s.stopped && s.running
                || s.results.iter().any(|r| {
                    !id(&r.id)
                        || r.actual.len() > 16
                        || r.actual.iter().any(|m| !valid_mailbox(m))
                        || r.redirects.len() > crate::sieve_delivery_policy::MAX_REDIRECTS
                        || r.redirects
                            .iter()
                            .any(|s| crate::sieve_envelope::bare(s).is_err())
                        || r.rejection
                            .as_ref()
                            .is_some_and(|s| s.len() > crate::sieve_lex::MAX_STRING)
                        || r.vacation.as_ref().is_some_and(|v| v.validate().is_err())
                        || r.error.as_ref().is_some_and(|e| e.len() > 2048)
                        || r.error.is_some() != r.fault_kind.is_some()
                })
            {
                return Err("Filter state exceeds the supported draft/result bounds.".into());
            }
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            schema: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment),
            practice: Scope::new(&d.example),
        })
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<(), String> {
        d.structural(self)?;
        let (t, s) = if practice {
            (&d.example, &mut self.practice)
        } else {
            (&d.assignment, &mut self.assignment)
        };
        match a {
            Action::Edit { script } => s.edit(script.clone())?,
            Action::Run => s.start_for(t),
            Action::Tick => s.tick(t),
            Action::Stop => s.stop(),
            Action::Reset => *s = Scope::new(t),
            Action::Select { index } => {
                if *index >= t.cases.len() {
                    return Err("Choose a supplied sample message.".into());
                }
                s.selected = *index;
            }
        }
        d.structural(self)
    }
}
