use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::Ipv4Addr};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub mode: u16,
    pub owner: String,
    pub group: String,
    pub directory: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Allocation {
    pub name: String,
    pub hosts: u32,
    pub cidr: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ServiceEvent {
    Rejected { port: String },
    Started { port: u16 },
    Stopped,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum State {
    LinuxRouter { simulation: Box<crate::linux_router_workspace::Session> },
    LinuxV6 { simulation: Box<crate::linux_v6_workspace::Session> },
    LinuxServiceRouter { simulation: Box<crate::linux_service_router::Session> },
    LinuxV6Administration { simulation: Box<crate::linux_v6_admin_workspace::Session> },
    LinuxV6Startup { simulation: Box<crate::linux_v6_startup_workspace::Session> },
    IdsRule { simulation: Box<crate::ids_rule_workspace::Session> },
    AuthFilter { simulation: Box<crate::auth_filter_workspace::Session> },
    VpnPoint { simulation: Box<crate::vpn_point_workspace::Session> },
    Poe { simulation: Box<crate::poe_workspace::Session> },
    KernelBuild { simulation: Box<crate::kernel_workspace::Session> },
    Dkms { simulation: Box<crate::dkms_workspace::Session> },
    Initramfs { simulation: Box<crate::initramfs_workspace::Session> },
    Pxe { simulation: Box<crate::pxe::session::Session> },
    Iscsi { simulation: Box<crate::iscsi::session::Session> },
    AtaSmart { simulation: Box<crate::ata_smart::session::Session> },
    ApacheHttps { simulation: Box<crate::apache_https::session::Session> },
    Desktop { simulation: Box<crate::desktop::session::Session> },
    SecurityInvestigation { simulation: Box<crate::security_investigation::session::Session> },
    AutomationWorkshop { simulation: Box<crate::automation_workshop::session::Session> },
    ServiceExposure { simulation: Box<crate::service_exposure_workspace::Session> },
    FtpIdentity { simulation: Box<crate::ftp_identity_workspace::Session> },
    LinuxFlow { simulation: Box<crate::linux_flow_workspace::Session> },
    SieveFiltering { simulation: Box<crate::sieve_workspace::Session> },
    MailboxAccess { simulation: Box<crate::mailbox_workspace::Session> },
    MailboxAdministration { simulation: Box<crate::mailbox_workspace::Session> },
    SmtpRouting { simulation: Box<crate::smtp_workspace::Session> },
    VirtualIsolation { simulation: Box<crate::virtual_isolation_workspace::Session> },
    GuestDrivers { simulation: Box<crate::guest_driver_workspace::Session> },
    SambaShare { simulation: Box<crate::samba_share_workspace::Session> },
    CampusArchitecture { simulation: Box<crate::network_architecture_workspace::Session> },
    NetworkTransfer { simulation: Box<crate::network_transfer_workspace::Session> },
    EditorHandover { simulation: Box<crate::editor_handover::Session> },
    ModalEditing { simulation: Box<crate::modal_workspace::Session> },
    SshAccess { simulation: Box<crate::ssh_workspace::Session> },
    MaintenanceNotice { simulation: Box<crate::maintenance_notice_workspace::Session> },
    DnsAccess { simulation: Box<crate::dns_access_workspace::Session> },
    PrintQueues { simulation: Box<crate::print_queue_workspace::Session> },
    Localization { simulation: Box<crate::localization_workspace::Session> },
    WirelessChannels { simulation: Box<crate::wireless_channel_workspace::Session> },
    WlanProfiles { simulation: Box<crate::wlan_workspace::Session> },
    HttpCache { simulation: Box<crate::http_cache_workspace::Session> },
    ServiceClients { simulation: Box<crate::service_workspace::Session> },
    ReverseProxy { simulation: Box<crate::reverse_proxy_workspace::Session> },
    ProxyPolicy { simulation: Box<crate::proxy_workspace::Session> },
    SnmpMonitor { simulation: Box<crate::snmp_workspace::Session> },
    WlanPaths { simulation: Box<crate::wlan_paths_workspace::Session> },
    Timezone { simulation: Box<crate::timezone_workspace::Session> },
    SourceInstall { simulation: Box<crate::source_install_workspace::Session> },
    CapacityPlanning { simulation: Box<crate::capacity_planning_workspace::Session> },
    ResourceObservations { simulation: Box<crate::resource_observation_workspace::Session> },
    CopiedJournal { simulation: Box<crate::journal_archive_workspace::Session> },
    JsonInventory { simulation: Box<crate::json_inventory_workspace::Session> },
    RestController { simulation: Box<crate::rest_workspace::Session> },
    DevicePath { simulation: Box<crate::device_path_workspace::Session> },
    LocalMail { simulation: Box<crate::local_mail_workspace::Session> },
    Chrony { simulation: Box<crate::chrony_workspace::Session> },
    GrubOnce { simulation: Box<crate::grub_once_workspace::Session> },
    EfiLoader { simulation: Box<crate::efi_loader_workspace::Session> },
    BiosLoader { simulation: Box<crate::bios_loader_workspace::Session> },
    BootEntry { simulation: Box<crate::boot_entry_workspace::Session> },
    RaidAssembly { simulation: Box<crate::raid_assembly_workspace::Session> },
    Autofs { simulation: Box<crate::autofs_workspace::Session> },
    KernelModules {
        simulation: Box<crate::kernel_module_workspace::Session>,
    },
    RecursiveTree {
        simulation: Box<crate::recursive_tree_workspace::Session>,
    },
    Transport {
        simulation: Box<crate::transport_workspace::Session>,
    },
    FileIndex {
        simulation: Box<crate::file_index_workspace::Session>,
    },
    Raid1 {
        simulation: Box<crate::raid1_workspace::Session>,
    },
    SharedLibraries {
        simulation: Box<crate::shared_libraries_workspace::Session>,
    },
    StartupFiles {
        simulation: Box<crate::startup_workspace::Session>,
    },
    FsIntegrity {
        simulation: Box<crate::fs_integrity_workspace::Session>,
    },
    UserMounts {
        simulation: Box<crate::user_mounts_workspace::Session>,
    },
    Environment {
        simulation: Box<crate::environment_workspace::Session>,
    },
    DebianSources {
        simulation: Box<crate::debian_sources_workspace::Session>,
    },
    ClientNames {
        simulation: Box<crate::client_names_workspace::Session>,
    },
    RemovableMounts {
        simulation: Box<crate::removable_workspace::Session>,
    },
    NmProfiles {
        simulation: Box<crate::nm_workspace::Session>,
    },
    PamStack {
        simulation: Box<crate::pam_workspace::Session>,
    },
    Aging {
        simulation: Box<crate::aging_workspace::Session>,
    },
    PortSecurity {
        simulation: Box<crate::port_security_workspace::Session>,
    },
    DhcpSnooping {
        simulation: Box<crate::dhcp_snooping_workspace::Session>,
    },
    ArpInspection {
        simulation: Box<crate::arp_inspection_workspace::Session>,
    },
    ManagementPolicy {
        simulation: Box<crate::management_workspace::Session>,
    },
    LocalAccess {
        simulation: Box<crate::local_access_workspace::Session>,
    },
    AaaAccess {
        simulation: Box<crate::aaa_workspace::Session>,
    },
    LdapClient {
        simulation: Box<crate::ldap_workspace::Session>,
    },
    LdapAccess {
        simulation: Box<crate::ldap_access_workspace::Session>,
    },
    LdapSchema {
        simulation: Box<crate::ldap_schema_workspace::Session>,
    },
    LdapSchemaQuery {
        simulation: Box<crate::ldap_schema_query_workspace::Session>,
    },
    LdapPassword {
        simulation: Box<crate::ldap_password_workspace::Session>,
    },
    LdapChanges {
        simulation: Box<crate::ldap_change_workspace::Session>,
    },
    DhcpConfig {
        simulation: Box<crate::dhcp_server_workspace::Session>,
    },
    Qos {
        simulation: Box<crate::qos_workspace::Session>,
    },
    LinuxLogging {
        simulation: Box<crate::linux_logging_workspace::Session>,
    },
    HostNetwork {
        simulation: Box<crate::host_workspace::Session>,
    },
    Repositories {
        simulation: Box<crate::repository_workspace::Session>,
    },
    Capacity {
        simulation: Box<crate::capacity_workspace::Session>,
    },
    DirectoryPolicy {
        simulation: Box<crate::directory_policy_workspace::Session>,
    },
    Links {
        simulation: Box<crate::links_workspace::Session>,
    },
    MagicSubnet {
        session: Box<crate::magic_subnet::Session>,
    },
    PartitionFs {
        simulation: Box<crate::partition_fs_workspace::Session>,
    },
    Hsrp {
        simulation: Box<crate::hsrp_workspace::Session>,
    },
    Syslog {
        simulation: Box<crate::syslog_workspace::Session>,
    },
    Ntp {
        simulation: Box<crate::ntp_workspace::Session>,
    },
    Lvm {
        simulation: Box<crate::lvm_workspace::Session>,
    },
    PhysicalLink {
        simulation: crate::physical_link::State,
    },
    OneOff {
        simulation: Box<crate::one_off_workspace::Assignment>,
    },
    FileBackup {
        simulation: crate::file_backup_terminal::Store,
    },
    EtherChannel {
        simulation: crate::etherchannel::State,
    },
    Cron {
        simulation: crate::cron::State,
    },
    Nfs {
        simulation: crate::nfs::State,
    },
    RpmPackages {
        simulation: crate::rpm_packages::State,
    },
    BasicNat {
        simulation: crate::nat::State,
    },
    ApacheStatic {
        simulation: crate::apache_static::State,
    },
    DebianPackages {
        simulation: crate::debian_packages::State,
    },
    SpanningTree {
        simulation: crate::spanning_tree::State,
    },
    DhcpV4 {
        simulation: crate::dhcpv4::State,
    },
    MacLearning {
        simulation: crate::mac_learning::State,
    },
    UnitManager {
        simulation: crate::unit_manager::State,
    },
    Ospf {
        simulation: crate::ospf::State,
    },
    Ipv4Acl {
        network: crate::acl_routing::Network,
    },
    Accounts {
        simulation: crate::accounts::State,
    },
    PersistentMounts {
        storage: crate::storage::State,
    },
    Discovery {
        simulation: crate::discovery::State,
    },
    Processes {
        table: crate::processes::Table,
    },
    AuthoritativeDns {
        server: crate::dns::Server,
    },
    Switching {
        fabric: crate::switching::Fabric,
    },
    Ipv6 {
        topology: crate::ipv6::Topology,
    },
    Ipv4Routing {
        topology: crate::ipv4_routing::Topology,
    },
    Filesystem {
        tree: crate::filesystem::Tree,
    },
    Script {
        source: String,
        #[serde(default)]
        run: Option<crate::shell::Record>,
    },
    Subnet {
        ip: Ipv4Addr,
        prefix: u8,
        answers: [String; 4],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        second: Option<String>,
    },
    Vlsm {
        base: Ipv4Addr,
        prefix: u8,
        allocations: Vec<Allocation>,
    },
    Network {
        address: Ipv4Addr,
        prefix: u8,
        gateway: Ipv4Addr,
        router: Ipv4Addr,
        lan_prefix: u8,
        target: Ipv4Addr,
    },
    Unix {
        nodes: BTreeMap<String, Node>,
        users: BTreeMap<String, Vec<String>>,
    },
    Logs {
        files: BTreeMap<String, String>,
        result: String,
    },
    Service {
        port: String,
        running: bool,
        enabled: bool,
        loaded_port: Option<u16>,
        #[serde(default)]
        failed: bool,
        #[serde(default)]
        journal: Vec<ServiceEvent>,
    },
    Dns {
        records: BTreeMap<String, String>,
        loaded: BTreeMap<String, String>,
        serial: u32,
        loaded_serial: u32,
    },
    Mount {
        capacity: u64,
        used: u64,
        cache: u64,
        mounted: bool,
        device: String,
        configured_device: String,
        writable: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Goal {
    LinuxRouter { definition: Box<crate::linux_router_workspace::Definition> },
    LinuxV6 { definition: Box<crate::linux_v6_workspace::Definition> },
    LinuxServiceRouter { definition: Box<crate::linux_service_router::Definition> },
    LinuxV6Administration { definition: Box<crate::linux_v6_admin_workspace::Definition> },
    LinuxV6Startup { definition: Box<crate::linux_v6_startup_workspace::Definition> },
    IdsRule { definition: Box<crate::ids_rule_workspace::Definition> },
    AuthFilter { definition: Box<crate::auth_filter_workspace::Definition> },
    VpnPoint { definition: Box<crate::vpn_point_workspace::Definition> },
    Poe { definition: Box<crate::poe_workspace::Definition> },
    KernelBuild { definition: Box<crate::kernel_workspace::Definition> },
    Dkms { definition: Box<crate::dkms_workspace::Definition> },
    Initramfs { definition: Box<crate::initramfs_workspace::Definition> },
    Pxe { definition: Box<crate::pxe::session::Definition> },
    Iscsi { definition: Box<crate::iscsi::session::Definition> },
    AtaSmart { definition: Box<crate::ata_smart::session::Definition> },
    ApacheHttps { definition: Box<crate::apache_https::session::Definition> },
    Desktop { definition: Box<crate::desktop::session::Definition> },
    SecurityInvestigation { definition: Box<crate::security_investigation::session::Definition> },
    AutomationWorkshop { definition: Box<crate::automation_workshop::session::Definition> },
    ServiceExposure { definition: Box<crate::service_exposure_workspace::Definition> },
    FtpIdentity { definition: Box<crate::ftp_identity_workspace::Definition> },
    LinuxFlow { definition: Box<crate::linux_flow_workspace::Definition> },
    SieveFiltering { definition: Box<crate::sieve_workspace::Definition> },
    MailboxAccess { definition: Box<crate::mailbox_workspace::Definition> },
    MailboxAdministration { definition: Box<crate::mailbox_workspace::Definition> },
    SmtpRouting { definition: Box<crate::smtp_workspace::Definition> },
    VirtualIsolation { definition: Box<crate::virtual_isolation_workspace::Definition> },
    GuestDrivers { definition: Box<crate::guest_driver_workspace::Definition> },
    SambaShare { definition: Box<crate::samba_share_workspace::Definition> },
    CampusArchitecture { definition: Box<crate::network_architecture_workspace::Definition> },
    NetworkTransfer { definition: Box<crate::network_transfer_workspace::Definition> },
    EditorHandover { definition: Box<crate::editor_handover::Definition> },
    ModalEditing { definition: Box<crate::modal_workspace::Definition> },
    SshAccess { definition: Box<crate::ssh_workspace::Definition> },
    MaintenanceNotice { definition: Box<crate::maintenance_notice_workspace::Definition> },
    DnsAccess { definition: Box<crate::dns_access_workspace::Definition> },
    PrintQueues { definition: Box<crate::print_queue_workspace::Definition> },
    Localization { definition: Box<crate::localization_workspace::Definition> },
    WirelessChannels { definition: Box<crate::wireless_channel_workspace::Definition> },
    WlanProfiles { definition: Box<crate::wlan_workspace::Definition> },
    HttpCache { definition: Box<crate::http_cache_workspace::Definition> },
    ServiceClients { definition: Box<crate::service_workspace::Definition> },
    ReverseProxy { definition: Box<crate::reverse_proxy_workspace::Definition> },
    ProxyPolicy { definition: Box<crate::proxy_workspace::Definition> },
    SnmpMonitor { definition: Box<crate::snmp_workspace::Definition> },
    WlanPaths { definition: Box<crate::wlan_paths_workspace::Definition> },
    Timezone { definition: Box<crate::timezone_workspace::Definition> },
    SourceInstall { definition: Box<crate::source_install_workspace::Definition> },
    CapacityPlanning { definition: Box<crate::capacity_planning_workspace::Definition> },
    ResourceObservations { definition: Box<crate::resource_observation_workspace::Definition> },
    CopiedJournal { definition: Box<crate::journal_archive_workspace::Definition> },
    JsonInventory { definition: Box<crate::json_inventory_workspace::Definition> },
    RestController { definition: Box<crate::rest_workspace::Definition> },
    DevicePath { definition: Box<crate::device_path_workspace::Definition> },
    LocalMail { definition: Box<crate::local_mail_workspace::Definition> },
    Chrony { definition: Box<crate::chrony_workspace::Definition> },
    GrubOnce { definition: Box<crate::grub_once_workspace::Definition> },
    EfiLoader { definition: Box<crate::efi_loader_workspace::Definition> },
    BiosLoader { definition: Box<crate::bios_loader_workspace::Definition> },
    BootEntry { definition: Box<crate::boot_entry_workspace::Definition> },
    RaidAssembly { definition: Box<crate::raid_assembly_workspace::Definition> },
    Autofs { definition: Box<crate::autofs_workspace::Definition> },
    KernelModules {
        definition: Box<crate::kernel_module_workspace::Definition>,
    },
    RecursiveTree {
        definition: Box<crate::recursive_tree_workspace::Definition>,
    },
    Transport {
        definition: Box<crate::transport_workspace::Definition>,
    },
    FileIndex {
        definition: Box<crate::file_index_workspace::Definition>,
    },
    Raid1 {
        definition: Box<crate::raid1_workspace::Definition>,
    },
    SharedLibraries {
        definition: Box<crate::shared_libraries_workspace::Definition>,
    },
    StartupFiles {
        definition: Box<crate::startup_workspace::Definition>,
    },
    FsIntegrity {
        definition: Box<crate::fs_integrity_workspace::Definition>,
    },
    UserMounts {
        definition: Box<crate::user_mounts_workspace::Definition>,
    },
    Environment {
        definition: Box<crate::environment_workspace::Definition>,
    },
    DebianSources {
        definition: Box<crate::debian_sources_workspace::Definition>,
    },
    ClientNames {
        definition: Box<crate::client_names_workspace::Definition>,
    },
    RemovableMounts {
        definition: Box<crate::removable_workspace::Definition>,
    },
    NmProfiles {
        definition: Box<crate::nm_workspace::Definition>,
    },
    PamStack {
        definition: Box<crate::pam_workspace::Definition>,
    },
    Aging {
        definition: Box<crate::aging_workspace::Definition>,
    },
    PortSecurity {
        definition: Box<crate::port_security_workspace::Definition>,
    },
    DhcpSnooping {
        definition: Box<crate::dhcp_snooping_workspace::Definition>,
    },
    ArpInspection {
        definition: Box<crate::arp_inspection_workspace::Definition>,
    },
    ManagementPolicy {
        definition: Box<crate::management_workspace::Definition>,
    },
    LocalAccess {
        definition: Box<crate::local_access_workspace::Definition>,
    },
    AaaAccess {
        definition: Box<crate::aaa_workspace::Definition>,
    },
    LdapClient {
        definition: Box<crate::ldap_workspace::Definition>,
    },
    LdapAccess {
        definition: Box<crate::ldap_access_workspace::Definition>,
    },
    LdapSchema {
        definition: Box<crate::ldap_schema_workspace::Definition>,
    },
    LdapSchemaQuery {
        definition: Box<crate::ldap_schema_query_workspace::Definition>,
    },
    LdapPassword {
        definition: Box<crate::ldap_password_workspace::Definition>,
    },
    LdapChanges {
        definition: Box<crate::ldap_change_workspace::Definition>,
    },
    DhcpConfig {
        definition: Box<crate::dhcp_server_workspace::Definition>,
    },
    Qos {
        definition: Box<crate::qos_workspace::Definition>,
    },
    LinuxLogging {
        definition: Box<crate::linux_logging_workspace::Definition>,
    },
    HostNetwork {
        definition: Box<crate::host_workspace::Definition>,
    },
    Repositories {
        definition: Box<crate::repository_workspace::Definition>,
    },
    Capacity {
        definition: Box<crate::capacity_workspace::Definition>,
    },
    DirectoryPolicy {
        definition: Box<crate::directory_policy_workspace::Definition>,
    },
    Links {
        definition: Box<crate::links_workspace::Definition>,
    },
    MagicSubnet {
        problem: crate::magic_subnet::Problem,
        stage: crate::magic_subnet::Stage,
    },
    PartitionFs {
        definition: Box<crate::partition_fs_workspace::Definition>,
    },
    Hsrp {
        definition: Box<crate::hsrp_workspace::Definition>,
    },
    Syslog {
        definition: Box<crate::syslog_workspace::Definition>,
    },
    Ntp {
        definition: Box<crate::ntp_workspace::Definition>,
    },
    Lvm {
        definition: Box<crate::lvm_workspace::Definition>,
    },
    PhysicalLink {
        definition: Box<crate::physical_link_assignment::Definition>,
    },
    OneOff {
        definition: Box<crate::one_off_workspace::Definition>,
    },
    FileBackup {
        definition: Box<crate::file_backup_workspace::Definition>,
    },
    EtherChannel {
        definition: Box<crate::etherchannel_workspace::Definition>,
    },
    Cron {
        definition: Box<crate::cron_workspace::Definition>,
    },
    Nfs {
        definition: Box<crate::nfs_workspace::Definition>,
    },
    RpmPackages {
        definition: Box<crate::rpm_packages_workspace::Definition>,
    },
    BasicNat {
        definition: Box<crate::nat_workspace::Definition>,
    },
    ApacheStatic {
        definition: Box<crate::apache_static_workspace::Definition>,
    },
    DebianPackages {
        definition: Box<crate::debian_packages_workspace::Definition>,
    },
    SpanningTree {
        initial: crate::spanning_tree::State,
        #[serde(deserialize_with = "crate::spanning_tree_workspace::requirements")]
        requirements: Vec<crate::spanning_tree_workspace::Requirement>,
    },
    DhcpV4 {
        initial: crate::dhcpv4::State,
        #[serde(deserialize_with = "crate::dhcpv4_workspace::requirements")]
        requirements: Vec<crate::dhcpv4_workspace::Requirement>,
        #[serde(deserialize_with = "crate::dhcpv4_workspace::probes")]
        probes: Vec<crate::dhcpv4_workspace::Probe>,
    },
    MacLearning {
        initial: crate::mac_learning::State,
        #[serde(deserialize_with = "crate::mac_learning_workspace::requirements")]
        requirements: Vec<crate::mac_learning_workspace::Requirement>,
    },
    UnitManager {
        definition: Box<crate::unit_manager_workspace::Definition>,
    },
    Ospf {
        initial: crate::ospf::State,
        #[serde(deserialize_with = "crate::ospf_workspace::requirements")]
        requirements: Vec<crate::ospf_workspace::Requirement>,
    },
    Ipv4Acl {
        definition: crate::acl_editor::Definition,
    },
    Accounts {
        initial: crate::accounts::State,
        #[serde(deserialize_with = "crate::accounts_teaching::requirements")]
        requirements: Vec<crate::accounts::Requirement>,
    },
    PersistentMounts {
        initial: crate::storage::Inventory,
        requirements: Vec<crate::storage_workspace::Requirement>,
    },
    Discovery {
        initial: crate::discovery::Topology,
        requirements: Vec<crate::discovery::Requirement>,
    },
    Processes {
        requirements: Vec<crate::processes::Requirement>,
    },
    AuthoritativeDns {
        requirements: Vec<crate::dns_workspace::Requirement>,
    },
    Switching {
        requirements: Vec<crate::switching::Requirement>,
    },
    Ipv6 {
        requirements: Vec<crate::ipv6::Requirement>,
    },
    Ipv4Routing {
        requirements: Vec<crate::ipv4_routing::Requirement>,
    },
    Filesystem {
        requirements: Vec<crate::filesystem::Requirement>,
    },
    Script {
        cases: Vec<crate::shell::Case>,
    },
    Subnet,
    Vlsm,
    Network {
        address: Ipv4Addr,
    },
    Access {
        path: String,
        user: String,
        read: bool,
        write: bool,
        private: bool,
        #[serde(default)]
        exclusive: bool,
    },
    Logs {
        path: String,
        needle: String,
    },
    Service {
        port: u16,
    },
    Dns {
        name: String,
        address: Ipv4Addr,
    },
    Mount,
}
pub fn bounds(ip: Ipv4Addr, prefix: u8) -> Option<(u32, u32)> {
    if prefix > 32 {
        return None;
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let net = u32::from(ip) & mask;
    Some((net, net | !mask))
}
pub fn cidr(s: &str) -> Option<(u32, u32, u8)> {
    let (a, p) = s.split_once('/')?;
    let ip = a.parse().ok()?;
    let p = p.parse().ok()?;
    let (n, b) = bounds(ip, p)?;
    if u32::from(ip) != n {
        return None;
    }
    Some((n, b, p))
}
fn bits(n: &Node, user: &str, groups: &[String]) -> u16 {
    if user == n.owner {
        (n.mode >> 6) & 7
    } else if groups.contains(&n.group) {
        (n.mode >> 3) & 7
    } else {
        n.mode & 7
    }
}
pub fn access(
    nodes: &BTreeMap<String, Node>,
    users: &BTreeMap<String, Vec<String>>,
    path: &str,
    user: &str,
    mask: u16,
) -> bool {
    let Some(groups) = users.get(user) else {
        return false;
    };
    let mut parent = String::new();
    let parts: Vec<_> = path.trim_start_matches('/').split('/').collect();
    if let Some(root) = nodes.get("/") {
        if !root.directory || bits(root, user, groups) & 1 == 0 {
            return false;
        }
    } else {
        return false;
    }
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        parent.push('/');
        parent.push_str(part);
        let Some(n) = nodes.get(&parent) else {
            return false;
        };
        if !n.directory || bits(n, user, groups) & 1 == 0 {
            return false;
        }
    }
    nodes
        .get(path)
        .is_some_and(|n| bits(n, user, groups) & mask == mask)
}
pub fn checks(s: &State, g: &Goal) -> Vec<(bool, String)> {
    if let (State::MagicSubnet { session }, Goal::MagicSubnet { problem, stage }) = (s, g) {
        if session.validate().is_err() || session.problem != *problem || session.stage != *stage {
            return vec![(
                false,
                "The supplied subnet problem must remain unchanged".into(),
            )];
        }
        return crate::magic_subnet::checks(*stage, *problem, &session.answers);
    }
    let c = |b, t: &str| (b, t.to_owned());
    match (s, g) {
        (State::FileIndex { simulation }, Goal::FileIndex { definition }) => {
            crate::file_index_source::checks(simulation, definition)
        }
        (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) => {
            definition.checks(simulation)
        }
        (State::Raid1 { simulation }, Goal::Raid1 { definition }) => definition.checks(simulation),
        (State::SharedLibraries { simulation }, Goal::SharedLibraries { definition }) => {
            definition.checks(simulation)
        }
        (State::FsIntegrity { simulation }, Goal::FsIntegrity { definition }) => {
            definition.checks(simulation)
        }
        (State::UserMounts { simulation }, Goal::UserMounts { definition }) => {
            if simulation.definition != **definition || simulation.validate(definition).is_err() {
                vec![(
                    false,
                    "Saved caller-aware mount work does not match this assignment.".into(),
                )]
            } else {
                crate::user_mounts_workspace::checks(&definition.assignment, &simulation.assignment)
            }
        }
        (State::Environment { simulation }, Goal::Environment { definition }) => {
            if simulation.definition != **definition || simulation.validate(definition).is_err() {
                vec![(
                    false,
                    "Saved environment work does not match this assignment.".into(),
                )]
            } else {
                crate::environment_workspace::checks(&definition.assignment, &simulation.assignment)
            }
        }
        (State::DebianSources { simulation }, Goal::DebianSources { definition }) => {
            definition.checks(simulation)
        }
        (State::ClientNames { simulation }, Goal::ClientNames { definition }) => {
            definition.checks(simulation)
        }
        (State::RemovableMounts { simulation }, Goal::RemovableMounts { definition }) => {
            definition.checks(simulation)
        }
        (State::NmProfiles { simulation }, Goal::NmProfiles { definition }) => {
            definition.checks(simulation)
        }
        (State::PamStack { simulation }, Goal::PamStack { definition }) => {
            definition.checks(simulation)
        }
        (State::Aging { simulation }, Goal::Aging { definition }) => {
            definition.checks(simulation)
        }
        (State::PortSecurity { simulation }, Goal::PortSecurity { definition }) => {
            definition.checks(simulation)
        }
        (State::DhcpSnooping { simulation }, Goal::DhcpSnooping { definition }) => {
            definition.checks(simulation)
        }
        (State::ArpInspection { simulation }, Goal::ArpInspection { definition }) => {
            definition.checks(simulation)
        }
        (State::ManagementPolicy { simulation }, Goal::ManagementPolicy { definition }) => {
            definition.checks(simulation)
        }
        (State::LocalAccess { simulation }, Goal::LocalAccess { definition }) => {
            definition.checks(simulation)
        }
        (State::AaaAccess { simulation }, Goal::AaaAccess { definition }) => {
            definition.checks(simulation)
        }
        (State::LdapClient { simulation }, Goal::LdapClient { definition }) => {
            definition.checks(simulation)
        }
        (State::LdapAccess { simulation }, Goal::LdapAccess { definition }) => {
            definition.checks(simulation)
        }
        (State::LdapSchema { simulation }, Goal::LdapSchema { definition }) => {
            definition.checks(simulation)
        }
        (State::CopiedJournal { simulation }, Goal::CopiedJournal { definition }) => definition.checks(simulation),
        (State::JsonInventory { simulation }, Goal::JsonInventory { definition }) => definition.checks(simulation),
        (State::RestController { simulation }, Goal::RestController { definition }) => definition.checks(simulation),
        (State::DevicePath { simulation }, Goal::DevicePath { definition }) => definition.checks(simulation),
        (State::LocalMail { simulation }, Goal::LocalMail { definition }) => definition.checks(simulation),
        (State::Chrony { simulation }, Goal::Chrony { definition }) => definition.checks(simulation),
        (State::GrubOnce { simulation }, Goal::GrubOnce { definition }) => definition.checks(simulation),
        (State::EfiLoader { simulation }, Goal::EfiLoader { definition }) => definition.checks(simulation),
        (State::BiosLoader { simulation }, Goal::BiosLoader { definition }) => definition.checks(simulation),
        (State::BootEntry { simulation }, Goal::BootEntry { definition }) => definition.checks(simulation),
        (State::LdapSchemaQuery { simulation }, Goal::LdapSchemaQuery { definition }) => {
            definition.checks(simulation)
        }
        (State::LdapPassword { simulation }, Goal::LdapPassword { definition }) => {
            definition.checks(simulation)
        }
        (State::LdapChanges { simulation }, Goal::LdapChanges { definition }) => {
            definition.checks(simulation)
        }
        (State::DhcpConfig { simulation }, Goal::DhcpConfig { definition }) => {
            definition.checks(simulation)
        }
        (State::Qos { simulation }, Goal::Qos { definition }) => definition.checks(simulation),
        (State::LinuxLogging { simulation }, Goal::LinuxLogging { definition }) => {
            definition.checks(simulation, false)
        }
        (State::VirtualIsolation { simulation }, Goal::VirtualIsolation { definition }) => simulation.checks(definition,false),
        (State::SieveFiltering { simulation }, Goal::SieveFiltering { definition }) => definition.checks(simulation).into_iter().map(|r|(r.passed,format!("{}: {}",r.label,r.detail))).collect(),
        (State::LinuxRouter { simulation }, Goal::LinuxRouter { definition }) => crate::linux_router_source::checks(simulation,definition),
        (State::LinuxV6 { simulation }, Goal::LinuxV6 { definition }) => crate::linux_v6_source::checks(simulation,definition),
        (State::LinuxServiceRouter { simulation }, Goal::LinuxServiceRouter { definition }) => crate::linux_service_router_source::checks(simulation,definition),
        (State::LinuxV6Administration { simulation }, Goal::LinuxV6Administration { definition }) => crate::linux_v6_admin_source::checks(simulation,definition),
        (State::LinuxV6Startup { simulation }, Goal::LinuxV6Startup { definition }) => crate::linux_v6_startup_source::checks(simulation,definition),
        (State::IdsRule { simulation }, Goal::IdsRule { definition }) => crate::ids_rule_source::checks(simulation,definition),
        (State::AuthFilter { simulation }, Goal::AuthFilter { definition }) => crate::auth_filter_source::checks(simulation,definition),
        (State::VpnPoint { simulation }, Goal::VpnPoint { definition }) => crate::vpn_point_source::checks(simulation,definition),
        (State::Poe { simulation }, Goal::Poe { definition }) => crate::poe_source::checks(simulation,definition),
        (State::KernelBuild { simulation }, Goal::KernelBuild { definition }) => crate::kernel_source::checks(simulation,definition),
        (State::Dkms { simulation }, Goal::Dkms { definition }) => crate::dkms_source::checks(simulation,definition),
        (State::Initramfs { simulation }, Goal::Initramfs { definition }) => crate::initramfs_source::checks(simulation,definition),
        (State::Pxe { simulation }, Goal::Pxe { definition }) => crate::pxe::source::checks(simulation,definition),
        (State::Iscsi { simulation }, Goal::Iscsi { definition }) => crate::iscsi::source::checks(simulation,definition),
        (State::AtaSmart { simulation }, Goal::AtaSmart { definition }) => crate::ata_smart::source::checks(simulation,definition),
        (State::ApacheHttps { simulation }, Goal::ApacheHttps { definition }) => crate::apache_https::source::checks(simulation,definition),
        (State::Desktop { simulation }, Goal::Desktop { definition }) => crate::desktop::source::checks(simulation,definition),
        (State::SecurityInvestigation { simulation }, Goal::SecurityInvestigation { definition }) => crate::security_investigation::source::checks(simulation,definition),
        (State::AutomationWorkshop { simulation }, Goal::AutomationWorkshop { definition }) => crate::automation_workshop::source::checks(simulation,definition),
        (State::ServiceExposure { simulation }, Goal::ServiceExposure { definition }) => crate::service_exposure_source::checks(simulation,definition),
        (State::FtpIdentity { simulation }, Goal::FtpIdentity { definition }) => crate::ftp_identity_source::checks(simulation,definition),
        (State::LinuxFlow { simulation }, Goal::LinuxFlow { definition }) => crate::linux_flow_source::checks(simulation,definition),
        (State::MailboxAdministration { simulation }, Goal::MailboxAdministration { definition }) |
        (State::MailboxAccess { simulation }, Goal::MailboxAccess { definition }) => definition.checks(simulation).into_iter().map(|r|(r.passed,format!("{}: {}",r.label,r.detail))).collect(),
        (State::SmtpRouting { simulation }, Goal::SmtpRouting { definition }) => definition.checks(simulation),
        (State::GuestDrivers { simulation }, Goal::GuestDrivers { definition }) => definition.checks(simulation),
        (State::SambaShare { simulation }, Goal::SambaShare { definition }) => simulation.checks(definition,false),
        (State::CampusArchitecture { simulation }, Goal::CampusArchitecture { definition }) => simulation.checks(definition,false),
        (State::NetworkTransfer { simulation }, Goal::NetworkTransfer { definition }) => simulation.checks(definition,false),
        (State::EditorHandover { simulation }, Goal::EditorHandover { definition }) => simulation.checks(definition,false),
        (State::ModalEditing { simulation }, Goal::ModalEditing { definition }) => definition.checks(simulation,false),
        (State::SshAccess { simulation }, Goal::SshAccess { definition }) => definition.checks(simulation,false),
        (State::HostNetwork { simulation }, Goal::HostNetwork { definition }) => {
            definition.checks(simulation, false)
        }
        (State::Repositories { simulation }, Goal::Repositories { definition }) => {
            definition.checks(simulation, false)
        }
        (State::MaintenanceNotice { simulation }, Goal::MaintenanceNotice { definition }) => definition.checks(simulation,false),
        (State::DnsAccess { simulation }, Goal::DnsAccess { definition }) => definition.checks(simulation,false),
        (State::PrintQueues { simulation }, Goal::PrintQueues { definition }) => definition.checks(simulation,false),
        (State::Localization { simulation }, Goal::Localization { definition }) => definition.checks(simulation,false),
        (State::WirelessChannels { simulation }, Goal::WirelessChannels { definition }) => definition.checks(simulation,false),
        (State::WlanProfiles { simulation }, Goal::WlanProfiles { definition }) => definition.checks(simulation,false),
        (State::HttpCache { simulation }, Goal::HttpCache { definition }) => definition.checks(simulation),
        (State::ServiceClients { simulation }, Goal::ServiceClients { definition }) => definition.checks(simulation,false),
        (State::ReverseProxy { simulation }, Goal::ReverseProxy { definition }) => definition.checks(simulation,false),
        (State::ProxyPolicy { simulation }, Goal::ProxyPolicy { definition }) => definition.checks(simulation,false),
        (State::SnmpMonitor { simulation }, Goal::SnmpMonitor { definition }) => definition.checks(simulation,false),
        (State::WlanPaths { simulation }, Goal::WlanPaths { definition }) => definition.checks(simulation,false),
        (State::Timezone { simulation }, Goal::Timezone { definition }) => definition.checks(simulation,false),
        (State::SourceInstall { simulation }, Goal::SourceInstall { definition }) => definition.checks(simulation,false),
        (State::CapacityPlanning { simulation }, Goal::CapacityPlanning { definition }) => definition.checks(simulation,false),
        (State::ResourceObservations { simulation }, Goal::ResourceObservations { definition }) => definition.checks(simulation,false),
        (State::Capacity { simulation }, Goal::Capacity { definition }) => {
            definition.checks(simulation, false)
        }
        (State::RaidAssembly { simulation }, Goal::RaidAssembly { definition }) => definition.checks(simulation),
        (State::Autofs { simulation }, Goal::Autofs { definition }) => definition.checks(simulation),
        (State::KernelModules { simulation }, Goal::KernelModules { definition }) => {
            definition.checks(simulation)
        }
        (State::RecursiveTree { simulation }, Goal::RecursiveTree { definition }) => {
            definition.checks(simulation)
        }
        (State::Transport { simulation }, Goal::Transport { definition }) => {
            definition.checks(simulation)
        }
        (State::DirectoryPolicy { simulation }, Goal::DirectoryPolicy { definition }) => {
            definition.checks(simulation, false)
        }
        (State::Links { simulation }, Goal::Links { definition }) => {
            definition.checks(simulation, false)
        }
        (State::PartitionFs { simulation }, Goal::PartitionFs { definition }) => {
            definition.checks(simulation)
        }
        (State::Hsrp { simulation }, Goal::Hsrp { definition }) => definition.checks(simulation),
        (State::Syslog { simulation }, Goal::Syslog { definition }) => {
            definition.checks(simulation)
        }
        (State::Ntp { simulation }, Goal::Ntp { definition }) => definition.checks(simulation),
        (State::Lvm { simulation }, Goal::Lvm { definition }) => definition.checks(simulation),
        (State::PhysicalLink { simulation }, Goal::PhysicalLink { definition }) => {
            definition.checks(simulation)
        }
        (State::FileBackup { simulation }, Goal::FileBackup { definition }) => {
            definition.checks(simulation)
        }
        (State::EtherChannel { simulation }, Goal::EtherChannel { definition }) => {
            definition.checks(simulation)
        }
        (State::OneOff { simulation }, Goal::OneOff { definition }) => {
            definition.checks(simulation)
        }
        (State::Cron { simulation }, Goal::Cron { definition }) => definition.checks(simulation),
        (State::Nfs { simulation }, Goal::Nfs { definition }) => definition.checks(simulation),
        (State::RpmPackages { simulation }, Goal::RpmPackages { definition }) => {
            definition.checks(simulation)
        }
        (State::BasicNat { simulation }, Goal::BasicNat { definition }) => {
            definition.checks(simulation)
        }
        (State::ApacheStatic { simulation }, Goal::ApacheStatic { definition }) => {
            definition.checks(simulation)
        }
        (State::DebianPackages { simulation }, Goal::DebianPackages { definition }) => {
            definition.checks(simulation)
        }
        (
            State::SpanningTree { simulation },
            Goal::SpanningTree {
                initial,
                requirements,
            },
        ) => crate::spanning_tree_workspace::checks(initial, simulation, requirements),
        (
            State::DhcpV4 { simulation },
            Goal::DhcpV4 {
                initial,
                requirements,
                probes,
            },
        ) => crate::dhcpv4_workspace::checks(initial, simulation, requirements, probes),
        (State::UnitManager { simulation }, Goal::UnitManager { definition }) => {
            match crate::unit_manager_workspace::Draft::new(simulation) {
                Ok(draft) => crate::unit_manager_workspace::checks(definition, simulation, &draft),
                Err(_) => vec![(false, "Invalid virtual unit-manager state".into())],
            }
        }
        (
            State::Ospf { simulation },
            Goal::Ospf {
                initial,
                requirements,
            },
        ) => crate::ospf_workspace::checks(initial, simulation, requirements),
        (State::Ipv4Acl { network }, Goal::Ipv4Acl { definition }) => {
            crate::acl_workspace::checks(network, definition, &crate::acl_editor::Drafts::default())
        }
        (
            State::Accounts { simulation },
            Goal::Accounts {
                initial,
                requirements,
            },
        ) => crate::accounts::checks(initial, simulation, requirements),
        (
            State::PersistentMounts { storage },
            Goal::PersistentMounts {
                initial,
                requirements,
            },
        ) => {
            if storage.inventory() != initial
                || !crate::storage_workspace::valid_requirements(requirements, initial)
            {
                vec![c(
                    false,
                    "Storage inventory or authored objectives differ from this assignment",
                )]
            } else {
                crate::storage_workspace::checks(storage, requirements)
            }
        }
        (
            State::Discovery { simulation },
            Goal::Discovery {
                initial,
                requirements,
            },
        ) => crate::discovery_workspace::checks(initial, simulation, requirements),
        (State::Processes { table }, Goal::Processes { requirements }) => {
            requirements.iter().map(|r| r.check(table)).collect()
        }
        (State::AuthoritativeDns { server }, Goal::AuthoritativeDns { requirements }) => {
            requirements.iter().map(|r| r.check(server)).collect()
        }
        (
            State::MacLearning { simulation },
            Goal::MacLearning {
                initial,
                requirements,
            },
        ) => crate::mac_learning_workspace::checks(initial, simulation, requirements),
        (State::Switching { fabric }, Goal::Switching { requirements }) => {
            requirements.iter().map(|r| r.check(fabric)).collect()
        }
        (State::Ipv6 { topology }, Goal::Ipv6 { requirements }) => {
            requirements.iter().map(|r| r.check(topology)).collect()
        }
        (State::Ipv4Routing { topology }, Goal::Ipv4Routing { requirements }) => {
            requirements.iter().map(|r| r.check(topology)).collect()
        }
        (State::Filesystem { tree }, Goal::Filesystem { requirements }) => {
            if let Err(error) = crate::filesystem::discovery_policy::for_current(requirements, tree)
            {
                return vec![(false, error)];
            }
            requirements.iter().map(|r| r.check(tree)).collect()
        }
        (State::Script { source, run }, Goal::Script { cases }) => {
            crate::shell::checks(source, run, cases)
        }
        (
            State::Subnet {
                ip,
                prefix,
                answers,
                second,
            },
            Goal::Subnet,
        ) => {
            if *prefix > 30 {
                return vec![c(
                    false,
                    "Only conventional subnets through /30 are supported",
                )];
            }
            let Some((n, b)) = bounds(*ip, *prefix) else {
                return vec![c(false, "Invalid prefix")];
            };
            let want = [n, b, n + 1, b - 1];
            let mut results: Vec<_> = [
                "Network address",
                "Broadcast address",
                "First usable host",
                "Last usable host",
            ]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                c(
                    answers[i]
                        .trim()
                        .parse::<Ipv4Addr>()
                        .is_ok_and(|v| u32::from(v) == want[i]),
                    t,
                )
            })
            .collect();
            if let Some(second) = second {
                results.push(c(
                    crate::ipv4::second_result(*ip, *prefix, second)
                        == crate::ipv4::SecondResult::Valid,
                    "Second workstation uses a different usable address",
                ));
            }
            results
        }
        (
            State::Vlsm {
                base,
                prefix,
                allocations,
            },
            Goal::Vlsm,
        ) => {
            let (start, end) = bounds(*base, *prefix).unwrap_or((1, 0));
            allocations
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let ok = cidr(a.cidr.trim()).is_some_and(|(n, b, p)| {
                        p <= 30
                            && n >= start
                            && b <= end
                            && (b as u64 - n as u64 + 1) >= a.hosts as u64 + 2
                            && allocations.iter().enumerate().all(|(j, other)| {
                                i == j
                                    || cidr(other.cidr.trim())
                                        .is_none_or(|(on, ob, _)| b < on || n > ob)
                            })
                    });
                    c(
                        ok,
                        &format!("{}: {} hosts, aligned and non-overlapping", a.name, a.hosts),
                    )
                })
                .collect()
        }
        (
            State::Network {
                address,
                prefix,
                gateway,
                router,
                lan_prefix,
                target: _,
            },
            Goal::Network { address: required },
        ) => vec![
            c(address == required, "Client retains its assigned address"),
            c(prefix == lan_prefix, "Client mask matches the LAN"),
            c(
                gateway == router
                    && bounds(*address, *prefix)
                        .zip(bounds(*router, *prefix))
                        .is_some_and(|(a, b)| a == b),
                "Default route uses the reachable router",
            ),
        ],
        (
            State::Unix { nodes, users },
            Goal::Access {
                path,
                user,
                read,
                write,
                private,
                exclusive,
            },
        ) => vec![
            c(
                access(
                    nodes,
                    users,
                    path,
                    user,
                    if *read { 4 } else { 0 } | if *write { 2 } else { 0 },
                ),
                "Required user has access through every directory",
            ),
            c(
                (!private || nodes.get(path).is_some_and(|n| n.mode & 7 == 0))
                    && (!exclusive
                        || users.keys().all(|other| {
                            other == user
                                || other == "root"
                                || ![1, 2, 4]
                                    .iter()
                                    .any(|bit| access(nodes, users, path, other, *bit))
                        })),
                if *exclusive {
                    "Only the required user can access the file (administrator excluded)"
                } else {
                    "Other-class file permissions are removed"
                },
            ),
        ],
        (State::Logs { files, result }, Goal::Logs { path, needle }) => vec![c(
            files.get(path).is_some_and(|text| {
                *result
                    == text
                        .lines()
                        .filter(|l| l.contains(needle))
                        .collect::<Vec<_>>()
                        .join("\n")
            }),
            "Last text query contains exactly the requested log lines",
        )],
        (
            State::Service {
                port,
                running,
                enabled,
                loaded_port,
                ..
            },
            Goal::Service { port: expected },
        ) => vec![
            c(
                port.parse::<u16>() == Ok(*expected),
                "Configuration has the required valid port",
            ),
            c(
                *running && *loaded_port == Some(*expected),
                "Service is running with the repaired configuration",
            ),
            c(*enabled, "Service is enabled for boot"),
        ],
        (
            State::Dns {
                records,
                loaded,
                serial,
                loaded_serial,
            },
            Goal::Dns { name, address },
        ) => vec![
            c(
                records.get(name).and_then(|v| v.parse::<Ipv4Addr>().ok()) == Some(*address),
                "A record has the required address",
            ),
            c(
                loaded.get(name).and_then(|v| v.parse::<Ipv4Addr>().ok()) == Some(*address)
                    && serial == loaded_serial,
                "Authoritative zone is reloaded",
            ),
        ],
        (
            State::Mount {
                capacity,
                used,
                mounted,
                device,
                configured_device,
                writable,
                ..
            },
            Goal::Mount,
        ) => vec![
            c(
                *capacity > 0 && (*used as u128) * 100 <= (*capacity as u128) * 80,
                "Filesystem usage is at most 80%",
            ),
            c(
                device == configured_device,
                "Persistent mount references the correct device",
            ),
            c(
                *mounted && *writable,
                "Data filesystem is mounted read-write",
            ),
        ],
        _ => vec![c(false, "State and objective do not match")],
    }
}
pub fn solved(s: &State, g: &Goal) -> bool {
    let c = checks(s, g);
    !c.is_empty() && c.iter().all(|c| c.0)
}
pub fn help(s: &State) -> &'static str {
    match s {
        State::Filesystem { tree } if tree.command_location.is_some() => {
            "Command-location activity: which NAME (first eligible supplied path); type -a -p NAME (all eligible supplied paths); ls -l PATH (permissions/owner); chmod MODE PATH (only supplied caller-owned command files); cat PATH (inert text); pwd/cd (navigation). PATH and all other files stay fixed. help/show are simulator helpers. No found file executes. Plain type, aliases, hashing, whereis, composition and arbitrary shell syntax are unsupported. Other filesystem commands remain limited by this activity’s preservation policy."
        }
        State::StartupFiles { .. } => crate::startup_workspace::HELP,
        State::Raid1 { .. } => crate::raid1::HELP,
        State::FileIndex { .. } => crate::file_index_workspace::HELP,
        State::SharedLibraries { .. } => crate::shared_libraries_workspace::HELP,
        State::FsIntegrity { .. } => crate::fs_integrity_terminal::HELP,
        State::UserMounts { .. } => {
            "Use the simulated configuration editor and caller selector. Save policy, mount or unmount as a supplied caller, and check both actual state and prospective cases. No host filesystem or account is accessed."
        }
        State::Environment { .. } => {
            "Edit the setup script, Run setup + child, and inspect actual parent values, export flags and the supplied child report. Everything runs in the bounded in-process simulator. No host environment, process or file is changed."
        }
        State::DebianSources { .. } => crate::debian_sources_workspace::HELP,
        State::ClientNames { simulation } => crate::client_names_workspace::help(
            if simulation.assignment.diagnostic.is_some() {
                crate::client_names_workspace::Mode::SourcesWithSearch
            } else if simulation.assignment.draft.contains_key("resolv.conf") {
                crate::client_names_workspace::Mode::Search
            } else {
                crate::client_names_workspace::Mode::Sources
            },
        ),
        State::RemovableMounts { .. } => crate::removable_workspace::HELP,
        State::NmProfiles { .. } => crate::nm_workspace::HELP,
        State::PamStack { .. } => crate::pam_workspace::HELP,
        State::Aging { .. } => crate::aging::HELP,
        State::PortSecurity { .. } => crate::port_security_workspace::HELP,
        State::DhcpSnooping { .. } => crate::dhcp_snooping_workspace::HELP,
        State::ArpInspection { .. } => crate::arp_inspection_workspace::HELP,
        State::ManagementPolicy { .. } => crate::management_workspace::HELP,
        State::LocalAccess { .. } => crate::local_access_workspace::HELP,
        State::AaaAccess { .. } => crate::aaa_workspace::HELP,
        State::LdapClient { .. } => crate::ldap_workspace::HELP,
        State::LdapAccess { .. } => crate::ldap_access_workspace::HELP,
        State::LdapSchema { .. } => crate::ldap_schema_workspace::HELP,
        State::CopiedJournal { .. } => crate::journal_archive_workspace::HELP,
        State::JsonInventory { .. } => crate::json_inventory_workspace::HELP,
        State::RestController { .. } => crate::rest_workspace::HELP,
        State::DevicePath { .. } => crate::device_path_workspace::HELP,
        State::LocalMail { .. } => crate::local_mail_workspace::HELP,
        State::Chrony { .. } => crate::chrony::HELP,
        State::GrubOnce { .. } => crate::grub_once_workspace::HELP,
        State::EfiLoader { .. } => crate::efi_loader::HELP,
        State::BiosLoader { .. } => crate::bios_loader::HELP,
        State::BootEntry { .. } => crate::boot_entry::HELP,
        State::LdapSchemaQuery { .. } => crate::ldap_schema_query_workspace::HELP,
        State::LdapPassword { .. } => crate::ldap_password_workspace::HELP,
        State::LdapChanges { .. } => crate::ldap_change_workspace::HELP,
        State::DhcpConfig { .. } => crate::dhcp_server_workspace::HELP,
        State::Qos { .. } => crate::qos::HELP,
        State::LinuxLogging { .. } => crate::linux_logging::HELP,
        State::VirtualIsolation { .. } => crate::virtual_isolation_source::HELP,
        State::SieveFiltering { .. } => crate::sieve_source::HELP,
        State::MailboxAccess { .. } => crate::mailbox_source::HELP,
        State::MailboxAdministration { .. } => crate::mailbox_source::ADMIN_HELP,
        State::LinuxRouter { .. } => crate::linux_router_source::HELP,
        State::LinuxV6 { .. } => crate::linux_v6_source::HELP,
        State::LinuxServiceRouter { .. } => crate::linux_service_router_source::HELP,
        State::LinuxV6Administration { .. } => crate::linux_v6_admin_source::HELP,
        State::LinuxV6Startup { .. } => crate::linux_v6_startup_source::HELP,
        State::IdsRule { .. } => crate::ids_rule_source::HELP,
        State::AuthFilter { .. } => crate::auth_filter_source::HELP,
        State::VpnPoint { .. } => crate::vpn_point_source::HELP,
        State::Poe { .. } => crate::poe_source::HELP,
        State::KernelBuild { .. } => crate::kernel_source::HELP,
        State::Dkms { .. } => crate::dkms_source::HELP,
        State::Initramfs { .. } => crate::initramfs_source::HELP,
        State::Pxe { .. } => crate::pxe::source::HELP,
        State::Iscsi { .. } => crate::iscsi::source::HELP,
        State::AtaSmart { .. } => crate::ata_smart::source::HELP,
        State::ApacheHttps { .. } => crate::apache_https::source::HELP,
        State::Desktop { .. } => crate::desktop::source::HELP,
        State::SecurityInvestigation { .. } => crate::security_investigation::source::HELP,
        State::AutomationWorkshop { .. } => crate::automation_workshop::source::HELP,
        State::ServiceExposure { .. } => crate::service_exposure_source::HELP,
        State::FtpIdentity { .. } => crate::ftp_identity_source::HELP,
        State::LinuxFlow { .. } => crate::linux_flow_source::HELP,
        State::SmtpRouting { .. } => crate::smtp_console::HELP,
        State::GuestDrivers { .. } => crate::guest_driver::HELP,
        State::SambaShare { .. } => crate::samba_share_source::HELP,
        State::CampusArchitecture { .. } => crate::network_architecture_source::HELP,
        State::NetworkTransfer { .. } => crate::network_transfer_source::HELP,
        State::EditorHandover { .. } => crate::editor_handover::HELP,
        State::ModalEditing { .. } => crate::modal_source::HELP,
        State::SshAccess { .. } => crate::ssh_workspace::HELP,
        State::HostNetwork { simulation } => simulation.assignment.world.command_help(),
        State::Repositories { .. } => crate::repository_workspace::HELP,
        State::MaintenanceNotice { .. } => crate::maintenance_notice_workspace::HELP,
        State::DnsAccess { .. } => crate::dns_access_workspace::HELP,
        State::PrintQueues { .. } => crate::print_queue_workspace::HELP,
        State::Localization { .. } => crate::localization_workspace::HELP,
        State::WirelessChannels { .. } => crate::wireless_channel_workspace::HELP,
        State::WlanProfiles { .. } => crate::wlan_workspace::HELP,
        State::HttpCache { .. } => crate::http_cache_workspace::HELP,
        State::ServiceClients { .. } => crate::service_workspace::HELP,
        State::ReverseProxy { .. } => crate::reverse_proxy_workspace::HELP,
        State::ProxyPolicy { .. } => crate::proxy_workspace::HELP,
        State::SnmpMonitor { .. } => crate::snmp_workspace::HELP,
        State::WlanPaths { .. } => crate::wlan_paths_workspace::HELP,
        State::Timezone { .. } => crate::timezone_workspace::HELP,
        State::SourceInstall { .. } => crate::source_install_workspace::HELP,
        State::CapacityPlanning { .. } => crate::capacity_planning_workspace::HELP,
        State::ResourceObservations { .. } => crate::resource_observation_workspace::HELP,
        State::Capacity { .. } => crate::capacity_workspace::HELP,
        State::RaidAssembly { .. } => crate::raid_assembly_workspace::HELP,
        State::Autofs { .. } => crate::autofs_workspace::HELP,
        State::KernelModules { .. } => crate::kernel_modules::HELP,
        State::RecursiveTree { .. } => crate::filesystem::recursive::HELP,
        State::Transport { .. } => crate::transport_workspace::HELP,
        State::DirectoryPolicy { .. } => crate::filesystem::directory_policy::HELP,
        State::Links { .. } => crate::links_workspace::HELP,
        State::PartitionFs { .. } => crate::partition_fs_workspace::HELP,
        State::Hsrp { .. } => crate::hsrp::HELP,
        State::Syslog { .. } => crate::syslog_workspace::HELP,
        State::Ntp { .. } => crate::ntp::HELP,
        State::Lvm { simulation } => simulation.current.command_help(),
        State::PhysicalLink { .. } => crate::physical_link_workspace::HELP,
        State::FileBackup { .. } => crate::file_backup_terminal::HELP,
        State::EtherChannel { .. } => crate::etherchannel_workspace::HELP,
        State::OneOff { .. } => crate::one_off_workspace::HELP,
        State::Cron { .. } => crate::cron_workspace::HELP,
        State::Nfs { .. } => crate::nfs_workspace::HELP,
        State::RpmPackages { .. } => crate::rpm_packages_workspace::HELP,
        State::BasicNat { .. } => crate::nat_workspace::HELP,
        State::ApacheStatic { .. } => crate::apache_static_workspace::HELP,
        State::DebianPackages { .. } => crate::debian_packages_terminal::HELP,
        State::SpanningTree { .. } => crate::spanning_tree_workspace::HELP,
        State::DhcpV4 { .. } => crate::dhcpv4_workspace::HELP,
        State::MacLearning { .. } => crate::mac_learning_workspace::HELP,
        State::UnitManager { .. } => crate::unit_manager_workspace::HELP,
        State::Ipv4Acl { .. } => crate::acl_workspace::HELP,
        State::Ospf { .. } => crate::ospf_workspace::HELP,
        State::Accounts { .. } => crate::accounts_teaching::HELP,
        State::PersistentMounts { storage } if storage.inventory().swap().is_some() => {
            crate::storage_workspace::HELP_WITH_SWAP
        }
        State::PersistentMounts { .. } => crate::storage_workspace::HELP,
        State::Discovery { .. } => {
            "Use the native discovery settings and explicit Send advertisement / Advance time controls. These model LLDP events, not router commands or real packets. Timers are manual; no CDP or automatic scheduling is simulated."
        }
        State::Processes { .. } => crate::processes::HELP,
        State::AuthoritativeDns { .. } => {
            "Use the native record editor, explicit Save zone / Reload primary / Update secondary controls and in-memory lookups. Only the disclosed record subset is supported; reference actions are not Linux commands."
        }
        State::Switching { .. } => {
            "Use native switch/port tables and broadcast traces. This bounded VLAN data plane is not a Cisco CLI, ping, MAC learning or STP simulator."
        }
        State::Ipv6 { .. } => {
            "Use the native IPv6 configuration editors and path probes. Only the documented bounded static forwarding subset is simulated. Pack reference adapters are not router or Linux commands."
        }
        State::Ipv4Routing { .. } => {
            "Use native IPv4 configuration editors and applied path inspection. Only bounded static forwarding is modeled. Pack reference actions are not router/Linux commands."
        }
        State::Filesystem { .. } => crate::filesystem::HELP,
        State::Script { .. } => {
            "Use the script editor and Run script. Only the documented in-process shell subset is supported."
        }
        State::MagicSubnet { .. } => {
            "Use the native subnet worksheet. No terminal commands run in this lesson."
        }
        State::Subnet { .. } => {
            "Use the native address fields and Check configuration. This activity has no terminal."
        }
        State::Vlsm { .. } => {
            "Use the native allocation table and Check work. Networks must be aligned."
        }
        State::Network { .. } => {
            "show • ip addr • ip route • ip addr set IPv4/PREFIX • ip route replace default via IPv4 • ping IPv4"
        }
        State::Unix { .. } => {
            "ls -l|-ld PATH • namei PATH • id USER • chmod OCTAL|u=rw,g=r,o= PATH • chown USER:GROUP PATH • usermod -aG GROUP USER • Simulator-only helper: test-access USER PATH r|w|rw"
        }
        State::Logs { .. } => {
            "find /var/log -name NAME • cat PATH • grep LITERAL PATH • cat PATH | grep LITERAL [| tail -n N] • tail -n N PATH. Literals are one word, case-sensitive; no regex, shell quoting or redirection."
        }
        State::Service { .. } => {
            "cat /etc/atlas.conf • Use the simulated config editor to save the port. Legacy simulator-only helper: config set port VALUE • systemctl status|start|stop|restart|enable|disable|is-active|is-enabled atlas • journalctl -u atlas"
        }
        State::Dns { .. } => {
            "cat /etc/named/lab.zone • Simulator-only editors: dns set NAME IPv4; dns serial NUMBER • named-checkzone lab.test • systemctl reload named • dig NAME"
        }
        State::Mount { .. } => {
            "df -h • du -sh /data/cache • cat /etc/fstab • Simulator-only editor: config set device DEVICE • rm -r /data/cache • mount /data • mount -o remount,rw /data • umount /data"
        }
    }
}
/// Preserve the exact legacy path for other kinds. Account mutations must retain
/// compatibility with the immutable authored initial state before publication.
pub fn execute_for_goal(s: &mut State, goal: &Goal, input: &str) -> Result<String, String> {
    if let State::LinuxRouter{simulation}=s {let Goal::LinuxRouter{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_router_reference(definition,input);}
    if let State::LinuxV6{simulation}=s {let Goal::LinuxV6{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_v6_reference(definition,input);}
    if let State::LinuxServiceRouter{simulation}=s {let Goal::LinuxServiceRouter{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_service_router_reference(definition,input);}
    if let State::LinuxV6Administration{simulation}=s {let Goal::LinuxV6Administration{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_v6_admin_reference(definition,input);}
    if let State::LinuxV6Startup{simulation}=s {let Goal::LinuxV6Startup{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_v6_startup_reference(definition,input);}
    if let State::IdsRule{simulation}=s {let Goal::IdsRule{definition}=goal else{return Err("Rule work requires its authored task.".into());};return crate::ids_rule_source::execute(simulation,definition,input);}
    if let State::AuthFilter{simulation}=s {let Goal::AuthFilter{definition}=goal else{return Err("Filter work requires its authored task.".into());};return crate::auth_filter_source::execute(simulation,definition,input);}
    if let State::VpnPoint{simulation}=s {let Goal::VpnPoint{definition}=goal else{return Err("VPN work requires its authored task.".into());};return crate::vpn_point_source::execute(simulation,definition,input);}
    if let State::Poe{simulation}=s {let Goal::Poe{definition}=goal else{return Err("Power work requires its authored task.".into());};return crate::poe_source::execute(simulation,definition,input);}
    if let State::KernelBuild{simulation}=s {let Goal::KernelBuild{definition}=goal else{return Err("Kernel work requires its authored task.".into());};return crate::kernel_source::execute(simulation,definition,input);}
    if let State::Dkms{simulation}=s {let Goal::Dkms{definition}=goal else{return Err("DKMS work requires its authored task.".into());};return crate::dkms_source::execute(simulation,definition,input);}
    if let State::Initramfs{simulation}=s {let Goal::Initramfs{definition}=goal else{return Err("Early-userspace work requires its authored task.".into());};return crate::initramfs_source::execute(simulation,definition,input);}
    if let State::Pxe{simulation}=s {let Goal::Pxe{definition}=goal else{return Err("network boot work requires its authored task.".into());};return crate::pxe::source::execute(simulation,definition,input);}
    if let State::Iscsi{simulation}=s {let Goal::Iscsi{definition}=goal else{return Err("iSCSI work requires its authored task.".into());};return crate::iscsi::source::execute(simulation,definition,input);}
    if let State::AtaSmart{simulation}=s {let Goal::AtaSmart{definition}=goal else{return Err("ATA SMART work requires its authored task.".into());};return crate::ata_smart::source::execute(simulation,definition,input);}
    if let State::ApacheHttps{simulation}=s {let Goal::ApacheHttps{definition}=goal else{return Err("Apache HTTPS work requires its authored task.".into());};return crate::apache_https::source::execute(simulation,definition,input);}
    if let State::Desktop{simulation}=s {let Goal::Desktop{definition}=goal else{return Err("virtual desktop work requires its authored task.".into());};return crate::desktop::source::execute(simulation,definition,input);}
    if let State::SecurityInvestigation{simulation}=s {let Goal::SecurityInvestigation{definition}=goal else{return Err("virtual security investigation work requires its authored task.".into());};return crate::security_investigation::source::execute(simulation,definition,input);}
    if let State::AutomationWorkshop{simulation}=s {let Goal::AutomationWorkshop{definition}=goal else{return Err("virtual staged network changes work requires its authored task.".into());};return crate::automation_workshop::source::execute(simulation,definition,input);}
    if let State::ServiceExposure{simulation}=s {let Goal::ServiceExposure{definition}=goal else{return Err("Exposure work requires its authored task.".into());};return crate::service_exposure_source::execute(simulation,definition,input);}
    if let State::FtpIdentity{simulation}=s {let Goal::FtpIdentity{definition}=goal else{return Err("FTP work requires its authored task.".into());};return simulation.reference(definition,input);}
    if let State::LinuxFlow{simulation}=s {let Goal::LinuxFlow{definition}=goal else{return Err("Router work requires its authored task.".into());};return simulation.execute_flow_reference(definition,input);}
    if let State::VirtualIsolation{simulation}=s {let Goal::VirtualIsolation{definition}=goal else{return Err("Represented-network work requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::SieveFiltering{simulation}=s {let Goal::SieveFiltering{definition}=goal else{return Err("Filter work requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::MailboxAdministration{simulation}=s {let Goal::MailboxAdministration{definition}=goal else{return Err("Mail administration requires its supplied task.".into());};return simulation.execute_admin_reference(definition,input);}
    if let State::MailboxAccess{simulation}=s {let Goal::MailboxAccess{definition}=goal else{return Err("Mail-access work requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::SmtpRouting{simulation}=s {let Goal::SmtpRouting{definition}=goal else{return Err("Mail work requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::GuestDrivers{simulation}=s {let Goal::GuestDrivers{definition}=goal else{return Err("Guest-image work requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::SambaShare{simulation}=s {let Goal::SambaShare{definition}=goal else{return Err("Sharing requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::CampusArchitecture{simulation}=s {let Goal::CampusArchitecture{definition}=goal else{return Err("Campus layout requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::NetworkTransfer{simulation}=s {let Goal::NetworkTransfer{definition}=goal else{return Err("File transfer requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::EditorHandover{simulation}=s {let Goal::EditorHandover{definition}=goal else{return Err("Editor handover requires its supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::ModalEditing{simulation}=s {let Goal::ModalEditing{definition}=goal else{return Err("Editing actions require the supplied task.".into());};return simulation.execute_reference(definition,input);}
    if let State::SshAccess { simulation }=s {let Goal::SshAccess{definition}=goal else{return Err("SSH actions require the supplied task.".into())};return simulation.execute_reference(definition,input);}
    if matches!(s, State::RaidAssembly { .. }) || matches!(goal, Goal::RaidAssembly { .. }) {
        let (State::RaidAssembly { simulation }, Goal::RaidAssembly { definition }) = (s, goal) else { return Err("RAID assembly work requires its supplied task; previous work retained".into()); };
        return simulation.run(definition, input, false);
    }

    if matches!(s, State::Autofs { .. }) || matches!(goal, Goal::Autofs { .. }) {
        let (State::Autofs { simulation }, Goal::Autofs { definition }) = (s, goal) else { return Err("AutoFS work requires its supplied task; previous work retained".into()); };
        return simulation.run(definition, input, false);
    }

    if matches!(s, State::KernelModules { .. }) || matches!(goal, Goal::KernelModules { .. }) {
        let (State::KernelModules { simulation }, Goal::KernelModules { definition }) = (s, goal)
        else {
            return Err(
                "Module work requires its matching supplied task; original retained.".into(),
            );
        };
        return simulation.action(definition, input, false);
    }
    if matches!(s, State::RecursiveTree { .. }) || matches!(goal, Goal::RecursiveTree { .. }) {
        let (State::RecursiveTree { simulation }, Goal::RecursiveTree { definition }) = (s, goal)
        else {
            return Err(
                "Recursive file work requires its matching supplied task; original retained."
                    .into(),
            );
        };
        return crate::recursive_tree_source::execute(simulation, definition, input);
    }
    if matches!(s, State::Transport { .. }) || matches!(goal, Goal::Transport { .. }) {
        let (State::Transport { simulation }, Goal::Transport { definition }) = (s, goal) else {
            return Err(
                "Transport work requires its matching supplied task; original retained.".into(),
            );
        };
        return simulation.execute(definition, input);
    }
    if matches!(s, State::DirectoryPolicy { .. }) || matches!(goal, Goal::DirectoryPolicy { .. }) {
        let (State::DirectoryPolicy { simulation }, Goal::DirectoryPolicy { definition }) =
            (s, goal)
        else {
            return Err("Directory-policy work requires its matching supplied task; previous work retained.".into());
        };
        return simulation.run(definition, input, false);
    }
    // Refuse either wrong-family direction before generic help/show dispatch.
    if matches!(s, State::FileIndex { .. }) || matches!(goal, Goal::FileIndex { .. }) {
        let (State::FileIndex { simulation }, Goal::FileIndex { definition }) = (s, goal) else {
            return Err(
                "Private-index work requires its supplied task; previous work retained".into(),
            );
        };
        return crate::file_index_source::execute(simulation, definition, input);
    }
    if matches!(goal,Goal::Filesystem {requirements} if crate::filesystem::discovery_policy::has_reference(requirements))
        && !matches!(s, State::Filesystem { .. })
    {
        return Err("Command-location objectives require their supplied filesystem activity; previous work retained".into());
    }
    if let State::Filesystem { tree } = s {
        if let Goal::Filesystem { requirements } = goal {
            if let Some(definition) =
                crate::filesystem::discovery_policy::for_current(requirements, tree)?
            {
                return definition.execute(tree, input);
            }
        } else if tree.command_location.is_some() {
            return Err("Command-location work needs its supplied filesystem task".into());
        }
    }
    if let State::SharedLibraries { simulation } = s {
        let Goal::SharedLibraries { definition } = goal else {
            return Err("Library work requires its authored task; no host utility ran".into());
        };
        return simulation.execute_reference(definition, input);
    }

    if let State::Raid1 { simulation } = s {
        let Goal::Raid1 { definition } = goal else {
            return Err("Mirror work requires its supplied assignment".into());
        };
        let result = if let Some(raw) = input.strip_prefix(crate::raid1_workspace::PREFIX) {
            if raw.len() > 2048 {
                return Err("Mirror action exceeds its encoded bound".into());
            }
            let action = serde_json::from_str(raw)
                .map_err(|e| format!("Invalid native mirror action: {e}"))?;
            simulation.action(definition, &action, false)?
        } else {
            simulation.execute(definition, input, false)?
        };
        if result.status != 0 {
            return Err(result.output);
        }
        return Ok(result.output);
    }

    if let State::StartupFiles { simulation } = s {
        let Goal::StartupFiles { definition } = goal else {
            return Err("Startup files require their authored task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::FsIntegrity { simulation } = s {
        let Goal::FsIntegrity { definition } = goal else {
            return Err("Integrity work requires its supplied assignment".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::UserMounts { simulation } = s {
        let Goal::UserMounts { definition } = goal else {
            return Err("Caller-aware mounts require their supplied assignment".into());
        };
        simulation.validate(definition)?;
        let raw = input
            .strip_prefix(crate::user_mounts_workspace::PREFIX)
            .ok_or("Use the native mount workspace or its declared action codec")?;
        if raw.len() > 65_536 {
            return Err("Mount action exceeds its encoded JSON bound".into());
        }
        let action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid simulated mount action: {e}"))?;
        return simulation
            .assignment
            .apply(&definition.assignment, action)
            .map(|effect| effect.explanation().into());
    }
    if let State::Environment { simulation } = s {
        let Goal::Environment { definition } = goal else {
            return Err("Setup scripts require their supplied environment task".into());
        };
        simulation.validate(definition)?;
        simulation.assignment.edit(input.into())?;
        return Ok("Setup saved; Run setup + child checks the actual inherited settings.".into());
    }

    if let State::DebianSources { simulation } = s {
        let Goal::DebianSources { definition } = goal else {
            return Err("Source editing needs its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }

    if let State::ClientNames { simulation } = s {
        let Goal::ClientNames { definition } = goal else {
            return Err("Configuration testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::RemovableMounts { simulation } = s {
        let Goal::RemovableMounts { definition } = goal else {
            return Err("Configuration testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::NmProfiles { simulation } = s {
        let Goal::NmProfiles { definition } = goal else {
            return Err("Configuration testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::PamStack { simulation } = s {
        let Goal::PamStack { definition } = goal else {
            return Err("Configuration testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::Aging { simulation } = s {
        let Goal::Aging { definition } = goal else {
            return Err("Aging testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::PortSecurity { simulation } = s {
        let Goal::PortSecurity { definition } = goal else {
            return Err("PortSecurity testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::DhcpSnooping { simulation } = s {
        let Goal::DhcpSnooping { definition } = goal else {
            return Err("DhcpSnooping testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::ArpInspection { simulation } = s {
        let Goal::ArpInspection { definition } = goal else {
            return Err("ArpInspection testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::ManagementPolicy { simulation } = s {
        let Goal::ManagementPolicy { definition } = goal else {
            return Err("ManagementPolicy testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LocalAccess { simulation } = s {
        let Goal::LocalAccess { definition } = goal else {
            return Err("LocalAccess testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::AaaAccess { simulation } = s {
        let Goal::AaaAccess { definition } = goal else {
            return Err("AaaAccess testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LdapClient { simulation } = s {
        let Goal::LdapClient { definition } = goal else {
            return Err("LdapClient testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LdapAccess { simulation } = s {
        let Goal::LdapAccess { definition } = goal else {
            return Err("LdapAccess testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LdapSchema { simulation } = s {
        let Goal::LdapSchema { definition } = goal else {
            return Err("LdapSchema testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::CopiedJournal { simulation } = s {
        let Goal::CopiedJournal { definition } = goal else {
            return Err("Copied-journal checks need the supplied task.".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::JsonInventory { simulation } = s {
        let Goal::JsonInventory { definition } = goal else { return Err("JSON inventory checks need the supplied task.".into()); };
        return simulation.execute_reference(definition, input);
    }
    if let State::RestController { simulation } = s {
        let Goal::RestController { definition } = goal else { return Err("REST controller checks need the supplied task.".into()); };
        return simulation.execute_reference(definition, input);
    }
    if let State::DevicePath { simulation } = s {
        let Goal::DevicePath { definition } = goal else { return Err("Device-path checks need the supplied task.".into()); };
        return simulation.execute_reference(definition, input);
    }
    if let State::LocalMail { simulation } = s {
        let Goal::LocalMail { definition } = goal else {
            return Err("Local-mail checks need the supplied task.".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::Chrony { simulation } = s {
        let Goal::Chrony { definition } = goal else {
            return Err("Time-source checks need the supplied task.".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::GrubOnce { simulation } = s {
        let Goal::GrubOnce { definition } = goal else {
            return Err("Temporary GRUB tests need the supplied task.".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::EfiLoader { simulation } = s {
        let Goal::EfiLoader { definition } = goal else {
            return Err("EFI installation tests need the supplied task.".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::BiosLoader { simulation } = s {
        let Goal::BiosLoader { definition } = goal else { return Err("BIOS installation tests need the supplied task.".into()); };
        return simulation.execute(definition,input);
    }
    if let State::BootEntry { simulation } = s {
        let Goal::BootEntry { definition } = goal else { return Err("Boot-entry tests need the supplied task.".into()); };
        return simulation.execute_reference(definition,input);
    }
    if let State::LdapSchemaQuery { simulation } = s {
        let Goal::LdapSchemaQuery { definition } = goal else {
            return Err("LdapSchemaQuery testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LdapPassword { simulation } = s {
        let Goal::LdapPassword { definition } = goal else {
            return Err("LdapPassword testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LdapChanges { simulation } = s {
        let Goal::LdapChanges { definition } = goal else {
            return Err("LdapChanges testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::DhcpConfig { simulation } = s {
        let Goal::DhcpConfig { definition } = goal else {
            return Err("Configuration testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::Qos { simulation } = s {
        let Goal::Qos { definition } = goal else {
            return Err("Traffic policy testing requires its supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::LinuxLogging { simulation } = s {
        let Goal::LinuxLogging { definition } = goal else {
            return Err("Selector testing requires the supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::HostNetwork { simulation } = s {
        let Goal::HostNetwork { definition } = goal else {
            return Err("Host commands require the supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::Repositories { simulation } = s {
        let Goal::Repositories { definition } = goal else {
            return Err("Repository operations require the supplied task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::DnsAccess { simulation } = s {
        let Goal::DnsAccess { definition } = goal else {return Err("DNS-policy operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::PrintQueues { simulation } = s {
        let Goal::PrintQueues { definition } = goal else {return Err("Print-queue operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::Localization { simulation } = s {
        let Goal::Localization { definition } = goal else {return Err("Locale/encoding operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::WirelessChannels { simulation } = s {
        let Goal::WirelessChannels { definition } = goal else { return Err("Channel planning requires its authored task".into()) };
        return simulation.execute_reference(definition,input);
    }
    if let State::WlanProfiles { simulation } = s {
        let Goal::WlanProfiles { definition } = goal else { return Err("WLAN configuration requires its authored task".into()) };
        return simulation.execute_reference(definition,input);
    }
    if let State::HttpCache { simulation } = s {
        let Goal::HttpCache { definition } = goal else {return Err("Cache replay requires its authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::ServiceClients { simulation } = s {
        let Goal::ServiceClients { definition } = goal else {return Err("Service clients require their authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::ReverseProxy { simulation } = s {
        let Goal::ReverseProxy { definition } = goal else {return Err("Reverse-proxy routing requires its authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::ProxyPolicy { simulation } = s {
        let Goal::ProxyPolicy { definition } = goal else {return Err("Proxy policy requires its authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::SnmpMonitor { simulation } = s {
        let Goal::SnmpMonitor { definition } = goal else {return Err("Monitoring configuration requires its authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::WlanPaths { simulation } = s {
        let Goal::WlanPaths { definition } = goal else { return Err("AP-path configuration requires its authored task".into()) };
        return simulation.execute_reference(definition,input);
    }
    if let State::Timezone { simulation } = s {
        let Goal::Timezone { definition } = goal else {return Err("Time-zone operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::SourceInstall { simulation } = s {
        let Goal::SourceInstall { definition } = goal else {return Err("Source-install operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::MaintenanceNotice { simulation } = s {
        let Goal::MaintenanceNotice { definition } = goal else {return Err("Notice operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::CapacityPlanning { simulation } = s {
        let Goal::CapacityPlanning { definition } = goal else {return Err("Planning operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::ResourceObservations { simulation } = s {
        let Goal::ResourceObservations { definition } = goal else {return Err("Observation operations require the authored task".into())};
        return simulation.execute_reference(definition,input);
    }
    if let State::Capacity { simulation } = s {
        let Goal::Capacity { definition } = goal else {
            return Err("Capacity operations require their authored task".into());
        };
        return simulation.execute_reference(definition, input);
    }
    if let State::Links { simulation } = s {
        let Goal::Links { definition } = goal else {
            return Err("Link commands require the authored task".into());
        };
        return simulation.run(definition, input, false);
    }
    if matches!(s, State::DebianSources { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err("Use the native simulated source editor; no host APT or command runs".into()),
        };
    }
    if matches!(s, State::ClientNames { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated name-source editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::RemovableMounts { .. }) {
        return match input {
            "help" => Ok(crate::removable_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated mount-policy editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::NmProfiles { .. }) {
        return match input {
            "help" => Ok(crate::nm_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated network profile editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::DhcpConfig { .. }) {
        return match input{"help"=>Ok(crate::dhcp_server_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native simulated DHCP configuration editor; no host daemon or command runs".into())};
    }
    if matches!(s, State::Qos { .. }) {
        return match input {
            "help" => Ok(crate::qos::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native traffic policy controls; no host networking or IOS command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::LinuxLogging { .. }) {
        return match input {
            "help" => Ok(crate::linux_logging::HELP.into()),
            "show" => Ok(describe(s)),
            _ => {
                Err("Use the native simulated selector editor; no host logging commands run".into())
            }
        };
    }
    if matches!(s, State::HostNetwork { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err("Use this task’s simulated terminal and virtual resolver editor".into()),
        };
    }
    if matches!(s, State::Repositories { .. }) {
        return match input{"help"=>Ok(crate::repository_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native simulated repository editor/query controls; no host package command runs".into())};
    }
    if matches!(s, State::MaintenanceNotice { .. }) {
        return match input {"help"=>Ok(crate::maintenance_notice_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Notice commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::DnsAccess { .. }) {
        return match input {"help"=>Ok(crate::dns_access_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("DNS-policy commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::PrintQueues { .. }) {
        return match input {"help"=>Ok(crate::print_queue_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Print-queue commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::Localization { .. }) {
        return match input {"help"=>Ok(crate::localization_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Locale/encoding commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::WirelessChannels { .. }) {
        return match input {"help"=>Ok(crate::wireless_channel_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Channel planning requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::WlanProfiles { .. }) {
        return match input {"help"=>Ok(crate::wlan_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("WLAN configuration requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::HttpCache { .. }) {
        return match input {"help"=>Ok(crate::http_cache_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Cache policy requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::ServiceClients { .. }) {
        return match input {"help"=>Ok(crate::service_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the supplied service workspace; no host command runs".into())};
    }
    if matches!(s, State::ReverseProxy { .. }) {
        return match input {"help"=>Ok(crate::reverse_proxy_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Reverse-proxy routing requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::ProxyPolicy { .. }) {
        return match input {"help"=>Ok(crate::proxy_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Proxy configuration requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::SnmpMonitor { .. }) {
        return match input {"help"=>Ok(crate::snmp_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Monitoring requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::WlanPaths { .. }) {
        return match input {"help"=>Ok(crate::wlan_paths_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("AP-path configuration requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::Timezone { .. }) {
        return match input {"help"=>Ok(crate::timezone_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Time-zone commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::SourceInstall { .. }) {
        return match input {"help"=>Ok(crate::source_install_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Source-install commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::CapacityPlanning { .. }) {
        return match input {"help"=>Ok(crate::capacity_planning_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native capacity-planning controls; all operations are simulated".into())};
    }
    if matches!(s, State::ResourceObservations { .. }) {
        return match input {"help"=>Ok(crate::resource_observation_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the recorded-host terminal and report controls; no host command runs".into())};
    }
    if matches!(s, State::Capacity { .. }) {
        return match input {
            "help" => Ok(crate::capacity_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native virtual file table and editor; no host commands run here".into(),
            ),
        };
    }
    if matches!(s, State::PartitionFs { .. }) {
        let (State::PartitionFs { simulation }, Goal::PartitionFs { definition }) = (&*s, goal)
        else {
            return Err("Disk preparation requires its supplied task".into());
        };
        definition.bound(simulation)?;
        return execute(s, input);
    }
    if let State::Hsrp { simulation } = s {
        let Goal::Hsrp { definition } = goal else {
            return Err("HSRP changes require the authored task".into());
        };
        return simulation.execute(definition, input, false);
    }
    if let State::Syslog { simulation } = s {
        let Goal::Syslog { definition } = goal else {
            return Err("Syslog actions require their authored Goal".into());
        };
        return crate::syslog_workspace::execute_authored(simulation, definition, input);
    }
    if let State::Ntp { simulation } = s {
        let Goal::Ntp { definition } = goal else {
            return Err("NTP commands require the supplied task".into());
        };
        return simulation.execute(definition, input, false);
    }
    if let State::Lvm { simulation } = s {
        let Goal::Lvm { definition } = goal else {
            return Err("LVM commands require their authored Goal".into());
        };
        return simulation.execute(definition, input, false);
    }
    if let State::PhysicalLink { simulation } = s {
        let Goal::PhysicalLink { definition } = goal else {
            return Err("Physical-link actions need their authored task".into());
        };
        return crate::physical_link_workspace::execute(definition, simulation, input);
    }
    if let State::FileBackup { simulation } = s {
        if input.len() > 4096 {
            return Err(
                "Use one command of at most4096bytes; previous backup work retained".into(),
            );
        }
        let Goal::FileBackup { definition } = goal else {
            return Err("Backup commands require their supplied task".into());
        };
        definition.validate()?;
        if !definition.bound_state(simulation) {
            return Err(
                "This backup workspace does not match the task; previous work retained".into(),
            );
        }
        return simulation.execute(input);
    }
    if let State::EtherChannel { simulation } = s {
        let Goal::EtherChannel { definition } = goal else {
            return Err("LACP internal action needs authored Goal context".into());
        };
        return crate::etherchannel_workspace::execute(definition, simulation, input);
    }

    if let State::OneOff { simulation } = s {
        let Goal::OneOff { definition } = goal else {
            return Err("One-off authored goal differs".into());
        };
        return crate::one_off_workspace::execute(definition, simulation, input);
    }
    if let State::Cron { simulation } = s {
        let Goal::Cron { definition } = goal else {
            return Err("Scheduling needs its authored context".into());
        };
        return crate::cron_workspace::execute(definition, simulation, input);
    }
    if let State::Nfs { simulation } = s {
        if input.len() > crate::nfs_workspace::PREFIX.len() + crate::nfs::MAX_ACTION {
            return Err("NFS typed input exceeds raw bound before trimming".into());
        }
        let Goal::Nfs { definition } = goal else {
            return Err("NFS action needs its authored context".into());
        };
        return crate::nfs_workspace::execute(definition, simulation, input);
    }
    if let State::RpmPackages { simulation } = s {
        if input.len() > crate::rpm_packages_terminal::MAX_COMMAND {
            return Err("RPM input exceeds2048 bytes before any trimming".into());
        }
        let Goal::RpmPackages { definition } = goal else {
            return Err("RPM assignment requires its authored context".into());
        };
        return crate::rpm_packages_terminal::output_text(crate::rpm_packages_terminal::execute(
            definition, simulation, input,
        ));
    }
    if let State::BasicNat { simulation } = s {
        if input.len() > crate::nat_workspace::PREFIX.len() + crate::nat::MAX_ACTION_JSON {
            return Err("Typed NAT input exceeds raw bound".into());
        }
        let Goal::BasicNat { definition } = goal else {
            return Err("NAT action needs authored inventory context".into());
        };
        if !definition.same_assignment(simulation) {
            return Err("Foreign or oversized NAT context".into());
        }
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::nat_workspace::execute(&definition.initial, simulation, input);
    }
    if let State::ApacheStatic { simulation } = s {
        if input.len()
            > crate::apache_static_workspace::PREFIX.len() + crate::apache_static::MAX_ACTION_JSON
        {
            return Err("Typed Apache input exceeds raw bound".into());
        }
        let Goal::ApacheStatic { definition } = goal else {
            return Err("Apache action needs fixed assignment context".into());
        };
        if !definition.same_assignment(simulation) {
            return Err("Foreign/oversized Apache assignment; original retained".into());
        }
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::apache_static_workspace::execute(&definition.initial, simulation, input);
    }

    if let State::DebianPackages { simulation } = s {
        if input.len() > crate::debian_packages_workspace::MAX_ACTION_BYTES {
            return Err("Package authoring input exceeds16KiB".into());
        }
        if !input.starts_with("packages ")
            && input.len() > crate::debian_packages_terminal::MAX_COMMAND_BYTES
        {
            return Err("Package command exceeds2048 UTF8 bytes".into());
        }
        let Goal::DebianPackages { definition } = goal else {
            return Err("Logical package actions need immutable assignment context".into());
        };
        if !definition.same_assignment(simulation) {
            return Err("Foreign/oversized package assignment; original retained".into());
        }
        if matches!(input.trim(), "show" | "help") {
            return execute(s, input);
        }
        if input.starts_with("packages ") {
            return crate::debian_packages_workspace::execute(definition, simulation, input);
        }
        return crate::debian_packages_terminal::output_text(
            crate::debian_packages_terminal::execute(definition, simulation, input),
        );
    }
    if let State::SpanningTree { simulation } = s {
        if input.len()
            > crate::spanning_tree_workspace::PREFIX.len() + crate::spanning_tree::MAX_ACTION_BYTES
        {
            return Err("Spanning-tree typed input exceeds64KiB plus its fixed prefix".into());
        }
        let Goal::SpanningTree {
            initial,
            requirements,
        } = goal
        else {
            return Err("Spanning-tree changes require fixed assignment context".into());
        };
        if !crate::spanning_tree_workspace::valid_requirements(initial, requirements)
            || !crate::spanning_tree_workspace::same_context(initial, simulation)
        {
            return Err(
                "Invalid or foreign spanning-tree state/objectives; prior state retained".into(),
            );
        }
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::spanning_tree_workspace::execute(initial, simulation, input);
    }

    if let State::DhcpV4 { simulation } = s {
        if input.len() > crate::dhcpv4_workspace::MAX_ACTION_BYTES {
            return Err("DHCP authoring action exceeds256KiB".into());
        }
        let Goal::DhcpV4 {
            initial,
            requirements,
            probes,
        } = goal
        else {
            return Err("DHCP mutations need immutable assignment context".into());
        };
        if !crate::dhcpv4_workspace::valid_goal(initial, requirements, probes) {
            return Err("Invalid authored DHCP goals; state retained".into());
        }
        simulation
            .validate_context(initial.topology())
            .map_err(|e| crate::dhcpv4_workspace::notice(e.message))?;
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::dhcpv4_workspace::execute(initial, simulation, input);
    }
    if let State::UnitManager { simulation } = s {
        let Goal::UnitManager { definition } = goal else {
            return Err("Virtual unit actions require a matching fixed assignment".into());
        };
        if !definition.same_assignment(simulation) {
            return Err(
                "Invalid or foreign virtual unit goal/state; previous work retained".into(),
            );
        }
        if input.len() > crate::unit_manager_workspace::MAX_ACTION_BYTES {
            return Err("Virtual unit authoring action exceeds its bounded size".into());
        }
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::unit_manager_workspace::adapter(&definition.initial, simulation, input);
    }
    if let State::Ospf { simulation } = s {
        let Goal::Ospf {
            initial,
            requirements,
        } = goal
        else {
            return Err("OSPF changes require a matching immutable goal context".into());
        };
        if !crate::ospf_workspace::valid_requirements(initial, requirements) {
            return Err("Invalid authored OSPF requirements; state retained".into());
        }
        if matches!(input.trim(), "help" | "show") {
            return execute(s, input);
        }
        return crate::ospf_workspace::execute(initial, simulation, input);
    }
    if let State::Accounts { simulation } = s {
        let Goal::Accounts {
            initial,
            requirements,
        } = goal
        else {
            return Err("Account changes require a matching accounts goal context".into());
        };
        if !crate::accounts::valid_requirements(initial, requirements) {
            return Err("Invalid authored account requirements; previous state retained".into());
        }
        return crate::accounts_teaching::output_text(
            crate::accounts_teaching::execute_in_context(initial, simulation, input)?,
        );
    }
    execute(s, input)
}
/// The parser only mutates typed values. It has no OS, shell, filesystem or network APIs.
pub fn execute(s: &mut State, input: &str) -> Result<String, String> {
    if matches!(s,State::LinuxRouter{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::LinuxV6{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::LinuxServiceRouter{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::LinuxV6Administration{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::LinuxV6Startup{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::IdsRule{..}){return Err("Use the source-bound native rule workspace; no host command is executed.".into());}
    if matches!(s,State::AuthFilter{..}){return Err("Use the source-bound native filter workspace; no host command is executed.".into());}
    if matches!(s,State::VpnPoint{..}){return Err("Use the source-bound native peer workspace; no host command is executed.".into());}
    if matches!(s,State::Poe{..}){return Err("Use the source-bound native power workspace; no host command is executed.".into());}
    if matches!(s,State::KernelBuild{..}){return Err("Use the source-bound native kernel workspace; no host command is executed.".into());}
    if matches!(s,State::Dkms{..}){return Err("Use the source-bound native DKMS workspace; no host command is executed.".into());}
    if matches!(s,State::Initramfs{..}){return Err("Use the source-bound native Early-userspace workspace; no host command is executed.".into());}
    if matches!(s,State::Pxe{..}){return Err("Use the source-bound native network boot workspace; no host command is executed.".into());}
    if matches!(s,State::Iscsi{..}){return Err("Use the source-bound native iSCSI workspace; no host command is executed.".into());}
    if matches!(s,State::AtaSmart{..}){return Err("Use the source-bound native ATA SMART workspace; no host command is executed.".into());}
    if matches!(s,State::ApacheHttps{..}){return Err("Use the source-bound native Apache HTTPS workspace; no host command is executed.".into());}
    if matches!(s,State::Desktop{..}){return Err("Use the source-bound native virtual desktop workspace; no host command is executed.".into());}
    if matches!(s,State::SecurityInvestigation{..}){return Err("Use the source-bound native virtual security investigation workspace; no host command is executed.".into());}
    if matches!(s,State::AutomationWorkshop{..}){return Err("Use the source-bound native virtual staged network changes workspace; no host command is executed.".into());}
    if matches!(s,State::ServiceExposure{..}){return Err("Use the source-bound native exposure workspace; no host command is executed.".into());}
    if matches!(s,State::FtpIdentity{..}){return Err("Use the source-bound native FTP workspace; no host command is executed.".into());}
    if matches!(s,State::LinuxFlow{..}){return Err("Use the source-bound native router workspace; no host command is executed.".into());}
    if matches!(s,State::VirtualIsolation{..}){return Err("Use the native represented-network controls; no host command is executed.".into());}
    if matches!(s,State::SieveFiltering{..}){return Err("Use the source-bound native filter workspace; no host command is executed.".into());}
    if matches!(s,State::MailboxAdministration{..}){return Err("Use the source-bound native administration workspace; no host command is executed.".into());}
    if matches!(s,State::MailboxAccess{..}){return Err("Use the source-bound native mail-access workspace; no host command is executed.".into());}
    if matches!(s,State::SmtpRouting{..}){return Err("Mail actions require their supplied task; no host command is executed.".into());}
    if matches!(s,State::GuestDrivers{..}){return Err("Guest-image actions require their supplied task; no host command is executed.".into());}
    if matches!(s,State::SambaShare{..}){return Err("Sharing uses its source-bound virtual files and native controls.".into());}
    if matches!(s,State::CampusArchitecture{..}){return Err("Campus layout uses its source-bound virtual files and native controls.".into());}
    if matches!(s,State::NetworkTransfer{..}){return Err("File transfer uses its source-bound virtual files and native controls.".into());}
    if matches!(s,State::EditorHandover{..}){return Err("Editor handover uses its source-bound setup and native editor.".into());}
    if matches!(s,State::ModalEditing{..}){return Err("Modal editing uses its source-bound native editor.".into());}
    if matches!(s,State::SshAccess{..}){return Err("SSH configuration uses its source-bound native workspace.".into());}
    if matches!(s, State::FileIndex { .. }) {
        return Err("Private-index work requires its supplied task; no host utility ran".into());
    }
    if matches!(s, State::SharedLibraries { .. }) {
        return Err("Library operations require their authored task; no host utility ran".into());
    }
    if matches!(s, State::Raid1 { .. }) {
        return Err(
            "Mirror operations require their authored assignment; no host command ran".into(),
        );
    }

    if matches!(s, State::StartupFiles { .. }) {
        return Err(
            "Use the native startup-file editor and its authored task; no host command was run"
                .into(),
        );
    }
    if let State::FsIntegrity { simulation } = s {
        let definition = simulation.definition.clone();
        return simulation.execute_reference(&definition, input);
    }
    if let State::UserMounts { simulation } = s {
        let raw = input
            .strip_prefix(crate::user_mounts_workspace::PREFIX)
            .ok_or("Use the native mount workspace or its declared action codec")?;
        if raw.len() > 65_536 {
            return Err("Mount action exceeds its encoded JSON bound".into());
        }
        let action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid simulated mount action: {e}"))?;
        return simulation
            .assignment
            .apply(&simulation.definition.assignment, action)
            .map(|effect| effect.explanation().into());
    }
    if let State::Environment { simulation } = s {
        simulation.assignment.edit(input.into())?;
        return Ok("Setup saved; use the native Run setup + child action.".into());
    }

    if matches!(s, State::RaidAssembly { .. }) {
        return match input.trim() { "help"=>Ok(help(s).into()), "show"=>Ok(describe(s)), _=>Err("RAID assembly commands need their supplied task; use the native workspace".into()) };
    }
    if matches!(s, State::Autofs { .. }) {
        return match input.trim() { "help"=>Ok(help(s).into()), "show"=>Ok(describe(s)), _=>Err("AutoFS commands need their supplied task; use the native workspace".into()) };
    }
    if matches!(s, State::KernelModules { .. }) {
        return match input.trim() {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => {
                Err("Module operations need their supplied task; use the native workspace.".into())
            }
        };
    }
    if matches!(s, State::RecursiveTree { .. }) {
        return match input.trim() {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err("Recursive file work needs its supplied task; original retained.".into()),
        };
    }
    if matches!(s, State::Transport { .. }) {
        return match input.trim() {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Transport settings require their supplied task; use the native workspace.".into(),
            ),
        };
    }
    if matches!(s, State::DirectoryPolicy { .. }) {
        return match input.trim() {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Directory operations require the supplied task; use the simulated terminal."
                    .into(),
            ),
        };
    }
    if matches!(s, State::Links { .. }) {
        return match input.trim() {
            "help" => Ok(crate::links_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("Link commands require the supplied goal; use the simulated terminal".into()),
        };
    }
    if let State::MagicSubnet { session } = s {
        // Declarative pack reachability adapter; the learner uses native fields.
        let words: Vec<_> = input.split_whitespace().collect();
        if let ["set", index, value] = words.as_slice() {
            let i = index
                .parse::<usize>()
                .map_err(|_| "Invalid worksheet field")?;
            if i >= session.stage.fields(session.problem)?.len() || value.len() > 64 {
                return Err("Invalid worksheet field/value".into());
            }
            session.answers[i] = value.to_string();
            return Ok("Worksheet value updated".into());
        }
        return Err("Use the native worksheet fields; no shell runs here".into());
    }

    if matches!(s, State::DebianSources { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err("Use the native simulated source editor; no host APT or command runs".into()),
        };
    }
    if matches!(s, State::ClientNames { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated name-source editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::RemovableMounts { .. }) {
        return match input {
            "help" => Ok(crate::removable_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated mount-policy editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::NmProfiles { .. }) {
        return match input {
            "help" => Ok(crate::nm_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native simulated network profile editor; no host daemon or command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::DhcpConfig { .. }) {
        return match input{"help"=>Ok(crate::dhcp_server_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native simulated DHCP configuration editor; no host daemon or command runs".into())};
    }
    if matches!(s, State::Qos { .. }) {
        return match input {
            "help" => Ok(crate::qos::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native traffic policy controls; no host networking or IOS command runs"
                    .into(),
            ),
        };
    }
    if matches!(s, State::LinuxLogging { .. }) {
        return match input {
            "help" => Ok(crate::linux_logging::HELP.into()),
            "show" => Ok(describe(s)),
            _ => {
                Err("Use the native simulated selector editor; no host logging commands run".into())
            }
        };
    }
    if matches!(s, State::HostNetwork { .. }) {
        return match input {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err("Use this task’s simulated terminal and virtual resolver editor".into()),
        };
    }
    if matches!(s, State::Repositories { .. }) {
        return match input{"help"=>Ok(crate::repository_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native simulated repository editor/query controls; no host package command runs".into())};
    }
    if matches!(s, State::MaintenanceNotice { .. }) {
        return match input {"help"=>Ok(crate::maintenance_notice_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Notice commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::DnsAccess { .. }) {
        return match input {"help"=>Ok(crate::dns_access_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("DNS-policy commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::PrintQueues { .. }) {
        return match input {"help"=>Ok(crate::print_queue_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Print-queue commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::Localization { .. }) {
        return match input {"help"=>Ok(crate::localization_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Locale/encoding commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::WirelessChannels { .. }) {
        return match input {"help"=>Ok(crate::wireless_channel_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Channel planning requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::WlanProfiles { .. }) {
        return match input {"help"=>Ok(crate::wlan_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("WLAN configuration requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::HttpCache { .. }) {
        return match input {"help"=>Ok(crate::http_cache_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Cache policy requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::ServiceClients { .. }) {
        return match input {"help"=>Ok(crate::service_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the supplied service workspace; no host command runs".into())};
    }
    if matches!(s, State::ReverseProxy { .. }) {
        return match input {"help"=>Ok(crate::reverse_proxy_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Reverse-proxy routing requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::ProxyPolicy { .. }) {
        return match input {"help"=>Ok(crate::proxy_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Proxy configuration requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::SnmpMonitor { .. }) {
        return match input {"help"=>Ok(crate::snmp_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Monitoring requires its native workspace and authored task; no host command or network request runs".into())};
    }
    if matches!(s, State::WlanPaths { .. }) {
        return match input {"help"=>Ok(crate::wlan_paths_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("AP-path configuration requires native controls and its authored task; no host command runs".into())};
    }
    if matches!(s, State::Timezone { .. }) {
        return match input {"help"=>Ok(crate::timezone_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Time-zone commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::SourceInstall { .. }) {
        return match input {"help"=>Ok(crate::source_install_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Source-install commands require their authored source; no host command runs".into())};
    }
    if matches!(s, State::CapacityPlanning { .. }) {
        return match input {"help"=>Ok(crate::capacity_planning_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the native capacity-planning controls; all operations are simulated".into())};
    }
    if matches!(s, State::ResourceObservations { .. }) {
        return match input {"help"=>Ok(crate::resource_observation_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use the recorded-host terminal and report controls; no host command runs".into())};
    }
    if matches!(s, State::Capacity { .. }) {
        return match input {
            "help" => Ok(crate::capacity_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use the native virtual file table and editor; no host commands run here".into(),
            ),
        };
    }
    if matches!(s, State::PartitionFs { .. }) {
        return match input { "help" => Ok(crate::partition_fs_workspace::HELP.into()), "show" => Ok(describe(s)), _ => Err("Use the virtual disk table and explicit configuration actions; this workspace does not run parted, mkfs, mount or host disk commands".into()) };
    }
    if matches!(s, State::Hsrp { .. }) {
        return match input {
            "help" => Ok(crate::hsrp::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Use native HSRP configuration controls; changes require the authored Goal".into(),
            ),
        };
    }
    if matches!(s, State::Syslog { .. }) {
        return match input {
            "help" => Ok(crate::syslog_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("Use this activity's native settings and event controls".into()),
        };
    }
    if matches!(s, State::Ntp { .. }) {
        return match input {
            "help" => Ok(crate::ntp::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("NTP changes require their authored Goal".into()),
        };
    }
    if let State::Lvm { simulation } = s {
        return match input {
            "help" => Ok(simulation.current.command_help().into()),
            "show" => Ok(describe(s)),
            _ => Err("LVM changes require the supplied task context".into()),
        };
    }
    if matches!(s, State::PhysicalLink { .. }) {
        if input.len()
            > crate::physical_link_workspace::PREFIX.len() + crate::physical_link::MAX_ACTION
        {
            return Err("Physical-link raw action exceeds its bound".into());
        }
        return match input.trim() {
            "help" => Ok(help(s).into()), "show" => Ok(describe(s)),
            _ => Err("Physical-link changes need their supplied task; use native configuration tools. No host or IOS command runs".into()),
        };
    }
    if matches!(s, State::FileBackup { .. }) {
        if input.len() > 4096 {
            return Err("Backup command exceeds4096bytes".into());
        }
        return match input.trim() {
            "help" => Ok(help(s).into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Backup commands require the supplied task context; no host command runs".into(),
            ),
        };
    }
    if matches!(s, State::EtherChannel { .. }) {
        if input.len()
            > crate::etherchannel_workspace::PREFIX.len() + crate::etherchannel::MAX_ACTION_BYTES
        {
            return Err("LACP raw input bound before trimming".into());
        }
        return match input.trim() {
            "help" => Ok(crate::etherchannel_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "LACP mutation needs authored context; use native editors, not host/IOS commands"
                    .into(),
            ),
        };
    }

    if matches!(s, State::OneOff { .. }) {
        if input.len()
            > crate::one_off_workspace::PREFIX.len() + crate::one_off_workspace::MAX_ACTION
        {
            return Err("One-off input exceeds raw bound before trimming".into());
        }
        return match input.trim() {
            "help" => Ok(crate::one_off_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("One-off changes require authored context; no host command runs".into()),
        };
    }
    if matches!(s, State::Cron { .. }) {
        if input.len() > crate::cron_workspace::PREFIX.len() + crate::cron_workspace::MAX_ACTION {
            return Err("Scheduling input exceeds raw bound".into());
        }
        return match input.trim() {
            "help" => Ok(crate::cron_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Scheduling changes require authored virtual context; no host command runs".into(),
            ),
        };
    }
    if matches!(s, State::Nfs { .. }) {
        if input.len() > crate::nfs_workspace::PREFIX.len() + crate::nfs::MAX_ACTION {
            return Err("NFS typed input exceeds raw bound before trimming".into());
        }
        return match input.trim() {
            "help" => Ok(crate::nfs_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "NFS mutations need goal-aware supplied context; no host NFS command runs".into(),
            ),
        };
    }
    if matches!(s, State::RpmPackages { .. }) {
        return Err(
            "RPM commands require goal-aware supplied archive context; no host execution".into(),
        );
    }
    if matches!(s, State::BasicNat { .. }) {
        if input.len() > crate::nat_workspace::PREFIX.len() + crate::nat::MAX_ACTION_JSON {
            return Err("Typed NAT input exceeds raw bound".into());
        }
        return match input.trim() {
            "help" => Ok(crate::nat_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("NAT mutation requires fixed goal context; no router command runs".into()),
        };
    }
    if matches!(s, State::ApacheStatic { .. }) {
        if input.len()
            > crate::apache_static_workspace::PREFIX.len() + crate::apache_static::MAX_ACTION_JSON
        {
            return Err("Typed Apache input exceeds raw bound".into());
        }
        return match input.trim() {
            "help" => Ok(crate::apache_static_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Apache mutations require fixed goal context; no real server command runs".into(),
            ),
        };
    }

    if matches!(s, State::DebianPackages { .. }) {
        if input.len() > crate::debian_packages_workspace::MAX_ACTION_BYTES {
            return Err("Package authoring input exceeds16KiB".into());
        }
        return match input.trim() {
            "help" => Ok(crate::debian_packages_terminal::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err(
                "Package commands need fixed goal context; no host package program is run".into(),
            ),
        };
    }
    if matches!(s, State::SpanningTree { .. }) {
        if input.len()
            > crate::spanning_tree_workspace::PREFIX.len() + crate::spanning_tree::MAX_ACTION_BYTES
        {
            return Err("Spanning-tree input exceeds64KiB plus its fixed prefix".into());
        }
        return match input.trim() { "help"=>Ok(crate::spanning_tree_workspace::HELP.into()), "show"=>Ok(describe(s)), _=>Err("Spanning-tree mutations require fixed goal context; use typed native settings, not IOS or a shell".into()) };
    }

    if matches!(s, State::DhcpV4 { .. }) {
        if input.len() > crate::dhcpv4_workspace::MAX_ACTION_BYTES {
            return Err("DHCP input exceeds256KiB".into());
        }
        return match input.trim() {"help"=>Ok(crate::dhcpv4_workspace::HELP.into()),"show"=>Ok(describe(s)),_=>Err("Use native DHCP controls; mutations require immutable goal context, not Linux/IOS commands".into())};
    }
    if matches!(s, State::UnitManager { .. }) {
        if input.len() > crate::unit_manager_workspace::MAX_ACTION_BYTES {
            return Err("Virtual unit input exceeds its bounded size".into());
        }
        return match input.trim() {
            "help" => Ok(crate::unit_manager_workspace::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("Use native virtual unit controls; this workspace does not execute systemctl, shell commands or host services".into()),
        };
    }
    if matches!(s, State::Ospf { .. }) {
        return match input.trim() { "help" => Ok(crate::ospf_workspace::HELP.into()), "show" => Ok(describe(s)), _ => Err("OSPF mutations require immutable goal context; use native typed settings, not IOS commands".into()) };
    }
    if let State::Ipv4Acl { network } = s {
        return crate::acl_workspace::adapter(network, input);
    }
    if let State::Accounts { simulation } = s {
        if input.len() > crate::accounts_teaching::MAX_COMMAND_BYTES {
            return Err("Account command exceeds2,048 bytes".into());
        }
        if input.trim() == "help" {
            return Ok(crate::accounts_teaching::HELP.into());
        }
        return match crate::accounts::parse(input).map_err(|e| e.to_string())? {
            crate::accounts::Request::Query(query) => crate::accounts_teaching::output_text(
                simulation.query(&query).map_err(|e| e.to_string())?,
            ),
            _ => Err("Account changes require this assignment's immutable goal context".into()),
        };
    }
    if let State::PersistentMounts { storage } = s {
        return crate::storage_workspace::adapter(storage, input);
    }
    if let State::Discovery { simulation } = s {
        return crate::discovery_workspace::adapter(simulation, input);
    }
    if let State::Processes { table } = s {
        let output = table.execute(input)?;
        let text = format!("{}{}", output.stdout, output.stderr);
        return if output.status == 0 {
            Ok(text)
        } else {
            Err(format!(
                "{text}Simulator note: command exit status {}. A no-match search returns 1; inspect any diagnostic above.",
                output.status
            ))
        };
    }
    if let State::AuthoritativeDns { server } = s {
        return crate::dns_workspace::adapter(server, input);
    }
    if let State::MacLearning { simulation } = s {
        return crate::mac_learning_workspace::adapter(simulation, input);
    }
    if let State::Switching { fabric } = s {
        return crate::switching_workspace::adapter(fabric, input);
    }
    if let State::Ipv6 { topology } = s {
        return crate::ipv6_workspace::adapter(topology, input);
    }

    if let State::Ipv4Routing { topology } = s {
        return crate::ipv4_routing_workspace::adapter(topology, input);
    }
    if let State::Filesystem { tree } = s {
        if tree.command_location.is_some() {
            return Err(
                "Command-location work needs its supplied task before commands or help can run"
                    .into(),
            );
        }
        if input.len() > 4096 {
            return Err("Command exceeds 4,096 bytes".into());
        }
        if input.trim() == "help" {
            return Ok(crate::filesystem::HELP.into());
        }
        if input.trim() == "show" {
            return Ok(format!(
                "Simulator state · User: {} · Working directory: {} · {} entries",
                tree.user,
                tree.cwd,
                tree.nodes.len()
            ));
        }
        return tree.execute(input);
    }
    if let State::Script { source, run } = s {
        if input.len() > crate::shell::MAX_SOURCE {
            return Err("Script exceeds 16 KiB".into());
        }
        *source = input.into();
        *run = None;
        return Ok("Script saved; use Run script to execute the test cases".into());
    }
    if input.len() > 4096 {
        return Err("Command exceeds 4096 characters".into());
    }
    let v: Vec<&str> = input.split_whitespace().collect();
    let limitation = format!(
        "Unsupported command or arguments. This is a simulation. Supported subset: {}",
        help(s)
    );
    let bad = || limitation.clone();
    if v == ["help"] {
        return Ok(help(s).into());
    }
    if v == ["show"] {
        return Ok(describe(s));
    }
    match s {
        State::CopiedJournal { .. }
        | State::JsonInventory { .. }
        | State::RestController { .. }
        | State::DevicePath { .. }
        | State::LocalMail { .. }
        | State::Chrony { .. }
        | State::GrubOnce { .. }
        | State::EfiLoader { .. }
        | State::BiosLoader { .. }
        | State::BootEntry { .. }
        | State::FileIndex { .. }
        | State::SharedLibraries { .. }
        | State::Raid1 { .. }
        | State::StartupFiles { .. }
        | State::FsIntegrity { .. }
        | State::UserMounts { .. }
        | State::Environment { .. }
        | State::DebianSources { .. }
        | State::ClientNames { .. }
        | State::RemovableMounts { .. }
        | State::NmProfiles { .. }
        | State::PamStack { .. }
        | State::Aging { .. }
        | State::PortSecurity { .. }
        | State::DhcpSnooping { .. }
        | State::ArpInspection { .. }
        | State::ManagementPolicy { .. }
        | State::LocalAccess { .. }
        | State::AaaAccess { .. }
        | State::LdapClient { .. }
        | State::LdapAccess { .. }
        | State::LdapSchema { .. }
        | State::LdapSchemaQuery { .. }
        | State::LdapPassword { .. }
        | State::LdapChanges { .. }
        | State::DhcpConfig { .. }
        | State::Qos { .. }
        | State::LinuxLogging { .. }
        | State::VirtualIsolation { .. }
        | State::SieveFiltering { .. }
        | State::MailboxAccess { .. }
        | State::LinuxRouter { .. }
        | State::LinuxV6 { .. }
        | State::LinuxServiceRouter { .. }
        | State::LinuxV6Administration { .. }
        | State::LinuxV6Startup { .. }
        | State::IdsRule { .. }
        | State::AuthFilter { .. }
        | State::VpnPoint { .. }
        | State::Poe { .. }
        | State::KernelBuild { .. }
        | State::Dkms { .. }
        | State::Initramfs { .. }
        | State::Pxe { .. }
        | State::Iscsi { .. }
        | State::AtaSmart { .. }
        | State::ApacheHttps { .. }
        | State::Desktop { .. }
        | State::SecurityInvestigation { .. }
        | State::AutomationWorkshop { .. }
        | State::ServiceExposure { .. }
        | State::FtpIdentity { .. }
        | State::LinuxFlow { .. }
        | State::MailboxAdministration { .. }
        | State::SmtpRouting { .. }
        | State::GuestDrivers { .. }
        | State::SambaShare { .. }
        | State::CampusArchitecture { .. }
        | State::NetworkTransfer { .. }
        | State::EditorHandover { .. }
        | State::ModalEditing { .. }
        | State::SshAccess { .. }
        | State::HostNetwork { .. }
        | State::Repositories { .. }
        | State::Capacity { .. }
        | State::DnsAccess { .. }
        | State::PrintQueues { .. }
        | State::Localization { .. }
        | State::WirelessChannels { .. }
        | State::WlanProfiles { .. }
        | State::HttpCache { .. }
        | State::ServiceClients { .. }
        | State::ReverseProxy { .. }
        | State::ProxyPolicy { .. }
        | State::SnmpMonitor { .. }
        | State::WlanPaths { .. }
        | State::Timezone { .. }
        | State::SourceInstall { .. }
        | State::MaintenanceNotice { .. }
        | State::CapacityPlanning { .. }
        | State::ResourceObservations { .. }
        | State::RaidAssembly { .. }
        | State::Autofs { .. }
        | State::KernelModules { .. }
        | State::RecursiveTree { .. }
        | State::Transport { .. }
        | State::DirectoryPolicy { .. }
        | State::Links { .. }
        | State::PartitionFs { .. }
        | State::Hsrp { .. }
        | State::Ntp { .. }
        | State::Syslog { .. }
        | State::Lvm { .. }
        | State::PhysicalLink { .. }
        | State::OneOff { .. }
        | State::FileBackup { .. }
        | State::EtherChannel { .. }
        | State::Cron { .. }
        | State::Nfs { .. }
        | State::RpmPackages { .. }
        | State::BasicNat { .. }
        | State::ApacheStatic { .. }
        | State::DebianPackages { .. }
        | State::SpanningTree { .. }
        | State::DhcpV4 { .. }
        | State::UnitManager { .. }
        | State::Ospf { .. }
        | State::Ipv4Acl { .. }
        | State::Accounts { .. }
        | State::PersistentMounts { .. }
        | State::Discovery { .. }
        | State::Processes { .. }
        | State::AuthoritativeDns { .. }
        | State::MacLearning { .. }
        | State::Switching { .. }
        | State::Ipv6 { .. }
        | State::Ipv4Routing { .. }
        | State::Filesystem { .. }
        | State::Script { .. } => unreachable!(),
        State::MagicSubnet { .. } => unreachable!(),
        State::Subnet {
            answers, second, ..
        } => match v.as_slice() {
            ["answer", field, value] => {
                if *field == "second" {
                    value
                        .parse::<Ipv4Addr>()
                        .map_err(|_| "Invalid IPv4 address")?;
                    *second = Some(value.to_string());
                    return Ok("Second workstation address saved".into());
                }
                let i = match *field {
                    "network" => 0,
                    "broadcast" => 1,
                    "first" => 2,
                    "last" => 3,
                    _ => return Err("Use network, broadcast, first or last".into()),
                };
                value
                    .parse::<Ipv4Addr>()
                    .map_err(|_| "Invalid IPv4 address")?;
                answers[i] = value.to_string();
                Ok("Answer saved. Check work when ready.".into())
            }
            _ => Err(bad()),
        },
        State::Vlsm { allocations, .. } => match v.as_slice() {
            ["allocate", name, value] => {
                cidr(value).ok_or("Use an aligned IPv4 network/prefix")?;
                let a = allocations
                    .iter_mut()
                    .find(|a| a.name == *name)
                    .ok_or("Unknown allocation name")?;
                a.cidr = value.to_string();
                Ok("Allocation saved".into())
            }
            _ => Err(bad()),
        },
        State::Network {
            address,
            prefix,
            gateway,
            router,
            lan_prefix,
            target,
        } => match v.as_slice() {
            ["ip", "addr"] => Ok(format!(
                "client {address}/{prefix}; LAN router {router}/{lan_prefix}"
            )),
            ["ip", "route"] => Ok(format!("default via {gateway}")),
            ["ip", "addr", "set", value] => {
                let (a, p) = value.split_once('/').ok_or("Use IPv4/prefix")?;
                let a = a.parse::<Ipv4Addr>().map_err(|_| "Invalid address")?;
                let p = p.parse::<u8>().map_err(|_| "Invalid prefix")?;
                if p > 30 {
                    return Err("LAN prefix must be 0..30".into());
                }
                *address = a;
                *prefix = p;
                Ok("Client address updated".into())
            }
            ["ip", "route", "replace", "default", "via", a] => {
                *gateway = a.parse().map_err(|_| "Invalid gateway")?;
                Ok("Default route updated".into())
            }
            ["ping", a] => {
                let a = a.parse::<Ipv4Addr>().map_err(|_| "Use an IPv4 address")?;
                let lan = bounds(*address, *prefix) == bounds(*router, *prefix);
                let ok = lan
                    && ((a == *router)
                        || (a == *target && gateway == router && prefix == lan_prefix));
                Ok(if ok {
                    format!("Reply from {a} (simulated)")
                } else {
                    "Destination unreachable (simulated)".into()
                })
            }
            _ => Err(bad()),
        },
        State::Unix { nodes, users } => match v.as_slice() {
            ["ls", option, path] if ["-l", "-ld"].contains(option) => {
                let n = nodes.get(*path).ok_or("No simulated path")?;
                if n.directory && *option == "-l" {
                    let children = nodes
                        .iter()
                        .filter(|(p, _)| {
                            p.as_str() != *path
                                && p.rsplit_once('/').is_some_and(|(parent, _)| {
                                    if parent.is_empty() {
                                        *path == "/"
                                    } else {
                                        parent == *path
                                    }
                                })
                        })
                        .map(|(p, n)| long_listing(p.rsplit('/').next().unwrap_or(p), n))
                        .collect::<Vec<_>>();
                    Ok(format!("total 0\n{}", children.join("\n")))
                } else {
                    Ok(long_listing(path, n))
                }
            }
            ["namei", path] => {
                if !nodes.contains_key(*path) {
                    return Err("No simulated path".into());
                }
                Ok(nodes
                    .iter()
                    .filter(|(p, _)| {
                        *p == "/" || path.starts_with(&format!("{p}/")) || p.as_str() == *path
                    })
                    .map(|(p, n)| format!("{:03o} {}:{} {p}", n.mode, n.owner, n.group))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
            ["id", user] => {
                let groups = users.get(*user).ok_or("Unknown user")?;
                let uid = user_id(users, user);
                let primary = groups.first().ok_or("User has no primary group")?;
                Ok(format!(
                    "uid={uid}({user}) gid={}({primary}) groups={}",
                    group_id(users, primary),
                    groups
                        .iter()
                        .map(|g| format!("{}({g})", group_id(users, g)))
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            }
            ["chmod", mode, path] => {
                let n = nodes.get_mut(*path).ok_or("No simulated path")?;
                let parsed = u16::from_str_radix(mode, 8).ok().filter(|m| *m <= 0o777);
                let mode = if let Some(m) = parsed {
                    m
                } else {
                    let mut m = n.mode;
                    for clause in mode.split(',') {
                        let (who, perms) = clause
                            .split_once('=')
                            .ok_or("Use octal or explicit u=,g=,o= clauses")?;
                        let shift = match who {
                            "u" => 6,
                            "g" => 3,
                            "o" => 0,
                            _ => return Err("Only u, g, o are supported".into()),
                        };
                        let mut b = 0;
                        for ch in perms.chars() {
                            b |= match ch {
                                'r' => 4,
                                'w' => 2,
                                'x' => 1,
                                _ => return Err("Use r,w,x".into()),
                            };
                        }
                        m = (m & !(7 << shift)) | (b << shift);
                    }
                    m
                };
                n.mode = mode;
                Ok("Permissions updated".into())
            }
            ["chown", owner, path] => {
                let (u, g) = owner.split_once(':').ok_or("Use USER:GROUP")?;
                if !users.contains_key(u) || !users.values().any(|gs| gs.iter().any(|v| v == g)) {
                    return Err("Unknown user or group".into());
                }
                let n = nodes.get_mut(*path).ok_or("No simulated path")?;
                n.owner = u.into();
                n.group = g.into();
                Ok("Ownership updated".into())
            }
            ["usermod", "-aG", group, user] => {
                if !users.values().any(|gs| gs.iter().any(|g| g == group)) {
                    return Err("Unknown group".into());
                }
                let groups = users.get_mut(*user).ok_or("Unknown user")?;
                if !groups.iter().any(|g| g == group) {
                    groups.push(group.to_string());
                }
                Ok("Group added; simulated sessions update immediately".into())
            }
            ["test-access", user, path, mode] => {
                let mask = match *mode {
                    "r" => 4,
                    "w" => 2,
                    "rw" => 6,
                    _ => return Err("Use r, w or rw".into()),
                };
                Ok(if access(nodes, users, path, user, mask) {
                    "Access allowed"
                } else {
                    "Permission denied"
                }
                .into())
            }
            _ => Err(bad()),
        },
        State::Logs { files, result } => {
            if let ["find", "/var/log", "-name", name] = v.as_slice() {
                return Ok(files
                    .keys()
                    .filter(|p| p.rsplit('/').next() == Some(*name))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n"));
            }
            let chunks: Vec<_> = v.split(|t| *t == "|").collect();
            let mut text = match chunks[0] {
                ["cat", path] => files.get(*path).ok_or("No simulated file")?.clone(),
                ["grep", needle, path] => files
                    .get(*path)
                    .ok_or("No simulated file")?
                    .lines()
                    .filter(|l| l.contains(needle))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ["tail", "-n", count, path] => {
                    tail(files.get(*path).ok_or("No simulated file")?, count)?
                }
                _ => return Err(bad()),
            };
            for chunk in chunks.iter().skip(1) {
                text = match *chunk {
                    ["grep", needle] => text
                        .lines()
                        .filter(|l| l.contains(needle))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    ["tail", "-n", count] => tail(&text, count)?,
                    _ => return Err(bad()),
                };
            }
            *result = text.clone();
            Ok(if text.is_empty() {
                "(no matching lines)".into()
            } else {
                text
            })
        }
        State::Service {
            port,
            running,
            enabled,
            loaded_port,
            failed,
            journal,
        } => match v.as_slice() {
            ["cat", "/etc/atlas.conf"] => Ok(format!("port={port}")),
            ["config", "set", "port", value] => {
                if value.len() > 256 {
                    return Err("Port field exceeds 256 characters".into());
                }
                *port = value.to_string();
                Ok("Simulator-only editor: configuration saved; restart to apply".into())
            }
            ["journalctl", "-u", "atlas"] => Ok(journal_output(journal)),
            ["systemctl", action, "atlas"] => match *action {
                "status" => Ok(service_status(*running, *enabled, *failed, *loaded_port)),
                "is-active" => Ok(if *running {
                    "active"
                } else if *failed {
                    "failed"
                } else {
                    "inactive"
                }
                .into()),
                "is-enabled" => Ok(if *enabled { "enabled" } else { "disabled" }.into()),
                "start" | "restart" => {
                    if *action == "start" && *running {
                        return Ok(
                            "Unit is already active; restart to apply saved configuration.".into(),
                        );
                    }
                    *running = false;
                    *loaded_port = None;
                    if let Some(p) = port.parse::<u16>().ok().filter(|p| *p > 0) {
                        *running = true;
                        *failed = false;
                        *loaded_port = Some(p);
                        push_event(journal, ServiceEvent::Started { port: p });
                        Ok(String::new())
                    } else {
                        *failed = true;
                        push_event(journal, ServiceEvent::Rejected { port: port.clone() });
                        Err("Job for atlas.service failed because the control process exited with error code. See systemctl status atlas and journalctl -u atlas for details.".into())
                    }
                }
                "stop" => {
                    *running = false;
                    *loaded_port = None;
                    *failed = false;
                    push_event(journal, ServiceEvent::Stopped);
                    Ok(String::new())
                }
                "enable" | "disable" => {
                    *enabled = *action == "enable";
                    Ok(if *enabled {
                        "Created symlink /etc/systemd/system/multi-user.target.wants/atlas.service -> /etc/systemd/system/atlas.service."
                    } else {
                        "Removed /etc/systemd/system/multi-user.target.wants/atlas.service."
                    }.into())
                }
                _ => Err(bad()),
            },
            _ => Err(bad()),
        },
        State::Dns {
            records,
            loaded,
            serial,
            loaded_serial,
        } => match v.as_slice() {
            ["cat", "/etc/named/lab.zone"] => Ok(format!(
                "serial {serial}\n{}",
                records
                    .iter()
                    .map(|(n, a)| format!("{n} IN A {a}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )),
            ["dns", "set", name, addr] => {
                if !records.contains_key(*name) {
                    return Err("Use an existing zone name".into());
                }
                addr.parse::<Ipv4Addr>()
                    .map_err(|_| "Invalid IPv4 A record")?;
                records.insert(name.to_string(), addr.to_string());
                Ok("Record saved; increase serial and reload".into())
            }
            ["dns", "serial", value] => {
                *serial = value.parse().map_err(|_| "Serial must be a u32 integer")?;
                Ok("Serial saved".into())
            }
            ["named-checkzone", "lab.test"] => {
                if records.values().all(|a| a.parse::<Ipv4Addr>().is_ok()) {
                    Ok(format!("zone lab.test serial {serial}: OK"))
                } else {
                    Err("Invalid A record".into())
                }
            }
            ["systemctl", "reload", "named"] => {
                if *serial <= *loaded_serial {
                    return Err("Increase serial above the loaded serial before reload".into());
                }
                if !records.values().all(|a| a.parse::<Ipv4Addr>().is_ok()) {
                    return Err("Invalid A record".into());
                }
                *loaded = records.clone();
                *loaded_serial = *serial;
                Ok("Zone loaded (authoritative simulation; no cache)".into())
            }
            ["dig", name] => Ok(loaded
                .get(*name)
                .map(|a| format!("{name} IN A {a}"))
                .unwrap_or("NXDOMAIN".into())),
            _ => Err(bad()),
        },
        State::Mount {
            capacity,
            used,
            cache,
            mounted,
            device,
            configured_device,
            writable,
        } => match v.as_slice() {
            ["df", "-h"] => Ok(format!(
                "{device} /data: {used}/{capacity} MiB used; mounted={mounted}, rw={writable}"
            )),
            ["du", "-sh", "/data/cache"] => {
                Ok(format!("{cache} MiB /data/cache (disposable cache)"))
            }
            ["cat", "/etc/fstab"] => Ok(format!(
                "{configured_device} /data ext4 defaults 0 2\nActual data device: {device}"
            )),
            ["config", "set", "device", value] => {
                if value.len() > 256 {
                    return Err("Device field exceeds 256 characters".into());
                }
                *configured_device = value.to_string();
                Ok("fstab device updated".into())
            }
            ["rm", "-r", "/data/cache"] => {
                if !*mounted || !*writable {
                    return Err("/data must be mounted read-write".into());
                }
                *used = used.saturating_sub(*cache);
                *cache = 0;
                Ok("Disposable cache removed".into())
            }
            ["mount", "/data"] => {
                if device != configured_device {
                    return Err("fstab references an unknown device".into());
                }
                *mounted = true;
                *writable = true;
                Ok("/data mounted read-write".into())
            }
            ["mount", "-o", "remount,rw", "/data"] => {
                if !*mounted {
                    return Err("Not mounted".into());
                }
                *writable = true;
                Ok("/data remounted read-write".into())
            }
            ["umount", "/data"] => {
                *mounted = false;
                *writable = false;
                Ok("/data unmounted".into())
            }
            _ => Err(bad()),
        },
    }
}
fn tail(text: &str, count: &str) -> Result<String, String> {
    let n = count
        .parse::<usize>()
        .map_err(|_| "Count must be a positive integer")?;
    if n > 10000 {
        return Err("Count limit is 10000".into());
    }
    let l: Vec<_> = text.lines().collect();
    Ok(l[l.len().saturating_sub(n)..].join("\n"))
}
pub fn describe(s: &State) -> String {
    match s {
        State::StartupFiles {..} => "Simulated Bash startup files: bounded in-process settings and new-program verification; no host Bash or profiles".into(),
        State::FileIndex {..} => "Saved private index, virtual configuration and files; use the native editor and simulated terminal".into(),
        State::SharedLibraries {..} => "Supplied library dependency metadata, simulated search settings and virtual cache; no real ELF or utility execution".into(),
        State::Raid1 {..} => "Simulated two-member RAID1: supplied devices, retained data and explicit bounded recovery; no host devices".into(),
        State::FsIntegrity {..} => "Simulated filesystem consistency: supplied volumes, derived metadata findings and retained data; no host filesystem access".into(),
        State::UserMounts {..} => "Simulated mount policy, supplied callers and replayed mount actions; no host mounts".into(),
        State::Environment {..} => "Simulated parent variables and a supplied child report; no host environment or processes".into(),
        State::DebianSources {..} => "Saved virtual source selection and supplied cached binary-index snapshots; no host files, APT or transport".into(),
        State::ClientNames { .. } => "Virtual local names and supplied DNS snapshots; no host resolver or commands".into(),
        State::RemovableMounts { .. } => "Virtual mount policy and supplied media requests; no host commands or actual boot".into(),
        State::NmProfiles{..}=>"Virtual saved Ethernet profiles and bounded startup projections; no host daemon or files".into(),
        State::PamStack{..}=>"Virtual PAM auth rules and supplied synthetic module results; no host authentication or credentials".into(),
        State::Aging{..}=>"Virtual non-secret aging records and supplied UTC policy observations; no host accounts or authentication".into(),
        State::PortSecurity{..}=>"Virtual secure access ports and actual bounded metadata forwarding; no host interfaces, packets or IOS process".into(),
        State::DhcpSnooping{..}=>"Virtual DHCP admission, observed initial exchanges and expiring lease records; no host packets, services or IOS process".into(),
        State::ArpInspection{..}=>"Virtual ARP admission, DHCP address records and exact address-list policies; no host packets, ARP caches or IOS process".into(),
        State::ManagementPolicy{..}=>"Virtual router-local IPv4 paths and VTY policy before authentication; no real SSH, crypto, host commands or network".into(),
        State::LocalAccess{..}=>"Virtual management paths, incoming policy and local-case training records/EXEC levels; no host authentication, SSH process or crypto".into(),
        State::AaaAccess{..}=>"Virtual incoming paths, ordered training identity methods and explicit local EXEC profiles; no real AAA protocol, service, host login or crypto".into(),
        State::LdapClient{..}=>"Virtual directory entries, client defaults and bounded in-process search; no host files, processes, real directory connection or authentication".into(),
        State::LdapAccess{..}=>"Virtual ordered ACLs and actual modeled contact changes; no host server, command, files or network".into(),
        State::LdapSchema{..}=>"Virtual schema registration and actual typed directory entries; no host files, commands, server or network".into(),
        State::CopiedJournal { .. } => "Immutable copied journal and current simulated incident query".into(),
        State::JsonInventory { .. } => "Edited JSON inventory and applied simulated access-port behavior".into(),
        State::RestController { .. } => "Edited REST controller and applied simulated access-port behavior".into(),
        State::DevicePath { .. } => "Virtual cable endpoints, Ethernet domains and routed request/reply paths".into(),
        State::LocalMail { .. } => "Saved virtual mail files, separate alias lookup and sample delivery outcomes".into(),
        State::Chrony { .. } => "Saved chrony configuration and supplied simulated source observations".into(),
        State::GrubOnce { .. } => "Temporary GRUB linux line, compared with ordinary startup from the saved menu".into(),
        State::EfiLoader { .. } => "Supplied EFI partition, loader files, firmware order and hypothetical Linux handoff".into(),
        State::BiosLoader { .. } => "Supplied BIOS disk, virtual loader installation and hypothetical boot chain; no firmware, host disks or subprocesses".into(),
        State::BootEntry { .. } => "Supplied boot-entry text and hypothetical early-root handoff; no reboot, host commands or firmware execution".into(),
        State::LdapSchemaQuery{..}=>"Virtual schema, actual entries and bounded filters; no full search, host commands, server or network".into(),
        State::LdapPassword{..}=>"Virtual protected authentication, client settings and fictional credential changes; no actual TLS, password store, host command or network".into(),
        State::LdapChanges{..}=>"Virtual directory entries and bounded LDIF editing with supplied delegated rights; no real bind, host files, subprocesses or network".into(),
        State::DhcpConfig{..}=>"Virtual DHCP configuration and bounded client cases; no daemon, network or host files".into(),
        State::Qos{..}=>"Virtual traffic labels, finite waiting queues and supplied packet replay; no real packets or network".into(),
        State::LinuxLogging{..}=>"Virtual classic logging selector fragment and sample replay; no daemon, host files or network".into(),
        State::VirtualIsolation{..}=>crate::virtual_isolation_source::HELP.into(),
        State::SieveFiltering{..}=>crate::sieve_source::HELP.into(),
        State::MailboxAccess{..}=>crate::mailbox_source::HELP.into(),
        State::MailboxAdministration{..}=>crate::mailbox_source::ADMIN_HELP.into(),
        State::LinuxRouter{..}=>crate::linux_router_source::HELP.into(),
        State::LinuxV6{..}=>crate::linux_v6_source::HELP.into(),
        State::LinuxServiceRouter{..}=>crate::linux_service_router_source::HELP.into(),
        State::LinuxV6Administration{..}=>crate::linux_v6_admin_source::HELP.into(),
        State::LinuxV6Startup{..}=>crate::linux_v6_startup_source::HELP.into(),
        State::IdsRule{..}=>crate::ids_rule_source::HELP.into(),
        State::AuthFilter{..}=>crate::auth_filter_source::HELP.into(),
        State::VpnPoint{..}=>crate::vpn_point_source::HELP.into(),
        State::Poe{..}=>crate::poe_source::HELP.into(),
        State::KernelBuild{..}=>crate::kernel_source::HELP.into(),
        State::Dkms{..}=>crate::dkms_source::HELP.into(),
        State::Initramfs{..}=>crate::initramfs_source::HELP.into(),
        State::Pxe{..}=>crate::pxe::source::HELP.into(),
        State::Iscsi{..}=>crate::iscsi::source::HELP.into(),
        State::AtaSmart{..}=>crate::ata_smart::source::HELP.into(),
        State::ApacheHttps{..}=>crate::apache_https::source::HELP.into(),
        State::Desktop{..}=>crate::desktop::source::HELP.into(),
        State::SecurityInvestigation{..}=>crate::security_investigation::source::HELP.into(),
        State::AutomationWorkshop{..}=>crate::automation_workshop::source::HELP.into(),
        State::ServiceExposure{..}=>crate::service_exposure_source::HELP.into(),
        State::FtpIdentity{..}=>crate::ftp_identity_source::HELP.into(),
        State::LinuxFlow{..}=>crate::linux_flow_source::HELP.into(),
        State::SmtpRouting{..}=>"Simulated mail configuration, relay policy and sample queue".into(),
        State::GuestDrivers{..}=>"Simulated guest-image files and loaded driver state".into(),
        State::SambaShare{..}=>crate::samba_share_source::HELP.into(),
        State::CampusArchitecture{..}=>crate::network_architecture::HELP.into(),
        State::NetworkTransfer{..}=>"Bounded virtual file transfers over simulated routes and ACLs; TFTP octet and passive FTP only, no host client or real target".into(),
        State::EditorHandover{..}=>"Bounded simulated environment setup and inherited editor selection, followed by virtual-file editing; no host program or environment change".into(),
        State::ModalEditing{..}=>"Supplied virtual text files and bounded modal buffer; no host editor, files, shell or external commands".into(),
        State::SshAccess{..}=>"Supplied SSH public identities, virtual key files and supported policy/trust checks; no actual SSH, private keys, crypto verification or host access".into(),
        State::HostNetwork{simulation}=>if simulation.assignment.world.profile.ipv6.is_some(){"Virtual Linux dual-stack interfaces and separate IPv4/IPv6 main-table routes; no real packets, host networking or files".into()}else{"Virtual Linux IPv4 interfaces, main-table routes and resolver inputs; no host networking or files".into()},
        State::Repositories{..}=>"Supplied virtual repositories, cached snapshots and exact-name candidate queries; no real transport or installation".into(),
        State::MaintenanceNotice {..}=>"Local maintenance notices · supplied sessions and simulated timing".into(),
        State::DnsAccess {..}=>"Simulated DNS client policies · saved/live configuration, cache and zone copies".into(),
        State::PrintQueues {..}=>"Supplied print queues · virtual printer and request state".into(),
        State::Localization {..}=>"Supplied locale settings and virtual byte documents".into(),
        State::WirelessChannels {..}=>"Supplied radio channels and finite survey relationships".into(),
        State::WlanProfiles {..}=>"Supplied WLAN and client configuration compatibility · no radio or encryption operation".into(),
        State::HttpCache {..}=>"Virtual shared HTTP cache · supplied request traces, no host or network operation".into(),
        State::ServiceClients {..}=>"Virtual service names and clients · supplied files/listeners, no host lookup or network operation".into(),
        State::ReverseProxy {..}=>"Virtual reverse-proxy routing · supplied paths, no host listener or network operation".into(),
        State::ProxyPolicy {..}=>"Virtual proxy access policy · supplied requests, no host or network operation".into(),
        State::SnmpMonitor {..}=>"Virtual MIB monitoring · supplied snapshots only, no host or network operation".into(),
        State::WlanPaths {..}=>"AP/controller VLAN forwarding paths · no controller join, radio or host operation".into(),
        State::Timezone {..}=>"Supplied time zones and immutable virtual UTC instants".into(),
        State::SourceInstall {..}=>"Supplied source project · virtual build and staging state".into(),
        State::CapacityPlanning {..}=>"Capacity planning · supplied observations, projections and workload tests".into(),
        State::ResourceObservations {..}=>"Recorded-host resource investigation · supplied counters and verified reports; no host reads".into(),
        State::Capacity {..}=>"Virtual filesystem capacity · blocks, inodes and ordinary availability; no host disk operations".into(),
        State::RaidAssembly { .. } => "Supplied clean RAID1 member metadata, saved assembly policy and active original data; no host files, devices or processes".into(),
        State::Autofs { .. } => "Simulated local-file AutoFS maps, demand mounts and nonbusy idle checks; no host daemon, files or commands".into(),
        State::KernelModules { .. } => "Simulated supplied loadable modules, saved options and effective state; no host kernel or commands".into(),
        State::RecursiveTree { .. } => "Simulated ordinary directory transfer and retirement; no host files or commands".into(),
        State::Transport { .. } => "Simulated TCP/UDP endpoints, routed packet traces and ACL configuration; no host sockets or commands".into(),
        State::DirectoryPolicy { .. } => "Simulated shared folders, ordinary participants and restricted deletion; no host accounts or files".into(),
        State::Links { .. } => "Simulated file names, inodes and symbolic links; fixed virtual directories, no host filesystem access".into(),
        State::MagicSubnet { session } => format!("IPv4 subnet worksheet · {}/{}",session.problem.ip,session.problem.prefix),
        State::PartitionFs { simulation } => format!(
            "Virtual GPT/ext4/swap preparation · {} supplied disks · applied state and separate unapplied drafts; no real devices",
            simulation.current.disks.len()
        ),
        State::Hsrp { simulation } => format!(
            "Settled HSRP simulator: Active {:?}, Standby {:?}; virtual gateway {}. No packets or measured convergence.",
            simulation.current.active(),
            simulation.current.standby(),
            simulation.current.virtual_ip()
        ),
        State::Syslog { simulation } => format!(
            "Native virtual syslog policy: {}. Use the settings and event controls; historical logs are not current-policy proof.",
            if simulation.validate().is_ok() {
                "bounded supplied workspace"
            } else {
                "invalid retained workspace"
            }
        ),
        State::Ntp { simulation } => format!(
            "Virtual single-upstream NTP · {} supplied nodes · read-only reference/delivery facts; no host clock or packets",
            simulation.current.nodes.len()
        ),
        State::Lvm { simulation } => format!(
            "Virtual linear LVM · {} supplied groups · separate LV/filesystem capacity; no real devices",
            simulation.current.groups.len()
        ),
        State::PhysicalLink { simulation } => format!(
            "Simulated physical uplinks: {} supplied interfaces and {} point-to-point links. Qualification and deterministic samples are not physical measurements or real network traffic",
            simulation.topology().ports.len(),
            simulation.topology().links.len()
        ),
        State::FileBackup { simulation } => format!(
            "Virtual backup administrator: root. {} supplied file trees, {} saved archives and {} snapshots; current folder {}. All paths and permissions are simulated.",
            simulation.trees.len(),
            simulation.archives.len(),
            simulation.snapshots.len(),
            simulation.cwd
        ),
        State::EtherChannel { simulation } => format!(
            "Virtual L2 LACP: {} supplied devices, {} physical pairs. No packets, timers, routed channel or host command",
            simulation.topology().devices.len(),
            simulation.topology().cables.len()
        ),
        State::OneOff { simulation } => format!(
            "Virtual UTC {} · {} pending one-off jobs · {} virtual files. Explicit bounded simulation only",
            simulation.simulation.clock.text(),
            simulation.simulation.jobs.len(),
            simulation.simulation.files.len()
        ),
        State::Cron { simulation } => format!(
            "Virtual UTC {} · {} supplied accounts · {} installed user tables · {} virtual files. Only the explicit clock advances; no host scheduler runs",
            simulation.clock.text(),
            simulation.users.len(),
            simulation.tables.len(),
            simulation.files.len()
        ),
        State::Nfs { simulation } => format!(
            "Supplied virtual NFS inventory: {} servers, {} clients, {} current mounts and {} existing text files. No host mount, RPC or trusted identity",
            simulation.inventory().servers.len(),
            simulation.inventory().clients.len(),
            simulation.runtime().mounts.len(),
            simulation.runtime().contents.len()
        ),
        State::RpmPackages { simulation } => format!(
            "Restricted script-free RPM simulation: {} installed records, {} virtual files; no host RPM or full verification",
            simulation.runtime().installed.len(),
            simulation.runtime().files.len()
        ),
        State::BasicNat { simulation } => format!(
            "Basic NAT address-only model: {} inside hosts, {} outside peers, {} static and {} current dynamic mappings. No sessions, routing, connectivity or security verdict",
            simulation.inventory().inside.len(),
            simulation.inventory().outside.len(),
            simulation.configuration().statics.len(),
            simulation.bindings().len()
        ),
        State::ApacheStatic { simulation } => format!(
            "Virtual static HTTP: {} fixed files; {}. Saved configuration and active source are separate; no host listener or process",
            simulation.files.len(),
            if simulation.active.is_some() {
                "active source loaded"
            } else {
                "stopped"
            }
        ),

        State::DebianPackages { simulation } => format!(
            "Logical script-free Debian packages: {} database records, {} virtual text files, cached metadata {}. No real package transaction, archive or host utility",
            simulation.runtime().records.len(),
            simulation.runtime().files.len(),
            simulation.runtime().cached.as_deref().unwrap_or("absent")
        ),
        State::SpanningTree { simulation } => format!(
            "Ideal P2P steady-state spanning tree: {} supplied bridges, {} physical links. No BPDUs, convergence timing, packets or PVST/guard claim",
            simulation.topology().bridges.len(),
            simulation.topology().links.len()
        ),
        State::DhcpV4 { simulation } => crate::dhcpv4_workspace::describe(simulation),
        State::UnitManager { simulation } => format!(
            "Virtual unit manager · {} supplied units · {} loaded definitions · current runtime and future-session enablement are separate; no host systemd or boot",
            simulation.inventory().units.len(),
            simulation.loaded_sources().len()
        ),
        State::Ospf { simulation } => format!(
            "OSPFv2 derived P2P compatibility · {} virtual routers · no observed Full adjacency or packet forwarding",
            simulation.configuration().routers.len()
        ),
        State::Ipv4Acl { network } => format!(
            "Virtual ACL configuration · {} devices · explicit service-flow tests only; no real packets or full IOS CLI",
            network.topology.devices.len()
        ),
        State::Accounts { simulation } => format!(
            "Virtual administrator · {} account records · {} groups · no host accounts or authentication",
            simulation.configuration().users.len(),
            simulation.configuration().groups.len()
        ),
        State::PersistentMounts { storage } => format!(
            "Virtual storage · {} volumes · {} live mounts · saved configuration and live state are separate",
            storage.inventory().volumes().len(),
            storage.live().len()
        ),
        State::Discovery { simulation } => format!(
            "LLDP discovery · {} virtual devices · simulated time {}s · no real packets",
            simulation.topology().devices.len(),
            simulation.now()
        ),
        State::Processes { table } => format!(
            "Virtual process workspace · User: {} · Inspect with ps. Only fixed simulated PIDs exist.",
            table.user.name
        ),
        State::AuthoritativeDns { server } => format!(
            "{} · saved serial {} · primary {} · secondary {}",
            server.configured.origin,
            server.configured.serial,
            server.primary.serial,
            server.secondary.serial
        ),
        State::MacLearning { simulation } => format!(
            "Ethernet source learning · {} switches · {} learned entries · {} simulated seconds",
            simulation.fabric().switches.len(),
            simulation.entries().len(),
            simulation.now()
        ),
        State::Switching { fabric } => format!(
            "VLAN data plane · {} switches · {} workstations",
            fabric.switches.len(),
            fabric.hosts.len()
        ),
        State::Ipv6 { topology } => format!(
            "IPv6 static topology · {} simulated devices",
            topology.devices.len()
        ),
        State::Ipv4Routing { topology } => format!(
            "IPv4 static topology · {} simulated devices",
            topology.devices.len()
        ),
        State::Filesystem { tree } => format!("Virtual filesystem · {} · {}", tree.user, tree.cwd),
        State::Script { .. } => "Script editor · virtual files only".into(),
        State::Subnet {
            ip,
            prefix,
            answers,
            ..
        } => format!(
            "Address card\n{ip}/{prefix}\n\nNetwork: {}\nBroadcast: {}\nFirst host: {}\nLast host: {}",
            answers[0], answers[1], answers[2], answers[3]
        ),
        State::Vlsm {
            base,
            prefix,
            allocations,
        } => format!(
            "Address pool: {base}/{prefix}\n{}",
            allocations
                .iter()
                .map(|a| format!(
                    "{} • {} hosts • {}",
                    a.name,
                    a.hosts,
                    if a.cidr.is_empty() {
                        "unassigned"
                    } else {
                        &a.cidr
                    }
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        State::Network {
            address,
            prefix,
            gateway,
            router,
            lan_prefix,
            target,
        } => format!(
            "CLIENT {address}/{prefix} · default gateway: {gateway}\n  |\nLAN ROUTER {router}/{lan_prefix} → REMOTE SERVER {target}"
        ),
        State::Unix { nodes, .. } => nodes
            .iter()
            .map(|(p, n)| format!("{:03o} {}:{} {p}", n.mode, n.owner, n.group))
            .collect::<Vec<_>>()
            .join("\n"),
        State::Logs { files, .. } => format!(
            "Log files\n{}",
            files.keys().cloned().collect::<Vec<_>>().join("\n")
        ),
        State::Service {
            running,
            enabled,
            failed,
            loaded_port,
            ..
        } => format!(
            "atlas.service  |  {}  |  {}{}",
            if *running {
                "Running"
            } else if *failed {
                "Stopped (last start failed)"
            } else {
                "Stopped"
            },
            if *enabled {
                "Enabled at boot"
            } else {
                "Disabled at boot"
            },
            loaded_port
                .filter(|_| *running)
                .map(|p| format!("\nRunning port: {p}"))
                .unwrap_or_default()
        ),
        State::Dns {
            loaded,
            loaded_serial,
            ..
        } => format!(
            "Authoritative DNS • lab.test\nLoaded serial: {loaded_serial}\n{}",
            loaded
                .iter()
                .map(|(n, a)| format!("{n} → {a}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        State::Mount {
            capacity,
            used,
            mounted,
            writable,
            ..
        } => format!(
            "Data volume /data\n{used} / {capacity} MiB used\n{} | {}",
            if *mounted { "Mounted" } else { "Not mounted" },
            if *writable { "Read-write" } else { "Read-only" }
        ),
    }
}

/// Native service-file editor; saving does not mutate the running configuration or journal.
pub fn edit_service_port(state: &mut State, value: &str) -> Result<(), String> {
    if value.len() > 256 {
        return Err("Port field exceeds 256 characters".into());
    }
    if let State::Service { port, .. } = state {
        *port = value.into();
        Ok(())
    } else {
        Err("This state has no service configuration".into())
    }
}
fn push_event(journal: &mut Vec<ServiceEvent>, event: ServiceEvent) {
    journal.push(event);
    if journal.len() > 60 {
        journal.remove(0);
    }
}
pub fn mode_string(n: &Node) -> String {
    let mut s = String::from(if n.directory { "d" } else { "-" });
    for shift in [6, 3, 0] {
        for (bit, ch) in [(4, 'r'), (2, 'w'), (1, 'x')] {
            s.push(if (n.mode >> shift) & bit != 0 {
                ch
            } else {
                '-'
            });
        }
    }
    s
}
pub(crate) fn user_id(users: &BTreeMap<String, Vec<String>>, name: &str) -> u32 {
    match name {
        "root" => 0,
        "nobody" => 65534,
        _ => {
            1000 + users
                .keys()
                .filter(|u| u.as_str() != "root" && u.as_str() != "nobody")
                .position(|u| u == name)
                .unwrap_or(999) as u32
        }
    }
}
pub(crate) fn group_id(users: &BTreeMap<String, Vec<String>>, name: &str) -> u32 {
    if users.contains_key(name) {
        return user_id(users, name);
    }
    let groups: std::collections::BTreeSet<_> = users
        .values()
        .flatten()
        .filter(|g| !users.contains_key(*g))
        .collect();
    2000 + groups
        .iter()
        .position(|g| g.as_str() == name)
        .unwrap_or(999) as u32
}

fn long_listing(path: &str, n: &Node) -> String {
    format!(
        "{} {} {} {} 0 Oct 21 09:00 {path}",
        mode_string(n),
        if n.directory { 2 } else { 1 },
        n.owner,
        n.group
    )
}
fn service_status(running: bool, enabled: bool, failed: bool, loaded_port: Option<u16>) -> String {
    let mark = if failed { "x" } else { "*" };
    let boot = if enabled { "enabled" } else { "disabled" };
    let active = if running {
        "active (running)"
    } else if failed {
        "failed (Result: exit-code)"
    } else {
        "inactive (dead)"
    };
    let runtime=loaded_port.filter(|_|running).map(|p|format!("\n   Main PID: 100 (atlas)\n      Tasks: 1\n     CGroup: /system.slice/atlas.service\n             100 atlas --port {p}")).unwrap_or_default();
    format!(
        "{mark} atlas.service - Atlas application service\n     Loaded: loaded (/etc/systemd/system/atlas.service; {boot}; preset: disabled)\n     Active: {active}{runtime}"
    )
}
fn journal_output(journal: &[ServiceEvent]) -> String {
    if journal.is_empty() {
        return "-- No entries --\nLegacy attempts may not contain journal history; a start/restart records modeled events.".into();
    }
    journal.iter().enumerate().map(|(i,event)|{
        let stamp=format!("Oct 21 09:{:02}:00 lab",i.min(59));
        match event{
            ServiceEvent::Rejected{port}=>format!("{stamp} atlas[100]: Invalid port '{port}': expected integer 1..65535\n{stamp} systemd[1]: atlas.service: Main process exited, code=exited, status=1/FAILURE\n{stamp} systemd[1]: atlas.service: Failed with result 'exit-code'."),
            ServiceEvent::Started{port}=>format!("{stamp} atlas[100]: Listening on port {port}\n{stamp} systemd[1]: Started atlas.service - Atlas application service."),
            ServiceEvent::Stopped=>format!("{stamp} systemd[1]: Stopped atlas.service - Atlas application service."),
        }
    }).collect::<Vec<_>>().join("\n")
}
