//! Optional, bounded swap-area policy over virtual storage. Never host memory or devices.
use super::{Error, ErrorKind, Result, Source, at, bounded_config, error, path, uuid};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const MAX_AREAS: usize = 4;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub uuid: String,
    /// Supplied usable area size, not raw partition geometry or a measurement.
    pub size_bytes: u64,
    pub prepared: bool,
    pub available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Area,4>")]
    pub areas: Vec<Area>,
}
impl Inventory {
    pub fn validate(&self) -> Result<()> {
        let mut ids = BTreeSet::new();
        let mut devices = BTreeSet::new();
        let mut uuids = BTreeSet::new();
        if self.areas.is_empty() || self.areas.len() > MAX_AREAS {
            return Err(error(
                ErrorKind::Limit,
                "Supply one to four virtual swap areas",
            ));
        }
        for a in &self.areas {
            if a.id.is_empty()
                || a.id.len() > 32
                || !a
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                || !path(&a.device)
                || !a.device.starts_with("/dev/")
                || a.device.len() > 64
                || !uuid(&a.uuid)
                || a.uuid != a.uuid.to_ascii_lowercase()
                || a.size_bytes < 4096
                || a.size_bytes > (1u64 << 40)
                || a.size_bytes % 4096 != 0
                || !ids.insert(&a.id)
                || !devices.insert(&a.device)
                || !uuids.insert(&a.uuid)
            {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid or duplicate supplied swap area identity, alias or usable size",
                ));
            }
        }
        Ok(())
    }
    pub fn resolve(&self, source: &Source) -> Result<&Area> {
        self.areas
            .iter()
            .find(|a| match source {
                Source::Device(d) => a.device == *d,
                Source::Uuid(u) => a.uuid == *u,
            })
            .ok_or_else(|| {
                error(
                    ErrorKind::UnknownSource,
                    "No supplied swap area matches that exact device or UUID",
                )
            })
    }
    pub fn validate_active(&self, active: &BTreeMap<String, u16>) -> Result<()> {
        self.validate()?;
        if active.len() > MAX_AREAS
            || active.iter().any(|(id, p)| {
                *p > 32767
                    || !self
                        .areas
                        .iter()
                        .any(|a| a.id == *id && a.available && a.prepared)
            })
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid active swap table; only available prepared supplied areas and priorities0–32767 are supported",
            ));
        }
        Ok(())
    }
    pub fn validate_against(&self, mounts: &super::Inventory) -> Result<()> {
        self.validate()?;
        if self.areas.iter().any(|a| {
            mounts.devices.contains_key(&a.device)
                || mounts.volumes.contains_key(&a.id)
                || mounts
                    .volumes
                    .values()
                    .any(|v| v.uuid.eq_ignore_ascii_case(&a.uuid))
        }) {
            return Err(error(
                ErrorKind::Invalid,
                "Swap identities/aliases must be distinct from supplied mount volumes",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub line: usize,
    pub source: Source,
    pub automatic: bool,
    pub priority: u16,
}
pub struct Table {
    pub mounts: Vec<super::Entry>,
    pub swaps: Vec<Entry>,
}
fn priority(s: &str) -> Result<u16> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(
            ErrorKind::Invalid,
            "Use an explicit whole-number swap priority0–32767; larger values are preferred",
        ));
    }
    s.parse::<u16>()
        .ok()
        .filter(|n| *n <= 32767)
        .ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "Swap priority must be between0 and32767 in this activity",
            )
        })
}
pub fn source(s: &str) -> Result<Source> {
    if let Some(u) = s.strip_prefix("UUID=") {
        if uuid(u) {
            return Ok(Source::Uuid(u.into()));
        }
        return Err(error(
            ErrorKind::Invalid,
            "Use UUID= with a full hyphenated UUID from the supplied inventory",
        ));
    }
    if path(s) && s.starts_with("/dev/") && s.len() <= 64 {
        return Ok(Source::Device(s.into()));
    }
    Err(error(
        ErrorKind::Unsupported,
        "Only supplied /dev paths or UUID= identities are supported for swap; no swap files, labels, PARTUUID or host devices",
    ))
}
fn entry(fields: &[&str], line: usize) -> Result<Entry> {
    let [src, "none", "swap", opts, "0", "0"] = fields else {
        return Err(at(
            error(
                ErrorKind::Unsupported,
                "Use six fields for this swap subset: source none swap options 0 0. The unused target is not a mount directory; other dump/pass forms are outside this activity",
            ),
            line,
        ));
    };
    let mut automatic = true;
    let mut pri = None;
    for opt in opts.split(',') {
        match opt {
            // swapon -a checks the presence of noauto; adding auto later
            // does not remove that option (unlike mount flag merging).
            "defaults" | "auto" => {}
            "noauto" => automatic = false,
            _ if opt.starts_with("pri=") && pri.is_none() => {
                pri = Some(priority(&opt[4..]).map_err(|e| at(e, line))?)
            }
            _ => {
                return Err(at(
                    error(
                        ErrorKind::Unsupported,
                        "Supported swap options: defaults, auto/noauto and one explicit pri=0–32767. Unknown options are refused by this simulator; real swapon may ignore them. No discard/nofail or repeated pri is modeled",
                    ),
                    line,
                ));
            }
        }
    }
    Ok(Entry { line, source: source(src).map_err(|e|at(e,line))?, automatic,
        priority: pri.ok_or_else(||at(error(ErrorKind::Unsupported,"This activity requires explicit pri=0–32767; kernel-selected negative defaults are not simulated"),line))? })
}
/// Preserve line numbers and the established mount parser; enable only with source facts.
pub fn table(text: &str) -> Result<Table> {
    bounded_config(text)?;
    let mut mount_text = String::with_capacity(text.len());
    let mut swaps = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let fields = trimmed.split_ascii_whitespace().collect::<Vec<_>>();
        if !trimmed.starts_with('#') && fields.get(2) == Some(&"swap") {
            if trimmed.contains(['\\', '\'', '"', '#']) {
                return Err(at(
                    error(
                        ErrorKind::Unsupported,
                        "Escaped/quoted swap fields and inline comments are outside this literal subset",
                    ),
                    i + 1,
                ));
            }
            if swaps.len() >= MAX_AREAS {
                return Err(at(
                    error(
                        ErrorKind::Limit,
                        "At most four saved swap entries are supported",
                    ),
                    i + 1,
                ));
            }
            swaps.push(entry(&fields, i + 1)?);
        } else {
            mount_text.push_str(line);
        }
        mount_text.push('\n');
    }
    Ok(Table {
        mounts: super::parse(&mount_text)?,
        swaps,
    })
}
pub fn configuration_errors(inventory: &Inventory, entries: &[Entry]) -> Vec<Error> {
    let mut ids = BTreeSet::new();
    let mut errors = vec![];
    for e in entries {
        match inventory.resolve(&e.source) {
            Err(err) => errors.push(at(err,e.line)),
            Ok(a) if !ids.insert(&a.id) => errors.push(at(error(ErrorKind::Unsupported,"Two swap rows select the same supplied area, possibly through different aliases; duplicate policy is outside this activity"),e.line)),
            _ => {},
        }
    }
    errors
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Activated { id: String },
    Skipped { id: String },
    Failed { line: usize, reason: String },
}
pub fn activate(
    inventory: &Inventory,
    active: &mut BTreeMap<String, u16>,
    source: &Source,
    pri: u16,
    skip_active: bool,
) -> Result<bool> {
    inventory.validate_active(active)?;
    if pri > 32767 {
        return Err(error(ErrorKind::Invalid, "Swap priority must be0–32767"));
    }
    let area = inventory.resolve(source)?;
    if !area.available {
        return Err(error(
            ErrorKind::UnknownSource,
            "The supplied swap device is unavailable",
        ));
    }
    if !area.prepared {
        return Err(error(
            ErrorKind::TypeMismatch,
            "The supplied device has no prepared swap signature. This activity uses prepared areas; mkswap is not an activation command",
        ));
    }
    if active.contains_key(&area.id) {
        return if skip_active {
            Ok(false)
        } else {
            Err(error(
                ErrorKind::Occupied,
                "Swap area is already active; editing saved pri or activating again does not reprioritize it. Use the supported swapoff, then activate with the intended priority",
            ))
        };
    }
    active.insert(area.id.clone(), pri);
    Ok(true)
}
/// Validates complete syntax before starting; valid row activation failures are partial.
/// This finite preparse policy differs from general utility error handling.
pub fn activate_all(
    inventory: &Inventory,
    active: &mut BTreeMap<String, u16>,
    text: &str,
) -> Result<Vec<Outcome>> {
    inventory.validate_active(active)?;
    let entries = table(text)?.swaps;
    // Duplicates are an unsupported policy shape; unknown/unavailable rows remain
    // independent activation failures so earlier successful areas stay active.
    let mut ids = BTreeSet::new();
    for e in &entries {
        if let Ok(a) = inventory.resolve(&e.source)
            && !ids.insert(&a.id)
        {
            return Err(at(
                error(
                    ErrorKind::Unsupported,
                    "Duplicate swap-area rows are outside this activity",
                ),
                e.line,
            ));
        }
    }
    let mut outcomes = vec![];
    for e in entries {
        if !e.automatic {
            outcomes.push(Outcome::Skipped {
                id: inventory
                    .resolve(&e.source)
                    .map(|a| a.id.clone())
                    .unwrap_or_else(|_| format!("line{}", e.line)),
            });
            continue;
        }
        match activate(inventory, active, &e.source, e.priority, true) {
            Ok(true) => outcomes.push(Outcome::Activated {
                id: inventory.resolve(&e.source)?.id.clone(),
            }),
            Ok(false) => outcomes.push(Outcome::Skipped {
                id: inventory.resolve(&e.source)?.id.clone(),
            }),
            Err(e2) => outcomes.push(Outcome::Failed {
                line: e.line,
                reason: e2.to_string(),
            }),
        }
    }
    Ok(outcomes)
}
pub fn deactivate(
    inventory: &Inventory,
    active: &mut BTreeMap<String, u16>,
    source: &Source,
) -> Result<()> {
    inventory.validate_active(active)?;
    let area = inventory.resolve(source)?;
    if active.remove(&area.id).is_none() {
        return Err(error(
            ErrorKind::NotMounted,
            "The supplied swap area is not active",
        ));
    }
    Ok(())
}
pub fn listing(inventory: &Inventory, active: &BTreeMap<String, u16>) -> Result<String> {
    inventory.validate_active(active)?;
    let mut out = String::from("NAME TYPE SIZE USED PRIO\n");
    for (id, pri) in active {
        let a = inventory.areas.iter().find(|a| a.id == *id).unwrap();
        out.push_str(&format!(
            "{} partition {} 0 {}\n",
            a.device, a.size_bytes, pri
        ));
    }
    Ok(out)
}
