//! Native chrony source diagnosis. Editor, applied state and checks are separate.
use super::*;
use dlms_learning_lab::{chrony, chrony_workspace as work};

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

fn editor(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    task: &work::Task,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let label = ui.strong("chrony.conf · simulated file");
    let mut text = draft.source.clone();
    let response = ui
        .add(
            egui::TextEdit::multiline(&mut text)
                .font(egui::TextStyle::Monospace)
                .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                .desired_width((ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.))
                .desired_rows(5)
                .char_limit(chrony::MAX_CONFIG),
        )
        .labelled_by(label.id);
    metrics.record(ui, "editor", &response);
    if response.changed() {
        *notice = draft.edit(text).err();
    }
    lesson_presentation::prose(
        ui,
        if draft.source == draft.applied {
            "The edited file matches the applied configuration."
        } else {
            "The simulation still uses the previous configuration. Apply the edited file before checking."
        },
    );
    ui.horizontal_wrapped(|ui| {
        let apply = ui.add(button("Apply configuration"));
        metrics.record(ui, "apply", &apply);
        if apply.clicked() {
            *notice = draft.apply(task).err();
        }
        let check = ui.add(primary(ui, "Check source"));
        metrics.record(ui, "check", &check);
        if check.clicked() {
            *notice = draft.test(task).err();
        }
    });
    if draft.checked.is_some() {
        let checks = draft.checks(task);
        if checks.iter().all(|check| check.0) {
            let result = ui.strong("Source selected and backup retained");
            metrics.record(ui, "result", &result);
        } else {
            for (_, message) in checks.iter().filter(|row| !row.0) {
                lesson_presentation::prose(ui, message);
            }
        }
    }
}

fn console(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    task: &work::Task,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let label = ui.strong("Simulated chrony console");
    let mut command = draft.command.clone();
    let response = ui
        .add(
            controls::singleline_fitting(ui, &mut command, ui.available_width())
                .hint_text("chronyc sources or chronyc tracking")
                .char_limit(256),
        )
        .labelled_by(label.id);
    metrics.record(ui, "input", &response);
    if response.changed() {
        *notice = draft.set_command(command).err();
    }
    let submitted = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    let run = ui.add(button("Run command"));
    metrics.record(ui, "run", &run);
    if run.clicked() || submitted {
        *notice = draft.run(task).err();
        response.request_focus();
    }
    ui.label("Last successful inspection of the applied configuration");
    let output = draft.output(task).unwrap_or_else(Some);
    if let Some(output) = output {
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 9.;
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("chrony-output")
                .max_height(height)
                .auto_shrink([false, true]),
            |ui| lesson_presentation::monospace(ui, &output, false),
        );
    } else {
        ui.label("No command run yet.");
    }
}

impl LabApp {
    pub(super) fn chrony_page(&mut self, ui: &mut egui::Ui, index: usize) {
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
            Ok(value) => value,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let engine::State::Chrony { simulation } = &attempt.state else {
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
        self.metrics.chrony = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(choice == view));
                    let key = match choice {
                        teaching::View::Introduction => "introduction",
                        teaching::View::Example => "example",
                        teaching::View::Assignment => "assignment",
                    };
                    self.metrics.chrony.record(ui, key, &response);
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
                    "Review IPv4 addresses and saved Linux settings.",
                    std::iter::empty(),
                );
                lesson_presentation::worked(ui, &definition.example.instructions);
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
                let step = if draft.applied != task.initial {
                    2
                } else if draft.inspection.is_some() {
                    1
                } else {
                    0
                };
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
                            &mut self.metrics.chrony,
                        );
                        console(
                            &mut columns[1],
                            draft,
                            task,
                            &mut self.notice,
                            &mut self.metrics.chrony,
                        );
                    });
                } else {
                    console(ui, draft, task, &mut self.notice, &mut self.metrics.chrony);
                    ui.add_space(controls::GAP);
                    editor(ui, draft, task, &mut self.notice, &mut self.metrics.chrony);
                }
                if let Some(notice) = &self.notice {
                    lesson_presentation::prose(ui, notice);
                }
                lesson_presentation::prose(
                    ui,
                    &format!("Display time zone remains {}.", task.timezone),
                );
                controls::collapsing_header(ui, "Hints and supported behavior · optional").show(
                    ui,
                    |ui| {
                        for hint in &source.hints {
                            lesson_presentation::prose(ui, hint);
                        }
                        lesson_presentation::prose(ui, chrony::HELP);
                    },
                );
                if practice {
                    controls::collapsing_header(ui, "Worked practice · optional").show(ui, |ui| {
                        lesson_presentation::monospace(ui, &task.reference, true);
                        if ui.add(button("Fill worked example")).clicked() {
                            self.notice = draft.edit(task.reference.clone()).err();
                        }
                    });
                }
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::Chrony {
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
