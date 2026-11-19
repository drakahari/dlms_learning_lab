//! Authored context and data-only actions, never host execution.
use super::*;
pub const HELP: &str = "Edit the virtual squid.conf, save it, validate its syntax and apply the saved policy. Inspect supplied request facts, then choose Check policy. Only src, port, method and ordered http_access rules are supported. No real proxy, network, authentication, cache or host command runs.";
pub const PREFIX: &str = "@proxy-policy ";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        config: String,
    },
    Save {},
    Validate {},
    Apply {},
    Check {},
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::ProxyPolicy { simulation },
        crate::engine::Goal::ProxyPolicy { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Proxy state and objectives must declare the same activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition)? {
        return Err("Proxy sources need matching identities and separate fresh, unchecked practice and assignment drafts.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::ProxyPolicy { .. })
        || matches!(s.initial, crate::engine::State::ProxyPolicy { .. })
        || matches!(s.goal, crate::engine::Goal::ProxyPolicy { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::ProxyPolicy { simulation } = &a.state else {
        return Err("Saved proxy work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible saved proxy metadata; original retained.".into());
    }
    simulation.validate(d)
}
impl Session {
    /// Pack reachability only; these serialized actions are not Squid or shell commands.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        self.validate(d)?;
        if input.len() > 65536 || input.chars().any(char::is_control) {
            return Err(
                "Source actions must be bounded single-line JSON, not terminal commands.".into(),
            );
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native proxy workspace; source actions are not host commands.")?;
        let action: Action = serde_json::from_str(raw).map_err(
            |_| "Unsupported proxy action or malformed source JSON; no changes applied.",
        )?;
        let mut next = self.clone();
        let t = &d.initial;
        let a = &mut next.assignment;
        let message = match action {
            Action::Edit { config } => {
                a.edit(config)?;
                "Updated the assignment editor; save and apply before checking.".into()
            }
            Action::Save {} => a.save(t)?,
            Action::Validate {} => a.validate_saved(t)?,
            Action::Apply {} => a.apply(t)?,
            Action::Check {} => {
                let r = a.check(t)?;
                if r.passed() {
                    "All policy requirements verified.".into()
                } else {
                    "Policy needs attention; inspect the individual request and policy feedback."
                        .into()
                }
            }
        };
        next.validate(d)?;
        *self = next;
        Ok(message)
    }
}
