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
    let c = |b, t: &str| (b, t.to_owned());
    match (s, g) {
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
        State::Ntp { .. } => crate::ntp::HELP,
        State::Lvm { .. } => crate::lvm::HELP,
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
    if matches!(s, State::Ntp { .. }) {
        return match input {
            "help" => Ok(crate::ntp::HELP.into()),
            "show" => Ok(describe(s)),
            _ => Err("NTP changes require their authored Goal".into()),
        };
    }
    if matches!(s, State::Lvm { .. }) {
        return match input {
            "help" => Ok(crate::lvm::HELP.into()),
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
        State::Ntp { .. }
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
