//! DKMS investigation with a real finite command evaluator and separate practice.
use super::*;
use dlms_learning_lab::{dkms, dkms_source as binding, dkms_workspace as work, engine::State};
type Metrics = super::linux_v6_page::Metrics;
fn apply(t: &work::Task, w: &mut work::Work, a: work::Action, notice: &mut Option<String>) {
    if let Err(e) = t.apply(w, &a) {
        // Console failures already have an actual-result history entry.
        if !matches!(a, work::Action::Run { .. }) {
            *notice = Some(e);
        }
    }
}
fn supplied(ui: &mut egui::Ui, t: &work::Task, m: &mut Metrics) {
    let r = ui.scope(|ui| {
        ui.strong("Supplied kernels and build-ready sources");
        for (n, k) in t.facts.kernels.iter().enumerate() {
            controls::form_row(
                ui,
                if n == t.facts.running {
                    "Running kernel"
                } else {
                    "Other supplied kernel"
                },
                |ui, _| {
                    lesson_presentation::monospace(ui, &k.release, true);
                },
            );
        }
        ui.label(format!(
            "Architecture: {} · installation is simulated; drivers are not loaded",
            t.facts.architecture
        ));
        for (n, s) in t.facts.sources.iter().enumerate() {
            controls::form_row(ui, "Source available", |ui, _| {
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "{}/{} · {}",
                        s.name,
                        s.version,
                        t.facts.source_path(n).unwrap_or_default()
                    ),
                    true,
                );
            });
        }
    });
    m.record(ui, "supplied", &r.response);
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    w: &mut work::Work,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("DKMS console · simulated commands");
    let id = egui::Id::new(("dkms-command", &t.id, practice));
    let r = shared_libraries_page::fitting_input(
        ui,
        &w.command_draft,
        id,
        2048,
        ui.is_enabled(),
        "One supported dkms command",
    );
    m.record(ui, "command", &r.response);
    m.fields
        .push(("command".into(), r.response.rect, r.response.id));
    if r.rejected {
        ui.label("The command exceeds the input limit; the previous draft is retained.");
    }
    if let Some(text) = r.value {
        apply(t, w, work::Action::CommandDraft { text }, notice);
    }
    let enter = r.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let run = ui.add(button("Run"));
    m.record(ui, "run", &run);
    if run.clicked() || enter {
        let input = w.command_draft.clone();
        apply(t, w, work::Action::Run { input }, notice);
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 9.;
    let r = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(("dkms-console", &t.id, practice))
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(true),
        |ui| {
            ui.strong("Actual console results");
            if w.log.is_empty() {
                ui.label("No operation has run. Start with dkms status.");
            }
            for line in &w.log {
                lesson_presentation::monospace(ui, &format!("$ {}", line.input), true);
                if !line.ok {
                    ui.strong("Operation refused · read the reason and retry");
                }
                lesson_presentation::monospace(
                    ui,
                    if line.output.is_empty() {
                        "(No matching registry entries)"
                    } else {
                        &line.output
                    },
                    true,
                );
            }
        },
    );
    let response = ui.interact(
        r.inner_rect,
        ui.id().with(("dkms-output", &t.id, practice)),
        egui::Sense::hover(),
    );
    m.record(ui, "console", &response);
}
fn results(ui: &mut egui::Ui, t: &work::Task, w: &work::Work, practice: bool, m: &mut Metrics) {
    let checks = t.checks(w);
    let passed = checks.iter().filter(|c| c.passed).count();
    let solved = t.solved(w);
    let r = ui.strong(if !w.checked {
        "Not checked · run your changes, then Check configuration".into()
    } else if solved {
        if practice {
            "Example verified · assignment remains separate".into()
        } else {
            "Assignment complete · all module requirements met".into()
        }
    } else {
        format!("Needs repair · {passed}/{} requirements met", checks.len())
    });
    m.record(ui, "result", &r);
    m.result = Some((passed, checks.len(), r.rect));
    if !w.checked {
        return;
    }
    for c in checks.iter().filter(|c| !c.passed) {
        let r = ui.scope(|ui| {
            ui.strong(format!("Needs repair · {}", c.label));
            lesson_presentation::prose(ui, &c.detail);
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
    .id_salt(("dkms-results", &t.id, practice))
    .show(ui, |ui| {
        for c in checks.iter().filter(|c| c.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
        }
    });
}
impl LabApp {
    pub(super) fn dkms_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[i].clone();
        let original = &self.progress.attempts[&scenario.id];
        let d = match binding::source(&scenario) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(e) = binding::context(original, &scenario) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let State::Dkms { simulation } = &original.state else {
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
        lesson_presentation::rich(ui, RichText::new(&scenario.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.dkms.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        if view == teaching::View::Introduction {
            let r =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            self.metrics
                .dkms
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(ui, "Different supplied module and kernels");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            self.metrics.dkms.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.dkms.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (t, s) = if practice {
                (&d.example, &mut next.example)
            } else {
                (&d.assignment, &mut next.assignment)
            };
            let w = &mut s.work;
            if !practice {
                let r = lesson_presentation::assignment(ui, &scenario);
                self.metrics.dkms.record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
                if practice {lesson_presentation::practice_step(ui,&scenario,0);}
                supplied(ui,t,&mut self.metrics.dkms);ui.separator();
                if practice {lesson_presentation::practice_step(ui,&scenario,1);}
                console(ui,t,w,practice,&mut self.metrics.dkms,&mut self.notice);
                if practice {lesson_presentation::practice_step(ui,&scenario,2);}
                let r=ui.add(primary(ui,"Check configuration"));self.metrics.dkms.record(ui,"check",&r);
                if r.clicked(){apply(t,w,work::Action::Check,&mut self.notice);}
                results(ui,t,w,practice,&mut self.metrics.dkms);
                if t.solved(w) {lesson_presentation::prose(ui,if practice {"Use the same lifecycle reasoning with the assignment’s different source/kernel facts."}else{&scenario.explanation});}
                if practice {let r=controls::collapsing_header(ui,"Show worked answer · example only").id_salt(("dkms-worked",&scenario.id)).show(ui,|ui|{
                    for a in &t.reference {if let work::Action::Run{input}=a {lesson_presentation::monospace(ui,input,true);}}
                    let r=ui.add(button("Apply worked example"));self.metrics.dkms.record(ui,"fill-example",&r);
                    if r.clicked(){let _=t.apply(w,&work::Action::Reset);for a in t.reference.iter().filter(|a|!matches!(a,work::Action::Check)){if let Err(e)=t.apply(w,a){self.notice=Some(e);break;}}}
                    ui.label("Resets this example and runs these supported operations. Inspect the actual results, then check it.");
                });self.metrics.dkms.record(ui,"worked-answer",&r.header_response);}
            });
            controls::collapsing_header(ui, "Help, supported commands and references")
                .id_salt(("dkms-help", &scenario.id))
                .show(ui, |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                            a.hints = (a.hints + 1).min(scenario.hints.len());
                            self.dirty = true;
                        }
                        for h in scenario
                            .hints
                            .iter()
                            .take(self.progress.attempts[&scenario.id].hints)
                        {
                            lesson_presentation::prose(ui, h);
                        }
                        controls::collapsing_header(ui, "Why does this work?").show(ui, |ui| {
                            lesson_presentation::prose(ui, &scenario.explanation)
                        });
                    }
                    lesson_presentation::prose(ui, dkms::HELP);
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    controls::collapsing_header(ui, "Virtual module files actually present").show(
                        ui,
                        |ui| {
                            for path in w.state.virtual_files(&t.facts).unwrap_or_default() {
                                lesson_presentation::monospace(ui, &path, true);
                            }
                        },
                    );
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }
        if active && next != before {
            match next.bind(d) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state = State::Dkms {
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
            self.next_unit_lab(ui, i);
        }
        Self::workspace_footer(ui);
    }
}
