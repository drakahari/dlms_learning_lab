//! Bounded drafts and typed reference practice, bound by the optional source/
//! session profile. No host paths, processes or commands.
use super::device_reference as goals;
use super::*;
use crate::lvm::device_reference as config;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub initial: config::Configuration,
    pub plan: goals::Plan,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    SaveReference {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        caller: String,
        #[serde(deserialize_with = "crate::cron::text::<_,256>")]
        path: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,24>")]
    pub reference: Vec<Operation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Practice {
    pub context: Context,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,12>")]
    pub worked: Vec<Step>,
    pub experiment: Operation,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,12>")]
    pub repair: Vec<Operation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft(#[serde(deserialize_with = "crate::cron::text::<_,256>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Live {
    pub configuration: config::Configuration,
    #[serde(deserialize_with = "draft_map")]
    pub drafts: std::collections::BTreeMap<String, Draft>,
}
fn draft_map<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<std::collections::BTreeMap<String, Draft>, D::Error> {
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = std::collections::BTreeMap<String, Draft>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most four unique bounded reference drafts")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error;
            #[derive(Deserialize)]
            struct Key(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
            let mut out = std::collections::BTreeMap::new();
            while let Some(Key(key)) = a.next_key()? {
                if out.len() >= 4 || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Reference draft count/duplicate before value",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries)
}
impl Context {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.plan.validate(initial, &self.initial)
    }
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
impl Operation {
    pub fn shape(&self, c: &Context) -> Result<(), String> {
        match self {
            Self::Command { text } => lvm::validate_command(text).map_err(|e| e.to_string()),
            Self::SaveReference { caller, path } => {
                if !c.initial.callers.contains_key(caller) || !config::valid_reference_text(path) {
                    return Err(
                        "Save only a supplied application's bounded single-line reference".into(),
                    );
                }
                Ok(())
            }
        }
    }
    pub fn description(&self, c: &Context) -> String {
        match self {
            Self::Command { text } => text.clone(),
            Self::SaveReference { caller, path } => format!(
                "Simulator configuration save · {}: {path}",
                c.initial
                    .callers
                    .get(caller)
                    .map_or(caller.as_str(), |v| v.label.as_str())
            ),
        }
    }
}
impl Live {
    pub fn new(c: &Context) -> Result<Self, String> {
        c.initial.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            configuration: c.initial.clone(),
            drafts: c
                .initial
                .callers
                .iter()
                .map(|(k, v)| (k.clone(), Draft(v.path.clone())))
                .collect(),
        })
    }
    pub fn validate(
        &self,
        c: &Context,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Result<(), String> {
        c.shape(initial)?;
        if !goals::same_context(initial, current, &c.initial, &self.configuration)
            || self.drafts.keys().ne(c.initial.callers.keys())
            || self
                .drafts
                .values()
                .any(|v| !config::valid_reference_text(&v.0))
            || !bounded::within(self, 16 * 1024)
        {
            return Err("Reference drafts/configuration differ from their supplied context or bounds; previous work retained".into());
        }
        Ok(())
    }
    pub fn set_draft(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &lvm::State,
        caller: &str,
        text: String,
    ) -> Result<(), String> {
        self.validate(c, initial, current)?;
        if !self.drafts.contains_key(caller) || !config::valid_reference_text(&text) {
            return Err("Use one supplied reference field up to256bytes; prior draft and saved configuration retained".into());
        }
        let mut next = self.clone();
        next.drafts.insert(caller.into(), Draft(text));
        next.validate(c, initial, current)?;
        *self = next;
        Ok(())
    }
    pub fn checks(
        &self,
        c: &Context,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if self.validate(c, initial, current).is_err() {
            return vec![(
                false,
                "Reference work does not match its source context".into(),
            )];
        }
        let mut rows = goals::outcomes(initial, current, &c.initial, &self.configuration, &c.plan);
        rows.push((self.drafts.iter().all(|(key,d)|d.0==self.configuration.callers[key].path),"Save edited references before checking the assignment; displayed drafts and saved application configuration must agree".into()));
        rows
    }
    /// Transaction over the actual virtual storage plus saved config and draft.
    /// Reference commands run only against State::execute, never a host process.
    pub fn apply(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &mut lvm::State,
        op: &Operation,
    ) -> Result<String, String> {
        self.validate(c, initial, current)?;
        op.shape(c)?;
        let mut storage = current.clone();
        let mut next = self.clone();
        let output = match op {
            Operation::Command { text } => storage.execute(text).map_err(|e| e.to_string())?,
            Operation::SaveReference { caller, path } => {
                next.configuration
                    .save(caller, path)
                    .map_err(|e| e.to_string())?;
                next.drafts.insert(caller.clone(), Draft(path.clone()));
                match next.configuration.inspect(&storage, caller) {
                    Ok(_) => format!(
                        "Saved simulated application reference for {}. Its actual virtual data is readable; check the requested target and reference form.",
                        next.configuration.callers[caller].label
                    ),
                    Err(e) => format!(
                        "Saved simulated application reference for {}. Reader needs attention: {e}",
                        next.configuration.callers[caller].label
                    ),
                }
            }
        };
        model(&storage)?;
        next.validate(c, initial, &storage)?;
        *current = storage;
        *self = next;
        Ok(output)
    }
    pub fn save_draft(
        &mut self,
        c: &Context,
        initial: &lvm::State,
        current: &mut lvm::State,
        caller: &str,
    ) -> Result<String, String> {
        let path = self
            .drafts
            .get(caller)
            .ok_or("No supplied reference draft")?
            .0
            .clone();
        self.apply(
            c,
            initial,
            current,
            &Operation::SaveReference {
                caller: caller.into(),
                path,
            },
        )
    }
}
impl Task {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.context.shape(initial)?;
        if self.reference.is_empty() || self.reference.len() > 24 {
            return Err("Reference assignment needs1–24bounded reference operations".into());
        }
        for op in &self.reference {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    pub fn validate(&self, initial: &lvm::State) -> Result<(), String> {
        self.shape(initial)?;
        let mut current = initial.clone();
        let mut live = Live::new(&self.context)?;
        if all(&live.checks(&self.context, initial, &current)) {
            return Err("Reference assignment must require real work".into());
        }
        for op in &self.reference {
            live.apply(&self.context, initial, &mut current, op)?;
        }
        if !all(&live.checks(&self.context, initial, &current)) {
            return Err("Reference solution misses actual application/storage outcomes".into());
        }
        Ok(())
    }
}
impl Practice {
    pub fn shape(&self, initial: &lvm::State) -> Result<(), String> {
        self.context.shape(initial)?;
        if self.worked.is_empty()
            || self.worked.len() > 12
            || self.repair.is_empty()
            || self.repair.len() > 12
        {
            return Err(
                "Reference practice needs bounded worked steps, a real experiment and repair"
                    .into(),
            );
        }
        for step in &self.worked {
            if step.explanation.trim().is_empty()
                || step.explanation.len() > 800
                || step
                    .explanation
                    .chars()
                    .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
            {
                return Err("Every reference worked step needs a clear bounded explanation".into());
            }
            step.operation.shape(&self.context)?;
        }
        self.experiment.shape(&self.context)?;
        for op in &self.repair {
            op.shape(&self.context)?;
        }
        Ok(())
    }
    pub fn validate(&self, initial: &lvm::State) -> Result<(), String> {
        self.shape(initial)?;
        let mut current = initial.clone();
        let mut live = Live::new(&self.context)?;
        if all(&live.checks(&self.context, initial, &current)) {
            return Err("Reference practice must start incomplete".into());
        }
        for step in &self.worked {
            live.apply(&self.context, initial, &mut current, &step.operation)?;
        }
        if !all(&live.checks(&self.context, initial, &current)) {
            return Err("Worked reference practice misses its outcomes".into());
        }
        let before = (current.clone(), live.configuration.clone());
        live.apply(&self.context, initial, &mut current, &self.experiment)?;
        if before == (current.clone(), live.configuration.clone())
            || all(&live.checks(&self.context, initial, &current))
        {
            return Err(
                "Reference exploration must expose a real changed incomplete outcome".into(),
            );
        }
        for op in &self.repair {
            live.apply(&self.context, initial, &mut current, op)?;
        }
        if !all(&live.checks(&self.context, initial, &current)) {
            return Err("Reference repair does not restore its actual outcomes".into());
        }
        Ok(())
    }
}
