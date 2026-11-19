//! Guided and independent practice on a virtual Open-iSCSI initiator.
use super::*;
use dlms_learning_lab::{
    engine::State,
    iscsi::{
        command, source,
        workspace::{Attempt, Task},
    },
};

type Metrics = super::linux_v6_page::Metrics;

fn supplied(ui: &mut egui::Ui, task: &Task) {
    let target = &task.goal.target;
    if !task.goal.automatic_at_startup
        && let Some(export) = task.inventory.export(target)
    {
        controls::form_row(ui, "Discovery address", |ui, _| {
            lesson_presentation::monospace(ui, &export.discovery.to_string(), true)
        });
    }
    for (label, value) in [
        ("Required target", target.target.clone()),
        ("Required portal", target.portal.to_string()),
        (
            "Inventory ticket",
            format!("LUN {} · WWN {}", task.goal.lun, task.goal.wwn),
        ),
    ] {
        controls::form_row(ui, label, |ui, _| {
            lesson_presentation::monospace(ui, &value, true)
        });
    }
}

fn console(
    ui: &mut egui::Ui,
    key: egui::Id,
    task: &Task,
    work: &mut Attempt,
    metrics: &mut Metrics,
) {
    ui.strong("Initiator console");
    ui.label(format!(
        "Saved nodes: {} · Active sessions: {}",
        work.initiator.nodes.len(),
        work.initiator.active.len()
    ));

    let input = shared_libraries_page::fitting_input(
        ui,
        &work.input,
        key.with("command"),
        command::INPUT_LIMIT,
        ui.is_enabled(),
        "Enter an iscsiadm or lsblk command",
    );
    metrics.record(ui, "command", &input.response);
    metrics
        .fields
        .push(("command".into(), input.response.rect, input.response.id));
    if let Some(text) = input.value {
        work.input = text;
    }
    if input.rejected {
        ui.label("Keep commands within 1,024 bytes. Your previous text was retained.");
    }
    let enter = input.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let run = ui.add_enabled(!work.input.trim().is_empty(), button("Run"));
    metrics.record(ui, "run", &run);
    if run.clicked() || ui.is_enabled() && enter {
        work.run(task);
        input.response.request_focus();
    }

    if work.history.is_empty() {
        let response = ui.label("No commands have run.");
        metrics.record(ui, "console-output", &response);
        return;
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8.;
    let output = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(key.with("log"))
            .max_height(height)
            .auto_shrink([false, true])
            .stick_to_bottom(true),
        |ui| {
            for result in &work.history {
                lesson_presentation::monospace(ui, &format!("$ {}", result.input), true);
                ui.strong(if result.ok {
                    "Actual result"
                } else {
                    "Command failed"
                });
                lesson_presentation::monospace(ui, &result.output, true);
            }
        },
    );
    let response = ui.interact(output.inner_rect, key.with("output"), egui::Sense::hover());
    metrics.record(ui, "console-output", &response);
}

fn startup_preview(ui: &mut egui::Ui, key: egui::Id, t: &Task, w: &Attempt, metrics: &mut Metrics) {
    let r = controls::collapsing_header(ui, "Preview automatic logins")
        .id_salt(key.with("startup"))
        .show(ui, |ui| {
            ui.label(
                "Uses the saved node policies. The preview leaves current sessions unchanged.",
            );
            let mut plan = t.startup_plan(&w.initiator).peekable();
            if plan.peek().is_none() {
                ui.label("No node will attempt automatic login.");
            }
            for (target, available) in plan {
                lesson_presentation::monospace(ui, &target.to_string(), true);
                ui.label(if available {
                    "Would connect"
                } else {
                    "Would attempt login and fail"
                });
            }
        });
    metrics.record(ui, "startup-preview", &r.header_response);
}

fn results(
    ui: &mut egui::Ui,
    key: egui::Id,
    task: &Task,
    work: &Attempt,
    practice: bool,
    m: &mut Metrics,
) -> bool {
    let checks = task.checks(&work.initiator);
    let count = checks.iter().filter(|result| result.passed).count();
    let solved = count == checks.len();
    let r = ui.strong(if !work.checked {
        "Not checked".into()
    } else if solved && practice {
        "Example verified".into()
    } else if solved {
        "Assignment complete".into()
    } else {
        format!("Needs work · {count}/{} requirements met", checks.len())
    });
    m.record(ui, "result", &r);
    m.result = Some((count, checks.len(), r.rect));
    if !work.checked {
        return false;
    }
    for result in checks.iter().filter(|c| !c.passed) {
        let r = ui.scope(|ui| {
            ui.strong(format!("Needs work · {}", result.label));
            lesson_presentation::prose(ui, &result.feedback);
        });
        m.record(ui, "failed-case", &r.response);
    }
    controls::collapsing_header(
        ui,
        if solved {
            "Objective results"
        } else {
            "Requirements already met"
        },
    )
    .id_salt(key.with("results"))
    .show(ui, |ui| {
        for result in checks.iter().filter(|c| c.passed) {
            ui.strong(format!("Passed · {}", result.label));
            lesson_presentation::prose(ui, &result.feedback);
        }
    });
    solved
}

impl LabApp {
    pub(super) fn iscsi_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let original = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(error) = source::context(original, &scenario) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::Iscsi { simulation } = &original.state else {
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
        let metrics = &mut self.metrics.iscsi;
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
            lesson_presentation::worked(ui, "Example on a different virtual initiator");
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
            let key = egui::Id::new(("iscsi", &scenario.id, practice));
            if !practice {
                let r = lesson_presentation::assignment_responses(ui, &scenario);
                metrics.record(ui, "assignment-content", &r.goal);
            } else {
                lesson_presentation::practice_step(ui, &scenario, 0);
            }
            supplied(ui, task);
            ui.separator();
            if practice {
                lesson_presentation::practice_step_commands(ui, &scenario, 1);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                console(ui, key, task, work, metrics);
                if task.goal.automatic_at_startup {
                    startup_preview(ui, key, task, work, metrics);
                }
                ui.separator();
                if practice {
                    lesson_presentation::practice_step(ui, &scenario, 2);
                }
                let check = ui.add(primary(ui, "Check configuration"));
                metrics.record(ui, "check", &check);
                if check.clicked() {
                    work.checked = true;
                }
                if results(ui, key, task, work, practice, metrics) && !practice {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = ui.add(button("Reset example"));
                    metrics.record(ui, "reset-example", &r);
                    reset = r.clicked();
                }
            });
            if reset && let Err(e) = next.reset_example(definition) {
                self.notice = Some(e);
            }
            controls::collapsing_header(ui, "Supported commands, hints and references")
                .id_salt(key.with("help"))
                .show(ui, |ui| {
                    lesson_presentation::monospace(ui, command::HELP, true);
                    if ui.add_enabled(active, button("Show a hint")).clicked() {
                        let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                        a.hints = (a.hints + 1).min(scenario.hints.len());
                        self.dirty = true;
                    }
                    for hint in scenario
                        .hints
                        .iter()
                        .take(self.progress.attempts[&scenario.id].hints)
                    {
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
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state = State::Iscsi {
                        simulation: Box::new(next),
                    };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
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
