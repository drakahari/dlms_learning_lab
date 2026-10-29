//! Native widgets for the bounded LVM session; never a host terminal.
use super::*;
use dlms_learning_lab::{engine::State, lvm_workspace as model, script_teaching::View};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub run_id: Option<egui::Id>,
    pub notice: Option<egui::Rect>,
    pub example_choices: Vec<(usize, egui::Rect, egui::Rect)>,
    #[cfg(test)]
    pub capacity_texts: Vec<String>,
    #[cfg(test)]
    pub feedback_texts: Vec<String>,
    #[cfg(test)]
    pub inspected_files: Vec<(String, String)>,
    pub files_viewport: Option<egui::Rect>,
    #[cfg(test)]
    pub requested_hints: Vec<String>,
    #[cfg(test)]
    pub worked_commands: Vec<String>,
    #[cfg(test)]
    pub reference_urls: Vec<String>,
}
#[derive(Default)]
pub(super) struct Presentation {
    veto_root: Option<u64>,
    held_veto: bool,
    generation: [u64; 2],
}
impl Presentation {
    pub(super) fn reset_all(&mut self) {
        self.generation[0] = self.generation[0].wrapping_add(1);
        self.generation[1] = self.generation[1].wrapping_add(1);
        self.veto_root = None;
        self.held_veto = false;
    }
    fn reset_practice(&mut self) {
        self.generation[1] = self.generation[1].wrapping_add(1);
        self.veto_root = None;
        self.held_veto = false;
    }
}
// Native edits retain the complete old draft and native undo/selection state.
// The field never truncates an insertion to make an invalid packet executable.
struct NativeEditBuffer<'a> {
    value: &'a mut String,
    rejected: bool,
    received: bool,
}
impl egui::TextBuffer for NativeEditBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<NativeEditBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, at: usize) -> usize {
        if text.is_empty() || self.rejected {
            return 0;
        }
        self.received = true;
        if text.contains(['\0', '\n', '\r'])
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > 2048)
        {
            self.rejected = true;
            return 0;
        }
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if !self.rejected {
            egui::TextBuffer::delete_char_range(self.value, range);
        }
    }
}
struct NativeBefore {
    state: egui::text_edit::TextEditState,
    undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
}
impl NativeBefore {
    fn restore(mut self, ctx: &egui::Context, id: egui::Id) {
        self.state.set_undoer(self.undo);
        self.state.store(ctx, id);
    }
}
impl LabApp {
    pub(super) fn lvm_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let source = self.scenarios[i].clone();
        if let Err(e) = model::context(&self.progress.attempts[&source.id], &source) {
            ui.label(format!("LVM work retained without applying changes: {e}"));
            return;
        }
        let engine::Goal::Lvm { definition } = &source.goal else {
            return;
        };
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        let inherited = self.lvm_presentation.held_veto;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.lvm_presentation.held_veto = false;
        }
        let blocked = inherited || self.lvm_presentation.veto_root == Some(root);
        // Header actions wait until an edit packet has been consumed by its
        // actual field; an ignored/oversized packet cannot dispatch an old action.
        let phase = (
            self.selected,
            self.reset_pending,
            self.progress.attempts[&source.id].challenge,
        );
        let challenge = ui
            .add_enabled_ui(!packet && !blocked, |ui| self.workspace_header(ui, i))
            .inner;
        if phase
            != (
                self.selected,
                self.reset_pending,
                self.progress.attempts[&source.id].challenge,
            )
        {
            return;
        }
        let mut navigated = false;
        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        if let Err(e) = model::context(attempt, &source) {
            ui.colored_label(palette(ui).error, e);
            return;
        }
        let State::Lvm {
            simulation: session,
        } = &mut attempt.state
        else {
            return;
        };
        let view = if challenge {
            View::Assignment
        } else {
            session.view
        };
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label, key) in [
                    (View::Introduction, "Introduction", "introduction"),
                    (View::Example, "Separate example · ungraded", "example"),
                    (View::Assignment, "Assignment", "assignment"),
                ] {
                    let r = ui.add_enabled(
                        !packet && !blocked,
                        egui::Button::new(label).selected(view == v),
                    );
                    self.metrics
                        .lvm
                        .controls
                        .push((key.into(), r.rect, ui.clip_rect()));
                    if r.clicked() {
                        match session.set_view(definition, v) {
                            Ok(()) => {
                                navigated = true;
                                self.dirty = true;
                                self.notice = None;
                            }
                            Err(e) => self.notice = Some(e),
                        }
                    }
                }
            });
        }
        if navigated {
            return;
        }
        ui.heading(&source.title);
        if view == View::Introduction {
            ui.label(&definition.lesson.prerequisite_note);
            for c in &definition.lesson.concepts {
                ui.strong(&c.title);
                ui.label(&c.text);
            }
            ui.strong("A different runnable example");
            ui.label(&definition.lesson.description);
            for c in &definition.lesson.worked {
                ui.monospace(&c.text);
                ui.label(&c.explanation);
            }
            ui.label(&definition.lesson.try_change);
            ui.label("Open Separate example to insert and run one command at a time. You may go straight to Assignment. No example work is installed in the assignment.");
            return;
        }
        let practice = view == View::Example;
        if practice {
            ui.strong("Separate example · No assignment credit");
            ui.label(&definition.lesson.description);
            let mut selected = session.selected_example;
            let r = ui
                .add_enabled_ui(!packet && !blocked, |ui| {
                    controls::dropdown(
                        ui,
                        ("lvm-example", &source.id, attempt.visited),
                        ui.available_width(),
                    )
                    .selected_text(&definition.lesson.worked[selected].text)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, c) in definition.lesson.worked.iter().enumerate() {
                            let r = ui.selectable_value(&mut selected, n, &c.text);
                            self.metrics
                                .lvm
                                .example_choices
                                .push((n, r.rect, ui.clip_rect()));
                        }
                    })
                })
                .inner;
            self.metrics.terminal_example_picker = Some(r.response.rect);
            if selected != session.selected_example {
                match session.select_example(definition, selected) {
                    Ok(()) => {
                        navigated = true;
                        self.dirty = true;
                        self.notice = None;
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(!packet && !blocked, button("Insert selected command"));
                self.metrics.insert = Some(r.rect);
                self.metrics
                    .lvm
                    .controls
                    .push(("insert".into(), r.rect, ui.clip_rect()));
                if r.clicked() {
                    match session.insert(definition, selected) {
                        Ok(()) => {
                            navigated = true;
                            self.dirty = true;
                            self.notice = None;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
                let r = ui.add_enabled(!packet && !blocked, button("Reset example only"));
                self.metrics
                    .lvm
                    .controls
                    .push(("reset-example".into(), r.rect, ui.clip_rect()));
                if r.clicked() {
                    match session.reset_example(definition) {
                        Ok(()) => {
                            self.lvm_presentation.reset_practice();
                            navigated = true;
                            self.dirty = true;
                            self.notice = None;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
            });
            if navigated {
                return;
            }
            ui.label(&definition.lesson.worked[selected].explanation);
            ui.label(format!("Try a change: {}", definition.lesson.try_change));
            ui.monospace(&definition.lesson.experiment);
            ui.monospace(&definition.lesson.repair);
        } else {
            let r = ui.label(&source.objective);
            self.metrics.lvm.goal = Some(r.rect);
        }
        ui.label("Simulated storage terminal · pvs, vgs and lvs report current state. No host command runs.");
        let mut command = if practice {
            session.example_command.clone()
        } else {
            session.command.clone()
        };
        let id = egui::Id::new((
            "lvm-command",
            &source.id,
            source.version,
            attempt.visited,
            practice,
            self.lvm_presentation.generation[usize::from(practice)],
        ));
        let old_native = ui
            .input(|i| !i.events.is_empty())
            .then(|| {
                egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
                    let undo = state.undoer();
                    NativeBefore { state, undo }
                })
            })
            .flatten();
        let active = ui.is_enabled()
            && !blocked
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        // Inspect every text event before egui can stop reading at Enter or
        // ignore a paste delivered to an unfocused field. No payload is copied.
        let invalid_packet = ui.input(|i| {
            i.events.iter().any(|e| match e {
                egui::Event::Text(t) | egui::Event::Paste(t) => {
                    t.len() > 2048 || t.contains(['\0', '\n', '\r'])
                }
                _ => false,
            })
        });
        let after_return = ui.input(|i| {
            let mut returned = false;
            i.events.iter().any(|e| {
                if matches!(e, egui::Event::Key {key:egui::Key::Enter,pressed:true,modifiers,..}
                    if modifiers.matches_logically(egui::Modifiers::NONE))
                {
                    returned = true;
                }
                returned
                    && matches!(e, egui::Event::Text(t) | egui::Event::Paste(t) if !t.is_empty())
            })
        });
        let mut buffer = NativeEditBuffer {
            value: &mut command,
            rejected: invalid_packet,
            received: false,
        };
        let edit = ui.add_enabled(
            active,
            controls::singleline(ui, &mut buffer)
                .hint_text("e.g. lvs")
                .id(id)
                .desired_width(ui.available_width()),
        );
        let mut rejected = buffer.rejected;
        let ignored = packet && (!buffer.received || after_return);
        self.metrics
            .lvm
            .fields
            .push(("command".into(), edit.rect, id, ui.clip_rect()));
        self.metrics.inputs.push(edit.rect);
        if !rejected && !ignored && edit.changed() {
            match session.set_command(definition, command, practice) {
                Ok(()) => {
                    self.dirty = true;
                    self.notice = None;
                }
                Err(e) => {
                    rejected = true;
                    self.notice = Some(e);
                }
            }
        }
        if rejected || ignored {
            self.lvm_presentation.veto_root = Some(root);
            self.lvm_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
            if let Some(state) = old_native {
                state.restore(ui.ctx(), id);
            }
            self.notice = Some(if rejected {
                "That edit exceeds the command limit or contains a line break. The previous draft and selection are retained; nothing ran."
            } else {
                "That text was not applied. Click the command field, retry the edit, then choose the action again."
            }.into());
        }
        let allowed = active && !rejected && !ignored;
        let enter = (edit.has_focus() || edit.lost_focus())
            && ui.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    && i.modifiers.matches_logically(egui::Modifiers::NONE)
            });
        let (run, check) = ui
            .horizontal_wrapped(|ui| {
                let run = ui.add_enabled(allowed, primary(ui, "Run simulated command"));
                let check = ui.add_enabled(
                    allowed,
                    primary(
                        ui,
                        if practice {
                            "Check separate example"
                        } else {
                            "Check work"
                        },
                    ),
                );
                (run, check)
            })
            .inner;
        self.metrics.terminal_run = Some(run.rect);
        self.metrics.lvm.run_id = Some(run.id);
        self.metrics
            .lvm
            .controls
            .push(("run".into(), run.rect, ui.clip_rect()));
        self.metrics.check = Some(check.rect);
        self.metrics
            .lvm
            .controls
            .push(("check".into(), check.rect, ui.clip_rect()));
        let check_clicked = check.clicked();
        let command_action = allowed && (run.clicked() || enter);
        if command_action {
            let text = if practice {
                session.example_command.clone()
            } else {
                session.command.clone()
            };
            if !text.trim().is_empty() {
                match session.execute(definition, &text, practice) {
                    Ok(_) => {
                        self.dirty = true;
                        self.notice = None;
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
        }
        if check_clicked {
            self.notice = None;
            if practice {
                session.example_checked = true
            } else {
                session.checked = true
            }
            self.dirty = true;
        }
        let checked = if practice {
            session.example_checked
        } else {
            session.checked
        };
        let storage = if practice {
            &session.example
        } else {
            &session.current
        };
        let rows = checked.then(|| {
            if practice {
                definition.practice_checks(session)
            } else {
                definition.checks(session)
            }
        });
        let targets = if practice {
            &definition.lesson.targets
        } else {
            &definition.targets
        };
        let expected_rows = targets
            .iter()
            .map(|t| 3 + usize::from(t.fill_lv))
            .sum::<usize>()
            + 1;
        let capacity_rows = rows.as_ref().is_some_and(|r| r.len() == expected_rows);
        if let Some(text) = &self.notice {
            let r = ui.colored_label(palette(ui).error, format!("Needs attention · {text}"));
            self.metrics.lvm.notice = Some(r.rect);
            self.metrics
                .lvm
                .controls
                .push(("notice".into(), r.rect, ui.clip_rect()));
        }
        if self.notice.is_none()
            && let Some(rows) = &rows
        {
            let failures = rows.iter().filter(|r| !r.0).count();
            if let Some((index, (_, text))) = rows.iter().enumerate().find(|(_, r)| !r.0) {
                let first = if capacity_rows {
                    capacity_check_text(index, text.clone(), storage, targets)
                } else {
                    text.clone()
                };
                ui.colored_label(
                    palette(ui).error,
                    format!(
                        "Needs attention · {failures} of {} checks. First issue: {first}",
                        rows.len()
                    ),
                );
            } else {
                ui.colored_label(
                    palette(ui).success,
                    format!("Passed · all {} checks", rows.len()),
                );
            }
        }
        let output = if practice {
            &session.example_transcript
        } else {
            &session.transcript
        };
        let h = ui.text_style_height(&egui::TextStyle::Monospace) * 6.;
        self.metrics.output = Some(if output.is_empty() {
            ui.label("No command output yet.").rect
        } else {
            egui::ScrollArea::vertical()
                .id_salt(("lvm-output", &source.id, practice))
                .max_height(h)
                // Nested main scrolling can offer zero remaining height.
                // Reserve six font-derived rows rather than inheriting zero.
                .min_scrolled_height(h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for row in output {
                        ui.monospace(format!("virtual $ {}", row.input));
                        ui.add(
                            egui::Label::new(RichText::new(&row.output).monospace())
                                .wrap()
                                .selectable(true),
                        );
                    }
                })
                .inner_rect
        });
        ui.strong("Modeled LV and filesystem capacity");
        for (vg, g) in &storage.groups {
            for (lv, v) in &g.volumes {
                if let Some(fs) = &v.filesystem {
                    let text = format!(
                        "{vg}/{lv}: LV {} · filesystem {}",
                        capacity(v.extents.len() as u64 * g.extent_bytes),
                        capacity(fs.blocks * dlms_learning_lab::lvm::BLOCK_BYTES)
                    );
                    #[cfg(test)]
                    self.metrics.lvm.capacity_texts.push(text.clone());
                    ui.label(text);
                }
            }
        }
        let details = ui
            .push_id(
                (
                    "lvm-retained-files",
                    &source.id,
                    source.version,
                    practice,
                    challenge,
                ),
                |ui| {
                    controls::collapsing_header(
                        ui,
                        "Inspect retained files and exact byte capacities",
                    )
                    .show(ui, |ui| {
                        let files = egui::ScrollArea::vertical()
                            .id_salt(("lvm-files", &source.id, practice))
                            .max_height(h)
                            .min_scrolled_height(h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (vg, g) in &storage.groups {
                                    for (lv, v) in &g.volumes {
                                        if let Some(fs) = &v.filesystem {
                                            ui.strong(format!("{vg}/{lv}"));
                                            ui.label(format!(
                                                "LV {} bytes; filesystem {} bytes",
                                                v.extents.len() as u64 * g.extent_bytes,
                                                fs.blocks * dlms_learning_lab::lvm::BLOCK_BYTES
                                            ));
                                            if fs.files.is_empty() {
                                                ui.label("No retained files in this filesystem.");
                                            }
                                            for (path, file) in &fs.files {
                                                #[cfg(test)]
                                                self.metrics
                                                    .lvm
                                                    .inspected_files
                                                    .push((path.clone(), file.contents.clone()));
                                                ui.strong(path);
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(&file.contents).monospace(),
                                                    )
                                                    .wrap()
                                                    .selectable(true),
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                        self.metrics.lvm.files_viewport = Some(files.inner_rect);
                    })
                },
            )
            .inner;
        self.metrics.lvm.controls.push((
            "inspect-files".into(),
            details.header_response.rect,
            ui.clip_rect(),
        ));
        if let Some(rows) = rows {
            for (index, (ok, text)) in rows.into_iter().enumerate() {
                let text = if capacity_rows {
                    capacity_check_text(index, text, storage, targets)
                } else {
                    text
                };
                #[cfg(test)]
                self.metrics.lvm.feedback_texts.push(text.clone());
                let r = ui.colored_label(
                    if ok {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    format!("{} · {text}", if ok { "Passed" } else { "Needs attention" }),
                );
                self.metrics.lvm.rows.push((ok, r.rect));
            }
        }
        if !practice {
            self.metrics.completion = progress::current_solved(attempt, &source);
            if self.metrics.completion && !checked {
                ui.strong("Current assignment checks pass");
            }
        }
        if !practice {
            let help = ui
                .push_id(
                    (
                        "lvm-help",
                        &source.id,
                        source.version,
                        attempt.visited,
                        challenge,
                    ),
                    |ui| {
                        controls::collapsing_header(ui, "Optional help").show(ui, |ui| {
                            if attempt.hints < source.hints.len() {
                                let hint =
                                    ui.add_enabled(allowed && !packet, button("Show a hint"));
                                self.metrics.lvm.controls.push((
                                    "show-hint".into(),
                                    hint.rect,
                                    ui.clip_rect(),
                                ));
                                if hint.clicked() {
                                    attempt.hints += 1;
                                    self.dirty = true;
                                }
                            }
                            for (n, text) in source.hints.iter().take(attempt.hints).enumerate() {
                                #[cfg(test)]
                                self.metrics.lvm.requested_hints.push(text.clone());
                                ui.label(format!("Hint {}: {text}", n + 1));
                            }
                            let worked = controls::collapsing_header(
                                ui,
                                "Worked approach · reveals a complete solution",
                            )
                            .show(ui, |ui| {
                                ui.label(&source.explanation);
                                for command in &source.solution {
                                    #[cfg(test)]
                                    self.metrics.lvm.worked_commands.push(command.clone());
                                    ui.monospace(command);
                                }
                            });
                            self.metrics.lvm.controls.push((
                                "worked-approach".into(),
                                worked.header_response.rect,
                                ui.clip_rect(),
                            ));
                            let refs =
                                controls::collapsing_header(ui, "References").show(ui, |ui| {
                                    for url in &source.sources {
                                        #[cfg(test)]
                                        self.metrics.lvm.reference_urls.push(url.clone());
                                        ui.hyperlink_to(url, url);
                                    }
                                });
                            self.metrics.lvm.controls.push((
                                "references".into(),
                                refs.header_response.rect,
                                ui.clip_rect(),
                            ));
                        })
                    },
                )
                .inner;
            self.metrics.lvm.controls.push((
                "optional-help".into(),
                help.header_response.rect,
                ui.clip_rect(),
            ));
        }
        if !practice
            && self.metrics.completion
            && self.selected == Some(i)
            && allowed
            && !packet
            && !command_action
            && !check_clicked
        {
            self.next_unit_lab(ui, i);
            if let Some(rect) = self.metrics.next_unit {
                self.metrics
                    .lvm
                    .controls
                    .push(("next".into(), rect, ui.clip_rect()));
            }
        }
    }
}

/// Binary capacities match the supported LVM M/G meanings. Exact bytes remain on request.
fn capacity(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        if bytes.is_multiple_of(MIB) {
            format!("{} MiB", bytes / MIB)
        } else {
            format!("≈{:.2} MiB", bytes as f64 / MIB as f64)
        }
    } else if bytes >= 1024 {
        if bytes.is_multiple_of(1024) {
            format!("{} KiB", bytes / 1024)
        } else {
            format!("≈{:.2} KiB", bytes as f64 / 1024.)
        }
    } else {
        format!("{bytes} B")
    }
}
/// Format known capacity rows from their actual modeled values; no English parsing
/// and no change to the model's truth bits, row order or diagnostic fallback.
fn capacity_check_text(
    index: usize,
    fallback: String,
    storage: &dlms_learning_lab::lvm::State,
    targets: &[model::Target],
) -> String {
    let mut first = 0;
    for target in targets {
        let value = if index == first {
            storage
                .lv_bytes(&target.volume)
                .ok()
                .map(|n| ("LV", n, target.minimum_lv))
        } else if index == first + 1 {
            storage
                .filesystem_bytes(&target.volume)
                .ok()
                .map(|n| ("filesystem", n, target.minimum_filesystem))
        } else {
            None
        };
        if let Some((name, current, required)) = value {
            return format!(
                "{}: Current {name} {}; required at least {}",
                target.volume,
                capacity(current),
                capacity(required)
            );
        }
        first += 3 + usize::from(target.fill_lv);
    }
    fallback
}
