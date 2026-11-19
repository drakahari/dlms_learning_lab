//! Source/progress binding for bounded service-exposure lessons.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    service_exposure_workspace::{Action, Definition, Scope, Session},
};
use serde::Deserialize;
pub const PREFIX: &str = "service-exposure-workspace ";
pub const HELP: &str = "Offline supplied-server TCP-connect subset. Server terminal supports the documented ps/pgrep/kill/renice/ss and numeric iptables subset. Observer terminal: nmap -n -Pn -sT -p PORT[,PORT...] SUPPLIED_IP or nc -n -v -z SUPPLIED_IP PORT, from a selected supplied viewpoint. Telnet transport: telnet -c SUPPLIED_IP NONDEFAULT_PORT. A successful transport leaves its client running with no exit status yet; choose Escape to local prompt, then type quit and Run command to exit. Ports 1–65535 except 23 only: TELNET negotiation, login, data, trace and startup files are not modeled. Do not reuse nc flags; telnet -n is a trace option and is unsupported. nc requires numeric (-n), visible (-v), zero-I/O (-z) TCP probing; grouped/reordered flags are supported. Connection success has exit status 0; refusal or a modeled drop has status 1. A rejected command has no connection observation. These describe reachability, not safety or application health. Only the supplied external path and one host's INPUT/OUTPUT gates are modeled. No discovery, DNS, raw/UDP/ACK scans, payload sessions, listen/execute modes, service/version detection, NSE scripts, actual network, sockets, host processes or files. Limits, versioned history and primary references: SERVICE_EXPOSURE_SUBSET.md, TCP_PROBE_SUBSET.md and TELNET_TRANSPORT_SUBSET.md.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Routed {
    practice: bool,
    action: Action,
}
pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    let Some(raw) = input.strip_prefix(PREFIX) else {
        return Err("Use the native source-bound exposure workspace. Learner terminal commands run only inside its virtual contexts.".into());
    };
    if raw.len() > 8192 {
        return Err("Keep native exposure actions within8KiB.".into());
    }
    let request: Routed =
        serde_json::from_str(raw).map_err(|e| format!("Malformed native exposure action: {e}"))?;
    s.action(d, &request.action, request.practice)?;
    Ok("Simulated exposure workspace updated.".into())
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::ServiceExposure { simulation }, Goal::ServiceExposure { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Exposure state and goal must use the same source-bound activity.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match exposure identity/version and separate pristine example/assignment workspaces."
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
        return Err("Teach bounded real command names in their named server or observer context; native actions are not Linux commands.".into());
    }
    let mut scope = Scope::new(&definition.example)?;
    let mut reachable = vec![scope.world.clone()];
    for action in &definition.worked {
        scope.apply(&definition.example, action)?;
        reachable.push(scope.world.clone());
    }
    for c in s.guide.iter().flat_map(|g| &g.commands) {
        let supported = reachable.iter().any(|world| {
            if c.command.split_whitespace().next() == Some("nmap") {
                (0..world.viewpoints.len()).any(|i| world.scan_command(i, &c.command).is_ok())
            } else {
                let mut w = world.clone();
                w.admin_command(&c.command).is_ok_and(|o| o.status == 0)
            }
        });
        if !supported {
            return Err(format!(
                "Unsupported or unreachable example command: {}",
                c.command
            ));
        }
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::ServiceExposure { simulation } = &a.state else {
        return Err(
            "Saved exposure work belongs to another activity; previous work retained.".into(),
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
            "Saved exposure context differs from its authored source; previous work retained."
                .into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = d.bound(s) {
        return vec![(false, e)];
    }
    let mut checks = d
        .assignment
        .checks(&s.assignment.world)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect::<Vec<_>>();
    if !s.assignment.checked {
        checks.push((
            false,
            "Check current work: choose Check configuration after your changes.".into(),
        ));
    }
    checks
}
