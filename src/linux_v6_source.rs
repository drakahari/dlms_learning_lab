//! Authored router-state binding. No command in this module reaches a host shell.
use crate::{
    engine::{Goal, State},
    linux_v6_workspace::{Action, Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "v6-workspace ";
pub const HELP: &str = "Simulated IPv6 header and TCP/UDP direction-state practice. ip6tables [-t filter] -S/-A/-I/-D by number/-P/-F; numeric IPv6 prefixes, exact interfaces, TCP/UDP ports, numeric --icmpv6-type type[/code]. State rules require -p tcp or -p udp plus -m conntrack --ctstate NEW, ESTABLISHED or their comma-list union. ICMPv6 is stateless; RELATED, NAT, fragments, NDP and actual TCP behavior are not modeled. This activity edits FORWARD only; router-local INPUT/OUTPUT, routes and interfaces stay supplied. ip -6 route show and sysctl net.ipv6.conf.all.forwarding inspect the fixed environment. Run traffic test starts its own sample inputs and empty simulated connection table. An allowed header is not proof of a working neighbor, session or application. No host shell, filesystem, service, socket, packet or kernel change.";

pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (simulation, definition) = match (&s.initial, &s.goal) {
        (State::LinuxV6 { simulation }, Goal::LinuxV6 { definition }) => (simulation, definition),
        _ => return Err("Flow source and task must use the same workspace.".into()),
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the flow lesson identifier, version and independent initial workspaces.".into(),
        );
    }
    if s.guide.len() > 8
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 2048 || c.command.starts_with(PREFIX) || c.simulator_only)
    {
        return Err(
            "Flow command examples must be bounded supported commands, not internal actions."
                .into(),
        );
    }
    for command in s.guide.iter().flat_map(|g| &g.commands) {
        let mut probe = definition.example.initial.clone();
        definition
            .example
            .command(&mut probe, &command.command)
            .map_err(|e| format!("Unsupported worked command: {}", e.message))?;
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::LinuxV6 { simulation } = &a.state else {
        return Err("Flow progress belongs to another activity; original retained.".into());
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
            "Saved flow context differs from its lesson; original retained for recovery.".into(),
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
    pub fn execute_v6_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        if input.len() > 256 * 1024 {
            return Err("Flow authoring action exceeds its bound.".into());
        }
        let raw = input.strip_prefix(PREFIX).ok_or("Use the native flow terminal and traffic controls; internal actions are not learner commands.")?;
        let a: Action =
            serde_json::from_str(raw).map_err(|e| format!("Invalid bounded flow action: {e}"))?;
        // Author recipes must not pass merely because an unsupported command was logged.
        if let Action::Command { line } = &a {
            let mut state = self.assignment.state.clone();
            d.assignment
                .command(&mut state, line)
                .map_err(|e| e.message)?;
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
