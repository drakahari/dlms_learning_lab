//! A real setup-script / inherited-choice / saved-file composition, all simulated.
use super::*;
use dlms_learning_lab::{
    editor_handover as handover, editor_handover_source as binding, engine::State,
    environment_workspace as env, modal_workspace::Event,
};
pub(super) struct ActiveRun {
    scenario: String,
    practice: bool,
    visited: u64,
    automatic: bool,
    source: String,
    runner: env::Job,
}
fn record(m: &mut modal_page::Metrics, key: &str, r: &egui::Response) {
    m.controls.push((key.into(), r.rect));
}
impl LabApp {
    fn start_editor_handover(&mut self, i: usize, practice: bool, automatic: bool) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        if binding::context(a, s).is_err() {
            return;
        }
        let (State::EditorHandover { simulation }, engine::Goal::EditorHandover { definition }) =
            (&a.state, &s.goal)
        else {
            return;
        };
        let job = env::Job::new(
            simulation.task(definition, practice),
            &simulation.draft(practice).source,
        );
        match job {
            Ok(runner) => {
                let id = s.id.clone();
                let visited = a.visited;
                let source=simulation.draft(practice).source.clone();
                let State::EditorHandover { simulation } =
                    &mut self.progress.attempts.get_mut(&id).unwrap().state
                else {
                    unreachable!()
                };
                if !automatic {
                    if let Err(e) = simulation.begin_run(definition, practice) {
                        self.notice = Some(e);
                        return;
                    }
                    self.editor_handover_suppressed.remove(&(id.clone(),practice));
                }
                self.editor_handover_run = Some(ActiveRun {
                    scenario: id,
                    practice,
                    visited,
                    automatic,
                    source,
                    runner,
                });
                self.completion_index.forget(&s.id);
                self.dirty = true;
            }
            Err(e) => {
                if automatic {self.editor_handover_suppressed.insert((s.id.clone(),practice));}
                self.notice = Some(e);
            },
        }
    }
    pub(super) fn advance_editor_handover(&mut self, ctx: &egui::Context) {
        let allowed=!self.reset_pending && !self.settings_open && !self.theme_preview && !self.data_tools.modal_open();
        if let Some(job)=&self.editor_handover_run {
            let valid = allowed && self.scenarios.iter().find(|s|s.id==job.scenario).is_some_and(|s| {
                self.progress.attempts.get(&s.id).is_some_and(|a| {
                    a.visited == job.visited && binding::context(a, s).is_ok()
                        && matches!(&a.state,State::EditorHandover{simulation} if simulation.draft(job.practice).source==job.source)
                        && (job.automatic || (self.selected.is_some_and(|i|self.scenarios[i].id==s.id) && matches!(&a.state, State::EditorHandover { simulation } if
                            (if a.challenge { teaching::View::Assignment } else { simulation.view })
                            == if job.practice { teaching::View::Example } else { teaching::View::Assignment })))
                })
            });
            if !valid { self.editor_handover_run = None; }
        }
        if self.editor_handover_run.is_none() && allowed {
            let next=self.scenarios.iter().enumerate().find_map(|(i,s)|{
                let a=self.progress.attempts.get(&s.id)?;
                let (State::EditorHandover{simulation},engine::Goal::EditorHandover{definition})=(&a.state,&s.goal)else{return None;};
                if binding::context(a,s).is_err(){return None;}
                [false,true].into_iter().find(|p|!self.editor_handover_suppressed.contains(&(s.id.clone(),*p))&&simulation.draft(*p).needs_reverification(simulation.task(definition,*p))).map(|p|(i,p))
            });
            if let Some((i,practice))=next{self.start_editor_handover(i,practice,true);}
        }
        let Some(job)=self.editor_handover_run.as_mut()else{return;};
        job.runner.advance(16);
        if !job.runner.done() {
            ctx.request_repaint();
            return;
        }
        let job = self.editor_handover_run.take().unwrap();
        if let Some(result) = job.runner.record() {
            let Some(s) = self.scenarios.iter().find(|s| s.id == job.scenario) else {
                return;
            };
            let Some(a) = self.progress.attempts.get_mut(&job.scenario) else {
                return;
            };
            let (State::EditorHandover { simulation }, engine::Goal::EditorHandover { definition }) =
                (&mut a.state, &s.goal)
            else {
                return;
            };
            if a.visited == job.visited {
                let stored=if job.automatic{simulation.reverify_setup(definition,job.practice,result)}else{simulation.store_setup(definition,job.practice,result)};
                if let Err(e) = stored {
                    self.editor_handover_suppressed.insert((job.scenario.clone(),job.practice));
                    self.notice = Some(e);
                }
                self.completion_index.forget(&job.scenario);
                self.dirty = true;
            }
        } else {
            self.editor_handover_suppressed.insert((job.scenario,job.practice));
            self.notice = Some("Setup stopped at its execution limit. Your draft remains; no completion was awarded.".into());
        }
        ctx.request_repaint();
    }
    pub(super) fn editor_handover_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.editor_handover_run = None;
            return;
        }
        let s = self.scenarios[i].clone();
        let a = &self.progress.attempts[&s.id];
        if let Err(e) = binding::context(a, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (State::EditorHandover { simulation }, engine::Goal::EditorHandover { definition }) =
            (&a.state, &s.goal)
        else {
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
                        self.focus_input = true;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let mut run = false;
        let mut stop = false;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &s, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(
                ui,
                "A separate training handover shows the setup, the actual child choice and the saved-file change.",
            );
            lesson_presentation::worked_rows(ui, &s);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "example", "Try this example"),
                    (
                        teaching::View::Assignment,
                        "assignment",
                        "Go straight to assignment",
                    ),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.modal, key, &r);
                    if v == teaching::View::Example {
                        self.metrics.example_action = Some(r.rect);
                    } else {
                        self.metrics.assignment_action = Some(r.rect);
                    }
                    if r.clicked() {
                        next.view = v;
                        self.focus_input = true;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if practice {
                lesson_presentation::practice_step(ui, &s, 0);
            } else {
                lesson_presentation::assignment(ui, &s);
            }
            let running = self
                .editor_handover_run
                .as_ref()
                .is_some_and(|j| j.scenario == s.id && j.practice == practice);
            let task = next.task(definition, practice).clone();
            controls::collapsing_header(ui,"Sample environments · supplied inputs").default_open(true).show(ui,|ui|{
                if ui.available_width()>=720. {egui::Grid::new(("handover-inputs",&s.id,practice)).striped(true).show(ui,|ui|{
                    for label in ["Sample","PREFERRED_EDITOR","Initial EDITOR"] {ui.strong(label);}ui.end_row();
                    for c in &task.cases {ui.label(&c.input.name);ui.monospace(c.input.env.get("PREFERRED_EDITOR").map_or("not supplied",String::as_str));
                        ui.monospace(match c.input.env.get("EDITOR"){Some(v)if v.is_empty()=>"empty value",Some(v)=>v,None=>"not defined"});ui.end_row();}
                });}else{for c in &task.cases {ui.strong(&c.input.name);lesson_presentation::monospace(ui,&format!("PREFERRED_EDITOR: {}",c.input.env.get("PREFERRED_EDITOR").map_or("not supplied",String::as_str)),true);lesson_presentation::monospace(ui,&format!("Initial EDITOR: {}",match c.input.env.get("EDITOR"){Some(v)if v.is_empty()=>"empty value",Some(v)=>v,None=>"not defined"}),true);}}
                lesson_presentation::prose(ui,"TEAM and the other supplied values must stay unchanged. Each setup test starts with its own sample environment; the virtual handover file is edited separately below.");
            });
            ui.strong("Setup script · simulated shell");
            let mut source = next.draft(practice).source.clone();
            let height = (ui.text_style_height(&egui::TextStyle::Monospace) * 6. + 24.).max(120.);
            let area = controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(("handover-source-scroll", &s.id, practice))
                    .max_height(height)
                    .min_scrolled_height(height),
                |ui| {
                    ui.add_enabled_ui(active && !running, |ui| {
                        ui.add_sized(
                            [ui.available_width(), height],
                            egui::TextEdit::multiline(&mut source)
                                .id_salt(("handover-source", &s.id, practice))
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .char_limit(env::MAX_SOURCE),
                        )
                    })
                    .inner
                },
            );
            let input = area.inner;
            name_control(&input, "Setup script — simulated shell");
            self.metrics.inputs.push(input.rect);
            self.metrics.focus = Some(input.id);
            self.metrics.editor_viewport = Some(area.inner_rect);
            record(&mut self.metrics.modal, "setup-editor", &input);
            if self.focus_input && active && !running && !next.opened(definition, practice) {
                input.request_focus();
                self.focus_input = false;
            }
            if input.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                input.surrender_focus();
            }
            if input.changed() {
                match next.edit_setup(definition, practice, source) {
                    Ok(()) => self.editor_handover_run = None,
                    Err(e) => self.notice = Some(e),
                }
            }
            lesson_presentation::prose(
                ui,
                "Tab indents in the setup script; Escape releases its focus. Run setup tests the draft without opening a host program.",
            );
            if practice {
                lesson_presentation::practice_step(ui, &s, 1);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active && !running, primary(ui, "Run setup"));
                record(&mut self.metrics.modal, "run-setup", &r);
                run = r.clicked()
                    || (active
                        && !running
                        && input.has_focus()
                        && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)));
                let r = ui.add_enabled(active && running, button("Stop"));
                record(&mut self.metrics.modal, "stop-setup", &r);
                stop = r.clicked();
                if practice {
                    let r = ui.add_enabled(active && !running, button("Fill worked setup"));
                    record(&mut self.metrics.modal, "worked-setup", &r);
                    if r.clicked()
                        && let Err(e) =
                            next.edit_setup(definition, true, definition.example_setup.clone())
                    {
                        self.notice = Some(e);
                    }
                    let r = ui.add_enabled(active && !running, button("Reset example only"));
                    record(&mut self.metrics.modal, "reset-example", &r);
                    if r.clicked()
                        && let Err(e) = next.reset(definition, true)
                    {
                        self.notice = Some(e);
                    }
                }
            });
            if running {
                lesson_presentation::prose(
                    ui,
                    if self.editor_handover_run.as_ref().is_some_and(|j|j.automatic){"Checking saved setup against the sample environments before restoring current results… Stop keeps your draft."}else{"Running setup against the sample environments… Stop keeps your draft."},
                );
            } else {
                let rows = env::checks(&task, next.draft(practice));
                let ready = next.ready(definition, practice);
                lesson_presentation::prose(
                    ui,
                    if ready {
                        "Setup verified for every sample. Open the handover file next."
                    } else if next.draft(practice).result.is_some() {
                        "Setup needs attention. Repair the failing samples and run again."
                    } else {
                        "Setup not checked. Run setup to inspect the child's actual editor choice."
                    },
                );
                if !ready && next.draft(practice).result.is_some() {
                    for (_, why) in rows.iter().filter(|r| !r.0) {
                        lesson_presentation::prose(ui, why);
                    }
                }
                let panel = controls::collapsing_header(
                    ui,
                    "Actual setup results · output and inherited choice",
                )
                .show(ui, |ui| {
                    if let Some(result) = &next.draft(practice).result {
                        for (n, (case, row)) in task.cases.iter().zip(&result.results).enumerate() {
                            let verdict = if rows.len() == task.cases.len() {
                                if rows[n].0 {
                                    "Passed"
                                } else {
                                    "Needs attention"
                                }
                            } else {
                                "Not verified"
                            };
                            ui.strong(format!("{} · {verdict}", case.input.name));
                            lesson_presentation::prose(
                                ui,
                                &format!("Setup exit status: {}", row.parent.status),
                            );
                            if let Some(e) = &row.parent.error {
                                lesson_presentation::prose(ui, e);
                            }
                            let actual = row.bindings.as_ref().and_then(|b| b.child.get("EDITOR"));
                            lesson_presentation::prose(
                                ui,
                                &format!(
                                    "Child EDITOR: {}",
                                    actual.map_or("not inherited", String::as_str)
                                ),
                            );
                            if let Some(child) = &row.child {
                                lesson_presentation::code_value(
                                    ui,
                                    ("handover-actual-output", &case.input.name),
                                    &child.stdout,
                                    ui.available_width(),
                                );
                            }
                        }
                    } else {
                        lesson_presentation::prose(ui, "No actual run results yet.");
                    }
                });
                record(
                    &mut self.metrics.modal,
                    "actual-setup",
                    &panel.header_response,
                );
                if let Some(body) = panel.body_response {
                    record(&mut self.metrics.modal, "actual-setup-body", &body);
                }
                let r = ui.add_enabled(
                    active && ready,
                    button("Open handover file with selected editor"),
                );
                record(&mut self.metrics.modal, "open-handover", &r);
                if r.clicked() {
                    match next.open_editor(definition, practice) {
                        Ok(()) => self.focus_input = true,
                        Err(e) => self.notice = Some(e),
                    }
                }
            }
            if next.opened(definition, practice) {
                ui.separator();
                if practice {
                    lesson_presentation::practice_step(ui, &s, 2);
                }
                let launch = next.launch(practice).as_ref().unwrap();
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Opened by the simulated handover application: {} · sample “{}” (initial environment). The same bounded vi-style keys apply.",
                        launch.editor, launch.sample
                    ),
                );
                let (fixture, files) = if practice {
                    (&definition.editing.example, &mut next.files.practice)
                } else {
                    (&definition.editing.initial, &mut next.files.assignment)
                };
                let out = ui
                    .add_enabled_ui(active && !running, |ui| {
                        modal_editor::show(ui, files, fixture, &mut self.focus_input)
                    })
                    .inner;
                self.metrics.modal.editor = out.surface;
                self.metrics.modal.cursor = out.cursor;
                if let Some(e) = out.diagnostic {
                    lesson_presentation::prose(ui, &e);
                }
                controls::collapsing_header(ui, "Actual saved handover").show(ui, |ui| {
                    lesson_presentation::code_value(
                        ui,
                        ("handover-saved", &s.id, practice),
                        files.editor().file_bytes(),
                        ui.available_width(),
                    );
                });
            }
            let r = ui.add_enabled(
                active && !running,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check handover"
                    },
                ),
            );
            record(&mut self.metrics.modal, "check", &r);
            self.metrics.check = Some(r.rect);
            if r.clicked()
                && let Err(e) = next.apply_file(definition, practice, &Event::Check {})
            {
                self.notice = Some(e);
            }
            let checked = if practice {
                next.files.practice.checked
            } else {
                next.files.assignment.checked
            };
            if checked {
                let rows = next.checks(definition, practice);
                let pass = rows.iter().all(|r| r.0);
                let response = ui
                    .group(|ui| {
                        ui.strong(if pass {
                            if practice {
                                "Example checks pass"
                            } else {
                                "Assignment complete"
                            }
                        } else {
                            "Handover needs attention"
                        });
                        if pass {
                            lesson_presentation::prose(ui, &s.explanation);
                        } else {
                            for (_, why) in rows.iter().filter(|r| !r.0) {
                                lesson_presentation::prose(ui, why);
                            }
                        }
                        controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
                            for (ok, why) in &rows {
                                lesson_presentation::prose(
                                    ui,
                                    &format!(
                                        "{} · {why}",
                                        if *ok { "Verified" } else { "Needs attention" }
                                    ),
                                );
                            }
                        });
                    })
                    .response;
                self.metrics.modal.output = Some(response.rect);
            }
            if practice {
                let r = ui.add_enabled(active, button("Open full assignment"));
                record(&mut self.metrics.modal, "assignment", &r);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Optional help and simulation scope").show(ui, |ui| {
                lesson_presentation::prose(ui, handover::HELP);
                lesson_presentation::prose(ui, dlms_learning_lab::modal_source::HELP);
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
                }
                for link in &s.sources {
                    ui.hyperlink_to(link, link);
                }
            });
        }
        if next != before && active {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state = State::EditorHandover {
                        simulation: Box::new(next),
                    };
                    self.completion_index.forget(&s.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        if stop {
            if let Some(mut job) = self.editor_handover_run.take() {
                job.runner.stop();
                self.editor_handover_suppressed.insert((job.scenario,job.practice));
            }
            self.notice = Some(
                "Setup stopped. Your script and file draft remain saved; run again when ready."
                    .into(),
            );
        }
        if run {
            self.start_editor_handover(i, view == teaching::View::Example, false);
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&s.id], &s);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
