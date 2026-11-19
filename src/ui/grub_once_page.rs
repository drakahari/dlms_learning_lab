//! Native workspace for editing one GRUB linux line without changing the saved menu.
use super::*;
use dlms_learning_lab::{grub_menu::Menu, grub_once_workspace as work};

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub result: Option<bool>,
}

fn record(metrics: &mut Metrics, key: &str, response: &egui::Response, ui: &egui::Ui) {
    metrics
        .controls
        .push((key.into(), response.rect, ui.clip_rect()));
}

fn facts(ui: &mut egui::Ui, task: &work::Task, menu: &Menu) {
    ui.strong("Supplied GRUB menu");
    if let Ok(item) = menu.selected() {
        lesson_presentation::prose(ui, &format!("Saved default is {}.", item.title));
    }
    for item in &menu.items {
        let release = task
            .inventory
            .kernels
            .iter()
            .find(|image| image.path == item.entry.linux)
            .map_or("missing", |image| image.release.as_str());
        ui.label(&item.title);
        lesson_presentation::monospace(
            ui,
            &format!("{} · kernel {release}", item.entry.linux),
            true,
        );
    }
    ui.add_space(controls::GAP);
    ui.strong("Supplied root filesystems");
    for root in &task.inventory.roots {
        ui.label(&root.label);
        lesson_presentation::monospace(ui, &root.uuid, true);
    }
    controls::collapsing_header(ui, "Read saved GRUB menu · optional").show(ui, |ui| {
        lesson_presentation::monospace(ui, &task.saved_menu, true);
    });
}

fn edit(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::Draft,
    menu: &Menu,
    focus: &mut bool,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) -> (egui::Rect, egui::Id, egui::Rect) {
    let label = ui.strong("Select an entry to edit for this boot");
    let mut choice = draft.input.entry_id.clone();
    let title = menu
        .items
        .iter()
        .find(|item| item.id == choice)
        .map_or(choice.as_str(), |item| item.title.as_str())
        .to_owned();
    let choice_response = egui::ComboBox::from_id_salt("temporary-menu-entry")
        .selected_text(title)
        .show_ui(ui, |ui| {
            for item in &menu.items {
                let option = ui.selectable_value(&mut choice, item.id.clone(), &item.title);
                record(metrics, &format!("menu-{}", item.id), &option, ui);
            }
        })
        .response
        .labelled_by(label.id);
    record(metrics, "choice", &choice_response, ui);
    if choice != draft.input.entry_id {
        *notice = task
            .select(&choice)
            .and_then(|input| draft.edit(input))
            .err();
    }

    let label = ui.strong("Temporary linux line");
    lesson_presentation::prose(
        ui,
        "Choosing another entry loads its saved line. The saved menu stays unchanged.",
    );
    let mut input = draft.input.clone();
    let field = ui
        .add(
            egui::TextEdit::multiline(&mut input.linux)
                .font(egui::TextStyle::Monospace)
                .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                .desired_width((ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.))
                .desired_rows(3)
                .return_key(None)
                .char_limit(dlms_learning_lab::boot_entry::MAX_SOURCE)
                .hint_text("linux and its kernel arguments"),
        )
        .labelled_by(label.id);
    record(metrics, "input", &field, ui);
    if *focus && ui.is_enabled() {
        field.request_focus();
        *focus = false;
    }
    if field.changed() {
        *notice = draft.edit(input).err();
    }
    let action = ui
        .horizontal_wrapped(|ui| {
            let boot = ui.add(button("Boot once · simulation"));
            record(metrics, "test", &boot, ui);
            if boot.clicked() {
                *notice = draft.test(task).err();
            }
            let discard = ui.add(button("Discard edit"));
            record(metrics, "discard", &discard, ui);
            if discard.clicked() {
                *notice = task
                    .select(&draft.input.entry_id)
                    .and_then(|input| draft.edit(input))
                    .err();
                draft.observed = None;
            }
            boot.rect
        })
        .inner;
    (field.rect, field.id, action)
}

fn outcomes(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &work::Draft,
    practice: bool,
    metrics: &mut Metrics,
) {
    let result = match draft.report(task) {
        None => {
            ui.label("Current edit has not been tested.");
            return;
        }
        Some(Err(error)) => {
            metrics.result = Some(false);
            let status = ui.strong("Fix the linux line and try again.");
            record(metrics, "result", &status, ui);
            lesson_presentation::prose(ui, &error);
            return;
        }
        Some(Ok(report)) => report,
    };
    let checks = task.checks(&result);
    let all = checks.iter().all(|row| row.0);
    metrics.result = Some(all);
    let status = ui.strong(match (all, practice) {
        (true, false) => "Assignment complete",
        (true, true) => "Practice passed",
        _ => "Recovery requirements not met yet",
    });
    record(metrics, "result", &status, ui);
    for (label, report) in [
        ("Edited boot", &result.edited),
        ("Later ordinary boot", &result.ordinary),
    ] {
        ui.strong(label);
        if let Some(target) = &report.target {
            lesson_presentation::prose(ui, &format!("Reached Linux init and requested {target}."));
        } else if let Some(stage) = report.stages.last() {
            lesson_presentation::prose(ui, &format!("{} · {}", stage.title, stage.detail));
        }
    }
    if result.edited.passed() {
        for (passed, detail) in checks.into_iter().skip(1) {
            if !passed {
                lesson_presentation::prose(ui, &detail);
            }
        }
    }
    controls::collapsing_header(ui, "See boot stages · optional").show(ui, |ui| {
        for (label, report) in [
            ("Edited boot", &result.edited),
            ("Later ordinary boot", &result.ordinary),
        ] {
            ui.strong(label);
            for stage in &report.stages {
                let verdict = if stage.passed { "Reached" } else { "Blocked" };
                lesson_presentation::prose(
                    ui,
                    &format!("{verdict} · {} · {}", stage.title, stage.detail),
                );
            }
        }
    });
}

fn practice_step(draft: &work::Draft, task: &work::Task) -> usize {
    if draft.input.entry_id != task.reference.entry_id {
        return 0;
    }
    match draft.report(task) {
        Some(Ok(result)) if result.edited.target.as_deref() == Some("rescue.target") => 3,
        Some(Ok(result)) if result.edited.passed() => 2,
        _ => 1,
    }
}

impl LabApp {
    pub(super) fn grub_once_page(&mut self, ui: &mut egui::Ui, index: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&source.id];
        let Ok(definition) = work::source(&source)
            .and_then(|definition| work::context(attempt, &source).map(|()| definition))
        else {
            self.notice = Some("Could not open this saved GRUB workspace.".into());
            return;
        };
        let engine::State::GrubOnce { simulation } = &attempt.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        self.metrics.grub_once = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let tab = ui.add_enabled(active, button(label).selected(choice == view));
                    let key = match choice {
                        teaching::View::Introduction => "introduction",
                        teaching::View::Example => "example",
                        teaching::View::Assignment => "assignment",
                    };
                    record(&mut self.metrics.grub_once, key, &tab, ui);
                    self.metrics.lesson_tabs.push(tab.rect);
                    if tab.clicked() {
                        view = choice;
                        next.view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
        }
        ui.push_id((&source.id, view as u8, challenge), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Review GRUB menus and root UUIDs before this lesson.",
                    std::iter::empty(),
                );
                lesson_presentation::worked(ui, &definition.example.instructions);
                lesson_presentation::wrapped_worked_commands(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (choice, label) in [
                        (teaching::View::Example, "Try this example"),
                        (teaching::View::Assignment, "Open full assignment"),
                    ] {
                        if ui.add_enabled(active, button(label)).clicked() {
                            next.view = choice;
                            self.lesson_scroll_reset = true;
                            self.notice = None;
                        }
                    }
                });
                return;
            }
            let practice = view == teaching::View::Example;
            let (task, draft) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            let Ok(menu) = Menu::parse(&task.saved_menu) else {
                return;
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, practice_step(draft, task));
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                let mut workspace = |ui: &mut egui::Ui| {
                    let (field, focus, action) = edit(
                        ui,
                        task,
                        draft,
                        &menu,
                        &mut self.focus_input,
                        &mut self.notice,
                        &mut self.metrics.grub_once,
                    );
                    self.metrics.editor_viewport = Some(field);
                    self.metrics.focus = Some(focus);
                    self.metrics.check = Some(action);
                    outcomes(ui, task, draft, practice, &mut self.metrics.grub_once);
                };
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 46. {
                    ui.columns(2, |cols| {
                        workspace(&mut cols[0]);
                        facts(&mut cols[1], task, &menu);
                    });
                } else {
                    facts(ui, task, &menu);
                    ui.add_space(controls::GAP);
                    workspace(ui);
                }
                controls::collapsing_header(ui, "Hints and supported behavior · optional").show(
                    ui,
                    |ui| {
                        for hint in &source.hints {
                            lesson_presentation::prose(ui, hint);
                        }
                        lesson_presentation::prose(ui, work::HELP);
                    },
                );
                if practice {
                    let worked = controls::collapsing_header(ui, "Worked practice · optional")
                        .show(ui, |ui| {
                            lesson_presentation::monospace(ui, &task.reference.linux, true);
                            let fill = ui.add(button("Fill worked example"));
                            record(&mut self.metrics.grub_once, "insert-0", &fill, ui);
                            if fill.clicked() {
                                self.notice = draft.edit(task.reference.clone()).err();
                            }
                        });
                    record(
                        &mut self.metrics.grub_once,
                        "worked",
                        &worked.header_response,
                        ui,
                    );
                }
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::GrubOnce {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, index);
        }
    }
}
