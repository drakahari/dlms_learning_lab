//! Native virtual FTP editor and two distinct command contexts.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    ftp_identity as core, ftp_identity_source as binding,
    ftp_identity_task::{Check, Task},
    ftp_identity_workspace as work,
};
/// One cause may prevent several transfers. Keep every case result, but teach
/// the cause once instead of filling the page with identical diagnostics.
pub(super) fn failure_groups(checks: &[Check]) -> Vec<(&str, Vec<&str>)> {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for c in checks.iter().filter(|c| !c.passed) {
        if let Some((_, labels)) = groups.iter_mut().find(|(detail, _)| *detail == c.detail) {
            labels.push(&c.label);
        } else {
            groups.push((&c.detail, vec![&c.label]));
        }
    }
    groups
}
fn apply(
    s: &mut work::Scope,
    t: &Task,
    inputs: &std::collections::BTreeMap<String, String>,
    a: work::Action,
    n: &mut Option<String>,
) {
    if let Err(e) = s.apply(t, inputs, &a) {
        *n = Some(e);
    }
}
fn output(ui: &mut egui::Ui, key: &str, text: &str, lines: usize, m: &mut Metrics) {
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * lines as f32;
    let r = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_X)
        .show(ui, |ui| {
            ui.strong("Actual command results");
            controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(key)
                    .auto_shrink([false, false])
                    .min_scrolled_height(height)
                    .max_height(height)
                    .stick_to_bottom(true),
                |ui| {
                    if text.is_empty() {
                        lesson_presentation::prose(ui, "No command has run in this terminal.");
                    } else {
                        ui.add(egui::Label::new(RichText::new(text).monospace()).extend());
                    }
                },
            );
        });
    m.record(ui, key, &r.response);
}
fn command(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    inputs: &std::collections::BTreeMap<String, String>,
    admin: bool,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let current = if admin {
        s.admin_input.clone()
    } else {
        s.client_input.clone()
    };
    let mut run = false;
    let key = if admin { "admin-input" } else { "client-input" };
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &current,
            egui::Id::new(("ftp-input", &t.id, key)),
            512,
            ui.is_enabled(),
            "Command · Enter runs",
        );
        m.record(ui, key, &r.response);
        m.fields.push((key.into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(
                s,
                t,
                inputs,
                if admin {
                    work::Action::AdminInput { text }
                } else {
                    work::Action::ClientInput { text }
                },
                n,
            );
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Keep one literal command within512bytes; previous input retained.",
            );
        }
    };
    let bw = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > bw + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - bw - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| field(ui, m),
            );
            let r = ui.add(button("Run command"));
            m.record(ui, if admin { "admin-run" } else { "client-run" }, &r);
            r.clicked()
        })
        .inner
    } else {
        field(ui, m);
        let r = ui.add(button("Run command"));
        m.record(ui, if admin { "admin-run" } else { "client-run" }, &r);
        r.clicked()
    };
    if run || clicked {
        let input = if admin {
            s.admin_input.clone()
        } else {
            s.client_input.clone()
        };
        apply(
            s,
            t,
            inputs,
            work::Action::Event {
                event: if admin {
                    work::Event::Admin { input }
                } else {
                    work::Event::Client { input }
                },
            },
            n,
        );
    }
}

fn editor(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    inputs: &std::collections::BTreeMap<String, String>,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let label = ui.strong("Server configuration · simulated files");
    let mut selected = s.editor_path.clone();
    controls::form_row(ui, "Virtual file", |ui, w| {
        let r = controls::dropdown(ui, "ftp-config-file", w)
            .selected_text(file_label(&selected))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let r = ui.selectable_value(&mut selected, String::new(), "vsftpd.conf");
                m.record(ui, "file-choice:", &r);
                for p in &t.editable_files {
                    let r = ui.selectable_value(&mut selected, p.clone(), file_label(p));
                    m.record(ui, &format!("file-choice:{p}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "file-choice", &r.response);
    });
    if selected != s.editor_path {
        apply(s, t, inputs, work::Action::ChooseFile { path: selected }, n);
    }
    lesson_presentation::monospace(
        ui,
        if s.editor_path.is_empty() {
            "/etc/vsftpd.conf"
        } else {
            &s.editor_path
        },
        true,
    );
    let mut text = s.editor.clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = ui.visuals().text_color();
    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(buffer.as_str().to_owned(), font.clone(), color))
    };
    let h =
        ui.text_style_height(&egui::TextStyle::Monospace) * 10. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        h + ui.spacing().scroll.allocated_width(),
    );
    let (outer, r) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("ftp-config-scroll")
            .auto_shrink([false, false])
            .min_scrolled_height(h)
            .max_height(size.y)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt(("ftp-config-editor", &t.id, &s.editor_path))
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .char_limit(16384)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .lock_focus(false)
                        .layouter(&mut layouter),
                )
            })
    });
    m.controls
        .push(("editor-fixed-area".into(), outer, ui.clip_rect()));
    m.controls.push((
        "editor".into(),
        r.inner.rect.intersect(r.inner_rect),
        ui.clip_rect(),
    ));
    let response = r.inner.labelled_by(label.id);
    m.focus.push(("editor".into(), response.id));
    m.controls
        .push(("editor-viewport".into(), r.inner_rect, ui.clip_rect()));
    m.fields.push(("editor".into(), response.rect, response.id));
    if text != s.editor {
        apply(s, t, inputs, work::Action::Edit { text }, n);
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save virtual file"));
        m.record(ui, "save", &r);
        if r.clicked() {
            apply(s, t, inputs, work::Action::Save, n);
        }
        ui.label(if s.editor == s.saved_editor() {
            "Saved"
        } else {
            "Unsaved changes"
        });
    });
    controls::collapsing_header(ui, "Reload saved file · discards this editor draft").show(
        ui,
        |ui| {
            let r = ui.add(button("Reload saved file"));
            m.record(ui, "reload", &r);
            if r.clicked() {
                apply(s, t, inputs, work::Action::Reload, n);
            }
        },
    );
}
fn file_label(path: &str) -> &str {
    if path.is_empty() {
        "vsftpd.conf"
    } else {
        path.rsplit('/').next().unwrap_or(path)
    }
}
fn client_label(c: &core::Credential) -> String {
    match c {
        core::Credential::Valid { user } => format!("{user} · valid"),
        core::Credential::Invalid { user } => format!("{user} · invalid"),
        core::Credential::Anonymous => "Anonymous".into(),
    }
}
fn client(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    inputs: &std::collections::BTreeMap<String, String>,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    ui.strong("FTP client · simulated connection");
    let choices = work::clients(t);
    let mut at = s.selected_client;
    controls::form_row(ui, "Connect as", |ui, w| {
        let r = controls::dropdown(ui, "ftp-client-identity", w)
            .selected_text(client_label(&choices[at]))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in choices.iter().enumerate() {
                    let r = ui.selectable_value(&mut at, i, client_label(c));
                    m.record(ui, &format!("client-choice:{i}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "client-choice", &r.response);
    });
    if at != s.selected_client {
        apply(s, t, inputs, work::Action::ChooseClient { client: at }, n);
    }
    let r = ui.add(button("Connect"));
    m.record(ui, "connect", &r);
    if r.clicked() {
        apply(
            s,
            t,
            inputs,
            work::Action::Event {
                event: work::Event::Connect {
                    client: s.selected_client,
                },
            },
            n,
        );
    }
    output(ui, "client-output", &s.client_output, 5, m);
    command(ui, t, s, inputs, false, m, n);
    controls::collapsing_header(ui, "Client files · simulated inputs and downloads").show(
        ui,
        |ui| {
            for (name, text) in &s.local_files {
                ui.monospace(format!("{name} · {} bytes", text.len()));
                lesson_presentation::code_value(
                    ui,
                    ("ftp-local", name),
                    text,
                    ui.available_width(),
                );
            }
        },
    );
}
impl LabApp {
    pub(super) fn ftp_identity_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase=(self.selected,self.reset_pending);
        let challenge=self.workspace_header(ui,i);
        if phase!=(self.selected,self.reset_pending){return;}
        let source = self.scenarios[i].clone();
        let d = match binding::source(&source) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        let original = &self.progress.attempts[&source.id];
        if let Err(e) = binding::context(original, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::FtpIdentity { simulation } = &original.state else {
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.ftp_identity.record(ui, key, &r);
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
            let r = lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .ftp_identity
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(
                ui,
                "Expected configuration and actions · separate practice",
            );
            let worked = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics
                .ftp_identity
                .record(ui, "worked", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.ftp_identity.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s, inputs) = if practice {
                (&d.example, &mut next.example, &d.example_inputs)
            } else {
                (&d.assignment, &mut next.assignment, &d.assignment_inputs)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics
                    .ftp_identity
                    .record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active, |ui| {
                ui.label(if s.world.published.is_some() {
                    "FTP service: Running"
                } else {
                    "FTP service: Stopped"
                });
                let server = |ui: &mut egui::Ui,
                              s: &mut work::Scope,
                              m: &mut Metrics,
                              n: &mut Option<String>| {
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 0);
                    }
                    editor(ui, t, s, inputs, m, n);
                    ui.add_space(12.);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 1);
                    }
                    ui.strong("Server terminal · simulated administration");
                    output(ui, "admin-output", &s.admin_output, 4, m);
                    command(ui, t, s, inputs, true, m, n);
                };
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54. {
                    ui.columns(2, |cols| {
                        server(
                            &mut cols[0],
                            s,
                            &mut self.metrics.ftp_identity,
                            &mut self.notice,
                        );
                        if practice {
                            lesson_presentation::practice_step(&mut cols[1], &source, 2);
                        }
                        client(
                            &mut cols[1],
                            t,
                            s,
                            inputs,
                            &mut self.metrics.ftp_identity,
                            &mut self.notice,
                        );
                    });
                } else {
                    server(ui, s, &mut self.metrics.ftp_identity, &mut self.notice);
                    ui.add_space(12.);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    client(
                        ui,
                        t,
                        s,
                        inputs,
                        &mut self.metrics.ftp_identity,
                        &mut self.notice,
                    );
                }
                ui.separator();
                let r = ui.add(primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check configuration"
                    },
                ));
                self.metrics.ftp_identity.record(ui, "check", &r);
                if r.clicked() {
                    apply(s, t, inputs, work::Action::Check, &mut self.notice);
                }
                if s.checked {
                    let checks = s.checks(t, inputs);
                    let pass = checks.iter().filter(|c| c.passed).count();
                    let r = ui.strong(if pass == checks.len() {
                        if practice {
                            "Example checks passed".into()
                        } else {
                            "Assignment complete".into()
                        }
                    } else {
                        format!("Not complete · {pass}/{} outcomes pass", checks.len())
                    });
                    self.metrics.ftp_identity.result = Some((pass, checks.len(), r.rect));
                    self.metrics.ftp_identity.record(ui, "result", &r);
                    for (index, (detail, labels)) in failure_groups(&checks).iter().enumerate() {
                        ui.strong(if labels.len() == 1 {
                            format!("Needs work · {}", labels[0])
                        } else {
                            format!("Needs work · {} outcomes share this cause", labels.len())
                        });
                        let r = lesson_presentation::prose(ui, detail);
                        self.metrics.ftp_identity.record(ui, "failed-case", &r);
                        if labels.len() > 1 {
                            let group = controls::collapsing_header(
                                ui,
                                &format!("Affected outcomes ({})", labels.len()),
                            )
                                .id_salt(("ftp-failure-group", index))
                                .show(ui, |ui| {
                                    for label in labels {
                                        lesson_presentation::prose(
                                            ui,
                                            &format!("Not passed · {label}"),
                                        );
                                    }
                                });
                            self.metrics.ftp_identity.record(
                                ui,
                                &format!("failed-outcomes-{index}"),
                                &group.header_response,
                            );
                        }
                    }
                    if pass > 0 {
                        controls::collapsing_header(ui, &format!("Inspect passing checks ({pass})"))
                            .show(ui, |ui| {
                                for c in checks.iter().filter(|c| c.passed) {
                                    lesson_presentation::prose(
                                        ui,
                                        &format!("Passed · {}\n{}", c.label, c.detail),
                                    );
                                }
                            });
                    }
                    if pass == checks.len() {
                        lesson_presentation::prose(
                            ui,
                            if practice {
                                "Apply the same reasoning independently to the assignment."
                            } else {
                                &source.explanation
                            },
                        );
                    }
                }
                if practice {
                    controls::collapsing_header(ui, "Show worked answer · example only").show(
                        ui,
                        |ui| {
                            for a in &d.worked {
                                match a {
                                    work::Action::Edit { text } => {
                                        lesson_presentation::code_value(
                                            ui,
                                            "ftp-worked-config",
                                            text,
                                            ui.available_width(),
                                        );
                                        if ui.add_enabled(s.editor_path.is_empty(),button("Fill worked example")).clicked() {
                                            apply(
                                                s,
                                                t,
                                                inputs,
                                                work::Action::Edit { text: text.clone() },
                                                &mut self.notice,
                                            );
                                        }
                                    }
                                    work::Action::Event {
                                        event: work::Event::Admin { input },
                                    } => {
                                        ui.monospace(input);
                                    }
                                    work::Action::Save => {lesson_presentation::prose(ui,"Save the example configuration, then apply it in Server terminal. Filling text does not run it.");}
                                    work::Action::Event{event:work::Event::SaveFile{path,text}}=> {lesson_presentation::monospace(ui,path,true);lesson_presentation::code_value(ui,("ftp-worked-file",path),text,ui.available_width());}
                                    work::Action::Event{event:work::Event::Client{input}}=> {ui.monospace(format!("FTP client: {input}"));}
                                    _ => {}
                                }
                            }
                        },
                    );
                }
            });
            controls::collapsing_header(ui, "Help, supported subset and references").show(
                ui,
                |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&source.id).unwrap();
                            a.hints = (a.hints + 1).min(source.hints.len());
                            self.dirty = true;
                        }
                        for hint in source
                            .hints
                            .iter()
                            .take(self.progress.attempts[&source.id].hints)
                        {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match d.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::FtpIdentity {
                            simulation: Box::new(next),
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
        Self::workspace_footer(ui);
    }
}
