//! Temporary edits to a supplied GRUB2 entry for one simulated boot.

use super::{Menu, Result, boot, error, lines, load_linux, string};
use crate::boot_entry::{ErrorKind, Report, Stage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootComparison {
    pub edited: Report,
    pub ordinary: Report,
}

/// Test a temporary linux line, then start from the unmodified menu again.
pub fn test(
    inventory: &boot::Inventory,
    saved_source: &str,
    selected_id: &str,
    command: &str,
) -> Result<BootComparison> {
    let menu = Menu::parse(saved_source)?;
    let selected = menu
        .items
        .iter()
        .find(|item| item.id == selected_id)
        .ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                "Choose an entry from the supplied GRUB menu.",
            )
        })?;
    if command.contains(['\r', '\n']) {
        return Err(error(
            ErrorKind::Unsupported,
            "Edit the single linux line. Other menu commands are outside this exercise.",
        ));
    }
    let tokens = lines(command)?;
    let [line] = tokens.as_slice() else {
        return Err(error(
            ErrorKind::Syntax,
            "Enter the selected entry's linux line.",
        ));
    };
    let (verb, args) = line
        .split_first()
        .ok_or_else(|| error(ErrorKind::Syntax, "Enter the selected entry's linux line."))?;
    if string(verb)? != "linux" {
        return Err(error(
            ErrorKind::Unsupported,
            "Only the linux line is editable here. The initrd line and saved menu stay unchanged.",
        ));
    }
    let args = args.iter().map(string).collect::<Result<Vec<_>>>()?;
    let mut edit = selected.entry.clone();
    load_linux(&mut edit, &args)?;
    let mut edited = boot::trace_entry(inventory, &edit)?;
    edited.stages.insert(
        0,
        Stage {
            title: "Temporary menu selection",
            passed: true,
            detail: format!(
                "Boot once with the edited linux line for {:?}.",
                selected.title
            ),
        },
    );

    let default = menu.selected()?;
    let mut ordinary = boot::trace_entry(inventory, &default.entry)?;
    ordinary.stages.insert(
        0,
        Stage {
            title: "Saved menu selection",
            passed: true,
            detail: format!(
                "A later ordinary boot uses the saved entry {:?}.",
                default.title
            ),
        },
    );
    Ok(BootComparison { edited, ordinary })
}
