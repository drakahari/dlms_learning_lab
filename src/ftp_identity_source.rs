//! Authored source/progress binding for the virtual FTP identity workspace.
use crate::{
    engine::{Goal, State},
    ftp_identity_workspace::{Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub const PREFIX: &str = "ftp-identity-workspace ";
pub const HELP:&str="Offline FTP identity and regular-file subset. The native configuration editor saves /etc/vsftpd.conf or a supplied account list; restart the simulated service to apply configuration. Local/anonymous login, write/download/list gates, user allow/deny lists, chroot-list inversion and ordinary Unix rights are modeled. A writable jail root prevents login; provide a writable child. Supplied credential proofs are fixtures, not passwords or PAM. Administrative terminal: ls, id, pwd, cat, owner-only chmod/chgrp and systemctl status/start/restart/stop vsftpd. FTP client: pwd, cd PATH, ls [PATH], get REMOTE [LOCAL], put LOCAL [REMOTE], quit. Client filenames name only the displayed simulated client buffers. Paths, traversal, transfers, files, histories and output are bounded. No real sockets, daemon, host files or commands, general shell, TLS, symlinks, virtual users, per-user/local_root overrides, anonymous writes or full FTP protocol. Each behavior check starts with its own sample transfer files; current configuration, account lists and permission changes remain. Examples are separate and ungraded. Reference details and exclusions are in FTP_IDENTITY_SUBSET.md.";
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::FtpIdentity { simulation }, Goal::FtpIdentity { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("FTP state and task must use the same source-bound workspace.".into());
    };
    definition.validate()?;
    if s.track != "LPIC-2"
        || s.version != 1
        || s.id != definition.source_id
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Match the FTP identifier, version and separate initial example/assignment workspaces."
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
        return Err("Teach bounded supported commands in their named administrative or FTP client context; native reference actions are not Linux commands.".into());
    }
    let mut reachable = vec![definition.example.initial.clone()];
    let mut scope =
        crate::ftp_identity_workspace::Scope::new(&definition.example, &definition.example_inputs);
    for action in &definition.worked {
        scope.apply(&definition.example, &definition.example_inputs, action)?;
        reachable.push(scope.world.clone());
    }
    for command in s.guide.iter().flat_map(|g| &g.commands) {
        if !reachable
            .iter()
            .any(|w| crate::ftp_identity_workspace::example_admin_supported(w, &command.command))
        {
            return Err(format!(
                "Unsupported or unreachable administrative example command: {}",
                command.command
            ));
        }
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::FtpIdentity { simulation } = &a.state else {
        return Err("Saved FTP work belongs to another activity; previous work retained.".into());
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
            "Saved FTP context differs from its authored source; previous work retained.".into(),
        );
    }
    d.bound(simulation)
}
pub fn checks(s: &Session, d: &Definition) -> Vec<(bool, String)> {
    if let Err(e) = d.bound(s) {
        return vec![(false, e)];
    }
    s.assignment
        .checks(&d.assignment, &d.assignment_inputs)
        .into_iter()
        .map(|c| (c.passed, format!("{}: {}", c.label, c.detail)))
        .collect()
}
