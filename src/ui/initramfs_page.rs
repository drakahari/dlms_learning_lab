//! Native early-userspace and boot-entry workspace for the simulated kernel fixtures.
use super::*;
use dlms_learning_lab::{engine::State, initramfs_source as binding, initramfs_workspace as work};
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
    let id = egui::Id::new(("initramfs-field", &t.id, practice, key));
    let r = if multiline {
        shared_libraries_page::configuration_input(
            ui,
            value,
            id,
            cap,
            ui.is_enabled(),
            if key == "entry-text" {
                "Boot entry contents"
            } else {
                "Driver configuration"
            },
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
                "Enter a dracut or lsinitrd command"
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
    ui.strong("Driver configuration · virtual dracut fragment");
    if let Some(text) = edit(ui, ("config", &w.config_draft, 4096, true), m, t, practice) {
        apply(t, w, work::Action::ConfigDraft { text }, notice);
    }
    let r = ui.add(button("Save configuration"));
    m.record(ui, "save-config", &r);
    if r.clicked() {
        apply(t, w, work::Action::SaveConfig, notice);
    }
    let saved = w.saved_config_source.as_ref() == Some(&w.config_draft);
    ui.label(if saved {
        "Saved · generation reads this file"
    } else {
        "Draft differs · save before regenerating"
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
    ui.strong("Early-userspace console · simulated commands");
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
            .id_salt(("initramfs-console", &t.id, practice))
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .stick_to_bottom(true),
        |ui| {
            if w.log.is_empty() {
                ui.label(
                    "No operation has run. Inspect the supplied selected image with lsinitrd.",
                );
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
        ui.id().with(("initramfs-output", &t.id, practice)),
        egui::Sense::hover(),
    );
    m.record(ui, "console-output", &response);
}

fn entry(
    ui: &mut egui::Ui,
    t: &work::Task,
    w: &mut work::Work,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Candidate entry · virtual loader file");
    let mut filename = w.entry_filename.clone();
    let mut source = w.entry_draft.clone();
    controls::form_row(ui, "Entry filename", |ui, _| {
        if let Some(value) = edit(ui, ("entry-name", &filename, 48, false), m, t, practice) {
            filename = value;
        }
    });
    if let Some(value) = edit(ui, ("entry-text", &source, 4096, true), m, t, practice) {
        source = value;
    }
    if filename != w.entry_filename || source != w.entry_draft {
        apply(
            t,
            w,
            work::Action::EntryDraft {
                filename,
                text: source,
            },
            notice,
        );
    }
    let r = ui.add(button("Save entry"));
    m.record(ui, "save-entry", &r);
    if r.clicked() {
        apply(t, w, work::Action::SaveEntry, notice);
    }
}
fn supplied(ui: &mut egui::Ui, t: &work::Task, w: &work::Work, m: &mut Metrics) {
    let r = ui.scope(|ui| {
        controls::form_row(ui, "Target kernel", |ui, _| {
            lesson_presentation::monospace(ui, &t.wanted_release, true)
        });
        controls::form_row(ui, "Selected entry", |ui, _| {
            lesson_presentation::monospace(
                ui,
                w.saved_entry.as_deref().unwrap_or("No entry saved"),
                true,
            )
        });
        if let Some(entry) = w
            .store
            .entries
            .iter()
            .find(|entry| Some(&entry.path) == w.saved_entry.as_ref())
            .and_then(|entry| dlms_learning_lab::boot_entry::Entry::parse(&entry.source).ok())
            && let Ok(boot_dir) = dlms_learning_lab::source_install::join(&t.prefix, "boot")
        {
            for path in entry.initrd {
                controls::form_row(ui, "Selected initramfs", |ui, _| {
                    lesson_presentation::monospace(ui, &format!("{boot_dir}{path}"), true)
                });
            }
        }
        ui.label(
            "Supplied generic base: hostonly=\"no\" · prepared NVMe/ext4 kernel and module inputs",
        );
    });
    m.record(ui, "supplied", &r.response);
}
fn results(ui: &mut egui::Ui, t: &work::Task, w: &work::Work, practice: bool, m: &mut Metrics) {
    let checks = t.checks(w);
    let passed = checks.iter().filter(|c| c.passed).count();
    let r = ui.strong(if !w.checked {
        "Not checked · verify after your saved-file changes".into()
    } else if t.solved(w) {
        if practice {
            "Example verified".into()
        } else {
            "Assignment complete · current handoff and fallback verified".into()
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
        if t.solved(w) {
            "Objective results"
        } else {
            "Requirements already met"
        },
    )
    .id_salt(("initramfs-results", &t.id, practice))
    .show(ui, |ui| {
        for c in checks.iter().filter(|c| c.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
        }
    });
}
impl LabApp {
    pub(super) fn initramfs_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let State::Initramfs { simulation } = &original.state else {
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
                    self.metrics.early_userspace.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        if !challenge {
            next.view = view;
        }
        if view == teaching::View::Introduction {
            let r =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            self.metrics
                .early_userspace
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(ui, "Different supplied kernels, root and files");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            self.metrics
                .early_userspace
                .record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.early_userspace.record(ui, "try-example", &r);
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
                let r = lesson_presentation::assignment_responses(ui, &scenario).goal;
                self.metrics
                    .early_userspace
                    .record(ui, "assignment-content", &r);
            }
            supplied(ui, t, w, &mut self.metrics.early_userspace);
            ui.separator();
            ui.add_enabled_ui(active, |ui| {
                let metrics = &mut self.metrics.early_userspace;
                let notice = &mut self.notice;
                let left = |ui: &mut egui::Ui, w: &mut work::Work, metrics: &mut Metrics, notice: &mut Option<String>| {
                    if practice {
                        lesson_presentation::practice_step(ui, &scenario, 0);
                    }
                    if t.kind == work::Kind::Drivers {
                        config(ui, t, w, practice, metrics, notice);
                    } else {
                        if practice {
                            lesson_presentation::practice_step(ui, &scenario, 1);
                        }
                        console(ui, t, w, practice, metrics, notice);
                    }
                };
                let right = |ui: &mut egui::Ui, w: &mut work::Work, metrics: &mut Metrics, notice: &mut Option<String>| {
                    if t.kind == work::Kind::Drivers {
                        if practice {
                            lesson_presentation::practice_step(ui, &scenario, 1);
                        }
                        console(ui, t, w, practice, metrics, notice);
                    } else {
                        if practice {
                            lesson_presentation::practice_step(ui, &scenario, 2);
                        }
                        entry(ui, t, w, practice, metrics, notice);
                    }
                };
                let column_width = ui.text_style_height(&egui::TextStyle::Body) * 27.;
                if ui.available_width() >= 2. * column_width + ui.spacing().item_spacing.x {
                    ui.columns(2, |columns| {
                        left(&mut columns[0], w, metrics, notice);
                        right(&mut columns[1], w, metrics, notice);
                    });
                } else {
                    left(ui, w, metrics, notice);
                    ui.add_space(controls::GAP);
                    right(ui, w, metrics, notice);
                }
                if t.kind == work::Kind::Handover {
                    controls::collapsing_header(ui, "Driver fragment · optional exploration")
                        .id_salt(("initramfs-optional-config", &t.id, practice))
                        .show(ui, |ui| config(ui, t, w, practice, metrics, notice));
                }
                ui.separator();
                if practice {
                    let step = if t.kind == work::Kind::Drivers { 2 } else { 3 };
                    lesson_presentation::practice_step(ui, &scenario, step);
                }
                let r = ui.add(primary(ui, "Check configuration"));
                metrics.record(ui, "check", &r);
                if r.clicked() {
                    apply(t, w, work::Action::Check, notice);
                }
                results(ui, t, w, practice, metrics);
                if t.solved(w) {
                    lesson_presentation::prose(ui, if practice {
                        "Use the same reasoning with the assignment’s different supplied kernels and files."
                    } else {
                        &scenario.explanation
                    });
                }
                if practice {
                    let r = controls::collapsing_header(ui, "Show worked answer · example only")
                        .id_salt(("initramfs-worked", &scenario.id))
                        .show(ui, |ui| {
                            let r = ui.add(button("Apply worked example"));
                            metrics.record(ui, "fill-example", &r);
                            if r.clicked() {
                                apply(t, w, work::Action::Reset, notice);
                                for action in t.reference.iter().filter(|a| !matches!(a, work::Action::Check)) {
                                    if let Err(error) = t.apply(w, action) {
                                        *notice = Some(error);
                                        break;
                                    }
                                }
                            }
                            ui.label("Resets this example and applies the worked operations. Inspect the results, then check the configuration.");
                        });
                    metrics.record(ui, "worked-answer", &r.header_response);
                }
            });
            controls::collapsing_header(ui, "Help, supported operations and references")
                .id_salt(("initramfs-help", &scenario.id))
                .show(ui, |ui| {
                    if ui.add_enabled(active, button("Show a hint")).clicked() {
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
                    lesson_presentation::prose(ui, dlms_learning_lab::initramfs::HELP);
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
                        State::Initramfs {
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
