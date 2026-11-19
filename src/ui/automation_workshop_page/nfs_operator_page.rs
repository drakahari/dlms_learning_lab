//! Edit the virtual policy, re-export it in the bounded terminal, and test a client.
use super::*;
use dlms_learning_lab::{
    automation_workshop::nfs_tools::workspace::{INPUT_LIMIT, Origin},
    nfs,
};

pub(super) fn body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (Task::NfsExportRetirement { supplied }, Scope::NfsExportRetirement { terminal }) =
        (task, &work.current)
    else {
        return;
    };
    guide(ui, practice, 0);
    let server = &terminal.state.inventory().servers[0];
    controls::form_row(ui, "Supplied NFSv3 server", |ui, width| {
        controls::label(
            ui,
            RichText::new(server.address.to_string()).monospace(),
            width,
        );
    });
    let layers = &terminal.state.runtime().servers[0];
    ui.strong("Server file · /etc/exports draft");
    let draft = shared_libraries_page::configuration_input(
        ui,
        &layers.draft,
        id.with("nfs-exports"),
        nfs::MAX_DOCUMENT,
        ui.is_enabled(),
        "Export path and client options",
    );
    m.controls
        .push(("nfs-exports".into(), draft.viewport, draft.clip));
    m.focus.push(("nfs-exports".into(), draft.response.id));
    if let Some(text) = draft.value {
        apply(task, work, Action::NfsExportDraft { text }, notice);
    }
    if draft.rejected {
        ui.label("The earlier draft was kept. Shorten the export document before trying again.");
    }
    let save = ui.add(button("Save export file"));
    m.record(ui, "nfs-save", &save);
    if save.clicked() {
        apply(task, work, Action::NfsSaveExports, notice);
    }
    let Scope::NfsExportRetirement { terminal } = &work.current else {
        return;
    };
    let layers = &terminal.state.runtime().servers[0];
    if layers.draft == layers.saved {
        ui.label("The displayed export draft is saved.");
    } else {
        ui.label("The draft has unsaved edits.");
    }
    ui.separator();
    guide(ui, practice, 1);
    ui.strong("Server and client query terminal");
    let input = shared_libraries_page::fitting_input(
        ui,
        &terminal.draft,
        id.with("nfs-command"),
        INPUT_LIMIT,
        ui.is_enabled(),
        "Supported NFS command",
    );
    m.record(ui, "nfs-command", &input.response);
    let enter =
        input.response.lost_focus() && ui.input(|events| events.key_pressed(egui::Key::Enter));
    if let Some(text) = input.value {
        apply(task, work, Action::NfsTerminalDraft { text }, notice);
    }
    let Scope::NfsExportRetirement { terminal } = &work.current else {
        return;
    };
    let command = terminal.draft.trim();
    let run = ui.add_enabled(!command.is_empty(), button("Run command"));
    m.record(ui, "nfs-run", &run);
    if !command.is_empty() && (enter || run.clicked()) {
        apply(
            task,
            work,
            Action::NfsTerminalRun {
                text: command.to_owned(),
            },
            notice,
        );
    }
    controls::collapsing_header(ui, "Supported NFS commands")
        .id_salt(id.with("nfs-commands-help"))
        .show(ui, |ui| {
            for help in [
                "exportfs -v · inspect the applied export options",
                "exportfs -r or -ra · re-export the saved virtual file",
                "showmount -e <supplied IPv4 address> · list the advertised NFSv3 exports",
            ] {
                lesson_presentation::monospace(ui, help, true);
            }
        });
    let Scope::NfsExportRetirement { terminal } = &work.current else {
        return;
    };
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(id.with("nfs-output"))
            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 8.)
            .auto_shrink([false, true])
            .stick_to_bottom(true),
        |ui| {
            if !terminal
                .history
                .iter()
                .any(|entry| entry.origin == Origin::Command)
            {
                ui.label("Run a supported command to see its results.");
            }
            for entry in terminal
                .history
                .iter()
                .filter(|entry| entry.origin == Origin::Command)
            {
                lesson_presentation::monospace(ui, &entry.request, true);
                if entry.reply.is_empty() {
                    ui.label("Command completed.");
                } else {
                    ui.label(if entry.success {
                        "Result"
                    } else {
                        "Needs attention"
                    });
                    lesson_presentation::monospace(ui, &entry.reply, true);
                }
            }
        },
    );
    ui.separator();
    guide(ui, practice, 2);
    for (index, target) in supplied.clients.iter().enumerate() {
        let title = if index == 0 {
            "Retiring share"
        } else {
            "Approved reference share"
        };
        controls::form_row(ui, title, |ui, width| {
            controls::label(ui, RichText::new(&target.mountpoint).monospace(), width)
        });
        ui.horizontal_wrapped(|ui| {
            if index == 0 {
                let unmount = ui.add(button("Unmount retired client"));
                m.record(ui, "nfs-unmount", &unmount);
                if unmount.clicked() {
                    apply(task, work, Action::NfsUnmountRetired, notice);
                }
            }
            let probe = ui.add(button(if index == 0 {
                "Test retired mount"
            } else {
                "Mount approved share"
            }));
            m.record(ui, &format!("nfs-mount-{index}"), &probe);
            if probe.clicked() {
                apply(task, work, Action::NfsMount { index }, notice);
            }
            if index == 1 {
                let read = ui.add(button("Read reference file"));
                m.record(ui, "nfs-read", &read);
                if read.clicked() {
                    apply(task, work, Action::NfsRead { index }, notice);
                }
            }
        });
    }
    if let Scope::NfsExportRetirement { terminal } = &work.current
        && let Some(result) = terminal.history.last()
        && result.origin == Origin::Client
    {
        ui.strong(&result.request);
        lesson_presentation::prose(ui, &result.reply);
    }
    if let Some(error) = notice {
        lesson_presentation::prose(ui, error);
    }
}
