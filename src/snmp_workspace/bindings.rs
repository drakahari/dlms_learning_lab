//! Authored-context binding; no learner-host execution or network operations.
use super::*;
pub const HELP: &str = "Use the native MIB browser for simulated GET/GETNEXT reads, then edit the monitor configuration and choose Check monitor. Only the supplied virtual snapshots are read. No SNMP packets, live agents, host commands, services or credentials are accessed; writable objects, GETBULK and SNMP security are outside this model.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::SnmpMonitor { simulation },
        crate::engine::Goal::SnmpMonitor { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Monitoring state and objectives must use the same declared activity.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || **simulation != Session::new(definition) {
        return Err("Monitoring scenarios need matching identities and separate fresh, unchecked practice and assignment drafts.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::SnmpMonitor { .. })
        || matches!(s.initial, crate::engine::State::SnmpMonitor { .. })
        || matches!(s.goal, crate::engine::Goal::SnmpMonitor { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let def = source(s)?;
    let crate::engine::State::SnmpMonitor { simulation } = &a.state else {
        return Err("Saved monitoring work belongs to another activity; original retained.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible monitoring progress metadata; original retained.".into());
    }
    simulation.validate(def)
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let (d, t) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !d.checked() {
            return vec![(false,"Not checked since editing. Inspect the supplied agent and requirements, then choose Check monitor.".into())];
        }
        match t.report(&d.fields) {
            Err(e) => vec![(false, e)],
            Ok(r) => {
                if let Some(error) = r.configuration_error {
                    return vec![(false, error)];
                }
                r.cases
                    .into_iter()
                    .map(|c| (c.passed, format!("{}: {}", c.label, c.actual.explanation)))
                    .collect()
            }
        }
    }
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        fn query(q: &Query) -> bool {
            identifier(&q.case, 64) && q.oid.len() <= 384 && !q.oid.chars().any(char::is_control)
        }
        if self.version != 1 {
            return Err("Unsupported monitoring progress version; original retained.".into());
        }
        for d in [&self.practice, &self.assignment] {
            if !identifier(&d.task_id, 80)
                || !query(&d.query)
                || d.queried.as_ref().is_some_and(|q| !query(q))
            {
                return Err(
                    "Malformed monitoring identity or browser input; original retained.".into(),
                );
            }
            d.fields.shape()?;
            if let Some(f) = &d.checked_fields {
                f.shape()?
            }
        }
        Ok(())
    }
    /// Pack reference reachability only. These are not Linux or SNMP command strings.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        self.validate(d)?;
        if raw.len() > 1024 || raw.chars().any(char::is_control) {
            return Err("Use one bounded monitoring source action.".into());
        }
        if raw == "help" {
            return Ok(HELP.into());
        }
        let mut next = self.clone();
        let message = if raw == "@monitor-check" {
            next.assignment.check(&d.initial)?;
            "Checked the current monitor against every supplied measurement."
        } else if let Some(json) = raw.strip_prefix("@monitor-set ") {
            let fields:Fields=serde_json::from_str(json).map_err(|_|"A monitoring source edit needs only the declared bounded configuration fields.")?;
            d.initial.authorized(&fields)?;
            next.assignment.fields = fields;
            next.assignment.changed();
            "Updated simulated monitor configuration; check it before claiming completion."
        } else {
            return Err("Unsupported monitoring source action. Use the native workspace; no host command or network request runs.".into());
        };
        next.validate(d)?;
        *self = next;
        Ok(message.into())
    }
}
