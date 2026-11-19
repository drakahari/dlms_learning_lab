//! Defensive investigations over a supplied virtual workplace.
type Result<T> = std::result::Result<T, String>;

pub mod access_plan;
pub mod change_evidence;
pub mod session;
pub mod source;
pub mod workspace;
pub mod credential_task;
pub mod credentials;
pub mod vpn_design;
pub mod vpn_design_task;
