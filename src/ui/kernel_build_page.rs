//! Native supplied-kernel configuration/build workspace; all operations in process.
use super::*;
use dlms_learning_lab::{
    engine::State, kernel_build as core, kernel_source as binding, kernel_workspace as work,
};
type Metrics = super::linux_v6_page::Metrics;
fn apply(t: &work::Task, w: &mut work::Work, a: work::Action, notice: &mut Option<String>) {
    if let Err(e) = t.apply(w, &a) {
        *notice = Some(e);
    }
}
fn edit(
    ui: &mut egui::Ui,
    field: (&str, &str, usize, bool),
    m: &mut Metrics,
    t: &work::Task,
    practice: bool,
) -> Option<String> {
    let (key, value, cap, multiline) = field;
    let id = egui::Id::new(("kernel-build-field", &t.id, practice, key));
    let r = if multiline {
        shared_libraries_page::configuration_input(
            ui,
            value,
            id,
            cap,
            ui.is_enabled(),
            "Configuration text",
        )
    } else {
        shared_libraries_page::fitting_input(
            ui,
            value,
            id,
            cap,
            ui.is_enabled(),
            if key == "entry-name" {
                "Entry filename.conf"
            } else {
                "One supported make operation"
            },
        )
    };
    m.record(ui, key, &r.response);
    m.fields.push((key.into(), r.response.rect, r.response.id));
    if r.rejected {
        ui.label("Input exceeds this field’s plain-text limit; the previous draft is retained.");
    }
    r.value
}
fn config(
    ui: &mut egui::Ui,
    t: &work::Task,
    w: &mut work::Work,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Configuration · supplied .config fragment");
    if let Some(text) = edit(ui, ("config", &w.config_draft, 8192, true), m, t, practice) {
        apply(t, w, work::Action::Config { text }, notice);
    }
    let r = ui.add(button("Save configuration"));
    m.record(ui, "save-config", &r);
    if r.clicked() {
        apply(t, w, work::Action::SaveConfig, notice);
    }
    let saved = core::Config::parse(&w.config_draft).ok().as_ref() == w.saved_config.as_ref()
        && w.saved_config.is_some();
    ui.label(if saved {
        "Saved · builds read these settings"
    } else {
        "Draft differs or is invalid · save before rebuilding"
    });
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    w: &mut work::Work,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Build console · simulated make subset");
    if let Some(text) = edit(
        ui,
        ("command", &w.command_draft, 2048, false),
        m,
        t,
        practice,
    ) {
        apply(t, w, work::Action::CommandDraft { text }, notice);
    }
    let r = ui.add(button("Run"));
    m.record(ui, "run", &r);
    if r.clicked() {
        let input = w.command_draft.clone();
        apply(t, w, work::Action::Run { input }, notice);
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8.;
    let r = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(("kernel-console", &t.id, practice))
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(true),
        |ui| {
            if w.log.is_empty() {
                ui.label("No command has run. Try make help for supported operations.");
            }
            for line in &w.log {
                lesson_presentation::monospace(ui, &format!("$ {}", line.input), true);
                ui.strong(if line.ok {
                    "Actual result"
                } else {
                    "Command failed"
                });
                lesson_presentation::monospace(ui, &line.output, true);
            }
        },
    );
    let response = ui.interact(
        r.inner_rect,
        ui.id().with(("kernel-output", &t.id, practice)),
        egui::Sense::hover(),
    );
    m.record(ui, "console-output", &response);
}
fn artifacts(ui: &mut egui::Ui, t: &work::Task, w: &work::Work, m: &mut Metrics) {
    ui.strong("Actual build outputs");
    let source = &t.sources[w.source];
    let mut rows = Vec::new();
    for (label, record) in [
        ("Image build", &w.outputs.image),
        ("Module build", &w.outputs.modules),
    ] {
        let value = record
            .as_ref()
            .map(|b| {
                let current = w
                    .saved_config
                    .as_ref()
                    .is_some_and(|c| b.current_for(&source.id, &source.release, c));
                format!(
                    "{} · {}",
                    b.release,
                    if current {
                        "matches selected saved inputs"
                    } else {
                        "earlier/different inputs"
                    }
                )
            })
            .unwrap_or_else(|| "Not built".into());
        rows.push((label, value));
    }
    rows.push((
        "Build initramfs",
        w.outputs
            .initramfs
            .as_ref()
            .map(|i| {
                format!(
                    "{} · {} supplied driver objects",
                    i.image_build.release,
                    i.included.len()
                )
            })
            .unwrap_or_else(|| "Not assembled".into()),
    ));
    let r = ui.scope(|ui| {
        if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 34. {
            egui::Grid::new(("kernel-output-table", &t.id))
                .num_columns(2)
                .spacing(egui::vec2(controls::GAP * 2., controls::GAP))
                .show(ui, |ui| {
                    for (label, value) in rows {
                        let label_r = ui.label(label);
                        m.record(ui, &format!("artifact-label:{label}"), &label_r);
                        let value_r =
                            ui.scope(|ui| lesson_presentation::monospace(ui, &value, true));
                        m.record(ui, &format!("artifact-value:{label}"), &value_r.response);
                        ui.end_row();
                    }
                });
        } else {
            for (label, value) in rows {
                let label_r = ui.strong(label);
                m.record(ui, &format!("artifact-label:{label}"), &label_r);
                let value_r = ui.scope(|ui| lesson_presentation::monospace(ui, &value, true));
                m.record(ui, &format!("artifact-value:{label}"), &value_r.response);
                ui.add_space(controls::GAP);
            }
        }
    });
    m.record(ui, "artifacts", &r.response);
}

fn staging(
    ui: &mut egui::Ui,
    t: &work::Task,
    w: &mut work::Work,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Virtual staging · simulator controls");
    lesson_presentation::monospace(ui, &format!("Install root: {}", t.stage_root), true);
    if let Some(root) = &t.root {
        lesson_presentation::monospace(ui, &format!("Supplied root UUID: {}", root.uuid), true);
    }
    let r = ui.add(button("Stage image"));
    m.record(ui, "stage-image", &r);
    if r.clicked() {
        apply(t, w, work::Action::StageImage, notice);
    }
    let key = egui::Id::new(("kernel-staged-image", &t.id, practice));
    let mut selected = ui
        .ctx()
        .data_mut(|d| d.get_temp::<String>(key))
        .unwrap_or_default();
    if !w.store.images.iter().any(|f| f.path == selected) {
        selected = w
            .store
            .images
            .last()
            .map(|f| f.path.clone())
            .unwrap_or_default();
    }
    controls::form_row(ui, "Staged image", |ui, width| {
        let r = controls::dropdown(ui, ("kernel-staged-choice", &t.id, practice), width)
            .selected_text(if selected.is_empty() {
                "No staged images"
            } else {
                &selected
            })
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (n, f) in w.store.images.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, f.path.clone(), &f.path);
                    m.record(ui, &format!("staged-option:{n}"), &r);
                }
            });
        m.record(ui, "staged-choice", &r.response);
    });
    ui.ctx().data_mut(|d| d.insert_temp(key, selected.clone()));
    let r = ui.add_enabled(!selected.is_empty(), button("Assemble staged initramfs"));
    m.record(ui, "staged-initramfs", &r);
    if r.clicked() {
        apply(
            t,
            w,
            work::Action::Initramfs {
                image_path: selected,
            },
            notice,
        );
    }
    let r = ui.scope(|ui| {
        ui.strong("Virtual files actually present");
        for f in &w.store.images {
            lesson_presentation::monospace(ui, &f.path, true);
        }
        for f in &w.store.modules {
            lesson_presentation::monospace(ui, &f.path, true);
        }
        for f in &w.store.initramfs {
            lesson_presentation::monospace(ui, &f.path, true);
        }
        for f in &w.store.entries {
            lesson_presentation::monospace(ui, &f.path, true);
        }
    });
    m.record(ui, "staged-files", &r.response);
    controls::collapsing_header(ui, "Read supplied fallback entries")
        .id_salt(("kernel-fallback", &t.id, practice))
        .show(ui, |ui| {
            for f in &t.initial_store.entries {
                lesson_presentation::monospace(ui, &f.path, true);
                lesson_presentation::code_value(
                    ui,
                    ("kernel-fallback-text", &f.path),
                    &f.source,
                    ui.available_width(),
                );
            }
        });
    ui.add_space(controls::GAP);
    ui.strong("New loader entry");
    let mut filename = w.entry_filename.clone();
    if let Some(v) = edit(ui, ("entry-name", &filename, 48, false), m, t, practice) {
        filename = v;
    }
    let mut text = w.entry_draft.clone();
    if let Some(v) = edit(ui, ("entry-text", &text, 4096, true), m, t, practice) {
        text = v;
    }
    if filename != w.entry_filename || text != w.entry_draft {
        apply(t, w, work::Action::EntryDraft { filename, text }, notice);
    }
    let r = ui.add(button("Save entry"));
    m.record(ui, "save-entry", &r);
    if r.clicked() {
        apply(t, w, work::Action::SaveEntry, notice);
    }
}
fn results(ui: &mut egui::Ui, t: &work::Task, w: &work::Work, practice: bool, m: &mut Metrics) {
    if !w.checked {
        ui.label("Unchecked · check after your final changes.");
        return;
    }
    let checks = t.checks(w);
    let passed = checks.iter().filter(|c| c.passed).count();
    let solved = passed == checks.len();
    let r = ui.strong(if solved {
        if practice {
            "Example meets all requirements"
        } else {
            "Assignment complete"
        }
        .into()
    } else {
        format!(
            "Needs attention · {passed}/{} requirements met",
            checks.len()
        )
    });
    m.record(ui, "result", &r);
    m.result = Some((passed, checks.len(), r.rect));
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
    .id_salt(("kernel-results", &t.id, practice))
    .show(ui, |ui| {
        for c in checks.iter().filter(|c| c.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
        }
    });
}
impl LabApp {
    pub(super) fn kernel_build_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let State::KernelBuild { simulation } = &original.state else {
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
                for (v, key, text) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(text).selected(view == v));
                    self.metrics.kernel_build.record(ui, key, &r);
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
            let r =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            self.metrics
                .kernel_build
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(ui, "Worked example · separate supplied inputs");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            self.metrics.kernel_build.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.kernel_build.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&d.example, &mut next.example)
            } else {
                (&d.assignment, &mut next.assignment)
            };
            let w = &mut s.work;
            if !practice {
                let r = lesson_presentation::assignment(ui, &scenario);
                self.metrics
                    .kernel_build
                    .record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
            if practice{lesson_presentation::practice_step(ui,&scenario,0);}
            controls::form_row(ui,"Supplied source",|ui,width|{let mut source=w.source;let r=controls::dropdown(ui,("kernel-source",&t.id,practice),width).selected_text(&t.sources[source].title).show_ui(ui,|ui|{controls::fit_popup_contents(ui);for(n,s)in t.sources.iter().enumerate(){let r=ui.selectable_value(&mut source,n,&s.title);self.metrics.kernel_build.record(ui,&format!("source-option:{n}"),&r);}});self.metrics.kernel_build.record(ui,"source",&r.response);if source!=w.source{apply(t,w,work::Action::Source{index:source},&mut self.notice);}});
            if ui.available_width() >= 2. * ui.text_style_height(&egui::TextStyle::Body) * 27. + ui.spacing().item_spacing.x{ui.columns(2,|cols|{config(&mut cols[0],t,w,practice,&mut self.metrics.kernel_build,&mut self.notice);if practice{lesson_presentation::practice_step(&mut cols[1],&scenario,1);}console(&mut cols[1],t,w,practice,&mut self.metrics.kernel_build,&mut self.notice);});}else{config(ui,t,w,practice,&mut self.metrics.kernel_build,&mut self.notice);ui.add_space(controls::GAP);if practice{lesson_presentation::practice_step(ui,&scenario,1);}console(ui,t,w,practice,&mut self.metrics.kernel_build,&mut self.notice);}
            ui.add_space(controls::GAP);artifacts(ui,t,w,&mut self.metrics.kernel_build);
            if t.kind==work::Kind::Boot{if practice{lesson_presentation::practice_step(ui,&scenario,2);}staging(ui,t,w,practice,&mut self.metrics.kernel_build,&mut self.notice);if practice{lesson_presentation::practice_step(ui,&scenario,3);}}else{if practice{lesson_presentation::practice_step(ui,&scenario,2);}let r=ui.add(button("Assemble build initramfs"));self.metrics.kernel_build.record(ui,"build-initramfs",&r);if r.clicked(){apply(t,w,work::Action::WorkingInitramfs,&mut self.notice);}}
            ui.separator();let r=ui.add(primary(ui,if t.kind==work::Kind::Boot{"Check configuration"}else{"Check build"}));self.metrics.kernel_build.record(ui,"check",&r);if r.clicked(){apply(t,w,work::Action::Check,&mut self.notice);}
            results(ui,t,w,practice,&mut self.metrics.kernel_build);if t.solved(w){lesson_presentation::prose(ui,if practice{"Use the same reasoning with the assignment’s different supplied inputs."}else{&scenario.explanation});}
            if practice{let r=controls::collapsing_header(ui,"Show worked answer · example only").id_salt(("kernel-worked",&scenario.id)).show(ui,|ui|{let r=ui.add(button("Apply worked example"));self.metrics.kernel_build.record(ui,"fill-example",&r);if r.clicked(){for action in t.reference.iter().filter(|a|!matches!(a,work::Action::Check)){if let Err(e)=t.apply(w,action){self.notice=Some(e);break;}}}lesson_presentation::prose(ui,"Applies the displayed worked operations to this example only. Inspect actual outputs, then check it.");});self.metrics.kernel_build.record(ui,"worked-answer",&r.header_response);}
        });
            controls::collapsing_header(ui, "Help, supported operations and references")
                .id_salt(("kernel-help", &scenario.id))
                .show(ui, |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                            a.hints = (a.hints + 1).min(scenario.hints.len());
                            self.dirty = true;
                        }
                        for hint in scenario
                            .hints
                            .iter()
                            .take(self.progress.attempts[&scenario.id].hints)
                        {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Why does this work?").show(ui, |ui| {
                            lesson_presentation::prose(ui, &scenario.explanation)
                        });
                    }
                    lesson_presentation::prose(ui, dlms_learning_lab::kernel_commands::HELP);
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }
        if active && next != before {
            match next.bind(d) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state =
                        State::KernelBuild {
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
