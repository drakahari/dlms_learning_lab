//! Source/engine/progress binding for the opt-in IPv6 administration workspace.
use crate::{
    engine::{Goal, State},
    linux_v6_admin_workspace::{Action, Definition, Scope, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "v6-admin-workspace ";
pub const HELP: &str = "Offline IPv6 router configuration. Supported literal host forms: ip -6 address show [dev NAME]; ip -6 address add|del ADDRESS/PREFIX dev NAME; ip -6 route show; ip -6 route add|replace|del PREFIX via GATEWAY dev NAME metric N; ip link show. One documentation/locally-assigned ULA primary /48–/126 per supplied interface; remove its manual gateway routes before deleting that address. No address replace/flush or extra addresses. IPv6 link-local next hops require their explicit dev scope. Supported supplied settings: sysctl KEY, sysctl -w KEY=VALUE, and cat of one virtual /proc/sys/net/ipv6/conf/... file. Global forwarding, per-interface host/router and force_forwarding flags, accept_ra 0/1/2 follow the declared Linux 6.17 profile. A supplied already-received advertisement is observed through a clearly labeled native simulator action; it is not a Linux command. No general RA parser, SLAAC, NDP, clock/expiry, ECMP or packets. Learned defaults use fixed metric 1024/medium preference. Unsupported learned-route editing/collisions fail explicitly. File-enabled lessons also provide the single simulated /etc/sysctl.d/70-learning-lab-ipv6.conf editor, cat of that saved file and sysctl -p for that exact file. Native Save validates the small settings subset before loading, not full procps partial-error behavior. The scoped restart test assumes the supplied network is already persistent; it resets flags/learned routes, loads this file and observes supplied messages. It is not a full Linux boot or firewall persistence model. 512-byte commands, eight combined gateway routes, four learned routers and bounded histories/output; no host files, subprocesses, network or privileges. Insert fills input without executing. Check compares full route regions, supplied physical facts and actual configuration; examples remain separate and ungraded.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (simulation, definition) = match (&s.initial, &s.goal) {
        (
            State::LinuxV6Administration { simulation },
            Goal::LinuxV6Administration { definition },
        ) => (simulation, definition),
        _ => return Err("IPv6 configuration source and task must use the same workspace.".into()),
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the IPv6 configuration identifier/version and separate initial workspaces."
                .into(),
        );
    }
    if s.guide.len() > 8
        || s.guide.iter().any(|g| g.commands.len() > 8)
        || s.guide
            .iter()
            .flat_map(|g| &g.commands)
            .any(|c| c.command.len() > 512 || c.command.starts_with(PREFIX) || c.simulator_only)
    {
        return Err("Command examples must use bounded supported real command forms; native observations are not Linux commands.".into());
    }
    // Adding an address or deleting a route can require an earlier action. Validate
    // each taught command at an actual reachable prefix of the separate example,
    // while still refusing every unsupported command before curriculum admission.
    let mut reachable = vec![Scope::new(&definition.example)];
    let mut scope = reachable[0].clone();
    for action in &definition.worked {
        scope.reference(&definition.example, action)?;
        reachable.push(scope.clone());
    }
    for command in s.guide.iter().flat_map(|g| &g.commands) {
        if !reachable.iter().any(|base| {
            base.clone()
                .reference(
                    &definition.example,
                    &Action::Command {
                        input: command.command.clone(),
                    },
                )
                .is_ok()
        }) {
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
    let State::LinuxV6Administration { simulation } = &a.state else {
        return Err(
            "IPv6 configuration progress belongs to another activity; previous work retained."
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
        return Err("Saved IPv6 configuration context differs from its lesson; previous work retained for recovery.".into());
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
    pub fn execute_v6_admin_reference(
        &mut self,
        d: &Definition,
        input: &str,
    ) -> Result<String, String> {
        if input.len() > 16384 {
            return Err("IPv6 configuration authoring action exceeds its bound.".into());
        }
        let raw=input.strip_prefix(PREFIX).ok_or("Use the native IPv6 terminal, editor and observation controls; internal records are not learner commands.")?;
        let action: Action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid bounded IPv6 configuration action: {e}"))?;
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
