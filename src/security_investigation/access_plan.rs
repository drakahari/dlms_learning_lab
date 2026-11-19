//! A supplied facility access table. No real readers, credentials or hardware.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    NetworkOperator,
    FacilitiesTechnician,
    RegisteredVisitor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    Staffed,
    AfterHours,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Denied,
    StaffedOnly,
    AnyWindow,
    StaffedWithEscort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reports {
    FacilityDesk,
    SecurityDesk,
    BothDesks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub operator: Access,
    pub technician: Access,
    pub visitor: Access,
    pub record_denials: bool,
    pub suspicious_contact_reports: Reports,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    pub role: Role,
    pub window: Window,
    pub approved_identity: bool,
    pub escort_present: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Admitted,
    UnrecognizedIdentity,
    NoRoleAccess,
    OutsideWindow,
    EscortNeeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub access: Decision,
    pub recorded: bool,
}

impl Policy {
    pub const fn entry(self, attempt: Trial) -> Entry {
        let rule = match attempt.role {
            Role::NetworkOperator => self.operator,
            Role::FacilitiesTechnician => self.technician,
            Role::RegisteredVisitor => self.visitor,
        };
        let access = if !attempt.approved_identity {
            Decision::UnrecognizedIdentity
        } else {
            match (rule, attempt.window, attempt.escort_present) {
                (Access::Denied, _, _) => Decision::NoRoleAccess,
                (Access::StaffedOnly | Access::StaffedWithEscort, Window::AfterHours, _) => {
                    Decision::OutsideWindow
                }
                (Access::StaffedWithEscort, Window::Staffed, false) => Decision::EscortNeeded,
                _ => Decision::Admitted,
            }
        };
        Entry {
            access,
            recorded: matches!(access, Decision::Admitted) || self.record_denials,
        }
    }

    pub const fn report_reaches_security(self) -> bool {
        matches!(
            self.suspicious_contact_reports,
            Reports::SecurityDesk | Reports::BothDesks
        )
    }
}

impl Decision {
    pub const fn detail(self) -> &'static str {
        match self {
            Self::Admitted => "Allowed by the supplied access plan.",
            Self::UnrecognizedIdentity => "Denied. The supplied visit or identity is unverified.",
            Self::NoRoleAccess => "Denied. This role has no permission for the protected room.",
            Self::OutsideWindow => "Denied. This role may enter only during the staffed window.",
            Self::EscortNeeded => "Denied. The allowed visit needs a present escort.",
        }
    }
}
