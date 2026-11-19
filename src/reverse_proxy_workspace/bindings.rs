//! Authored context and data-only actions, never host execution.
use super::*;
pub const HELP: &str = "Edit the virtual server fragment, Save configuration, optionally Check syntax, then Apply saved configuration. Inspect supplied request paths and choose Check routing. Supports one explicit listen port, prefix/exact locations and literal return or IPv4 HTTP proxy_pass actions. No real Nginx, listener, HTTP transaction, backend server, host file or network operation runs.";
pub const PREFIX: &str = "@reverse-proxy ";
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
        crate::engine::State::ReverseProxy { simulation },
        crate::engine::Goal::ReverseProxy { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Reverse-proxy state and objectives must declare the same activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition)? {
        return Err("Reverse-proxy sources need matching identities and separate fresh, unchecked practice and assignment drafts.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::ReverseProxy { .. })
        || matches!(s.initial, crate::engine::State::ReverseProxy { .. })
        || matches!(s.goal, crate::engine::Goal::ReverseProxy { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::ReverseProxy { simulation } = &a.state else {
        return Err(
            "Saved reverse-proxy work belongs to another activity; original retained.".into(),
        );
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible saved reverse-proxy metadata; original retained.".into());
    }
    simulation.validate(d)
}
impl Session {
    /// Pack reachability only; these serialized actions are not Nginx or shell commands.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        self.validate(d)?;
        if input.len() > 65536 || input.chars().any(char::is_control) {
            return Err(
                "Source actions must be bounded single-line JSON, not terminal commands.".into(),
            );
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native routing workspace; source actions are not host commands.")?;
        let action: Action = serde_json::from_str(raw).map_err(
            |_| "Unsupported routing action or malformed source JSON; no changes applied.",
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
                    "All routing requirements verified.".into()
                } else {
                    "Routing needs attention; inspect the individual request and routing feedback."
                        .into()
                }
            }
        };
        next.validate(d)?;
        *self = next;
        Ok(message)
    }
}
