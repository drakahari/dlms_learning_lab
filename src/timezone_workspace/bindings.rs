//! Bind only the declared source; no cached world/output can grant credit.
use super::*;
pub const HELP: &str = "Supported simulated forms: timedatectl list-timezones; timedatectl show -p Timezone [--value]; timedatectl set-timezone ZONE; readlink /etc/localtime; printenv TZ; export TZ=value; unset TZ; date [-u] [--date=@SECONDS] [+FORMAT]. Supplied zones and2026 instants only. Date fields: %Y %m %d %e %H %M %S %F %T %R %z %:z %Z %s %a %b %% %n %t. C-locale output. UTC clock never changes; this activity does not run host commands, services, scripts or file operations. Full status, clock setting, arbitrary TZ/paths, scripts, expansions, pipes, redirection and unlisted options are outside the supported subset.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::Timezone { simulation },
        crate::engine::Goal::Timezone { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Time-zone state and objective must use the same supplied activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id {
        return Err("The independent time-zone task ID must match its scenario.".into());
    }
    if **simulation != Session::new(definition) {
        return Err(
            "Time-zone work must start with separate declared example and assignment drafts."
                .into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::Timezone { .. })
        || matches!(s.initial, crate::engine::State::Timezone { .. })
        || matches!(s.goal, crate::engine::Goal::Timezone { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::Timezone { simulation } = &a.state else {
        return Err("Saved time-zone work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible time-zone metadata; original retained.".into());
    }
    simulation.validate(d)
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(error) = self.validate().and_then(|()| s.validate(self)) {
            return vec![(false, error)];
        }
        let (draft, task) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !draft.attempted {
            return vec![(false,"Not checked. Investigate the supplied timestamps, repair the policy, then choose Check work.".into())];
        }
        match draft.checks(task) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.correct, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(error) => vec![(false, error)],
        }
    }
}
impl Session {
    /// Source reachability route only. Native terminal input never accepts the
    /// check helper; its Check work button calls the typed check operation.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        d.validate()?;
        self.validate(d)?;
        if raw.len() > 1024 {
            return Err("Keep one time-zone command within1,024bytes.".into());
        }
        let mut next = self.clone();
        let message = if raw == "@timezone-check" {
            next.assignment.check(&d.initial)?;
            "Checked actual current, separate-login and supplied handover outcomes.".into()
        } else {
            next.assignment.input = raw.into();
            let out = next
                .assignment
                .run(&d.initial)?
                .map_err(|e| e.to_string())?;
            format!("Exit status {}\n{}{}", out.status, out.stdout, out.stderr)
        };
        next.validate(d)?;
        *self = next;
        Ok(message)
    }
}
