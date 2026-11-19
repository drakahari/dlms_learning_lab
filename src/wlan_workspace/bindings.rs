//! Source-bound configuration work; no learner-host execution.
use super::*;
pub const HELP: &str = "Use the native WLAN configuration controls and Check configuration. This bounded model compares supplied network names, personal-security settings, management-frame protection and fictional training passphrases with supplied client profiles. Configuration compatibility is not a real radio connection or cryptographic handshake. No scanning, encryption, host files, processes or network traffic. Transition/enterprise/legacy WPA, hex keys and arbitrary Unicode SSIDs are outside this activity.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::WlanProfiles { simulation },
        crate::engine::Goal::WlanProfiles { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("WLAN state and objectives must use the same supplied activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id {
        return Err("The independent WLAN task identifier must match its scenario.".into());
    }
    if **simulation != Session::new(definition) {
        return Err(
            "WLAN work must begin with separate declared assignment and example drafts.".into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::WlanProfiles { .. })
        || matches!(s.initial, crate::engine::State::WlanProfiles { .. })
        || matches!(s.goal, crate::engine::Goal::WlanProfiles { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::WlanProfiles { simulation } = &a.state else {
        return Err("Saved WLAN work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible WLAN progress metadata; original retained.".into());
    }
    simulation.validate(d)
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
            return vec![(false,"Not checked since editing. Review the supplied client profiles, then choose Check configuration.".into())];
        }
        match task.rows(&draft.fields) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.correct, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
impl Session {
    /// Fixed declarative-source reachability helpers, never taught as IOS commands.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        self.validate(d)?;
        if raw.len() > 1024 || raw.chars().any(char::is_control) {
            return Err("Use one bounded WLAN source action.".into());
        }
        let mut next = self.clone();
        let message = if raw == "@wlan-check" {
            next.assignment.check(&d.initial)?;
            "Checked the current configuration against every supplied policy and client profile."
        } else if let Some(rest) = raw.strip_prefix("@wlan-set ") {
            let (id, json) = rest.split_once(' ').ok_or(
                "A source WLAN action needs its profile and bounded configuration object.",
            )?;
            let value: Configuration = serde_json::from_str(json).map_err(|_| {
                "Use a bounded supported WLAN configuration object; no host command runs."
                    .to_string()
            })?;
            next.assignment.set(&d.initial, id, value)?;
            "Updated the managed configuration; not checked."
        } else {
            return Err("This activity uses native configuration controls. Only fixed source reachability helpers are supported here; no host command runs.".into());
        };
        next.validate(d)?;
        *self = next;
        Ok(message.into())
    }
}
