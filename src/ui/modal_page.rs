//! Guided native modal-editing lessons with independent saved-file assignments.
use super::*;
use dlms_learning_lab::{modal_source, modal_workspace::Event};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Rect, egui::Id)>,
    pub cursor: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
    pub popup: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response) {
    m.controls.push((key.into(), r.rect));
}
impl LabApp {
    pub(super) fn modal_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let s = self.scenarios[i].clone();
        let original = &self.progress.attempts[&s.id];
        if let Err(e) = modal_source::context(original, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::Goal::ModalEditing { definition } = &s.goal else {
            return;
        };
        let engine::State::ModalEditing { simulation } = &original.state else {
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
        lesson_presentation::rich(ui, RichText::new(&s.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    record(&mut self.metrics.modal, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &s, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(
                ui,
                "Use the supplied example first; it has different settings or records from the independent task.",
            );
            lesson_presentation::worked_rows(ui, &s);
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active, primary(ui, "Try this example"));
                record(&mut self.metrics.modal, "example", &r);
                self.metrics.example_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Example;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
                let r = ui.add_enabled(active, button("Go straight to assignment"));
                record(&mut self.metrics.modal, "assignment", &r);
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if practice {
                lesson_presentation::practice_step(ui, &s, 0);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(active, button("Show worked answer"));
                    record(&mut self.metrics.modal, "worked", &r);
                    if r.clicked() {
                        if let Err(e) = next.show_worked(definition) {
                            self.notice = Some(e);
                        }
                        self.focus_input = true;
                    }
                    let r = ui.add_enabled(active, button("Reset example only"));
                    record(&mut self.metrics.modal, "reset-example", &r);
                    if r.clicked() {
                        if let Err(e) = next.reset_practice(definition) {
                            self.notice = Some(e);
                        }
                        self.focus_input = true;
                    }
                });
            } else {
                lesson_presentation::assignment(ui, &s);
            }
            let (f, a) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            let message_id = ui.make_persistent_id(("modal-feedback", &s.id, practice));
            let mut message = ui.ctx().data(|d| d.get_temp::<String>(message_id));
            ui.add_enabled_ui(active,|ui|{
                ui.push_id(("modal-workspace",&s.id,practice),|ui|{
                    let choice_id=ui.make_persistent_id("file-choice");
                    let mut choice=ui.ctx().data(|d|d.get_temp::<String>(choice_id)).filter(|p|f.files.contains_key(p)).unwrap_or_else(||a.selected.clone());
                    controls::form_row(ui,"Virtual file",|ui,width|{
                        let r=egui::ComboBox::from_id_salt("modal-file").selected_text(&choice).width(width).show_ui(ui,|ui|{
                            for(path,file)in &f.files {let r=ui.selectable_value(&mut choice,path.clone(),format!("{path}{}",if file.writable{""}else{" · read-only"}));record(&mut self.metrics.modal,&format!("file:{path}"),&r);}
                            self.metrics.modal.popup=Some(controls::popup_content_clip(ui));
                        }).response;record(&mut self.metrics.modal,"file",&r);
                    });
                    ui.ctx().data_mut(|d|d.insert_temp(choice_id,choice.clone()));
                    let r=ui.add(button("Open selected file"));record(&mut self.metrics.modal,"open",&r);
                    if r.clicked(){match a.apply(f,&Event::Open{path:choice}){Ok(())=>{message=None;self.focus_input=true;},Err(e)=>message=Some(e)}}
                    if practice{lesson_presentation::practice_step(ui,&s,1);}
                    lesson_presentation::prose(ui,"Click the editor to type. Escape returns to NORMAL; Tab moves to app controls.");
                    let out=modal_editor::show(ui,a,f,&mut self.focus_input);
                    self.metrics.modal.editor=out.surface;self.metrics.modal.cursor=out.cursor;
                    if out.changed {message=None;}
                    if let Some(e)=out.diagnostic {message=Some(e);}
                    controls::collapsing_header(ui,"Actual saved file · inspect what was written").show(ui,|ui|{
                        lesson_presentation::code_value(ui,("modal-saved-file",&a.selected),a.editor().file_bytes(),ui.available_width());
                    });
                });
            });
            if let Some(e) = &message {
                lesson_presentation::prose(ui, &format!("Editor feedback · {e}"));
            }
            ui.ctx().data_mut(|d| {
                if let Some(message) = message {
                    d.insert_temp(message_id, message);
                } else {
                    d.remove::<String>(message_id);
                }
            });
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check saved files"
                    },
                ),
            );
            record(&mut self.metrics.modal, "check", &r);
            self.metrics.check = Some(r.rect);
            if r.clicked()
                && let Err(e) = a.apply(f, &Event::Check {})
            {
                self.notice = Some(e);
            }
            if a.checked {
                let rows = a.checks(f);
                let pass = rows.iter().all(|r| r.0);
                let r = ui
                    .group(|ui| {
                        ui.label(
                            RichText::new(if pass {
                                if practice {
                                    "Example checks pass"
                                } else {
                                    "Assignment complete"
                                }
                            } else {
                                "Needs attention"
                            })
                            .strong()
                            .color(if pass {
                                palette(ui).success
                            } else {
                                palette(ui).error
                            }),
                        );
                        if pass {
                            lesson_presentation::prose(ui, &s.explanation);
                        } else {
                            for (_, text) in rows.iter().filter(|r| !r.0) {
                                lesson_presentation::prose(ui, text);
                            }
                        }
                        controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
                            for (pass, text) in &rows {
                                lesson_presentation::prose(
                                    ui,
                                    &format!(
                                        "{} · {text}",
                                        if *pass { "Verified" } else { "Needs attention" }
                                    ),
                                );
                            }
                        });
                    })
                    .response;
                self.metrics.modal.output = Some(r.rect);
            }
            if practice {
                controls::collapsing_header(ui, "Explore after practice").show(ui, |ui| {
                    lesson_presentation::practice_step(ui, &s, 2);
                });
                let r = ui.add_enabled(active, button("Open full assignment"));
                record(&mut self.metrics.modal, "assignment", &r);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Optional help and supported keys").show(ui, |ui| {
                lesson_presentation::prose(ui, modal_source::HELP);
                if !practice {
                    let a = self.progress.attempts.get_mut(&s.id).unwrap();
                    if ui
                        .add_enabled(active && a.hints < s.hints.len(), button("Show a hint"))
                        .clicked()
                    {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for h in s.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, h);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals reasoning").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &s.explanation);
                        },
                    );
                }
                for link in &s.sources {
                    ui.hyperlink_to(link, link);
                }
            });
        }
        if next != before && active {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::ModalEditing {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&s.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&s.id], &s);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
