//! Outcome grading over virtual peer snapshots; never an OpenVPN execution route.
use crate::vpn_point::{self as core, Class, Config, Error, Material, Network, Role, Supplied};
use serde::{Deserialize, Deserializer, Serialize};
use std::net::Ipv4Addr;
type Result<T> = std::result::Result<T, Error>;
fn fail(message: impl Into<String>) -> Error {
    Error {
        class: Class::Configuration,
        line: None,
        message: message.into(),
    }
}
#[derive(Deserialize)]
struct Profile(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
fn profiles<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<[String; 2], D::Error> {
    Ok(<[Profile; 2]>::deserialize(d)?.map(|s| s.0))
}
fn applied_profiles<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<[String; 2]>, D::Error> {
    Ok(Option::<[Profile; 2]>::deserialize(d)?.map(|a| a.map(|s| s.0)))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub supplied: Supplied,
    pub lans: [Network; 2],
    #[serde(deserialize_with = "profiles")]
    pub saved: [String; 2],
    pub applied: Option<[Config; 2]>,
    #[serde(deserialize_with = "applied_profiles")]
    pub applied_source: Option<[String; 2]>,
}
impl World {
    fn parsed_profiles(&self, texts: &[String; 2]) -> Result<[Config; 2]> {
        let parse = |i: usize| {
            core::parse(&texts[i]).map_err(|mut e| {
                e.message = format!("{} profile: {}", self.supplied.peers[i].label, e.message);
                e
            })
        };
        Ok([parse(0)?, parse(1)?])
    }
    pub fn new(supplied: Supplied, lans: [Network; 2], saved: [String; 2]) -> Result<Self> {
        let w = Self {
            supplied,
            lans,
            saved,
            applied: None,
            applied_source: None,
        };
        w.validate()?;
        Ok(w)
    }
    pub fn validate(&self) -> Result<()> {
        self.supplied.validate()?;
        if self
            .saved
            .iter()
            .any(|s| s.len() > 8192 || s.contains('\0'))
            || serde_json::to_vec(self)
                .map_err(|e| fail(e.to_string()))?
                .len()
                > 64 * 1024
        {
            return Err(fail(
                "Keep the supplied peer world within 64 KiB and each saved profile within 8 KiB.",
            ));
        }
        for lan in &self.lans {
            lan.validate()?;
            if !(1..=30).contains(&lan.prefix) {
                return Err(fail(
                    "These two conventional LANs use prefixes 1 through 30; other link topologies are outside this exercise.",
                ));
            }
        }
        if self.lans[0].contains(self.lans[1].address)
            || self.lans[1].contains(self.lans[0].address)
            || self
                .supplied
                .peers
                .iter()
                .any(|p| self.lans.iter().any(|n| n.contains(p.endpoint)))
        {
            return Err(fail(
                "Supply disjoint LANs and distinct outside peer endpoints.",
            ));
        }
        match (&self.applied, &self.applied_source) {
            (None, None) => {}
            (Some(configs), Some(texts)) => {
                let parsed = self.parsed_profiles(texts)?;
                if configs != &parsed {
                    return Err(fail(
                        "Applied values must derive from their actual supplied profile text.",
                    ));
                }
                core::peer_checks(&self.supplied, configs)?;
            }
            _ => return Err(fail("Retain applied profiles and their source together.")),
        }
        Ok(())
    }
    pub fn save(&mut self, peer: usize, text: &str) -> Result<()> {
        if peer > 1 || text.len() > 8192 || text.contains('\0') {
            return Err(fail(
                "Choose a supplied peer and keep its profile within 8 KiB without NUL.",
            ));
        }
        let mut next = self.clone();
        next.saved[peer] = text.into();
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn apply(&mut self) -> Result<()> {
        self.validate()?;
        let configs = self.parsed_profiles(&self.saved)?;
        // Unresolvable files/types refuse atomically. Incorrect but supported peer
        // settings remain observable; passing a policy is not a parsing prerequisite.
        core::peer_checks(&self.supplied, &configs)?;
        let mut next = self.clone();
        next.applied = Some(configs);
        next.applied_source = Some(self.saved.clone());
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn probe(&self, from: usize, source: Ipv4Addr, destination: Ipv4Addr) -> Result<Probe> {
        self.validate()?;
        if from > 1 || !usable(self.lans[from], source) {
            return Err(fail(
                "Choose an ordinary source host in the selected supplied LAN.",
            ));
        }
        let row = |outcome, detail: &str| Probe {
            from,
            source,
            destination,
            outcome,
            detail: detail.into(),
        };
        let Some(c) = &self.applied else {
            return Ok(row(
                Outcome::Unavailable,
                "Apply the supported peer profiles before testing the modeled path.",
            ));
        };
        let outgoing = c[from].routes.iter().any(|n| n.contains(destination));
        if !self.lans[1 - from].contains(destination) {
            return Ok(if outgoing {
                row(
                    Outcome::SelectedTunnel,
                    "A configured route selects the tunnel, but this destination is outside the supplied remote LAN; no external connectivity is claimed.",
                )
            } else {
                row(
                    Outcome::OutsideTunnel,
                    "No configured route redirects this destination through the tunnel. Its external reachability is not modeled.",
                )
            });
        }
        if !usable(self.lans[1 - from], destination) {
            return Err(fail(
                "Use an ordinary remote LAN host; its network and broadcast addresses are not host endpoints in this exercise.",
            ));
        }
        if !core::peer_checks(&self.supplied, c)?
            .iter()
            .filter(|c| c.evaluated)
            .all(|c| c.passed)
        {
            return Ok(row(
                Outcome::Unavailable,
                "The evaluated peer prerequisites do not pass. Read their individual results before changing LAN routes.",
            ));
        }
        if !outgoing {
            return Ok(row(
                Outcome::MissingOutbound,
                "The source peer has no tunnel route covering this remote LAN destination.",
            ));
        }
        if !c[1 - from].routes.iter().any(|n| n.contains(source)) {
            return Ok(row(
                Outcome::MissingReturn,
                "The outward route exists, but the other peer has no tunnel route back to this source host.",
            ));
        }
        Ok(row(
            Outcome::Reachable,
            "The supplied peer prerequisites, outward route and return route pass in this finite model. No packets or TLS handshake were run.",
        ))
    }
}
fn usable(n: Network, a: Ipv4Addr) -> bool {
    n.interval()
        .is_ok_and(|(lo, hi)| lo < u32::from(a) && u32::from(a) < hi)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Unavailable,
    OutsideTunnel,
    SelectedTunnel,
    MissingOutbound,
    MissingReturn,
    Reachable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub from: usize,
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub outcome: Outcome,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub role: Role,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub subject: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub authority: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Network,32>")]
    pub destinations: Vec<Network>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub label: String,
    pub from: usize,
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub expected: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    pub initial: World,
    pub requirements: [Requirement; 2],
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,16>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}
fn check(rows: &mut Vec<Check>, label: impl Into<String>, passed: bool, detail: impl Into<String>) {
    rows.push(Check {
        label: label.into(),
        passed,
        detail: detail.into(),
    });
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.initial.parsed_profiles(&self.initial.saved)?;
        if self.version != 1
            || !self.initial.supplied.underlay_available
            || self.requirements[0].role == self.requirements[1].role
            || self.cases.is_empty()
            || self.cases.len() > 16
            || serde_json::to_vec(self)
                .map_err(|e| fail(e.to_string()))?
                .len()
                > 128 * 1024
        {
            return Err(fail(
                "Use a version 1 bounded two-peer repair with supplied underlay, complementary roles and real path cases.",
            ));
        }
        for (i, r) in self.requirements.iter().enumerate() {
            if r.destinations.as_slice() != [self.initial.lans[1 - i]] {
                return Err(fail(
                    "This version requires each entire remote LAN, with no unrelated destinations.",
                ));
            }
            if r.destinations.is_empty()||r.destinations.len()>32||!self.initial.supplied.files.iter().any(|f|matches!(&f.material,Material::Certificate{subject,role,..}if subject==&r.subject&&role==&r.role))||!self.initial.supplied.files.iter().any(|f|matches!(&f.material,Material::Ca{id}if id==&r.authority)){return Err(fail("Required peer identities/authorities must exist in the supplied roster, with explicit route destinations."));}
            for n in &r.destinations {
                n.validate()?;
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for c in &self.cases {
            if c.from < 2
                && ((c.expected == Outcome::Reachable
                    && !usable(self.initial.lans[1 - c.from], c.destination))
                    || (c.expected == Outcome::OutsideTunnel
                        && self.initial.lans[1 - c.from].contains(c.destination)))
            {
                return Err(fail(
                    "Reachability cases need real remote-LAN hosts; outside-tunnel cases must be outside that remote LAN.",
                ));
            }
            if c.id.is_empty()
                || c.id.len() > 64
                || !c
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 160
                || c.from > 1
                || !usable(self.initial.lans[c.from], c.source)
                || !matches!(c.expected, Outcome::Reachable | Outcome::OutsideTunnel)
            {
                return Err(fail(
                    "Use uniquely named supplied-LAN host cases with reachable or outside-tunnel outcomes.",
                ));
            }
        }
        for from in 0..2 {
            if !self
                .cases
                .iter()
                .any(|c| c.from == from && c.expected == Outcome::Reachable)
                || !self
                    .cases
                    .iter()
                    .any(|c| c.from == from && c.expected == Outcome::OutsideTunnel)
            {
                return Err(fail(
                    "Test required connectivity and unintended redirection from both sides.",
                ));
            }
        }
        let mut trial = self.initial.clone();
        if trial.apply().is_ok() && self.checks(&trial).iter().all(|c| c.passed) {
            return Err(fail(
                "Author a substantive configuration repair; merely applying an already-correct profile is not an assignment.",
            ));
        }
        Ok(())
    }
    pub fn checks(&self, w: &World) -> Vec<Check> {
        let mut rows = vec![];
        if self.version != 1 || self.cases.len() > 16 {
            check(
                &mut rows,
                "Task bounds",
                false,
                "Use the supported version and bounded case roster.",
            );
            return rows;
        }
        if let Err(e) = w.validate() {
            check(&mut rows, "Workspace integrity", false, e.message);
            return rows;
        }
        if w.supplied != self.initial.supplied || w.lans != self.initial.lans {
            check(
                &mut rows,
                "Supplied facts preserved",
                false,
                "Peer identities, outside-path facts and LANs are supplied evidence, not editable answers.",
            );
            return rows;
        }
        check(
            &mut rows,
            "Saved settings are applied",
            w.applied.is_some() && w.applied_source.as_ref() == Some(&w.saved),
            "Save and apply the current supported profiles; older applied settings do not prove the new saved work.",
        );
        let Some(c) = &w.applied else {
            return rows;
        };
        match core::peer_checks(&w.supplied, c) {
            Ok(checks) => {
                for r in checks {
                    check(&mut rows, r.label, r.evaluated && r.passed, r.detail);
                }
            }
            Err(e) => {
                check(&mut rows, "Peer settings", false, e.message);
                return rows;
            }
        }
        let assigned = match self.initial.parsed_profiles(&self.initial.saved) {
            Ok(configs) => configs,
            Err(e) => {
                check(&mut rows, "Assigned peer settings", false, e.message);
                return rows;
            }
        };
        for (i, own) in c.iter().enumerate() {
            check(
                &mut rows,
                format!(
                    "{}: assigned port and tunnel addresses",
                    w.supplied.peers[i].label
                ),
                own.port == assigned[i].port
                    && own.local_tunnel == assigned[i].local_tunnel
                    && own.remote_tunnel == assigned[i].remote_tunnel,
                "Keep this peer's supplied local port and local/remote tunnel addresses. A different mutually matching pair does not preserve the assigned configuration.",
            );
            let required = &self.requirements[i];
            let other = &self.requirements[1 - i];
            let label = &w.supplied.peers[i].label;
            check(
                &mut rows,
                format!("{label}: assigned TLS role"),
                own.role == required.role,
                "Use the assigned server/client responsibility; changing both roles does not preserve this task.",
            );
            check(&mut rows,format!("{label}: own identity"),w.supplied.files.iter().any(|f|f.path==own.cert&&matches!(&f.material,Material::Certificate{subject,..}if subject==&required.subject)),"Choose a supplied certificate for this peer's assigned identity, with its matching key.");
            check(
                &mut rows,
                format!("{label}: trusted authority"),
                w.supplied.files.iter().any(|f| {
                    f.path == own.ca
                        && matches!(&f.material,Material::Ca{id}if id==&required.authority)
                }),
                "Select the required supplied authority; a different authority or a filename alone does not verify the peer.",
            );
            check(
                &mut rows,
                format!("{label}: explicit peer verification"),
                own.expected_role == Some(other.role)
                    && own.expected_name.as_ref() == Some(&other.subject),
                "Request the other assigned peer's certificate purpose and exact name. Unconfigured checks are not verified identity.",
            );
            check(
                &mut rows,
                format!("{label}: full destination policy"),
                core::equivalent_routes(own, &required.destinations).unwrap_or(false),
                "Cover the required destination ranges without redirecting unrelated destinations. A sample host, a one-sided fix or an overly broad default is insufficient.",
            );
            check(
                &mut rows,
                format!("{label}: distinct tunnel addresses"),
                !w.lans
                    .iter()
                    .any(|n| n.contains(own.local_tunnel) || n.contains(own.remote_tunnel)),
                "Tunnel endpoint addresses must not overlap either supplied LAN.",
            );
        }
        for case in &self.cases {
            match w.probe(case.from, case.source, case.destination) {
                Ok(p) => check(
                    &mut rows,
                    &case.label,
                    p.outcome == case.expected,
                    format!(
                        "Expected {:?}; actual {:?}. {}",
                        case.expected, p.outcome, p.detail
                    ),
                ),
                Err(e) => check(&mut rows, &case.label, false, e.message),
            }
        }
        rows
    }
}
