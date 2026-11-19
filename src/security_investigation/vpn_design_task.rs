//! Save two editable tunnel policies and recompute fixed passage tests.
use super::{
    vpn_design::{Pair, Path, Profile, Purpose, Topology, Traffic},
    workspace::{Outcome, label},
};
use crate::ipv4_routing::Prefix;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub purpose: Purpose,
    pub endpoints: Pair,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub left_traffic: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub right_traffic: String,
}

impl Draft {
    pub fn from_profile(profile: Profile) -> Self {
        Self {
            purpose: profile.purpose,
            endpoints: profile.endpoints,
            left_traffic: profile.left_traffic.text(),
            right_traffic: profile.right_traffic.text(),
        }
    }

    pub fn validate_size(&self) -> Result<(), String> {
        if [self.left_traffic.as_str(), self.right_traffic.as_str()]
            .iter()
            .any(|text| text.len() > 32 || text.chars().any(char::is_control))
        {
            return Err("Keep each protected prefix to a single line up to 32 bytes.".into());
        }
        Ok(())
    }

    pub fn profile(&self) -> Result<Profile, String> {
        self.validate_size()?;
        let profile = Profile {
            purpose: self.purpose,
            endpoints: self.endpoints,
            left_traffic: Prefix::parse(self.left_traffic.trim())
                .map_err(|reason| format!("Left traffic range. {reason}"))?,
            right_traffic: Prefix::parse(self.right_traffic.trim())
                .map_err(|reason| format!("Right traffic range. {reason}"))?,
        };
        profile.validate()?;
        Ok(profile)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Local,
    SiteToSite,
    RemoteAccess,
    Outside,
}

impl Expected {
    fn matches(self, actual: Path) -> bool {
        matches!(
            (self, actual),
            (Self::Local, Path::Local)
                | (
                    Self::SiteToSite,
                    Path::Protected {
                        purpose: Purpose::SiteToSite,
                        ..
                    }
                )
                | (
                    Self::RemoteAccess,
                    Path::Protected {
                        purpose: Purpose::RemoteAccess,
                        ..
                    }
                )
                | (Self::Outside, Path::NoMatchingProfile)
        )
    }

    pub const fn describe(self) -> &'static str {
        match self {
            Self::Local => "Ordinary local path",
            Self::SiteToSite => "Protected through both site gateways",
            Self::RemoteAccess => "Protected through the remote client and office gateway",
            Self::Outside => "Outside both protected traffic ranges",
        }
    }
}

pub fn describe_path(actual: Path) -> String {
    match actual {
        Path::Local => "Ordinary local path within one LAN.".into(),
        Path::NoMatchingProfile => "Neither applied profile selects these inner addresses.".into(),
        Path::WrongEndpoints => "A profile selects this traffic, but its endpoints do not serve these addresses in the stated roles.".into(),
        Path::AmbiguousProtection => "Both profiles select this flow. Narrow their traffic ranges before applying a design.".into(),
        Path::Protected { purpose, from, to } => {
            let method = match purpose {
                Purpose::SiteToSite => "Site-to-site",
                Purpose::RemoteAccess => "Remote access",
            };
            format!("{method} protected path in this model. Outer addresses {} → {}.", from.outer_address, to.outer_address)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    #[serde(deserialize_with = "crate::cron::text::<_,72>")]
    pub name: String,
    pub traffic: Traffic,
    pub expected: Expected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VpnTask {
    pub topology: Topology,
    pub starter: [Profile; 2],
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Trial,10>")]
    pub trials: Vec<Trial>,
}

impl VpnTask {
    pub fn validate(&self) -> Result<(), String> {
        self.topology.validate()?;
        for profile in self.starter {
            profile.validate()?;
        }
        let expected = [
            Expected::Local,
            Expected::SiteToSite,
            Expected::RemoteAccess,
            Expected::Outside,
        ];
        let canonical = [
            self.topology.intended(Purpose::SiteToSite),
            self.topology.intended(Purpose::RemoteAccess),
        ];
        let mut names = BTreeSet::new();
        if !(5..=10).contains(&self.trials.len())
            || self.trials.iter().any(|trial| {
                !label(&trial.name, 72)
                    || !names.insert(&trial.name)
                    || !self
                        .topology
                        .trace(&canonical, trial.traffic)
                        .is_ok_and(|actual| trial.expected.matches(actual))
            })
            || !expected
                .iter()
                .all(|class| self.trials.iter().any(|trial| trial.expected == *class))
            || self
                .outcomes(&self.starter)
                .iter()
                .all(|outcome| outcome.passed)
        {
            return Err("Supply correct distinct local, protected and excluded trials, with a starter that needs repair.".into());
        }
        Ok(())
    }

    pub fn outcomes(&self, applied: &[Profile; 2]) -> Vec<Outcome> {
        let mut results = Vec::with_capacity(2 + self.trials.len());
        for (purpose, title, mismatch) in [
            (
                Purpose::SiteToSite,
                "Site-to-site coverage",
                "Use both site gateways and select their full supplied LANs.",
            ),
            (
                Purpose::RemoteAccess,
                "Remote-access coverage",
                "Use the remote client and office gateway. Limit the inner ranges to the device address and permitted service range.",
            ),
        ] {
            let expected = self.topology.intended(purpose);
            let matched = applied.iter().any(|policy| policy.same_policy(expected));
            results.push(Outcome::new(
                title,
                matched,
                if matched {
                    "The endpoint roles and protected traffic ranges match the supplied request."
                } else {
                    mismatch
                },
            ));
        }
        for trial in &self.trials {
            let result = self.topology.trace(applied, trial.traffic);
            let passed = result
                .as_ref()
                .is_ok_and(|actual| trial.expected.matches(*actual));
            let observed = result.map(describe_path).unwrap_or_else(|error| error);
            results.push(Outcome::new(
                &trial.name,
                passed,
                format!(
                    "Expected {}. Observed {observed}",
                    trial.expected.describe()
                ),
            ));
        }
        results
    }
}
