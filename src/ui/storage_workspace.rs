//! Native saved-file editing and simulated mount investigation.
use super::*;
use dlms_learning_lab::{
    storage::{SessionOutcome, State as Storage},
    storage_teaching::Learning,
    storage_workspace::{self as model, Draft, Requirement},
};
impl LabApp {
    pub(super) fn storage_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.storage_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_storage_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .and_then(|w| w.storage_learning.as_mut())
        {
            l.view = view;
        }
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn storage_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let scenario = self.scenarios[i].clone();
        let view = self.storage_view(i);
        if !self.progress.attempts[&scenario.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_storage_view(i, v);
                    }
                }
            });
        }
        ui.heading(&scenario.title);
        let lesson = scenario
            .storage_teaching
            .as_ref()
            .expect("validated storage teaching");
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                for example in &lesson.example_commands {
                    ui.label(RichText::new(&example.command).monospace());
                    ui.label(&example.explanation);
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_storage_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_storage_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Ungraded; your assignment remains saved separately.");
                ui.label(&lesson.example_description);
                let mut l = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .storage_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&l).unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        l.fill_example(lesson).expect("validated example");
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        l = Learning::new(lesson);
                        l.view = teaching::View::Example;
                    }
                });
                ui.push_id(("storage-practice", &scenario.id), |ui| {
                    if configuration(
                        ui,
                        &mut l.state,
                        &mut l.draft,
                        &lesson.outcomes,
                        &mut self.metrics,
                        &mut self.focus_input,
                    ) {
                        l.checked = false;
                    }
                    egui::CollapsingHeader::new("Example command guide").show(ui, |ui| {
                        for example in &lesson.example_commands {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(&example.command).monospace());
                                let r = ui.add(button("Insert command"));
                                self.metrics.command_insert = Some(r.rect);
                                if r.clicked() {
                                    l.draft.command = example.command.clone();
                                }
                            });
                            ui.label(&example.explanation);
                        }
                    });
                    let r = ui.add(primary(ui, "Check example"));
                    self.metrics.check = Some(r.rect);
                    if r.clicked() {
                        l.checked = true;
                    }
                    let result = controls::feedback(ui, r.clicked(), |ui| {
                        if l.checked {
                            let checks=l.checks(lesson);
                            if checks.iter().all(|c|c.0) {
                                ui.colored_label(palette(ui).success,"Example checks pass · No assignment credit");
                            } else if !l.draft.matches(&l.state) {
                                ui.colored_label(palette(ui).error,"The example editor has unsaved changes. Save the file before checking its results.");
                            } else {
                                for message in model::issues(&l.state,&lesson.outcomes).iter().take(3) {ui.colored_label(palette(ui).error,message);}
                            }
                        }
                    });
                    self.metrics.feedback = Some(result.response.rect);
                });
                ui.label(format!("Try a change: {}", lesson.try_change));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    l.view = teaching::View::Assignment;
                    self.set_storage_view(i, teaching::View::Assignment);
                }
                if serde_json::to_vec(&l).unwrap() != before {
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&scenario.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .storage_learning = Some(l);
            }
            teaching::View::Assignment => {
                ui.label(&scenario.objective);
                let engine::State::PersistentMounts { mut storage } =
                    self.progress.attempts[&scenario.id].state.clone()
                else {
                    unreachable!()
                };
                let engine::Goal::PersistentMounts { requirements, .. } = &scenario.goal else {
                    unreachable!()
                };
                let mut draft = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .storage_draft
                    .clone()
                    .unwrap_or_else(|| Draft::new(&storage));
                let before = (storage.clone(), draft.clone());
                let changed = ui
                    .push_id(("storage-assignment", &scenario.id), |ui| {
                        configuration(
                            ui,
                            &mut storage,
                            &mut draft,
                            requirements,
                            &mut self.metrics,
                            &mut self.focus_input,
                        )
                    })
                    .inner;
                if changed {
                    self.checked = false;
                }
                let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                if before != (storage.clone(), draft.clone()) {
                    self.dirty = true;
                }
                a.state = engine::State::PersistentMounts { storage };
                let w = a.workspace.as_mut().unwrap();
                w.storage_draft = Some(draft.clone());
                if changed {
                    w.checked = false;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Check configuration"),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    self.checked = true;
                    self.dirty = true;
                    self.progress
                        .attempts
                        .get_mut(&scenario.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = true;
                }
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if self.checked {
                        let a = &self.progress.attempts[&scenario.id];
                        let complete = progress::current_solved(a, &scenario);
                        self.metrics.completion = complete;
                        if complete {
                            ui.colored_label(palette(ui).success,"Objective complete · Saved settings, live mounts and fresh-session checks pass.");
                            if !a.challenge {
                                ui.label(&scenario.explanation);
                            }
                            self.next_unit_lab(ui, i);
                        } else if let engine::State::PersistentMounts { storage } = &a.state {
                            if !draft.matches(storage) {
                                ui.colored_label(palette(ui).error,"Unsaved editor changes · Save the configuration, then check the saved and live results.");
                            } else {
                                for message in model::issues(storage, requirements).iter().take(3) {
                                    ui.colored_label(palette(ui).error, message);
                                }
                            }
                        }
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
            }
        }
    }
}

fn configuration(
    ui: &mut egui::Ui,
    state: &mut Storage,
    draft: &mut Draft,
    requirements: &[Requirement],
    metrics: &mut Metrics,
    focus: &mut bool,
) -> bool {
    let before = (state.clone(), draft.text.clone());
    ui.label(RichText::new("Current live mounts").strong());
    for target in state.inventory().mountpoints() {
        let text = match state.live().get(target) {
            Some(m) => format!(
                "{target} · {} · {}",
                state.inventory().device_for(&m.volume).unwrap(),
                if m.read_only { "Read-only" } else { "Writable" }
            ),
            None => format!("{target} · Unmounted"),
        };
        ui.add(egui::Label::new(text).wrap());
    }
    if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 52. {
        ui.columns(2, |columns| {
            editor(&mut columns[0], state, draft, requirements, metrics, focus);
            terminal(&mut columns[1], state, draft, metrics);
        });
    } else {
        editor(ui, state, draft, requirements, metrics, focus);
        terminal(ui, state, draft, metrics);
    }
    before != (state.clone(), draft.text.clone())
}
fn editor(
    ui: &mut egui::Ui,
    state: &mut Storage,
    draft: &mut Draft,
    requirements: &[Requirement],
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let label = ui.label(RichText::new("Saved mount configuration · /etc/fstab").strong());
    ui.label(
        RichText::new("Source  mountpoint  type  options  dump  check")
            .small()
            .color(palette(ui).muted),
    );
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 6. + 2. * controls::PAD_Y as f32;
    let width = (ui.available_width()
        - 2. * controls::PAD_X as f32
        - ui.spacing().scroll.allocated_width())
    .max(0.);
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout(text.as_str().into(), font.clone(), color, f32::INFINITY))
    };
    let edit = egui::ScrollArea::both()
        .id_salt("storage-file-scroll")
        .min_scrolled_height(h)
        .max_height(h)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut draft.text)
                    .id_salt("storage-file")
                    .code_editor()
                    .layouter(&mut layouter)
                    .lock_focus(false)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width(width)
                    .desired_rows(6)
                    .char_limit(dlms_learning_lab::storage::MAX_CONFIG_BYTES),
            )
            .labelled_by(label.id)
        });
    metrics.editor_viewport = Some(edit.inner_rect);
    metrics.terminal = Some(edit.inner.rect);
    if *focus {
        edit.inner.request_focus();
        *focus = false;
    }
    metrics.focus = ui.memory(|m| m.focused());
    if edit.inner.changed() {
        draft.notice = None;
    }
    ui.horizontal_wrapped(|ui| {
        let r=ui.add(button("Save configuration"));metrics.editor_save=Some(r.rect);if r.clicked(){draft.save(state);}
        let r=ui.add(button("Validate configuration"));metrics.storage_validate=Some(r.rect);if r.clicked(){
            let errors=state.validate_configuration();
            draft.notice=Some(if !draft.matches(state){"Validation reads the saved file. The editor has unsaved changes; save them to validate the new text.".into()}else if errors.is_empty(){"Saved configuration is valid in this simulator. Validation does not change live mounts.".into()}else{errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")});
        }
        let r=ui.add(button("Test saved mounts"));metrics.storage_test=Some(r.rect);if r.clicked(){
            draft.append_output(&fresh_report(state,requirements));
            draft.notice=Some("Fresh-session results are in terminal output. The test uses independent copies; your current mounts and device names did not change.".into());
        }
    });
    if let Some(notice) = &draft.notice {
        ui.add(egui::Label::new(notice).wrap());
    }
}
fn terminal(ui: &mut egui::Ui, state: &mut Storage, draft: &mut Draft, metrics: &mut Metrics) {
    ui.label(RichText::new("Simulated administrator terminal").strong());
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 8. + 16.;
    let out = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_X)
        .corner_radius(6.)
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("storage-terminal-output")
                .max_height(h)
                .min_scrolled_height(h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(if draft.output.is_empty() {
                                "Ready. Type blkid or mount."
                            } else {
                                &draft.output
                            })
                            .monospace(),
                        )
                        .extend()
                        .selectable(true),
                    );
                });
        });
    metrics.output = Some(out.response.rect);
    let mut run = false;
    let button_width = controls::button_width(ui, "Run command");
    let minimum_field = ui.text_style_height(&egui::TextStyle::Body) * 8.;
    let field_width = if ui.available_width() >= minimum_field + button_width + controls::GAP {
        ui.available_width() - button_width - controls::GAP
    } else {
        ui.available_width()
    };
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(
            controls::singleline(ui, &mut draft.command)
                .id_salt("storage-command")
                .hint_text("Command · Enter to run")
                .desired_width((field_width - 2. * controls::PAD_X as f32).max(0.))
                .char_limit(1024),
        );
        name_control(&r, "Simulated storage terminal command");
        metrics.storage_command = Some(r.rect);
        metrics.inputs.push(r.rect);
        run = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let b = ui.add(button("Run command"));
        metrics.storage_run = Some(b.rect);
        run |= b.clicked();
        if run {
            r.request_focus();
        }
    });
    if run && !draft.command.trim().is_empty() {
        draft.run_command(state);
    }
}
pub(super) fn fresh_report(state: &Storage, requirements: &[Requirement]) -> String {
    let mut out =
        String::from("Simulator fresh-session test · independent copies, not a system boot\n");
    for issue in state.validate_configuration() {
        out.push_str(&format!("Saved configuration needs attention: {issue}\n"));
    }
    for (i, requirement) in requirements
        .iter()
        .filter(|r| matches!(r, Requirement::FreshSession { .. }))
        .enumerate()
    {
        let Requirement::FreshSession { devices, .. } = requirement else {
            unreachable!()
        };
        out.push_str(&format!("Test copy {}\n", i + 1));
        match state.fresh_session(devices.clone()) {
            Ok(fresh) => {
                for issue in fresh.state.validate_configuration() {
                    out.push_str(&format!("Test-copy saved configuration: {issue}\n"));
                }
                if let Ok(entries) = dlms_learning_lab::storage::parse(fresh.state.configuration())
                {
                    for entry in entries {
                        if !entry.automatic {
                            match fresh.state.inventory().resolve(&entry.source) {
                                Ok(id)=>out.push_str(&format!("Saved manual {}: resolves to volume {id} · UUID={} · not mounted automatically\n",entry.target,fresh.state.inventory().volumes()[id].uuid)),
                                Err(error)=>out.push_str(&format!("Saved manual {}: {error}\n",entry.target)),
                            }
                        }
                    }
                }
                for item in &fresh.outcomes {
                    match item {
                        SessionOutcome::Manual { target } => out.push_str(&format!(
                            "{target}: skipped (noauto; this alone does not validate the entry)\n"
                        )),
                        SessionOutcome::Failed { target, error } => {
                            out.push_str(&format!("{target}: {error}\n"))
                        }
                        SessionOutcome::Mounted { .. } => {}
                    }
                }
                for (target, mount) in fresh.state.live() {
                    out.push_str(&format!(
                        "{target}: volume {} · UUID={} · {} · {}\n",
                        mount.volume,
                        fresh.state.inventory().volumes()[&mount.volume].uuid,
                        fresh.state.inventory().device_for(&mount.volume).unwrap(),
                        if mount.read_only {
                            "Read-only"
                        } else {
                            "Writable"
                        }
                    ));
                }
            }
            Err(e) => out.push_str(&format!("Cannot test this saved file: {e}\n")),
        }
    }
    out
}
