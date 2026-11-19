//! Native widgets for the bounded LVM session; never a host terminal.
mod config_scan;
mod device_reference;
mod mounts;
mod snapshot;
use super::*;
use dlms_learning_lab::{engine::State, lvm_workspace as model, script_teaching::View};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    /// Complete task extent, including all requirements.
    pub goal: Option<egui::Rect>,
    /// Actual lead Label response; scroll anchor, not whole-task coverage.
    pub goal_anchor: Option<egui::Rect>,
    pub run_id: Option<egui::Id>,
    pub notice: Option<egui::Rect>,
    pub example_choices: Vec<(usize, egui::Rect, egui::Rect)>,
    #[cfg(test)]
    pub capacity_texts: Vec<String>,
    #[cfg(test)]
    pub physical_facts: Vec<String>,
    #[cfg(test)]
    pub feedback_texts: Vec<String>,
    #[cfg(test)]
    pub inspected_files: Vec<(String, String)>,
    pub files_viewport: Option<egui::Rect>,
    pub reader_records: Vec<(String, bool)>,
    pub reader_rows: Vec<(String, egui::Rect, egui::Rect)>,
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
    limit: usize,
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
                .is_none_or(|n| n > self.limit)
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
        if definition.config_scan_task.is_some() {
            self.lvm_config_scan_page(ui, i);
            return;
        }
        let example_steps = definition.example_steps();
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
            lesson_presentation::concepts(
                ui,
                &source,
                &definition.lesson.prerequisite_note,
                definition
                    .lesson
                    .concepts
                    .iter()
                    .map(|c| (c.title.clone(), c.text.clone())),
            );
            lesson_presentation::worked(ui, &definition.lesson.description);
            if let Some(p) = &definition.lesson.mount_practice {
                for step in &p.worked {
                    mounts::worked(ui, &step.operation, &step.explanation);
                }
            } else if definition.initial.pv_capacity.is_some()
                || definition.snapshot_task.is_some()
                || definition.device_reference_task.is_some()
            {
                for command in &example_steps {
                    lesson_presentation::command(ui, &command.text, &command.explanation);
                }
            } else {
                lesson_presentation::worked_rows(ui, &source);
            }
            controls::collapsing_header(ui, "Explore after the worked example").show(ui, |ui| {
                lesson_presentation::prose(ui, &definition.lesson.try_change);
            });
            if definition.mount_task.is_some()
                || definition.device_reference_task.is_some()
                || definition.snapshot_task.is_some()
                || definition.plan.is_some()
                || definition.retirement.is_some()
                || definition.availability.is_some()
                || definition.renaming.is_some()
                || definition.targets.iter().any(|t| !t.pv_sizes.is_empty())
            {
                let mut next = None;
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(!packet && !blocked, primary(ui, "Try this example"));
                    self.metrics.lvm.controls.push((
                        "practice-start".into(),
                        r.rect,
                        ui.clip_rect(),
                    ));
                    if r.clicked() {
                        next = Some(View::Example);
                    }
                    if ui
                        .add_enabled(!packet && !blocked, button("Go straight to assignment"))
                        .clicked()
                    {
                        next = Some(View::Assignment);
                    }
                });
                if let Some(next) = next {
                    match session.set_view(definition, next) {
                        Ok(()) => {
                            self.dirty = true;
                            self.lesson_scroll_reset = true;
                            self.focus_input = true;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
            }
            return;
        }
        let practice = view == View::Example;
        if practice {
            lesson_presentation::practice_step(ui, &source, 0);
            let mut selected = session.selected_example;
            let r = ui
                .add_enabled_ui(!packet && !blocked, |ui| {
                    controls::dropdown(
                        ui,
                        ("lvm-example", &source.id, attempt.visited),
                        ui.available_width(),
                    )
                    .selected_text(&example_steps[selected].text)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, c) in example_steps.iter().enumerate() {
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
            let simulator_step = definition
                .lesson
                .snapshot_practice
                .as_ref()
                .and_then(|p| p.worked.get(selected))
                .filter(|s| {
                    !matches!(s.operation, model::snapshot_flow::Operation::Command { .. })
                });
            let reference_step = definition
                .lesson
                .device_reference_practice
                .as_ref()
                .and_then(|p| p.worked.get(selected))
                .filter(|_| definition.reference_helper(selected));
            let mount_step = definition
                .lesson
                .mount_practice
                .as_ref()
                .and_then(|p| p.worked.get(selected))
                .filter(|_| definition.mount_helper(selected));
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(
                    !packet && !blocked,
                    button(if mount_step.is_some() {
                        "Fill selected example"
                    } else if simulator_step.is_some() || reference_step.is_some() {
                        "Apply selected simulator action"
                    } else {
                        "Insert selected command"
                    }),
                );
                self.metrics.insert = Some(r.rect);
                self.metrics
                    .lvm
                    .controls
                    .push(("insert".into(), r.rect, ui.clip_rect()));
                if r.clicked() {
                    let result = if let Some(step) = mount_step {
                        let model::mount_flow::Operation::SavePolicy { text } = &step.operation
                        else {
                            unreachable!()
                        };
                        session.edit_mount(definition, text.clone(), true)
                    } else if let Some(step) = reference_step {
                        session
                            .apply_reference(definition, &step.operation, true)
                            .map(|_| ())
                    } else if let Some(step) = simulator_step {
                        session
                            .apply_snapshot(definition, &step.operation, true)
                            .map(|_| ())
                    } else {
                        session.insert(definition, selected)
                    };
                    match result {
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
            if let Some(step) = mount_step {
                mounts::worked(ui, &step.operation, &example_steps[selected].explanation);
                ui.label("Fill copies this example into its editor only. Choose Save configuration before using mount.");
            } else {
                ui.label(&example_steps[selected].explanation);
            }
            controls::collapsing_header(ui, if definition.lesson.mount_practice.is_some() || definition.lesson.snapshot_practice.is_some() || definition.lesson.device_reference_practice.is_some(){"Optional exploration · after the worked example"}else{"Experiment commands · after the worked example"}).show(
                ui,
                |ui| {
                    if let Some(p)=&definition.lesson.mount_practice {
                        lesson_presentation::prose(ui,&definition.lesson.try_change);
                        mounts::worked(ui,&p.experiment,"Make this deliberate change in the separate example; check which actual requirement now fails.");
                        for op in &p.repair {mounts::worked(ui,op,"Repair this practice change, inspect the mounted data and check again.");}
                        return;
                    }
                    if let Some(p)=&definition.lesson.device_reference_practice {
                        lesson_presentation::prose(ui,&definition.lesson.try_change);
                        lesson_presentation::command(ui,&p.experiment.description(&p.context),"Save this deliberate practice mistake, inspect the actual reader, then check again.");
                        for op in &p.repair { lesson_presentation::command(ui,&op.description(&p.context),"Repair the separate example, then check again."); }
                        return;
                    }
                    if let Some(p)=&definition.lesson.snapshot_practice {
                        lesson_presentation::prose(ui,&definition.lesson.try_change);
                        lesson_presentation::command(ui,&p.experiment.description(&p.context),"Make this deliberate change, inspect the records, then check the example again.");
                        for operation in &p.repair {
                            lesson_presentation::command(ui,&operation.description(&p.context),"Repair this practice change, then check again.");
                        }
                        if p.repair.is_empty(){ui.label("An exhausted saved view cannot be revived. Reset the example to try again with sufficient COW space before the writes.");}
                        return;
                    }
                    lesson_presentation::command(
                        ui,
                        &definition.lesson.experiment,
                        "Create the described incomplete change.",
                    );
                    lesson_presentation::command(
                        ui,
                        &definition.lesson.repair,
                        if !definition.lesson.targets.iter().all(|t| t.pv_sizes.is_empty()) {
                            "Restore the requested recognized PV end; retain the existing data and other capacity layers."
                        } else if definition.renaming.is_some() {
                            "Restore the requested name of the same original object."
                        } else if definition.availability.is_some() {
                            "Restore only the requested availability state."
                        } else if definition.retirement.is_some() {
                            "Retire only the reintroduced raw allocation or PV label."
                        } else if definition.plan.is_some() {
                            "Recreate the requested new raw allocation."
                        } else {
                            "Restore filesystem-to-LV agreement."
                        },
                    );
                },
            );
        } else {
            let r = lesson_presentation::assignment_responses(ui, &source);
            self.metrics.lvm.goal = Some(r.whole.rect);
            self.metrics.lvm.goal_anchor = Some(r.goal.rect);
        }
        if practice {
            lesson_presentation::practice_step(ui, &source, 1);
        }
        if definition.snapshot_task.is_some() {
            let result = snapshot::tools(
                ui,
                definition,
                session,
                practice,
                !packet
                    && !blocked
                    && !self.reset_pending
                    && !self.settings_open
                    && !self.theme_preview
                    && !self.data_tools.modal_open(),
                &mut self.metrics.lvm,
            );
            if result.changed {
                self.dirty = true;
                self.notice = None;
            }
            if let Some(error) = result.error {
                self.notice = Some(error);
            }
        }
        let reference_edit = device_reference::tools(
            ui,
            definition,
            session,
            practice,
            (
                !blocked
                    && !self.reset_pending
                    && !self.settings_open
                    && !self.theme_preview
                    && !self.data_tools.modal_open(),
                packet,
            ),
            (
                &source.id,
                source.version,
                attempt.visited,
                self.lvm_presentation.generation[usize::from(practice)],
            ),
            &mut self.metrics.lvm,
        );
        if reference_edit.changed {
            self.dirty = true;
            self.notice = None;
        }
        if let Some(error) = reference_edit.error {
            self.notice = Some(error);
        }
        let mount_edit = mounts::tools(
            ui,
            definition,
            session,
            practice,
            (
                !blocked
                    && !self.reset_pending
                    && !self.settings_open
                    && !self.theme_preview
                    && !self.data_tools.modal_open(),
                packet,
            ),
            (
                &source.id,
                source.version,
                attempt.visited,
                self.lvm_presentation.generation[usize::from(practice)],
            ),
            &mut self.metrics.lvm,
        );
        if mount_edit.changed {
            self.dirty = true;
            self.notice = None;
        }
        if let Some(error) = mount_edit.error {
            self.notice = Some(error);
        }
        let supplied = if practice {
            &session.example
        } else {
            &session.current
        };
        if supplied.provisioning.is_some() || supplied.pv_capacity.is_some() {
            controls::collapsing_header(ui, "Supplied device facts").default_open(definition.device_reference_task.is_none()&&definition.mount_task.is_none()).show(ui, |ui| {
                lesson_presentation::prose(ui, if supplied.pv_capacity.is_some(){"Device and recognized PV sizes are separate. This exercise reserves 1 MiB before data; each group allocates whole physical extents (PEs). Placement facts below are derived simulator facts, not extra pvs columns."}else{"pvs lists initialized physical volumes. This supplied inventory also shows blank devices and data you must preserve."});
                let rows=supplied.devices.iter().map(|(path,d)| {
                    let role=match d.role {
                        dlms_learning_lab::lvm::DeviceRole::Blank=>if supplied.provisioning == Some(dlms_learning_lab::lvm::provision::Profile::Lifecycle) {"No modeled PV label · not secure erasure".to_owned()}else{"Blank · available for initialization".to_owned()},
                        dlms_learning_lab::lvm::DeviceRole::Foreign=>"Contains other data · leave unchanged".to_owned(),
                        dlms_learning_lab::lvm::DeviceRole::PhysicalVolume=>if d.group.is_empty(){"Physical volume · not in a group".to_owned()}else{format!("Physical volume · {}",d.group)},
                    };
                    let mut text=format!("{} · {role}",capacity(d.bytes));
                    if let Some(recognized)=d.pv_bytes {
                        text=format!("Device {} · recognized PV {} · {role}",capacity(d.bytes),capacity(recognized));
                        if let Some(g)=supplied.groups.get(&d.group){
                            let count=(recognized-dlms_learning_lab::lvm::DATA_START)/g.extent_bytes;
                            let high=dlms_learning_lab::lvm::pv_capacity::highest_allocated(supplied,path).ok().flatten().map_or_else(||"none".to_owned(),|v|v.to_string());
                            text.push_str(&format!(" · {count} PEs of {} · highest allocated PE {high} (numbering starts at 0)",capacity(g.extent_bytes)));
                        }
                        #[cfg(test)] self.metrics.lvm.physical_facts.push(format!("{path}: {text}"));
                    }
                    (path,text)
                }).collect::<Vec<_>>();
                if ui.available_width()<ui.text_style_height(&egui::TextStyle::Body)*40. {
                    for (path,text) in rows { ui.monospace(path);ui.label(text);ui.add_space(ui.spacing().item_spacing.y); }
                } else {
                    let width=ui.available_width()*0.65;
                    egui::Grid::new(("raw-supplied-devices", &source.id, practice)).num_columns(2).max_col_width(width).spacing([16.,8.]).show(ui, |ui| {
                        for (path,text) in rows { ui.monospace(path);ui.label(text);ui.end_row(); }
                    });
                }
            });
        }
        ui.label(if definition.mount_task.is_some(){"Simulated terminal · inspect with blkid, findmnt and cat; apply saved policy with mount/umount."}else{"Simulated storage terminal · pvs, vgs and lvs report current state."});
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
        let invalid_packet = !mount_edit.received
            && ui.input(|i| {
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
            limit: 2048,
        };
        let edit = ui.add_enabled(
            active,
            controls::singleline(ui, &mut buffer)
                .hint_text(if definition.mount_task.is_some() {
                    "e.g. findmnt"
                } else {
                    "e.g. lvs"
                })
                .id(id)
                .desired_width(ui.available_width()),
        );
        let mut rejected = buffer.rejected;
        let ignored = packet
            && ((!buffer.received && !reference_edit.received && !mount_edit.received)
                || after_return);
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
        if rejected || ignored || reference_edit.rejected || mount_edit.rejected {
            self.lvm_presentation.veto_root = Some(root);
            self.lvm_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
            if let Some(state) = old_native {
                state.restore(ui.ctx(), id);
            }
            if !reference_edit.rejected && !mount_edit.rejected {
                self.notice = Some(if rejected {
                "That edit exceeds the command limit or contains a line break. The previous draft and selection are retained; nothing ran."
            } else {
                "That text was not applied. Click the command field, retry the edit, then choose the action again."
            }.into());
            }
        }
        let allowed = active
            && !rejected
            && !ignored
            && !reference_edit.rejected
            && !reference_edit.received
            && !mount_edit.rejected
            && !mount_edit.received;
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
        let initial = if practice {
            &definition.lesson.fixture
        } else {
            &definition.initial
        };
        let extra = if storage.pv_capacity.is_some() {
            storage.devices.len() + targets.iter().filter(|t| t.preserve_volume).count()
        } else {
            0
        };
        let capacity_rows = rows
            .as_ref()
            .is_some_and(|r| r.len() == expected_rows + extra);
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
                    storage_check_text(index, text.clone(), storage, initial, targets)
                } else {
                    text.clone()
                };
                ui.colored_label(
                    palette(ui).error,
                    if definition.mount_task.is_some() {
                        format!(
                            "Needs attention · {failures} of {} checks. Details below.",
                            rows.len()
                        )
                    } else {
                        format!(
                            "Needs attention · {failures} of {} checks. First issue: {first}",
                            rows.len()
                        )
                    },
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
                        if definition.mount_task.is_some() {
                            ui.monospace(mounts::history(&row.input));
                        } else if let Some(label) =
                            device_reference::action_label(definition, &row.input, practice, true)
                        {
                            ui.monospace(label);
                        } else if let Some(context) = definition.snapshot_context(practice) {
                            let label = model::snapshot_flow::decode(&row.input)
                                .map_or_else(|_| row.input.clone(), |op| op.description(context));
                            ui.monospace(if row.input.starts_with(model::snapshot_flow::PREFIX) {
                                label
                            } else {
                                format!("virtual $ {label}")
                            });
                        } else {
                            ui.monospace(format!("virtual $ {}", row.input));
                        }
                        ui.add(
                            egui::Label::new(RichText::new(&row.output).monospace())
                                .wrap()
                                .selectable(true),
                        );
                    }
                })
                .inner_rect
        });
        if practice {
            lesson_presentation::practice_step(ui, &source, 2);
        }
        if definition.mount_task.is_none() {
            ui.strong(
                if storage.provisioning.is_some()
                    || storage.pv_capacity.is_some()
                    || storage.snapshots.is_some()
                {
                    "Current allocations and remaining space"
                } else {
                    "Modeled LV and filesystem capacity"
                },
            );
            for (vg, g) in &storage.groups {
                if storage.provisioning.is_some()
                    || storage.pv_capacity.is_some()
                    || storage.snapshots.is_some()
                {
                    let free = storage.free_extents(vg).unwrap_or(0);
                    ui.label(format!(
                        "{vg}: {free} free extents · {} each",
                        capacity(g.extent_bytes)
                    ));
                }
                for (lv, v) in &g.volumes {
                    if v.filesystem.is_none()
                        && (storage.provisioning.is_some() || storage.snapshots.is_some())
                    {
                        let mut text = format!(
                            "{vg}/{lv}: {} · Raw storage · no filesystem or mount",
                            capacity(v.extents.len() as u64 * g.extent_bytes)
                        );
                        if let Some(availability) = &storage.activation {
                            let id = format!("{vg}/{lv}");
                            text.push_str(if availability.active[&id] {
                                " · Active"
                            } else {
                                " · Inactive"
                            });
                            if availability.open.contains(&id) {
                                text.push_str(" · Marked open in this exercise");
                            }
                        }
                        #[cfg(test)]
                        self.metrics.lvm.capacity_texts.push(text.clone());
                        ui.label(text);
                    }
                    if let Some(fs) = &v.filesystem {
                        let mut text = format!(
                            "{vg}/{lv}: LV {} · filesystem {}",
                            capacity(v.extents.len() as u64 * g.extent_bytes),
                            capacity(fs.blocks * dlms_learning_lab::lvm::BLOCK_BYTES)
                        );
                        if let Some(availability) = &storage.activation {
                            let id = format!("{vg}/{lv}");
                            text.push_str(if availability.active[&id] {
                                " · Active"
                            } else {
                                " · Inactive"
                            });
                            if availability.open.contains(&id) {
                                text.push_str(" · Marked open in this exercise");
                            }
                        }
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
                        if storage.activation.is_some(){lesson_presentation::prose(ui,"Simulated application reader · availability is required. This is not a Linux mount or a command reading a raw device.");}else if storage.pv_capacity.is_some(){lesson_presentation::prose(ui,"Simulated application reader over supplied data; this is not a Linux mount or a command reading a raw device. Mount/kernel availability is not modeled here.");}
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
                                                ui.strong(path);
                                                let result=if storage.activation.is_some() || storage.pv_capacity.is_some(){storage.read_file(&format!("{vg}/{lv}"),path)}else{Ok(file.contents.as_str())};
                                                if storage.activation.is_some() || storage.pv_capacity.is_some(){self.metrics.lvm.reader_records.push((format!("{vg}/{lv}:{path}"),result.is_ok()));}
                                                let response=match result {
                                                    Ok(contents)=>{
                                                        #[cfg(test)]
                                                        self.metrics.lvm.inspected_files.push((path.clone(),contents.to_owned()));
                                                        ui.add(egui::Label::new(RichText::new(contents).monospace()).wrap().selectable(true))
                                                    }
                                                    Err(e)=>ui.label(format!("Reader unavailable: {e}")),
                                                };
                                                if storage.activation.is_some() || storage.pv_capacity.is_some(){self.metrics.lvm.reader_rows.push((format!("{vg}/{lv}:{path}"),response.rect,ui.clip_rect()));}
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
        }
        if let Some(rows) = rows {
            if definition.mount_task.is_some() {
                mounts::feedback(ui, &rows, &mut self.metrics.lvm);
            } else {
                for (index, (ok, text)) in rows.into_iter().enumerate() {
                    let text = if capacity_rows {
                        storage_check_text(index, text, storage, initial, targets)
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
                                if let Some(task)=&definition.mount_task {for operation in &task.reference {mounts::worked(ui,operation,"Apply this approach to the supplied assignment, then verify the current mounted data.");}}else{
                                for (command_index, command) in source.solution.iter().enumerate() {
                                    #[cfg(test)]
                                    self.metrics.lvm.worked_commands.push(command.clone());
                                    if let Some(label) = device_reference::action_label(definition, command, false, false) {
                                        let r=ui.monospace(label);
                                        self.metrics.lvm.controls.push((format!("reference-worked-line-{command_index}"),r.rect,ui.clip_rect()));
                                    } else if let Some(context) = definition.snapshot_context(false) {
                                        ui.monospace(
                                            model::snapshot_flow::decode(command).map_or_else(
                                                |_| command.clone(),
                                                |op| op.description(context),
                                            ),
                                        );
                                    } else {
                                        ui.monospace(command);
                                    }
                                }
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

// Only the explicit PV extension prepends metadata/protection rows. Keep the
// legacy formatter and row ordering unchanged for all earlier source profiles.
fn storage_check_text(
    index: usize,
    fallback: String,
    storage: &dlms_learning_lab::lvm::State,
    initial: &dlms_learning_lab::lvm::State,
    targets: &[model::Target],
) -> String {
    if storage.pv_capacity.is_none() {
        return capacity_check_text(index, fallback, storage, targets);
    }
    let mut at = 0;
    let mut requested = std::collections::BTreeSet::new();
    for t in targets {
        for p in &t.pv_sizes {
            requested.insert(p.device.as_str());
            if index == at {
                return format!(
                    "{}: recognized PV {}; required {}. Device {}. LV and filesystem are separate layers.",
                    p.device,
                    capacity(storage.devices[&p.device].pv_bytes.unwrap()),
                    capacity(p.bytes),
                    capacity(storage.devices[&p.device].bytes)
                );
            }
            at += 1;
        }
        if t.preserve_volume {
            if index == at {
                return fallback;
            }
            at += 1;
        }
    }
    for (path, d) in &initial.devices {
        if !requested.contains(path.as_str()) {
            if index == at {
                return if let Some(before) = d.pv_bytes {
                    format!(
                        "Unrelated {path}: recognized PV {}; keep the supplied {}, signature, group and device size",
                        capacity(storage.devices[path].pv_bytes.unwrap()),
                        capacity(before)
                    )
                } else {
                    fallback
                };
            }
            at += 1;
        }
    }
    capacity_check_text(index - at, fallback, storage, targets)
}
