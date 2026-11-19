//! Complete ordinary-directory work in independent bounded virtual terminals.
use super::*;
use dlms_learning_lab::{recursive_tree_source as source_api, recursive_tree_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn append(scope: &mut work::Scope, input: &str, result: &Result<String, String>) {
    let mut text = format!(
        "{}@{} $ {input}\n{}",
        scope.tree.user,
        scope.tree.cwd,
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
    pub(super) fn recursive_tree_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::Goal::RecursiveTree { definition } = &source.goal else {
            return;
        };
        let engine::State::RecursiveTree { simulation } = &a.state else {
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
                    record(&mut self.metrics.recursive_tree, key, &r, ui);
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
                "After ordinary paths, file permissions, creation masks and recursive searches.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Use these separate practice files to learn the method before the assignment.",
            );
            lesson_presentation::wrapped_worked_commands(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.recursive_tree, key, &r, ui);
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
                        controls::dropdown(ui, ("recursive-tree-example", &source.id), width)
                            .selected_text(&definition.example_commands[candidate.example_command])
                            .show_ui(ui, |ui| {
                                for (n, c) in definition.example_commands.iter().enumerate() {
                                    let r =
                                        ui.selectable_value(&mut candidate.example_command, n, c);
                                    record(
                                        &mut self.metrics.recursive_tree,
                                        &format!("choice-{n}"),
                                        &r,
                                        ui,
                                    );
                                }
                            });
                    record(
                        &mut self.metrics.recursive_tree,
                        "picker",
                        &picker.response,
                        ui,
                    );
                    let r = ui.add_enabled(active, button("Insert command"));
                    record(&mut self.metrics.recursive_tree, "insert", &r, ui);
                    if r.clicked() {
                        candidate.practice.command =
                            definition.example_commands[candidate.example_command].clone();
                        self.focus_input = true;
                    }
                });
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let height = (ui.ctx().content_rect().height() * 0.28).clamp(140., 260.);
            let frame=egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui,|ui|{
    ui.label(format!("{} · {} in {}",if practice{"Actual practice terminal"}else{"Simulated assignment terminal"},scope.tree.user,scope.tree.cwd));
    controls::nested_scroll(ui,egui::ScrollArea::both().id_salt(("recursive-tree-output",&source.id,practice)).auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
     if scope.transcript.is_empty(){ui.label("Type a supported command below, then press Enter or Run. Use ls -ld to inspect a directory and ls -l to inspect its files.");}
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
                            "recursive-tree-command",
                            &source.id,
                            visited,
                            practice,
                        )))
                        .hint_text("Command · Enter to run")
                        .char_limit(4096),
                );
                self.metrics.inputs.push(r.rect);
                self.metrics
                    .recursive_tree
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
                record(&mut self.metrics.recursive_tree, "run", &r, ui);
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
            record(&mut self.metrics.recursive_tree, "check", &r, ui);
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
                self.metrics.recursive_tree.result = Some((rows.len(), failed, result_rect));
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
                    "Not checked · choose Check work to evaluate the current directory tree."
                });
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.recursive_tree, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition)
                        .expect("admitted recursive definition")
                        .practice;
                    candidate.example_command = 0;
                }
            }
            ui.push_id(
                (
                    "recursive-tree-help",
                    &source.id,
                    source.version,
                    visited,
                    practice,
                    challenge,
                ),
                |ui| {
                    controls::collapsing_header(ui, "Help and supported command subset").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(
                                ui,
                                &if practice {
                                    definition.example.command_help()
                                } else {
                                    definition.assignment.command_help()
                                },
                            );
                            if !practice {
                                let a = self.progress.attempts.get_mut(&source.id).unwrap();
                                let r = ui.add_enabled(
                                    active && a.hints < source.hints.len(),
                                    button("Show a hint"),
                                );
                                record(&mut self.metrics.recursive_tree, "hint", &r, ui);
                                if r.clicked() {
                                    a.hints += 1;
                                    self.dirty = true;
                                }
                                for hint in source.hints.iter().take(a.hints) {
                                    lesson_presentation::prose(ui, hint);
                                }
                                controls::collapsing_header(
                                    ui,
                                    "Worked approach · reveals a solution",
                                )
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &source.explanation);
                                    for c in source
                                        .solution
                                        .iter()
                                        .filter(|c| c.as_str() != work::CHECK_ACTION)
                                    {
                                        lesson_presentation::command(ui, c, "");
                                    }
                                });
                            }
                            lesson_presentation::prose(ui, &source.simplifications);
                            for link in &source.sources {
                                ui.hyperlink_to(
                                    link,
                                    link.split_whitespace().next().unwrap_or(link),
                                );
                            }
                        },
                    );
                },
            );
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::RecursiveTree {
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
