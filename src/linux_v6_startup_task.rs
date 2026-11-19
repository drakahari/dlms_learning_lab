//! Outcome checks for the supplied IPv6 loader; no catalog/source binding yet.
use crate::{
    linux_v6_filter::{Chain, Protocol},
    linux_v6_flow as flow, linux_v6_path as path, linux_v6_startup as core,
    linux_v6_workspace::{self as traffic, Check},
};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
fn check(passed: bool, label: impl Into<String>, detail: impl Into<String>) -> Check {
    Check {
        passed,
        label: label.into(),
        detail: detail.into(),
    }
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: core::State,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub supplied_settings: String,
    pub traffic: traffic::Task,
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate().map_err(|e| e.message)?;
        self.traffic.validate()?;
        if !id(&self.id)
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 512 * 1024
            || self
                .traffic
                .cases
                .iter()
                .map(|c| c.events.len())
                .sum::<usize>()
                > 24
        {
            return Err(
                "Use one stable startup task within 512 KiB and 24 supplied traffic events.".into(),
            );
        }
        let facts = self.initial.router.facts()?;
        if facts != self.traffic.facts
            || self.traffic.initial.filter != self.initial.filter
            || !self.traffic.forwarding
            || !self.initial.router.settings.global_forwarding
            || !self
                .traffic
                .cases
                .iter()
                .flat_map(|c| &c.events)
                .any(|e| e.expected == traffic::Expected::Filtered)
        {
            return Err("Match the supplied static router, initial running table and forwarded policy; require allowed and filtered traffic.".into());
        }
        for link in &facts.links {
            if !self
                .initial
                .router
                .settings
                .forwards_from(&link.name)
                .map_err(|e| e.message)?
            {
                return Err("Supply persistent forwarding on both fixed links.".into());
            }
        }
        let fresh = self
            .initial
            .startup_trial(&self.supplied_settings)
            .map_err(|e| e.message)?;
        if fresh.router != self.initial.router {
            return Err("The declared startup network/settings must reproduce the supplied persistent router; RA/timing and mutable host configuration are separate activities.".into());
        }
        // An allowed transit must have a real supplied neighbor/onward/host fact.
        // Router/physical facts remain immutable throughout this activity. They cannot
        // be changed to make a missing path look like a successful filter refusal.
        for c in &self.traffic.cases {
            for e in &c.events {
                if let traffic::PacketInput::Literal { header } = &e.packet
                    && e.expected == traffic::Expected::Allowed
                {
                    let trace = path::Runtime {
                        filter: crate::linux_v6_filter::Table::new(
                            crate::linux_v6_filter::Verdict::Accept,
                        ),
                        forwarding: true,
                    }
                    .evaluate(&facts, &path::Origin::Incoming(e.ingress.clone()), header)
                    .map_err(|e| e.message)?;
                    if trace.gate != path::Gate::Allowed {
                        return Err(
                            "Allowed cases must have a working supplied route before filtering."
                                .into(),
                        );
                    }
                    if trace.chain == Some(Chain::Forward) {
                        let p = self.initial.router.supplied_path(header.destination)?;
                        if !p.pass {
                            return Err(format!(
                                "Allowed target {} has no supplied physical path: {}",
                                header.destination, p.reason
                            ));
                        }
                    }
                    if matches!(header.protocol, Protocol::Tcp | Protocol::Udp) {
                        let p = self.initial.router.supplied_path(header.source)?;
                        if !p.pass {
                            return Err(format!(
                                "The request source needs a supplied return path: {}",
                                p.reason
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &core::State) -> Result<()> {
        s.validate().map_err(|e| e.message)?;
        if s.router != self.initial.router {
            return Err("Keep this activity's supplied addresses, manual routes and settings unchanged; only its filter, file and provided loader are editable.".into());
        }
        Ok(())
    }
    pub fn trial(&self, s: &core::State, index: usize) -> Result<traffic::Trial> {
        self.bound(s)?;
        let c = self
            .traffic
            .cases
            .get(index)
            .ok_or("Choose one supplied traffic case.")?;
        self.traffic.trial(
            &flow::State {
                filter: s.filter.clone(),
                connections: self.traffic.initial.connections.clone(),
            },
            c,
        )
    }
    fn live_checks(&self, s: &core::State, prefix: &str) -> Vec<Check> {
        if let Err(e) = self.bound(s) {
            return vec![check(false, format!("{prefix}Supplied router"), e)];
        }
        let t = &s.filter;
        let initial = &self.initial.filter;
        let mut out = vec![
            check(
                t.base.input == initial.base.input && t.input == initial.input,
                format!("{prefix}Preserve INPUT"),
                "Keep the complete supplied INPUT policy and rules, including the selected control-header permit.",
            ),
            check(
                t.base.output == initial.base.output && t.output == initial.output,
                format!("{prefix}Preserve OUTPUT"),
                "Keep the complete supplied OUTPUT policy and rules.",
            ),
        ];
        let names = self
            .traffic
            .facts
            .links
            .iter()
            .map(|l| l.name.clone())
            .collect::<Vec<_>>();
        for policy in &self.traffic.policies {
            let context = policy.class.map_or("ICMPv6 headers", |c| match c {
                crate::linux_v6_connections::Class::New => "NEW TCP/UDP",
                crate::linux_v6_connections::Class::Established => "ESTABLISHED TCP/UDP",
            });
            let label = format!("{prefix}FORWARD · {context}");
            out.push(match t.compare(Chain::Forward,policy.class,&names,&policy.domains,&policy.allowed){
    Ok(None)=>check(true,label,"The running table matches the complete stated address/port/header policy in this direction-state context."),
    Ok(Some(m))=>{
        let port=|v:Option<u16>|v.map_or("not supplied".into(),|n|n.to_string());
        let header=match m.packet.protocol {
            Protocol::Tcp|Protocol::Udp=>format!("{}; source port {}, destination port {}",if m.packet.protocol==Protocol::Tcp{"TCP"}else{"UDP"},port(m.packet.source_port),port(m.packet.destination_port)),
            Protocol::Icmp6=>format!("ICMPv6 type {}, code {}",m.packet.icmp_type.map_or("not supplied".into(),|n|n.to_string()),m.packet.icmp_code.map_or("not supplied".into(),|n|n.to_string())),
        };
        let verdict=|v|if v==crate::linux_v6_filter::Verdict::Accept{"permit"}else{"drop"};
        let first=m.rule.map_or("the chain default".into(),|n|format!("running rule {n}"));
        check(false,label,format!("For the modeled header {} → {} ({header}): expected {}, actual {}. First match: {first}. Inspect prefixes, ports, direction state and chain policy; matching only the displayed servers is insufficient.",m.packet.source,m.packet.destination,verdict(m.expected),verdict(m.actual)))
    },
    Err(e)=>check(false,label,format!("[{:?}] {} No successful policy comparison accepted.",e.class,e.message)),
   });
        }
        for c in &self.traffic.cases {
            match self.traffic.trial(
                &flow::State {
                    filter: t.clone(),
                    connections: self.traffic.initial.connections.clone(),
                },
                c,
            ) {
                Ok(trial) => out.extend(trial.steps.into_iter().map(|s| {
                    check(
                        s.check.passed,
                        format!("{prefix}{}", s.check.label),
                        s.check.detail,
                    )
                })),
                Err(e) => out.push(check(false, format!("{prefix}{}", c.label), e)),
            }
        }
        out
    }
    pub fn startup(&self, s: &core::State) -> Result<(core::State, Vec<Check>)> {
        self.bound(s)?;
        let boot = s
            .startup_trial(&self.supplied_settings)
            .map_err(|e| e.message)?;
        let loaded = matches!(boot.status, core::Status::Loaded);
        let detail=match &boot.status{core::Status::Loaded=>"The enabled provided loader restored the saved file into a fresh filter table.".into(),core::Status::Inactive=>"The provided loader is disabled. The saved file exists, but this startup did not read it; the fresh filter still uses its declared ACCEPT defaults.".into(),core::Status::Failed{detail}=>format!("The enabled loader could not read a valid complete filter table: {detail}")};
        let mut checks = vec![check(loaded, "Startup · provided loader", detail)];
        checks.extend(self.live_checks(&boot, "Startup · "));
        Ok((boot, checks))
    }
    pub fn checks(&self, s: &core::State) -> Vec<Check> {
        let mut out = self.live_checks(s, "Current · ");
        out.push(check(s.enabled,"Enable the provided startup loader","Starting the oneshot loader changes current rules; enabling it separately selects the declared next-startup restore."));
        match self.startup(s) {
            Ok((_, checks)) => out.extend(checks),
            Err(e) => out.push(check(false, "Startup · supplied test", e)),
        };
        out
    }
}
