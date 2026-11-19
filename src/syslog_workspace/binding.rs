//! Whole held-source/Attempt admission for every native policy workspace change.
//! No reference replay, host activity or historical-family budget dependencies.
use super::*;
use crate::{engine::State, pack::Scenario, progress::Attempt, workspace::WorkspaceData};

#[derive(Serialize)]
#[serde(tag = "kind")]
enum StateRef<'a> {
    Syslog { simulation: &'a Session },
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
fn projected<'a>(a: &'a Attempt, session: &'a Session, hints: usize) -> AttemptRef<'a> {
    AttemptRef {
        scenario_version: a.scenario_version,
        state: StateRef::Syslog {
            simulation: session,
        },
        step: a.step,
        challenge: a.challenge,
        hints,
        transcript: &a.transcript,
        visited: a.visited,
        ipv4: a.ipv4.as_ref(),
        workspace: a.workspace.as_ref(),
    }
}
/// A source must fit its widest fresh recency plus the generic prepared workspace.
/// This is borrowed serialization before Attempt creation, not an answer replay.
pub(super) fn fresh_context(s: &Scenario, session: &Session) -> bool {
    let workspace = WorkspaceData::new();
    let candidate = AttemptRef {
        scenario_version: s.version,
        state: StateRef::Syslog {
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
/// Publish a complete bounded candidate only after checking the actual held source,
/// common metadata, inactive example, drafts and prospective hint count together.
/// The caller may stage a whole render's native edits locally; none is assignment
/// state until this function succeeds. A failure leaves the entire Attempt exact.
pub fn publish(a: &mut Attempt, s: &Scenario, candidate: &Session, hints: usize) -> Result<()> {
    context(a, s)?;
    let crate::engine::Goal::Syslog { definition } = &s.goal else {
        unreachable!()
    };
    definition.bound(candidate)?;
    if hints > s.hints.len() || !bounded::within(&(s, &projected(a, candidate, hints)), MAX_CONTEXT)
    {
        return Err("That change would exceed this saved workspace's size limit. The complete prior work, check results and text selection were retained; reduce other saved text before retrying.".into());
    }
    let State::Syslog { simulation } = &mut a.state else {
        unreachable!()
    };
    if simulation.as_ref() != candidate {
        **simulation = candidate.clone();
    }
    a.hints = hints;
    Ok(())
}
pub fn request_hint(a: &mut Attempt, s: &Scenario) -> Result<()> {
    context(a, s)?;
    let hints = a.hints.checked_add(1).ok_or("Hint count exhausted")?;
    let State::Syslog { simulation } = &a.state else {
        unreachable!()
    };
    if hints > s.hints.len()
        || !bounded::within(&(s, &projected(a, simulation, hints)), MAX_CONTEXT)
    {
        return Err(
            "The next hint cannot fit this saved workspace; previous work and hint count retained"
                .into(),
        );
    }
    a.hints = hints;
    Ok(())
}
pub fn set_challenge(a: &mut Attempt, s: &Scenario, challenge: bool) -> Result<()> {
    context(a, s)?;
    let State::Syslog { simulation } = &a.state else {
        unreachable!()
    };
    let mut candidate = projected(a, simulation, a.hints);
    candidate.challenge = challenge;
    if !bounded::within(&(s, &candidate), MAX_CONTEXT) {
        return Err(
            "That mode change cannot fit this saved workspace; previous work and mode retained"
                .into(),
        );
    }
    a.challenge = challenge;
    Ok(())
}
/// Preflight the exact next recency and prepared workspace before the app changes
/// its global clock or selected lab. Source admission also covers the fresh case.
pub fn admit_launch(a: Option<&Attempt>, s: &Scenario, visited: u64) -> Result<()> {
    source(s)?;
    if let Some(a) = a {
        context(a, s)?;
        let State::Syslog { simulation } = &a.state else {
            unreachable!()
        };
        let blank = WorkspaceData::new();
        let mut candidate = projected(a, simulation, a.hints);
        candidate.visited = visited;
        candidate.workspace = a.workspace.as_ref().or(Some(&blank));
        if !bounded::within(&(s, &candidate), MAX_CONTEXT) {
            return Err(
                "Reopening cannot fit this saved workspace; prior work and recency retained".into(),
            );
        }
    }
    Ok(())
}
pub fn prepare(a: &mut Attempt, s: &Scenario) -> Result<()> {
    context(a, s)?;
    if a.workspace.is_none() {
        let blank = WorkspaceData::new();
        let State::Syslog { simulation } = &a.state else {
            unreachable!()
        };
        let mut candidate = projected(a, simulation, a.hints);
        candidate.workspace = Some(&blank);
        if !bounded::within(&(s, &candidate), MAX_CONTEXT) {
            return Err("Prepared workspace cannot fit; original work retained".into());
        }
        a.workspace = Some(blank);
    }
    Ok(())
}
