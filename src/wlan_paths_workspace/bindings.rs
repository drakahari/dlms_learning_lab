//! Draft source binding; only in-process typed configuration edits, never IOS/shell.
use super::*;
pub const HELP: &str = "Use native AP/WLAN and switch-port controls, then Check paths. This supplied-VLAN model compares management transport and local/central client-frame paths. It does not join an actual controller, transmit packets, authenticate clients or run host commands. Routed CAPWAP, radio behavior, certificates, DHCP/Internet, LAG and other AP modes are outside this activity.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::WlanPaths { simulation },
        crate::engine::Goal::WlanPaths { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("AP-path state and objectives must use the same declared activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition) {
        return Err("AP-path tasks must start with matching identities and separate unchecked example/assignment drafts.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::WlanPaths { .. })
        || matches!(s.initial, crate::engine::State::WlanPaths { .. })
        || matches!(s.goal, crate::engine::Goal::WlanPaths { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::WlanPaths { simulation } = &a.state else {
        return Err("Saved AP-path work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible AP-path progress metadata; original retained.".into());
    }
    simulation.validate(d)
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ReferenceEdit {
    Mode {
        value: ApMode,
    },
    Native {
        value: String,
    },
    Forwarding {
        wlan: String,
        value: Forwarding,
    },
    LocalVlan {
        wlan: String,
        value: String,
    },
    ControllerVlan {
        wlan: String,
        value: String,
    },
    Port {
        switch: String,
        port: String,
        settings: switching_workspace::PortDraft,
    },
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let (draft, task) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !draft.checked() {
            return vec![(false,"Not checked since editing. Compare the supplied topology and requirements, then choose Check paths.".into())];
        }
        task.checks(&draft.fields)
            .into_iter()
            .map(|c| {
                (
                    c.correct && c.evaluated,
                    format!("{}: {}", c.label, c.feedback),
                )
            })
            .collect()
    }
}
impl Session {
    pub fn validate(&self, def: &Definition) -> Result<()> {
        def.validate()?;
        if !self.valid(def) {
            return Err(
                "Saved AP-path drafts do not match this activity; original retained.".into(),
            );
        }
        Ok(())
    }
    /// Internal pack reachability only. Not an IOS/Linux command language.
    pub fn execute_reference(&mut self, def: &Definition, raw: &str) -> Result<String> {
        self.validate(def)?;
        if raw.len() > 1024 || raw.chars().any(char::is_control) {
            return Err("Use one bounded AP-path source action.".into());
        }
        let mut next = self.clone();
        let message = if raw == "@ap-check" {
            next.assignment.check(&def.initial)?;
            "Checked current configuration and every supplied forwarding requirement."
        } else if let Some(json) = raw.strip_prefix("@ap-edit ") {
            let edit: ReferenceEdit = serde_json::from_str(json).map_err(|_| {
                "Use a supported bounded AP-path source object; no host command runs.".to_string()
            })?;
            let fields = &mut next.assignment.fields;
            match edit {
                ReferenceEdit::Mode { value } => fields.mode = value,
                ReferenceEdit::Native { value } => fields.native = value,
                ReferenceEdit::Port {
                    switch,
                    port,
                    settings,
                } => {
                    let row = fields
                        .ports
                        .switches
                        .get_mut(&switch)
                        .and_then(|s| s.ports.get_mut(&port))
                        .ok_or("Choose a supplied switch attachment.")?;
                    *row = settings;
                }
                ReferenceEdit::Forwarding { wlan, value } => {
                    fields
                        .wlans
                        .iter_mut()
                        .find(|w| w.id == wlan)
                        .ok_or("Choose a supplied WLAN.")?
                        .forwarding = value;
                }
                ReferenceEdit::LocalVlan { wlan, value } => {
                    fields
                        .wlans
                        .iter_mut()
                        .find(|w| w.id == wlan)
                        .ok_or("Choose a supplied WLAN.")?
                        .local_vlan = value;
                }
                ReferenceEdit::ControllerVlan { wlan, value } => {
                    fields
                        .wlans
                        .iter_mut()
                        .find(|w| w.id == wlan)
                        .ok_or("Choose a supplied WLAN.")?
                        .controller_vlan = value;
                }
            }
            def.initial.world(fields)?;
            next.assignment.edited();
            "Updated the allowed AP-path configuration; not checked."
        } else {
            return Err("This activity uses native configuration controls. Only fixed source reachability actions are supported here; no host command runs.".into());
        };
        next.validate(def)?;
        *self = next;
        Ok(message.into())
    }
}
