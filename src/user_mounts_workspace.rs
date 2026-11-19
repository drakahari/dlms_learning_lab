//! Caller-aware storage activity. Every live fact is derived from bounded actions.
use crate::{removable_mount::Environment, script_teaching::View, storage, user_mounts as model};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const REVISION: u32 = 1;
pub const MAX_HISTORY: usize = 48;
pub const MAX_HISTORY_TEXT: usize = 65_536;
pub const MAX_SESSION: usize = 2_097_152;
pub const PREFIX: &str = "USER MOUNT ";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn document(s: &str) -> bool {
    s.len() <= storage::MAX_CONFIG_BYTES
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
fn fits<T: Serialize>(v: &T, limit: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|raw| raw.len() <= limit)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Save {
        #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
        configuration: String,
    },
    Mount {
        actor: model::Actor,
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        target: String,
    },
    Unmount {
        actor: model::Actor,
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        target: String,
    },
}
impl Action {
    fn validate(&self, env: &Environment) -> Result<(), String> {
        match self {
            Self::Save { configuration } if document(configuration) => Ok(()),
            Self::Mount { target, .. } | Self::Unmount { target, .. }
                if env.inventory.mountpoints().contains(target) =>
            {
                Ok(())
            }
            _ => Err(
                "Use the supplied mountpoints and at most16KiB of supported configuration".into(),
            ),
        }
    }
    pub fn perform(
        &self,
        env: &Environment,
        live: &mut model::Live,
    ) -> Result<Effect, model::Error> {
        match self {
            Self::Save { configuration } => {
                let p = model::Policy::parse(configuration)?;
                live.save_policy(&p)?;
                Ok(Effect::Saved)
            }
            Self::Mount { actor, target } => live.mount_as(env, *actor, target).map(|o| match o {
                storage::MountOutcome::Mounted => Effect::Mounted,
                storage::MountOutcome::AlreadyMounted => Effect::AlreadyMounted,
            }),
            Self::Unmount { actor, target } => live
                .unmount_as(env, *actor, target)
                .map(|_| Effect::Unmounted),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Policy {
        target: String,
        volume: String,
        read_only: bool,
        automatic: bool,
        optional: bool,
        permission: model::Permission,
        flags: model::Flags,
    },
    Mounted {
        target: String,
        volume: String,
        read_only: bool,
        mounter: model::Actor,
        flags: model::Flags,
    },
    NotMounted {
        target: String,
    },
    TargetsOnly {
        targets: Vec<String>,
    },
}
fn permission(p: model::Permission) -> &'static str {
    match p {
        model::Permission::RootOnly => "administrator only",
        model::Permission::MounterOnly => {
            "ordinary mounting; only the original mounter may unmount"
        }
        model::Permission::AnyUser => "shared mounting and unmounting",
    }
}
fn flags(f: model::Flags) -> String {
    format!(
        "noexec {}; nosuid {}; nodev {}",
        if f.noexec { "set" } else { "clear" },
        if f.nosuid { "set" } else { "clear" },
        if f.nodev { "set" } else { "clear" }
    )
}
impl Requirement {
    fn valid(&self, env: &Environment) -> bool {
        match self {
            Self::Policy { target, volume, .. } | Self::Mounted { target, volume, .. } => {
                env.inventory.mountpoints().contains(target)
                    && env.inventory.volumes().contains_key(volume)
            }
            Self::NotMounted { target } => env.inventory.mountpoints().contains(target),
            Self::TargetsOnly { targets } => {
                !targets.is_empty()
                    && targets.len() <= 8
                    && targets.iter().collect::<BTreeSet<_>>().len() == targets.len()
                    && targets
                        .iter()
                        .all(|t| env.inventory.mountpoints().contains(t))
            }
        }
    }
    fn check(&self, env: &Environment, live: &model::Live) -> (bool, String) {
        match self {
            Self::Policy {
                target,
                volume,
                read_only,
                automatic,
                optional,
                permission: wanted,
                flags: expected,
            } => {
                let parsed = model::Policy::parse(live.configuration());
                let Some(r) = parsed.as_ref().ok().and_then(|p| p.rule(target)) else {
                    return (
                        false,
                        format!(
                            "{target}: no supported saved entry. Review the line and save again."
                        ),
                    );
                };
                let actual_volume = env.inventory.resolve(&r.entry.mount.source).ok();
                let ok = actual_volume == Some(volume.as_str())
                    && r.entry.mount.read_only == *read_only
                    && r.entry.mount.automatic == *automatic
                    && (!*automatic || r.entry.optional == *optional)
                    && r.permission == *wanted
                    && r.flags == *expected;
                (
                    ok,
                    if ok {
                        format!("{target}: saved source and policy match.")
                    } else if actual_volume != Some(volume.as_str()) {
                        format!(
                            "{target}: this source does not select the required {volume} filesystem. Compare its UUID with the supplied media."
                        )
                    } else if r.permission != *wanted {
                        format!(
                            "{target}: expected {}; current policy is {}.",
                            permission(*wanted),
                            permission(r.permission)
                        )
                    } else if r.flags != *expected {
                        format!(
                            "{target}: expected {}; saved policy has {}. Later options can override earlier restrictions.",
                            flags(*expected),
                            flags(r.flags)
                        )
                    } else {
                        format!(
                            "{target}: check read-only/read-write, automatic/manual and required/optional settings. Keep the stated storage policy."
                        )
                    },
                )
            }
            Self::Mounted {
                target,
                volume,
                read_only,
                mounter,
                flags: expected,
            } => {
                let mount = live.state().live().get(target);
                let grant = live.ledger().get(target);
                let ok = mount
                    == Some(&storage::Mount {
                        volume: volume.clone(),
                        read_only: *read_only,
                    })
                    && grant.is_some_and(|g| {
                        g.volume == *volume
                            && g.mounter == Some(*mounter)
                            && g.flags == Some(*expected)
                    });
                (
                    ok,
                    if ok {
                        format!("{target}: the actual mount, caller and applied options match.")
                    } else if mount.is_none() {
                        format!(
                            "{target}: not mounted. Save the intended policy, select the required operator and choose Mount; saving alone does not attach the filesystem."
                        )
                    } else if mount.is_some_and(|m| m.volume != *volume) {
                        format!(
                            "{target}: a different filesystem is mounted. Compare the supplied UUIDs, then use the appropriate caller to unmount and mount the intended medium."
                        )
                    } else if mount.is_some_and(|m| m.read_only != *read_only) {
                        format!(
                            "{target}: the live read-only/read-write state differs from the task. Save the intended setting, then unmount and mount again to apply it."
                        )
                    } else if grant.is_none_or(|g| g.mounter != Some(*mounter)) {
                        format!(
                            "{target}: the actual mounter is different from the required operator. A repeated Mount request does not transfer this session; use an authorized caller to unmount, then select the required operator and Mount."
                        )
                    } else {
                        format!(
                            "{target}: applied options differ: expected {}. Saving does not change an existing mount; unmount and mount again after correcting the option order.",
                            flags(*expected)
                        )
                    },
                )
            }
            Self::NotMounted { target } => (
                !live.state().live().contains_key(target),
                format!("{target}: must remain unmounted."),
            ),
            Self::TargetsOnly { targets } => {
                let expected = targets.iter().map(String::as_str).collect::<BTreeSet<_>>();
                let p = model::Policy::parse(live.configuration());
                let actual = p.as_ref().ok().map(|p| {
                    p.rules()
                        .iter()
                        .map(|r| r.entry.mount.target.as_str())
                        .collect::<BTreeSet<_>>()
                });
                (actual==Some(expected),"Keep exactly the requested saved mountpoints, including protected system storage.".into())
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Decision {
    Succeeded { effect: Effect },
    Refused { reason: model::ErrorKind },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub action: Action,
    pub expected: Decision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub label: String,
    pub environment: Environment,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,8>")]
    pub seed: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,12>")]
    pub probes: Vec<Probe>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub final_requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub instructions: String,
    pub first_actor: String,
    pub second_actor: String,
    pub focus_target: String,
    pub environment: Environment,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,8>")]
    pub seed: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn actor_name(&self, actor: model::Actor) -> &str {
        match actor {
            model::Actor::Root => "Administrator",
            model::Actor::First => &self.first_actor,
            model::Actor::Second => &self.second_actor,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.environment.validate().map_err(|e| e.to_string())?;
        if !self
            .environment
            .inventory
            .mountpoints()
            .contains(&self.focus_target)
            || !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 4096
            || !document(&self.starter)
            || !fits(self, 262_144)
            || !(2..=8).contains(&self.cases.len())
            || self.requirements.is_empty()
            || self.requirements.len() > 12
            || self.seed.len() > 8
            || [&self.first_actor, &self.second_actor]
                .iter()
                .any(|n| n.trim().is_empty() || n.len() > 40 || n.chars().any(char::is_control))
            || self.first_actor == self.second_actor
        {
            return Err("Mount task needs bounded instructions, distinct supplied callers, requirements and two to eight sample cases".into());
        }
        model::Policy::parse(&self.starter).map_err(|e| e.to_string())?;
        for a in &self.seed {
            a.validate(&self.environment)?;
            if matches!(a, Action::Save { .. }) {
                return Err(
                    "Source seed actions use the declared starter, not a hidden policy replacement"
                        .into(),
                );
            }
        }
        if self
            .requirements
            .iter()
            .any(|r| !r.valid(&self.environment))
        {
            return Err("Unknown required mountpoint or filesystem".into());
        }
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            c.environment.validate().map_err(|e| e.to_string())?;
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || c.probes.is_empty()
                || c.probes.len() > 12
                || c.seed.len() > 8
                || c.final_requirements.len() > 12
                || c.environment.inventory.volumes() != self.environment.inventory.volumes()
                || c.environment.inventory.mountpoints() != self.environment.inventory.mountpoints()
            {
                return Err("Mount cases need unique labels, bounded probes and consistent supplied filesystems".into());
            }
            for a in c.seed.iter().chain(c.probes.iter().map(|p| &p.action)) {
                a.validate(&c.environment)?;
                if matches!(a, Action::Save { .. }) {
                    return Err("A probe cannot replace the learner's policy with an answer".into());
                }
            }
            if c.final_requirements
                .iter()
                .any(|r| !r.valid(&c.environment))
            {
                return Err("Invalid case requirement".into());
            }
        }
        Ok(())
    }
    /// Authoring-only bounded reachability gate. Saved learner actions are checked
    /// separately by actual replay; no hidden configuration is inserted.
    pub fn validate_authored(&self) -> Result<(), String> {
        self.validate()?;
        self.initial()?;
        for c in &self.cases {
            self.seeded(&c.environment, &c.seed)?;
        }
        Ok(())
    }
    fn seeded(&self, environment: &Environment, seed: &[Action]) -> Result<model::Live, String> {
        let p = model::Policy::parse(&self.starter).map_err(|e| e.to_string())?;
        let mut live = p.startup(environment).map_err(|e| e.to_string())?.live;
        for a in seed {
            a.perform(environment, &mut live)
                .map_err(|e| format!("Authored starting action failed: {e}"))?;
        }
        Ok(live)
    }
    pub fn initial(&self) -> Result<model::Live, String> {
        self.seeded(&self.environment, &self.seed)
    }
    pub fn replay(&self, history: &[Action]) -> Result<model::Live, String> {
        if history.len() > MAX_HISTORY || !fits(&history, MAX_HISTORY_TEXT) {
            return Err(
                "Saved action history exceeds its48action/64KiB limit; previous work retained"
                    .into(),
            );
        }
        let mut live = self.initial()?;
        for a in history {
            a.validate(&self.environment)?;
            a.perform(&self.environment, &mut live)
                .map_err(|e| format!("Saved action history cannot be replayed: {e}"))?;
        }
        Ok(live)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Saved,
    Mounted,
    AlreadyMounted,
    Unmounted,
}
impl Effect {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::Saved => {
                "Saved. Existing mounts retain their applied options until unmounted and mounted again."
            }
            Self::Mounted => "Mounted with this caller and the current saved options.",
            Self::AlreadyMounted => {
                "Already mounted. The original mounter and applied options are unchanged."
            }
            Self::Unmounted => "Unmounted.",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub assignment: Task,
    pub example: Task,
    pub example_actions: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.assignment.id != self.id
            || self.example.id == self.id
            || self.example_actions.len() > 16
            || !fits(self, 524_288)
        {
            return Err("Independent bounded mount tasks and example actions required".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self
            .assignment
            .environment
            .inventory
            .volumes()
            .values()
            .any(|a| {
                self.example
                    .environment
                    .inventory
                    .volumes()
                    .values()
                    .any(|b| a.uuid == b.uuid)
            })
        {
            return Err("Practice media identities must differ from the assignment".into());
        }
        for a in &self.example_actions {
            a.validate(&self.example.environment)?;
        }
        Ok(())
    }
    pub fn validate_authored(&self) -> Result<(), String> {
        self.validate()?;
        self.assignment.validate_authored()?;
        self.example.validate_authored()?;
        for task in [&self.assignment, &self.example] {
            let fresh = Work::new(task);
            let result = run(task, &fresh)?;
            if result.current.iter().all(|r| r.0) && result.cases.iter().all(CaseResult::passed) {
                return Err("Each mount activity needs meaningful unfinished practical work, not a precompleted starter".into());
            }
        }
        let mut work = Work::new(&self.example);
        for a in &self.example_actions {
            work.apply(&self.example, a.clone())?;
        }
        let record = run(&self.example, &work)?;
        work.store(record, &self.example);
        if !work.solved(&self.example) {
            return Err(
                "The supplied example does not reach its own practical requirements".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,48>")]
    pub history: Vec<Action>,
    pub result: Option<Record>,
}
impl Work {
    pub fn new(task: &Task) -> Self {
        Self {
            draft: task.starter.clone(),
            history: vec![],
            result: None,
        }
    }
    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if !document(&text) {
            return Err("Use at most16KiB of text without unsupported control characters".into());
        }
        if self.draft != text {
            self.draft = text;
            self.result = None;
        }
        Ok(())
    }
    pub fn live(&self, task: &Task) -> Result<model::Live, String> {
        task.replay(&self.history)
    }
    pub fn apply(&mut self, task: &Task, action: Action) -> Result<Effect, String> {
        action.validate(&task.environment)?;
        if let Action::Save { configuration } = &action {
            self.edit(configuration.clone())?;
        }
        let mut live = self.live(task)?;
        let before = live.clone();
        let effect = action
            .perform(&task.environment, &mut live)
            .map_err(|e| e.to_string())?;
        if live != before {
            let mut next = self.history.clone();
            next.push(action);
            if next.len() > MAX_HISTORY || !fits(&next, MAX_HISTORY_TEXT) {
                return Err("This attempt reached its48action/64KiB history limit. Your draft is kept, but no operation was applied. Reset only this attempt to start a new session.".into());
            }
            self.history = next;
            self.result = None;
        }
        Ok(effect)
    }
    pub fn solved(&self, task: &Task) -> bool {
        self.result.as_ref().is_some_and(|r| {
            r.current(task, self)
                && r.current.iter().all(|r| r.0)
                && r.cases.iter().all(CaseResult::passed)
        })
    }
    pub fn needs_verification(&self, task: &Task) -> bool {
        self.result.as_ref().is_some_and(|r| {
            r.revision == REVISION
                && r.task == *task
                && r.draft == self.draft
                && r.history == self.history
                && !r.verified()
        })
    }
    pub fn store(&mut self, r: Record, task: &Task) -> bool {
        if r.current(task, self) && fits(&r, 524_288) {
            self.result = Some(r);
            true
        } else {
            false
        }
    }
    pub fn validate(&self, task: &Task) -> Result<(), String> {
        if !document(&self.draft) {
            return Err("Saved draft exceeds its mount-policy bounds".into());
        }
        task.replay(&self.history)?;
        if let Some(r) = &self.result {
            if r.revision != REVISION
                || r.task != *task
                || r.draft != self.draft
                || r.history != self.history
                || r.current.len() != task.requirements.len() + 1
                || r.cases.len() != task.cases.len()
                || !fits(r, 524_288)
            {
                return Err(
                    "Saved checks belong to a different mount task, input or version".into(),
                );
            }
            for ((case, row), index) in task.cases.iter().zip(&r.cases).zip(0..) {
                let shape = match &row.diagnostic {
                    Some(d) => d.len() <= 2048 && row.checks == vec![(false, d.clone())],
                    None => row.checks.len() == case.probes.len() + case.final_requirements.len(),
                };
                if !shape
                    || row.id != case.id
                    || row.label != case.label
                    || row.checks.is_empty()
                    || row.checks.len() > 32
                    || row.checks.iter().any(|(_, m)| m.len() > 2048)
                    || r.current.iter().any(|(_, m)| m.len() > 2048)
                {
                    return Err(format!("Invalid saved mount check{}", index + 1));
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub definition: Definition,
    pub view: View,
    pub assignment: Work,
    pub example: Work,
}
impl Session {
    pub fn new(definition: Definition) -> Self {
        Self {
            assignment: Work::new(&definition.assignment),
            example: Work::new(&definition.example),
            definition,
            view: View::Introduction,
        }
    }
    pub fn validate(&self, expected: &Definition) -> Result<(), String> {
        expected.validate()?;
        if self.definition != *expected || !fits(self, MAX_SESSION) {
            return Err(
                "Mount attempt differs from its bundled task or exceeds storage bounds".into(),
            );
        }
        self.assignment.validate(&expected.assignment)?;
        self.example.validate(&expected.example)
    }
    pub fn decode(raw: &[u8], expected: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved mount activity exceeds2MiB; original retained".into());
        }
        let s: Self = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
        s.validate(expected)?;
        Ok(s)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub checks: Vec<(bool, String)>,
    #[serde(default)]
    pub diagnostic: Option<String>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub revision: u32,
    pub task: Task,
    pub draft: String,
    pub history: Vec<Action>,
    pub current: Vec<(bool, String)>,
    pub cases: Vec<CaseResult>,
    #[serde(skip)]
    witness: Option<u64>,
}
impl Record {
    fn signature(&self) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let bytes = serde_json::to_vec(self).ok()?;
        if bytes.len() > 524_288 {
            return None;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut h);
        Some(h.finish())
    }
    pub fn verified(&self) -> bool {
        self.witness.is_some() && self.signature() == self.witness
    }
    pub fn current(&self, task: &Task, work: &Work) -> bool {
        self.revision == REVISION
            && self.task == *task
            && self.draft == work.draft
            && self.history == work.history
            && self.verified()
    }
}
/// Initial tick replays at most48successful actions plus bounded source startup/
/// seed. Each later tick performs at most8startup+8seed+12probe operations.
/// Parsed policy is retained; no learner loop, subprocess or kernel call exists.
pub struct Run {
    task: Task,
    draft: String,
    history: Vec<Action>,
    current: Vec<(bool, String)>,
    rows: Vec<CaseResult>,
    stopped: bool,
    policy: Result<model::Policy, String>,
}
impl Run {
    pub fn new(task: &Task, work: &Work) -> Result<Self, String> {
        task.validate()?;
        if !document(&work.draft)
            || work.history.len() > 48
            || !fits(&work.history, MAX_HISTORY_TEXT)
        {
            return Err("Mount check inputs exceed their bounds".into());
        }
        Ok(Self {
            task: task.clone(),
            draft: work.draft.clone(),
            history: work.history.clone(),
            current: vec![],
            rows: vec![],
            stopped: false,
            policy: Err("Current saved policy is not checked yet".into()),
        })
    }
    pub fn done(&self) -> bool {
        self.stopped || (!self.current.is_empty() && self.rows.len() == self.task.cases.len())
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    pub fn tick(&mut self) {
        if self.done() {
            return;
        }
        if self.current.is_empty() {
            self.current = match self.task.replay(&self.history) {
                Ok(live) => {
                    self.policy =
                        model::Policy::parse(live.configuration()).map_err(|e| e.to_string());
                    let mut rows = vec![(
                        live.configuration() == self.draft,
                        if live.configuration() == self.draft {
                            "Editor matches the saved configuration.".into()
                        } else {
                            "Editor changes are not saved. Save configuration before checking the full task.".into()
                        },
                    )];
                    rows.extend(
                        self.task
                            .requirements
                            .iter()
                            .map(|r| r.check(&self.task.environment, &live)),
                    );
                    rows
                }
                Err(e) => {
                    let mut rows = vec![(false, e)];
                    rows.extend(self.task.requirements.iter().map(|_|(false,"Saved actions could not be replayed; current mount state is unverified.".into())));
                    rows
                }
            };
            return;
        }
        let case = &self.task.cases[self.rows.len()];
        let configured = self.policy.as_ref().map_err(Clone::clone).and_then(|p| {
            let mut live = self.task.seeded(&case.environment, &case.seed)?;
            live.save_policy(p).map_err(|e| e.to_string())?;
            Ok(live)
        });
        let mut diagnostic = None;
        let checks = match configured {
            Err(e) => {
                diagnostic = Some(e.clone());
                vec![(false, e)]
            }
            Ok(mut live) => {
                let mut rows = vec![];
                for probe in &case.probes {
                    let before = live.clone();
                    let result = probe.action.perform(&case.environment, &mut live);
                    let ok = match (&probe.expected, &result) {
                        (Decision::Succeeded { effect }, Ok(actual)) => effect == actual,
                        (Decision::Refused { reason }, Err(actual)) => {
                            *reason == actual.kind && live == before
                        }
                        _ => false,
                    };
                    let actual = match &result {
                        Ok(effect) => effect.explanation().into(),
                        Err(e) => e.to_string(),
                    };
                    let expected = match &probe.expected {
                        Decision::Succeeded { effect } => effect.explanation().to_owned(),
                        Decision::Refused {
                            reason: model::ErrorKind::PermissionDenied,
                        } => "This caller should be refused without changing the mount.".into(),
                        Decision::Refused {
                            reason: model::ErrorKind::Unavailable,
                        } => "The missing medium should be reported without changing the mount."
                            .into(),
                        Decision::Refused { reason } => {
                            format!("A {reason:?} refusal with no state change.")
                        }
                    };
                    let action = match &probe.action {
                        Action::Mount { actor, target } => {
                            format!("Mount {target} as {}", self.task.actor_name(*actor))
                        }
                        Action::Unmount { actor, target } => {
                            format!("Unmount {target} as {}", self.task.actor_name(*actor))
                        }
                        Action::Save { .. } => "Save configuration".into(),
                    };
                    rows.push((
                        ok,
                        if ok {
                            format!("{action}: {actual}")
                        } else {
                            format!("{action}: expected {expected} Actual: {actual}")
                        },
                    ));
                }
                rows.extend(
                    case.final_requirements
                        .iter()
                        .map(|r| r.check(&case.environment, &live)),
                );
                rows
            }
        };
        self.rows.push(CaseResult {
            id: case.id.clone(),
            label: case.label.clone(),
            checks,
            diagnostic,
        });
    }
    pub fn record(self) -> Option<Record> {
        if !self.stopped && !self.current.is_empty() && self.rows.len() == self.task.cases.len() {
            let mut r = Record {
                revision: REVISION,
                task: self.task,
                draft: self.draft,
                history: self.history,
                current: self.current,
                cases: self.rows,
                witness: None,
            };
            r.witness = r.signature();
            if r.witness.is_some() { Some(r) } else { None }
        } else {
            None
        }
    }
}
pub fn run(task: &Task, work: &Work) -> Result<Record, String> {
    let mut r = Run::new(task, work)?;
    for _ in 0..10 {
        if r.done() {
            return r
                .record()
                .ok_or_else(|| "Stopped or oversized mount check; no completion awarded".into());
        }
        r.tick();
    }
    Err("Mount check exceeded its fixed tick bound".into())
}

pub fn checks(task: &Task, work: &Work) -> Vec<(bool, String)> {
    let Some(record) = &work.result else {
        return vec![(false, "Not checked. Save the configuration, perform the required mount actions, then choose Check work.".into())];
    };
    if !record.current(task, work) {
        return vec![(false, "Earlier or changed checks. Check this saved session again before counting it as current success.".into())];
    }
    let mut rows = record.current.clone();
    rows.extend(record.cases.iter().map(|c| {
        (
            c.passed(),
            format!(
                "{}: {}",
                c.label,
                c.checks
                    .iter()
                    .find(|r| !r.0)
                    .map_or("All supplied caller checks match.", |r| r.1.as_str())
            ),
        )
    }));
    rows
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::UserMounts { simulation },
        crate::engine::Goal::UserMounts { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Caller-aware mount source/state mismatch".into());
    };
    definition.validate()?;
    if definition.id != s.id
        || definition.assignment.instructions != s.objective
        || **simulation != Session::new((**definition).clone())
    {
        return Err("Mount identity, assignment and untouched initial work must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let definition = source(s)?;
    let crate::engine::State::UserMounts { simulation } = &a.state else {
        return Err("Saved mount work belongs to another activity".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible caller-aware mount metadata; original retained".into());
    }
    simulation.validate(definition)
}
