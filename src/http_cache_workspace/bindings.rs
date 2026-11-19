//! Catalog-aware context and bounded data actions; never HTTP or shell execution.
use super::*;
pub const PREFIX: &str = "@http-cache ";
pub const HELP: &str = "Edit the supplied virtual origin's Cache-Control values, then Check caching policy. Each trace starts with its own sample inputs and an empty simulated cache. Inspect actual stored, downloaded and validated responses. No real HTTP, browser, Squid, host command or network request runs.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Header,8>")]
        headers: Vec<Header>,
    },
    Check {},
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::HttpCache { simulation },
        crate::engine::Goal::HttpCache { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Cache state and requirements must declare the same authored activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition)? {
        return Err("Cache sources need matching identities and separate fresh, unchecked example and assignment drafts.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::HttpCache { .. })
        || matches!(s.initial, crate::engine::State::HttpCache { .. })
        || matches!(s.goal, crate::engine::Goal::HttpCache { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::HttpCache { simulation } = &a.state else {
        return Err("Saved cache work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible saved cache metadata; original retained.".into());
    }
    simulation.validate(d)
}
impl Session {
    /// Declarative reachability helpers; not real HTTP or Linux commands.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        self.validate(d)?;
        if input.len() > 65536 || input.chars().any(char::is_control) {
            return Err(
                "Source actions must be bounded single-line JSON, not terminal commands.".into(),
            );
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native cache workspace; source actions are not host commands.")?;
        let action: Action = serde_json::from_str(raw).map_err(
            |_| "Unsupported cache action or malformed source JSON; no changes applied.",
        )?;
        let mut next = self.clone();
        let message = match action {
            Action::Edit { headers } => {
                next.assignment.edit(&d.initial, headers)?;
                "Updated assignment headers; choose Check caching policy.".into()
            }
            Action::Check {} => {
                if next.assignment.check(&d.initial)?.passed() {
                    "All declared cache requirements verified.".into()
                } else {
                    "Caching policy needs attention; inspect the actual request feedback.".into()
                }
            }
        };
        next.validate(d)?;
        *self = next;
        Ok(message)
    }
}
