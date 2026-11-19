//! Authored router-state binding. No command in this module reaches a host shell.
use crate::{
    engine::{Goal, State},
    linux_router_workspace::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "router-workspace ";
pub const HELP: &str = "Simulated IPv4 router: iptables [-t filter] -S/-A/-I/-D/-P/-F; ip link show/set; ip -4 address show; ip -4 route show/add/replace/del PREFIX via GATEWAY dev NAME [metric N]; named sysctl reads, -w net.ipv4.ip_forward=0|1 and -p /etc/sysctl.d/lab-forwarding.conf; fixed-file cat; iptables-save [-t filter] [-f /etc/lab-filter.rules]; iptables-restore [--test] /etc/lab-filter.rules. Numeric IPv4, literal words, no shell expansion. Trace policy is a read-only simulator analysis, not a Linux command or real packet capture. No NAT, connection tracking, IPv6, host files, sockets or kernel changes.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (simulation, definition) = match (&s.initial, &s.goal) {
        (State::LinuxRouter { simulation }, Goal::LinuxRouter { definition }) => {
            (simulation, definition)
        }
        _ => return Err("Router source and task must use the same workspace.".into()),
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the router lesson identifier, version and independent initial workspaces."
                .into(),
        );
    }
    if s.guide.len() > 8
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 2048 || c.command.starts_with(PREFIX) || c.simulator_only)
    {
        return Err(
            "Router command examples must be bounded supported commands, not internal actions."
                .into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::LinuxRouter { simulation } = &a.state else {
        return Err("Router progress belongs to another activity; original retained.".into());
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
            "Saved router context differs from its lesson; original retained for recovery.".into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = d.bound(s) {
        return vec![(false, e)];
    }
    let mut rows = vec![(
        s.assignment.checked,
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
    pub fn execute_router_reference(
        &mut self,
        d: &Definition,
        input: &str,
    ) -> Result<String, String> {
        if input.len() > 256 * 1024 {
            return Err("Router authoring action exceeds its bound.".into());
        }
        let raw = input.strip_prefix(PREFIX).ok_or("Use the native router terminal and editors; internal actions are not learner commands.")?;
        let a: Action =
            serde_json::from_str(raw).map_err(|e| format!("Invalid bounded router action: {e}"))?;
        // Author recipes must not pass merely because an unsupported command was logged.
        if let Action::Command { line } = &a {
            let mut state = self.assignment.state.clone();
            state.command(line).map_err(|e| e.message)?;
        }
        self.action(d, &a, false)?;
        Ok(if matches!(a, Action::Check) {
            let rows = d.assignment.checks(&self.assignment);
            format!(
                "{}/{} required outcomes pass.",
                rows.iter().filter(|c| c.passed).count(),
                rows.len()
            )
        } else {
            self.assignment.output.clone()
        })
    }
}
