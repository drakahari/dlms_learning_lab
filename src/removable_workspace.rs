//! Source-bound optional-media exercise state; no host operations.
use crate::{removable_mount as model, script_teaching::View, storage as core};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt};
pub const MAX_DEFINITION: usize = 524_288;
pub const MAX_SESSION: usize = 150_000;
pub const MAX_TICKS: usize = 16;
pub const PREFIX: &str = "MOUNT POLICY\n";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn document<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Text;
    impl Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16KiB of virtual mount configuration")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > core::MAX_CONFIG_BYTES {
                Err(E::custom("Configuration draft byte limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text)
}
fn optional_document<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Opt;
    impl<'de> Visitor<'de> for Opt {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an optional bounded tested configuration")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            document(d).map(Some)
        }
    }
    d.deserialize_option(Opt)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} bounded items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                match a.next_element()? {
                    Some(x) => out.push(x),
                    None => return Ok(out),
                }
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Configuration exercise item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn fits<T: Serialize>(value: &T, max: usize) -> bool {
    struct Count {
        n: usize,
        max: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.max)
                .ok_or_else(|| std::io::Error::other("Configuration exercise byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, max }, value).is_ok()
}
fn draft_bound(text: &str) -> bool {
    text.len() <= core::MAX_CONFIG_BYTES
        && !text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
fn dependency_name(d: model::Dependency) -> &'static str {
    match d {
        model::Dependency::Required => "Required automatic request",
        model::Dependency::Wanted => "Optional automatic request",
        model::Dependency::NotEnrolled => "Manual only",
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Policy {
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        target: String,
        mode: model::Dependency,
    },
    Mounted {
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        target: String,
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        volume: String,
        read_only: bool,
    },
    NotMounted {
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        target: String,
    },
    RequiredAttempts {
        satisfied: bool,
    },
    EntriesOnly {
        #[serde(deserialize_with = "targets")]
        targets: Vec<String>,
    },
}
fn targets<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Target(#[serde(deserialize_with = "crate::cron::text::<_,256>")] String);
    list::<D, Target, 8>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
impl Requirement {
    fn identity(&self) -> String {
        match self {
            Self::Policy { target, .. } => format!("policy-{target}"),
            Self::Mounted { target, .. } | Self::NotMounted { target } => format!("mount-{target}"),
            Self::RequiredAttempts { .. } => "required-attempts".into(),
            Self::EntriesOnly { .. } => "entries-only".into(),
        }
    }
    fn references(&self) -> Vec<&str> {
        match self {
            Self::Policy { target, .. }
            | Self::Mounted { target, .. }
            | Self::NotMounted { target } => vec![target],
            Self::EntriesOnly { targets } => targets.iter().map(String::as_str).collect(),
            _ => vec![],
        }
    }
    fn check(&self, policy: &model::Policy, result: &model::Projection) -> (bool, String) {
        match self {
            Self::Policy { target, mode } => {
                let actual = policy
                    .entries()
                    .iter()
                    .find(|e| e.mount.target == *target)
                    .map(model::Entry::dependency);
                (
                    actual == Some(*mode),
                    format!(
                        "{target}: expected {}; actual {}. noauto removes this table's automatic request; nofail makes an automatic request optional, not ready by a deadline.",
                        dependency_name(*mode),
                        actual.map(dependency_name).unwrap_or("Not configured")
                    ),
                )
            }
            Self::Mounted {
                target,
                volume,
                read_only,
            } => {
                let actual = result.state.live().get(target);
                let expected = core::Mount {
                    volume: volume.clone(),
                    read_only: *read_only,
                };
                (
                    actual == Some(&expected),
                    format!(
                        "{target}: expected volume {volume}, {}; actual {}. A target that can proceed does not prove this intended filesystem mounted.",
                        if *read_only {
                            "Read only"
                        } else {
                            "Read/write"
                        },
                        actual
                            .map(|m| format!(
                                "volume {}, {}",
                                m.volume,
                                if m.read_only {
                                    "Read only"
                                } else {
                                    "Read/write"
                                }
                            ))
                            .unwrap_or_else(|| "Not mounted".into())
                    ),
                )
            }
            Self::NotMounted { target } => (
                !result.state.live().contains_key(target),
                format!(
                    "{target}: expected Not mounted in this sample; actual {}.",
                    if result.state.live().contains_key(target) {
                        "Mounted"
                    } else {
                        "Not mounted"
                    }
                ),
            ),
            Self::RequiredAttempts { satisfied } => {
                let actual = result.required_attempts_satisfied();
                let required = result
                    .rows
                    .iter()
                    .any(|r| r.dependency == model::Dependency::Required);
                (
                    actual == *satisfied && required,
                    format!(
                        "Required-storage check: expected {}; actual {}. This is the supplied request set, not actual boot completion.",
                        if *satisfied {
                            "all required attempts succeed"
                        } else {
                            "a missing required filesystem remains a reported problem"
                        },
                        if !required {
                            "no required requests remain"
                        } else if actual {
                            "all required attempts succeed"
                        } else {
                            "a required request has failed"
                        }
                    ),
                )
            }
            Self::EntriesOnly { targets } => {
                let actual = policy
                    .entries()
                    .iter()
                    .map(|e| e.mount.target.as_str())
                    .collect::<BTreeSet<_>>();
                let wanted = targets.iter().map(String::as_str).collect::<BTreeSet<_>>();
                (
                    actual == wanted,
                    format!(
                        "Configured mountpoints: expected {}; actual {}. Keep the complete requested set without extra entries.",
                        targets.join(", "),
                        actual.into_iter().collect::<Vec<_>>().join(", ")
                    ),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub environment: model::Environment,
    #[serde(deserialize_with = "list::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    #[serde(deserialize_with = "document")]
    pub starter: String,
    #[serde(deserialize_with = "list::<_,Case,3>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1800
            || self
                .instructions
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            || !draft_bound(&self.starter)
            || !(2..=3).contains(&self.cases.len())
            || !fits(self, MAX_DEFINITION / 2)
        {
            return Err("Invalid bounded mount-policy task".into());
        }
        model::Policy::parse(&self.starter)
            .map_err(|e| format!("Starter must be supported unfinished configuration: {e}"))?;
        let first = &self.cases[0].environment.inventory;
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            c.environment.validate().map_err(|e| e.to_string())?;
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || c.requirements.is_empty()
                || c.requirements.len() > 12
                || c.environment.inventory.volumes() != first.volumes()
                || c.environment.inventory.mountpoints() != first.mountpoints()
            {
                return Err(
                    "Cases need unique identities and consistent supplied media/targets".into(),
                );
            }
            let mut goals = BTreeSet::new();
            for r in &c.requirements {
                if !goals.insert(r.identity())
                    || r.references()
                        .iter()
                        .any(|target| !first.mountpoints().contains(*target))
                {
                    return Err("Unknown or duplicate mount outcome reference".into());
                }
                match r {
                    Requirement::Mounted { volume, .. }
                        if !first.volumes().contains_key(volume)
                            || !c.environment.present.contains(volume) =>
                    {
                        return Err(
                            "Expected mounted volume must be present in its supplied sample".into(),
                        );
                    }
                    Requirement::EntriesOnly { targets }
                        if targets.is_empty()
                            || targets.len() > 8
                            || targets.iter().collect::<BTreeSet<_>>().len() != targets.len() =>
                    {
                        return Err(
                            "Expected mountpoint set is empty, duplicated or oversized".into()
                        );
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn run(&self, text: &str) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, text)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.results);
            }
        }
        Err("Mount-policy checks exceeded their bounded phase budget".into())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub diagnostic: Option<String>,
    pub checks: Vec<(bool, String)>,
    pub projection: Option<model::Projection>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none() && !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
pub struct Run {
    task: Task,
    text: String,
    policy: Result<model::Policy, String>,
    index: usize,
    results: Vec<CaseResult>,
    cancelled: bool,
    phase: u8,
    pending: Option<CaseResult>,
}
impl Run {
    pub fn new(task: &Task, text: &str) -> Result<Self, String> {
        task.validate()?;
        if !draft_bound(text) {
            return Err(
                "Use at most16KiB without control characters; previous draft retained".into(),
            );
        }
        Ok(Self {
            task: task.clone(),
            text: text.into(),
            policy: model::Policy::parse(text).map_err(|e| e.to_string()),
            index: 0,
            results: vec![],
            cancelled: false,
            phase: 0,
            pending: None,
        })
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.index == self.task.cases.len()
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.results.clear();
        self.pending = None;
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.results)
    }
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        let c = &self.task.cases[self.index];
        if self.phase == 0 {
            let projected = self
                .policy
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|p| p.startup(&c.environment).map_err(|e| e.to_string()));
            self.pending = Some(match projected {
                Err(reason) => CaseResult {
                    id: c.id.clone(),
                    label: c.label.clone(),
                    diagnostic: Some(reason.clone()),
                    checks: vec![(false, reason)],
                    projection: None,
                },
                Ok(projection) => CaseResult {
                    id: c.id.clone(),
                    label: c.label.clone(),
                    diagnostic: None,
                    checks: vec![],
                    projection: Some(projection),
                },
            });
            self.phase = 1;
            return false;
        }
        if self.phase == 1 {
            let row = self.pending.as_mut().unwrap();
            if let Some(projection) = &row.projection {
                let p = self.policy.as_ref().unwrap();
                row.checks=vec![(true,"Configuration parsed and supplied mount attempts evaluated; outcomes remain separate from requirement checks.".into())];
                row.checks
                    .extend(c.requirements.iter().map(|r| r.check(p, projection)));
            }
            self.phase = 2;
            return false;
        }
        self.results.push(self.pending.take().unwrap());
        self.index += 1;
        self.phase = 0;
        self.done()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "document")]
    pub example_worked: String,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || !draft_bound(&self.example_worked)
            || !fits(self, MAX_DEFINITION)
        {
            return Err("Independent mount tasks need distinct bounded sources".into());
        }
        let a = self.initial.cases[0].environment.inventory.volumes();
        let b = self.example.cases[0].environment.inventory.volumes();
        if a.values().any(|x| b.values().any(|y| x.uuid == y.uuid)) {
            return Err("Worked media identities must differ from assignment media".into());
        }
        for task in [&self.initial, &self.example] {
            if task.run(&task.starter)?.iter().all(CaseResult::passed) {
                return Err("Each starter must retain meaningful unfinished mount policy".into());
            }
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked policy does not reach all practice outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("Mount-policy work belongs to another source task; retained".into());
        }
        Ok(())
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if s.assignment.tested.as_deref() != Some(s.assignment.draft.as_str()) {
            return vec![(
                false,
                "Not checked · run the current mount policy against the supplied samples.".into(),
            )];
        }
        match self.initial.run(&s.assignment.draft) {
            Err(e) => vec![(false, e)],
            Ok(rows) => rows
                .into_iter()
                .flat_map(|r| {
                    r.checks
                        .into_iter()
                        .map(move |(ok, text)| (ok, format!("{} · {text}", r.label)))
                })
                .collect(),
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_DEFINITION {
            return Err("Mount-policy lesson exceeds512KiB".into());
        }
        let d: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid mount-policy lesson: {e}"))?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "document")]
    pub draft: String,
    #[serde(deserialize_with = "optional_document")]
    pub tested: Option<String>,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            tested: None,
        }
    }
    pub fn replace(&mut self, text: &str) -> Result<(), String> {
        if !draft_bound(text) {
            return Err(
                "Use at most16KiB without control characters; previous draft retained".into(),
            );
        }
        if self.draft != text {
            self.draft = text.into();
            self.tested = None;
        }
        Ok(())
    }
    pub fn begin(&mut self, t: &Task) -> Result<Run, String> {
        self.tested = None;
        Run::new(t, &self.draft)
    }
    pub fn finish(&mut self, t: &Task, run: &Run) -> Result<(), String> {
        if !run.done() || run.text != self.draft || run.task != *t {
            return Err(
                "Stopped, unfinished or changed mount checks cannot award current credit".into(),
            );
        }
        self.tested = Some(self.draft.clone());
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(t)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(t, &run)?;
                return Ok(run.results);
            }
        }
        Err("Mount-policy checks exceeded bounded phases".into())
    }
    pub fn passed(&self, t: &Task) -> bool {
        self.tested.as_deref() == Some(self.draft.as_str())
            && t.run(&self.draft)
                .is_ok_and(|r| !r.is_empty() && r.iter().all(CaseResult::passed))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || !fits(self, MAX_SESSION)
            || [&self.assignment, &self.practice].iter().any(|s| {
                !draft_bound(&s.draft) || s.tested.as_ref().is_some_and(|s| !draft_bound(s))
            })
        {
            return Err("Mount-policy progress exceeds its version or draft bounds".into());
        }
        Ok(())
    }
    pub fn from_json(text: &str, d: &Definition) -> Result<Self, String> {
        if text.len() > MAX_SESSION {
            return Err("Mount-policy progress exceeds its byte limit".into());
        }
        let s: Self = serde_json::from_str(text)
            .map_err(|e| format!("Invalid mount-policy progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native policy editor; this is internal reference data")?;
        self.assignment.replace(text)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Simulated mount-policy checked: {}/{} sample requirements pass",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}
pub const HELP: &str = "Edit the virtual six-field mount table, then Run mount checks. Only supplied possible media, presence and aliases are evaluated. Required/Wanted/Not enrolled requests are separate from actual mount attempts; an optional failure remains failed, and a reported required failure can be the expected protective outcome. No real boot, timing guarantee, filesystem check, device, command or host file is accessed. Supported: exact supplied UUID or virtual /dev source, fixed targets, ext4/xfs metadata, defaults/ro/rw/auto/noauto/nofail. user/users/owner, labels, escaping, nested/network mounts, automount, hotplug and unsupported options are refused.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::RemovableMounts { simulation },
        crate::engine::Goal::RemovableMounts { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Mount-policy source/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Mount-policy identity, task and supplied drafts must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::RemovableMounts { simulation } = &a.state else {
        return Err("Mount-policy work belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible mount-policy attempt metadata; original work retained".into());
    }
    d.bound(simulation)
}
