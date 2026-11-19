//! Finite NEW/ESTABLISHED and destination-translation flag matches layered over the accepted numeric filter.
//! No full conntrack, TCP, ICMP association or kernel parity claim.
use crate::{
    linux_connections::Class as State,
    linux_filter::{self as base, Chain, Class, Fault, Verdict},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Match {
    New,
    Established,
    Both,
    Dnat,
    NewDnat,
    EstablishedDnat,
    BothDnat,
}
impl Match {
    fn accepts(self, state: State, dnat: bool) -> bool {
        let (new, established, translated) = match self {
            Self::New => (true, false, false),
            Self::Established => (false, true, false),
            Self::Both => (true, true, false),
            Self::Dnat => (false, false, true),
            Self::NewDnat => (true, false, true),
            Self::EstablishedDnat => (false, true, true),
            Self::BothDnat => (true, true, true),
        };
        (new && state == State::New)
            || (established && state == State::Established)
            || (translated && dnat)
    }
    fn text(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Established => "ESTABLISHED",
            Self::Both => "NEW,ESTABLISHED",
            Self::Dnat => "DNAT",
            Self::NewDnat => "NEW,DNAT",
            Self::EstablishedDnat => "ESTABLISHED,DNAT",
            Self::BothDnat => "NEW,ESTABLISHED,DNAT",
        }
    }
    fn parse(s: &str) -> Result<Self, Fault> {
        let mut bits = 0u8;
        for name in s.split(',') {
            bits |= match name {
                "NEW" => 1,
                "ESTABLISHED" => 2,
                "DNAT" => 4,
                _ => {
                    return Err(fault(
                        Class::Unsupported,
                        "This normal-flow profile supports NEW, ESTABLISHED and the destination-translation flag DNAT; no unsupported state was approximated.",
                    ));
                }
            };
        }
        match bits {
            1 => Ok(Self::New),
            2 => Ok(Self::Established),
            3 => Ok(Self::Both),
            4 => Ok(Self::Dnat),
            5 => Ok(Self::NewDnat),
            6 => Ok(Self::EstablishedDnat),
            7 => Ok(Self::BothDnat),
            _ => Err(fault(
                Class::Syntax,
                "Supply at least one supported connection-state name.",
            )),
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
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
const CHAINS: [Chain; 3] = [Chain::Input, Chain::Forward, Chain::Output];
fn chain(s: &str) -> Result<Chain, Fault> {
    match s {
        "INPUT" => Ok(Chain::Input),
        "FORWARD" => Ok(Chain::Forward),
        "OUTPUT" => Ok(Chain::Output),
        _ => Err(fault(
            Class::Unsupported,
            "Only the built-in INPUT, FORWARD and OUTPUT chains are modeled.",
        )),
    }
}
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
fn rules_mut(t: &mut base::Table, c: Chain) -> &mut base::ChainRules {
    match c {
        Chain::Input => &mut t.input,
        Chain::Forward => &mut t.forward,
        Chain::Output => &mut t.output,
    }
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
            if self.masks(c).len() != self.base.chain(c).rules.len() {
                return Err(fault(
                    Class::Context,
                    "Every state match must belong to one actual filter rule.",
                ));
            }
        }
        Ok(())
    }
    pub fn specialize(&self, state: State) -> Result<base::Table, Fault> {
        self.specialize_context(state, false)
    }
    pub fn specialize_context(&self, state: State, dnat: bool) -> Result<base::Table, Fault> {
        self.validate()?;
        let mut out = self.base.clone();
        for c in CHAINS {
            rules_mut(&mut out, c).rules = self
                .base
                .chain(c)
                .rules
                .iter()
                .zip(self.masks(c))
                .filter(|(_, m)| m.is_none_or(|m| m.accepts(state, dnat)))
                .map(|(r, _)| r.clone())
                .collect();
        }
        Ok(out)
    }
    pub fn evaluate(&self, c: Chain, p: &base::Packet, s: State) -> Result<base::Decision, Fault> {
        self.evaluate_context(c, p, s, false)
    }
    pub fn evaluate_context(
        &self,
        c: Chain,
        p: &base::Packet,
        s: State,
        dnat: bool,
    ) -> Result<base::Decision, Fault> {
        let mut d = self.specialize_context(s, dnat)?.evaluate(c, p)?;
        if let Some(i) = d.rule {
            d.rule = Some(
                self.masks(c)
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.is_none_or(|m| m.accepts(s, dnat)))
                    .nth(i - 1)
                    .ok_or_else(|| fault(Class::Context, "State rule index mismatch"))?
                    .0
                    + 1,
            );
        }
        Ok(d)
    }
    pub fn listing(&self, which: Option<Chain>) -> Result<String, Fault> {
        self.validate()?;
        let mut out = String::new();
        let mut policies = String::new();
        for c in CHAINS.into_iter().filter(|c| which.is_none_or(|v| v == *c)) {
            let text = self.base.listing(Some(c));
            let mut masks = self.masks(c).iter();
            for line in text.lines() {
                if line.starts_with("-P ") {
                    policies.push_str(line);
                    policies.push('\n');
                    continue;
                }
                if line.starts_with("-A ") {
                    let m = masks
                        .next()
                        .ok_or_else(|| fault(Class::Context, "State listing index mismatch"))?;
                    if let Some(m) = m {
                        let (a, b) = line
                            .rsplit_once(" -j ")
                            .ok_or_else(|| fault(Class::Context, "Rule target missing"))?;
                        out.push_str(&format!("{a} -m conntrack --ctstate {} -j {b}\n", m.text()));
                        continue;
                    }
                }
                out.push_str(line);
                out.push('\n');
            }
        }
        policies.push_str(&out);
        let out = policies;
        if out.len() > 32768 {
            return Err(fault(Class::Limit, "State filter listing exceeds32KiB."));
        }
        Ok(out)
    }
    pub fn command(&mut self, line: &str) -> Result<base::Output, Fault> {
        self.validate()?;
        let mut words = base::words(line)?;
        if words.first().map(String::as_str) != Some("iptables") {
            return Err(fault(
                Class::Unsupported,
                "Use the documented simulated iptables subset; no host command ran.",
            ));
        }
        let (mut module, mut state) = (None, None);
        let mut i = 1;
        while i < words.len() {
            if matches!(words[i].as_str(), "-m" | "--match")
                && words.get(i + 1).is_some_and(|s| s == "conntrack")
            {
                if module.replace(i).is_some() {
                    return Err(fault(Class::Syntax, "Specify conntrack only once."));
                }
                i += 2;
            } else if words[i] == "--ctstate" {
                if state.is_some() {
                    return Err(fault(Class::Syntax, "Specify --ctstate only once."));
                }
                let value = words.get(i + 1).ok_or_else(|| {
                    fault(
                        Class::Syntax,
                        "Supply NEW, ESTABLISHED or both after --ctstate.",
                    )
                })?;
                state = Some((i, Match::parse(value)?));
                i += 2;
            } else {
                i += 1;
            }
        }
        let match_state = match (module, state) {
            (None, None) => None,
            (Some(m), Some((s, value))) if m < s => {
                let mut remove = [m, m + 1, s, s + 1];
                remove.sort_unstable();
                for i in remove.into_iter().rev() {
                    words.remove(i);
                }
                Some(value)
            }
            _ => {
                return Err(fault(
                    Class::Unsupported,
                    "Load -m conntrack before --ctstate; supply both parts in this supported form.",
                ));
            }
        };
        // Re-encode every literal word; joining unquoted values could turn one bad argument into options.
        let command = words.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ");
        let mut args = words[1..].to_vec();
        if let Some(i) = args
            .iter()
            .position(|s| matches!(s.as_str(), "-t" | "--table"))
        {
            if i + 1 >= args.len() {
                return Err(fault(Class::Syntax, "Name the table after -t."));
            }
            args.drain(i..i + 2);
        }
        let op = args.first().map(String::as_str).unwrap_or("");
        if match_state.is_some() && !matches!(op, "-A" | "--append" | "-I" | "--insert") {
            return Err(fault(
                Class::Unsupported,
                "State matches belong to appended or inserted rules.",
            ));
        }
        let mut next = self.clone();
        let mut result = next.base.command(&command)?;
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
                .push(match_state),
            "-I" | "--insert" => {
                let c = selected.ok_or_else(|| fault(Class::Syntax, "Name a chain."))?;
                let n = args
                    .get(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                next.masks_mut(c).insert(n - 1, match_state);
            }
            "-D" | "--delete" => {
                let c = selected.ok_or_else(|| fault(Class::Syntax, "Name a chain."))?;
                let n = args
                    .get(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| fault(Class::Syntax, "Delete by one-based rule number."))?;
                next.masks_mut(c).remove(n - 1);
            }
            "-P" | "--policy" => {}
            _ => {
                return Err(fault(
                    Class::Unsupported,
                    "Only documented filter operations are supported.",
                ));
            }
        }
        next.validate()?;
        result.changed = next != *self;
        *self = next;
        Ok(result)
    }
}
