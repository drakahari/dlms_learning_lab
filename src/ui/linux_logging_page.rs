//! Native configuration editing and simulated event replay, no daemon or host logs.
use super::*;
use dlms_learning_lab::{linux_logging as core, linux_logging_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
impl LabApp {
    pub(super) fn linux_logging_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            ui.label(e);
            return;
        }
        let (engine::Goal::LinuxLogging { definition }, engine::State::LinuxLogging { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let visited = a.visited;
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            candidate.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    self.metrics.lesson_tabs.push(r.rect);
                    record(&mut self.metrics.linux_logging, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            candidate.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on reading and filtering log records; no real system setup is needed.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.task);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.linux_logging, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &definition.example.task);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let case = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            ui.strong("Configuration editor · simulated rsyslog selector subset");
            if practice {
                let r = ui.add_enabled(active, button("Insert worked configuration"));
                record(&mut self.metrics.linux_logging, "insert", &r, ui);
                if r.clicked() {
                    scope.draft = definition.example_worked.clone();
                    scope.diagnostic = None;
                }
            }
            let r = ui.add_enabled(
                active,
                egui::TextEdit::multiline(&mut scope.draft)
                    .code_editor()
                    .id(egui::Id::new((
                        "linux-log-config",
                        &source.id,
                        visited,
                        practice,
                    )))
                    .desired_width(f32::INFINITY)
                    .desired_rows(7)
                    .char_limit(8192),
            );
            name_control(&r, "Simulated logging selector configuration");
            self.metrics.linux_logging.editor = Some((r.rect, r.id));
            let shortcut = r.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
            if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                r.surrender_focus();
            }
            ui.label(
                RichText::new("Tab indents · Ctrl+Enter tests · Esc releases editor focus")
                    .small()
                    .color(palette(ui).muted),
            );
            if r.changed() {
                workspace::bound(&mut scope.draft, 8192);
                scope.diagnostic = None;
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let r = ui.add_enabled(active, primary(ui, "Run routing test"));
            record(&mut self.metrics.linux_logging, "run", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() || shortcut {
                let _ = candidate.run(definition, practice);
            }
            let scope = if practice {
                &candidate.practice
            } else {
                &candidate.assignment
            };
            if let Some(error) = &scope.diagnostic {
                ui.colored_label(palette(ui).error, "Configuration not applied");
                lesson_presentation::prose(ui, error);
            }
            let current = scope.last_run.as_deref() == Some(scope.draft.as_str());
            if let Some(last_run) = &scope.last_run {
                ui.strong(if current {
                    "Actual replay · supplied sample events"
                } else {
                    "Previous replay · draft changed; run again for current results"
                });
                lesson_presentation::prose(
                    ui,
                    "The sample events illustrate the replay; routing checks include all192 facility/severity combinations.",
                );
                let rules = core::parse(last_run, &definition.destinations)
                    .expect("validated saved replay");
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::horizontal().id_salt((
                        "linux-log-actual",
                        &source.id,
                        practice,
                    )),
                    |ui| {
                        egui::Grid::new(("linux-log-events", &source.id, practice))
                            .striped(true)
                            .spacing(egui::vec2(20., 8.))
                            .show(ui, |ui| {
                                for title in
                                    ["Source · severity", "Sample event", "Actual destinations"]
                                {
                                    ui.strong(title);
                                }
                                ui.end_row();
                                for event in &case.events {
                                    ui.monospace(format!(
                                        "{}.{}({})",
                                        core::FACILITIES[event.facility as usize],
                                        core::SEVERITIES[event.severity as usize],
                                        event.severity
                                    ));
                                    ui.label(&event.message);
                                    ui.monospace(core::describe_destinations(&core::deliveries(
                                        &rules,
                                        event.facility,
                                        event.severity,
                                    )));
                                    ui.end_row();
                                }
                            });
                    },
                );
                if current {
                    let checks = definition.checks(&candidate, practice);
                    let failed = checks.iter().filter(|r| !r.0).count();
                    let r = ui.colored_label(
                        if failed == 0 {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        if failed == 0 {
                            "Passed · complete routing policy".into()
                        } else {
                            format!(
                                "Needs attention · {failed} source groups differ from the policy"
                            )
                        },
                    );
                    self.metrics.linux_logging.result = Some((failed, r.rect));
                    if failed > 0 {
                        lesson_presentation::prose(
                            ui,
                            "Lower severity numbers are more urgent. Check every matching rule: duplicate matches add copies.",
                        );
                    }
                    controls::nested_scroll(
                        ui,
                        egui::ScrollArea::vertical()
                            .id_salt(("linux-log-feedback", &source.id, practice))
                            .max_height(220.),
                        |ui| {
                            for (_, message) in checks {
                                lesson_presentation::prose(ui, &message);
                            }
                        },
                    );
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    "Not checked · edit the configuration and run the routing test.",
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.linux_logging, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).practice;
                }
            }
            ui.push_id(("linux-log-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and supported configuration subset").show(
                    ui,
                    |ui| {
                        lesson_presentation::prose(ui, core::HELP);
                        if !practice {
                            let a = self.progress.attempts.get_mut(&source.id).unwrap();
                            let r = ui.add_enabled(
                                active && a.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            record(&mut self.metrics.linux_logging, "hint", &r, ui);
                            if r.clicked() {
                                a.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(a.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Worked approach · reveals a solution")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &source.explanation);
                                    for config in &source.solution {
                                        lesson_presentation::monospace(
                                            ui,
                                            config.strip_prefix(work::PREFIX).unwrap_or(config),
                                            false,
                                        );
                                    }
                                });
                        }
                        for source in &source.sources {
                            ui.hyperlink_to(source, source);
                        }
                    },
                );
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LinuxLogging {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
