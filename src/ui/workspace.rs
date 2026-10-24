use super::*;
use engine::State;
impl LabApp {
    pub(super) fn workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let mut challenge = self.progress.attempts[&s.id].challenge;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(button("Back to library"));
            self.metrics.back = Some(r.rect);
            if r.clicked() {
                self.script_job = None;
                self.selected = None;
            }
            ui.label(
                RichText::new(format!("{} / {}", s.track, s.topic))
                    .small()
                    .color(palette(ui).accent),
            );
            let before = challenge;
            ui.selectable_value(&mut challenge, false, "Learn");
            ui.selectable_value(&mut challenge, true, "Challenge");
            if before != challenge {
                self.progress.attempts.get_mut(&s.id).unwrap().challenge = challenge;
                self.help_open = false;
                self.script_job = None;
                if let Some(view) = &mut self.ipv4_view {
                    view.hide_guidance();
                }
                self.dirty = true;
            }
            let r = ui.add(button("Reset"));
            self.metrics.reset = Some(r.rect);
            if r.clicked() {
                self.reset_pending = true;
                self.lesson_scroll_reset = true;
                self.focus_input = false;
                if let Some(view) = &mut self.ipv4_view {
                    view.focus = false;
                }
                if let Some(view) = &mut self.address_example {
                    view.focus = false;
                }
            }
        });
        if self.reset_pending {
            ui.label("Start this lab again? This clears its answers, completion, simulation changes, drafts, practice and command history. Other labs and favorites stay saved.");
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Confirm reset"));
                self.metrics.confirm_reset = Some(r.rect);
                if r.clicked() {
                    self.reset_attempt(i);
                }
                if ui.add(button("Cancel")).clicked() {
                    self.reset_pending = false;
                }
            });
        }
        ui.add_space(8.);
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if matches!(s.initial, State::Ospf { .. }) {
                self.ospf_workspace(ui, i);
            } else if matches!(s.initial, State::Accounts { .. }) {
                self.accounts_workspace(ui, i);
            } else if matches!(s.initial, State::Ipv4Acl { .. }) {
                self.acl_workspace(ui, i);
            } else if matches!(s.initial, State::PersistentMounts { .. }) {
                self.storage_workspace(ui, i);
            } else if matches!(s.initial, State::Discovery { .. }) {
                self.discovery_learning_workspace(ui, i);
            } else if matches!(s.initial, State::AuthoritativeDns { .. }) {
                self.dns_workspace(ui, i);
            } else if s.service_teaching.is_some() {
                self.service_learning_workspace(ui, i);
            } else if s.address_teaching.is_some() {
                self.address_learning_workspace(ui, i);
            } else if matches!(self.progress.attempts[&s.id].state, State::Subnet { .. }) {
                self.ipv4_assignment(ui, i);
            } else {
                ui.label(RichText::new(&s.title).strong().size(23.));
                if !matches!(
                    s.initial,
                    State::Script { .. }
                        | State::Ipv6 { .. }
                        | State::Ipv4Routing { .. }
                        | State::Switching { .. }
                ) && s.terminal_teaching.is_none()
                    && s.process_teaching.is_none()
                {
                    ui.label(&s.objective);
                }
                if !challenge
                    && s.terminal_teaching.is_none()
                    && s.process_teaching.is_none()
                    && !matches!(
                        s.initial,
                        State::Ipv6 { .. } | State::Ipv4Routing { .. } | State::Switching { .. }
                    )
                {
                    ui.label(RichText::new(starting_point(&s.id)).color(palette(ui).muted));
                }
                if matches!(s.initial, State::Switching { .. }) {
                    self.switching_learning_workspace(ui, i);
                } else if matches!(s.initial, State::Ipv4Routing { .. }) {
                    self.routing_v4_learning_workspace(ui, i);
                } else if matches!(s.initial, State::Ipv6 { .. }) {
                    self.ipv6_learning_workspace(ui, i);
                } else if matches!(s.initial, State::Script { .. }) {
                    self.script_learning_workspace(ui, i);
                } else if s.process_teaching.is_some() {
                    self.process_learning_workspace(ui, i);
                } else if s.terminal_teaching.is_some() {
                    self.terminal_learning_workspace(ui, i);
                } else {
                    match &self.progress.attempts[&s.id].state {
                        State::Vlsm { .. } => self.allocations(ui, i),
                        State::Service { .. } | State::Dns { .. } | State::Mount { .. } => {
                            self.configuration(ui, i);
                            self.terminal(ui, i);
                        }
                        State::Network { .. } => {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(engine::describe(
                                        &self.progress.attempts[&s.id].state,
                                    ))
                                    .monospace(),
                                )
                                .wrap(),
                            );
                            self.terminal(ui, i);
                        }
                        _ => self.terminal(ui, i),
                    }
                    self.check_terminal_work(ui, i);
                }
            }
            let show_help = if matches!(s.initial, State::Ospf { .. }) {
                self.ospf_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Accounts { .. }) {
                self.accounts_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Ipv4Acl { .. }) {
                self.acl_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::PersistentMounts { .. }) {
                self.storage_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Discovery { .. }) {
                self.discovery_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::AuthoritativeDns { .. }) {
                self.dns_view(i) == teaching::View::Assignment
            } else if s.service_teaching.is_some() {
                self.service_view(i) == teaching::View::Assignment
            } else if s.address_teaching.is_some() {
                self.address_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Switching { .. }) {
                self.switching_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Ipv4Routing { .. }) {
                self.routing_v4_view(i) == teaching::View::Assignment
            } else if matches!(s.initial, State::Ipv6 { .. }) {
                self.ipv6_view(i) == teaching::View::Assignment
            } else if s.process_teaching.is_some() {
                self.process_view(i) == teaching::View::Assignment
            } else if s.terminal_teaching.is_some() {
                self.terminal_view(i) == teaching::View::Assignment
            } else {
                challenge
                    || !matches!(s.initial, State::Script { .. })
                    || self.script_view(i) == teaching::View::Assignment
            };
            if show_help {
                ui.add_space(2.);
                let r = ui.link(if self.help_open {
                    "Hide help"
                } else if matches!(s.initial, State::Subnet { .. }) {
                    "References & more practice"
                } else if challenge {
                    "Request help & references"
                } else {
                    "Hints, walkthrough & references"
                });
                self.metrics.help = Some(r.rect);
                if r.clicked() {
                    self.help_open = !self.help_open;
                }
                if self.help_open {
                    if matches!(s.initial, State::Ospf { .. }) {
                        ui.push_id(("ospf-help", &s.id, challenge), |ui| self.ospf_help(ui, i));
                    } else if matches!(s.initial, State::Ipv4Acl { .. }) {
                        ui.push_id(("acl-help", &s.id, challenge), |ui| self.acl_help(ui, i));
                    } else if matches!(s.initial, State::Accounts { .. }) {
                        ui.push_id(("accounts-help", &s.id, challenge), |ui| self.help(ui, i));
                    } else {
                        self.help(ui, i);
                    }
                }
            }
        });
        ui.label(
            RichText::new(
                "Simulation only · Leaving keeps your work and editor drafts saved locally.",
            )
            .small()
            .color(palette(ui).muted),
        );
    }
    pub(super) fn check_terminal_work(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let r = ui.add_sized(
            [220_f32.min(ui.available_width()), controls::height(ui)],
            primary(ui, "Check work"),
        );
        self.metrics.check = Some(r.rect);
        if r.clicked() {
            self.checked = true;
            self.progress
                .attempts
                .get_mut(&s.id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .checked = true;
            self.dirty = true;
        }
        let result = controls::feedback(ui, r.clicked(), |ui| self.report(ui, i));
        self.metrics.feedback = Some(result.response.rect);
    }
    pub(super) fn ipv4_assignment(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let practice = a.ipv4.as_ref().is_some_and(|l| l.use_variant);
        if practice {
            ui.label(
                RichText::new("Numerical practice · Original assignment stays saved")
                    .small()
                    .color(palette(ui).accent),
            );
        }
        let view = self.ipv4_view.as_mut().unwrap();
        let before = view.exercise.clone();
        let checked_before = view.checked_answers.clone();
        let hints = view.hints;
        ui.push_id(("ipv4-assignment", &s.id, practice), |ui| {
            view.render(ui, a.challenge, &mut self.metrics)
        });
        let w = a.workspace.as_mut().unwrap();
        view.save_ui(w, practice);
        if checked_before != view.checked_answers && view.checked_answers.is_some() {
            if practice {
                w.legacy_practice_reviewed = true;
            } else {
                w.legacy_host_reviewed = true;
            }
        }
        a.hints = view.hints.min(s.hints.len());
        if before != view.exercise || checked_before != view.checked_answers || hints != view.hints
        {
            self.dirty = true;
        }
        if practice {
            a.ipv4.as_mut().unwrap().variant = Some(view.exercise.clone());
        } else {
            ipv4::write_exercise(&mut a.state, &view.exercise);
        }
        if if practice {
            w.migrated_practice_host
        } else {
            w.migrated_host
        } {
            ui.label(RichText::new("Your earlier subnet analysis stays complete. This lab now also asks for a second workstation: enter and check its address. If an older app supplied a value, review it before checking.").small().color(palette(ui).muted));
        }
        self.metrics.completion = view.verified();
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
    pub(super) fn allocations(&mut self, ui: &mut egui::Ui, i: usize) {
        let state = &mut self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .state;
        self.dirty |= allocation_fields(
            ui,
            state,
            self.checked,
            &mut self.metrics,
            &mut self.focus_input,
        );
    }
    pub(super) fn configuration(&mut self, ui: &mut egui::Ui, i: usize) {
        let a = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap();
        let w = a.workspace.as_mut().unwrap();
        let mut save = false;
        let mut unsaved = false;
        match &a.state {
            State::Service { port, .. } => {
                ui.label(
                    RichText::new(engine::describe(&a.state))
                        .small()
                        .color(palette(ui).muted),
                );
                ui.label(
                    RichText::new("/etc/atlas.conf · simulated file")
                        .small()
                        .color(palette(ui).accent),
                );
                let draft = w.service_port.get_or_insert_with(|| port.clone());
                if editor_row(
                    ui,
                    "Port",
                    "Assignment service configuration port",
                    draft,
                    "service-port",
                    &mut self.metrics,
                    false,
                ) {
                    self.notice = None;
                    self.dirty = true;
                }
                unsaved = draft != port;
            }
            State::Dns {
                records,
                serial,
                loaded_serial,
                ..
            } => {
                ui.label(
                    RichText::new(format!(
                        "/etc/named/lab.zone · loaded serial {loaded_serial}"
                    ))
                    .small()
                    .color(palette(ui).accent),
                );
                let (name, address) = records.iter().next().unwrap();
                ui.label(format!("A record: {name}"));
                let draft = w.dns_address.get_or_insert_with(|| address.clone());
                if editor_row(
                    ui,
                    "IPv4 address",
                    "DNS A record IPv4 address",
                    draft,
                    "dns-address",
                    &mut self.metrics,
                    false,
                ) {
                    self.notice = None;
                    self.dirty = true;
                }
                unsaved |= draft != address;
                let draft = w.dns_serial.get_or_insert_with(|| serial.to_string());
                if editor_row(
                    ui,
                    "Zone serial",
                    "DNS zone serial",
                    draft,
                    "dns-serial",
                    &mut self.metrics,
                    false,
                ) {
                    self.notice = None;
                    self.dirty = true;
                }
                unsaved |= *draft != serial.to_string();
            }
            State::Mount {
                configured_device, ..
            } => {
                ui.label(
                    RichText::new("/etc/fstab · /data device (simulated file)")
                        .small()
                        .color(palette(ui).accent),
                );
                let draft = w
                    .mount_device
                    .get_or_insert_with(|| configured_device.clone());
                if editor_row(
                    ui,
                    "Device",
                    "Filesystem persistent device",
                    draft,
                    "mount-device",
                    &mut self.metrics,
                    false,
                ) {
                    self.notice = None;
                    self.dirty = true;
                }
                unsaved = draft != configured_device;
            }
            _ => {}
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(button("Save configuration"));
            self.metrics.editor_save = Some(r.rect);
            save = r.clicked();
            if unsaved {
                ui.label(
                    RichText::new("Unsaved draft · retained when leaving")
                        .small()
                        .color(palette(ui).accent),
                );
            }
        });
        if save {
            let result = match &a.state {
                State::Service { .. } => engine::edit_service_port(
                    &mut a.state,
                    w.service_port.as_deref().unwrap_or("").trim(),
                ),
                State::Dns { .. } => model::edit_dns(
                    &mut a.state,
                    w.dns_address.as_deref().unwrap_or(""),
                    w.dns_serial.as_deref().unwrap_or(""),
                ),
                State::Mount { .. } => {
                    model::edit_mount(&mut a.state, w.mount_device.as_deref().unwrap_or(""))
                }
                _ => Ok(()),
            };
            self.notice = Some(match result {
                Ok(()) => {
                    match &a.state {
                        State::Service { port, .. } => w.service_port = Some(port.clone()),
                        State::Dns {
                            records, serial, ..
                        } => {
                            w.dns_address = records.values().next().cloned();
                            w.dns_serial = Some(serial.to_string());
                        }
                        State::Mount {
                            configured_device, ..
                        } => w.mount_device = Some(configured_device.clone()),
                        _ => {}
                    }
                    self.dirty = true;
                    "Saved to the simulated configuration. Apply and verify it using the terminal."
                        .into()
                }
                Err(e) => e,
            });
        }
        if let Some(notice) = &self.notice {
            ui.label(RichText::new(notice).small().color(palette(ui).accent));
        }
    }
    pub(super) fn terminal(&mut self, ui: &mut egui::Ui, i: usize) {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        let accounts = matches!(a.state, State::Accounts { .. });
        let command_limit = if accounts {
            dlms_learning_lab::accounts_teaching::MAX_COMMAND_BYTES
        } else {
            4096
        };
        let height =
            (ui.ctx().input(|input| input.content_rect().bottom()) - ui.cursor().top() - 190.)
                .clamp(140., 260.);
        let output = egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(10.)
            .corner_radius(6.)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(match &a.state {
                        State::Filesystem { tree } => {
                            format!("Simulated terminal · {} · {}", tree.user, tree.cwd)
                        }
                        State::Processes { table } => format!(
                            "Simulated terminal · User: {} · virtual processes",
                            table.user.name
                        ),
                        State::Accounts { .. } => {
                            "Simulated terminal · Virtual administrator".into()
                        }
                        _ => "Simulated terminal · administrator".into(),
                    })
                    .small()
                    .color(palette(ui).muted),
                );
                let scroll = egui::ScrollArea::both()
                    .id_salt(("terminal", i))
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if a.transcript.is_empty() {
                            ui.label(
                                RichText::new("Ready for your command.")
                                    .monospace()
                                    .color(palette(ui).muted),
                            );
                        }
                        for line in &a.transcript {
                            let label =
                                egui::Label::new(RichText::new(line).monospace()).selectable(true);
                            // Account diagnostics are prose as well as tabular output.
                            // Soft wrapping keeps the whole explanation readable.
                            let row = ui.add(if accounts {
                                label.wrap()
                            } else {
                                label.extend()
                            });
                            if accounts {
                                self.metrics.accounts_latest_output = Some(row.rect);
                            }
                        }
                    });
                if accounts {
                    self.metrics.accounts_output_scroll =
                        Some((scroll.state.offset.y, scroll.inner_rect));
                }
            });
        self.metrics.output = Some(output.response.rect);
        let prompt = match &a.state {
            State::Processes { table } => table.prompt(),
            State::Filesystem { .. } => "alex $",
            State::Accounts { .. } => "Virtual administrator #",
            _ => "lab #",
        };
        let mut run = false;
        if accounts {
            ui.label(RichText::new(prompt).monospace().color(palette(ui).accent));
        }
        ui.horizontal(|ui| {
            if !accounts {
                ui.label(RichText::new(prompt).monospace().color(palette(ui).accent));
            }
            let width = (ui.available_width() - 90.).max(40.);
            let r = ui.add_sized(
                [width, controls::height(ui)],
                controls::singleline(ui, &mut self.input)
                    .id_salt(("assignment-command", &self.scenarios[i].id))
                    .hint_text("Command · Enter to run")
                    .char_limit(command_limit),
            );
            name_control(
                &r,
                format!("{} — assignment terminal command", self.scenarios[i].title),
            );
            self.metrics.terminal = Some(r.rect);
            self.metrics.focus = Some(r.id);
            if self.focus_input
                && (!accounts
                    || !ui
                        .input(|input| input.pointer.any_pressed() || input.pointer.any_released()))
            {
                r.request_focus();
                if self.scroll_command {
                    r.scroll_to_me(Some(egui::Align::Center));
                    self.scroll_command = false;
                }
                self.focus_input = false;
            } else if accounts && self.focus_input {
                ui.ctx().request_repaint();
            }
            if r.changed() {
                bound(&mut self.input, command_limit);
                self.progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .command = self.input.clone();
                self.dirty = true;
            }
            run = r.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let r = ui.add_sized([78., controls::height(ui)], button("Run"));
            if accounts {
                self.metrics.accounts_run = Some(r.rect);
            }
            run |= r.clicked();
        });
        if run {
            self.submit(i);
        }
    }
    pub(super) fn submit(&mut self, i: usize) {
        if self.input.trim().is_empty() {
            return;
        }
        let cmd = std::mem::take(&mut self.input);
        let a = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap();
        let before = a.state.clone();
        let output = engine::execute_for_goal(&mut a.state, &self.scenarios[i].goal, &cmd)
            .unwrap_or_else(|e| e);
        let prompt = match &a.state {
            State::Processes { table } => table.prompt(),
            State::Filesystem { .. } => "alex $",
            State::Accounts { .. } => "Virtual administrator #",
            _ => "lab #",
        };
        a.transcript.push(if output.is_empty() {
            format!("{prompt} {cmd}")
        } else {
            format!("{prompt} {cmd}\n{output}")
        });
        if a.transcript.len() > 100 {
            a.transcript.drain(..a.transcript.len() - 100);
        }
        let w = a.workspace.as_mut().unwrap();
        w.command.clear();
        refresh_drafts(w, &before, &a.state);
        self.notice = None;
        self.dirty = true;
        self.focus_input = true;
    }
    fn report(&mut self, ui: &mut egui::Ui, i: usize) {
        if !self.checked {
            return;
        }
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        if model::pending_service_draft(a) {
            ui.label("Unsaved configuration · Save your draft, then verify the service again.");
            return;
        }
        let now = progress::current_solved(a, s);
        self.metrics.completion = now;
        if now {
            ui.label(
                RichText::new("Objective complete · All current checks pass.")
                    .strong()
                    .color(palette(ui).success),
            );
            self.next_unit_lab(ui, i);
        } else if matches!(a.state, State::Vlsm { .. }) && ui.available_width() < 640. {
            let State::Vlsm { allocations, .. } = &a.state else {
                unreachable!()
            };
            // Native row feedback is primary; the summary also works in stacked layouts.
            for (index, allocation) in allocations.iter().enumerate() {
                if let Some(issue) = model::vlsm_issue(&a.state, index) {
                    ui.colored_label(palette(ui).error, format!("{}: {issue}", allocation.name));
                }
            }
        } else if !matches!(a.state, State::Vlsm { .. }) {
            for issue in model::issues(&a.state, &s.goal) {
                ui.add(egui::Label::new(RichText::new(issue).color(palette(ui).error)).wrap());
            }
        }
    }
    fn help(&mut self, ui: &mut egui::Ui, i: usize) {
        self.metrics.help_visible = true;
        if matches!(self.scenarios[i].initial, State::Script { .. }) {
            egui::CollapsingHeader::new("Supported shell subset & limits").show(ui, |ui| {
                ui.label(include_str!("../../docs/SHELL_SUBSET.md"));
            });
        }
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        if !matches!(a.state, State::Subnet { .. }) {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        a.hints < s.hints.len(),
                        button(if a.hints == 0 {
                            "Show a hint"
                        } else {
                            "Next hint"
                        }),
                    )
                    .clicked()
                {
                    a.hints += 1;
                    self.dirty = true;
                }
                if a.hints > 0 {
                    ui.label(&s.hints[a.hints - 1]);
                }
            });
        }
        if !a.challenge && !s.guide.is_empty() {
            let response = egui::CollapsingHeader::new("Optional walkthrough")
                .id_salt(("walkthrough", i))
                .show(ui, |ui| {
                    self.metrics.guide = true;
                    let g = &s.guide[a.step];
                    ui.label(RichText::new(&g.title).strong());
                    ui.label(&g.why);
                    ui.label(&g.action);
                    for c in &g.commands {
                        ui.separator();
                        ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                        if c.simulator_only {
                            ui.label(
                                RichText::new("Simulator-only helper · not a Linux command")
                                    .small()
                                    .color(palette(ui).accent),
                            );
                        }
                        ui.label(&c.explanation);
                        if matches!(
                            a.state,
                            State::Discovery { .. }
                                | State::AuthoritativeDns { .. }
                                | State::Ipv6 { .. }
                                | State::Ipv4Routing { .. }
                        ) {
                            continue;
                        }
                        let r = ui.add(button("Insert command"));
                        self.metrics.insert = Some(r.rect);
                        if r.clicked() {
                            self.input = c.command.clone();
                            a.workspace.as_mut().unwrap().command = self.input.clone();
                            self.focus_input = true;
                            self.scroll_command = true;
                            self.dirty = true;
                        }
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .add_enabled(a.step > 0, button("Previous section"))
                            .clicked()
                        {
                            a.step -= 1;
                            self.dirty = true;
                        }
                        if ui
                            .add_enabled(a.step + 1 < s.guide.len(), button("Next section"))
                            .clicked()
                        {
                            a.step += 1;
                            self.dirty = true;
                        }
                    });
                });
            self.metrics.walkthrough = Some(response.header_response.rect);
        }
        egui::CollapsingHeader::new("Worked reasoning (optional)")
            .id_salt(("reasoning", i))
            .show(ui, |ui| {
                ui.label(&s.explanation);
            });
        egui::CollapsingHeader::new("Supported commands, limits & sources").id_salt(("references",i)).show(ui,|ui|{
            ui.add(egui::Label::new(engine::help(&a.state)).wrap());ui.label(&s.simplifications);
            if matches!(a.state,State::Discovery{..}) {ui.label("Native controls act only on the virtual LLDP topology. Manual advertisements and the simulated clock are teaching controls; no IOS/Linux CLI, CDP or real network is modeled.");} else if matches!(a.state,State::Processes{..}) {ui.label("Only this activity’s virtual process table exists. No host processes are inspected or changed. show is a simulator-only snapshot; shell syntax and job control are not supported.");} else if matches!(a.state,State::Accounts{..}) {ui.label("Virtual administration uses only this activity’s account/group records and virtual files. Expiry is metadata only; no host accounts, credentials, authentication, PAM, login or sessions are modeled.");} else if !matches!(a.state,State::Filesystem{..}) { ui.label("All commands act only on this simulation. test-access, config set, dns set, dns serial, named-checkzone’s teaching form, and ip addr set are simulator helpers, not complete real Linux commands."); }
            for source in &s.sources{ui.add(egui::Label::new(source).wrap());}
        });
        if matches!(a.state, State::Subnet { .. }) {
            egui::CollapsingHeader::new("More IPv4 practice (optional)").show(ui, |ui| {
                let mut practice = false;
                let mut original = false;
                ui.horizontal_wrapped(|ui| {
                    practice = ui.add(button("Try another address")).clicked();
                    original = ui
                        .add_enabled(
                            a.ipv4.as_ref().is_some_and(|l| l.use_variant),
                            button("Return to original assignment"),
                        )
                        .clicked();
                    if ui.add(button("Open host-bit activity")).clicked() {
                        self.host_activity = true;
                    }
                });
                if practice {
                    model::start_practice(a);
                    self.dirty = true;
                }
                if original {
                    model::select_original(a);
                    self.dirty = true;
                }
                if practice || original {
                    self.ipv4_view = None;
                }
            });
        }
        if self.ipv4_view.is_none() && matches!(a.state, State::Subnet { .. }) {
            self.load_ipv4_view(i);
        }
    }
    pub(super) fn host_bits(&mut self, ctx: &egui::Context, i: usize) {
        let a = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap();
        let e = self.ipv4_view.as_ref().unwrap().exercise.clone();
        let l = a
            .ipv4
            .get_or_insert_with(|| ipv4::Lesson::from_legacy(e.ip, e.prefix, a.step));
        egui::Window::new("Host-bit activity · separate practice").open(&mut self.host_activity).default_width(480.).show(ctx,|ui|{
            ui.label("Toggle host bits to explore addresses. This activity does not fill or change your assignment answers.");
            ui.label(format!("Network bits stay fixed for {}/{}.",e.ip,e.prefix));
            egui::ScrollArea::vertical().max_height(300.).show(ui,|ui|{
                ui.horizontal_wrapped(|ui|{for bit in e.prefix..32 {let weight=1_u32<<(31-bit);let on=l.sandbox_host&weight!=0;if ui.add(button(&format!("{} · {}",weight,if on{1}else{0}))).clicked(){l.sandbox_host=ipv4::toggle_host(&e,l.sandbox_host,bit).unwrap();self.dirty=true;}}});
            });
            ui.label(RichText::new(format!("Address: {}",ipv4::sandbox_address(&e,l.sandbox_host).unwrap())).strong());
            ui.label("All-zero host bits identify the network; all-one host bits identify broadcast in this exercise.");
        });
    }
}
fn starting_point(id: &str) -> &'static str {
    match id {
        "ccna-vlsm" => {
            "Plan all three allocations in the table. Larger valid blocks are accepted if they fit without overlap."
        }
        "ccna-gateway" => {
            "Investigate with ip addr and ip route, repair the client, then test the remote path."
        }
        "lpic1-permissions" => {
            "Start with ls -l /srv/report.txt and id alex. Choose an ownership or permission repair that meets the goal."
        }
        "lpic1-traversal" => {
            "Start with id blair and namei /srv/project/notes.txt. Investigate the full path before changing access."
        }
        "lpic1-logs" => {
            "Start with find /var/log -name app.log. Inspect the text, then choose a filter for the failing requests."
        }
        "lpic2-service" => {
            "Investigate with systemctl status atlas and journalctl -u atlas. Saving, running now and enabling at boot are separate actions."
        }
        "lpic2-dns" => {
            "Inspect the saved record and use dig portal.lab.test to compare the loaded answer. Save, validate and activate your repair."
        }
        "lpic2-storage" => {
            "Use df -h, du -sh /data/cache and cat /etc/fstab to investigate. Repair the mount and reclaim only the disposable cache."
        }
        _ => "",
    }
}
pub(super) fn bound(text: &mut String, limit: usize) {
    if text.len() > limit {
        let mut end = limit;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
}
fn row_label(ui: &mut egui::Ui, label: &str, width: f32) {
    controls::label(ui, label, width);
}
pub(super) fn editor_row(
    ui: &mut egui::Ui,
    label: &str,
    accessible_name: &str,
    draft: &mut String,
    id: &str,
    metrics: &mut Metrics,
    focus: bool,
) -> bool {
    let label_width = (ui.available_width() * 0.4).clamp(96., 154.);
    let width = (ui.available_width() - label_width - 12.).clamp(60., 220.);
    let mut changed = false;
    ui.horizontal(|ui| {
        row_label(ui, label, label_width);
        let r = ui.add_sized(
            [width, controls::height(ui)],
            controls::singleline(ui, draft).id_salt(id).char_limit(64),
        );
        name_control(&r, accessible_name);
        metrics.inputs.push(r.rect);
        if focus {
            r.request_focus();
        }
        changed = r.changed();
        if changed {
            bound(draft, 256);
        }
    });
    changed
}
fn refresh_drafts(w: &mut model::WorkspaceData, before: &State, after: &State) {
    let replace = |draft: &mut Option<String>, old: String, new: String| {
        if draft.as_ref().is_none_or(|s| *s == old) {
            *draft = Some(new);
        }
    };
    match (before, after) {
        (State::Service { port: old, .. }, State::Service { port: new, .. }) => {
            replace(&mut w.service_port, old.clone(), new.clone())
        }
        (
            State::Dns {
                records: old,
                serial: os,
                ..
            },
            State::Dns {
                records: new,
                serial: ns,
                ..
            },
        ) => {
            replace(
                &mut w.dns_address,
                old.values().next().cloned().unwrap_or_default(),
                new.values().next().cloned().unwrap_or_default(),
            );
            replace(&mut w.dns_serial, os.to_string(), ns.to_string());
        }
        (
            State::Mount {
                configured_device: old,
                ..
            },
            State::Mount {
                configured_device: new,
                ..
            },
        ) => replace(&mut w.mount_device, old.clone(), new.clone()),
        _ => {}
    }
}

pub(super) fn allocation_fields(
    ui: &mut egui::Ui,
    state: &mut State,
    checked: bool,
    metrics: &mut Metrics,
    focus_input: &mut bool,
) -> bool {
    let mut changed = false;
    let State::Vlsm {
        base,
        prefix,
        allocations,
    } = state
    else {
        return false;
    };
    ui.label(RichText::new(format!("Address pool: {base}/{prefix}")).color(palette(ui).accent));
    let wide = ui.available_width() >= 640.;
    let label_width = (ui.available_width() * 0.4).clamp(100., 154.);
    let width = (ui.available_width() - label_width - 12.).clamp(70., 220.);
    let feedback_width = (ui.available_width() - label_width - width - 24.).max(80.);
    let mut first = true;
    let issues: Vec<_> = (0..allocations.len())
        .map(|n| {
            model::vlsm_issue(
                &State::Vlsm {
                    base: *base,
                    prefix: *prefix,
                    allocations: allocations.clone(),
                },
                n,
            )
        })
        .collect();
    egui::Grid::new("vlsm-table")
        .num_columns(if wide { 3 } else { 2 })
        .spacing([12., 8.])
        .show(ui, |ui| {
            ui.label(RichText::new("Team / required hosts").strong());
            ui.label(RichText::new("Network/prefix").strong());
            if wide {
                ui.label("Feedback");
            }
            ui.end_row();
            for (index, allocation) in allocations.iter_mut().enumerate() {
                row_label(
                    ui,
                    &format!("{} · {} hosts", allocation.name, allocation.hosts),
                    label_width,
                );
                let r = ui.add_sized(
                    [width, controls::height(ui)],
                    controls::singleline(ui, &mut allocation.cidr)
                        .id_salt(("allocation", index))
                        .hint_text("Network/prefix")
                        .char_limit(64),
                );
                name_control(
                    &r,
                    format!(
                        "{} — network/prefix for {} required hosts",
                        allocation.name, allocation.hosts
                    ),
                );
                metrics.inputs.push(r.rect);
                if first {
                    metrics.focus = Some(r.id);
                    if *focus_input {
                        r.request_focus();
                        *focus_input = false;
                    }
                    first = false;
                }
                if r.changed() {
                    bound(&mut allocation.cidr, 64);
                    changed = true;
                }
                if wide {
                    ui.allocate_ui_with_layout(
                        egui::vec2(feedback_width, controls::height(ui)),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if checked && let Some(issue) = &issues[index] {
                                ui.add(
                                    egui::Label::new(RichText::new(issue).color(palette(ui).error))
                                        .wrap(),
                                );
                            }
                        },
                    );
                }
                ui.end_row();
            }
        });
    changed
}
