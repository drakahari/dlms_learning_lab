//! Offline simulation, declarative content validation, catalog, and local persistence.
#![forbid(unsafe_code)]
pub mod print_queues;
pub mod print_queue_workspace;
pub mod dns_access;
pub mod host_direct_dns;
pub mod dns_query_tools;
pub mod dns_access_workspace;
pub mod source_install;
pub mod source_install_workspace;
pub mod maintenance_notice;
pub mod maintenance_notice_workspace;
pub mod bios_loader;
pub mod efi_loader;
pub mod efi_loader_workspace;
pub mod bios_loader_workspace;
pub mod boot_entry;
pub mod boot_entry_workspace;
pub mod grub_menu;
pub mod chrony;
pub mod chrony_workspace;
pub mod journal_archive;
pub mod json_inventory;
pub mod rest_controller;
pub mod rest_workspace;
pub mod json_inventory_workspace;
pub mod device_path;
pub mod device_path_workspace;
pub mod local_mail;
pub mod local_mail_workspace;
pub mod mailbox_config;
pub mod mailbox_access;
pub mod mailbox_protocol;
pub mod mailbox_integrity;
pub mod mailbox_workspace;
pub mod mailbox_source;
pub mod mailbox_admin;
pub mod smtp_config;
pub mod smtp_routing;
pub mod smtp_queue;
pub mod smtp_equivalence;
pub mod smtp_workspace;
pub mod smtp_console;
pub mod smtp_source;
pub mod sieve_lex;
pub mod sieve_parse;
pub mod sieve_address;
pub mod sieve_envelope;
pub mod sieve_delivery_policy;
pub mod sieve_vacation_history;
pub mod sieve_vacation_policy;
pub mod sieve_eval;
pub mod sieve_workspace;
pub mod sieve_source;
pub mod journal_archive_workspace;
pub mod grub_once_workspace;
pub mod grub_settings;
pub mod pam_control;
pub mod dhcp_snooping;
pub mod arp_inspection;
pub mod arp_inspection_workspace;
pub mod dhcp_snooping_workspace;
pub mod pam_workspace;
pub mod apache_static;
pub mod autofs;
pub mod autofs_workspace;
pub mod autofs_source;
pub mod apache_static_teaching;
pub mod apache_static_workspace;
pub mod catalog;
pub mod client_teaching;
pub mod cron;
pub mod cron_teaching;
pub mod cron_workspace;
pub mod debian_contents;
pub mod debian_source_cache;
pub mod debian_sources;
pub mod debian_sources_workspace;
pub mod debian_packages;
pub mod debian_packages_teaching;
pub mod debian_packages_terminal;
pub mod debian_packages_workspace;
pub mod engine;
pub mod filesystem;
pub mod samba_share_config;
pub mod samba_share;
pub mod samba_share_workspace;
pub mod directory_policy_workspace;
pub mod directory_policy_source;
pub mod native_observations;
pub mod nfs;
pub mod nfs_teaching;
pub mod nfs_workspace;
pub mod pack;
pub mod progress;

pub mod ipv4;
pub mod ipv6;
pub mod ipv6_teaching;
pub mod ipv6_workspace;
pub mod magic_subnet;

pub mod workspace;

pub mod shell;
pub mod environment_workspace;

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

/// Bounded virtual linear-LVM growth foundation; no curriculum integration.
pub mod lvm;
pub mod lvm_workspace;
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
pub mod accounts_template;
pub mod accounts_template_workspace;
pub mod command_location;
pub mod command_location_workspace;

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
pub mod nat_teaching;
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

pub mod rpm_packages_teaching;
pub mod rpm_packages_terminal;
pub mod rpm_packages_workspace;

/// Bounded L2 LACP eligibility; no app/curriculum integration.
pub mod etherchannel;

/// Bounded virtual archive/restore foundation; not yet bundled curriculum.
pub mod file_backup;

pub mod etherchannel_teaching;
pub mod etherchannel_workspace;
pub mod file_backup_teaching;
pub mod file_backup_terminal;
pub mod file_backup_workspace;

pub mod one_off;
pub mod one_off_teaching;
pub mod one_off_workspace;
pub mod physical_link;
pub mod physical_link_assignment;
pub mod physical_link_teaching;
pub mod physical_link_workspace;

/// Bounded single-upstream virtual NTP foundation; no curriculum integration.
pub mod ntp;
// Shared developer-QA manifest hashing; not a production cryptography API.
// Keep one module owner per crate instead of loading duplicate test modules.
// The lib-unit harness already owns the immutable historical reader's digest.
#[cfg(not(test))]
#[doc(hidden)]
#[path = "../examples/support/automation_digest.rs"]
pub mod developer_digest;
pub mod ntp_workspace;

/// Bounded settled gateway-failover model; authored integration is separate.
pub mod hsrp;
pub mod hsrp_workspace;
/// Bounded inode/link VFS; separate from the existing file model.
pub mod links;

/// Original bounded C-byte ERE engine; utilities remain simulated.
pub mod ere;

/// Bounded virtual GPT/ext4/swap preparation; no curriculum registration.
pub mod partition_fs;
pub mod partition_fs_workspace;

pub mod links_workspace;
/// Bounded virtual logging policy and source-bound native editors.
pub mod syslog;
pub mod syslog_workspace;

pub mod fs_capacity;
pub mod fs_integrity;
pub mod fs_integrity_terminal;
pub mod fs_integrity_workspace;
/// Bounded pure repository/cache and capacity foundations; no curriculum credit.
pub mod rpm_repositories;

pub mod capacity_workspace;
pub mod repository_workspace;

pub mod host_echo;
pub mod host_workspace;
pub mod linux_host;

pub mod linux_logging;

pub mod linux_logging_workspace;

pub mod qos;

pub mod qos_workspace;

/// Pure configuration foundation; no server workspace or curriculum credit yet.
pub mod dhcp_config;
pub mod dhcp_server_workspace;

pub mod nm_profiles;

pub mod nm_dhcp;
pub mod nm_policy;

pub mod nm_workspace;

/// Optional-media presence and dependency foundation; no new curriculum credit.
pub mod removable_mount;

pub mod removable_workspace;

pub mod client_names;
pub mod client_diagnostic;
pub mod client_names_workspace;
pub mod resolver_search;

/// Opt-in virtual caller authorization; no host mounts or account lookup.
pub mod user_mounts;

/// Source-bound actor/mount work reconstructed from bounded simulated actions.
pub mod user_mounts_workspace;

pub mod startup_workspace;

pub mod raid_assembly_source;
pub mod raid_assembly;
pub mod raid_assembly_workspace;
pub mod raid1;
pub mod raid1_workspace;

pub mod shared_libraries;
pub mod shared_libraries_workspace;

/// Bounded private indexed-discovery capability; no host utility execution.
pub mod file_index;

pub mod file_index_workspace;
pub mod file_index_source;

/// Bounded read-only explicit-category discovery; application integration pending.
pub mod whereis;

pub mod transport_core;
pub mod transport_workspace;

pub mod recursive_tree_workspace;

pub mod recursive_tree_source;

/// Bounded virtual module transitions; no host kernel operation.
pub mod kernel_modules;
pub mod kernel_module_workspace;
pub mod kernel_module_source;

pub mod vlan_host;

/// Optional bounded control profiles; never host networking.
pub mod bond_control;

/// Explicit control-snapshot to tagged-path composition.
pub mod bond_host;

/// Source-bound untagged acyclic bridge forwarding foundation.
pub mod bridge_fabric;

/// Source-bound bridge keyfiles and explicit current/future membership.
pub mod bridge_control;

/// Source-bound manual IPv4 and actual untagged bridge delivery.
pub mod bridge_host;

/// Non-secret aging metadata over supplied UTC dates; no host authentication.
pub mod aging;

/// Independent virtual aging assignments and dated behavioral checks.
pub mod aging_workspace;

/// Explicit configured destination overlay; old dynamic learning state is unchanged.
pub mod static_cam;

/// Source-bound secure access-port admission and actual virtual frame delivery.
pub mod port_security;

/// Separate secure-port practice, drafts and ordered behavioral assignments.
pub mod port_security_workspace;

/// Explicit VTY policy conditions only; no network or authenticated-session claim.
pub mod management_policy;

/// Source-held management endpoint paths; preserves the transit-only adapters.
pub mod management_path;

/// Complete policy documents using the existing bounded standard-ACL parser.
pub mod management_document;

/// Source-held multi-client, every-line behavioral management grading.
pub mod management_workspace;

pub mod local_access;

pub mod local_access_document;

pub mod local_access_workspace;

/// Bounded training-identity method order; no wire protocol or host execution.
pub mod aaa_methods;

/// Bounded explicit authentication-list configuration documents.
pub mod aaa_document;

/// Source-held ordered-authentication tasks and separate bounded drafts.
pub mod aaa_workspace;

/// Bounded read-only directory-query foundation; no curriculum integration yet.
pub mod ldap_query;
pub mod ldap_client;
pub mod ldap_workspace;

/// LDIF parsing foundation; no update execution or new curriculum credit.
pub mod ldap_ldif;
pub mod ldap_changes;
pub mod ldap_change_workspace;

pub mod ldap_utility;
pub mod ldap_identity;

/// Symbolic protected-channel foundation only; no TLS or new curriculum credit.
pub mod ldap_protection;

/// Protected-password drafts and behavior grading; catalog integration pending.
pub mod ldap_password_workspace;

/// Ordered static ACL decisions only; no server or curriculum integration yet.
pub mod ldap_access;

/// Declarative ACL task/session foundation; no catalog integration yet.
pub mod ldap_access_workspace;
pub mod ldap_configuration;
pub mod ldap_schema;

pub mod ldap_schema_workspace;

pub mod ldap_schema_query_workspace;

pub mod resource_measurements;

pub mod resource_observations;

pub mod resource_observation_workspace;

pub mod capacity_planning_workspace;

pub mod localization;
pub mod timezones;
pub mod timezone_workspace;

pub mod localization_workspace;

pub mod wireless_channels;

pub mod wireless_channel_workspace;

/// Supplied-profile compatibility only; WLAN curriculum integration is pending.
pub mod wlan_identity;

/// Declarative WLAN requirements and separate drafts; catalog integration pending.
pub mod wlan_workspace;

/// Supplied native-management and VLAN forwarding paths; no controller join or I/O.
pub mod wlan_paths;
pub mod wlan_paths_workspace;

/// Bounded virtual MIB queries; not a host-network implementation.
pub mod snmp;

/// Bounded monitoring tasks and separate practice/assignment drafts.
pub mod snmp_workspace;

/// Bounded proxy access-policy foundation; no registered lesson yet.
pub mod proxy_policy;

/// Separate editor/task foundation; no registered proxy lesson yet.
pub mod proxy_workspace;

pub mod http_cache;
pub mod http_cache_workspace;

pub mod reverse_proxy;

pub mod reverse_proxy_workspace;

pub mod service_names;

pub mod service_clients;

pub mod service_workspace;

pub mod ssh_identity;
pub mod ssh_policy;
pub mod ssh_key_access;
pub mod ssh_access;
pub mod ssh_workspace;

/// Text/undo foundation only; no registered modal-editor lesson or vi claim yet.
pub mod modal_text;
pub mod editor_handover;
pub mod editor_handover_source;
/// Initial bounded modal key controller; not registered curriculum or full vi.
pub mod modal_editing;

pub mod modal_workspace;

pub mod modal_source;

pub mod network_architecture;
pub mod network_architecture_source;
pub mod network_architecture_workspace;
pub mod network_transfer;
pub mod network_transfer_workspace;
pub mod network_transfer_source;

pub mod samba_share_source;

pub mod virtual_workloads;
pub mod virtual_routing;
pub mod virtual_isolation_workspace;
pub mod virtual_isolation_source;
pub mod guest_driver;
pub mod guest_driver_workspace;

pub mod guest_driver_source;

pub mod linux_filter;
pub mod linux_router_path;
pub mod linux_filter_regions;

pub mod linux_filter_policy;

pub mod linux_router_admin;

pub mod linux_router_workspace;

pub mod linux_router_source;

/// Bounded tuple foundation only; no registered connection/NAT lesson yet.
pub mod linux_connections;

/// Finite connection-state filter foundation; no new curriculum credit yet.
pub mod linux_state_filter;

/// Typed normal-flow translation foundation; no registered NAT lesson yet.
pub mod linux_flow_nat;

/// Bounded normal-flow path composition; no new registered lesson yet.
pub mod linux_connection_flow;

// Literal NAT command parser extends only the new typed NAT table.
mod linux_nat_commands;

/// Exact numeric translation properties; no new authored coverage yet.
pub mod linux_nat_policy;

/// Independent traffic-trial/draft foundation; curriculum registration pending.
pub mod linux_flow_workspace;

pub mod linux_flow_source;

pub mod linux_v6_regions;
pub mod linux_v6_filter;
pub mod linux_v6_policy;

pub mod linux_v6_connections;
pub mod linux_v6_state_filter;
pub mod linux_v6_path;
pub mod linux_v6_flow;
pub mod linux_v6_workspace;
pub mod linux_v6_source;

// Opt-in virtual service-name lookup for future router lessons.
pub mod linux_named_ports;
pub mod linux_service_router;
pub mod linux_service_router_source;

pub mod linux_v6_settings;
pub mod linux_v6_route_equivalence;

pub mod linux_v6_admin;

pub mod linux_v6_admin_workspace;

pub mod linux_v6_saved_settings;
pub mod linux_v6_saved_filter;
pub mod linux_v6_startup;
pub mod linux_v6_startup_task;
pub mod linux_v6_startup_workspace;
pub mod linux_v6_startup_source;

pub mod linux_v6_admin_source;

// Opt-in FTP identity/confinement capability; authored unit integration follows.
pub mod ftp_identity_config;
pub mod ftp_identity;
pub mod ftp_identity_task;
pub mod ftp_identity_workspace;

pub mod ftp_identity_source;

pub mod service_exposure;
pub mod service_exposure_task;
pub mod service_exposure_workspace;
pub mod service_exposure_source;

pub mod vpn_point;

pub mod vpn_point_task;

pub mod vpn_point_workspace;

pub mod vpn_point_source;

pub mod rpm_configuration;
pub mod rpm_patch_policy;

/// Bounded offline log-filter diagnosis; no host utility bridge.
pub mod log_filter;

pub mod auth_filter_task;

pub mod auth_filter_workspace;

pub mod auth_filter_source;

pub mod ids_rule;

pub mod ids_rule_task;

pub mod ids_rule_workspace;

pub mod ids_rule_source;

pub mod nc_probe;

pub mod telnet_client;

pub mod poe;

pub mod poe_task;
pub mod poe_workspace;

pub mod poe_source;

pub mod kernel_build;

pub mod kernel_stage;

pub mod kernel_commands;

pub mod kernel_workspace;

pub mod kernel_source;

pub mod dkms;
pub mod dkms_workspace;

pub mod dkms_source;

pub mod initramfs;

pub mod initramfs_workspace;

pub mod initramfs_source;

pub mod pxe;
pub mod iscsi;
pub mod ata_smart;
pub mod apache_https;
pub mod desktop;
pub mod security_investigation;
pub mod automation_workshop;
