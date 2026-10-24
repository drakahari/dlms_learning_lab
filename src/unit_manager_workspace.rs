//! Pure native-editor data and current/fresh outcome grading. No host or CLI engine.
use crate::unit_manager::{self as core, Action, Program, Runtime, Snapshot, State, UnitType};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const MAX_REQUIREMENTS: usize = 48;
pub const MAX_ACTIONS: usize = 32;
pub const MAX_ACTION_BYTES: usize = 1_572_864;
pub const HELP: &str = "Edit the supplied virtual unit or application configuration. Save only updates raw text; manager reload, program start/initialization and enable/default policy are separate simulator actions. No host systemd, shell, network or boot is used.";

pub(crate) fn sequence<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} bounded items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            while items.len() < N {
                match a.next_element()? {
                    Some(item) => items.push(item),
                    None => return Ok(items),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Virtual manager item limit"));
            }
            Ok(items)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
pub(crate) fn requirements<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    sequence::<D, Requirement, MAX_REQUIREMENTS>(d)
}
pub(crate) fn actions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<EditAction>, D::Error> {
    sequence::<D, EditAction, MAX_ACTIONS>(d)
}
fn orderings<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Ordering>, D::Error> {
    sequence::<D, Ordering, 16>(d)
}
fn links<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Link>, D::Error> {
    sequence::<D, Link, 256>(d)
}
fn bounded_message(text: String) -> String {
    if text.len() <= 512 {
        return text;
    }
    let mut end = 509;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Inactive,
    TargetActive,
    Dispatched,
    Running,
    Exited,
    Failed,
}
impl Phase {
    fn label(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::TargetActive => "an active target",
            Self::Dispatched => "dispatched, with initialization pending",
            Self::Running => "running",
            Self::Exited => "active after a successful oneshot exit",
            Self::Failed => "failed",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    Requires,
    Wants,
    After,
    Before,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "core::text128")]
    pub target: String,
    #[serde(deserialize_with = "core::text128")]
    pub unit: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ordering {
    #[serde(deserialize_with = "core::text128")]
    pub before: String,
    #[serde(deserialize_with = "core::text128")]
    pub after: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    SavedConfigurationValid {},
    LoadedCurrent {
        #[serde(deserialize_with = "core::text128")]
        unit: String,
    },
    Runtime {
        #[serde(deserialize_with = "core::text128")]
        unit: String,
        phase: Phase,
        port: Option<u16>,
        value: Option<core::SmallValue>,
    },
    ExecutionCurrent {
        #[serde(deserialize_with = "core::text128")]
        unit: String,
    },
    Artifact {
        #[serde(deserialize_with = "core::text128")]
        artifact: String,
        #[serde(deserialize_with = "core::text128")]
        producer: String,
        value: core::SmallValue,
        current_inputs: bool,
    },
    InstallLink {
        #[serde(deserialize_with = "core::text128")]
        target: String,
        #[serde(deserialize_with = "core::text128")]
        unit: String,
        present: bool,
    },
    DefaultTarget {
        #[serde(deserialize_with = "core::text128")]
        target: String,
    },
    Dependency {
        #[serde(deserialize_with = "core::text128")]
        unit: String,
        dependency: DependencyKind,
        #[serde(deserialize_with = "core::text128")]
        target: String,
        present: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fresh {
    #[serde(deserialize_with = "requirements")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "core::names")]
    pub started: BTreeSet<String>,
    #[serde(deserialize_with = "orderings")]
    pub ordering: Vec<Ordering>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    #[serde(deserialize_with = "core::names")]
    pub unit_documents: BTreeSet<String>,
    #[serde(deserialize_with = "core::names")]
    pub configuration: BTreeSet<String>,
    pub preserve_other_boot_links: bool,
    #[serde(deserialize_with = "links")]
    pub editable_boot_links: Vec<Link>,
    pub default_target: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: State,
    #[serde(deserialize_with = "requirements")]
    pub requirements: Vec<Requirement>,
    pub fresh: Fresh,
    pub preservation: Preservation,
}
fn is_false(value: &bool) -> bool {
    !*value
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    #[serde(deserialize_with = "core::documents")]
    pub documents: BTreeMap<String, String>,
    #[serde(deserialize_with = "core::configurations")]
    pub configuration: BTreeMap<String, String>,
    #[serde(deserialize_with = "core::text128")]
    pub selected_unit: String,
    #[serde(deserialize_with = "core::text128")]
    pub selected_config: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub editing_configuration: bool,
}
impl Draft {
    pub fn new(state: &State) -> Result<Self, String> {
        state.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            version: 1,
            documents: state.documents().clone(),
            configuration: state.configuration().clone(),
            selected_unit: state.inventory().units.iter().next().cloned().unwrap(),
            editing_configuration: false,
            selected_config: state
                .inventory()
                .config_paths
                .iter()
                .next()
                .cloned()
                .unwrap_or_default(),
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && (!self.editing_configuration || !self.configuration.is_empty())
            && !self.documents.is_empty()
            && self.documents.len() <= core::MAX_UNITS
            && self.configuration.len() <= core::MAX_CONFIGS
            && self
                .documents
                .iter()
                .all(|(k, v)| !k.is_empty() && k.len() <= 64 && v.len() <= core::MAX_UNIT_BYTES)
            && self
                .configuration
                .iter()
                .all(|(k, v)| !k.is_empty() && k.len() <= 128 && v.len() <= core::MAX_CONFIG_BYTES)
            && self.selected_unit.len() <= 64
            && self.documents.contains_key(&self.selected_unit)
            && self.selected_config.len() <= 128
            && (self.configuration.contains_key(&self.selected_config)
                || (self.configuration.is_empty() && self.selected_config.is_empty()))
    }
    pub fn same_shape(&self, state: &State) -> bool {
        self.valid()
            && state.validate().is_ok()
            && self.documents.len() == state.inventory().units.len()
            && self.configuration.len() == state.inventory().config_paths.len()
            && self.documents.iter().all(|(k, v)| {
                state.inventory().units.contains(k) && v.len() <= core::MAX_UNIT_BYTES
            })
            && self.configuration.iter().all(|(k, v)| {
                state.inventory().config_paths.contains(k) && v.len() <= core::MAX_CONFIG_BYTES
            })
            && self.documents.contains_key(&self.selected_unit)
            && (self.configuration.contains_key(&self.selected_config)
                || (self.configuration.is_empty() && self.selected_config.is_empty()))
    }
    pub fn matches(&self, state: &State) -> bool {
        self.same_shape(state)
            && self.documents == *state.documents()
            && self.configuration == *state.configuration()
    }
    pub fn apply(&self, state: &mut State) -> Result<(), String> {
        if !self.same_shape(state) {
            return Err(
                "Editor slots differ from the supplied unit/configuration inventory".into(),
            );
        }
        state
            .replace_saved(&self.documents, &self.configuration)
            .map_err(|e| e.to_string())
    }
}
/// Typed data for native controls/authoring; never shell code or a systemctl CLI.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum EditAction {
    Save { draft: Draft },
    Manager { action: Action },
}
/// Borrowed shape/scalar preflight for direct callers and teaching metadata.
/// It deliberately does not execute a job or parse a future edited document.
pub(crate) fn valid_action(initial: &State, action: &EditAction) -> bool {
    if initial.validate().is_err() {
        return false;
    }
    let inventory = initial.inventory();
    match action {
        EditAction::Save { draft } => draft.same_shape(initial),
        EditAction::Manager { action } => match action {
            Action::SaveUnit { unit, text } => {
                inventory.units.contains(unit) && text.len() <= core::MAX_UNIT_BYTES
            }
            Action::SaveConfig { path, text } => {
                inventory.config_paths.contains(path) && text.len() <= core::MAX_CONFIG_BYTES
            }
            Action::Enable { unit }
            | Action::Disable { unit }
            | Action::Start { unit }
            | Action::Stop { unit } => inventory.units.contains(unit),
            Action::SetDefault { target } => {
                inventory.units.contains(target) && target.ends_with(".target")
            }
            Action::DaemonReload {} | Action::AdvancePrograms {} => true,
        },
    }
}
pub fn apply(
    state: &mut State,
    initial: &State,
    action: &EditAction,
) -> Result<Option<core::Report>, String> {
    if !state.same_context(initial) || !valid_action(initial, action) {
        return Err("Virtual unit inventory differs from this assignment".into());
    }
    match action {
        EditAction::Save { draft } => {
            draft.apply(state)?;
            Ok(None)
        }
        EditAction::Manager { action } => state.apply(action).map(Some).map_err(|e| e.to_string()),
    }
}
fn link_present(state: &State, target: &str, unit: &str) -> bool {
    state
        .boot_links()
        .get(target)
        .is_some_and(|v| v.contains(unit))
}
fn snapshot_current(state: &State, unit: &str, snapshot: &Snapshot) -> bool {
    if state.loaded_sources().get(unit) != Some(&snapshot.unit_source)
        || state.documents().get(unit) != Some(&snapshot.unit_source)
    {
        return false;
    }
    let Ok(definitions) = state.definitions() else {
        return false;
    };
    let Some(d) = definitions.get(unit) else {
        return false;
    };
    let UnitType::Service { exec, .. } = &d.kind else {
        return false;
    };
    let Some(p) = state.inventory().programs.get(exec) else {
        return false;
    };
    let (config, need) = match p {
        Program::ValueTask {
            config,
            prerequisite,
            ..
        }
        | Program::PortDaemon {
            config,
            prerequisite,
            ..
        } => (config, prerequisite),
    };
    state.configuration().get(config) == Some(&snapshot.configuration)
        && need
            .as_ref()
            .map(|n| state.artifacts().get(&n.artifact).map(|a| a.value.as_str()))
            == snapshot.artifact_input.as_ref().map(|v| Some(v.as_str()))
}
fn positive(requirement: &Requirement) -> bool {
    matches!(
        requirement,
        Requirement::Runtime {
            phase: Phase::Running | Phase::Exited,
            ..
        } | Requirement::Artifact {
            current_inputs: true,
            ..
        }
    )
}
fn valid_text(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
pub(crate) fn valid_requirements(
    requirements: &[Requirement],
    state: &State,
    require_positive: bool,
) -> bool {
    if requirements.is_empty() || requirements.len() > MAX_REQUIREMENTS {
        return false;
    }
    let inventory = state.inventory();
    let mut unique = BTreeSet::new();
    for r in requirements {
        let valid = match r {
            Requirement::SavedConfigurationValid {} => true,
            Requirement::LoadedCurrent { unit } | Requirement::ExecutionCurrent { unit } => {
                inventory.units.contains(unit)
            }
            Requirement::Runtime {
                unit,
                phase,
                port,
                value,
            } => {
                inventory.units.contains(unit)
                    && port.is_none_or(|p| p > 0)
                    && value.as_ref().is_none_or(|v| valid_text(v.as_str()))
                    && !(port.is_some() && value.is_some())
                    && (matches!(phase, Phase::Running | Phase::Exited)
                        || (port.is_none() && value.is_none()))
                    && (!matches!(phase, Phase::Running | Phase::Exited)
                        || requirements
                            .iter()
                            .any(|r| matches!(r,Requirement::ExecutionCurrent{unit:u}if u==unit)))
            }
            Requirement::Artifact {
                artifact,
                producer,
                value,
                ..
            } => {
                inventory.artifact_ids.contains(artifact)
                    && inventory.units.contains(producer)
                    && valid_text(value.as_str())
            }
            Requirement::InstallLink { target, unit, .. } => {
                inventory.units.contains(target)
                    && target.ends_with(".target")
                    && inventory.units.contains(unit)
            }
            Requirement::DefaultTarget { target } => {
                inventory.units.contains(target) && target.ends_with(".target")
            }
            Requirement::Dependency { unit, target, .. } => {
                inventory.units.contains(unit) && inventory.units.contains(target) && unit != target
            }
        };
        if !valid {
            return false;
        }
        // Validate fixed names before deriving keys; no serialization/allocation
        // of a directly constructed unbounded field occurs during validation.
        let key = match r {
            Requirement::SavedConfigurationValid {} => (0, "", "", 0),
            Requirement::LoadedCurrent { unit } => (1, unit.as_str(), "", 0),
            Requirement::Runtime { unit, .. } => (2, unit.as_str(), "", 0),
            Requirement::ExecutionCurrent { unit } => (3, unit.as_str(), "", 0),
            Requirement::Artifact { artifact, .. } => (4, artifact.as_str(), "", 0),
            Requirement::InstallLink { target, unit, .. } => (5, target.as_str(), unit.as_str(), 0),
            Requirement::DefaultTarget { .. } => (6, "", "", 0),
            Requirement::Dependency {
                unit,
                target,
                dependency,
                ..
            } => (7, unit.as_str(), target.as_str(), *dependency as u8),
        };
        if !unique.insert(key) {
            return false;
        }
    }
    !require_positive || requirements.iter().any(positive)
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.to_string())?;
        let i = self.initial.inventory();
        let p = &self.preservation;
        if self.requirements.len() + self.fresh.requirements.len() > MAX_REQUIREMENTS
            || !valid_requirements(&self.requirements, &self.initial, true)
            || !valid_requirements(&self.fresh.requirements, &self.initial, true)
            || !self
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::SavedConfigurationValid {}))
            || self.fresh.started.len() > core::MAX_UNITS
            || self.fresh.ordering.len() > 16
            || self.fresh.started.iter().any(|s| !i.units.contains(s))
            || self.fresh.ordering.iter().any(|o| {
                o.before == o.after || !i.units.contains(&o.before) || !i.units.contains(&o.after)
            })
            || p.unit_documents.len() > core::MAX_UNITS
            || p.configuration.len() > core::MAX_CONFIGS
            || p.editable_boot_links.len() > 256
            || p.unit_documents.iter().any(|s| !i.units.contains(s))
            || p.configuration.iter().any(|s| !i.config_paths.contains(s))
            || p.editable_boot_links.iter().any(|l| {
                !i.units.contains(&l.target)
                    || !l.target.ends_with(".target")
                    || !i.units.contains(&l.unit)
            })
            || (!p.preserve_other_boot_links && !p.editable_boot_links.is_empty())
        {
            return Err(
                "Invalid virtual-unit outcomes, positive workloads or preservation metadata".into(),
            );
        }
        let mut pairs = BTreeSet::new();
        for l in &p.editable_boot_links {
            if !pairs.insert((&l.target, &l.unit)) {
                return Err("Duplicate editable boot link".into());
            }
        }
        let mut order = BTreeSet::new();
        for o in &self.fresh.ordering {
            if !order.insert((&o.before, &o.after)) {
                return Err("Duplicate fresh ordering check".into());
            }
        }
        for r in self.requirements.iter().chain(&self.fresh.requirements) {
            if let Requirement::DefaultTarget { target } = r
                && p.default_target
                && target != self.initial.default_target()
            {
                return Err(
                    "Requested default target contradicts its explicit preservation rule".into(),
                );
            }
            if let Requirement::InstallLink {
                target,
                unit,
                present,
            } = r
                && p.preserve_other_boot_links
                && !p
                    .editable_boot_links
                    .iter()
                    .any(|l| l.target == *target && l.unit == *unit)
                && link_present(&self.initial, target, unit) != *present
            {
                return Err(
                    "Requested enablement contradicts its explicit preservation rule".into(),
                );
            }
            if let Requirement::Dependency { unit, .. } = r
                && p.unit_documents.contains(unit)
                && !check_requirement(&self.initial, r).0
            {
                return Err("Requested dependency contradicts its preserved unit document".into());
            }
        }
        Ok(())
    }
    pub fn same_assignment(&self, current: &State) -> bool {
        self.validate().is_ok() && current.same_context(&self.initial)
    }
}
pub(crate) fn check_requirement(state: &State, r: &Requirement) -> (bool, String) {
    match r {
        Requirement::SavedConfigurationValid {} => match state.validate_documents() {
            Ok(()) => (
                true,
                "Saved unit documents are valid within this simulator".into(),
            ),
            Err(e) => (false, format!("Saved unit document: {e}")),
        },
        Requirement::LoadedCurrent { unit } => (
            state.needs_daemon_reload(unit).is_ok_and(|v| !v),
            format!(
                "{unit}: manager definitions must match current saved text; saving alone does not update them"
            ),
        ),
        Requirement::Runtime {
            unit,
            phase,
            port,
            value,
        } => {
            let actual = match state.runtime().get(unit) {
                None | Some(Runtime::Inactive {}) => Phase::Inactive,
                Some(Runtime::TargetActive {}) => Phase::TargetActive,
                Some(Runtime::Dispatched { .. }) => Phase::Dispatched,
                Some(Runtime::Running { .. }) => Phase::Running,
                Some(Runtime::Exited { .. }) => Phase::Exited,
                Some(Runtime::Failed { .. }) => Phase::Failed,
            };
            if actual != *phase {
                return (
                    false,
                    format!(
                        "{unit}: currently {}; the task needs {}. Inspect the current manager and program state.",
                        actual.label(),
                        phase.label()
                    ),
                );
            }
            if port.is_some() || value.is_some() {
                let actual = state.result(unit).ok().flatten();
                if actual.as_ref().is_none_or(|result| {
                    port.is_some_and(|p| result.port != Some(p))
                        || value
                            .as_ref()
                            .is_some_and(|v| result.value.as_deref() != Some(v.as_str()))
                }) {
                    return (
                        false,
                        format!(
                            "{unit}: its previous run produced a different result. Check the application settings, then stop and start the service to use the saved values."
                        ),
                    );
                }
            }
            (true, format!("{unit}: required runtime state and result"))
        }
        Requirement::ExecutionCurrent { unit } => {
            let fresh = match state.runtime().get(unit) {
                Some(Runtime::Running { snapshot } | Runtime::Exited { snapshot }) => {
                    snapshot_current(state, unit, snapshot)
                }
                _ => false,
            };
            (
                fresh,
                format!(
                    "{unit}: the last successful run must use the current unit file, application settings and prepared data. Reload changed unit files, then stop and start the service"
                ),
            )
        }
        Requirement::Artifact {
            artifact,
            producer,
            value,
            current_inputs,
        } => {
            let valid = state.artifacts().get(artifact).is_some_and(|a| {
                a.producer == *producer
                    && a.value == *value
                    && (!*current_inputs
                        || (snapshot_current(state, producer, &a.snapshot)
                            && !matches!(
                                state.runtime().get(producer),
                                Some(Runtime::Failed { .. } | Runtime::Dispatched { .. })
                            )))
            });
            (
                valid,
                format!(
                    "{artifact}: required value from {producer}{}",
                    if *current_inputs {
                        " using its current saved/loaded definition and inputs"
                    } else {
                        ""
                    }
                ),
            )
        }
        Requirement::InstallLink {
            target,
            unit,
            present,
        } => (
            link_present(state, target, unit) == *present,
            format!(
                "{unit}: {} for future {target} sessions; enablement is separate from current running state",
                if *present { "enabled" } else { "not enabled" }
            ),
        ),
        Requirement::DefaultTarget { target } => (
            state.default_target() == target,
            format!(
                "The future default target must be {target}; selecting it does not switch the current runtime"
            ),
        ),
        Requirement::Dependency {
            unit,
            dependency,
            target,
            present,
        } => {
            let Some(text) = state.documents().get(unit) else {
                return (false, format!("Missing supplied unit {unit}"));
            };
            let d = match core::parse_unit(state.inventory(), unit, text) {
                Ok(d) => d,
                Err(e) => return (false, format!("Saved {unit}: {e}")),
            };
            let list = match dependency {
                DependencyKind::Requires => &d.requires,
                DependencyKind::Wants => &d.wants,
                DependencyKind::After => &d.after,
                DependencyKind::Before => &d.before,
            };
            (
                list.contains(target) == *present,
                format!(
                    "Saved {unit}: {dependency:?} {target} must be {}",
                    if *present { "present" } else { "absent" }
                ),
            )
        }
    }
}
fn preservation_checks(def: &Definition, state: &State) -> Vec<(bool, String)> {
    let mut out = vec![];
    let p = &def.preservation;
    let unit = p
        .unit_documents
        .iter()
        .find(|u| state.documents().get(*u) != def.initial.documents().get(*u));
    out.push((unit.is_none(),unit.map_or_else(||"Unaffected unit documents retained".into(),|u|format!("Keep the supplied {u} document unchanged; repair only the task's editable units"))));
    let config = p
        .configuration
        .iter()
        .find(|u| state.configuration().get(*u) != def.initial.configuration().get(*u));
    out.push((
        config.is_none(),
        config.map_or_else(
            || "Unaffected application configuration retained".into(),
            |u| format!("Keep the supplied {u} configuration unchanged"),
        ),
    ));
    if p.default_target {
        out.push((
            state.default_target() == def.initial.default_target(),
            "Keep the unaffected default target unchanged".into(),
        ));
    }
    if p.preserve_other_boot_links {
        let editable = p
            .editable_boot_links
            .iter()
            .map(|p| (&p.target, &p.unit))
            .collect::<BTreeSet<_>>();
        let wrong = state
            .inventory()
            .units
            .iter()
            .filter(|s| s.ends_with(".target"))
            .find_map(|target| {
                state
                    .inventory()
                    .units
                    .iter()
                    .find(|unit| {
                        !editable.contains(&(target, *unit))
                            && link_present(state, target, unit)
                                != link_present(&def.initial, target, unit)
                    })
                    .map(|unit| (target, unit))
            });
        out.push((
            wrong.is_none(),
            wrong.map_or_else(
                || "Unaffected future-session enablement retained".into(),
                |(t, u)| format!("Keep the unaffected {u} enablement at {t} unchanged"),
            ),
        ));
    }
    out
}
/// Derive a single fixed fresh session; reports returned for inspection are never
/// accepted back as grading input. Main state, draft and event history are untouched.
pub fn inspect_fresh(
    state: &State,
) -> Result<(State, core::Report, core::Report, core::StartOrdering), String> {
    let (mut fresh, start, ordering) =
        state.fresh_session_inspected().map_err(|e| e.to_string())?;
    let initialized = fresh
        .apply(&Action::AdvancePrograms {})
        .map_err(|e| e.to_string())?;
    Ok((fresh, start, initialized, ordering))
}
pub fn checks(def: &Definition, state: &State, draft: &Draft) -> Vec<(bool, String)> {
    if !def.same_assignment(state) || !draft.same_shape(state) {
        return vec![(
            false,
            "This workspace or draft differs from the supplied virtual-unit assignment".into(),
        )];
    }
    let mut out = vec![(
        draft.matches(state),
        "Editor changes must be saved before this configuration can be verified".into(),
    )];
    out.extend(def.requirements.iter().map(|r| check_requirement(state, r)));
    out.extend(preservation_checks(def, state));
    match inspect_fresh(state) {
        Ok((fresh, start, _initialized, ordering)) => {
            out.extend(def.fresh.requirements.iter().map(|r| {
                let (ok, message) = check_requirement(&fresh, r);
                (ok, format!("Fresh session: {message}"))
            }));
            for unit in &def.fresh.started {
                out.push((start.jobs.iter().any(|j|j.unit==*unit&&j.result.succeeded()),format!("Fresh session: {unit} must complete its start job; program outcomes are checked separately")));
            }
            for o in &def.fresh.ordering {
                out.push((ordering.precedes(&o.before,&o.after),format!("Fresh session: an actual transaction ordering path must place {} before {}; unrelated layer positions do not guarantee this",o.before,o.after)));
            }
        }
        Err(e) => out.push((false, format!("Fresh-session test: {e}"))),
    }
    out.into_iter()
        .map(|(p, s)| (p, bounded_message(s)))
        .collect()
}
pub fn solved(def: &Definition, state: &State, draft: &Draft) -> bool {
    checks(def, state, draft).iter().all(|c| c.0)
}

/// Pack authoring/reachability data only. It is not taught systemctl or shell syntax.
pub fn adapter(initial: &State, state: &mut State, input: &str) -> Result<String, String> {
    if input.len() > MAX_ACTION_BYTES {
        return Err("Virtual unit authoring action exceeds 1.5 MiB".into());
    }
    let raw = input
        .strip_prefix("unit-manager ")
        .ok_or("Use native virtual unit controls; this data adapter is authoring-only")?;
    let action: EditAction = serde_json::from_str(raw)
        .map_err(|_| "Invalid bounded virtual unit action data".to_owned())?;
    match apply(state, initial, &action)? {
        None => Ok("Virtual raw source saved; manager definitions, runtime and enablement remain separate.".into()),
        Some(report) => Ok(format!("Virtual manager action returned {} jobs ({} failed); inspect current state and future-session policy separately.", report.jobs.len(), report.jobs.iter().filter(|j| !j.result.succeeded()).count())),
    }
}
