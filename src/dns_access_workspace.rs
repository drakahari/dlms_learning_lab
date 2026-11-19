//! Source-bound policy tasks; separate learner drafts are implemented in session.
//! No host configuration, command execution or network access.
use crate::{
    dns::{self, Data, QueryType},
    dns_access::{
        acl::Acl,
        world::{Facts, Key, World},
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub mod session;
pub use session::{Definition, Session};
pub const MAX_HISTORY: usize = 64;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Config(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,254>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Save {
        config: Config,
    },
    Reload {},
    Query {
        client: Ipv4Addr,
        name: Name,
        record_type: QueryType,
        recursive: bool,
    },
    Transfer {
        client: Ipv4Addr,
        zone: Name,
    },
    ClearCache {},
}
pub fn apply(world: &mut World, action: &Action) -> Result<String, String> {
    let mut next = world.clone();
    let output=match action{
        Action::Save{config}=>{next.save(&config.0)?;"Configuration saved. Reload it to apply these access rules.".into()},
        Action::Reload{}=>{next.reload();"Saved configuration reloaded. Existing simulated cache entries remain.".into()},
        Action::Query{client,name,record_type,recursive}=>next.query(*client,&name.0,*record_type,*recursive)?.text()?,
        Action::Transfer{client,zone}=>match next.transfer(&zone.0,*client)? {
            crate::dns_access::world::TransferReply::Refused => "Transfer refused: this simulated source address is outside the zone's effective allow-transfer list. No secondary data changed.".into(),
            crate::dns_access::world::TransferReply::Secondary{decision,primary_serial,secondary_serial,records} => {
                let message = match decision {
                    dns::Transfer::Updated => "The supplied secondary now has the primary's current serial and records.",
                    dns::Transfer::UnchangedEqual => "The serials are equal, so the secondary retained its existing copy.",
                    dns::Transfer::UnchangedOlder => "The primary serial is older, so the secondary retained its existing copy.",
                };
                format!("Transfer allowed for supplied secondary {client}. {message}\nPrimary serial: {primary_serial}\nSecondary serial: {secondary_serial}\nSecondary records: {records}")
            },
            crate::dns_access::world::TransferReply::OtherClient{zone:served} => {
                let mut text = format!("Transfer allowed to simulated client {client}; this is not the supplied secondary. Its copy remains unchanged.\nExposed zone: {} · serial {}\n",served.origin,served.serial);
                for record in &served.records {text.push_str(&crate::dns_workspace::record_text(record));text.push('\n');}
                text
            },
        },
        Action::ClearCache{}=>{next.clear_cache();"Simulated cache cleared. Published zone data and saved/live access rules are unchanged.".into()},
    };
    if output.len() > 32768 {
        return Err("Result exceeds the simulator's 32 KiB output limit.".into());
    }
    *world = next;
    Ok(output)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferRequirement {
    pub zone: Name,
    pub clients: Acl,
    pub synchronized: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Required {
    pub public_queries: Acl,
    pub cached_answers: Acl,
    pub recursive_lookups: Acl,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,TransferRequirement,4>")]
    pub transfers: Vec<TransferRequirement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cache {
    Cold,
    SuppliedWarm,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Data {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Data,64>")]
        data: Vec<Data>,
    },
    NoExternalAnswer {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    pub client: Ipv4Addr,
    pub name: Name,
    pub record_type: QueryType,
    pub recursive: bool,
    pub cache: Cache,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub initial_config: Config,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,64>")]
    pub starter: Vec<Action>,
    pub required: Required,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,16>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,64>")]
    pub reference: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
fn row(label: impl Into<String>, correct: bool, yes: &str, no: impl Into<String>) -> Check {
    Check {
        label: label.into(),
        correct,
        feedback: if correct { yes.into() } else { no.into() },
        expected: None,
        actual: None,
    }
}
fn ranges(rows: &[(u32, u32)]) -> String {
    if rows.is_empty() {
        return "no clients".into();
    }
    rows.iter()
        .map(|&(a, b)| {
            if a == b {
                Ipv4Addr::from(a).to_string()
            } else {
                format!("{}–{}", Ipv4Addr::from(a), Ipv4Addr::from(b))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
fn data(values: &[Data]) -> Result<Vec<Data>, String> {
    let mut result = values.to_vec();
    for value in &mut result {
        match value {
            Data::Cname { target } | Data::Ns { target } | Data::Ptr { target } => {
                *target = dns::canonical_name(target).map_err(|e| e.message)?
            }
            Data::Mx { exchange, .. } => {
                *exchange = dns::canonical_name(exchange).map_err(|e| e.message)?
            }
            _ => {}
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}
impl Task {
    pub fn initial(&self) -> Result<World, String> {
        self.replay(&self.starter)
    }
    pub fn replay(&self, history: &[Action]) -> Result<World, String> {
        if history.len() > MAX_HISTORY || !history.starts_with(&self.starter) {
            return Err(
                "This draft must retain the supplied starting actions and stay within 64 actions."
                    .into(),
            );
        }
        let mut world = World::new(&self.facts, &self.initial_config.0)?;
        for action in history {
            apply(&mut world, action)?;
        }
        Ok(world)
    }
    pub fn grade(&self, world: &World) -> Vec<Check> {
        if !world.source_is(&self.facts) {
            return vec![row(
                "Supplied world",
                false,
                "",
                "This world belongs to a different source exercise; restore the authored attempt.",
            )];
        }
        let mut checks = vec![];
        for (label, actual, wanted) in [
            (
                "Public query access",
                world.live().query.intervals(),
                self.required.public_queries.intervals(),
            ),
            (
                "Cached-data access",
                world.effective_cache(),
                self.required.cached_answers.intervals(),
            ),
            (
                "New recursive lookups",
                world.effective_recursion(),
                self.required.recursive_lookups.intervals(),
            ),
        ] {
            match(actual,wanted){
                (Ok(actual),Ok(wanted))=>checks.push(row(label,actual==wanted,"The current effective client range matches the task.",format!("Current effective access: {}. Required: {}. Check the saved access lists and reload the configuration; one broad or conflicting list can change the result.",ranges(&actual),ranges(&wanted)))),
                _=>checks.push(row(label,false,"","The policy cannot be evaluated in this supplied world.")),
            }
        }
        for wanted in &self.required.transfers {
            let actual = world.effective_transfer(&wanted.zone.0);
            let correct = actual.is_ok_and(|a| a.equivalent(&wanted.clients).unwrap_or(false));
            checks.push(row(format!("Transfer access · {}",wanted.zone.0),correct,"The effective zone-transfer scope matches the requested clients.","The global transfer list may be inherited unless this zone overrides it. Permit the approved peer only where required and preserve the other zone's restriction; source IP lists do not authenticate a peer."));
            if let Some((index, source)) = self
                .facts
                .zones
                .iter()
                .enumerate()
                .find(|(_, z)| z.server.primary.origin == wanted.zone.0)
            {
                let actual = world.zones().get(index);
                let correct = actual.is_some_and(|a| {
                    if wanted.synchronized {
                        a.server.secondary == a.server.primary
                    } else {
                        a.server.secondary == source.server.secondary
                    }
                });
                checks.push(row(format!("Secondary copy · {}",wanted.zone.0),correct,"The required secondary state is preserved or synchronized.",if wanted.synchronized{"The secondary still needs the primary's current records. Applying a transfer policy does not copy a zone; test the approved peer's transfer."}else{"This zone's existing secondary copy changed. Restore the supplied attempt before handing over the restricted policy."}));
            }
        }
        for case in &self.cases {
            let mut sample = world.clone();
            let prepared = match case.cache {
                Cache::Cold => {
                    sample.clear_cache();
                    Ok(())
                }
                Cache::SuppliedWarm => sample.reset_cache_to_supplied(),
            };
            let result = prepared.and_then(|()| {
                sample.query(case.client, &case.name.0, case.record_type, case.recursive)
            });
            let (correct, actual) = match result {
                Ok(reply) => {
                    let correct = match &case.expected {
                        Expected::NoExternalAnswer {} => reply.answer.is_none(),
                        Expected::Data { data: expected } => {
                            reply.answer.as_ref().is_some_and(|answer| {
                                answer.code == dns::Code::Noerror
                                    && match (
                                        data(
                                            &answer
                                                .records
                                                .iter()
                                                .filter(|r| r.data.query_type() == case.record_type)
                                                .map(|r| r.data.clone())
                                                .collect::<Vec<_>>(),
                                        ),
                                        data(expected),
                                    ) {
                                        (Ok(actual), Ok(expected)) => actual == expected,
                                        _ => false,
                                    }
                            })
                        }
                    };
                    (correct, reply.text().unwrap_or_else(|e| e))
                }
                Err(error) => (
                    false,
                    format!("Simulation limitation or invalid question: {error}"),
                ),
            };
            let expected = match &case.expected {
                Expected::NoExternalAnswer {} => format!(
                    "No external answer for this simulated client {}.",
                    if case.cache == Cache::SuppliedWarm {
                        "even though the answer is already cached"
                    } else {
                        "with an empty cache"
                    }
                ),
                Expected::Data { data } => format!(
                    "DNS result: NOERROR\n{} values: {}",
                    crate::dns_workspace::type_text(case.record_type),
                    data.iter()
                        .map(crate::dns_workspace::data_text)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
            checks.push(Check{label:format!("Query case · {}",case.label),correct,feedback:if correct{"This sample has the requested outcome.".into()}else{"This client gets the wrong result. Compare its source address, requested name and cache state with the effective access rules; saving alone does not reload them.".into()},expected:Some(expected),actual:Some(actual)});
        }
        checks
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.cases.len() < 4
            || self.cases.len() > 16
            || self.reference.is_empty()
            || self.starter.len() + self.reference.len() > MAX_HISTORY
        {
            return Err("Invalid task ID, case count or bounded action sequence.".into());
        }
        self.facts.validate()?;
        for acl in [
            &self.required.public_queries,
            &self.required.cached_answers,
            &self.required.recursive_lookups,
        ] {
            acl.validate()?;
        }
        let mut zones = BTreeSet::new();
        for wanted in &self.required.transfers {
            wanted.clients.validate()?;
            if !zones.insert(wanted.zone.0.clone())
                || !self.facts.known().contains_key(&wanted.zone.0)
            {
                return Err(
                    "Every transfer requirement must name a different supplied canonical zone."
                        .into(),
                );
            }
        }
        if zones.len() != self.facts.zones.len() {
            return Err(
                "State the transfer/preservation requirement for every supplied zone.".into(),
            );
        }
        let mut labels = BTreeSet::new();
        for case in &self.cases {
            if case.label.is_empty()
                || case.label.len() > 64
                || case.label.chars().any(char::is_control)
                || !labels.insert(&case.label)
            {
                return Err("Query cases need distinct readable labels.".into());
            }
            let key = Key::new(&case.name.0, case.record_type)?;
            if !self.facts.supports_name(&key.name) {
                return Err("Each graded question needs supplied authoritative/upstream data; unknown fixtures cannot count as denied service.".into());
            }
            if case.cache == Cache::SuppliedWarm && !self.facts.warm_cache.contains(&key) {
                return Err(
                    "A warm-cache case must name an actual supplied initial cache entry.".into(),
                );
            }

            if let Expected::Data { data: values } = &case.expected {
                if values.is_empty()
                    || values.len() > 64
                    || values.iter().any(|d| d.query_type() != case.record_type)
                {
                    return Err(
                        "State a nonempty requested-type result for each positive query case."
                            .into(),
                    );
                }
                data(values)?;
            }
        }
        let mut reached = self.initial()?;
        if self.grade(&reached).iter().all(|r| r.correct) {
            return Err("The supplied independent task is already solved.".into());
        }
        for action in &self.reference {
            apply(&mut reached, action)?;
        }
        if self.grade(&reached).iter().any(|r| !r.correct) {
            return Err("The declared supported method cannot reach every task requirement and sample case.".into());
        }
        Ok(())
    }
}

pub const HELP: &str = "Edit the simulated named.conf, save it and reload to apply its rules. Supports explicit recursion yes/no, allow-query, allow-recursion, allow-query-cache and allow-transfer lists, plus the supplied primary zone declarations. Use complete IPv4 hosts or canonical network/prefix values, any or none. Includes, implicit defaults, named/negative ACLs, views, keys and unlisted settings are unsupported. Query and transfer controls use only supplied virtual DNS data; no host file, process or network is accessed.";
impl Definition {
    pub fn checks(&self, session: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(error) = session.validate(self) {
            return vec![(false, error)];
        }
        let (draft, task) = if practice {
            (&session.practice, &self.example)
        } else {
            (&session.assignment, &self.initial)
        };
        if !draft.attempted {
            return vec![(false,"Not checked. Save and reload the intended policy, inspect its behavior, then choose Check access.".into())];
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
    pub fn shape(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unknown DNS access workspace version; original retained.".into());
        }
        for draft in [&self.assignment, &self.practice] {
            draft.shape()?;
            if draft.task_id.is_empty()
                || draft.task_id.len() > 64
                || draft
                    .last
                    .as_ref()
                    .is_some_and(|r| r.before > draft.history.len())
            {
                return Err("Invalid saved DNS task or action position.".into());
            }
            for action in draft
                .history
                .iter()
                .chain(draft.last.as_ref().map(|r| &r.action))
            {
                let valid = match action {
                    Action::Save { config } => config.0.len() <= 8192,
                    Action::Query { name, .. } => name.0.len() <= 254,
                    Action::Transfer { zone, .. } => zone.0.len() <= 254,
                    _ => true,
                };
                if !valid {
                    return Err("Saved DNS operation exceeds its input bound.".into());
                }
            }
        }
        Ok(())
    }
    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.validate(definition)?;
        let mut next = self.clone();
        let output = if input == "dns-check" {
            next.assignment.check(&definition.initial)?;
            "Checked current simulated DNS requirements.".into()
        } else {
            let raw = input
                .strip_prefix("dns-policy ")
                .ok_or("Use native DNS controls; the authoring helper is not a Linux command.")?;
            if raw.len() > 32768 {
                return Err("Simulator-only DNS operation exceeds32KiB.".into());
            }
            let action: Action = serde_json::from_str(raw)
                .map_err(|_| "Invalid or unsupported simulator-only DNS operation.")?;
            if let Action::Save { config } = &action {
                next.assignment.set_editor(config.0.clone())?;
            }
            next.assignment.run(&definition.initial, action)??
        };
        next.validate(definition)?;
        *self = next;
        Ok(output)
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::DnsAccess { simulation },
        crate::engine::Goal::DnsAccess { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("DNS access state and objectives must use the same authored task.".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition)? {
        return Err(
            "DNS work must start with separate declared example and assignment drafts.".into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::DnsAccess { .. })
        || matches!(s.initial, crate::engine::State::DnsAccess { .. })
        || matches!(s.goal, crate::engine::Goal::DnsAccess { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let definition = source(s)?;
    let crate::engine::State::DnsAccess { simulation } = &a.state else {
        return Err("Saved DNS work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible DNS access metadata; original progress retained.".into());
    }
    simulation.validate(definition)
}
