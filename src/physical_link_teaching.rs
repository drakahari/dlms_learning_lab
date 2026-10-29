//! Isolated native practice backend; no assignment mutation or completion credit.
use crate::{physical_link as core, physical_link_assignment as work, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_LESSON: usize = 393_216;
pub const MAX_LEARNING: usize = 1_048_576;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::short::<_,120>")]
    pub title: String,
    #[serde(deserialize_with = "work::short::<_,1200>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub version: u32,
    #[serde(deserialize_with = "work::short::<_,1600>")]
    pub introduction: String,
    #[serde(deserialize_with = "work::list::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    pub practice: work::Definition,
    pub reference: core::Configuration,
    pub try_change: core::Configuration,
    pub repair: core::Configuration,
    #[serde(deserialize_with = "work::short::<_,1200>")]
    pub experiment_description: String,
}
fn text(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.contains('\0')
}
impl Lesson {
    pub fn from_json(raw: &[u8], assignment: &work::Definition) -> Result<Self, String> {
        if raw.len() > MAX_LESSON {
            return Err("Physical lesson exceeds384KiB".into());
        }
        let lesson: Self =
            serde_json::from_slice(raw).map_err(|e| format!("Invalid physical lesson: {e}"))?;
        lesson.validate(assignment)?;
        Ok(lesson)
    }
    pub fn bounded(&self) -> bool {
        self.version == 1
            && text(&self.introduction, 1600)
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self
                .concepts
                .iter()
                .all(|c| text(&c.title, 120) && text(&c.text, 1200))
            && text(&self.experiment_description, 1200)
            && work::bytes(self, MAX_LESSON).is_some()
    }
    pub fn separate_from(&self, assignment: &work::Definition) -> bool {
        let a = assignment.initial.topology();
        let b = self.practice.initial.topology();
        fn devices(t: &core::Topology) -> BTreeSet<&str> {
            t.ports.values().map(|p| p.device.as_str()).collect()
        }
        a.ports.keys().all(|k| !b.ports.contains_key(k))
            && a.cables.keys().all(|k| !b.cables.contains_key(k))
            && a.links.keys().all(|k| !b.links.contains_key(k))
            && devices(a).is_disjoint(&devices(b))
    }
    pub fn validate(&self, assignment: &work::Definition) -> Result<(), String> {
        assignment.validate()?;
        if !self.bounded() || !self.separate_from(assignment) {
            return Err(
                "Practice must be bounded and use separate equipment/cable identities".into(),
            );
        }
        self.practice.validate_reference(&self.reference)?;
        self.practice.validate_reference(&self.repair)?;
        self.try_change
            .validate(self.practice.initial.topology())
            .map_err(|e| e.to_string())?;
        let mut changed = self.practice.initial.clone();
        changed
            .perform(&core::Action::Configure {
                configuration: self.try_change.clone(),
            })
            .map_err(|e| e.to_string())?;
        if self.practice.solved(&changed) {
            return Err("The suggested experiment must expose a genuine unmet outcome".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    context: Lesson,
    pub view: View,
    pub current: core::State,
    pub draft: work::Draft,
    pub checked: bool,
}
impl Learning {
    pub fn valid(&self) -> bool {
        self.valid_for(&self.context)
    }

    pub fn new(lesson: &Lesson, assignment: &work::Definition) -> Result<Self, String> {
        lesson.validate(assignment)?;
        // Borrowed aggregate size before the full source/state copies. Existing hard
        // caps allow at most one bounded practice/context/editor bundle, not scripts.
        let mut draft = work::Draft::from_state(&lesson.practice.initial)?;
        let link = &lesson.practice.requirements[0].link;
        draft.selected_link = link.clone();
        draft.selected_port = lesson.practice.initial.topology().links[link].a.clone();
        let n = work::bytes(lesson, MAX_LESSON)
            .and_then(|x| {
                work::bytes(&lesson.practice.initial, core::MAX_STATE)
                    .and_then(|y| x.checked_add(y))
            })
            .and_then(|n| work::bytes(&draft, work::MAX_DRAFT).and_then(|d| n.checked_add(d)))
            .and_then(|n| n.checked_add(256));
        if n.is_none_or(|n| n > MAX_LEARNING) {
            return Err("Separate practice exceeds its saved-work allowance".into());
        }
        Ok(Self {
            version: 1,
            context: lesson.clone(),
            view: View::Introduction,
            current: lesson.practice.initial.clone(),
            draft,
            checked: false,
        })
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.version == 1
            && &self.context == lesson
            && lesson.bounded()
            && lesson.practice.context(&self.current).is_ok()
            && self.draft.valid_shape(self.current.topology())
            && work::bytes(self, MAX_LEARNING).is_some()
    }
    fn admitted(&self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            Err("Saved practice differs from its supplied lesson; previous work retained".into())
        } else {
            Ok(())
        }
    }
    pub fn set_draft(&mut self, lesson: &Lesson, draft: &work::Draft) -> Result<(), String> {
        self.admitted(lesson)?;
        if !draft.valid_shape(self.current.topology()) {
            return Err("Practice draft belongs to different supplied equipment".into());
        }
        let checked = self.checked && self.draft.configuration == draft.configuration;
        let proposed = work::bytes(self, MAX_LEARNING)
            .and_then(|n| {
                work::bytes(&self.draft, work::MAX_DRAFT).and_then(|old| n.checked_sub(old))
            })
            .and_then(|n| work::bytes(draft, work::MAX_DRAFT).and_then(|new| n.checked_add(new)))
            .and_then(|n| n.checked_add(if self.checked && !checked { 1 } else { 0 }));
        if proposed.is_none_or(|n| n > MAX_LEARNING) {
            return Err(
                "Practice edit exceeds its saved-work allowance; earlier draft retained".into(),
            );
        }
        self.draft = draft.clone();
        self.checked = checked;
        Ok(())
    }
    /// Populate a draft only. Filling never applies changes, runs probes or grades.
    pub fn fill_worked_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        self.admitted(lesson)?;
        lesson
            .reference
            .validate(self.current.topology())
            .map_err(|e| e.to_string())?;
        let mut draft = work::Draft::from_state(&self.current)?;
        let link = &lesson.practice.requirements[0].link;
        draft.selected_link = link.clone();
        draft.selected_port = self.current.topology().links[link].a.clone();
        draft.configuration = lesson.reference.clone();
        self.set_draft(lesson, &draft)
    }
    pub fn apply(&mut self, lesson: &Lesson) -> Result<(), String> {
        self.admitted(lesson)?;
        self.draft
            .configuration
            .validate(self.current.topology())
            .map_err(|e| e.to_string())?;
        let mut next = self.clone();
        work::apply(&lesson.practice, &mut next.current, &next.draft)?;
        next.checked = false;
        if !next.valid_for(lesson) {
            return Err("Practice apply exceeds its complete saved-work allowance".into());
        }
        *self = next;
        Ok(())
    }
    pub fn probe(
        &mut self,
        lesson: &Lesson,
        link: &str,
        a_frames: u32,
        b_frames: u32,
    ) -> Result<core::ProbeReport, String> {
        self.admitted(lesson)?;
        if link.len() > 48
            || !self.current.topology().links.contains_key(link)
            || a_frames > core::MAX_PROBE
            || b_frames > core::MAX_PROBE
            || a_frames + b_frames == 0
        {
            return Err("Choose a supplied link and bounded nonempty sample traffic".into());
        }
        let mut next = self.clone();
        let report = next
            .current
            .perform(&core::Action::Probe {
                link: link.into(),
                a_frames,
                b_frames,
            })
            .map_err(|e| e.to_string())?
            .ok_or("The probe did not produce observations")?;
        next.checked = false;
        if !next.valid_for(lesson) {
            return Err("Practice probe exceeds its saved-work allowance".into());
        }
        *self = next;
        Ok(report)
    }
    pub fn checks(&self, lesson: &Lesson) -> work::Rows {
        if let Err(e) = self.admitted(lesson) {
            return vec![(false, e)];
        }
        work::current_checks(&lesson.practice, &self.current, &self.draft)
    }
    pub fn reset(&mut self, lesson: &Lesson, assignment: &work::Definition) -> Result<(), String> {
        let next = Self::new(lesson, assignment)?;
        *self = next;
        Ok(())
    }
}

/// Exact borrowed wire shape for new/resumed practice admission. These views
/// carry no authority and are never installed as Learning or completion credit.
#[derive(Serialize)]
pub(crate) struct LearningProjection<'a, T: Serialize + ?Sized, D: Serialize + ?Sized> {
    version: u32,
    context: &'a Lesson,
    view: View,
    current: &'a T,
    draft: &'a D,
    checked: bool,
}
pub(crate) fn fresh_size(lesson: &Lesson, view: View) -> Option<usize> {
    let draft = work::fresh_draft_for(
        &lesson.practice.initial,
        &lesson.practice.requirements[0].link,
    );
    work::bytes(
        &LearningProjection {
            version: 1,
            context: lesson,
            view,
            current: &lesson.practice.initial,
            draft: &draft,
            checked: false,
        },
        MAX_LEARNING,
    )
}
impl Learning {
    pub(crate) fn projection<'a, T: Serialize + ?Sized, D: Serialize + ?Sized>(
        &'a self,
        current: &'a T,
        draft: &'a D,
        view: View,
        checked: bool,
    ) -> LearningProjection<'a, T, D> {
        LearningProjection {
            version: self.version,
            context: &self.context,
            view,
            current,
            draft,
            checked,
        }
    }
    pub(crate) fn install_view(&mut self, view: View) {
        self.view = view;
    }
    pub(crate) fn install_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
}
