//! Optional saved workspace drafts and state-based diagnostics. No host command execution.
use crate::{
    engine::{self, State},
    ipv4,
    progress::Attempt,
};
use serde::{Deserialize, Serialize};
const MAX_SCRIPT_HISTORY: usize = 32;
fn script_histories<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<crate::shell::Record>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, crate::shell::Record, MAX_SCRIPT_HISTORY>(d)
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorkspaceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcpv4_draft: Option<crate::dhcpv4_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcpv4_learning: Option<crate::dhcpv4_teaching::Learning>,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_manager_draft: Option<crate::unit_manager_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_manager_learning: Option<crate::unit_manager_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_manager_terminal: Option<crate::unit_manager_terminal::Session>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ospf_draft: Option<crate::ospf_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ospf_learning: Option<crate::ospf_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_draft: Option<crate::acl_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_learning: Option<crate::acl_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_learning: Option<crate::script_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_history: Option<crate::shell::Record>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "script_histories"
    )]
    pub script_history_more: Vec<crate::shell::Record>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_learning: Option<crate::terminal_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_learning: Option<crate::process_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_learning: Option<crate::accounts_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_draft: Option<crate::discovery_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_learning: Option<crate::discovery_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_draft: Option<crate::storage_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_learning: Option<crate::storage_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_draft: Option<crate::ipv6_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_learning: Option<crate::ipv6_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_v4_draft: Option<crate::ipv4_routing_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_v4_learning: Option<crate::ipv4_routing_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_learning: Option<crate::address_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_learning: Option<crate::service_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_learning: Option<crate::client_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_dns_learning: Option<crate::seed_dns_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_mount_learning: Option<crate::seed_mount_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_access_learning: Option<crate::legacy_access_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_logs_learning: Option<crate::legacy_logs_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_draft: Option<crate::client_teaching::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_learning: Option<crate::dns_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_draft: Option<crate::dns_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switching_draft: Option<crate::switching_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switching_learning: Option<crate::switching_teaching::Learning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_draft: Option<crate::mac_learning_workspace::Draft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_learning: Option<crate::mac_learning_teaching::Learning>,
    pub command: String,
    pub service_port: Option<String>,
    pub dns_address: Option<String>,
    pub dns_serial: Option<String>,
    pub mount_device: Option<String>,
    pub checked: bool,
    pub ipv4_check: Option<[String; 5]>,
    pub practice_check: Option<[String; 5]>,
    pub migrated_host: bool,
    pub legacy_host_reviewed: bool,
    pub legacy_practice_reviewed: bool,
    pub migrated_practice_host: bool,
}
impl WorkspaceData {
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Default::default()
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.dhcpv4_draft.as_ref().is_none_or(|d| d.valid())
            && self.dhcpv4_learning.as_ref().is_none_or(|l| l.valid())
            && self
                .unit_manager_terminal
                .as_ref()
                .is_none_or(|s| s.valid())
            && self.unit_manager_draft.as_ref().is_none_or(|d| d.valid())
            && self
                .unit_manager_learning
                .as_ref()
                .is_none_or(|l| l.valid())
            && self.acl_draft.as_ref().is_none_or(|d| d.valid())
            && self.acl_learning.as_ref().is_none_or(|l| l.valid())
            && self.mac_draft.as_ref().is_none_or(|d| d.valid())
            && self.mac_learning.as_ref().is_none_or(|l| l.valid())
            && self.switching_draft.as_ref().is_none_or(|d| d.valid())
            && self.switching_learning.as_ref().is_none_or(|l| l.valid())
            && self.script_learning.as_ref().is_none_or(|l| l.valid())
            && self
                .script_history
                .as_ref()
                .is_none_or(|r| r.valid() && r.engine_revision < crate::shell::EVALUATOR_REVISION)
            && self.script_history_more.len() + usize::from(self.script_history.is_some())
                <= MAX_SCRIPT_HISTORY
            && self
                .script_history_more
                .iter()
                .all(|r| r.valid() && r.engine_revision < crate::shell::EVALUATOR_REVISION)
            && self.terminal_learning.as_ref().is_none_or(|l| l.valid())
            && self.process_learning.as_ref().is_none_or(|l| l.valid())
            && self.accounts_learning.as_ref().is_none_or(|l| l.valid())
            && self.ospf_draft.as_ref().is_none_or(|d| d.valid())
            && self.ospf_learning.as_ref().is_none_or(|l| l.valid())
            && self.discovery_draft.as_ref().is_none_or(|d| d.valid())
            && self.discovery_learning.as_ref().is_none_or(|l| l.valid())
            && self.storage_draft.as_ref().is_none_or(|d| d.valid())
            && self.storage_learning.as_ref().is_none_or(|l| l.valid())
            && self.ipv6_draft.as_ref().is_none_or(|d| d.valid())
            && self.ipv6_learning.as_ref().is_none_or(|l| l.valid())
            && self.routing_v4_draft.as_ref().is_none_or(|d| d.valid())
            && self.routing_v4_learning.as_ref().is_none_or(|l| l.valid())
            && self.address_learning.as_ref().is_none_or(|l| l.valid())
            && self.service_learning.as_ref().is_none_or(|l| l.valid())
            && self.client_learning.as_ref().is_none_or(|l| l.valid())
            && self.seed_dns_learning.as_ref().is_none_or(|l| l.valid())
            && self.seed_mount_learning.as_ref().is_none_or(|l| l.valid())
            && self
                .legacy_access_learning
                .as_ref()
                .is_none_or(|l| l.valid())
            && self.legacy_logs_learning.as_ref().is_none_or(|l| l.valid())
            && self.client_draft.as_ref().is_none_or(|d| d.valid())
            && self.dns_learning.as_ref().is_none_or(|l| l.valid())
            && self.dns_draft.as_ref().is_none_or(|d| d.valid())
            && self.command.len() <= 4096
            && [
                &self.service_port,
                &self.dns_address,
                &self.dns_serial,
                &self.mount_device,
            ]
            .iter()
            .all(|s| s.as_ref().is_none_or(|s| s.len() <= 256))
            && [&self.ipv4_check, &self.practice_check]
                .iter()
                .all(|s| s.as_ref().is_none_or(|s| s.iter().all(|a| a.len() <= 64)))
    }
}
pub fn can_preserve_script_history(a: &Attempt) -> bool {
    let State::Script {
        run: Some(record), ..
    } = &a.state
    else {
        return true;
    };
    if record.current() {
        return true;
    }
    a.workspace.as_ref().is_none_or(|w| {
        w.script_history
            .as_ref()
            .into_iter()
            .chain(&w.script_history_more)
            .any(|r| r == record)
            || w.script_history_more.len() + usize::from(w.script_history.is_some())
                < MAX_SCRIPT_HISTORY
    })
}
pub fn preserve_script_history(a: &mut Attempt) {
    // Imported/full history is rejected before entering a writable workspace.
    if !can_preserve_script_history(a) {
        return;
    }
    if let State::Script { run, .. } = &mut a.state
        && run.as_ref().is_some_and(|r| !r.current())
    {
        let w = a.workspace.get_or_insert_with(WorkspaceData::new);
        let record = run.take().unwrap();
        if w.script_history.is_none() {
            w.script_history = Some(record);
        } else if !w
            .script_history
            .as_ref()
            .into_iter()
            .chain(&w.script_history_more)
            .any(|r| r == &record)
        {
            w.script_history_more.push(record);
        }
        w.checked = false;
    }
}
pub fn prepare(a: &mut Attempt) {
    preserve_script_history(a);
    let w = a.workspace.get_or_insert_with(WorkspaceData::new);
    if let State::DhcpV4 { simulation } = &a.state
        && w.dhcpv4_draft.is_none()
    {
        w.dhcpv4_draft = crate::dhcpv4_workspace::Draft::new(simulation).ok();
    }
    if let State::UnitManager { simulation } = &a.state
        && w.unit_manager_draft.is_none()
    {
        w.unit_manager_draft = crate::unit_manager_workspace::Draft::new(simulation).ok();
    }
    if let State::Ospf { simulation } = &a.state
        && w.ospf_draft.is_none()
    {
        w.ospf_draft = crate::ospf_workspace::Draft::new(simulation).ok();
    }
    if let State::Ipv4Acl { network } = &a.state
        && w.acl_draft.is_none()
    {
        w.acl_draft = crate::acl_workspace::Draft::new(network).ok();
    }
    if let State::PersistentMounts { storage } = &a.state
        && w.storage_draft.is_none()
    {
        w.storage_draft = Some(crate::storage_workspace::Draft::new(storage));
    }
    if let State::Discovery { simulation } = &a.state
        && w.discovery_draft.is_none()
    {
        w.discovery_draft = Some(crate::discovery_workspace::Draft::new(simulation));
    }
    if let State::AuthoritativeDns { server } = &a.state
        && w.dns_draft.is_none()
    {
        w.dns_draft = Some(crate::dns_workspace::Draft::new(server));
    }
    if let State::MacLearning { simulation } = &a.state
        && w.mac_draft.is_none()
    {
        w.mac_draft = Some(
            crate::mac_learning_workspace::Draft::new(simulation).expect("validated MAC state"),
        );
    }
    if let State::Switching { fabric } = &a.state
        && w.switching_draft.is_none()
    {
        w.switching_draft = Some(crate::switching_workspace::Draft::new(fabric));
    }
    if let State::Ipv6 { topology } = &a.state
        && w.ipv6_draft.is_none()
    {
        w.ipv6_draft = Some(crate::ipv6_workspace::Draft::new(topology));
    }
    if let State::Ipv4Routing { topology } = &a.state
        && w.routing_v4_draft.is_none()
    {
        w.routing_v4_draft = Some(crate::ipv4_routing_workspace::Draft::new(topology));
    }
    if let Some(mut e) = ipv4::Exercise::from_state(&a.state) {
        w.migrated_host |= ipv4::upgrade_exercise(&mut e);
        ipv4::write_exercise(&mut a.state, &e);
        if w.migrated_host && !w.legacy_host_reviewed {
            w.ipv4_check = None;
        }
        if let Some(e) = a.ipv4.as_mut().and_then(|l| l.variant.as_mut()) {
            w.migrated_practice_host |= ipv4::upgrade_exercise(e);
            if w.migrated_practice_host && !w.legacy_practice_reviewed {
                w.practice_check = None;
            }
        }
    }
}
pub fn snapshot(e: &ipv4::Exercise) -> [String; 5] {
    [
        e.answers[0].clone(),
        e.answers[1].clone(),
        e.answers[2].clone(),
        e.answers[3].clone(),
        e.second.clone().unwrap_or_default(),
    ]
}
pub fn start_practice(a: &mut Attempt) {
    prepare(a);
    let original = ipv4::Exercise::from_state(&a.state).expect("IPv4 practice state");
    let l = a
        .ipv4
        .get_or_insert_with(|| ipv4::Lesson::from_legacy(original.ip, original.prefix, a.step));
    if !l.use_variant {
        l.original_section = l.section;
    }
    l.sequence = l.sequence.saturating_add(1);
    let e = ipv4::generated(l.sequence);
    l.sandbox_host = u32::from(e.ip) & !ipv4::mask(e.prefix).unwrap();
    l.variant = Some(e);
    l.use_variant = true;
    l.section = ipv4::Section::Original;
    l.variant_section = ipv4::Section::Original;
    l.variant_submitted = [false; 3];
    l.variant_revealed = [false; 3];
    let w = a.workspace.as_mut().unwrap();
    w.practice_check = None;
    w.migrated_practice_host = false;
}
pub fn select_original(a: &mut Attempt) {
    let original = ipv4::Exercise::from_state(&a.state).expect("IPv4 original state");
    if let Some(l) = &mut a.ipv4 {
        l.use_variant = false;
        l.section = l.original_section;
        // A wider practice subnet can have more host bits than the authored /27.
        l.sandbox_host &= !ipv4::mask(original.prefix).unwrap();
    }
}
pub fn vlsm_issue(s: &State, index: usize) -> Option<String> {
    let State::Vlsm {
        base,
        prefix,
        allocations,
    } = s
    else {
        return None;
    };
    let a = allocations.get(index)?;
    if a.cidr.trim().is_empty() {
        return Some("Enter a network/prefix for this team.".into());
    }
    let Some((address, p)) = a.cidr.trim().split_once('/') else {
        return Some("Use an IPv4 network followed by /prefix.".into());
    };
    let (Ok(address), Ok(p)) = (address.parse::<std::net::Ipv4Addr>(), p.parse::<u8>()) else {
        return Some("Use four IPv4 numbers and a prefix from 0 to 30.".into());
    };
    if p > 30 {
        return Some(
            "Use a conventional subnet through /30; reserve network and broadcast.".into(),
        );
    }
    let (n, b) = engine::bounds(address, p)?;
    if u32::from(address) != n {
        return Some("This is a host address. Use the aligned first address of this block.".into());
    }
    let (pool_start, pool_end) = engine::bounds(*base, *prefix)?;
    if n < pool_start || b > pool_end {
        return Some("This allocation extends outside the supplied address pool.".into());
    }
    if (b as u64 - n as u64 + 1) < a.hosts as u64 + 2 {
        return Some(format!(
            "Too small for {} hosts plus the two reserved addresses.",
            a.hosts
        ));
    }
    for (other_index, other) in allocations.iter().enumerate() {
        if other_index != index
            && let Some((on, ob, _)) = engine::cidr(other.cidr.trim())
            && n <= ob
            && on <= b
        {
            return Some(format!(
                "This block overlaps {}. Give each team its own addresses.",
                other.name
            ));
        }
    }
    None
}
/// Save both DNS editor fields atomically; changing disk does not reload the authoritative zone.
pub fn edit_dns(s: &mut State, address: &str, serial: &str) -> Result<(), String> {
    address
        .trim()
        .parse::<std::net::Ipv4Addr>()
        .map_err(|_| "A record: enter a full IPv4 address.")?;
    let serial = serial
        .trim()
        .parse::<u32>()
        .map_err(|_| "Serial: enter a whole number from 0 to 4294967295.")?;
    let State::Dns {
        records,
        serial: current,
        ..
    } = s
    else {
        return Err("No DNS configuration in this activity".into());
    };
    let record = records.values_mut().next().ok_or("No modeled A record")?;
    *record = address.trim().into();
    *current = serial;
    Ok(())
}
pub fn edit_mount(s: &mut State, device: &str) -> Result<(), String> {
    let device = device.trim();
    if !device.starts_with("/dev/")
        || device.len() > 256
        || device.contains(char::is_whitespace)
        || device.contains("..")
    {
        return Err(
            "Enter a device path such as /dev/vdb1; this edits only the simulated fstab entry."
                .into(),
        );
    }
    let State::Mount {
        configured_device, ..
    } = s
    else {
        return Err("No mount configuration in this activity".into());
    };
    *configured_device = device.into();
    Ok(())
}
/// Explain failed checks using the current model, rather than prescribing one command sequence.
pub fn issues(s: &State, g: &engine::Goal) -> Vec<String> {
    if let (State::Processes { table }, engine::Goal::Processes { requirements }) = (s, g) {
        // Several objectives can depend on one live process. Explain the first
        // failing prerequisite once rather than repeating three reset messages.
        let mut explained = std::collections::BTreeSet::new();
        return requirements
            .iter()
            .filter_map(|requirement| {
                use crate::processes::Requirement;
                let pid = match requirement {
                    Requirement::Phase { pid, .. }
                    | Requirement::Niceness { pid, .. }
                    | Requirement::NoPendingTermination { pid }
                    | Requirement::Unchanged { pid, .. } => *pid,
                };
                let (pass, message) = requirement.check(table);
                (!pass && explained.insert(pid)).then_some(message)
            })
            .collect();
    }
    let checks = engine::checks(s, g);
    checks.iter().enumerate().filter(|(_,c)|!c.0).map(|(i,(_,label))| match s {
        State::Filesystem{tree}=>if let engine::Goal::Filesystem{requirements}=g {
            use crate::filesystem::Requirement;
            match &requirements[i] {
                Requirement::Contents{path,..}=>if tree.files.contains_key(path) {format!("{path} has different text. Check which source you copied; compare it with cat before retrying.")} else {format!("{path} is missing as a file. Inspect its parent folder and check the destination name.")},
                Requirement::Entry{path,directory}=>format!("{path} must be a {}. Use ls on its parent to check the name and type.",if *directory {"directory"}else{"file"}),
                Requirement::Absent{path}=>format!("{path} still exists. Check that its replacement is safe before removing the requested old entry."),
                Requirement::Directory{path}=>format!("You are in {}. Finish in {path}; pwd shows your current directory.",tree.cwd),
                requirement @ Requirement::DirectoryEntries{..}=>requirement.check(tree).1,
                Requirement::Mode{path,mode}=>format!("{path} does not have permission {mode:03o}. Inspect with ls -l (or -ld for a directory). umask changes new entries, not existing ones."),
                Requirement::Ownership{path,owner,group}=>format!("{path} must retain owner {owner} and group {group}. Inspect with ls -l. Recreating a file gives it a new owner; moving preserves its ownership."),
                Requirement::Access{path,user,operation,allowed}=>if !tree.nodes.contains_key(path) {format!("{path} is missing. Restore the required target; deleting it does not satisfy an access restriction.")} else if !tree.users.contains_key(user) {format!("The required simulated account {user} is missing.")} else if tree.nodes[path].directory != operation.directory() {format!("{path} must be a {}; access checks cannot be satisfied by replacing it with a different kind of entry.",if operation.directory() {"directory"} else {"file"})} else {format!("{user} {} {} at {path}. Inspect ownership/group with ls -l and id; check search permission on every parent directory. Owner permissions take precedence over group/other permissions.",if *allowed {"cannot yet"} else {"can still"},operation.description())},
                Requirement::Mask{mask}=>format!("The current creation mask is {:03o}; the task requires {mask:03o}. Run umask alone to inspect it.",tree.umask),
            }
        }else{label.clone()},
        State::Service{port,running,loaded_port,failed,..} => if let engine::Goal::Service{port:expected}=g {match i {
            0=>if port.parse::<u16>().ok().filter(|p|*p>0).is_none(){format!("Saved port {port:?} is not a number from 1 to 65535. This task requires port {expected}; inspect the configuration and journal.")}else{format!("Saved port is {port}; this task requires {expected}. Repair the saved configuration.")},
            1=>if !*running {if *failed{"Atlas failed to start. Its journal explains the recorded error; repair the configuration before starting it again.".into()}else{"Atlas is stopped. Start it after checking the saved configuration.".into()}}else{format!("Atlas is running on port {}, but this task requires {expected}. Saving a file does not apply changes; restart it to load the saved configuration.",loaded_port.unwrap_or(0))},
            _=>"Atlas is disabled at boot. Running now and starting at boot are separate settings.".into(),
        }}else{label.clone()},
        State::Dns{serial,loaded_serial,..}=>if i==0{"The saved A record differs from the requested answer. Edit the record before reloading.".into()}else if serial<=loaded_serial{"The new zone has not been activated. Use a serial greater than the loaded serial, validate and reload.".into()}else{"The saved zone is not the loaded answer yet. Validate, reload named, then query resolution.".into()},
        State::Mount{mounted,writable,..}=>match i {0=>"Usage is above 80%. Inspect the disposable cache; cleanup requires a writable mounted volume.".into(),1=>"The persistent entry points to the wrong device. Correct the /data configuration and verify mounting.".into(),_=>if !*mounted{"/data is unmounted. Check its saved device entry before mounting.".into()}else if !*writable{"/data is read-only. Repair the mount state before deleting the disposable cache.".into()}else{label.clone()}},
        State::Unix{nodes,users}=>if i==0 {
            if let engine::Goal::Access{path,user,read,write,..}=g {
                let mut directories=vec!["/".to_string()];let mut current=String::new();
                let parts:Vec<_>=path.trim_start_matches('/').split('/').collect();
                for part in parts.iter().take(parts.len().saturating_sub(1)){current.push('/');current.push_str(part);directories.push(current.clone());}
                if let Some(blocked)=directories.iter().find(|p|!engine::access(nodes,users,p,user,1)) {
                    format!("{user} cannot traverse {blocked}. Directory search/execute permission is needed before the file’s own permissions can help.")
                } else if let Some(file)=nodes.get(path) {
                    let class=if file.owner==*user{"owner"}else if users.get(user).is_some_and(|groups|groups.contains(&file.group)){"group"}else{"other"};
                    format!("{user} reaches the file, but its {class} permissions do not allow {}. The matching owner/group/other class determines access.",if *read&&*write{"reading and writing"}else if *write{"writing"}else{"reading"})
                } else {"The required simulated file is missing.".into()}
            } else {label.clone()}
        }else{
            if let engine::Goal::Access{path,user,exclusive:true,..}=g && let Some(other)=users.keys().find(|other| other.as_str()!=user && other.as_str()!="root" && [1,2,4].iter().any(|bit|engine::access(nodes,users,path,other,*bit))) { format!("{other} can still access the file. Inspect group membership and the file’s owner/group permissions; keep access for {user} only.") } else { "The file still grants permissions to the other class. Remove those permissions while retaining the required user’s access.".into() }
        },
        State::Logs{result,..}=>if result.is_empty(){"No matching text is selected yet. Locate and inspect the log, then filter its ERROR lines.".into()}else{"The last query does not contain exactly the ERROR lines. Check the file, case-sensitive pattern and any unfiltered extra lines.".into()},
        State::Network{..}=>match i {0=>"The client address changed. Keep the workstation’s assigned address.".into(),1=>"The client mask differs from the router’s LAN mask. Inspect the interface configuration.".into(),_=>"The default route does not use the reachable LAN router. Inspect both the prefix and next hop, then test the remote path.".into()},
        _=>label.clone(),
    }).collect()
}

/// Pending file edits must not borrow current assignment success from an older saved value.
pub fn pending_service_draft(a: &Attempt) -> bool {
    matches!(&a.state,State::Service{port,..}if a.workspace.as_ref().and_then(|w|w.service_port.as_ref()).is_some_and(|d|d!=port))
}

/// Missing legacy drafts describe the applied state, not an invented unsaved edit.
pub fn pending_seed_configuration_draft(a: &Attempt) -> bool {
    let Some(w) = &a.workspace else {
        return false;
    };
    match &a.state {
        State::Dns {
            records, serial, ..
        } => {
            w.dns_address
                .as_ref()
                .is_some_and(|draft| records.values().next().is_none_or(|saved| draft != saved))
                || w.dns_serial
                    .as_ref()
                    .is_some_and(|draft| draft != &serial.to_string())
        }
        State::Mount {
            configured_device, ..
        } => w
            .mount_device
            .as_ref()
            .is_some_and(|draft| draft != configured_device),
        _ => false,
    }
}
