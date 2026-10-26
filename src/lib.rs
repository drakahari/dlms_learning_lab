//! Offline simulation, declarative content validation, catalog, and local persistence.
#![forbid(unsafe_code)]
pub mod apache_static;
pub mod apache_static_teaching;
pub mod apache_static_workspace;
pub mod catalog;
pub mod client_teaching;
pub mod debian_packages;
pub mod debian_packages_teaching;
pub mod debian_packages_terminal;
pub mod debian_packages_workspace;
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

pub mod storage;

pub mod storage_workspace;

pub mod storage_teaching;

pub mod accounts;
pub mod acl;
pub mod acl_editor;
pub mod acl_routing;
pub mod acl_teaching;
pub mod acl_workspace;

pub mod accounts_teaching;

mod local_file;

pub mod ospf;
pub mod unit_manager;

pub mod ospf_teaching;
pub mod ospf_workspace;
pub mod unit_manager_teaching;
pub mod unit_manager_terminal;
pub mod unit_manager_workspace;

/// Bounded Ethernet source learning; not yet bundled curriculum.
pub mod mac_learning;
pub mod mac_learning_teaching;
pub mod mac_learning_workspace;
pub mod nat;
pub mod nat_workspace;

pub mod seed_dns_teaching;
pub mod seed_mount_teaching;

pub mod legacy_access_teaching;
pub mod legacy_logs_teaching;

pub mod dhcpv4;

pub mod dhcpv4_teaching;
pub mod dhcpv4_workspace;

pub mod spanning_tree;

pub mod spanning_tree_teaching;
pub mod spanning_tree_workspace;

#[cfg(test)]
#[path = "../tests/support/actual9_evidence.rs"]
pub(crate) mod actual9_test_evidence;

/// Restricted RPM EVR/provider observations; no CLI or curriculum.
pub mod rpm_packages;
