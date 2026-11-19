//! Native virtual-document editing and fresh access checks; no terminal/SSH process.
use super::*;
use dlms_learning_lab::ssh_workspace::{self as work, Attempt, Document, Event, Fixture};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub popup: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response) {
    m.controls.push((key.into(), r.rect));
}
fn public_data(ui: &mut egui::Ui, f: &Fixture) {
    controls::collapsing_header(ui,"Supplied public identities · expand to copy").default_open(true).show(ui,|ui|{
  lesson_presentation::prose(ui,"Use these provided test identities. The server public key identifies the server; client public keys identify the supplied users. Private keys and real SSH connections are not used.");
  for(label,value)in std::iter::once((format!("Verified server · {}",f.scope.host),f.scope.host_key.line())).chain(f.scope.identities.iter().map(|(id,key)|(format!("Client · {}",id.replace('-'," ")),key.line()))) {
   ui.horizontal_wrapped(|ui|{ui.label(RichText::new(&label).strong());if ui.add(button("Copy public key")).clicked(){ui.ctx().copy_text(value.clone());}});
   lesson_presentation::code_value(ui,(&label,"ssh-given"),&value,ui.available_width());
  }
 });
}
fn editor(
    ui: &mut egui::Ui,
    f: &Fixture,
    a: &Attempt,
    m: &mut Metrics,
    focus: &mut bool,
) -> Vec<Event> {
    let mut events = vec![];
    let mut selected = a.drafts.selected.clone();
    controls::form_row(ui, "Virtual document", |ui, width| {
        let r = egui::ComboBox::from_id_salt("ssh-document")
            .selected_text(selected.label())
            .width(width)
            .show_ui(ui, |ui| {
                for (i, doc) in f.documents.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, doc.clone(), doc.label());
                    record(m, format!("document:{i}"), &r);
                }
                m.popup = Some(controls::popup_content_clip(ui));
            })
            .response;
        record(m, "document", &r);
    });
    if selected != a.drafts.selected {
        events.push(Event::Select {
            document: selected.clone(),
        });
    }
    let text = match &selected {
        Document::KnownHosts => Some(a.drafts.known_hosts.clone()),
        Document::ServerConfig => Some(a.drafts.config.clone()),
        Document::KeyFile { path } => a
            .drafts
            .files
            .iter()
            .find(|d| &d.path == path)
            .map(|d| d.text.clone()),
        Document::Attributes { .. } => None,
    };
    if let Some(mut text) = text {
        let before = text.clone();
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8. + 16.;
        let width = (ui.available_width()
            - 2. * f32::from(controls::PAD_X)
            - ui.spacing().scroll.allocated_width())
        .max(1.);
        let response = egui::ScrollArea::both()
            .id_salt(("ssh-editor", selected.label()))
            .min_scrolled_height(height)
            .max_height(height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt(("ssh-text", selected.label()))
                        .code_editor()
                        .lock_focus(false)
                        .desired_width(width)
                        .desired_rows(8)
                        .char_limit(16384)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y)),
                )
            });
        m.fields
            .push(("editor".into(), response.inner_rect, response.inner.id));
        response.inner.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                ui.is_enabled(),
                selected.label(),
            )
        });
        if *focus {
            response.inner.request_focus();
            *focus = false;
        }
        if text != before {
            events.push(Event::Text {
                document: selected.clone(),
                text,
            });
        }
    }
    if let Document::KeyFile { path } | Document::Attributes { path } = &selected
        && let Some(d) = a.drafts.metadata.iter().find(|d| &d.path == path)
    {
        let mut owner = d.owner.clone();
        let mut group = d.group.clone();
        let mut mode = d.mode.clone();
        ui.label(RichText::new("Virtual permissions · administrator controls").strong());
        controls::form_row(ui, "Owner", |ui, width| {
            let r = egui::ComboBox::from_id_salt("ssh-owner")
                .selected_text(&owner)
                .width(width)
                .show_ui(ui, |ui| {
                    for name in f.scope.initial_tree.users.keys() {
                        let r = ui.selectable_value(&mut owner, name.clone(), name);
                        record(m, format!("owner:{name}"), &r);
                    }
                    m.popup = Some(controls::popup_content_clip(ui));
                })
                .response;
            record(m, "owner", &r);
        });
        let groups = f
            .scope
            .initial_tree
            .users
            .values()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        controls::form_row(ui, "Group", |ui, width| {
            let r = egui::ComboBox::from_id_salt("ssh-group")
                .selected_text(&group)
                .width(width)
                .show_ui(ui, |ui| {
                    for name in groups {
                        let r = ui.selectable_value(&mut group, name.clone(), name);
                        record(m, format!("group:{name}"), &r);
                    }
                    m.popup = Some(controls::popup_content_clip(ui));
                })
                .response;
            record(m, "group", &r);
        });
        controls::form_row(ui, "Permissions (octal)", |ui, width| {
            let r = ui.add(
                controls::singleline(ui, &mut mode)
                    .desired_width((width - 2. * f32::from(controls::PAD_X)).max(1.))
                    .hint_text("000–777")
                    .char_limit(16)
                    .font(egui::TextStyle::Monospace),
            );
            m.fields.push(("mode".into(), r.rect, r.id));
        });
        if (owner.as_str(), group.as_str(), mode.as_str())
            != (d.owner.as_str(), d.group.as_str(), d.mode.as_str())
        {
            events.push(Event::Metadata {
                path: path.clone(),
                owner,
                group,
                mode,
            });
        }
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button(if matches!(selected, Document::Attributes { .. }) {
            "Save permissions"
        } else {
            "Save file"
        }));
        record(m, "save", &r);
        if r.clicked() {
            events.push(Event::Save {
                document: selected.clone(),
            });
        }
        if selected == Document::ServerConfig {
            let r = ui.add(button("Apply saved policy"));
            record(m, "apply", &r);
            if r.clicked() {
                events.push(Event::ApplyPolicy {});
            }
        }
    });
    if selected == Document::ServerConfig {
        lesson_presentation::prose(
            ui,
            if a.world.published() {
                "Saved policy matches the active supported configuration."
            } else {
                "Saved and active policy differ. Saving alone does not change access."
            },
        );
        controls::collapsing_header(ui, "Inspect active policy").show(ui, |ui| {
            match &a.world.active_config {
                Some(text) => lesson_presentation::code_value(
                    ui,
                    "ssh-active-policy",
                    text,
                    ui.available_width(),
                ),
                None => {
                    lesson_presentation::prose(
                        ui,
                        "No policy is active. Save a valid configuration, then apply it.",
                    );
                }
            };
        });
    }
    events
}
impl LabApp {
    pub(super) fn ssh_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let s = self.scenarios[i].clone();
        let original = &self.progress.attempts[&s.id];
        if let Err(e) = work::context(original, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::Goal::SshAccess { definition } = &s.goal else {
            return;
        };
        let engine::State::SshAccess { simulation } = &original.state else {
            return;
        };
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
        lesson_presentation::rich(ui, RichText::new(&s.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    record(&mut self.metrics.ssh, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &s,
                "Use the supplied test identities and virtual files. No personal SSH setup is needed.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Work through the separate supplied example before making the independent repair.",
            );
            lesson_presentation::worked_rows(ui, &s);
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active, primary(ui, "Try this example"));
                record(&mut self.metrics.ssh, "example", &r);
                self.metrics.example_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Example;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
                let r = ui.add_enabled(active, button("Go straight to assignment"));
                record(&mut self.metrics.ssh, "assignment", &r);
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if practice {
                lesson_presentation::practice_step(ui, &s, 0);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(active, button("Fill worked edits"));
                    record(&mut self.metrics.ssh, "fill", &r);
                    if r.clicked()
                        && let Err(e) = next.fill_example(definition)
                    {
                        next.practice.notice = Some(e);
                    }
                    let r = ui.add_enabled(active, button("Reset example only"));
                    record(&mut self.metrics.ssh, "reset-example", &r);
                    if r.clicked() {
                        next.reset_example(definition);
                        self.focus_input = true;
                    }
                });
            } else {
                lesson_presentation::assignment(ui, &s);
            }
            let f = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            ui.add_enabled_ui(active, |ui| public_data(ui, f));
            if practice {
                lesson_presentation::practice_step(ui, &s, 1);
            }
            let a = if practice {
                &next.practice
            } else {
                &next.assignment
            };
            let events = ui
                .add_enabled_ui(active, |ui| {
                    ui.push_id(("ssh-workspace", &s.id, practice), |ui| {
                        editor(ui, f, a, &mut self.metrics.ssh, &mut self.focus_input)
                    })
                    .inner
                })
                .inner;
            for event in events {
                let _ = next.apply(definition, &event, practice);
            }
            let a = if practice {
                &next.practice
            } else {
                &next.assignment
            };
            if let Some(message) = &a.notice {
                ui.label(RichText::new(message).color(palette(ui).accent));
            }
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check access"
                    },
                ),
            );
            record(&mut self.metrics.ssh, "check", &r);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                let _ = next.apply(definition, &Event::Check {}, practice);
            }
            let a = if practice {
                &next.practice
            } else {
                &next.assignment
            };
            if a.checked {
                let rows = a.checks(f);
                let pass = rows.iter().all(|r| r.0);
                let heading = if pass {
                    if practice {
                        "Example checks pass"
                    } else {
                        "Assignment complete"
                    }
                } else {
                    "Needs attention"
                };
                let response = ui
                    .group(|ui| {
                        ui.label(RichText::new(heading).strong().color(if pass {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        }));
                        if pass {
                            lesson_presentation::prose(ui, &s.explanation);
                        } else {
                            for (_, text) in rows.iter().filter(|r| !r.0) {
                                lesson_presentation::prose(ui, text);
                            }
                        }
                        controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
                            for (pass, text) in &rows {
                                ui.label(format!(
                                    "{} · {text}",
                                    if *pass { "Verified" } else { "Needs attention" }
                                ));
                            }
                        });
                    })
                    .response;
                self.metrics.ssh.output = Some(response.rect);
            }
            if practice {
                controls::collapsing_header(ui, "Explore after practice").show(ui, |ui| {
                    lesson_presentation::practice_step(ui, &s, 2);
                });
                let r = ui.add_enabled(active, button("Open full assignment"));
                record(&mut self.metrics.ssh, "assignment", &r);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Optional help and supported behavior").show(
                ui,
                |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&s.id).unwrap();
                        if ui
                            .add_enabled(active && a.hints < s.hints.len(), button("Show a hint"))
                            .clicked()
                        {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for h in s.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, h);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals reasoning")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &s.explanation);
                            });
                    }
                    for link in &s.sources {
                        ui.hyperlink_to(link, link);
                    }
                },
            );
        }
        if next != before && active {
            match next.bound(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::SshAccess {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&s.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&s.id], &s);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
