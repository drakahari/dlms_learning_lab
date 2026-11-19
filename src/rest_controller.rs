//! A finite, in-process teaching origin. No HTTP transport, sockets or credentials.
//!
//! Resources project into the existing Ethernet simulator. HTTP-like status is
//! request feedback, never completion authority. See docs/REST_CONTROLLER_SUBSET.md.
use crate::{json_inventory as inventory, switching};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const MAX_BODY: usize = 4096;
pub const MAX_PROFILES: usize = 12;

pub fn identifier(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 32
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn bounded_map<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    struct Entries<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} unique named entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if out.len() == N || !identifier(&key) || out.contains_key(&key) {
                    return Err(de::Error::custom(
                        "Entry limit, invalid identifier or duplicate key",
                    ));
                }
                out.insert(key, map.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries::<T, N>(std::marker::PhantomData))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub up: bool,
    pub vlan: u16,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_description"
    )]
    pub description: Option<String>,
}
fn optional_description<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<String>, D::Error> {
    // The origin omits absent descriptions. JSON null is a patch deletion, not a
    // stored description; requiring a string avoids pretending null is a string.
    crate::cron::text::<D, 160>(d).map(Some)
}
impl Profile {
    fn validate(&self) -> Result<(), String> {
        if !(1..=4094).contains(&self.vlan)
            || self
                .description
                .as_ref()
                .is_some_and(|s| s.len() > 160 || s.chars().any(char::is_control))
        {
            return Err("A profile needs up as true/false, vlan as an integer from 1 to 4094, and an optional printable description up to 160 bytes.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub switch: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub profile: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    #[serde(deserialize_with = "bounded_map::<_,Profile,12>")]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(deserialize_with = "bounded_map::<_,Binding,12>")]
    pub bindings: BTreeMap<String, Binding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub inventory: inventory::Spec,
    #[serde(deserialize_with = "bounded_map::<_,Slot,12>")]
    pub ports: BTreeMap<String, Slot>,
}
impl Spec {
    pub fn validate(&self) -> Result<(), String> {
        self.inventory.validate()?;
        let expected: BTreeSet<_> = self
            .inventory
            .editable
            .iter()
            .flat_map(|(s, ps)| ps.iter().map(move |p| (s.clone(), p.clone())))
            .collect();
        let actual: BTreeSet<_> = self
            .ports
            .values()
            .map(|p| (p.switch.clone(), p.port.clone()))
            .collect();
        if self.ports.is_empty()
            || self.ports.len() > 12
            || self.ports.keys().any(|s| !identifier(s))
            || actual.len() != self.ports.len()
            || actual != expected
        {
            return Err("Supply one unique resource name for every editable access port, and no other port.".into());
        }
        Ok(())
    }
    pub fn project(&self, store: &Store) -> Result<switching::Fabric, String> {
        self.validate()?;
        if store.profiles.is_empty()
            || store.profiles.len() > MAX_PROFILES
            || store.profiles.keys().any(|s| !identifier(s))
            || store.bindings.keys().ne(self.ports.keys())
        {
            return Err("Keep 1–12 named profiles and exactly the supplied port bindings.".into());
        }
        for profile in store.profiles.values() {
            profile.validate()?;
        }
        let mut switches = BTreeMap::<String, Vec<inventory::PortRecord>>::new();
        for (name, slot) in &self.ports {
            let binding = &store.bindings[name];
            let profile = store.profiles.get(&binding.profile).ok_or_else(|| {
                format!("Binding {name} selects a missing profile. Create it before binding.")
            })?;
            switches
                .entry(slot.switch.clone())
                .or_default()
                .push(inventory::PortRecord {
                    name: slot.port.clone(),
                    up: profile.up,
                    vlan: profile.vlan,
                });
        }
        let document = inventory::Document {
            note: None,
            switches: switches
                .into_iter()
                .map(|(name, ports)| inventory::SwitchRecord { name, ports })
                .collect(),
        };
        self.inventory
            .apply(&serde_json::to_string(&document).map_err(|e| e.to_string())?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub method: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub authorization: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub content_type: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub body: String,
}
impl Request {
    pub fn validate(&self) -> Result<(), String> {
        if self.method.is_empty()
            || self.method.len() > 16
            || !self.method.bytes().all(|b| b.is_ascii_alphabetic())
            || self.path.len() > 96
            || self.authorization.len() > 96
            || self.content_type.len() > 96
            || self.body.len() > MAX_BODY
            || [&self.path, &self.authorization, &self.content_type]
                .iter()
                .any(|s| !s.is_ascii() || s.chars().any(char::is_control))
            || self.body.contains('\0')
        {
            return Err("Request exceeds the supported fields/4KiB body, contains controls, or has an invalid method token.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Response {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
    pub changed: bool,
}
impl Response {
    fn json(status: u16, value: Value) -> Self {
        Self {
            status,
            headers: BTreeMap::from([("Content-Type".into(), "application/json".into())]),
            body: serde_json::to_string_pretty(&value).expect("JSON value"),
            changed: false,
        }
    }
    fn error(status: u16, message: impl Into<String>) -> Self {
        Self::json(status, serde_json::json!({"error":message.into()}))
    }
    fn no_content(changed: bool) -> Self {
        Self {
            status: 204,
            headers: BTreeMap::new(),
            body: String::new(),
            changed,
        }
    }
}

enum Target<'a> {
    Profiles,
    Profile(&'a str),
    Bindings,
    Binding(&'a str),
}
fn target(path: &str) -> Result<Target<'_>, Response> {
    match path {
        "/profiles" => Ok(Target::Profiles),
        "/bindings" => Ok(Target::Bindings),
        _ => {
            for (prefix, profile) in [("/profiles/", true), ("/bindings/", false)] {
                if let Some(name) = path.strip_prefix(prefix) {
                    if identifier(name) {
                        return Ok(if profile {
                            Target::Profile(name)
                        } else {
                            Target::Binding(name)
                        });
                    }
                    return Err(Response::error(
                        400,
                        "Use a supplied resource name: lowercase letters, digits and hyphens. Encoded paths, queries and nested paths are outside this origin's subset.",
                    ));
                }
            }
            if !path.starts_with('/') || path.contains(['?', '#', '%']) || path.contains("..") {
                Err(Response::error(
                    400,
                    "Use a resource path, not a URL, query, encoded path or filesystem path.",
                ))
            } else {
                Err(Response::error(
                    404,
                    "No resource at this path. Read /profiles or /bindings.",
                ))
            }
        }
    }
}
fn method_allowed(method: &str, target: &Target<'_>) -> Result<(), Response> {
    let allow = match target {
        Target::Profiles => "GET, POST",
        Target::Profile(_) => "GET, PUT, PATCH, DELETE",
        Target::Bindings => "GET",
        Target::Binding(_) => "GET, PUT, PATCH",
    };
    if allow.split(", ").any(|s| s == method) {
        Ok(())
    } else {
        let mut response = Response::error(
            405,
            format!("{method} is not allowed here. Allowed methods: {allow}."),
        );
        response.headers.insert("Allow".into(), allow.into());
        Err(response)
    }
}

/// Evaluates one bounded request, atomically committing only valid resource state.
pub fn send(spec: &Spec, store: &mut Store, request: &Request) -> Result<Response, String> {
    // Invalid source/saved state is not disguised as an HTTP response.
    spec.project(store)?;
    request.validate()?;
    let mut words = request.authorization.split(' ');
    let role = match (words.next(), words.next(), words.next()) {
        (Some(scheme), Some("lab-reader"), None) if scheme.eq_ignore_ascii_case("Bearer") => {
            Some(false)
        }
        (Some(scheme), Some("lab-operator"), None) if scheme.eq_ignore_ascii_case("Bearer") => {
            Some(true)
        }
        _ => None,
    };
    let Some(operator) = role else {
        let mut result = Response::error(
            401,
            "Missing or invalid simulated identity. Use the supplied lab-reader or lab-operator Bearer value; these are teaching labels, not real credentials.",
        );
        result.headers.insert(
            "WWW-Authenticate".into(),
            "Bearer realm=\"learning-lab\"".into(),
        );
        return Ok(result);
    };
    if !["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&request.method.as_str()) {
        return Ok(Response::error(
            501,
            "Unsupported method. This finite origin implements GET, POST, PUT, PATCH and DELETE only; method names are case-sensitive.",
        ));
    }
    if request.method != "GET" && !operator {
        return Ok(Response::error(
            403,
            "This identity can read, but cannot change resources. Choose the supplied operator identity for a change.",
        ));
    }
    let target = match target(&request.path) {
        Ok(t) => t,
        Err(r) => return Ok(r),
    };
    if let Err(response) = method_allowed(&request.method, &target) {
        return Ok(response);
    }
    if request.method == "GET" || request.method == "DELETE" {
        if !request.body.trim().is_empty() {
            return Ok(Response::error(
                400,
                "This origin does not accept a body on GET or DELETE. Clear the body before sending.",
            ));
        }
    } else {
        let expected = if request.method == "PATCH" {
            "application/merge-patch+json"
        } else {
            "application/json"
        };
        if !request
            .content_type
            .trim_matches(' ')
            .eq_ignore_ascii_case(expected)
        {
            return Ok(Response::error(
                415,
                format!(
                    "Use Content-Type: {expected} for {}. Media-type parameters and other encodings are outside this origin's subset.",
                    request.method
                ),
            ));
        }
    }
    if request.method == "GET" {
        return Ok(match target {
            Target::Profiles => Response::json(
                200,
                serde_json::to_value(&store.profiles).expect("profiles"),
            ),
            Target::Bindings => Response::json(
                200,
                serde_json::to_value(&store.bindings).expect("bindings"),
            ),
            Target::Profile(name) => store
                .profiles
                .get(name)
                .map(|v| Response::json(200, serde_json::to_value(v).expect("profile")))
                .unwrap_or_else(|| {
                    Response::error(404, "Profile not found. Read /profiles for current names.")
                }),
            Target::Binding(name) => store
                .bindings
                .get(name)
                .map(|v| Response::json(200, serde_json::to_value(v).expect("binding")))
                .unwrap_or_else(|| {
                    Response::error(404, "Binding not found. Read /bindings for supplied names.")
                }),
        });
    }
    if request.method == "DELETE" {
        let Target::Profile(name) = target else {
            unreachable!("method allowed")
        };
        if !store.profiles.contains_key(name) {
            return Ok(Response::error(404, "Profile not found."));
        }
        if store.bindings.values().any(|b| b.profile == name) {
            return Ok(Response::error(
                409,
                "Profile is still selected by a port. Rebind its ports before retiring it.",
            ));
        }
        let mut next = store.clone();
        next.profiles.remove(name);
        if let Err(e) = spec.project(&next) {
            return Ok(Response::error(422, e));
        }
        *store = next;
        return Ok(Response::no_content(true));
    }
    let body = match parse_json(&request.body) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Response::error(
                400,
                format!("JSON syntax or bounds error: {e}"),
            ));
        }
    };
    let mut next = store.clone();
    let mut created = false;
    let result: Result<(), String> = (|| {
        match target {
            Target::Profiles => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct NewProfile {
                    name: String,
                    up: bool,
                    vlan: u16,
                    #[serde(default, deserialize_with = "optional_description")]
                    description: Option<String>,
                }
                let record: NewProfile =
                    serde_json::from_value(resource_object(body)?).map_err(|e| e.to_string())?;
                if !identifier(&record.name) {
                    return Err(
                        "Use a lowercase profile name with digits/hyphens, at most 32 characters."
                            .into(),
                    );
                }
                if next.profiles.contains_key(&record.name) {
                    return Err("CONFLICT:Profile name already exists. Use PUT or PATCH on its resource, or choose another name.".into());
                }
                if next.profiles.len() == MAX_PROFILES {
                    return Err("CONFLICT:At most 12 profiles; retire an unused profile before creating another.".into());
                }
                next.profiles.insert(
                    record.name,
                    Profile {
                        up: record.up,
                        vlan: record.vlan,
                        description: record.description,
                    },
                );
                created = true;
            }
            Target::Profile(name) => {
                let value = if request.method == "PATCH" {
                    let old = store
                        .profiles
                        .get(name)
                        .ok_or("NOT_FOUND:Profile not found; create it with POST or PUT first.")?;
                    merge(serde_json::to_value(old).expect("profile"), body)
                } else {
                    created = !store.profiles.contains_key(name);
                    body
                };
                let profile: Profile =
                    serde_json::from_value(resource_object(value)?).map_err(|e| e.to_string())?;
                if created && next.profiles.len() == MAX_PROFILES {
                    return Err("CONFLICT:At most 12 profiles.".into());
                }
                next.profiles.insert(name.into(), profile);
            }
            Target::Binding(name) => {
                let old = store
                    .bindings
                    .get(name)
                    .ok_or("NOT_FOUND:Binding not found. Only supplied ports have bindings.")?;
                let value = if request.method == "PATCH" {
                    merge(serde_json::to_value(old).expect("binding"), body)
                } else {
                    body
                };
                let binding: Binding =
                    serde_json::from_value(resource_object(value)?).map_err(|e| e.to_string())?;
                next.bindings.insert(name.into(), binding);
            }
            Target::Bindings => unreachable!("method allowed"),
        }
        spec.project(&next)?;
        Ok(())
    })();
    if let Err(error) = result {
        let (status, message) = if let Some(m) = error.strip_prefix("CONFLICT:") {
            (409, m)
        } else if let Some(m) = error.strip_prefix("NOT_FOUND:") {
            (404, m)
        } else {
            (422, error.as_str())
        };
        return Ok(Response::error(
            status,
            format!("{message} No resource changed."),
        ));
    }
    let changed = next != *store;
    *store = next;
    if created {
        // POST carries its chosen name; PUT uses the target's existing path.
        let location = if request.method == "POST" {
            let name: Value = parse_json(&request.body).expect("already parsed");
            format!(
                "/profiles/{}",
                name["name"].as_str().expect("validated name")
            )
        } else {
            request.path.clone()
        };
        let mut response = Response::json(201, serde_json::json!({"resource":location}));
        response.headers.insert("Location".into(), location);
        response.changed = changed;
        Ok(response)
    } else {
        Ok(Response::no_content(changed))
    }
}

// Original bounded implementation of JSON Merge Patch semantics. This helper is
// private: every result still passes the origin's flat typed resource schema.
fn resource_object(value: Value) -> Result<Value, String> {
    if value.is_object() {
        Ok(value)
    } else {
        Err("This resource requires a JSON object with named fields; arrays, strings, numbers and null cannot represent it.".into())
    }
}
fn merge(previous: Value, changes: Value) -> Value {
    match changes {
        Value::Object(fields) => {
            let mut result = match previous {
                Value::Object(m) => m,
                _ => Map::new(),
            };
            for (key, value) in fields {
                if value.is_null() {
                    result.remove(&key);
                } else {
                    let prior = result.remove(&key).unwrap_or(Value::Null);
                    result.insert(key, merge(prior, value));
                }
            }
            Value::Object(result)
        }
        replacement => replacement,
    }
}

// Reject ambiguous duplicate keys at every depth, before typed conversion. Depth
// and node limits bound parsing itself, not just the accepted final resource.
struct JsonSeed<'a> {
    remaining: &'a mut usize,
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for JsonSeed<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        if self.depth > 8 || *self.remaining == 0 {
            return Err(de::Error::custom(
                "Use at most 8 nested levels and 128 JSON values.",
            ));
        }
        *self.remaining -= 1;
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for JsonSeed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded unambiguous JSON")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("Nonfinite number"))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.into()))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut out = Vec::new();
        while let Some(v) = seq.next_element_seed(JsonSeed {
            remaining: self.remaining,
            depth: self.depth + 1,
        })? {
            out.push(v);
        }
        Ok(Value::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if out.contains_key(&key) {
                return Err(de::Error::custom(format!("Duplicate JSON field {key}")));
            }
            out.insert(
                key,
                map.next_value_seed(JsonSeed {
                    remaining: self.remaining,
                    depth: self.depth + 1,
                })?,
            );
        }
        Ok(Value::Object(out))
    }
}
fn parse_json(text: &str) -> Result<Value, String> {
    if text.len() > MAX_BODY || text.contains('\0') {
        return Err("Keep JSON within 4 KiB with no NUL.".into());
    }
    let mut parser = serde_json::Deserializer::from_str(text);
    let value = JsonSeed {
        remaining: &mut 128,
        depth: 0,
    }
    .deserialize(&mut parser)
    .map_err(|e| e.to_string())?;
    parser.end().map_err(|e| e.to_string())?;
    Ok(value)
}
