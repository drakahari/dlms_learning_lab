//! Source context is mandatory; learner work never selects a host action.
use super::*;
pub const HELP: &str = "Use the native channel table to edit managed radios, then choose Check plan. This supplied 2.4 GHz / 20 MHz activity uses the site's 1 / 6 / 11 policy and finite survey relationships. Protected neighbours stay fixed. No scanning, radio activation, throughput measurement, IOS commands, host files or network traffic. Other bands, widths, regulations, SSIDs and encryption are outside this activity.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::WirelessChannels { simulation },
        crate::engine::Goal::WirelessChannels { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err(
            "Channel-planning state and objective must use the same supplied activity.".into(),
        );
    };
    definition.validate()?;
    if definition.initial.id != s.id {
        return Err("The independent channel-plan identifier must match its scenario.".into());
    }
    if **simulation != Session::new(definition) {
        return Err(
            "Channel planning must begin with separate declared assignment and example drafts."
                .into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::WirelessChannels { .. })
        || matches!(s.initial, crate::engine::State::WirelessChannels { .. })
        || matches!(s.goal, crate::engine::Goal::WirelessChannels { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::WirelessChannels { simulation } = &a.state else {
        return Err("Saved channel work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible channel-plan metadata; original retained.".into());
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
            return vec![(
                false,
                "Not checked since editing. Use the supplied surveys, then choose Check plan."
                    .into(),
            )];
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
    /// Fixed source reachability helpers only; never taught as real commands.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        self.validate(d)?;
        if raw.len() > 128 || raw.chars().any(char::is_control) {
            return Err("Use one bounded channel-plan source action.".into());
        }
        let mut next = self.clone();
        let words = raw.split_ascii_whitespace().collect::<Vec<_>>();
        let message = match words.as_slice() {
            ["@wireless-set", id, value] => { next.assignment.set(&d.initial,id,(*value).into())?; "Updated the supplied channel field; not checked." },
            ["@wireless-check"] => { next.assignment.check(&d.initial)?; "Checked the actual plan against every supplied survey." },
            _ => return Err("This activity uses native controls. Only bounded source reachability helpers are supported here; no host command runs.".into()),
        };
        next.validate(d)?;
        *self = next;
        Ok(message.into())
    }
}
