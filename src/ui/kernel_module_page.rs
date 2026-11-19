//! Supplied module diagnosis, options editing and independent bounded terminals.
use super::*;
use dlms_learning_lab::{kernel_module_source as source_api, kernel_module_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub worked_editor: Option<egui::Rect>,
}
impl Metrics {
    pub(super) fn capture_target(&self, worked: bool) -> Option<egui::Rect> {
        if worked {
            self.worked_editor
        } else {
            self.result.map(|r| r.2)
        }
    }
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn append(scope: &mut work::Scope, input: &str, result: &Result<String, String>) {
    let mut text = format!(
        "learner@virtual-system $ {input}\n{}",
        match result {
            Ok(t) => t.clone(),
            Err(e) => format!("{e}\n"),
        }
    );
    if text.len() > 8192 {
        let mut n = 8000;
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        text.truncate(n);
        text.push_str("\n[Output preview truncated.]\n");
    }
    scope.transcript.push(text);
    if scope.transcript.len() > 40 {
        scope.transcript.remove(0);
    }
}
impl LabApp {
    pub(super) fn kernel_module_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = source_api::context(a, &source) {
            ui.label(e);
            return;
        }
        let engine::Goal::KernelModules { definition } = &source.goal else {
            return;
        };
        let engine::State::KernelModules { simulation } = &a.state else {
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
                    record(&mut self.metrics.kernel_module, key, &r, ui);
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
                "After ordinary terminal commands, file inspection and saved/current configuration.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Use the supplied example devices and modules to learn the method before the assignment.",
            );
            lesson_presentation::wrapped_worked_commands(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.kernel_module, key, &r, ui);
                    if key == "try" {
                        self.metrics.example_action = Some(r.rect);
                    }
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            if practice {
                ui.horizontal_wrapped(|ui| {
                    let width = (ui.available_width()
                        - controls::button_width(ui, "Insert command")
                        - ui.spacing().item_spacing.x)
                        .clamp(100., 620.)
                        .min(ui.available_width());
                    let picker =
                        controls::dropdown(ui, ("kernel-module-example", &source.id), width)
                            .selected_text(
                                definition
                                    .example
                                    .step_label(&definition.example_steps[candidate.example_step]),
                            )
                            .show_ui(ui, |ui| {
                                for (n, c) in definition.example_steps.iter().enumerate() {
                                    let r = ui.selectable_value(
                                        &mut candidate.example_step,
                                        n,
                                        definition.example.step_label(c),
                                    );
                                    record(
                                        &mut self.metrics.kernel_module,
                                        &format!("choice-{n}"),
                                        &r,
                                        ui,
                                    );
                                }
                            });
                    record(
                        &mut self.metrics.kernel_module,
                        "picker",
                        &picker.response,
                        ui,
                    );
                    let insert_label = match &definition.example_steps[candidate.example_step] {
                        work::Step::Command { .. } | work::Step::ExpectedFailure { .. } => {
                            "Insert command"
                        }
                        work::Step::SaveConfig { .. } => "Fill practice editor",
                    };
                    let r = ui.add_enabled(active, button(insert_label));
                    record(&mut self.metrics.kernel_module, "insert", &r, ui);
                    if r.clicked() {
                        match &definition.example_steps[candidate.example_step] {
                            work::Step::Command { text } | work::Step::ExpectedFailure { text } => {
                                candidate.practice.command = text.clone()
                            }
                            work::Step::SaveConfig { text } => {
                                candidate.practice.config_draft = text.clone();
                                candidate.practice.checked = false;
                            }
                        }
                        self.focus_input = true;
                    }
                });
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let activity = if practice {
                &definition.example
            } else {
                &definition.assignment
            };
            if activity.target.saved_options || activity.named() {
                lesson_presentation::rich(
                    ui,
                    RichText::new(if activity.named() {
                        "Saved module names"
                    } else {
                        "Module options file"
                    })
                    .strong(),
                );
                lesson_presentation::monospace(
                    ui,
                    dlms_learning_lab::kernel_modules::CONFIG_PATH,
                    true,
                );
                let draft = if practice {
                    &candidate.practice.config_draft
                } else {
                    &candidate.assignment.config_draft
                };
                let e = shared_libraries_page::text_input(
                    ui,
                    draft,
                    egui::Id::new(("module-options", &source.id, practice)),
                    8192,
                    true,
                    active,
                    if activity.named() {
                        "# alias REQUEST PHYSICAL_MODULE"
                    } else {
                        "# options MODULE parameter=integer"
                    },
                );
                self.metrics.kernel_module.fields.push((
                    "config".into(),
                    e.response.rect,
                    e.response.id,
                ));
                if let Some(text) = e.value
                    && let Err(error) = candidate.edit(definition, text, practice)
                {
                    self.notice = Some(error);
                }
                let r = ui.add_enabled(
                    active && !e.rejected,
                    button(if activity.named() {
                        "Save configuration"
                    } else {
                        "Save options"
                    }),
                );
                record(&mut self.metrics.kernel_module, "save-options", &r, ui);
                if r.clicked()
                    && let Err(error) = candidate.save(definition, practice)
                {
                    self.notice = Some(error);
                }
                let scope = if practice {
                    &candidate.practice
                } else {
                    &candidate.assignment
                };
                lesson_presentation::prose(
                    ui,
                    if scope.config_draft != scope.runtime.config {
                        "Unsaved draft · current insertions still read the last saved file."
                    } else if activity.named() {
                        "Saved names · the next request reads this file. Already loaded modules keep their names."
                    } else {
                        "Saved options · affect future insertion, not an already loaded module."
                    },
                );
                ui.add_space(controls::GAP);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let height = (ui.ctx().content_rect().height() * 0.28).clamp(140., 260.);
            let frame=egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui,|ui|{
    ui.label(if practice{"Actual practice terminal · supplied example system"}else{"Simulated assignment terminal · supplied system"});
    controls::nested_scroll(ui,egui::ScrollArea::both().id_salt(("kernel-module-output",&source.id,practice)).auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
     if scope.transcript.is_empty(){ui.label("Type a supported command below, then press Enter or Run. Start with lspci -k, lsmod or modinfo on a supplied module.");}
     for line in &scope.transcript{ui.add(egui::Label::new(RichText::new(line).monospace()).extend());}
    });
   });
            self.metrics.output = Some(frame.response.rect);
            let mut run = false;
            ui.horizontal(|ui| {
                ui.monospace("$");
                let width = (ui.available_width()
                    - controls::button_width(ui, "Run")
                    - ui.spacing().item_spacing.x)
                    .max(40.);
                let r = ui.add_enabled(
                    active,
                    controls::singleline_fitting(ui, &mut scope.command, width)
                        .id(egui::Id::new((
                            "kernel-module-command",
                            &source.id,
                            visited,
                            practice,
                        )))
                        .hint_text("Command · Enter to run")
                        .char_limit(4096),
                );
                self.metrics.inputs.push(r.rect);
                self.metrics
                    .kernel_module
                    .fields
                    .push(("command".into(), r.rect, r.id));
                if self.focus_input && active {
                    r.request_focus();
                    self.focus_input = false;
                }
                run |= active
                    && (r.has_focus() || r.lost_focus())
                    && ui.input(|v| v.key_pressed(egui::Key::Enter));
                let r = ui.add_enabled(active, primary(ui, "Run"));
                record(&mut self.metrics.kernel_module, "run", &r, ui);
                run |= r.clicked();
            });
            let command = if run {
                Some(std::mem::take(&mut scope.command))
            } else {
                None
            };
            if let Some(input) = command.filter(|s| !s.trim().is_empty()) {
                let result = candidate.run(definition, &input, practice);
                let scope = if practice {
                    &mut candidate.practice
                } else {
                    &mut candidate.assignment
                };
                append(scope, &input, &result);
                self.focus_input = true;
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check work"
                    },
                ),
            );
            record(&mut self.metrics.kernel_module, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                scope.checked = true;
            }
            if scope.checked {
                let rows = scope.checks(if practice {
                    &definition.example
                } else {
                    &definition.assignment
                });
                let failed = rows.iter().filter(|r| !r.0).count();
                let r = ui.strong(if failed == 0 {
                    format!("Passed · all {} outcome checks", rows.len())
                } else {
                    format!(
                        "Needs attention · {failed} remaining {}",
                        if failed == 1 { "check" } else { "checks" }
                    )
                });
                let mut result_rect = r.rect;
                for (n, (_, text)) in rows.iter().filter(|r| !r.0).enumerate() {
                    let row =
                        ui.colored_label(palette(ui).error, format!("Needs attention · {text}"));
                    if n == 0 {
                        result_rect = result_rect.union(row.rect);
                    }
                }
                self.metrics.kernel_module.result = Some((rows.len(), failed, result_rect));
                controls::collapsing_header(ui, "See all outcome checks").show(ui, |ui| {
                    for (ok, text) in &rows {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "{} · {text}",
                                if *ok { "Passed" } else { "Needs attention" }
                            ),
                        );
                    }
                });
            } else {
                ui.label(if practice {
                    "Not checked · try the actions, then choose Check example."
                } else {
                    "Not checked · choose Check work to evaluate current loaded and saved state."
                });
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.kernel_module, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition)
                        .expect("admitted module definition")
                        .practice;
                    candidate.example_step = 0;
                }
            }
            ui.push_id(
                (
                    "kernel-module-help",
                    &source.id,
                    source.version,
                    visited,
                    practice,
                    challenge,
                ),
                |ui| {
                    let help = controls::collapsing_header(ui, "Help and supported command subset")
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, activity.command_help());
                            if !practice {
                                let a = self.progress.attempts.get_mut(&source.id).unwrap();
                                let r = ui.add_enabled(
                                    active && a.hints < source.hints.len(),
                                    button("Show a hint"),
                                );
                                record(&mut self.metrics.kernel_module, "hint", &r, ui);
                                if r.clicked() {
                                    a.hints += 1;
                                    self.dirty = true;
                                }
                                for hint in source.hints.iter().take(a.hints) {
                                    lesson_presentation::prose(ui, hint);
                                }
                                let worked = controls::collapsing_header(
                                    ui,
                                    "Worked approach · reveals a solution",
                                )
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &source.explanation);
                                    for (n, step) in definition.reference.iter().enumerate() {
                                        match step {
                                            work::Step::Command { text }
                                            | work::Step::ExpectedFailure { text } => {
                                                lesson_presentation::command(ui, text, "")
                                            }
                                            work::Step::SaveConfig { text } => {
                                                let block = ui.scope(|ui| {
                                                    ui.strong(if activity.named() {
                                                        "Editor contents · Save configuration"
                                                    } else {
                                                        "Editor contents · Save options"
                                                    });
                                                    lesson_presentation::code_value(
                                                        ui,
                                                        ("module-worked-editor", &source.id, n),
                                                        text,
                                                        ui.available_width(),
                                                    );
                                                });
                                                self.metrics.kernel_module.worked_editor =
                                                    Some(block.response.rect);
                                            }
                                        }
                                    }
                                });
                                record(
                                    &mut self.metrics.kernel_module,
                                    "worked",
                                    &worked.header_response,
                                    ui,
                                );
                            }
                            lesson_presentation::prose(ui, &source.simplifications);
                            for link in &source.sources {
                                ui.hyperlink_to(
                                    link,
                                    link.split_whitespace().next().unwrap_or(link),
                                );
                            }
                        });
                    record(
                        &mut self.metrics.kernel_module,
                        "help",
                        &help.header_response,
                        ui,
                    );
                },
            );
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::KernelModules {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        if view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source)
        {
            self.next_unit_lab(ui, i);
        }
    }
}
