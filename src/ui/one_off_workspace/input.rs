//! Borrowed native edit-packet admission. No per-keystroke simulation clone.
use super::*;
use dlms_learning_lab::cron_workspace as bounded;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub(super) struct Packet<'a> {
    pub selected: Option<&'a str>,
    pub actor: Option<&'a str>,
    pub document: Option<(&'a str, &'a str)>,
    pub command: Option<&'a str>,
    pub end: Option<&'a str>,
    pub environment: Option<&'a BTreeMap<String, String>>,
    pub presentation: Option<(View, usize)>,
}
fn option_size<T: serde::Serialize>(v: Option<&T>) -> Option<Option<usize>> {
    match v {
        None => Some(None),
        Some(v) => Some(Some(bounded::size(v, work::MAX_ATTEMPT)?)),
    }
}
fn text_delta(n: usize, old: &str, new: &str) -> Option<usize> {
    n.checked_sub(bounded::string_bytes(old))?
        .checked_add(bounded::string_bytes(new))
}
pub(super) fn variable_names(d: &work::Definition, session: &work::Session) -> Option<Vec<String>> {
    let mut names: BTreeSet<&str> = session.environment.keys().map(String::as_str).collect();
    for r in &d.requirements {
        if let work::Requirement::CapturedEnvironment { values, .. } = r {
            names.extend(values.keys().map(String::as_str));
        }
    }
    (names.len() <= 16).then(|| names.into_iter().map(str::to_owned).collect())
}
pub(super) fn shape(a: &Attempt, s: &Scenario) -> bool {
    if work::context(a, s).is_err() {
        return false;
    }
    let Some((_, session)) = selected(a, false) else {
        return false;
    };
    let Some(d) = definition(s, false) else {
        return false;
    };
    if variable_names(d, session).is_none() {
        return false;
    }
    if s.one_off_teaching.is_some()
        && a.workspace
            .as_ref()
            .is_none_or(|w| w.one_off_learning.is_none())
    {
        return false;
    }
    if let Some(l) = a
        .workspace
        .as_ref()
        .and_then(|w| w.one_off_learning.as_ref())
        && variable_names(&l.definition, &l.session).is_none()
    {
        return false;
    }
    true
}
/// All text/settings/presentation fields are validated and projected together.
/// Only those admitted leaf fields are copied; no partly applied packet survives.
pub(super) fn commit(
    a: &mut Attempt,
    s: &Scenario,
    practice: bool,
    p: Packet<'_>,
) -> Result<(), String> {
    let d = work::context(a, s)?;
    let (_, session) = selected(a, practice).ok_or("Prepare one-off workspace")?;
    let d = if practice {
        definition(s, true).ok_or("No separate example")?
    } else {
        d
    };
    if !shape(a, s) {
        return Err(
            "This activity needs an unsupported native variable layout; previous work retained"
                .into(),
        );
    }
    let mut next_session =
        bounded::size(session, work::MAX_ATTEMPT).ok_or("Saved one-off draft capacity")?;
    if let Some(actor) = p.actor {
        if !d.initial.users.contains(actor) {
            return Err("Choose a supplied simulated account".into());
        }
        next_session =
            text_delta(next_session, &session.actor, actor).ok_or("One-off identity capacity")?;
    }
    if let Some(path) = p.selected {
        if !d.editable.iter().any(|v| v == path) {
            return Err("Choose a supplied virtual file".into());
        }
        next_session = text_delta(next_session, &session.selected, path)
            .ok_or("One-off selection capacity")?;
    }
    if let Some((path, text)) = p.document {
        let old = session
            .drafts
            .get(path)
            .filter(|_| d.editable.iter().any(|v| v == path))
            .ok_or("Edit a supplied virtual file")?;
        if text.contains('\0') || text.len() > core::MAX_FILES_BYTES {
            return Err("The virtual file draft exceeds its supported limit".into());
        }
        let total = session
            .drafts
            .iter()
            .try_fold(0usize, |n, (k, v)| {
                n.checked_add(k.len())?.checked_add(v.len())
            })
            .ok_or("One-off draft capacity")?;
        if total
            .checked_sub(old.len())
            .and_then(|n| n.checked_add(text.len()))
            .is_none_or(|n| n > core::MAX_FILES_BYTES)
        {
            return Err(
                "Keep the combined virtual file drafts within96KiB; prior text retained".into(),
            );
        }
        next_session = text_delta(next_session, old, text).ok_or("One-off draft capacity")?;
    }
    if let Some(text) = p.command {
        if text.len() > 2048 || text.contains(['\0', '\n', '\r']) {
            return Err("Use one supported scheduler command at a time".into());
        }
        next_session =
            text_delta(next_session, &session.command, text).ok_or("One-off command capacity")?;
    }
    if let Some(text) = p.end {
        if text.len() > 32 || text.contains('\0') {
            return Err("Keep the virtual UTC time within32characters".into());
        }
        next_session =
            text_delta(next_session, &session.window_end, text).ok_or("One-off time capacity")?;
    }
    if let Some(values) = p.environment {
        let allowed = variable_names(d, session).ok_or("Too many native variable fields")?;
        if values.len() > 16
            || values
                .iter()
                .any(|(k, v)| !allowed.contains(k) || v.len() > 1024 || v.contains('\0'))
        {
            return Err(
                "Use the supplied ordinary variable names and bounded literal values".into(),
            );
        }
        next_session = next_session
            .checked_sub(bounded::size(&session.environment, work::MAX_ATTEMPT).unwrap())
            .and_then(|n| n.checked_add(bounded::size(values, work::MAX_ATTEMPT)?))
            .ok_or("One-off variable capacity")?;
    }
    let w = a.workspace.as_ref().ok_or("Prepare one-off workspace")?;
    let mut next_learning =
        option_size(w.one_off_learning.as_ref()).ok_or("Separate example capacity")?;
    if practice {
        let l = w.one_off_learning.as_ref().ok_or("No separate example")?;
        next_learning = Some(
            next_learning
                .unwrap()
                .checked_sub(bounded::size(&l.session, work::MAX_ATTEMPT).unwrap())
                .and_then(|n| n.checked_add(next_session))
                .ok_or("Separate example capacity")?,
        );
    }
    if let Some((view, choice)) = p.presentation {
        let lesson = s
            .one_off_teaching
            .as_ref()
            .ok_or("No one-off introduction")?;
        let l = w
            .one_off_learning
            .as_ref()
            .ok_or("Prepare separate example")?;
        if choice >= lesson.example_actions.len() {
            return Err("Choose an existing example action".into());
        }
        next_learning = Some(
            next_learning
                .unwrap()
                .checked_sub(bounded::size(&l.view, work::MAX_ATTEMPT).unwrap())
                .and_then(|n| n.checked_sub(bounded::size(&l.selected_example, work::MAX_ATTEMPT)?))
                .and_then(|n| n.checked_add(bounded::size(&view, work::MAX_ATTEMPT)?))
                .and_then(|n| n.checked_add(bounded::size(&choice, work::MAX_ATTEMPT)?))
                .ok_or("One-off navigation capacity")?,
        );
    }
    let engine::State::OneOff {
        simulation: assignment,
    } = &a.state
    else {
        return Err("Not a one-off assignment".into());
    };
    let next = work::budget::projected(
        a,
        assignment,
        if practice {
            option_size(w.one_off_session.as_ref()).ok_or("One-off draft capacity")?
        } else {
            Some(next_session)
        },
        next_learning,
    )
    .and_then(|n| n.checked_add(1));
    if next.is_none_or(|n| n > work::MAX_ATTEMPT) {
        return Err(
            "Saved work limit reached. Previous fields are retained and nothing ran".into(),
        );
    }
    // No fallible operations after admission.
    let w = a.workspace.as_mut().unwrap();
    let session = if practice {
        &mut w.one_off_learning.as_mut().unwrap().session
    } else {
        w.one_off_session.as_mut().unwrap()
    };
    if let Some(v) = p.actor {
        session.actor = v.into();
    }
    if let Some(v) = p.selected {
        session.selected = v.into();
    }
    if let Some((path, text)) = p.document {
        session.drafts.insert(path.into(), text.into());
    }
    if let Some(v) = p.command {
        session.command = v.into();
    }
    if let Some(v) = p.end {
        session.window_end = v.into();
    }
    if let Some(v) = p.environment {
        session.environment = v.clone();
    }
    if let Some((view, choice)) = p.presentation {
        let l = w.one_off_learning.as_mut().unwrap();
        l.view = view;
        l.selected_example = choice;
    }
    if !practice {
        w.checked = false;
    }
    Ok(())
}
pub(super) fn reset_example(a: &mut Attempt, s: &Scenario) -> Result<(), String> {
    work::context(a, s)?;
    let lesson = s.one_off_teaching.as_ref().ok_or("No separate example")?;
    let next = work::budget::fresh_learning_size(lesson).ok_or("Separate example capacity")?;
    let engine::State::OneOff { simulation } = &a.state else {
        return Err("Not one-off assignment".into());
    };
    let w = a.workspace.as_ref().ok_or("Prepare one-off workspace")?;
    if work::budget::projected(
        a,
        simulation,
        option_size(w.one_off_session.as_ref()).ok_or("Saved draft capacity")?,
        Some(next),
    )
    .is_none_or(|n| n > work::MAX_ATTEMPT)
    {
        return Err("No room to reset the separate example; previous work retained".into());
    }
    let learning = dlms_learning_lab::one_off_teaching::Learning::new(lesson)?;
    a.workspace.as_mut().unwrap().one_off_learning = Some(learning);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atomic_leaf_packet_refuses_mixed_valid_and_overlong_fields_without_partial_publication() {
        let (s, mut a) = crate::ui::tests::one_off_admission::prepared();
        let original = serde_json::to_vec(&a).unwrap();
        let too_long = "x".repeat(2049);
        assert!(
            commit(
                &mut a,
                &s,
                false,
                Packet {
                    actor: Some("root"),
                    document: Some(("/work/job.sh", "echo changed\n")),
                    command: Some(&too_long),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(serde_json::to_vec(&a).unwrap(), original);
        commit(
            &mut a,
            &s,
            false,
            Packet {
                actor: Some("alex"),
                document: Some(("/work/job.sh", "echo revised\n")),
                command: Some("atq"),
                end: Some("invalid but retained UTC draft"),
                ..Default::default()
            },
        )
        .unwrap();
        let session = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_session
            .as_ref()
            .unwrap();
        assert_eq!(session.command, "atq");
        assert_eq!(session.window_end, "invalid but retained UTC draft");
        assert_eq!(session.drafts["/work/job.sh"], "echo revised\n");
        assert!(work::budget::PendingAttempt::from_draft(&a, &s, false).is_err());
    }
    #[test]
    fn checked_fresh_example_projection_refuses_malformed_shape_before_indexing_or_copy() {
        let (s, _) = crate::ui::tests::one_off_admission::prepared();
        let mut lesson = s.one_off_teaching.unwrap();
        assert!(work::budget::fresh_learning_size(&lesson).is_some());
        lesson.definition.editable.clear();
        assert!(work::budget::fresh_learning_size(&lesson).is_none());
    }
}
