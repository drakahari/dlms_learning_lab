//! Native supplied print-queue terminal; actual investigation, no host commands.
use super::*;
use dlms_learning_lab::print_queue_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub summary: Option<String>,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn code(ui: &mut egui::Ui, text: &str) {
    lesson_presentation::monospace(ui, text, true);
}
#[derive(Clone, Copy)]
struct Lesson {
    practice: bool,
    active: bool,
}
fn guidance(ui: &mut egui::Ui, s: &pack::Scenario, step: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, s, step);
    }
}
fn terminal(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::session::Draft,
    lesson: Lesson,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Lesson { practice, active } = lesson;
    if practice {
        let id = ui.make_persistent_id("print-worked-command");
        let mut selected = ui
            .ctx()
            .data(|d| d.get_temp::<usize>(id))
            .unwrap_or(0)
            .min(task.reference.len().saturating_sub(1));
        let selected_text = task
            .reference
            .get(selected)
            .map(|c| c.0.as_str())
            .unwrap_or("No worked command");
        let menu = controls::dropdown(ui, "print-command-menu", ui.available_width())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, command) in task.reference.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, index, &command.0);
                    metrics.record(ui, &format!("worked-{index}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for entry in &mut metrics.controls {
                    if entry.0.starts_with("worked-") {
                        entry.2 = clip;
                    }
                }
            });
        metrics.record(ui, "worked-menu", &menu.response);
        ui.ctx().data_mut(|d| d.insert_temp(id, selected));
        let r = ui.add_enabled(active, button("Insert command"));
        metrics.record(ui, "insert", &r);
        if r.clicked()
            && let Some(command) = task.reference.get(selected)
        {
            draft.input = command.clone();
        }
        ui.label("Fills the input only. Run command executes it in this simulated workspace.");
    }
    let label = ui.strong("Simulated terminal");
    let mut input = draft.input.0.clone();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut input, ui.available_width())
                .hint_text("Command from Supported commands")
                .char_limit(1024),
        )
        .labelled_by(label.id);
    metrics.record(ui, "command", &r);
    if r.changed() {
        if input.len() <= 1024 {
            draft.input = work::Command(input);
        } else {
            *notice = Some("Keep one command within1024bytes.".into());
        }
    }
    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let r = ui.add_enabled(active, button("Run command"));
    metrics.record(ui, "run", &r);
    if r.clicked() || active && enter {
        match draft.run(task) {
            Ok(_) => *notice = None,
            Err(error) => *notice = Some(error),
        }
    }
    let output = ui.scope(|ui| {
        ui.strong("Actual last run");
        if let Some(run) = &draft.last_run {
            code(ui, &format!("$ {}", run.command.0));
        }
        match draft.last_output(task) {
            Ok(None) => {
                ui.label("No command has run in this draft yet.");
            }
            Ok(Some(Ok(output))) => {
                ui.label(format!("Command finished · exit status {}", output.status));
                let height = ui.text_style_height(&egui::TextStyle::Monospace) * 12.;
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt("print-actual-output")
                        .max_height(height)
                        .auto_shrink([false, true]),
                    |ui| {
                        if output.stdout.is_empty() {
                            ui.label("No standard output.");
                        } else {
                            lesson_presentation::monospace(ui, &output.stdout, false);
                        }
                    },
                );
            }
            Ok(Some(Err(error))) => {
                let heading = match error.kind {
                    dlms_learning_lab::boot_entry::ErrorKind::Syntax => "Command syntax",
                    dlms_learning_lab::boot_entry::ErrorKind::Unsupported => {
                        "Outside the supported subset"
                    }
                    dlms_learning_lab::boot_entry::ErrorKind::Limit => "Simulator limit",
                    dlms_learning_lab::boot_entry::ErrorKind::Invalid => {
                        "Cannot perform this command"
                    }
                };
                ui.colored_label(palette(ui).error, format!("{heading}: {error}"));
            }
            Err(error) => {
                ui.colored_label(palette(ui).error, error);
            }
        }
    });
    metrics.record(ui, "last-result", &output.response);
}
fn provided(ui: &mut egui::Ui, t: &work::Task) {
    ui.strong("Supplied documents · /work");
    for document in &t.facts.documents {
        controls::form_row(ui, &document.title, |ui, _| {
            lesson_presentation::monospace(ui, &document.path, true)
        });
    }
}
fn inspect(ui: &mut egui::Ui, t: &work::Task, d: &work::session::Draft) {
    controls::collapsing_header(ui, "Inspect simulated queues and requests · optional").show(
        ui,
        |ui| match d.world(t) {
            Ok(world) => {
                ui.strong("Current printer controls");
                for p in &world.printers {
                    code(
                        ui,
                        &format!(
                            "{} · {} · {}",
                            p.name.0,
                            if p.accepting {
                                "Accepting requests"
                            } else {
                                "Rejecting requests"
                            },
                            if p.enabled {
                                "Enabled for printing"
                            } else {
                                "Stopped"
                            }
                        ),
                    );
                }
                ui.label(format!(
                    "Local default: {}",
                    world
                        .local_default
                        .as_ref()
                        .map(|p| p.0.as_str())
                        .unwrap_or("None selected")
                ));
                ui.strong("Current requests");
                for j in &world.jobs {
                    code(
                        ui,
                        &format!(
                            "{}-{} · {:?} · {} copies · {}",
                            j.printer.0, j.id, j.state, j.copies, j.document
                        ),
                    );
                }
            }
            Err(error) => {
                lesson_presentation::prose(ui, &error);
            }
        },
    );
}
fn feedback(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &work::session::Draft,
    practice: bool,
    m: &mut Metrics,
) {
    if !d.attempted {
        let r = ui.label("Not checked");
        m.record(ui, "result", &r);
        return;
    }
    let rows = match d.checks(t) {
        Ok(rows) => rows,
        Err(error) => {
            ui.label(error);
            return;
        }
    };
    let failed = rows.iter().filter(|r| !r.correct).count();
    let summary = if failed == 0 {
        if practice {
            "Example requirements met".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs attention · {failed} {}",
            if failed == 1 {
                "requirement fails"
            } else {
                "requirements fail"
            }
        )
    };
    m.summary = Some(summary.clone());
    let r = lesson_presentation::rich(
        ui,
        RichText::new(summary).color(if failed == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    let draw = |ui: &mut egui::Ui, index: usize, row: work::Check, m: &mut Metrics| {
        let r = ui.push_id(index, |ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}: {}",
                    if row.correct {
                        "Meets requirement"
                    } else {
                        "Needs attention"
                    },
                    row.label,
                    row.feedback
                ))
                .color(if row.correct {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            if !row.correct
                && let (Some(expected), Some(actual)) = (row.expected, row.actual)
            {
                controls::collapsing_header(ui, "Expected versus actual sample").show(ui, |ui| {
                    ui.strong("Expected");
                    code(ui, &expected);
                    ui.strong("Actual");
                    code(ui, &actual);
                });
            }
        });
        m.record(ui, &format!("feedback-{index}"), &r.response);
    };
    for (index, row) in rows.iter().enumerate().filter(|(_, r)| !r.correct) {
        draw(ui, index, row.clone(), m);
    }
    let passed = rows.len() - failed;
    if passed > 0 {
        let r = controls::collapsing_header(ui, &format!("Show {passed} passing checks")).show(
            ui,
            |ui| {
                for (index, row) in rows.into_iter().enumerate().filter(|(_, r)| r.correct) {
                    draw(ui, index, row, m);
                }
            },
        );
        m.record(ui, "passing-checks", &r.header_response);
    }
}
impl LabApp {
    pub(super) fn print_queue_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let def =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(error) => {
                    self.notice = Some(error);
                    return;
                }
            };
        let engine::State::PrintQueues { simulation } = &attempt.state else {
            return;
        };
        let before = (**simulation).clone();
        let mut next = before.clone();
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
        self.metrics.print_queues = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.print_queues.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8),|ui|{
            if view==teaching::View::Introduction {
                lesson_presentation::concepts(ui,&source,"Builds on the file and terminal foundations. All printers and documents are supplied in this activity.",Vec::<(String,String)>::new());
                lesson_presentation::worked(ui,"Different supplied printers, requests and documents demonstrate the method before independent work.");for (number,response) in lesson_presentation::worked_command_rows(ui,&source).iter().enumerate(){self.metrics.print_queues.record(ui,&format!("worked-value-{number}"),response);}
                ui.horizontal_wrapped(|ui|{for (choice,key,label) in [(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.print_queues.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let (task,draft)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    provided(ui,task);ui.separator();
                    if practice&&ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*48. {
                      ui.columns(2,|columns|{guidance(&mut columns[0],&source,0,true);guidance(&mut columns[0],&source,1,true);terminal(&mut columns[1],task,draft,Lesson{practice,active},&mut self.metrics.print_queues,&mut self.notice);});
                    }else{guidance(ui,&source,0,practice);guidance(ui,&source,1,practice);terminal(ui,task,draft,Lesson{practice,active},&mut self.metrics.print_queues,&mut self.notice);}
                    inspect(ui,task,draft);
                    ui.separator();guidance(ui,&source,2,practice);
                    let r=ui.add_enabled(active,button(if practice{"Check example work"}else{"Check print work"}).fill(palette(ui).accent));self.metrics.print_queues.record(ui,"check",&r);
                    if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,practice,&mut self.metrics.print_queues);
                    if practice{let r=ui.add_enabled(active,button("Reset example only"));self.metrics.print_queues.record(ui,"reset-example",&r);if r.clicked(){*draft=work::session::Draft::new(task);self.notice=None;}}
                    controls::collapsing_header(ui,"Supported commands and help · optional").show(ui,|ui|{
                        lesson_presentation::prose(ui,work::HELP);let attempt=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&attempt.hints<source.hints.len(),button("Show a hint"));if r.clicked(){attempt.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}lesson_presentation::prose(ui,&source.simplifications);controls::collapsing_header(ui,"Worked approach · reveals one method").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                    });
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::PrintQueues {
                    simulation: Box::new(next),
                };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, i);
        }
    }
}
