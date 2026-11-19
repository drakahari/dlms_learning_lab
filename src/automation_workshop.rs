//! Bounded staged changes and controller policy over supplied virtual networks.
type Result<T> = std::result::Result<T, String>;

pub mod change_rollout;
pub mod intent_path;
pub mod rollout_task;
pub mod session;
pub mod source;
pub mod workspace;
pub mod claim_review;
pub mod desired_config;
pub mod plan_editor;
pub mod site_connectivity;
pub mod site_connectivity_task;
pub mod socket_retirement;
pub mod socket_terminal;
pub mod retirement_console;
pub mod authenticated_transfer;
pub mod transfer_editor;
pub mod access_defaults;
pub mod svi_draft;
pub mod svi_projection;
pub mod nfs_tools;
pub mod route_reading;
pub mod ospf_broadcast;
