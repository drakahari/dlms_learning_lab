//! Opt-in raw-record COW storage. No host devices, filesystem or commands.
//! Native/source-goal admission remains closed until the teaching workspace exists.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    #[serde(deserialize_with = "map::<_,cow::Bank,4>")]
    pub origins: BTreeMap<String, cow::Bank>,
}
pub const HELP: &str = "Virtual classic COW snapshots over supplied raw records only: pvs; vgs; lvs; lvcreate -s -L SIZE[M|G] -n NAME [-c CHUNK[K]] VG/ORIGIN; lvextend -L SIZE|+SIZE[M|G] VG/SNAPSHOT; lvremove -y VG/SNAPSHOT. --snapshot, --size, --name, --chunksize and --yes are supported equivalents. Name/size/chunk options may occur in either order. LVM sizes are integer MiB by default and round up to whole supplied extents; chunk values are integer KiB by default, power-of-two4–512KiB, default4KiB. COW physical storage is reserved separately from the origin address range. This exercise reserves one chunk for metadata, not actual LVM metadata accounting. No thin/merge, origin resizing/removal, mounts, filesystems, automatic growth, activation, force, other utility options or host commands. Fixed record writes/reads are separate simulator-only application stimuli, not Linux commands.";
fn canonical(id: &str) -> Result<String> {
    let (g, v) = volume_name(id)?;
    Ok(format!("{g}/{v}"))
}
fn raw_volume<'a>(s: &'a State, id: &str) -> Result<(&'a VolumeGroup, &'a LogicalVolume)> {
    let (g, v) = volume_name(id)?;
    let group = s
        .groups
        .get(g)
        .ok_or_else(|| invalid("Snapshot refers to an unknown VG"))?;
    let volume = group
        .volumes
        .get(v)
        .ok_or_else(|| invalid("Snapshot refers to an unknown LV"))?;
    Ok((group, volume))
}
pub(super) fn validate(s: &State) -> Result<()> {
    let Some(model) = &s.snapshots else {
        return Ok(());
    };
    if s.provisioning.is_some()
        || s.activation.is_some()
        || s.identity.is_some()
        || s.pv_capacity.is_some()
        || model.origins.is_empty()
        || model.origins.len() > 4
    {
        return Err(invalid(
            "Raw-record snapshot practice is separate from other optional LVM profiles",
        ));
    }
    let mut stores = BTreeSet::new();
    for (id, bank) in &model.origins {
        if canonical(id)? != *id {
            return Err(invalid("Origin identities use canonical VG/LV names"));
        }
        bank.validate()?;
        let (g, v) = raw_volume(s, id)?;
        if v.filesystem.is_some() || v.extents.len() as u64 * g.extent_bytes != bank.byte_len() {
            return Err(invalid(
                "Fixed records need an equally sized raw origin, not a modeled filesystem",
            ));
        }
        let (vg, origin) = volume_name(id)?;
        for name in bank.snapshot_names() {
            let path = format!("{vg}/{name}");
            let status = bank.status(name)?;
            if name == origin || model.origins.contains_key(&path) || !stores.insert(path.clone()) {
                return Err(invalid("Snapshot stores and origins must be distinct"));
            }
            let (group, volume) = raw_volume(s, &path)?;
            if volume.filesystem.is_some()
                || volume.extents.len() as u64 * group.extent_bytes != status.capacity_bytes
            {
                return Err(invalid(
                    "Snapshot COW capacity must equal its distinct reserved raw LV allocation",
                ));
            }
        }
    }
    Ok(())
}
fn snapshot_origin(s: &State, path: &str) -> Result<String> {
    let path = canonical(path)?;
    let (vg, name) = volume_name(&path)?;
    s.snapshots.as_ref().ok_or_else(||error(ErrorKind::Unsupported,HELP))?.origins.iter()
        .find(|(origin,b)|origin.starts_with(&format!("{vg}/"))&&b.snapshot_names().any(|n|n==name))
        .map(|(id,_)|id.clone()).ok_or_else(||error(ErrorKind::Unsupported,"Only a modeled COW snapshot store can be changed; origins and unrelated LVs are protected"))
}
fn allocation(s: &State, vg: &str, count: usize) -> Result<Vec<Extent>> {
    if count == 0 || count > MAX_EXTENTS {
        return Err(limit("COW allocation needs1–4096extents"));
    }
    if count > s.free_extents(vg)? as usize {
        return Err(error(
            ErrorKind::NoSpace,
            "The VG does not have enough free physical extents for this COW store",
        ));
    }
    let used = s.used();
    let mut result = Vec::with_capacity(count);
    for (p, d) in s.devices.iter().filter(|(_, d)| d.group == vg) {
        for index in 0..s.pv_extents_for(d, &s.groups[vg])? {
            if !used.contains(&(p.as_str(), index)) {
                result.push(Extent {
                    pv: p.clone(),
                    index,
                });
                if result.len() == count {
                    return Ok(result);
                }
            }
        }
    }
    Err(invalid("COW physical allocation invariant differs"))
}
#[derive(Debug)]
enum Command {
    Create {
        origin: String,
        name: String,
        bytes: u64,
        chunk: u32,
    },
    Extend {
        path: String,
        bytes: u64,
        relative: bool,
    },
    Remove(String),
    Report(Report),
}
fn chunks(raw: &str) -> Result<u32> {
    let raw = raw.strip_suffix(['k', 'K']).unwrap_or(raw);
    let n = decimal(raw)?;
    if !(4..=512).contains(&n) || !n.is_power_of_two() {
        return Err(error(
            ErrorKind::Unsupported,
            "Snapshot chunks use power-of-two integer KiB from4 through512",
        ));
    }
    Ok((n * 1024) as u32)
}
fn parse_command(input: &str) -> Result<Command> {
    if input.len() > MAX_COMMAND_BYTES {
        return Err(limit("Snapshot command exceeds2048bytes"));
    }
    if !input.is_ascii()
        || input
            .bytes()
            .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\t' | b'\r'))
    {
        return Err(error(ErrorKind::Unsupported, HELP));
    }
    let a = input.split_ascii_whitespace().collect::<Vec<_>>();
    match a.as_slice() {
        ["pvs"] => Ok(Command::Report(Report::Pvs)),
        ["vgs"] => Ok(Command::Report(Report::Vgs)),
        ["lvs"] => Ok(Command::Report(Report::Lvs)),
        ["lvextend", "-L" | "--size", amount, path] => {
            let (relative, value) = amount
                .strip_prefix('+')
                .map_or((false, *amount), |s| (true, s));
            Ok(Command::Extend {
                path: canonical(path)?,
                bytes: bytes(value, MIB)?,
                relative,
            })
        }
        ["lvremove", "-y" | "--yes", path] => Ok(Command::Remove(canonical(path)?)),
        ["lvcreate", tail @ ..] => {
            let (mut snap, mut size, mut name, mut chunk) = (false, None, None, None);
            let mut i = 0;
            while i < tail.len() && tail[i].starts_with('-') {
                let flag = tail[i];
                if matches!(flag, "-s" | "--snapshot") && !snap {
                    snap = true;
                    i += 1;
                    continue;
                }
                let value = tail
                    .get(i + 1)
                    .ok_or_else(|| invalid("Give a value after each supported snapshot option"))?;
                match flag {
                    "-L" | "--size" if size.is_none() => size = Some(bytes(value, MIB)?),
                    "-n" | "--name" if name.is_none() && provision::valid_volume_name(value) => {
                        name = Some((*value).to_owned())
                    }
                    "-c" | "--chunksize" if chunk.is_none() => chunk = Some(chunks(value)?),
                    _ => return Err(error(ErrorKind::Unsupported, HELP)),
                }
                i += 2;
            }
            if !snap || tail.len() != i + 1 {
                return Err(error(ErrorKind::Unsupported, HELP));
            }
            Ok(Command::Create {
                origin: canonical(tail[i])?,
                name: name.ok_or_else(|| invalid("Give an explicit snapshot name"))?,
                bytes: size.ok_or_else(|| invalid("Give an explicit COW store size"))?,
                chunk: chunk.unwrap_or(4096),
            })
        }
        _ => Err(error(ErrorKind::Unsupported, HELP)),
    }
}
/// Syntax admission for this explicit profile, never execution or host dispatch.
pub fn validate_command(input: &str) -> Result<()> {
    parse_command(input).map(|_| ())
}
/// Caller is the normal bounded in-process engine, never a host shell.
pub(super) fn execute(s: &mut State, input: &str) -> Result<String> {
    s.validate()?;
    let command = parse_command(input)?;
    if let Command::Report(kind) = command {
        let show_snapshots = matches!(kind, Report::Lvs);
        let mut text = s.report(kind)?;
        if show_snapshots {
            text.push_str("SIMULATOR COW facts: Origin Snapshot AddressRange(B) Reserved(B) UsedChunks UsableChunks Status\n");
            for (origin, bank) in &s.snapshots.as_ref().unwrap().origins {
                for name in bank.snapshot_names() {
                    let r = bank.status(name)?;
                    text.push_str(&format!(
                        "{origin} {name} {} {} {} {} {}\n",
                        bank.byte_len(),
                        r.capacity_bytes,
                        r.copied_chunks,
                        r.usable_chunks,
                        if r.valid {
                            "Valid"
                        } else {
                            "Invalid · data unavailable"
                        }
                    ));
                }
            }
        }
        return Ok(text);
    }
    let mut next = s.clone();
    let output = match command {
        Command::Create {
            origin,
            name,
            bytes,
            chunk,
        } => {
            let model = s.snapshots.as_ref().unwrap();
            if !model.origins.contains_key(&origin) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Choose one of the supplied raw-record origins; snapshot-of-snapshot and arbitrary origins are unsupported",
                ));
            }
            let (vg, _) = volume_name(&origin)?;
            let g = &s.groups[vg];
            if g.volumes.contains_key(&name) {
                return Err(error(
                    ErrorKind::Occupied,
                    "An LV with this name already exists",
                ));
            }
            let count = bytes.div_ceil(g.extent_bytes);
            if count == 0 || count > MAX_EXTENTS as u64 {
                return Err(limit("COW store exceeds the bounded extent count"));
            }
            let reserved = count * g.extent_bytes;
            let extents = allocation(s, vg, count as usize)?;
            next.snapshots
                .as_mut()
                .unwrap()
                .origins
                .get_mut(&origin)
                .unwrap()
                .create(&name, reserved, chunk)?;
            next.groups.get_mut(vg).unwrap().volumes.insert(
                name.clone(),
                LogicalVolume {
                    extents,
                    filesystem: None,
                },
            );
            format!(
                "Snapshot {vg}/{name} created with {reserved} bytes of separate COW storage. No origin data has been copied yet."
            )
        }
        Command::Extend {
            path,
            bytes,
            relative,
        } => {
            let origin = snapshot_origin(s, &path)?;
            let (vg, name) = volume_name(&path)?;
            let g = &s.groups[vg];
            let v = &g.volumes[name];
            let current = v.extents.len() as u64 * g.extent_bytes;
            let desired = if relative {
                current
                    .checked_add(bytes)
                    .ok_or_else(|| limit("COW size overflow"))?
            } else {
                bytes
            };
            let count = desired.div_ceil(g.extent_bytes);
            if count <= v.extents.len() as u64 || count > MAX_EXTENTS as u64 {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Choose a larger supported COW size; reduction is not modeled",
                ));
            }
            let reserved = count * g.extent_bytes;
            let more = allocation(s, vg, count as usize - v.extents.len())?;
            next.snapshots
                .as_mut()
                .unwrap()
                .origins
                .get_mut(&origin)
                .unwrap()
                .extend(name, reserved)?;
            next.groups
                .get_mut(vg)
                .unwrap()
                .volumes
                .get_mut(name)
                .unwrap()
                .extents
                .extend(more);
            format!(
                "COW store {path} extended to {reserved} bytes; existing point-in-time data is retained."
            )
        }
        Command::Remove(path) => {
            let origin = snapshot_origin(s, &path)?;
            let (vg, name) = volume_name(&path)?;
            next.snapshots
                .as_mut()
                .unwrap()
                .origins
                .get_mut(&origin)
                .unwrap()
                .remove(name)?;
            next.groups.get_mut(vg).unwrap().volumes.remove(name);
            format!(
                "Snapshot {path} removed and its COW extents freed. The origin remains unchanged."
            )
        }
        Command::Report(_) => unreachable!(),
    };
    next.validate()?;
    *s = next;
    Ok(output)
}
pub fn read_record(
    s: &State,
    origin: &str,
    snapshot: Option<&str>,
    record: &str,
) -> Result<String> {
    s.validate()?;
    let origin = canonical(origin)?;
    let bank = s
        .snapshots
        .as_ref()
        .and_then(|m| m.origins.get(&origin))
        .ok_or_else(|| error(ErrorKind::Unknown, "No supplied raw-record origin"))?;
    snapshot.map_or_else(
        || bank.current(record).map(str::to_owned),
        |id| bank.read(id, record),
    )
}
/// Explicit simulator-only application stimulus, never a Linux utility.
pub fn write_record(
    s: &mut State,
    origin: &str,
    record: &str,
    data: &str,
) -> Result<cow::WriteResult> {
    s.validate()?;
    let origin = canonical(origin)?;
    let mut next = s.clone();
    let bank = next
        .snapshots
        .as_mut()
        .and_then(|m| m.origins.get_mut(&origin))
        .ok_or_else(|| error(ErrorKind::Unknown, "No supplied raw-record origin"))?;
    let result = bank.write(record, data)?;
    next.validate()?;
    *s = next;
    Ok(result)
}
pub fn restore_record(
    s: &mut State,
    origin: &str,
    snapshot: &str,
    record: &str,
) -> Result<cow::WriteResult> {
    let data = read_record(s, origin, Some(snapshot), record)?;
    write_record(s, origin, record, &data)
}
