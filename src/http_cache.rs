//! Bounded shared-cache decisions over supplied facts; no I/O.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_HEADER: usize = 1024;
pub const MAX_STEPS: usize = 64;
pub const MAX_PATHS: usize = 16;
pub const MAX_BODY: usize = 4096;
const AGE_LIMIT: u64 = 2_147_483_648;
type Result<T> = std::result::Result<T, Fault>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Syntax,
    Unsupported,
    Limit,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
fn fault(kind: FaultKind, message: impl Into<String>) -> Fault {
    Fault {
        kind,
        message: message.into(),
    }
}
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Control {
    pub no_store: bool,
    pub no_cache: bool,
    pub private: bool,
    pub public: bool,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
    pub max_age: Option<u64>,
    pub shared_max_age: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Request,
    Response,
}
fn seconds(value: &str) -> Result<u64> {
    let v = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    };
    if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fault(
            FaultKind::Syntax,
            "A freshness duration needs whole nonnegative seconds, such as max-age=60.",
        ));
    }
    // RFC delta-seconds saturation; do not wrap huge input or require u64 parsing.
    Ok(v.bytes().fold(0_u64, |n, b| {
        n.saturating_mul(10)
            .saturating_add(u64::from(b - b'0'))
            .min(AGE_LIMIT)
    }))
}
impl Control {
    /// Parses a deliberately closed Cache-Control field value, not HTTP messages.
    pub fn parse(raw: &str, side: Side) -> Result<Self> {
        if raw.len() > MAX_HEADER {
            return Err(fault(
                FaultKind::Limit,
                "Keep Cache-Control within 1024 bytes.",
            ));
        }
        if !raw.is_ascii() || raw.bytes().any(|b| b.is_ascii_control() && b != b'\t') {
            return Err(fault(
                FaultKind::Syntax,
                "Use one ASCII Cache-Control value without line breaks.",
            ));
        }
        let mut answer = Self::default();
        let mut seen = BTreeSet::new();
        if raw.trim().is_empty() {
            return Ok(answer);
        }
        for item in raw.split(',') {
            let item = item.trim_matches([' ', '\t']);
            if item.is_empty() {
                return Err(fault(
                    FaultKind::Unsupported,
                    "Empty list members are outside this lesson's header subset.",
                ));
            }
            let (name, arg) = match item.split_once('=') {
                Some((n, v)) => (n, Some(v)),
                None => (item, None),
            };
            if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
                return Err(fault(
                    FaultKind::Syntax,
                    "Use directive names separated by commas; do not put spaces around '='.",
                ));
            }
            let name = name.to_ascii_lowercase();
            if !seen.insert(name.clone()) {
                return Err(fault(
                    FaultKind::Unsupported,
                    format!(
                        "Repeated {name} directives need conflict handling outside this closed subset; keep one value."
                    ),
                ));
            }
            if seen.len() > 8 {
                return Err(fault(
                    FaultKind::Limit,
                    "At most eight distinct cache directives are supported.",
                ));
            }
            if side == Side::Request
                && !matches!(name.as_str(), "no-cache" | "no-store" | "max-age")
            {
                return Err(fault(
                    FaultKind::Unsupported,
                    format!(
                        "Request directive {name} is outside the supported no-cache, no-store, max-age subset."
                    ),
                ));
            }
            match name.as_str() {
                "max-age" | "s-maxage" => {
                    let value = seconds(arg.ok_or_else(|| {
                        fault(FaultKind::Syntax, format!("{name} needs '=seconds'."))
                    })?)?;
                    if name == "max-age" {
                        answer.max_age = Some(value);
                    } else {
                        answer.shared_max_age = Some(value);
                    }
                }
                "no-cache" | "no-store" | "private" | "public" | "must-revalidate"
                | "proxy-revalidate" => {
                    if arg.is_some() {
                        return Err(fault(
                            FaultKind::Unsupported,
                            format!(
                                "{name} with an argument is outside this subset; only unqualified flags are supported."
                            ),
                        ));
                    }
                    match name.as_str() {
                        "no-cache" => answer.no_cache = true,
                        "no-store" => answer.no_store = true,
                        "private" => answer.private = true,
                        "public" => answer.public = true,
                        "must-revalidate" => answer.must_revalidate = true,
                        _ => answer.proxy_revalidate = true,
                    }
                }
                _ => {
                    return Err(fault(
                        FaultKind::Unsupported,
                        format!(
                            "Cache directive {name} is not implemented; no behavior has been assumed."
                        ),
                    ));
                }
            }
        }
        Ok(answer)
    }
    pub fn freshness_lifetime(&self) -> u64 {
        self.shared_max_age.or(self.max_age).unwrap_or(0)
    }
    pub fn shared_storage_allowed(&self) -> bool {
        !self.no_store && !self.private
    }
}
fn path_ok(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 128
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        && !path.split('/').any(|v| v == "." || v == "..")
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OriginReply {
    pub response_time: u64,
    pub date: u64,
    pub age: u64,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub etag: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub body: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub request_time: u64,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub request_control: String,
    /// None means the source-held virtual origin is unavailable for this request.
    pub origin: Option<OriginReply>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Delivery {
    Cached,
    Validated,
    Downloaded,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub path: String,
    pub delivery: Delivery,
    pub body: Option<String>,
    pub etag: Option<String>,
    pub age: Option<u64>,
    pub origin_status: Option<u16>,
    pub stored: bool,
    pub explanation: String,
}
#[derive(Clone, Debug)]
struct Entry {
    reply: OriginReply,
    control: Control,
    initial_age: u64,
}
impl Entry {
    fn age(&self, now: u64) -> u64 {
        self.initial_age
            .saturating_add(now.saturating_sub(self.reply.response_time))
            .min(AGE_LIMIT)
    }
    fn new(reply: OriginReply, control: Control, request_time: u64) -> Self {
        let apparent = reply.response_time.saturating_sub(reply.date);
        let corrected = reply.age.saturating_add(reply.response_time - request_time);
        let initial_age = apparent.max(corrected).min(AGE_LIMIT);
        Self {
            reply,
            control,
            initial_age,
        }
    }
}
fn validate_steps(
    steps: &[Step],
    headers: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Control>> {
    if steps.is_empty()
        || steps.len() > MAX_STEPS
        || headers.is_empty()
        || headers.len() > MAX_PATHS
    {
        return Err(fault(
            FaultKind::Limit,
            "A cache trace needs1–64 requests and1–16 configured paths.",
        ));
    }
    let mut controls = BTreeMap::new();
    for (path, text) in headers {
        if !path_ok(path) {
            return Err(fault(
                FaultKind::Invalid,
                "Use supplied absolute virtual paths without query strings or dot segments.",
            ));
        }
        controls.insert(path.clone(), Control::parse(text, Side::Response)?);
    }
    let mut latest = 0;
    let mut versions = BTreeMap::new();
    for step in steps {
        if !controls.contains_key(&step.path) || step.request_time < latest {
            return Err(fault(
                FaultKind::Invalid,
                "Every request needs a configured path and nondecreasing source-held times after the preceding response.",
            ));
        }
        Control::parse(&step.request_control, Side::Request)?;
        latest = step.request_time;
        if let Some(o) = &step.origin {
            if o.response_time < step.request_time
                || o.response_time - step.request_time > 3600
                || o.body.len() > MAX_BODY
                || o.etag.is_empty()
                || o.etag.len() > 80
                || !o
                    .etag
                    .bytes()
                    .all(|b| b.is_ascii_graphic() && b != b'"' && b != b'\\')
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "A source-held origin reply needs bounded content, a strong opaque tag and an ordered response within one hour.",
                ));
            }
            let key = (step.path.clone(), o.etag.clone());
            if versions
                .insert(key, o.body.clone())
                .is_some_and(|prior| prior != o.body)
            {
                return Err(fault(
                    FaultKind::Invalid,
                    "One strong ETag cannot identify different bodies for the same path.",
                ));
            }
            latest = o.response_time;
        }
    }
    Ok(controls)
}
/// Runs a new private cache for one deterministic trace. No persistent cache or I/O.
pub fn run(steps: &[Step], headers: &BTreeMap<String, String>) -> Result<Vec<Observation>> {
    let controls = validate_steps(steps, headers)?;
    let mut cache: BTreeMap<String, Entry> = BTreeMap::new();
    let mut results = Vec::with_capacity(steps.len());
    for step in steps {
        let request = Control::parse(&step.request_control, Side::Request)?;
        let old = cache.get(&step.path);
        if let Some(entry) = old {
            let age = entry.age(step.request_time);
            if !request.no_cache
                && !entry.control.no_cache
                && age < entry.control.freshness_lifetime()
                && request.max_age.is_none_or(|limit| age <= limit)
            {
                results.push(Observation{path:step.path.clone(),delivery:Delivery::Cached,body:Some(entry.reply.body.clone()),etag:Some(entry.reply.etag.clone()),age:Some(age),origin_status:None,stored:true,explanation:format!("Stored response age {age} s is below its shared freshness lifetime {} s; this GET did not contact the origin.",entry.control.freshness_lifetime())});
                continue;
            }
        }
        let Some(origin) = &step.origin else {
            results.push(Observation{path:step.path.clone(),delivery:Delivery::Unavailable,body:None,etag:None,age:None,origin_status:None,stored:old.is_some(),explanation:"A usable fresh response was unavailable and the origin could not validate or replace it. This subset never serves stale content offline.".into()});
            continue;
        };
        let matched = old.is_some_and(|e| e.reply.etag == origin.etag);
        let mut reply = origin.clone();
        if let Some(previous) = old.filter(|_| matched) {
            reply.body = previous.reply.body.clone();
        }
        let entry = Entry::new(reply, controls[&step.path].clone(), step.request_time);
        let age = entry.age(origin.response_time);
        let body = entry.reply.body.clone();
        let etag = entry.reply.etag.clone();
        let store = !request.no_store && entry.control.shared_storage_allowed();
        // Once contacting the origin, discard the old entry before possibly storing
        // the new metadata. Optional conservative eviction is explicitly modeled.
        cache.remove(&step.path);
        if store {
            cache.insert(step.path.clone(), entry);
        }
        results.push(Observation{path:step.path.clone(),delivery:if matched { Delivery::Validated } else { Delivery::Downloaded },body:Some(body),etag:Some(etag),age:Some(age),origin_status:Some(if matched {304}else{200}),stored:store,explanation:if matched {"The origin confirmed the strong ETag:304 reused the body with updated freshness metadata.".into()}else{"The origin returned the current full representation:200. Storage eligibility is separate from whether this request received its body.".into()}});
    }
    Ok(results)
}

#[cfg(test)]
mod tests;
