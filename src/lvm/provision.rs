//! Opt-in bounded linear VG/LV provisioning. No device, filesystem or host I/O.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    Linear,
    Lifecycle,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Group {
        name: String,
        extent_bytes: u64,
        devices: Vec<String>,
    },
    Volume {
        group: String,
        name: String,
        size: Size,
    },
    Remove {
        id: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Size {
    Bytes(u64),
    Extents(u64),
    Free,
}
pub const HELP: &str = "Opt-in virtual linear provisioning: vgcreate [-s SIZE[M|G]] VG /dev/NAME...; lvcreate -L SIZE[M|G] -n NAME VG; lvcreate -l COUNT|100%FREE -n NAME VG; lvremove -y VG/LV. --physicalextentsize, --size, --extents, --name and --yes are accepted equivalents; name/size options may occur in either order. VG extent default is 4 MiB; this subset supports powers of two from1 through64 MiB. New LVs are raw, with no filesystem or mount. lvremove only admits raw volumes without a modeled filesystem, and requires explicit -y; real lvremove is destructive and has wider behavior. No snapshots, VG/PV removal, rename, lvreduce, thin/RAID/striped volumes, activation, force, host commands or shell. Growth forms: pvs; vgs; lvs; pvcreate /dev/NAME...; vgextend VG /dev/NAME...; lvextend -l N|+N|+100%FREE VG/LV; lvextend -L N|+N[M|G] VG/LV; resize2fs /dev/VG/LV [BLOCKS|SIZE[M|G]]. LVM tools also accept /dev/VG/LV; resize2fs requires that path. Suffixless LVM sizes mean MiB; suffixless resize2fs sizes mean4096-byte blocks. Reports are fixed simulator tables with explicit byte columns. Only supplied devices exist; name collisions test those virtual entries, not the host /dev tree.";
pub fn valid_volume_name(s: &str) -> bool {
    name(s)
        && !matches!(s, "snapshot" | "pvmove")
        && ![
            "_cdata", "_cmeta", "_corig", "_iorig", "_mimage", "_mlog", "_pmspare", "_rimage",
            "_rmeta", "_tdata", "_tmeta", "_vdata", "_vorigin", "_wcorig",
        ]
        .iter()
        .any(|reserved| s.contains(reserved))
}
pub(super) fn parse(args: &[&str]) -> Result<Action> {
    match args.first().copied() {
        Some("vgcreate") => {
            let mut pos = 1;
            let mut extent_bytes = 4 * MIB;
            if args
                .get(pos)
                .is_some_and(|f| matches!(*f, "-s" | "--physicalextentsize"))
            {
                extent_bytes = bytes(
                    args.get(pos + 1)
                        .ok_or_else(|| invalid("Give an extent size after -s"))?,
                    MIB,
                )?;
                pos += 2;
            }
            let vg = args
                .get(pos)
                .filter(|s| name(s))
                .ok_or_else(|| invalid("Give a simple VG name before its supplied devices"))?;
            if !(MIB..=64 * MIB).contains(&extent_bytes) || !extent_bytes.is_power_of_two() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This profile models power-of-two VG extent sizes from1 through64 MiB",
                ));
            }
            Ok(Action::Group {
                name: (*vg).into(),
                extent_bytes,
                devices: paths(&args[pos + 1..])?,
            })
        }
        Some("lvcreate") => {
            // Deliberately disclosed short/long pair options; reject everything
            // else instead of pretending to be getopt/the complete LVM parser.
            let mut pos = 1;
            let mut id = None;
            let mut size = None;
            while pos < args.len() && args[pos].starts_with('-') {
                let flag = args[pos];
                let value = args
                    .get(pos + 1)
                    .ok_or_else(|| invalid("Give a value after each supported lvcreate option"))?;
                match flag {
                    "-n" | "--name" if id.is_none() && valid_volume_name(value) => {
                        id = Some((*value).to_owned())
                    }
                    "-l" | "--extents" if size.is_none() => {
                        size = Some(if *value == "100%FREE" {
                            Size::Free
                        } else {
                            Size::Extents(decimal(value)?)
                        })
                    }
                    "-L" | "--size" if size.is_none() => {
                        size = Some(Size::Bytes(bytes(value, MIB)?))
                    }
                    _ => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Use one explicit name and one supported size; repeated, combined and other options are not supported",
                        ));
                    }
                }
                pos += 2;
            }
            if pos + 1 != args.len() || !name(args[pos]) {
                return Err(invalid("After the options, give one simple VG name"));
            }
            Ok(Action::Volume {
                group: args[pos].into(),
                name: id.ok_or_else(|| invalid("Name the new LV with -n NAME"))?,
                size: size.ok_or_else(|| invalid("Set the new LV size with -l or -L"))?,
            })
        }
        Some("lvremove") => match args {
            [_, "-y" | "--yes", id] => {
                volume_name(id)?;
                Ok(Action::Remove { id: (*id).into() })
            }
            _ => Err(error(
                ErrorKind::Unsupported,
                "This noninteractive simulator requires explicit lvremove -y VG/LV and only removes raw LVs without a modeled filesystem. Real removal is destructive.",
            )),
        },
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}
pub(super) fn apply(s: &mut State, action: Action) -> Result<String> {
    if s.provisioning.is_none() {
        return Err(error(
            ErrorKind::Unsupported,
            "Creating groups or volumes and removing raw volumes are supported only in new-volume lessons. This lesson supports existing-volume growth only; no work changed.",
        ));
    }
    match action {
        Action::Group {
            name,
            extent_bytes,
            devices,
        } => {
            if s.devices.contains_key(&format!("/dev/{name}")) {
                return Err(error(
                    ErrorKind::Occupied,
                    "That VG name collides with a supplied /dev entry; choose a different simple name",
                ));
            }
            if s.groups.contains_key(&name) {
                return Err(error(
                    ErrorKind::Occupied,
                    "That VG already exists; vgcreate does not extend or replace it",
                ));
            }
            if s.groups.len() >= MAX_GROUPS {
                return Err(limit("At most four VGs are modeled"));
            }
            let group = VolumeGroup {
                extent_bytes,
                volumes: BTreeMap::new(),
            };
            for p in &devices {
                let d = s
                    .devices
                    .get(p)
                    .ok_or_else(|| error(ErrorKind::Unknown, format!("No supplied device {p}")))?;
                if d.role == DeviceRole::Foreign || !d.group.is_empty() {
                    return Err(error(
                        ErrorKind::Occupied,
                        format!(
                            "{p} is occupied or belongs to a VG; no force or migration is modeled"
                        ),
                    ));
                }
                s.pv_extents_for(d, &group)?;
            }
            for p in &devices {
                let d = s.devices.get_mut(p).unwrap();
                d.role = DeviceRole::PhysicalVolume;
                d.group.clone_from(&name);
            }
            s.groups.insert(name.clone(), group);
            Ok(format!(
                "Virtual volume group {name} created with {extent_bytes}-byte extents; supplied blank devices initialized as needed"
            ))
        }
        Action::Volume { group, name, size } => {
            let g = s.groups.get(&group).ok_or_else(|| {
                error(
                    ErrorKind::Unknown,
                    "No such VG; inspect vgs before creating its LV",
                )
            })?;
            if g.volumes.contains_key(&name) {
                return Err(error(
                    ErrorKind::Occupied,
                    "That LV already exists; lvcreate does not overwrite or grow it",
                ));
            }
            if s.groups.values().map(|g| g.volumes.len()).sum::<usize>() >= MAX_VOLUMES {
                return Err(limit("At most16 logical volumes are modeled"));
            }
            let amount = match size {
                Size::Extents(n) => n,
                Size::Bytes(n) => n.div_ceil(g.extent_bytes),
                Size::Free => u64::from(s.free_extents(&group)?),
            };
            if amount == 0 || amount > MAX_EXTENTS as u64 {
                return Err(limit(
                    "Choose1..4096 extents, not zero or an oversized allocation",
                ));
            }
            if amount > u64::from(s.free_extents(&group)?) {
                return Err(error(
                    ErrorKind::NoSpace,
                    "The VG has too few free physical extents for that new LV; inspect vgs and choose a supported smaller allocation",
                ));
            }
            let used = s.used();
            let mut extents = Vec::with_capacity(amount as usize);
            for (path, d) in s.devices.iter().filter(|(_, d)| d.group == group) {
                for index in 0..s.pv_extents_for(d, g)? {
                    if !used.contains(&(path.as_str(), index)) {
                        extents.push(Extent {
                            pv: path.clone(),
                            index,
                        });
                        if extents.len() == amount as usize {
                            break;
                        }
                    }
                }
                if extents.len() == amount as usize {
                    break;
                }
            }
            if extents.len() != amount as usize {
                return Err(invalid(
                    "Free-extent accounting disagrees; no candidate is published",
                ));
            }
            let bytes = amount * g.extent_bytes;
            s.groups.get_mut(&group).unwrap().volumes.insert(
                name.clone(),
                LogicalVolume {
                    extents,
                    filesystem: None,
                },
            );
            if let Some(availability) = &mut s.activation {
                availability.active.insert(format!("{group}/{name}"), true);
            }
            Ok(format!(
                "Virtual raw LV {group}/{name} created: {bytes} bytes, {amount} extents. It has no filesystem or mount."
            ))
        }
        Action::Remove { id } => {
            activation::removable(s, &id)?;
            let (vg, lv) = volume_name(&id)?;
            let (_, v) = s.volume(&id)?;
            if v.filesystem.is_some() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Removal of a filesystem-bearing LV is outside this raw-allocation practice subset. No data was removed; real lvremove can destroy a filesystem.",
                ));
            }
            s.groups.get_mut(vg).unwrap().volumes.remove(lv);
            if let Some(availability) = &mut s.activation {
                availability.active.remove(&format!("{vg}/{lv}"));
            }
            Ok(format!(
                "Virtual raw LV {vg}/{lv} removed and its extents returned to the VG"
            ))
        }
    }
}
