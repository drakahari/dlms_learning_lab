//! Native mail-access editors and a protocol client, never a host terminal.
use super::*;
mod admin;
use dlms_learning_lab::{
    mailbox_protocol::Phase, mailbox_source as binding, mailbox_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub focus: Vec<(String, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
        self.focus.push((key.into(), r.id));
    }
}
fn apply(s: &mut work::Scope, t: &work::Task, a: &work::Action, notice: &mut Option<String>) {
    if let Err(e) = s.apply(t, a) {
        *notice = Some(e);
    }
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    let label = ui.strong("/etc/dovecot/dovecot.conf · simulated file");
    let line = ui.text_style_height(&egui::TextStyle::Monospace);
    let height = line * 10. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let mut text = s.draft.clone();
    let (outer, scroll) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("mailbox-config-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("mailbox-config")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(10)
                        .char_limit(8192),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let response = scroll.inner.labelled_by(label.id);
    let visible = response.rect.intersect(scroll.inner_rect);
    m.controls.push(("editor".into(), visible, ui.clip_rect()));
    m.fields.push(("editor".into(), visible, response.id));
    m.focus.push(("editor".into(), response.id));
    if response.changed() {
        apply(s, t, &work::Action::Edit { text }, notice);
    }
    ui.horizontal_wrapped(|ui| {
        for (key, label, action) in [
            ("save", "Save configuration", work::Action::Save),
            ("restart", "Restart service", work::Action::Restart),
            ("enable", "Enable at boot", work::Action::Enable),
        ] {
            let r = ui.add(button(label));
            m.record(ui, key, &r);
            if r.clicked() {
                apply(s, t, &action, notice);
            }
        }
    });
    let status = if s.draft != s.runtime.saved {
        "Unsaved editor changes"
    } else if s.runtime.active.as_ref() != Some(&s.runtime.saved) {
        "Saved · restart needed"
    } else {
        "Saved · running this file"
    };
    lesson_presentation::prose(
        ui,
        &format!(
            "{status} · {} · {}",
            if s.runtime.running {
                "Running"
            } else {
                "Stopped"
            },
            if s.runtime.enabled {
                "Enabled at boot"
            } else {
                "Disabled at boot"
            }
        ),
    );
    egui::Grid::new("mailbox-store-facts")
        .striped(true)
        .spacing(egui::vec2(12., 6.))
        .show(ui, |ui| {
            ui.strong("Account");
            ui.strong("Supplied Maildir");
            ui.end_row();
            for (user, a) in &t.facts.accounts {
                ui.monospace(user);
                lesson_presentation::monospace(ui, &a.maildir_path, false);
                ui.end_row();
            }
        });
    if let Some(error) = &s.runtime.error {
        lesson_presentation::prose(ui, &format!("Last restart: {}", error.message));
    }
    controls::collapsing_header(ui, "Supplied virtual certificate files").show(ui, |ui| {
        for (path, cert) in &t.facts.certificates {
            lesson_presentation::monospace(ui, path, false);
            lesson_presentation::prose(
                ui,
                &format!(
                    "Names: {} · CA: {} · valid days{}–{} · exercise day{} · key identity{}",
                    cert.dns_names.join(", "),
                    cert.issuer_id,
                    cert.valid_from_day,
                    cert.valid_through_day,
                    t.facts.day,
                    cert.key_id
                ),
            );
        }
        for (path, key) in &t.facts.keys {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "{path} · key{} · {} · {:04o}",
                    key.key_id, key.owner, key.mode
                ),
                false,
            );
        }
    });
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut Metrics,
    notice: &mut Option<String>,
    phase: (bool, &mut bool, bool),
) {
    let (practice, focus, administration) = phase;
    if administration {
        admin::console(ui, t, s, source, m, notice, (practice, focus));
        return;
    }
    if practice && !t.editable_config {
        lesson_presentation::practice_step(ui, source, 0);
    }
    ui.strong("Virtual mail client");
    controls::form_row(ui, "Next connection", |ui, width| {
        let mut selected = s.client.clone();
        let r = controls::dropdown(ui, "mailbox-client", width)
            .selected_text(&selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = m.controls.len();
                for (id, c) in &t.clients {
                    let r = ui.selectable_value(
                        &mut selected,
                        id.clone(),
                        format!(
                            "{id} · {} port {} · {}",
                            c.protocol.name().to_ascii_uppercase(),
                            c.port,
                            if c.implicit_tls {
                                "TLS on connect"
                            } else {
                                "plain, then optional TLS"
                            }
                        ),
                    );
                    m.record(ui, &format!("client:{id}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for r in &mut m.controls[start..] {
                    r.2 = clip;
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "client", &r.response);
        if selected != s.client {
            apply(s, t, &work::Action::Client { id: selected }, notice);
        }
    });
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Connect"));
        m.record(ui, "connect", &r);
        if r.clicked() {
            apply(s, t, &work::Action::Connect, notice);
            *focus = true;
        }
        let r = ui.add_enabled(s.connection.is_some(), button("Disconnect"));
        m.record(ui, "disconnect", &r);
        if r.clicked() {
            apply(s, t, &work::Action::Disconnect, notice);
        }
    });
    let state = match &s.connection {
        None => "Not connected".into(),
        Some(c) if c.link.closed => "Connection closed".into(),
        Some(c) => {
            let account = match &c.phase {
                Phase::Imap { user, selected } => format!(
                    "{}{}",
                    user.as_deref().unwrap_or("Not signed in"),
                    selected.as_ref().map_or(String::new(), |s| format!(
                        " · {} · {}",
                        s.mailbox,
                        if s.read_only {
                            "read only"
                        } else {
                            "read/write"
                        }
                    ))
                ),
                Phase::Pop { user, .. } => user
                    .as_ref()
                    .map_or("Authorization: not signed in".into(), |u| {
                        format!("{u} · transaction")
                    }),
            };
            format!(
                "{} port {} · {} · {account}",
                c.link.protocol.name().to_ascii_uppercase(),
                c.link.client.port,
                if c.link.tls {
                    "TLS verified (simulated)"
                } else {
                    "Unprotected"
                }
            )
        }
    };
    lesson_presentation::prose(ui, &state);
    // Credentials are essential supplied task inputs, never hidden behind a hint.
    egui::Grid::new("mailbox-accounts")
        .striped(true)
        .spacing(egui::vec2(12., 6.))
        .show(ui, |ui| {
            ui.strong("Account");
            ui.strong("Sample password");
            ui.end_row();
            for (user, account) in &t.facts.accounts {
                ui.monospace(user);
                ui.monospace(&account.sample_password);
                ui.end_row();
            }
        });
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
        let commands: Vec<_> = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .filter(|c| !c.simulator_only)
            .collect();
        if !commands.is_empty() {
            let key = egui::Id::new(("mailbox-example-command", &source.id));
            let mut n = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "mailbox-example-choice", width)
                    .selected_text(&commands[n].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.controls.len();
                        for (at, c) in commands.iter().enumerate() {
                            let r = ui.selectable_value(&mut n, at, &c.command);
                            m.record(ui, &format!("example-command:{at}"), &r);
                        }
                        let clip = controls::popup_content_clip(ui);
                        for r in &mut m.controls[start..] {
                            r.2 = clip;
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[n].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                apply(
                    s,
                    t,
                    &work::Action::Input {
                        text: commands[n].command.clone(),
                    },
                    notice,
                );
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, n));
        }
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace)
        * s.output.lines().count().clamp(3, 8) as f32;
    let output=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
  let r=ui.strong("Actual client/server results");m.record(ui,"output-title",&r);
  controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("mailbox-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
   if s.output.is_empty(){lesson_presentation::prose(ui,"Connect a supplied virtual client, then send a supported protocol command. C: is client input; S: is the server reply.");}
   else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}
  });
 });
    m.record(ui, "output-viewport", &output.response);
    let connected = s.connection.as_ref().is_some_and(|c| !c.link.closed);
    let mut run = false;
    let mut input = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("mailbox-input", &source.id, practice)),
            2048,
            ui.is_enabled(),
            "Protocol command · Enter to send",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(s, t, &work::Action::Input { text }, notice);
        }
        if *focus && ui.is_enabled() {
            r.response.request_focus();
            *focus = false;
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Use one protocol command within2KiB; the previous input was retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Send");
    let mut clicked = false;
    if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 8. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let field = (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(field, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| input(ui, m),
            );
            let r = ui.add_enabled(connected, button("Send"));
            m.record(ui, "send", &r);
            clicked = r.clicked();
        });
    } else {
        input(ui, m);
        let r = ui.add_enabled(connected, button("Send"));
        m.record(ui, "send", &r);
        clicked = r.clicked();
    }
    if run || clicked {
        let line = s.input.clone();
        apply(s, t, &work::Action::Command { line }, notice);
        *focus = true;
    }
}
impl LabApp {
    pub(super) fn mailbox_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (definition, simulation, administration) = match (&source.goal, &original.state) {
            (
                engine::Goal::MailboxAccess { definition },
                engine::State::MailboxAccess { simulation },
            ) => (definition, simulation, false),
            (
                engine::Goal::MailboxAdministration { definition },
                engine::State::MailboxAdministration { simulation },
            ) => (definition, simulation, true),
            _ => return,
        };
        let first = egui::Id::new(("mailbox-first", &source.id, original.visited));
        if !ui
            .ctx()
            .data(|d| d.get_temp::<bool>(first))
            .unwrap_or(false)
        {
            if let Err(e) = binding::context(original, &source) {
                lesson_presentation::prose(ui, &e);
                return;
            }
            self.focus_input = false;
            ui.ctx().data_mut(|d| d.insert_temp(first, true));
        }
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.mailbox.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            let worked = ui.scope(|ui| {
                lesson_presentation::worked(
                    ui,
                    &format!("Example endpoint: {}", definition.example.facts.server_name),
                );
                lesson_presentation::worked_rows(ui, &source);
                if let Some(text) = &definition.worked_config {
                    if administration {
                        controls::collapsing_header(ui, "Full worked configuration")
                            .show(ui, |ui| lesson_presentation::monospace(ui, text, true));
                    } else {
                        lesson_presentation::monospace(ui, text, true);
                    }
                }
            });
            self.metrics.mailbox.record(ui, "worked", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.mailbox.record(ui, "try", &r);
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            if !practice {
                lesson_presentation::assignment(ui, &source);
            }
            if practice && let Some(text) = &definition.worked_config {
                let r = ui.add_enabled(active, button("Fill worked configuration"));
                self.metrics.mailbox.record(ui, "fill", &r);
                if r.clicked() {
                    apply(
                        &mut next.practice,
                        &definition.example,
                        &work::Action::Edit { text: text.clone() },
                        &mut self.notice,
                    );
                }
            }
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.assignment, &mut next.assignment)
            };
            ui.push_id(("mailbox-workspace", &source.id, practice), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if t.editable_config {
                        if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 48.
                        {
                            ui.columns(2, |cols| {
                                editor(
                                    &mut cols[0],
                                    t,
                                    s,
                                    &source,
                                    practice,
                                    &mut self.metrics.mailbox,
                                    &mut self.notice,
                                );
                                console(
                                    &mut cols[1],
                                    t,
                                    s,
                                    &source,
                                    &mut self.metrics.mailbox,
                                    &mut self.notice,
                                    (practice, &mut self.focus_input, administration),
                                );
                            });
                        } else {
                            editor(
                                ui,
                                t,
                                s,
                                &source,
                                practice,
                                &mut self.metrics.mailbox,
                                &mut self.notice,
                            );
                            ui.add_space(12.);
                            console(
                                ui,
                                t,
                                s,
                                &source,
                                &mut self.metrics.mailbox,
                                &mut self.notice,
                                (practice, &mut self.focus_input, administration),
                            );
                        }
                    } else {
                        console(
                            ui,
                            t,
                            s,
                            &source,
                            &mut self.metrics.mailbox,
                            &mut self.notice,
                            (practice, &mut self.focus_input, administration),
                        );
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    let r = ui.add(primary(
                        ui,
                        if practice {
                            "Check example"
                        } else {
                            "Check work"
                        },
                    ));
                    self.metrics.mailbox.record(ui, "check", &r);
                    if r.clicked() {
                        apply(s, t, &work::Action::Check, &mut self.notice);
                    }
                    if s.checked {
                        let checks = s.checks(t);
                        let passed = checks.iter().filter(|c| c.passed).count();
                        let r = ui.strong(if passed == checks.len() {
                            if practice {
                                "Example outcomes passed"
                            } else {
                                "Assignment complete"
                            }
                            .to_owned()
                        } else {
                            format!("Not complete · {passed}/{} outcomes pass", checks.len())
                        });
                        self.metrics.mailbox.result = Some((passed, checks.len(), r.rect));
                        self.metrics.mailbox.record(ui, "result", &r);
                        for c in checks.iter().filter(|c| !c.passed) {
                            let r = lesson_presentation::prose(
                                ui,
                                &format!("Needs work · {}\n{}", c.label, c.detail),
                            );
                            self.metrics.mailbox.record(ui, "failed-case", &r);
                        }
                        controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
                            for c in checks.iter().filter(|c| c.passed) {
                                lesson_presentation::prose(
                                    ui,
                                    &format!("Passed · {}\n{}", c.label, c.detail),
                                );
                            }
                        });
                        if passed == checks.len() {
                            lesson_presentation::prose(ui, &source.explanation);
                        }
                    }
                });
            });
            controls::collapsing_header(ui, "Optional help and protocol reference").show(
                ui,
                |ui| {
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        if ui
                            .add_enabled(
                                active && a.hints < source.hints.len(),
                                button("Show a hint"),
                            )
                            .clicked()
                        {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals the assignment")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                            });
                    }
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match next.validate() {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state = if administration {
                        engine::State::MailboxAdministration {
                            simulation: Box::new(next),
                        }
                    } else {
                        engine::State::MailboxAccess {
                            simulation: Box::new(next),
                        }
                    };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
