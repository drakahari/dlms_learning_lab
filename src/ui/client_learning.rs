//! Native teaching for the original single-client seed; no assignment substitution.
use super::*;
use dlms_learning_lab::client_teaching::{self as client, Client, Draft};
impl LabApp {
    pub(super) fn client_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.client_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_client_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .and_then(|a| a.workspace.as_mut())
            .and_then(|w| w.client_learning.as_mut())
        {
            l.view = view;
        }
        self.notice = None;
        self.help_open = false;
        self.lesson_scroll_reset = true;
        self.focus_input = true;
        self.dirty = true;
    }
    pub(super) fn client_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let active = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if !active {
            self.focus_input = false;
        }
        ui.add_enabled_ui(active, |ui| self.client_contents(ui, i));
    }
    fn client_contents(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.client_teaching.as_ref().unwrap();
        let view = self.client_view(i);
        ui.label(RichText::new(&s.title).strong().size(23.));
        if !self.progress.attempts[&s.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_client_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                ui.label(&lesson.prerequisite_note);
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A different client to try").strong());
                ui.label(&lesson.example_description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_client_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_client_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Assignment => {
                ui.label(RichText::new("Understand the task").strong());
                ui.label(&s.objective);
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let current = Client::from_state(&a.state).unwrap();
                diagram(ui, &current);
                let draft = a.workspace.as_mut().unwrap().client_draft.as_mut().unwrap();
                let before = draft.clone();
                fields(ui, &s.id, false, draft, &mut self.metrics);
                if before != *draft {
                    self.dirty = true;
                    self.notice = None;
                }
                let r = ui.add(button("Apply client settings"));
                self.metrics.editor_save = Some(r.rect);
                if r.clicked() {
                    self.notice = Some(match client::apply_assignment(&mut a.state, draft) {
                        Ok(()) => {
                            *draft = Draft::new(&Client::from_state(&a.state).unwrap());
                            self.dirty = true;
                            "Applied to this virtual client. Inspect and test the path below."
                                .into()
                        }
                        Err(e) => e,
                    });
                }
                if client::pending_draft(a) {
                    ui.label("Pending client settings · Applied client and path checks still use the previous values.");
                }
                if let Some(notice) = &self.notice {
                    ui.add(
                        egui::Label::new(RichText::new(notice).color(palette(ui).accent)).wrap(),
                    );
                }
                self.terminal(ui, i);
                self.check_terminal_work(ui, i);
            }
            teaching::View::Example => {
                ui.label(
                    "Practice only · Separate client, settings and terminal; no assignment credit.",
                );
                ui.label(&lesson.example_description);
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let l = a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .client_learning
                    .as_mut()
                    .unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example inputs"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.notice = l.fill(lesson).err();
                        self.dirty = true;
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        *l = client::Learning::new(lesson).expect("validated client lesson");
                        l.view = teaching::View::Example;
                        self.notice = None;
                        self.dirty = true;
                    }
                });
                diagram(ui, &l.client);
                let before = l.draft.clone();
                fields(ui, &s.id, true, &mut l.draft, &mut self.metrics);
                if before != l.draft {
                    l.checked = false;
                    self.dirty = true;
                    self.notice = None;
                }
                let r = ui.add(button("Apply client settings"));
                self.metrics.editor_save = Some(r.rect);
                if r.clicked() {
                    self.notice = l.apply().err();
                    self.dirty = true;
                }
                if !l.draft.matches(&l.client) {
                    ui.label("Example settings are pending · Apply client settings before checking these edits.");
                }
                let insert_width =
                    controls::button_width(ui, "Insert example action").min(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label("Example action");
                    let width = 300_f32.min(ui.available_width());
                    let picker = controls::cell(ui, width, |ui| {
                        egui::ComboBox::from_id_salt(("client-action", &s.id))
                            .width(width)
                            .truncate()
                            .selected_text(&lesson.example_actions[l.selected_action].title)
                            .show_ui(ui, |ui| {
                                for (n, action) in lesson.example_actions.iter().enumerate() {
                                    let r = ui.selectable_value(
                                        &mut l.selected_action,
                                        n,
                                        &action.title,
                                    );
                                    self.metrics.client_actions.push((n, r.rect));
                                    if r.changed() {
                                        self.dirty = true;
                                    }
                                }
                                self.metrics.client_popup = Some(controls::popup_content_clip(ui));
                            })
                    })
                    .inner;
                    self.metrics.client_action = Some(picker.response.rect);
                    name_control(&picker.response, "Ungraded client example action");
                    let r = ui.add_sized(
                        [insert_width, controls::height(ui)],
                        button("Insert example action").wrap(),
                    );
                    self.metrics.command_insert = Some(r.rect);
                    if r.clicked() {
                        self.notice = l.insert(lesson).err();
                        self.dirty = true;
                    }
                });
                let action = &lesson.example_actions[l.selected_action];
                ui.label(&action.explanation);
                match &action.action {
                    client::Action::Command { command } => {
                        ui.add(
                            egui::Label::new(RichText::new(&command.command).monospace()).wrap(),
                        );
                        ui.label(&command.explanation);
                    }
                    client::Action::Apply { .. } => {
                        ui.label("Insert fills the three example fields only. Use Apply client settings to change this example.");
                    }
                }
                let h = (ui.ctx().content_rect().height() * 0.2).clamp(90., 180.);
                let output=egui::Frame::new().fill(palette(ui).input).inner_margin(10.).corner_radius(6.).show(ui,|ui|{
                    ui.label(RichText::new("Simulated terminal · client example only").small());
                    egui::ScrollArea::vertical().id_salt(("client-practice-output",&s.id)).max_height(h).min_scrolled_height(h).auto_shrink([false,false]).stick_to_bottom(true).show(ui,|ui|{
                        if l.transcript.is_empty(){ui.label("Inspect with ip addr or ip route, or insert an example action.");}
                        for line in &l.transcript{ui.add(egui::Label::new(RichText::new(line).monospace()).wrap().selectable(true));}
                    });
                });
                self.metrics.output = Some(output.response.rect);
                let mut run = false;
                ui.horizontal(|ui| {
                    let run_width = controls::button_width(ui, "Run");
                    let width =
                        (ui.available_width() - run_width - ui.spacing().item_spacing.x).max(40.);
                    let r = ui.add_sized(
                        [width, controls::height(ui)],
                        controls::singleline(ui, &mut l.command)
                            .id_salt(("client-practice-command", &s.id))
                            .hint_text("Command · Enter to run")
                            .char_limit(4096),
                    );
                    name_control(&r, "Ungraded client practice terminal command");
                    self.metrics.terminal = Some(r.rect);
                    self.metrics.focus = Some(r.id);
                    if self.focus_input
                        && !ui.input(|v| v.pointer.any_pressed() || v.pointer.any_released())
                    {
                        r.request_focus();
                        self.focus_input = false;
                    }
                    if r.changed() {
                        workspace::bound(&mut l.command, 4096);
                        self.dirty = true;
                    }
                    run |= r.lost_focus() && ui.input(|v| v.key_pressed(egui::Key::Enter));
                    let r = ui.add_sized([run_width, controls::height(ui)], button("Run"));
                    self.metrics.client_run = Some(r.rect);
                    run |= r.clicked();
                });
                if run && ui.is_enabled() {
                    self.notice = l.submit().err();
                    self.focus_input = true;
                    self.dirty = true;
                }
                if let Some(notice) = &self.notice {
                    ui.add(egui::Label::new(RichText::new(notice).color(palette(ui).error)).wrap());
                }
                let r = ui.add(primary(ui, "Check example"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    l.checked = true;
                    self.dirty = true;
                }
                let feedback = controls::feedback(ui, r.clicked(), |ui| {
                    if l.checked {
                        let c = l.checks(lesson);
                        if c.iter().all(|c| c.0) {
                            ui.colored_label(
                                palette(ui).success,
                                "Example checks pass · Assignment unchanged.",
                            );
                        } else {
                            for (_, message) in c.iter().filter(|c| !c.0) {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(message).color(palette(ui).error),
                                    )
                                    .wrap(),
                                );
                            }
                        }
                    }
                });
                self.metrics.feedback = Some(feedback.response.rect);
                ui.label(format!("Try a change: {}", lesson.try_change));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    self.set_client_view(i, teaching::View::Assignment);
                }
            }
        }
    }
}
fn fields(ui: &mut egui::Ui, id: &str, practice: bool, draft: &mut Draft, metrics: &mut Metrics) {
    for (key, label, value, limit) in [
        ("address", "Client IPv4 address", &mut draft.address, 64),
        ("prefix", "Prefix length", &mut draft.prefix, 16),
        ("gateway", "Default gateway", &mut draft.gateway, 64),
    ] {
        ui.horizontal(|ui| {
            let label_width = controls::form_label_width(ui);
            controls::label(ui, label, label_width);
            let width = ui.available_width().min(380.);
            let r = ui.add_sized(
                [width, controls::height(ui)],
                controls::singleline(ui, value)
                    .id_salt(("single-client", id, practice, key))
                    .char_limit(limit),
            );
            name_control(
                &r,
                format!(
                    "{} client {label}",
                    if practice {
                        "Ungraded practice"
                    } else {
                        "Assignment"
                    }
                ),
            );
            metrics.inputs.push(r.rect);
            metrics.client_fields.push((key.into(), r.id, r.rect));
            if r.changed() {
                workspace::bound(value, limit);
            }
        });
    }
}
fn diagram(ui: &mut egui::Ui, c: &Client) {
    let client = format!("{}/{}", c.address, c.prefix);
    let router = format!("{}/{}", c.router, c.lan_prefix);
    let target = c.target.to_string();
    if !controls::topology_star(
        ui,
        ["LAN router", &router, "Supplied LAN plan"],
        &[
            ["Client", &client, "Applied address/prefix"],
            ["Remote target", &target, "Simulated path destination"],
        ],
        "Client to router to remote target · abstract path only",
    ) {
        ui.label(format!("Client: {client} (applied)"));
        ui.label(format!("LAN router: {router} (supplied plan)"));
        ui.label(format!("Remote target: {target} (abstract path)"));
    }
    ui.label(format!("Applied default gateway: {}", c.gateway));
}
