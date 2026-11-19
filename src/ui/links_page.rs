//! Guided link investigation. All terminal/editor actions use the in-process VFS.
use super::*;
use dlms_learning_lab::links_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn output(scope: &mut work::Scope, input: &str, result: &Result<String, String>) {
    let mut line = format!(
        "{} $ {input}\n{}",
        scope.world.cwd().unwrap_or("/work"),
        match result {
            Ok(s) if s.is_empty() => String::new(),
            Ok(s) => s.clone(),
            Err(e) => format!("{e}\n"),
        }
    );
    if line.len() > 8192 {
        let mut n = 8000;
        while !line.is_char_boundary(n) {
            n -= 1;
        }
        line.truncate(n);
        line.push_str(
            "\n[Output preview truncated; inspect a smaller file or use the simulated editor.]\n",
        );
    }
    scope.transcript.push(line);
    if scope.transcript.len() > 40 {
        scope.transcript.remove(0);
    }
}
impl LabApp {
    pub(super) fn links_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::Goal::Links { definition } = &source.goal else {
            return;
        };
        let engine::State::Links { simulation } = &a.state else {
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
                    record(&mut self.metrics.links, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        if !challenge {
            candidate.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on files, paths and handovers.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "These commands operate on different practice files. Insert only fills the command field; Run executes in that separate simulation.",
            );
            lesson_presentation::wrapped_worked_commands(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.links, key, &r, ui);
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
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            if practice {
                ui.horizontal_wrapped(|ui| {
                    let width = (ui.available_width() - 160.)
                        .clamp(100., 600.)
                        .min(ui.available_width());
                    let picker =
                        controls::dropdown(ui, ("links-example-command", &source.id), width)
                            .selected_text(&definition.example_commands[candidate.example_command])
                            .show_ui(ui, |ui| {
                                for (n, command) in definition.example_commands.iter().enumerate() {
                                    let r = ui.selectable_value(
                                        &mut candidate.example_command,
                                        n,
                                        command,
                                    );
                                    record(&mut self.metrics.links, &format!("choice-{n}"), &r, ui);
                                }
                            });
                    record(&mut self.metrics.links, "picker", &picker.response, ui);
                    let r = ui.add_enabled(active, button("Insert command"));
                    record(&mut self.metrics.links, "insert", &r, ui);
                    if r.clicked() {
                        scope.command =
                            definition.example_commands[candidate.example_command].clone();
                        self.focus_input = true;
                    }
                });
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let height = (ui.ctx().content_rect().height() * 0.3).clamp(150., 300.);
            let area=egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui,|ui|{
    ui.label(if practice{"Actual practice terminal"}else{"Simulated terminal · assignment"});
    egui::ScrollArea::both().id_salt(("links-output",&source.id,practice)).auto_shrink([false, false])
                        .min_scrolled_height(height).max_height(height).stick_to_bottom(true).show(ui,|ui|{
     if scope.transcript.is_empty(){ui.label("Type a supported command below, then press Enter or Run. Start with stat, readlink, cat or ls to investigate the supplied paths.");}
     for line in &scope.transcript{ui.add(egui::Label::new(RichText::new(line).monospace()).extend());}
    });
   });
            self.metrics.output = Some(area.response.rect);
            let mut run = false;
            ui.horizontal(|ui| {
                ui.monospace("/work $");
                let width = (ui.available_width() - 90.).max(40.);
                let r = ui.add_enabled(
                    active,
                    controls::singleline(ui, &mut scope.command)
                        .id(egui::Id::new((
                            "links-command",
                            &source.id,
                            visited,
                            practice,
                        )))
                        .hint_text("Command · Enter to run")
                        .char_limit(4096)
                        .desired_width(width),
                );
                self.metrics.inputs.push(r.rect);
                self.metrics
                    .links
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
                record(&mut self.metrics.links, "run", &r, ui);
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
                output(scope, &input, &result);
                self.focus_input = true;
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let mut save_editor = false;
            ui.push_id(("links-editor",&source.id,visited,practice),|ui|{let disclosure=controls::collapsing_header(ui,"File editor · optional · simulated files only").show(ui,|ui|{
    ui.label("Open an existing virtual file. Saving through a hard-link name changes the shared inode; a copied file stays independent.");
    ui.horizontal_wrapped(|ui|{
     let r=ui.add_enabled(active,controls::singleline(ui,&mut scope.editor_path).hint_text("Existing virtual file path").char_limit(1024).desired_width(ui.available_width().min(330.)));self.metrics.links.fields.push(("editor-path".into(),r.rect,r.id));
     let r=ui.add_enabled(active,button("Open / discard edits"));record(&mut self.metrics.links,"open-file",&r,ui);if r.clicked(){match scope.world.read_file(&scope.editor_path){Ok(text)=>{scope.editor_text=text;scope.editor_open_path=scope.editor_path.clone();scope.editor_loaded=true;},Err(e)=>self.notice=Some(e.to_string())}}
    });
    if scope.editor_loaded{ui.monospace(&scope.editor_open_path);let r=ui.add_enabled(active,egui::TextEdit::multiline(&mut scope.editor_text).code_editor().desired_width(f32::INFINITY).desired_rows(5).char_limit(8192));self.metrics.links.fields.push(("editor-text".into(),r.rect,r.id));let r=ui.add_enabled(active&&scope.editor_path==scope.editor_open_path,button("Save file"));record(&mut self.metrics.links,"save-file",&r,ui);save_editor=r.clicked();}
   });record(&mut self.metrics.links,"editor",&disclosure.header_response,ui);});
            if save_editor {
                if let Err(e) = candidate.save_editor(definition, practice) {
                    self.notice = Some(e);
                } else {
                    self.notice =
                        Some("Saved the simulated file; inspect its aliases or check work.".into());
                }
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
            record(&mut self.metrics.links, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                scope.checked = true;
            }
            if scope.checked {
                let rows = definition.checks(&candidate, practice);
                let passed = rows.iter().filter(|r| r.0).count();
                let r = ui.strong(if passed == rows.len() {
                    format!("Passed · all {} outcome checks", rows.len())
                } else {
                    format!(
                        "Needs attention · {} of {} checks",
                        rows.len() - passed,
                        rows.len()
                    )
                });
                self.metrics.links.result = Some((rows.len(), rows.len() - passed, r.rect));
                for (ok, text) in &rows {
                    ui.colored_label(
                        if *ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        format!(
                            "{} · {text}",
                            if *ok { "Passed" } else { "Needs attention" }
                        ),
                    );
                }
            } else {
                ui.label(if practice {
                    "Not checked · choose Check example to evaluate the practice files."
                } else {
                    "Not checked · choose Check work to evaluate the assignment files."
                });
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.links, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).practice;
                    candidate.example_command = 0;
                }
            }
            ui.push_id(("links-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and supported command subset").show(
                    ui,
                    |ui| {
                        lesson_presentation::prose(ui, work::HELP);
                        if !practice {
                            let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                            let r = ui.add_enabled(
                                active && attempt.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            record(&mut self.metrics.links, "hint", &r, ui);
                            if r.clicked() {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(attempt.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Worked approach · reveals a solution")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &source.explanation);
                                    for command in &source.solution {
                                        ui.monospace(command);
                                    }
                                });
                        }
                        for link in &source.sources {
                            ui.hyperlink_to(link, link);
                        }
                    },
                );
            });
        }
        if let Some(message) = &self.notice {
            ui.label(message);
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    a.state = engine::State::Links {
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
