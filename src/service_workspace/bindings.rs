//! Data-only native actions; never a terminal or host-shell command language.
use super::*;
pub const HELP: &str = names::HELP;
pub const PREFIX: &str = "@service-clients ";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Configure {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        client: String,
        protocol: net::Protocol,
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        service: String,
    },
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        database: String,
    },
    Save {},
    Check {},
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::ServiceClients { simulation },
        crate::engine::Goal::ServiceClients { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Service state and objectives must declare the same activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition)? {
        return Err(
            "Service sources require matching identities and separate fresh, unchecked drafts."
                .into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::ServiceClients { simulation } = &a.state else {
        return Err("Saved service work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible saved service metadata; original retained.".into());
    }
    simulation.validate(d)
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        self.validate(d)?;
        if input.len() > 200000 || input.chars().any(char::is_control) {
            return Err("Source actions need bounded single-line JSON, not shell commands.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native service workspace; source actions are not host commands.")?;
        let action: Action = serde_json::from_str(raw)
            .map_err(|_| "Unsupported service action or malformed JSON; no changes applied.")?;
        let mut next = self.clone();
        let t = &d.initial;
        let a = &mut next.assignment;
        let result = match action {
            Action::Configure {
                client,
                protocol,
                service,
            } => {
                a.configure(t, &client, protocol, service)?;
                "Updated the virtual client; check its destination and behavior."
            }
            Action::Edit { database } => {
                a.edit(t, database)?;
                "Edited the virtual file; save before checking."
            }
            Action::Save {} => {
                a.save(t)?;
                "Saved the virtual services file. Server listeners have not changed."
            }
            Action::Check {} => {
                if a.check(t)?.passed() {
                    "All service requirements verified."
                } else {
                    "Service checks need attention; read the case feedback."
                }
            }
        }
        .to_string();
        next.validate(d)?;
        *self = next;
        Ok(result)
    }
}
