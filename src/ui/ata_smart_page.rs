//! Native disk diagnosis and monitoring configuration over supplied virtual data.
use super::*;
use dlms_learning_lab::{
    ata_smart::{
        Layout, command, monitor,
        source::{self, Action},
        workspace::{Assignment, Disposition, Task, Work},
    },
    engine::State,
};

type Metrics = super::linux_v6_page::Metrics;

fn apply(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = source::apply(task, work, action) {
        *notice = Some(error);
    }
}

fn supplied(ui: &mut egui::Ui, task: &Task) {
    match &task.goal {
        Assignment::Diagnostic(goal) => {
            controls::form_row(ui, "Ticket serial", |ui, _| {
                lesson_presentation::monospace(ui, &goal.primary_serial, true)
            });
            let names = task
                .inventory
                .disks
                .iter()
                .map(|d| d.current_name.as_str())
                .collect::<Vec<_>>()
                .join("  ");
            controls::form_row(ui, "Available device paths", |ui, _| {
                lesson_presentation::monospace(ui, &names, true)
            });
        }
        Assignment::Monitoring(goal) => {
            for (role, serial) in [
                ("Primary disk", &goal.primary_serial),
                ("Auxiliary disk", &goal.auxiliary_serial),
            ] {
                controls::form_row(ui, role, |ui, _| {
                    lesson_presentation::monospace(ui, serial, true)
                });
            }
            controls::collapsing_header(ui, "Supplied device names and stable paths").show(
                ui,
                |ui| {
                    for disk in &task.inventory.disks {
                        ui.strong(&disk.serial);
                        for (label, value) in [
                            ("Current name", &disk.current_name),
                            ("Maintenance name", &disk.maintenance_name),
                            ("Stable path", &disk.stable),
                        ] {
                            controls::form_row(ui, label, |ui, _| {
                                lesson_presentation::monospace(ui, value, true)
                            });
                        }
                    }
                },
            );
        }
    }
}

fn console(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("ATA diagnostic console");
    let input = shared_libraries_page::fitting_input(
        ui,
        &work.command,
        id.with("command"),
        command::INPUT_LIMIT,
        ui.is_enabled(),
        "Enter a supported smartctl command",
    );
    metrics.record(ui, "command", &input.response);
    metrics
        .fields
        .push(("command".into(), input.response.rect, input.response.id));
    if let Some(text) = input.value {
        work.command = text;
    }
    if input.rejected {
        ui.label("Keep commands within 1,024 bytes. Your previous text was retained.");
    }
    let enter = input.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    ui.horizontal_wrapped(|ui| {
        let run = ui.add_enabled(!work.command.trim().is_empty(), button("Run"));
        metrics.record(ui, "run", &run);
        if run.clicked() || ui.is_enabled() && enter {
            let input_text = work.command.clone();
            apply(task, work, Action::Run { input: input_text }, notice);
            input.response.request_focus();
        }
        let advance = ui.add_enabled(
            work.state
                .disks
                .iter()
                .any(|d| d.remaining_minutes.is_some()),
            button("Advance virtual time by 1 minute"),
        );
        metrics.record(ui, "advance", &advance);
        if advance.clicked() {
            apply(task, work, Action::Advance, notice);
        }
    });
    if work.transcript.is_empty() {
        ui.label("No commands have run.");
        return;
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 9.;
    let output = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(id.with("output"))
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(true),
        |ui| {
            for row in &work.transcript {
                let clock = row.command == "Advance virtual time";
                if clock {
                    ui.strong("Virtual clock");
                } else {
                    lesson_presentation::monospace(ui, &format!("$ {}", row.command), true);
                    if let Some(status) = row.status {
                        ui.label(format!("Actual result · exit status {status}"));
                    } else {
                        ui.strong("Command could not run");
                    }
                }
                lesson_presentation::monospace(ui, &row.output, true);
            }
        },
    );
    let r = ui.interact(
        output.inner_rect,
        id.with("output-extent"),
        egui::Sense::hover(),
    );
    metrics.record(ui, "console-output", &r);
}

fn disposition(value: Disposition) -> &'static str {
    match value {
        Disposition::Unset => "Select observed result",
        Disposition::Completed => "Completed without error",
        Disposition::ReadFailure => "Completed with a read failure",
    }
}

fn ticket(
    ui: &mut egui::Ui,
    id: egui::Id,
    t: &Task,
    w: &mut Work,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Maintenance ticket");
    let mut draft = w.ticket.clone();
    controls::form_row(ui, "Latest short test", |ui, width| {
        let response = controls::dropdown(ui, id.with("result"), width)
            .selected_text(disposition(draft.disposition))
            .show_ui(ui, |ui| {
                for (value, key) in [
                    (Disposition::Unset, "ticket-unset"),
                    (Disposition::Completed, "ticket-completed"),
                    (Disposition::ReadFailure, "ticket-read-failure"),
                ] {
                    let response =
                        ui.selectable_value(&mut draft.disposition, value, disposition(value));
                    m.record(ui, key, &response);
                }
            })
            .response;
        controls::retain_dropdown_focus(ui, &response);
        m.record(ui, "disposition", &response);
    });
    if draft.disposition == Disposition::ReadFailure {
        controls::form_row(ui, "First-error LBA", |ui, _| {
            let input = shared_libraries_page::fitting_input(
                ui,
                &draft.first_lba,
                id.with("lba"),
                24,
                ui.is_enabled(),
                "Decimal block address",
            );
            m.record(ui, "lba", &input.response);
            m.fields
                .push(("lba".into(), input.response.rect, input.response.id));
            if let Some(text) = input.value {
                draft.first_lba = text;
            }
        });
    } else {
        draft.first_lba.clear();
    }
    if draft != w.ticket {
        apply(t, w, Action::Record { ticket: draft }, notice);
    }
}

fn config(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Virtual smartd.conf");
    let r = shared_libraries_page::configuration_input(
        ui,
        &work.monitor_draft,
        id.with("config"),
        monitor::LIMIT,
        ui.is_enabled(),
        "One rule per supplied device",
    );
    metrics.record(ui, "config", &r.response);
    metrics
        .fields
        .push(("config".into(), r.response.rect, r.response.id));
    if let Some(contents) = r.value {
        apply(task, work, Action::Draft { contents }, notice);
    }
    if r.rejected {
        ui.label("The last edit exceeded the configuration limit. Your earlier draft was kept.");
    }
    let save = ui.add(button("Save configuration"));
    metrics.record(ui, "save", &save);
    if save.clicked() {
        apply(task, work, Action::Save, notice);
    }
    ui.label(if work.monitor_draft == work.monitor_saved {
        "Saved · both previews use this configuration"
    } else {
        "Draft differs · previews still use the saved file"
    });
}

fn preview(ui: &mut egui::Ui, id: egui::Id, task: &Task, work: &Work, m: &mut Metrics) {
    ui.strong("Monitoring preview");
    let store = id.with("preview-layout");
    let mut layout = ui
        .data(|d| d.get_temp::<Layout>(store))
        .unwrap_or(Layout::Current);
    ui.horizontal_wrapped(|ui| {
        for (choice, key, label) in [
            (Layout::Current, "current-layout", "Current layout"),
            (
                Layout::Maintenance,
                "maintenance-layout",
                "After maintenance",
            ),
        ] {
            let r = ui.add(button(label).selected(layout == choice));
            m.record(ui, key, &r);
            if r.clicked() {
                layout = choice;
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(store, layout));
    if let Ok(rules) = monitor::parse(&work.monitor_saved) {
        if rules.is_empty() {
            ui.label("This file selects no device.");
        }
        for watched in monitor::preview(&rules, &task.inventory, layout) {
            lesson_presentation::monospace(ui, &watched.rule.selector, true);
            ui.label(
                watched
                    .disk
                    .map_or("No supplied disk has this name here.", |d| {
                        d.serial.as_str()
                    }),
            );
            ui.label(match (watched.rule.health, watched.rule.selftest) {
                (true, true) => "Watches health and the self-test log",
                (true, false) => "Watches health",
                (false, true) => "Watches the self-test log",
                _ => "No checks",
            });
        }
    }
}

fn results(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &Work,
    example: bool,
    m: &mut Metrics,
) -> bool {
    let outcomes = work.checks(task);
    let passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    let solved = passed == outcomes.len();
    let headline = ui.strong(if !work.checked {
        "Not checked".into()
    } else if solved && example {
        "Example verified".into()
    } else if solved {
        "Assignment complete".into()
    } else {
        format!("Needs work · {passed}/{} requirements met", outcomes.len())
    });
    m.record(ui, "result", &headline);
    m.result = Some((passed, outcomes.len(), headline.rect));
    if !work.checked {
        return false;
    }
    for outcome in outcomes.iter().filter(|o| !o.passed) {
        ui.strong(format!("Needs work · {}", outcome.label));
        lesson_presentation::prose(ui, &outcome.detail);
    }
    controls::collapsing_header(
        ui,
        if solved {
            "Objective results"
        } else {
            "Requirements already met"
        },
    )
    .id_salt(id.with("details"))
    .show(ui, |ui| {
        for outcome in outcomes.iter().filter(|o| o.passed) {
            ui.strong(format!("Passed · {}", outcome.label));
            lesson_presentation::prose(ui, &outcome.detail);
        }
    });
    solved
}

impl LabApp {
    pub(super) fn ata_smart_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let original = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario) {
            Ok(d) => d,
            Err(error) => {
                lesson_presentation::prose(ui, &error);
                return;
            }
        };
        if let Err(error) = source::context(original, &scenario) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::AtaSmart { simulation } = &original.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        let metrics = &mut self.metrics.ata_smart;
        lesson_presentation::rich(ui, RichText::new(&scenario.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (candidate, name, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == candidate));
                    metrics.record(ui, name, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = candidate;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            metrics.record(ui, "introduction-content", &intro.objective);
            lesson_presentation::worked(ui, "Example with different virtual disks");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            metrics.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            metrics.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (task, work) = if practice {
                (&definition.example, &mut next.example.work)
            } else {
                (&definition.assignment, &mut next.assignment.work)
            };
            let id = egui::Id::new(("ata-smart", &scenario.id, practice));
            if practice {
                lesson_presentation::practice_step(ui, &scenario, 0);
            } else {
                let assignment = lesson_presentation::assignment_responses(ui, &scenario);
                metrics.record(ui, "assignment-content", &assignment.goal);
            }
            supplied(ui, task);
            ui.separator();
            if practice {
                lesson_presentation::practice_step_commands(ui, &scenario, 1);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                match task.goal {
                    Assignment::Diagnostic(_) => {
                        console(ui, id, task, work, metrics, &mut self.notice);
                        ui.separator();
                        if practice {
                            lesson_presentation::practice_step(ui, &scenario, 2);
                        }
                        ticket(ui, id, task, work, metrics, &mut self.notice);
                    }
                    Assignment::Monitoring(_) => {
                        config(ui, id, task, work, metrics, &mut self.notice);
                        ui.separator();
                        if practice {
                            lesson_presentation::practice_step(ui, &scenario, 2);
                        }
                        preview(ui, id, task, work, metrics);
                    }
                }
                ui.separator();
                let check_label = match task.goal {
                    Assignment::Diagnostic(_) => "Check diagnosis",
                    Assignment::Monitoring(_) => "Check configuration",
                };
                let check = ui.add(primary(ui, check_label));
                metrics.record(ui, "check", &check);
                if check.clicked() {
                    apply(task, work, Action::Check, &mut self.notice);
                }
                if results(ui, id, task, work, practice, metrics) && !practice {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = ui.add(button("Reset example"));
                    metrics.record(ui, "reset-example", &r);
                    reset = r.clicked();
                }
            });
            if reset && let Err(error) = next.reset_example(definition) {
                self.notice = Some(error);
            }
            controls::collapsing_header(ui, "Supported operations, hints and references").id_salt(id.with("help")).show(ui, |ui| {
                match task.goal {
                    Assignment::Diagnostic(_) => { lesson_presentation::monospace(ui, command::HELP, true); }
                    Assignment::Monitoring(_) => {
                        lesson_presentation::prose(ui, "One supplied device path per line, followed by -H, -l selftest, or both. Use # for comments.");
                    }
                }
                if ui.add_enabled(active, button("Show a hint")).clicked() {
                    let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                    a.hints = (a.hints + 1).min(scenario.hints.len());
                    self.dirty = true;
                }
                for hint in scenario.hints.iter().take(self.progress.attempts[&scenario.id].hints) {
                    lesson_presentation::prose(ui, hint);
                }
                lesson_presentation::prose(ui, &scenario.simplifications);
                for url in &scenario.sources {
                    ui.hyperlink_to(url, url);
                }
            });
        }
        if active && next != before {
            match next.bind(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state = State::AtaSmart {
                        simulation: Box::new(next),
                    };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(error) => self.notice = Some(error),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&scenario.id], &scenario);
        if self.metrics.completion {
            self.next_unit_lab(ui, index);
        }
        Self::workspace_footer(ui);
    }
}
