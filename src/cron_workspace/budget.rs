//! Borrowed exact-wire projections. No fixture/draft copy precedes admission.
use super::*;
use serde::ser::SerializeMap;
use std::borrow::Cow;

fn installed<'a>(document: &Document, state: &'a State) -> Cow<'a, str> {
    match document {
        Document::User { owner } => {
            Cow::Borrowed(state.tables.get(owner).map(String::as_str).unwrap_or(""))
        }
        Document::System => Cow::Borrowed(&state.system_table),
        Document::Allow => Cow::Owned(policy_text(state.allow.as_ref())),
        Document::Deny => Cow::Owned(policy_text(state.deny.as_ref())),
    }
}
#[derive(Clone, Copy, Default)]
pub struct SessionEdit<'a> {
    pub selected: Option<&'a Document>,
    pub actor: Option<&'a str>,
    pub draft: Option<(&'a Document, &'a str)>,
    pub window_end: Option<&'a str>,
}
enum Drafts<'a> {
    Initial(&'a Definition, &'a State),
    Existing(&'a Session, Option<(&'a Document, &'a str)>),
}
impl Serialize for Drafts<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Initial(d, state) => {
                for t in &d.editable {
                    map.serialize_entry(&t.key(), &installed(t, state))?;
                }
            }
            Self::Existing(w, edit) => {
                for (key, value) in &w.drafts {
                    let value = edit
                        .filter(|(target, _)| target.key() == *key)
                        .map_or(value.as_str(), |(_, text)| text);
                    map.serialize_entry(key, value)?;
                }
            }
        }
        map.end()
    }
}
#[derive(Serialize)]
pub(crate) struct SessionRef<'a> {
    initial: &'a State,
    selected: &'a Document,
    actor: &'a str,
    drafts: Drafts<'a>,
    window_end: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_run: Option<&'a RunLog>,
}
impl SessionRef<'_> {
    fn owned(&self) -> Session {
        #[cfg(test)]
        SESSION_COPIES.with(|n| n.set(n.get() + 1));
        let drafts = match &self.drafts {
            Drafts::Initial(d, state) => d
                .editable
                .iter()
                .map(|t| (t.key(), installed(t, state).into_owned()))
                .collect(),
            Drafts::Existing(w, edit) => w
                .drafts
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        edit.filter(|(target, _)| target.key() == *key)
                            .map_or(value.as_str(), |(_, text)| text)
                            .into(),
                    )
                })
                .collect(),
        };
        Session {
            initial: self.initial.clone(),
            selected: self.selected.clone(),
            actor: self.actor.into(),
            drafts,
            window_end: self.window_end.to_string(),
            last_run: self.last_run.cloned(),
        }
    }
}
fn initial_session(d: &Definition) -> SessionRef<'_> {
    session_from_state(d, &d.initial)
}
fn session_from_state<'a>(d: &'a Definition, state: &'a State) -> SessionRef<'a> {
    SessionRef {
        initial: &d.initial,
        selected: &d.editable[0],
        actor: if d.initial.users.contains("alex") {
            "alex"
        } else {
            "root"
        },
        drafts: Drafts::Initial(d, state),
        last_run: None,
        window_end: Cow::Owned(state.clock.after(60).unwrap_or(state.clock).text()),
    }
}
fn edited_session<'a>(
    d: &Definition,
    w: &'a Session,
    edit: SessionEdit<'a>,
) -> Result<SessionRef<'a>, String> {
    if !w.valid_for(d)
        || edit.selected.is_some_and(|t| !d.editable.contains(t))
        || edit.actor.is_some_and(|n| !d.initial.users.contains(n))
        || edit.draft.is_some_and(|(t, text)| {
            !d.editable.contains(t) || text.len() > core::MAX_TABLE_BYTES || text.contains('\0')
        })
        || edit
            .window_end
            .is_some_and(|s| s.len() > 32 || s.contains('\0'))
    {
        return Err(
            "Scheduling edit exceeds its bounds or supplied context; previous fields retained"
                .into(),
        );
    }
    Ok(SessionRef {
        initial: &w.initial,
        selected: edit.selected.unwrap_or(&w.selected),
        actor: edit.actor.unwrap_or(&w.actor),
        drafts: Drafts::Existing(w, edit.draft),
        last_run: w.last_run.as_ref(),
        window_end: Cow::Borrowed(edit.window_end.unwrap_or(&w.window_end)),
    })
}
#[derive(Serialize)]
struct LearningRef<'a, T: Serialize = State> {
    version: u32,
    view: crate::script_teaching::View,
    definition: &'a Definition,
    simulation: &'a T,
    session: SessionRef<'a>,
    selected_example: usize,
}
#[derive(Serialize)]
#[serde(tag = "kind")]
enum StateRef<'a, T: Serialize> {
    Cron { simulation: &'a T },
}
fn field_bytes(name: &str, value: Option<usize>) -> Option<usize> {
    value.map_or(Some(0), |n| n.checked_add(name.len() + 4))
}
fn optional_size<T: Serialize>(value: Option<&T>) -> Option<Option<usize>> {
    match value {
        None => Some(None),
        Some(v) => Some(Some(size(v, MAX_ATTEMPT)?)),
    }
}
/// Exact size substitution includes the entire existing Attempt, its transcript,
/// all raw workspace fields and optional-key punctuation. It is not a grade.
pub fn projected_attempt_size(
    a: &crate::progress::Attempt,
    state: &State,
    session: Option<usize>,
    learning: Option<usize>,
) -> Option<usize> {
    if !matches!(a.state, crate::engine::State::Cron { .. }) {
        return None;
    }
    projected_size(a, &StateRef::Cron { simulation: state }, session, learning)
}
fn projected_size<T: Serialize>(
    a: &crate::progress::Attempt,
    state: &T,
    session: Option<usize>,
    learning: Option<usize>,
) -> Option<usize> {
    let old = size(a, MAX_ATTEMPT)?;
    let old_state = size(&a.state, MAX_ATTEMPT)?;
    let next_state = size(state, MAX_ATTEMPT)?;
    let blank = crate::workspace::WorkspaceData::new();
    let w = a.workspace.as_ref().unwrap_or(&blank);
    let old_w = size(w, MAX_ATTEMPT)?;
    let next_w = old_w
        .checked_sub(field_bytes(
            "cron_session",
            optional_size(w.cron_session.as_ref())?,
        )?)?
        .checked_sub(field_bytes(
            "cron_learning",
            optional_size(w.cron_learning.as_ref())?,
        )?)?
        .checked_add(field_bytes("cron_session", session)?)?
        .checked_add(field_bytes("cron_learning", learning)?)?;
    let next = old.checked_sub(old_state)?.checked_add(next_state)?;
    if a.workspace.is_some() {
        next.checked_sub(old_w)?.checked_add(next_w)
    } else {
        next.checked_add("workspace".len() + 4)?.checked_add(next_w)
    }
}
fn admit(bytes: Option<usize>) -> Result<(), String> {
    if bytes.is_some_and(|n| n <= MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("Saved work has no room for this edit. Previous text is retained and nothing ran; shorten it and retry.".into())
    }
}
/// Initialize only after complete borrowed projection fits. Existing saved
/// drafts/practice are preserved; no example execution or credit is performed.
pub fn prepare(a: &mut crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::Cron { simulation } = &a.state else {
        return Err("Not a scheduling attempt".into());
    };
    let old = a.workspace.as_ref();
    let new_session = old.is_none_or(|w| w.cron_session.is_none());
    let new_learning = s.cron_teaching.is_some() && old.is_none_or(|w| w.cron_learning.is_none());
    let session = if new_session {
        Some(
            size(&session_from_state(d, simulation), MAX_SESSION)
                .ok_or("Scheduling session bound")?,
        )
    } else {
        optional_size(old.and_then(|w| w.cron_session.as_ref())).ok_or("Retained session bound")?
    };
    let learning = if new_learning {
        let l = s.cron_teaching.as_ref().unwrap();
        l.validate_context()?;
        Some(
            size(
                &LearningRef {
                    version: 1,
                    view: crate::script_teaching::View::Introduction,
                    definition: &l.definition,
                    simulation: &l.definition.initial,
                    session: initial_session(&l.definition),
                    selected_example: 0,
                },
                crate::cron_teaching::MAX_LEARNING,
            )
            .ok_or("New scheduling practice bound")?,
        )
    } else {
        optional_size(old.and_then(|w| w.cron_learning.as_ref()))
            .ok_or("Retained practice bound")?
    };
    admit(projected_attempt_size(a, simulation, session, learning))?;
    let session = new_session.then(|| session_from_state(d, simulation).owned());
    let learning = if new_learning {
        Some(crate::cron_teaching::Learning::new(
            s.cron_teaching.as_ref().unwrap(),
        )?)
    } else {
        None
    };
    let w = a
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new);
    if let Some(session) = session {
        w.cron_session = Some(session)
    }
    if let Some(learning) = learning {
        w.cron_learning = Some(learning)
    }
    Ok(())
}
pub fn edit_attempt(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    edit: SessionEdit<'_>,
    practice: bool,
) -> Result<(), String> {
    edit_batch(a, s, edit, practice, None)
}
/// One native edit packet may also switch the separate example/view. Both wire
/// deltas are admitted together before any saved draft or presentation copy.
pub fn edit_batch(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    edit: SessionEdit<'_>,
    practice: bool,
    presentation: Option<(crate::script_teaching::View, usize)>,
) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::Cron { simulation } = &a.state else {
        return Err("Not a scheduling attempt".into());
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let (d, old) = if practice {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        (&l.definition, &l.session)
    } else {
        (
            d,
            w.cron_session
                .as_ref()
                .ok_or("Scheduling editor unprepared")?,
        )
    };
    if let Some((_, selected)) = presentation {
        let lesson = s
            .cron_teaching
            .as_ref()
            .ok_or("Scheduling introduction unavailable")?;
        if w.cron_learning.is_none() || selected >= lesson.example_actions.len() {
            return Err("Choose an existing scheduling example action".into());
        }
    }
    let proposed = edited_session(d, old, edit)?;
    let session_size = size(&proposed, MAX_SESSION).ok_or("Scheduling session bound")?;
    let session = if practice {
        optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?
    } else {
        Some(session_size)
    };
    let learning = if practice || presentation.is_some() {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        let (view, selected_example) = presentation.unwrap_or((l.view, l.selected_example));
        let session = if practice {
            edited_session(d, old, edit)?
        } else {
            edited_session(&l.definition, &l.session, SessionEdit::default())?
        };
        Some(
            size(
                &LearningRef {
                    version: l.version,
                    view,
                    definition: &l.definition,
                    simulation: &l.simulation,
                    session,
                    selected_example,
                },
                crate::cron_teaching::MAX_LEARNING,
            )
            .ok_or("Scheduling practice bound")?,
        )
    } else {
        optional_size(w.cron_learning.as_ref()).ok_or("Retained practice bound")?
    };
    admit(projected_attempt_size(a, simulation, session, learning))?;
    let owned = proposed.owned();
    let w = a.workspace.as_mut().unwrap();
    if practice {
        w.cron_learning.as_mut().unwrap().session = owned
    } else {
        w.cron_session = Some(owned)
    }
    if let Some((view, selected)) = presentation {
        let l = w.cron_learning.as_mut().unwrap();
        l.view = view;
        l.selected_example = selected;
    }
    Ok(())
}
/// Publish the bounded Run's owned result only after the complete saved Attempt
/// fits. Caller keeps old state on refusal; this function never runs the VM.
pub fn publish_run(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    result: State,
    practice: bool,
) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::Cron { simulation } = &a.state else {
        return Err("Not a scheduling attempt".into());
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let session = optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?;
    let (next_state, learning) = if practice {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        if !l.definition.bound_state(&result) {
            return Err("Scheduling result context differs".into());
        };
        let ref_session = edited_session(&l.definition, &l.session, SessionEdit::default())?;
        (
            simulation,
            Some(
                size(
                    &LearningRef {
                        version: l.version,
                        view: l.view,
                        definition: &l.definition,
                        simulation: &result,
                        session: ref_session,
                        selected_example: l.selected_example,
                    },
                    crate::cron_teaching::MAX_LEARNING,
                )
                .ok_or("Scheduling practice bound")?,
            ),
        )
    } else {
        if !d.bound_state(&result) {
            return Err("Scheduling result context differs".into());
        };
        (
            &result,
            optional_size(w.cron_learning.as_ref()).ok_or("Retained practice bound")?,
        )
    };
    admit(projected_attempt_size(a, next_state, session, learning))?;
    if practice {
        a.workspace
            .as_mut()
            .unwrap()
            .cron_learning
            .as_mut()
            .unwrap()
            .simulation = result
    } else {
        a.state = crate::engine::State::Cron { simulation: result }
    }
    Ok(())
}
#[cfg(test)]
thread_local! {static SESSION_COPIES:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}

struct TableRef<'a> {
    old: &'a BTreeMap<String, String>,
    edit: Option<(&'a str, Option<&'a str>)>,
}
impl Serialize for TableRef<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        for (key, value) in self.old {
            if self.edit.is_some_and(|(owner, _)| owner == key) {
                if let Some((_, Some(text))) = self.edit {
                    map.serialize_entry(key, text)?;
                }
            } else {
                map.serialize_entry(key, value)?;
            }
        }
        if let Some((owner, Some(text))) = self.edit
            && !self.old.contains_key(owner)
        {
            map.serialize_entry(owner, text)?;
        }
        map.end()
    }
}
#[derive(Serialize)]
struct StateDraftRef<'a> {
    clock: &'a Moment,
    users: &'a BTreeSet<String>,
    tables: TableRef<'a>,
    system_table: &'a str,
    allow: Option<&'a BTreeSet<String>>,
    deny: Option<&'a BTreeSet<String>>,
    files: &'a BTreeMap<String, String>,
}
/// Install/Remove are atomic native operations. A proposed complete wire is
/// borrowed (including the edited 8KiB string) before core state/session copies.
/// Run is deliberately absent: UI execution owns its yielding continuation.
pub fn perform_attempt(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    action: &Action,
    practice: bool,
) -> Result<(), String> {
    match action {
        Action::Edit { target, text } => {
            return edit_attempt(
                a,
                s,
                SessionEdit {
                    draft: Some((target, text)),
                    ..SessionEdit::default()
                },
                practice,
            );
        }
        Action::Actor { actor } => {
            return edit_attempt(
                a,
                s,
                SessionEdit {
                    actor: Some(actor),
                    ..SessionEdit::default()
                },
                practice,
            );
        }
        Action::Select { target } => {
            return edit_attempt(
                a,
                s,
                SessionEdit {
                    selected: Some(target),
                    ..SessionEdit::default()
                },
                practice,
            );
        }
        Action::Run { .. } => return Err(
            "Use the native Run/Stop continuation; no synchronous learner execution is dispatched"
                .into(),
        ),
        _ => {}
    }
    let d = context(a, s)?;
    let crate::engine::State::Cron {
        simulation: assignment,
    } = &a.state
    else {
        return Err("Not a scheduling attempt".into());
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let (d, state, session) = if practice {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        (&l.definition, &l.simulation, &l.session)
    } else {
        (
            d,
            assignment,
            w.cron_session
                .as_ref()
                .ok_or("Scheduling editor unprepared")?,
        )
    };
    let (target, remove) = match action {
        Action::Install { target } => (target, false),
        Action::Remove { target } => (target, true),
        _ => unreachable!(),
    };
    if !d.editable.contains(target) {
        return Err("This document is not editable in this assignment".into());
    }
    let text = session
        .drafts
        .get(&target.key())
        .ok_or("Scheduling draft missing")?;
    // Access policy is small supplied-name data. Parsing it does not copy any
    // table/script text. The core still performs all authoritative permission,
    // parse and combined-entry checks after whole-Attempt admission.
    let names = if !remove && matches!(target, Document::Allow | Document::Deny) {
        Some(policy(text, &d.initial)?)
    } else {
        None
    };
    let mut proposed = StateDraftRef {
        clock: &state.clock,
        users: &state.users,
        tables: TableRef {
            old: &state.tables,
            edit: None,
        },
        system_table: &state.system_table,
        allow: state.allow.as_ref(),
        deny: state.deny.as_ref(),
        files: &state.files,
    };
    match target {
        Document::User { owner } => {
            proposed.tables.edit = Some((owner, if remove { None } else { Some(text) }))
        }
        Document::System => proposed.system_table = if remove { "" } else { text },
        Document::Allow => proposed.allow = names.as_ref(),
        Document::Deny => proposed.deny = names.as_ref(),
    }
    let edit = SessionEdit {
        draft: remove.then_some((target, "")),
        ..SessionEdit::default()
    };
    let session_size =
        size(&edited_session(d, session, edit)?, MAX_SESSION).ok_or("Scheduling session bound")?;
    let learning = if practice {
        let l = w.cron_learning.as_ref().unwrap();
        let size = size(
            &LearningRef {
                version: l.version,
                view: l.view,
                definition: &l.definition,
                simulation: &proposed,
                session: edited_session(d, session, edit)?,
                selected_example: l.selected_example,
            },
            crate::cron_teaching::MAX_LEARNING,
        )
        .ok_or("Scheduling practice bound")?;
        Some(size)
    } else {
        optional_size(w.cron_learning.as_ref()).ok_or("Retained practice bound")?
    };
    let session = if practice {
        optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?
    } else {
        Some(session_size)
    };
    let projected = if practice {
        projected_size(
            a,
            &StateRef::Cron {
                simulation: assignment,
            },
            session,
            learning,
        )
    } else {
        projected_size(
            a,
            &StateRef::Cron {
                simulation: &proposed,
            },
            session,
            learning,
        )
    };
    admit(projected)?;
    if practice {
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .cron_learning
            .as_mut()
            .unwrap();
        apply(&l.definition, &mut l.simulation, &mut l.session, action)
    } else {
        let crate::engine::State::Cron { simulation } = &mut a.state else {
            unreachable!()
        };
        // Reborrow the authored assignment directly. The preflight's `d` may
        // originate in the practice branch and hence borrow WorkspaceData.
        let crate::engine::Goal::Cron { definition } = &s.goal else {
            unreachable!()
        };
        apply(
            definition,
            simulation,
            a.workspace.as_mut().unwrap().cron_session.as_mut().unwrap(),
            action,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scenario() -> crate::pack::Scenario {
        crate::pack::load(include_str!("../../tests/fixtures/cron-workbench-v1.json"))
            .unwrap()
            .remove(0)
    }
    fn bytes(a: &crate::progress::Attempt) -> Vec<u8> {
        serde_json::to_vec(a).unwrap()
    }
    fn full(a: &mut crate::progress::Attempt) {
        a.transcript = vec![String::new()];
        let n = bytes(a).len();
        assert!(n < MAX_ATTEMPT);
        a.transcript[0] = "x".repeat(MAX_ATTEMPT - n);
        assert_eq!(bytes(a).len(), MAX_ATTEMPT);
    }
    fn selected(a: &crate::progress::Attempt) -> &Session {
        a.workspace.as_ref().unwrap().cron_session.as_ref().unwrap()
    }
    #[test]
    fn exact_borrowed_initial_projection_and_full_attempt_pre_copy_refusal() {
        let s = scenario();
        let d = match &s.goal {
            crate::engine::Goal::Cron { definition } => definition,
            _ => unreachable!(),
        };
        let mut a = crate::progress::Attempt::new(&s, 17);
        let before = bytes(&a);
        let initial = initial_session(d);
        assert_eq!(
            size(&initial, MAX_SESSION),
            size(&Session::new(d).unwrap(), MAX_SESSION)
        );
        let lesson = s.cron_teaching.as_ref().unwrap();
        let proposed = LearningRef {
            version: 1,
            view: crate::script_teaching::View::Introduction,
            definition: &lesson.definition,
            simulation: &lesson.definition.initial,
            session: initial_session(&lesson.definition),
            selected_example: 0,
        };
        assert_eq!(
            size(&proposed, MAX_ATTEMPT),
            size(
                &crate::cron_teaching::Learning::new(lesson).unwrap(),
                MAX_ATTEMPT
            )
        );
        let expected = projected_attempt_size(
            &a,
            &d.initial,
            size(&initial, MAX_SESSION),
            size(&proposed, MAX_ATTEMPT),
        )
        .unwrap();
        prepare(&mut a, &s).unwrap();
        assert_eq!(bytes(&a).len(), expected);
        assert_ne!(bytes(&a), before);
        let mut capped = crate::progress::Attempt::new(&s, 17);
        full(&mut capped);
        let before = bytes(&capped);
        let count = SESSION_COPIES.with(|n| n.get());
        assert!(prepare(&mut capped, &s).is_err());
        assert_eq!(bytes(&capped), before);
        assert_eq!(SESSION_COPIES.with(|n| n.get()), count);
    }
    #[test]
    fn full_attempt_rejects_whole_edit_install_and_result_without_publishing() {
        let s = scenario();
        let mut a = crate::progress::Attempt::new(&s, 17);
        prepare(&mut a, &s).unwrap();
        let target = Document::User {
            owner: "alex".into(),
        };
        full(&mut a);
        let before = bytes(&a);
        let count = SESSION_COPIES.with(|n| n.get());
        assert!(
            edit_attempt(
                &mut a,
                &s,
                SessionEdit {
                    draft: Some((&target, "* * * * * echo changed\n")),
                    ..SessionEdit::default()
                },
                false
            )
            .is_err()
        );
        assert_eq!(bytes(&a), before);
        assert_eq!(SESSION_COPIES.with(|n| n.get()), count);
        // Make room, keep a real pending table, then refill the complete Attempt.
        a.transcript.clear();
        edit_attempt(
            &mut a,
            &s,
            SessionEdit {
                draft: Some((&target, "* * * * * cat /work/source > /work/report\n")),
                ..SessionEdit::default()
            },
            false,
        )
        .unwrap();
        full(&mut a);
        let before = bytes(&a);
        assert!(
            perform_attempt(
                &mut a,
                &s,
                &Action::Install {
                    target: target.clone()
                },
                false
            )
            .is_err()
        );
        assert_eq!(bytes(&a), before);
        a.transcript.clear();
        perform_attempt(&mut a, &s, &Action::Install { target }, false).unwrap();
        let crate::engine::State::Cron { simulation } = &a.state else {
            unreachable!()
        };
        let mut run =
            core::Run::begin(simulation, Moment::parse("2026-10-28 09:00").unwrap()).unwrap();
        while !run.done() {
            run.tick(64)
        }
        let result = run.take_result().unwrap().unwrap();
        full(&mut a);
        let before = bytes(&a);
        assert!(publish_run(&mut a, &s, result.state, false).is_err());
        assert_eq!(bytes(&a), before);
    }
    #[test]
    fn selected_document_and_example_drafts_preserve_other_work_and_restore_exactly() {
        let s = scenario();
        let mut a = crate::progress::Attempt::new(&s, 41);
        prepare(&mut a, &s).unwrap();
        let saved = bytes(&a);
        let assignment = serde_json::to_value(selected(&a)).unwrap();
        let target = Document::User {
            owner: "mira".into(),
        };
        let count = SESSION_COPIES.with(|n| n.get());
        edit_attempt(
            &mut a,
            &s,
            SessionEdit {
                draft: Some((&target, "* * * * * echo practice\n")),
                actor: Some("mira"),
                ..SessionEdit::default()
            },
            true,
        )
        .unwrap();
        assert_eq!(serde_json::to_value(selected(&a)).unwrap(), assignment);
        assert_eq!(SESSION_COPIES.with(|n| n.get()), count + 1);
        let raw = serde_json::to_vec(&a).unwrap();
        let restored: crate::progress::Attempt = serde_json::from_slice(&raw).unwrap();
        assert_eq!(bytes(&restored), raw);
        context(&restored, &s).unwrap();
        assert_ne!(bytes(&a), saved);
        let before = bytes(&a);
        assert!(
            perform_attempt(
                &mut a,
                &s,
                &Action::Run {
                    end: Moment::parse("2026-10-28 09:00").unwrap()
                },
                true
            )
            .unwrap_err()
            .contains("continuation")
        );
        assert_eq!(bytes(&a), before);
    }
}

/// Presentation edits are saved work too; predict the entire Learning wire while
/// borrowing every draft/definition/state before changing view or selected help.
pub fn set_presentation(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    view: crate::script_teaching::View,
    selected: usize,
) -> Result<(), String> {
    context(a, s)?;
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let l = w
        .cron_learning
        .as_ref()
        .ok_or("Scheduling example unprepared")?;
    let lesson = s
        .cron_teaching
        .as_ref()
        .ok_or("Scheduling introduction unavailable")?;
    if selected >= lesson.example_actions.len() {
        return Err("Choose an existing example action".into());
    }
    let next = LearningRef {
        version: l.version,
        view,
        definition: &l.definition,
        simulation: &l.simulation,
        session: edited_session(&l.definition, &l.session, SessionEdit::default())?,
        selected_example: selected,
    };
    let learning =
        size(&next, crate::cron_teaching::MAX_LEARNING).ok_or("Scheduling practice bound")?;
    let crate::engine::State::Cron { simulation } = &a.state else {
        unreachable!()
    };
    admit(projected_attempt_size(
        a,
        simulation,
        optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?,
        Some(learning),
    ))?;
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .cron_learning
        .as_mut()
        .unwrap();
    l.view = view;
    l.selected_example = selected;
    Ok(())
}
pub fn fill_example(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.cron_learning.as_ref())
        .ok_or("Scheduling example unprepared")?;
    let action = &s
        .cron_teaching
        .as_ref()
        .ok_or("Scheduling example unavailable")?
        .example_actions[l.selected_example]
        .action;
    match action {
        Action::Edit { target, text } => edit_attempt(
            a,
            s,
            SessionEdit {
                draft: Some((target, text)),
                ..SessionEdit::default()
            },
            true,
        ),
        Action::Actor { actor } => edit_attempt(
            a,
            s,
            SessionEdit {
                actor: Some(actor),
                ..SessionEdit::default()
            },
            true,
        ),
        Action::Select { target } | Action::Install { target } | Action::Remove { target } => {
            edit_attempt(
                a,
                s,
                SessionEdit {
                    selected: Some(target),
                    ..SessionEdit::default()
                },
                true,
            )
        }
        Action::Run { end } => {
            let text = end.text();
            edit_attempt(
                a,
                s,
                SessionEdit {
                    window_end: Some(&text),
                    ..SessionEdit::default()
                },
                true,
            )
        }
    }
}
pub fn reset_example(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
) -> Result<(), String> {
    context(a, s)?;
    let lesson = s
        .cron_teaching
        .as_ref()
        .ok_or("Scheduling example unavailable")?;
    lesson.validate_context()?;
    let next = LearningRef {
        version: 1,
        view: crate::script_teaching::View::Example,
        definition: &lesson.definition,
        simulation: &lesson.definition.initial,
        session: initial_session(&lesson.definition),
        selected_example: 0,
    };
    let bytes =
        size(&next, crate::cron_teaching::MAX_LEARNING).ok_or("Scheduling practice bound")?;
    let crate::engine::State::Cron { simulation } = &a.state else {
        unreachable!()
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    admit(projected_attempt_size(
        a,
        simulation,
        optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?,
        Some(bytes),
    ))?;
    let mut next = crate::cron_teaching::Learning::new(lesson)?;
    next.view = crate::script_teaching::View::Example;
    a.workspace.as_mut().unwrap().cron_learning = Some(next);
    Ok(())
}
pub fn record_visit(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    visited: u64,
) -> Result<(), String> {
    context(a, s)?;
    let old = size(a, MAX_ATTEMPT).ok_or("Scheduling saved capacity bound")?;
    let before = size(&a.visited, 32).ok_or("Prior visit bound")?;
    let after = size(&visited, 32).ok_or("Proposed visit bound")?;
    admit(old.checked_sub(before).and_then(|n|n.checked_add(after))).map_err(|_|"This attempt has no room for a later visit count. All work and its earlier visit are retained.".to_string())?;
    a.visited = visited;
    Ok(())
}

/// Move actual completed VM output into saved history, with no output-string
/// copy before the complete borrowed State+Session+Learning projection fits.
pub fn publish_completed(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    start: Moment,
    result: core::Completed,
    practice: bool,
) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::Cron {
        simulation: assignment,
    } = &a.state
    else {
        return Err("Not a scheduling attempt".into());
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let (d, session) = if practice {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        (&l.definition, &l.session)
    } else {
        (
            d,
            w.cron_session
                .as_ref()
                .ok_or("Scheduling editor unprepared")?,
        )
    };
    if !d.bound_state(&result.state) {
        return Err("Scheduling result context differs".into());
    }
    let log = RunLog {
        start,
        end: result.state.clock,
        events: result
            .records
            .into_iter()
            .map(|r| RunEvent {
                when: r.when,
                owner: r.owner,
                stdout: r.stdout,
                stderr: r.stderr,
                status: r.status,
            })
            .collect(),
    };
    if !log.valid(&d.initial.users) {
        return Err("Completed scheduling output exceeds its supplied-context bounds".into());
    }
    let mut proposed = edited_session(d, session, SessionEdit::default())?;
    proposed.last_run = Some(&log);
    let proposed_session = size(&proposed, MAX_SESSION).ok_or("Scheduling output/session bound")?;
    let (state, session_bytes, learning) = if practice {
        let l = w.cron_learning.as_ref().unwrap();
        let bytes = size(
            &LearningRef {
                version: l.version,
                view: l.view,
                definition: &l.definition,
                simulation: &result.state,
                session: proposed,
                selected_example: l.selected_example,
            },
            crate::cron_teaching::MAX_LEARNING,
        )
        .ok_or("Scheduling output/practice bound")?;
        (
            assignment,
            optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?,
            Some(bytes),
        )
    } else {
        (
            &result.state,
            Some(proposed_session),
            optional_size(w.cron_learning.as_ref()).ok_or("Retained practice bound")?,
        )
    };
    admit(projected_attempt_size(a,state,session_bytes,learning)).map_err(|_|"The private run finished, but saved work has no room for its result. Previous files, clock and output are retained; no changes from this run were applied.".to_string())?;
    if practice {
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .cron_learning
            .as_mut()
            .unwrap();
        l.simulation = result.state;
        l.session.last_run = Some(log)
    } else {
        a.state = crate::engine::State::Cron {
            simulation: result.state,
        };
        a.workspace
            .as_mut()
            .unwrap()
            .cron_session
            .as_mut()
            .unwrap()
            .last_run = Some(log)
    }
    Ok(())
}

/// Clearly simulator-only retry: keep configured schedules, policies and drafts;
/// restore this scope's supplied files/clock and discard only its run output.
pub fn restore_samples(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    practice: bool,
) -> Result<(), String> {
    let d = context(a, s)?;
    let crate::engine::State::Cron {
        simulation: assignment,
    } = &a.state
    else {
        return Err("Not a scheduling attempt".into());
    };
    let w = a
        .workspace
        .as_ref()
        .ok_or("Scheduling workspace unprepared")?;
    let (d, state, session) = if practice {
        let l = w
            .cron_learning
            .as_ref()
            .ok_or("Scheduling example unprepared")?;
        (&l.definition, &l.simulation, &l.session)
    } else {
        (
            d,
            assignment,
            w.cron_session
                .as_ref()
                .ok_or("Scheduling editor unprepared")?,
        )
    };
    let proposed = StateDraftRef {
        clock: &d.initial.clock,
        users: &state.users,
        tables: TableRef {
            old: &state.tables,
            edit: None,
        },
        system_table: &state.system_table,
        allow: state.allow.as_ref(),
        deny: state.deny.as_ref(),
        files: &d.initial.files,
    };
    let mut next_session = edited_session(d, session, SessionEdit::default())?;
    next_session.last_run = None;
    let bytes = size(&next_session, MAX_SESSION).ok_or("Scheduling session bound")?;
    let (session_bytes, learning) = if practice {
        let l = w.cron_learning.as_ref().unwrap();
        (
            optional_size(w.cron_session.as_ref()).ok_or("Retained session bound")?,
            Some(
                size(
                    &LearningRef {
                        version: l.version,
                        view: l.view,
                        definition: &l.definition,
                        simulation: &proposed,
                        session: next_session,
                        selected_example: l.selected_example,
                    },
                    crate::cron_teaching::MAX_LEARNING,
                )
                .ok_or("Scheduling practice bound")?,
            ),
        )
    } else {
        (
            Some(bytes),
            optional_size(w.cron_learning.as_ref()).ok_or("Retained practice bound")?,
        )
    };
    admit(if practice {
        projected_size(
            a,
            &StateRef::Cron {
                simulation: assignment,
            },
            session_bytes,
            learning,
        )
    } else {
        projected_size(
            a,
            &StateRef::Cron {
                simulation: &proposed,
            },
            session_bytes,
            learning,
        )
    })?;
    let clock = d.initial.clock;
    let files = d.initial.files.clone();
    if practice {
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .cron_learning
            .as_mut()
            .unwrap();
        l.simulation.clock = clock;
        l.simulation.files = files;
        l.session.last_run = None;
    } else {
        let crate::engine::State::Cron { simulation } = &mut a.state else {
            unreachable!()
        };
        simulation.clock = clock;
        simulation.files = files;
        a.workspace
            .as_mut()
            .unwrap()
            .cron_session
            .as_mut()
            .unwrap()
            .last_run = None;
    }
    Ok(())
}

#[cfg(test)]
mod native_support_tests {
    use super::*;
    fn source() -> crate::pack::Scenario {
        crate::pack::load(include_str!("../../tests/fixtures/cron-workbench-v1.json"))
            .unwrap()
            .remove(0)
    }
    fn raw(a: &crate::progress::Attempt) -> Vec<u8> {
        serde_json::to_vec(a).unwrap()
    }
    fn full(a: &mut crate::progress::Attempt) {
        a.transcript = vec![String::new()];
        let n = raw(a).len();
        a.transcript[0] = "x".repeat(MAX_ATTEMPT - n);
        assert_eq!(raw(a).len(), MAX_ATTEMPT);
    }
    #[test]
    fn resumed_state_without_editor_metadata_uses_current_installed_documents_without_history() {
        let s = source();
        let mut a = crate::progress::Attempt::new(&s, 7);
        for input in &s.solution {
            crate::engine::execute_for_goal(&mut a.state, &s.goal, input).unwrap();
        }
        assert!(crate::engine::solved(&a.state, &s.goal));
        let before = a.state.clone();
        assert!(a.workspace.is_none());
        prepare(&mut a, &s).unwrap();
        assert_eq!(a.state, before);
        let d = context(&a, &s).unwrap();
        let crate::engine::State::Cron { simulation } = &a.state else {
            unreachable!()
        };
        let session = a.workspace.as_ref().unwrap().cron_session.as_ref().unwrap();
        assert!(session.applied(d, simulation));
        assert_eq!(session.initial, d.initial);
        assert!(session.last_run.is_none());
        assert_eq!(session.drafts["user:alex"], simulation.tables["alex"]);
        assert!(crate::engine::solved(&a.state, &s.goal));
    }
    #[test]
    fn combined_draft_and_navigation_refusal_is_whole_attempt_atomic_before_copy() {
        let s = source();
        let mut a = crate::progress::Attempt::new(&s, 7);
        prepare(&mut a, &s).unwrap();
        full(&mut a);
        let before = raw(&a);
        let copied = SESSION_COPIES.with(|n| n.get());
        let target = Document::User {
            owner: "alex".into(),
        };
        assert!(
            edit_batch(
                &mut a,
                &s,
                SessionEdit {
                    draft: Some((&target, "* * * * * echo too-large-for-full-attempt\n")),
                    ..SessionEdit::default()
                },
                false,
                Some((crate::script_teaching::View::Assignment, 0))
            )
            .is_err()
        );
        assert_eq!(raw(&a), before);
        assert_eq!(SESSION_COPIES.with(|n| n.get()), copied);
    }
    #[test]
    fn actual_completed_output_is_saved_and_sample_restore_preserves_tables_policies_and_drafts() {
        for practice in [false, true] {
            let s = source();
            let mut a = crate::progress::Attempt::new(&s, 7);
            prepare(&mut a, &s).unwrap();
            let owner = if practice { "mira" } else { "alex" };
            let path = if practice {
                "/work/example-input"
            } else {
                "/work/source"
            };
            let target = Document::User {
                owner: owner.into(),
            };
            perform_attempt(
                &mut a,
                &s,
                &Action::Actor {
                    actor: owner.into(),
                },
                practice,
            )
            .unwrap();
            perform_attempt(
                &mut a,
                &s,
                &Action::Edit {
                    target: target.clone(),
                    text: format!("* * * * * cat {path} > /work/stray\n"),
                },
                practice,
            )
            .unwrap();
            perform_attempt(&mut a, &s, &Action::Install { target }, practice).unwrap();
            let (state, session) = if practice {
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .cron_learning
                    .as_ref()
                    .unwrap();
                (&l.simulation, &l.session)
            } else {
                let crate::engine::State::Cron { simulation } = &a.state else {
                    unreachable!()
                };
                (
                    simulation,
                    a.workspace.as_ref().unwrap().cron_session.as_ref().unwrap(),
                )
            };
            let before_session = session.clone();
            let before_tables = state.tables.clone();
            let before_allow = state.allow.clone();
            let before_deny = state.deny.clone();
            let start = state.clock;
            let mut run = core::Run::begin(state, start.next().unwrap()).unwrap();
            while !run.done() {
                run.tick(64)
            }
            let result = run.take_result().unwrap().unwrap();
            publish_completed(&mut a, &s, start, result, practice).unwrap();
            let other = if practice {
                raw(&crate::progress::Attempt {
                    workspace: None,
                    ..a.clone()
                })
            } else {
                serde_json::to_vec(&a.workspace.as_ref().unwrap().cron_learning).unwrap()
            };
            let log = if practice {
                a.workspace
                    .as_ref()
                    .unwrap()
                    .cron_learning
                    .as_ref()
                    .unwrap()
                    .session
                    .last_run
                    .as_ref()
                    .unwrap()
            } else {
                a.workspace
                    .as_ref()
                    .unwrap()
                    .cron_session
                    .as_ref()
                    .unwrap()
                    .last_run
                    .as_ref()
                    .unwrap()
            };
            assert_eq!(log.events.len(), 1);
            assert_eq!(log.events[0].owner, owner);
            assert_eq!(log.events[0].status, 0);
            let restored: crate::progress::Attempt = serde_json::from_slice(&raw(&a)).unwrap();
            context(&restored, &s).unwrap();
            assert_eq!(raw(&restored), raw(&a));
            restore_samples(&mut a, &s, practice).unwrap();
            let (state, session, initial) = if practice {
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .cron_learning
                    .as_ref()
                    .unwrap();
                (&l.simulation, &l.session, &l.definition.initial)
            } else {
                let crate::engine::State::Cron { simulation } = &a.state else {
                    unreachable!()
                };
                (
                    simulation,
                    a.workspace.as_ref().unwrap().cron_session.as_ref().unwrap(),
                    &context(&a, &s).unwrap().initial,
                )
            };
            assert_eq!(state.clock, initial.clock);
            assert_eq!(state.files, initial.files);
            assert_eq!(state.tables, before_tables);
            assert_eq!(state.allow, before_allow);
            assert_eq!(state.deny, before_deny);
            assert_eq!(session.drafts, before_session.drafts);
            assert_eq!(session.actor, before_session.actor);
            assert_eq!(session.window_end, before_session.window_end);
            assert!(session.last_run.is_none());
            assert_eq!(a.visited, 7);
            let after_other = if practice {
                raw(&crate::progress::Attempt {
                    workspace: None,
                    ..a.clone()
                })
            } else {
                serde_json::to_vec(&a.workspace.as_ref().unwrap().cron_learning).unwrap()
            };
            assert_eq!(after_other, other);
        }
    }
}
