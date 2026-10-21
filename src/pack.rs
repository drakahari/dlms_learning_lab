use crate::engine::{self, Goal, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub version: u32,
    pub track: String,
    pub topic: String,
    pub title: String,
    pub summary: String,
    pub objective: String,
    pub difficulty: String,
    pub minutes: u16,
    pub steps: Vec<String>,
    pub hints: Vec<String>,
    pub sources: Vec<String>,
    pub simplifications: String,
    pub initial: State,
    pub goal: Goal,
    pub solution: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub schema_version: u32,
    pub scenarios: Vec<Scenario>,
}
pub fn load(text: &str) -> Result<Vec<Scenario>, String> {
    if text.len() > 2_000_000 {
        return Err("Pack exceeds 2 MB".into());
    }
    let p: Pack = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if p.schema_version != 1 || p.scenarios.is_empty() || p.scenarios.len() > 1000 {
        return Err("Unsupported schema or catalog size".into());
    }
    let mut ids = BTreeSet::new();
    for s in &p.scenarios {
        if s.id.is_empty()
            || s.id.len() > 80
            || !s
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !ids.insert(&s.id)
        {
            return Err("Invalid or duplicate scenario identifier".into());
        }
        if s.version == 0
            || !["CCNA", "LPIC-1", "LPIC-2"].contains(&s.track.as_str())
            || !["Beginner", "Intermediate"].contains(&s.difficulty.as_str())
            || s.minutes == 0
            || s.minutes > 180
        {
            return Err(format!("{}: invalid metadata", s.id));
        }
        if [
            &s.title,
            &s.summary,
            &s.objective,
            &s.topic,
            &s.simplifications,
        ]
        .iter()
        .any(|v| v.trim().is_empty() || v.len() > 4000)
            || [&s.steps, &s.hints, &s.sources, &s.solution]
                .iter()
                .any(|v| {
                    v.is_empty()
                        || v.len() > 50
                        || v.iter().any(|v| v.trim().is_empty() || v.len() > 4000)
                })
            || s.sources.iter().any(|v| !v.starts_with("https://"))
        {
            return Err(format!("{}: missing or oversized authoring fields", s.id));
        }
        validate_state(&s.initial).map_err(|e| format!("{}: {e}", s.id))?;
        if engine::solved(&s.initial, &s.goal) {
            return Err(format!("{}: starts solved", s.id));
        }
        let mut state = s.initial.clone();
        for cmd in &s.solution {
            engine::execute(&mut state, cmd)
                .map_err(|e| format!("{}: unreachable solution: {e}", s.id))?;
        }
        if !engine::solved(&state, &s.goal) {
            return Err(format!("{}: solution does not reach objective", s.id));
        }
    }
    Ok(p.scenarios)
}
pub fn validate_state(s: &State) -> Result<(), String> {
    let valid = match s {
        State::Subnet { prefix, .. } => *prefix <= 30,
        State::Vlsm {
            base,
            prefix,
            allocations,
        } => {
            *prefix <= 30
                && engine::bounds(*base, *prefix).is_some_and(|(n, _)| n == u32::from(*base))
                && !allocations.is_empty()
                && allocations.len() <= 20
                && {
                    let mut names = BTreeSet::new();
                    allocations.iter().all(|a| {
                        a.hosts > 0
                            && a.hosts <= 1_000_000
                            && !a.name.is_empty()
                            && a.name
                                .bytes()
                                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                            && names.insert(&a.name)
                            && (a.cidr.is_empty() || engine::cidr(&a.cidr).is_some())
                    })
                }
        }
        State::Network {
            prefix, lan_prefix, ..
        } => *prefix <= 30 && *lan_prefix <= 30,
        State::Unix { nodes, users } => {
            !users.is_empty()
                && nodes.get("/").is_some_and(|n| n.directory)
                && nodes.iter().all(|(p, n)| {
                    p.starts_with('/')
                        && !p.contains("..")
                        && n.mode <= 0o777
                        && users.contains_key(&n.owner)
                        && users.values().any(|g| g.contains(&n.group))
                        && (p == "/" || {
                            let parent = p
                                .rsplit_once('/')
                                .map(|(a, _)| if a.is_empty() { "/" } else { a })
                                .unwrap_or("/");
                            nodes.get(parent).is_some_and(|n| n.directory)
                        })
                })
        }
        State::Logs { files, result } => {
            !files.is_empty()
                && result.len() < 100_000
                && files.iter().all(|(p, v)| {
                    p.starts_with("/var/log/") && !p.contains("..") && v.len() < 100_000
                })
        }
        State::Service {
            running,
            loaded_port,
            ..
        } => !*running || loaded_port.is_some_and(|p| p > 0),
        State::Dns {
            records, loaded, ..
        } => {
            !records.is_empty()
                && records.len() < 100
                && records.keys().all(|k| {
                    k.ends_with(".lab.test")
                        && k.bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
                })
                && loaded.keys().all(|k| records.contains_key(k))
        }
        State::Mount {
            capacity,
            used,
            cache,
            device,
            configured_device,
            ..
        } => {
            *capacity > 0
                && used <= capacity
                && cache <= used
                && device.starts_with("/dev/")
                && !configured_device.is_empty()
                && configured_device.len() <= 256
        }
    };
    if valid {
        Ok(())
    } else {
        Err("Invalid modeled initial state".into())
    }
}
pub fn bundled() -> Result<Vec<Scenario>, String> {
    load(include_str!("../scenarios/starter-v1.json"))
}
