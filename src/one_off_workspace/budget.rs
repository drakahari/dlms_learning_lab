//! Complete-Attempt projection and isolated assignment/example publication.
//! No files/processes outside the virtual model; native caller ticks once/frame.
use super::*;
use crate::{engine::State, pack::Scenario, progress::Attempt, script_teaching::View};
use serde::{Serialize, ser::SerializeMap};
use std::borrow::Cow;
#[derive(Serialize)]
struct AssignmentRef<'a> {
    simulation: &'a core::State,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_run: Option<&'a RunProof>,
}
struct InitialDrafts<'a> {
    d: &'a Definition,
    a: &'a core::State,
}
impl Serialize for InitialDrafts<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.d.editable.len()))?;
        for p in &self.d.editable {
            map.serialize_entry(p, self.a.files.get(p).map_or("", String::as_str))?
        }
        map.end()
    }
}
struct Empty;
impl Serialize for Empty {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_map(Some(0))?.end()
    }
}
#[derive(Serialize)]
struct SessionRef<'a> {
    initial: &'a core::State,
    selected: &'a str,
    actor: &'a str,
    drafts: InitialDrafts<'a>,
    environment: Empty,
    command: &'a str,
    window_end: Cow<'a, str>,
    output: &'a str,
}
fn initial_session<'a>(d: &'a Definition, a: &'a core::State) -> SessionRef<'a> {
    SessionRef {
        initial: &d.initial,
        selected: &d.editable[0],
        actor: if d.initial.users.contains("alex") {
            "alex"
        } else {
            "root"
        },
        drafts: InitialDrafts { d, a },
        environment: Empty,
        command: "",
        window_end: Cow::Owned(a.clock.after(60).unwrap_or(a.clock).text()),
        output: "",
    }
}
pub(super) fn initial_session_size(d: &Definition, a: &core::State) -> Option<usize> {
    bounded::size(&initial_session(d, a), MAX_ATTEMPT)
}
#[derive(Serialize)]
struct LearningRef<'a> {
    version: u32,
    view: View,
    definition: &'a Definition,
    assignment: AssignmentRef<'a>,
    session: SessionRef<'a>,
    selected_example: usize,
}
pub(crate) fn initial_learning_size(l: &crate::one_off_teaching::Lesson) -> Option<usize> {
    bounded::size(
        &LearningRef {
            version: 1,
            view: View::Introduction,
            definition: &l.definition,
            assignment: AssignmentRef {
                simulation: &l.definition.initial,
                last_run: None,
            },
            session: initial_session(&l.definition, &l.definition.initial),
            selected_example: 0,
        },
        MAX_ATTEMPT,
    )
}
/// Checked borrowed projection for the separate native binary. Invalid lesson
/// shapes are refused before the internal initial-session projection indexes it.
pub fn fresh_learning_size(l: &crate::one_off_teaching::Lesson) -> Option<usize> {
    l.validate_context().ok()?;
    initial_learning_size(l)
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum StateRef<'a> {
    OneOff { simulation: &'a Assignment },
}
fn optional_size<T: Serialize>(v: Option<&T>) -> Option<Option<usize>> {
    match v {
        None => Some(None),
        Some(v) => Some(Some(bounded::size(v, MAX_ATTEMPT)?)),
    }
}
fn field(name: &str, value: Option<usize>) -> Option<usize> {
    value.map_or(Some(0), |n| n.checked_add(name.len() + 4))
}
pub fn projected(
    a: &Attempt,
    state: &Assignment,
    session: Option<usize>,
    learning: Option<usize>,
) -> Option<usize> {
    if !matches!(a.state, State::OneOff { .. }) {
        return None;
    }
    let old = bounded::size(a, MAX_ATTEMPT)?;
    let old_state = bounded::size(&a.state, MAX_ATTEMPT)?;
    let next_state = bounded::size(&StateRef::OneOff { simulation: state }, MAX_ATTEMPT)?;
    let blank = crate::workspace::WorkspaceData::new();
    let w = a.workspace.as_ref().unwrap_or(&blank);
    let old_w = bounded::size(w, MAX_ATTEMPT)?;
    let next_w = old_w
        .checked_sub(field(
            "one_off_session",
            optional_size(w.one_off_session.as_ref())?,
        )?)?
        .checked_sub(field(
            "one_off_learning",
            optional_size(w.one_off_learning.as_ref())?,
        )?)?
        .checked_add(field("one_off_session", session)?)?
        .checked_add(field("one_off_learning", learning)?)?;
    let next = old.checked_sub(old_state)?.checked_add(next_state)?;
    if a.workspace.is_some() {
        next.checked_sub(old_w)?.checked_add(next_w)
    } else {
        next.checked_add("workspace".len() + 4)?.checked_add(next_w)
    }
}
fn admit(v: Option<usize>) -> Result<(), String> {
    if v.is_some_and(|n| n <= MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("Saved one-off capacity reached; previous work retained and nothing ran".into())
    }
}
pub fn prepare(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    let d = context(a, s)?;
    let State::OneOff { simulation } = &a.state else {
        return Err("Not a one-off assignment".into());
    };
    let old = a.workspace.as_ref();
    let new_session = old.is_none_or(|w| w.one_off_session.is_none());
    let new_learning =
        s.one_off_teaching.is_some() && old.is_none_or(|w| w.one_off_learning.is_none());
    let session = if new_session {
        Some(
            bounded::size(&initial_session(d, &simulation.simulation), MAX_ATTEMPT)
                .ok_or("One-off session bound")?,
        )
    } else {
        optional_size(old.and_then(|w| w.one_off_session.as_ref()))
            .ok_or("Retained one-off session bound")?
    };
    let learning = if new_learning {
        let l = s.one_off_teaching.as_ref().unwrap();
        Some(
            bounded::size(
                &LearningRef {
                    version: 1,
                    view: View::Introduction,
                    definition: &l.definition,
                    assignment: AssignmentRef {
                        simulation: &l.definition.initial,
                        last_run: None,
                    },
                    session: initial_session(&l.definition, &l.definition.initial),
                    selected_example: 0,
                },
                MAX_ATTEMPT,
            )
            .ok_or("One-off example bound")?,
        )
    } else {
        optional_size(old.and_then(|w| w.one_off_learning.as_ref()))
            .ok_or("Retained one-off practice bound")?
    };
    admit(projected(a, simulation, session, learning))?;
    // Large copies begin only after the complete borrowed projection fits.
    let session = if new_session {
        Some(Session::from_state(d, simulation)?)
    } else {
        None
    };
    let learning = if new_learning {
        Some(crate::one_off_teaching::Learning::new(
            s.one_off_teaching.as_ref().unwrap(),
        )?)
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if let Some(v) = session {
        w.one_off_session = Some(v)
    }
    if let Some(v) = learning {
        w.one_off_learning = Some(v)
    }
    Ok(())
}
pub fn apply_attempt(
    a: &mut Attempt,
    s: &Scenario,
    action: &Action,
    practice: bool,
) -> Result<String, String> {
    let d = context(a, s)?;
    let whole = bounded::size(a, MAX_ATTEMPT).ok_or("One-off complete Attempt bound")?;
    let State::OneOff { simulation } = &mut a.state else {
        return Err("Not a one-off assignment".into());
    };
    let w = a.workspace.as_mut().ok_or("Prepare one-off workspace")?;
    let output = if practice {
        let l = w
            .one_off_learning
            .as_mut()
            .ok_or("Prepare the separate example")?;
        apply(
            &l.definition,
            &mut l.assignment,
            &mut l.session,
            action,
            whole,
        )?
    } else {
        apply(
            d,
            simulation,
            w.one_off_session.as_mut().ok_or("Prepare one-off editor")?,
            action,
            whole,
        )?
    };
    if !practice {
        w.checked = false
    }
    Ok(output)
}
pub fn set_view(a: &mut Attempt, s: &Scenario, view: View, selected: usize) -> Result<(), String> {
    context(a, s)?;
    let lesson = s
        .one_off_teaching
        .as_ref()
        .ok_or("No one-off introduction")?;
    if selected >= lesson.example_actions.len() {
        return Err("Choose an existing worked action".into());
    }
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.one_off_learning.as_ref())
        .ok_or("Prepare separate one-off practice")?;
    let old = bounded::size(a, MAX_ATTEMPT).ok_or("One-off complete Attempt bound")?;
    let projected = old
        .checked_sub(bounded::size(&l.view, MAX_ATTEMPT).unwrap())
        .and_then(|n| n.checked_sub(bounded::size(&l.selected_example, MAX_ATTEMPT)?))
        .and_then(|n| n.checked_add(bounded::size(&view, MAX_ATTEMPT)?))
        .and_then(|n| n.checked_add(bounded::size(&selected, MAX_ATTEMPT)?));
    admit(projected)?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .one_off_learning
        .as_mut()
        .unwrap();
    l.view = view;
    l.selected_example = selected;
    Ok(())
}
/// An async run owns its admitted source+Attempt snapshot and original core
/// continuation. Completion compares both whole contexts before moving only the
/// chosen assignment/example result. Stop never installs this saved snapshot.
pub struct PendingAttempt {
    source: Scenario,
    before: Attempt,
    practice: bool,
    run: PendingRun,
}
impl PendingAttempt {
    pub fn begin(a: &Attempt, s: &Scenario, end: Moment, practice: bool) -> Result<Self, String> {
        let d = context(a, s)?;
        if !bounded::within(&(s, a), MAX_CONTEXT) {
            return Err(
                "Whole one-off source/Attempt exceeds bounded async snapshot capacity".into(),
            );
        }
        let whole = bounded::size(a, MAX_ATTEMPT).ok_or("One-off complete Attempt bound")?;
        let w = a.workspace.as_ref().ok_or("Prepare one-off workspace")?;
        let State::OneOff { simulation } = &a.state else {
            return Err("Not a one-off assignment".into());
        };
        let run = if practice {
            let l = w
                .one_off_learning
                .as_ref()
                .ok_or("Prepare one-off example")?;
            PendingRun::begin(&l.definition, &l.assignment, &l.session, end, whole)?
        } else {
            PendingRun::begin(
                d,
                simulation,
                w.one_off_session
                    .as_ref()
                    .ok_or("Prepare one-off session")?,
                end,
                whole,
            )?
        };
        Ok(Self {
            source: s.clone(),
            before: a.clone(),
            practice,
            run,
        })
    }
    pub fn from_draft(a: &Attempt, s: &Scenario, practice: bool) -> Result<Self, String> {
        context(a, s)?;
        let w = a.workspace.as_ref().ok_or("Prepare one-off workspace")?;
        let session = if practice {
            &w.one_off_learning
                .as_ref()
                .ok_or("Prepare separate practice")?
                .session
        } else {
            w.one_off_session
                .as_ref()
                .ok_or("Prepare one-off session")?
        };
        let end = Moment::parse(&session.window_end)
            .map_err(|_| "Use YYYY-MM-DD HH:MM in the virtual UTC window field")?;
        Self::begin(a, s, end, practice)
    }
    pub fn tick(&mut self, budget: usize) {
        self.run.tick(budget.min(core::MAX_TICK));
    }
    pub fn done(&self) -> bool {
        self.run.done()
    }
    pub fn cancel(&mut self) {
        self.run.cancel();
    }
    pub fn finish(&mut self, a: &mut Attempt, s: &Scenario) -> Result<(), String> {
        context(a, s)?;
        if !bounded::within(&(s, &*a), MAX_CONTEXT) {
            return Err("Current one-off source/Attempt exceeds its complete bound".into());
        }
        // Serialization is admitted and bounded first; these are exact transient
        // context comparisons, never expected results or installed Progress.
        if serde_json::to_vec(s).map_err(|_| "Source comparison failed")?
            != serde_json::to_vec(&self.source).map_err(|_| "Source comparison failed")?
            || serde_json::to_vec(a).map_err(|_| "Attempt comparison failed")?
                != serde_json::to_vec(&self.before).map_err(|_| "Attempt comparison failed")?
        {
            return Err(
                "One-off work changed during Run; candidate discarded, retry when ready".into(),
            );
        }
        let result = self
            .run
            .take_result()
            .ok_or("Run has not completed or was already consumed")??;
        let State::OneOff { simulation } = &a.state else {
            return Err("Not a one-off assignment".into());
        };
        let w = a.workspace.as_ref().unwrap();
        let (next_state, next_learning) = if self.practice {
            let l = w.one_off_learning.as_ref().unwrap();
            let old_l = bounded::size(l, MAX_ATTEMPT).unwrap();
            let next = old_l
                .checked_sub(bounded::size(&l.assignment, MAX_ATTEMPT).unwrap())
                .and_then(|n| n.checked_add(bounded::size(&result, MAX_ATTEMPT)?))
                .ok_or("One-off result capacity")?;
            (simulation.as_ref(), Some(next))
        } else {
            (
                &result,
                optional_size(w.one_off_learning.as_ref()).ok_or("Retained practice bound")?,
            )
        };
        admit(projected(
            a,
            next_state,
            optional_size(w.one_off_session.as_ref()).ok_or("Retained session bound")?,
            next_learning,
        ))?;
        if self.practice {
            a.workspace
                .as_mut()
                .unwrap()
                .one_off_learning
                .as_mut()
                .unwrap()
                .assignment = result;
        } else {
            let State::OneOff { simulation } = &mut a.state else {
                unreachable!()
            };
            **simulation = result;
            a.workspace.as_mut().unwrap().checked = false;
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct FreshWorkspace<'a> {
    #[serde(flatten)]
    base: &'a crate::workspace::WorkspaceData,
    one_off_session: SessionRef<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    one_off_learning: Option<LearningRef<'a>>,
}
#[derive(Serialize)]
struct FreshAttempt<'a> {
    scenario_version: u32,
    state: &'a State,
    step: usize,
    challenge: bool,
    hints: usize,
    transcript: &'a [String],
    visited: u64,
    workspace: FreshWorkspace<'a>,
}
/// Exact fresh wire count, including all default workspace scalars and optional
/// field punctuation. No initial State, draft or example is copied to count it.
pub fn fresh_size(s: &Scenario, visited: u64) -> Option<usize> {
    let (State::OneOff { simulation }, crate::engine::Goal::OneOff { definition }) =
        (&s.initial, &s.goal)
    else {
        return None;
    };
    if !source_shape(s)
        || definition.validate().is_err()
        || !simulation.valid()
        || simulation.simulation != definition.initial
        || simulation.last_run.is_some()
        || s.one_off_teaching
            .as_ref()
            .is_some_and(|l| l.validate_context().is_err() || !l.separate_from(definition))
    {
        return None;
    }
    let blank = crate::workspace::WorkspaceData::new();
    let learning = s.one_off_teaching.as_ref().map(|l| LearningRef {
        version: 1,
        view: View::Introduction,
        definition: &l.definition,
        assignment: AssignmentRef {
            simulation: &l.definition.initial,
            last_run: None,
        },
        session: initial_session(&l.definition, &l.definition.initial),
        selected_example: 0,
    });
    bounded::size(
        &FreshAttempt {
            scenario_version: s.version,
            state: &s.initial,
            step: 0,
            challenge: false,
            hints: 0,
            transcript: &[],
            visited,
            workspace: FreshWorkspace {
                base: &blank,
                one_off_session: initial_session(definition, &simulation.simulation),
                one_off_learning: learning,
            },
        },
        MAX_ATTEMPT,
    )
}
pub fn reset_attempt(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    context(a, s)?;
    admit(fresh_size(s, a.visited))?;
    let mut fresh = Attempt::new(s, a.visited);
    prepare(&mut fresh, s)?;
    if bounded::size(&fresh, MAX_ATTEMPT) != fresh_size(s, a.visited) {
        return Err("Fresh one-off wire projection differs; prior work retained".into());
    }
    *a = fresh;
    Ok(())
}
pub fn record_visit(a: &mut Attempt, s: &Scenario, visited: u64) -> Result<(), String> {
    context(a, s)?;
    let old = bounded::size(a, MAX_ATTEMPT).ok_or("One-off complete Attempt bound")?;
    admit(
        old.checked_sub(bounded::size(&a.visited, MAX_ATTEMPT).unwrap())
            .and_then(|n| n.checked_add(bounded::size(&visited, MAX_ATTEMPT)?)),
    )?;
    a.visited = visited;
    Ok(())
}
pub fn set_challenge(a: &mut Attempt, s: &Scenario, challenge: bool) -> Result<(), String> {
    context(a, s)?;
    let old = bounded::size(a, MAX_ATTEMPT).ok_or("One-off complete Attempt bound")?;
    admit(
        old.checked_sub(bounded::size(&a.challenge, MAX_ATTEMPT).unwrap())
            .and_then(|n| n.checked_add(bounded::size(&challenge, MAX_ATTEMPT)?)),
    )?;
    a.challenge = challenge;
    Ok(())
}
