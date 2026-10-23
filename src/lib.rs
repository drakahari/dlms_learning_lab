//! Offline simulation, declarative content validation, catalog, and local persistence.
#![forbid(unsafe_code)]
pub mod catalog;
pub mod engine;
pub mod filesystem;
pub mod pack;
pub mod progress;

pub mod ipv4;
pub mod ipv6;
pub mod ipv6_teaching;
pub mod ipv6_workspace;

pub mod workspace;

pub mod shell;

pub mod terminal_teaching;
pub mod theme;

pub mod script_teaching;

pub mod backup;

pub mod course;

/// Public build identity; contains no profile or machine paths.
pub fn build_identity() -> String {
    format!(
        "DLMS Learning Lab {} ({})",
        env!("CARGO_PKG_VERSION"),
        option_env!("DLMS_BUILD_REVISION").unwrap_or("development")
    )
}

pub mod switching;

pub mod switching_teaching;
pub mod switching_workspace;

pub mod ipv4_routing;

pub mod ipv4_routing_teaching;
pub mod ipv4_routing_workspace;

pub mod address_teaching;

pub mod service_teaching;

pub mod dns;
pub mod dns_teaching;
pub mod dns_workspace;

pub mod processes;

pub mod process_teaching;

pub mod discovery;

pub mod discovery_teaching;
pub mod discovery_workspace;
