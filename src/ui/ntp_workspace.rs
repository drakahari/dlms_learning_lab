//! Native widgets for the bounded NTP session; never a clock or network service.
use super::*;
use dlms_learning_lab::{engine::State, ntp_workspace as model, script_teaching::View};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub run_id: Option<egui::Id>,
    pub notice: Option<egui::Rect>,
    pub example_choices: Vec<(usize, egui::Rect, egui::Rect)>,
    pub topology: Option<(egui::Rect, egui::Rect)>,
    pub evidence: Vec<(String, egui::Rect)>,
    pub inventory: Option<egui::Rect>,
    pub help_bodies: Vec<(String, egui::Rect, egui::Rect)>,
    pub inspection_nodes: Vec<(String, egui::Rect)>,
    pub inspection_facts: Vec<(String, egui::Rect)>,
    #[cfg(test)]
    pub check_texts: Vec<String>,
    #[cfg(test)]
    pub notice_text: Option<String>,
    #[cfg(test)]
    pub command_help_text: Option<String>,
    #[cfg(test)]
    pub simulation_label: Option<String>,
    #[cfg(test)]
    pub worked_explanations: Vec<String>,
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
                .is_none_or(|n| n > 512)
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
    pub(super) fn ntp_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let source = self.scenarios[i].clone();
        if let Err(e) = model::context(&self.progress.attempts[&source.id], &source) {
            ui.label(format!(
                "NTP work was retained without applying changes: {e}"
            ));
            return;
        }
        let engine::Goal::Ntp { definition } = &source.goal else {
            return;
        };
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        let inherited = self.ntp_presentation.held_veto;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.ntp_presentation.held_veto = false;
        }
        let blocked = inherited || self.ntp_presentation.veto_root == Some(root);
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
        let State::Ntp {
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
                        .ntp
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
                ui.monospace(format!("Select {} · {}", c.device, c.text));
                let explanation = visible_text(&c.explanation);
                #[cfg(test)]
                self.metrics
                    .ntp
                    .worked_explanations
                    .push(explanation.to_string());
                ui.label(explanation);
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
                        ("ntp-example", &source.id, attempt.visited),
                        ui.available_width(),
                    )
                    .selected_text(format!(
                        "{} · {}",
                        definition.lesson.worked[selected].device,
                        definition.lesson.worked[selected].text
                    ))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, c) in definition.lesson.worked.iter().enumerate() {
                            let choice = ui.selectable_value(
                                &mut selected,
                                n,
                                format!("{} · {}", c.device, c.text),
                            );
                            self.metrics
                                .ntp
                                .example_choices
                                .push((n, choice.rect, ui.clip_rect()));
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
                    .ntp
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
                    .ntp
                    .controls
                    .push(("reset-example".into(), r.rect, ui.clip_rect()));
                if r.clicked() {
                    match session.reset_example(definition) {
                        Ok(()) => {
                            navigated = true;
                            self.ntp_presentation.reset_practice();
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
            let explanation = visible_text(&definition.lesson.worked[selected].explanation);
            #[cfg(test)]
            self.metrics
                .ntp
                .worked_explanations
                .push(explanation.to_string());
            ui.label(explanation);
            ui.label(format!("Try a change: {}", definition.lesson.try_change));
            ui.monospace(format!(
                "{} · {}",
                definition.lesson.experiment.device, definition.lesson.experiment.text
            ));
            ui.monospace(format!(
                "{} · {}",
                definition.lesson.repair.device, definition.lesson.repair.text
            ));
        } else {
            let r = ui.label(&source.objective);
            self.metrics.ntp.goal = Some(r.rect);
        }
        let simulation_label = "Simulated NTP configuration · no real clock, daemon or packets.";
        #[cfg(test)]
        {
            self.metrics.ntp.simulation_label = Some(simulation_label.into());
        }
        ui.label(simulation_label);
        let world = if practice {
            &session.example
        } else {
            &session.current
        };
        let mut chosen = if practice {
            session.example_selected
        } else {
            session.selected
        };
        controls::form_row(ui, "Selected simulated device", |ui, width| {
            ui.add_enabled_ui(!packet && !blocked, |ui| {
                let picker = controls::dropdown(ui, ("ntp-device", &source.id, practice), width)
                    .selected_text(format!("{} ({chosen})", world.nodes[&chosen].label))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (ip, node) in &world.nodes {
                            if matches!(node.role, dlms_learning_lab::ntp::Role::Device { .. }) {
                                let r = ui.selectable_value(
                                    &mut chosen,
                                    *ip,
                                    format!("{} ({ip})", node.label),
                                );
                                self.metrics.ntp.choices.push((
                                    ip.to_string(),
                                    r.rect,
                                    ui.clip_rect(),
                                ));
                            }
                        }
                    });
                self.metrics.ntp.controls.push((
                    "device".into(),
                    picker.response.rect,
                    ui.clip_rect(),
                ));
            });
        });
        if chosen
            != if practice {
                session.example_selected
            } else {
                session.selected
            }
        {
            match session.select_device(definition, chosen, practice) {
                Ok(()) => {
                    navigated = true;
                    self.dirty = true;
                    self.notice = None;
                }
                Err(e) => self.notice = Some(e),
            }
        }
        if navigated {
            return;
        }
        ui.label("One IOS-like subset command for the selected device. Run explicitly; typing and selection do not configure it.");
        let mut command = if practice {
            session.example_command.clone()
        } else {
            session.command.clone()
        };
        let id = egui::Id::new((
            "ntp-command",
            &source.id,
            source.version,
            attempt.visited,
            practice,
            self.ntp_presentation.generation[usize::from(practice)],
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
                    t.len() > 512 || t.contains(['\0', '\n', '\r'])
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
                .id(id)
                .desired_width(ui.available_width()),
        );
        let mut rejected = buffer.rejected;
        let ignored = packet && (!buffer.received || after_return);
        self.metrics
            .ntp
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
            self.ntp_presentation.veto_root = Some(root);
            self.ntp_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
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
        let (run_clicked, check_clicked) = ui
            .horizontal_wrapped(|ui| {
                let run = ui.add_enabled(allowed, primary(ui, "Run simulated command"));
                self.metrics.terminal_run = Some(run.rect);
                self.metrics.ntp.run_id = Some(run.id);
                self.metrics
                    .ntp
                    .controls
                    .push(("run".into(), run.rect, ui.clip_rect()));
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
                self.metrics.check = Some(check.rect);
                self.metrics
                    .ntp
                    .controls
                    .push(("check".into(), check.rect, ui.clip_rect()));
                (run.clicked(), check.clicked())
            })
            .inner;
        let checked = if practice {
            session.example_checked
        } else {
            session.checked
        };
        if let Some(text) = &self.notice {
            let message = if text == dlms_learning_lab::ntp::HELP {
                "Unsupported command. Open Supported commands and limits for the available subset."
            } else {
                text.as_str()
            };
            let message = format!("Needs attention · {}", visible_text(message));
            #[cfg(test)]
            {
                self.metrics.ntp.notice_text = Some(message.clone());
            }
            self.metrics.ntp.notice = Some(ui.colored_label(palette(ui).error, message).rect);
        }
        if checked {
            let rows = if practice {
                definition.practice_checks(session)
            } else {
                definition.checks(session)
            };
            for (ok, text) in rows {
                let text = visible_text(&text);
                #[cfg(test)]
                self.metrics.ntp.check_texts.push(text.to_string());
                let r = ui.colored_label(
                    if ok {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    format!("{} · {text}", if ok { "Passed" } else { "Needs attention" }),
                );
                self.metrics.ntp.rows.push((ok, r.rect));
            }
        }
        let world = if practice {
            &session.example
        } else {
            &session.current
        };
        let selected = if practice {
            session.example_selected
        } else {
            session.selected
        };
        compact_dependency(
            ui,
            world,
            selected,
            definition,
            practice,
            &mut self.metrics.ntp,
        );
        let output = if practice {
            &session.example_transcript
        } else {
            &session.transcript
        };
        let h = ui.text_style_height(&egui::TextStyle::Monospace) * 6.;
        if output.is_empty() {
            self.metrics.output = Some(ui.label("No command output yet.").rect);
        } else {
            let area = egui::ScrollArea::vertical()
                .id_salt(("ntp-output", &source.id, practice))
                .max_height(h)
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
                });
            self.metrics.output = Some(area.inner_rect);
        }
        let inspection=controls::collapsing_header(ui,"Inspect all devices and directed facts")
            .id_salt(("ntp-world",&source.id,practice)).show(ui,|ui|{
                let first=ui.min_rect();
                for (ip,node) in &world.nodes{
                    let status=world.status(*ip).expect("admitted NTP world");
                    let row=ui.strong(format!("{} ({ip})",node.label));
                    self.metrics.ntp.inspection_nodes.push((ip.to_string(),row.rect));
                    ui.label(node_configuration(node));
                    ui.label(status_line(&status));
                    ui.label(format!("Complete current chain: {}",chain_text(&status)));
                    if let Some(failure)=status.failure{ui.label(format!("{} at {}",failure.reason.explanation(),failure.at));}
                }
                let facts=controls::collapsing_header(ui,"Every supplied directed request, reply and UDP123 fact")
                    .id_salt(("ntp-all-facts",&source.id,practice)).show(ui,|ui|{
                        ui.label("Supplied facts, not measured packets. Missing directed entries mean unreachable.");
                        for d in &world.deliveries{
                            let text=exchange_text(world,d.client,d.server);let row=ui.label(&text);
                            self.metrics.ntp.inspection_facts.push((text,row.rect));
                        }
                    });
                self.metrics.ntp.controls.push(("all-facts".into(),facts.header_response.rect,ui.clip_rect()));
                self.metrics.ntp.inventory=Some(first.union(ui.min_rect()));
            });
        self.metrics.ntp.controls.push((
            "inspection".into(),
            inspection.header_response.rect,
            ui.clip_rect(),
        ));
        let guidance_enabled =
            allowed && !packet && !blocked && !run_clicked && !enter && !check_clicked;
        let mut guidance_action = false;
        let mut hint_requested = false;
        let command_help = ui
            .add_enabled_ui(guidance_enabled, |ui| {
                controls::collapsing_header(ui, "Supported commands and limits")
                    .id_salt((
                        "ntp-command-help",
                        &source.id,
                        attempt.visited,
                        challenge,
                        practice,
                    ))
                    .show(ui, |ui| {
                        #[cfg(test)]
                        {
                            self.metrics.ntp.command_help_text =
                                Some(dlms_learning_lab::ntp::HELP.into());
                        }
                        let response = ui.label(dlms_learning_lab::ntp::HELP);
                        self.metrics.ntp.help_bodies.push((
                            dlms_learning_lab::ntp::HELP.into(),
                            response.rect,
                            ui.clip_rect(),
                        ));
                    })
            })
            .inner;
        guidance_action |= guidance_intent(ui, &command_help.header_response);
        self.metrics.ntp.controls.push((
            "command-help".into(),
            command_help.header_response.rect,
            ui.clip_rect(),
        ));
        if !practice {
            ui.push_id(
                (
                    "ntp-assignment-help",
                    &source.id,
                    source.version,
                    attempt.visited,
                    challenge,
                ),
                |ui| {
                    let help = ui
                        .add_enabled_ui(guidance_enabled, |ui| {
                            controls::collapsing_header(ui, "Optional help").show(ui, |ui| {
                                if attempt.hints < source.hints.len() {
                                    let hint =
                                        ui.add_enabled(guidance_enabled, button("Show a hint"));
                                    hint_requested = hint.clicked();
                                    guidance_action |= guidance_intent(ui, &hint);
                                    self.metrics.ntp.controls.push((
                                        "show-hint".into(),
                                        hint.rect,
                                        ui.clip_rect(),
                                    ));
                                } else {
                                    ui.label("All hints have been requested.");
                                }
                                for (i, hint) in source.hints.iter().take(attempt.hints).enumerate()
                                {
                                    #[cfg(test)]
                                    self.metrics.ntp.requested_hints.push(hint.clone());
                                    let text = format!("Hint {}: {hint}", i + 1);
                                    let response = ui.label(&text);
                                    self.metrics.ntp.help_bodies.push((
                                        text,
                                        response.rect,
                                        ui.clip_rect(),
                                    ));
                                }
                                let worked = controls::collapsing_header(
                                    ui,
                                    "Worked approach · reveals a complete solution",
                                )
                                .show(ui, |ui| {
                                    let text = visible_text(&source.explanation).into_owned();
                                    let response = ui.label(&text);
                                    self.metrics.ntp.help_bodies.push((
                                        text,
                                        response.rect,
                                        ui.clip_rect(),
                                    ));
                                    for command in &source.solution {
                                        if let Some(ip) = command.strip_prefix("SIM select ") {
                                            let instruction =
                                                format!("Choose device {ip} in Selected device.");
                                            #[cfg(test)]
                                            self.metrics
                                                .ntp
                                                .worked_commands
                                                .push(instruction.clone());
                                            ui.label(instruction);
                                        } else {
                                            #[cfg(test)]
                                            self.metrics.ntp.worked_commands.push(command.clone());
                                            ui.monospace(command);
                                        }
                                    }
                                });
                                guidance_action |= guidance_intent(ui, &worked.header_response);
                                self.metrics.ntp.controls.push((
                                    "worked-approach".into(),
                                    worked.header_response.rect,
                                    ui.clip_rect(),
                                ));
                                let refs = controls::collapsing_header(ui, "Technical references")
                                    .show(ui, |ui| {
                                        for url in &source.sources {
                                            #[cfg(test)]
                                            self.metrics.ntp.reference_urls.push(url.clone());
                                            let response = ui.hyperlink_to(url, url);
                                            self.metrics.ntp.help_bodies.push((
                                                url.clone(),
                                                response.rect,
                                                ui.clip_rect(),
                                            ));
                                        }
                                    });
                                guidance_action |= guidance_intent(ui, &refs.header_response);
                                self.metrics.ntp.controls.push((
                                    "references".into(),
                                    refs.header_response.rect,
                                    ui.clip_rect(),
                                ));
                            })
                        })
                        .inner;
                    guidance_action |= guidance_intent(ui, &help.header_response);
                    self.metrics.ntp.controls.push((
                        "optional-help".into(),
                        help.header_response.rect,
                        ui.clip_rect(),
                    ));
                },
            );
        }
        // Even a disabled guidance control has a real geometry/intent. Do not
        // let Enter delivered with that click dispatch the old focused command.
        if guidance_action && (packet || enter || run_clicked || check_clicked || blocked) {
            self.ntp_presentation.veto_root = Some(root);
            self.ntp_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
        }
        if hint_requested && guidance_enabled {
            let old = attempt.hints;
            attempt.hints += 1;
            if let Err(e) = model::context(attempt, &source) {
                attempt.hints = old;
                self.notice = Some(e);
            } else {
                self.dirty = true;
            }
            ui.ctx().request_repaint();
        }
        let dispatch_allowed =
            allowed && !guidance_action && self.ntp_presentation.veto_root != Some(root);
        let State::Ntp {
            simulation: session,
        } = &mut attempt.state
        else {
            return;
        };
        if dispatch_allowed && (run_clicked || enter) {
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
        if dispatch_allowed && check_clicked {
            self.notice = None;
            if practice {
                session.example_checked = true;
            } else {
                session.checked = true;
            }
            self.dirty = true;
        }
        if dispatch_allowed && (run_clicked || enter || check_clicked) {
            ui.ctx().request_repaint();
        }
        if !practice {
            self.metrics.completion = progress::current_solved(attempt, &source);
            if self.metrics.completion {
                ui.strong("Current assignment checks pass");
            }
        }
        if !practice
            && self.metrics.completion
            && self.selected == Some(i)
            && allowed
            && !packet
            && !run_clicked
            && !enter
            && !check_clicked
            && !guidance_action
            && self.ntp_presentation.veto_root != Some(root)
        {
            self.next_unit_lab(ui, i);
            if let Some(rect) = self.metrics.next_unit {
                self.metrics
                    .ntp
                    .controls
                    .push(("next".into(), rect, ui.clip_rect()));
            }
        }
    }
}

// Observe attempted pointer presses/releases even when a guidance control is
// disabled by this frame's Enter/text packet. This stores no focus or cursor.
fn guidance_intent(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || ui.input(|i| {
            (i.pointer.primary_pressed() || i.pointer.primary_released())
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| response.rect.contains(p) && ui.clip_rect().contains(p))
        })
}

// Presentation only. Keep canonical lessons, checks and source fingerprints
// intact while using a separator present in the native bundled text font.
fn visible_text(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('→') {
        return std::borrow::Cow::Borrowed(text);
    }
    std::borrow::Cow::Owned(
        text.split('→')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" -> "),
    )
}

fn node_configuration(node: &dlms_learning_lab::ntp::Node) -> String {
    match node.role {
        dlms_learning_lab::ntp::Role::Device { upstream } => format!(
            "Configured upstream: {}",
            upstream
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "none".into())
        ),
        dlms_learning_lab::ntp::Role::Reference { stratum, healthy } => format!(
            "Read-only reference; stratum {stratum}; {}",
            if healthy { "healthy" } else { "unhealthy" }
        ),
    }
}
fn status_line(status: &dlms_learning_lab::ntp::Status) -> String {
    format!(
        "{} · derived stratum {}",
        if status.synchronized {
            "Synchronized in this model"
        } else {
            "Unsynchronized"
        },
        status.stratum
    )
}
fn chain_text(status: &dlms_learning_lab::ntp::Status) -> String {
    status
        .chain
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" -> ")
}
fn exchange_text(
    world: &dlms_learning_lab::ntp::State,
    client: std::net::Ipv4Addr,
    server: std::net::Ipv4Addr,
) -> String {
    match world
        .deliveries
        .iter()
        .find(|d| d.client == client && d.server == server)
    {
        Some(d) => format!(
            "{client} -> {server}: request {}; reply {}; UDP123 {}",
            if d.request_delivered {
                "delivered"
            } else {
                "not delivered"
            },
            if d.reply_delivered {
                "delivered"
            } else {
                "not delivered"
            },
            if d.udp123_allowed {
                "allowed"
            } else {
                "blocked"
            }
        ),
        None => format!(
            "{client} -> {server}: no supplied directed delivery entry; exchange unreachable"
        ),
    }
}
fn compact_dependency(
    ui: &mut egui::Ui,
    world: &dlms_learning_lab::ntp::State,
    selected: std::net::Ipv4Addr,
    definition: &model::Definition,
    practice: bool,
    m: &mut Metrics,
) {
    use dlms_learning_lab::ntp::Role;
    let node = &world.nodes[&selected];
    let status = world.status(selected).expect("admitted NTP world");
    let Role::Device { upstream } = node.role else {
        return;
    };
    let left = [
        node.label.clone(),
        selected.to_string(),
        status_line(&status),
    ];
    let right = match upstream {
        Some(ip) => match world.nodes.get(&ip) {
            Some(n) => [n.label.clone(), ip.to_string(), node_configuration(n)],
            None => [
                "Unknown supplied address".into(),
                ip.to_string(),
                "Not in this topology".into(),
            ],
        },
        None => [
            "No upstream configured".into(),
            "No server address".into(),
            "Choose a supplied source".into(),
        ],
    };
    let color = if status.synchronized {
        palette(ui).success
    } else {
        palette(ui).error
    };
    let diagram=ui.scope(|ui|{
        if upstream.is_some(){
            controls::topology_pair(ui,[&left[0],&left[1],&left[2]],[&right[0],&right[1],&right[2]],color,
                "Selected client -> configured upstream · dependency, not physical cabling or measured packets");
        }else{
            controls::topology_frame(ui).show(ui,|ui|{
                ui.vertical_centered(|ui|{for line in &left{ui.label(line);}ui.label("No upstream configured; no dependency edge exists yet.");});
            });
        }
    });
    m.topology = Some((diagram.response.rect, ui.clip_rect()));
    let (initial, targets) = if practice {
        (&definition.lesson.fixture, &definition.lesson.targets)
    } else {
        (&definition.initial, &definition.targets)
    };
    if let Some(target) = targets.iter().find(|r| r.client == selected) {
        let sources = target
            .allowed_references
            .iter()
            .map(|ip| {
                let node = &world.nodes[ip];
                let fact = match node.role {
                    Role::Reference { healthy, .. } => {
                        if !node.online {
                            "offline reference"
                        } else if healthy {
                            "healthy reference"
                        } else {
                            "unhealthy reference"
                        }
                    }
                    Role::Device { .. } => {
                        if world.status(*ip).is_ok_and(|v| v.synchronized) {
                            "synchronized device"
                        } else {
                            "unsynchronized device"
                        }
                    }
                };
                format!("{} ({ip}; {fact})", node.label)
            })
            .collect::<Vec<_>>()
            .join("; ");
        let text = format!("Permitted upstreams for this device: {sources}");
        let r = ui.label(&text);
        m.evidence.push((text, r.rect));
    } else if let Role::Device { upstream } = initial.nodes[&selected].role {
        let text = format!(
            "Keep this device’s supplied upstream: {}",
            upstream
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into())
        );
        let r = ui.label(&text);
        m.evidence.push((text, r.rect));
    }
    let text = format!("Complete current dependency chain: {}", chain_text(&status));
    let r = ui.label(&text);
    m.evidence.push((text, r.rect));
    if let Some(f) = &status.failure {
        let text = format!("Needs attention: {} at {}", f.reason.explanation(), f.at);
        let r = ui.label(&text);
        m.evidence.push((text, r.rect));
    }
    if let Some(server) = upstream {
        let text = exchange_text(world, selected, server);
        let r = ui.label(&text);
        m.evidence.push((text, r.rect));
    }
    // If failure is upstream of the selected node, expose that directed hop too.
    // A healthy local exchange must not conceal a missing reply farther away.
    if let Some(f) = &status.failure
        && let Some(pair) = status.chain.windows(2).rev().find(|pair| pair[1] == f.at)
        && (pair[0] != selected || Some(pair[1]) != upstream)
    {
        let text = exchange_text(world, pair[0], pair[1]);
        let r = ui.label(&text);
        m.evidence.push((text, r.rect));
    }
}
