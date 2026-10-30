//! Native virtual disk editing over the pure preparation-only core.
//! Drafts, applied configuration, practice and source authority remain distinct.
use crate::{
    cron_workspace as bounded, partition_fs as core,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub mod binding;
pub const HELP: &str = "Virtual disk preparation only: inspect supplied disks, create an empty GPT, add a non-overlapping named partition, then explicitly initialize blank content as ext4 or swap. Draft fields do not change applied state. No delete/resize/overwrite, MBR/logical partitions, mounts, swap activation, host devices or complete utility emulation. Practice can be reset and replayed.";
/// Human-readable native labels; enum/wire representations remain unchanged.
pub fn type_hint_name(value: core::TypeHint) -> &'static str {
    match value {
        core::TypeHint::LinuxData => "Linux data",
        core::TypeHint::LinuxSwap => "Linux swap",
    }
}
pub fn format_name(value: FormatChoice) -> &'static str {
    match value {
        FormatChoice::Ext4 => "ext4",
        FormatChoice::Swap => "swap",
    }
}
pub const MAX_MODEL: usize = 64 * 1024;
pub const MAX_DEFINITION: usize = 192 * 1024;
pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_CONTEXT: usize = 1024 * 1024;
pub const MAX_TARGETS: usize = 8;
const MAX_STEPS: usize = 12;
const MAX_ROWS: usize = 32;

#[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
struct PathText(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
fn disk_paths<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    bounded::sequence::<D, PathText, 4>(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
fn optional_label<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Label(#[serde(deserialize_with = "crate::cron::text::<_,16>")] String);
    Option::<Label>::deserialize(d).map(|v| v.map(|v| v.0))
}
fn optional_uuid<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Uuid(#[serde(deserialize_with = "crate::cron::text::<_,36>")] String);
    Option::<Uuid>::deserialize(d).map(|v| v.map(|v| v.0))
}
fn text_ok(s: &str, n: usize) -> bool {
    s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
}
fn name_ok(s: &str, n: usize) -> bool {
    !s.is_empty()
        && s.len() <= n
        && !s.starts_with(['.', '-'])
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn disk_ok(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 8 && b.starts_with(b"/dev/vd") && b[7].is_ascii_lowercase()
}
fn partition_path(s: &str) -> Result<(&str, u8), String> {
    if !(9..=10).contains(&s.len()) {
        return Err("Choose a bounded canonical virtual partition path".into());
    }
    let disk = s
        .get(..8)
        .filter(|p| disk_ok(p))
        .ok_or("Choose an exact supplied virtual disk partition")?;
    let n = s.get(8..).ok_or("Missing partition number")?;
    if n.is_empty() || n.starts_with('0') || !n.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Use a canonical partition number1–16".into());
    }
    let n = n
        .parse::<u8>()
        .map_err(|_| "Partition number is too large")?;
    if !(1..=16).contains(&n) {
        return Err("Partition number must be1–16".into());
    }
    Ok((disk, n))
}
fn model(s: &core::State) -> Result<(), String> {
    s.validate().map_err(|e| e.to_string())?;
    if !bounded::within(s, MAX_MODEL) {
        return Err(
            "Virtual disk model exceeds the64KiB workspace limit; previous work retained".into(),
        );
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DesiredContent {
    Blank,
    Ext4 {
        #[serde(default, deserialize_with = "optional_label")]
        label: Option<String>,
    },
    Swap {
        #[serde(default, deserialize_with = "optional_label")]
        label: Option<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub name: String,
    #[serde(deserialize_with = "disk_paths")]
    pub eligible_disks: Vec<String>,
    pub type_hint: core::TypeHint,
    pub minimum_bytes: u64,
    pub maximum_bytes: u64,
    pub aligned_start: bool,
    pub content: DesiredContent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Target,MAX_TARGETS>")]
    pub targets: Vec<Target>,
}
impl Plan {
    pub fn validate(&self, initial: &core::State) -> Result<(), String> {
        model(initial)?;
        if self.targets.is_empty() || self.targets.len() > MAX_TARGETS {
            return Err("Need1–8 meaningful partition requirements".into());
        }
        let mut names = BTreeSet::new();
        for t in &self.targets {
            let format_floor = match t.content {
                DesiredContent::Blank => core::SECTOR_BYTES,
                DesiredContent::Ext4 { .. } => 1024 * 1024,
                DesiredContent::Swap { .. } => 10 * core::BLOCK_BYTES,
            };
            let label = match &t.content {
                DesiredContent::Blank => None,
                DesiredContent::Ext4 { label } | DesiredContent::Swap { label } => label.as_ref(),
            };
            if !name_ok(&t.name, 36)
                || !names.insert(&t.name)
                || t.eligible_disks.is_empty()
                || t.eligible_disks.len() > 4
                || t.eligible_disks
                    .iter()
                    .any(|p| !disk_ok(p) || !initial.disks.contains_key(p))
                || t.eligible_disks.iter().collect::<BTreeSet<_>>().len() != t.eligible_disks.len()
                || t.minimum_bytes == 0
                || t.minimum_bytes > t.maximum_bytes
                || t.maximum_bytes < format_floor
                || t.maximum_bytes > core::MAX_SECTORS * core::SECTOR_BYTES
                || !t.minimum_bytes.is_multiple_of(core::SECTOR_BYTES)
                || !t.maximum_bytes.is_multiple_of(core::SECTOR_BYTES)
                || label.is_some_and(|s| {
                    s.len() > 16
                        || !s
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
                })
            {
                return Err(
                    "Invalid named disk, extent, label or duplicate goal requirement".into(),
                );
            }
            let existing = matches(initial, t);
            if existing.len() > 1 {
                return Err("An authored target name is ambiguous across eligible disks".into());
            }
            if let Some((disk, p)) = existing.first() {
                // This preparation core cannot resize, rename or change a hint.
                if p.type_hint != t.type_hint
                    || p.bytes().map_err(|e| e.to_string())? < t.minimum_bytes
                    || p.bytes().map_err(|e| e.to_string())? > t.maximum_bytes
                    || p.bytes().map_err(|e| e.to_string())? < format_floor
                    || (t.aligned_start
                        && !p
                            .start
                            .is_multiple_of(initial.disks[*disk].optimal_alignment))
                    || (p.content != core::Content::Blank
                        && !content_matches(&p.content, &t.content))
                {
                    return Err("A supplied target would need unsupported resize, hint replacement or overwrite".into());
                }
            }
        }
        Ok(())
    }
}
fn matches<'a>(s: &'a core::State, t: &Target) -> Vec<(&'a str, &'a core::Partition)> {
    t.eligible_disks
        .iter()
        .filter_map(|d| s.disks.get_key_value(d))
        .flat_map(|(disk, d)| {
            d.table
                .iter()
                .flat_map(move |table| table.partitions.iter().map(move |p| (disk.as_str(), p)))
        })
        .filter(|(_, p)| p.name == t.name)
        .collect()
}
fn content_matches(actual: &core::Content, want: &DesiredContent) -> bool {
    match (actual, want) {
        (core::Content::Blank, DesiredContent::Blank) => true,
        (core::Content::Ext4 { label, .. }, DesiredContent::Ext4 { label: expected })
        | (core::Content::Swap { label, .. }, DesiredContent::Swap { label: expected }) => {
            expected.as_ref().is_none_or(|v| v == label)
        }
        _ => false,
    }
}
/// Source compatibility, not success. All core-legal new allocations remain
/// visible wrong work; supplied facts/existing data cannot be replaced.
pub fn same_problem(initial: &core::State, current: &core::State) -> bool {
    if model(initial).is_err()
        || model(current).is_err()
        || !initial.environment_matches(current)
        || current.next_uuid < initial.next_uuid
    {
        return false;
    }
    for (path, before) in &initial.disks {
        let after = &current.disks[path];
        if (!before.writable || before.in_use || before.occupied_without_table) && before != after {
            return false;
        }
        if let Some(t) = &before.table {
            let Some(a) = &after.table else {
                return false;
            };
            for p in &t.partitions {
                let Some(q) = a.partitions.iter().find(|q| q.number == p.number) else {
                    return false;
                };
                if p.name != q.name
                    || p.start != q.start
                    || p.end != q.end
                    || p.type_hint != q.type_hint
                {
                    return false;
                }
                if p.content == core::Content::Blank {
                    if matches!(q.content, core::Content::Foreign { .. })
                        || matches!(&q.content, core::Content::Ext4{files,..} if !files.is_empty())
                    {
                        return false;
                    }
                } else if p.content != q.content {
                    return false;
                }
            }
        }
        if let Some(t) = &after.table {
            for p in &t.partitions {
                let old = before
                    .table
                    .as_ref()
                    .is_some_and(|t| t.partitions.iter().any(|q| q.number == p.number));
                if !old
                    && (matches!(p.content, core::Content::Foreign { .. })
                        || matches!(&p.content, core::Content::Ext4{files,..} if !files.is_empty()))
                {
                    return false;
                }
            }
        }
    }
    true
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn content_name(c: &core::Content) -> &'static str {
    match c {
        core::Content::Blank => "unformatted",
        core::Content::Ext4 { .. } => "ext4",
        core::Content::Swap { .. } => "swap",
        core::Content::Foreign { .. } => "supplied foreign data",
    }
}
fn labeled_content(kind: &str, label: Option<&str>) -> String {
    match label {
        Some("") => format!("{kind} with no label"),
        Some(label) => format!("{kind} labeled {label:?}"),
        None => format!("{kind} (any label)"),
    }
}
fn actual_content(c: &core::Content) -> String {
    match c {
        core::Content::Ext4 { label, .. } => labeled_content("ext4", Some(label)),
        core::Content::Swap { label, .. } => labeled_content("swap", Some(label)),
        _ => content_name(c).into(),
    }
}
fn required_content(c: &DesiredContent) -> String {
    match c {
        DesiredContent::Blank => "unformatted content".into(),
        DesiredContent::Ext4 { label } => labeled_content("ext4", label.as_deref()),
        DesiredContent::Swap { label } => labeled_content("swap", label.as_deref()),
    }
}
fn outcomes(initial: &core::State, current: &core::State, plan: &Plan) -> Vec<(bool, String)> {
    if !same_problem(initial, current) || plan.validate(initial).is_err() {
        return vec![(false,"The supplied disk geometry, existing data or task context differs; current success was not accepted".into())];
    }
    let mut rows = Vec::new();
    let mut used_disks = BTreeSet::new();
    for t in &plan.targets {
        let found = matches(current, t);
        if found.len() != 1 {
            rows.push((
                false,
                format!(
                    "{}: {} matching partitions on eligible media; need exactly one",
                    t.name,
                    found.len()
                ),
            ));
            continue;
        }
        let (disk, p) = found[0];
        used_disks.insert(disk);
        let bytes = p.bytes().unwrap();
        let aligned = p
            .start
            .is_multiple_of(current.disks[disk].optimal_alignment);
        let mut issues = Vec::new();
        if p.type_hint != t.type_hint {
            issues.push(format!(
                "partition type hint is {}; need {}",
                type_hint_name(p.type_hint),
                type_hint_name(t.type_hint)
            ));
        }
        if bytes < t.minimum_bytes || bytes > t.maximum_bytes {
            issues.push(format!(
                "capacity {bytes}B; required{}..{}B",
                t.minimum_bytes, t.maximum_bytes
            ));
        }
        if t.aligned_start && !aligned {
            issues.push(format!(
                "start {} is not aligned to {} sectors",
                p.start, current.disks[disk].optimal_alignment
            ));
        }
        if !content_matches(&p.content, &t.content) {
            issues.push(format!(
                "actual content is {}; need {}",
                actual_content(&p.content),
                required_content(&t.content)
            ));
        }
        rows.push((issues.is_empty(),if issues.is_empty(){format!("{} on {disk}{}: current type, {} B extent, alignment and {} content satisfy the task",t.name,p.number,bytes,content_name(&p.content))}else{format!("{} on {disk}{} needs attention: {}",t.name,p.number,issues.join("; "))}));
    }
    let mut protected = true;
    let mut protection_issues = Vec::new();
    for (disk, before) in &initial.disks {
        let after = &current.disks[disk];
        if !used_disks.contains(disk.as_str()) {
            protected &= before == after;
            if before != after {
                protection_issues.push(format!("Unrequested disk {disk} changed"));
            }
            continue;
        }
        if let Some(t) = &after.table {
            for p in &t.partitions {
                let original = before
                    .table
                    .as_ref()
                    .and_then(|t| t.partitions.iter().find(|q| q.number == p.number));
                let is_target = plan
                    .targets
                    .iter()
                    .any(|t| t.name == p.name && t.eligible_disks.iter().any(|d| d == disk));
                if let Some(old) = original {
                    if !is_target {
                        protected &= old == p;
                        if old != p {
                            protection_issues
                                .push(format!("Supplied partition {disk}{} changed", p.number));
                        }
                    }
                } else {
                    protected &= is_target;
                    if !is_target {
                        protection_issues.push(format!(
                            "Unrequested partition {disk}{} ({})",
                            p.number, p.name
                        ));
                    }
                }
            }
        }
    }
    rows.push((
        protected,
        if protected { "Supplied data, unrelated media and all unrequested partition allocations remain unchanged".into() } else { format!("{}. This preparation-only subset cannot delete/undo an allocation; reset the affected practice or lab and replay its intended configuration.", protection_issues.join("; ")) },
    ));
    rows
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatChoice {
    #[default]
    Ext4,
    Swap,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    CreateGpt {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        disk: String,
    },
    AddPartition {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        disk: String,
        #[serde(deserialize_with = "crate::cron::text::<_,36>")]
        name: String,
        start: u64,
        end: u64,
        type_hint: core::TypeHint,
    },
    Format {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        partition: String,
        format: FormatChoice,
        #[serde(deserialize_with = "crate::cron::text::<_,16>")]
        label: String,
        #[serde(default, deserialize_with = "optional_uuid")]
        uuid: Option<String>,
    },
}
impl Operation {
    pub fn shape(&self) -> Result<(), String> {
        match self {
            Self::CreateGpt { disk } => {
                if !disk_ok(disk) {
                    return Err("Choose a supplied virtual disk".into());
                }
            }
            Self::AddPartition { disk, name, .. } => {
                if !disk_ok(disk) || !name_ok(name, 36) {
                    return Err("Use a supplied disk and a bounded ordinary partition name".into());
                }
            }
            Self::Format {
                partition,
                label,
                uuid,
                ..
            } => {
                partition_path(partition)?;
                if !text_ok(label, 16) || uuid.as_ref().is_some_and(|v| !text_ok(v, 36)) {
                    return Err("Formatting fields exceed their byte or single-line bounds".into());
                }
            }
        }
        Ok(())
    }
    fn core_action(&self) -> core::Action {
        match self {
            Self::CreateGpt { disk } => core::Action::CreateGpt { disk: disk.clone() },
            Self::AddPartition {
                disk,
                name,
                start,
                end,
                type_hint,
            } => core::Action::AddPartition {
                disk: disk.clone(),
                name: name.clone(),
                start: *start,
                end: *end,
                type_hint: *type_hint,
            },
            Self::Format {
                partition,
                format,
                label,
                uuid,
            } => match format {
                FormatChoice::Ext4 => core::Action::FormatExt4 {
                    partition: partition.clone(),
                    label: label.clone(),
                    uuid: uuid.clone(),
                },
                FormatChoice::Swap => core::Action::InitializeSwap {
                    partition: partition.clone(),
                    label: label.clone(),
                    uuid: uuid.clone(),
                },
            },
        }
    }
    pub fn description(&self) -> String {
        if self.shape().is_err() {
            return "Invalid native preparation action; nothing applied".into();
        }
        match self {
            Self::CreateGpt { disk } => format!("Create GPT on {disk}"),
            Self::AddPartition {
                disk,
                name,
                start,
                end,
                type_hint,
            } => {
                format!(
                    "Add {name} on {disk}, inclusive sectors {start}..{end}, hint {}",
                    type_hint_name(*type_hint)
                )
            }
            Self::Format {
                partition,
                format,
                label,
                ..
            } => format!(
                "Initialize {partition} as {}, label {}",
                format_name(*format),
                if label.is_empty() {
                    "(no label)"
                } else {
                    label
                }
            ),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkedStep {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub description: String,
    pub fixture: core::State,
    pub plan: Plan,
    #[serde(deserialize_with = "bounded::sequence::<_,WorkedStep,MAX_STEPS>")]
    pub worked: Vec<WorkedStep>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    pub experiment: Operation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    pub plan: Plan,
    /// Authoring-only proof and explicitly requested native worked approach.
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,32>")]
    pub reference: Vec<Operation>,
    pub lesson: Lesson,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub initial: core::State,
    pub plan: Plan,
    pub example_initial: core::State,
    pub example_plan: Plan,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub selected_partition: Option<u8>,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub start: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub end: String,
    pub type_hint: core::TypeHint,
    pub format: FormatChoice,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub uuid: String,
}
impl Draft {
    fn new(d: &core::Disk) -> Self {
        Self {
            selected_partition: d
                .table
                .as_ref()
                .and_then(|t| t.partitions.first())
                .map(|p| p.number),
            name: String::new(),
            start: String::new(),
            end: String::new(),
            type_hint: core::TypeHint::LinuxData,
            format: FormatChoice::Ext4,
            label: String::new(),
            uuid: String::new(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self
            .selected_partition
            .is_some_and(|n| !(1..=16).contains(&n))
            || !text_ok(&self.name, 36)
            || !text_ok(&self.start, 32)
            || !text_ok(&self.end, 32)
            || !text_ok(&self.label, 16)
            || !text_ok(&self.uuid, 36)
        {
            return Err(
                "Editor values exceed their single-line byte limits; nothing was applied".into(),
            );
        }
        Ok(())
    }
    pub fn add_operation(&self, disk: &str) -> Result<Operation, String> {
        self.validate()?;
        if !disk_ok(disk) {
            return Err("Choose an exact supplied virtual disk".into());
        }
        let number = |s: &str| {
            let s = s.trim();
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Use unsigned whole sector numbers in Start and End".to_owned());
            }
            s.parse::<u64>()
                .map_err(|_| "Sector number is too large".into())
        };
        let op = Operation::AddPartition {
            disk: disk.into(),
            name: self.name.clone(),
            start: number(&self.start)?,
            end: number(&self.end)?,
            type_hint: self.type_hint,
        };
        op.shape()?;
        Ok(op)
    }
    pub fn format_operation(&self, disk: &str) -> Result<Operation, String> {
        self.validate()?;
        if !disk_ok(disk) {
            return Err("Choose an exact supplied virtual disk".into());
        }
        let n = self
            .selected_partition
            .ok_or("Select a partition to initialize")?;
        let op = Operation::Format {
            partition: format!("{disk}{n}"),
            format: self.format,
            label: self.label.clone(),
            uuid: (!self.uuid.is_empty()).then(|| self.uuid.clone()),
        };
        op.shape()?;
        Ok(op)
    }
}
fn draft_map<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Draft>, D::Error> {
    struct M;
    impl<'de> serde::de::Visitor<'de> for M {
        type Value = BTreeMap<String, Draft>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to4 distinct per-disk drafts")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(PathText(p)) = a.next_key::<PathText>()? {
                if out.len() >= 4 || out.contains_key(&p) {
                    return Err(serde::de::Error::custom(
                        "Disk draft count/duplicate before value",
                    ));
                }
                out.insert(p, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected: String,
    #[serde(deserialize_with = "draft_map")]
    pub drafts: BTreeMap<String, Draft>,
}
impl Editor {
    fn new(model: &core::State) -> Result<Self, String> {
        model.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            selected: model
                .disks
                .keys()
                .next()
                .ok_or("Missing supplied disks")?
                .clone(),
            drafts: model
                .disks
                .iter()
                .map(|(p, d)| (p.clone(), Draft::new(d)))
                .collect(),
        })
    }
    fn validate(&self, model: &core::State) -> Result<(), String> {
        if !model.disks.contains_key(&self.selected) || self.drafts.keys().ne(model.disks.keys()) {
            return Err(
                "Selected disk or inactive draft inventory differs from supplied media".into(),
            );
        }
        for d in self.drafts.values() {
            d.validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub output: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub origin: Origin,
    pub current: core::State,
    pub example: core::State,
    pub assignment_editor: Editor,
    pub example_editor: Editor,
    pub view: View,
    pub selected_example: usize,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,MAX_ROWS>")]
    pub transcript: Vec<Row>,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,MAX_ROWS>")]
    pub example_transcript: Vec<Row>,
    pub checked: bool,
    pub example_checked: bool,
}
impl Definition {
    /// Structural source admission only. Never executes examples on idle/save.
    pub fn shape(&self) -> Result<(), String> {
        model(&self.initial)?;
        model(&self.lesson.fixture)?;
        self.plan.validate(&self.initial)?;
        self.lesson.plan.validate(&self.lesson.fixture)?;
        let l = &self.lesson;
        if self.reference.is_empty() || self.reference.len() > 32 {
            return Err("Need1–32 bounded typed native reference actions".into());
        }
        for op in &self.reference {
            op.shape()?;
        }
        if !bounded::within(self, MAX_DEFINITION)
            || l.concepts.is_empty()
            || l.concepts.len() > 6
            || l.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || !text_ok(&c.title, 100)
                    || c.text.trim().is_empty()
                    || !text_ok(&c.text, 1200)
            })
            || [&l.prerequisite_note, &l.description, &l.try_change]
                .iter()
                .any(|s| s.trim().is_empty() || !text_ok(s, 1600))
            || l.worked.is_empty()
            || l.worked.len() > MAX_STEPS
            || l.worked
                .iter()
                .any(|s| s.explanation.trim().is_empty() || !text_ok(&s.explanation, 800))
            || self
                .initial
                .disks
                .keys()
                .any(|p| l.fixture.disks.contains_key(p))
        {
            return Err(
                "Invalid, oversized or nonseparate disk assignment/practice definition".into(),
            );
        }
        for step in &l.worked {
            step.operation.shape()?;
        }
        l.experiment.shape()?;
        Ok(())
    }
    /// Authoring gate. Native insertion only fills fields and never runs steps.
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if all(&outcomes(&self.initial, &self.initial, &self.plan)) {
            return Err("Assignment must require real disk changes".into());
        }
        let mut assignment = self.initial.clone();
        for op in &self.reference {
            assignment
                .apply(op.core_action())
                .map_err(|e| e.to_string())?;
            model(&assignment)?;
        }
        if !all(&outcomes(&self.initial, &assignment, &self.plan)) {
            return Err(
                "Native reference does not reach every assignment/protection requirement".into(),
            );
        }
        let l = &self.lesson;
        let mut example = l.fixture.clone();
        if all(&outcomes(&l.fixture, &example, &l.plan)) {
            return Err("Separate practice must initially need changes".into());
        }
        for step in &l.worked {
            example
                .apply(step.operation.core_action())
                .map_err(|e| e.to_string())?;
            model(&example)?;
        }
        if !all(&outcomes(&l.fixture, &example, &l.plan)) {
            return Err("Worked native steps do not meet practice outcomes".into());
        }
        let before = example.clone();
        example
            .apply(l.experiment.core_action())
            .map_err(|e| e.to_string())?;
        if example == before || all(&outcomes(&l.fixture, &example, &l.plan)) {
            return Err("Practice experiment must expose a changed incomplete disk result".into());
        }
        // No deletion/reformat capability is invented. Reset then manual replay
        // is the documented recoverable practice path.
        let mut repaired = l.fixture.clone();
        for step in &l.worked {
            repaired
                .apply(step.operation.core_action())
                .map_err(|e| e.to_string())?;
        }
        if !all(&outcomes(&l.fixture, &repaired, &l.plan)) {
            return Err("Practice reset/replay does not repair its outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.shape()?;
        s.validate()?;
        if s.origin.initial != self.initial
            || s.origin.plan != self.plan
            || s.origin.example_initial != self.lesson.fixture
            || s.origin.example_plan != self.lesson.plan
            || s.selected_example >= self.lesson.worked.len()
        {
            return Err("Disk session belongs to another authored assignment or practice".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if self.bound(s).is_err() {
            return vec![(
                false,
                "The disk source/session context differs; no current success accepted".into(),
            )];
        }
        outcomes(&self.initial, &s.current, &self.plan)
    }
    pub fn practice_checks(&self, s: &Session) -> Vec<(bool, String)> {
        if self.bound(s).is_err() {
            return vec![(false, "Separate practice context differs".into())];
        }
        outcomes(&self.lesson.fixture, &s.example, &self.lesson.plan)
    }
    pub fn fresh(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.current == self.initial
            && s.example == self.lesson.fixture
            && s.view == View::Introduction
            && s.selected_example == 0
            && !s.checked
            && !s.example_checked
            && s.transcript.is_empty()
            && s.example_transcript.is_empty()
            && fresh_editor(&s.assignment_editor, &self.initial)
            && fresh_editor(&s.example_editor, &self.lesson.fixture)
    }
}
fn fresh_editor(e: &Editor, model: &core::State) -> bool {
    model.disks.keys().next() == Some(&e.selected)
        && e.validate(model).is_ok()
        && e.drafts.iter().all(|(path, d)| {
            d.selected_partition
                == model.disks[path]
                    .table
                    .as_ref()
                    .and_then(|t| t.partitions.first())
                    .map(|p| p.number)
                && d.name.is_empty()
                && d.start.is_empty()
                && d.end.is_empty()
                && d.type_hint == core::TypeHint::LinuxData
                && d.format == FormatChoice::Ext4
                && d.label.is_empty()
                && d.uuid.is_empty()
        })
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.shape()?;
        let s = Self {
            version: 1,
            origin: Origin {
                initial: d.initial.clone(),
                plan: d.plan.clone(),
                example_initial: d.lesson.fixture.clone(),
                example_plan: d.lesson.plan.clone(),
            },
            current: d.initial.clone(),
            example: d.lesson.fixture.clone(),
            assignment_editor: Editor::new(&d.initial)?,
            example_editor: Editor::new(&d.lesson.fixture)?,
            view: View::Introduction,
            selected_example: 0,
            transcript: Vec::new(),
            example_transcript: Vec::new(),
            checked: false,
            example_checked: false,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !same_problem(&self.origin.initial, &self.current)
            || !same_problem(&self.origin.example_initial, &self.example)
            || self.selected_example >= MAX_STEPS
            || self.transcript.len() > MAX_ROWS
            || self.example_transcript.len() > MAX_ROWS
            || self
                .transcript
                .iter()
                .chain(&self.example_transcript)
                .any(|r| !text_ok(&r.input, 512) || r.output.len() > 512 || r.output.contains('\0'))
            || !bounded::within(self, MAX_SESSION)
        {
            return Err("Invalid or oversized disk session; previous work retained".into());
        }
        self.origin.plan.validate(&self.origin.initial)?;
        self.origin
            .example_plan
            .validate(&self.origin.example_initial)?;
        self.assignment_editor.validate(&self.current)?;
        self.example_editor.validate(&self.example)?;
        Ok(())
    }
    pub fn selected_editor(&self, practice: bool) -> &Editor {
        if practice {
            &self.example_editor
        } else {
            &self.assignment_editor
        }
    }
    pub fn selected_model(&self, practice: bool) -> &core::State {
        if practice {
            &self.example
        } else {
            &self.current
        }
    }
    pub fn set_view(&mut self, d: &Definition, view: View) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.view = view;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn select_disk(
        &mut self,
        d: &Definition,
        path: &str,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        if !self.selected_model(practice).disks.contains_key(path) {
            return Err("No such supplied disk".into());
        }
        let mut next = self.clone();
        let e = if practice {
            &mut next.example_editor
        } else {
            &mut next.assignment_editor
        };
        e.selected = path.into();
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn set_draft(
        &mut self,
        d: &Definition,
        path: &str,
        draft: Draft,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        draft.validate()?;
        if !self.selected_model(practice).disks.contains_key(path) {
            return Err("No such supplied disk draft".into());
        }
        let mut next = self.clone();
        let e = if practice {
            &mut next.example_editor
        } else {
            &mut next.assignment_editor
        };
        e.drafts.insert(path.into(), draft);
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn select_example(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        if index >= d.lesson.worked.len() {
            return Err("No such worked native step".into());
        }
        let mut next = self.clone();
        next.selected_example = index;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn insert(&mut self, d: &Definition, index: usize) -> Result<(), String> {
        d.bound(self)?;
        let step = d
            .lesson
            .worked
            .get(index)
            .ok_or("No such worked native step")?;
        step.operation.shape()?;
        let mut next = self.clone();
        next.selected_example = index;
        let e = &mut next.example_editor;
        match &step.operation {
            Operation::CreateGpt { disk } => {
                if !e.drafts.contains_key(disk) {
                    return Err("Worked step uses another disk inventory".into());
                }
                e.selected = disk.clone();
            }
            Operation::AddPartition {
                disk,
                name,
                start,
                end,
                type_hint,
            } => {
                let draft = e
                    .drafts
                    .get_mut(disk)
                    .ok_or("Worked step uses another disk inventory")?;
                e.selected = disk.clone();
                draft.name = name.clone();
                draft.start = start.to_string();
                draft.end = end.to_string();
                draft.type_hint = *type_hint;
            }
            Operation::Format {
                partition,
                format,
                label,
                uuid,
            } => {
                let (disk, n) = partition_path(partition)?;
                let draft = e
                    .drafts
                    .get_mut(disk)
                    .ok_or("Worked step uses another disk inventory")?;
                e.selected = disk.into();
                draft.selected_partition = Some(n);
                draft.format = *format;
                draft.label = label.clone();
                draft.uuid = uuid.clone().unwrap_or_default();
            }
        }
        next.example_checked = false;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn apply(
        &mut self,
        d: &Definition,
        operation: &Operation,
        practice: bool,
    ) -> Result<String, String> {
        d.bound(self)?;
        operation.shape()?;
        let description = operation.description();
        if description.len() > 512 {
            return Err("Native action summary exceeds its bound".into());
        }
        let mut next = self.clone();
        let model = if practice {
            &mut next.example
        } else {
            &mut next.current
        };
        let output = model
            .apply(operation.core_action())
            .map_err(|e| e.to_string())?
            .stdout;
        if output.len() > 512 {
            return Err("Native action result exceeds its bound".into());
        }
        let editor = if practice {
            &mut next.example_editor
        } else {
            &mut next.assignment_editor
        };
        if let Operation::AddPartition {
            disk,
            name,
            start,
            end,
            ..
        } = operation
        {
            // The actual allocated slot is derived from the new table, never a
            // saved success flag or guessed fixed partition number.
            let number = model.disks[disk]
                .table
                .as_ref()
                .unwrap()
                .partitions
                .iter()
                .find(|p| p.name == *name && p.start == *start && p.end == *end)
                .map(|p| p.number);
            editor.drafts.get_mut(disk).unwrap().selected_partition = number;
        }
        let rows = if practice {
            &mut next.example_transcript
        } else {
            &mut next.transcript
        };
        if rows.len() == MAX_ROWS {
            rows.remove(0);
        }
        rows.push(Row {
            input: description,
            output: output.clone(),
        });
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(output)
    }
    pub fn check(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        if practice {
            self.example_checked = true;
        } else {
            self.checked = true;
        }
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        next.example = d.lesson.fixture.clone();
        next.example_editor = Editor::new(&d.lesson.fixture)?;
        next.example_transcript.clear();
        next.example_checked = false;
        next.selected_example = 0;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
}
pub use binding::{context, metadata, source};
