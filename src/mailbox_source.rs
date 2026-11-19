//! Bind mail-access state to its authored lesson and native workspace.
use crate::{
    engine::{Goal, State},
    mailbox_workspace::{self as work, Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "mailbox-workspace ";
pub const HELP: &str = "Use the simulated Dovecot editor and IMAP/POP client. Save, restart and enable are explicit simulator actions. Protocol commands stay in process; no shell, host files, real credentials or network connections.";
pub const ADMIN_HELP: &str = "Use the simulated server editor and administration command field. Supported: doveconf [-h] named-settings; doveadm mailbox list/search/fetch/expunge for one supplied account. No shell expansion, real files, daemons or sockets. Read the subset before using other flags.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (simulation, definition) = match (&s.initial, &s.goal) {
        (State::MailboxAccess { simulation }, Goal::MailboxAccess { definition })
        | (
            State::MailboxAdministration { simulation },
            Goal::MailboxAdministration { definition },
        ) => (simulation, definition),
        _ => return Err("Mail-access source and workspace must agree.".into()),
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the mail-access source identifier, version and separate initial workspaces."
                .into(),
        );
    }
    if s.guide.len() > 8
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 2048 || c.command.starts_with(PREFIX))
    {
        return Err("Mail-access guidance must use bounded protocol examples, not internal authoring actions.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let simulation = match (&a.state, &s.initial) {
        (State::MailboxAccess { simulation }, State::MailboxAccess { .. })
        | (State::MailboxAdministration { simulation }, State::MailboxAdministration { .. }) => {
            simulation
        }
        _ => return Err("Saved work belongs to another activity; original retained.".into()),
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved mail-access context differs; retain the original for recovery.".into());
    }
    d.bound(simulation)
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.execute_bound_reference(d, input, false)
    }
    pub fn execute_admin_reference(
        &mut self,
        d: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.execute_bound_reference(d, input, true)
    }
    fn execute_bound_reference(
        &mut self,
        d: &Definition,
        input: &str,
        admin: bool,
    ) -> Result<String, String> {
        if input.len() > work::MAX_ACTION + PREFIX.len() {
            return Err("Mail-access authoring action exceeds its bounded size.".into());
        }
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native mail-access workspace; this is not a host command shell.")?;
        let a: Action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid bounded mail-access action: {e}"))?;
        if matches!(a, Action::Admin { .. }) && !admin {
            return Err("Administration commands belong to the separate administration activity; protocol lessons retain their client workspace.".into());
        }
        let rows = self.action(d, &a, false)?;
        Ok(if matches!(a, Action::Check) {
            format!(
                "{}/{} required outcomes pass.",
                rows.iter().filter(|r| r.passed).count(),
                rows.len()
            )
        } else {
            self.assignment.output.clone()
        })
    }
}
