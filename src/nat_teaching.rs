//! Bounded, independent Basic NAT practice. No packet delivery or saved grade.
use crate::{nat as core, nat_workspace as work, script_teaching::View};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{fmt, marker::PhantomData};
pub const MAX_ACTIONS: usize = 16;
pub const MAX_LESSON: usize = 1024 * 1024;
pub const MAX_LEARNING: usize = 1024 * 1024;
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct T<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for T<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "text up to {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("NAT teaching text bound"))
            } else {
                Ok(s.to_owned())
            }
        }
    }
    d.deserialize_str(T::<N>)
}
struct Item<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Item<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("NAT teaching collection bound"));
        }
        T::deserialize(d)
    }
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "up to {N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Item {
                allowed: out.len() < N,
                marker: PhantomData,
            })? {
                out.push(v)
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(PhantomData))
}
#[derive(Deserialize)]
struct Id(#[serde(deserialize_with = "text::<_,64>")] String);
fn prerequisites<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(list::<D, Id, 16>(d)?.into_iter().map(|s| s.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "text::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "text::<_,2400>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionStep {
    #[serde(deserialize_with = "text::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "text::<_,800>")]
    pub explanation: String,
    pub action: core::Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    #[serde(deserialize_with = "text::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "list::<_,core::Action,16>")]
    pub actions: Vec<core::Action>,
    #[serde(deserialize_with = "list::<_,bool,37>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "prerequisites")]
    pub prerequisites: Vec<String>,
    #[serde(deserialize_with = "text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "list::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "text::<_,1600>")]
    pub example_description: String,
    pub fixture: core::State,
    pub definition: work::Definition,
    #[serde(deserialize_with = "list::<_,ActionStep,16>")]
    pub example_actions: Vec<ActionStep>,
    pub try_change: TryChange,
    #[serde(deserialize_with = "list::<_,core::Action,16>")]
    pub repair: Vec<core::Action>,
}
fn prose(s: &str, n: usize) -> bool {
    s.len() <= n && !s.contains('\0') && !s.trim().is_empty()
}
fn actions<'a>(
    mut a: impl Iterator<Item = &'a core::Action>,
    n: usize,
    i: &core::Inventory,
) -> bool {
    (1..=MAX_ACTIONS).contains(&n)
        && a.all(|a| a.validate_for(i).is_ok() && work::within(a, core::MAX_ACTION_JSON))
}
impl Lesson {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LESSON {
            return Err("NAT lesson raw bound".into());
        }
        let v: Self = serde_json::from_slice(raw).map_err(|_| "Invalid bounded NAT lesson")?;
        v.validate()?;
        Ok(v)
    }
    /// Borrowed construction limits; no serialization, copies or fresh trials.
    pub fn shape(&self) -> bool {
        self.definition.requirements.len() <= 32
            && self.definition.trials.len() <= 4
            && self.prerequisites.len() <= 16
            && self.prerequisites.iter().enumerate().all(|(n, s)| {
                prose(s, 64)
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
                    && !self.prerequisites[..n].contains(s)
            })
            && prose(&self.prerequisite_note, 1600)
            && prose(&self.example_description, 1600)
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 160) && prose(&c.text, 2400))
            && (1..=16).contains(&self.example_actions.len())
            && self
                .example_actions
                .iter()
                .all(|a| prose(&a.title, 160) && prose(&a.explanation, 800))
            && prose(&self.try_change.description, 1600)
            && (1..=16).contains(&self.try_change.actions.len())
            && (1..=16).contains(&self.repair.len())
            && self.try_change.expected.len() <= 37
            && self.try_change.expected.len()
                == 1 + self.definition.requirements.len() + self.definition.trials.len()
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.shape() {
            return Err("Invalid bounded NAT introduction or action lists".into());
        }
        if !work::admitted(&self.fixture) || self.definition.initial != self.fixture {
            return Err("NAT practice definition must pin its own initial fixture".into());
        }
        self.definition.validate()?;
        if !actions(
            self.example_actions.iter().map(|s| &s.action),
            self.example_actions.len(),
            self.fixture.inventory(),
        ) || !actions(
            self.try_change.actions.iter(),
            self.try_change.actions.len(),
            self.fixture.inventory(),
        ) || !actions(
            self.repair.iter(),
            self.repair.len(),
            self.fixture.inventory(),
        ) || !work::within(self, MAX_LESSON)
        {
            return Err("NAT teaching action/context/aggregate bound".into());
        }
        if self.definition.checks(&self.fixture).iter().all(|r| r.0) {
            return Err("Practice needs a substantive change".into());
        }
        let mut s = self.fixture.clone();
        for a in &self.example_actions {
            work::perform(&self.fixture, &mut s, &a.action)?;
        }
        if !self.definition.checks(&s).iter().all(|r| r.0) {
            return Err("NAT worked actions do not satisfy practice outcomes".into());
        }
        let worked = s.clone();
        for a in &self.try_change.actions {
            work::perform(&self.fixture, &mut s, a)?;
        }
        let mut expected = vec![true];
        expected.extend(self.definition.checks(&s).iter().map(|r| r.0));
        if s == worked || expected != self.try_change.expected || expected.iter().all(|v| *v) {
            return Err("NAT experiment must match its complete adverse vector".into());
        }
        for a in &self.repair {
            work::perform(&self.fixture, &mut s, a)?;
        }
        if !self.definition.checks(&s).iter().all(|r| r.0) {
            return Err("NAT practice repair does not restore all outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, s: &core::State) -> bool {
        let a = self.fixture.inventory();
        let b = s.inventory();
        a.fixture != b.fixture
            && a.inside
                .keys()
                .chain(a.outside.keys())
                .all(|k| !b.inside.contains_key(k) && !b.outside.contains_key(k))
            && a.inside
                .values()
                .chain(a.outside.values())
                .chain(a.globals.iter())
                .all(|v| {
                    !b.inside
                        .values()
                        .chain(b.outside.values())
                        .chain(b.globals.iter())
                        .any(|w| v == w)
                })
    }
    pub fn validate_for(&self, s: &core::State) -> Result<(), String> {
        self.validate()?;
        if !work::admitted(s) || !self.separate_from(s) {
            return Err("NAT practice requires different supplied identities and addresses".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Learning {
    pub version: u8,
    pub fixture: core::State,
    pub simulation: core::State,
    pub draft: work::Draft,
    pub view: View,
    pub selected_action: usize,
    pub checked: bool,
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u8,
            fixture: core::State,
            simulation: core::State,
            draft: work::Draft,
            view: View,
            selected_action: usize,
            checked: bool,
            notice: Option<Notice>,
        }
        #[derive(Deserialize)]
        struct Notice(#[serde(deserialize_with = "text::<_,512>")] String);
        let r = Raw::deserialize(d)?;
        let v = Self {
            version: r.version,
            fixture: r.fixture,
            simulation: r.simulation,
            draft: r.draft,
            view: r.view,
            selected_action: r.selected_action,
            checked: r.checked,
            notice: r.notice.map(|s| s.0),
        };
        if !v.valid() {
            return Err(D::Error::custom("Invalid bounded NAT practice"));
        }
        Ok(v)
    }
}
impl Learning {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LEARNING {
            return Err("NAT practice raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NAT practice".into())
    }
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate()?;
        let draft = work::Draft::new(&l.fixture)?;
        // A borrowed proposed aggregate is admitted BEFORE cloning either state.
        #[derive(Serialize)]
        struct Proposed<'a> {
            version: u8,
            fixture: &'a core::State,
            simulation: &'a core::State,
            draft: &'a work::Draft,
            view: View,
            selected_action: usize,
            checked: bool,
            notice: Option<&'a str>,
        }
        if !work::within(
            &Proposed {
                version: 1,
                fixture: &l.fixture,
                simulation: &l.fixture,
                draft: &draft,
                view: View::Introduction,
                selected_action: 0,
                checked: false,
                notice: None,
            },
            MAX_LEARNING,
        ) {
            return Err("NAT practice aggregate bound".into());
        }
        Ok(Self {
            version: 1,
            fixture: l.fixture.clone(),
            simulation: l.fixture.clone(),
            draft,
            view: View::Introduction,
            selected_action: 0,
            checked: false,
            notice: None,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.selected_action < 16
            && self
                .notice
                .as_ref()
                .is_none_or(|s| s.len() <= 512 && !s.contains('\0'))
            && work::same_context(&self.fixture, &self.simulation)
            && self.draft.valid_for(&self.simulation)
            && work::within(self, MAX_LEARNING)
    }
    pub fn separate_from(&self, s: &core::State) -> bool {
        let a = self.fixture.inventory();
        let b = s.inventory();
        a.fixture != b.fixture
            && a.inside
                .keys()
                .chain(a.outside.keys())
                .all(|k| !b.inside.contains_key(k) && !b.outside.contains_key(k))
            && a.inside
                .values()
                .chain(a.outside.values())
                .chain(a.globals.iter())
                .all(|v| {
                    !b.inside
                        .values()
                        .chain(b.outside.values())
                        .chain(b.globals.iter())
                        .any(|w| w == v)
                })
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid() && self.fixture == l.fixture && self.selected_action < l.example_actions.len()
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "NAT practice differs from its authored example".into(),
            )];
        }
        let applied = self.draft.matches(&self.simulation);
        let mut rows = vec![(
            applied,
            if applied {
                "Practice policy edits match the applied policy"
            } else {
                "Apply or repair the pending practice policy before checking the example"
            }
            .into(),
        )];
        rows.extend(l.definition.checks(&self.simulation));
        rows
    }
    pub fn fill_selected(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign NAT practice".into());
        }
        l.validate()?;
        let a = &l.example_actions[self.selected_action].action;
        match a {
            core::Action::Configure { configuration } => {
                let mut d = work::Draft::new(&self.simulation)?;
                d.statics = configuration
                    .statics
                    .iter()
                    .map(|(i, g)| work::StaticRow {
                        inside: i.clone(),
                        global: g.to_string(),
                    })
                    .collect();
                d.dynamic_enabled = configuration.dynamic.is_some();
                if let Some(r) = configuration.dynamic {
                    d.network = r.source.network.to_string();
                    d.prefix = r.source.length.to_string();
                    d.first = r.pool.first.to_string();
                    d.last = r.pool.last.to_string();
                }
                self.draft = d;
            }
            core::Action::Outbound { inside, peer } => {
                self.draft.inside = Some(inside.clone());
                self.draft.peer = Some(peer.clone())
            }
            core::Action::Inbound { peer, global } => {
                self.draft.peer = Some(peer.clone());
                self.draft.inbound_global = global.to_string()
            }
            core::Action::ClearDynamic {} => {}
        }
        self.checked = false;
        self.notice=Some("Practice fields inserted only. Use the matching Apply or probe control to perform this action.".into());
        Ok(())
    }
}
/// Initialize only absent metadata after all checks. Never replace a retained draft.
pub fn prepare(
    a: &mut crate::progress::Attempt,
    l: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    crate::workspace::reject_nfs_metadata(a)?;
    let crate::engine::State::BasicNat { simulation } = &a.state else {
        return Err("NAT practice on another activity".into());
    };
    crate::progress::validate_nat_metadata(a)?;
    l.validate_for(simulation)?;
    if a.workspace.as_ref().is_some_and(|w| {
        !w.valid()
            || w.nat_draft
                .as_ref()
                .is_some_and(|d| !d.valid_for(simulation))
            || w.nat_learning.as_ref().is_some_and(|v| !v.valid_for(l))
    }) {
        return Err("Foreign NAT practice/editor retained without changes".into());
    }
    let mut fresh = if a
        .workspace
        .as_ref()
        .is_none_or(|w| w.nat_learning.is_none())
    {
        Some(Learning::new(l)?)
    } else {
        None
    };
    if !new_attempt && let Some(v) = &mut fresh {
        v.view = View::Assignment;
    }
    let d = if a.workspace.as_ref().is_none_or(|w| w.nat_draft.is_none()) {
        Some(work::Draft::new(simulation)?)
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if let Some(v) = fresh {
        w.nat_learning = Some(v)
    }
    if let Some(d) = d {
        w.nat_draft = Some(d)
    }
    Ok(())
}
