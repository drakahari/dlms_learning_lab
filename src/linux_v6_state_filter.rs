//! Optional direction-state predicates for the bounded IPv6 TCP/UDP profile.
//! ICMPv6 stays stateless: no fabricated echo/error/RELATED tracking.
use crate::{linux_v6_connections::Class as State, linux_v6_filter as base};
use base::{Chain, Class, Fault, Packet, Protocol, Verdict};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Match {
    New,
    Established,
    Both,
}
impl Match {
    fn accepts(self, state: Option<State>) -> bool {
        matches!(
            (self, state),
            (Self::Both, Some(_))
                | (Self::New, Some(State::New))
                | (Self::Established, Some(State::Established))
        )
    }
    fn text(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Established => "ESTABLISHED",
            Self::Both => "NEW,ESTABLISHED",
        }
    }
    fn parse(s: &str) -> Result<Self, Fault> {
        let mut bits = 0;
        for item in s.split(',') {
            bits |= match item {
                "NEW" => 1,
                "ESTABLISHED" => 2,
                _ => {
                    return Err(fault(
                        Class::Unsupported,
                        "This IPv6 profile tracks NEW and ESTABLISHED TCP/UDP directions only; RELATED, INVALID, UNTRACKED and NAT states are not modeled.",
                    ));
                }
            };
        }
        match bits {
            1 => Ok(Self::New),
            2 => Ok(Self::Established),
            3 => Ok(Self::Both),
            _ => Err(fault(Class::Syntax, "Name NEW, ESTABLISHED or both.")),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub base: base::Table,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Option<Match>,96>")]
    pub input: Vec<Option<Match>>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Option<Match>,96>")]
    pub forward: Vec<Option<Match>>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Option<Match>,96>")]
    pub output: Vec<Option<Match>>,
}
const CHAINS: [Chain; 3] = [Chain::Input, Chain::Forward, Chain::Output];
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn chain(s: &str) -> Result<Chain, Fault> {
    match s {
        "INPUT" => Ok(Chain::Input),
        "FORWARD" => Ok(Chain::Forward),
        "OUTPUT" => Ok(Chain::Output),
        _ => Err(fault(
            Class::Unsupported,
            "Choose INPUT, FORWARD or OUTPUT.",
        )),
    }
}
fn rules_mut(t: &mut base::Table, c: Chain) -> &mut base::ChainRules {
    match c {
        Chain::Input => &mut t.input,
        Chain::Forward => &mut t.forward,
        Chain::Output => &mut t.output,
    }
}
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
impl Table {
    pub fn new(policy: Verdict) -> Self {
        Self {
            base: base::Table::new(policy),
            input: vec![],
            forward: vec![],
            output: vec![],
        }
    }
    fn masks(&self, c: Chain) -> &[Option<Match>] {
        match c {
            Chain::Input => &self.input,
            Chain::Forward => &self.forward,
            Chain::Output => &self.output,
        }
    }
    fn masks_mut(&mut self, c: Chain) -> &mut Vec<Option<Match>> {
        match c {
            Chain::Input => &mut self.input,
            Chain::Forward => &mut self.forward,
            Chain::Output => &mut self.output,
        }
    }
    pub fn validate(&self) -> Result<(), Fault> {
        self.base.validate()?;
        for c in CHAINS {
            let rules = &self.base.chain(c).rules;
            if self.masks(c).len() != rules.len() {
                return Err(fault(
                    Class::Context,
                    "Each state match must belong to one running IPv6 filter rule.",
                ));
            }
            for (r, m) in rules.iter().zip(self.masks(c)) {
                if m.is_some() && !matches!(r.protocol, Some(Protocol::Tcp | Protocol::Udp)) {
                    return Err(fault(
                        Class::Unsupported,
                        "State matches require explicit -p tcp or -p udp in this profile. ICMPv6 is a separate stateless header exercise.",
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn specialize(&self, state: Option<State>) -> Result<base::Table, Fault> {
        self.validate()?;
        let mut out = self.base.clone();
        for c in CHAINS {
            rules_mut(&mut out, c).rules = self
                .base
                .chain(c)
                .rules
                .iter()
                .zip(self.masks(c))
                .filter(|(_, m)| m.is_none_or(|m| m.accepts(state)))
                .map(|(r, _)| r.clone())
                .collect();
        }
        Ok(out)
    }
    pub fn evaluate(
        &self,
        c: Chain,
        p: &Packet,
        state: Option<State>,
    ) -> Result<base::Decision, Fault> {
        if (p.protocol == Protocol::Icmp6) != state.is_none() {
            return Err(fault(
                Class::Context,
                "Supply direction state for TCP/UDP and no invented connection state for ICMPv6.",
            ));
        }
        let mut decision = self.specialize(state)?.evaluate(c, p)?;
        if let Some(index) = decision.rule {
            decision.rule = Some(
                self.masks(c)
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.is_none_or(|m| m.accepts(state)))
                    .nth(index - 1)
                    .ok_or_else(|| fault(Class::Context, "IPv6 state rule index mismatch."))?
                    .0
                    + 1,
            );
        }
        Ok(decision)
    }
    /// Exact header comparison in one direction-state context. Rechecks the witness
    /// through the live table so feedback uses original, not specialized, row numbers.
    pub fn compare(
        &self,
        c: Chain,
        state: Option<State>,
        names: &[String],
        domains: &[base::Rule],
        allowed: &[base::Rule],
    ) -> Result<Option<crate::linux_v6_policy::Mismatch>, Fault> {
        if domains.iter().any(|r| match state {
            None => r.protocol != Some(Protocol::Icmp6),
            Some(_) => !matches!(r.protocol, Some(Protocol::Tcp | Protocol::Udp)),
        }) {
            return Err(fault(
                Class::Context,
                "Comparison domains must explicitly select the protocol for their direction-state context.",
            ));
        }
        let mut result = crate::linux_v6_policy::compare_filter(
            &self.specialize(state)?,
            c,
            names,
            domains,
            allowed,
        )?;
        if let Some(mismatch) = &mut result {
            let live = self.evaluate(c, &mismatch.packet, state)?;
            if live.verdict != mismatch.actual {
                return Err(fault(
                    Class::Context,
                    "IPv6 witness changed between comparison and live evaluation; no check accepted.",
                ));
            }
            mismatch.rule = live.rule;
        }
        Ok(result)
    }
    pub fn listing(&self, selected: Option<Chain>) -> Result<String, Fault> {
        self.validate()?;
        let (mut policies, mut rules) = (String::new(), String::new());
        for c in CHAINS
            .into_iter()
            .filter(|c| selected.is_none_or(|s| s == *c))
        {
            let text = self.base.listing(Some(c));
            let mut masks = self.masks(c).iter();
            for line in text.lines() {
                if line.starts_with("-P ") {
                    policies.push_str(line);
                    policies.push('\n');
                    continue;
                }
                if line.starts_with("-A ")
                    && let Some(m) = masks
                        .next()
                        .ok_or_else(|| fault(Class::Context, "IPv6 listing mask mismatch."))?
                {
                    let (a, b) = line
                        .rsplit_once(" -j ")
                        .ok_or_else(|| fault(Class::Context, "IPv6 rule target missing."))?;
                    rules.push_str(&format!("{a} -m conntrack --ctstate {} -j {b}\n", m.text()));
                    continue;
                }
                rules.push_str(line);
                rules.push('\n');
            }
        }
        policies.push_str(&rules);
        if policies.len() > 32768 {
            return Err(fault(
                Class::Limit,
                "IPv6 state filter listing exceeds32KiB.",
            ));
        }
        Ok(policies)
    }
    pub fn command(&mut self, line: &str) -> Result<base::Output, Fault> {
        self.validate()?;
        let words = crate::linux_filter::words(line).map_err(|mut e| {
            e.message = e.message.replace("iptables", "ip6tables");
            e
        })?;
        if words.first().map(String::as_str) != Some("ip6tables") {
            return Err(fault(
                Class::Unsupported,
                "Use the documented simulated ip6tables subset; no host command ran.",
            ));
        }
        let (mut plain, mut module, mut matched) = (vec![words[0].clone()], false, None);
        let mut i = 1;
        while i < words.len() {
            let key = words[i].as_str();
            if matches!(key, "-m" | "--match") {
                if module || words.get(i + 1).map(String::as_str) != Some("conntrack") {
                    return Err(fault(
                        Class::Unsupported,
                        "Only one explicit -m conntrack is supported here.",
                    ));
                }
                module = true;
                i += 2;
                continue;
            }
            if key == "--ctstate" {
                if !module || matched.is_some() {
                    return Err(fault(
                        Class::Syntax,
                        "Load -m conntrack before one --ctstate value.",
                    ));
                }
                matched = Some(Match::parse(words.get(i + 1).ok_or_else(|| {
                    fault(Class::Syntax, "Supply a value after --ctstate.")
                })?)?);
                i += 2;
                continue;
            }
            // Consume literal option values before looking for extension options.
            // A quoted interface value '--ctstate' must never become a state flag.
            let count = match key {
                "-P" | "--policy" => 3,
                "-t" | "--table" | "-A" | "--append" | "-I" | "--insert" | "-D" | "--delete"
                | "-s" | "--source" | "-d" | "--destination" | "-i" | "--in-interface" | "-o"
                | "--out-interface" | "-p" | "--protocol" | "--sport" | "--source-port"
                | "--dport" | "--destination-port" | "--icmpv6-type" | "-j" | "--jump" => 2,
                _ => 1,
            };
            let end = (i + count).min(words.len());
            plain.extend_from_slice(&words[i..end]);
            i = end;
        }
        if module != matched.is_some() {
            return Err(fault(
                Class::Syntax,
                "Supply both -m conntrack and --ctstate in this profile.",
            ));
        }
        let encoded = plain.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ");
        let mut args = plain[1..].to_vec();
        let mut i = 0;
        while i < args.len() {
            if matches!(args[i].as_str(), "-t" | "--table") {
                if i + 1 >= args.len() {
                    return Err(fault(Class::Syntax, "Name the filter table."));
                }
                args.drain(i..i + 2);
                continue;
            }
            i += match args[i].as_str() {
                "-P" | "--policy" => 3,
                "-A" | "--append" | "-I" | "--insert" | "-D" | "--delete" | "-s" | "--source"
                | "-d" | "--destination" | "-i" | "--in-interface" | "-o" | "--out-interface"
                | "-p" | "--protocol" | "--sport" | "--source-port" | "--dport"
                | "--destination-port" | "--icmpv6-type" | "-j" | "--jump" => 2,
                _ => 1,
            };
        }
        let op = args.first().map(String::as_str).unwrap_or("");
        if matched.is_some() && !matches!(op, "-A" | "--append" | "-I" | "--insert") {
            return Err(fault(
                Class::Unsupported,
                "Connection-state matches belong to appended or inserted rules.",
            ));
        }
        let mut next = self.clone();
        let mut result = next.base.command(&encoded)?;
        let selected = args.get(1).map(|s| chain(s)).transpose()?;
        match op {
            "-S" | "--list-rules" => result.text = next.listing(selected)?,
            "-F" | "--flush" => {
                for c in CHAINS {
                    if selected.is_none_or(|s| s == c) {
                        next.masks_mut(c).clear();
                    }
                }
            }
            "-A" | "--append" => next
                .masks_mut(selected.ok_or_else(|| fault(Class::Syntax, "Name a chain."))?)
                .push(matched),
            "-I" | "--insert" => {
                let c = selected.ok_or_else(|| fault(Class::Syntax, "Name a chain."))?;
                let index = args
                    .get(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                next.masks_mut(c).insert(index - 1, matched);
            }
            "-D" | "--delete" => {
                let c = selected.ok_or_else(|| fault(Class::Syntax, "Name a chain."))?;
                let index = args
                    .get(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| fault(Class::Syntax, "Delete by one-based rule number."))?;
                next.masks_mut(c).remove(index - 1);
            }
            "-P" | "--policy" => {}
            _ => {
                return Err(fault(
                    Class::Unsupported,
                    "Choose a documented IPv6 filter operation.",
                ));
            }
        }
        next.validate()?;
        next.listing(None)?;
        result.changed = next != *self;
        *self = next;
        Ok(result)
    }
}
