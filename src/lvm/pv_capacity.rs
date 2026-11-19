//! Source-enabled recognized PV size, separate from immutable virtual geometry.
//! No device, partition, kernel, filesystem or subprocess I/O.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    MetadataSize,
}
pub const HELP: &str = "Virtual PV metadata capacity: pvresize /dev/NAME...; pvresize --setphysicalvolumesize SIZE[M|G] /dev/NAME... . Integer M/G sizes are binary; without a suffix the size is MiB. Only supplied labeled PVs exist. The physical device size is read-only; the recognized PV end can change within it. Shrinking refuses allocated physical extents beyond the new end; free-space totals alone are insufficient. This changes neither LV size nor filesystem capacity. Existing pvs/vgs/lvs, pvcreate, vgextend, lvextend and resize2fs growth forms retain their disclosed bounds. No actual disk/partition resize, pvmove, LV/filesystem shrink, force, other options/units, mount, kernel or host shell. Whole candidate refusal is a simulator property, not real LVM rollback assurance.";
pub(super) fn validate(s: &State) -> Result<()> {
    if s.pv_capacity.is_some()
        && (s.provisioning.is_some() || s.activation.is_some() || s.identity.is_some())
    {
        return Err(invalid(
            "PV metadata-size practice is separate from other optional LVM profiles",
        ));
    }
    for d in s.devices.values() {
        match (s.pv_capacity, d.role, d.pv_bytes) {
            (Some(_), DeviceRole::PhysicalVolume, Some(bytes))
                if bytes >= DATA_START + MIB && bytes <= d.bytes && bytes.is_multiple_of(512) => {}
            (Some(_), DeviceRole::Blank | DeviceRole::Foreign, None) | (None, _, None) => {}
            _ => {
                return Err(invalid(
                    "Recognized PV bytes need the explicit source profile, a labeled PV and a sector-aligned end inside its supplied device",
                ));
            }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Action {
    devices: Vec<String>,
    size: Option<u64>,
}
pub(super) fn parse(args: &[&str]) -> Result<Action> {
    match args {
        ["pvresize", "--setphysicalvolumesize", size, tail @ ..] => Ok(Action {
            devices: paths(tail)?,
            size: Some(bytes(size, MIB)?),
        }),
        ["pvresize", tail @ ..] if tail.iter().all(|s| !s.starts_with('-')) => Ok(Action {
            devices: paths(tail)?,
            size: None,
        }),
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}
/// Derived placement, not a free-space estimate or an actual pvdisplay report.
pub fn highest_allocated(s: &State, path: &str) -> Result<Option<u32>> {
    s.validate()?;
    if !s.devices.contains_key(path) {
        return Err(error(ErrorKind::Unknown, "No supplied physical device"));
    }
    Ok(highest(s, path))
}
fn highest(s: &State, path: &str) -> Option<u32> {
    s.groups
        .values()
        .flat_map(|g| g.volumes.values())
        .flat_map(|v| v.extents.iter())
        .filter(|e| e.pv == path)
        .map(|e| e.index)
        .max()
}
pub(super) fn apply(s: &mut State, action: Action) -> Result<String> {
    if s.pv_capacity.is_none() {
        return Err(error(ErrorKind::Unsupported, s.command_help()));
    }
    // Validate the complete named set before mutation. This does not claim that
    // real multi-PV commands have the simulator's all-or-nothing publication.
    for path in &action.devices {
        let d = s
            .devices
            .get(path)
            .ok_or_else(|| error(ErrorKind::Unknown, format!("No supplied device {path}")))?;
        if d.role != DeviceRole::PhysicalVolume {
            return Err(error(
                ErrorKind::Occupied,
                format!(
                    "{path} is not a supplied labeled PV. pvresize does not initialize or wipe it"
                ),
            ));
        }
        let target = action.size.unwrap_or(d.bytes);
        if target < DATA_START + MIB || target > d.bytes || !target.is_multiple_of(512) {
            return Err(invalid(
                "Recognized PV size must leave the disclosed data start, be sector aligned, and fit inside the unchanged supplied device",
            ));
        }
        if !d.group.is_empty() {
            let count = (target - DATA_START) / s.groups[&d.group].extent_bytes;
            if count == 0 || count > MAX_EXTENTS as u64 {
                return Err(limit(
                    "The requested PV end needs 1..4096 whole extents in this simulator",
                ));
            }
            if let Some(high) = highest(s, path)
                && u64::from(high) >= count
            {
                return Err(error(
                    ErrorKind::Occupied,
                    format!(
                        "{path}: allocated physical extent {high} lies beyond the requested end ({count} extents, numbered 0 through {}). Keep an end beyond every allocated extent; total free space does not show its placement. No relocation or LV/filesystem shrink is modeled.",
                        count - 1
                    ),
                ));
            }
        }
    }
    for path in &action.devices {
        let d = s.devices.get_mut(path).unwrap();
        d.pv_bytes = Some(action.size.unwrap_or(d.bytes));
    }
    Ok(format!(
        "Virtual recognized PV metadata updated for {}. Device geometry, LV sizes, filesystem capacities and files are unchanged; inspect pvs/vgs/lvs before choosing the next layer.",
        action.devices.join(", ")
    ))
}
