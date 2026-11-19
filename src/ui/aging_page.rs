//! Focused virtual aging terminal and supplied dated observations.
use super::*;
use dlms_learning_lab::{aging, aging_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub input: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn observation(o: aging::Observation) -> String {
    let mut s = o.verdict.label().to_string();
    if let Some(n) = o.warning_days {
        s.push_str(&format!(" · warning: {n} days left"));
    }
    if o.change_too_soon {
        s.push_str(" · too soon to change password");
    }
    if o.future_last_change {
        s.push_str(" · recorded change is in the future");
    }
    s
}
fn dated_table(ui: &mut egui::Ui, task: &work::Task, scope: &work::Scope) {
    lesson_presentation::rich(
        ui,
        RichText::new("Supplied dates and policy observations").strong(),
    );
    lesson_presentation::prose(ui, aging::POLICY);
    let rows = task.observations(&scope.world).unwrap();
    let width = (ui.available_width() - 2. * ui.spacing().item_spacing.x).max(1.);
    let date_width = (ui.text_style_height(&egui::TextStyle::Monospace) * 6.6).min(width * 0.29);
    let state_width = ((width - date_width) * 0.5).max(1.);
    egui::Grid::new(("aging-dates", &task.id))
        .spacing(egui::vec2(12., 10.))
        .show(ui, |ui| {
            for (label, w) in [
                ("UTC date", date_width),
                ("Expected", state_width),
                ("Actual", state_width),
            ] {
                controls::cell(ui, w, |ui| {
                    ui.label(RichText::new(label).strong());
                });
            }
            ui.end_row();
            for (case, actual) in rows {
                controls::cell(ui, date_width, |ui| {
                    ui.vertical(|ui| {
                        lesson_presentation::monospace(ui, &case.day.iso(), true);
                        ui.label(&case.label);
                    });
                });
                controls::cell(ui, state_width, |ui| {
                    lesson_presentation::prose(ui, &observation(case.expected));
                });
                controls::cell(ui, state_width, |ui| {
                    ui.vertical(|ui| {
                        if scope.checked {
                            lesson_presentation::prose(ui, &observation(actual));
                            ui.label(
                                RichText::new(if actual == case.expected {
                                    "Matches"
                                } else {
                                    "Needs work"
                                })
                                .color(
                                    if actual == case.expected {
                                        palette(ui).success
                                    } else {
                                        palette(ui).error
                                    },
                                ),
                            );
                        } else {
                            lesson_presentation::prose(ui, "Not checked");
                        }
                    });
                });
                ui.end_row();
            }
        });
}
impl LabApp {
    pub(super) fn aging_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::Aging { definition }, engine::State::Aging { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
        let visited = a.visited;
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
                    record(&mut self.metrics.aging, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
            candidate.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on local account administration. No passwords or actual login changes.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.aging, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            let task = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let m = &mut self.metrics.aging;
            let mut terminal = |ui: &mut egui::Ui,
                                scope: &mut work::Scope,
                                m: &mut Metrics,
                                focus: &mut bool| {
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!("Virtual administrator · target {}", task.target))
                        .strong(),
                );
                let r = ui.add_enabled(active, button("Insert record query"));
                record(m, "query", &r, ui);
                if r.clicked() {
                    scope.input = format!("chage -l {}", task.target);
                    *focus = true;
                }
                let height =
                    (ui.text_style_height(&egui::TextStyle::Monospace) * 12.).clamp(180., 420.);
                let output = controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt(("aging-terminal", &source.id, practice, visited))
                        .max_height(height)
                        .min_scrolled_height(height)
                        .stick_to_bottom(true),
                    |ui| {
                        ui.set_min_width(ui.available_width());
                        if scope.transcript.is_empty() {
                            ui.label(RichText::new("Use Insert record query, then Run command, to inspect the supplied aging fields.").color(palette(ui).muted));
                        }
                        for line in &scope.transcript {
                            ui.label(RichText::new(&line.text).monospace().color(if line.error {
                                palette(ui).error
                            } else {
                                palette(ui).text
                            }));
                        }
                    },
                );
                self.metrics.output = Some(output.inner_rect);
                if practice {
                    lesson_presentation::practice_step(ui, &source, 1);
                }
                ui.label("Command · one virtual chage command");
                let width = ui.available_width();
                let r = ui.add_enabled(
                    active,
                    controls::singleline_fitting(ui, &mut scope.input, width)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("chage OPTION VALUE ACCOUNT")
                        .id_salt(("aging-input", &source.id, practice, visited)),
                );
                name_control(&r, "Virtual aging command");
                m.input = Some((r.rect, r.id));
                if r.changed() {
                    self.notice = None;
                }
                if *focus
                    && active
                    && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released())
                {
                    r.request_focus();
                    *focus = false;
                }
                let enter = active
                    && (r.has_focus() || r.lost_focus())
                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if r.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                    r.surrender_focus();
                }
                workspace::bound(&mut scope.input, aging::MAX_COMMAND);
                let run = ui.add_enabled(active, primary(ui, "Run command"));
                record(m, "run", &run, ui);
                if run.clicked() || enter {
                    if let Err(e) = scope.execute(task) {
                        self.notice = Some(e);
                    } else {
                        self.notice = None;
                    }
                    *focus = true;
                }
            };
            // Terminal and dated policy table use the same content-driven breakpoint.
            // Multiline output stays top aligned; single-line input uses shared metrics.
            let mut focus = self.focus_input;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    terminal(&mut columns[0], scope, m, &mut focus);
                    dated_table(&mut columns[1], task, scope);
                });
            } else {
                terminal(ui, scope, m, &mut focus);
                ui.separator();
                dated_table(ui, task, scope);
            }
            self.focus_input = focus;
            let r = ui.add_enabled(active, button("Check aging policy"));
            record(&mut self.metrics.aging, "check", &r, ui);
            if r.clicked()
                && let Err(e) = scope.check(task)
            {
                self.notice = Some(e);
            }
            if scope.checked {
                ui.separator();
                let rows = task.checks(&scope.world);
                let passed = rows.iter().filter(|r| r.0).count();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(if passed == rows.len() {
                        "Aging policy meets every requirement"
                    } else {
                        "Aging policy needs work"
                    })
                    .strong()
                    .color(if passed == rows.len() {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                );
                self.metrics.aging.result = Some((passed, rows.len(), r.rect));
                for (_, message) in rows.iter().take(7).filter(|r| !r.0) {
                    lesson_presentation::prose(ui, message);
                }
                if rows.iter().skip(7).any(|r| !r.0) {
                    lesson_presentation::prose(
                        ui,
                        "Review the rows marked Needs work above; Expected and Actual show the dated differences.",
                    );
                }
                if passed == rows.len() {
                    lesson_presentation::prose(
                        ui,
                        "The record fields, dated outcomes and protected accounts match the task. This does not authenticate anyone or perform an actual password change.",
                    );
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    "Not checked · run your commands, then compare the current policy.",
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                controls::collapsing_header(ui,"Worked practice commands · optional").show(ui,|ui|{
                    for(n,c)in definition.example_worked.iter().enumerate(){
                        lesson_presentation::monospace(ui,&c.0,false);let r=ui.add_enabled(active,button("Insert command"));record(&mut self.metrics.aging,&format!("worked-{n}"),&r,ui);
                        if r.clicked(){scope.input=c.0.clone();self.focus_input=true;}
                    }
                    lesson_presentation::prose(ui,"Insert only fills the command field. Choose Run command to apply it to practice.");
                });
                let r = ui.add_enabled(active, button("Reset example"));
                record(&mut self.metrics.aging, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("aging-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, aging::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.aging, "hint", &r, ui);
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
                                for answer in &source.solution {
                                    if let Some(raw) = answer.strip_prefix(work::PREFIX)
                                        && let Ok(commands) =
                                            serde_json::from_str::<Vec<work::Command>>(raw)
                                    {
                                        for c in commands {
                                            lesson_presentation::monospace(ui, &c.0, false);
                                        }
                                    }
                                }
                            });
                    }
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::Aging {
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
