//! Native virtual services editor plus the retained router terminal and trials.
use super::linux_v6_page as base;
use super::*;
use dlms_learning_lab::{
    linux_service_router as work, linux_service_router_source as binding,
    linux_v6_workspace as router,
};
fn apply(s: &mut work::Scope, t: &work::Task, a: &work::Action, n: &mut Option<String>) {
    if let Err(e) = s.apply(t, a) {
        *n = Some(e);
    }
}
fn show_worked(ui: &mut egui::Ui, source: &Scenario, d: &work::Definition) {
    for a in &d.worked {
        match a {
            work::Action::Edit { text } => {
                ui.strong("Worked /etc/services · example only");
                lesson_presentation::monospace(ui, text, true);
            }
            work::Action::Save => {
                lesson_presentation::prose(
                    ui,
                    "Choose Save virtual file. This changes future lookups; installed rules keep their old numbers.",
                );
            }
            work::Action::Router {
                action: router::Action::Command { line },
            } => {
                lesson_presentation::monospace(ui, line, true);
                if let Some(c) = source
                    .guide
                    .iter()
                    .flat_map(|g| &g.commands)
                    .find(|c| c.command == *line)
                {
                    lesson_presentation::prose(ui, &c.explanation);
                }
            }
            work::Action::Router {
                action: router::Action::Check,
            } => {
                lesson_presentation::prose(ui, "Run the traffic tests, then choose Check example.");
            }
            _ => {}
        }
        ui.add_space(6.);
    }
}
fn file_editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut base::Metrics,
    notice: &mut Option<String>,
) {
    let label = ui.strong("/etc/services · simulated file");
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 9. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let mut value = s.editor.clone();
    let error_key = egui::Id::new(("router-services-save-error", &source.id, practice));
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("router-services-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut value)
                        .id_salt(("router-services-editor", &source.id, practice))
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(9)
                        .char_limit(32768),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let response = scrolled.inner.labelled_by(label.id);
    m.fields.push(("editor".into(), response.rect, response.id));
    m.controls.push((
        "editor".into(),
        response.rect.intersect(scrolled.inner_rect),
        ui.clip_rect(),
    ));
    m.focus.push(("editor".into(), response.id));
    if response.changed() {
        apply(s, t, &work::Action::Edit { text: value }, notice);
        ui.ctx()
            .data_mut(|d| d.remove::<(String, String)>(error_key));
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save virtual file"));
        m.record(ui, "save", &r);
        if r.clicked() {
            match s.apply(t, &work::Action::Save) {
                Ok(()) => {
                    ui.ctx()
                        .data_mut(|d| d.remove::<(String, String)>(error_key));
                }
                Err(e) => {
                    let draft = s.editor.clone();
                    ui.ctx().data_mut(|d| d.insert_temp(error_key, (draft, e)));
                }
            }
        }
        let r = ui.label(if s.editor == s.saved_services {
            "Saved"
        } else {
            "Unsaved changes"
        });
        m.record(ui, "file-state", &r);
    });
    if let Some((_, error)) = ui
        .ctx()
        .data(|d| d.get_temp::<(String, String)>(error_key))
        .filter(|(draft, _)| *draft == s.editor)
    {
        let r = lesson_presentation::prose(ui, &error);
        m.record(ui, "save-error", &r);
    }
    if practice {
        let worked =
            controls::collapsing_header(ui, "Show worked file · example only").show(ui, |ui| {
                lesson_presentation::monospace(ui, &t.required_services, true);
                let r = ui.add(button("Fill worked file"));
                m.record(ui, "fill-file", &r);
                if r.clicked() {
                    apply(
                        s,
                        t,
                        &work::Action::Edit {
                            text: t.required_services.clone(),
                        },
                        notice,
                    );
                    ui.ctx()
                        .data_mut(|d| d.remove::<(String, String)>(error_key));
                }
                lesson_presentation::prose(
                    ui,
                    "Fill changes this example draft only. Save it when ready to use the mapping.",
                );
            });
        m.record(ui, "worked-file-toggle", &worked.header_response);
    }
}
fn terminal_and_traffic(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut base::Metrics,
    notice: &mut Option<String>,
    phase: (bool, &mut bool),
) {
    let (practice, focus) = phase;
    let saved = &s.saved_services;
    base::console_with(
        ui,
        &mut s.router,
        source,
        m,
        notice,
        (practice, focus, 1),
        &mut |scope, a, n| {
            if let Err(e) = work::router_action(scope, &t.router, saved, a) {
                *n = Some(e);
            }
        },
    );
    if practice {
        lesson_presentation::practice_step(ui, source, 2);
    }
    base::traffic(
        ui,
        &t.router,
        &mut s.router,
        &source.id,
        practice,
        m,
        notice,
    );
}

impl LabApp {
    pub(super) fn services_router_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (
            engine::Goal::LinuxServiceRouter { definition },
            engine::State::LinuxServiceRouter { simulation },
        ) = (&source.goal, &original.state)
        else {
            return;
        };
        let first = egui::Id::new(("services-router-first", &source.id, original.visited));
        if !ui
            .ctx()
            .data(|d| d.get_temp::<bool>(first))
            .unwrap_or(false)
        {
            if let Err(e) = binding::context(original, &source) {
                lesson_presentation::prose(ui, &e);
                return;
            }
            self.focus_input = false;
            ui.ctx().data_mut(|d| d.insert_temp(first, true));
        }
        if let Err(e) = definition.bound(simulation) {
            lesson_presentation::prose(ui, &e);
            return;
        }
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.services_router.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .services_router
                .record(ui, "introduction", &intro.objective);
            lesson_presentation::worked(
                ui,
                "Separate worked file and router example · use Try example to investigate and change it.",
            );
            let worked = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics
                .services_router
                .record(ui, "worked", &worked.response);
            controls::collapsing_header(ui, "Complete worked command sequence · example only")
                .show(ui, |ui| {
                    show_worked(ui, &source, definition);
                });
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.services_router.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics.services_router.record(ui, "assignment", &r);
            }
            ui.add_enabled_ui(active, |ui| {
                let r=base::topology(ui,&t.router);self.metrics.services_router.record(ui,"topology",&r);
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54. {
                    ui.columns(2,|columns|{
                        file_editor(&mut columns[0],t,s,&source,practice,&mut self.metrics.services_router,&mut self.notice);
                        terminal_and_traffic(&mut columns[1],t,s,&source,&mut self.metrics.services_router,&mut self.notice,(practice,&mut self.focus_input));
                    });
                }else{
                    file_editor(ui,t,s,&source,practice,&mut self.metrics.services_router,&mut self.notice);
                    terminal_and_traffic(ui,t,s,&source,&mut self.metrics.services_router,&mut self.notice,(practice,&mut self.focus_input));
                }
                let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));self.metrics.services_router.record(ui,"check",&r);
                if r.clicked(){apply(s,t,&work::Action::Router{action:router::Action::Check},&mut self.notice);}
                if s.router.checked {
                    let checks = t.checks(s);
                    let passed = checks.iter().filter(|c| c.passed).count();
                    let r = ui.strong(if passed == checks.len() {
                        if practice { "Example configuration checks passed" } else { "Assignment complete" }.into()
                    } else {
                        format!("Not complete · {passed}/{} outcomes pass", checks.len())
                    });
                    self.metrics.services_router.result = Some((passed, checks.len(), r.rect));
                    self.metrics.services_router.record(ui, "result", &r);
                    for c in checks.iter().filter(|c| !c.passed) {
                        let r = lesson_presentation::prose(ui, &format!("Needs work · {}\n{}", c.label, c.detail));
                        self.metrics.services_router.record(ui, "failed-case", &r);
                    }
                    controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
                        for c in checks.iter().filter(|c| c.passed) {
                            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
                        }
                    });
                    if passed == checks.len() {
                        lesson_presentation::prose(ui, if practice {
                            "The saved example mappings and current rules now meet the file, policy and traffic requirements. "
                        } else { &source.explanation });
                    }
                }
            });
            controls::collapsing_header(ui, "Optional help and supported commands").show(
                ui,
                |ui| {
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        if ui
                            .add_enabled(
                                active && a.hints < source.hints.len(),
                                button("Show a hint"),
                            )
                            .clicked()
                        {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
                    } else {
                        controls::collapsing_header(ui, "Show worked answer · example only").show(
                            ui,
                            |ui| {
                                show_worked(ui, &source, definition);
                            },
                        );
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LinuxServiceRouter {
                            simulation: Box::new(next),
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
