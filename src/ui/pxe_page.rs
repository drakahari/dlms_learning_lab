//! Native boot-offer table, client traces and PXELINUX configuration editor.
use super::*;
use dlms_learning_lab::engine::State;
use dlms_learning_lab::pxe::{
    self, Architecture, Artifact, Inventory, Request, source,
    workspace::{self as work, Action, Mode, Rule, Task, Work},
};

type Metrics = super::linux_v6_page::Metrics;

fn apply(task: &Task, state: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = task.apply(state, &action) {
        *notice = Some(error);
    }
}

fn single(
    ui: &mut egui::Ui,
    key: egui::Id,
    content: (&str, usize, &str),
    name: &str,
    metrics: &mut Metrics,
) -> Option<String> {
    let (text, capacity, placeholder) = content;
    let response =
        shared_libraries_page::fitting_input(ui, text, key, capacity, ui.is_enabled(), placeholder);
    metrics.record(ui, name, &response.response);
    metrics
        .fields
        .push((name.into(), response.response.rect, response.response.id));
    if response.rejected {
        ui.label("The field exceeded its text limit. The previous draft is still here.");
    }
    response.value
}

fn policy_row(
    ui: &mut egui::Ui,
    row: (&str, &str, &mut Rule),
    key: egui::Id,
    metrics: &mut Metrics,
    columns: Option<[f32; 4]>,
) {
    let (id, title, rule) = row;
    let toggle = |ui: &mut egui::Ui, rule: &mut Rule, metrics: &mut Metrics| {
        let r = ui.checkbox(&mut rule.enabled, "Offer");
        metrics.record(ui, &format!("{id}-enabled"), &r);
    };
    let server = |ui: &mut egui::Ui, rule: &mut Rule, metrics: &mut Metrics| {
        if let Some(text) = single(
            ui,
            key.with("server"),
            (&rule.server, 15, "IPv4 address"),
            &format!("{id}-server"),
            metrics,
        ) {
            rule.server = text;
        }
    };
    let filename = |ui: &mut egui::Ui, rule: &mut Rule, metrics: &mut Metrics| {
        if let Some(text) = single(
            ui,
            key.with("filename"),
            (&rule.filename, 127, "Relative boot filename"),
            &format!("{id}-filename"),
            metrics,
        ) {
            rule.filename = text;
        }
    };
    if let Some([label, enabled, address, path]) = columns {
        ui.horizontal(|ui| {
            controls::label(ui, title, label);
            controls::cell(ui, enabled, |ui| toggle(ui, rule, metrics));
            controls::cell(ui, address, |ui| {
                ui.add_enabled_ui(rule.enabled, |ui| server(ui, rule, metrics));
            });
            controls::cell(ui, path, |ui| {
                ui.add_enabled_ui(rule.enabled, |ui| filename(ui, rule, metrics));
            });
        });
    } else {
        ui.strong(title);
        toggle(ui, rule, metrics);
        ui.add_enabled_ui(rule.enabled, |ui| {
            controls::form_row(ui, "Server address", |ui, _| server(ui, rule, metrics));
            controls::form_row(ui, "Boot filename", |ui, _| filename(ui, rule, metrics));
        });
        ui.add_space(controls::GAP);
    }
}

fn policy(
    ui: &mut egui::Ui,
    key: egui::Id,
    task: &Task,
    state: &mut Work,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Boot-file offers");
    let mut draft = state.policy_draft.clone();
    let height = ui.text_style_height(&egui::TextStyle::Body);
    let columns = if ui.available_width() >= 48. * height {
        let widths = [
            12. * height,
            5. * height,
            10. * height,
            ui.available_width() - 27. * height - 3. * ui.spacing().item_spacing.x,
        ];
        ui.horizontal(|ui| {
            for (name, width) in ["Request", "Enabled", "Server address", "Boot filename"]
                .into_iter()
                .zip(widths)
            {
                controls::table_text(ui, name, width);
            }
        });
        Some(widths)
    } else {
        None
    };
    for row in [
        ("bios", "PC BIOS · architecture 0", &mut draft.bios),
        ("uefi", "x64 UEFI · architecture 7", &mut draft.x64_uefi),
        ("default", "Unconditional offer", &mut draft.unconditional),
    ] {
        let row_key = key.with(row.0);
        policy_row(ui, row, row_key, metrics, columns);
    }
    if draft != state.policy_draft {
        apply(task, state, Action::EditPolicy { draft }, notice);
    }
    let save = ui.add(button("Save boot-file policy"));
    metrics.record(ui, "save-policy", &save);
    if save.clicked() {
        apply(task, state, Action::SavePolicy, notice);
    }
}

fn entry(
    ui: &mut egui::Ui,
    key: egui::Id,
    task: &Task,
    state: &mut Work,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Some(mut draft) = state.entry_draft.clone() else {
        return;
    };
    ui.strong("PXELINUX configuration");
    controls::form_row(ui, "Name in pxelinux.cfg", |ui, _| {
        if let Some(name) = single(
            ui,
            key.with("entry-name"),
            (&draft.leaf, 127, "Configuration filename"),
            "entry-name",
            metrics,
        ) {
            draft.leaf = name;
        }
    });
    let response = shared_libraries_page::configuration_input(
        ui,
        &draft.source,
        key.with("entry-text"),
        4096,
        ui.is_enabled(),
        "Configuration text",
    );
    metrics.record(ui, "entry-text", &response.response);
    metrics.fields.push((
        "entry-text".into(),
        response.response.rect,
        response.response.id,
    ));
    if response.rejected {
        ui.label("The editor is limited to 4 KiB. The previous draft is still here.");
    }
    if let Some(text) = response.value {
        draft.source = text;
    }
    if state.entry_draft.as_ref() != Some(&draft) {
        apply(task, state, Action::EditEntry { draft }, notice);
    }
    let r = ui.add(button("Save configuration"));
    metrics.record(ui, "save-entry", &r);
    if r.clicked() {
        apply(task, state, Action::SaveEntry, notice);
    }
}

fn inventory(ui: &mut egui::Ui, inventory: &Inventory, key: egui::Id) {
    controls::collapsing_header(ui, "Inspect boot server files")
        .id_salt(key)
        .show(ui, |ui| {
            for record in inventory.files() {
                let kind = match &record.artifact {
                    Artifact::BiosPxelinux6 => "PC BIOS PXELINUX 6.x".into(),
                    Artifact::X64EfiApplication => "x64 UEFI application".into(),
                    Artifact::PxelinuxCore => "PXELINUX core module".into(),
                    Artifact::Configuration { .. } => "PXELINUX configuration".into(),
                    Artifact::Kernel { release } => format!("Linux kernel · {release}"),
                    Artifact::Initramfs { release } => format!("Initramfs · {release}"),
                };
                lesson_presentation::monospace(
                    ui,
                    &format!("{}/{}", record.file.server, record.file.name),
                    true,
                );
                ui.label(kind);
            }
        });
}

fn trace(
    ui: &mut egui::Ui,
    task: &Task,
    state: &Work,
    inventory: &Inventory,
    key: egui::Id,
    metrics: &mut Metrics,
) {
    ui.strong("Inspect a client");
    let mut selected = ui.ctx().data(|data| data.get_temp::<usize>(key));
    ui.horizontal_wrapped(|ui| {
        for (index, case) in task.cases.iter().enumerate() {
            let r = ui.add(button(&case.label).selected(selected == Some(index)));
            metrics.record(ui, &format!("inspect-{index}"), &r);
            if r.clicked() {
                selected = Some(index);
                ui.ctx().data_mut(|data| data.insert_temp(key, index));
            }
        }
    });
    let Some(case) = selected.and_then(|index| task.cases.get(index)) else {
        ui.label("Choose a client to trace the current saved configuration.");
        return;
    };
    let r = ui.scope(|ui| {
        let architecture = match case.client.request {
            Request::OrdinaryDhcp => "Ordinary DHCP client".into(),
            Request::Pxe { option_93 } => option_93
                .and_then(Architecture::from_option_93)
                .map_or_else(
                    || "Unknown architecture".into(),
                    |value| value.label().to_owned(),
                ),
        };
        lesson_presentation::prose(
            ui,
            &format!(
                "{} · supplied lease {} · {}",
                case.label, case.client.address, architecture
            ),
        );
        match pxe::bootstrap(inventory, &state.policy, &case.client) {
            Err(detail) => {
                lesson_presentation::prose(ui, &work::explain(detail));
            }
            Ok(None) => {
                ui.label("No boot-file offer.");
            }
            Ok(Some(boot)) => {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} supplies {} for {}.",
                        boot.file.server,
                        boot.file.name,
                        boot.architecture.label()
                    ),
                );
                if !matches!(task.mode, Mode::Configuration { .. }) {
                    return;
                }
                match pxe::trace_configuration(inventory, &boot.file, &case.client) {
                    Err(detail) => {
                        lesson_presentation::prose(ui, &work::explain(detail));
                    }
                    Ok(trace) => {
                        controls::collapsing_header(ui, "Configuration paths tried")
                            .id_salt(key.with("paths"))
                            .show(ui, |ui| {
                                for path in trace.attempted {
                                    let note = if trace.selected.as_ref() == Some(&path) {
                                        "Selected"
                                    } else {
                                        "Absent"
                                    };
                                    lesson_presentation::monospace(
                                        ui,
                                        &format!("{note} · {}", path.name),
                                        true,
                                    );
                                }
                            });
                        if let Some(selected) = trace.selected {
                            lesson_presentation::monospace(
                                ui,
                                &format!("Selected configuration: {}", selected.name),
                                true,
                            );
                        }
                        match pxe::resolve_linux_pair(inventory, &boot.file, &case.client) {
                            Ok(pair) => {
                                ui.label(format!(
                                    "Kernel and initramfs both match {}.",
                                    pair.release
                                ));
                            }
                            Err(detail) => {
                                lesson_presentation::prose(ui, &work::explain(detail));
                            }
                        }
                    }
                }
            }
        }
    });
    metrics.record(ui, "trace", &r.response);
}

fn outcomes(ui: &mut egui::Ui, task: &Task, state: &Work, practice: bool, metrics: &mut Metrics) {
    let checks = task.checks(state);
    let passed = checks.iter().filter(|row| row.passed).count();
    let headline = if !state.checked {
        "Not checked".into()
    } else if task.solved(state) {
        if practice {
            "Example verified".into()
        } else {
            "Assignment complete · client outcomes verified".into()
        }
    } else {
        format!("Needs repair · {passed}/{} requirements met", checks.len())
    };
    let r = ui.strong(headline);
    metrics.record(ui, "result", &r);
    metrics.result = Some((passed, checks.len(), r.rect));
    if state.checked {
        for check in checks {
            let mark = if check.passed {
                "Passed"
            } else {
                "Needs repair"
            };
            ui.strong(format!("{mark} · {}", check.label));
            lesson_presentation::prose(ui, &check.detail);
        }
    }
}

impl LabApp {
    pub(super) fn pxe_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario) {
            Ok(definition) => definition,
            Err(error) => {
                lesson_presentation::prose(ui, &error);
                return;
            }
        };
        if let Err(error) = source::context(attempt, &scenario) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::Pxe { simulation } = &attempt.state else {
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
                for (candidate, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == candidate));
                    self.metrics.network_boot.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = candidate;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            self.metrics
                .network_boot
                .record(ui, "introduction-content", &intro.objective);
            lesson_presentation::worked(ui, "Example at a different boot server");
            let worked = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            self.metrics
                .network_boot
                .record(ui, "worked", &worked.response);
            let start = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.network_boot.record(ui, "try-example", &start);
            if start.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (task, state) = if practice {
                (&definition.example, &mut next.example.work)
            } else {
                (&definition.assignment, &mut next.assignment.work)
            };
            let key = egui::Id::new(("network-boot", &task.id));
            if !practice {
                let assignment = lesson_presentation::assignment_responses(ui, &scenario);
                self.metrics
                    .network_boot
                    .record(ui, "assignment-content", &assignment.goal);
            }
            ui.add_enabled_ui(active, |ui| {
                let metrics = &mut self.metrics.network_boot;
                if practice {
                    lesson_presentation::practice_step(ui, &scenario, 0);
                }
                if let Ok(files) = task.effective_inventory(state) {
                    inventory(ui, &files, key.with("inventory"));
                    trace(ui, task, state, &files, key.with("trace"), metrics);
                }
                ui.separator();
                if practice {
                    lesson_presentation::practice_step(ui, &scenario, 1);
                }
                match task.mode {
                    Mode::Policy => policy(ui, key, task, state, metrics, &mut self.notice),
                    Mode::Configuration { .. } => {
                        entry(ui, key, task, state, metrics, &mut self.notice)
                    }
                }
                if let Ok(files) = task.effective_inventory(state) {
                    ui.label(if task.has_unsaved_changes(state, &files) {
                        "Draft differs · client traces still use saved settings"
                    } else {
                        "Saved · client traces use these settings"
                    });
                }
                ui.separator();
                if practice {
                    lesson_presentation::practice_step(ui, &scenario, 2);
                }
                let check = ui.add(primary(ui, "Check configuration"));
                metrics.record(ui, "check", &check);
                if check.clicked() {
                    apply(task, state, Action::Check, &mut self.notice);
                }
                outcomes(ui, task, state, practice, metrics);
                if !practice && task.solved(state) {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = controls::collapsing_header(ui, "Show worked answer · example only")
                        .id_salt(key.with("worked-answer"))
                        .show(ui, |ui| {
                            let apply_worked = ui.add(button("Apply worked example"));
                            metrics.record(ui, "fill-example", &apply_worked);
                            if apply_worked.clicked() {
                                apply(task, state, Action::Reset, &mut self.notice);
                                for action in task
                                    .reference
                                    .iter()
                                    .filter(|a| !matches!(a, Action::Check))
                                {
                                    if let Err(error) = task.apply(state, action) {
                                        self.notice = Some(error);
                                        break;
                                    }
                                }
                            }
                            ui.label(
                                "Fills the worked example. Inspect the saved results and check it.",
                            );
                        });
                    metrics.record(ui, "worked-answer", &r.header_response);
                }
            });
            controls::collapsing_header(ui, "Help, supported operations and references")
                .id_salt(key.with("help"))
                .show(ui, |ui| {
                    if ui.add_enabled(active, button("Show a hint")).clicked() {
                        let attempt = self.progress.attempts.get_mut(&scenario.id).unwrap();
                        attempt.hints = (attempt.hints + 1).min(scenario.hints.len());
                        self.dirty = true;
                    }
                    for hint in scenario
                        .hints
                        .iter()
                        .take(self.progress.attempts[&scenario.id].hints)
                    {
                        lesson_presentation::prose(ui, hint);
                    }
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }

        if active && next != before {
            match next.bind(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state = State::Pxe {
                        simulation: Box::new(next),
                    };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(error) => self.notice = Some(error),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&scenario.id], &scenario);
        if self.metrics.completion {
            self.next_unit_lab(ui, index);
        }
        Self::workspace_footer(ui);
    }
}
