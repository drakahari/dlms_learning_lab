//! Proposed bounded DNS-policy world. Unintegrated; no curriculum credit.
use super::{acl::Acl, config::Policy};
use crate::{
    dns::{self, Data, QueryType, Server, Zone},
    dns_workspace,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneSource {
    pub secondary_client: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub file: String,
    pub server: Server,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    #[serde(deserialize_with = "crate::cron::text::<_,254>")]
    pub name: String,
    pub kind: QueryType,
}
impl Key {
    pub fn new(name: &str, kind: QueryType) -> Result<Self, String> {
        Ok(Self {
            name: dns::canonical_name(name).map_err(|e| e.message)?,
            kind,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ZoneSource,4>")]
    pub zones: Vec<ZoneSource>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Zone,4>")]
    pub upstream: Vec<Zone>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Key,16>")]
    pub warm_cache: Vec<Key>,
}
fn within(name: &str, origin: &str) -> bool {
    name == origin || name.ends_with(&format!(".{origin}"))
}
fn zone_ok(zone: &Zone) -> Result<(), String> {
    zone.check_configuration().map_err(|e| e.message)?;
    let origin = dns::canonical_name(&zone.origin).map_err(|e| e.message)?;
    if origin != zone.origin {
        return Err("Supplied zone origins must use canonical full names.".into());
    }
    for record in &zone.records {
        if let Data::Cname { target } = &record.data
            && !within(
                &dns::canonical_name(target).map_err(|e| e.message)?,
                &origin,
            )
        {
            return Err("Cross-zone aliases are unsupported in this access-policy model.".into());
        }
    }
    Ok(())
}
impl Facts {
    pub fn supports_name(&self, name: &str) -> bool {
        let Ok(name) = dns::canonical_name(name) else {
            return false;
        };
        self.zones
            .iter()
            .any(|z| within(&name, &z.server.primary.origin))
            || self.upstream.iter().any(|z| within(&name, &z.origin))
    }
    pub fn known(&self) -> BTreeMap<String, String> {
        self.zones
            .iter()
            .map(|z| (z.server.primary.origin.clone(), z.file.clone()))
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.zones.is_empty()
            || self.zones.len() > 4
            || self.upstream.len() > 4
            || self.warm_cache.len() > 16
        {
            return Err("Invalid or oversized supplied DNS policy world.".into());
        }
        let mut origins = vec![];
        let mut files = BTreeSet::new();
        for z in &self.zones {
            if z.secondary_client.is_unspecified()
                || z.secondary_client.is_multicast()
                || z.secondary_client.is_broadcast()
            {
                return Err("A supplied secondary needs a specified unicast IPv4 address.".into());
            }
            z.server.validate().map_err(|e| e.message)?;
            for zone in [&z.server.configured, &z.server.primary, &z.server.secondary] {
                zone_ok(zone)?;
            }
            let simple_file = !z.file.is_empty()
                && z.file.len() <= 128
                && !matches!(z.file.as_str(), "." | "..")
                && z.file
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b));
            if !(simple_file || crate::boot_entry::path(&z.file)) || !files.insert(&z.file) {
                return Err(
                    "Supply unique virtual zone filenames without traversal or escapes.".into(),
                );
            }
            origins.push(z.server.primary.origin.as_str());
        }
        for zone in &self.upstream {
            zone_ok(zone)?;
            origins.push(&zone.origin);
        }
        for (i, a) in origins.iter().enumerate() {
            for b in &origins[..i] {
                if within(a, b) || within(b, a) {
                    return Err(
                        "Overlapping authoritative/upstream fixtures are unsupported.".into(),
                    );
                }
            }
        }
        let mut seen = BTreeSet::new();
        for key in &self.warm_cache {
            if Key::new(&key.name, key.kind)? != *key || !seen.insert((key.name.clone(), key.kind))
            {
                return Err("Initial cache keys must be unique canonical questions.".into());
            }
            self.upstream_answer(key)?;
        }
        Ok(())
    }
    fn upstream_answer(&self, key: &Key) -> Result<dns::Answer, String> {
        let zone=self.upstream.iter().find(|z|within(&key.name,&z.origin)).ok_or("No supplied upstream data exists for this name. This simulator will not perform a real lookup or invent an answer.")?;
        let mut answer = zone.query(&key.name, key.kind).map_err(|e| e.message)?;
        if !matches!(answer.code, dns::Code::Noerror | dns::Code::Nxdomain) {
            return Err(
                "Only completed supplied upstream answers can populate this finite cache.".into(),
            );
        }
        answer.authoritative = false;
        Ok(answer)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Authoritative,
    Cached,
    SuppliedRecursion,
    NoRecursion,
    DeniedQuery,
    DeniedCache,
    DeniedRecursion,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub route: Route,
    pub answer: Option<dns::Answer>,
    pub note: String,
}
impl Reply {
    pub fn text(&self) -> Result<String, String> {
        let mut text = self.note.clone();
        if let Some(answer) = &self.answer {
            let code = match answer.code {
                dns::Code::Noerror => "NOERROR",
                dns::Code::Nxdomain => "NXDOMAIN",
                dns::Code::Refused => "REFUSED",
                dns::Code::Servfail => "SERVFAIL",
            };
            let source = if answer.authoritative {
                "this server's zone"
            } else {
                "supplied upstream/cache data"
            };
            text.push_str(&format!("\nResult: {code} · Answer source: {source}\n"));
            for row in &answer.records {
                text.push_str(&dns_workspace::record_text(row));
                text.push('\n');
            }
            text.push_str(&answer.note);
        }
        if text.len() > 32768 {
            return Err("Query result exceeds the simulator’s 32 KiB output limit.".into());
        }
        Ok(text)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferReply {
    Refused,
    Secondary {
        decision: dns::Transfer,
        primary_serial: u32,
        secondary_serial: u32,
        records: usize,
    },
    OtherClient {
        zone: Zone,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Cached {
    key: Key,
    answer: dns::Answer,
}
/// Constructed only from validated source facts and replayed bounded actions.
/// No imported world/output/cache is accepted as learner truth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    facts: Facts,
    saved_text: String,
    saved: Policy,
    live: Policy,
    zones: Vec<ZoneSource>,
    cache: Vec<Cached>,
}
impl World {
    pub fn new(facts: &Facts, config: &str) -> Result<Self, String> {
        facts.validate()?;
        let policy = Policy::parse(config, &facts.known())?;
        let cache = facts
            .warm_cache
            .iter()
            .map(|key| {
                Ok(Cached {
                    key: key.clone(),
                    answer: facts.upstream_answer(key)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            facts: facts.clone(),
            saved_text: config.into(),
            saved: policy.clone(),
            live: policy,
            zones: facts.zones.clone(),
            cache,
        })
    }
    pub fn source_is(&self, facts: &Facts) -> bool {
        &self.facts == facts
    }
    pub fn saved_text(&self) -> &str {
        &self.saved_text
    }
    pub fn live(&self) -> &Policy {
        &self.live
    }
    pub fn saved(&self) -> &Policy {
        &self.saved
    }
    pub fn zones(&self) -> &[ZoneSource] {
        &self.zones
    }
    pub fn cache_keys(&self) -> Vec<Key> {
        self.cache.iter().map(|e| e.key.clone()).collect()
    }
    pub fn save(&mut self, text: &str) -> Result<(), String> {
        let policy = Policy::parse(text, &self.facts.known())?;
        self.saved = policy;
        self.saved_text = text.into();
        Ok(())
    }
    pub fn reload(&mut self) {
        self.live = self.saved.clone();
    }
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    pub fn effective_cache(&self) -> Result<Vec<(u32, u32)>, String> {
        Acl::intersection(&[&self.live.query, &self.live.cache])
    }
    pub fn effective_recursion(&self) -> Result<Vec<(u32, u32)>, String> {
        if !self.live.recursion {
            return Ok(vec![]);
        }
        Acl::intersection(&[&self.live.query, &self.live.cache, &self.live.recursive])
    }
    /// Deterministic grading fixture: restore only supplied initial cache data
    /// in a cloned case world. Never imported answers or learner completion.
    pub fn reset_cache_to_supplied(&mut self) -> Result<(), String> {
        let cache = self
            .facts
            .warm_cache
            .iter()
            .map(|key| {
                Ok(Cached {
                    key: key.clone(),
                    answer: self.facts.upstream_answer(key)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.cache = cache;
        Ok(())
    }

    pub fn query(
        &mut self,
        client: Ipv4Addr,
        name: &str,
        kind: QueryType,
        recurse: bool,
    ) -> Result<Reply, String> {
        let key = Key::new(name, kind)?;
        let empty = |route, note: &str| Reply {
            route,
            answer: None,
            note: note.into(),
        };
        if !self.live.query.permits(client)? {
            return Ok(empty(
                Route::DeniedQuery,
                "Query refused: this simulated client is outside allow-query.",
            ));
        }
        if let Some(zone) = self
            .zones
            .iter()
            .find(|z| within(&key.name, &z.server.primary.origin))
        {
            let reply = Reply {
                route: Route::Authoritative,
                answer: Some(
                    zone.server
                        .primary
                        .query(&key.name, kind)
                        .map_err(|e| e.message)?,
                ),
                note:
                    "Authoritative answer from the supplied primary zone; no recursion is needed."
                        .into(),
            };
            reply.text()?;
            return Ok(reply);
        }
        if !self.live.cache.permits(client)? {
            return Ok(empty(
                Route::DeniedCache,
                "Cache access refused: allow-query-cache excludes this simulated client.",
            ));
        }
        if let Some(entry) = self.cache.iter().find(|e| e.key == key) {
            let reply=Reply{route:Route::Cached,answer:Some(entry.answer.clone()),note:"Existing simulated cached answer. Disabling recursion alone does not remove cache access.".into()};
            reply.text()?;
            return Ok(reply);
        }
        if !recurse || !self.live.recursion {
            return Ok(empty(
                Route::NoRecursion,
                "No cached answer. A new recursive lookup is not enabled/requested; full referral packets are outside this model.",
            ));
        }
        if !self.live.recursive.permits(client)? {
            return Ok(empty(
                Route::DeniedRecursion,
                "Recursive lookup refused: allow-recursion excludes this simulated client.",
            ));
        }
        if self.cache.len() == 32 {
            return Err(
                "The simulated cache is full (32 questions); clear it before another new lookup."
                    .into(),
            );
        }
        let answer = self.facts.upstream_answer(&key)?;
        let reply=Reply{route:Route::SuppliedRecursion,answer:Some(answer.clone()),note:"Answered from supplied upstream data and added to the simulated cache. No network lookup occurred.".into()};
        reply.text()?;
        self.cache.push(Cached { key, answer });
        Ok(reply)
    }
    pub fn transfer(&mut self, origin: &str, client: Ipv4Addr) -> Result<TransferReply, String> {
        let origin = dns::canonical_name(origin).map_err(|e| e.message)?;
        if !self.live.transfer_for(&origin)?.permits(client)? {
            return Ok(TransferReply::Refused);
        }
        let source = self
            .zones
            .iter_mut()
            .find(|z| z.server.primary.origin == origin)
            .ok_or("No supplied primary has that zone.")?;
        if client != source.secondary_client {
            // A served transfer to some other client is a disclosure, not an
            // update to the exercise's independently identified secondary.
            return Ok(TransferReply::OtherClient {
                zone: source.server.primary.clone(),
            });
        }
        let decision = source.server.update_secondary().map_err(|e| e.message)?;
        Ok(TransferReply::Secondary {
            decision,
            primary_serial: source.server.primary.serial,
            secondary_serial: source.server.secondary.serial,
            records: source.server.secondary.records.len(),
        })
    }
    pub fn effective_transfer(&self, origin: &str) -> Result<&Acl, String> {
        self.live.transfer_for(origin)
    }
}
