//! Native editor/investigation workspace for the bounded virtual manager.
use super::*;
use dlms_learning_lab::{
    unit_manager::{self as core, Action, Runtime},
    unit_manager_teaching::Learning,
    unit_manager_terminal::{self as terminal, Session},
    unit_manager_workspace::{self as model, Definition, Draft},
};
struct PanelControls<'a> {
    metrics: &'a mut Metrics,
    focus: &'a mut bool,
    notice: &'a mut Option<String>,
}
#[derive(Default)]
struct Effects {
    dirty: bool,
    invalidate: bool,
}
impl Effects {
    fn merge(&mut self, other: Self) {
        self.dirty |= other.dirty;
        self.invalidate |= other.invalidate;
    }
}
impl LabApp {
    pub(super) fn unit_manager_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.unit_manager_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_unit_manager_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .and_then(|a| a.workspace.as_mut())
            .and_then(|w| w.unit_manager_learning.as_mut())
        {
            l.view = view;
        }
        self.help_open = false;
        self.notice = None;
        self.focus_input = view != teaching::View::Introduction;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn unit_manager_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let view = self.unit_manager_view(i);
        let challenge = self.progress.attempts[&self.scenarios[i].id].challenge;
        let mut requested_view = None;
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        requested_view = Some(v);
                    }
                }
            });
        }
        if let Some(v) = requested_view {
            self.set_unit_manager_view(i, v);
            return;
        }
        let s = &self.scenarios[i];
        let lesson = s
            .unit_manager_teaching
            .as_ref()
            .expect("validated unit-manager teaching");
        ui.heading(&s.title);
        let blocked = self.reset_pending
            || self.settings_open
            || self.theme_preview
            || self.data_tools.modal_open();
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A separate example to investigate").strong());
                ui.label(&lesson.example_description);
                ui.label("Edit only the supplied virtual files. Save updates the text; type commands in the virtual terminal and press Enter or Run command to investigate and apply changes.");
                ui.label("Open Loaded definition and current inputs below the editor to inspect the loaded virtual program and the data actually consumed by its current run. This read-only view does not apply your edits.");
                if let Some(unit) = lesson
                    .definition
                    .initial
                    .inventory()
                    .units
                    .iter()
                    .find(|u| u.ends_with(".service"))
                {
                    ui.label(RichText::new(format!("systemctl status {unit}")).monospace());
                    ui.label("Read its loaded definition and current runtime state without changing either.");
                    ui.label(RichText::new(format!("journalctl -u {unit} -o cat")).monospace());
                    ui.label("Read retained simulated job messages. No timestamps or process IDs are invented.");
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                let mut next = None;
                ui.add_enabled_ui(!blocked, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(primary(ui, "Try this example"));
                        self.metrics.example_action = Some(r.rect);
                        if r.clicked() {
                            next = Some(teaching::View::Example);
                        }
                        let r = ui.add(button("Go straight to assignment"));
                        self.metrics.assignment_action = Some(r.rect);
                        if r.clicked() {
                            next = Some(teaching::View::Assignment);
                        }
                    });
                });
                if let Some(v) = next {
                    self.set_unit_manager_view(i, v);
                }
            }
            teaching::View::Example => {
                ui.label(
                    "Practice example · Ungraded; your assignment and its drafts stay separate.",
                );
                ui.label(&lesson.example_description);
                let l = self
                    .progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .unit_manager_learning
                    .as_mut()
                    .unwrap();
                let mut changed = false;
                ui.add_enabled_ui(!blocked,|ui|{
                    ui.horizontal_wrapped(|ui|{
                        let r=ui.add(button("Fill worked editor drafts"));self.metrics.insert=Some(r.rect);
                        if r.clicked(){self.notice=l.fill_worked(lesson).err();changed=true;}
                        let r=ui.add(button("Reset example only"));self.metrics.example_reset=Some(r.rect);
                        if r.clicked(){*l=Learning::new(lesson).expect("validated lesson");l.view=teaching::View::Example;self.notice=None;changed=true;self.focus_input=false;}
                    });
                    ui.label(RichText::new("Fill changes example text only. Save it and use the virtual terminal to investigate and apply it.").small().color(palette(ui).muted));
                    let effects=panel(ui,(&s.id,true),&mut l.state,&mut l.draft,l.terminal.get_or_insert_with(Session::default),&lesson.definition,&mut PanelControls { metrics: &mut self.metrics, focus: &mut self.focus_input, notice: &mut self.notice });
                    changed|=effects.dirty;if effects.invalidate{l.checked=false;}
                    egui::CollapsingHeader::new("Example action guide").id_salt(("manager-example-guide",&s.id)).show(ui,|ui|{
                        for a in &lesson.example_actions{
                            if matches!(&a.action, model::EditAction::Manager { action: Action::AdvancePrograms {} }) { continue; }
                            ui.label(RichText::new(&a.title).strong());ui.label(&a.explanation);
                            if let Some(command)=real_command(&a.action){
                                ui.horizontal_wrapped(|ui|{ui.label(RichText::new(&command).monospace());let r=ui.add(button("Insert command"));self.metrics.command_insert=Some(r.rect);
                                    if r.clicked(){l.terminal.get_or_insert_with(Session::default).command=command;changed=true;self.focus_input=true;}
                                });
                            }
                        }
                    });
                    let r=ui.add(primary(ui,"Check example · ungraded"));self.metrics.check=Some(r.rect);
                    if r.clicked(){l.checked=true;changed=true;}
                    let result=controls::feedback(ui,r.clicked(),|ui|{if l.checked{let checks=l.checks(lesson);if checks.iter().all(|c|c.0){ui.colored_label(palette(ui).success,"Example checks pass · No assignment credit");}else{for(_,message)in checks.iter().filter(|c|!c.0).take(2){ui.colored_label(palette(ui).error,message);}}}});self.metrics.feedback=Some(result.response.rect);
                });
                ui.label(format!("Try a change: {}", lesson.try_change));
                let r = ui.add_enabled(!blocked, button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                let assignment = r.clicked();
                self.dirty |= changed;
                if assignment {
                    self.set_unit_manager_view(i, teaching::View::Assignment);
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let engine::State::UnitManager { simulation } = &mut a.state else {
                    unreachable!()
                };
                let engine::Goal::UnitManager { definition } = &s.goal else {
                    unreachable!()
                };
                let w = a.workspace.as_mut().unwrap();
                let effects = ui
                    .add_enabled_ui(!blocked, |ui| {
                        panel(
                            ui,
                            (&s.id, false),
                            simulation,
                            w.unit_manager_draft.as_mut().unwrap(),
                            w.unit_manager_terminal.get_or_insert_with(Session::default),
                            definition,
                            &mut PanelControls {
                                metrics: &mut self.metrics,
                                focus: &mut self.focus_input,
                                notice: &mut self.notice,
                            },
                        )
                    })
                    .inner;
                self.dirty |= effects.dirty;
                if effects.invalidate {
                    w.checked = false;
                    self.checked = false;
                }
                let r = ui.add_enabled(!blocked, primary(ui, "Check work"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    w.checked = true;
                    self.checked = true;
                    self.dirty = true;
                }
                let checked = w.checked;
                let mut completed = false;
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if checked {
                        let a = &self.progress.attempts[&s.id];
                        let engine::State::UnitManager { simulation } = &a.state else {
                            unreachable!()
                        };
                        let draft = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .unit_manager_draft
                            .as_ref()
                            .unwrap();
                        let checks = model::checks(definition, simulation, draft);
                        completed = checks.iter().all(|c| c.0);
                        if completed {
                            ui.colored_label(
                                palette(ui).success,
                                "Objective complete · Current and fresh-session checks pass.",
                            );
                        } else {
                            for (_, message) in checks.iter().filter(|c| !c.0).take(2) {
                                ui.colored_label(palette(ui).error, message);
                            }
                        }
                        egui::CollapsingHeader::new("Objective results")
                            .id_salt(("manager-results", &s.id))
                            .show(ui, |ui| {
                                for (pass, message) in checks {
                                    ui.label(format!(
                                        "{} · {message}",
                                        if pass { "Pass" } else { "Needs work" }
                                    ));
                                }
                            });
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
                self.metrics.completion = completed;
                if completed {
                    self.next_unit_lab(ui, i);
                }
            }
        }
    }
    pub(super) fn unit_manager_help(&mut self, ui: &mut egui::Ui, i: usize) {
        self.metrics.help_visible = true;
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        if a.hints > 0 {
            ui.label(&s.hints[a.hints - 1]);
        }
        if ui
            .add_enabled(
                a.hints < s.hints.len(),
                button(if a.hints == 0 {
                    "Show a hint"
                } else {
                    "Another hint"
                }),
            )
            .clicked()
        {
            a.hints += 1;
            self.dirty = true;
        }
        egui::CollapsingHeader::new("Why these layers are separate")
            .id_salt(("manager-why", &s.id))
            .show(ui, |ui| {
                ui.label(&s.explanation);
            });
        egui::CollapsingHeader::new("Supported commands, boundaries & sources")
            .id_salt(("manager-reference", &s.id))
            .show(ui, |ui| {
                ui.label(terminal::HELP);
                ui.label(&s.simplifications);
                for source in &s.sources {
                    ui.label(source);
                }
            });
    }
}
fn real_command(action: &model::EditAction) -> Option<String> {
    let model::EditAction::Manager { action } = action else {
        return None;
    };
    Some(match action {
        Action::Start { unit } => format!("systemctl start {unit}"),
        Action::Stop { unit } => format!("systemctl stop {unit}"),
        Action::Enable { unit } => format!("systemctl enable {unit}"),
        Action::Disable { unit } => format!("systemctl disable {unit}"),
        Action::SetDefault { target } => format!("systemctl set-default {target}"),
        Action::DaemonReload {} => "systemctl daemon-reload".into(),
        _ => return None,
    })
}
fn panel(
    ui: &mut egui::Ui,
    scope: (&str, bool),
    state: &mut core::State,
    draft: &mut Draft,
    session: &mut Session,
    definition: &Definition,
    controls: &mut PanelControls<'_>,
) -> Effects {
    summary(ui, state, draft);
    let mut effects = Effects::default();
    if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 52. {
        ui.columns(2, |columns| {
            effects.merge(editor(
                &mut columns[0],
                scope,
                state,
                draft,
                controls.metrics,
                controls.notice,
            ));
            effects.merge(command_terminal(
                &mut columns[1],
                scope,
                state,
                session,
                definition,
                controls,
            ));
        });
    } else {
        effects.merge(editor(
            ui,
            scope,
            state,
            draft,
            controls.metrics,
            controls.notice,
        ));
        effects.merge(command_terminal(
            ui, scope, state, session, definition, controls,
        ));
    }
    if let Some(message) = controls.notice.as_ref() {
        ui.label(message);
    }
    let inspector = egui::CollapsingHeader::new("Loaded definition and current inputs")
        .id_salt(("manager-input-inspector", scope))
        .show(ui, |ui| {
            controls.metrics.manager_inspector_open = true;
            ui.label("Read-only virtual inspection. Saved text, loaded definitions and previously consumed runtime inputs are separate; opening this does not reload or run anything.");
            let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8.;
            egui::ScrollArea::both().id_salt(("manager-inspector-scroll", scope))
                .max_height(height).min_scrolled_height(height).auto_shrink([false, false]).show(ui, |ui| {
                    for (label, text) in inspection_rows(state, &draft.selected_unit) {
                        ui.label(RichText::new(label).strong());
                        ui.add(egui::Label::new(RichText::new(text).monospace()).extend().selectable(true));
                    }
                });
        });
    controls.metrics.manager_inspector = Some(inspector.header_response.rect);
    effects
}

fn summary(ui: &mut egui::Ui, state: &core::State, draft: &Draft) {
    let selected = &draft.selected_unit;
    let phase = match state.runtime().get(selected) {
        Some(Runtime::Running { .. }) => "Running",
        Some(Runtime::Exited { .. }) => "Active after successful exit",
        Some(Runtime::TargetActive {}) => "Active target",
        Some(Runtime::Dispatched { .. }) => "Initialization pending",
        Some(Runtime::Failed { .. }) => "Failed",
        _ => "Stopped",
    };
    let loaded = if !state.loaded_sources().contains_key(selected) {
        "Not loaded"
    } else if state.needs_daemon_reload(selected).unwrap_or(true) {
        "Reload needed"
    } else {
        "Current saved unit loaded"
    };
    let stale = matches!(
        state.runtime().get(selected),
        Some(Runtime::Running { .. } | Runtime::Exited { .. } | Runtime::Failed { .. })
    ) && state.execution_differs(selected).unwrap_or(false);
    let observed = state
        .result(selected)
        .ok()
        .flatten()
        .map(|result| {
            if let Some(port) = result.port {
                format!(" · virtual port {port}")
            } else if let Some(value) = result.value {
                let preview: String = value.chars().take(48).collect();
                format!(
                    " · value {preview}{}",
                    if preview.len() < value.len() {
                        "… (inspect inputs)"
                    } else {
                        ""
                    }
                )
            } else {
                String::new()
            }
        })
        .unwrap_or_default();
    let phase = format!(
        "{phase}{observed}{}",
        if stale {
            " · earlier execution inputs"
        } else {
            ""
        }
    );
    ui.label(format!(
        "Editor: {} · Manager: {loaded} · Selected runtime: {phase}",
        if draft.matches(state) {
            "Saved"
        } else {
            "Unsaved changes"
        }
    ));
    let policy = terminal::read_only(state, &format!("systemctl is-enabled {selected}"))
        .map(|o| o.stdout.trim().to_owned())
        .unwrap_or_else(|_| "Saved install data needs attention".into());
    ui.label(RichText::new(format!("Future-session target: {} · Selected install policy: {policy}. This is separate from running now.",state.default_target())).small().color(palette(ui).muted));
}
fn editor(
    ui: &mut egui::Ui,
    scope: (&str, bool),
    state: &mut core::State,
    draft: &mut Draft,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) -> Effects {
    let mut effects = Effects::default();
    let mut configuration = draft.editing_configuration;
    let old_mode = configuration;
    ui.horizontal_wrapped(|ui| {
        let unit = ui.selectable_value(&mut configuration, false, "Unit file");
        metrics.manager_kind.push((false, unit.rect));
        ui.add_enabled_ui(!draft.configuration.is_empty(), |ui| {
            let config = ui.selectable_value(&mut configuration, true, "Application configuration");
            metrics.manager_kind.push((true, config.rect));
        });
    });
    if configuration != old_mode {
        draft.editing_configuration = configuration;
        effects.dirty = true;
    }
    let selected = if configuration {
        &mut draft.selected_config
    } else {
        &mut draft.selected_unit
    };
    let choices = if configuration {
        draft.configuration.keys().collect::<Vec<_>>()
    } else {
        draft.documents.keys().collect::<Vec<_>>()
    };
    let width = ui.available_width();
    let before = selected.clone();
    controls::cell(ui, width, |ui| {
        let r = egui::ComboBox::from_id_salt(("manager-slot", scope, configuration))
            .width(width)
            .truncate()
            .selected_text(selected.as_str())
            .show_ui(ui, |ui| {
                for name in choices {
                    let option = ui.selectable_value(selected, name.clone(), name);
                    metrics.manager_items.push((name.clone(), option.rect));
                }
            });
        metrics.manager_slot = Some(r.response.rect);
        name_control(
            &r.response,
            if configuration {
                "Selected virtual application configuration"
            } else {
                "Selected virtual unit file"
            },
        );
    });
    effects.dirty |= *selected != before;
    let key = selected.clone();
    let limit = if configuration {
        core::MAX_CONFIG_BYTES
    } else {
        core::MAX_UNIT_BYTES
    };
    let text = if configuration {
        draft.configuration.get_mut(&key).unwrap()
    } else {
        draft.documents.get_mut(&key).unwrap()
    };
    let label = ui.label(
        RichText::new(if configuration {
            format!("{key} · virtual application file")
        } else {
            format!("{key} · virtual unit source")
        })
        .strong(),
    );
    let h =
        ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * f32::from(controls::PAD_Y);
    let width = (ui.available_width()
        - 2. * f32::from(controls::PAD_X)
        - ui.spacing().scroll.allocated_width())
    .max(0.);
    let id = egui::Id::new(("manager-editor", scope, configuration, &key));
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let field = egui::ScrollArea::both()
        .id_salt(("manager-editor-scroll", scope, configuration, &key))
        .min_scrolled_height(h)
        .max_height(h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut *text)
                    .id(id)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .desired_width(width)
                    .desired_rows(7)
                    .char_limit(limit)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y)),
            )
            .labelled_by(label.id)
        });
    name_control(
        &field.inner,
        format!(
            "{} {} editor · {}",
            if scope.1 {
                "Ungraded example"
            } else {
                "Assignment"
            },
            if configuration {
                "application configuration"
            } else {
                "unit file"
            },
            key
        ),
    );
    metrics.manager_file = Some(field.inner.rect);
    metrics.editor_viewport = Some(field.inner_rect);
    metrics.inputs.push(field.inner.rect);
    if field.inner.changed() {
        if text.len() > limit {
            workspace::bound(text, limit);
            *notice = Some(format!(
                "This virtual file keeps at most{limit} UTF-8 bytes; extra text was not retained."
            ));
        } else {
            *notice = None;
        }
        effects.dirty = true;
        effects.invalidate = true;
    }
    let save = ui.add(button("Save editor drafts"));
    metrics.editor_save = Some(save.rect);
    if save.clicked() {
        let differs = !draft.matches(state);
        *notice=Some(match draft.apply(state){Ok(())=>"Saved editor drafts. Unit-file edits need manager reload; changed application inputs need stop, then start. Saving does not run or enable anything.".into(),Err(e)=>e});
        effects.dirty = true;
        effects.invalidate |= differs;
    }
    effects
}
fn command_terminal(
    ui: &mut egui::Ui,
    scope: (&str, bool),
    state: &mut core::State,
    session: &mut Session,
    definition: &Definition,
    controls: &mut PanelControls<'_>,
) -> Effects {
    let metrics = &mut *controls.metrics;
    let focus = &mut *controls.focus;
    let notice = &mut *controls.notice;
    let mut effects = Effects::default();
    ui.label(RichText::new("Virtual terminal · supplied services only").strong());
    let h =
        ui.text_style_height(&egui::TextStyle::Monospace) * 9. + 2. * f32::from(controls::PAD_Y);
    let output = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_X)
        .corner_radius(6.)
        .show(ui, |ui| {
            // Capture the finite viewport before horizontal content (commands or
            // tabular stdout) can enlarge the scroll area's available width.
            let diagnostic_width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(1.);
            ui.set_min_width(diagnostic_width);
            egui::ScrollArea::both()
                .id_salt(("manager-output", scope))
                .min_scrolled_height(h)
                .max_height(h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if session.entries.is_empty() {
                        ui.label("Use systemctl status to inspect a supplied unit. No host service is accessed.");
                    }
                    for entry in &session.entries {
                        ui.add(egui::Label::new(RichText::new(format!("$ {}", visible_command(&entry.command))).monospace()).extend().selectable(true));
                        if !entry.stdout.is_empty() {
                            ui.add(egui::Label::new(RichText::new(&entry.stdout).monospace()).extend().selectable(true));
                        }
                        if !entry.stderr.is_empty() {
                            ui.allocate_ui_with_layout(egui::vec2(diagnostic_width, 0.), egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(diagnostic_width);
                                let text = ui.add(egui::Label::new(RichText::new(&entry.stderr).color(palette(ui).error)).wrap().selectable(true));
                                metrics.manager_diagnostics.push((text.rect, diagnostic_width));
                            });
                        }
                        if let Some(kind) = entry.diagnostic {
                            ui.label(RichText::new(diagnostic_name(kind)).small());
                        } else if entry.status.is_some_and(|s| s != 0) {
                            ui.label(RichText::new(format!("Exit status {} · use this result to investigate", entry.status.unwrap())).small());
                        }
                    }
                });
        });
    metrics.output = Some(output.response.rect);
    let id = egui::Id::new(("manager-command", scope));
    let had_focus = ui.memory(|m| m.has_focus(id));
    let run_key = ui.is_enabled()
        && had_focus
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter)
        });
    let button_width = controls::button_width(ui, "Run command");
    let gap = controls::GAP;
    let minimum = ui.text_style_height(&egui::TextStyle::Body) * 8.;
    let field_width = if ui.available_width() >= minimum + button_width + gap {
        ui.available_width() - button_width - gap
    } else {
        ui.available_width()
    };
    let mut run = false;
    ui.horizontal_wrapped(|ui| {
        let field = ui.add(
            controls::singleline(ui, &mut session.command)
                .id(id)
                .desired_width((field_width - 2. * f32::from(controls::PAD_X)).max(0.))
                .hint_text("systemctl … · Enter to run")
                .char_limit(terminal::MAX_COMMAND_BYTES),
        );
        name_control(
            &field,
            if scope.1 {
                "Ungraded example virtual service command"
            } else {
                "Assignment virtual service command"
            },
        );
        metrics.terminal = Some(field.rect);
        metrics.inputs.push(field.rect);
        metrics.focus = ui.memory(|m| m.focused());
        if field.changed() {
            workspace::bound(&mut session.command, terminal::MAX_COMMAND_BYTES);
            effects.dirty = true;
        }
        if *focus && ui.is_enabled() && !ui.input(|i| i.pointer.any_down()) {
            field.request_focus();
            *focus = false;
        }
        let r = ui.add(button("Run command"));
        metrics.manager_run = Some(r.rect);
        run = ui.is_enabled() && (r.clicked() || run_key);
        if run {
            field.request_focus();
        }
    });
    if run && !session.command.trim().is_empty() {
        match session.submit(definition, state) {
            Ok(changed) => {
                effects.invalidate |= changed;
                *notice = None;
            }
            Err(error) => *notice = Some(error),
        }
        effects.dirty = true;
    }
    effects
}
fn visible_command(command: &str) -> String {
    command
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

fn diagnostic_name(kind: terminal::DiagnosticKind) -> &'static str {
    match kind {
        terminal::DiagnosticKind::Syntax => "Syntax problem · correct the command and retry",
        terminal::DiagnosticKind::Unsupported => "Outside the supported simulator scope",
        terminal::DiagnosticKind::Context => {
            "Activity context problem · no host lookup is performed"
        }
        terminal::DiagnosticKind::Limit => "Simulator resource limit · narrow the request",
        terminal::DiagnosticKind::Model => "Virtual manager validation problem",
    }
}

fn program_config(program: &core::Program) -> &str {
    match program {
        core::Program::ValueTask { config, .. } | core::Program::PortDaemon { config, .. } => {
            config
        }
    }
}
/// Pure selected-unit observation; no load, program advancement or grading.
pub(super) fn inspection_rows(state: &core::State, unit: &str) -> Vec<(String, String)> {
    if state.validate().is_err() || !state.inventory().units.contains(unit) {
        return vec![(
            "Virtual inspection unavailable".into(),
            "Invalid activity context; work is retained".into(),
        )];
    }
    let mut rows = vec![("Saved unit source".into(), state.documents()[unit].clone())];
    if let Some(source) = state.loaded_sources().get(unit) {
        rows.push(("Loaded unit source".into(), source.clone()));
        if let Ok(definition) = core::parse_unit(state.inventory(), unit, source)
            && let core::UnitType::Service { exec, .. } = definition.kind
        {
            rows.push(("Loaded virtual program".into(), exec.clone()));
            let config = program_config(&state.inventory().programs[&exec]);
            rows.push((
                "Current saved application input".into(),
                format!("{config}\n{}", state.configuration()[config]),
            ));
        }
    } else {
        rows.push(("Loaded unit source".into(), "No definition loaded".into()));
    }
    match state.runtime().get(unit) {
        Some(
            Runtime::Running { snapshot }
            | Runtime::Exited { snapshot }
            | Runtime::Failed { snapshot },
        ) => {
            rows.push((
                "Unit source used by current runtime".into(),
                snapshot.unit_source.clone(),
            ));
            if let Ok(definition) = core::parse_unit(state.inventory(), unit, &snapshot.unit_source)
                && let core::UnitType::Service { exec, .. } = definition.kind
            {
                let config = program_config(&state.inventory().programs[&exec]);
                rows.push(("Runtime virtual program".into(), exec));
                rows.push((
                    "Application input consumed by runtime".into(),
                    format!("{config}\n{}", snapshot.configuration),
                ));
            }
            rows.push((
                "Prerequisite value consumed by runtime".into(),
                snapshot
                    .artifact_input
                    .as_ref()
                    .map_or("None", |v| v.as_str())
                    .into(),
            ));
            let outcome = match state.result(unit) {
                Ok(Some(result)) => if let Some(port) = result.port { format!("Successful virtual program · port {port}; no real listener") }
                    else { format!("Successful virtual program · value {}", result.value.unwrap_or_default()) },
                _ => "Failed virtual initialization using these captured inputs; read its retained journal for the recorded cause".into(),
            };
            rows.push(("Derived current runtime outcome".into(), outcome));
        }
        Some(Runtime::Dispatched { unit_source }) => {
            rows.push(("Dispatched virtual source".into(), unit_source.clone()));
            rows.push((
                "Derived current runtime outcome".into(),
                "Start job completed; virtual program has not initialized".into(),
            ));
        }
        Some(Runtime::TargetActive {}) => rows.push((
            "Derived current runtime outcome".into(),
            "Active virtual target · no program or application input".into(),
        )),
        _ => rows.push((
            "Derived current runtime outcome".into(),
            "Stopped · no current program execution snapshot".into(),
        )),
    }
    rows
}
