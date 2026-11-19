//! Bounded configuration exercises over virtual DHCP clients; never host execution.
use crate::{dhcp_config as config, dhcpv4 as core, script_teaching::View};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt, net::Ipv4Addr};
pub const MAX_DEFINITION: usize = 131_072;
pub const MAX_SESSION: usize = 150_000;
pub const MAX_TICKS: usize = 64;
pub const PREFIX: &str = "DHCP CONFIG\n";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn document<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Text;
    impl Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a script-free configuration draft of at most16KiB")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > config::MAX_TEXT {
                Err(E::custom("Configuration draft byte limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text)
}
fn optional_document<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Opt;
    impl<'de> Visitor<'de> for Opt {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an optional bounded tested configuration")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            document(d).map(Some)
        }
    }
    d.deserialize_option(Opt)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} bounded items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                match a.next_element()? {
                    Some(x) => out.push(x),
                    None => return Ok(out),
                }
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Configuration exercise item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn fits<T: Serialize>(value: &T, max: usize) -> bool {
    struct Count {
        n: usize,
        max: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.max)
                .ok_or_else(|| std::io::Error::other("Configuration exercise byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, max }, value).is_ok()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Operation {
    Acquire {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        client: String,
    },
    Renew {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        client: String,
    },
    Advance {
        seconds: u64,
    },
}
impl Operation {
    fn client(&self) -> Option<&str> {
        match self {
            Self::Acquire { client } | Self::Renew { client } => Some(client),
            Self::Advance { .. } => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Parameters {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        client: String,
        subnet: core::Subnet,
        router: Option<Ipv4Addr>,
        #[serde(deserialize_with = "list::<_,Ipv4Addr,4>")]
        dns: Vec<Ipv4Addr>,
    },
    Duration {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        client: String,
        seconds: u32,
    },
    PreserveBinding {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        client: String,
    },
    PoolWindow {
        subnet: core::Subnet,
        first: Ipv4Addr,
        last: Ipv4Addr,
    },
    PoolsOnly {
        #[serde(deserialize_with = "list::<_,core::Subnet,8>")]
        subnets: Vec<core::Subnet>,
    },
}
impl Requirement {
    fn actor(&self) -> Option<&str> {
        match self {
            Self::Parameters { client, .. }
            | Self::Duration { client, .. }
            | Self::PreserveBinding { client } => Some(client),
            _ => None,
        }
    }
    fn identity(&self) -> String {
        match self {
            Self::Parameters { client, .. } => format!("parameters-{client}"),
            Self::Duration { client, .. } => format!("duration-{client}"),
            Self::PreserveBinding { client } => format!("preserve-{client}"),
            Self::PoolWindow { subnet, .. } => {
                format!("range-{}/{}", subnet.network, subnet.prefix)
            }
            Self::PoolsOnly { .. } => "pools-only".into(),
        }
    }
    fn check(&self, before: &core::State, after: &core::State) -> (bool, String) {
        match self {
            Self::Parameters {
                client,
                subnet,
                router,
                dns,
            } => {
                let b = after.leases().get(client);
                let pass = b.is_some_and(|b| {
                    b.parameters.subnet == *subnet
                        && b.parameters.router == *router
                        && b.parameters.dns == *dns
                });
                (
                    pass,
                    match b {
                        Some(b) => format!(
                            "{client}: expected router {} and DNS {:?} on {}/{}; actual router {}, DNS {:?} on {}/{}. A local option overrides its inherited default.",
                            router
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "not supplied".into()),
                            dns,
                            subnet.network,
                            subnet.prefix,
                            b.parameters
                                .router
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "not supplied".into()),
                            b.parameters.dns,
                            b.parameters.subnet.network,
                            b.parameters.subnet.prefix
                        ),
                        None => format!(
                            "{client}: no usable model lease; inspect the dynamic range, available addresses and exchange result."
                        ),
                    },
                )
            }
            Self::Duration { client, seconds } => {
                let got = after.leases().get(client).map(|b| b.granted_seconds);
                (
                    got == Some(*seconds),
                    format!(
                        "{client}: expected a {seconds}s grant; actual {}. Scoped defaults and the maximum both matter.",
                        got.map(|x| format!("{x}s"))
                            .unwrap_or_else(|| "no lease".into())
                    ),
                )
            }
            Self::PreserveBinding { client } => {
                let pass = before.leases().get(client).is_some()
                    && before.leases().get(client) == after.leases().get(client);
                (
                    pass,
                    format!(
                        "{client}: {}. A reload must not rewrite an already held lease.",
                        if pass {
                            "the supplied binding is preserved"
                        } else {
                            "the supplied binding changed or disappeared"
                        }
                    ),
                )
            }
            Self::PoolWindow {
                subnet,
                first,
                last,
            } => {
                let pools = after
                    .policy()
                    .pools
                    .values()
                    .filter(|p| p.subnet == *subnet)
                    .collect::<Vec<_>>();
                let pass = pools.len() == 1 && pools[0].first >= *first && pools[0].last <= *last;
                (
                    pass,
                    format!(
                        "{}/{}: dynamic range must stay inside {first} through {last}; actual {}.",
                        subnet.network,
                        subnet.prefix,
                        pools
                            .first()
                            .map(|p| format!("{} through {}", p.first, p.last))
                            .unwrap_or_else(|| "no matching dynamic pool".into())
                    ),
                )
            }
            Self::PoolsOnly { subnets } => {
                let actual = after
                    .policy()
                    .pools
                    .values()
                    .map(|p| (p.subnet.network, p.subnet.prefix))
                    .collect::<BTreeSet<_>>();
                let wanted = subnets
                    .iter()
                    .map(|s| (s.network, s.prefix))
                    .collect::<BTreeSet<_>>();
                (
                    actual == wanted,
                    format!(
                        "Dynamic subnets: expected {}; actual {}.",
                        wanted
                            .iter()
                            .map(|(n, p)| format!("{n}/{p}"))
                            .collect::<Vec<_>>()
                            .join(", "),
                        actual
                            .iter()
                            .map(|(n, p)| format!("{n}/{p}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub initial: core::State,
    #[serde(deserialize_with = "list::<_,Operation,16>")]
    pub actions: Vec<Operation>,
    #[serde(deserialize_with = "list::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "document")]
    pub starter: String,
    #[serde(deserialize_with = "list::<_,Case,3>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self
                .instructions
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            || !draft_bound(&self.starter)
            || !(2..=3).contains(&self.cases.len())
            || !fits(self, MAX_DEFINITION)
        {
            return Err("Invalid bounded DHCP configuration task".into());
        }
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            c.initial.validate().map_err(|e| e.to_string())?;
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || c.actions.is_empty()
                || c.actions.len() > 16
                || c.requirements.is_empty()
                || c.requirements.len() > 12
                || !c.initial.offers().is_empty()
            {
                return Err("Invalid or duplicate DHCP configuration case".into());
            }
            let mut goals = BTreeSet::new();
            let mut has_pools = false;
            for r in &c.requirements {
                if !goals.insert(r.identity())
                    || r.actor()
                        .is_some_and(|id| !c.initial.topology().clients.contains_key(id))
                {
                    return Err("Duplicate or unknown DHCP outcome reference".into());
                }
                let valid = match r {
                    Requirement::Parameters { subnet, dns, .. } => subnet.valid() && dns.len() <= 4,
                    Requirement::Duration { seconds, .. } => {
                        (core::MIN_LEASE..=core::MAX_LEASE).contains(seconds)
                    }
                    Requirement::PreserveBinding { client } => {
                        c.initial.leases().contains_key(client)
                    }
                    Requirement::PoolWindow {
                        subnet,
                        first,
                        last,
                    } => {
                        subnet.valid()
                            && subnet.host(*first)
                            && subnet.host(*last)
                            && u32::from(*first) <= u32::from(*last)
                    }
                    Requirement::PoolsOnly { subnets } => {
                        has_pools = true;
                        !subnets.is_empty()
                            && subnets.len() <= 8
                            && subnets.iter().all(|s| s.valid())
                            && subnets
                                .iter()
                                .map(|s| (s.network, s.prefix))
                                .collect::<BTreeSet<_>>()
                                .len()
                                == subnets.len()
                    }
                };
                if !valid {
                    return Err("Invalid DHCP configuration outcome".into());
                }
            }
            if !has_pools {
                return Err(
                    "Each case must explicitly bound all configured dynamic subnets".into(),
                );
            }
            let mut elapsed = 0u64;
            for op in &c.actions {
                if let Some(client) = op.client() {
                    if !c.initial.topology().clients.contains_key(client)
                        || !c.requirements.iter().any(|r| r.actor() == Some(client))
                    {
                        return Err("Every acted-on client needs a known outcome".into());
                    }
                } else if let Operation::Advance { seconds } = op {
                    if *seconds > core::MAX_ADVANCE {
                        return Err("Virtual clock action exceeds the existing bound".into());
                    }
                    elapsed = elapsed
                        .checked_add(*seconds)
                        .filter(|n| *n <= core::MAX_TIME - c.initial.now())
                        .ok_or("Virtual clock overflows supplied context")?;
                }
            }
        }
        Ok(())
    }
    pub fn run(&self, text: &str) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, text)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.results);
            }
        }
        Err("Configuration checking exceeded its bounded action budget".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub action: String,
    pub ok: bool,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub diagnostic: Option<String>,
    pub checks: Vec<(bool, String)>,
    pub events: Vec<Event>,
    pub after: Option<core::State>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none()
            && !self.checks.is_empty()
            && self.checks.iter().all(|r| r.0)
            && self.events.iter().all(|e| e.ok)
    }
}
fn reason(r: core::Reason) -> &'static str {
    match r {
        core::Reason::ServerDisabled => "The modeled server is stopped.",
        core::Reason::NoRelay => "No relay is supplied for this segment.",
        core::Reason::WrongRelayTarget => "The relay points to a different server.",
        core::Reason::WrongRelayAddress => "The relay's subnet selector does not match.",
        core::Reason::RelayUnreachable => "The supplied relay path cannot reach the server.",
        core::Reason::UnicastUnreachable => "The supplied unicast path cannot reach the server.",
        core::Reason::NoPool => "No configured dynamic pool matches this client's subnet.",
        core::Reason::PoolChanged => {
            "The pool changed during an outstanding offer; this exchange cannot finish."
        }
        core::Reason::Exhausted => "No available address remains in the dynamic range.",
        core::Reason::ReservationUnavailable => "The reserved address is unavailable.",
        core::Reason::NoOffer => "The client has no current offer.",
        core::Reason::StaleOffer => "The offer is no longer current.",
    }
}
fn perform(state: &mut core::State, op: &Operation) -> Event {
    let label = match op {
        Operation::Acquire { client } => format!("Request configuration · {client}"),
        Operation::Renew { client } => format!("Renew configuration · {client}"),
        Operation::Advance { seconds } => format!("Advance virtual time · {seconds}s"),
    };
    let result = (|| -> Result<core::Outcome, String> {
        match op {
            Operation::Acquire { client } => {
                if state.leases().contains_key(client) || state.bindings().contains_key(client) {
                    return Err("Unsupported exercise context: a fresh request needs a client without a held binding. Held-lease rediscovery/cache behavior is not modeled here; use a supplied eligible renewal.".into());
                }
                let o = state.discover(client).map_err(|e| e.to_string())?;
                if let core::Outcome::Offered(offer) = o {
                    state
                        .request(client, offer.token)
                        .map_err(|e| e.to_string())
                } else {
                    Ok(o)
                }
            }
            Operation::Renew { client } => {
                if state.window(client).map_err(|e| e.to_string())? != core::Window::Renewal {
                    return Err("Unsupported exercise context: renew only in the supplied renewal window; early/cache/rebinding behavior is not modeled here.".into());
                }
                let old = state
                    .leases()
                    .get(client)
                    .ok_or("No current client lease to renew")?;
                if !state.policy().pools.values().any(|p| {
                    p.subnet == old.parameters.subnet
                        && old.address >= p.first
                        && old.address <= p.last
                        && !p.excluded.contains(&old.address)
                        && p.reservations.get(client).is_none_or(|a| *a == old.address)
                }) {
                    return Err(format!(
                        "Unsupported renewal context: {client}'s current address is outside the new eligible pool. Keep active clients eligible; this activity does not simulate an ISC-specific invalid-lease reply."
                    ));
                }
                state.renew(client).map_err(|e| e.to_string())
            }
            Operation::Advance { seconds } => state.advance(*seconds).map_err(|e| e.to_string()),
        }
    })();
    match result {
        Err(e) => Event {
            action: label,
            ok: false,
            message: e.chars().take(512).collect(),
        },
        Ok(core::Outcome::Ack { binding, .. }) => Event {
            action: label,
            ok: true,
            message: format!(
                "Client holds {} with router {}, DNS {:?}, grant {}s.",
                binding.address,
                binding
                    .parameters
                    .router
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "not supplied".into()),
                binding.parameters.dns,
                binding.granted_seconds
            ),
        },
        Ok(core::Outcome::NoReply(r)) => Event {
            action: label,
            ok: false,
            message: reason(r).into(),
        },
        Ok(core::Outcome::Advanced { .. }) => Event {
            action: label,
            ok: true,
            message:
                "Only the virtual clock advanced; no real time or automatic client packet was used."
                    .into(),
        },
        Ok(other) => Event {
            action: label,
            ok: false,
            message: format!("Exchange did not finish: {other:?}")
                .chars()
                .take(512)
                .collect(),
        },
    }
}
struct Working {
    state: core::State,
    events: Vec<Event>,
}
pub struct Run {
    task: Task,
    text: String,
    index: usize,
    action: usize,
    working: Option<Working>,
    results: Vec<CaseResult>,
    cancelled: bool,
}
impl Run {
    pub fn new(task: &Task, text: &str) -> Result<Self, String> {
        task.validate()?;
        if !draft_bound(text) {
            return Err("Use configuration text of at most16KiB without control characters; previous work retained".into());
        }
        Ok(Self {
            task: task.clone(),
            text: text.into(),
            index: 0,
            action: 0,
            working: None,
            results: vec![],
            cancelled: false,
        })
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.working = None;
        self.results.clear();
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.index == self.task.cases.len()
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.results)
    }
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        let case = &self.task.cases[self.index];
        if self.working.is_none() {
            match config::compile(
                &self.text,
                case.initial.topology(),
                &case.initial.policy().relays,
            ) {
                Err(e) => {
                    self.results.push(CaseResult {
                        id: case.id.clone(),
                        label: case.label.clone(),
                        diagnostic: Some(e.to_string()),
                        checks: vec![(false, format!("Configuration was not loaded: {e}"))],
                        events: vec![],
                        after: None,
                    });
                    self.index += 1;
                    return self.done();
                }
                Ok(policy) => {
                    let mut state = case.initial.clone();
                    if let Err(e) = state.replace_policy(policy) {
                        self.results.push(CaseResult {
                            id: case.id.clone(),
                            label: case.label.clone(),
                            diagnostic: Some(e.to_string()),
                            checks: vec![(false, "The supplied model refused this policy.".into())],
                            events: vec![],
                            after: None,
                        });
                        self.index += 1;
                        return self.done();
                    }
                    self.working = Some(Working {
                        state,
                        events: vec![],
                    });
                    self.action = 0;
                    return false;
                }
            }
        }
        let mut working = self.working.take().unwrap();
        if self.action < case.actions.len() {
            working
                .events
                .push(perform(&mut working.state, &case.actions[self.action]));
            self.action += 1;
            self.working = Some(working);
            return false;
        }
        let mut checks = vec![(
            working.events.iter().all(|e| e.ok),
            working
                .events
                .iter()
                .find(|e| !e.ok)
                .map(|e| format!("{}: {}", e.action, e.message))
                .unwrap_or_else(|| {
                    "Every supplied client action completed in the logical model.".into()
                }),
        )];
        checks.extend(
            case.requirements
                .iter()
                .map(|r| r.check(&case.initial, &working.state)),
        );
        self.results.push(CaseResult {
            id: case.id.clone(),
            label: case.label.clone(),
            diagnostic: None,
            checks,
            events: working.events,
            after: Some(working.state),
        });
        self.index += 1;
        self.done()
    }
}

fn draft_bound(text: &str) -> bool {
    text.len() <= config::MAX_TEXT
        && text
            .chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "document")]
    pub example_worked: String,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || !draft_bound(&self.example_worked)
            || !fits(self, MAX_DEFINITION)
        {
            return Err(
                "Configuration assignment and example need distinct bounded sources".into(),
            );
        }
        for a in &self.initial.cases {
            for b in &self.example.cases {
                if a.id == b.id
                    || a.initial
                        .topology()
                        .clients
                        .keys()
                        .any(|id| b.initial.topology().clients.contains_key(id))
                    || a.initial.topology().segments.values().any(|s| {
                        b.initial.topology().segments.values().any(|e| {
                            s.subnet.contains(e.subnet.network)
                                || e.subnet.contains(s.subnet.network)
                        })
                    })
                {
                    return Err("Practice client identities and networks must be separate from assignment data".into());
                }
            }
        }
        if self
            .initial
            .run(&self.initial.starter)?
            .iter()
            .all(CaseResult::passed)
            || self
                .example
                .run(&self.example.starter)?
                .iter()
                .all(CaseResult::passed)
        {
            return Err("Starters must retain meaningful unfinished configuration work".into());
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked configuration does not reach every example outcome".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err(
                "Configuration progress belongs to a different source task; work retained".into(),
            );
        }
        Ok(())
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if s.assignment.tested.as_deref() != Some(&s.assignment.draft) {
            return vec![(
                false,
                "Not checked · run the current configuration against the sample clients.".into(),
            )];
        }
        match self.initial.run(&s.assignment.draft) {
            Err(e) => vec![(false, e)],
            Ok(results) => results
                .into_iter()
                .flat_map(|r| {
                    r.checks
                        .into_iter()
                        .map(move |(ok, text)| (ok, format!("{} · {text}", r.label)))
                })
                .collect(),
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_DEFINITION {
            return Err("Configuration definition exceeds128KiB".into());
        }
        let d: Self = serde_json::from_str(text)
            .map_err(|e| format!("Invalid configuration definition: {e}"))?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "document")]
    pub draft: String,
    #[serde(deserialize_with = "optional_document")]
    pub tested: Option<String>,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            tested: None,
        }
    }
    pub fn replace(&mut self, text: &str) -> Result<(), String> {
        if !draft_bound(text) {
            return Err("Use at most16KiB of configuration text without control characters; previous draft retained".into());
        }
        if self.draft != text {
            self.draft = text.into();
            self.tested = None;
        }
        Ok(())
    }
    pub fn begin(&mut self, t: &Task) -> Result<Run, String> {
        self.tested = None;
        Run::new(t, &self.draft)
    }
    pub fn finish(&mut self, t: &Task, run: &Run) -> Result<(), String> {
        if !run.done() || run.text != self.draft || run.task != *t {
            return Err(
                "Stopped, unfinished or changed configuration run cannot award current credit"
                    .into(),
            );
        }
        self.tested = Some(self.draft.clone());
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(t)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(t, &run)?;
                return Ok(run.results);
            }
        }
        Err("Configuration run hit its bounded action limit".into())
    }
    pub fn passed(&self, t: &Task) -> bool {
        self.tested.as_deref() == Some(&self.draft)
            && t.run(&self.draft)
                .is_ok_and(|rows| !rows.is_empty() && rows.iter().all(CaseResult::passed))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || [&self.assignment, &self.practice].iter().any(|s| {
                !draft_bound(&s.draft) || s.tested.as_ref().is_some_and(|d| !draft_bound(d))
            })
            || !fits(self, MAX_SESSION)
        {
            Err("Configuration progress exceeds its version or draft limits".into())
        } else {
            Ok(())
        }
    }
    pub fn from_json(text: &str, d: &Definition) -> Result<Self, String> {
        if text.len() > MAX_SESSION {
            return Err("Configuration progress exceeds its byte limit".into());
        }
        let s: Self = serde_json::from_str(text)
            .map_err(|e| format!("Invalid configuration progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
}

/// Internal data-only authoring adapter, never a taught shell command.
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native configuration editor; this is internal reference data")?;
        self.assignment.replace(text)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Simulated configuration checked: {}/{} client cases passed",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}
pub const HELP: &str = "Edit the supplied ISC-style DHCP configuration subset, then Run client tests. Only virtual clients, addresses and time are used. Each test starts with its own sample inputs and held leases. Supported: subnet/network mask, one inclusive dynamic range per subnet, numeric router/DNS options, global/subnet defaults and lease durations. # starts a comment; semicolons end parameters. Includes, hooks, host/static declarations, real daemon commands and unsupported syntax are refused. This legacy-format exercise does not run ISC DHCP or model its wire/lease-file/cache behavior.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::DhcpConfig { simulation },
        crate::engine::Goal::DhcpConfig { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("DHCP configuration source/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || definition.initial.instructions != s.objective {
        return Err("DHCP source identity and independent task must match the scenario".into());
    }
    if **simulation != Session::new(definition)? {
        return Err("DHCP initial drafts differ from supplied source".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::DhcpConfig { simulation } = &a.state else {
        return Err("Saved DHCP configuration belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible DHCP attempt metadata; original work retained".into());
    }
    d.bound(simulation)
}
