//! Typed practice/assignment stimuli under proof; not a native admission route.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub record: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub contents: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub explanation: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Change,16>")]
    pub writes: Vec<Change>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub plan: snapshot::Plan,
    #[serde(deserialize_with = "bounded::sequence::<_,Update,4>")]
    pub updates: Vec<Update>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    ApplyUpdate {
        index: usize,
    },
    CopyRecord {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        origin: String,
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        snapshot: String,
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        record: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,24>")]
    pub reference: Vec<Operation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Practice {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,16>")]
    pub worked: Vec<Step>,
    pub experiment: Operation,
    #[serde(default, deserialize_with = "bounded::sequence::<_,Operation,16>")]
    pub repair: Vec<Operation>,
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= max
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
impl Context {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.plan.validate(initial)?;
        if self.updates.len() > 4 {
            return Err("At most four supplied application updates".into());
        }
        for update in &self.updates {
            if !prose(&update.label, 160)
                || !prose(&update.explanation, 1600)
                || update.writes.is_empty()
                || update.writes.len() > 16
            {
                return Err(
                    "Each update needs a label, explanation and1–16fixed record writes".into(),
                );
            }
            let mut keys = BTreeSet::new();
            for write in &update.writes {
                let bank = initial
                    .snapshots
                    .as_ref()
                    .and_then(|m| m.origins.get(&write.origin))
                    .ok_or("Application update names an unknown supplied origin")?;
                if !keys.insert((&write.origin, &write.record))
                    || bank
                        .validate_record_value(&write.record, &write.contents)
                        .is_err()
                {
                    return Err("Application updates need distinct supplied records and supported fixed-size text".into());
                }
            }
        }
        Ok(())
    }
    pub fn checks(&self, initial: &lvm::State, current: &lvm::State) -> Vec<(bool, String)> {
        snapshot::outcomes(initial, current, &self.plan)
    }
}
impl Operation {
    pub fn shape(&self, context: &Context, initial: &lvm::State) -> Result<(), String> {
        match self {
            Self::Command { text } => {
                lvm::snapshot::validate_command(text).map_err(|e| e.to_string())
            }
            Self::ApplyUpdate { index } if *index < context.updates.len() => Ok(()),
            Self::ApplyUpdate { .. } => Err("Select a supplied application update".into()),
            Self::CopyRecord {
                origin,
                snapshot,
                record,
            } => {
                let b = initial
                    .snapshots
                    .as_ref()
                    .and_then(|m| m.origins.get(origin))
                    .ok_or("Copy needs a supplied raw-record origin")?;
                if !lvm::provision::valid_volume_name(snapshot) || b.current(record).is_err() {
                    return Err("Copy needs a supported snapshot name and supplied record".into());
                }
                Ok(())
            }
        }
    }
    pub fn description(&self, context: &Context) -> String {
        match self {
            Self::Command { text } => text.clone(),
            Self::ApplyUpdate { index } => format!(
                "Simulator application update · {}",
                context
                    .updates
                    .get(*index)
                    .map_or("Unknown update", |u| u.label.as_str())
            ),
            Self::CopyRecord {
                origin,
                snapshot,
                record,
            } => format!("Simulator record copy · {origin}: {snapshot} to current {record}"),
        }
    }
    /// This does not award credit. The future binding must validate source/context
    /// and keep assignment and example sessions separate before publication.
    pub fn apply(
        &self,
        context: &Context,
        initial: &lvm::State,
        current: &mut lvm::State,
    ) -> Result<String, String> {
        context.shape(initial)?;
        self.shape(context, initial)?;
        model(current)?;
        if !snapshot::same_context(initial, current) {
            return Err("This state no longer belongs to its supplied snapshot task".into());
        }
        let mut next = current.clone();
        let output = match self {
            Self::Command { text } => next.execute(text).map_err(|e| e.to_string())?,
            Self::ApplyUpdate { index } => {
                let update = &context.updates[*index];
                let mut invalidated = BTreeSet::new();
                for write in &update.writes {
                    let result = lvm::snapshot::write_record(
                        &mut next,
                        &write.origin,
                        &write.record,
                        &write.contents,
                    )
                    .map_err(|e| e.to_string())?;
                    let group = write.origin.split_once('/').unwrap().0;
                    invalidated.extend(
                        result
                            .invalidated
                            .into_iter()
                            .map(|name| format!("{group}/{name}")),
                    );
                }
                if invalidated.is_empty() {
                    format!(
                        "Simulated application update applied: {}. Inspect current and saved records before checking.",
                        update.label
                    )
                } else {
                    format!(
                        "Simulated application update applied: {}. These snapshots became invalid: {}. The origin write succeeded; their old data is unavailable.",
                        update.label,
                        invalidated.into_iter().collect::<Vec<_>>().join(", ")
                    )
                }
            }
            Self::CopyRecord {
                origin,
                snapshot,
                record,
            } => {
                let result = lvm::snapshot::restore_record(&mut next, origin, snapshot, record)
                    .map_err(|e| e.to_string())?;
                format!(
                    "Simulated record copy applied: {origin} {record} from {snapshot}. {}",
                    if result.invalidated.is_empty() {
                        "Inspect the current record."
                    } else {
                        "Another snapshot became invalid while recording this write; inspect its status."
                    }
                )
            }
        };
        model(&next)?;
        // Protection may become false after a permitted deliberate practice mistake;
        // the next explicit repair still needs the same immutable source identity.
        *current = next;
        Ok(output)
    }
}
fn all(rows: &[(bool, String)]) -> bool {
    rows.iter().all(|r| r.0)
}
impl Task {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.context.shape(initial)?;
        if self.reference.is_empty() || self.reference.len() > 24 {
            return Err("Snapshot assignment needs a bounded reference solution".into());
        }
        for op in &self.reference {
            op.shape(&self.context, initial)?;
        }
        Ok(())
    }
    pub fn validate(&self, initial: &lvm::State) -> Result<(), String> {
        self.shape(initial)?;
        if all(&self.context.checks(initial, initial)) {
            return Err("Assignment must require real changes".into());
        }
        let mut state = initial.clone();
        for op in &self.reference {
            op.apply(&self.context, initial, &mut state)?;
        }
        if !all(&self.context.checks(initial, &state)) {
            return Err("Reference solution does not reach every current snapshot outcome".into());
        }
        Ok(())
    }
}
impl Practice {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.context.shape(initial)?;
        if self.worked.is_empty() || self.worked.len() > 16 || self.repair.len() > 16 {
            return Err("Practice needs bounded annotated steps".into());
        }
        for step in &self.worked {
            if !prose(&step.explanation, 800) {
                return Err("Every practice operation needs an explanation".into());
            }
            step.operation.shape(&self.context, initial)?;
        }
        self.experiment.shape(&self.context, initial)?;
        for op in &self.repair {
            op.shape(&self.context, initial)?;
        }
        Ok(())
    }
    pub fn validate(&self, initial: &lvm::State) -> Result<(), String> {
        self.shape(initial)?;
        if all(&self.context.checks(initial, initial)) {
            return Err("Practice must start unsolved".into());
        }
        let mut state = initial.clone();
        for step in &self.worked {
            step.operation.apply(&self.context, initial, &mut state)?;
        }
        if !all(&self.context.checks(initial, &state)) {
            return Err("Worked practice misses its actual outcomes".into());
        }
        let before = state.clone();
        self.experiment.apply(&self.context, initial, &mut state)?;
        if state == before || all(&self.context.checks(initial, &state)) {
            return Err(
                "Practice exploration must change state and expose a real failed outcome".into(),
            );
        }
        if !self.repair.is_empty() {
            for op in &self.repair {
                op.apply(&self.context, initial, &mut state)?;
            }
            if !all(&self.context.checks(initial, &state)) {
                return Err("Practice repair does not restore its outcomes".into());
            }
        }
        Ok(())
    }
}

/// Internal serialization for native actions and declarative reachability only.
/// This prefix is not a Linux command and never reaches a host process.
pub const PREFIX: &str = "simulated-lvm-record ";
pub fn encode(op: &Operation) -> Result<String, String> {
    let text = match op {
        Operation::Command { text } => text.clone(),
        _ => format!(
            "{PREFIX}{}",
            serde_json::to_string(op).map_err(|e| e.to_string())?
        ),
    };
    if text.len() > 2048 || text.contains(['\0', '\n', '\r']) {
        return Err("Simulator action exceeds its bounded packet".into());
    }
    Ok(text)
}
pub fn decode(input: &str) -> Result<Operation, String> {
    if input.len() > 2048 || input.contains(['\0', '\n', '\r']) {
        return Err("Simulator action exceeds its bounded packet".into());
    }
    if let Some(json) = input.strip_prefix(PREFIX) {
        serde_json::from_str(json).map_err(|e| format!("Invalid simulator action: {e}"))
    } else {
        Ok(Operation::Command { text: input.into() })
    }
}
pub(super) fn session_context(initial: &lvm::State, current: &lvm::State) -> bool {
    if initial.snapshots.is_some() {
        snapshot::same_context(initial, current)
    } else {
        super::same_problem(initial, current)
    }
}
pub(super) fn definition_shape(d: &Definition) -> Result<(), String> {
    let (Some(task), Some(practice)) = (&d.snapshot_task, &d.lesson.snapshot_practice) else {
        return Err(
            "Snapshot tasks are not admitted without explicit typed assignment and separate guided practice".into(),
        );
    };
    let l = &d.lesson;
    if d.initial.snapshots.is_none()
        || l.fixture.snapshots.is_none()
        || !d.targets.is_empty()
        || !l.targets.is_empty()
        || d.plan.is_some()
        || l.plan.is_some()
        || d.retirement.is_some()
        || l.retirement.is_some()
        || d.availability.is_some()
        || l.availability.is_some()
        || d.renaming.is_some()
        || l.renaming.is_some()
        || !l.worked.is_empty()
        || !l.experiment.is_empty()
        || !l.repair.is_empty()
        || !bounded::within(d, MAX_DEFINITION)
        || l.concepts.is_empty()
        || l.concepts.len() > 6
        || [&l.prerequisite_note, &l.description, &l.try_change]
            .iter()
            .any(|s| !prose(s, 1600))
        || l.concepts
            .iter()
            .any(|c| !prose(&c.title, 100) || !prose(&c.text, 1200))
        || d.initial
            .devices
            .keys()
            .any(|p| l.fixture.devices.contains_key(p))
        || d.initial
            .groups
            .keys()
            .any(|g| l.fixture.groups.contains_key(g))
    {
        return Err("Snapshot teaching needs bounded concepts, disjoint assignment/practice storage and no mixed legacy goals".into());
    }
    task.shape(&d.initial)?;
    practice.shape(&l.fixture)
}
impl Definition {
    pub fn snapshot_context(&self, practice: bool) -> Option<&Context> {
        if practice {
            self.lesson.snapshot_practice.as_ref().map(|p| &p.context)
        } else {
            self.snapshot_task.as_ref().map(|t| &t.context)
        }
    }
    pub fn example_steps(&self) -> Vec<Command> {
        if let Some(p)=&self.lesson.mount_practice {return p.worked.iter().map(|s|Command{text:s.operation.description(),explanation:s.explanation.clone()}).collect();}
        if let Some(p) = &self.lesson.config_scan_practice {
            p.worked.iter().map(|s|Command{text:s.operation.label().into(),explanation:s.explanation.clone()}).collect()
        } else if let Some(p) = &self.lesson.device_reference_practice {
            p.worked
                .iter()
                .map(|s| Command {
                    text: s.operation.description(&p.context),
                    explanation: s.explanation.clone(),
                })
                .collect()
        } else if let Some(p) = &self.lesson.snapshot_practice {
            p.worked
                .iter()
                .map(|s| Command {
                    text: s.operation.description(&p.context),
                    explanation: s.explanation.clone(),
                })
                .collect()
        } else {
            self.lesson.worked.clone()
        }
    }
}
impl Session {
    pub fn apply_snapshot(
        &mut self,
        d: &Definition,
        op: &Operation,
        practice: bool,
    ) -> Result<String, String> {
        // execute performs the complete source-bound transaction, transcript and
        // current-feedback invalidation used by ordinary terminal commands.
        self.execute(d, &encode(op)?, practice)
    }
}
