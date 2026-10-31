//! Held-source admission for native HSRP changes. No answer replay or host work.
use super::*;
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    workspace::WorkspaceData,
};
#[derive(Serialize)]
#[serde(tag = "kind")]
enum StateRef<'a> {
    Hsrp { simulation: &'a Session },
}
#[derive(Serialize)]
struct AttemptRef<'a> {
    scenario_version: u32,
    state: StateRef<'a>,
    step: usize,
    challenge: bool,
    hints: usize,
    transcript: &'a [String],
    visited: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv4: Option<&'a crate::ipv4::Lesson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace: Option<&'a WorkspaceData>,
}
fn projected<'a>(a: &'a Attempt, s: &'a Session) -> AttemptRef<'a> {
    AttemptRef {
        scenario_version: a.scenario_version,
        state: StateRef::Hsrp { simulation: s },
        step: a.step,
        challenge: a.challenge,
        hints: a.hints,
        transcript: &a.transcript,
        visited: a.visited,
        ipv4: a.ipv4.as_ref(),
        workspace: a.workspace.as_ref(),
    }
}
pub(super) fn fresh_context(s: &Scenario, session: &Session) -> bool {
    let workspace = WorkspaceData::new();
    let candidate = AttemptRef {
        scenario_version: s.version,
        state: StateRef::Hsrp {
            simulation: session,
        },
        step: 0,
        challenge: false,
        hints: 0,
        transcript: &[],
        visited: u64::MAX,
        ipv4: None,
        workspace: Some(&workspace),
    };
    bounded::within(&(s, &candidate), MAX_CONTEXT)
}
/// Native request only: actual candidate copies are made on a gesture, never on
/// an ordinary render. Common metadata and inactive practice stay in admission.
pub enum Change {
    View(View),
    SelectExample(usize),
    InsertExample(usize),
    ResetExample,
    Draft { draft: Draft, practice: bool },
    Apply { practice: bool },
    Check { practice: bool },
}
pub fn change(a: &mut Attempt, source: &Scenario, request: Change) -> Result<(), String> {
    context(a, source)?;
    let Goal::Hsrp { definition } = &source.goal else {
        unreachable!()
    };
    let State::Hsrp { simulation } = &a.state else {
        unreachable!()
    };
    let mut candidate = simulation.as_ref().clone();
    match request {
        Change::View(v) => candidate.view = v,
        Change::SelectExample(index) => {
            if index >= definition.lesson.worked.len() {
                return Err("Unknown worked example".into());
            }
            candidate.selected_example = index;
        }
        Change::InsertExample(index) => candidate.insert_example(definition, index)?,
        Change::ResetExample => candidate.reset_example(definition)?,
        Change::Draft { draft, practice } => candidate.set_draft(definition, draft, practice)?,
        Change::Apply { practice } => candidate.apply_draft(definition, practice)?,
        Change::Check { practice } => {
            if practice {
                candidate.example_checked = true
            } else {
                candidate.checked = true
            }
        }
    }
    definition.bound(&candidate)?;
    if !bounded::within(&(source, &projected(a, &candidate)), MAX_CONTEXT) {
        return Err("That change cannot fit this saved workspace. The entire prior configuration, fields and check results were retained; reduce other saved text before retrying.".into());
    }
    let State::Hsrp { simulation } = &mut a.state else {
        unreachable!()
    };
    **simulation = candidate;
    Ok(())
}
pub fn request_hint(a: &mut Attempt, source: &Scenario) -> Result<(), String> {
    context(a, source)?;
    let hints = a.hints.checked_add(1).ok_or("Hint count exhausted")?;
    let State::Hsrp { simulation } = &a.state else {
        unreachable!()
    };
    let mut candidate = projected(a, simulation);
    candidate.hints = hints;
    if hints > source.hints.len() || !bounded::within(&(source, &candidate), MAX_CONTEXT) {
        return Err(
            "That hint cannot fit this saved workspace; prior hint count and work retained".into(),
        );
    }
    a.hints = hints;
    Ok(())
}
pub fn set_challenge(a: &mut Attempt, source: &Scenario, challenge: bool) -> Result<(), String> {
    context(a, source)?;
    let State::Hsrp { simulation } = &a.state else {
        unreachable!()
    };
    let mut candidate = projected(a, simulation);
    candidate.challenge = challenge;
    if !bounded::within(&(source, &candidate), MAX_CONTEXT) {
        return Err(
            "That mode change cannot fit this saved workspace; prior mode and work retained".into(),
        );
    }
    a.challenge = challenge;
    Ok(())
}
pub fn admit_launch(a: Option<&Attempt>, source: &Scenario, visited: u64) -> Result<(), String> {
    super::source(source)?;
    if let Some(a) = a {
        context(a, source)?;
        let State::Hsrp { simulation } = &a.state else {
            unreachable!()
        };
        let blank = WorkspaceData::new();
        let mut candidate = projected(a, simulation);
        candidate.visited = visited;
        candidate.workspace = a.workspace.as_ref().or(Some(&blank));
        if !bounded::within(&(source, &candidate), MAX_CONTEXT) {
            return Err(
                "Reopening cannot fit this saved workspace; prior recency and work retained".into(),
            );
        }
    }
    Ok(())
}
pub fn prepare(a: &mut Attempt, source: &Scenario) -> Result<(), String> {
    context(a, source)?;
    if a.workspace.is_none() {
        let blank = WorkspaceData::new();
        let State::Hsrp { simulation } = &a.state else {
            unreachable!()
        };
        let mut candidate = projected(a, simulation);
        candidate.workspace = Some(&blank);
        if !bounded::within(&(source, &candidate), MAX_CONTEXT) {
            return Err("Prepared workspace cannot fit; prior work retained".into());
        }
        a.workspace = Some(blank);
    }
    Ok(())
}
