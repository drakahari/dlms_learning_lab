//! IPsec endpoint and traffic selection exercises over fixed, offline facts.
//! A protected result describes the policy, not a negotiated tunnel or packet.
use crate::ipv4_routing::{Prefix, supported_address};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    BranchGateway,
    HeadquartersGateway,
    RemoteClient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub role: Role,
    pub outer_address: Ipv4Addr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    SiteToSite,
    RemoteAccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub left: Role,
    pub right: Role,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub purpose: Purpose,
    pub endpoints: Pair,
    pub left_traffic: Prefix,
    pub right_traffic: Prefix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    pub branch_gateway: Ipv4Addr,
    pub headquarters_gateway: Ipv4Addr,
    pub remote_client: Ipv4Addr,
    pub branch_lan: Prefix,
    pub headquarters_lan: Prefix,
    pub remote_inner: Ipv4Addr,
    pub remote_service: Prefix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Traffic {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Local,
    Protected {
        purpose: Purpose,
        from: Endpoint,
        to: Endpoint,
    },
    NoMatchingProfile,
    WrongEndpoints,
    AmbiguousProtection,
}

pub fn overlaps(a: Prefix, b: Prefix) -> bool {
    a.contains(b.address) || b.contains(a.address)
}

impl Profile {
    pub fn validate(self) -> Result<(), String> {
        if !self.left_traffic.valid()
            || !self.right_traffic.valid()
            || overlaps(self.left_traffic, self.right_traffic)
        {
            return Err("Use separate, aligned IPv4 ranges for the two protected sides.".into());
        }
        Ok(())
    }

    fn candidates(self, traffic: Traffic) -> Option<Pair> {
        if self.left_traffic.contains(traffic.source)
            && self.right_traffic.contains(traffic.destination)
        {
            Some(self.endpoints)
        } else if self.right_traffic.contains(traffic.source)
            && self.left_traffic.contains(traffic.destination)
        {
            Some(Pair {
                left: self.endpoints.right,
                right: self.endpoints.left,
            })
        } else {
            None
        }
    }

    fn swapped(self) -> Self {
        Self {
            endpoints: Pair {
                left: self.endpoints.right,
                right: self.endpoints.left,
            },
            left_traffic: self.right_traffic,
            right_traffic: self.left_traffic,
            ..self
        }
    }

    pub fn same_policy(self, expected: Self) -> bool {
        self == expected || self.swapped() == expected
    }
}

impl Topology {
    pub fn validate(self) -> Result<(), String> {
        let outer = [
            self.branch_gateway,
            self.headquarters_gateway,
            self.remote_client,
        ];
        if !outer.iter().all(Ipv4Addr::is_documentation)
            || outer[0] == outer[1]
            || outer[0] == outer[2]
            || outer[1] == outer[2]
        {
            return Err("Supply three distinct public-side documentation addresses.".into());
        }
        if ![self.branch_lan, self.headquarters_lan, self.remote_service]
            .iter()
            .all(|p| {
                p.valid()
                    && p.length >= 8
                    && p.address.is_private()
                    && p.last_address().is_some_and(|last| last.is_private())
            })
            || overlaps(self.branch_lan, self.headquarters_lan)
            || !self.headquarters_lan.contains(self.remote_service.address)
            || self.remote_service.length <= self.headquarters_lan.length
            || !self.remote_inner.is_private()
            || self.branch_lan.contains(self.remote_inner)
            || self.headquarters_lan.contains(self.remote_inner)
        {
            return Err("Supply separate private LANs, a narrower office service range, and a distinct remote-device address.".into());
        }
        Ok(())
    }

    fn endpoint(self, role: Role) -> Endpoint {
        let outer_address = match role {
            Role::BranchGateway => self.branch_gateway,
            Role::HeadquartersGateway => self.headquarters_gateway,
            Role::RemoteClient => self.remote_client,
        };
        Endpoint {
            role,
            outer_address,
        }
    }

    pub fn intended(self, purpose: Purpose) -> Profile {
        let (left, source, destination) = match purpose {
            Purpose::SiteToSite => (Role::BranchGateway, self.branch_lan, self.headquarters_lan),
            Purpose::RemoteAccess => (
                Role::RemoteClient,
                Prefix {
                    address: self.remote_inner,
                    length: 32,
                },
                self.remote_service,
            ),
        };
        Profile {
            purpose,
            endpoints: Pair {
                left,
                right: Role::HeadquartersGateway,
            },
            left_traffic: source,
            right_traffic: destination,
        }
    }

    fn served_by(self, role: Role, address: Ipv4Addr) -> bool {
        match role {
            Role::BranchGateway => self.branch_lan.contains(address),
            Role::HeadquartersGateway => self.headquarters_lan.contains(address),
            Role::RemoteClient => self.remote_inner == address,
        }
    }

    pub fn trace(self, profiles: &[Profile; 2], traffic: Traffic) -> Result<Path, String> {
        if !supported_address(traffic.source) || !supported_address(traffic.destination) {
            return Err("Choose the supplied training addresses for a virtual path trial.".into());
        }
        if [self.branch_lan, self.headquarters_lan]
            .iter()
            .any(|lan| lan.contains(traffic.source) && lan.contains(traffic.destination))
        {
            return Ok(Path::Local);
        }
        let mut matched = profiles
            .iter()
            .filter_map(|p| p.candidates(traffic).map(|pair| (p.purpose, pair)));
        let Some((purpose, pair)) = matched.next() else {
            return Ok(Path::NoMatchingProfile);
        };
        if matched.next().is_some() {
            return Ok(Path::AmbiguousProtection);
        }
        let expected_pair = self.intended(purpose).endpoints;
        let correct_roles = pair == expected_pair
            || (pair.left == expected_pair.right && pair.right == expected_pair.left);
        if !correct_roles
            || !self.served_by(pair.left, traffic.source)
            || !self.served_by(pair.right, traffic.destination)
        {
            return Ok(Path::WrongEndpoints);
        }
        Ok(Path::Protected {
            purpose,
            from: self.endpoint(pair.left),
            to: self.endpoint(pair.right),
        })
    }
}
