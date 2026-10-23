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
    pub process_teaching: Option<crate::process_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounts_teaching: Option<crate::accounts_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_teaching: Option<crate::discovery_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_teaching: Option<crate::storage_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acl_teaching: Option<crate::acl_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6_teaching: Option<crate::ipv6_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4_routing_teaching: Option<crate::ipv4_routing_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_teaching: Option<crate::address_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_teaching: Option<crate::service_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_teaching: Option<crate::dns_teaching::Lesson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switching_teaching: Option<crate::switching_teaching::Lesson>,
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
/// Developer pack-file validation; this entry point is never a learner command.
/// Bound ordinary-file input before UTF-8 decoding or JSON allocation.
pub fn load_file(path: &std::path::Path) -> Result<Vec<Scenario>, String> {
    let bytes = crate::local_file::read_bounded(path, MAX_PACK_BYTES)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| "Scenario packs must contain UTF-8 JSON".to_string())?;
    load(&text)
}
pub const MAX_PACK_BYTES: usize = 2_000_000;
/// Shared authoring/filter labels; difficulty is advisory, never a mastery gate.
pub const DIFFICULTIES: [&str; 3] = ["Beginner", "Intermediate", "Advanced"];
pub fn load(text: &str) -> Result<Vec<Scenario>, String> {
    if text.len() > MAX_PACK_BYTES {
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
            || !DIFFICULTIES.contains(&s.difficulty.as_str())
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
            || [&s.steps, &s.hints, &s.sources].iter().any(|v| {
                v.is_empty()
                    || v.len() > 50
                    || v.iter().any(|v| v.trim().is_empty() || v.len() > 4000)
            })
            || s.solution.is_empty()
            || s.solution.len() > 50
            || s.solution.iter().any(|v| {
                v.trim().is_empty()
                    || v.len()
                        > if matches!(s.initial, State::Ipv4Acl { .. }) {
                            crate::acl_editor::MAX_JSON + 64
                        } else {
                            4000
                        }
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
        if let Some(lesson) = &s.switching_teaching {
            if !matches!(s.initial, State::Switching { .. }) {
                return Err("Switching teaching attached to incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Switching { requirements } = &s.goal {
            let State::Switching { fabric } = &s.initial else {
                return Err("Switching goals require a switching fabric".into());
            };
            if !crate::switching::valid_requirements(requirements, fabric) {
                return Err(format!(
                    "{}: invalid switching goals; every isolation endpoint needs a positive service case",
                    s.id
                ));
            }
        }
        if let Some(lesson) = &s.service_teaching {
            if !matches!(s.initial, State::Service { .. })
                || !matches!(s.goal,Goal::Service{port}if port!=lesson.required_port)
            {
                return Err(
                    "Service teaching requires a distinct example port and service task".into(),
                );
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.dns_teaching {
            let State::AuthoritativeDns { server } = &s.initial else {
                return Err("DNS teaching needs an authoritative DNS activity".into());
            };
            if crate::dns::canonical_name(&server.configured.origin).ok()
                == crate::dns::canonical_name(&lesson.fixture.configured.origin).ok()
            {
                return Err("DNS example must use a different zone from the assignment".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let State::Ipv4Acl { network } = &s.initial {
            let Goal::Ipv4Acl { definition } = &s.goal else {
                return Err("ACL state requires a matching fixed flow definition".into());
            };
            if network != &definition.initial {
                return Err(
                    "Initial ACL state must exactly equal the definition's initial network".into(),
                );
            }
            definition.validate().map_err(|e| e.to_string())?;
            let lesson = s.acl_teaching.as_ref().ok_or(
                "ACL activities require introduction and a separate bounded practice fixture",
            )?;
            lesson.validate()?;
            if !lesson.distinct_from(network) {
                return Err(
                    "Use distinct example topology identities and address values for ACL practice"
                        .into(),
                );
            }
        } else if s.acl_teaching.is_some() || matches!(s.goal, Goal::Ipv4Acl { .. }) {
            return Err("ACL teaching or objectives attached to another activity".into());
        }
        if let State::Accounts { simulation } = &s.initial {
            let Goal::Accounts {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("Account activities require matching typed account goals".into());
            };
            let Some(lesson) = &s.accounts_teaching else {
                return Err("Account activities require validated prerequisite teaching and separate practice".into());
            };
            if initial != simulation || !crate::accounts::valid_requirements(initial, requirements)
            {
                return Err(
                    "Account goals must pin the authored initial state and valid outcomes".into(),
                );
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
            if !lesson.separate_from(simulation) {
                return Err("Use different nonroot user/group identities and a different supplied date in the ungraded account example".into());
            }
        } else if s.accounts_teaching.is_some() || matches!(s.goal, Goal::Accounts { .. }) {
            return Err("Account teaching/goals attached to another activity".into());
        }
        if let State::PersistentMounts { storage } = &s.initial {
            let Some(lesson) = &s.storage_teaching else {
                return Err("Persistent mount activities require validated introduction and separate practice".into());
            };
            let Goal::PersistentMounts {
                initial,
                requirements,
            } = &s.goal
            else {
                return Err("Storage objectives require the matching typed state".into());
            };
            if initial != storage.inventory()
                || !crate::storage_workspace::valid_requirements(requirements, initial)
            {
                return Err("Invalid or unpinned persistent mount objectives".into());
            }
            if storage.inventory().volumes().values().any(|v| {
                lesson
                    .fixture
                    .inventory()
                    .volumes()
                    .values()
                    .any(|other| other.uuid == v.uuid)
            }) {
                return Err("Use different volume UUIDs for the ungraded storage example".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        } else if s.storage_teaching.is_some() || matches!(s.goal, Goal::PersistentMounts { .. }) {
            return Err("Storage teaching/objectives attached to another activity".into());
        }
        if matches!(s.initial, State::Discovery { .. }) && s.discovery_teaching.is_none() {
            return Err(
                "Discovery activities require validated introduction and practice content".into(),
            );
        }
        if let Goal::Discovery {
            initial,
            requirements,
        } = &s.goal
        {
            let State::Discovery { simulation } = &s.initial else {
                return Err("Discovery objectives require a discovery topology".into());
            };
            if initial != simulation.topology()
                || !crate::discovery_workspace::valid_requirements(requirements, initial)
            {
                return Err("Invalid, unpinned or excessive discovery objectives".into());
            }
        }
        if let Some(lesson) = &s.discovery_teaching {
            let State::Discovery { simulation } = &s.initial else {
                return Err("Discovery teaching requires a discovery assignment".into());
            };
            if simulation
                .topology()
                .same_identity(lesson.fixture.topology())
            {
                return Err("Use a different topology for the discovery practice example".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Processes { requirements } = &s.goal {
            let State::Processes { table } = &s.initial else {
                return Err("Process objectives require a process table".into());
            };
            if requirements.is_empty()
                || requirements.len() > 64
                || requirements.iter().any(|r| !r.valid(table))
            {
                return Err("Invalid or excessive process objectives".into());
            }
        }
        if let Goal::AuthoritativeDns { requirements } = &s.goal {
            let State::AuthoritativeDns { server } = &s.initial else {
                return Err("DNS objectives need an authoritative DNS state".into());
            };
            if !crate::dns_workspace::valid_requirements(requirements, server) {
                return Err(
                    "DNS objectives need bounded outcomes, publication and a served lookup".into(),
                );
            }
        }
        if let Some(lesson) = &s.address_teaching {
            if crate::address_teaching::goal(&s.initial).is_none()
                || std::mem::discriminant(&s.initial) != std::mem::discriminant(&lesson.fixture)
                || !crate::address_teaching::distinct_example(&s.initial, &lesson.fixture)
            {
                return Err(format!(
                    "{}: addressing teaching needs a separate compatible example problem",
                    s.id
                ));
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.ipv4_routing_teaching {
            if !matches!(s.initial, State::Ipv4Routing { .. }) {
                return Err("IPv4 routing teaching attached to an incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Goal::Ipv4Routing { requirements } = &s.goal {
            let State::Ipv4Routing { topology } = &s.initial else {
                return Err("IPv4 routing objectives require an IPv4 routing topology".into());
            };
            if !crate::ipv4_routing::valid_requirements(requirements, topology) {
                return Err(format!(
                    "{}: invalid IPv4 routing objectives or missing normal-state positive connectivity",
                    s.id
                ));
            }
        }
        if let Some(lesson) = &s.ipv6_teaching {
            if !matches!(s.initial, State::Ipv6 { .. }) {
                return Err("IPv6 teaching attached to an incompatible state".into());
            }
            lesson.validate().map_err(|e| format!("{}: {e}", s.id))?;
        }
        if let Some(lesson) = &s.process_teaching {
            let State::Processes { table } = &s.initial else {
                return Err("Process teaching requires a process assignment".into());
            };
            if table.same_problem(&lesson.fixture) {
                return Err("Use different process identities for the ungraded example".into());
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
        if let Goal::Filesystem { requirements } = &s.goal {
            let State::Filesystem { tree } = &s.initial else {
                return Err("Filesystem objectives require a filesystem state".into());
            };
            if requirements.is_empty()
                || requirements.len() > 20
                || requirements.iter().any(|r| !r.valid_for(tree))
            {
                return Err(format!("{}: invalid filesystem objectives", s.id));
            }
        }
        validate_state(&s.initial).map_err(|e| format!("{}: {e}", s.id))?;
        if engine::solved(&s.initial, &s.goal) {
            return Err(format!("{}: starts solved", s.id));
        }
        let mut state = s.initial.clone();
        for cmd in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, cmd)
                .map_err(|e| format!("{}: unreachable solution: {e}", s.id))?;
        }
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut state, &s.goal) {
            if cases.len() < 3
                || cases.len() > crate::shell::MAX_CASES
                || cases.iter().any(|c| !c.valid())
            {
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
        State::Ipv4Acl { network } => crate::acl_workspace::valid_network(network).is_ok(),
        State::Accounts { simulation } => simulation.valid(),
        State::PersistentMounts { storage } => storage.validate().is_ok(),
        State::Discovery { simulation } => simulation.validate().is_ok(),
        State::Processes { table } => table.valid(),
        State::AuthoritativeDns { server } => server.validate().is_ok(),
        State::Switching { fabric } => fabric.validate().is_ok(),
        State::Ipv6 { topology } => topology.valid(),
        State::Ipv4Routing { topology } => topology.valid(),
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
                            && a.name.len() <= 80
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
            port,
            running,
            loaded_port,
            failed,
            journal,
            ..
        } => {
            port.len() <= 256
                && (!*running || (loaded_port.is_some_and(|p| p > 0) && !*failed))
                && journal.len() <= 60
                && journal
                    .iter()
                    .all(|e| !matches!(e,engine::ServiceEvent::Rejected{port}if port.len()>256))
        }

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
        if text.len() > MAX_PACK_BYTES {
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
        include_str!("../scenarios/shell-counts-v1.json"),
        include_str!("../scenarios/linux-access-v1.json"),
        include_str!("../scenarios/vlan-data-v1.json"),
        include_str!("../scenarios/shell-file-batches-v1.json"),
        include_str!("../scenarios/ipv4-static-v1.json"),
        include_str!("../scenarios/dns-forward-v1.json"),
        include_str!("../scenarios/shell-arrays-v1.json"),
        include_str!("../scenarios/linux-processes-v1.json"),
        include_str!("../scenarios/discovery-v1.json"),
        include_str!("../scenarios/storage-v1.json"),
        include_str!("../scenarios/shell-capture-v1.json"),
        include_str!("../scenarios/acl-policy-v1.json"),
        include_str!("../scenarios/accounts-lifecycle-v1.json"),
    ])
}
