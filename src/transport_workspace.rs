//! Source-held transport activities over the existing readonly route/ACL model.
//! Persist settings, never trusted packet reports or learner-supplied callbacks.
use crate::{
    acl_editor, acl_routing as routing, acl_workspace, script_teaching::View,
    transport_core as core,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Optional echo observations reuse the same supplied route/ACL model.
pub mod echo;

pub fn protocol_name(p: core::Protocol) -> &'static str {
    match p {
        core::Protocol::Tcp => "TCP",
        core::Protocol::Udp => "UDP",
    }
}
pub fn packet_name(p: core::PacketKind) -> &'static str {
    match p {
        core::PacketKind::Syn => "SYN",
        core::PacketKind::SynAck => "SYN-ACK",
        core::PacketKind::Ack => "ACK",
        core::PacketKind::TcpData => "TCP data",
        core::PacketKind::Datagram => "UDP datagram",
    }
}
pub const PREFIX: &str = "@transport ";
pub const MAX_SESSION: usize = 256 * 1024;
pub const HELP: &str = "Edit the supplied protocol/port settings or the selected simulated router's ACL document, then Check configuration. TCP generates SYN, reverse SYN-ACK, final ACK and one data fragment; UDP generates one datagram. No sockets, host commands, real network, retransmissions or full IOS/Linux environment.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListenerDraft {
    pub protocol: core::Protocol,
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeDraft {
    pub id: String,
    pub protocol: core::Protocol,
    pub client_port: String,
    pub server_port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub listeners: Vec<ListenerDraft>,
    pub exchanges: Vec<ExchangeDraft>,
    pub documents: acl_editor::Drafts,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    Receive {
        exchange: String,
        protocol: core::Protocol,
        server_port: Option<u16>,
    },
    Block {
        exchange: String,
        at: routing::Location,
    },
    EchoReceive {
        query: String,
    },
}
impl Requirement {
    fn id(&self) -> &str {
        match self {
            Self::Receive { exchange, .. } | Self::Block { exchange, .. } => exchange,
            Self::EchoReceive { query } => query,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub transport: core::Config,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "crate::cron_workspace::sequence::<_,core::echo::Query,8>"
    )]
    pub echoes: Vec<core::echo::Query>,
    pub network: routing::Network,
    pub editable_listeners: Vec<usize>,
    pub editable_exchanges: Vec<String>,
    pub editable_routers: Vec<String>,
    pub requirements: Vec<Requirement>,
}
fn port(text: &str, label: &str) -> Result<u16, String> {
    if text.is_empty() {
        return Err(format!("{label}: enter a port number before checking."));
    }
    if text.len() > 5 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "{label}: use a whole decimal port number from 1 to 65535."
        ));
    }
    text.parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| format!("{label}: this activity uses ports 1–65535."))
}
impl Draft {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.transport.validate().map_err(|e| e.to_string())?;
        let documents = acl_workspace::Draft::new(&task.network)?.documents;
        Ok(Self {
            listeners: task
                .transport
                .listeners
                .iter()
                .map(|l| ListenerDraft {
                    protocol: l.protocol,
                    port: l.endpoint.port.to_string(),
                })
                .collect(),
            exchanges: task
                .transport
                .exchanges
                .iter()
                .map(|e| ExchangeDraft {
                    id: e.id.clone(),
                    protocol: e.protocol,
                    client_port: e.client.port.to_string(),
                    server_port: e.server.port.to_string(),
                })
                .collect(),
            documents,
        })
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        let initial = Self::new(task)?;
        if self.listeners.len() != initial.listeners.len()
            || self.exchanges.len() != initial.exchanges.len()
            || self
                .documents
                .routers
                .keys()
                .ne(initial.documents.routers.keys())
            || self.documents.schema_version != 1
        {
            return Err("The supplied listener, exchange and router rows must stay intact.".into());
        }
        for (i, (a, b)) in self.listeners.iter().zip(&initial.listeners).enumerate() {
            if a.port.len() > 16 || (!task.editable_listeners.contains(&i) && a != b) {
                return Err("A protected listener changed or a port field is too long.".into());
            }
        }
        for (a, b) in self.exchanges.iter().zip(&initial.exchanges) {
            if a.id != b.id
                || a.client_port.len() > 16
                || a.server_port.len() > 16
                || (!task.editable_exchanges.contains(&a.id) && a != b)
            {
                return Err("A supplied exchange identity or protected setting changed.".into());
            }
        }
        self.documents
            .validate(&task.network)
            .map_err(|e| e.to_string())?;
        for (router, text) in &self.documents.routers {
            if !task.editable_routers.contains(router)
                && initial.documents.routers.get(router) != Some(text)
            {
                return Err("A protected router configuration changed.".into());
            }
        }
        Ok(())
    }
    pub fn compile(&self, task: &Task) -> Result<(core::Config, routing::Network), String> {
        self.bound(task)?;
        let mut config = task.transport.clone();
        for (i, (l, d)) in config.listeners.iter_mut().zip(&self.listeners).enumerate() {
            l.protocol = d.protocol;
            l.endpoint.port = port(&d.port, &format!("Listener {}", i + 1))?;
        }
        for (e, d) in config.exchanges.iter_mut().zip(&self.exchanges) {
            e.protocol = d.protocol;
            e.client.port = port(&d.client_port, &format!("{} client port", e.id))?;
            e.server.port = port(&d.server_port, &format!("{} service port", e.id))?;
        }
        config.validate().map_err(|e| e.to_string())?;
        let network = acl_workspace::apply_documents(&task.network, &self.documents)?;
        Ok((config, network))
    }
}
fn binding(
    network: &routing::Network,
    address: std::net::Ipv4Addr,
) -> Result<routing::Endpoint, core::Error> {
    for (host, interface) in &network.host_endpoints {
        if network.topology.devices[host].interfaces[interface]
            .address
            .as_ref()
            .is_some_and(|a| a.address == address)
        {
            return Ok(routing::Endpoint {
                device: host.clone(),
                interface: interface.clone(),
            });
        }
    }
    Err(core::Error::new(
        core::ErrorKind::Classifier,
        "Packet address is not a supplied endpoint.",
    ))
}
pub fn inspect(
    network: &routing::Network,
    packet: &core::Packet,
) -> Result<routing::Inspection, core::Error> {
    // Public entry validates before indexed lookup. No interface-name guess or host call.
    network.validate().map_err(|_| {
        core::Error::new(
            core::ErrorKind::Classifier,
            "The supplied simulated network is invalid.",
        )
    })?;
    let flow = routing::Flow {
        source: binding(network, packet.source.address)?,
        destination: binding(network, packet.destination.address)?,
        protocol: core::packet_protocol(packet),
    };
    network.inspect(&flow).map_err(|_| {
        core::Error::new(
            core::ErrorKind::Classifier,
            "This packet path is unsupported; inspect the supplied route/interface settings.",
        )
    })
}
pub fn reports(task: &Task, draft: &Draft) -> Result<Vec<core::Report>, String> {
    let (config, network) = draft.compile(task)?;
    core::run_all(&config, &mut |packet| {
        let observed = inspect(&network, packet)?;
        if !matches!(observed.baseline, routing::Baseline::Deliverable(_)) {
            return Err(core::Error::new(
                core::ErrorKind::Classifier,
                "The supplied route cannot reach the intended host even without ACLs.",
            ));
        }
        Ok(match observed.actual {
            routing::Outcome::Delivered { .. } => {
                core::Decision::delivered("Network delivered this packet to the destination host.")
            }
            routing::Outcome::Denied { denial, .. } => core::Decision::dropped(&format!(
                "ACL blocked at {}/{} {:?}.",
                denial.at.device, denial.at.interface, denial.at.direction
            )),
            routing::Outcome::Failed { .. } => {
                return Err(core::Error::new(
                    core::ErrorKind::Classifier,
                    "Simulated forwarding failed; no receipt is assumed.",
                ));
            }
        })
    })
    .map_err(|e| e.to_string())
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.transport.validate().map_err(|e| e.to_string())?;
        core::echo::validate(&self.transport.hosts, &self.echoes).map_err(|e| e.to_string())?;
        if self
            .echoes
            .iter()
            .any(|q| self.transport.exchanges.iter().any(|e| e.id == q.id))
        {
            return Err(
                "Echo query names and transport conversation names must be distinct.".into(),
            );
        }
        acl_workspace::valid_network(&self.network)?;
        if self.network.host_endpoints.len() != self.transport.hosts.len() {
            return Err("Transport hosts must exactly match supplied network endpoints.".into());
        }
        for host in &self.transport.hosts {
            let endpoint = binding(&self.network, host.address).map_err(|e| e.to_string())?;
            if endpoint.device != host.id {
                return Err(
                    "Transport host identity/address differs from its network binding.".into(),
                );
            }
        }
        let initial = Draft::new(self)?;
        if self.editable_listeners.len() > core::MAX_LISTENERS
            || self.editable_exchanges.len() > core::MAX_EXCHANGES
            || self.editable_routers.len() > 12
            || self.requirements.len() != self.transport.exchanges.len() + self.echoes.len()
        {
            return Err("Invalid transport task control or requirement count.".into());
        }
        for values in [&self.editable_exchanges, &self.editable_routers] {
            if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
                return Err("Duplicate editable control.".into());
            }
        }
        if self
            .editable_listeners
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != self.editable_listeners.len()
            || self
                .editable_listeners
                .iter()
                .any(|i| *i >= initial.listeners.len())
            || self
                .editable_exchanges
                .iter()
                .any(|id| !initial.exchanges.iter().any(|e| &e.id == id))
            || self
                .editable_routers
                .iter()
                .any(|id| !initial.documents.routers.contains_key(id))
        {
            return Err("Unknown or duplicate editable transport control.".into());
        }
        let mut ids = BTreeSet::new();
        let mut positive = false;
        for requirement in &self.requirements {
            let known = match requirement {
                Requirement::EchoReceive { query } => self.echoes.iter().any(|q| &q.id == query),
                _ => self
                    .transport
                    .exchanges
                    .iter()
                    .any(|e| e.id == requirement.id()),
            };
            if !ids.insert(requirement.id()) || !known {
                return Err(
                    "Each supplied conversation/query needs one matching typed requirement.".into(),
                );
            }
            match requirement {
                // These comparison tasks retain a positive transport receipt too.
                Requirement::EchoReceive { .. } => {}
                Requirement::Receive { server_port, .. } => {
                    positive = true;
                    if *server_port == Some(0) {
                        return Err("Required activity port must be nonzero.".into());
                    }
                }
                Requirement::Block { at, .. } => {
                    if !self
                        .network
                        .topology
                        .devices
                        .get(&at.device)
                        .is_some_and(|d| d.interfaces.contains_key(&at.interface))
                    {
                        return Err("Unknown protected ACL boundary.".into());
                    }
                }
            }
        }
        if !positive {
            return Err(
                "Keep at least one real positive receipt requirement; deny-all cannot pass.".into(),
            );
        }
        initial.bound(self)
    }
    pub fn checks(&self, draft: &Draft) -> Vec<(bool, String)> {
        if let Err(e) = self.validate() {
            return vec![(false, e)];
        }
        let result = reports(self, draft);
        let actual = match result {
            Ok(r) => r,
            Err(e) => return vec![(false, e)],
        };
        let network = match draft.compile(self) {
            Ok((_, n)) => n,
            Err(e) => return vec![(false, e)],
        };
        let echoes = match echo::with_network(self, &network) {
            Ok(reports) => reports,
            Err(e) => return vec![(false, e)],
        };
        self.requirements.iter().map(|required| {
            if let Requirement::EchoReceive { query } = required {
                let source = self.echoes.iter().find(|q| &q.id == query).expect("validated echo source");
                let report = echoes.iter().find(|r| &r.query == query).expect("validated echo report");
                return echo::check(source, report);
            }
            let r=actual.iter().find(|r|r.exchange==required.id()).expect("validated fixed roster");
            match required {
                Requirement::EchoReceive { .. } => unreachable!("echo handled above"),
                Requirement::Receive{protocol,server_port,..}=>{
                    let expected=&self.transport.exchanges.iter().find(|e|e.id==r.exchange).expect("fixed source").payload;
                    let pass=r.protocol==*protocol && server_port.is_none_or(|p|p==r.server.port) && r.outcome==core::Outcome::Delivered && r.received.as_ref()==Some(expected);
                    let why=if r.protocol!=*protocol {format!("Choose {}; changing both ends to another protocol does not meet this task.",protocol_name(*protocol))} else if server_port.is_some_and(|p|p!=r.server.port){format!("Use the required service port {}; the client port identifies this conversation.",server_port.unwrap())} else {match r.outcome {core::Outcome::Delivered=>"The destination received the complete supplied content.".into(),core::Outcome::NoListener=>"The packet reached the host, but no listener matches both its protocol and destination port. Compare both ends.".into(),core::Outcome::PacketDropped{kind}=>format!("{} was filtered. Inspect the actual direction and both ports; sending is not receipt.",packet_name(kind)),_=>"The exchange has not delivered its content.".into()}};
                    (pass,format!("{} — {why}",r.exchange))
                },
                Requirement::Block{at,..}=>{
                    let blocked=r.trace.last().and_then(|e|inspect(&network,&e.packet).ok()).is_some_and(|i|matches!(i.baseline,routing::Baseline::Deliverable(_)) && matches!(i.actual,routing::Outcome::Denied{denial,..} if denial.at==*at));
                    let crossing=actual.iter().filter(|r|r.outcome==core::Outcome::Delivered && self.requirements.iter().any(|requirement|matches!(requirement,Requirement::Receive{exchange,protocol,server_port} if exchange==&r.exchange && *protocol==r.protocol && server_port.is_none_or(|p|p==r.server.port)))).flat_map(|r|&r.trace).any(|e|inspect(&network,&e.packet).is_ok_and(|i|matches!(i.actual,routing::Outcome::Delivered{events,..} if events.iter().any(|e|matches!(e,routing::Event::Boundary{at:seen,..} if seen==at)))));
                    let boundary=format!("{}/{} {}",at.device,at.interface,match at.direction{routing::Direction::Inbound=>"inbound",routing::Direction::Outbound=>"outbound"});
                    let why=if blocked&&crossing{format!("Forbidden traffic is filtered at {boundary}; a required service still crosses this boundary.")}
                    else if blocked{format!("Traffic is filtered at {boundary}, but no required service currently crosses it successfully. Repair useful traffic too.")}
                    else if r.outcome==core::Outcome::NoListener{format!("No listener received this traffic, but that is not an ACL restriction at {boundary}. Keep the supplied endpoints and repair policy.")}
                    else{format!("This traffic is not filtered at {boundary}. Inspect its actual trace and retain the working service rules.")};
                    (blocked && crossing && r.received.is_none(),format!("{} — {why}",r.exchange))
                },
            }
        }).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub draft: Draft,
    pub tested: Option<Draft>,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        Ok(Self {
            draft: Draft::new(task)?,
            tested: None,
        })
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.draft.bound(task)?;
        if self.tested.as_ref().is_some_and(|d| d != &self.draft) {
            return Err("Saved transport feedback is stale; original retained.".into());
        }
        Ok(())
    }
    pub fn edit(&mut self, task: &Task, draft: Draft) -> Result<(), String> {
        draft.bound(task)?;
        if self.draft != draft {
            self.draft = draft;
            self.tested = None;
        }
        Ok(())
    }
    pub fn check(&mut self, task: &Task) -> Result<Vec<(bool, String)>, String> {
        self.bound(task)?;
        self.tested = Some(self.draft.clone());
        Ok(task.checks(&self.draft))
    }
    pub fn checks(&self, task: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(task) {
            return vec![(false, e)];
        }
        if self.tested.is_none() {
            return vec![(
                false,
                "Not checked. Use Check configuration to run the supplied exchanges.".into(),
            )];
        }
        task.checks(&self.draft)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    pub worked_assignment: Draft,
    pub worked_example: Draft,
}
impl Definition {
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| self.bound(s)) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.assignment)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !crate::acl_workspace::name(&self.source_id) {
            return Err("Unknown transport source/version.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.transport.hosts == self.example.transport.hosts {
            return Err("Transport practice needs separate supplied hosts.".into());
        }
        if !self
            .assignment
            .checks(&self.worked_assignment)
            .iter()
            .all(|(ok, _)| *ok)
            || !self
                .example
                .checks(&self.worked_example)
                .iter()
                .all(|(ok, _)| *ok)
        {
            return Err(
                "Transport worked settings do not reach their supplied requirements.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        if s.version != 1
            || s.origin != self.source_id
            || serde_json::to_vec(s).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err(
                "Transport source, version or saved size differs; original retained.".into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        Ok(Self {
            version: 1,
            origin: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn valid_shape(&self) -> bool {
        self.version == 1
            && crate::acl_workspace::name(&self.origin)
            && serde_json::to_vec(self).is_ok_and(|b| b.len() <= MAX_SESSION)
    }

    pub fn execute(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.validate()?;
        d.bound(self)?;
        if input == "help" {
            return Ok(HELP.into());
        }
        let raw = input.strip_prefix(PREFIX).ok_or(
            "Use the native protocol/port controls and ACL editor, then Check configuration.",
        )?;
        if raw.len() > MAX_SESSION {
            return Err("Transport action exceeds its input bound.".into());
        }
        let action: Action =
            serde_json::from_str(raw).map_err(|_| "Malformed or unsupported transport action.")?;
        match action {
            Action::Edit { draft } => self.assignment.edit(&d.assignment, draft)?,
            Action::Check => {
                self.assignment.check(&d.assignment)?;
            }
        }
        Ok("Simulated transport settings checked or saved; no host network was used.".into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit { draft: Draft },
    Check,
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::Transport { simulation },
        crate::engine::Goal::Transport { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Transport task and workspace must agree.".into());
    };
    definition.validate()?;
    if definition.source_id != s.id || **simulation != Session::new(definition)? {
        return Err(
            "Transport initial settings or source identity differ from the supplied task.".into(),
        );
    }
    if s.steps.len() < 3 || s.guide.is_empty() || s.explanation.trim().is_empty() {
        return Err("Transport tasks need essential teaching, separate practice and independent requirements.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::Transport { simulation } = &a.state else {
        return Err("Saved transport work belongs to another workspace; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible transport attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
