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
    #[serde(default)]
    pub unit: String,
    pub title: String,
    pub summary: String,
    pub objective: String,
    #[serde(default)]
    pub retry_guidance: String,
    pub difficulty: String,
    pub minutes: u16,
    pub steps: Vec<String>,
    #[serde(default)]
    pub guide: Vec<LessonStep>,
    #[serde(default)]
    pub explanation: String,
    pub hints: Vec<String>,
    pub sources: Vec<String>,
    pub simplifications: String,
    pub initial: State,
    pub goal: Goal,
    pub solution: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teaching: Option<crate::script_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_teaching: Option<crate::terminal_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6_teaching: Option<crate::ipv6_teaching::Lesson>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonStep {
    pub title: String,
    pub why: String,
    pub action: String,
    pub commands: Vec<CommandExample>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandExample {
    pub command: String,
    pub explanation: String,
    pub simulator_only: bool,
}
#[derive(Serialize, Deserialize)]
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
    validate(p)
}
fn validate(p: Pack) -> Result<Vec<Scenario>, String> {
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
            || !["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].contains(&s.track.as_str())
            || !["Beginner", "Intermediate"].contains(&s.difficulty.as_str())
            || s.minutes == 0
            || s.retry_guidance.len() > 1000
            || s.minutes > 180
            || s.unit.len() > 80
            || (!s.unit.is_empty()
                && !s
                    .unit
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-'))
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
        let invalid_guide = s.guide.iter().any(|g| {
            [&g.title, &g.why, &g.action]
                .iter()
                .any(|t| t.trim().is_empty() || t.len() > 4000)
                || g.commands.len() > 10
                || g.commands.iter().any(|c| {
                    c.command.trim().is_empty()
                        || c.command.len() > 4096
                        || c.explanation.trim().is_empty()
                        || c.explanation.len() > 4000
                })
        });
        if (!s.guide.is_empty() && (s.guide.len() != s.steps.len() || invalid_guide))
            || s.explanation.len() > 8000
        {
            return Err(format!("{}: invalid learning guidance", s.id));
        }
        if let State::Vlsm { allocations, .. } = &s.initial
            && allocations
                .iter()
                .any(|a| !a.cidr.is_empty() && engine::cidr(&a.cidr).is_none())
        {
            return Err("Invalid initial subnet allocation".into());
        }
        if let Some(lesson) = &s.ipv6_teaching {
            if !matches!(s.initial, State::Ipv6 { .. }) {
                return Err("IPv6 teaching attached to an incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.terminal_teaching {
            if !matches!(s.initial, State::Filesystem { .. }) {
                return Err("Terminal teaching attached to an incompatible lab type".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.teaching {
            if !matches!(s.initial, State::Script { .. }) {
                return Err("Script teaching attached to a different lab type".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Ipv6 { requirements } = &s.goal {
            let State::Ipv6 { topology } = &s.initial else {
                return Err("IPv6 objective requires an IPv6 topology".into());
            };
            if requirements.is_empty()
                || requirements.len() > 20
                || requirements.iter().any(|r| !r.valid_for(topology))
                || !requirements.iter().any(
                    |r| matches!(r,crate::ipv6::Requirement::Reachable{when,..} if when.is_empty()),
                )
            {
                return Err(format!(
                    "{}: invalid IPv6 objectives or missing unconditional positive connectivity case",
                    s.id
                ));
            }
        }
        if let Goal::Filesystem { requirements } = &s.goal
            && (requirements.is_empty()
                || requirements.len() > 20
                || requirements.iter().any(|r| !r.valid()))
        {
            return Err(format!("{}: invalid filesystem objectives", s.id));
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
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut state, &s.goal) {
            if cases.len() < 3 || cases.len() > 8 || cases.iter().any(|c| !c.valid()) {
                return Err(format!("{}: invalid script cases", s.id));
            }
            *run = Some(crate::shell::Record {
                engine_revision: crate::shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases.iter().map(|c| crate::shell::run(source, c)).collect(),
            });
        }
        if !engine::solved(&state, &s.goal) {
            return Err(format!("{}: solution does not reach objective", s.id));
        }
    }
    for s in &p.scenarios {
        if s.prerequisites.len() > 8
            || s.prerequisites
                .iter()
                .any(|id| id == &s.id || !ids.contains(id))
        {
            return Err(format!("{}: invalid prerequisites", s.id));
        }
    }
    // Any cycle would make a prerequisite learning path impossible.
    for start in &p.scenarios {
        let mut pending = start.prerequisites.clone();
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if id == start.id {
                return Err(format!("{}: cyclic prerequisites", start.id));
            }
            if seen.insert(id.clone()) {
                pending.extend(
                    p.scenarios
                        .iter()
                        .find(|s| s.id == id)
                        .unwrap()
                        .prerequisites
                        .clone(),
                );
            }
        }
    }
    Ok(p.scenarios)
}
pub fn validate_state(s: &State) -> Result<(), String> {
    let valid = match s {
        State::Ipv6 { topology } => topology.valid(),
        State::Filesystem { tree } => tree.valid(),
        State::Script { source, run } => {
            source.len() <= crate::shell::MAX_SOURCE && run.as_ref().is_none_or(|r| r.valid())
        }
        State::Subnet {
            prefix,
            answers,
            second,
            ..
        } => {
            *prefix <= 30
                && answers.iter().all(|a| a.len() <= 64)
                && second.as_ref().is_none_or(|a| a.len() <= 64)
        }
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
                            && a.cidr.len() <= 64
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
            failed,
            journal,
            ..
        } => (!*running || (loaded_port.is_some_and(|p| p > 0) && !*failed)) && journal.len() <= 60,

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
/// Bound each declarative pack independently, then validate the combined prerequisite graph.
pub fn load_catalog(packs: &[&str]) -> Result<Vec<Scenario>, String> {
    if packs.is_empty()
        || packs.len() > 100
        || packs.iter().map(|p| p.len()).sum::<usize>() > 32_000_000
    {
        return Err("Invalid pack collection size".into());
    }
    let mut all = Pack {
        schema_version: 1,
        scenarios: Vec::new(),
    };
    for text in packs {
        if text.len() > 2_000_000 {
            return Err("Pack exceeds 2 MB".into());
        }
        let p: Pack = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if p.schema_version != 1 || p.scenarios.is_empty() {
            return Err("Unsupported or empty pack".into());
        }
        if all.scenarios.len() + p.scenarios.len() > 1000 {
            return Err("Catalog exceeds 1,000 authored labs".into());
        }
        all.scenarios.extend(p.scenarios);
    }
    validate(all)
}
pub fn bundled() -> Result<Vec<Scenario>, String> {
    load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/shell-unit-v1.json"),
        include_str!("../scenarios/shell-composition-v1.json"),
        include_str!("../scenarios/shell-reusable-v1.json"),
        include_str!("../scenarios/linux-files-v1.json"),
        include_str!("../scenarios/ipv6-static-v1.json"),
        include_str!("../scenarios/shell-scope-v1.json"),
        include_str!("../scenarios/shell-text-v1.json"),
    ])
}
