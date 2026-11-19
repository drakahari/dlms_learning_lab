//! Native local-mail editing with actual bounded delivery results.
use super::*;
use dlms_learning_lab::{local_mail, local_mail_workspace as work};

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

fn document_title(document: &work::Document) -> String {
    match document {
        work::Document::Aliases => "/etc/aliases".into(),
        work::Document::Forward(user) => format!("/home/{user}/.forward"),
    }
}

fn guided_step(draft: &work::Draft, task: &work::Task) -> usize {
    if (draft.tested.is_some() && draft.checks(task).iter().all(|check| check.0))
        || (draft.saved != task.initial
            && draft.documents == draft.saved
            && draft.index == draft.saved.aliases)
    {
        2
    } else if draft.documents != task.initial
        || draft.saved != task.initial
        || draft.index != task.initial_index
    {
        1
    } else {
        0
    }
}

fn editor(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    task: &work::Task,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    if !task.initial.forwards.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label("Virtual file");
            let dropdown = controls::dropdown(ui, "mail-document", ui.available_width())
                .selected_text(document_title(&draft.selected))
                .show_ui(ui, |ui| {
                    let choice = ui.selectable_value(
                        &mut draft.selected,
                        work::Document::Aliases,
                        "/etc/aliases",
                    );
                    metrics.record(ui, "document-aliases", &choice);
                    for user in task.initial.forwards.keys() {
                        let choice = ui.selectable_value(
                            &mut draft.selected,
                            work::Document::Forward(user.clone()),
                            format!("/home/{user}/.forward"),
                        );
                        metrics.record(ui, &format!("document-forward-{user}"), &choice);
                    }
                });
            metrics.record(ui, "document", &dropdown.response);
        });
    }
    let mut text = draft.text().to_owned();
    let label = ui.strong(format!(
        "{} · simulated file",
        document_title(&draft.selected)
    ));
    let input = ui
        .add(
            egui::TextEdit::multiline(&mut text)
                .font(egui::TextStyle::Monospace)
                .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                .desired_width((ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.))
                .desired_rows(match &draft.selected {
                    work::Document::Aliases => 8,
                    work::Document::Forward(_) => 4,
                })
                .char_limit(local_mail::MAX_TEXT),
        )
        .labelled_by(label.id);
    metrics.record(ui, "editor", &input);
    if input.changed() {
        *notice = draft.edit(text).err();
    }
    let saved = match &draft.selected {
        work::Document::Aliases => &draft.saved.aliases,
        work::Document::Forward(user) => &draft.saved.forwards[user],
    };
    ui.label(if draft.text() == saved {
        "This editor matches the saved virtual file."
    } else {
        "Unsaved changes. Save this virtual file before testing."
    });
    ui.horizontal_wrapped(|ui| {
        let save = ui.add(button("Save file"));
        metrics.record(ui, "save-file", &save);
        if save.clicked() {
            *notice = draft.save_file(task).err();
        }
        if draft.selected == work::Document::Aliases {
            let rebuild = ui.add(button("Rebuild aliases"));
            metrics.record(ui, "rebuild", &rebuild);
            if rebuild.clicked() {
                *notice = draft.rebuild_aliases(task).err();
            }
        }
        let test = ui.add(primary(ui, "Run delivery test"));
        metrics.record(ui, "test", &test);
        if test.clicked() {
            *notice = draft.test(task).err();
        }
    });
    if draft.selected == work::Document::Aliases {
        lesson_presentation::prose(
            ui,
            if draft.index == draft.saved.aliases {
                "The aliases lookup matches the saved file."
            } else {
                "The aliases lookup uses earlier text. Rebuild it after saving your repair."
            },
        );
    }
    if let Some(message) = notice {
        lesson_presentation::prose(ui, message);
    }
}

fn results(ui: &mut egui::Ui, draft: &work::Draft, task: &work::Task, metrics: &mut Metrics) {
    let deliveries = match draft.deliveries(task) {
        Ok(Some(deliveries)) => deliveries,
        Ok(None) => {
            ui.strong("Sample messages");
            for probe in &task.probes {
                lesson_presentation::prose(
                    ui,
                    &format!("{} sent to {}.", probe.label, probe.recipient),
                );
            }
            ui.label("Run the delivery test to see actual destination mailboxes.");
            return;
        }
        Err(error) => {
            lesson_presentation::prose(ui, &error);
            return;
        }
    };
    let checks = draft.checks(task);
    let passed = checks.iter().filter(|row| row.0).count();
    let complete = passed == checks.len();
    let title = if complete {
        "Local delivery and saved policy verified".to_owned()
    } else {
        format!("{passed}/{} requirements met", checks.len())
    };
    let result = ui.strong(title);
    metrics.record(ui, "result", &result);
    if !checks[0].0 {
        lesson_presentation::prose(ui, &checks[0].1);
    }
    ui.label("Actual deliveries from this test");
    for (index, (label, delivery)) in deliveries.iter().enumerate() {
        ui.add_space(controls::GAP);
        let matched = checks.get(index + 1).is_some_and(|row| row.0);
        ui.strong(format!(
            "{label} · {}",
            if matched { "Pass" } else { "Needs change" }
        ));
        let mailboxes = delivery.mailboxes.iter().cloned().collect::<Vec<_>>();
        lesson_presentation::prose(
            ui,
            &if mailboxes.is_empty() {
                "No destination mailbox.".into()
            } else {
                format!("Delivered to {}.", mailboxes.join(", "))
            },
        );
        if !matched && let Some((_, message)) = checks.get(index + 1) {
            lesson_presentation::prose(ui, message);
        }
    }
}

impl LabApp {
    pub(super) fn local_mail_page(&mut self, ui: &mut egui::Ui, index: usize) {
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
        let engine::State::LocalMail { simulation } = &attempt.state else {
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
        self.metrics.local_mail = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title, key) in [
                    (teaching::View::Introduction, "Introduction", "introduction"),
                    (teaching::View::Example, "Try example · ungraded", "example"),
                    (teaching::View::Assignment, "Assignment", "assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(view == choice));
                    self.metrics.local_mail.record(ui, key, &response);
                    self.metrics.lesson_tabs.push(response.rect);
                    if response.clicked() {
                        next.view = choice;
                        view = choice;
                        self.notice = None;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        ui.push_id((&source.id, view as u8, challenge), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Review local user identities and editing virtual files.",
                    std::iter::empty(),
                );
                lesson_presentation::worked(ui, &definition.example.instructions);
                lesson_presentation::wrapped_worked_commands(ui, &source);
                ui.strong("Expected deliveries · worked example");
                for probe in &definition.example.probes {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "Mail to {} reaches {}.",
                            probe.recipient,
                            probe.expected.join(", ")
                        ),
                    );
                }
                if ui.add_enabled(active, button("Try this example")).clicked() {
                    next.view = teaching::View::Example;
                    self.lesson_scroll_reset = true;
                    self.notice = None;
                }
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
                let step = guided_step(draft, task);
                lesson_presentation::practice_step(ui, &source, step);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 48. {
                    ui.columns(2, |columns| {
                        editor(
                            &mut columns[0],
                            draft,
                            task,
                            &mut self.notice,
                            &mut self.metrics.local_mail,
                        );
                        results(&mut columns[1], draft, task, &mut self.metrics.local_mail);
                    });
                } else {
                    editor(
                        ui,
                        draft,
                        task,
                        &mut self.notice,
                        &mut self.metrics.local_mail,
                    );
                    ui.separator();
                    results(ui, draft, task, &mut self.metrics.local_mail);
                }
                let help =
                    controls::collapsing_header(ui, "Current lookup and mail help · optional")
                        .show(ui, |ui| {
                            ui.strong("Current aliases lookup");
                            ui.add(
                                egui::Label::new(RichText::new(&draft.index).monospace())
                                    .wrap()
                                    .selectable(true),
                            );
                            for hint in &source.hints {
                                lesson_presentation::prose(ui, hint);
                            }
                            lesson_presentation::prose(ui, &source.simplifications);
                        });
                self.metrics
                    .local_mail
                    .record(ui, "help", &help.header_response);
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::LocalMail {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> work::Definition {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../../scenarios/boot-entry-lessons-v11.json"))
                .unwrap();
        serde_json::from_value(
            value["scenarios"].as_array().unwrap().last().unwrap()["goal"]["definition"].clone(),
        )
        .unwrap()
    }

    #[test]
    fn failed_starter_inspection_keeps_the_diagnosis_guidance() {
        let definition = definition();
        let task = &definition.example;
        let mut draft = work::Draft::new(task);
        assert_eq!(guided_step(&draft, task), 0);
        draft.test(task).unwrap();
        assert_eq!(guided_step(&draft, task), 0);
        draft.rebuild_aliases(task).unwrap();
        draft.test(task).unwrap();
        assert_eq!(guided_step(&draft, task), 1);
        draft.edit(task.reference.aliases.clone()).unwrap();
        draft.save_file(task).unwrap();
        assert_eq!(guided_step(&draft, task), 1);
        draft.rebuild_aliases(task).unwrap();
        assert_eq!(guided_step(&draft, task), 2);
    }
}
