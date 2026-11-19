//! Source-enabled names over stable virtual objects. No host I/O or real UUIDs.
use super::*;

pub const HELP: &str = "Virtual LVM names: lvrename VG OLD NEW; lvrename VG/OLD NEW; lvrename /dev/VG/OLD NEW; vgrename OLD NEW. Use simple supplied names. Rename keeps the same allocation, files and availability; it does not copy or recreate storage, mount filesystems, or update application/fstab references. Existing disclosed linear creation/growth, raw retirement and vgchange -a y|n VG remain available. No flags, UUID source, duplicate-VG discovery, historical LVs, mapper escaping, boot/udev/kernel operations, real devices or host shell. Reports are fixed simulator tables; ModelActive is a simulator column.";
const MAX_NEXT: u32 = 65_536;

/// Internal simulation identity, never presented as a real LVM UUID. Only new
/// source-enabled worlds contain it; old snapshots omit this field unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(deserialize_with = "map::<_,u32,4>")]
    pub groups: BTreeMap<String, u32>,
    #[serde(deserialize_with = "map::<_,u32,16>")]
    pub volumes: BTreeMap<String, u32>,
    pub next: u32,
}
impl Identity {
    pub(super) fn validate(&self, s: &State) -> Result<()> {
        if s.provisioning != Some(provision::Profile::Lifecycle)
            || s.activation.is_none()
            || self.groups.len() > MAX_GROUPS
            || self.volumes.len() > MAX_VOLUMES
            || !(1..=MAX_NEXT).contains(&self.next)
            || self.groups.keys().ne(s.groups.keys())
        {
            return Err(invalid(
                "Names need the supplied lifecycle/availability profile and exact bounded object inventory",
            ));
        }
        let expected = s
            .groups
            .iter()
            .flat_map(|(g, d)| d.volumes.keys().map(move |v| format!("{g}/{v}")))
            .collect::<BTreeSet<_>>();
        if self.volumes.keys().ne(expected.iter()) {
            return Err(invalid(
                "Name identity must cover each canonical VG/LV exactly once",
            ));
        }
        let mut ids = BTreeSet::new();
        for id in self.groups.values().chain(self.volumes.values()) {
            if *id == 0 || *id >= self.next || !ids.insert(*id) {
                return Err(invalid(
                    "Object identities must be nonzero, distinct and below the bounded next identity",
                ));
            }
        }
        Ok(())
    }
    fn fresh(&mut self) -> Result<u32> {
        if self.next >= MAX_NEXT {
            return Err(limit(
                "This simulated session exhausted its object-creation budget; no change applied",
            ));
        }
        let id = self.next;
        self.next += 1;
        Ok(id)
    }
}

/// Creation/deletion maintain identities centrally after the existing action.
/// Rename moves the keys itself, so this never turns a rename into recreation.
pub(super) fn reconcile(s: &mut State) -> Result<()> {
    let Some(mut ids) = s.identity.take() else {
        return Ok(());
    };
    ids.groups.retain(|g, _| s.groups.contains_key(g));
    let volumes = s
        .groups
        .iter()
        .flat_map(|(g, d)| d.volumes.keys().map(move |v| format!("{g}/{v}")))
        .collect::<BTreeSet<_>>();
    ids.volumes.retain(|v, _| volumes.contains(v));
    for g in s.groups.keys() {
        if !ids.groups.contains_key(g) {
            let id = ids.fresh()?;
            ids.groups.insert(g.clone(), id);
        }
    }
    for v in volumes {
        if !ids.volumes.contains_key(&v) {
            let id = ids.fresh()?;
            ids.volumes.insert(v, id);
        }
    }
    s.identity = Some(ids);
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Group {
        old: String,
        new: String,
    },
    Volume {
        group: String,
        old: String,
        new: String,
    },
}
pub(super) fn parse(args: &[&str]) -> Result<Action> {
    match args {
        ["vgrename", old, new] if name(old) && name(new) => Ok(Action::Group {
            old: (*old).into(),
            new: (*new).into(),
        }),
        ["lvrename", group, old, new]
            if name(group) && name(old) && provision::valid_volume_name(new) =>
        {
            Ok(Action::Volume {
                group: (*group).into(),
                old: (*old).into(),
                new: (*new).into(),
            })
        }
        ["lvrename", path, new] if provision::valid_volume_name(new) => {
            let (group, old) = volume_name(path)?;
            Ok(Action::Volume {
                group: group.into(),
                old: old.into(),
                new: (*new).into(),
            })
        }
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}
fn move_key<T>(map: &mut BTreeMap<String, T>, old: &str, new: String) {
    let value = map.remove(old).expect("validated rename source");
    map.insert(new, value);
}
pub(super) fn apply(s: &mut State, action: Action) -> Result<String> {
    if s.identity.is_none() {
        return Err(error(ErrorKind::Unsupported, s.command_help()));
    }
    match action {
        Action::Group { old, new } => {
            if !s.groups.contains_key(&old) {
                return Err(error(
                    ErrorKind::Unknown,
                    "No supplied source VG; inspect vgs before renaming",
                ));
            }
            if s.groups.contains_key(&new) {
                return Err(error(
                    ErrorKind::Occupied,
                    "The destination VG already exists. Rename does not merge or overwrite groups; choose an unused name",
                ));
            }
            let members = s.groups[&old]
                .volumes
                .keys()
                .map(|v| (format!("{old}/{v}"), format!("{new}/{v}")))
                .collect::<Vec<_>>();
            move_key(&mut s.groups, &old, new.clone());
            for d in s.devices.values_mut().filter(|d| d.group == old) {
                d.group.clone_from(&new);
            }
            let ids = s.identity.as_mut().unwrap();
            move_key(&mut ids.groups, &old, new.clone());
            let availability = s.activation.as_mut().unwrap();
            for (before, after) in members {
                move_key(&mut ids.volumes, &before, after.clone());
                move_key(&mut availability.active, &before, after.clone());
                for open in availability.open.iter_mut().filter(|v| **v == before) {
                    open.clone_from(&after);
                }
            }
            Ok(format!(
                "Virtual group {old} renamed to {new}; allocations, files and availability retained. Name-based application and mount references are not automatically edited."
            ))
        }
        Action::Volume { group, old, new } => {
            let g = s
                .groups
                .get_mut(&group)
                .ok_or_else(|| error(ErrorKind::Unknown, "No supplied VG; inspect vgs"))?;
            if !g.volumes.contains_key(&old) {
                return Err(error(
                    ErrorKind::Unknown,
                    "No supplied source LV at that name; inspect lvs",
                ));
            }
            if g.volumes.contains_key(&new) {
                return Err(error(
                    ErrorKind::Occupied,
                    "The destination LV already exists. Rename does not overwrite another volume; choose an unused name",
                ));
            }
            move_key(&mut g.volumes, &old, new.clone());
            let before = format!("{group}/{old}");
            let after = format!("{group}/{new}");
            move_key(
                &mut s.identity.as_mut().unwrap().volumes,
                &before,
                after.clone(),
            );
            let a = s.activation.as_mut().unwrap();
            move_key(&mut a.active, &before, after.clone());
            for open in a.open.iter_mut().filter(|v| **v == before) {
                open.clone_from(&after);
            }
            Ok(format!(
                "Virtual LV {before} renamed to {after}; same allocation, files and availability. The old pathname no longer resolves in this model."
            ))
        }
    }
}
