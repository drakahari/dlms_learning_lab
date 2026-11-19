//! Source-bound, untagged acyclic Linux bridge forwarding; never host networking.
use crate::{cron_workspace as bounded, switching as s};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, String>;
const BRIDGE: &str = "sim-bridge";
const LOCAL: &str = "sim-bridge-self";
const LOCAL_CABLE: &str = "sim-bridge-local";
fn device(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 15
        && !matches!(n, "." | "..")
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface(#[serde(deserialize_with = "crate::cron::text::<_,15>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub interface: Interface,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub endpoint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    pub interface: Interface,
    pub fabric: s::Fabric,
    #[serde(deserialize_with = "bounded::sequence::<_,Attachment,2>")]
    pub ports: Vec<Attachment>,
}
/// Explicit current state only. Being attached does not imply physical carrier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub up: bool,
    #[serde(deserialize_with = "bounded::sequence::<_,Interface,2>")]
    pub members: Vec<Interface>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint<'a> {
    Local,
    Peer(&'a str),
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        self.fabric.validate()?;
        if self.version != 1
            || !device(&self.interface.0)
            || self.ports.len() != 2
            || self.fabric.switches.len() > 2
            || self.fabric.hosts.len() > 8
            || self.fabric.hosts.len() < 3
            || self.fabric.cables.len() > 12
            || !bounded::within(self, 64 * 1024)
        {
            return Err("Use version1 bridge facts with one named bridge, two supplied NICs and a small untagged fabric".into());
        }
        for n in [BRIDGE, LOCAL, LOCAL_CABLE] {
            if self.fabric.switches.contains_key(n)
                || self.fabric.hosts.contains_key(n)
                || self.fabric.cables.iter().any(|c| c.id == n)
            {
                return Err("Supplied bridge facts use a reserved internal identifier".into());
            }
        }
        if self.fabric.switches.values().any(|sw| {
            sw.ports
                .values()
                .any(|p| !matches!(p.mode, s::Mode::Access { .. }))
        }) {
            return Err("This bridge foundation supports untagged Access attachments only; tagged/trunk forwarding is not modeled".into());
        }
        let mut names = BTreeSet::new();
        let mut ends = BTreeSet::new();
        for p in &self.ports {
            if !device(&p.interface.0)
                || p.interface == self.interface
                || !names.insert(&p.interface.0)
                || !ends.insert(&p.endpoint)
                || !self.fabric.hosts.contains_key(&p.endpoint)
            {
                return Err("Bridge NICs need distinct supplied names and leaf endpoints".into());
            }
            let jack = s::Jack::Host {
                host: p.endpoint.clone(),
            };
            let other = self.fabric.cables.iter().find_map(|c| {
                if c.a == jack {
                    Some(&c.b)
                } else if c.b == jack {
                    Some(&c.a)
                } else {
                    None
                }
            });
            if !matches!(other, Some(s::Jack::Switch { .. })) {
                return Err(
                    "Each supplied bridge NIC needs its own switch attachment, which may be down"
                        .into(),
                );
            }
        }
        // Even a currently down cable can later close a loop. Validate the full
        // potential topology, never treat a visited-set traversal as STP.
        let all = Current {
            up: true,
            members: self.ports.iter().map(|p| p.interface.clone()).collect(),
        };
        self.project_checked(&all)
            .validate()
            .map_err(|e| format!("Unsupported bridge topology: {e}"))
    }
    fn current(&self, c: &Current) -> Result<()> {
        let mut names = BTreeSet::new();
        if c.members.len() > 2
            || c.members.iter().any(|n| {
                !device(&n.0)
                    || !names.insert(&n.0)
                    || !self.ports.iter().any(|p| p.interface == *n)
            })
        {
            return Err("Attach each provided NIC at most once; foreign bridge member names are not supported".into());
        }
        Ok(())
    }
    /// Caller has validated all source identities and current membership.
    fn project_checked(&self, c: &Current) -> s::Fabric {
        let mut f = self.fabric.clone();
        let mut ports = BTreeMap::new();
        for (i, p) in self.ports.iter().enumerate() {
            let key = format!("port-{i}");
            let up = c.up && c.members.contains(&p.interface) && f.hosts[&p.endpoint].up;
            ports.insert(
                key.clone(),
                s::Port {
                    up,
                    mode: s::Mode::Access { vlan: 1 },
                },
            );
            let old = s::Jack::Host {
                host: p.endpoint.clone(),
            };
            let new = s::Jack::Switch {
                switch: BRIDGE.into(),
                port: key,
            };
            for cable in &mut f.cables {
                if cable.a == old {
                    cable.a = new.clone();
                }
                if cable.b == old {
                    cable.b = new.clone();
                }
            }
            f.hosts.remove(&p.endpoint);
        }
        ports.insert(
            "local".into(),
            s::Port {
                up: c.up,
                mode: s::Mode::Access { vlan: 1 },
            },
        );
        f.switches.insert(
            BRIDGE.into(),
            s::Switch {
                label: self.interface.0.clone(),
                vlans: BTreeSet::from([1]),
                ports,
            },
        );
        f.hosts.insert(
            LOCAL.into(),
            s::Host {
                label: format!("{} local endpoint", self.interface.0),
                up: c.up,
            },
        );
        f.cables.push(s::Cable {
            id: LOCAL_CABLE.into(),
            a: s::Jack::Host { host: LOCAL.into() },
            b: s::Jack::Switch {
                switch: BRIDGE.into(),
                port: "local".into(),
            },
            up: true,
        });
        f
    }
    pub fn project(&self, c: &Current) -> Result<s::Fabric> {
        self.validate()?;
        self.current(c)?;
        let f = self.project_checked(c);
        f.validate()?;
        Ok(f)
    }
    fn endpoint<'a>(&self, e: Endpoint<'a>) -> Result<&'a str> {
        match e {
            Endpoint::Local => Ok(LOCAL),
            Endpoint::Peer(n)
                if self.fabric.hosts.contains_key(n)
                    && !self.ports.iter().any(|p| p.endpoint == n) =>
            {
                Ok(n)
            }
            _ => Err(
                "Choose the local bridge endpoint or a provided peer, not a physical NIC endpoint"
                    .into(),
            ),
        }
    }
    pub fn broadcast(
        &self,
        c: &Current,
        from: Endpoint<'_>,
        tag: Option<u16>,
    ) -> Result<s::Broadcast> {
        if tag.is_some() {
            return Err("Tagged bridge input is outside this untagged foundation; no tag is silently stripped".into());
        }
        let f = self.project(c)?;
        f.broadcast(self.endpoint(from)?)
    }
    pub fn visibility(
        &self,
        c: &Current,
        from: Endpoint<'_>,
        to: Endpoint<'_>,
    ) -> Result<s::Visibility> {
        let f = self.project(c)?;
        let a = self.endpoint(from)?;
        let b = self.endpoint(to)?;
        if a == b {
            return Err("Choose two distinct bridge/peer endpoints".into());
        }
        Ok(s::Visibility {
            forward: f.broadcast(a)?.recipients.contains_key(b),
            reverse: f.broadcast(b)?.recipients.contains_key(a),
        })
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > 64 * 1024 {
            return Err("Bridge source facts exceed64KiB".into());
        }
        let f: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid bounded bridge facts: {e}"))?;
        f.validate()?;
        Ok(f)
    }
}
