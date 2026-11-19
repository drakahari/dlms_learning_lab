//! Opt-in native configuration editor and actual simulated scan results.
//! Other LVM workspaces retain their existing controls and appearance.
use super::*;
use dlms_learning_lab::lvm::config_scan as core;
use model::config_scan as flow;
fn record(metrics: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    metrics.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn mode(mode: core::Mode) -> &'static str {
    match mode {
        core::Mode::DevicesFile => "Provided devices file · filter ignored",
        core::Mode::FilterBecauseFileMissing => "Filter policy · no devices file supplied",
        core::Mode::FilterSelected => "Filter policy · devices-file use disabled",
    }
}
fn reason(value: &core::Reason) -> String {
    match value {
        core::Reason::FilterRule { index, action } => format!(
            "First match: rule{} · {}",
            index + 1,
            if *action == core::Action::Accept {
                "accept"
            } else {
                "reject"
            }
        ),
        core::Reason::NoFilterMatched => "No rule matched · allowed by default".into(),
        core::Reason::ListedInDevicesFile => "Listed in enabled file".into(),
        core::Reason::NotInDevicesFile => "Absent from enabled file".into(),
    }
}
fn worked(ui: &mut egui::Ui, op: &flow::Operation) {
    ui.strong(op.label());
    match op {
        flow::Operation::SaveConfig { text } => {
            lesson_presentation::monospace(ui, &text.0, true);
        }
        flow::Operation::SaveSelection { selection } => {
            lesson_presentation::monospace(
                ui,
                &selection
                    .entries
                    .iter()
                    .map(|v| v.0.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                true,
            );
        }
        flow::Operation::Scan => {
            ui.label("Read each actual scan result and compare it with the requested selections.");
        }
    }
}
impl LabApp {
    pub(super) fn lvm_config_scan_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let source = self.scenarios[i].clone();
        if let Err(e) = model::context(&self.progress.attempts[&source.id], &source) {
            ui.label(format!(
                "Configuration work retained without applying changes: {e}"
            ));
            return;
        }
        let packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let inherited = self.lvm_presentation.held_veto;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.lvm_presentation.held_veto = false;
        }
        let blocked = inherited || self.lvm_presentation.veto_root == Some(root);
        let before = (
            self.selected,
            self.reset_pending,
            self.progress.attempts[&source.id].challenge,
        );
        let challenge = ui
            .add_enabled_ui(!packet && !blocked, |ui| self.workspace_header(ui, i))
            .inner;
        if before
            != (
                self.selected,
                self.reset_pending,
                self.progress.attempts[&source.id].challenge,
            )
        {
            return;
        }
        let engine::Goal::Lvm { definition: d } = &source.goal else {
            return;
        };
        let allowed = !blocked
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();

        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        let State::Lvm { simulation: s } = &mut attempt.state else {
            return;
        };
        let view = if challenge { View::Assignment } else { s.view };
        if !challenge {
            let mut next = None;
            ui.horizontal_wrapped(|ui| {
                for (v, label, key) in [
                    (View::Introduction, "Introduction", "introduction"),
                    (View::Example, "Try example · ungraded", "example"),
                    (View::Assignment, "Assignment", "assignment"),
                ] {
                    let r = ui.add_enabled(
                        allowed && !packet,
                        egui::Button::new(label).selected(v == view),
                    );
                    record(&mut self.metrics.lvm, key, &r, ui);
                    if r.clicked() {
                        next = Some(v);
                    }
                }
            });
            if let Some(v) = next {
                match s.set_view(d, v) {
                    Ok(()) => {
                        self.dirty = true;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                    Err(e) => self.notice = Some(e),
                }
                return;
            }
        }
        ui.heading(&source.title);
        if view == View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                &d.lesson.prerequisite_note,
                d.lesson
                    .concepts
                    .iter()
                    .map(|c| (c.title.clone(), c.text.clone())),
            );
            lesson_presentation::worked(ui, &d.lesson.description);
            for step in &d.lesson.config_scan_practice.as_ref().unwrap().worked {
                worked(ui, &step.operation);
                lesson_presentation::prose(ui, &step.explanation);
                ui.add_space(ui.spacing().item_spacing.y);
            }
            let mut next = None;
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(allowed && !packet, primary(ui, "Try this example"));
                record(&mut self.metrics.lvm, "practice-start", &r, ui);
                if r.clicked() {
                    next = Some(View::Example);
                }
                let r = ui.add_enabled(allowed && !packet, button("Go straight to assignment"));
                record(&mut self.metrics.lvm, "assignment-start", &r, ui);
                if r.clicked() {
                    next = Some(View::Assignment);
                }
            });
            if let Some(v) = next {
                match s.set_view(d, v) {
                    Ok(()) => {
                        self.dirty = true;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            return;
        }
        let practice = view == View::Example;
        let c = d.config_context(practice).unwrap();
        if practice {
            lesson_presentation::lead(ui, "GUIDED PRACTICE", &d.lesson.description);
            lesson_presentation::practice_step(ui, &source, 0);
            // Expected selections are an example goal, never presented as an
            // actual run result or as a filled assignment configuration.
            let worked_panel = controls::collapsing_header(
                ui,
                "Example goal and worked configuration",
            )
            .show(ui, |ui| {
                for step in &d.lesson.config_scan_practice.as_ref().unwrap().worked {
                    worked(ui, &step.operation);
                    lesson_presentation::prose(ui, &step.explanation);
                    let action = match &step.operation {
                        flow::Operation::SaveConfig { .. } => Some("Fill worked configuration"),
                        flow::Operation::SaveSelection { .. } => {
                            Some("Fill worked device selection")
                        }
                        flow::Operation::Scan => None,
                    };
                    if let Some(label) = action {
                        let r = ui.add_enabled(allowed && !packet, button(label));
                        record(
                            &mut self.metrics.lvm,
                            if matches!(step.operation, flow::Operation::SaveConfig { .. }) {
                                "fill-config"
                            } else {
                                "fill-selection"
                            },
                            &r,
                            ui,
                        );
                        if r.clicked() {
                            let result = match &step.operation {
                                flow::Operation::SaveConfig { text } => {
                                    s.config_edit(d, text.0.clone(), true)
                                }
                                flow::Operation::SaveSelection { selection } => {
                                    s.config_selection_edit(d, selection.clone(), true)
                                }
                                flow::Operation::Scan => unreachable!(),
                            };
                            match result {
                                Ok(()) => {
                                    self.dirty = true;
                                    self.notice = None;
                                }
                                Err(e) => self.notice = Some(e),
                            }
                        }
                    }
                }
            });
            record(
                &mut self.metrics.lvm,
                "example-worked",
                &worked_panel.header_response,
                ui,
            );
        } else {
            let response = lesson_presentation::assignment_responses(ui, &source);
            self.metrics.lvm.goal = Some(response.whole.rect);
            self.metrics.lvm.goal_anchor = Some(response.goal.rect);
        }
        if practice {
            ui.separator();
        }
        ui.strong("Simulated /etc/lvm/lvm.conf · devices fragment");
        let current = if practice {
            &s.config_scan_runtime.as_ref().unwrap().example
        } else {
            &s.config_scan_runtime.as_ref().unwrap().assignment
        };
        let editor_id = egui::Id::new((
            "lvm-config-editor",
            &source.id,
            source.version,
            attempt.visited,
            practice,
            self.lvm_presentation.generation[usize::from(practice)],
        ));
        let previous = egui::text_edit::TextEditState::load(ui.ctx(), editor_id).map(|state| {
            let undo = state.undoer();
            NativeBefore { state, undo }
        });
        let edit = shared_libraries_page::configuration_input(
            ui,
            &current.draft.0,
            editor_id,
            4096,
            allowed,
            "Edit the supplied devices section",
        );
        self.metrics.lvm.fields.push((
            "config-editor".into(),
            edit.response.rect,
            editor_id,
            edit.clip,
        ));
        let mut rejected = edit.rejected;
        if let Some(text) = edit.value {
            match s.config_edit(d, text, practice) {
                Ok(()) => {
                    self.dirty = true;
                    self.notice = None;
                }
                Err(e) => {
                    rejected = true;
                    self.notice = Some(e);
                    if let Some(previous) = previous {
                        previous.restore(ui.ctx(), editor_id);
                    }
                }
            }
        }
        if edit.rejected {
            self.notice=Some("Configuration text exceeds4096bytes or contains a rejected edit. Previous draft and saved work retained; nothing ran.".into());
        }
        if rejected {
            self.lvm_presentation.veto_root = Some(root);
            self.lvm_presentation.held_veto = ui.input(|i| i.pointer.primary_down());
        }
        if self.focus_input && allowed && !rejected {
            edit.response.request_focus();
            self.focus_input = false;
        }
        let runtime = s.config_scan_runtime.as_ref().unwrap();
        let live = if practice {
            &runtime.example
        } else {
            &runtime.assignment
        };
        let config_unsaved = live.draft != live.saved.config;
        if config_unsaved {
            ui.label("Unsaved configuration · scan still reads the saved version");
        }
        let mut save_config = false;
        let r = ui.add_enabled(
            allowed && !packet && !rejected,
            button("Save configuration"),
        );
        record(&mut self.metrics.lvm, "save-config", &r, ui);
        save_config |= r.clicked();
        if save_config {
            let text = if practice {
                &s.config_scan_runtime.as_ref().unwrap().example
            } else {
                &s.config_scan_runtime.as_ref().unwrap().assignment
            }
            .draft
            .clone();
            match s.config_action(d, &flow::Operation::SaveConfig { text }, practice) {
                Ok(_) => {
                    self.dirty = true;
                    self.notice = None;
                }
                Err(e) => self.notice = Some(e),
            }
        }
        let live = if practice {
            &s.config_scan_runtime.as_ref().unwrap().example
        } else {
            &s.config_scan_runtime.as_ref().unwrap().assignment
        };
        if let Some(selection) = &live.draft_file {
            ui.add_space(ui.spacing().item_spacing.y);
            ui.strong("Simulated devices-file selection");
            lesson_presentation::prose(
                ui,
                "This table models membership only. Real system.devices stores hardware IDs and is normally managed with lvmdevices; this simulator does not edit that format or execute that command.",
            );
            let mut selected = selection
                .entries
                .iter()
                .map(|e| e.0.clone())
                .collect::<std::collections::BTreeSet<_>>();
            let names = c
                .cases
                .iter()
                .flat_map(|case| case.inventory.devices.keys().cloned())
                .chain(
                    c.initial_file
                        .iter()
                        .flat_map(|f| f.entries.iter().map(|e| e.0.clone())),
                )
                .collect::<std::collections::BTreeSet<_>>();
            let mut changed = false;
            for path in names {
                let mut included = selected.contains(&path);
                let r = ui.add_enabled(
                    allowed && c.selection_editable && !packet && !rejected,
                    egui::Checkbox::new(&mut included, format!("Include {path}")),
                );
                record(&mut self.metrics.lvm, &format!("select-{path}"), &r, ui);
                if r.changed() {
                    changed = true;
                    if included {
                        selected.insert(path);
                    } else {
                        selected.remove(&path);
                    }
                }
            }
            if changed {
                let selection = flow::Selection {
                    entries: selected.into_iter().map(flow::DeviceName).collect(),
                };
                match s.config_selection_edit(d, selection, practice) {
                    Ok(()) => {
                        self.dirty = true;
                        self.notice = None;
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            if c.selection_editable {
                let runtime = s.config_scan_runtime.as_ref().unwrap();
                let live = if practice {
                    &runtime.example
                } else {
                    &runtime.assignment
                };
                if live.draft_file != live.saved.file {
                    ui.label("Unsaved selection · scan still reads saved entries");
                }
                let r = ui.add_enabled(
                    allowed && !packet && !rejected,
                    button("Save device selection"),
                );
                record(&mut self.metrics.lvm, "save-selection", &r, ui);
                if r.clicked() {
                    let selection = if practice {
                        &s.config_scan_runtime.as_ref().unwrap().example
                    } else {
                        &s.config_scan_runtime.as_ref().unwrap().assignment
                    }
                    .draft_file
                    .clone()
                    .unwrap();
                    match s.config_action(
                        d,
                        &flow::Operation::SaveSelection { selection },
                        practice,
                    ) {
                        Ok(_) => {
                            self.dirty = true;
                            self.notice = None;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
            }
        } else {
            ui.label("No devices file supplied · scan uses the filter policy");
        }
        if practice {
            lesson_presentation::practice_step(ui, &source, 1);
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(allowed && !packet && !rejected, primary(ui, "Run scan"));
            record(&mut self.metrics.lvm, "run", &r, ui);
            self.metrics.lvm.run_id = Some(r.id);
            if r.clicked() {
                let result = s.config_action(d, &flow::Operation::Scan, practice);
                self.dirty = true;
                let error = result.err();
                let live = if practice {
                    &s.config_scan_runtime.as_ref().unwrap().example
                } else {
                    &s.config_scan_runtime.as_ref().unwrap().assignment
                };
                // The actual scan area already renders its diagnostic once.
                self.notice = error.filter(
                    |e| !matches!(live.last_results(c),Ok(Some(Err(ref shown))) if shown==e),
                );
            }
            let r = ui.add_enabled(allowed && !packet && !rejected, button("Check answers"));
            record(&mut self.metrics.lvm, "check", &r, ui);
            if r.clicked() {
                if practice {
                    s.example_checked = true;
                } else {
                    s.checked = true;
                }
                self.dirty = true;
                self.notice = None;
            }
            if practice {
                let r = ui.add_enabled(allowed && !packet && !rejected, button("Reset example"));
                record(&mut self.metrics.lvm, "reset-example", &r, ui);
                if r.clicked() {
                    match s.reset_example(d) {
                        Ok(()) => {
                            self.dirty = true;
                            self.notice = None;
                            self.lvm_presentation.reset_practice();
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
            }
        });
        let runtime = s.config_scan_runtime.as_ref().unwrap();
        let live = if practice {
            &runtime.example
        } else {
            &runtime.assignment
        };
        ui.strong("Actual simulated scan results");
        if live.changed_since_run() && live.last_run.is_some() {
            ui.label(
                "Out of date · showing the previous scan. Save changes and Run scan to update it.",
            );
        }
        let actual_results = live.last_results(c);
        let shown_error = match (&actual_results, live.last_run.as_ref() == Some(&live.saved)) {
            (Ok(Some(Err(e))), true) => Some(e.clone()),
            _ => None,
        };
        let output = ui.scope(|ui| match actual_results {
            Err(e) => {
                ui.label(format!("Results unavailable: {e}"));
            }
            Ok(None) => {
                ui.label("Not run yet · save your changes, then Run scan");
            }
            Ok(Some(Err(e))) => {
                let r = ui.colored_label(palette(ui).error, format!("Scan failed · {e}"));
                record(&mut self.metrics.lvm, "scan-error", &r, ui);
            }
            Ok(Some(Ok(reports))) => {
                for (case_index, (case, report)) in c.cases.iter().zip(&reports).enumerate() {
                    ui.strong(&case.label);
                    ui.label(mode(report.mode));
                    for (record_index, r) in report.records.iter().enumerate() {
                        let metadata = match &r.metadata {
                            Some(core::Metadata::PhysicalVolume { group: Some(group) }) => {
                                format!("PV label read · group {group}")
                            }
                            Some(core::Metadata::PhysicalVolume { group: None }) => {
                                "PV label read · not in a group".into()
                            }
                            Some(core::Metadata::NoPhysicalVolumeLabel) => {
                                "No modeled PV label".into()
                            }
                            None => "Metadata not read".into(),
                        };
                        let status = if r.allowed { "Allowed" } else { "Excluded" };
                        let text =
                            format!("{} · {status}\n{metadata}\n{}", r.path, reason(&r.reason));
                        let response = lesson_presentation::monospace(ui, &text, true);
                        self.metrics.lvm.reader_rows.push((
                            format!("scan-{case_index}-{record_index}"),
                            response.rect,
                            ui.clip_rect(),
                        ));
                        self.metrics
                            .lvm
                            .reader_records
                            .push((format!("scan-{case_index}-{}", r.path), r.allowed));
                        #[cfg(test)]
                        self.metrics
                            .lvm
                            .inspected_files
                            .push((r.path.clone(), text));
                        ui.add_space(ui.spacing().item_spacing.y);
                    }
                    if !report.unresolved_file_entries.is_empty() {
                        ui.label(format!(
                            "Missing from this supplied inventory: {}",
                            report.unresolved_file_entries.join(", ")
                        ));
                    }
                }
            }
        });
        self.metrics.output = Some(output.response.rect);
        if let Some(e) = &self.notice {
            let r = ui.colored_label(palette(ui).error, format!("Needs attention · {e}"));
            self.metrics.lvm.notice = Some(r.rect);
        }
        let checked = if practice {
            s.example_checked
        } else {
            s.checked
        };
        let rows = if practice {
            d.practice_checks(s)
        } else {
            d.checks(s)
        };
        let complete = !rows.is_empty() && rows.iter().all(|r| r.0);
        if checked {
            let failures = rows.iter().filter(|r| !r.0).count();
            if complete {
                ui.strong(if practice {
                    "Example checks pass"
                } else {
                    "Assignment checks pass"
                });
                if !practice {
                    lesson_presentation::prose(ui, &source.explanation);
                }
            } else {
                ui.strong(format!(
                    "Needs attention · {failures} {}",
                    if failures == 1 { "check" } else { "checks" }
                ));
            }
            for (ok, text) in &rows {
                // The current scan diagnostic is already visible above. Keep
                // every objective row intact in the requested full-results view.
                if !ok && shown_error.as_ref() != Some(text) {
                    let r =
                        ui.colored_label(palette(ui).error, format!("Needs attention · {text}"));
                    self.metrics.lvm.rows.push((*ok, r.rect));
                }
            }
            controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
                for (ok, text) in &rows {
                    let r = ui.label(format!(
                        "{} · {text}",
                        if *ok { "Passed" } else { "Needs attention" }
                    ));
                    self.metrics.lvm.rows.push((*ok, r.rect));
                    #[cfg(test)]
                    self.metrics.lvm.feedback_texts.push(text.clone());
                }
            });
        }
        if practice {
            ui.add_space(ui.spacing().item_spacing.y);
            lesson_presentation::practice_step(ui, &source, 2);
            if lesson_presentation::entry(&source).is_none() {
                lesson_presentation::prose(ui, &d.lesson.try_change);
            }
        } else {
            self.metrics.completion = progress::current_solved(attempt, &source);
            ui.push_id(
                ("lvm-config-help", &source.id, attempt.visited, challenge),
                |ui| {
                    let help = controls::collapsing_header(ui, "Optional help").show(ui, |ui| {
                        if attempt.hints < source.hints.len() {
                            let r = ui.add_enabled(
                                allowed && !packet && !rejected,
                                button("Show a hint"),
                            );
                            record(&mut self.metrics.lvm, "show-hint", &r, ui);
                            if r.clicked() {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                        }
                        for text in source.hints.iter().take(attempt.hints) {
                            lesson_presentation::prose(ui, text);
                            #[cfg(test)]
                            self.metrics.lvm.requested_hints.push(text.clone());
                        }
                        let r = controls::collapsing_header(
                            ui,
                            "Show worked answer · reveals a solution",
                        )
                        .show(ui, |ui| {
                            for op in &d.config_scan_task.as_ref().unwrap().reference {
                                worked(ui, op);
                                #[cfg(test)]
                                self.metrics.lvm.worked_commands.push(op.label().into());
                            }
                        });
                        record(
                            &mut self.metrics.lvm,
                            "worked-approach",
                            &r.header_response,
                            ui,
                        );
                        let r = controls::collapsing_header(ui, "References and simulation limits")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.simplifications);
                                for url in &source.sources {
                                    ui.hyperlink_to(url, url);
                                }
                            });
                        record(&mut self.metrics.lvm, "references", &r.header_response, ui);
                    });
                    record(
                        &mut self.metrics.lvm,
                        "optional-help",
                        &help.header_response,
                        ui,
                    );
                },
            );
            if complete && checked && allowed && !packet && !rejected {
                self.next_unit_lab(ui, i);
            }
        }
    }
}
