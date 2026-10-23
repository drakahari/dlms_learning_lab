//! Typed native ACL drafts/actions and fresh checks. No terminal or host I/O.
use crate::{acl, acl_editor as editor, acl_routing as routing};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const HELP: &str = "Edit the complete selected virtual router's ACL document and use Check configuration. Names are supplied virtual interface labels. This is a bounded native editor, not a full IOS/Linux terminal. Only the stated topology and service flows are verified; no real packets or host execution.";
pub fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
pub fn valid_network(network: &routing::Network) -> Result<(), String> {
    network.validate().map_err(|e| e.message)?;
    if network
        .devices
        .keys()
        .any(|id| network.host_endpoints.contains_key(id))
    {
        return Err(
            "Native ACL policies must belong to transit routers, not host endpoints".into(),
        );
    }
    let mut total = 0;
    for router in network
        .topology
        .devices
        .keys()
        .filter(|id| !network.host_endpoints.contains_key(*id))
    {
        let text = editor::render_router(network, router).map_err(|e| e.to_string())?;
        total += text.len();
        if total > editor::MAX_DRAFT_TOTAL {
            return Err("Rendered ACL documents exceed the bounded workspace".into());
        }
    }
    Ok(())
}
pub fn same_assignment(network: &routing::Network, initial: &routing::Network) -> bool {
    valid_network(network).is_ok()
        && valid_network(initial).is_ok()
        && network.topology == initial.topology
        && network.host_endpoints == initial.host_endpoints
}
/// Parse against the original state and check FINAL totals before the only network clone.
pub fn apply_documents(
    current: &routing::Network,
    documents: &editor::Drafts,
) -> Result<routing::Network, String> {
    valid_network(current)?;
    documents.validate(current).map_err(|e| e.to_string())?;
    let mut parsed = BTreeMap::new();
    let mut new_lists = 0;
    let mut new_rules = 0;
    for (router, text) in &documents.routers {
        let config =
            editor::parse_router(current, router, text).map_err(|e| format!("{router}: {e}"))?;
        new_lists += config.policy.lists.len();
        new_rules += config
            .policy
            .lists
            .values()
            .map(|l| l.rules.len())
            .sum::<usize>();
        if new_lists > acl::MAX_LISTS || new_rules > acl::MAX_LISTS * acl::MAX_RULES {
            return Err("Supplied replacement documents exceed eight lists or512 rules".into());
        }
        parsed.insert(router.clone(), config);
    }
    let mut lists = new_lists;
    let mut rules = new_rules;
    for (router, config) in &current.devices {
        if !parsed.contains_key(router) {
            lists += config.policy.lists.len();
            rules += config
                .policy
                .lists
                .values()
                .map(|l| l.rules.len())
                .sum::<usize>();
        }
    }
    if lists > acl::MAX_LISTS || rules > acl::MAX_LISTS * acl::MAX_RULES {
        return Err("Final ACL network exceeds eight lists or512 rules; no router changed".into());
    }
    let mut candidate = current.clone();
    for (router, config) in parsed {
        if config.policy.lists.is_empty() && config.attachments.is_empty() {
            candidate.devices.remove(&router);
        } else {
            candidate.devices.insert(router, config);
        }
    }
    valid_network(&candidate)?;
    Ok(candidate)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub documents: editor::Drafts,
    pub selected_router: Option<String>,
    pub notice: Option<String>,
}
impl Draft {
    pub fn new(network: &routing::Network) -> Result<Self, String> {
        valid_network(network)?;
        let mut routers = BTreeMap::new();
        for router in network
            .topology
            .devices
            .keys()
            .filter(|id| !network.host_endpoints.contains_key(*id))
        {
            routers.insert(
                router.clone(),
                editor::render_router(network, router).map_err(|e| e.to_string())?,
            );
        }
        let draft = Self {
            version: 1,
            selected_router: routers.keys().next().cloned(),
            documents: editor::Drafts {
                schema_version: 1,
                routers,
            },
            notice: None,
        };
        if !draft.same_shape(network) {
            return Err("Cannot create a bounded ACL draft".into());
        }
        Ok(draft)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.documents.schema_version == 1
            && self.documents.routers.len() <= 12
            && self
                .documents
                .routers
                .iter()
                .all(|(id, text)| name(id) && text.len() <= editor::MAX_TEXT)
            && self
                .documents
                .routers
                .values()
                .map(String::len)
                .sum::<usize>()
                <= editor::MAX_DRAFT_TOTAL
            && self
                .selected_router
                .as_ref()
                .is_none_or(|s| self.documents.routers.contains_key(s))
            && self.notice.as_ref().is_none_or(|s| s.len() <= 512)
    }
    pub fn same_shape(&self, network: &routing::Network) -> bool {
        self.valid()
            && network.validate().is_ok()
            && self.documents.validate(network).is_ok()
            && self.documents.routers.len()
                == network
                    .topology
                    .devices
                    .keys()
                    .filter(|id| !network.host_endpoints.contains_key(*id))
                    .count()
            && network
                .topology
                .devices
                .keys()
                .filter(|id| !network.host_endpoints.contains_key(*id))
                .all(|id| self.documents.routers.contains_key(id))
            && (self.selected_router.is_some() || self.documents.routers.is_empty())
    }
    pub fn matches(&self, network: &routing::Network) -> bool {
        self.same_shape(network) && self.documents.matches(network).is_ok()
    }
    pub fn apply(&mut self, network: &mut routing::Network) -> Result<(), String> {
        if !self.same_shape(network) {
            return Err("ACL draft router identities differ from the workspace".into());
        }
        let next = apply_documents(network, &self.documents)?;
        *network = next;
        self.notice = Some(
            "Configuration applied in this simulated network; check the stated traffic flows."
                .into(),
        );
        Ok(())
    }
}
fn location(at: &routing::Location) -> String {
    format!(
        "{}/{} {}",
        at.device,
        at.interface,
        match at.direction {
            routing::Direction::Inbound => "in",
            routing::Direction::Outbound => "out",
        }
    )
}
fn cause_matches(expected: &Option<routing::ExpectedCause>, actual: &acl::Cause) -> bool {
    match (expected, actual) {
        (None, _) => true,
        (Some(routing::ExpectedCause::Implicit {}), acl::Cause::ImplicitDeny) => true,
        (
            Some(routing::ExpectedCause::Explicit { rule }),
            acl::Cause::Matched { rule: hit, .. },
        ) => rule.as_ref().is_none_or(|r| r == hit),
        _ => false,
    }
}
pub fn checks(
    network: &routing::Network,
    definition: &editor::Definition,
    documents: &editor::Drafts,
) -> Vec<(bool, String)> {
    let report = match editor::grade(definition, network, documents) {
        Err(e) => return vec![(false, e.to_string())],
        Ok(report) => report,
    };
    report.cases.into_iter().zip(&definition.cases).map(|(case,task)| {
        let detail = match &case.observation.actual {
            routing::Outcome::Delivered{..} => if case.passed { "allowed as required".to_owned() }
                else { "allowed, but this flow must be blocked. Check the attachment direction and matching rule.".into() },
            routing::Outcome::Denied{denial,..} => {
                let cause = match &denial.decision.cause {
                    acl::Cause::Matched{rule,row} => match rule.strip_prefix("seq-").and_then(|s|s.parse::<u16>().ok()) {
                        Some(sequence) => format!("sequence {sequence} (rule {})",row+1), None=>format!("rule {}",row+1),
                    },
                    acl::Cause::ImplicitDeny => "implicit deny (no rule matched)".into(),
                };
                let observed=format!("at {} via {cause}",location(&denial.at));
                if case.passed { format!("denied as required {observed}") }
                else { match &task.expectation {
                    editor::Expectation::Allowed{} => format!("blocked {observed}. This flow must be allowed; check matching and earlier rules."),
                    editor::Expectation::Denied{expected} if denial.at != expected.at => format!("blocked {observed}, but the task requires {}. Check the interface and direction.",location(&expected.at)),
                    editor::Expectation::Denied{expected} if !cause_matches(&expected.cause,&denial.decision.cause) => format!("blocked {observed}, but the task requires a different deny rule or explicit/implicit cause. Review that requirement."),
                    editor::Expectation::Denied{..} => format!("block observed {observed}. Required allowed traffic also fails; restore those flows before this security check can pass."),
                }}
            },
            routing::Outcome::Failed{failure,..} => format!("path/configuration failure is not an ACL denial: {}",failure.message),
        };
        (case.passed,format!("{}: {detail}",case.label))
    }).collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    ApplyDocuments { documents: editor::Drafts },
}
impl Action {
    pub fn apply(&self, network: &mut routing::Network) -> Result<(), String> {
        match self {
            Self::ApplyDocuments { documents } => {
                let candidate = apply_documents(network, documents)?;
                *network = candidate;
                Ok(())
            }
        }
    }
}
pub fn adapter(network: &mut routing::Network, input: &str) -> Result<String, String> {
    if input.len() > editor::MAX_JSON + 64 {
        return Err("ACL authoring action exceeds2 MB".into());
    }
    let text=input.strip_prefix("acl-author ").ok_or("Use native configuration editing. acl-author is an internal pack reachability adapter, not Cisco or Linux syntax.")?;
    let action: Action =
        serde_json::from_str(text).map_err(|_| "Malformed or unsupported ACL authoring data")?;
    action.apply(network)?;
    Ok("Typed ACL authoring action applied in memory only".into())
}
