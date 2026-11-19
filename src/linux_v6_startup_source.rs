//! Authored source/progress binding for the fixed virtual IPv6 loader.
use crate::{
    engine::{Goal, State},
    linux_v6_startup_workspace::{Action, Definition, Scope, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "v6-startup-workspace ";
pub const HELP: &str = "Offline IPv6 filter persistence. ip6tables [-t filter] -S/-A/-I/-D by number/-P/-F uses the existing numeric IPv6/ICMPv6 and TCP/UDP NEW/ESTABLISHED subset. ip6tables-save [-t filter] [-f /etc/lab-ipv6-filter.rules]; ip6tables-restore [--test|-t] /etc/lab-ipv6-filter.rules; cat of that exact virtual file. One complete *filter snapshot, three built-in chain policies with zero counters, supported -A rules, COMMIT and whole-line comments. Other tables, chains, counters, extension modules, shell redirection and module loading are unsupported. The editor can save unfinished syntax; a failed atomic load retains the previous current table. systemctl cat/status/is-active/is-enabled/enable/disable/start/restart/stop addresses only the supplied custom lab-ipv6-filter.service. This is not a universal distribution firewall unit. Enable selects startup, restart loads now, start of an active oneshot does not reload; stop has no ExecStop and retains rules. active (exited) proves a load, not the intended policy. Read-only supplied ip link show, ip -6 address show, ip -6 route show and sysctl KEY remain available. Native Run startup test is a simulator action: it starts with fresh ACCEPT rules, supplied persistent addresses/manual routes and source-owned settings, then runs the enabled provided loader. No full OS boot, systemd ordering, dynamic interface/RA lifecycle, real packets, host files/kernel/subprocesses or general shell. Current and startup full-range comparisons plus fresh sample conversations are graded. A traffic case starts with empty simulated direction state. INPUT/OUTPUT must be preserved; only the stated FORWARD policy may change. Examples are separate and ungraded. Files32KiB/256lines, commands512bytes,96configuration changes, bounded output and comparison regions.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::LinuxV6Startup { simulation }, Goal::LinuxV6Startup { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("IPv6 startup source and task must use the same workspace.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the IPv6 startup identifier, version and separate initial workspaces.".into(),
        );
    }
    if s.guide.len() > 8
        || s.guide.iter().any(|g| g.commands.len() > 8)
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 512 || c.command.starts_with(PREFIX) || c.simulator_only)
    {
        return Err("Teach bounded supported literal commands; native startup/traffic actions are not Linux commands.".into());
    }
    let mut reachable = vec![Scope::new(&definition.example)];
    let mut scope = reachable[0].clone();
    for action in &definition.worked {
        scope.reference(&definition.example, action)?;
        reachable.push(scope.clone());
    }
    for command in s.guide.iter().flat_map(|g| &g.commands) {
        // is-active/status/is-enabled legitimately return nonzero while inspecting
        // unfinished work. Capability validation must not pretend this is a parse error.
        if !reachable
            .iter()
            .any(|base| base.world.clone().command(&command.command).is_ok())
        {
            return Err(format!(
                "Unsupported or unreachable example command: {}",
                command.command
            ));
        }
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::LinuxV6Startup { simulation } = &a.state else {
        return Err(
            "Saved firewall startup work belongs to another activity; previous work retained."
                .into(),
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
        return Err("Saved firewall startup context differs from its lesson; previous work retained for recovery.".into());
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
        s.assignment
            .checks(&d.assignment)
            .into_iter()
            .map(|c| (c.passed, format!("{}: {}", c.label, c.detail))),
    );
    rows
}
impl Session {
    pub fn execute_v6_startup_reference(
        &mut self,
        d: &Definition,
        input: &str,
    ) -> Result<String, String> {
        if input.len() > 96 * 1024 {
            return Err("IPv6 startup authoring action exceeds its bound.".into());
        }
        let raw=input.strip_prefix(PREFIX).ok_or("Use the native firewall editor, terminal and tests; internal records are not learner commands.")?;
        let action: Action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid bounded IPv6 startup action: {e}"))?;
        d.bound(self)?;
        let mut next = self.clone();
        next.assignment.reference(&d.assignment, &action)?;
        d.bound(&next)?;
        *self = next;
        Ok(if matches!(action, Action::Check) {
            let rows = self.assignment.checks(&d.assignment);
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
