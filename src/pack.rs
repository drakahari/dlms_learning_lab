use crate::engine::{self, Goal, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_link_teaching: Option<crate::physical_link_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub one_off_teaching: Option<crate::one_off_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_backup_teaching: Option<crate::file_backup_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etherchannel_teaching: Option<crate::etherchannel_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron_teaching: Option<crate::cron_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nfs_teaching: Option<crate::nfs_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nat_teaching: Option<crate::nat_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rpm_packages_teaching: Option<crate::rpm_packages_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apache_static_teaching: Option<crate::apache_static_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spanning_tree_teaching: Option<crate::spanning_tree_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dhcpv4_teaching: Option<crate::dhcpv4_teaching::Lesson>,
    pub id: String,
    pub version: u32,
    pub track: String,
    pub topic: String,
    #[serde(default)]
    pub unit: String,
    pub title: String,
    pub summary: String,
    pub objective: String,
    #[serde(default)]
    pub retry_guidance: String,
    pub difficulty: String,
    pub minutes: u16,
    pub steps: Vec<String>,
    #[serde(default)]
    pub guide: Vec<LessonStep>,
    #[serde(default)]
    pub explanation: String,
    pub hints: Vec<String>,
    pub sources: Vec<String>,
    pub simplifications: String,
    pub initial: State,
    pub goal: Goal,
    pub solution: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teaching: Option<crate::script_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_teaching: Option<crate::terminal_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_teaching: Option<crate::process_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounts_teaching: Option<crate::accounts_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_manager_teaching: Option<crate::unit_manager_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ospf_teaching: Option<crate::ospf_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_teaching: Option<crate::discovery_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_teaching: Option<crate::storage_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acl_teaching: Option<crate::acl_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6_teaching: Option<crate::ipv6_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4_routing_teaching: Option<crate::ipv4_routing_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_teaching: Option<crate::address_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_teaching: Option<crate::service_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_teaching: Option<crate::client_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_dns_teaching: Option<crate::seed_dns_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_mount_teaching: Option<crate::seed_mount_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_access_teaching: Option<crate::legacy_access_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_logs_teaching: Option<crate::legacy_logs_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_teaching: Option<crate::dns_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switching_teaching: Option<crate::switching_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac_teaching: Option<crate::mac_learning_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debian_packages_teaching: Option<crate::debian_packages_teaching::Lesson>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonStep {
    pub title: String,
    pub why: String,
    pub action: String,
    pub commands: Vec<CommandExample>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandExample {
    pub command: String,
    pub explanation: String,
    pub simulator_only: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub schema_version: u32,
    pub scenarios: Vec<Scenario>,
}
/// Developer pack-file validation; this entry point is never a learner command.
/// Bound ordinary-file input before UTF-8 decoding or JSON allocation.
pub fn load_file(path: &std::path::Path) -> Result<Vec<Scenario>, String> {
    let bytes = crate::local_file::read_bounded(path, MAX_PACK_BYTES)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| "Scenario packs must contain UTF-8 JSON".to_string())?;
    load(&text)
}
pub const MAX_PACK_BYTES: usize = collection_budget::PACK_BYTES;
/// Source-owned static catalog file capacity; public load_catalog remains100.
pub const MAX_BUNDLED_PACK_FILES: usize = 128;
/// Shared authoring/filter labels; difficulty is advisory, never a mastery gate.
pub const DIFFICULTIES: [&str; 3] = ["Beginner", "Intermediate", "Advanced"];
pub fn load(text: &str) -> Result<Vec<Scenario>, String> {
    if text.len() > MAX_PACK_BYTES {
        return Err("Pack exceeds 2 MB".into());
    }
    let p: Pack = serde_json::from_str(text).map_err(|e| e.to_string())?;
    validate(p)
}
fn validate(p: Pack) -> Result<Vec<Scenario>, String> {
    if p.schema_version != 1 || p.scenarios.is_empty() || p.scenarios.len() > 1000 {
        return Err("Unsupported schema or catalog size".into());
    }
    let mut ids = BTreeSet::new();
    for s in &p.scenarios {
        if let State::MagicSubnet { session } = &s.initial
            && (s.track != "CCNA"
                || !matches!(&s.goal, Goal::MagicSubnet { problem, stage } if *problem==session.problem && *stage==session.stage))
        {
            return Err("Subnet-method task and supplied worksheet must agree".into());
        }
        if s.id.is_empty()
            || s.id.len() > 80
            || !s
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !ids.insert(&s.id)
        {
            return Err("Invalid or duplicate scenario identifier".into());
        }
        if s.version == 0
            || !["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].contains(&s.track.as_str())
            || !DIFFICULTIES.contains(&s.difficulty.as_str())
            || s.minutes == 0
            || s.retry_guidance.len() > 1000
            || s.minutes > 180
            || s.unit.len() > 80
            || (!s.unit.is_empty()
                && !s
                    .unit
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-'))
        {
            return Err(format!("{}: invalid metadata", s.id));
        }
        if [
            &s.title,
            &s.summary,
            &s.objective,
            &s.topic,
            &s.simplifications,
        ]
        .iter()
        .any(|v| v.trim().is_empty() || v.len() > 4000)
            || [&s.steps, &s.hints, &s.sources].iter().any(|v| {
                v.is_empty()
                    || v.len() > 50
                    || v.iter().any(|v| v.trim().is_empty() || v.len() > 4000)
            })
            || s.solution.is_empty()
            || s.solution.len() > 50
            || s.solution.iter().any(|v| {
                v.trim().is_empty()
                    || v.len()
                        > if matches!(s.initial, State::PhysicalLink { .. }) {
                            crate::physical_link_workspace::PREFIX.len()
                                + crate::physical_link::MAX_ACTION
                        } else if matches!(s.initial, State::EtherChannel { .. }) {
                            crate::etherchannel_workspace::PREFIX.len()
                                + crate::etherchannel::MAX_ACTION_BYTES
                        } else if matches!(s.initial, State::Nfs { .. }) {
                            crate::nfs_workspace::PREFIX.len() + crate::nfs::MAX_ACTION
                        } else if matches!(s.initial, State::RpmPackages { .. }) {
                            crate::rpm_packages_terminal::MAX_COMMAND
                        } else if matches!(s.initial, State::BasicNat { .. }) {
                            crate::nat_workspace::PREFIX.len() + crate::nat::MAX_ACTION_JSON
                        } else if matches!(s.initial, State::ApacheStatic { .. }) {
                            crate::apache_static_workspace::PREFIX.len()
                                + crate::apache_static::MAX_ACTION_JSON
                        } else if matches!(s.initial, State::Ipv4Acl { .. }) {
                            crate::acl_editor::MAX_JSON + 64
                        } else if matches!(s.initial, State::DebianPackages { .. }) {
                            crate::debian_packages_workspace::MAX_ACTION_BYTES
                        } else if matches!(s.initial, State::SpanningTree { .. }) {
                            crate::spanning_tree_workspace::PREFIX.len()
                                + crate::spanning_tree::MAX_ACTION_BYTES
                        } else if matches!(s.initial, State::DhcpV4 { .. }) {
                            crate::dhcpv4_workspace::MAX_ACTION_BYTES
                        } else if matches!(s.initial, State::UnitManager { .. }) {
                            crate::unit_manager_workspace::MAX_ACTION_BYTES
                        } else {
                            4000
                        }
            })
            || s.sources.iter().any(|v| !v.starts_with("https://"))
        {
            return Err(format!("{}: missing or oversized authoring fields", s.id));
        }
        let invalid_guide = s.guide.iter().any(|g| {
            [&g.title, &g.why, &g.action]
                .iter()
                .any(|t| t.trim().is_empty() || t.len() > 4000)
                || g.commands.len() > 10
                || g.commands.iter().any(|c| {
                    c.command.trim().is_empty()
                        || c.command.len() > 4096
                        || c.explanation.trim().is_empty()
                        || c.explanation.len() > 4000
                })
        });
        if (!s.guide.is_empty() && (s.guide.len() != s.steps.len() || invalid_guide))
            || s.explanation.len() > 8000
        {
            return Err(format!("{}: invalid learning guidance", s.id));
        }
        if matches!(s.initial,State::StartupFiles{..}) || matches!(s.goal,Goal::StartupFiles{..}) {
            crate::startup_workspace::source(s)?.validate()?;
        }
        if matches!(s.initial,State::FileIndex{..}) || matches!(s.goal,Goal::FileIndex{..}) {
            let definition=crate::file_index_source::source(s)?;
            definition.validate_reachability().map_err(|e|e.to_string())?;
            if s.steps.len()<3||s.explanation.trim().is_empty(){return Err("Private-index lessons need essential teaching, separate practice and full assignment instructions".into())}
        }
        if matches!(s.initial,State::SharedLibraries{..}) || matches!(s.goal,Goal::SharedLibraries{..}) {
            crate::shared_libraries_workspace::source(s)?;
            let Goal::SharedLibraries{definition}=&s.goal else{return Err("Missing library goal".into())};
            definition.verify_worked()?;
            if s.steps.len()<3||s.explanation.trim().is_empty(){return Err("Library lessons need taught concepts, separate practice and full assignment instructions".into());}
        }
        if matches!(s.initial,State::Raid1{..}) || matches!(s.goal,Goal::Raid1{..}) {
            crate::raid1_workspace::source(s)?;
            let Goal::Raid1{definition}=&s.goal else{return Err("Missing mirror goal".into())};
            definition.validate()?;
            if s.steps.len()<3||s.explanation.trim().is_empty(){return Err("Mirror lessons need essential teaching, separate practice and full assignment instructions".into())}
        }
        if matches!(s.initial,State::FsIntegrity{..}) || matches!(s.goal,Goal::FsIntegrity{..}) {
            crate::fs_integrity_workspace::source(s)?;
        }
        if matches!(s.initial,State::UserMounts{..}) || matches!(s.goal,Goal::UserMounts{..}) {
            crate::user_mounts_workspace::source(s)?.validate_authored()?;
            if s.steps.len()<3 || s.explanation.trim().is_empty() {return Err("Caller-aware mount lessons need essential teaching, worked practice and assignment instructions".into());}
        }
        if matches!(s.initial, State::Environment {..}) || matches!(s.goal,Goal::Environment {..}) {
            let d=crate::environment_workspace::source(s)?;
            let lesson=s.teaching.as_ref().ok_or("Environment lesson needs an introduction and separate example")?;
            if lesson.example!=d.example.starter || lesson.fixture!=d.example.cases[0].input {
                return Err("Environment worked source/input must match its separate example task".into());
            }
            d.example.validate_child_examples()?;
            let r=crate::environment_workspace::run(&d.example,&d.example.starter)?;
            let draft=crate::environment_workspace::Draft{source:d.example.starter.clone(),result:Some(r)};
            if !crate::environment_workspace::checks(&d.example,&draft).iter().all(|(ok,_)|*ok) {
                return Err("Environment worked example does not reach its own requirements".into());
            }
        }
        if matches!(s.initial, State::DebianSources { .. })
            || matches!(s.goal, Goal::DebianSources { .. })
        {
            crate::debian_sources_workspace::source(s)?;
        }
        if matches!(s.initial, State::ClientNames { .. })
            || matches!(s.goal, Goal::ClientNames { .. })
        {
            crate::client_names_workspace::source(s)?;
        }
        if matches!(s.initial, State::RemovableMounts { .. })
            || matches!(s.goal, Goal::RemovableMounts { .. })
        {
            crate::removable_workspace::source(s)?;
        }
        if matches!(s.initial, State::NmProfiles { .. })
            || matches!(s.goal, Goal::NmProfiles { .. })
        {
            crate::nm_workspace::source(s)?;
        }
        if matches!(s.initial, State::PamStack { .. })
            || matches!(s.goal, Goal::PamStack { .. })
        {
            crate::pam_workspace::source(s)?;
        }
        if matches!(s.initial, State::Aging { .. })
            || matches!(s.goal, Goal::Aging { .. })
        {
            crate::aging_workspace::source(s)?;
        }
        if matches!(s.initial, State::PortSecurity { .. })
            || matches!(s.goal, Goal::PortSecurity { .. })
        {
            crate::port_security_workspace::source(s)?;
        }
        if matches!(s.initial, State::DhcpSnooping { .. })
            || matches!(s.goal, Goal::DhcpSnooping { .. })
        {
            crate::dhcp_snooping_workspace::source(s)?;
        }
        if matches!(s.initial, State::ArpInspection { .. })
            || matches!(s.goal, Goal::ArpInspection { .. })
        {
            crate::arp_inspection_workspace::source(s)?;
        }
        if matches!(s.initial, State::ManagementPolicy { .. })
            || matches!(s.goal, Goal::ManagementPolicy { .. })
        {
            crate::management_workspace::source(s)?;
        }
        if matches!(s.initial, State::LocalAccess { .. })
            || matches!(s.goal, Goal::LocalAccess { .. })
        {
            crate::local_access_workspace::source(s)?;
        }
        if matches!(s.initial, State::AaaAccess { .. })
            || matches!(s.goal, Goal::AaaAccess { .. })
        {
            crate::aaa_workspace::source(s)?;
        }
        if matches!(s.initial, State::LdapClient { .. })
            || matches!(s.goal, Goal::LdapClient { .. })
        {
            crate::ldap_workspace::source(s)?;
        }
        if matches!(s.initial, State::LdapAccess { .. })
            || matches!(s.goal, Goal::LdapAccess { .. })
        {
            crate::ldap_access_workspace::source(s)?;
        }
        if matches!(s.initial, State::LdapSchema { .. })
            || matches!(s.goal, Goal::LdapSchema { .. })
        {
            crate::ldap_schema_workspace::source(s)?;
        }
        if matches!(s.initial,State::CopiedJournal{..}) || matches!(s.goal,Goal::CopiedJournal{..}) { crate::journal_archive_workspace::source(s)?; }
        if matches!(s.initial,State::JsonInventory{..}) || matches!(s.goal,Goal::JsonInventory{..}) { crate::json_inventory_workspace::source(s)?; }
        if matches!(s.initial,State::RestController{..}) || matches!(s.goal,Goal::RestController{..}) { crate::rest_workspace::source(s)?; }
        if matches!(s.initial,State::DevicePath{..}) || matches!(s.goal,Goal::DevicePath{..}) { crate::device_path_workspace::source(s)?; }
        if matches!(s.initial,State::LocalMail{..}) || matches!(s.goal,Goal::LocalMail{..}) { crate::local_mail_workspace::source(s)?; }
        if matches!(s.initial,State::Chrony{..}) || matches!(s.goal,Goal::Chrony{..}) { crate::chrony_workspace::source(s)?; }
        if matches!(s.initial,State::GrubOnce{..}) || matches!(s.goal,Goal::GrubOnce{..}) { crate::grub_once_workspace::source(s)?; }
        if matches!(s.initial,State::EfiLoader{..}) || matches!(s.goal,Goal::EfiLoader{..}) { crate::efi_loader_workspace::source(s)?; }
        if matches!(s.initial,State::BiosLoader{..}) || matches!(s.goal,Goal::BiosLoader{..}) { crate::bios_loader_workspace::source(s)?; }
        if matches!(s.initial,State::BootEntry{..}) || matches!(s.goal,Goal::BootEntry{..}) { crate::boot_entry_workspace::source(s)?; }
        if matches!(s.initial, State::LdapSchemaQuery { .. })
            || matches!(s.goal, Goal::LdapSchemaQuery { .. })
        {
            crate::ldap_schema_query_workspace::source(s)?;
        }
        if matches!(s.initial, State::LdapPassword { .. })
            || matches!(s.goal, Goal::LdapPassword { .. })
        {
            crate::ldap_password_workspace::source(s)?;
        }
        if matches!(s.initial, State::LdapChanges { .. })
            || matches!(s.goal, Goal::LdapChanges { .. })
        {
            crate::ldap_change_workspace::source(s)?;
        }
        if matches!(s.initial, State::DhcpConfig { .. })
            || matches!(s.goal, Goal::DhcpConfig { .. })
        {
            crate::dhcp_server_workspace::source(s)?;
        }
        if matches!(s.initial, State::Qos { .. }) || matches!(s.goal, Goal::Qos { .. }) {
            crate::qos_workspace::source(s)?;
        }
        if matches!(s.initial, State::LinuxLogging { .. })
            || matches!(s.goal, Goal::LinuxLogging { .. })
        {
            crate::linux_logging_workspace::source(s)?;
        }
        if matches!(s.initial,State::VirtualIsolation{..})||matches!(s.goal,Goal::VirtualIsolation{..}){crate::virtual_isolation_source::source(s)?;}
        if matches!(s.initial,State::SieveFiltering{..})||matches!(s.goal,Goal::SieveFiltering{..}){crate::sieve_source::source(s)?;}
        if matches!(s.initial,State::LinuxRouter{..})||matches!(s.goal,Goal::LinuxRouter{..}){crate::linux_router_source::source(s)?;}
        if matches!(s.initial,State::LinuxV6{..})||matches!(s.goal,Goal::LinuxV6{..}){crate::linux_v6_source::source(s)?;}
        if matches!(s.initial,State::LinuxServiceRouter{..})||matches!(s.goal,Goal::LinuxServiceRouter{..}){crate::linux_service_router_source::source(s)?;}
        if matches!(s.initial,State::LinuxV6Administration{..})||matches!(s.goal,Goal::LinuxV6Administration{..}){crate::linux_v6_admin_source::source(s)?;}
        if matches!(s.initial,State::LinuxV6Startup{..})||matches!(s.goal,Goal::LinuxV6Startup{..}){crate::linux_v6_startup_source::source(s)?;}
        if matches!(s.initial,State::IdsRule{..})||matches!(s.goal,Goal::IdsRule{..}){crate::ids_rule_source::source(s)?;}
        if matches!(s.initial,State::AuthFilter{..})||matches!(s.goal,Goal::AuthFilter{..}){crate::auth_filter_source::source(s)?;}
        if matches!(s.initial,State::VpnPoint{..})||matches!(s.goal,Goal::VpnPoint{..}){crate::vpn_point_source::source(s)?;}
        if matches!(s.initial,State::Poe{..})||matches!(s.goal,Goal::Poe{..}){crate::poe_source::source(s)?;}
        if matches!(s.initial,State::KernelBuild{..})||matches!(s.goal,Goal::KernelBuild{..}){crate::kernel_source::source(s)?;}
        if matches!(s.initial,State::Dkms{..})||matches!(s.goal,Goal::Dkms{..}){crate::dkms_source::source(s)?;}
        if matches!(s.initial,State::Initramfs{..})||matches!(s.goal,Goal::Initramfs{..}){crate::initramfs_source::source(s)?;}
        if matches!(s.initial,State::Pxe{..})||matches!(s.goal,Goal::Pxe{..}){crate::pxe::source::source(s)?;}
        if matches!(s.initial,State::Iscsi{..})||matches!(s.goal,Goal::Iscsi{..}){crate::iscsi::source::source(s)?;}
        if matches!(s.initial,State::AtaSmart{..})||matches!(s.goal,Goal::AtaSmart{..}){crate::ata_smart::source::source(s)?;}
        if matches!(s.initial,State::ApacheHttps{..})||matches!(s.goal,Goal::ApacheHttps{..}){crate::apache_https::source::source(s)?;}
        if matches!(s.initial,State::Desktop{..})||matches!(s.goal,Goal::Desktop{..}){crate::desktop::source::source(s)?;}
        if matches!(s.initial,State::SecurityInvestigation{..})||matches!(s.goal,Goal::SecurityInvestigation{..}){crate::security_investigation::source::source(s)?;}
        if matches!(s.initial,State::AutomationWorkshop{..})||matches!(s.goal,Goal::AutomationWorkshop{..}){crate::automation_workshop::source::source(s)?;}
        if matches!(s.initial,State::ServiceExposure{..})||matches!(s.goal,Goal::ServiceExposure{..}){crate::service_exposure_source::source(s)?;}
        if matches!(s.initial,State::FtpIdentity{..})||matches!(s.goal,Goal::FtpIdentity{..}){crate::ftp_identity_source::source(s)?;}
        if matches!(s.initial,State::LinuxFlow{..})||matches!(s.goal,Goal::LinuxFlow{..}){crate::linux_flow_source::source(s)?;}
        if matches!(s.initial,State::MailboxAccess{..}|State::MailboxAdministration{..})||matches!(s.goal,Goal::MailboxAccess{..}|Goal::MailboxAdministration{..}){crate::mailbox_source::source(s)?;}
        if matches!(s.initial,State::SmtpRouting{..})||matches!(s.goal,Goal::SmtpRouting{..}){crate::smtp_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial,State::GuestDrivers{..})||matches!(s.goal,Goal::GuestDrivers{..}){crate::guest_driver_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial,State::SambaShare{..})||matches!(s.goal,Goal::SambaShare{..}){crate::samba_share_source::source(s)?;}
        if matches!(s.initial,State::CampusArchitecture{..})||matches!(s.goal,Goal::CampusArchitecture{..}){crate::network_architecture_source::source(s)?;}
        if matches!(s.initial,State::NetworkTransfer{..})||matches!(s.goal,Goal::NetworkTransfer{..}){crate::network_transfer_source::source(s)?;}
        if matches!(s.initial,State::EditorHandover{..})||matches!(s.goal,Goal::EditorHandover{..}){crate::editor_handover_source::source(s)?;}
        if matches!(s.initial,State::ModalEditing{..})||matches!(s.goal,Goal::ModalEditing{..}){crate::modal_source::source(s)?;}
        if matches!(s.initial,State::SshAccess{..})||matches!(s.goal,Goal::SshAccess{..}) {crate::ssh_workspace::source(s)?;}
        if matches!(s.initial, State::HostNetwork { .. })
            || matches!(s.goal, Goal::HostNetwork { .. })
        {
            crate::host_workspace::source(s)?;
        }
        if matches!(s.initial, State::Repositories { .. })
            || matches!(s.goal, Goal::Repositories { .. })
        {
            crate::repository_workspace::source(s)?;
        }
        if matches!(s.initial, State::MaintenanceNotice { .. }) || matches!(s.goal, Goal::MaintenanceNotice { .. }) {
            crate::maintenance_notice_workspace::source(s)?;
        }
        if matches!(s.initial, State::DnsAccess { .. }) || matches!(s.goal, Goal::DnsAccess { .. }) {
            crate::dns_access_workspace::source(s)?;
        }
        if matches!(s.initial, State::PrintQueues { .. }) || matches!(s.goal, Goal::PrintQueues { .. }) {
            crate::print_queue_workspace::source(s)?;
        }
        if matches!(s.initial, State::Localization { .. }) || matches!(s.goal, Goal::Localization { .. }) {
            crate::localization_workspace::source(s)?;
        }
        if matches!(s.initial, State::WirelessChannels { .. }) || matches!(s.goal, Goal::WirelessChannels { .. }) { crate::wireless_channel_workspace::source(s)?; }
        if matches!(s.initial, State::WlanProfiles { .. }) || matches!(s.goal, Goal::WlanProfiles { .. }) { crate::wlan_workspace::source(s)?; }
        if matches!(s.initial, State::HttpCache { .. }) || matches!(s.goal, Goal::HttpCache { .. }) { crate::http_cache_workspace::source(s)?; }
        if matches!(s.initial, State::ServiceClients { .. }) || matches!(s.goal, Goal::ServiceClients { .. }) { crate::service_workspace::source(s)?; }
        if matches!(s.initial, State::ReverseProxy { .. }) || matches!(s.goal, Goal::ReverseProxy { .. }) { crate::reverse_proxy_workspace::source(s)?; }
        if matches!(s.initial, State::ProxyPolicy { .. }) || matches!(s.goal, Goal::ProxyPolicy { .. }) { crate::proxy_workspace::source(s)?; }
        if matches!(s.initial, State::SnmpMonitor { .. }) || matches!(s.goal, Goal::SnmpMonitor { .. }) { crate::snmp_workspace::source(s)?; }
        if matches!(s.initial, State::WlanPaths { .. }) || matches!(s.goal, Goal::WlanPaths { .. }) { crate::wlan_paths_workspace::source(s)?; }
        if matches!(s.initial, State::Timezone { .. }) || matches!(s.goal, Goal::Timezone { .. }) {
            crate::timezone_workspace::source(s)?;
        }
        if matches!(s.initial, State::SourceInstall { .. }) || matches!(s.goal, Goal::SourceInstall { .. }) {
            crate::source_install_workspace::source(s)?;
        }
        if matches!(s.initial, State::CapacityPlanning { .. }) || matches!(s.goal, Goal::CapacityPlanning { .. }) {
            crate::capacity_planning_workspace::source(s)?;
        }
        if matches!(s.initial, State::ResourceObservations { .. }) || matches!(s.goal, Goal::ResourceObservations { .. }) {
            crate::resource_observation_workspace::source(s)?;
        }
        if matches!(s.initial, State::Capacity { .. }) || matches!(s.goal, Goal::Capacity { .. }) {
            crate::capacity_workspace::source(s)?;
        }
        if matches!(s.initial, State::RaidAssembly { .. }) || matches!(s.goal, Goal::RaidAssembly { .. }) {crate::raid_assembly_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial, State::Autofs { .. }) || matches!(s.goal, Goal::Autofs { .. }) {crate::autofs_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial, State::KernelModules { .. }) || matches!(s.goal, Goal::KernelModules { .. }) {crate::kernel_module_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial, State::RecursiveTree { .. }) || matches!(s.goal, Goal::RecursiveTree { .. }) {crate::recursive_tree_source::source(s)?.validate_reachability()?;}
        if matches!(s.initial, State::Transport { .. }) || matches!(s.goal, Goal::Transport { .. }) {crate::transport_workspace::source(s)?;}
        if matches!(s.initial, State::DirectoryPolicy { .. }) || matches!(s.goal, Goal::DirectoryPolicy { .. }) {crate::directory_policy_source::source(s)?;}
        if matches!(s.initial, State::Links { .. }) || matches!(s.goal, Goal::Links { .. }) {
            crate::links_workspace::source(s)?;
        }
        if matches!(s.initial, State::PartitionFs { .. })
            || matches!(s.goal, Goal::PartitionFs { .. })
        {
            let definition = crate::partition_fs_workspace::binding::source(s)?;
            definition.validate()?;
        }
        if matches!(s.initial, State::Lvm { .. }) || matches!(s.goal, Goal::Lvm { .. }) {
            crate::lvm_workspace::source(s)?;
            let Goal::Lvm { definition } = &s.goal else {
                return Err("Wrong LVM Goal".into());
            };
            definition.validate()?;
        }
        if matches!(s.initial, State::Syslog { .. }) || matches!(s.goal, Goal::Syslog { .. }) {
            crate::syslog_workspace::source(s)?;
            let Goal::Syslog { definition } = &s.goal else {
                return Err("Wrong Syslog Goal".into());
            };
            definition.validate()?;
        }
        if matches!(s.initial, State::Ntp { .. }) || matches!(s.goal, Goal::Ntp { .. }) {
            crate::ntp_workspace::source(s)?;
            let Goal::Ntp { definition } = &s.goal else {
                return Err("Wrong NTP Goal".into());
            };
            definition.validate()?;
        }
        if matches!(s.initial, State::Hsrp { .. }) || matches!(s.goal, Goal::Hsrp { .. }) {
            crate::hsrp_workspace::source(s)?;
            let Goal::Hsrp { definition } = &s.goal else {
                return Err("Wrong HSRP Goal".into());
            };
            definition.validate()?;
        }
        if let State::Vlsm { allocations, .. } = &s.initial
            && allocations
                .iter()
                .any(|a| !a.cidr.is_empty() && engine::cidr(&a.cidr).is_none())
        {
            return Err("Invalid initial subnet allocation".into());
        }
        if let State::MacLearning { simulation } = &s.initial {
            let Goal::MacLearning {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("MAC objectives need the matching state".into());
            };
            let Some(lesson) = &s.mac_teaching else {
                return Err("MAC activities need a separate introduction and practice".into());
            };
            if initial != simulation
                || !crate::mac_learning_workspace::valid_requirements(requirements, initial)
            {
                return Err("Invalid or unpinned MAC objectives".into());
            }
            lesson.validate()?;
            if s.guide.iter().any(|step| !step.commands.is_empty()) {
                return Err(
                    "MAC walkthroughs use native-action prose, not terminal commands".into(),
                );
            }
            if simulation
                .fabric()
                .hosts
                .keys()
                .any(|h| lesson.fixture.fabric().hosts.contains_key(h))
                || simulation.fabric().hosts.keys().any(|h| {
                    lesson
                        .fixture
                        .fabric()
                        .hosts
                        .keys()
                        .any(|other| simulation.address(h) == lesson.fixture.address(other))
                })
            {
                return Err("MAC practice must use different workstation IDs and addresses".into());
            }
        } else if s.mac_teaching.is_some() || matches!(s.goal, Goal::MacLearning { .. }) {
            return Err("MAC metadata attached to another activity".into());
        }
        if let Some(lesson) = &s.switching_teaching {
            if !matches!(s.initial, State::Switching { .. }) {
                return Err("Switching teaching attached to incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Switching { requirements } = &s.goal {
            let State::Switching { fabric } = &s.initial else {
                return Err("Switching goals require a switching fabric".into());
            };
            if !crate::switching::valid_requirements(requirements, fabric) {
                return Err(format!(
                    "{}: invalid switching goals; every isolation endpoint needs a positive service case",
                    s.id
                ));
            }
        }
        if let Some(lesson) = &s.service_teaching {
            if !matches!(s.initial, State::Service { .. })
                || !matches!(s.goal,Goal::Service{port}if port!=lesson.required_port)
            {
                return Err(
                    "Service teaching requires a distinct example port and service task".into(),
                );
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if s.seed_dns_teaching
            .as_ref()
            .is_some_and(|lesson| !lesson.valid_for(&s.initial, &s.goal))
        {
            return Err("Seed DNS teaching requires a distinct bounded one-record example".into());
        }
        if s.seed_mount_teaching
            .as_ref()
            .is_some_and(|lesson| !lesson.valid_for(&s.initial, &s.goal))
        {
            return Err("Seed mount teaching requires distinct fixed capacity and device".into());
        }
        if s.legacy_access_teaching
            .as_ref()
            .is_some_and(|l| !l.valid_for(&s.initial, &s.goal))
        {
            return Err("Legacy access teaching requires a distinct bounded Unix example".into());
        }
        if s.legacy_logs_teaching
            .as_ref()
            .is_some_and(|l| !l.valid_for(&s.initial, &s.goal))
        {
            return Err("Legacy log teaching requires a distinct bounded Logs example".into());
        }
        if let Some(lesson) = &s.dns_teaching {
            let State::AuthoritativeDns { server } = &s.initial else {
                return Err("DNS teaching needs an authoritative DNS activity".into());
            };
            if let Some(lookup) = &lesson.lookup {
                lookup.assignment.validate(server).map_err(|e|format!("{}: {e}",s.id))?;
            }
            if crate::dns::canonical_name(&server.configured.origin).ok()
                == crate::dns::canonical_name(&lesson.fixture.configured.origin).ok()
            {
                return Err("DNS example must use a different zone from the assignment".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let State::DebianPackages { simulation } = &s.initial {
            let Goal::DebianPackages { definition } = &s.goal else {
                return Err("Logical package state needs matching fixed goals".into());
            };
            if simulation != &definition.initial || definition.validate().is_err() {
                return Err(
                    "Package goals must pin the exact admitted initial state/catalog".into(),
                );
            }
            if s.guide.iter().any(|step| {
                step.commands.iter().any(|command| {
                    crate::debian_packages_terminal::validate_guidance(
                        &command.command,
                        command.simulator_only,
                    )
                    .is_err()
                })
            }) {
                return Err(
                    "Package guidance needs supported literal commands and labeled simulator helpers"
                        .into(),
                );
            }
            if let Some(lesson) = &s.debian_packages_teaching {
                lesson.validate()?;
                if !lesson.separate_from(simulation) {
                    return Err("Package practice must use separate catalogs, package/artifact/index and path identities".into());
                }
            }
        } else if s.debian_packages_teaching.is_some()
            || matches!(s.goal, Goal::DebianPackages { .. })
        {
            return Err("Package teaching/objectives on another activity".into());
        }
        if let State::PhysicalLink { simulation } = &s.initial {
            let Goal::PhysicalLink { definition } = &s.goal else {
                return Err("Physical-link State needs matching typed Goal".into());
            };
            let Some(lesson) = &s.physical_link_teaching else {
                return Err(
                    "Physical-link tasks need an introduction and separate runnable example".into(),
                );
            };
            if simulation != &definition.initial
                || !crate::physical_link_workspace::source_shape(s)
                || !crate::physical_link_workspace::within(
                    s,
                    crate::physical_link_workspace::MAX_SCENARIO,
                )
            {
                return Err("Physical-link initial/context differs or exceeds its bound".into());
            }
            definition.validate()?;
            lesson.validate(definition)?;
            if s.guide.iter().any(|step| !step.commands.is_empty()) {
                return Err("Physical-link tasks use native tools; internal author JSON must not be taught as IOS".into());
            }
        } else if s.physical_link_teaching.is_some() || matches!(s.goal, Goal::PhysicalLink { .. })
        {
            return Err("Physical-link teaching/Goal belongs to another activity".into());
        }
        if let State::FileBackup { simulation } = &s.initial {
            let Goal::FileBackup { definition } = &s.goal else {
                return Err("Backup state needs its matching authored goal".into());
            };
            if simulation != &definition.initial
                || definition.validate().is_err()
                || !crate::file_backup_workspace::within(
                    s,
                    crate::file_backup_workspace::MAX_SCENARIO,
                )
            {
                return Err("Backup initial files or bounded objectives differ".into());
            }
            let Some(lesson) = &s.file_backup_teaching else {
                return Err(
                    "Backup tasks need a separate introduction and runnable example".into(),
                );
            };
            lesson.validate()?;
            if !lesson.separate_from(simulation) {
                return Err("Backup practice and assignment paths must be separate".into());
            }
        } else if s.file_backup_teaching.is_some() || matches!(s.goal, Goal::FileBackup { .. }) {
            return Err("Backup teaching/objectives on another activity".into());
        }
        if let State::EtherChannel { simulation } = &s.initial {
            let Goal::EtherChannel { definition } = &s.goal else {
                return Err("LACP State needs its matching authored Goal".into());
            };
            if simulation != &definition.initial
                || definition.validate().is_err()
                || !crate::etherchannel_workspace::within(
                    s,
                    crate::etherchannel_workspace::MAX_SCENARIO,
                )
            {
                return Err("LACP exact initial/source aggregate differs".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err("LACP uses native editors; internal JSON is not taught as IOS".into());
            }
            if let Some(l) = &s.etherchannel_teaching {
                l.validate()?;
                if !l.separate_from(simulation) {
                    return Err("LACP practice needs disjoint supplied identities".into());
                }
            }
        } else if s.etherchannel_teaching.is_some() || matches!(s.goal, Goal::EtherChannel { .. }) {
            return Err("LACP Goal/teaching belongs to another activity".into());
        }
        if let State::OneOff { simulation } = &s.initial {
            let Goal::OneOff { definition } = &s.goal else {
                return Err("One-off State needs its authored Goal".into());
            };
            if !crate::one_off_workspace::source_shape(s)
                || simulation.simulation != definition.initial
                || simulation.last_run.is_some()
                || definition.validate().is_err()
                || !crate::cron_workspace::within(s, crate::one_off_workspace::MAX_CONTEXT)
            {
                return Err("One-off initial/source context differs or exceeds2MiB".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err("One-off native actions must not teach the internal JSON authoring codec as Linux".into());
            }
            if crate::one_off_workspace::budget::fresh_size(s, 0).is_none() {
                return Err("Fresh complete one-off workspace exceeds2MiB before copy".into());
            }
            if let Some(l) = &s.one_off_teaching {
                l.validate()?;
                if !l.separate_from(definition) {
                    return Err(
                        "One-off practice needs disjoint supplied files and accounts".into(),
                    );
                }
            }
        } else if s.one_off_teaching.is_some() || matches!(s.goal, Goal::OneOff { .. }) {
            return Err("One-off Goal/teaching attached to another activity".into());
        }
        if let State::Cron { simulation } = &s.initial {
            let Goal::Cron { definition } = &s.goal else {
                return Err("Scheduling state needs its matching authored goal".into());
            };
            if simulation != &definition.initial
                || definition.validate().is_err()
                || !crate::cron_workspace::within(s, crate::cron_workspace::MAX_CONTEXT)
            {
                return Err("Scheduling initial context or complete Scenario bound differs".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err(
                    "Scheduling uses native editors and Run, not learner JSON commands".into(),
                );
            }
            if let Some(l) = &s.cron_teaching {
                l.validate()?;
                if !l.separate_from(definition) {
                    return Err(
                        "Scheduling practice must use separate supplied users and files".into(),
                    );
                }
            }
        } else if s.cron_teaching.is_some() || matches!(s.goal, Goal::Cron { .. }) {
            return Err("Scheduling goal or teaching on another activity".into());
        }
        if let State::Nfs { simulation } = &s.initial {
            let Goal::Nfs { definition } = &s.goal else {
                return Err("NFS State needs its matching authored goal".into());
            };
            if simulation != &definition.initial
                || definition.validate().is_err()
                || !crate::nfs_workspace::within(s, crate::nfs_workspace::MAX_SCENARIO)
            {
                return Err("NFS exact initial context or full Scenario bound differs".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err(
                    "NFS uses explicit virtual editors/actions, not learner JSON commands".into(),
                );
            }
            if let Some(l) = &s.nfs_teaching {
                l.validate()?;
                if !l.separate_from(simulation) {
                    return Err("NFS practice must use separate complete supplied context".into());
                }
            }
        } else if s.nfs_teaching.is_some() || matches!(s.goal, Goal::Nfs { .. }) {
            return Err("NFS goal or teaching on another activity".into());
        }
        if let State::RpmPackages { simulation } = &s.initial {
            let Goal::RpmPackages { definition } = &s.goal else {
                return Err("RPM needs its matching authored goal".into());
            };
            if simulation != &definition.initial
                || definition.validate().is_err()
                || !crate::rpm_packages_workspace::within(
                    s,
                    crate::rpm_packages_workspace::MAX_SCENARIO,
                )
            {
                return Err("RPM exact initial context or complete Scenario bound differs".into());
            }
            if s.guide.iter().any(|step| {
                step.commands
                    .iter()
                    .any(|c| crate::rpm_packages_terminal::validate_guidance_for(definition,&c.command).is_err())
            }) {
                return Err("RPM guide command is outside the closed real-command grammar".into());
            }
            if let Some(l) = &s.rpm_packages_teaching {
                l.validate()?;
                if !l.separate_from_definition(definition) {
                    return Err(
                        "RPM practice must have independent catalogs/packages/capabilities/paths"
                            .into(),
                    );
                }
            }
        } else if s.rpm_packages_teaching.is_some() || matches!(s.goal, Goal::RpmPackages { .. }) {
            return Err("RPM metadata on another activity".into());
        }
        if let State::DhcpV4 { simulation } = &s.initial {
            let Goal::DhcpV4 {
                initial,
                requirements,
                probes,
            } = &s.goal
            else {
                return Err("DHCP state needs matching immutable objectives".into());
            };
            if simulation != initial
                || !crate::dhcpv4_workspace::valid_goal(initial, requirements, probes)
            {
                return Err("DHCP objectives must pin initial context and bounded positive current/fresh behavior".into());
            }
            if let Some(lesson) = &s.dhcpv4_teaching {
                lesson.validate()?;
                if !lesson.separate_from(initial) {
                    return Err(
                        "DHCP practice needs separate immutable server/segment/client identities"
                            .into(),
                    );
                }
            }
        } else if s.dhcpv4_teaching.is_some() || matches!(s.goal, Goal::DhcpV4 { .. }) {
            return Err("DHCP objectives/teaching attached to another activity".into());
        }

        if let State::BasicNat { simulation } = &s.initial {
            let Goal::BasicNat { definition } = &s.goal else {
                return Err("NAT state needs matching authored context/outcomes".into());
            };
            if simulation != &definition.initial || definition.validate().is_err() {
                return Err("NAT goal must pin exact admitted initial state".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err("NAT stage1 has no learner command guidance".into());
            }
            if let Some(l) = &s.nat_teaching {
                l.validate_for(simulation)?;
                if l.prerequisites != s.prerequisites {
                    return Err("NAT lesson prerequisites differ from actual IDs".into());
                }
            }
        } else if s.nat_teaching.is_some() || matches!(s.goal, Goal::BasicNat { .. }) {
            return Err("NAT goal on another activity".into());
        }
        if let State::ApacheStatic { simulation } = &s.initial {
            let Goal::ApacheStatic { definition } = &s.goal else {
                return Err("Apache state needs matching context/outcomes".into());
            };
            if simulation != &definition.initial || definition.validate().is_err() {
                return Err("Apache goals must pin exact admitted initial inventory/state".into());
            }
            if s.guide.iter().any(|g| !g.commands.is_empty()) {
                return Err(
                    "Apache native guidance cannot advertise unimplemented Linux commands".into(),
                );
            }
            if let Some(l) = &s.apache_static_teaching {
                l.validate()?;
                if !l.separate_from(simulation) {
                    return Err("Apache example requires separate fixed identities".into());
                }
            }
        } else if s.apache_static_teaching.is_some() || matches!(s.goal, Goal::ApacheStatic { .. })
        {
            return Err("Apache teaching/goals on another activity".into());
        }
        if let State::SpanningTree { simulation } = &s.initial {
            let Goal::SpanningTree {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("Spanning-tree state needs matching fixed-context goals".into());
            };
            if simulation != initial
                || !crate::spanning_tree_workspace::valid_requirements(initial, requirements)
            {
                return Err("Spanning-tree objectives must pin the initial graph, declared availability and positive connected behavior".into());
            }
            if let Some(lesson) = &s.spanning_tree_teaching {
                lesson.validate()?;
                if !lesson.separate_from(initial) {
                    return Err(
                        "Spanning-tree practice needs separate fixed bridge/link IDs and MACs"
                            .into(),
                    );
                }
                if lesson.prerequisites != s.prerequisites {
                    return Err("Spanning-tree lesson prerequisites must match the scenario's real prerequisite IDs".into());
                }
            }
        } else if s.spanning_tree_teaching.is_some() || matches!(s.goal, Goal::SpanningTree { .. })
        {
            return Err("Spanning-tree teaching/goals attached to another activity".into());
        }
        if let State::UnitManager { simulation } = &s.initial {
            let Goal::UnitManager { definition } = &s.goal else {
                return Err("Virtual unit state requires matching fixed objectives".into());
            };
            definition.validate()?;
            if simulation != &definition.initial {
                return Err(
                    "Virtual unit objectives must pin the exact authored original state".into(),
                );
            }
            let lesson = s.unit_manager_teaching.as_ref().ok_or(
                "Virtual unit activities need prerequisites and an isolated practice fixture",
            )?;
            lesson.validate()?;
            if !lesson.distinct_from(simulation) {
                return Err("Virtual unit practice needs distinct immutable origin, units and configuration paths".into());
            }
        } else if s.unit_manager_teaching.is_some() || matches!(s.goal, Goal::UnitManager { .. }) {
            return Err(
                "Virtual unit teaching/objectives belong only to their own workspace".into(),
            );
        }
        if let State::Ospf { simulation } = &s.initial {
            let Goal::Ospf {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("OSPF settings need matching fixed-context objectives".into());
            };
            if simulation != initial
                || !crate::ospf_workspace::valid_requirements(initial, requirements)
            {
                return Err("OSPF objectives must pin the initial graph and bounded meaningful positive outcomes".into());
            }
            let lesson = s.ospf_teaching.as_ref().ok_or(
                "OSPF activities need prerequisite teaching and a separate native practice fixture",
            )?;
            lesson.validate()?;
            if !lesson.separate_from(initial) {
                return Err("OSPF practice must use separate topology names, link identities, addresses and router IDs".into());
            }
        } else if s.ospf_teaching.is_some() || matches!(s.goal, Goal::Ospf { .. }) {
            return Err("OSPF objectives/teaching attached to another activity".into());
        }
        if let State::Ipv4Acl { network } = &s.initial {
            let Goal::Ipv4Acl { definition } = &s.goal else {
                return Err("ACL state requires a matching fixed flow definition".into());
            };
            if network != &definition.initial {
                return Err(
                    "Initial ACL state must exactly equal the definition's initial network".into(),
                );
            }
            definition.validate().map_err(|e| e.to_string())?;
            let lesson = s.acl_teaching.as_ref().ok_or(
                "ACL activities require introduction and a separate bounded practice fixture",
            )?;
            lesson.validate()?;
            if !lesson.distinct_from(network) {
                return Err(
                    "Use distinct example topology identities and address values for ACL practice"
                        .into(),
                );
            }
        } else if s.acl_teaching.is_some() || matches!(s.goal, Goal::Ipv4Acl { .. }) {
            return Err("ACL teaching or objectives attached to another activity".into());
        }
        if let State::Accounts { simulation } = &s.initial {
            let Goal::Accounts {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("Account activities require matching typed account goals".into());
            };
            let Some(lesson) = &s.accounts_teaching else {
                return Err("Account activities require validated prerequisite teaching and separate practice".into());
            };
            if initial != simulation || !crate::accounts::valid_requirements(initial, requirements)
            {
                return Err(
                    "Account goals must pin the authored initial state and valid outcomes".into(),
                );
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
            lesson.validate_assignment_templates(initial).map_err(|e| format!("{}: {e}", s.id))?;
            if !lesson.separate_from(simulation) {
                return Err("Use different nonroot user/group identities and a different supplied date in the ungraded account example".into());
            }
        } else if s.accounts_teaching.is_some() || matches!(s.goal, Goal::Accounts { .. }) {
            return Err("Account teaching/goals attached to another activity".into());
        }
        if let State::PersistentMounts { storage } = &s.initial {
            let Some(lesson) = &s.storage_teaching else {
                return Err("Persistent mount activities require validated introduction and separate practice".into());
            };
            let Goal::PersistentMounts {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("Storage objectives require the matching typed state".into());
            };
            if initial != storage.inventory()
                || !crate::storage_workspace::valid_requirements(requirements, initial)
            {
                return Err("Invalid or unpinned persistent mount objectives".into());
            }
            if storage.inventory().swap().is_some() != lesson.fixture.inventory().swap().is_some() {
                return Err("Storage assignment and practice must enable the same optional swap capability".into());
            }
            if let (Some(assignment), Some(example)) = (storage.inventory().swap(),lesson.fixture.inventory().swap())
                && assignment.areas.iter().any(|a|example.areas.iter().any(|e|e.uuid==a.uuid)) {
                return Err("Use different swap UUIDs for the separate ungraded storage example".into());
            }
            if storage.inventory().volumes().values().any(|v| {
                lesson
                    .fixture
                    .inventory()
                    .volumes()
                    .values()
                    .any(|other| other.uuid == v.uuid)
            }) {
                return Err("Use different volume UUIDs for the ungraded storage example".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        } else if s.storage_teaching.is_some() || matches!(s.goal, Goal::PersistentMounts { .. }) {
            return Err("Storage teaching/objectives attached to another activity".into());
        }
        if matches!(s.initial, State::Discovery { .. }) && s.discovery_teaching.is_none() {
            return Err(
                "Discovery activities require validated introduction and practice content".into(),
            );
        }
        if let Goal::Discovery {
            initial,
            requirements,
        } = &s.goal
        {
            let State::Discovery { simulation } = &s.initial else {
                return Err("Discovery objectives require a discovery topology".into());
            };
            if initial != simulation.topology()
                || !crate::discovery_workspace::valid_requirements(requirements, initial)
            {
                return Err("Invalid, unpinned or excessive discovery objectives".into());
            }
        }
        if let Some(lesson) = &s.discovery_teaching {
            let State::Discovery { simulation } = &s.initial else {
                return Err("Discovery teaching requires a discovery assignment".into());
            };
            if simulation
                .topology()
                .same_identity(lesson.fixture.topology())
            {
                return Err("Use a different topology for the discovery practice example".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Processes { requirements } = &s.goal {
            let State::Processes { table } = &s.initial else {
                return Err("Process objectives require a process table".into());
            };
            if requirements.is_empty()
                || requirements.len() > 64
                || requirements.iter().any(|r| !r.valid(table))
            {
                return Err("Invalid or excessive process objectives".into());
            }
        }
        if let Goal::AuthoritativeDns { requirements } = &s.goal {
            let State::AuthoritativeDns { server } = &s.initial else {
                return Err("DNS objectives need an authoritative DNS state".into());
            };
            if !crate::dns_workspace::valid_requirements(requirements, server) {
                return Err(
                    "DNS objectives need bounded outcomes, publication and a served lookup".into(),
                );
            }
        }
        if let Some(lesson) = &s.address_teaching {
            if crate::address_teaching::goal(&s.initial).is_none()
                || std::mem::discriminant(&s.initial) != std::mem::discriminant(&lesson.fixture)
                || !crate::address_teaching::distinct_example(&s.initial, &lesson.fixture)
            {
                return Err(format!(
                    "{}: addressing teaching needs a separate compatible example problem",
                    s.id
                ));
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.client_teaching
            && (!matches!(s.goal, Goal::Network { .. }) || !lesson.valid_for(&s.initial))
        {
            return Err(format!(
                "{}: client teaching needs a separate compatible single-client example",
                s.id
            ));
        }
        if let Some(lesson) = &s.ipv4_routing_teaching {
            if !matches!(s.initial, State::Ipv4Routing { .. }) {
                return Err("IPv4 routing teaching attached to an incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Ipv4Routing { requirements } = &s.goal {
            let State::Ipv4Routing { topology } = &s.initial else {
                return Err("IPv4 routing objectives require an IPv4 routing topology".into());
            };
            if !crate::ipv4_routing::valid_requirements(requirements, topology) {
                return Err(format!(
                    "{}: invalid IPv4 routing objectives or missing normal-state positive connectivity",
                    s.id
                ));
            }
        }
        if let Some(lesson) = &s.ipv6_teaching {
            if !matches!(s.initial, State::Ipv6 { .. }) {
                return Err("IPv6 teaching attached to an incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.process_teaching {
            let State::Processes { table } = &s.initial else {
                return Err("Process teaching requires a process assignment".into());
            };
            if table.same_problem(&lesson.fixture) {
                return Err("Use different process identities for the ungraded example".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.terminal_teaching {
            if !matches!(s.initial, State::Filesystem { .. }) {
                return Err("Terminal teaching attached to an incompatible lab type".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.teaching {
            if !matches!(s.initial, State::Script { .. } | State::Environment {..}) {
                return Err("Script teaching attached to a different lab type".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Ipv6 { requirements } = &s.goal {
            let State::Ipv6 { topology } = &s.initial else {
                return Err("IPv6 objective requires an IPv6 topology".into());
            };
            if requirements.is_empty()
                || requirements.len() > 20
                || requirements.iter().any(|r| !r.valid_for(topology))
                || !requirements.iter().any(
                    |r| matches!(r,crate::ipv6::Requirement::Reachable{when,..} if when.is_empty()),
                )
            {
                return Err(format!(
                    "{}: invalid IPv6 objectives or missing unconditional positive connectivity case",
                    s.id
                ));
            }
        }
        crate::command_location_workspace::source(s)?;
        if let Goal::Filesystem { requirements } = &s.goal {
            let State::Filesystem { tree } = &s.initial else {
                return Err("Filesystem objectives require a filesystem state".into());
            };
            let reference=crate::filesystem::discovery_policy::for_current(requirements,tree)?;
            if reference.is_some_and(|r|!r.valid_for(tree)) {
                return Err(format!("{}: command-location reference differs from its supplied initial state",s.id));
            }
            if requirements.is_empty()
                || requirements.len() > 20
                || requirements.iter().any(|r| !r.valid_for(tree))
            {
                return Err(format!("{}: invalid filesystem objectives", s.id));
            }
        }
        validate_state(&s.initial).map_err(|e| format!("{}: {e}", s.id))?;
        if engine::solved(&s.initial, &s.goal) {
            return Err(format!("{}: starts solved", s.id));
        }
        let mut state = s.initial.clone();
        if let Some(reference)=crate::accounts_template_workspace::reference_state(s)? {
            state=State::Accounts{simulation:reference};
        }
        if let (State::RaidAssembly { simulation }, Goal::RaidAssembly { definition }) = (&mut state, &s.goal) {
            for step in &definition.reference {let result=simulation.apply(definition,&step.operation,false)?;if result.status!=step.status{return Err(format!("{}: unreachable assembly reference",s.id));}}
            simulation.run(definition,crate::raid_assembly_workspace::CHECK,false)?;
        } else if let (State::Autofs { simulation }, Goal::Autofs { definition }) = (&mut state, &s.goal) {
            for step in &definition.reference {let result=simulation.apply(definition,&step.operation,false)?;if result.status!=step.status{return Err(format!("{}: unreachable AutoFS reference",s.id));}}
            simulation.run(definition,crate::autofs_workspace::CHECK,false)?;
        } else if let (State::PartitionFs { simulation }, Goal::PartitionFs { definition }) =
            (&mut state, &s.goal)
        {
            for operation in &definition.reference {
                simulation
                    .apply(definition, operation, false)
                    .map_err(|e| format!("{}: unreachable native reference: {e}", s.id))?;
            }
        } else if let (State::Lvm{simulation},Goal::Lvm{definition})=(&mut state,&s.goal)
            && let Some(task)=&definition.mount_task {
            for operation in &task.reference {simulation.mount_action(definition,operation,false)
                .map_err(|e|format!("{}: unreachable native mount reference: {e}",s.id))?;}
        } else if let (State::Lvm{simulation},Goal::Lvm{definition})=(&mut state,&s.goal)
            && let Some(task)=&definition.config_scan_task {
            for operation in &task.reference {simulation.config_action(definition,operation,false)
                .map_err(|e|format!("{}: unreachable native configuration reference: {e}",s.id))?;}
        } else {
            for cmd in &s.solution {
                engine::execute_for_goal(&mut state, &s.goal, cmd)
                    .map_err(|e| format!("{}: unreachable solution: {e}", s.id))?;
            }
        }
        if let (State::UserMounts{simulation},Goal::UserMounts{definition})=(&mut state,&s.goal) {
            let record=crate::user_mounts_workspace::run(&definition.assignment,&simulation.assignment)?;
            if !simulation.assignment.store(record,&definition.assignment){return Err("Mount reference checks exceeded their bounds".into());}
        }
        if let (State::Environment{simulation},Goal::Environment{definition})=(&mut state,&s.goal) {
            let record=crate::environment_workspace::run(&definition.assignment,&simulation.assignment.source)?;
            if !simulation.assignment.store(record,&definition.assignment){return Err("Environment reference result exceeded its bounds".into());}
        }
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut state, &s.goal) {
            if cases.len() < 3
                || cases.len() > crate::shell::MAX_CASES
                || cases.iter().any(|c| !c.valid())
            {
                return Err(format!("{}: invalid script cases", s.id));
            }
            *run = Some(crate::shell::Record {
                engine_revision: crate::shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases.iter().map(|c| crate::shell::run(source, c)).collect(),
            });
        }
        if !engine::solved(&state, &s.goal) {
            return Err(format!("{}: solution does not reach objective", s.id));
        }
    }
    for s in &p.scenarios {
        if s.prerequisites.len() > 8
            || s.prerequisites
                .iter()
                .any(|id| id == &s.id || !ids.contains(id))
        {
            return Err(format!("{}: invalid prerequisites", s.id));
        }
    }
    // Any cycle would make a prerequisite learning path impossible.
    for start in &p.scenarios {
        let mut pending = start.prerequisites.clone();
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if id == start.id {
                return Err(format!("{}: cyclic prerequisites", start.id));
            }
            if seen.insert(id.clone()) {
                pending.extend(
                    p.scenarios
                        .iter()
                        .find(|s| s.id == id)
                        .unwrap()
                        .prerequisites
                        .clone(),
                );
            }
        }
    }
    Ok(p.scenarios)
}
mod state_validation;
pub use state_validation::validate_state;
mod collection_budget;
/// Bound each declarative pack independently, then validate the combined prerequisite graph.
pub fn load_catalog(packs: &[&str]) -> Result<Vec<Scenario>, String> {
    load_catalog_with_limit(packs, 100)
}
// Source-owned static bundles may have more versioned files, under the same
// 32MB aggregate, 2MB per-file and 1,000-scenario budgets. External import stays100.
fn load_bundled_catalog(packs: &[&'static str]) -> Result<Vec<Scenario>, String> {
    load_catalog_with_limit(packs, MAX_BUNDLED_PACK_FILES)
}
fn load_catalog_with_limit(packs: &[&str], maximum_files: usize) -> Result<Vec<Scenario>, String> {
    collection_budget::collection(packs, maximum_files)?;
    let mut all = Pack {
        schema_version: 1,
        scenarios: Vec::new(),
    };
    for text in packs {
        collection_budget::pack(text)?;
        let p: Pack = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if p.schema_version != 1 || p.scenarios.is_empty() {
            return Err("Unsupported or empty pack".into());
        }
        collection_budget::scenarios(all.scenarios.len(), p.scenarios.len())?;
        all.scenarios.extend(p.scenarios);
    }
    validate(all)
}
fn bundled_texts() -> Vec<&'static str> {
    vec![
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/shell-unit-v1.json"),
        include_str!("../scenarios/shell-composition-v1.json"),
        include_str!("../scenarios/shell-reusable-v1.json"),
        include_str!("../scenarios/linux-files-v1.json"),
        include_str!("../scenarios/ipv6-static-v1.json"),
        include_str!("../scenarios/shell-scope-v1.json"),
        include_str!("../scenarios/shell-text-v1.json"),
        include_str!("../scenarios/shell-counts-v1.json"),
        include_str!("../scenarios/linux-access-v1.json"),
        include_str!("../scenarios/vlan-data-v1.json"),
        include_str!("../scenarios/shell-file-batches-v1.json"),
        include_str!("../scenarios/ipv4-static-v1.json"),
        include_str!("../scenarios/dns-forward-v1.json"),
        include_str!("../scenarios/shell-arrays-v1.json"),
        include_str!("../scenarios/linux-processes-v1.json"),
        include_str!("../scenarios/discovery-v1.json"),
        include_str!("../scenarios/storage-v1.json"),
        include_str!("../scenarios/shell-capture-v1.json"),
        include_str!("../scenarios/acl-policy-v1.json"),
        include_str!("../scenarios/accounts-lifecycle-v1.json"),
        include_str!("../scenarios/ospf-p2p-v1.json"),
        include_str!("../scenarios/unit-manager-v1.json"),
        include_str!("../scenarios/shell-case-v1.json"),
        include_str!("../scenarios/mac-learning-v1.json"),
        include_str!("../scenarios/linux-find-v1.json"),
        include_str!("../scenarios/dns-reverse-v1.json"),
        include_str!("../scenarios/shell-getopts-v1.json"),
        include_str!("../scenarios/dhcpv4-client-v1.json"),
        include_str!("../scenarios/debian-packages-v1.json"),
        include_str!("../scenarios/spanning-tree-v1.json"),
        include_str!("../scenarios/shell-safe-automation-v1.json"),
        include_str!("../scenarios/apache-static-v1.json"),
        include_str!("../scenarios/nat-basic-v1.json"),
        include_str!("../scenarios/rpm-packages-v1.json"),
        include_str!("../scenarios/shell-command-vectors-v1.json"),
        include_str!("../scenarios/nfs-v1.json"),
        include_str!("../scenarios/etherchannel-v1.json"),
        include_str!("../scenarios/shell-optional-inputs-v1.json"),
        include_str!("../scenarios/linux-cron-v1.json"),
        include_str!("../scenarios/file-backup-v1.json"),
        include_str!("../scenarios/shell-numeric-validation-v1.json"),
        include_str!("../scenarios/physical-links-v1.json"),
        include_str!("../scenarios/ipv4-private-planning-v1.json"),
        include_str!("../scenarios/linux-one-off-v1.json"),
        include_str!("../scenarios/shell-loop-recovery-v1.json"),
        include_str!("../scenarios/lvm-growth-v1.json"),
        include_str!("../scenarios/ntp-client-server-v1.json"),
        include_str!("../scenarios/shell-ere-evidence-v1.json"),
        include_str!("../scenarios/disk-preparation-v1.json"),
        include_str!("../scenarios/shell-literal-normalization-draft.json"),
        include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    ]
}
pub fn bundled() -> Result<Vec<Scenario>, String> {
    load_catalog(&bundled_texts())
}
/// Exact accepted209 comparison catalog; later units never rewrite its objects.
fn accepted_texts() -> Vec<&'static str> {
    let mut texts = bundled_texts();
    texts.push(include_str!("../scenarios/ccna-magic-number-v1.json"));
    texts.push(include_str!("../scenarios/hsrp-resilient-gateway-v1.json"));
    texts
}
pub fn accepted209() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted_texts())
}
/// Current native review library. Historical bundled/accepted sources stay intact.
fn accepted213_texts() -> Vec<&'static str> {
    let mut texts = accepted_texts();
    texts.push(include_str!("../scenarios/links-lessons-v1.json"));
    texts
}
pub fn accepted213() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted213_texts())
}
fn accepted221_texts() -> Vec<&'static str> {
    let mut texts = accepted213_texts();
    texts.push(include_str!("../scenarios/syslog-policy-v1.json"));
    texts.push(include_str!("../scenarios/capacity-lessons-v1.json"));
    texts
}
pub fn accepted221() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted221_texts())
}
fn accepted225_texts() -> Vec<&'static str> {
    let mut texts = accepted221_texts();
    texts.push(include_str!("../scenarios/repository-lessons-v1.json"));
    texts
}
pub fn accepted225() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted225_texts())
}
fn accepted229_texts() -> Vec<&'static str> {
    let mut texts = accepted225_texts();
    texts.push(include_str!("../scenarios/linux-host-lessons-v1.json"));
    texts
}
pub fn accepted229() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted229_texts())
}
fn accepted231_texts() -> Vec<&'static str> {
    let mut texts = accepted229_texts();
    texts.push(include_str!("../scenarios/shell-debug-recovery-v1.json"));
    texts
}
pub fn accepted231() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted231_texts())
}
fn accepted235_texts() -> Vec<&'static str> {
    let mut texts = accepted231_texts();
    texts.push(include_str!("../scenarios/linux-logging-lessons-v1.json"));
    texts
}
pub fn accepted235() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted235_texts())
}
fn accepted238_texts() -> Vec<&'static str> {
    let mut texts = accepted235_texts();
    texts.push(include_str!("../scenarios/linux-startup-lessons-v1.json"));
    texts
}
pub fn accepted238() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted238_texts())
}
fn accepted241_texts() -> Vec<&'static str> {
    let mut texts = accepted238_texts();
    texts.push(include_str!("../scenarios/qos-lessons-v1.json"));
    texts
}
pub fn accepted241() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted241_texts())
}
fn accepted242_texts() -> Vec<&'static str> {
    let mut texts = accepted241_texts();
    texts.push(include_str!("../scenarios/file-backup-selection-v1.json"));
    texts
}
pub fn accepted242() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted242_texts())
}
fn accepted244_texts() -> Vec<&'static str> {
    let mut texts = accepted242_texts();
    texts.push(include_str!("../scenarios/linux-dhcp-lessons-v1.json"));
    texts
}
pub fn accepted244() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted244_texts())
}
fn accepted247_texts() -> Vec<&'static str> {
    let mut texts = accepted244_texts();
    texts.push(include_str!("../scenarios/linux-nm-lessons-v1.json"));
    texts
}
/// Technical snapshot, not owner or exam acceptance.
pub fn accepted247() -> Result<Vec<Scenario>, String> {
    load_catalog(&accepted247_texts())
}
fn accepted248_texts() -> Vec<&'static str> {
    let mut texts = accepted247_texts();
    texts.push(include_str!("../scenarios/removable-mount-lessons-v1.json"));
    texts
}
/// Technical snapshot; no owner or exam acceptance.
pub fn accepted248() -> Result<Vec<Scenario>, String> { load_catalog(&accepted248_texts()) }
fn accepted250_texts() -> Vec<&'static str> {
    let mut texts=accepted248_texts();texts.push(include_str!("../scenarios/client-name-lessons-v1.json"));texts
}
/// Technical snapshot, not owner or full-curriculum acceptance.
pub fn accepted250()->Result<Vec<Scenario>,String>{load_catalog(&accepted250_texts())}
fn accepted252_texts() -> Vec<&'static str> {
    let mut texts=accepted250_texts();texts.push(include_str!("../scenarios/resolver-search-lessons-v1.json"));texts
}
/// Technical source snapshot, not owner or full-curriculum acceptance.
pub fn accepted252() -> Result<Vec<Scenario>, String> {load_catalog(&accepted252_texts())}
fn accepted253_texts()->Vec<&'static str>{
    let mut texts=accepted252_texts();texts.push(include_str!("../scenarios/debian-contents-lessons-v1.json"));texts
}
/// Technical snapshot; no owner, issuer or novice acceptance implied.
pub fn accepted253()->Result<Vec<Scenario>,String>{load_catalog(&accepted253_texts())}
fn accepted255_texts() -> Vec<&'static str> {
    let mut texts=accepted253_texts();texts.push(include_str!("../scenarios/debian-sources-lessons-v1.json"));texts
}
/// Exact preceding source-metadata catalog, before tracing lessons.
pub fn accepted255() -> Result<Vec<Scenario>, String> {load_catalog(&accepted255_texts())}
fn accepted257_texts() -> Vec<&'static str> {
    let mut texts=accepted255_texts();texts.push(include_str!("../scenarios/shell-tracing-lessons-v1.json"));texts
}
/// Exact prior tracing catalog; no owner or full-objective acceptance implied.
pub fn accepted257() -> Result<Vec<Scenario>, String> { load_catalog(&accepted257_texts()) }
fn accepted259_texts() -> Vec<&'static str> {
    let mut texts=accepted257_texts();texts.push(include_str!("../scenarios/environment-lessons-v1.json"));texts
}
/// Preceding verified technical catalog; no full-objective or owner acceptance implied.
pub fn accepted259() -> Result<Vec<Scenario>,String>{load_catalog(&accepted259_texts())}
fn accepted261_texts()->Vec<&'static str>{
    let mut texts=accepted259_texts();texts.push(include_str!("../scenarios/user-mount-lessons-v1.json"));texts
}
/// Preceding verified caller-mount catalog; not full-objective or owner acceptance.
pub fn accepted261()->Result<Vec<Scenario>,String>{load_catalog(&accepted261_texts())}
fn accepted263_texts()->Vec<&'static str>{
    let mut texts=accepted261_texts();texts.push(include_str!("../scenarios/debian-conflict-lessons-v1.json"));texts
}
/// Preceding package-conflict catalog; no complete-objective or owner acceptance.
pub fn accepted263()->Result<Vec<Scenario>,String>{load_catalog(&accepted263_texts())}
fn accepted265_texts()->Vec<&'static str>{
    let mut texts=accepted263_texts();texts.push(include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"));texts
}
/// Preceding automatic-Ethernet catalog; no full-objective or owner acceptance.
pub fn accepted265()->Result<Vec<Scenario>,String>{load_catalog(&accepted265_texts())}
fn accepted267_texts()->Vec<&'static str>{
    let mut texts=accepted265_texts();texts.push(include_str!("../scenarios/fs-integrity-lessons-v1.json"));texts
}
/// Preceding filesystem-consistency source snapshot; no full-objective acceptance.
pub fn accepted267()->Result<Vec<Scenario>,String>{load_catalog(&accepted267_texts())}
fn accepted269_texts()->Vec<&'static str>{
    let mut texts=accepted267_texts();texts.push(include_str!("../scenarios/shell-startup-lessons-v1.json"));texts
}
/// Technical snapshot before the file-lifecycle unit; no full-objective acceptance.
pub fn accepted269()->Result<Vec<Scenario>,String>{load_catalog(&accepted269_texts())}
fn accepted272_texts()->Vec<&'static str>{
    let mut texts=accepted269_texts();texts.push(include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"));texts
}
/// Technical snapshot before configuration-upgrade teaching; no full-objective acceptance.
pub fn accepted272()->Result<Vec<Scenario>,String>{load_catalog(&accepted272_texts())}
fn accepted275_texts()->Vec<&'static str>{
    let mut texts=accepted272_texts();texts.push(include_str!("../scenarios/debian-conffile-lessons-v1.json"));texts
}
/// Exact technical snapshot before mirrored-storage recovery; no issuer acceptance.
pub fn accepted275()->Result<Vec<Scenario>,String>{load_catalog(&accepted275_texts())}
fn accepted277_texts()->Vec<&'static str>{
    let mut texts=accepted275_texts();texts.push(include_str!("../scenarios/raid1-lessons-v1.json"));texts
}
/// Exact verified technical snapshot before named-map lessons; no full-objective acceptance.
pub fn accepted277()->Result<Vec<Scenario>,String>{load_catalog(&accepted277_texts())}
fn accepted279_texts()->Vec<&'static str>{
    let mut texts=accepted277_texts();texts.push(include_str!("../scenarios/shell-associative-lessons-v1.json"));texts
}
/// Exact technical snapshot before library-resolution practice; no full-objective acceptance.
pub fn accepted279()->Result<Vec<Scenario>,String>{load_catalog(&accepted279_texts())}
fn accepted281_texts()->Vec<&'static str>{
    let mut texts=accepted279_texts();texts.push(include_str!("../scenarios/shared-library-lessons-v1.json"));texts
}
/// Exact preceding library-search catalog; no full-objective acceptance implied.
pub fn accepted281()->Result<Vec<Scenario>,String>{load_catalog(&accepted281_texts())}
fn accepted283_texts()->Vec<&'static str>{
    let mut texts=accepted281_texts();texts.push(include_str!("../scenarios/fhs-lessons-v1.json"));texts
}
/// Exact preceding file-location catalog; no full-objective acceptance implied.
pub fn accepted283()->Result<Vec<Scenario>,String>{load_catalog(&accepted283_texts())}
fn accepted284_texts()->Vec<&'static str>{
    let mut texts=accepted283_texts();texts.push(include_str!("../scenarios/shell-managed-config-lessons-v1.json"));texts
}
/// Exact preceding managed-configuration catalog; no full-objective acceptance.
pub fn accepted284()->Result<Vec<Scenario>,String>{load_catalog(&accepted284_texts())}
fn accepted285_texts()->Vec<&'static str>{
    let mut texts=accepted284_texts();texts.push(include_str!("../scenarios/listener-audit-lessons-v1.json"));texts
}
/// Exact preceding listener-audit catalog; no full-objective acceptance.
pub fn accepted285()->Result<Vec<Scenario>,String>{load_catalog(&accepted285_texts())}
fn accepted286_texts()->Vec<&'static str>{
    let mut texts=accepted285_texts();texts.push(include_str!("../scenarios/path-lookup-lessons-v1.json"));texts
}
/// Exact preceding worker-search catalog, retained for compatibility checks.
pub fn accepted286()->Result<Vec<Scenario>,String>{load_catalog(&accepted286_texts())}
fn accepted287_texts()->Vec<&'static str>{
    let mut texts=accepted286_texts();texts.push(include_str!("../scenarios/account-skeleton-lessons-v1.json"));texts
}
/// Exact preceding account-default catalog, retained for compatibility checks.
pub fn accepted287()->Result<Vec<Scenario>,String>{load_catalog(&accepted287_texts())}
fn accepted288_texts()->Vec<&'static str>{
    let mut texts=accepted287_texts();texts.push(include_str!("../scenarios/command-location-lessons-v1.json"));texts
}
/// Exact preceding command-eligibility catalog, retained for compatibility checks.
pub fn accepted288()->Result<Vec<Scenario>,String>{load_catalog(&accepted288_texts())}
fn accepted289_texts()->Vec<&'static str>{
    let mut texts=accepted288_texts();texts.push(include_str!("../scenarios/file-index-lessons-v1.json"));texts
}
/// Exact preceding private-index catalog; no implicit source/progress rewriting.
pub fn accepted289()->Result<Vec<Scenario>,String>{load_catalog(&accepted289_texts())}
fn accepted290_texts()->Vec<&'static str>{
    let mut texts=accepted289_texts();texts.push(include_str!("../scenarios/whereis-lessons-v1.json"));texts
}
/// Exact preceding 290-lab source snapshot; no full-objective acceptance implied.
pub fn accepted290()->Result<Vec<Scenario>,String>{load_catalog(&accepted290_texts())}
fn accepted292_texts()->Vec<&'static str>{
    let mut texts=accepted290_texts();texts.push(include_str!("../scenarios/sticky-directory-lessons-v1.json"));texts
}
/// Exact preceding292-lab source snapshot; existing progress is not rewritten.
pub fn accepted292()->Result<Vec<Scenario>,String>{load_catalog(&accepted292_texts())}
fn accepted294_texts()->Vec<&'static str>{let mut texts=accepted292_texts();texts.push(include_str!("../scenarios/shell-content-backup-lessons-v1.json"));texts}
/// Exact verified content-recovery catalog, retained for compatibility checks.
pub fn accepted294()->Result<Vec<Scenario>,String>{load_catalog(&accepted294_texts())}
fn accepted296_texts()->Vec<&'static str>{let mut texts=accepted294_texts();texts.push(include_str!("../scenarios/transport-lessons-v1.json"));texts}
/// Exact verified transport catalog; retained for progress and preservation checks.
pub fn accepted296()->Result<Vec<Scenario>,String>{load_catalog(&accepted296_texts())}
fn accepted298_texts()->Vec<&'static str>{let mut texts=accepted296_texts();texts.push(include_str!("../scenarios/recursive-tree-lessons-v1.json"));texts}
/// Exact verified recursive-handover catalog; no later requirements credited.
pub fn accepted298()->Result<Vec<Scenario>,String>{load_catalog(&accepted298_texts())}
fn accepted300_texts()->Vec<&'static str>{let mut texts=accepted298_texts();texts.push(include_str!("../scenarios/find-size-lessons-v1.json"));texts}
/// Exact verified300 catalogue before module lessons.
pub fn accepted300()->Result<Vec<Scenario>,String>{load_catalog(&accepted300_texts())}
fn accepted303_texts()->Vec<&'static str>{let mut texts=accepted300_texts();texts.push(include_str!("../scenarios/kernel-module-lessons-v1.json"));texts}
/// Exact verified303 module catalogue before named-request work.
pub fn accepted303()->Result<Vec<Scenario>,String>{load_catalog(&accepted303_texts())}
fn accepted304_texts()->Vec<&'static str>{let mut texts=accepted303_texts();texts.push(include_str!("../scenarios/kernel-module-names-lessons-v1.json"));texts}
/// Exact verified304 catalogue before the cross-file log project.
pub fn accepted304()->Result<Vec<Scenario>,String>{load_catalog(&accepted304_texts())}
fn accepted305_texts()->Vec<&'static str>{let mut texts=accepted304_texts();texts.push(include_str!("../scenarios/shell-log-triage-lessons-v1.json"));texts}
/// Exact verified305 catalogue before RPM metadata recovery.
pub fn accepted305()->Result<Vec<Scenario>,String>{load_catalog(&accepted305_texts())}
fn accepted307_texts()->Vec<&'static str>{let mut texts=accepted305_texts();texts.push(include_str!("../scenarios/rpm-metadata-lessons-v1.json"));texts}
/// Exact verified307 snapshot; later requirements earn no inherited credit.
pub fn accepted307()->Result<Vec<Scenario>,String>{load_catalog(&accepted307_texts())}
fn accepted310_texts()->Vec<&'static str>{let mut texts=accepted307_texts();texts.push(include_str!("../scenarios/host-ipv6-lessons-v1.json"));texts}
/// Exact verified310 snapshot before swap policy; no inherited new requirements.
pub fn accepted310()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted310_texts())}
fn accepted312_texts()->Vec<&'static str>{let mut texts=accepted310_texts();texts.push(include_str!("../scenarios/swap-policy-lessons-v1.json"));texts}
pub fn accepted312()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted312_texts())}
fn accepted315_texts()->Vec<&'static str>{let mut texts=accepted312_texts();texts.push(include_str!("../scenarios/lvm-provisioning-lessons-v1.json"));texts}
pub fn accepted315()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted315_texts())}
fn accepted317_texts()->Vec<&'static str>{let mut texts=accepted315_texts();texts.push(include_str!("../scenarios/lvm-retirement-lessons-v1.json"));texts}
pub fn accepted317()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted317_texts())}
fn accepted319_texts()->Vec<&'static str>{let mut texts=accepted317_texts();texts.push(include_str!("../scenarios/lvm-activation-lessons-v1.json"));texts}
pub fn accepted319()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted319_texts())}
fn accepted321_texts()->Vec<&'static str>{let mut texts=accepted319_texts();texts.push(include_str!("../scenarios/lvm-rename-lessons-v1.json"));texts}
pub fn accepted321()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted321_texts())}
fn accepted323_texts()->Vec<&'static str>{let mut texts=accepted321_texts();texts.push(include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"));texts}
pub fn accepted323()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted323_texts())}
fn accepted326_texts()->Vec<&'static str>{let mut texts=accepted323_texts();texts.push(include_str!("../scenarios/mbr-layout-lessons-v1.json"));texts}
/// Exact verified326 catalog before the raw-record snapshot unit.
pub fn accepted326()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted326_texts())}
fn accepted329_texts()->Vec<&'static str>{let mut texts=accepted326_texts();texts.push(include_str!("../scenarios/lvm-cow-lessons-v1.json"));texts}
pub fn accepted329()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted329_texts())}
fn accepted331_texts()->Vec<&'static str>{let mut texts=accepted329_texts();texts.push(include_str!("../scenarios/lvm-reference-lessons-v1.json"));texts}
/// Exact preceding331/113 catalog; new configuration tasks inherit no credit.
pub fn accepted331()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted331_texts())}
fn accepted333_texts()->Vec<&'static str>{let mut texts=accepted331_texts();texts.push(include_str!("../scenarios/lvm-config-lessons-v1.json"));texts}
/// Preserved device-selection snapshot for exact prior-catalog comparisons.
pub fn accepted333()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted333_texts())}
fn accepted335_texts()->Vec<&'static str>{let mut texts=accepted333_texts();texts.push(include_str!("../scenarios/lvm-mount-lessons-v1.json"));texts}
/// Exact previous mount catalogue; new AutoFS requirements earn no inherited credit.
pub fn accepted335()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted335_texts())}
fn accepted337_texts()->Vec<&'static str>{let mut texts=accepted335_texts();texts.push(include_str!("../scenarios/autofs-lessons-v1.json"));texts}
/// Exact preceding AutoFS catalog; future assembly credit is never inherited.
pub fn accepted337()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted337_texts())}
fn accepted338_texts()->Vec<&'static str>{let mut texts=accepted337_texts();texts.push(include_str!("../scenarios/raid-assembly-lessons-v1.json"));texts}
/// Exact preceding assembly catalog; VLAN requirements earn no inherited credit.
pub fn accepted338()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted338_texts())}
fn accepted340_texts()->Vec<&'static str>{let mut texts=accepted338_texts();texts.push(include_str!("../scenarios/linux-vlan-lessons-v1.json"));texts}
/// Exact preceding VLAN catalog; bond requirements earn no inherited credit.
pub fn accepted340()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted340_texts())}
fn accepted341_texts()->Vec<&'static str>{let mut texts=accepted340_texts();texts.push(include_str!("../scenarios/bond-vlan-lessons-v1.json"));texts}
/// Exact preceding bond catalog; new bridge requirements earn no inherited credit.
pub fn accepted341()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted341_texts())}
// Keep this checkpoint's identical static pack order without another deep helper hop.
fn accepted342_texts()->Vec<&'static str>{let mut texts=accepted337_texts();texts.push(include_str!("../scenarios/raid-assembly-lessons-v1.json"));texts.push(include_str!("../scenarios/linux-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bond-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bridge-lan-lessons-v1.json"));texts}
pub fn accepted342()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted342_texts())}
fn accepted343_texts()->Vec<&'static str>{let mut texts=accepted337_texts();texts.push(include_str!("../scenarios/raid-assembly-lessons-v1.json"));texts.push(include_str!("../scenarios/linux-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bond-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bridge-lan-lessons-v1.json"));texts.push(include_str!("../scenarios/pam-auth-lessons-v1.json"));texts}
pub fn accepted343()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted343_texts())}
fn accepted346_texts()->Vec<&'static str>{let mut texts=accepted337_texts();texts.push(include_str!("../scenarios/raid-assembly-lessons-v1.json"));texts.push(include_str!("../scenarios/linux-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bond-vlan-lessons-v1.json"));texts.push(include_str!("../scenarios/bridge-lan-lessons-v1.json"));texts.push(include_str!("../scenarios/pam-auth-lessons-v1.json"));texts.push(include_str!("../scenarios/aging-lessons-v1.json"));texts}
pub fn accepted346()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted346_texts())}
fn accepted349_texts()->Vec<&'static str>{let mut texts=accepted346_texts();texts.push(include_str!("../scenarios/port-security-lessons-v1.json"));texts}
pub fn accepted349()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted349_texts())}
fn accepted351_texts()->Vec<&'static str>{let mut texts=accepted349_texts();texts.push(include_str!("../scenarios/dhcp-snooping-lessons-v1.json"));texts}
pub fn accepted351()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted351_texts())}
pub fn accepted392() -> Result<Vec<Scenario>, String> { load_bundled_catalog(&accepted392_texts()) }
pub fn accepted394() -> Result<Vec<Scenario>,String> { load_bundled_catalog(&accepted394_texts()) }
pub fn accepted396()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted396_texts())}
pub fn accepted399()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted399_texts())}
pub fn accepted402()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted402_texts())}
pub fn accepted404()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted404_texts())}
pub fn accepted406()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted406_texts())}
pub fn accepted409()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted409_texts())}
pub fn accepted412()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted412_texts())}
pub fn accepted415()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted415_texts())}
pub fn accepted417()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted417_texts())}
pub fn accepted419()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted419_texts())}
/// Exact421 scenario data retained before WLAN-profile curriculum integration.
pub fn accepted421()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted421_texts())}
pub fn accepted487()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted487_texts())}
pub fn ccna_pilot() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted543_texts())
}
pub fn accepted390() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted390_texts())
}
pub fn accepted387() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted387_texts())
}
pub fn accepted386() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted386_texts())
}
pub fn accepted385() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted385_texts())
}
pub fn accepted384() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted384_texts())
}
/// Exact catalog from the 383-lab removable EFI candidate.
pub fn accepted383() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&accepted383_texts())
}

// Exact354 catalog retained for preservation/compatibility checks; no later credit.
fn accepted354_texts()->Vec<&'static str>{let mut texts=accepted346_texts();texts.push(include_str!("../scenarios/port-security-lessons-v1.json"));texts.push(include_str!("../scenarios/dhcp-snooping-lessons-v1.json"));texts.push(include_str!("../scenarios/arp-inspection-lessons-v1.json"));texts}
pub fn accepted354()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted354_texts())}

// Exact356review catalog, retained before later local-account additions.
fn accepted356_texts()->Vec<&'static str>{let mut texts=accepted354_texts();texts.push(include_str!("../scenarios/management-policy-lessons-v1.json"));texts}
pub fn accepted356()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted356_texts())}

// Exact358review source retained before later ordered-authentication work.
fn accepted358_texts()->Vec<&'static str>{let mut texts=accepted356_texts();texts.push(include_str!("../scenarios/local-access-lessons-v1.json"));texts}
pub fn accepted358()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted358_texts())}

// Current catalog flattened at360 to retain the strict64-helper resolver bound.
// Ordered paths come from verified358 reporting plus the two AAA lessons.
// Historical accepted helpers and their exact data remain unchanged.
fn accepted360_texts()->Vec<&'static str>{vec![
include_str!("../scenarios/starter-v1.json"),
include_str!("../scenarios/shell-unit-v1.json"),
include_str!("../scenarios/shell-composition-v1.json"),
include_str!("../scenarios/shell-reusable-v1.json"),
include_str!("../scenarios/linux-files-v1.json"),
include_str!("../scenarios/ipv6-static-v1.json"),
include_str!("../scenarios/shell-scope-v1.json"),
include_str!("../scenarios/shell-text-v1.json"),
include_str!("../scenarios/shell-counts-v1.json"),
include_str!("../scenarios/linux-access-v1.json"),
include_str!("../scenarios/vlan-data-v1.json"),
include_str!("../scenarios/shell-file-batches-v1.json"),
include_str!("../scenarios/ipv4-static-v1.json"),
include_str!("../scenarios/dns-forward-v1.json"),
include_str!("../scenarios/shell-arrays-v1.json"),
include_str!("../scenarios/linux-processes-v1.json"),
include_str!("../scenarios/discovery-v1.json"),
include_str!("../scenarios/storage-v1.json"),
include_str!("../scenarios/shell-capture-v1.json"),
include_str!("../scenarios/acl-policy-v1.json"),
include_str!("../scenarios/accounts-lifecycle-v1.json"),
include_str!("../scenarios/ospf-p2p-v1.json"),
include_str!("../scenarios/unit-manager-v1.json"),
include_str!("../scenarios/shell-case-v1.json"),
include_str!("../scenarios/mac-learning-v1.json"),
include_str!("../scenarios/linux-find-v1.json"),
include_str!("../scenarios/dns-reverse-v1.json"),
include_str!("../scenarios/shell-getopts-v1.json"),
include_str!("../scenarios/dhcpv4-client-v1.json"),
include_str!("../scenarios/debian-packages-v1.json"),
include_str!("../scenarios/spanning-tree-v1.json"),
include_str!("../scenarios/shell-safe-automation-v1.json"),
include_str!("../scenarios/apache-static-v1.json"),
include_str!("../scenarios/nat-basic-v1.json"),
include_str!("../scenarios/rpm-packages-v1.json"),
include_str!("../scenarios/shell-command-vectors-v1.json"),
include_str!("../scenarios/nfs-v1.json"),
include_str!("../scenarios/etherchannel-v1.json"),
include_str!("../scenarios/shell-optional-inputs-v1.json"),
include_str!("../scenarios/linux-cron-v1.json"),
include_str!("../scenarios/file-backup-v1.json"),
include_str!("../scenarios/shell-numeric-validation-v1.json"),
include_str!("../scenarios/physical-links-v1.json"),
include_str!("../scenarios/ipv4-private-planning-v1.json"),
include_str!("../scenarios/linux-one-off-v1.json"),
include_str!("../scenarios/shell-loop-recovery-v1.json"),
include_str!("../scenarios/lvm-growth-v1.json"),
include_str!("../scenarios/ntp-client-server-v1.json"),
include_str!("../scenarios/shell-ere-evidence-v1.json"),
include_str!("../scenarios/disk-preparation-v1.json"),
include_str!("../scenarios/shell-literal-normalization-draft.json"),
include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
include_str!("../scenarios/ccna-magic-number-v1.json"),
include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
include_str!("../scenarios/links-lessons-v1.json"),
include_str!("../scenarios/syslog-policy-v1.json"),
include_str!("../scenarios/capacity-lessons-v1.json"),
include_str!("../scenarios/repository-lessons-v1.json"),
include_str!("../scenarios/linux-host-lessons-v1.json"),
include_str!("../scenarios/shell-debug-recovery-v1.json"),
include_str!("../scenarios/linux-logging-lessons-v1.json"),
include_str!("../scenarios/linux-startup-lessons-v1.json"),
include_str!("../scenarios/qos-lessons-v1.json"),
include_str!("../scenarios/file-backup-selection-v1.json"),
include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
include_str!("../scenarios/linux-nm-lessons-v1.json"),
include_str!("../scenarios/removable-mount-lessons-v1.json"),
include_str!("../scenarios/client-name-lessons-v1.json"),
include_str!("../scenarios/resolver-search-lessons-v1.json"),
include_str!("../scenarios/debian-contents-lessons-v1.json"),
include_str!("../scenarios/debian-sources-lessons-v1.json"),
include_str!("../scenarios/shell-tracing-lessons-v1.json"),
include_str!("../scenarios/environment-lessons-v1.json"),
include_str!("../scenarios/user-mount-lessons-v1.json"),
include_str!("../scenarios/debian-conflict-lessons-v1.json"),
include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
include_str!("../scenarios/fs-integrity-lessons-v1.json"),
include_str!("../scenarios/shell-startup-lessons-v1.json"),
include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
include_str!("../scenarios/debian-conffile-lessons-v1.json"),
include_str!("../scenarios/raid1-lessons-v1.json"),
include_str!("../scenarios/shell-associative-lessons-v1.json"),
include_str!("../scenarios/shared-library-lessons-v1.json"),
include_str!("../scenarios/fhs-lessons-v1.json"),
include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
include_str!("../scenarios/listener-audit-lessons-v1.json"),
include_str!("../scenarios/path-lookup-lessons-v1.json"),
include_str!("../scenarios/account-skeleton-lessons-v1.json"),
include_str!("../scenarios/command-location-lessons-v1.json"),
include_str!("../scenarios/file-index-lessons-v1.json"),
include_str!("../scenarios/whereis-lessons-v1.json"),
include_str!("../scenarios/sticky-directory-lessons-v1.json"),
include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
include_str!("../scenarios/transport-lessons-v1.json"),
include_str!("../scenarios/recursive-tree-lessons-v1.json"),
include_str!("../scenarios/find-size-lessons-v1.json"),
include_str!("../scenarios/kernel-module-lessons-v1.json"),
include_str!("../scenarios/kernel-module-names-lessons-v1.json"),
include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
include_str!("../scenarios/host-ipv6-lessons-v1.json"),
include_str!("../scenarios/swap-policy-lessons-v1.json"),
include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
include_str!("../scenarios/lvm-activation-lessons-v1.json"),
include_str!("../scenarios/lvm-rename-lessons-v1.json"),
include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
include_str!("../scenarios/mbr-layout-lessons-v1.json"),
include_str!("../scenarios/lvm-cow-lessons-v1.json"),
include_str!("../scenarios/lvm-reference-lessons-v1.json"),
include_str!("../scenarios/lvm-config-lessons-v1.json"),
include_str!("../scenarios/lvm-mount-lessons-v1.json"),
include_str!("../scenarios/autofs-lessons-v1.json"),
include_str!("../scenarios/raid-assembly-lessons-v1.json"),
include_str!("../scenarios/linux-vlan-lessons-v1.json"),
include_str!("../scenarios/bond-vlan-lessons-v1.json"),
include_str!("../scenarios/bridge-lan-lessons-v1.json"),
include_str!("../scenarios/pam-auth-lessons-v1.json"),
include_str!("../scenarios/aging-lessons-v1.json"),
include_str!("../scenarios/port-security-lessons-v1.json"),
include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
include_str!("../scenarios/arp-inspection-lessons-v1.json"),
include_str!("../scenarios/management-policy-lessons-v1.json"),
include_str!("../scenarios/local-access-lessons-v1.json"),
include_str!("../scenarios/aaa-access-lessons-v1.json"),
]}

/// Exact360source retained before directory-client curriculum.
pub fn accepted360()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted360_texts())}

fn accepted362_texts()->Vec<&'static str>{let mut texts=accepted360_texts();texts.push(include_str!("../scenarios/ldap-client-lessons-v1.json"));texts}

pub fn accepted362()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted362_texts())}
fn accepted364_texts()->Vec<&'static str>{let mut texts=accepted362_texts();texts.push(include_str!("../scenarios/ldap-change-lessons-v1.json"));texts}

/// Exact364source retained before virtual utility curriculum.
pub fn accepted364()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted364_texts())}
fn accepted366_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before367-v1.json"));texts}
pub fn accepted366()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted366_texts())}
fn accepted367_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before368-v1.json"));texts}
pub fn accepted367()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted367_texts())}
fn accepted368_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before369-v1.json"));texts}
pub fn accepted368()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted368_texts())}
fn accepted369_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before370-v1.json"));texts}
pub fn accepted369()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted369_texts())}
fn accepted370_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before371-v1.json"));texts}
pub fn accepted370()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted370_texts())}
fn accepted371_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/history/ldap-utility-before372-v1.json"));texts}
pub fn accepted371()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted371_texts())}
fn accepted372_texts()->Vec<&'static str>{let mut texts=accepted364_texts();texts.push(include_str!("../scenarios/ldap-utility-lessons-v1.json"));texts}

pub fn accepted372()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted372_texts())}
fn accepted373_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-names-lessons-v1.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
]}

pub fn accepted373()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted373_texts())}
fn accepted376_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v1.json"),
]}

/// Exact376scenario registry retained before literal GRUB menu work.
pub fn accepted376()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted376_texts())}
fn accepted378_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v2.json"),
]}

/// The complete current application catalog. `bundled` remains the historical
/// default validation fixture; it must not select the learner application's scope.
pub fn current() -> Result<Vec<Scenario>, String> { ccna_pilot() }

/// Exact378 registry retained before persistent GRUB settings.
pub fn accepted378()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted378_texts())}
fn accepted380_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v3.json"),
]}

/// Exact admitted380 catalog; new requirements earn no inherited credit.
pub fn accepted380()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted380_texts())}
fn accepted381_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v4.json"),
]}

/// Exact verified 381-lab catalog.
pub fn accepted381()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted381_texts())}
fn accepted382_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v5.json"),
]}

/// Exact verified 382-lab catalog.
pub fn accepted382()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted382_texts())}
fn accepted383_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v6.json"),
]}

fn accepted384_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v7.json"),
]}

fn accepted385_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v8.json"),
]}

fn accepted386_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v9.json"),
]}

fn accepted387_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v10.json"),
]}

fn accepted390_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v11.json"),
]}

fn accepted392_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v12.json"),
]}

fn accepted394_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v13.json"),
]}

fn accepted396_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v14.json"),
]}

fn accepted399_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v15.json"),
]}

fn accepted402_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v16.json"),
]}

fn accepted404_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v17.json"),
]}


fn accepted406_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v18.json"),
]}

fn accepted409_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v19.json"),
]}

fn accepted412_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v20.json"),
]}

fn accepted415_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v21.json"),
]}

fn accepted417_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v22.json"),
]}

fn accepted419_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v23.json"),
]}

fn accepted421_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v24.json"),
]}

fn accepted424_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v25.json"),
]}

pub fn accepted424()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted424_texts())}

fn accepted426_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v26.json"),
]}
fn accepted428_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v27.json"),
]}
/// Current428 data; prior426 remains an exact historical snapshot.
pub fn accepted428()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted428_texts())}


pub fn accepted426()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted426_texts())}

fn accepted430_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v28.json"),
]}

pub fn accepted430()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted430_texts())}
fn accepted432_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v29.json"),
]}

pub fn accepted432()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted432_texts())}
fn accepted434_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v30.json"),
]}

/// Exact verified434 sources before service-name integration.
pub fn accepted434()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted434_texts())}
fn accepted436_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v31.json"),
]}

/// Exact verified436 scenario values before the mixed-IP teaching batch.
pub fn accepted436()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted436_texts())}
fn accepted437_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v32.json"),
]}

/// Retained437catalog before optional protocol/Echo curriculum.
pub fn accepted437()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted437_texts())}
fn accepted438_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v33.json"),
]}

/// Exact438catalog retained before opt-in host Echo diagnosis.
pub fn accepted438()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted438_texts())}
fn accepted439_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v34.json"),
]}

/// Exact439catalog retained before bounded REST curriculum integration.
pub fn accepted439()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted439_texts())}
fn accepted441_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v1.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v1.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v35.json"),
]}

/// Exact441catalog retained before combined client-name diagnostics.
pub fn accepted441()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted441_texts())}
fn accepted442_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v36.json"),
]}

/// Exact442 catalog retained before SSH administration integration.
pub fn accepted442()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted442_texts())}
fn accepted445_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v37.json"),
]}

/// Exact445 scenario data retained before modal-file editing integration.
pub fn accepted445()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted445_texts())}
fn accepted448_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v38.json"),
]}

/// Exact448 scenarios retained before inherited-editor/file handover integration.
pub fn accepted448()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted448_texts())}
fn accepted449_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v39.json"),
]}

/// Exact449 authored scenario values retained before file-transfer integration.
pub fn accepted449()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted449_texts())}
/// Exact452 scenario values retained before the campus architecture unit.
pub fn accepted452()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted452_texts())}
fn accepted452_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v40.json"),
]}

fn accepted454_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v41.json"),
]}

/// Frozen verified454 source catalog; never aliases the later selected catalog.
pub fn accepted454()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted454_texts())}
fn accepted455_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v42.json"),
]}

/// Exact verified455 catalog retained before standalone-share integration.
pub fn accepted455()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted455_texts())}
fn accepted458_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v43.json"),
]}

/// Exact verified458 data retained before virtual-isolation integration.
pub fn accepted458()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted458_texts())}
fn accepted461_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v44.json"),
]}

/// Exact verified461 scenario values retained before guest-image integration.
pub fn accepted461()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted461_texts())}
fn accepted464_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v45.json"),
]}

/// Retained exact464scenario registry before SMTP routing integration.
pub fn accepted464()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted464_texts())}
fn accepted467_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v46.json"),
]}

pub fn accepted467()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted467_texts())}
#[path = "pack_retained.rs"]
mod retained_catalogs;
pub use retained_catalogs::{accepted470, accepted472};
#[path="pack_retained_mail.rs"]
mod retained_mail_catalog;
fn accepted475_texts()->Vec<&'static str>{retained_mail_catalog::texts()}

pub fn accepted475()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted475_texts())}
fn accepted477_texts()->Vec<&'static str>{retained_mail_catalog::texts477()}

pub fn accepted477()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted477_texts())}
fn accepted478_texts()->Vec<&'static str>{retained_mail_catalog::texts478()}

pub fn accepted478()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted478_texts())}
fn accepted481_texts()->Vec<&'static str>{retained_mail_catalog::texts481()}

fn accepted482_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-client-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v2.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v53.json"),
]}
pub fn accepted481()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted481_texts())}

fn accepted485_texts()->Vec<&'static str>{retained_mail_catalog::texts485()}
pub fn accepted482()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted482_texts())}

fn accepted487_texts()->Vec<&'static str>{retained_mail_catalog::texts487()}
pub fn accepted485()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted485_texts())}

fn accepted489_texts()->Vec<&'static str>{retained_mail_catalog::texts489()}
pub fn accepted489()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted489_texts())}

mod retained_494_catalog;
fn accepted494_texts()->Vec<&'static str>{retained_494_catalog::texts()}

fn accepted490_texts()->Vec<&'static str>{let mut texts=accepted493_texts();texts.pop();texts.push(include_str!("../scenarios/boot-entry-lessons-v57.json"));texts}
pub fn accepted490()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted490_texts())}

fn accepted493_texts()->Vec<&'static str>{let mut texts=accepted494_texts();texts.pop();texts.push(include_str!("../scenarios/boot-entry-lessons-v58.json"));texts}
pub fn accepted493()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted493_texts())}

/// Exact494 catalog retained before FTP identity curriculum.
pub fn accepted494()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted494_texts())}
mod retained_497_catalog;
fn accepted497_texts()->Vec<&'static str>{retained_497_catalog::texts()}
pub fn accepted497()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted497_texts())}
mod retained_499_catalog;
fn accepted499_texts()->Vec<&'static str>{retained_499_catalog::texts()}
pub fn accepted499()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted499_texts())}
mod retained_501_catalog;
fn accepted501_texts()->Vec<&'static str>{retained_501_catalog::texts()}
mod retained_503_catalog;
fn accepted503_texts()->Vec<&'static str>{retained_503_catalog::texts()}

/// Exact catalog retained before advisory-patching curriculum.
pub fn accepted501()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted501_texts())}

/// Exact503 data retained before offline log-filter curriculum.
pub fn accepted503()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted503_texts())}

mod retained_505_catalog;
fn accepted505_texts()->Vec<&'static str>{retained_505_catalog::texts()}
pub fn accepted505()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted505_texts())}

mod retained_507_catalog;
fn accepted507_texts()->Vec<&'static str>{retained_507_catalog::texts()}

/// Exact507data retained before PoE; compiled with the current engine.
pub fn accepted507()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted507_texts())}

mod retained_509_catalog;
fn accepted509_texts()->Vec<&'static str>{retained_509_catalog::texts()}
pub fn accepted509()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted509_texts())}

mod retained_511_catalog;
fn accepted511_texts()->Vec<&'static str>{retained_511_catalog::texts()}
pub fn accepted511()->Result<Vec<Scenario>,String>{load_bundled_catalog(&accepted511_texts())}

mod retained_513_catalog;
pub fn accepted513()->Result<Vec<Scenario>,String>{load_bundled_catalog(&retained_513_catalog::texts())}

mod retained_515_catalog;
pub fn accepted515()->Result<Vec<Scenario>,String>{load_bundled_catalog(&retained_515_catalog::texts())}

mod retained_517_catalog;

/// Exact catalog retained from the network boot milestone.
pub fn accepted517() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_517_catalog::texts())
}

mod retained_519_catalog;

/// Exact catalog retained from the iSCSI milestone.
pub fn accepted519() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_519_catalog::texts())
}

mod retained_521_catalog;

pub fn accepted521() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_521_catalog::texts())
}

mod retained_523_catalog;

pub fn accepted523() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_523_catalog::texts())
}

mod retained_526_catalog;

pub fn accepted526() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_526_catalog::texts())
}

mod retained_528_catalog;

pub fn accepted528() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_528_catalog::texts())
}

mod retained_530_catalog;

pub fn accepted530() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_530_catalog::texts())
}

mod retained_532_catalog;

pub fn accepted532() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_532_catalog::texts())
}

mod retained_534_catalog;

pub fn accepted534() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_534_catalog::texts())
}

mod retained_536_catalog;

pub fn accepted536() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_536_catalog::texts())
}

mod retained_538_catalog;

pub fn accepted538() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_538_catalog::texts())
}

mod retained_540_catalog;

pub fn accepted540() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_540_catalog::texts())
}

mod retained_541_catalog;

pub fn accepted541() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_541_catalog::texts())
}

mod retained_542_catalog;

pub fn accepted542() -> Result<Vec<Scenario>, String> {
    load_bundled_catalog(&retained_542_catalog::texts())
}

fn accepted543_texts()->Vec<&'static str>{vec![
    include_str!("../scenarios/starter-v1.json"),
    include_str!("../scenarios/shell-unit-v1.json"),
    include_str!("../scenarios/shell-composition-v1.json"),
    include_str!("../scenarios/shell-reusable-v1.json"),
    include_str!("../scenarios/linux-files-v1.json"),
    include_str!("../scenarios/ipv6-static-v1.json"),
    include_str!("../scenarios/shell-scope-v1.json"),
    include_str!("../scenarios/shell-text-v1.json"),
    include_str!("../scenarios/shell-counts-v1.json"),
    include_str!("../scenarios/linux-access-v1.json"),
    include_str!("../scenarios/vlan-data-v1.json"),
    include_str!("../scenarios/shell-file-batches-v1.json"),
    include_str!("../scenarios/ipv4-static-v1.json"),
    include_str!("../scenarios/dns-forward-v2.json"),
    include_str!("../scenarios/shell-arrays-v1.json"),
    include_str!("../scenarios/linux-processes-v1.json"),
    include_str!("../scenarios/discovery-v1.json"),
    include_str!("../scenarios/storage-v1.json"),
    include_str!("../scenarios/shell-capture-v1.json"),
    include_str!("../scenarios/acl-policy-v1.json"),
    include_str!("../scenarios/accounts-lifecycle-v1.json"),
    include_str!("../scenarios/ospf-p2p-v1.json"),
    include_str!("../scenarios/unit-manager-v1.json"),
    include_str!("../scenarios/shell-case-v1.json"),
    include_str!("../scenarios/mac-learning-v1.json"),
    include_str!("../scenarios/linux-find-v1.json"),
    include_str!("../scenarios/dns-reverse-v1.json"),
    include_str!("../scenarios/shell-getopts-v1.json"),
    include_str!("../scenarios/dhcpv4-client-v1.json"),
    include_str!("../scenarios/debian-packages-v1.json"),
    include_str!("../scenarios/spanning-tree-v1.json"),
    include_str!("../scenarios/shell-safe-automation-v1.json"),
    include_str!("../scenarios/apache-static-v1.json"),
    include_str!("../scenarios/nat-basic-v1.json"),
    include_str!("../scenarios/rpm-packages-v1.json"),
    include_str!("../scenarios/shell-command-vectors-v1.json"),
    include_str!("../scenarios/nfs-v1.json"),
    include_str!("../scenarios/etherchannel-v1.json"),
    include_str!("../scenarios/shell-optional-inputs-v1.json"),
    include_str!("../scenarios/linux-cron-v1.json"),
    include_str!("../scenarios/file-backup-v1.json"),
    include_str!("../scenarios/shell-numeric-validation-v1.json"),
    include_str!("../scenarios/physical-links-v1.json"),
    include_str!("../scenarios/ipv4-private-planning-v1.json"),
    include_str!("../scenarios/linux-one-off-v1.json"),
    include_str!("../scenarios/shell-loop-recovery-v1.json"),
    include_str!("../scenarios/lvm-growth-v1.json"),
    include_str!("../scenarios/ntp-client-server-v1.json"),
    include_str!("../scenarios/shell-ere-evidence-v1.json"),
    include_str!("../scenarios/disk-preparation-v1.json"),
    include_str!("../scenarios/shell-literal-normalization-draft.json"),
    include_str!("../scenarios/shell-snapshot-rollout-draft.json"),
    include_str!("../scenarios/ccna-magic-number-v1.json"),
    include_str!("../scenarios/hsrp-resilient-gateway-v1.json"),
    include_str!("../scenarios/links-lessons-v1.json"),
    include_str!("../scenarios/syslog-policy-v1.json"),
    include_str!("../scenarios/capacity-lessons-v1.json"),
    include_str!("../scenarios/repository-lessons-v1.json"),
    include_str!("../scenarios/linux-host-lessons-v2.json"),
    include_str!("../scenarios/shell-debug-recovery-v1.json"),
    include_str!("../scenarios/linux-logging-lessons-v1.json"),
    include_str!("../scenarios/linux-startup-lessons-v1.json"),
    include_str!("../scenarios/qos-lessons-v1.json"),
    include_str!("../scenarios/file-backup-selection-v1.json"),
    include_str!("../scenarios/linux-dhcp-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-lessons-v1.json"),
    include_str!("../scenarios/removable-mount-lessons-v1.json"),
    include_str!("../scenarios/client-name-lessons-v1.json"),
    include_str!("../scenarios/resolver-search-lessons-v1.json"),
    include_str!("../scenarios/debian-contents-lessons-v1.json"),
    include_str!("../scenarios/debian-sources-lessons-v1.json"),
    include_str!("../scenarios/shell-tracing-lessons-v1.json"),
    include_str!("../scenarios/environment-lessons-v1.json"),
    include_str!("../scenarios/user-mount-lessons-v1.json"),
    include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json"),
    include_str!("../scenarios/fs-integrity-lessons-v1.json"),
    include_str!("../scenarios/shell-startup-lessons-v1.json"),
    include_str!("../scenarios/shell-file-lifecycle-lessons-v1.json"),
    include_str!("../scenarios/debian-conffile-lessons-v1.json"),
    include_str!("../scenarios/raid1-lessons-v1.json"),
    include_str!("../scenarios/shell-associative-lessons-v1.json"),
    include_str!("../scenarios/shared-library-lessons-v1.json"),
    include_str!("../scenarios/fhs-lessons-v1.json"),
    include_str!("../scenarios/shell-managed-config-lessons-v1.json"),
    include_str!("../scenarios/listener-audit-lessons-v1.json"),
    include_str!("../scenarios/path-lookup-lessons-v1.json"),
    include_str!("../scenarios/account-skeleton-lessons-v1.json"),
    include_str!("../scenarios/command-location-lessons-v1.json"),
    include_str!("../scenarios/file-index-lessons-v1.json"),
    include_str!("../scenarios/whereis-lessons-v1.json"),
    include_str!("../scenarios/sticky-directory-lessons-v1.json"),
    include_str!("../scenarios/shell-content-backup-lessons-v1.json"),
    include_str!("../scenarios/transport-lessons-v1.json"),
    include_str!("../scenarios/recursive-tree-lessons-v1.json"),
    include_str!("../scenarios/find-size-lessons-v1.json"),
    include_str!("../scenarios/kernel-module-curriculum-v2.json"),
    include_str!("../scenarios/shell-log-triage-lessons-v1.json"),
    include_str!("../scenarios/rpm-metadata-lessons-v1.json"),
    include_str!("../scenarios/host-ipv6-lessons-v1.json"),
    include_str!("../scenarios/swap-policy-lessons-v1.json"),
    include_str!("../scenarios/lvm-provisioning-lessons-v1.json"),
    include_str!("../scenarios/lvm-retirement-lessons-v1.json"),
    include_str!("../scenarios/lvm-activation-lessons-v1.json"),
    include_str!("../scenarios/lvm-rename-lessons-v1.json"),
    include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"),
    include_str!("../scenarios/mbr-layout-lessons-v1.json"),
    include_str!("../scenarios/lvm-cow-lessons-v1.json"),
    include_str!("../scenarios/lvm-reference-lessons-v1.json"),
    include_str!("../scenarios/lvm-config-lessons-v1.json"),
    include_str!("../scenarios/lvm-mount-lessons-v1.json"),
    include_str!("../scenarios/autofs-lessons-v1.json"),
    include_str!("../scenarios/raid-assembly-lessons-v1.json"),
    include_str!("../scenarios/linux-vlan-lessons-v1.json"),
    include_str!("../scenarios/bond-vlan-lessons-v1.json"),
    include_str!("../scenarios/bridge-lan-lessons-v1.json"),
    include_str!("../scenarios/pam-auth-lessons-v1.json"),
    include_str!("../scenarios/aging-lessons-v1.json"),
    include_str!("../scenarios/port-security-lessons-v1.json"),
    include_str!("../scenarios/dhcp-snooping-lessons-v1.json"),
    include_str!("../scenarios/arp-inspection-lessons-v1.json"),
    include_str!("../scenarios/management-policy-lessons-v1.json"),
    include_str!("../scenarios/local-access-lessons-v1.json"),
    include_str!("../scenarios/aaa-access-lessons-v1.json"),
    include_str!("../scenarios/ldap-administration-v3.json"),
    include_str!("../scenarios/ipv6-interface-id-v1.json"),
    include_str!("../scenarios/boot-entry-lessons-v63.json"),
    include_str!("../scenarios/curriculum-continuation-v19.json"),
]}
