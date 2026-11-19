//! Separate guided practice, saved policy drafts and independent behavioral cases.
use crate::{dhcp_snooping as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
pub const MAX_SESSION: usize = 2 * 1024 * 1024;
pub const PREFIX: &str = "DHCP OBSERVER CONFIG\n";
pub const HELP: &str = "Use the native policy and sample-traffic controls. This is a bounded DHCP observer, not IOS or a host networking command. Relay and Option82 insertion are outside the supplied direct-LAN setup.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] pub String);
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Forwarded {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
        accepted_by: Vec<Name>,
        learned: bool,
        removed: bool,
    },
    Bypassed {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
        accepted_by: Vec<Name>,
    },
    Rejected {
        cause: core::Rejection,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Message {
        host: Name,
        packet: core::Packet,
        expected: Expected,
    },
    Advance {
        seconds: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Step,8>")]
    pub steps: Vec<Step>,
    pub leases_after: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: Name,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    pub initial: core::World,
    pub reference: core::Config,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
/// Derived from the grading trials, with individual observations for optional detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseReport {
    pub label: Option<String>,
    pub failures: Vec<String>,
}
impl CaseReport {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
    pub fn check(&self) -> (bool, String) {
        let text = if self.passed() {
            "every message and lease observation matches".into()
        } else {
            self.failures.join(" ")
        };
        (
            self.passed(),
            match &self.label {
                Some(label) => format!("{label}: {text}"),
                None => text,
            },
        )
    }
    fn error(text: String) -> Self {
        Self {
            label: None,
            failures: vec![text],
        }
    }
}
impl Task {
    pub fn compatible(&self, w: &core::World) -> Result<()> {
        w.validate().map_err(|e| e.to_string())?;
        if w.baseline() != self.initial.baseline() {
            return Err(
                "Saved DHCP work belongs to different supplied equipment; original retained."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn trial(&self, config: &core::Config) -> Result<core::World> {
        let mut w = self.initial.clone();
        w.configure(config.clone()).map_err(|e| e.to_string())?;
        Ok(w)
    }
    pub fn reference_world(&self) -> Result<core::World> {
        self.trial(&self.reference)
    }
    pub fn validate(&self) -> Result<()> {
        self.initial.validate().map_err(|e| e.to_string())?;
        if !key(&self.id.0)
            || !prose(&self.instructions, 1800)
            || !(3..=8).contains(&self.cases.len())
        {
            return Err("A DHCP task needs a stable ID, clear work and three to eight meaningful traffic cases.".into());
        }
        let mut labels = BTreeSet::new();
        let mut delivery = false;
        let mut rejection = false;
        let mut binding = false;
        for c in &self.cases {
            if !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || c.steps.is_empty()
                || c.steps.len() > 8
                || usize::from(c.leases_after) > core::MAX_RECORDS
            {
                return Err(
                    "Use unique case labels, one to eight steps and bounded lease expectations."
                        .into(),
                );
            }
            let mut messages = 0;
            for step in &c.steps {
                match step {
                    Step::Advance { seconds } if *seconds <= 86_400 => {}
                    Step::Message {
                        host,
                        packet,
                        expected,
                    } => {
                        messages += 1;
                        if self.initial.baseline().address(&host.0).is_none()
                            || core::Packet::from_json(
                                &serde_json::to_string(packet)
                                    .map_err(|_| "Cannot encode supplied message.")?,
                            )
                            .is_err()
                        {
                            return Err(
                                "Use supplied endpoints and valid bounded DHCP metadata.".into()
                            );
                        }
                        match expected {
                            Expected::Forwarded { accepted_by, .. }
                            | Expected::Bypassed { accepted_by } => {
                                let recipients: BTreeSet<_> =
                                    accepted_by.iter().map(|n| &n.0).collect();
                                if accepted_by.is_empty()
                                    || accepted_by.len() > 16
                                    || recipients.len() != accepted_by.len()
                                    || recipients.iter().any(|n| {
                                        self.initial.baseline().address(n).is_none()
                                            || **n == host.0
                                    })
                                {
                                    return Err("Successful message cases require real supplied recipients, not an empty delivery shortcut.".into());
                                }
                                delivery = true;
                                if let Expected::Forwarded { learned: true, .. } = expected {
                                    binding = true;
                                }
                            }
                            Expected::Rejected { .. } => rejection = true,
                        }
                    }
                    _ => {
                        return Err(
                            "Advance at most one simulated day per bounded case step.".into()
                        );
                    }
                }
            }
            if messages == 0 {
                return Err("A traffic case must contain an actual message interaction.".into());
            }
        }
        if !delivery
            || !rejection
            || !binding
            || !self.checks(&self.reference_world()?).iter().all(|r| r.0)
            || self.checks(&self.initial).iter().all(|r| r.0)
        {
            return Err("Keep unfinished work, legitimate delivery, rejected traffic and a reachable observed lease; a reference label alone is not success.".into());
        }
        Ok(())
    }
    pub fn checks(&self, w: &core::World) -> Vec<(bool, String)> {
        self.reports(w).iter().map(CaseReport::check).collect()
    }
    pub fn reports(&self, w: &core::World) -> Vec<CaseReport> {
        if let Err(e) = self.compatible(w) {
            return vec![CaseReport::error(e)];
        }
        self.cases.iter().map(|case|{
            let mut trial=match self.trial(w.config()){Ok(w)=>w,Err(e)=>return CaseReport::error(e)};
            let mut failures=vec![];
            for (i,step) in case.steps.iter().enumerate(){match step{
                Step::Advance{seconds}=>if let Err(e)=trial.advance(*seconds){failures.push(format!("Step {}: {e}",i+1));},
                Step::Message{host,packet,expected}=>{
                    let result=trial.transmit(&host.0,packet);
                    let matches=match(expected,&result){
                        (Expected::Forwarded{accepted_by,learned,removed},Ok(core::Outcome::Forwarded{forwarding,learned:actual,removed:was_removed}))=>forwarding.accepted_by==accepted_by.iter().map(|n|n.0.clone()).collect()&&actual.is_some()==*learned&&was_removed==removed&&actual.as_ref().is_none_or(|b|{
                            case.steps[..i].iter().rev().find_map(|s|match s{Step::Message{host,packet:request,..}if request.message==core::Message::Request&&request.client==packet.client&&request.transaction==packet.transaction=>Some(&host.0),_=>None})
                                .and_then(|host|attachment_fact(&trial,host))
                                .is_some_and(|(port,vlan)|lease_matches(packet,b,port,vlan,trial.forwarding().now()))
                        }),
                        (Expected::Bypassed{accepted_by},Ok(core::Outcome::Bypassed{forwarding}))=>forwarding.accepted_by==accepted_by.iter().map(|n|n.0.clone()).collect(),
                        (Expected::Rejected{cause},Ok(core::Outcome::Rejected{cause:actual}))=>actual==cause,
                        _=>false,
                    };
                    if !matches{failures.push(format!("Message {}: expected {}; actual {}",i+1,expectation(expected),match result{Ok(o)=>describe(&o),Err(e)=>format!("{}: {e}",error_label(e.kind))}));}
                }
            }}
            if trial.bindings().len()!=usize::from(case.leases_after){failures.push(format!("Observed leases: expected {}, actual {}. An Offer is not an ACK; check trust, the client exchange and expiry.",case.leases_after,trial.bindings().len()));}
            CaseReport { label: Some(case.label.clone()), failures }
        }).collect()
    }
}
fn attachment_fact<'a>(w: &'a core::World, host: &str) -> Option<(&'a str, u16)> {
    use crate::switching::{Jack, Mode};
    let fabric = w.baseline().fabric();
    let peer = fabric.cables.iter().find_map(|c| {
        if matches!(&c.a,Jack::Host{host:h}if h==host) {
            Some(&c.b)
        } else if matches!(&c.b,Jack::Host{host:h}if h==host) {
            Some(&c.a)
        } else {
            None
        }
    })?;
    let Jack::Switch { switch, port } = peer else {
        return None;
    };
    let Mode::Access { vlan } = &fabric.switches.get(switch)?.ports.get(port)?.mode else {
        return None;
    };
    Some((port.as_str(), *vlan))
}
fn lease_matches(p: &core::Packet, b: &core::Binding, port: &str, vlan: u16, now: u64) -> bool {
    match p.message {
        core::Message::Ack { address, lease } => {
            b.client == p.client
                && b.address == address
                && b.port == port
                && b.vlan == vlan
                && now.checked_add(u64::from(lease)) == Some(b.expires)
        }
        _ => false,
    }
}
fn list(names: impl Iterator<Item = String>) -> String {
    let v = names.collect::<Vec<_>>();
    if v.is_empty() {
        return "no endpoint".into();
    }
    let result = v
        .iter()
        .take(4)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    if v.len() > 4 {
        format!("{result} (+{} others)", v.len() - 4)
    } else {
        result
    }
}
fn rejection(cause: core::Rejection) -> &'static str {
    match cause {
        core::Rejection::UntrustedServer => "a server reply arrived on an untrusted port",
        core::Rejection::SourceMismatch => "Ethernet source and DHCP client MAC addresses disagree",
        core::Rejection::RelayMetadata => "relay data arrived on an untrusted client port",
        core::Rejection::WrongReleasePort => {
            "the Release came from a different port than the recorded client lease"
        }
    }
}
fn expectation(e: &Expected) -> String {
    match e {
        Expected::Forwarded {
            accepted_by,
            learned,
            removed,
        } => format!(
            "inspected delivery to {}{}{}",
            list(accepted_by.iter().map(|n| n.0.clone())),
            if *learned {
                " and a new observed lease"
            } else {
                ""
            },
            if *removed {
                " and removal of the matching lease"
            } else {
                ""
            }
        ),
        Expected::Bypassed { accepted_by } => format!(
            "ordinary forwarding to {}",
            list(accepted_by.iter().map(|n| n.0.clone()))
        ),
        Expected::Rejected { cause } => format!("rejection because {}", rejection(*cause)),
    }
}
pub fn message_label(m: core::Message) -> String {
    match m {
        core::Message::Discover => "Discover — client looks for a server".into(),
        core::Message::Request => "Request — client asks to use an address".into(),
        core::Message::Offer { address } => format!("Offer — server proposes {address}"),
        core::Message::Ack { address, lease } => {
            format!("ACK — server confirms {address} for {lease} seconds")
        }
        core::Message::Release { address } => format!("Release — client gives up {address}"),
        core::Message::Nak => "NAK — server refuses the request".into(),
        core::Message::Decline => "Decline".into(),
        core::Message::Inform => "Inform".into(),
        core::Message::LeaseQuery => "Lease query".into(),
    }
}
pub fn error_label(kind: core::ErrorKind) -> &'static str {
    match kind {
        core::ErrorKind::Invalid => "Input needs attention",
        core::ErrorKind::Unsupported => "Not supported",
        core::ErrorKind::Limit => "Simulator limit",
    }
}
pub fn describe(o: &core::Outcome) -> String {
    match o {
        core::Outcome::Unavailable { reason } => format!("unavailable source: {reason}"),
        core::Outcome::Rejected { cause } => format!("rejected: {}", rejection(*cause)),
        core::Outcome::Bypassed { forwarding } => format!(
            "not inspected (global/VLAN snooping inactive); received by {}",
            list(forwarding.accepted_by.iter().cloned())
        ),
        core::Outcome::Forwarded {
            forwarding,
            learned,
            removed,
        } => format!(
            "inspected; received by {}{}{}",
            list(forwarding.accepted_by.iter().cloned()),
            if let Some(b) = learned {
                format!(
                    "; observed {} to {} on VLAN {}, {}",
                    b.address,
                    b.client.text(),
                    b.vlan,
                    b.port
                )
            } else {
                String::new()
            },
            if *removed {
                "; matching lease removed"
            } else {
                ""
            }
        ),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        let a = self.initial.initial.baseline().fabric();
        let b = self.example.initial.baseline().fabric();
        if self.initial.id == self.example.id
            || a.switches.keys().any(|k| b.switches.contains_key(k))
            || a.hosts.keys().any(|k| b.hosts.contains_key(k))
        {
            return Err(
                "Worked practice needs separate task, equipment and endpoint identities.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("Saved DHCP work belongs to another supplied task.".into());
        }
        self.initial.compatible(&s.assignment.world)?;
        self.example.compatible(&s.practice.world)?;
        if s.assignment.sample >= self.initial.cases.len()
            || s.practice.sample >= self.example.cases.len()
        {
            return Err("Choose a supplied sample traffic sequence.".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked || !s.assignment.applied() {
            return vec![(
                false,
                "Not checked · Apply current policy, then Check work.".into(),
            )];
        }
        self.initial.checks(&s.assignment.world)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("DHCP task data exceeds2MiB.".into());
        }
        let d: Self =
            serde_json::from_str(raw).map_err(|_| "Invalid or unsupported DHCP task data.")?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub enabled: bool,
    pub verify_source: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub vlans: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::PortTrust,16>")]
    pub ports: Vec<core::PortTrust>,
}
impl Draft {
    pub fn new(c: &core::Config) -> Self {
        Self {
            enabled: c.enabled,
            verify_source: c.verify_source,
            vlans: c
                .vlans
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            ports: c.ports.clone(),
        }
    }
    pub fn config(&self) -> Result<core::Config> {
        if self.vlans.len() > 128 || self.vlans.chars().any(char::is_control) {
            return Err("VLAN input exceeds its safe size or contains a control character.".into());
        }
        let vlans = if self.vlans.trim().is_empty() {
            vec![]
        } else {
            self.vlans
                .split(',')
                .map(|v| {
                    v.trim()
                        .parse::<u16>()
                        .map_err(|_| "VLANs: enter whole VLAN IDs separated by commas.".to_string())
                })
                .collect::<Result<Vec<_>>>()?
        };
        if vlans.len() > 16
            || vlans.iter().any(|v| !(1..=4094).contains(v))
            || vlans.iter().collect::<BTreeSet<_>>().len() != vlans.len()
        {
            return Err("VLANs: use at most16 unique IDs from1 to4094.".into());
        }
        Ok(core::Config {
            enabled: self.enabled,
            verify_source: self.verify_source,
            vlans,
            ports: self.ports.clone(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub text: String,
    pub error: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::World,
    pub draft: Draft,
    pub sample: usize,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,16>")]
    pub output: Vec<Line>,
    pub checked: bool,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            world: t.initial.clone(),
            draft: Draft::new(t.initial.config()),
            sample: 0,
            output: vec![],
            checked: false,
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.world.validate().map_err(|e| e.to_string())?;
        let expected: BTreeSet<_> = self.world.config().ports.iter().map(|p| &p.port).collect();
        if self.sample >= 8
            || self.draft.vlans.len() > 128
            || self.draft.vlans.chars().any(char::is_control)
            || self.draft.ports.len() != expected.len()
            || self
                .draft
                .ports
                .iter()
                .map(|p| &p.port)
                .collect::<BTreeSet<_>>()
                != expected
            || self.output.len() > 16
            || self.output.iter().any(|l| l.text.len() > 2048)
        {
            return Err(
                "DHCP policy drafts/sample results exceed supplied identities or safe bounds."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn applied(&self) -> bool {
        self.draft.config().is_ok_and(|mut c| {
            c.vlans.sort_unstable();
            c.ports.sort_by(|a, b| a.port.cmp(&b.port));
            let mut now = self.world.config().clone();
            now.vlans.sort_unstable();
            now.ports.sort_by(|a, b| a.port.cmp(&b.port));
            c == now
        })
    }
    pub fn apply(&mut self) -> Result<()> {
        self.validate()?;
        self.checked = false;
        self.world
            .configure(self.draft.config()?)
            .map_err(|e| e.to_string())?;
        self.output.clear();
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>> {
        self.validate()?;
        t.compatible(&self.world)?;
        self.checked = self.applied();
        if !self.checked {
            return Err(
                "Apply the current policy before checking; your draft remains editable.".into(),
            );
        }
        Ok(t.checks(&self.world))
    }
    pub fn fill(&mut self, t: &Task) -> Result<()> {
        t.compatible(&self.world)?;
        self.world = t.reference_world()?;
        self.draft = Draft::new(self.world.config());
        self.output.clear();
        self.checked = false;
        Ok(())
    }
    pub fn run_sample(&mut self, t: &Task) -> Result<()> {
        self.validate()?;
        t.compatible(&self.world)?;
        self.checked = false;
        if !self.applied() {
            return Err("Apply your current policy before replaying sample traffic.".into());
        }
        let c = t
            .cases
            .get(self.sample)
            .ok_or("Choose a supplied sample sequence.")?;
        let mut trial = t.trial(self.world.config())?;
        let mut output = vec![];
        for (i, step) in c.steps.iter().enumerate() {
            let result = match step {
                Step::Message { host, packet, .. } => trial.transmit(&host.0, packet).map(|o| {
                    format!(
                        "{} · {} · {} to client {}: {}",
                        host.0,
                        message_label(packet.message),
                        packet.source.text(),
                        packet.client.text(),
                        describe(&o)
                    )
                }),
                Step::Advance { seconds } => trial.advance(*seconds).map(|()| {
                    format!(
                        "Advanced {seconds} simulated seconds; {} observed leases remain.",
                        trial.bindings().len()
                    )
                }),
            };
            let failed = result.is_err();
            let text = match result {
                Ok(s) => s,
                Err(e) => format!("{}: {e}", error_label(e.kind)),
            };
            let text = format!("{}. {text}", i + 1);
            if text.len() > 2048 {
                return Err(
                    "Sample feedback exceeded its bounded size; original work retained.".into(),
                );
            }
            output.push(Line {
                text,
                error: failed,
            });
            if failed {
                break;
            }
        }
        self.world = trial;
        self.output = output;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    pub origin: Name,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || !key(&self.origin.0) {
            return Err("Unsupported DHCP progress version or task identity.".into());
        }
        self.assignment.validate()?;
        self.practice.validate()
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("DHCP progress exceeds2MiB.".into());
        }
        let s: Self =
            serde_json::from_str(raw).map_err(|_| "Invalid or unsupported DHCP progress.")?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        d.bound(self)?;
        let text = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if text.len() > 8192 {
            return Err("DHCP policy data exceeds8KiB.".into());
        }
        let c: core::Config = serde_json::from_str(text)
            .map_err(|_| "Invalid supplied DHCP policy data; use native controls.")?;
        let mut next = self.assignment.clone();
        next.draft = Draft::new(&c);
        next.apply()?;
        let checks = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "DHCP policy checked: {}/{} traffic cases match",
            checks.iter().filter(|r| r.0).count(),
            checks.len()
        ))
    }
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::DhcpSnooping { simulation },
        crate::engine::Goal::DhcpSnooping { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("DHCP observer source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id.0 != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err(
            "DHCP observer source identity, task and supplied equipment must match.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::DhcpSnooping { simulation } = &a.state else {
        return Err("DHCP observer work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible DHCP observer attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_lease_grade_requires_every_ack_field_not_just_a_present_record() {
        use crate::mac_learning::Mac;
        use std::net::Ipv4Addr;
        let packet = core::Packet {
            source: Mac([2, 0, 0, 0, 0, 2]),
            destination: Mac::BROADCAST,
            transaction: 7,
            client: Mac([2, 0, 0, 0, 0, 1]),
            relay: Ipv4Addr::UNSPECIFIED,
            option82: false,
            message: core::Message::Ack {
                address: Ipv4Addr::new(172, 19, 2, 81),
                lease: 60,
            },
        };
        let valid = core::Binding {
            client: packet.client,
            address: Ipv4Addr::new(172, 19, 2, 81),
            port: "client".into(),
            vlan: 10,
            expires: 80,
        };
        assert!(lease_matches(&packet, &valid, "client", 10, 20));
        for field in ["mac", "ip", "port", "vlan", "expiry"] {
            let mut wrong = valid.clone();
            match field {
                "mac" => wrong.client = Mac([2, 0, 0, 0, 0, 9]),
                "ip" => wrong.address = Ipv4Addr::new(172, 19, 2, 82),
                "port" => wrong.port = "peer".into(),
                "vlan" => wrong.vlan = 20,
                _ => wrong.expires = 81,
            }
            assert!(!lease_matches(&packet, &wrong, "client", 10, 20), "{field}");
        }
        assert!(!lease_matches(&packet, &valid, "client", 10, u64::MAX));
    }
}
