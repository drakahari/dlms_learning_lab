//! Opt-in empty-group and unowned-PV metadata retirement. No host I/O.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Group(String),
    Physical(Vec<String>),
}
pub const HELP: &str = "Virtual raw LVM lifecycle: vgremove VG (or vgremove -y VG) removes one EMPTY supplied group; pvremove /dev/NAME... removes labels only from supplied unowned PVs. --yes is equivalent for empty-group removal. Retire unused raw LVs first with lvremove -y VG/LV. Real vgremove can also remove LVs after confirmation and real pvremove has broader/force behavior; neither is modeled here. No force, filesystem-bearing removal, activation/mount state, secure erase or host devices/commands. Clearing a PV label does not securely erase data. Creation/growth forms: vgcreate [-s SIZE[M|G]] VG /dev/NAME...; lvcreate -L SIZE[M|G] -n NAME VG or -l COUNT|100%FREE; pvcreate /dev/NAME...; vgextend VG /dev/NAME...; lvextend -l N|+N|+100%FREE VG/LV or -L N|+N[M|G]; resize2fs /dev/VG/LV [BLOCKS|SIZE[M|G]]; pvs; vgs; lvs. Supported linear long-option equivalents remain available. Only supplied devices; no shell syntax. Reports use fixed simulator byte columns.";
pub(super) fn parse(args: &[&str]) -> Result<Action> {
    match args {
        ["vgremove", vg] | ["vgremove", "-y" | "--yes", vg] if name(vg) => {
            Ok(Action::Group((*vg).into()))
        }
        ["pvremove", tail @ ..] => Ok(Action::Physical(paths(tail)?)),
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}
pub(super) fn apply(s: &mut State, action: Action) -> Result<String> {
    if s.provisioning != Some(provision::Profile::Lifecycle) {
        return Err(error(ErrorKind::Unsupported, s.command_help()));
    }
    match action {
        Action::Group(vg) => {
            let g = s.groups.get(&vg).ok_or_else(|| {
                error(
                    ErrorKind::Unknown,
                    "No supplied group with that name; inspect vgs",
                )
            })?;
            if !g.volumes.is_empty() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This exercise removes only empty groups. Inspect lvs and retire only the intended unused raw LVs first. Real vgremove can destroy contained LVs; this refusal is not a real-system safety guarantee.",
                ));
            }
            // A valid current model has no external allocation referring into an empty VG.
            for d in s.devices.values_mut().filter(|d| d.group == vg) {
                d.group.clear();
            }
            s.groups.remove(&vg);
            Ok(format!(
                "Virtual group {vg} removed; its devices remain labeled physical volumes, now unowned. Inspect pvs before assuming their labels are gone."
            ))
        }
        Action::Physical(paths) => {
            for path in &paths {
                let d = s.devices.get(path).ok_or_else(|| {
                    error(ErrorKind::Unknown, format!("No supplied device {path}"))
                })?;
                if d.role == DeviceRole::Foreign {
                    return Err(error(
                        ErrorKind::Occupied,
                        format!("{path} carries other supplied data; no force or wipe is modeled"),
                    ));
                }
                if !d.group.is_empty() {
                    return Err(error(
                        ErrorKind::Occupied,
                        format!(
                            "{path} still belongs to {}. Retire the intended unused raw LVs and remove the empty group before clearing this label; no force or vgreduce is modeled.",
                            d.group
                        ),
                    ));
                }
                if d.role != DeviceRole::PhysicalVolume {
                    return Err(error(
                        ErrorKind::Invalid,
                        format!(
                            "{path} already has no modeled LVM PV label; inspect current state. No change was applied."
                        ),
                    ));
                }
            }
            for path in &paths {
                s.devices.get_mut(path).unwrap().role = DeviceRole::Blank;
            }
            Ok(format!(
                "Virtual PV labels removed from {}. These devices have no modeled signature; this is not secure erasure or filesystem formatting.",
                paths.join(", ")
            ))
        }
    }
}
