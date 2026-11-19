//! Native terminal practice for an immutable copied systemd journal.
use super::*;
use dlms_learning_lab::{journal_archive, journal_archive_workspace as work};

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}

impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, response: &egui::Response) {
        self.controls
            .push((key.into(), response.rect, ui.clip_rect()));
    }
}

fn query_result(ui: &mut egui::Ui, salt: &str, output: &str) {
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 12.;
    controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt(salt)
            .max_height(height)
            .auto_shrink([false, true]),
        |ui| {
            ui.add(
                egui::Label::new(RichText::new(output).monospace())
                    .extend()
                    .selectable(true),
            )
        },
    );
}

impl LabApp {
    pub(super) fn copied_journal_page(&mut self, ui: &mut egui::Ui, index: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&source.id];
        let definition = match work::source(&source)
            .and_then(|definition| work::context(attempt, &source).map(|()| definition))
        {
            Ok(definition) => definition,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let engine::State::CopiedJournal { simulation } = &attempt.state else {
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
        self.metrics.copied_journal = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(choice == view));
                    self.metrics.copied_journal.record(
                        ui,
                        match choice {
                            teaching::View::Introduction => "introduction",
                            teaching::View::Example => "example",
                            teaching::View::Assignment => "assignment",
                        },
                        &response,
                    );
                    self.metrics.lesson_tabs.push(response.rect);
                    if response.clicked() {
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
                    "Review basic Linux terminal and logging practice.",
                    std::iter::empty(),
                );
                lesson_presentation::worked(ui, &definition.example.instructions);
                ui.strong("Expected boot list · worked example");
                query_result(ui, "worked-boots", &definition.example.boot_list());
                lesson_presentation::wrapped_worked_commands(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (choice, title) in [
                        (teaching::View::Example, "Try this example"),
                        (teaching::View::Assignment, "Open full assignment"),
                    ] {
                        if ui.add_enabled(active, button(title)).clicked() {
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
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                let step = if !draft.listed_boots {
                    0
                } else if draft.ran_message_query() {
                    2
                } else {
                    1
                };
                lesson_presentation::practice_step(ui, &source, step);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                let mut command = draft.command.clone();
                let label = ui.strong("Simulated journal console");
                let input = ui
                    .add(
                        controls::singleline_fitting(ui, &mut command, ui.available_width())
                            .hint_text("Type a journalctl command")
                            .char_limit(journal_archive::MAX_COMMAND),
                    )
                    .labelled_by(label.id);
                self.metrics.copied_journal.record(ui, "input", &input);
                if input.changed() {
                    self.notice = draft.edit(command).err();
                }
                let submitted =
                    input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                ui.horizontal_wrapped(|ui| {
                    let run = ui.add(primary(ui, "Run query"));
                    self.metrics.copied_journal.record(ui, "run", &run);
                    if run.clicked() || submitted {
                        self.notice = draft.run(task).err();
                        if submitted {
                            input.request_focus();
                        }
                    }
                    let check = ui.add_enabled(draft.ran_current, button("Check query"));
                    self.metrics.copied_journal.record(ui, "check", &check);
                    if check.clicked() {
                        self.notice = draft.check(task).err();
                    }
                });
                if let Some(notice) = &self.notice {
                    lesson_presentation::prose(ui, notice);
                }
                if draft.checked {
                    let checks = draft.checks(task);
                    if checks.iter().all(|check| check.0) {
                        let result = ui.strong("Incident evidence verified");
                        self.metrics.copied_journal.record(ui, "result", &result);
                    } else {
                        for (_, message) in checks.iter().filter(|row| !row.0) {
                            lesson_presentation::prose(ui, message);
                        }
                    }
                }
                match draft.output(task) {
                    Ok(Some(output)) => {
                        ui.strong("Query output");
                        query_result(ui, "output", &output);
                    }
                    Ok(None) => {
                        ui.label("Run the current query to see its output.");
                    }
                    Err(error) => {
                        lesson_presentation::prose(ui, &error);
                    }
                }
                if draft.listed_boots && !draft.showing_boot_list() {
                    controls::collapsing_header(ui, "Previously listed boots").show(ui, |ui| {
                        query_result(ui, "previous-boots", &task.boot_list());
                    });
                }
                controls::collapsing_header(ui, "Command help · optional").show(ui, |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    for hint in &source.hints {
                        lesson_presentation::prose(ui, hint);
                    }
                });
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::CopiedJournal {
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
