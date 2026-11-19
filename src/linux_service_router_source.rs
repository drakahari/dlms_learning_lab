//! Authored router-state binding. No command in this module reaches a host shell.
use crate::{
    engine::{Goal, State},
    linux_service_router::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "services-router-workspace ";
pub const HELP: &str = "Simulated /etc/services editor and IPv6 router. Save edits before lookup or rule creation; Save does not reload installed numeric rules. cat /etc/services and getent services [NAME|PORT][/tcp|udp] read the saved virtual file. TCP and UDP names/aliases are separate and case-sensitive. This activity also supports the documented ip6tables FORWARD subset, with one named or numeric --sport/--dport under explicit -p tcp/6 or udp/17. Stored rules use resolved numbers. ip6tables -S shows current numbers; delete/replace an old permit after fixing its mapping. Route and named forwarding reads are supplied, not editable. Unsupported syntax is reported; no host file, NSS lookup, subprocess, real service or packet is used. Run traffic test starts separate sample inputs. Editing a draft clears old feedback; Check work evaluates both saved mappings and the current whole-range traffic policy.";

pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (simulation, definition) = match (&s.initial, &s.goal) {
        (State::LinuxServiceRouter { simulation }, Goal::LinuxServiceRouter { definition }) => {
            (simulation, definition)
        }
        _ => return Err("Services/router source and task must use the same workspace.".into()),
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the services/router lesson identifier, version and independent initial workspaces.".into(),
        );
    }
    if s.guide.len() > 8
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 2048 || c.command.starts_with(PREFIX) || c.simulator_only)
    {
        return Err(
            "Services/router command examples must be bounded supported commands, not internal actions."
                .into(),
        );
    }
    for command in s.guide.iter().flat_map(|g| &g.commands) {
        let mut probe = crate::linux_service_router::Scope::new(&definition.example)?;
        probe
            .reference(
                &definition.example,
                &Action::Router {
                    action: crate::linux_v6_workspace::Action::Command {
                        line: command.command.clone(),
                    },
                },
            )
            .map_err(|e| format!("Unsupported worked command: {e}"))?;
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::LinuxServiceRouter { simulation } = &a.state else {
        return Err(
            "Services/router progress belongs to another activity; original retained.".into(),
        );
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
        return Err(
            "Saved services/router context differs from its lesson; original retained for recovery.".into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = d.bound(s) {
        return vec![(false, e)];
    }
    let mut rows = vec![(
        s.assignment.router.checked,
        "Check the current assignment after making changes.".into(),
    )];
    rows.extend(
        d.assignment
            .checks(&s.assignment)
            .into_iter()
            .map(|c| (c.passed, format!("{}: {}", c.label, c.detail))),
    );
    rows
}
impl Session {
    pub fn execute_service_router_reference(
        &mut self,
        d: &Definition,
        input: &str,
    ) -> Result<String, String> {
        if input.len() > 256 * 1024 {
            return Err("Services/router authoring action exceeds its bound.".into());
        }
        let raw = input.strip_prefix(PREFIX).ok_or("Use the native services/router terminal and traffic controls; internal actions are not learner commands.")?;
        let a: Action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid bounded services/router action: {e}"))?;
        d.bound(self)?;
        let mut next = self.clone();
        next.assignment.reference(&d.assignment, &a)?;
        d.bound(&next)?;
        *self = next;
        Ok(
            if matches!(
                a,
                Action::Router {
                    action: crate::linux_v6_workspace::Action::Check
                }
            ) {
                let rows = d.assignment.checks(&self.assignment);
                format!(
                    "{}/{} required outcomes pass.",
                    rows.iter().filter(|c| c.passed).count(),
                    rows.len()
                )
            } else {
                self.assignment.router.output.clone()
            },
        )
    }
}
