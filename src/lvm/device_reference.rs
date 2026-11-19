//! Separate simulated application configurations. All paths are virtual data;
//! the opt-in workspace binds actual readers/drafts without filesystem/process I/O.
use super::*;
pub const MAX_CONFIGURATION: usize = 8 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Caller {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub file: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub path: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "map::<_,Caller,4>")]
    pub callers: BTreeMap<String, Caller>,
}
pub fn valid_reference_text(s: &str) -> bool {
    s.len() <= device_path::MAX_PATH && !s.chars().any(char::is_control)
}
impl Configuration {
    /// Syntax/existence errors in a saved reference remain valid editable data.
    /// Structural limits, immutable caller facts and actual resolution are separate.
    pub fn validate(&self) -> Result<()> {
        if self.callers.is_empty()
            || self.callers.len() > 4
            || !crate::cron_workspace::within(self, MAX_CONFIGURATION)
            || self.callers.iter().any(|(key, c)| {
                !name(key)
                    || c.label.trim().is_empty()
                    || c.label.len() > 120
                    || c.label.chars().any(char::is_control)
                    || !file_name(&c.file)
                    || !valid_reference_text(&c.path)
            })
        {
            return Err(invalid(
                "Application configurations need1–4supplied callers, bounded labels/flat virtual filenames and single-line device-reference text",
            ));
        }
        Ok(())
    }
    pub fn same_roster(&self, initial: &Self) -> bool {
        self.validate().is_ok()
            && initial.validate().is_ok()
            && self.callers.keys().eq(initial.callers.keys())
            && initial.callers.iter().all(|(key, c)| {
                self.callers[key].label == c.label && self.callers[key].file == c.file
            })
    }
    /// Save only a bounded reference. A bad path is saved as a learner mistake,
    /// then reported by the actual reader; it never becomes a host command.
    pub fn save(&mut self, caller: &str, path: &str) -> Result<()> {
        self.validate()?;
        if !valid_reference_text(path) {
            return Err(limit(
                "Use one device reference up to256bytes without control characters; prior saved configuration retained",
            ));
        }
        if !self.callers.contains_key(caller) {
            return Err(error(
                ErrorKind::Unknown,
                "Select a supplied simulated application",
            ));
        }
        let mut next = self.clone();
        next.callers.get_mut(caller).unwrap().path = path.into();
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn inspect<'a>(&self, s: &'a State, caller: &str) -> Result<Read<'a>> {
        self.validate()?;
        let c = self
            .callers
            .get(caller)
            .ok_or_else(|| error(ErrorKind::Unknown, "No supplied application has that name"))?;
        if c.path.trim().is_empty() {
            return Err(invalid(
                "The saved device reference is blank; enter and save a device path for this application",
            ));
        }
        let resolved = device_path::resolve(s, &c.path)?;
        let canonical = device_path::canonical(&resolved.parts.group, &resolved.parts.volume)?;
        let key = format!("{}/{}", resolved.parts.group, resolved.parts.volume);
        let identity = s
            .identity
            .as_ref()
            .and_then(|ids| ids.volumes.get(&key))
            .copied();
        let content = s.read_file(&key, &c.file)?;
        Ok(Read {
            form: resolved.parts.form,
            canonical,
            identity,
            content,
        })
    }
}
#[derive(Debug)]
pub struct Read<'a> {
    pub form: device_path::Form,
    /// Internal observation for goals/optional explanations; do not auto-fill an
    /// independent answer with this computed value in the native workspace.
    pub canonical: String,
    /// Internal simulation object identity, never an actual device UUID.
    pub identity: Option<u32>,
    pub content: &'a str,
}
