//! Opt-in bounded DOS geometry for authored virtual storage lessons.
//! Explicit abstract DOS layout, not raw EBR bytes or actual parted policy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    Dos512,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Primary,
    Extended,
    Logical,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub kind: Kind,
    pub partition: Partition,
    /// A disclosed one-sector teaching reservation, not a raw EBR chain.
    pub ebr: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    #[serde(deserialize_with = "sequence::<_,Entry,MAX_PARTITIONS>")]
    pub entries: Vec<Entry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Create {
        #[serde(deserialize_with = "text::<_,64>")]
        disk: String,
    },
    Add {
        #[serde(deserialize_with = "text::<_,64>")]
        disk: String,
        role: Kind,
        start: u64,
        end: u64,
        type_hint: TypeHint,
    },
}
pub const HELP: &str = "Virtual MBR preparation only: parted -s /dev/vdX mklabel msdos; parted -s /dev/vdX unit s mkpart primary|logical ext4|linux-swap STARTs ENDs; parted -s /dev/vdX unit s mkpart extended STARTs ENDs; parted -s /dev/vdX unit s print; parted -s /dev/vdX align-check optimal NUMBER; blkid [PARTITION]. Exact positive sectors and supplied paths only. Initialize an empty supplied disk, then allocate primary, extended or logical ranges. Primary/extended slots are 1–4; logical payloads start at number 5 inside one extended container. The container is not a filesystem. This bounded 512-byte-sector profile admits disks up to 2 TiB and reserves one explicit sector per logical partition for modeled EBR metadata. It does not emulate raw EBR chains or actual parted placement. Existing disclosed blank-only ext4/swap initialization remains. No overwrite, delete, resize, boot code, CHS, actual devices, kernel changes or host shell. Simulator refusal is not real-tool protection.";

fn overlaps(a: (u64, u64), b: (u64, u64)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}
pub(super) fn validate_geometry(d: &Disk, table: &Table) -> Result<()> {
    if d.table.is_some()
        || d.occupied_without_table
        || d.sectors > (1_u64 << 32)
        || table.entries.len() > MAX_PARTITIONS
    {
        return Err(invalid(
            "MBR geometry conflicts with GPT/data or exceeds this profile",
        ));
    }
    let mut numbers = BTreeSet::new();
    let mut logical = BTreeSet::new();
    let mut extended = None;
    for e in &table.entries {
        let p = &e.partition;
        if p.type_hint == TypeHint::EfiSystem {
            return Err(invalid("This EFI System Partition role belongs to the supplied GPT profile."));
        }
        if !numbers.insert(p.number)
            || p.name != format!("mbr{}", p.number)
            || p.start < 1
            || p.start > p.end
            || p.end >= d.sectors
        {
            return Err(invalid("Invalid MBR slot or inclusive payload range"));
        }
        match e.kind {
            Kind::Primary | Kind::Extended => {
                if !(1..=4).contains(&p.number) || e.ebr.is_some() {
                    return Err(invalid("Primary and extended entries use slots 1–4"));
                }
                if e.kind == Kind::Extended
                    && (extended.replace((p.start, p.end)).is_some()
                        || p.content != Content::Blank
                        || p.type_hint != TypeHint::LinuxData)
                {
                    return Err(invalid("Only one blank extended container is supported"));
                }
            }
            Kind::Logical => {
                if !(5..=MAX_PARTITIONS as u8).contains(&p.number) {
                    return Err(invalid("Logical partition numbering starts at 5"));
                }
                logical.insert(p.number);
                if e.ebr.is_none_or(|sector| sector >= p.start) {
                    return Err(invalid(
                        "A logical payload needs an earlier EBR reservation",
                    ));
                }
            }
        }
    }
    if logical.iter().copied().ne(5..5 + logical.len() as u8) {
        return Err(invalid(
            "This creation-only profile has consecutive logical slots",
        ));
    }
    for (i, e) in table.entries.iter().enumerate() {
        let p = &e.partition;
        if e.kind == Kind::Logical {
            let (start, end) =
                extended.ok_or_else(|| invalid("Create an extended container first"))?;
            let ebr = e.ebr.expect("checked EBR");
            if ebr < start || p.start < start || p.end > end {
                return Err(invalid(
                    "Logical payload and EBR must fit inside the extended container",
                ));
            }
        }
        for other in &table.entries[..i] {
            let q = &other.partition;
            if (e.kind == Kind::Logical) != (other.kind == Kind::Logical) {
                // Nested logical/extended geometry is intentional; primary
                // exclusion follows from containment and outer disjointness.
                continue;
            }
            if overlaps((p.start, p.end), (q.start, q.end)) {
                return Err(err(
                    ErrorKind::Overlap,
                    "MBR payloads or outer entries overlap",
                ));
            }
            if let (Some(a), Some(b)) = (e.ebr, other.ebr)
                && (a == b || (q.start..=q.end).contains(&a) || (p.start..=p.end).contains(&b))
            {
                return Err(err(
                    ErrorKind::Overlap,
                    "Logical payloads overlap reserved EBR space",
                ));
            }
        }
    }
    Ok(())
}

/// At most 32 intervals. Never walk sectors, even on a large virtual disk.
fn first_ebr(table: &Table, start: u64, before: u64) -> Result<u64> {
    let mut occupied = Vec::new();
    for e in table.entries.iter().filter(|e| e.kind == Kind::Logical) {
        occupied.push((e.partition.start, e.partition.end));
        let ebr = e.ebr.ok_or_else(|| invalid("Missing EBR reservation"))?;
        occupied.push((ebr, ebr));
    }
    occupied.sort_unstable();
    let mut candidate = start;
    for (a, b) in occupied {
        if candidate < a {
            break;
        }
        if candidate <= b {
            candidate = b
                .checked_add(1)
                .ok_or_else(|| invalid("EBR end overflow"))?;
        }
    }
    if candidate >= before {
        return Err(err(
            ErrorKind::Overlap,
            "Leave free modeled EBR space before the logical payload",
        ));
    }
    Ok(candidate)
}

pub(super) fn apply(s: &mut State, action: Action) -> Result<Output> {
    s.validate()?;
    if s.mbr_profile.is_none() {
        return Err(err(
            ErrorKind::Unsupported,
            "MBR preparation is not enabled by this scenario",
        ));
    }
    let mut next = s.clone();
    match action {
        Action::Create { disk } => {
            let d = s.disk(&disk)?;
            State::writable(d)?;
            if d.sectors > (1_u64 << 32) {
                return Err(limited("This 512-byte MBR profile admits at most 2 TiB"));
            }
            if d.table.is_some() || d.mbr.is_some() || d.occupied_without_table {
                return Err(err(
                    ErrorKind::Occupied,
                    "Initialize only an empty supplied disk; no replacement",
                ));
            }
            next.disks.get_mut(&disk).unwrap().mbr = Some(Table {
                entries: Vec::new(),
            });
        }
        Action::Add {
            disk,
            role,
            start,
            end,
            type_hint,
        } => {
            let d = s.disk(&disk)?;
            State::writable(d)?;
            let table = d
                .mbr
                .as_ref()
                .ok_or_else(|| invalid("Create an MBR table first"))?;
            if table.entries.len() >= MAX_PARTITIONS {
                return Err(limited("This profile's bounded partition slots are full"));
            }
            let (number, ebr) = if role == Kind::Logical {
                if table
                    .entries
                    .iter()
                    .filter(|e| e.kind == Kind::Logical)
                    .count()
                    >= MAX_PARTITIONS - 4
                {
                    return Err(limited(
                        "This bounded profile's logical slots 5–16 are full",
                    ));
                }
                let container = table
                    .entries
                    .iter()
                    .find(|e| e.kind == Kind::Extended)
                    .ok_or_else(|| invalid("Logical partitions need an extended container"))?;
                (
                    5 + table
                        .entries
                        .iter()
                        .filter(|e| e.kind == Kind::Logical)
                        .count() as u8,
                    Some(first_ebr(table, container.partition.start, start)?),
                )
            } else {
                ((1..=4).find(|n| table.entries.iter().all(|e| e.partition.number != *n))
                    .ok_or_else(|| limited("All four primary table slots are occupied; free space is not another slot"))?, None)
            };
            if role == Kind::Extended && type_hint != TypeHint::LinuxData {
                return Err(invalid(
                    "The extended container has no swap/filesystem type",
                ));
            }
            let entry = Entry {
                kind: role,
                partition: Partition {
                    number,
                    name: format!("mbr{number}"),
                    start,
                    end,
                    type_hint,
                    content: Content::Blank,
                },
                ebr,
            };
            next.disks
                .get_mut(&disk)
                .unwrap()
                .mbr
                .as_mut()
                .unwrap()
                .entries
                .push(entry);
            next.disks
                .get_mut(&disk)
                .unwrap()
                .mbr
                .as_mut()
                .unwrap()
                .entries
                .sort_by_key(|e| e.partition.number);
        }
    }
    next.validate()?;
    *s = next;
    Ok(Output::success("Virtual MBR allocation applied; contents are unchanged/blank until explicitly initialized.\n".into()))
}

pub(super) fn report(id: &str, d: &Disk, table: &Table) -> String {
    let mut output = format!(
        "SIMULATOR MBR report: {id}; {} sectors; logical512B; optimal{} sectors\nNumber Kind Start End Sectors TypeHint Signature ModelEBR\n",
        d.sectors, d.optimal_alignment,
    );
    for e in &table.entries {
        let p = &e.partition;
        let role = match e.kind {
            Kind::Primary => "primary",
            Kind::Extended => "extended-container",
            Kind::Logical => "logical",
        };
        let hint = if e.kind == Kind::Extended {
            "container"
        } else {
            match p.type_hint {
                TypeHint::LinuxData => "linux-data",
                TypeHint::LinuxSwap => "linux-swap",
                TypeHint::EfiSystem => "efi-system",
            }
        };
        output.push_str(&format!(
            "{} {role} {} {} {} {hint} {} {}\n",
            p.number,
            p.start,
            p.end,
            p.end - p.start + 1,
            content_type(&p.content),
            e.ebr.map_or_else(|| "none".into(), |n| n.to_string()),
        ));
    }
    output.push_str("ModelEBR is this exercise's one-sector reservation, not a raw chain or actual parted placement.\n");
    output
}
