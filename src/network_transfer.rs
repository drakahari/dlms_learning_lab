//! Finite file-transfer application layer over existing in-process transport/ACLs.
//! No sockets, host files, credentials, processes, clocks or protocol-codec bridge.
//! FTP uses a passive TCP opening; TFTP uses fixed source-held representative TIDs.
use crate::{acl_routing as route, acl_workspace, transport_core as tcp, transport_workspace};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const VERSION: u32 = 1;
pub const MAX_FILE: usize = 1024;
pub const MAX_FILES: usize = 24;
pub const MAX_TOTAL: usize = 64 * 1024;
pub const MAX_TRACE: usize = 16;
pub type Files = BTreeMap<String, String>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tftp,
    Ftp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Download,
    Upload,
}
impl Protocol {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tftp => "TFTP",
            Self::Ftp => "FTP",
        }
    }
}
impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Download => "Download · server to device",
            Self::Upload => "Upload · device to server",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Access {
    pub read: Vec<String>,
    pub write: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub tftp: bool,
    pub ftp: bool,
    pub tftp_tid: u16,
    pub passive_port: u16,
    /// Supplied permissions on relative virtual-store names, not POSIX modes.
    pub tftp_access: Access,
    /// Fictional already-authenticated login profiles, not USER/PASS emulation.
    pub logins: BTreeMap<String, Access>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub version: u32,
    pub network: route::Network,
    pub client: String,
    pub servers: BTreeMap<String, Server>,
    pub client_port: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    /// Exactly the source-held client/server labels; values are owned text bytes.
    pub files: BTreeMap<String, Files>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol: Protocol,
    pub direction: Direction,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub local_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub remote_path: String,
    pub login: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub category: Category,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn fail(category: Category, s: impl Into<String>) -> Error {
    Error {
        category,
        message: s.into(),
    }
}
fn invalid(s: impl Into<String>) -> Error {
    fail(Category::Invalid, s)
}
fn name(s: &str) -> bool {
    acl_workspace::name(s)
}
/// A virtual-store key, never a host path or a general filesystem expression.
pub fn path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 120
        && !s.starts_with('/')
        && !s.ends_with('/')
        && s.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn prefix(s: &str) -> bool {
    s.is_empty() || s.strip_suffix('/').is_some_and(path)
}
impl Access {
    fn validate(&self) -> Result<(), Error> {
        if self.read.len() + self.write.len() > 16
            || self.read.iter().chain(&self.write).any(|p| !prefix(p))
        {
            return Err(invalid(
                "Use at most 16 source-listed relative store prefixes.",
            ));
        }
        Ok(())
    }
    fn permits(&self, direction: Direction, path: &str) -> bool {
        let prefixes = match direction {
            Direction::Download => &self.read,
            Direction::Upload => &self.write,
        };
        prefixes.iter().any(|p| path.starts_with(p))
    }
}
impl Context {
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(fail(
                Category::Unsupported,
                "Unsupported transfer model version.",
            ));
        }
        acl_workspace::valid_network(&self.network).map_err(invalid)?;
        if self.servers.is_empty()
            || self.servers.len() > 3
            || self.client_port < 1024
            || !name(&self.client)
        {
            return Err(invalid(
                "Supply one client, one to three servers and a nonprivileged client port.",
            ));
        }
        let nodes: BTreeSet<_> = std::iter::once(&self.client)
            .chain(self.servers.keys())
            .collect();
        if nodes.len() != self.servers.len() + 1
            || nodes != self.network.host_endpoints.keys().collect()
        {
            return Err(invalid(
                "Transfer stores must exactly match the supplied network endpoints.",
            ));
        }
        for (id, s) in &self.servers {
            if !name(id)
                || (!s.tftp && !s.ftp)
                || s.tftp_tid < 1024
                || s.passive_port < 1024
                || s.logins.len() > 8
            {
                return Err(invalid(
                    "Supply bounded enabled services and representative server transfer ports.",
                ));
            }
            s.tftp_access.validate()?;
            for (label, a) in &s.logins {
                if !name(label) {
                    return Err(invalid("Invalid training login label."));
                }
                a.validate()?;
            }
        }
        Ok(())
    }
    fn socket(&self, host: &str, port: u16) -> Result<tcp::Socket, Error> {
        let interface = self
            .network
            .host_endpoints
            .get(host)
            .ok_or_else(|| invalid("Unknown supplied endpoint."))?;
        let assignment = self
            .network
            .topology
            .devices
            .get(host)
            .and_then(|d| d.interfaces.get(interface))
            .and_then(|i| i.address.as_ref())
            .ok_or_else(|| invalid("Endpoint has no supplied IPv4 address."))?;
        Ok(tcp::Socket {
            address: assignment.address,
            port,
        })
    }
}
impl World {
    pub fn validate(&self, c: &Context) -> Result<(), Error> {
        c.validate()?;
        let nodes: BTreeSet<_> = std::iter::once(&c.client).chain(c.servers.keys()).collect();
        if nodes != self.files.keys().collect() {
            return Err(invalid(
                "Virtual stores differ from the supplied endpoints.",
            ));
        }
        let mut total = 0usize;
        for files in self.files.values() {
            if files.len() > MAX_FILES {
                return Err(fail(Category::Limit, "Virtual store exceeds 24 files."));
            }
            for (p, text) in files {
                if !path(p) || text.len() > MAX_FILE || text.contains('\0') {
                    return Err(fail(
                        Category::Limit,
                        "Use relative file names and at most 1,024 UTF-8 bytes per fixture file, without NUL.",
                    ));
                }
                total = total
                    .checked_add(p.len() + text.len())
                    .ok_or_else(|| fail(Category::Limit, "Transfer byte budget overflow."))?;
            }
        }
        if total > MAX_TOTAL {
            return Err(fail(
                Category::Limit,
                "Virtual transfer stores exceed 64 KiB.",
            ));
        }
        Ok(())
    }
}
impl Request {
    pub fn validate(&self, c: &Context) -> Result<(), Error> {
        if !c.servers.contains_key(&self.server) {
            return Err(invalid(
                "Choose a supplied server; no real target can be contacted.",
            ));
        }
        for (field, value) in [
            ("Device file", &self.local_path),
            ("Server file", &self.remote_path),
        ] {
            if value.is_empty() {
                return Err(invalid(format!(
                    "{field}: enter a relative virtual file name."
                )));
            }
            if !path(value) {
                return Err(fail(
                    Category::Unsupported,
                    format!(
                        "{field}: use a relative store name such as configs/site.cfg; absolute paths, traversal, URLs, spaces and shell expressions are outside this activity."
                    ),
                ));
            }
        }
        if self.login.as_ref().is_some_and(|s| !name(s)) {
            return Err(invalid("Use a supplied training login label."));
        }
        if self.protocol == Protocol::Tftp && self.login.is_some() {
            return Err(fail(
                Category::Unsupported,
                "TFTP has no login in this exercise. Clear the FTP training login or use FTP.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Request,
    Control,
    Login,
    DataConnection,
    Data,
    Acknowledgement,
    File,
    Complete,
}
impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Request => "Initial request",
            Self::Control => "Control connection",
            Self::Login => "Training login",
            Self::DataConnection => "Data connection",
            Self::Data => "File data",
            Self::Acknowledgement => "Acknowledgement",
            Self::File => "File access",
            Self::Complete => "Transfer complete",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub stage: Stage,
    pub label: String,
    pub packet: Option<tcp::Packet>,
    pub delivered: bool,
    pub block: Option<usize>,
    pub detail: String,
}
/// Runtime observations are not deserializable and never progress authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub success: bool,
    pub stage: Stage,
    pub detail: String,
    pub bytes: Option<usize>,
    pub events: Vec<Event>,
}
impl Report {
    fn new() -> Self {
        Self {
            success: false,
            stage: Stage::Request,
            detail: String::new(),
            bytes: None,
            events: vec![],
        }
    }
    fn stop(mut self, stage: Stage, detail: impl Into<String>) -> Self {
        self.stage = stage;
        self.detail = detail.into();
        self
    }
    fn event(&mut self, event: Event) -> Result<(), Error> {
        if self.events.len() >= MAX_TRACE {
            return Err(fail(Category::Limit, "Transfer trace limit reached."));
        }
        self.events.push(event);
        Ok(())
    }
}
fn decision(network: &route::Network, packet: &tcp::Packet) -> Result<tcp::Decision, Error> {
    let observed =
        transport_workspace::inspect(network, packet).map_err(|e| invalid(e.to_string()))?;
    if !matches!(observed.baseline, route::Baseline::Deliverable(_)) {
        return Err(invalid(
            "The supplied path is not routable even without ACLs; no file receipt is assumed.",
        ));
    }
    match observed.actual {
        route::Outcome::Delivered { .. } => Ok(tcp::Decision::delivered(
            "Delivered by the current virtual route and ACL policy.",
        )),
        route::Outcome::Denied { denial, .. } => Ok(tcp::Decision::dropped(&format!(
            "Filtered at {}/{} {:?}.",
            denial.at.device, denial.at.interface, denial.at.direction
        ))),
        route::Outcome::Failed { .. } => Err(invalid(
            "Virtual forwarding failed; the destination has not received this file.",
        )),
    }
}
fn datagram(
    r: &mut Report,
    network: &route::Network,
    endpoints: (tcp::Socket, tcp::Socket),
    stage: Stage,
    block: Option<usize>,
    bytes: usize,
    label: &str,
) -> Result<bool, Error> {
    let (source, destination) = endpoints;
    let packet = tcp::Packet {
        protocol: tcp::Protocol::Udp,
        source,
        destination,
        kind: tcp::PacketKind::Datagram,
        payload_bytes: bytes,
    };
    let d = decision(network, &packet)?;
    let delivered = d.disposition == tcp::Disposition::Delivered;
    r.event(Event {
        stage,
        label: label.into(),
        packet: Some(packet),
        delivered,
        block,
        detail: d.detail,
    })?;
    Ok(delivered)
}
fn opening(
    r: &mut Report,
    c: &Context,
    network: &route::Network,
    server: &str,
    port: u16,
    stage: Stage,
) -> Result<bool, Error> {
    let client = c.socket(&c.client, c.client_port)?;
    let remote = c.socket(server, port)?;
    // The existing core requires a nonempty prospective payload. It is NEVER
    // written here: exactly the three opening actions run, with zero data bytes.
    let config = tcp::Config {
        version: 1,
        hosts: vec![
            tcp::Host {
                id: c.client.clone(),
                address: client.address,
            },
            tcp::Host {
                id: server.into(),
                address: remote.address,
            },
        ],
        listeners: vec![tcp::Listener {
            protocol: tcp::Protocol::Tcp,
            endpoint: tcp::Endpoint {
                host: server.into(),
                port,
            },
        }],
        exchanges: vec![tcp::Exchange {
            id: "opening".into(),
            protocol: tcp::Protocol::Tcp,
            client: tcp::Endpoint {
                host: c.client.clone(),
                port: c.client_port,
            },
            server: tcp::Endpoint {
                host: server.into(),
                port,
            },
            payload: "Not transmitted by the opening fixture".into(),
        }],
    };
    let mut machine = tcp::Machine::new(&config, "opening").map_err(|e| invalid(e.to_string()))?;
    for action in [
        tcp::Action::Start,
        tcp::Action::Advance,
        tcp::Action::Advance,
    ] {
        machine
            .apply(action, &mut |packet| {
                decision(network, packet)
                    .map_err(|e| tcp::Error::new(tcp::ErrorKind::Classifier, e.message))
            })
            .map_err(|e| invalid(e.to_string()))?;
        if machine.report().phase == tcp::Phase::Finished {
            break;
        }
    }
    for e in &machine.report().trace {
        r.event(Event {
            stage,
            label: format!("{:?}", e.packet.kind),
            packet: Some(e.packet),
            delivered: e.decision.disposition == tcp::Disposition::Delivered,
            block: None,
            detail: e.decision.detail.clone(),
        })?;
    }
    Ok(machine.report().phase == tcp::Phase::Connected
        && machine.report().client_state == Some(tcp::TcpState::Established)
        && machine.report().server_state == Some(tcp::TcpState::Established))
}
/// Complete one bounded byte-preserving transfer, committing destination bytes
/// only after every modeled stage succeeds. This atomic refusal is a disclosed
/// simplification: real interrupted transfers can leave partial destination data.
pub fn transfer(
    c: &Context,
    network: &route::Network,
    w: &mut World,
    q: &Request,
) -> Result<Report, Error> {
    w.validate(c)?;
    q.validate(c)?;
    if !acl_workspace::same_assignment(network, &c.network) {
        return Err(invalid(
            "Keep the source-held topology and endpoint bindings unchanged.",
        ));
    }
    let server = &c.servers[&q.server];
    let mut r = Report::new();
    let client = c.socket(&c.client, c.client_port)?;
    let permissions = match q.protocol {
        Protocol::Tftp => {
            if !server.tftp {
                return Ok(r.stop(
                    Stage::Request,
                    "This server does not offer TFTP. Choose its enabled service.",
                ));
            }
            let request_server = c.socket(&q.server, 69)?;
            if !datagram(
                &mut r,
                network,
                (client, request_server),
                Stage::Request,
                None,
                0,
                match q.direction {
                    Direction::Download => "Read request (RRQ)",
                    Direction::Upload => "Write request (WRQ)",
                },
            )? {
                return Ok(r.stop(Stage::Request,"The initial UDP request to server port 69 was filtered. No file data has moved."));
            }
            &server.tftp_access
        }
        Protocol::Ftp => {
            if !server.ftp {
                return Ok(r.stop(
                    Stage::Control,
                    "This server does not offer FTP. Choose its enabled service.",
                ));
            }
            if !opening(&mut r, c, network, &q.server, 21, Stage::Control)? {
                return Ok(r.stop(Stage::Control,"The TCP port 21 control connection did not open in both directions. Repair its actual failed packet before checking login or data."));
            }
            let Some(access) = q.login.as_ref().and_then(|id| server.logins.get(id)) else {
                return Ok(r.stop(Stage::Login,"The control connection works, but this supplied FTP login is missing or not admitted. Choose a training login from the server's permissions table."));
            };
            r.event(Event{stage:Stage::Login,label:"Training login admitted".into(),packet:None,delivered:true,block:None,detail:"Named profile admission is modeled; USER/PASS text and authentication packets are not emulated.".into()})?;
            access
        }
    };
    if !permissions.permits(q.direction, &q.remote_path) {
        return Ok(r.stop(Stage::File,match q.direction{Direction::Download=>"The selected service/login cannot read this server namespace. Check its read permission and the server file name.",Direction::Upload=>"The selected service/login cannot write this server namespace. A successful login does not grant every file operation."}));
    }
    let (source_host, source_path, dest_host, dest_path) = match q.direction {
        Direction::Download => (&q.server, &q.remote_path, &c.client, &q.local_path),
        Direction::Upload => (&c.client, &q.local_path, &q.server, &q.remote_path),
    };
    let Some(bytes) = w.files[source_host].get(source_path) else {
        return Ok(r.stop(Stage::File,format!("Source file {source_host}/{source_path} does not exist. Download reads the server; upload reads the device. Nothing was copied.")));
    };
    if w.files[dest_host].len() == MAX_FILES && !w.files[dest_host].contains_key(dest_path) {
        return Err(fail(
            Category::Limit,
            "Destination store has no free file slot; no partial file was created.",
        ));
    }
    let (sender, receiver) = match q.direction {
        Direction::Download => (
            c.socket(
                &q.server,
                match q.protocol {
                    Protocol::Tftp => server.tftp_tid,
                    Protocol::Ftp => server.passive_port,
                },
            )?,
            client,
        ),
        Direction::Upload => (
            client,
            c.socket(
                &q.server,
                match q.protocol {
                    Protocol::Tftp => server.tftp_tid,
                    Protocol::Ftp => server.passive_port,
                },
            )?,
        ),
    };
    match q.protocol {
        Protocol::Tftp => {
            if q.direction == Direction::Upload
                && !datagram(
                    &mut r,
                    network,
                    (receiver, sender),
                    Stage::Acknowledgement,
                    Some(0),
                    0,
                    "Write-request acknowledgement",
                )?
            {
                return Ok(r.stop(Stage::Acknowledgement,"The server's acknowledgement from its transfer port was filtered. UDP port 69 is only the initial request port."));
            }
            // Every exact multiple needs a final empty DATA block, including an
            // empty source. Slice byte counts only: UTF8 may span a block boundary.
            for block in 0..=bytes.len() / 512 {
                let n = bytes.len().saturating_sub(block * 512).min(512);
                if !datagram(
                    &mut r,
                    network,
                    (sender, receiver),
                    Stage::Data,
                    Some(block + 1),
                    n,
                    "Octet DATA block",
                )? {
                    return Ok(r.stop(Stage::Data,"A TFTP DATA block was filtered. After the first request, traffic uses the client/server transfer ports, not only UDP port 69. The destination remains unchanged in this atomic exercise."));
                }
                if !datagram(
                    &mut r,
                    network,
                    (receiver, sender),
                    Stage::Acknowledgement,
                    Some(block + 1),
                    0,
                    "Block acknowledgement",
                )? {
                    return Ok(r.stop(Stage::Acknowledgement,"The DATA block arrived but its acknowledgement was filtered. A one-way path is insufficient; the destination remains unchanged in this atomic exercise."));
                }
            }
        }
        Protocol::Ftp => {
            if !opening(
                &mut r,
                c,
                network,
                &q.server,
                server.passive_port,
                Stage::DataConnection,
            )? {
                return Ok(r.stop(Stage::DataConnection,format!("FTP control/login works, but the separate passive TCP{} data connection cannot open. Permitting TCP port 21 alone is insufficient.",server.passive_port)));
            }
            if !bytes.is_empty() {
                let packet = tcp::Packet {
                    protocol: tcp::Protocol::Tcp,
                    source: sender,
                    destination: receiver,
                    kind: tcp::PacketKind::TcpData,
                    payload_bytes: bytes.len(),
                };
                let d = decision(network, &packet)?;
                let delivered = d.disposition == tcp::Disposition::Delivered;
                r.event(Event {
                    stage: Stage::Data,
                    label: match q.direction {
                        Direction::Download => "Server sends the requested file",
                        Direction::Upload => "Device sends its local file",
                    }
                    .into(),
                    packet: Some(packet),
                    delivered,
                    block: None,
                    detail: d.detail,
                })?;
                if !delivered {
                    return Ok(r.stop(Stage::Data,"The data connection opened, but the file's data direction was filtered. Download data comes from the server; upload data goes to it."));
                }
            }
        }
    }
    let n = bytes.len();
    let mut next = w.clone();
    next.files
        .get_mut(dest_host)
        .unwrap()
        .insert(dest_path.clone(), bytes.clone());
    next.validate(c)?;
    *w = next;
    r.success = true;
    r.stage = Stage::Complete;
    r.bytes = Some(n);
    r.detail = format!(
        "Copied {n} bytes from {source_host}/{source_path} to {dest_host}/{dest_path}. The source remains intact."
    );
    Ok(r)
}
