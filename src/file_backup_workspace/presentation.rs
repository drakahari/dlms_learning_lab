//! Borrowed, transactional native presentation setters.
use crate::{
    file_backup_workspace as work, pack::Scenario, progress::Attempt, script_teaching::View,
};
#[derive(Default)]
pub struct Edit<'a> {
    pub view: Option<View>,
    pub example_command: Option<usize>,
    pub command: Option<&'a str>,
    pub checked: Option<bool>,
}
/// UI-only fields confer no credit. Perform no whole-Attempt clone for typing.
/// Admit conservatively before the first mutation; any refusal retains both drafts.
pub fn present(
    a: &mut Attempt,
    s: &Scenario,
    practice: bool,
    edit: Edit<'_>,
) -> Result<(), String> {
    work::context(a, s)?;
    let lesson = s
        .file_backup_teaching
        .as_ref()
        .ok_or("Backup introduction is missing")?;
    if edit
        .command
        .is_some_and(|v| v.len() > 4096 || v.contains(['\0', '\n', '\r']))
    {
        return Err(
            "Use one command line of at most4096bytes; your earlier input is retained".into(),
        );
    }
    if edit
        .example_command
        .is_some_and(|n| n >= lesson.example_commands.len())
    {
        return Err("Choose an available example command".into());
    }
    let w = a
        .workspace
        .as_ref()
        .ok_or("Open this lesson before editing its command")?;
    let learning = w
        .file_backup_learning
        .as_ref()
        .ok_or("Open this lesson's separate example first")?;
    let command = if practice {
        &learning.command
    } else {
        &w.command
    };
    let no_change = edit.command.is_none_or(|v| v == command)
        && edit.view.is_none_or(|v| v == learning.view)
        && edit
            .example_command
            .is_none_or(|n| n == learning.example_command)
        && edit.checked.is_none_or(|v| {
            v == if practice {
                learning.checked
            } else {
                w.checked
            }
        });
    if no_change {
        return Ok(());
    }
    // Exact borrowed replacement accounting lets a full workspace shrink or
    // navigate. All fields already exist; no unrelated value is serialized to a
    // Vec or cloned. Source/schema changes still require explicit decisions.
    fn replaced<A: serde::Serialize, B: serde::Serialize>(
        n: usize,
        old: A,
        new: B,
    ) -> Result<usize, String> {
        work::size(&old, 26 * 1024)
            .and_then(|old_n| n.checked_sub(old_n))
            .and_then(|n| work::size(&new, 26 * 1024).and_then(|new_n| n.checked_add(new_n)))
            .ok_or_else(|| "Backup presentation encoded-size bound".into())
    }
    let mut proposed = work::size(a, work::MAX_ATTEMPT).ok_or("Saved-work size bound")?;
    if let Some(value) = edit.command {
        proposed = replaced(proposed, command.as_str(), value)?;
    }
    if let Some(value) = edit.view {
        proposed = replaced(proposed, learning.view, value)?;
    }
    if let Some(value) = edit.example_command {
        proposed = replaced(proposed, learning.example_command, value)?;
    }
    let old_checked = if practice {
        learning.checked
    } else {
        w.checked
    };
    let new_checked = edit.checked.unwrap_or(if edit.command.is_some() {
        false
    } else {
        old_checked
    });
    proposed = replaced(proposed, old_checked, new_checked)?;
    if proposed > work::MAX_ATTEMPT {
        return Err(
            "This edit would exceed the saved-work allowance; your earlier draft is retained"
                .into(),
        );
    }
    let w = a.workspace.as_mut().unwrap();
    let learning = w.file_backup_learning.as_mut().unwrap();
    if let Some(view) = edit.view {
        learning.view = view;
    }
    if let Some(n) = edit.example_command {
        learning.example_command = n;
    }
    if let Some(value) = edit.command {
        if practice {
            learning.command = value.into();
            learning.checked = false;
        } else {
            w.command = value.into();
            w.checked = false;
        }
    }
    if let Some(checked) = edit.checked {
        if practice {
            learning.checked = checked;
        } else {
            w.checked = checked;
        }
    }
    Ok(())
}

fn replace_scalar<A: serde::Serialize, B: serde::Serialize>(
    a: &Attempt,
    old: A,
    new: B,
) -> Result<(), String> {
    let next = work::size(a, work::MAX_ATTEMPT)
        .and_then(|n| work::size(&old, 128).and_then(|v| n.checked_sub(v)))
        .and_then(|n| work::size(&new, 128).and_then(|v| n.checked_add(v)));
    if next.is_some_and(|n| n <= work::MAX_ATTEMPT) {
        Ok(())
    } else {
        Err("This navigation change exceeds the saved-work allowance; your work is retained".into())
    }
}
pub fn record_mode(a: &mut Attempt, s: &Scenario, challenge: bool) -> Result<(), String> {
    work::context(a, s)?;
    if a.challenge == challenge {
        return Ok(());
    }
    replace_scalar(a, a.challenge, challenge)?;
    a.challenge = challenge;
    Ok(())
}
pub fn record_visit(a: &mut Attempt, s: &Scenario, clock: u64) -> Result<(), String> {
    work::context(a, s)?;
    if clock < a.visited {
        return Err("Saved visit order differs; your work is retained".into());
    }
    replace_scalar(a, a.visited, clock)?;
    a.visited = clock;
    Ok(())
}
