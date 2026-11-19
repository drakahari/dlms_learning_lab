//! Native terminal and policy editor for independently authored Linux work orders.
use super::*;
use dlms_learning_lab::automation_workshop::{
    authenticated_transfer::{Client, KeyReference, KeyVersion, Policy},
    retirement_console::INPUT_LIMIT,
    transfer_editor::CONFIG_LIMIT,
};

fn update(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    *notice = work.apply(task, action).err();
}

pub(super) fn socket_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::SocketRetirement { supplied } = task else {
        return;
    };
    guide(ui, practice, 0);
    ui.strong("Supplied units");
    for (name, purpose) in [
        (&supplied.obsolete_service, "Obsolete service"),
        (&supplied.activating_socket, "Socket that can start it"),
        (&supplied.required_service, "Required status service"),
    ] {
        controls::form_row(ui, purpose, |ui, width| {
            controls::cell(ui, width, |ui| {
                lesson_presentation::monospace(ui, name, true)
            });
        });
    }
    ui.separator();
    guide(ui, practice, 1);
    let Scope::SocketRetirement { console } = &work.current else {
        return;
    };
    let typed = shared_libraries_page::fitting_input(
        ui,
        &console.draft,
        id.with("command"),
        INPUT_LIMIT,
        ui.is_enabled(),
        "Enter a supported systemctl command",
    );
    m.record(ui, "retirement-command", &typed.response);
    let enter =
        typed.response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    if let Some(text) = typed.value {
        update(task, work, Action::RetirementDraft { text }, notice);
    }
    if typed.rejected {
        ui.label("Type a single command within 512 bytes.");
    }
    let Scope::SocketRetirement { console } = &work.current else {
        return;
    };
    let run = ui.add_enabled(!console.draft.trim().is_empty(), button("Run command"));
    m.record(ui, "retirement-run", &run);
    if (run.clicked() || enter) && !console.draft.trim().is_empty() {
        let text = console.draft.clone();
        update(task, work, Action::RetirementCommand { text }, notice);
    }
    let Scope::SocketRetirement { console } = &work.current else {
        return;
    };
    ui.strong("Terminal output");
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(id.with("output"))
            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 7.)
            .stick_to_bottom(true),
        |ui| {
            if console.history.is_empty() {
                ui.label("Run a command or test a supplied request.");
            }
            for entry in &console.history {
                lesson_presentation::monospace(ui, &entry.input, true);
                if !entry.output.is_empty() {
                    lesson_presentation::monospace(ui, &entry.output, true);
                }
                ui.label(format!("Exit status {}", entry.status));
                ui.add_space(ui.spacing().item_spacing.y);
            }
        },
    );
    ui.separator();
    guide(ui, practice, 2);
    ui.strong("Test a supplied request");
    let actions = [
        (false, "Test now", "present"),
        (true, "Test after simulated startup", "boot"),
    ];
    let min_width = actions
        .iter()
        .map(|(_, label, _)| controls::button_width(ui, label))
        .sum::<f32>()
        + ui.spacing().item_spacing.x;
    for (required, title, key) in [
        (false, "Obsolete service", "obsolete"),
        (true, "Required service", "required"),
    ] {
        controls::form_row_group(ui, title, min_width, |ui, _| {
            ui.horizontal_wrapped(|ui| {
                for (next_start, label, suffix) in actions {
                    let response = ui.add(button(label));
                    m.record(ui, &format!("retirement-{key}-{suffix}"), &response);
                    if response.clicked() {
                        update(
                            task,
                            work,
                            Action::RetirementProbe {
                                required,
                                next_start,
                            },
                            notice,
                        );
                    }
                }
            });
        });
    }
    if let Scope::SocketRetirement { console } = &work.current
        && let Some(entry) = console.history.last()
        && entry.input.starts_with("Test ")
    {
        lesson_presentation::prose(ui, &format!("Actual request · {}", entry.output));
    }
    if let Some(error) = notice.as_ref() {
        lesson_presentation::prose(ui, error);
    }
}

fn key_label(reference: &KeyReference) -> String {
    let state = match reference.version {
        KeyVersion::ApprovedActive => "Current",
        KeyVersion::ApprovedStandby => "Current spare",
        KeyVersion::Superseded => "Outdated",
    };
    format!("{} · {state}", reference.name)
}

fn describe_policy(value: &Policy) -> String {
    match value {
        Policy::Deny => "Transfers denied".into(),
        Policy::SourceAddress => "Source address only".into(),
        Policy::VerifiedKey { reference } => format!("Verified key · {}", reference.name),
    }
}

pub(super) fn dns_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::KeyedTransfer { supplied } = task else {
        return;
    };
    guide(ui, practice, 0);
    ui.strong("Transfer trials using the applied rule");
    ui.horizontal_wrapped(|ui| {
        for (label, key, client, protected) in [
            (
                "Test unsigned request",
                "dns-unsigned",
                Client::UnsignedAtSecondaryAddress,
                false,
            ),
            (
                "Test outdated proof",
                "dns-outdated",
                Client::OldProofAtSecondaryAddress,
                false,
            ),
            (
                "Test protected zone",
                "dns-protected",
                Client::ConfiguredSecondary,
                true,
            ),
        ] {
            let response = ui.add(button(label));
            m.record(ui, key, &response);
            if response.clicked() {
                update(
                    task,
                    work,
                    Action::RunTransfer { client, protected },
                    notice,
                );
            }
        }
    });
    transfer_observation(ui, work, true);
    ui.separator();
    guide(ui, practice, 1);
    ui.strong("Proposed transfer rules");
    for (protected, label, key) in [
        (false, "Served zone rule", "dns-rule"),
        (true, "Protected zone rule", "dns-other-rule"),
    ] {
        let Scope::KeyedTransfer { edit } = &work.current else {
            return;
        };
        let prior = if protected {
            &edit.protected_fragment
        } else {
            &edit.served_fragment
        };
        let mut value = None;
        controls::form_row(ui, label, |ui, _| {
            let typed = shared_libraries_page::fitting_input(
                ui,
                prior,
                id.with(key),
                CONFIG_LIMIT,
                ui.is_enabled(),
                "allow-transfer statement",
            );
            m.record(ui, key, &typed.response);
            value = typed.value;
        });
        if let Some(text) = value {
            update(
                task,
                work,
                Action::ChangeTransferRule { protected, text },
                notice,
            );
        }
    }
    let Scope::KeyedTransfer { edit } = &work.current else {
        return;
    };
    let before = edit.selected_key.clone();
    let mut reference = before.clone();
    let options = std::iter::once((None, "No key selected".into()))
        .chain(
            supplied
                .approved
                .iter()
                .chain(std::iter::once(&supplied.stale))
                .map(|key| (Some(key.clone()), key_label(key))),
        )
        .collect::<Vec<_>>();
    select(
        ui,
        id,
        "dns-key",
        "Secondary's key",
        &mut reference,
        &options,
        m,
    );
    if reference != before {
        update(task, work, Action::SelectTransferKey { reference }, notice);
    }
    let apply = ui.add(button("Apply policy"));
    m.record(ui, "dns-apply", &apply);
    if apply.clicked() {
        update(task, work, Action::ApplyTransferPolicy, notice);
    }
    if let Some(error) = notice.as_ref() {
        lesson_presentation::prose(ui, error);
    }
    let Scope::KeyedTransfer { edit } = &work.current else {
        return;
    };
    controls::form_row(ui, "Applied served policy", |ui, width| {
        controls::label(ui, describe_policy(&edit.actual.applied.primary), width);
    });
    ui.separator();
    guide(ui, practice, 2);
    controls::form_row(ui, "Zone copies", |ui, width| {
        controls::label(
            ui,
            format!(
                "Primary serial {} · Secondary serial {}",
                edit.actual.copies.primary.serial, edit.actual.copies.secondary.serial
            ),
            width,
        );
    });
    ui.horizontal_wrapped(|ui| {
        let refresh = ui.add(button("Refresh secondary"));
        m.record(ui, "dns-refresh", &refresh);
        if refresh.clicked() {
            update(
                task,
                work,
                Action::RunTransfer {
                    client: Client::ConfiguredSecondary,
                    protected: false,
                },
                notice,
            );
        }
        for (secondary, label, key) in [
            (true, "Lookup secondary", "dns-secondary"),
            (false, "Public lookup", "dns-public"),
        ] {
            let lookup = ui.add(button(label));
            m.record(ui, key, &lookup);
            if lookup.clicked() {
                update(task, work, Action::TransferLookup { secondary }, notice);
            }
        }
    });
    transfer_observation(ui, work, false);
}

fn transfer_observation(ui: &mut egui::Ui, work: &Work, is_trial: bool) {
    if let Scope::KeyedTransfer { edit } = &work.current
        && let Some(observation) = &edit.observation
    {
        let transfer_trial = observation.starts_with("Unsigned request.")
            || observation.starts_with("Outdated key proof.")
            || observation.starts_with("Protected zone.");
        if transfer_trial == is_trial {
            lesson_presentation::prose(ui, &format!("Actual result · {observation}"));
        }
    }
}
