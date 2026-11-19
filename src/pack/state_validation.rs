//! Validate modeled initial state independently of literal catalog selection.
use crate::engine::{self, State};
use std::collections::BTreeSet;

pub fn validate_state(s: &State) -> Result<(), String> {
    let valid = match s {
        State::StartupFiles {simulation} => simulation.validate_unbound().is_ok(),
        State::Raid1 {simulation} => simulation.validate().is_ok(),
        State::FileIndex {simulation} => crate::file_index_source::shape(simulation),
        State::SharedLibraries {simulation} => simulation.validate_unbound().is_ok(),
        State::FsIntegrity {simulation} => simulation.validate(&simulation.definition).is_ok(),
        State::UserMounts {simulation} => simulation.validate(&simulation.definition).is_ok(),
        State::Environment {simulation} => simulation.validate(&simulation.definition).is_ok(),
        State::DebianSources {simulation} => simulation.validate().is_ok(),
        State::ClientNames { simulation } => simulation.validate().is_ok(),
        State::RemovableMounts { simulation } => simulation.validate().is_ok(),
        State::NmProfiles { simulation } => simulation.validate().is_ok(),
        State::PamStack { simulation } => simulation.validate().is_ok(),
        State::Aging { simulation } => simulation.validate().is_ok(),
        State::PortSecurity { simulation } => simulation.validate().is_ok(),
        State::DhcpSnooping { simulation } => simulation.validate().is_ok(),
        State::ArpInspection { simulation } => simulation.validate().is_ok(),
        State::ManagementPolicy { simulation } => simulation.validate().is_ok(),
        State::LocalAccess { simulation } => simulation.validate().is_ok(),
        State::AaaAccess { simulation } => simulation.validate().is_ok(),
        State::LdapClient { simulation } => simulation.validate().is_ok(),
        State::LdapAccess { simulation } => simulation.validate().is_ok(),
        State::LdapSchema { simulation } => simulation.validate().is_ok(),
        State::CopiedJournal { simulation } => simulation.validate().is_ok(),
        State::JsonInventory { simulation } => simulation.validate().is_ok(),
        State::RestController { simulation } => simulation.validate().is_ok(),
        State::DevicePath { simulation } => simulation.validate().is_ok(),
        State::LocalMail { simulation } => simulation.validate().is_ok(),
        State::Chrony { simulation } => simulation.validate().is_ok(),
        State::GrubOnce { simulation } => simulation.validate().is_ok(),
        State::EfiLoader { simulation } => simulation.validate().is_ok(),
        State::BiosLoader { simulation } => simulation.validate().is_ok(),
        State::BootEntry { simulation } => simulation.validate().is_ok(),
        State::LdapSchemaQuery { simulation } => simulation.validate().is_ok(),
        State::LdapPassword { simulation } => simulation.validate().is_ok(),
        State::LdapChanges { simulation } => simulation.validate().is_ok(),
        State::DhcpConfig { simulation } => simulation.validate().is_ok(),
        State::Qos { simulation } => simulation.validate().is_ok(),
        State::LinuxLogging { simulation } => simulation.validate().is_ok(),
        State::VirtualIsolation { simulation } => simulation.validate().is_ok(),
        State::SieveFiltering { simulation } => simulation.validate().is_ok(),
        State::LinuxRouter { simulation } => simulation.shape().is_ok(),
        State::LinuxV6 { simulation } => simulation.shape().is_ok(),
        State::LinuxServiceRouter { simulation } => simulation.shape().is_ok(),
        State::LinuxV6Administration { simulation } => simulation.shape().is_ok(),
        State::LinuxV6Startup { simulation } => simulation.shape().is_ok(),
        State::IdsRule { simulation } => simulation.shape().is_ok(),
        State::AuthFilter { simulation } => simulation.shape().is_ok(),
        State::VpnPoint { simulation } => simulation.shape().is_ok(),
        State::Poe { simulation } => simulation.validate().is_ok(),
        State::KernelBuild { simulation } => simulation.validate_shape().is_ok(),
        State::Dkms { simulation } => simulation.validate_shape().is_ok(),
        State::Initramfs { simulation } => simulation.validate_shape().is_ok(),
        State::Pxe { simulation } => simulation.validate_shape().is_ok(),
        State::Iscsi { simulation } => simulation.validate_shape().is_ok(),
        State::AtaSmart { simulation } => simulation.validate_shape().is_ok(),
        State::ApacheHttps { simulation } => simulation.validate_shape().is_ok(),
        State::Desktop { simulation } => simulation.validate_shape().is_ok(),
        State::SecurityInvestigation { simulation } => simulation.validate_shape().is_ok(),
        State::AutomationWorkshop { simulation } => simulation.validate_shape().is_ok(),
        State::ServiceExposure { simulation } => simulation.shape().is_ok(),
        State::FtpIdentity { simulation } => simulation.shape().is_ok(),
        State::LinuxFlow { simulation } => simulation.shape().is_ok(),
        State::MailboxAccess { simulation } | State::MailboxAdministration { simulation } => simulation.validate().is_ok(),
        State::SmtpRouting { simulation } => simulation.validate().is_ok(),
        State::GuestDrivers { simulation } => simulation.validate().is_ok(),
        State::SambaShare { simulation } => simulation.validate().is_ok(),
        State::CampusArchitecture { simulation } => simulation.validate().is_ok(),
        State::NetworkTransfer { simulation } => simulation.validate().is_ok(),
        State::EditorHandover { simulation } => simulation.validate().is_ok(),
        State::ModalEditing { simulation } => simulation.validate().is_ok(),
        State::SshAccess { simulation } => simulation.validate().is_ok(),
        State::HostNetwork { simulation } => simulation.validate().is_ok(),
        State::Repositories { simulation } => simulation.validate().is_ok(),
        State::MaintenanceNotice { simulation } => simulation.shape().is_ok(),
        State::DnsAccess { simulation } => simulation.shape().is_ok(),
        State::PrintQueues { simulation } => simulation.shape().is_ok(),
        State::Localization { simulation } => simulation.shape().is_ok(),
        State::WirelessChannels { simulation } => simulation.shape().is_ok(),
        State::WlanProfiles { simulation } => simulation.shape().is_ok(),
        State::HttpCache { simulation } => simulation.shape().is_ok(),
        State::ServiceClients { simulation } => simulation.shape().is_ok(),
        State::ReverseProxy { simulation } => simulation.shape().is_ok(),
        State::ProxyPolicy { simulation } => simulation.shape().is_ok(),
        State::SnmpMonitor { simulation } => simulation.shape().is_ok(),
        State::WlanPaths { simulation } => simulation.shape().is_ok(),
        State::Timezone { simulation } => simulation.shape().is_ok(),
        State::SourceInstall { simulation } => simulation.shape().is_ok(),
        State::CapacityPlanning { simulation } => simulation.validate().is_ok(),
        State::ResourceObservations { simulation } => simulation.validate().is_ok(),
        State::Capacity { simulation } => simulation.validate().is_ok(),
        State::RaidAssembly { simulation } => simulation.shape(),
        State::Autofs { simulation } => simulation.shape(),
        State::KernelModules { simulation } => simulation.shape(),
        State::RecursiveTree { simulation } => simulation.shape(),
        State::Transport { simulation } => simulation.valid_shape(),
        State::DirectoryPolicy { simulation } => simulation.validate().is_ok(),
        State::Links { simulation } => simulation.validate().is_ok(),
        State::MagicSubnet { session } => session.validate().is_ok(),
        State::PartitionFs { simulation } => simulation.validate().is_ok(),
        State::Hsrp { simulation } => simulation.validate().is_ok(),
        State::Syslog { simulation } => simulation.validate().is_ok(),
        State::Ntp { simulation } => simulation.validate().is_ok(),
        State::Lvm { simulation } => simulation.validate().is_ok(),
        State::PhysicalLink { simulation } => simulation.validate().is_ok(),
        State::FileBackup { simulation } => simulation.valid(),
        State::EtherChannel { simulation } => simulation.validate().is_ok(),
        State::OneOff { simulation } => simulation.valid(),
        State::Cron { simulation } => simulation.validate().is_ok(),
        State::Nfs { simulation } => simulation.validate().is_ok(),
        State::RpmPackages { simulation } => crate::rpm_packages_workspace::admitted(simulation),
        State::BasicNat { simulation } => crate::nat_workspace::admitted(simulation),
        State::ApacheStatic { simulation } => crate::apache_static_workspace::admitted(simulation),
        State::DebianPackages { simulation } => {
            crate::debian_packages_workspace::admitted(simulation)
        }
        State::SpanningTree { simulation } => simulation.validate().is_ok(),
        State::DhcpV4 { simulation } => simulation.validate().is_ok(),
        State::UnitManager { simulation } => simulation.validate().is_ok(),
        State::Ospf { simulation } => simulation.valid(),
        State::Ipv4Acl { network } => crate::acl_workspace::valid_network(network).is_ok(),
        State::Accounts { simulation } => simulation.valid(),
        State::PersistentMounts { storage } => storage.validate().is_ok(),
        State::Discovery { simulation } => simulation.validate().is_ok(),
        State::Processes { table } => table.valid(),
        State::AuthoritativeDns { server } => server.validate().is_ok(),
        State::MacLearning { simulation } => simulation.validate().is_ok(),
        State::Switching { fabric } => fabric.validate().is_ok(),
        State::Ipv6 { topology } => topology.valid(),
        State::Ipv4Routing { topology } => topology.valid(),
        State::Filesystem { tree } => tree.valid(),
        State::Script { source, run } => {
            source.len() <= crate::shell::MAX_SOURCE && run.as_ref().is_none_or(|r| r.valid())
        }
        State::Subnet {
            prefix,
            answers,
            second,
            ..
        } => {
            *prefix <= 30
                && answers.iter().all(|a| a.len() <= 64)
                && second.as_ref().is_none_or(|a| a.len() <= 64)
        }
        State::Vlsm {
            base,
            prefix,
            allocations,
        } => {
            *prefix <= 30
                && engine::bounds(*base, *prefix).is_some_and(|(n, _)| n == u32::from(*base))
                && !allocations.is_empty()
                && allocations.len() <= 20
                && {
                    let mut names = BTreeSet::new();
                    allocations.iter().all(|a| {
                        a.hosts > 0
                            && a.hosts <= 1_000_000
                            && !a.name.is_empty()
                            && a.name.len() <= 80
                            && a.name
                                .bytes()
                                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                            && names.insert(&a.name)
                            && a.cidr.len() <= 64
                    })
                }
        }
        State::Network {
            prefix, lan_prefix, ..
        } => *prefix <= 30 && *lan_prefix <= 30,
        State::Unix { nodes, users } => {
            !users.is_empty()
                && nodes.get("/").is_some_and(|n| n.directory)
                && nodes.iter().all(|(p, n)| {
                    p.starts_with('/')
                        && !p.contains("..")
                        && n.mode <= 0o777
                        && users.contains_key(&n.owner)
                        && users.values().any(|g| g.contains(&n.group))
                        && (p == "/" || {
                            let parent = p
                                .rsplit_once('/')
                                .map(|(a, _)| if a.is_empty() { "/" } else { a })
                                .unwrap_or("/");
                            nodes.get(parent).is_some_and(|n| n.directory)
                        })
                })
        }
        State::Logs { files, result } => {
            !files.is_empty()
                && result.len() < 100_000
                && files.iter().all(|(p, v)| {
                    p.starts_with("/var/log/") && !p.contains("..") && v.len() < 100_000
                })
        }
        State::Service {
            port,
            running,
            loaded_port,
            failed,
            journal,
            ..
        } => {
            port.len() <= 256
                && (!*running || (loaded_port.is_some_and(|p| p > 0) && !*failed))
                && journal.len() <= 60
                && journal
                    .iter()
                    .all(|e| !matches!(e,engine::ServiceEvent::Rejected{port}if port.len()>256))
        }

        State::Dns {
            records, loaded, ..
        } => {
            !records.is_empty()
                && records.len() < 100
                && records.keys().all(|k| {
                    k.ends_with(".lab.test")
                        && k.bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
                })
                && loaded.keys().all(|k| records.contains_key(k))
        }
        State::Mount {
            capacity,
            used,
            cache,
            device,
            configured_device,
            ..
        } => {
            *capacity > 0
                && used <= capacity
                && cache <= used
                && device.starts_with("/dev/")
                && !configured_device.is_empty()
                && configured_device.len() <= 256
        }
    };
    if valid {
        Ok(())
    } else {
        Err("Invalid modeled initial state".into())
    }
}
