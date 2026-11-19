//! Catalog-held authority for private-index work. No load/reset action executes a recipe.
use crate::{
    engine::{Goal, State},
    file_index_workspace as work,
    pack::Scenario,
    progress::Attempt,
    workspace::WorkspaceData,
};

pub const PREFIX: &str = "file-index ";
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    SaveConfiguration { text: String },
    Command { input: String },
    Check,
}
fn size<T: serde::Serialize>(value: &T, cap: usize) -> bool {
    struct Count {
        n: usize,
        cap: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Private-index context limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { n: 0, cap }, value).is_ok()
}
pub fn relevant(a: &Attempt, s: &Scenario) -> bool {
    matches!(a.state, State::FileIndex { .. })
        || matches!(s.initial, State::FileIndex { .. })
        || matches!(s.goal, Goal::FileIndex { .. })
}
pub fn source(s: &Scenario) -> Result<&work::Definition, String> {
    let (State::FileIndex { simulation }, Goal::FileIndex { definition }) = (&s.initial, &s.goal)
    else {
        return Err("Private-index work requires its supplied assignment".into());
    };
    if !size(s, 2 * 1024 * 1024) || definition.id != s.id || definition.version != s.version {
        return Err("Private-index source identity or size differs".into());
    }
    definition.validate().map_err(|e| e.to_string())?;
    for (scope, task) in [
        ("assignment", &definition.assignment),
        ("example", &definition.example),
    ] {
        if task.initial.context.source_id != format!("{}-{scope}", s.id) {
            return Err("Private-index source role differs from its catalog lesson".into());
        }
    }
    if **simulation != work::Session::new(definition).map_err(|e| e.to_string())? {
        return Err(
            "Private-index initial state must be the untouched task and separate practice".into(),
        );
    }
    // A new optional teaching family cannot silently attach to this dedicated workspace.
    let encoded = serde_json::to_value(s).map_err(|e| e.to_string())?;
    if encoded.as_object().is_none_or(|o| {
        o.iter()
            .any(|(k, v)| (k == "teaching" || k.ends_with("_teaching")) && !v.is_null())
    }) {
        return Err("Foreign teaching metadata on private-index source".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::FileIndex { simulation } = &a.state else {
        return Err(
            "Saved private-index work has a different activity kind; original retained".into(),
        );
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript.iter().any(|t| t.len() > 8192)
        || !size(a, 2 * 1024 * 1024)
    {
        return Err("Private-index attempt metadata or size differs; original retained".into());
    }
    if let Some(w) = &a.workspace
        && serde_json::to_value(w).map_err(|e| e.to_string())?
            != serde_json::to_value(WorkspaceData::new()).map_err(|e| e.to_string())?
    {
        return Err("Foreign workspace metadata on private-index work; original retained".into());
    }
    simulation.validate_for(d).map_err(|e| e.to_string())
}
pub fn checks(s: &work::Session, d: &work::Definition) -> Vec<(bool, String)> {
    match s.checks(d, work::Scope::Assignment) {
        Ok(r) => r
            .checks
            .into_iter()
            .map(|c| (c.passed, c.message))
            .collect(),
        Err(e) => vec![(
            false,
            format!("Private-index work needs its original supplied task: {e}"),
        )],
    }
}
/// Pack reachability adapter only. The GUI calls the same typed Save/Run methods.
pub fn execute(s: &mut work::Session, d: &work::Definition, input: &str) -> Result<String, String> {
    s.validate_for(d).map_err(|e| e.to_string())?;
    if input.len() > 8192 {
        return Err("Private-index action exceeds its input limit".into());
    }
    let action = if let Some(raw) = input.strip_prefix(PREFIX) {
        serde_json::from_str(raw)
            .map_err(|e| format!("Invalid native private-index action: {e}"))?
    } else {
        Action::Command {
            input: input.into(),
        }
    };
    let mut next = s.clone();
    let output = match action {
        Action::SaveConfiguration { text } => {
            next.set_config_draft(d, work::Scope::Assignment, text)
                .map_err(|e| e.to_string())?;
            next.save_config(d, work::Scope::Assignment)
                .map_err(|e| e.to_string())?;
            "Configuration saved; the private index has not been rebuilt.".into()
        }
        Action::Command { input } => {
            next.set_command(d, work::Scope::Assignment, input)
                .map_err(|e| e.to_string())?;
            let row = next
                .run(d, work::Scope::Assignment)
                .map_err(|e| e.to_string())?;
            row.stdout
        }
        Action::Check => {
            let report = next
                .check(d, work::Scope::Assignment)
                .map_err(|e| e.to_string())?;
            format!(
                "{} of {} requirements satisfied",
                report.checks.iter().filter(|c| c.passed).count(),
                report.checks.len()
            )
        }
    };
    next.validate_for(d).map_err(|e| e.to_string())?;
    *s = next;
    Ok(output)
}

pub fn shape(s: &work::Session) -> bool {
    s.validate_shape().is_ok()
}
