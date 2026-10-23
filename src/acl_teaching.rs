//! Separate ACL introduction/example data. No assignment access or completion cache.
use crate::{
    acl_editor as editor,
    acl_routing::Network,
    acl_workspace::{self, Draft},
    script_teaching::{Concept, View},
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};
fn expected<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, bool>, D::Error> {
    struct Expected;
    impl<'de> Visitor<'de> for Expected {
        type Value = BTreeMap<String, bool>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16 unique expected ACL case results")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(id) = a.next_key::<String>()? {
                if out.len() >= editor::MAX_CASES
                    || !acl_workspace::name(&id)
                    || out.contains_key(&id)
                {
                    return Err(A::Error::custom(
                        "Invalid/duplicate ACL outcome or case limit",
                    ));
                }
                out.insert(id, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Expected)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TryChange {
    pub description: String,
    pub documents: editor::Drafts,
    #[serde(deserialize_with = "expected")]
    pub expected: BTreeMap<String, bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: Network,
    pub definition: editor::Definition,
    pub example_documents: editor::Drafts,
    pub try_change: TryChange,
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        acl_workspace::valid_network(&self.fixture)?;
        self.definition.validate().map_err(|e| e.to_string())?;
        if self.fixture != self.definition.initial {
            return Err("ACL example initial state must exactly match its fixed definition".into());
        }
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self
                .concepts
                .iter()
                .any(|c| !prose(&c.title, 100) || !prose(&c.text, 1200))
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change.description,
            ]
            .iter()
            .any(|s| !prose(s, 1600))
            || self.try_change.expected.len() != self.definition.cases.len()
            || self.try_change.expected.len() > editor::MAX_CASES
            || !self
                .definition
                .cases
                .iter()
                .all(|c| self.try_change.expected.contains_key(&c.id))
        {
            return Err("Invalid ACL prerequisite/concept/example/try-result metadata".into());
        }
        let worked = acl_workspace::apply_documents(&self.fixture, &self.example_documents)?;
        if !editor::grade(&self.definition, &worked, &editor::Drafts::default())
            .map_err(|e| e.to_string())?
            .complete
        {
            return Err("ACL worked example does not meet every stated flow outcome".into());
        }
        let changed = acl_workspace::apply_documents(&worked, &self.try_change.documents)?;
        if changed == worked {
            return Err(
                "ACL try change must change applied policy, not merely redisplay it".into(),
            );
        }
        let report = editor::grade(&self.definition, &changed, &editor::Drafts::default())
            .map_err(|e| e.to_string())?;
        if report
            .cases
            .iter()
            .any(|c| self.try_change.expected.get(&c.id) != Some(&c.passed))
        {
            return Err("ACL try-change outcomes differ from the declared traffic results".into());
        }
        Ok(())
    }
    pub fn distinct_from(&self, assignment: &Network) -> bool {
        self.fixture.topology != assignment.topology
            && self.fixture.host_endpoints != assignment.host_endpoints
            && !assignment
                .topology
                .devices
                .values()
                .flat_map(|d| d.interfaces.values())
                .filter_map(|i| i.address.as_ref())
                .any(|a| {
                    self.fixture
                        .topology
                        .devices
                        .values()
                        .flat_map(|d| d.interfaces.values())
                        .filter_map(|i| i.address.as_ref())
                        .any(|b| a.address == b.address)
                })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub network: Network,
    pub draft: Draft,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            network: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture)?,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && acl_workspace::valid_network(&self.network).is_ok()
            && self.draft.same_shape(&self.network)
    }
    pub fn same_problem(&self, lesson: &Lesson) -> bool {
        self.valid() && acl_workspace::same_assignment(&self.network, &lesson.fixture)
    }
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.same_problem(lesson) {
            return Err("ACL practice context differs from this example".into());
        }
        let worked = acl_workspace::apply_documents(&lesson.fixture, &lesson.example_documents)?;
        let mut draft = Draft::new(&worked)?;
        if self
            .draft
            .selected_router
            .as_ref()
            .is_some_and(|id| draft.documents.routers.contains_key(id))
        {
            draft.selected_router = self.draft.selected_router.clone();
        }
        draft.notice=Some("Worked documents filled only in this ungraded example. Apply/check explicitly; your assignment and applied example have not changed.".into());
        self.draft = draft;
        self.checked = false;
        Ok(())
    }
    pub fn apply_check(&mut self, lesson: &Lesson) -> Result<Vec<(bool, String)>, String> {
        if !self.same_problem(lesson) {
            return Err("ACL practice context differs from its lesson".into());
        }
        self.checked = true;
        self.draft.apply(&mut self.network)?;
        Ok(self.checks(lesson))
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![];
        }
        if !self.same_problem(lesson) {
            return vec![(
                false,
                "ACL practice context does not match its authored fixture".into(),
            )];
        }
        acl_workspace::checks(&self.network, &lesson.definition, &self.draft.documents)
    }
}
pub fn prepare(
    attempt: &mut crate::progress::Attempt,
    lesson: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    if !matches!(attempt.state, crate::engine::State::Ipv4Acl { .. }) {
        return Err("ACL teaching metadata belongs only to an ACL assignment".into());
    }
    let w = attempt
        .workspace
        .as_mut()
        .ok_or("Prepare the workspace before the ACL lesson")?;
    if w.acl_learning.is_none() {
        let mut learning = Learning::new(lesson)?;
        if !new_attempt {
            learning.view = View::Assignment;
        }
        w.acl_learning = Some(learning);
    }
    Ok(())
}
