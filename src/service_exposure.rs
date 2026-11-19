//! Bounded observer-specific TCP evidence. No lesson opts in implicitly.
//! Reuses real simulated process/listener and numeric-filter models. No host APIs.
use crate::{linux_filter as filter, processes};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
type Result<T> = std::result::Result<T, filter::Fault>;
pub const MAX_PORTS: usize = 32;
fn fail<T>(class: filter::Class, message: &str) -> Result<T> {
    Err(filter::Fault {
        class,
        message: message.into(),
    })
}
fn unicast(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified()
        && !ip.is_multicast()
        && !ip.is_broadcast()
        && !ip.is_loopback()
        && !ip.is_link_local()
}
fn name(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewpoint {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub address: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub server: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Viewpoint,8>")]
    pub viewpoints: Vec<Viewpoint>,
    pub processes: processes::Table,
    pub filter: filter::Table,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortState {
    Open,
    Closed,
    Filtered,
}
impl PortState {
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Filtered => "filtered",
        }
    }
    pub fn explanation(self) -> &'static str {
        match self {
            Self::Open => {
                "The supplied path allows both directions and a live owner retains a matching TCP listener. This proves the modeled connection, not application health or safety."
            }
            Self::Closed => {
                "The supplied path allows the request and response, but no live owner has a matching TCP listener."
            }
            Self::Filtered => {
                "A modeled INPUT or OUTPUT drop prevents a conclusive response. This does not tell you whether the service is listening."
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub port: u16,
    pub state: PortState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminOutput {
    pub text: String,
    pub changed: bool,
    pub status: i32,
}
impl World {
    pub fn validate(&self) -> Result<()> {
        if !unicast(self.server)
            || !name(&self.device, 15)
            || self.device == "."
            || self.device == ".."
            || self.viewpoints.is_empty()
            || self.viewpoints.len() > 8
            || !self.processes.valid()
        {
            return fail(
                filter::Class::Context,
                "Use one supplied unicast server, one device, valid process metadata and one to eight observer positions.",
            );
        }
        self.filter.validate()?;
        if self.processes.listeners.iter().any(|l| {
            !l.address.is_unspecified() && !l.address.is_loopback() && l.address != self.server
        }) {
            return fail(
                filter::Class::Context,
                "Listeners must use the supplied server address, loopback or wildcard binding.",
            );
        }
        for chain in [
            &self.filter.input,
            &self.filter.forward,
            &self.filter.output,
        ] {
            for r in &chain.rules {
                if [&r.ingress, &r.egress]
                    .into_iter()
                    .flatten()
                    .any(|d| d != &self.device)
                {
                    return fail(
                        filter::Class::Context,
                        "Only the supplied virtual server interface is modeled.",
                    );
                }
            }
        }
        let mut ids = BTreeSet::new();
        let mut ips = BTreeSet::new();
        for v in &self.viewpoints {
            if !name(&v.id, 64)
                || !unicast(v.address)
                || v.address == self.server
                || !ids.insert(&v.id)
                || !ips.insert(v.address)
            {
                return fail(
                    filter::Class::Context,
                    "Observer names and unicast addresses must be distinct supplied facts.",
                );
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| filter::Fault {
                class: filter::Class::Context,
                message: "Invalid virtual exposure state.".into(),
            })?
            .len()
            > 128 * 1024
        {
            return fail(
                filter::Class::Limit,
                "The virtual exposure state exceeds128KiB.",
            );
        }
        Ok(())
    }
    pub fn same_problem(&self, initial: &Self) -> bool {
        self.validate().is_ok()
            && initial.validate().is_ok()
            && self.server == initial.server
            && self.device == initial.device
            && self.viewpoints == initial.viewpoints
            && self.processes.same_problem(&initial.processes)
    }
    /// The fixture supplies a reachable path before/after this one host. Only
    /// its INPUT and OUTPUT gates are modeled. No discovery/routing/packets.
    pub fn scan(&self, viewpoint: usize, ports: &[u16]) -> Result<Vec<Observation>> {
        self.validate()?;
        let Some(v) = self.viewpoints.get(viewpoint) else {
            return fail(
                filter::Class::Context,
                "Choose one of the supplied observer positions.",
            );
        };
        let unique: BTreeSet<_> = ports.iter().copied().collect();
        if ports.is_empty()
            || ports.len() > MAX_PORTS
            || unique.len() != ports.len()
            || unique.contains(&0)
        {
            return fail(
                filter::Class::Limit,
                "Choose one to32distinct TCP ports from1through65535.",
            );
        }
        let mut observations = Vec::with_capacity(ports.len());
        for port in ports {
            let request = filter::Packet {
                source: v.address,
                destination: self.server,
                ingress: Some(self.device.clone()),
                egress: None,
                protocol: filter::Protocol::Tcp,
                source_port: Some(40000),
                destination_port: Some(*port),
            };
            let response = filter::Packet {
                source: self.server,
                destination: v.address,
                ingress: None,
                egress: Some(self.device.clone()),
                protocol: filter::Protocol::Tcp,
                source_port: Some(*port),
                destination_port: Some(40000),
            };
            let blocked = self
                .filter
                .evaluate(filter::Chain::Input, &request)?
                .verdict
                == filter::Verdict::Drop
                || self
                    .filter
                    .evaluate(filter::Chain::Output, &response)?
                    .verdict
                    == filter::Verdict::Drop;
            let listening = self.processes.listeners.iter().any(|l| {
                l.protocol == processes::Protocol::Tcp
                    && l.port == *port
                    && (l.address.is_unspecified() || l.address == self.server)
                    && self
                        .processes
                        .processes
                        .get(&l.pid)
                        .is_some_and(|p| p.phase != processes::Phase::Exited)
            });
            observations.push(Observation {
                port: *port,
                state: if blocked {
                    PortState::Filtered
                } else if listening {
                    PortState::Open
                } else {
                    PortState::Closed
                },
            });
        }
        observations.sort_by_key(|r| r.port);
        Ok(observations)
    }
    /// Honest finite grammar; no source/hostname/target escapes and no subprocess.
    /// Observer is selected by a native control, not by spoofing a command flag.
    pub fn scan_command(&self, viewpoint: usize, command: &str) -> Result<Vec<Observation>> {
        if command.len() > 1024
            || !command.is_ascii()
            || command.bytes().any(|b| b.is_ascii_control() && b != b'\t')
        {
            return fail(
                filter::Class::Limit,
                "Keep a scan command to one ASCII line within1024bytes.",
            );
        }
        let words: Vec<_> = command.split_whitespace().collect();
        let mut flags = BTreeSet::new();
        let mut ports = None;
        let mut target = None;
        let mut i = 1;
        if words.first() != Some(&"nmap") {
            return fail(
                filter::Class::Unsupported,
                "Scanner subset: nmap -n -Pn -sT -p PORT[,PORT...] SUPPLIED_IP. Other scanner syntax is not implemented.",
            );
        }
        while i < words.len() {
            match words[i] {
                "-n" | "-Pn" | "-sT" => {
                    if !flags.insert(words[i]) {
                        return fail(filter::Class::Syntax, "Do not repeat a scan option.");
                    }
                }
                "-p" => {
                    if ports.is_some() || i + 1 >= words.len() {
                        return fail(
                            filter::Class::Syntax,
                            "Use one -p followed by a comma-separated TCP port list.",
                        );
                    }
                    i += 1;
                    let mut p = Vec::new();
                    for s in words[i].split(',') {
                        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                            return fail(
                                filter::Class::Unsupported,
                                "Use numeric individual ports, separated by commas; ranges/service names are not implemented.",
                            );
                        }
                        let n = s.parse::<u16>().map_err(|_| filter::Fault {
                            class: filter::Class::Syntax,
                            message: "TCP port must be between1and65535.".into(),
                        })?;
                        p.push(n);
                        if p.len() > MAX_PORTS {
                            return fail(
                                filter::Class::Limit,
                                "At most32TCP ports per observation.",
                            );
                        }
                    }
                    ports = Some(p);
                }
                t if !t.starts_with('-') => {
                    if target.replace(t).is_some() {
                        return fail(
                            filter::Class::Syntax,
                            "Choose exactly one supplied server address.",
                        );
                    }
                }
                _ => {
                    return fail(
                        filter::Class::Unsupported,
                        "Only -n, -Pn, -sT and a numeric -p list are supported; no scripts, service detection, UDP, discovery or host execution.",
                    );
                }
            }
            i += 1;
        }
        if flags != BTreeSet::from(["-n", "-Pn", "-sT"]) {
            return fail(
                filter::Class::Unsupported,
                "This lesson requires -n (numeric), -Pn (skip discovery) and -sT (TCP connect); other scan modes are not modeled.",
            );
        }
        if target != Some(self.server.to_string().as_str()) {
            return fail(
                filter::Class::Context,
                "Use only the supplied virtual server address; no real target is contacted.",
            );
        }
        let Some(ports) = ports else {
            return fail(filter::Class::Syntax, "Choose a numeric -p port list.");
        };
        self.scan(viewpoint, &ports)
    }
    pub fn admin_command(&mut self, line: &str) -> Result<AdminOutput> {
        self.validate()?;
        if line.len() > 4096 || !line.is_ascii() {
            return fail(
                filter::Class::Limit,
                "Keep server commands within4096ASCIIbytes.",
            );
        }
        let mut next = self.clone();
        let output = if line.split_whitespace().next() == Some("iptables") {
            if self.processes.user.uid != 0 {
                return fail(
                    filter::Class::Context,
                    "This subset requires the supplied administrator for firewall commands; sudo/login are not modeled.",
                );
            }
            let out = next.filter.command(line)?;
            AdminOutput {
                text: out.text,
                changed: out.changed,
                status: 0,
            }
        } else {
            let out = next
                .processes
                .execute(line)
                .map_err(|message| filter::Fault {
                    class: filter::Class::Unsupported,
                    message,
                })?;
            AdminOutput {
                text: format!("{}{}", out.stdout, out.stderr),
                changed: next.processes != self.processes,
                status: out.status,
            }
        };
        next.validate()?;
        *self = next;
        Ok(output)
    }
}
pub fn report(world: &World, viewpoint: usize, rows: &[Observation]) -> Result<String> {
    world.validate()?;
    let Some(v) = world.viewpoints.get(viewpoint) else {
        return fail(filter::Class::Context, "Unknown observer.");
    };
    if rows.len() > MAX_PORTS {
        return fail(filter::Class::Limit, "Too many observation rows.");
    }
    let mut text = format!(
        "Simulated TCP connect results · {} ({}) → {}\nPORT       STATE\n",
        v.id, v.address, world.server
    );
    for r in rows {
        text.push_str(&format!(
            "{:<10} {}\n",
            format!("{}/tcp", r.port),
            r.state.name()
        ));
    }
    text.push_str(
        "Supplied path; numeric TCP subset. No application/version/vulnerability inference.\n",
    );
    Ok(text)
}
