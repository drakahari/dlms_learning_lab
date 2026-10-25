//! Two explicit legacy-seed views; assignment editors and engine remain unchanged.
use super::*;
use dlms_learning_lab::{seed_dns_teaching as dns, seed_mount_teaching as mount};

impl LabApp {
    pub(super) fn seed_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        let w = a.workspace.as_ref().unwrap();
        w.seed_dns_learning
            .as_ref()
            .map(|l| l.view)
            .or_else(|| w.seed_mount_learning.as_ref().map(|l| l.view))
            .unwrap_or(teaching::View::Assignment)
    }
    pub(super) fn set_seed_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.seed_dns_learning {
            l.view = view;
        }
        if let Some(l) = &mut w.seed_mount_learning {
            l.view = view;
        }
        self.focus_input = true;
        self.scroll_command = false;
        self.lesson_scroll_reset = true;
        self.help_open = false;
        self.notice = None;
        self.dirty = true;
    }
    pub(super) fn seed_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.seed_view(i);
        ui.heading(&s.title);
        if !self.progress.attempts[&s.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(v == view, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_seed_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let (note, concepts, description) = if let Some(l) = &s.seed_dns_teaching {
                    (&l.prerequisite_note, &l.concepts, &l.example_description)
                } else {
                    let l = s.seed_mount_teaching.as_ref().unwrap();
                    (&l.prerequisite_note, &l.concepts, &l.example_description)
                };
                ui.label(RichText::new(note).color(palette(ui).muted));
                for c in concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_seed_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_seed_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                if let Some(l) = &s.seed_dns_teaching {
                    ui.push_id(("seed-dns-example", &s.id), |ui| {
                        self.seed_dns_example(ui, i, l)
                    });
                } else {
                    ui.push_id(("seed-mount-example", &s.id), |ui| {
                        self.seed_mount_example(ui, i, s.seed_mount_teaching.as_ref().unwrap())
                    });
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let active = ui.is_enabled()
                    && !(self.reset_pending
                        || self.settings_open
                        || self.theme_preview
                        || self.data_tools.modal_open());
                ui.push_id(("seed-assignment", &s.id), |ui| {
                    ui.add_enabled_ui(active, |ui| {
                        self.configuration(ui, i);
                        self.terminal(ui, i);
                        self.check_terminal_work(ui, i);
                    });
                });
            }
        }
    }
    fn seed_dns_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &dns::Lesson) {
        let id = self.scenarios[i].id.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        ui.label("Practice example · Independent; no assignment credit.");
        ui.label(&lesson.example_description);
        let l = self
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .seed_dns_learning
            .as_mut()
            .unwrap();
        let mut changed = false;
        let mut assignment = false;
        ui.add_enabled_ui(active, |ui| {
            let labels: Vec<_> = lesson
                .example_actions
                .iter()
                .map(|a| (a.title.as_str(), a.explanation.as_str()))
                .collect();
            let before = l.selected_action;
            if seed_picker(ui, &mut l.selected_action, &labels, &mut self.metrics) {
                if let Err(e) = l.insert(lesson) {
                    self.notice = Some(seed_message(&e));
                }
                changed = true;
                self.focus_input = true;
            }
            changed |= before != l.selected_action;
            if let dns::Action::Command { command } =
                &lesson.example_actions[l.selected_action].action
            {
                ui.monospace(&command.command);
                if command.simulator_only {
                    ui.label(
                        "Simulator-only abbreviated command; real BIND needs a zone-file argument.",
                    );
                }
            }
            let served = l
                .current
                .served
                .map_or_else(|| "No loaded answer".into(), |a| a.to_string());
            let summary = ui.label(format!(
                "{} · saved {} (serial {}) · serving {} (serial {})",
                l.current.name, l.current.saved, l.current.serial, served, l.current.loaded_serial
            ));
            self.metrics.seed_summary = Some(summary.rect);
            changed |= workspace::editor_row(
                ui,
                "IPv4 address",
                "Practice DNS A record address — ungraded",
                &mut l.draft.address,
                "seed-dns-address",
                &mut self.metrics,
                false,
            );
            changed |= workspace::editor_row(
                ui,
                "Zone serial",
                "Practice DNS serial — ungraded",
                &mut l.draft.serial,
                "seed-dns-serial",
                &mut self.metrics,
                false,
            );
            if changed {
                l.checked = false;
            }
            let r = ui.add(button("Save example configuration"));
            self.metrics.editor_save = Some(r.rect);
            if r.clicked() {
                self.notice = Some(match l.save(lesson) {
                    Ok(()) => "Saved example settings; validation and reload are separate.".into(),
                    Err(e) => seed_message(&e),
                });
                changed = true;
            }
            if !l.draft.matches(&l.current) {
                ui.label("Unsaved example settings · retained when leaving");
            }
            if let Some(note) = &self.notice {
                ui.label(note);
            }
            ui.label(format!("Try a change: {}", lesson.try_change));
            let (run, edited) = seed_terminal(
                ui,
                &mut l.command,
                &l.transcript,
                &id,
                "seed-dns",
                &mut self.metrics,
                &mut self.focus_input,
                &mut self.scroll_command,
                active,
            );
            changed |= edited;
            if run {
                let _ = l.submit(lesson);
                changed = true;
                self.focus_input = true;
            }
            let r = ui.add(primary(ui, "Check example"));
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                l.checked = true;
                changed = true;
            }
            if l.checked {
                seed_checks(ui, l.checks(lesson), &mut self.metrics);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Reset example only"));
                self.metrics.example_reset = Some(r.rect);
                if r.clicked() {
                    *l = dns::Learning::new(lesson).expect("validated lesson");
                    l.view = teaching::View::Example;
                    self.notice = None;
                    changed = true;
                }
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                assignment = r.clicked();
            });
        });
        self.dirty |= changed;
        if assignment {
            self.set_seed_view(i, teaching::View::Assignment);
        }
    }
    fn seed_mount_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &mount::Lesson) {
        let id = self.scenarios[i].id.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        ui.label("Practice example · Independent; no assignment credit.");
        ui.label(&lesson.example_description);
        let l = self
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .seed_mount_learning
            .as_mut()
            .unwrap();
        let mut changed = false;
        let mut assignment = false;
        ui.add_enabled_ui(active, |ui| {
            let labels: Vec<_> = lesson
                .example_actions
                .iter()
                .map(|a| (a.title.as_str(), a.explanation.as_str()))
                .collect();
            let before = l.selected_action;
            if seed_picker(ui, &mut l.selected_action, &labels, &mut self.metrics) {
                if let Err(e) = l.insert(lesson) {
                    self.notice = Some(seed_message(&e));
                }
                changed = true;
                self.focus_input = true;
            }
            changed |= before != l.selected_action;
            if let mount::Action::Command { command } =
                &lesson.example_actions[l.selected_action].action
            {
                ui.monospace(&command.command);
            }
            let status = if !l.current.mounted {
                "Unmounted"
            } else if l.current.writable {
                "Mounted read-write"
            } else {
                "Mounted read-only"
            };
            let summary = ui.label(format!(
                "Known device {} · saved pointer {} · {}/{} MiB used · {status}",
                l.current.device, l.current.configured_device, l.current.used, l.current.capacity
            ));
            self.metrics.seed_summary = Some(summary.rect);
            changed |= workspace::editor_row(
                ui,
                "Device",
                "Practice mount saved device — ungraded",
                &mut l.draft.device,
                "seed-mount-device",
                &mut self.metrics,
                false,
            );
            if changed {
                l.checked = false;
            }
            let r = ui.add(button("Save example configuration"));
            self.metrics.editor_save = Some(r.rect);
            if r.clicked() {
                self.notice = Some(match l.save(lesson) {
                    Ok(()) => "Saved example pointer; current mounting is separate.".into(),
                    Err(e) => seed_message(&e),
                });
                changed = true;
            }
            if !l.draft.matches(&l.current) {
                ui.label("Unsaved example settings · retained when leaving");
            }
            if let Some(note) = &self.notice {
                ui.label(note);
            }
            ui.label(format!("Try a change: {}", lesson.try_change));
            let (run, edited) = seed_terminal(
                ui,
                &mut l.command,
                &l.transcript,
                &id,
                "seed-mount",
                &mut self.metrics,
                &mut self.focus_input,
                &mut self.scroll_command,
                active,
            );
            changed |= edited;
            if run {
                let _ = l.submit(lesson);
                changed = true;
                self.focus_input = true;
            }
            let r = ui.add(primary(ui, "Check example"));
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                l.checked = true;
                changed = true;
            }
            if l.checked {
                seed_checks(ui, l.checks(lesson), &mut self.metrics);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Reset example only"));
                self.metrics.example_reset = Some(r.rect);
                if r.clicked() {
                    *l = mount::Learning::new(lesson).expect("validated lesson");
                    l.view = teaching::View::Example;
                    self.notice = None;
                    changed = true;
                }
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                assignment = r.clicked();
            });
        });
        self.dirty |= changed;
        if assignment {
            self.set_seed_view(i, teaching::View::Assignment);
        }
    }
}
pub(super) fn seed_picker(
    ui: &mut egui::Ui,
    selected: &mut usize,
    labels: &[(&str, &str)],
    metrics: &mut Metrics,
) -> bool {
    let mut insert = false;
    ui.horizontal_wrapped(|ui| {
        let width = (ui.available_width()
            - controls::button_width(ui, "Insert action")
            - ui.spacing().item_spacing.x)
            .max(100.)
            .min(ui.available_width());
        let picker = egui::ComboBox::from_id_salt("seed-example-action")
            .width(width)
            .truncate()
            .selected_text(format!("{}. {}", *selected + 1, labels[*selected].0))
            .show_ui(ui, |ui| {
                for (index, (title, _)) in labels.iter().enumerate() {
                    let r = ui.selectable_value(selected, index, format!("{}. {title}", index + 1));
                    metrics.terminal_example_choices.push((index, r.rect));
                }
                metrics.terminal_example_popup = Some(controls::popup_content_clip(ui));
            });
        metrics.terminal_example_picker = Some(picker.response.rect);
        name_control(&picker.response, "Explained practice action");
        let r = ui.add(button("Insert action"));
        metrics.insert = Some(r.rect);
        insert = r.clicked();
    });
    let explanation = ui.label(labels[*selected].1);
    metrics.seed_explanation = Some(explanation.rect);
    insert
}
pub(super) fn seed_checks(ui: &mut egui::Ui, checks: Vec<(bool, String)>, metrics: &mut Metrics) {
    for (pass, message) in checks {
        let row = ui.label(format!(
            "{} · {message}",
            if pass { "Pass" } else { "Needs attention" }
        ));
        metrics.seed_feedback.push((pass, row.rect));
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn seed_terminal(
    ui: &mut egui::Ui,
    command: &mut String,
    transcript: &[String],
    id: &str,
    kind: &str,
    metrics: &mut Metrics,
    focus: &mut bool,
    scroll: &mut bool,
    active: bool,
) -> (bool, bool) {
    let height = (ui.ctx().content_rect().height() * 0.24)
        .max(2. * ui.text_style_height(&egui::TextStyle::Monospace))
        .min(240.);
    let output = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_Y)
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt((kind, id, "output"))
                .max_height(height)
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if transcript.is_empty() {
                        ui.label("Insert a command or type your own, then choose Run.");
                    }
                    for line in transcript {
                        ui.add(
                            egui::Label::new(RichText::new(line).monospace())
                                .extend()
                                .selectable(true),
                        );
                    }
                });
        });
    metrics.output = Some(output.response.rect);
    let mut run = false;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("practice #").monospace());
        let run_width = controls::button_width(ui, "Run");
        let width = (ui.available_width() - run_width - ui.spacing().item_spacing.x).max(40.);
        let r = ui.add_sized(
            [width, controls::height(ui)],
            controls::singleline(ui, command)
                .id_salt((kind, id, "command"))
                .char_limit(4096)
                .hint_text("Command · Enter to run"),
        );
        name_control(
            &r,
            match kind {
                "seed-dns" => "DNS practice terminal command — ungraded",
                "legacy-access" => "Access practice terminal command — ungraded",
                "legacy-logs" => "Log practice terminal command — ungraded",
                _ => "Storage practice terminal command — ungraded",
            },
        );
        metrics.terminal = Some(r.rect);
        metrics.focus = Some(r.id);
        metrics.inputs.push(r.rect);
        if *focus && active && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
            r.request_focus();
            *focus = false;
            if *scroll {
                r.scroll_to_me(Some(egui::Align::Center));
                *scroll = false;
            }
        } else if *focus && active {
            ui.ctx().request_repaint();
        }
        changed = r.changed();
        if changed {
            workspace::bound(command, 4096);
        }
        run = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let b = ui
            .add_enabled_ui(active, |ui| {
                ui.add_sized([run_width, controls::height(ui)], button("Run"))
            })
            .inner;
        metrics.terminal_run = Some(b.rect);
        metrics.terminal_run_enabled = Some(b.enabled());
        run |= b.clicked();
    });
    (run && active && ui.is_enabled(), changed)
}

pub(super) fn seed_message(s: &str) -> String {
    let mut end = s.len().min(512);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}
