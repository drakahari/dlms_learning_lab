use super::*;
use engine::State;
impl LabApp {
    pub(super) fn workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        ui.horizontal_wrapped(|ui| {
            if ui.add(button("Back to library")).clicked() {
                self.selected = None;
            }
            ui.label(
                RichText::new(format!("{} / {}", s.track, s.topic))
                    .small()
                    .color(ACCENT),
            );
        });
        ui.heading(RichText::new(&s.title).size(27.));
        ui.label(&s.objective);
        ui.label(RichText::new("Simulation only. Leaving keeps your state, walkthrough position and hints saved locally.").small().color(MUTED));
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(!a.challenge, "Learn").clicked() {
                a.challenge = false;
                self.dirty = true;
            }
            if ui.selectable_label(a.challenge, "Challenge").clicked() {
                a.challenge = true;
                self.dirty = true;
            }
            if ui.add(button("Reset attempt")).clicked() {
                self.reset_pending = true;
            }
        });
        if self.reset_pending {
            card().show(ui,|ui|{ui.label("Reset only this lab’s state, walkthrough, hints and transcript? Favorites and other attempts stay saved.");ui.horizontal_wrapped(|ui|{if ui.add(button("Confirm reset")).clicked(){self.reset_attempt(i);}
 if ui.add(button("Cancel")).clicked(){self.reset_pending=false;}});});
        }
        let width = ui.available_width();
        let split = width >= 960.;
        self.metrics.split = split;
        if split {
            let left = (width * 0.36).clamp(300., 420.);
            let right = width - left - 16.;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 16.;
                ui.allocate_ui_with_layout(
                    egui::vec2(left, 0.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        self.instruction(ui, i);
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(right, 0.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(right);
                        self.practice(ui, i);
                    },
                );
            });
        } else {
            self.instruction(ui, i);
            ui.add_space(6.);
            self.practice(ui, i);
        }
        ui.add_space(12.);
        egui::CollapsingHeader::new("Command reference, simulation limits & sources").show(ui,|ui|{
            ui.label(engine::help(&self.progress.attempts[&s.id].state));ui.label(&s.simplifications);
            ui.label("Simulator-only helpers: test-access, config set, dns set, dns serial, and ip addr set. These are teaching controls, not Linux commands. No host shell, files, processes or network operations are executed.");
            ui.label("ls sizes/dates and numeric IDs are deterministic teaching metadata. Journal timestamps are synthetic; journal entries record modeled service events.");
            for source in &s.sources{ui.add(egui::Label::new(source).wrap());}
        });
    }
    fn instruction(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let complete = engine::solved(&a.state, &s.goal);
        if a.challenge {
            card().show(ui,|ui|{ui.label(RichText::new("CHALLENGE").color(ACCENT).strong());ui.label("Solve the objective using the workspace. Walkthrough commands are hidden; hints are optional.");});
        } else {
            if complete {
                ui.label(
                    RichText::new("Objective complete. The walkthrough is optional.").color(GREEN),
                );
            }
            let show = |ui: &mut egui::Ui,
                        a: &mut Attempt,
                        input: &mut String,
                        focus: &mut bool,
                        dirty: &mut bool,
                        metrics: &mut Metrics| {
                metrics.guide = true;
                card().fill(Color32::from_rgb(34,29,58)).show(ui,|ui|{
                    ui.label(RichText::new(format!("WALKTHROUGH · section {} of {}",a.step+1,s.steps.len())).small().color(ACCENT));
                    if let Some(g)=s.guide.get(a.step){
                        ui.label(RichText::new(&g.title).strong().size(21.));ui.label(&g.why);ui.label(RichText::new(&g.action).color(MUTED));
                        for c in &g.commands{
                            ui.separator();ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                            if c.simulator_only{ui.label(RichText::new("SIMULATOR-ONLY HELPER").small().color(ACCENT));}
                            ui.label(RichText::new(&c.explanation).small());
                            let r=ui.add(button("Insert command"));metrics.insert=Some(r.rect);
                            if r.clicked(){*input=c.command.clone();*focus=true;}
                        }
                    }else{ui.label(&s.steps[a.step]);}
                    ui.separator();ui.horizontal_wrapped(|ui|{if ui.add_enabled(a.step>0,button("Previous")).clicked(){a.step-=1;*dirty=true;}
 if ui.add_enabled(a.step+1<s.steps.len(),button("Next section")).clicked(){a.step+=1;*dirty=true;}});
                    ui.label(RichText::new("Sections guide learning; objective completion depends only on your work.").small().color(MUTED));
                });
            };
            if complete {
                egui::CollapsingHeader::new("Review walkthrough").show(ui, |ui| {
                    show(
                        ui,
                        a,
                        &mut self.input,
                        &mut self.focus_input,
                        &mut self.dirty,
                        &mut self.metrics,
                    )
                });
            } else {
                show(
                    ui,
                    a,
                    &mut self.input,
                    &mut self.focus_input,
                    &mut self.dirty,
                    &mut self.metrics,
                );
            }
        }
        if ui
            .add_enabled(
                a.hints < s.hints.len(),
                button(&format!("Reveal hint ({}/{})", a.hints, s.hints.len())),
            )
            .clicked()
        {
            a.hints += 1;
            self.dirty = true;
        }
        for hint in s.hints.iter().take(a.hints) {
            card().show(ui, |ui| {
                ui.label(RichText::new("Hint").color(ACCENT));
                ui.label(hint);
            });
        }
    }
    fn practice(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let native = matches!(
            self.progress.attempts[&s.id].state,
            State::Subnet { .. } | State::Vlsm { .. }
        );
        card().show(ui,|ui|{
            ui.label(RichText::new(if native{"YOUR WORK"}else{"SIMULATED WORKSPACE"}).small().color(ACCENT).strong());
            if native{self.answer_workspace(ui,i);}else{
                if matches!(self.progress.attempts[&s.id].state,State::Service{..}){self.service_editor(ui,i);}else{ui.add(egui::Label::new(RichText::new(engine::describe(&self.progress.attempts[&s.id].state)).monospace()).wrap());}
                self.terminal(ui,i);
            }
            let r=ui.add(primary("Check work"));self.metrics.check=Some(r.rect);if r.clicked(){self.checked=true;}
            let now=engine::solved(&self.progress.attempts[&s.id].state,&s.goal);
            if self.last_solved&&!now{self.invalidated=true;self.checked=true;}
            if now{self.invalidated=false;}self.last_solved=now;
            if self.invalidated{ui.label(RichText::new("Current success invalidated: the state changed. Repair the remaining objectives.").color(RED));}
            if !now&&self.checked{
                let checks=engine::checks(&self.progress.attempts[&s.id].state,&s.goal);ui.label(RichText::new(format!("{} of {} objectives met",checks.iter().filter(|x|x.0).count(),checks.len())).strong());
                for(pass,text)in checks{ui.colored_label(if pass{GREEN}else{RED},format!("{}  {text}",if pass{"PASS"}else{"REVIEW"}));}
            }
        });
        if engine::solved(&self.progress.attempts[&s.id].state, &s.goal) {
            self.completion(ui, i);
        }
    }
    fn answer_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let checks = engine::checks(&a.state, &s.goal);
        match &mut a.state {
            State::Subnet {
                ip,
                prefix,
                answers,
            } => {
                ui.label(
                    RichText::new(format!("Address card: {ip}/{prefix}"))
                        .strong()
                        .size(20.),
                );
                ui.label(
                    RichText::new(format!(
                        "32 bits · {prefix} network bits · {} host bits",
                        32 - *prefix
                    ))
                    .small()
                    .color(MUTED),
                );
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 45.),
                    egui::Sense::hover(),
                );
                let gap = 2.;
                let w = (rect.width() - 31. * gap) / 32.;
                for bit in 0..32 {
                    let x = rect.left() + bit as f32 * (w + gap);
                    let cell =
                        egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, 24.));
                    ui.painter().rect_filled(
                        cell,
                        2.,
                        if bit < *prefix as usize {
                            Color32::from_rgb(57, 106, 142)
                        } else {
                            Color32::from_rgb(120, 83, 174)
                        },
                    );
                    if w >= 9. {
                        ui.painter().text(
                            cell.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", (u32::from(*ip) >> (31 - bit)) & 1),
                            egui::FontId::monospace(11.),
                            Color32::WHITE,
                        );
                    }
                }
                ui.painter().text(
                    egui::pos2(rect.left(), rect.top() + 32.),
                    egui::Align2::LEFT_CENTER,
                    "Network portion",
                    egui::FontId::proportional(12.),
                    MUTED,
                );
                ui.painter().text(
                    egui::pos2(rect.right(), rect.top() + 32.),
                    egui::Align2::RIGHT_CENTER,
                    "Host portion",
                    egui::FontId::proportional(12.),
                    ACCENT,
                );
                egui::Grid::new("subnet-answers")
                    .num_columns(2)
                    .spacing([12., 10.])
                    .show(ui, |ui| {
                        for (index, label) in [
                            "Network address",
                            "Broadcast address",
                            "First usable host",
                            "Last usable host",
                        ]
                        .iter()
                        .enumerate()
                        {
                            ui.label(*label);
                            let width = (ui.available_width() - 8.).clamp(120., 230.);
                            let r = ui.add_sized(
                                [width, CONTROL],
                                egui::TextEdit::singleline(&mut answers[index])
                                    .id_salt(("answer", index))
                                    .hint_text("IPv4 address")
                                    .char_limit(64),
                            );
                            self.metrics.inputs.push(r.rect);
                            if index == 0 {
                                self.metrics.focus = Some(r.id);
                                if self.focus_input {
                                    r.request_focus();
                                    self.focus_input = false;
                                }
                            }
                            if r.changed() {
                                self.dirty = true;
                            }
                            ui.end_row();
                            if !answers[index].is_empty()
                                && answers[index].parse::<std::net::Ipv4Addr>().is_err()
                            {
                                ui.label("");
                                ui.colored_label(RED, "Enter a valid IPv4 address");
                                ui.end_row();
                            }
                        }
                    });
                ui.label(
                    RichText::new(
                        "Values save as you type. Check work validates the current answers.",
                    )
                    .small()
                    .color(MUTED),
                );
            }
            State::Vlsm {
                base,
                prefix,
                allocations,
            } => {
                ui.label(
                    RichText::new(format!("Address pool: {base}/{prefix}"))
                        .strong()
                        .size(20.),
                );
                ui.label(
                    RichText::new("Assign an aligned network/prefix to every team.")
                        .small()
                        .color(MUTED),
                );
                let field_width = (ui.available_width() - 230.).clamp(110., 220.);
                egui::Grid::new("allocation-table")
                    .num_columns(4)
                    .striped(true)
                    .spacing([10., 12.])
                    .show(ui, |ui| {
                        for heading in ["Team", "Hosts", "Network/prefix", "Result"] {
                            ui.label(RichText::new(heading).strong().size(14.));
                        }
                        ui.end_row();
                        for (index, allocation) in allocations.iter_mut().enumerate() {
                            ui.label(&allocation.name);
                            ui.label(allocation.hosts.to_string());
                            let r = ui.add_sized(
                                [field_width, CONTROL],
                                egui::TextEdit::singleline(&mut allocation.cidr)
                                    .id_salt(("allocation", index))
                                    .hint_text("e.g. 10.20.0.0/26")
                                    .char_limit(64),
                            );
                            self.metrics.inputs.push(r.rect);
                            if index == 0 {
                                self.metrics.focus = Some(r.id);
                                if self.focus_input {
                                    r.request_focus();
                                    self.focus_input = false;
                                }
                            }
                            if r.changed() {
                                self.dirty = true;
                            }
                            ui.label(
                                RichText::new(if self.checked {
                                    if checks[index].0 { "Valid" } else { "Review" }
                                } else if allocation.cidr.is_empty() {
                                    "Empty"
                                } else {
                                    "Draft"
                                })
                                .small(),
                            );
                            ui.end_row();
                        }
                    });
            }
            _ => {}
        }
    }
    fn service_editor(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        ui.label(RichText::new(engine::describe(&a.state)).strong());
        egui::CollapsingHeader::new("Simulated /etc/atlas.conf editor")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(
                        "Edits only the modeled file. Save, then restart the service to apply.",
                    )
                    .small()
                    .color(MUTED),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("port =").monospace());
                    let draft = self.service_draft.get_or_insert_with(String::new);
                    let edit = ui.add_sized(
                        [120., CONTROL],
                        egui::TextEdit::singleline(draft)
                            .id_salt("service-port")
                            .char_limit(256),
                    );
                    if edit.changed() {
                        self.notice = None;
                    }
                    let r = ui.add(button("Save configuration"));
                    self.metrics.editor_save = Some(r.rect);
                    if r.clicked() {
                        match engine::edit_service_port(&mut a.state, draft) {
                            Ok(()) => {
                                self.dirty = true;
                                self.notice = Some(
                                    "Configuration saved. Restart applies the saved port.".into(),
                                );
                            }
                            Err(e) => self.notice = Some(e),
                        }
                    }
                });
                if let (State::Service { port, .. }, Some(draft)) = (&a.state, &self.service_draft)
                    && port != draft
                {
                    ui.label(
                        RichText::new("Unsaved editor changes. Save configuration before leaving.")
                            .small()
                            .color(ACCENT),
                    );
                }
                if let Some(message) = &self.notice {
                    ui.label(RichText::new(message).small().color(ACCENT));
                }
            });
    }
    fn terminal(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        ui.label(
            RichText::new("Terminal · simulated administrator")
                .small()
                .color(MUTED),
        );
        let height = if self.metrics.split { 250. } else { 210. };
        let output = egui::Frame::new()
            .fill(Color32::from_rgb(7, 13, 24))
            .inner_margin(12.)
            .corner_radius(6.)
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .id_salt(("terminal", i))
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &a.transcript {
                            ui.add(
                                egui::Label::new(RichText::new(line).monospace().size(15.))
                                    .selectable(true)
                                    .extend(),
                            );
                        }
                    });
            });
        self.metrics.output = Some(output.response.rect);
        let mut run = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new("lab #").monospace().color(ACCENT));
            let width = (ui.available_width() - 80.).max(80.);
            let r = ui.add_sized(
                [width, CONTROL],
                egui::TextEdit::singleline(&mut self.input)
                    .id_salt("command")
                    .hint_text("Command (help lists the supported subset)")
                    .char_limit(4096),
            );
            self.metrics.terminal = Some(r.rect);
            self.metrics.focus = Some(r.id);
            if self.focus_input {
                r.request_focus();
                self.focus_input = false;
            }
            run = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            run |= ui.add(button("Run")).clicked();
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
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let output = engine::execute(&mut a.state, &cmd).unwrap_or_else(|e| e);
        a.transcript.push(if output.is_empty() {
            format!("lab # {cmd}")
        } else {
            format!("lab # {cmd}\n{output}")
        });
        if a.transcript.len() > 100 {
            a.transcript.drain(..a.transcript.len() - 100);
        }
        self.dirty = true;
        self.focus_input = true;
    }
    fn completion(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        self.metrics.completion = true;
        let mut back = false;
        let mut next = false;
        card().stroke(egui::Stroke::new(1_f32,GREEN)).show(ui,|ui|{
            ui.label(RichText::new("Objective complete").strong().size(23.).color(GREEN));
            ui.label("All current objective checks pass. No further walkthrough sections are required.");
            for(_,result)in engine::checks(&a.state,&s.goal){ui.label(format!("PASS  {result}"));}
            ui.separator();ui.label(RichText::new("Why this works").strong());ui.label(&s.explanation);
            ui.horizontal_wrapped(|ui|{back=ui.add(button("Back to library")).clicked();if i+1<self.scenarios.len(){next=ui.add(primary("Next lab")).clicked();}});
            if i+1<self.scenarios.len(){ui.label(RichText::new(format!("Next: {}",self.scenarios[i+1].title)).small().color(MUTED));}
            ui.label(RichText::new("Keep experimenting if you like. Breaking the state removes current completion.").small().color(MUTED));
        });
        if back {
            self.selected = None;
        }
        if next {
            self.launch(i + 1);
        }
    }
}
