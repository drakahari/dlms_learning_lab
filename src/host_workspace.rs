//! Source-bound host-networking practice. Commands/configuration remain virtual.
use crate::{
    ipv4_routing::Prefix, linux_host as core, pack::Scenario, progress::Attempt,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Address { device: String, network: Prefix },
    Up { device: String },
    Default { device: String, via: Ipv4Addr },
    Reach { target: Ipv4Addr, device: String },
    Resolve { name: String, address: Ipv4Addr },
    PreserveInterface { device: String },
    PreserveRoutes { destination: Prefix },
    Ipv6Address { device: String, network: crate::ipv6::Prefix },
    Ipv6Default { device: String, via: std::net::Ipv6Addr },
    Ipv6Reach { target: std::net::Ipv6Addr, device: String },
    PreserveIpv4 {},
    PreserveIpv6Interface { device: String },
    PreserveIpv6Routes { destination: crate::ipv6::Prefix },
}
impl Requirement {
    pub(crate) fn check(&self, initial: &core::State, world: &core::State) -> (bool, String) {
        match self {
            Self::Ipv6Address { device, network } => {
                let address = world.configuration.ipv6.as_ref().and_then(|c| c.interfaces.iter().find(|i| i.device == *device)).and_then(|i| i.address);
                let facts = world.profile.ipv6.as_ref();
                let pass = address.is_some_and(|a| a.valid() && a.network() == Some(*network)
                    && facts.is_some_and(|f| !f.nodes.iter().any(|n|n.address==a.address) && !f.gateways.iter().any(|g|g.address==a.address)));
                (pass, format!("{device}: choose an unused IPv6 primary address in {} with that prefix. Current: {}. IPv6 has no broadcast; this activity excludes subnet-router anycast and supplied peer/router addresses.",network.text(),address.map(|a|format!("{}/{}",a.address,a.prefix)).unwrap_or_else(||"no IPv6 primary address".into())))
            }
            Self::Ipv6Default { device, via } => {
                let selected = world.configuration.ipv6.as_ref().and_then(|c|c.routes.iter().filter(|r|r.destination.length==0).min_by_key(|r|r.metric));
                (selected.is_some_and(|r|r.device==*device&&r.via==*via),format!("The lowest-metric IPv6 default must use {via} on {device}. A link-local next hop is identified by both address and device; IPv4 defaults do not route IPv6."))
            }
            Self::Ipv6Reach { target, device } => {
                let p=world.probe_ipv6(*target);
                (p.pass&&p.selection.as_ref().is_some_and(|s|s.device==*device),format!("Virtual IPv6 path to {target} via {device}: {}{}",p.reason,p.selection.map(|s|format!(" · selected {} on {}",s.destination.text(),s.device)).unwrap_or_default()))
            }
            Self::PreserveIpv4 {} => {
                let mut old=initial.configuration.clone();let mut now=world.configuration.clone();old.ipv6=None;now.ipv6=None;
                (old==now,"Keep the complete supplied IPv4 configuration, shared interface Up states, IPv4 routes and IPv4 nameservers unchanged while repairing IPv6.".into())
            }
            Self::PreserveIpv6Interface { device } => {
                let address=|w:&core::State|w.configuration.ipv6.as_ref().and_then(|c|c.interfaces.iter().find(|i|i.device==*device)).cloned();
                let old=address(initial);(old.is_some()&&old==address(world),format!("Keep {device}'s complete supplied IPv6 primary-address configuration unchanged; repair the separate application path."))
            }
            Self::PreserveIpv6Routes { destination } => {
                let rows=|w:&core::State|{let mut rows=w.configuration.ipv6.as_ref().map(|c|c.routes.iter().filter(|r|r.destination==*destination).map(|r|format!("{}|{}|{}",r.device,r.via,r.metric)).collect::<Vec<_>>()).unwrap_or_default();rows.sort();rows};
                (rows(initial)==rows(world),format!("Keep all supplied IPv6 gateway-route entries for {} unchanged, including device scope and metric.",destination.text()))
            }
            Self::Address { device, network } => {
                let a = world
                    .configuration
                    .interfaces
                    .iter()
                    .find(|i| i.name == *device)
                    .and_then(|i| i.address.as_ref());
                let pass = a.is_some_and(|a| {
                    a.usable()
                        && Prefix::new(a.address, a.prefix) == Some(*network)
                        && !world.profile.nodes.iter().any(|n| n.address == a.address)
                        && !world
                            .profile
                            .gateways
                            .iter()
                            .any(|g| g.address == a.address)
                });
                (
                    pass,
                    format!(
                        "{device}: use one unused conventional host address in {} with the correct prefix. Current: {}. Network/broadcast and supplied peer/gateway addresses are not valid workstation choices in this activity.",
                        network.text(),
                        a.map(|a| format!("{}/{}", a.address, a.prefix))
                            .unwrap_or_else(|| "no IPv4 address".into())
                    ),
                )
            }
            Self::Up { device } => (
                world
                    .configuration
                    .interfaces
                    .iter()
                    .any(|i| i.name == *device && i.up),
                format!(
                    "{device} must be administratively up; changing the address alone does not bring a link up."
                ),
            ),
            Self::Default { device, via } => {
                let selected = world
                    .configuration
                    .routes
                    .iter()
                    .filter(|r| r.destination.length == 0)
                    .min_by_key(|r| r.metric);
                (
                    selected.is_some_and(|r| r.device == *device && r.via == *via),
                    format!(
                        "The lowest-metric configured default route must use {via} on {device}. Lower metrics win only between routes of the same prefix length."
                    ),
                )
            }
            Self::Reach { target, device } => {
                let p = world.probe(*target);
                (
                    p.pass && p.selection.as_ref().is_some_and(|s| s.device == *device),
                    format!(
                        "Virtual path to {target} via {device}: {}{}",
                        p.reason,
                        p.selection
                            .as_ref()
                            .map(|s| format!(
                                " · selected {} on {}",
                                s.destination.text(),
                                s.device
                            ))
                            .unwrap_or_default()
                    ),
                )
            }
            Self::Resolve { name, address } => {
                let got = world.resolve(name);
                (
                    got.as_ref() == Ok(address),
                    format!(
                        "{name} must resolve to {address}. Actual: {}. Test the IP path separately from the resolver’s supplied records.",
                        got.map(|ip| ip.to_string()).unwrap_or_else(|e| e)
                    ),
                )
            }
            Self::PreserveInterface { device } => {
                let before = initial
                    .configuration
                    .interfaces
                    .iter()
                    .find(|i| i.name == *device);
                let now = world
                    .configuration
                    .interfaces
                    .iter()
                    .find(|i| i.name == *device);
                (
                    before.is_some() && before == now,
                    format!(
                        "Retain the complete supplied {device} management interface state; repair the other path without disturbing it."
                    ),
                )
            }
            Self::PreserveRoutes { destination } => {
                let list = |w: &core::State| {
                    let mut x: Vec<_> = w
                        .configuration
                        .routes
                        .iter()
                        .filter(|r| r.destination == *destination)
                        .map(|r| format!("{}|{}|{}", r.device, r.via, r.metric))
                        .collect();
                    x.sort();
                    x
                };
                (
                    list(initial) == list(world),
                    format!(
                        "Retain all supplied gateway-route entries for {}; the management route is separate from the application default.",
                        destination.text()
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    Resolver {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
}
fn apply(w: &mut core::State, e: &Event) -> Result<String, String> {
    match e {
        Event::Command { input } => w.execute(input),
        Event::Resolver { text } => {
            w.save_resolver(text)?;
            Ok("Saved the virtual resolver configuration".into())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    /// Source-owned direct-query observations, absent from legacy definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_dns: Option<crate::host_direct_dns::Lesson>,
    /// Explicit source-owned Echo capability; absent old lessons serialize exactly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo: Option<crate::host_echo::Lesson>,
    pub initial: core::State,
    pub example: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub example_requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,24>")]
    pub example_actions: Vec<Event>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if let Some(dns) = &self.direct_dns {
            dns.initial.validate(&self.initial)?;
            dns.example.validate(&self.example)?;
        }
        if let Some(echo) = &self.echo {
            echo.initial.validate(&self.initial)?;
            echo.example.validate(&self.example)?;
        }
        if self.initial.profile.ipv6.is_some() != self.example.profile.ipv6.is_some() {
            return Err("Assignment and separate example must use the same source-enabled IP-family capabilities".into());
        }
        if self.initial.profile == self.example.profile
            || self.requirements.is_empty()
            || self.example_requirements.is_empty()
            || self.example_actions.is_empty()
            || self.requirements.len() > 16
            || self.example_requirements.len() > 16
            || self.example_actions.len() > 24
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 192 * 1024
        {
            return Err("Host lesson needs distinct sample facts and bounded outcomes".into());
        }
        for (w, rs) in [
            (&self.initial, &self.requirements),
            (&self.example, &self.example_requirements),
        ] {
            if w.generation != 0 {
                return Err("Supplied host worlds must start at generation zero".into());
            }
            for r in rs {
                match r {
                    Requirement::Ipv6Address { device, network } => {
                        if !network.valid() || !w.profile.ipv6.as_ref().is_some_and(|f|f.lans.iter().any(|l|l.device==*device&&l.network==*network)) {return Err("IPv6 address requirement must name a source-enabled supplied LAN".into());}
                    }
                    Requirement::Ipv6Default { device, via } => {
                        if !w.profile.ipv6.as_ref().is_some_and(|f|f.gateways.iter().any(|g|g.device==*device&&g.address==*via)){return Err("IPv6 default requirement must name a supplied scoped gateway".into());}
                    }
                    Requirement::Ipv6Reach { target, device } => {
                        if !w.profile.ipv6.as_ref().is_some_and(|f|f.lans.iter().any(|l|l.device==*device)&&f.nodes.iter().any(|n|n.address==*target)){return Err("IPv6 path requirement must name a supplied destination and interface".into());}
                    }
                    Requirement::PreserveIpv4 {} => {if w.profile.ipv6.is_none(){return Err("IPv4 preservation in this unit requires an IPv6-enabled supplied context".into());}}
                    Requirement::PreserveIpv6Interface { device } => {
                        if !w.configuration.ipv6.as_ref().is_some_and(|c|c.interfaces.iter().any(|i|i.device==*device&&i.address.is_some())){return Err("Protected IPv6 interface must have an initial primary address".into());}
                    }
                    Requirement::PreserveIpv6Routes { destination } => {
                        if !destination.valid()||!w.configuration.ipv6.as_ref().is_some_and(|c|c.routes.iter().any(|r|r.destination==*destination)){return Err("Protected IPv6 route must name an initial route".into());}
                    }
                    Requirement::Address { device, network } => {
                        if !network.valid()
                            || !w
                                .profile
                                .links
                                .iter()
                                .any(|l| l.name == *device && l.network == *network)
                        {
                            return Err("Host address requirement must name a supplied LAN".into());
                        }
                    }
                    Requirement::Up { device }
                    | Requirement::PreserveInterface { device }
                    | Requirement::Reach { device, .. } => {
                        if !w.profile.links.iter().any(|l| l.name == *device) {
                            return Err("Unknown required host interface".into());
                        }
                    }
                    Requirement::Default { device, via } => {
                        if !w
                            .profile
                            .gateways
                            .iter()
                            .any(|g| g.device == *device && g.address == *via)
                        {
                            return Err("Unknown required default gateway".into());
                        }
                    }
                    Requirement::Resolve { name, address } => {
                        if name.len() > 80
                            || !w.profile.resolvers.iter().any(|s| {
                                s.records.iter().any(|r| {
                                    r.name.eq_ignore_ascii_case(name) && r.address == *address
                                })
                            })
                        {
                            return Err("Unknown required supplied DNS answer".into());
                        }
                    }
                    Requirement::PreserveRoutes { destination } => {
                        if !destination.valid()
                            || !w
                                .configuration
                                .routes
                                .iter()
                                .any(|r| r.destination == *destination)
                        {
                            return Err("Protected route must name an initial route".into());
                        }
                    }
                }
            }
        }
        let mut s = Session::new(self);
        for e in &self.example_actions {
            s.run(self, e, true)?;
        }
        if !self.checks(&s, true).iter().all(|r| r.0) {
            return Err("Worked host example does not reach its outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.example_action >= self.example_actions.len() {
            return Err("Unknown worked host action".into());
        }
        for (scope, initial) in [(&s.assignment, &self.initial), (&s.practice, &self.example)] {
            let mut replay = initial.clone();
            let mut applied = initial.resolver_text();
            for e in &scope.history {
                apply(&mut replay, e)?;
                if let Event::Resolver { text } = e {
                    applied = text.clone();
                }
            }
            if replay != scope.world || scope.resolver_applied != applied {
                return Err("Host workspace differs from its supplied facts and actual successful edits; prior work retained".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (scope, initial, rs) = if practice {
            (&s.practice, &self.example, &self.example_requirements)
        } else {
            (&s.assignment, &self.initial, &self.requirements)
        };
        let clean = scope.resolver_draft == scope.resolver_applied;
        let mut rows=vec![(clean,if clean{"No unapplied virtual resolver changes."}else{"Save or discard the edited virtual /etc/resolv.conf; unsaved text is not applied network state."}.into())];
        rows.extend(rs.iter().map(|r| r.check(initial, &scope.world)));
        if let Some(echo) = &self.echo {
            let exercise = if practice { &echo.example } else { &echo.initial };
            rows.extend(exercise.checks(&scope.world));
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,64>")]
    pub history: Vec<Event>,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,24>")]
    pub transcript: Vec<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub resolver_draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub resolver_applied: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub probe_input: String,
    #[serde(default, skip_serializing_if = "String::is_empty", deserialize_with = "crate::cron::text::<_,64>")]
    pub probe_ipv6_input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub probe_result: String,
    pub checked: bool,
}
impl Scope {
    pub fn new(w: &core::State) -> Self {
        let text = w.resolver_text();
        Self {
            world: w.clone(),
            history: vec![],
            command: String::new(),
            transcript: vec![],
            resolver_draft: text.clone(),
            resolver_applied: text,
            probe_input: String::new(),
            probe_ipv6_input: String::new(),
            probe_result: String::new(),
            checked: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub example_action: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_action: 0,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.example_action >= 24
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 512 * 1024
        {
            return Err("Host workspace version/byte limit exceeded".into());
        }
        for s in [&self.assignment, &self.practice] {
            s.world.validate()?;
            if s.history.len() > 64
                || s.command.len() > 512
                || s.resolver_draft.len() > 512
                || s.resolver_applied.len() > 512
                || s.transcript.len() > 24
                || s.transcript.iter().any(|s| s.len() > 4096)
                || s.probe_input.len() > 32
                || s.probe_ipv6_input.len() > 64
                || (s.world.profile.ipv6.is_none() && !s.probe_ipv6_input.is_empty())
                || s.probe_result.len() > 1024
            {
                return Err("Host workspace input/history limit exceeded".into());
            }
        }
        Ok(())
    }
    pub fn run(&mut self, d: &Definition, e: &Event, practice: bool) -> Result<String, String> {
        d.bound(self)?;
        let mut next = self.clone();
        let s = if practice {
            &mut next.practice
        } else {
            &mut next.assignment
        };
        let before = s.world.clone();
        let direct = match (e, &d.direct_dns) {
            (Event::Command { input }, Some(dns)) if matches!(input.split_whitespace().next(), Some("dig" | "host")) => {
                let plan = if practice { &dns.example } else { &dns.initial };
                Some(plan.observe(&s.world, input)?.output())
            }
            _ => None,
        };
        let out = if let Some(output) = direct { output } else { match (e, &d.echo) {
            (Event::Command { input }, Some(echo)) if input.split_whitespace().next() == Some("ping") => {
                let target = crate::host_echo::Plan::command_target(input)?;
                let exercise = if practice { &echo.example } else { &echo.initial };
                exercise.plan.observe(&s.world, target)?.output()
            }
            (Event::Command { input }, _) if input == "help" && (d.echo.is_some() || d.direct_dns.is_some()) => {
                let mut help = s.world.command_help().to_owned();
                if d.echo.is_some() { help.push_str(&format!("\n{}", crate::host_echo::HELP)); }
                if d.direct_dns.is_some() { help.push_str(&format!("\n{}", crate::host_direct_dns::HELP)); }
                help
            }
            _ => apply(&mut s.world, e)?,
        }};
        let changed = s.world != before || matches!(e, Event::Resolver { .. });
        if changed {
            if s.history.len() == 64 {
                return Err("This attempt reached64 saved network edits; reset this example or assignment to start again".into());
            }
            s.history.push(e.clone());
        }
        if let Event::Resolver { text } = e {
            s.resolver_draft = text.clone();
            s.resolver_applied = text.clone();
        }
        // Opt-in Echo observations are read-only. Keep current verification;
        // actual edits still invalidate it. Existing lessons retain their behavior.
        if (d.echo.is_none() && d.direct_dns.is_none()) || changed {
            s.checked = false;
            s.probe_result.clear();
        }
        d.bound(&next)?;
        *self = next;
        Ok(out)
    }
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String, String> {
        let raw = text
            .strip_prefix("host-action ")
            .ok_or("Use this activity’s simulated terminal and virtual resolver editor")?;
        if raw.len() > 4096 {
            return Err("Host reference exceeds its limit".into());
        }
        let e: Event =
            serde_json::from_str(raw).map_err(|_| "Invalid simulator-only host action")?;
        self.run(d, &e, false)
    }
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::HostNetwork { simulation },
        crate::engine::Goal::HostNetwork { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Host task and supplied state must agree".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Host lesson must start with its declared separate worlds".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::HostNetwork { simulation } = &a.state else {
        return Err("Saved host work belongs to another task".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Foreign host workspace metadata; previous work retained".into());
    }
    d.bound(simulation)
}

pub const HELP: &str = "Supported literal commands: ip link show; ip link set dev NAME up|down; ip -4 address show [dev NAME]; ip -4 address add|del ADDRESS/PREFIX dev NAME; ip -4 address flush dev NAME; ip -4 route show; ip -4 route add|replace|del PREFIX via GATEWAY dev NAME [metric N]; getent ahostsv4 NAME. One primary IPv4 address per supplied interface. Delete it before adding a replacement; remove gateway routes before deleting that interface address. No shell operators, sudo, arbitrary commands, real files or packets. Test address is a simulator-only path check, not ping or ip route get. The virtual resolver editor supports up to two IPv4 nameserver lines and # comments only; no search/options/NSS or real DNS. All output derives from current simulated state.";
