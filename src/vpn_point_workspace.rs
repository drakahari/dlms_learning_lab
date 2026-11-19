//! Source-bound draft/replay state over the finite virtual peer model.
use crate::{
    script_teaching::View,
    vpn_point::{Class, Error},
    vpn_point_task::{Check, Probe, Task, World},
};
use serde::{Deserialize, Deserializer, Serialize};
type Result<T> = std::result::Result<T, String>;
#[derive(Deserialize)]
struct Profile(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
fn profiles<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<[String; 2], D::Error> {
    Ok(<[Profile; 2]>::deserialize(d)?.map(|s| s.0))
}
fn canonical(t: &Task) -> Result<String> {
    fn sort(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Array(a) => {
                for x in a {
                    sort(x)
                }
            }
            serde_json::Value::Object(m) => {
                let ordered: std::collections::BTreeMap<_, _> =
                    std::mem::take(m).into_iter().collect();
                for (k, mut v) in ordered {
                    sort(&mut v);
                    m.insert(k, v);
                }
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(t).map_err(|e| e.to_string())?;
    sort(&mut v);
    serde_json::to_string(&v).map_err(|e| e.to_string())
}
fn bounded<T: Serialize>(v: &T, max: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|s| s.len() <= max)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub from: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub destination: String,
}
impl Query {
    fn valid(&self) -> bool {
        self.from < 2
            && [&self.source, &self.destination]
                .iter()
                .all(|s| s.len() <= 64 && !s.contains('\0'))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub kind: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub message: String,
    #[serde(deserialize_with = "profiles")]
    pub profiles: [String; 2],
    pub query: Option<Query>,
}
fn notice(e: Error, w: &World, q: Option<Query>) -> Notice {
    let kind = match e.class {
        Class::Syntax => "Syntax error",
        Class::Unsupported => "Unsupported setting",
        Class::Configuration => "Configuration requirement",
        Class::Limit => "Simulation limit",
    };
    let text = format!(
        "{}{}",
        e.line.map_or(String::new(), |n| format!("Line {n}: ")),
        e.message
    );
    let message = if text.len() > 900 {
        format!("{}…", text.chars().take(850).collect::<String>())
    } else {
        text
    };
    Notice {
        kind: kind.into(),
        message,
        profiles: w.saved.clone(),
        query: q,
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Save {
        peer: usize,
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
    Apply,
    Probe {
        query: Query,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Edit {
        peer: usize,
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
    Save {
        peer: usize,
    },
    Apply,
    Probe,
    Query {
        query: Query,
    },
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub source_task: String,
    pub world: World,
    #[serde(deserialize_with = "profiles")]
    pub editors: [String; 2],
    pub query: Query,
    pub notice: Option<Notice>,
    pub last_probe: Option<Probe>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,64>")]
    pub history: Vec<Event>,
}
impl Scope {
    fn seed(t: &Task) -> Result<Self> {
        let c = t
            .cases
            .first()
            .ok_or("Supply a path case before opening this workspace.")?;
        Ok(Self {
            source_task: canonical(t)?,
            world: t.initial.clone(),
            editors: t.initial.saved.clone(),
            query: Query {
                from: c.from,
                source: c.source.to_string(),
                destination: c.destination.to_string(),
            },
            notice: None,
            last_probe: None,
            checked: false,
            history: vec![],
        })
    }
    pub fn new(t: &Task) -> Result<Self> {
        t.validate().map_err(|e| e.message)?;
        Self::seed(t)
    }
    fn event(&mut self, e: &Event) -> Result<()> {
        match e {
            Event::Save { peer, text } => {
                self.world.save(*peer, text).map_err(|e| e.message)?;
                self.last_probe = None;
                self.notice = None;
            }
            Event::Apply => {
                self.last_probe = None;
                self.notice = match self.world.apply() {
                    Ok(()) => None,
                    Err(e) => Some(notice(e, &self.world, None)),
                };
            }
            Event::Probe { query } => {
                if !query.valid() {
                    return Err("Choose a supplied source side and bounded address fields.".into());
                }
                let parsed = (|| {
                    if query.source.trim().is_empty() || query.destination.trim().is_empty() {
                        return Err(Error {
                            class: Class::Syntax,
                            line: None,
                            message:
                                "Enter both source and destination IPv4 addresses before testing."
                                    .into(),
                        });
                    }
                    let source = query.source.trim().parse().map_err(|_| Error {
                        class: Class::Syntax,
                        line: None,
                        message: "Source must be a dotted-decimal IPv4 host address.".into(),
                    })?;
                    let destination = query.destination.trim().parse().map_err(|_| Error {
                        class: Class::Syntax,
                        line: None,
                        message: "Destination must be a dotted-decimal IPv4 address.".into(),
                    })?;
                    self.world.probe(query.from, source, destination)
                })();
                match parsed {
                    Ok(probe) => {
                        self.last_probe = Some(probe);
                        self.notice = None;
                    }
                    Err(e) => {
                        self.last_probe = None;
                        self.notice = Some(notice(e, &self.world, Some(query.clone())));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        t.validate().map_err(|e| e.message)?;
        if self.source_task != canonical(t)? {
            return Err("Saved VPN work belongs to different requirements. Retain it for explicit migration rather than crediting the changed task.".into());
        }
        if !bounded(self, 768 * 1024)
            || self.history.len() > 64
            || !bounded(&self.history, 256 * 1024)
            || !self.query.valid()
            || self
                .editors
                .iter()
                .any(|s| s.len() > 8192 || s.contains('\0'))
        {
            return Err(
                "The saved VPN workspace exceeds its documented bounds; prior work is retained."
                    .into(),
            );
        }
        let mut replay = Self::seed(t)?;
        for e in &self.history {
            replay.event(e)?;
        }
        if self.world != replay.world
            || self.notice != replay.notice
            || self.last_probe != replay.last_probe
        {
            return Err("Saved peer state or reports do not match this attempt's actual virtual actions; prior work is retained.".into());
        }
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(detail) = self.bound(t) {
            return vec![Check {
                label: "Saved workspace integrity".into(),
                passed: false,
                detail,
            }];
        }
        let mut rows=vec![Check{label:"Current saved work".into(),passed:self.checked&&self.editors==self.world.saved,detail:"Save each current editor, apply the profiles, then check this assignment. Practice never earns assignment credit.".into()}];
        rows.extend(t.checks(&self.world));
        rows
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checks(t).iter().all(|c| c.passed)
    }
    pub fn visible_notice(&self) -> Option<&Notice> {
        self.notice.as_ref().filter(|n| {
            self.editors == self.world.saved
                && n.profiles == self.world.saved
                && n.query.as_ref().is_none_or(|q| q == &self.query)
        })
    }
    pub fn visible_probe(&self) -> Option<&Probe> {
        self.last_probe.as_ref().filter(|p| {
            self.editors == self.world.saved
                && self.world.applied_source.as_ref() == Some(&self.world.saved)
                && p.from == self.query.from
                && self.query.source.trim().parse() == Ok(p.source)
                && self.query.destination.trim().parse() == Ok(p.destination)
        })
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        let event = match a {
            Action::Edit { peer, text } => {
                if *peer > 1 || text.len() > 8192 || text.contains('\0') {
                    return Err(
                        "Choose a supplied editor and keep its draft within 8 KiB without NUL."
                            .into(),
                    );
                }
                if next.editors[*peer] != *text {
                    next.editors[*peer] = text.clone();
                    next.checked = false;
                }
                None
            }
            Action::Query { query } => {
                if !query.valid() {
                    return Err("Choose a supplied side and keep address drafts bounded.".into());
                }
                next.query = query.clone();
                None
            }
            Action::Check => {
                next.checked = true;
                None
            }
            Action::Save { peer } => {
                if *peer > 1 {
                    return Err("Choose one of the two supplied peer editors.".into());
                }
                Some(Event::Save {
                    peer: *peer,
                    text: next.editors[*peer].clone(),
                })
            }
            Action::Apply => Some(Event::Apply),
            Action::Probe => Some(Event::Probe {
                query: next.query.clone(),
            }),
        };
        if let Some(event) = event {
            if next.history.len() == 64 {
                return Err("This selected workspace reached 64 recorded actions. Back it up before resetting only this workspace.".into());
            }
            next.event(&event)?;
            if !matches!(event, Event::Probe { .. }) {
                next.checked = false;
            }
            next.history.push(event);
        }
        next.bound(t)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,32>")]
    pub worked: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !self.source_id.starts_with("lpic2-vpn-")
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.worked.is_empty()
            || self.worked.len() > 32
            || !bounded(self, 512 * 1024)
        {
            return Err(
                "Use a bounded version 1 source-owned VPN lesson and real worked actions.".into(),
            );
        }
        self.assignment.validate().map_err(|e| e.message)?;
        self.example.validate().map_err(|e| e.message)?;
        if self.assignment == self.example {
            return Err("Give the practice example different supplied inputs from the independent assignment.".into());
        }
        let mut practice = Scope::new(&self.example)?;
        for a in &self.worked {
            practice.apply(&self.example, a)?;
        }
        if !practice.solved(&self.example) {
            return Err("The taught example must reach its actual complete outcome.".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        if s.version != 1 || s.source_id != self.source_id || !bounded(s, 1536 * 1024) {
            return Err("Saved VPN work belongs to another lesson or exceeds the session bounds; prior work is retained.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.example.bound(&self.example)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            example: Scope::new(&d.example)?,
        })
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.example.apply(&d.example, a)?;
        } else {
            next.assignment.apply(&d.assignment, a)?;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.example = Scope::new(&d.example)?;
        } else {
            next.assignment = Scope::new(&d.assignment)?;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1
            || !self.source_id.starts_with("lpic2-vpn-")
            || self.source_id.len() > 64
            || !bounded(self, 1536 * 1024)
        {
            return Err("Unsupported or oversized saved VPN session; prior work retained.".into());
        }
        for s in [&self.assignment, &self.example] {
            let t: Task = serde_json::from_str(&s.source_task)
                .map_err(|e| format!("Malformed saved source requirements: {e}"))?;
            s.bound(&t)?;
        }
        Ok(())
    }
}
